use crate::Hammer2Options;
use hammer2_utils::hammer2fs;
use hammer2_utils::ondisk;
use hammer2_utils::sha;
use hammer2_utils::subs;
use hammer2_utils::util;
use hammer2_utils::xxhash;

macro_rules! tabprint {
    ($tab: expr, $($args: tt)*) => {
        print!("{}", " ".repeat($tab));
        print!($($args)*);
    }
}

macro_rules! tabprintln {
    ($tab: expr, $($args: tt)*) => {
        print!("{}", " ".repeat($tab));
        println!($($args)*);
    }
}

#[derive(Debug, Default)]
pub(crate) struct ShowOptions {
    pub(crate) all_volume_data: bool,
    pub(crate) tab: usize,
    pub(crate) depth: usize,
    pub(crate) min_mirror_tid: u64,
    pub(crate) min_modify_tid: u64,
    pub(crate) init_tab: usize,
}

impl ShowOptions {
    pub(crate) fn new(
        all_volume_data: bool,
        tab: usize,
        depth: usize,
        min_mirror_tid: u64,
        min_modify_tid: u64,
        init_tab: usize,
    ) -> Self {
        Self {
            all_volume_data,
            tab,
            depth,
            min_mirror_tid,
            min_modify_tid,
            init_tab,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct FreemapStat {
    pub(crate) accum16: [u64; 4],
    pub(crate) accum64: [u64; 4],
    pub(crate) unavail: u64,
    pub(crate) freemap: u64,
}

impl FreemapStat {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

pub(crate) fn print_volume_summary(id: usize, index: usize, mirror_tid: u64) {
    println!("Volume {id} header {index}: mirror_tid={mirror_tid:016x}");
}

pub(crate) fn show_volume_data(
    fso: &mut ondisk::Hammer2Ondisk,
    voldata: &hammer2fs::Hammer2VolumeData,
    bi: usize,
    sopt: &ShowOptions,
    opt: &Hammer2Options,
) -> std::io::Result<()> {
    println!();
    println!("Volume {} header {bi} {{", voldata.volu_id);
    println!("    magic          {:#018x}", voldata.magic);
    println!("    boot_beg       {:#018x}", voldata.boot_beg);
    println!(
        "    boot_end       {:#018x} ({:6.2}MB)",
        voldata.boot_end,
        (voldata.boot_end - voldata.boot_beg) as f64 / subs::M_F64
    );
    println!("    aux_beg        {:#018x}", voldata.aux_beg);
    println!(
        "    aux_end        {:#018x} ({:6.2}MB)",
        voldata.aux_end,
        (voldata.aux_end - voldata.aux_beg) as f64 / subs::M_F64
    );
    println!(
        "    volu_size      {:#018x} ({:6.2}GB)",
        voldata.volu_size,
        voldata.volu_size as f64 / subs::G_F64
    );
    println!("    version        {}", voldata.version);
    println!("    flags          {:#010x}", voldata.flags);
    println!("    copyid         {}", voldata.copyid);
    println!("    freemap_vers   {}", voldata.freemap_version);
    println!("    peer_type      {}", voldata.peer_type);
    println!("    volu_id        {}", voldata.volu_id);
    println!("    nvolumes       {}", voldata.nvolumes);

    println!(
        "    fsid           {}",
        subs::get_uuid_string_from_bytes(&voldata.fsid)
    );
    let s = subs::get_uuid_string_from_bytes(&voldata.fstype);
    println!("    fstype         {s}");
    let name = if s == hammer2fs::HAMMER2_UUID_STRING {
        "DragonFly HAMMER2"
    } else {
        "?"
    };
    println!("                   ({name})");

    println!(
        "    allocator_size {:#018x} ({:6.2}GB)",
        voldata.allocator_size,
        voldata.allocator_size as f64 / subs::G_F64
    );
    println!(
        "    allocator_free {:#018x} ({:6.2}GB)",
        voldata.allocator_free,
        voldata.allocator_free as f64 / subs::G_F64
    );
    println!(
        "    allocator_beg  {:#018x} ({:6.2}GB)",
        voldata.allocator_beg,
        voldata.allocator_beg as f64 / subs::G_F64
    );

    println!("    mirror_tid     {:#018x}", voldata.mirror_tid);
    println!("    reserved0080   {:#018x}", voldata.reserved0080);
    println!("    reserved0088   {:#018x}", voldata.reserved0088);
    println!("    freemap_tid    {:#018x}", voldata.freemap_tid);
    println!("    bulkfree_tid   {:#018x}", voldata.bulkfree_tid);
    for (i, x) in voldata.reserved00a0.iter().enumerate() {
        println!("    reserved00A0/{} {:#018x}", i, *x);
    }
    println!("    total_size     {:#018x}", voldata.total_size);

    print!("    copyexists    ");
    for x in &voldata.copyexists {
        print!(" {:#04x}", *x);
    }
    println!();

    // Note: Index numbers and ICRC_SECTn definitions are not matched,
    // the ICRC for sector 0 actually uses the last index, for
    // example.
    //
    // Note: The whole voldata CRC does not have to match critically
    // as certain sub-areas of the volume header have their own
    // CRCs.
    println!();
    for i in 0..voldata.icrc_sects.len() {
        print!("    icrc_sects[{i}]  ");
        match i {
            hammer2fs::HAMMER2_VOL_ICRC_SECT0 => {
                let a = voldata.get_crc(
                    hammer2fs::HAMMER2_VOLUME_ICRC0_OFF,
                    hammer2fs::HAMMER2_VOLUME_ICRC0_SIZE,
                );
                let b = voldata.icrc_sects[hammer2fs::HAMMER2_VOL_ICRC_SECT0];
                print!("{a:#010x}/{b:#010x}");
                if a == b {
                    print!(" (OK)");
                } else {
                    print!(" (FAILED)");
                }
            }
            hammer2fs::HAMMER2_VOL_ICRC_SECT1 => {
                let a = voldata.get_crc(
                    hammer2fs::HAMMER2_VOLUME_ICRC1_OFF,
                    hammer2fs::HAMMER2_VOLUME_ICRC1_SIZE,
                );
                let b = voldata.icrc_sects[hammer2fs::HAMMER2_VOL_ICRC_SECT1];
                print!("{a:#010x}/{b:#010x}");
                if a == b {
                    print!(" (OK)");
                } else {
                    print!(" (FAILED)");
                }
            }
            _ => print!("{:#010x} (reserved)", voldata.icrc_sects[i]),
        }
        println!();
    }
    let a = voldata.get_crc(
        hammer2fs::HAMMER2_VOLUME_ICRCVH_OFF,
        hammer2fs::HAMMER2_VOLUME_ICRCVH_SIZE,
    );
    let b = voldata.icrc_volheader;
    print!("    icrc_volhdr    {a:#010x}/{b:#010x}");
    if a == b {
        println!(" (OK)");
    } else {
        println!(" (FAILED - not a critical error))");
    }

    // The super-root and freemap blocksets (not recursed)
    println!();
    println!("    sroot_blockset {{");
    for i in 0..hammer2fs::HAMMER2_SET_COUNT {
        show_blockref(
            fso,
            voldata,
            sopt.init_tab,
            i,
            &voldata.sroot_blockset.blockref[i],
            true,
            &mut None,
            sopt,
            opt,
        )?;
    }
    println!("    }}");
    println!("    freemap_blockset {{");
    for i in 0..hammer2fs::HAMMER2_SET_COUNT {
        show_blockref(
            fso,
            voldata,
            sopt.init_tab,
            i,
            &voldata.freemap_blockset.blockref[i],
            true,
            &mut None,
            sopt,
            opt,
        )?;
    }
    println!("    }}");

    let mut all_zero = true;
    for x in &voldata.volu_loff {
        if *x != 0 {
            all_zero = false;
            break;
        }
    }
    if !all_zero {
        println!();
        for (i, x) in voldata.volu_loff.iter().enumerate() {
            if *x != u64::MAX {
                println!("    volu_loff[{}]   {:#018x}", i, *x);
            }
        }
    }
    println!("}}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn show_blockref(
    fso: &mut ondisk::Hammer2Ondisk,
    voldata: &hammer2fs::Hammer2VolumeData,
    tab: usize,
    bi: usize,
    bref: &hammer2fs::Hammer2Blockref,
    norecurse: bool,
    stat: &mut Option<FreemapStat>,
    sopt: &ShowOptions,
    opt: &Hammer2Options,
) -> std::io::Result<()> {
    // omit if smaller than mininum mirror_tid threshold
    if bref.mirror_tid < sopt.min_mirror_tid {
        return Ok(());
    }
    // omit if smaller than mininum modify_tid threshold
    if bref.modify_tid < sopt.min_modify_tid
        && (bref.modify_tid != 0
            || (bref.typ == hammer2fs::HAMMER2_BREF_TYPE_INODE && bref.leaf_count == 0))
    {
        return Ok(());
    }

    let radix = bref.data_off & hammer2fs::HAMMER2_OFF_MASK_RADIX;
    let bytes = if radix == 0 { 0 } else { 1 << radix };
    let media = read_media(fso, bref, bytes, opt)?;
    let type_str = subs::get_blockref_type_string(bref.typ);
    let type_pad = if type_str.len() > 8 {
        0
    } else {
        8 - type_str.len()
    };

    let bscan = match bref.typ {
        hammer2fs::HAMMER2_BREF_TYPE_INODE => {
            let ipdata = util::align_to::<hammer2fs::Hammer2InodeData>(&media);
            if ipdata.meta.op_flags & hammer2fs::HAMMER2_OPFLAG_DIRECTDATA == 0 {
                get_blockref_from_blockset(ipdata.u_as_blockset())
            } else {
                vec![]
            }
        }
        hammer2fs::HAMMER2_BREF_TYPE_INDIRECT | hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_NODE => {
            get_from_media(&media)
        }
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP => get_blockref_from_blockset(
            &util::align_to::<hammer2fs::Hammer2VolumeData>(&media).freemap_blockset,
        ),
        hammer2fs::HAMMER2_BREF_TYPE_VOLUME => get_blockref_from_blockset(
            &util::align_to::<hammer2fs::Hammer2VolumeData>(&media).sroot_blockset,
        ),
        _ => vec![],
    };

    let id = fso
        .get_volume(bref.data_off)
        .ok_or_else(util::notfound)?
        .get_id();
    if opt.quiet {
        tabprint!(
            tab,
            "{type_str}.{bi:<3} {:016x} {:016x}/{:<2} vol={id} mir={:016x} mod={:016x} leafcnt={} ",
            bref.data_off,
            bref.key,
            bref.keybits,
            bref.mirror_tid,
            bref.modify_tid,
            bref.leaf_count
        );
    } else {
        tabprintln!(
            tab,
            "{type_str}.{bi:<3}{} {:016x} {:016x}/{:<2} ",
            " ".repeat(type_pad),
            bref.data_off,
            bref.key,
            bref.keybits
        );
        tabprint!(
            tab + 13,
            "vol={id} mir={:016x} mod={:016x} lfcnt={} ",
            bref.mirror_tid,
            bref.modify_tid,
            bref.leaf_count
        );
        if !bscan.is_empty()
            || bref.flags != 0
            || bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_NODE
            || bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF
        {
            println!();
            tabprint!(tab + 13, "");
        }
    }
    if !bscan.is_empty() {
        print!("bcnt={} ", bscan.len());
    }
    if bref.flags != 0 {
        print!("flags={:02x} ", bref.flags);
    }
    if bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_NODE
        || bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF
    {
        let freemap = bref.check_as_freemap();
        print!("bigmask={:08x} avail={} ", freemap.bigmask, freemap.avail);
    }

    // Check data integrity in verbose mode, otherwise we are just doing
    // a quick meta-data scan.  Meta-data integrity is always checked.
    // (Also see the check above that ensures the media data is loaded,
    // otherwise there's no data to check!).
    //
    // WARNING! bref->check state may be used for other things when
    // bref has no data (bytes == 0).
    let mut failed = false;
    if bytes > 0 && (bref.typ != hammer2fs::HAMMER2_BREF_TYPE_DATA || opt.verbose) {
        if !opt.quiet {
            println!();
            tabprint!(tab + 13, "");
        }
        let check_algo = hammer2fs::dec_check(bref.methods);
        let check_str = subs::get_check_mode_string(check_algo);
        let comp_algo = hammer2fs::dec_comp(bref.methods);
        let comp_str = subs::get_comp_mode_string(comp_algo);
        let meth = format!("{check_str}|{comp_str}");
        match check_algo {
            hammer2fs::HAMMER2_CHECK_NONE | hammer2fs::HAMMER2_CHECK_DISABLED => {
                print!("meth={meth} ");
            }
            hammer2fs::HAMMER2_CHECK_ISCSI32 => {
                let cv = icrc32::iscsi_crc32(&media);
                let iscsi32 = bref.check_as_iscsi32();
                if iscsi32.value == cv {
                    print!("meth={meth} icrc={cv:08x} ");
                } else {
                    print!("(icrc {meth} {:08x}/{cv:08x} failed) ", iscsi32.value);
                    failed = true;
                }
            }
            hammer2fs::HAMMER2_CHECK_XXHASH64 => {
                let cv = xxhash::xxh64(&media);
                let xxhash64 = bref.check_as_xxhash64();
                if xxhash64.value == cv {
                    print!("meth={meth} xxh={cv:016x} ");
                } else {
                    print!("(xxh {meth} {:016x}/{cv:016x} failed) ", xxhash64.value);
                    failed = true;
                }
            }
            hammer2fs::HAMMER2_CHECK_SHA192 => {
                let cv = sha::sha256(&media);
                let sha256 = bref.check_as_sha256();
                if sha256.data == cv.as_slice() {
                    print!("meth={meth} ");
                } else {
                    print!("(sha192 {meth} failed) ");
                    failed = true;
                }
            }
            hammer2fs::HAMMER2_CHECK_FREEMAP => {
                let cv = icrc32::iscsi_crc32(&media);
                let freemap = bref.check_as_freemap();
                if freemap.icrc32 == cv {
                    print!("meth={meth} fcrc={cv:08x} ");
                } else {
                    print!("(fcrc {meth} {:08x}/{cv:08x} failed) ", freemap.icrc32);
                    failed = true;
                }
            }
            _ => panic!("{check_algo}"),
        }
    }

    let tab = tab + sopt.tab;
    let obrace = if opt.quiet {
        println!();
        false
    } else {
        show_blockref_data(&media, tab, bref, norecurse)
    };

    // Update statistics.
    if let Some(ref mut stat) = stat {
        if bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF {
            let bmdata = get_from_media(&media);
            for i in 0..hammer2fs::HAMMER2_FREEMAP_COUNT {
                let bmdata = &bmdata[i];
                let data_off =
                    bref.key + u64::try_from(i).unwrap() * hammer2fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                if data_off >= voldata.aux_end && data_off < fso.get_total_size() {
                    for j in 0..4 {
                        count_blocks(bmdata, j, stat);
                    }
                } else {
                    stat.unavail += hammer2fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                }
            }
            stat.freemap += hammer2fs::HAMMER2_FREEMAP_LEVEL1_SIZE;
        }
    }

    // Recurse if norecurse == 0.  If the CRC failed, pass norecurse = 1.
    // That is, if an indirect or inode fails we still try to list its
    // direct children to help with debugging, but go no further than
    // that because they are probably garbage.
    if (sopt.depth == usize::MAX || ((tab - sopt.init_tab) / sopt.tab) < sopt.depth) && !norecurse {
        for (i, bref) in bscan.iter().enumerate() {
            if bref.typ != hammer2fs::HAMMER2_BREF_TYPE_EMPTY {
                show_blockref(fso, voldata, tab, i, bref, failed, stat, sopt, opt)?;
            }
        }
    }
    let tab = tab - sopt.tab;
    if obrace {
        if bref.typ == hammer2fs::HAMMER2_BREF_TYPE_INODE {
            let ipdata = util::align_to::<hammer2fs::Hammer2InodeData>(&media);
            tabprintln!(
                tab,
                "}} ({}.{}, \"{}\")",
                type_str,
                bi,
                ipdata.get_filename_string()
            );
        } else {
            tabprintln!(tab, "}} ({}.{})", type_str, bi);
        }
    }
    Ok(())
}

fn read_media(
    fso: &mut ondisk::Hammer2Ondisk,
    bref: &hammer2fs::Hammer2Blockref,
    bytes: u64,
    opt: &Hammer2Options,
) -> std::io::Result<Vec<u8>> {
    if bytes == 0 {
        return Ok(vec![]);
    }
    let io_off = bref.data_off & !hammer2fs::HAMMER2_OFF_MASK_RADIX;
    let io_base = io_off & !(hammer2fs::HAMMER2_LBUFSIZE - 1);
    let boff = io_off - io_base;
    let mut io_bytes = hammer2fs::HAMMER2_LBUFSIZE;
    while io_bytes + boff < bytes {
        io_bytes <<= 1;
    }
    if io_bytes > hammer2fs::HAMMER2_PBUFSIZE {
        log::error!("(bad block size {bytes})");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    if bref.typ != hammer2fs::HAMMER2_BREF_TYPE_DATA || opt.verbose {
        let vol = fso.get_volume_mut(io_off).ok_or_else(util::notfound)?;
        Ok(vol.preadx(io_bytes, io_base - vol.get_offset())?
            [usize::try_from(boff).unwrap()..usize::try_from(boff + bytes).unwrap()]
            .to_vec())
    } else {
        Ok(vec![])
    }
}

fn get_from_media<T>(media: &[u8]) -> Vec<&T> {
    let x = std::mem::size_of::<T>();
    let n = media.len() / x;
    let mut v = vec![];
    for i in 0..n {
        v.push(util::align_to::<T>(&media[i * x..(i + 1) * x]));
    }
    v
}

fn get_blockref_from_blockset(
    blockset: &hammer2fs::Hammer2Blockset,
) -> Vec<&hammer2fs::Hammer2Blockref> {
    vec![
        &blockset.blockref[0],
        &blockset.blockref[1],
        &blockset.blockref[2],
        &blockset.blockref[3],
    ]
}

fn show_blockref_data(
    media: &[u8],
    tab: usize,
    bref: &hammer2fs::Hammer2Blockref,
    norecurse: bool,
) -> bool {
    match bref.typ {
        hammer2fs::HAMMER2_BREF_TYPE_EMPTY => {
            if norecurse {
                println!();
            }
            false
        }
        hammer2fs::HAMMER2_BREF_TYPE_INODE => {
            println!("{{");
            let ipdata = util::align_to::<hammer2fs::Hammer2InodeData>(media);
            let meta = &ipdata.meta;
            tabprintln!(tab, "filename \"{}\"", ipdata.get_filename_string());
            tabprintln!(tab, "version  {}", meta.version);
            let ispfs = (meta.op_flags & hammer2fs::HAMMER2_OPFLAG_PFSROOT) != 0
                || meta.pfs_type == hammer2fs::HAMMER2_PFSTYPE_SUPROOT;
            if ispfs {
                tabprintln!(
                    tab,
                    "pfs_st   {} ({})",
                    meta.pfs_subtype,
                    subs::get_pfs_subtype_string(meta.pfs_subtype)
                );
            }
            tabprintln!(tab, "uflags   {:#010x}", meta.uflags);
            if meta.rmajor != 0 || meta.rminor != 0 {
                tabprintln!(tab, "rmajor   {}", meta.rmajor);
                tabprintln!(tab, "rminor   {}", meta.rminor);
            }
            tabprintln!(tab, "ctime    {}", subs::get_local_time_string(meta.ctime));
            tabprintln!(tab, "mtime    {}", subs::get_local_time_string(meta.mtime));
            tabprintln!(tab, "atime    {}", subs::get_local_time_string(meta.atime));
            tabprintln!(tab, "btime    {}", subs::get_local_time_string(meta.btime));
            tabprintln!(
                tab,
                "uid      {}",
                subs::get_uuid_string_from_bytes(&meta.uid)
            );
            tabprintln!(
                tab,
                "gid      {}",
                subs::get_uuid_string_from_bytes(&meta.gid)
            );
            tabprintln!(tab, "type     {}", subs::get_inode_type_string(meta.typ));
            tabprintln!(tab, "opflgs   {:#04x}", meta.op_flags);
            tabprintln!(tab, "capflgs  {:#06x}", meta.cap_flags);
            tabprintln!(tab, "mode     {:<7o}", meta.mode);
            tabprintln!(tab, "inum     {:#018x}", meta.inum);
            tabprint!(tab, "size     {} ", meta.size);
            if (meta.op_flags & hammer2fs::HAMMER2_OPFLAG_DIRECTDATA) != 0
                && meta.size <= hammer2fs::HAMMER2_EMBEDDED_BYTES
            {
                println!("(embedded data)");
            } else {
                println!();
            }
            tabprintln!(tab, "nlinks   {}", meta.nlinks);
            tabprintln!(tab, "iparent  {:#018x}", meta.iparent);
            tabprintln!(tab, "name_key {:#018x}", meta.name_key);
            tabprintln!(tab, "name_len {}", meta.name_len);
            tabprintln!(tab, "ncopies  {}", meta.ncopies);
            tabprintln!(
                tab,
                "compalg  {}",
                subs::get_comp_mode_string(meta.comp_algo)
            );
            tabprintln!(
                tab,
                "checkalg {}",
                subs::get_check_mode_string(meta.check_algo)
            );
            if ispfs {
                tabprintln!(tab, "pfs_nmas {}", meta.pfs_nmasters);
                tabprintln!(
                    tab,
                    "pfs_type {} ({})",
                    meta.pfs_type,
                    subs::get_pfs_type_string(meta.pfs_type)
                );
                tabprintln!(tab, "pfs_inum {:#018x}", meta.pfs_inum);
                tabprintln!(
                    tab,
                    "pfs_clid {}",
                    subs::get_uuid_string_from_bytes(&meta.pfs_clid)
                );
                tabprintln!(
                    tab,
                    "pfs_fsid {}",
                    subs::get_uuid_string_from_bytes(&meta.pfs_fsid)
                );
                tabprintln!(tab, "pfs_lsnap_tid {:#018x}", meta.pfs_lsnap_tid);
            }
            tabprintln!(tab, "data_quota  {}", meta.data_quota);
            tabprintln!(tab, "data_count  {}", bref.embed_as_stats().data_count);
            tabprintln!(tab, "inode_quota {}", meta.inode_quota);
            tabprintln!(tab, "inode_count {}", bref.embed_as_stats().inode_count);
            true
        }
        hammer2fs::HAMMER2_BREF_TYPE_INDIRECT => {
            println!("{{");
            true
        }
        hammer2fs::HAMMER2_BREF_TYPE_DIRENT => {
            println!("{{");
            let dirent = bref.embed_as_dirent();
            let namelen = usize::from(dirent.namlen);
            tabprintln!(
                tab,
                "filename \"{}\"",
                if namelen <= bref.check.len() {
                    std::str::from_utf8(&bref.check[..namelen])
                } else {
                    std::str::from_utf8(&media[..namelen])
                }
                .unwrap()
            );
            tabprintln!(tab, "inum {:#018x}", dirent.inum);
            tabprintln!(tab, "nlen {}", dirent.namlen);
            tabprintln!(tab, "type {}", subs::get_inode_type_string(dirent.typ));
            true
        }
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_NODE | hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF => {
            println!("{{");
            let mut tmp = bref.data_off & !hammer2fs::HAMMER2_OFF_MASK_RADIX;
            tmp &= hammer2fs::HAMMER2_SEGMASK;
            tmp /= hammer2fs::HAMMER2_PBUFSIZE;
            let mut tmp = usize::try_from(tmp).unwrap();
            assert!(tmp >= hammer2fs::HAMMER2_ZONE_FREEMAP_00);
            assert!(tmp < hammer2fs::HAMMER2_ZONE_FREEMAP_END);
            tmp -= hammer2fs::HAMMER2_ZONE_FREEMAP_00;
            tmp /= hammer2fs::HAMMER2_ZONE_FREEMAP_INC;
            tabprintln!(tab, "rotation={}", tmp);
            if bref.typ == hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF {
                let bmdata = get_from_media::<hammer2fs::Hammer2BmapData>(media);
                for i in 0..hammer2fs::HAMMER2_FREEMAP_COUNT {
                    let bmdata = &bmdata[i];
                    let data_off = bref.key
                        + u64::try_from(i).unwrap() * hammer2fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                    tabprintln!(
                        tab + 4,
                        "{data_off:016x} {i:04}.{:04x} linear={:06x} avail={:06x} \
                        {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                        bmdata.class,
                        bmdata.linear,
                        bmdata.avail,
                        bmdata.bitmapq[0],
                        bmdata.bitmapq[1],
                        bmdata.bitmapq[2],
                        bmdata.bitmapq[3],
                        bmdata.bitmapq[4],
                        bmdata.bitmapq[5],
                        bmdata.bitmapq[6],
                        bmdata.bitmapq[7]
                    );
                }
            }
            tabprintln!(tab, "}}");
            true
        }
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP | hammer2fs::HAMMER2_BREF_TYPE_VOLUME => {
            let voldata = util::align_to::<hammer2fs::Hammer2VolumeData>(media);
            print!(
                "mirror_tid={:016x} freemap_tid={:016x} ",
                voldata.mirror_tid, voldata.freemap_tid
            );
            println!("{{");
            true
        }
        _ => {
            println!();
            false
        }
    }
}

fn count_blocks(bmap: &hammer2fs::Hammer2BmapData, value: usize, stat: &mut FreemapStat) {
    let bits = std::mem::size_of::<u64>() * 8;
    assert_eq!(bits, 64);
    let value16 = u64::try_from(value).unwrap();
    assert!(value16 < 4);
    let value64 = value16 << 6 | value16 << 4 | value16 << 2 | value16;
    assert!(value64 < 256);

    for i in 0..hammer2fs::HAMMER2_BMAP_ELEMENTS {
        let mask = 0x03; // 2 bits per 16KB
        let mut bm = bmap.bitmapq[i];
        let mut j = 0;
        while j < bits {
            if (bm & mask) == value16 {
                stat.accum16[value] += 16384;
            }
            bm >>= 2;
            j += 2;
        }
        let mask = 0xff; // 8 bits per 64KB chunk
        let mut bm = bmap.bitmapq[i];
        let mut j = 0;
        while j < bits {
            if (bm & mask) == value64 {
                stat.accum64[value] += 65536;
            }
            bm >>= 8;
            j += 8;
        }
    }
}
