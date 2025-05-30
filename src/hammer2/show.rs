const TAB_INDENT: usize = 1;

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

#[allow(clippy::too_many_lines)]
pub(crate) fn show_volume_data(
    fso: &mut libhammer2::ondisk::Ondisk,
    voldata: &libhammer2::fs::Hammer2VolumeData,
    bi: usize,
    sopt: &ShowOptions,
    opt: &crate::Opt,
) -> hammer2_utils::Result<()> {
    println!();
    println!("Volume {} header {bi} {{", voldata.volu_id);
    println!("    magic          {:#018x}", voldata.magic);
    println!("    boot_beg       {:#018x}", voldata.boot_beg);
    println!(
        "    boot_end       {:#018x} ({:6.2}MB)",
        voldata.boot_end,
        (voldata.boot_end - voldata.boot_beg) as f64 / libhammer2::subs::M_F64
    );
    println!("    aux_beg        {:#018x}", voldata.aux_beg);
    println!(
        "    aux_end        {:#018x} ({:6.2}MB)",
        voldata.aux_end,
        (voldata.aux_end - voldata.aux_beg) as f64 / libhammer2::subs::M_F64
    );
    println!(
        "    volu_size      {:#018x} ({:6.2}GB)",
        voldata.volu_size,
        voldata.volu_size as f64 / libhammer2::subs::G_F64
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
        libhammer2::subs::get_uuid_string_from_bytes(&voldata.fsid)
    );
    let s = libhammer2::subs::get_uuid_string_from_bytes(&voldata.fstype);
    println!("    fstype         {s}");
    let name = if s == libhammer2::fs::HAMMER2_UUID_STRING {
        "DragonFly HAMMER2"
    } else {
        "?"
    };
    println!("                   ({name})");

    println!(
        "    allocator_size {:#018x} ({:6.2}GB)",
        voldata.allocator_size,
        voldata.allocator_size as f64 / libhammer2::subs::G_F64
    );
    println!(
        "    allocator_free {:#018x} ({:6.2}GB)",
        voldata.allocator_free,
        voldata.allocator_free as f64 / libhammer2::subs::G_F64
    );
    println!(
        "    allocator_beg  {:#018x} ({:6.2}GB)",
        voldata.allocator_beg,
        voldata.allocator_beg as f64 / libhammer2::subs::G_F64
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
            libhammer2::fs::HAMMER2_VOL_ICRC_SECT0 => {
                let a = voldata.get_crc(
                    libhammer2::fs::HAMMER2_VOLUME_ICRC0_OFF,
                    libhammer2::fs::HAMMER2_VOLUME_ICRC0_SIZE,
                );
                let b = voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT0];
                print!("{a:#010x}/{b:#010x}");
                if a == b {
                    print!(" (OK)");
                } else {
                    print!(" (FAILED)");
                }
            }
            libhammer2::fs::HAMMER2_VOL_ICRC_SECT1 => {
                let a = voldata.get_crc(
                    libhammer2::fs::HAMMER2_VOLUME_ICRC1_OFF,
                    libhammer2::fs::HAMMER2_VOLUME_ICRC1_SIZE,
                );
                let b = voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT1];
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
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_SIZE,
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
    for i in 0..libhammer2::fs::HAMMER2_SET_COUNT {
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
    for i in 0..libhammer2::fs::HAMMER2_SET_COUNT {
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
#[allow(clippy::too_many_lines)]
pub(crate) fn show_blockref(
    fso: &mut libhammer2::ondisk::Ondisk,
    voldata: &libhammer2::fs::Hammer2VolumeData,
    tab: usize,
    bi: usize,
    bref: &libhammer2::fs::Hammer2Blockref,
    norecurse: bool,
    stat: &mut Option<FreemapStat>,
    sopt: &ShowOptions,
    opt: &crate::Opt,
) -> hammer2_utils::Result<()> {
    // omit if smaller than mininum mirror_tid threshold
    if bref.mirror_tid < sopt.min_mirror_tid {
        return Ok(());
    }
    // omit if smaller than mininum modify_tid threshold
    if bref.modify_tid < sopt.min_modify_tid
        && (bref.modify_tid != 0
            || (bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_INODE && bref.leaf_count == 0))
    {
        return Ok(());
    }

    // hammer2(8) checks I/O bytes vs media size unconditionally
    let media = if bref.typ != libhammer2::fs::HAMMER2_BREF_TYPE_DATA || opt.verbose {
        fso.read_media(bref)?
    } else {
        vec![]
    };
    let type_str = libhammer2::subs::get_blockref_type_string(bref.typ);
    let type_pad = if type_str.len() > 8 {
        0
    } else {
        8 - type_str.len()
    };

    let bscan = if media.is_empty() {
        vec![]
    } else {
        libhammer2::ondisk::media_as_blockref_safe(bref, &media)
    };
    let id = fso
        .get_volume(bref.data_off)
        .ok_or(nix::errno::Errno::ENODEV)?
        .get_id();
    if opt.quiet {
        hammer2_utils::tab::print!(
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
        hammer2_utils::tab::println!(
            tab,
            "{type_str}.{bi:<3}{} {:016x} {:016x}/{:<2} ",
            " ".repeat(type_pad),
            bref.data_off,
            bref.key,
            bref.keybits
        );
        hammer2_utils::tab::print!(
            tab + 13,
            "vol={id} mir={:016x} mod={:016x} lfcnt={} ",
            bref.mirror_tid,
            bref.modify_tid,
            bref.leaf_count
        );
        if !bscan.is_empty()
            || bref.flags != 0
            || bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE
            || bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF
        {
            println!();
            hammer2_utils::tab::print!(tab + 13, "");
        }
    }
    if !bscan.is_empty() {
        print!("bcnt={} ", bscan.len());
    }
    if bref.flags != 0 {
        print!("flags={:02x} ", bref.flags);
    }
    if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE
        || bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF
    {
        let freemap = bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckFreemap>();
        print!("bigmask={:08x} avail={} ", freemap.bigmask, freemap.avail);
    }

    // Check data integrity in verbose mode, otherwise we are just doing
    // a quick meta-data scan.  Meta-data integrity is always checked.
    // (Also see the check above that ensures the media data is loaded,
    // otherwise there's no data to check!).
    //
    // WARNING! bref->check state may be used for other things when
    // bref has no data (bytes == 0).
    let radix = bref.get_radix();
    let bytes = if radix == 0 { 0 } else { 1 << radix };
    let mut failed = false;
    if bytes > 0 && (bref.typ != libhammer2::fs::HAMMER2_BREF_TYPE_DATA || opt.verbose) {
        if !opt.quiet {
            println!();
            hammer2_utils::tab::print!(tab + 13, "");
        }
        let check_algo = libhammer2::fs::dec_check(bref.methods);
        let check_str = libhammer2::subs::get_check_mode_string(check_algo);
        let comp_algo = libhammer2::fs::dec_comp(bref.methods);
        let comp_str = libhammer2::subs::get_comp_mode_string(comp_algo);
        let meth = format!("{check_str}|{comp_str}");
        match check_algo {
            libhammer2::fs::HAMMER2_CHECK_NONE | libhammer2::fs::HAMMER2_CHECK_DISABLED => {
                print!("meth={meth} ");
            }
            libhammer2::fs::HAMMER2_CHECK_ISCSI32 => {
                let cv = icrc32::iscsi_crc32(&media);
                let iscsi32 = bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckIscsi>();
                if iscsi32.value == cv {
                    print!("meth={meth} icrc={cv:08x} ");
                } else {
                    print!("(icrc {meth} {:08x}/{cv:08x} failed) ", iscsi32.value);
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_XXHASH64 => {
                let cv = libhammer2::xxhash::xxh64(&media);
                let xxhash64 = bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckXxhash64>();
                if xxhash64.value == cv {
                    print!("meth={meth} xxh={cv:016x} ");
                } else {
                    print!("(xxh {meth} {:016x}/{cv:016x} failed) ", xxhash64.value);
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_SHA192 => {
                let cv = libhammer2::sha::sha256(&media);
                if bref
                    .check_as::<libhammer2::fs::Hammer2BlockrefCheckSha256>()
                    .data
                    == cv.as_slice()
                {
                    print!("meth={meth} ");
                } else {
                    print!("(sha192 {meth} failed) ");
                    failed = true;
                }
            }
            libhammer2::fs::HAMMER2_CHECK_FREEMAP => {
                let cv = icrc32::iscsi_crc32(&media);
                let freemap = bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckFreemap>();
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
        show_blockref_data(&media, tab, bref, norecurse)?
    };

    // Update statistics.
    if let Some(ref mut stat) = stat {
        if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF {
            let bmdata = libhammer2::fs::media_as(&media);
            for i in 0..libhammer2::fs::HAMMER2_FREEMAP_COUNT {
                let bmdata = &bmdata[i];
                let data_off =
                    bref.key + u64::try_from(i)? * libhammer2::fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                if data_off >= voldata.aux_end && data_off < fso.get_total_size() {
                    for j in 0..4 {
                        count_blocks(bmdata, j, stat)?;
                    }
                } else {
                    stat.unavail += libhammer2::fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                }
            }
            stat.freemap += libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE;
        }
    }

    // Recurse if norecurse == 0.  If the CRC failed, pass norecurse = 1.
    // That is, if an indirect or inode fails we still try to list its
    // direct children to help with debugging, but go no further than
    // that because they are probably garbage.
    if (sopt.depth == usize::MAX || ((tab - sopt.init_tab) / sopt.tab) < sopt.depth) && !norecurse {
        for (i, bref) in bscan.iter().enumerate() {
            if bref.typ != libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY {
                show_blockref(fso, voldata, tab, i, bref, failed, stat, sopt, opt)?;
            }
        }
    }
    let tab = tab - sopt.tab;
    if obrace {
        if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_INODE {
            let ipdata = libhammer2::ondisk::media_as_inode_data(&media);
            hammer2_utils::tab::println!(
                tab,
                "}} ({}.{}, \"{}\")",
                type_str,
                bi,
                ipdata.get_filename_string()?
            );
        } else {
            hammer2_utils::tab::println!(tab, "}} ({}.{})", type_str, bi);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn show_blockref_data(
    media: &[u8],
    tab: usize,
    bref: &libhammer2::fs::Hammer2Blockref,
    norecurse: bool,
) -> hammer2_utils::Result<bool> {
    match bref.typ {
        libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY => {
            if norecurse {
                println!();
            }
            Ok(false)
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
            println!("{{");
            let ipdata = libhammer2::ondisk::media_as_inode_data(media);
            let meta = &ipdata.meta;
            hammer2_utils::tab::println!(tab, "filename \"{}\"", ipdata.get_filename_string()?);
            hammer2_utils::tab::println!(tab, "version  {}", meta.version);
            if meta.is_root() {
                hammer2_utils::tab::println!(
                    tab,
                    "pfs_st   {} ({})",
                    meta.pfs_subtype,
                    libhammer2::subs::get_pfs_subtype_string(meta.pfs_subtype)
                );
            }
            hammer2_utils::tab::println!(tab, "uflags   {:#010x}", meta.uflags);
            if meta.rmajor != 0 || meta.rminor != 0 {
                hammer2_utils::tab::println!(tab, "rmajor   {}", meta.rmajor);
                hammer2_utils::tab::println!(tab, "rminor   {}", meta.rminor);
            }
            hammer2_utils::tab::println!(
                tab,
                "ctime    {}",
                libhammer2::subs::get_local_time_string(meta.ctime)
            );
            hammer2_utils::tab::println!(
                tab,
                "mtime    {}",
                libhammer2::subs::get_local_time_string(meta.mtime)
            );
            hammer2_utils::tab::println!(
                tab,
                "atime    {}",
                libhammer2::subs::get_local_time_string(meta.atime)
            );
            hammer2_utils::tab::println!(
                tab,
                "btime    {}",
                libhammer2::subs::get_local_time_string(meta.btime)
            );
            hammer2_utils::tab::println!(
                tab,
                "uid      {}",
                libhammer2::subs::get_uuid_string_from_bytes(&meta.uid)
            );
            hammer2_utils::tab::println!(
                tab,
                "gid      {}",
                libhammer2::subs::get_uuid_string_from_bytes(&meta.gid)
            );
            hammer2_utils::tab::println!(
                tab,
                "type     {}",
                libhammer2::subs::get_inode_type_string(meta.typ)
            );
            hammer2_utils::tab::println!(tab, "opflgs   {:#04x}", meta.op_flags);
            hammer2_utils::tab::println!(tab, "capflgs  {:#06x}", meta.cap_flags);
            hammer2_utils::tab::println!(tab, "mode     {:<7o}", meta.mode);
            hammer2_utils::tab::println!(tab, "inum     {:#018x}", meta.inum);
            hammer2_utils::tab::print!(tab, "size     {} ", meta.size);
            if meta.has_direct_data() && meta.size <= libhammer2::fs::HAMMER2_EMBEDDED_BYTES {
                println!("(embedded data)");
            } else {
                println!();
            }
            hammer2_utils::tab::println!(tab, "nlinks   {}", meta.nlinks);
            hammer2_utils::tab::println!(tab, "iparent  {:#018x}", meta.iparent);
            hammer2_utils::tab::println!(tab, "name_key {:#018x}", meta.name_key);
            hammer2_utils::tab::println!(tab, "name_len {}", meta.name_len);
            hammer2_utils::tab::println!(tab, "ncopies  {}", meta.ncopies);
            hammer2_utils::tab::println!(
                tab,
                "compalg  {}",
                libhammer2::subs::get_comp_mode_string(meta.comp_algo)
            );
            hammer2_utils::tab::println!(
                tab,
                "checkalg {}",
                libhammer2::subs::get_check_mode_string(meta.check_algo)
            );
            if meta.is_root() {
                hammer2_utils::tab::println!(tab, "pfs_nmas {}", meta.pfs_nmasters);
                hammer2_utils::tab::println!(
                    tab,
                    "pfs_type {} ({})",
                    meta.pfs_type,
                    libhammer2::subs::get_pfs_type_string(meta.pfs_type)
                );
                hammer2_utils::tab::println!(tab, "pfs_inum {:#018x}", meta.pfs_inum);
                hammer2_utils::tab::println!(
                    tab,
                    "pfs_clid {}",
                    libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_clid)
                );
                hammer2_utils::tab::println!(
                    tab,
                    "pfs_fsid {}",
                    libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_fsid)
                );
                hammer2_utils::tab::println!(tab, "pfs_lsnap_tid {:#018x}", meta.pfs_lsnap_tid);
            }
            hammer2_utils::tab::println!(tab, "data_quota  {}", meta.data_quota);
            hammer2_utils::tab::println!(
                tab,
                "data_count  {}",
                bref.embed_as::<libhammer2::fs::Hammer2BlockrefEmbedStats>()
                    .data_count
            );
            hammer2_utils::tab::println!(tab, "inode_quota {}", meta.inode_quota);
            hammer2_utils::tab::println!(
                tab,
                "inode_count {}",
                bref.embed_as::<libhammer2::fs::Hammer2BlockrefEmbedStats>()
                    .inode_count
            );
            Ok(true)
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT => {
            println!("{{");
            Ok(true)
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_DIRENT => {
            println!("{{");
            let dirent = bref.embed_as::<libhammer2::fs::Hammer2DirentHead>();
            let namelen = usize::from(dirent.namlen);
            hammer2_utils::tab::println!(
                tab,
                "filename \"{}\"",
                if namelen <= bref.check.len() {
                    std::str::from_utf8(&bref.check[..namelen])
                } else {
                    std::str::from_utf8(&media[..namelen])
                }?
            );
            hammer2_utils::tab::println!(tab, "inum {:#018x}", dirent.inum);
            hammer2_utils::tab::println!(tab, "nlen {}", dirent.namlen);
            hammer2_utils::tab::println!(
                tab,
                "type {}",
                libhammer2::subs::get_inode_type_string(dirent.typ)
            );
            Ok(true)
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_NODE
        | libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF => {
            println!("{{");
            let mut tmp = bref.get_raw_data_off();
            tmp &= libhammer2::fs::HAMMER2_SEGMASK;
            tmp /= libhammer2::fs::HAMMER2_PBUFSIZE;
            let mut tmp = usize::try_from(tmp)?;
            assert!(tmp >= libhammer2::fs::HAMMER2_ZONE_FREEMAP_00);
            assert!(tmp < libhammer2::fs::HAMMER2_ZONE_FREEMAP_END);
            tmp -= libhammer2::fs::HAMMER2_ZONE_FREEMAP_00;
            tmp /= libhammer2::fs::HAMMER2_ZONE_FREEMAP_INC;
            hammer2_utils::tab::println!(tab, "rotation={}", tmp);
            if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF {
                let bmdata = libhammer2::fs::media_as::<libhammer2::fs::Hammer2BmapData>(media);
                for i in 0..libhammer2::fs::HAMMER2_FREEMAP_COUNT {
                    let bmdata = &bmdata[i];
                    let data_off =
                        bref.key + u64::try_from(i)? * libhammer2::fs::HAMMER2_FREEMAP_LEVEL0_SIZE;
                    hammer2_utils::tab::println!(
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
            hammer2_utils::tab::println!(tab, "}}");
            Ok(true)
        }
        libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP | libhammer2::fs::HAMMER2_BREF_TYPE_VOLUME => {
            let voldata = libhammer2::ondisk::media_as_volume_data(media);
            print!(
                "mirror_tid={:016x} freemap_tid={:016x} ",
                voldata.mirror_tid, voldata.freemap_tid
            );
            println!("{{");
            Ok(true)
        }
        _ => {
            println!();
            Ok(false)
        }
    }
}

fn count_blocks(
    bmap: &libhammer2::fs::Hammer2BmapData,
    value: usize,
    stat: &mut FreemapStat,
) -> hammer2_utils::Result<()> {
    let bits = std::mem::size_of::<u64>() * 8;
    assert_eq!(bits, 64);
    let value16 = u64::try_from(value)?;
    assert!(value16 < 4);
    let value64 = (value16 << 6) | (value16 << 4) | (value16 << 2) | value16;
    assert!(value64 < 256);

    for i in 0..libhammer2::fs::HAMMER2_BMAP_ELEMENTS {
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
    Ok(())
}
