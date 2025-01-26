use crate::Hammer2FsckOptions;
use hammer2_utils::tab;

use std::io::Write;

const TAB_INDENT: usize = 8;

#[derive(Debug)]
struct BlockrefMessage {
    bref: libhammer2::fs::Hammer2Blockref,
    msg: [u8; 1024],
}

impl BlockrefMessage {
    fn new(bref: &libhammer2::fs::Hammer2Blockref) -> Self {
        Self {
            bref: *bref,
            msg: [0; 1024],
        }
    }

    fn new_from_str(bref: &libhammer2::fs::Hammer2Blockref, s: &str) -> Self {
        let mut m = BlockrefMessage::new(bref);
        m.msg[..s.len()].copy_from_slice(s.as_bytes());
        m
    }

    fn new_from<T>(bref: &libhammer2::fs::Hammer2Blockref, x: &T) -> Self
    where
        T: Clone + Copy,
    {
        let mut m = BlockrefMessage::new(bref);
        *libhammer2::util::align_head_to_mut(&mut m.msg) = *x;
        m
    }

    fn msg_as<T>(&self) -> &T {
        libhammer2::util::align_head_to(&self.msg)
    }
}

type BlockrefEntry = Vec<BlockrefMessage>;
type BlockrefMap = std::collections::BTreeMap<u64, BlockrefEntry>;

#[derive(Debug, Default)]
struct BlockrefStats {
    root: BlockrefMap,
    typ: u8, // HAMMER2_BREF_TYPE_VOLUME or FREEMAP
    total_blockref: u64,
    total_empty: u64,
    total_bytes: u64,
    volume: VolumeDeltaStats,
    freemap: FreemapDeltaStats,
}

impl BlockrefStats {
    fn new(typ: u8) -> Self {
        Self {
            typ,
            ..Default::default()
        }
    }

    fn load(&mut self, x: &DeltaStats) {
        self.total_blockref += x.total_blockref;
        self.total_empty += x.total_empty;
        self.total_bytes += x.total_bytes;

        match self.typ {
            libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP => {
                self.freemap.total_freemap_node += x.freemap.total_freemap_node;
                self.freemap.total_freemap_leaf += x.freemap.total_freemap_leaf;
            }
            libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => {
                self.volume.total_inode += x.volume.total_inode;
                self.volume.total_indirect += x.volume.total_indirect;
                self.volume.total_data += x.volume.total_data;
                self.volume.total_dirent += x.volume.total_dirent;
            }
            _ => panic!("{}", self.typ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DeltaStats {
    total_blockref: u64,
    total_empty: u64,
    total_bytes: u64,
    volume: VolumeDeltaStats,
    freemap: FreemapDeltaStats,
    count: usize,
}

impl DeltaStats {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn add(&mut self, x: &Self) {
        self.total_blockref += x.total_blockref;
        self.total_empty += x.total_empty;
        self.total_bytes += x.total_bytes;

        self.freemap.total_freemap_node += x.freemap.total_freemap_node;
        self.freemap.total_freemap_leaf += x.freemap.total_freemap_leaf;

        self.volume.total_inode += x.volume.total_inode;
        self.volume.total_indirect += x.volume.total_indirect;
        self.volume.total_data += x.volume.total_data;
        self.volume.total_dirent += x.volume.total_dirent;

        self.count += x.count;
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct VolumeDeltaStats {
    total_inode: u64,
    total_indirect: u64,
    total_data: u64,
    total_dirent: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct FreemapDeltaStats {
    total_freemap_node: u64,
    total_freemap_leaf: u64,
}

fn print_zone_summary(
    tab: usize,
    i: usize,
    zone: usize,
    bref: &libhammer2::fs::Hammer2Blockref,
    opt: &Hammer2FsckOptions,
) {
    let s = if !opt.scan_best && i == zone {
        " (best)"
    } else {
        ""
    };
    tab::println!(tab, "zone.{i} {:016x}{}", bref.data_off, s);
}

fn alloc_root_blockref(i: usize, typ: u8) -> libhammer2::fs::Hammer2Blockref {
    assert!(
        typ == libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY
            || typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP
            || typ == libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME
    );
    let mut bref = libhammer2::fs::Hammer2Blockref::new(typ);
    bref.data_off = libhammer2::volume::get_volume_data_offset(i)
        | u64::try_from(libhammer2::fs::HAMMER2_PBUFRADIX).unwrap();
    bref
}

fn test_volume_header(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    zone: usize,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<()> {
    let mut failure = None;
    for i in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
        if opt.scan_best && i != zone {
            continue;
        }
        let vol = fso
            .get_root_volume_mut()
            .ok_or_else(libhammer2::util::notfound)?;
        let offset = libhammer2::volume::get_volume_data_offset(i);
        if offset < vol.get_size() {
            let broot = alloc_root_blockref(i, libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY);
            print_zone_summary(0, i, zone, &broot, opt);
            let buf = vol.preadx(libhammer2::fs::HAMMER2_VOLUME_BYTES, offset)?;
            if let Err(e) = verify_volume_header(libhammer2::util::align_to(&buf)) {
                if failure.is_none() {
                    failure = Some(e);
                }
            }
        } else {
            tab::println!(0, "zone.{i} exceeds volume size");
            break;
        }
    }
    if let Some(e) = failure {
        Err(e)
    } else {
        Ok(())
    }
}

fn test_blockref(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    typ: u8,
    zone: usize,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<()> {
    let mut failure = None;
    let mut droot = BlockrefMap::new();
    for i in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
        if opt.scan_best && i != zone {
            continue;
        }
        let vol = fso
            .get_root_volume_mut()
            .ok_or_else(libhammer2::util::notfound)?;
        let offset = libhammer2::volume::get_volume_data_offset(i);
        if offset < vol.get_size() {
            let broot = alloc_root_blockref(i, typ);
            print_zone_summary(0, i, zone, &broot, opt);
            let mut bstats = BlockrefStats::new(typ);
            if let Err(e) = verify_blockref(fso, &broot, false, &mut bstats, &mut droot, opt) {
                if failure.is_none() {
                    failure = Some(e);
                }
            }
            print_blockref_stats(&bstats, true, opt)?;
            print_blockref_entry(fso, &bstats.root, opt)?;
        } else {
            tab::println!(0, "zone.{i} exceeds volume size");
            break;
        }
    }
    if let Some(e) = failure {
        Err(e)
    } else {
        Ok(())
    }
}

fn test_pfs_blockref(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    zone: usize,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<()> {
    let mut failure = None;
    let mut droot = BlockrefMap::new();
    for i in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
        if opt.scan_best && i != zone {
            continue;
        }
        let vol = fso
            .get_root_volume_mut()
            .ok_or_else(libhammer2::util::notfound)?;
        let offset = libhammer2::volume::get_volume_data_offset(i);
        if offset < vol.get_size() {
            let typ = libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME;
            let broot = alloc_root_blockref(i, typ);
            print_zone_summary(0, i, zone, &broot, opt);
            let blist = match scan_pfs_blockref(fso, &broot) {
                Ok(v) => v,
                Err(e) => {
                    tab::eprintln!(1, "Failed to read PFS blockref: {e}");
                    if failure.is_none() {
                        failure = Some(e);
                    }
                    continue;
                }
            };
            if blist.is_empty() {
                tab::eprintln!(1, "Failed to find PFS blockref");
                if failure.is_none() {
                    failure = Some(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                }
                continue;
            }
            let mut count = 0;
            for m in &blist {
                let mut found = false;
                let ipdata = m.msg_as::<libhammer2::fs::Hammer2InodeData>();
                let f = ipdata.get_filename_string();
                if !opt.pfs_names.is_empty() {
                    for s in &opt.pfs_names {
                        if *s == f {
                            found = true;
                        }
                    }
                } else {
                    found = true;
                }
                if !found {
                    continue;
                }
                count += 1;
                if opt.print_pfs {
                    print_pfs(ipdata);
                    continue;
                }
                tab::println!(1, "{f}");
                let mut bstats = BlockrefStats::new(typ);
                if let Err(e) = verify_blockref(fso, &m.bref, false, &mut bstats, &mut droot, opt) {
                    if failure.is_none() {
                        failure = Some(e);
                    }
                }
                print_blockref_stats(&bstats, true, opt)?;
                print_blockref_entry(fso, &bstats.root, opt)?;
            }
            if !opt.pfs_names.is_empty() && count == 0 {
                tab::println!(1, "PFS not found");
                if failure.is_none() {
                    failure = Some(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                }
            }
        } else {
            tab::println!(0, "zone.{i} exceeds volume size");
            break;
        }
    }
    if let Some(e) = failure {
        Err(e)
    } else {
        Ok(())
    }
}

fn add_blockref_entry_from_str(
    root: &mut BlockrefMap,
    bref: &libhammer2::fs::Hammer2Blockref,
    s: &str,
) {
    add_blockref_entry_impl(root, bref, BlockrefMessage::new_from_str(bref, s));
}

fn add_blockref_entry<T>(root: &mut BlockrefMap, bref: &libhammer2::fs::Hammer2Blockref, x: &T)
where
    T: Clone + Copy,
{
    add_blockref_entry_impl(root, bref, BlockrefMessage::new_from(bref, x));
}

fn add_blockref_entry_impl(
    root: &mut BlockrefMap,
    bref: &libhammer2::fs::Hammer2Blockref,
    m: BlockrefMessage,
) {
    if let Some(v) = root.get_mut(&bref.data_off) {
        v.push(m);
    } else {
        root.insert(bref.data_off, vec![m]);
    }
}

fn format_blockref(tab: usize, bref: &libhammer2::fs::Hammer2Blockref, msg: &str) -> String {
    tab::format!(
        tab,
        "{:016x} {:<12} {:016x}/{:<2}{}{}",
        bref.data_off,
        libhammer2::subs::get_blockref_type_string(bref.typ),
        bref.key,
        bref.keybits,
        if msg.is_empty() { "" } else { " " },
        if msg.is_empty() { "" } else { msg },
    )
}

fn print_blockref_debug(
    bref: &libhammer2::fs::Hammer2Blockref,
    msg: &str,
    opt: &Hammer2FsckOptions,
) {
    if opt.debug {
        println!("{}", format_blockref(1, bref, msg));
    }
}

fn print_blockref_entry(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    root: &BlockrefMap,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<()> {
    for e in root.values() {
        for m in e {
            eprintln!(
                "{}",
                format_blockref(
                    1,
                    &m.bref,
                    &libhammer2::util::bin_to_string(&m.msg).unwrap()
                )
            );
            if opt.verbose {
                match fso.read_media(&m.bref) {
                    Ok(v) => {
                        for s in &format_media(2, &m.bref, &v) {
                            eprint!("{s}");
                        }
                        std::io::stderr().flush()?;
                    }
                    Err(e) => {
                        tab::eprintln!(2, "Failed to read media: {e}");
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_blockref_stats(
    bstats: &BlockrefStats,
    newline: bool,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<()> {
    let emptybuf = if opt.count_empty {
        format!(", {} empty", bstats.total_empty)
    } else {
        String::new()
    };
    let buf = match bstats.typ {
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP => tab::format!(
            1,
            "{} blockref ({} node, {} leaf{}), {}",
            bstats.total_blockref,
            bstats.freemap.total_freemap_node,
            bstats.freemap.total_freemap_leaf,
            emptybuf,
            libhammer2::subs::get_size_string(bstats.total_bytes)
        ),
        libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => tab::format!(
            1,
            "{} blockref ({} inode, {} indirect, {} data, {} dirent{}), {}",
            bstats.total_blockref,
            bstats.volume.total_inode,
            bstats.volume.total_indirect,
            bstats.volume.total_data,
            bstats.volume.total_dirent,
            emptybuf,
            libhammer2::subs::get_size_string(bstats.total_bytes)
        ),
        _ => panic!("{}", bstats.typ),
    };
    let buf = if let Some(v) = buf.get(..libhammer2::subs::get_chars_per_line()) {
        v.to_string()
    } else {
        buf
    };
    if newline {
        println!("{buf}");
    } else {
        print!("{buf}\r");
        std::io::stdout().flush()?;
    }
    Ok(())
}

fn verify_volume_header(voldata: &libhammer2::fs::Hammer2VolumeData) -> std::io::Result<()> {
    if voldata.magic != libhammer2::fs::HAMMER2_VOLUME_ID_HBO
        && voldata.magic != libhammer2::fs::HAMMER2_VOLUME_ID_ABO
    {
        tab::eprintln!(1, "Bad magic {:x}", voldata.magic);
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    if voldata.magic == libhammer2::fs::HAMMER2_VOLUME_ID_ABO {
        tab::eprintln!(1, "Reverse endian");
    }

    let a = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRC0_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRC0_SIZE,
    );
    let b = voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT0];
    if a != b {
        tab::eprintln!(1, "Bad HAMMER2_VOL_ICRC_SECT0 CRC");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    let a = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRC1_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRC1_SIZE,
    );
    let b = voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT1];
    if a != b {
        tab::eprintln!(1, "Bad HAMMER2_VOL_ICRC_SECT1 CRC");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    let a = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_SIZE,
    );
    let b = voldata.icrc_volheader;
    if a != b {
        tab::eprintln!(1, "Bad volume header CRC");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn verify_blockref(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    bref: &libhammer2::fs::Hammer2Blockref,
    norecurse: bool,
    bstats: &mut BlockrefStats,
    droot: &mut BlockrefMap,
    opt: &Hammer2FsckOptions,
) -> std::io::Result<DeltaStats> {
    let mut dstats = DeltaStats::new();
    if bref.data_off != 0 {
        if let Some(v) = droot.get(&bref.data_off) {
            for m in v {
                if m.bref == *bref {
                    // delta contains cached delta
                    let ds = m.msg_as();
                    dstats.add(ds);
                    bstats.load(ds);
                    print_blockref_debug(&m.bref, "cache-hit", opt);
                    return Ok(dstats);
                }
            }
        }
    }
    bstats.total_blockref += 1;
    dstats.total_blockref += 1;

    let mut failed = false;
    match bref.typ {
        libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY => {
            if opt.count_empty {
                bstats.total_empty += 1;
                dstats.total_empty += 1;
            } else {
                bstats.total_blockref -= 1;
                dstats.total_blockref -= 1;
            }
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
            bstats.volume.total_inode += 1;
            dstats.volume.total_inode += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT => {
            bstats.volume.total_indirect += 1;
            dstats.volume.total_indirect += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_DATA => {
            bstats.volume.total_data += 1;
            dstats.volume.total_data += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_DIRENT => {
            bstats.volume.total_dirent += 1;
            dstats.volume.total_dirent += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE => {
            bstats.freemap.total_freemap_node += 1;
            dstats.freemap.total_freemap_node += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF => {
            bstats.freemap.total_freemap_leaf += 1;
            dstats.freemap.total_freemap_leaf += 1;
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP | libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => {
            bstats.total_blockref -= 1;
            dstats.total_blockref -= 1;
        }
        _ => {
            let msg = format!("Invalid blockref type {}", bref.typ);
            add_blockref_entry_from_str(&mut bstats.root, bref, &msg);
            print_blockref_debug(bref, &msg, opt);
            failed = true;
        }
    }

    let media = match fso.read_media(bref) {
        Ok(v) => v,
        Err(e) => {
            let msg = match e.kind() {
                std::io::ErrorKind::InvalidInput => "Bad I/O bytes",
                _ => "Failed to read media",
            };
            add_blockref_entry_from_str(&mut bstats.root, bref, msg);
            print_blockref_debug(bref, msg, opt);
            return Err(e);
        }
    };
    let bytes = u64::try_from(media.len()).unwrap();
    if bref.typ != libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME
        && bref.typ != libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP
    {
        bstats.total_bytes += bytes;
        dstats.total_bytes += bytes;
    }
    if !opt.count_empty && bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY {
        assert_eq!(bytes, 0);
        bstats.total_bytes -= bytes;
        dstats.total_bytes -= bytes;
    }
    if !opt.debug && !opt.quiet && bstats.total_blockref % 100 == 0 {
        print_blockref_stats(bstats, false, opt)?;
    }

    if bytes != 0 {
        match libhammer2::fs::dec_check(bref.methods) {
            libhammer2::fs::HAMMER2_CHECK_ISCSI32 => {
                let cv = icrc32::iscsi_crc32(&media);
                if bref
                    .check_as::<libhammer2::fs::Hammer2BlockrefCheckIscsi>()
                    .value
                    != cv
                {
                    let msg = "Bad HAMMER2_CHECK_ISCSI32";
                    add_blockref_entry_from_str(&mut bstats.root, bref, msg);
                    print_blockref_debug(bref, msg, opt);
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_XXHASH64 => {
                let cv = libhammer2::xxhash::xxh64(&media);
                if bref
                    .check_as::<libhammer2::fs::Hammer2BlockrefCheckXxhash64>()
                    .value
                    != cv
                {
                    let msg = "Bad HAMMER2_CHECK_XXHASH64";
                    add_blockref_entry_from_str(&mut bstats.root, bref, msg);
                    print_blockref_debug(bref, msg, opt);
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_SHA192 => {
                let cv = libhammer2::sha::sha256(&media);
                if bref
                    .check_as::<libhammer2::fs::Hammer2BlockrefCheckSha256>()
                    .data
                    != cv.as_slice()
                {
                    let msg = "Bad HAMMER2_CHECK_SHA192";
                    add_blockref_entry_from_str(&mut bstats.root, bref, msg);
                    print_blockref_debug(bref, msg, opt);
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_FREEMAP => {
                let cv = icrc32::iscsi_crc32(&media);
                if bref
                    .check_as::<libhammer2::fs::Hammer2BlockrefCheckFreemap>()
                    .icrc32
                    != cv
                {
                    let msg = "Bad HAMMER2_CHECK_FREEMAP";
                    add_blockref_entry_from_str(&mut bstats.root, bref, msg);
                    print_blockref_debug(bref, msg, opt);
                    failed = true;
                }
            }
            _ => (),
        }
        let bscan = match bref.typ {
            libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
                let ipdata = libhammer2::util::align_to::<libhammer2::fs::Hammer2InodeData>(&media);
                if ipdata.meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_DIRECTDATA == 0 {
                    ipdata
                        .u_as::<libhammer2::fs::Hammer2Blockset>()
                        .as_blockref()
                        .to_vec()
                } else {
                    vec![]
                }
            }
            libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT
            | libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE => libhammer2::extra::media_as(&media),
            libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP => {
                libhammer2::util::align_to::<libhammer2::fs::Hammer2VolumeData>(&media)
                    .freemap_blockset
                    .as_blockref()
                    .to_vec()
            }
            libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => {
                libhammer2::util::align_to::<libhammer2::fs::Hammer2VolumeData>(&media)
                    .sroot_blockset
                    .as_blockref()
                    .to_vec()
            }
            _ => vec![],
        };
        let norecurse = if opt.force { false } else { norecurse };
        // If failed, no recurse, but still verify its direct children.
        // Beyond that is probably garbage.
        if !norecurse {
            for bref in bscan {
                let ds = verify_blockref(fso, bref, failed, bstats, droot, opt)?;
                if !failed {
                    dstats.add(&ds);
                }
            }
        }
    }

    if failed {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    dstats.count += 1;
    if bref.data_off != 0
        && opt.blockref_cache_count > 0
        && dstats.count >= opt.blockref_cache_count
    {
        assert!(bytes > 0);
        add_blockref_entry(droot, bref, &dstats);
        print_blockref_debug(bref, "cache-add", opt);
    }
    Ok(dstats)
}

fn print_pfs(ipdata: &libhammer2::fs::Hammer2InodeData) {
    let meta = &ipdata.meta;
    let type_str = if meta.pfs_type == libhammer2::fs::HAMMER2_PFSTYPE_MASTER {
        if meta.pfs_subtype == libhammer2::fs::HAMMER2_PFSSUBTYPE_NONE {
            "MASTER"
        } else {
            libhammer2::subs::get_pfs_subtype_string(meta.pfs_subtype)
        }
    } else {
        libhammer2::subs::get_pfs_subtype_string(meta.pfs_type)
    };
    tab::println!(
        1,
        "{type_str:<11} {} {}",
        libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_clid),
        ipdata.get_filename_string()
    );
}

fn scan_pfs_blockref(
    fso: &mut libhammer2::ondisk::Hammer2Ondisk,
    bref: &libhammer2::fs::Hammer2Blockref,
) -> std::io::Result<BlockrefEntry> {
    let media = fso.read_media(bref)?;
    if media.is_empty() {
        return Ok(vec![]);
    }
    let mut v = vec![];
    let bscan = match bref.typ {
        libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
            let ipdata = libhammer2::util::align_to::<libhammer2::fs::Hammer2InodeData>(&media);
            if ipdata.meta.pfs_type == libhammer2::fs::HAMMER2_PFSTYPE_SUPROOT {
                ipdata
                    .u_as::<libhammer2::fs::Hammer2Blockset>()
                    .as_blockref()
                    .to_vec()
            } else {
                if ipdata.meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_PFSROOT != 0 {
                    v.push(BlockrefMessage::new_from(bref, ipdata));
                } else {
                    panic!("{}", ipdata.meta.inum); // should only see SUPROOT or PFS
                }
                vec![]
            }
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT => libhammer2::extra::media_as(&media),
        libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => {
            libhammer2::util::align_to::<libhammer2::fs::Hammer2VolumeData>(&media)
                .sroot_blockset
                .as_blockref()
                .to_vec()
        }
        _ => vec![],
    };
    for bref in &bscan {
        v.extend(scan_pfs_blockref(fso, bref)?);
    }
    v.sort_by_key(|m| m.msg_as::<libhammer2::fs::Hammer2InodeData>().filename);
    Ok(v)
}

#[allow(clippy::too_many_lines)]
fn format_media(tab: usize, bref: &libhammer2::fs::Hammer2Blockref, media: &[u8]) -> Vec<String> {
    let mut v = vec![];
    match bref.typ {
        libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
            let ipdata = libhammer2::util::align_to::<libhammer2::fs::Hammer2InodeData>(media);
            let meta = &ipdata.meta;
            v.push(tab::format!(
                tab,
                "filename \"{}\"\n",
                ipdata.get_filename_string()
            ));
            v.push(tab::format!(tab, "version {}\n", meta.version));
            let ispfs = (meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_PFSROOT) != 0
                || meta.pfs_type == libhammer2::fs::HAMMER2_PFSTYPE_SUPROOT;
            if ispfs {
                v.push(tab::format!(
                    tab,
                    "pfs_subtype {} ({})\n",
                    meta.pfs_subtype,
                    libhammer2::subs::get_pfs_subtype_string(meta.pfs_subtype)
                ));
            }
            v.push(tab::format!(tab, "uflags {:#010x}\n", meta.uflags));
            if meta.rmajor != 0 || meta.rminor != 0 {
                v.push(tab::format!(tab, "rmajor {}\n", meta.rmajor));
                v.push(tab::format!(tab, "rminor {}\n", meta.rminor));
            }
            v.push(tab::format!(
                tab,
                "ctime {}\n",
                libhammer2::subs::get_local_time_string(meta.ctime)
            ));
            v.push(tab::format!(
                tab,
                "mtime {}\n",
                libhammer2::subs::get_local_time_string(meta.mtime)
            ));
            v.push(tab::format!(
                tab,
                "atime {}\n",
                libhammer2::subs::get_local_time_string(meta.atime)
            ));
            v.push(tab::format!(
                tab,
                "btime {}\n",
                libhammer2::subs::get_local_time_string(meta.btime)
            ));
            v.push(tab::format!(
                tab,
                "uid {}\n",
                libhammer2::subs::get_uuid_string_from_bytes(&meta.uid)
            ));
            v.push(tab::format!(
                tab,
                "gid {}\n",
                libhammer2::subs::get_uuid_string_from_bytes(&meta.gid)
            ));
            v.push(tab::format!(
                tab,
                "type {}\n",
                libhammer2::subs::get_inode_type_string(meta.typ)
            ));
            v.push(tab::format!(tab, "op_flags {:#04x}\n", meta.op_flags));
            v.push(tab::format!(tab, "cap_flags {:#06x}\n", meta.cap_flags));
            v.push(tab::format!(tab, "mode {:<7o}\n", meta.mode));
            v.push(tab::format!(tab, "inum {:#018x}\n", meta.inum));
            v.push(tab::format!(tab, "size {} ", meta.size));
            if (meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_DIRECTDATA) != 0
                && meta.size <= libhammer2::fs::HAMMER2_EMBEDDED_BYTES
            {
                v.push("(embedded data)\n".to_string());
            } else {
                v.push("\n".to_string());
            }
            v.push(tab::format!(tab, "nlinks {}\n", meta.nlinks));
            v.push(tab::format!(tab, "iparent {:#018x}\n", meta.iparent));
            v.push(tab::format!(tab, "name_key {:#018x}\n", meta.name_key));
            v.push(tab::format!(tab, "name_len {}\n", meta.name_len));
            v.push(tab::format!(tab, "ncopies {}\n", meta.ncopies));
            v.push(tab::format!(
                tab,
                "comp_algo {}\n",
                libhammer2::subs::get_comp_mode_string(meta.comp_algo)
            ));
            v.push(tab::format!(
                tab,
                "check_algo {}\n",
                libhammer2::subs::get_check_mode_string(meta.check_algo)
            ));
            if ispfs {
                v.push(tab::format!(tab, "pfs_nmasters {}\n", meta.pfs_nmasters));
                v.push(tab::format!(
                    tab,
                    "pfs_type {} ({})\n",
                    meta.pfs_type,
                    libhammer2::subs::get_pfs_type_string(meta.pfs_type)
                ));
                v.push(tab::format!(tab, "pfs_inum {:#018x}\n", meta.pfs_inum));
                v.push(tab::format!(
                    tab,
                    "pfs_clid {}\n",
                    libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_clid)
                ));
                v.push(tab::format!(
                    tab,
                    "pfs_fsid {}\n",
                    libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_fsid)
                ));
                v.push(tab::format!(
                    tab,
                    "pfs_lsnap_tid {:#018x}\n",
                    meta.pfs_lsnap_tid
                ));
            }
            v.push(tab::format!(tab, "data_quota {}\n", meta.data_quota));
            v.push(tab::format!(
                tab,
                "data_count {}\n",
                bref.embed_as::<libhammer2::fs::Hammer2BlockrefEmbedStats>()
                    .data_count
            ));
            v.push(tab::format!(tab, "inode_quota {}\n", meta.inode_quota));
            v.push(tab::format!(
                tab,
                "inode_count {}\n",
                bref.embed_as::<libhammer2::fs::Hammer2BlockrefEmbedStats>()
                    .inode_count
            ));
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT
        | libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE => {
            for (i, bref) in libhammer2::extra::media_as::<libhammer2::fs::Hammer2Blockref>(media)
                .iter()
                .enumerate()
            {
                v.push(tab::format!(
                    tab,
                    "{i:<3} {:016x} {:<12} {:016x}/{:<2}\n",
                    bref.data_off,
                    libhammer2::subs::get_blockref_type_string(bref.typ),
                    bref.key,
                    bref.keybits
                ));
            }
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_DIRENT => {
            let dirent = bref.embed_as::<libhammer2::fs::Hammer2DirentHead>();
            let namelen = usize::from(dirent.namlen);
            v.push(tab::format!(
                tab,
                "filename \"{}\"\n",
                if namelen <= bref.check.len() {
                    std::str::from_utf8(&bref.check[..namelen])
                } else {
                    std::str::from_utf8(&media[..namelen])
                }
                .unwrap()
            ));
            v.push(tab::format!(tab, "inum {:#018x}\n", dirent.inum));
            v.push(tab::format!(tab, "namelen {}\n", dirent.namlen));
            v.push(tab::format!(
                tab,
                "type {}\n",
                libhammer2::subs::get_inode_type_string(dirent.typ)
            ));
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF => {
            let bmdata = libhammer2::extra::media_as::<libhammer2::fs::Hammer2BmapData>(media);
            for i in 0..libhammer2::fs::HAMMER2_FREEMAP_COUNT {
                let bmdata = &bmdata[i];
                let data_off = bref.key
                    + u64::try_from(i).unwrap() * libhammer2::fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                v.push(tab::format!(
                    tab,
                    "{data_off:016x} {i:04}.{:04x} (avail={:07}) \
                    {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}\n",
                    bmdata.class,
                    bmdata.avail,
                    bmdata.bitmapq[0],
                    bmdata.bitmapq[1],
                    bmdata.bitmapq[2],
                    bmdata.bitmapq[3],
                    bmdata.bitmapq[4],
                    bmdata.bitmapq[5],
                    bmdata.bitmapq[6],
                    bmdata.bitmapq[7]
                ));
            }
        }
        _ => (),
    }
    v
}

pub(crate) fn fsck(devpath: &str, opt: &Hammer2FsckOptions) -> std::io::Result<()> {
    let mut fso = libhammer2::ondisk::init(devpath, true)?;
    let best = fso.get_best_volume_data()?[libhammer2::fs::HAMMER2_ROOT_VOLUME as usize];
    let zone = best.0;
    if opt.print_pfs {
        return test_pfs_blockref(&mut fso, zone, opt);
    }
    println!("volume header");
    if let Err(e) = test_volume_header(&mut fso, zone, opt) {
        if !opt.force {
            return Err(e);
        }
    }
    println!("freemap");
    if let Err(e) = test_blockref(
        &mut fso,
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP,
        zone,
        opt,
    ) {
        if !opt.force {
            return Err(e);
        }
    }
    println!("volume");
    if !opt.scan_pfs {
        if let Err(e) = test_blockref(
            &mut fso,
            libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME,
            zone,
            opt,
        ) {
            if !opt.force {
                return Err(e);
            }
        }
    } else if true {
        if let Err(e) = test_pfs_blockref(&mut fso, zone, opt) {
            if !opt.force {
                return Err(e);
            }
        }
    } else {
        unreachable!();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    macro_rules! eq {
        ($val: expr, $ptr: expr) => {
            let a = format!("{:?}", std::ptr::addr_of!($val));
            let b = format!("{:?}", std::ptr::from_ref($ptr));
            assert_eq!(a, b);
        };
    }

    #[test]
    fn test_blockref_message_new_from_str() {
        let bref = libhammer2::fs::Hammer2Blockref::new_empty();
        for s in [String::new(), "A".to_string(), "A".repeat(1024)] {
            let m = super::BlockrefMessage::new_from_str(&bref, &s);
            assert_eq!(libhammer2::util::bin_to_string(&m.msg).unwrap(), s);
        }
    }

    #[test]
    fn test_blockref_message_new_from() {
        let bref = libhammer2::fs::Hammer2Blockref::new_empty();
        let m = super::BlockrefMessage::new_from(&bref, &super::DeltaStats::new());
        eq!(m.msg, m.msg_as::<super::DeltaStats>());
        let m = super::BlockrefMessage::new_from(&bref, &libhammer2::fs::Hammer2InodeData::new());
        eq!(m.msg, m.msg_as::<libhammer2::fs::Hammer2InodeData>());
    }
}
