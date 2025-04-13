#[derive(Debug)]
pub(crate) enum Label {
    #[allow(dead_code)]
    Boot,
    #[allow(dead_code)]
    Root,
    Data,
}

pub(crate) const MAXLABELS: usize = libhammer2::fs::HAMMER2_SET_COUNT;

fn get_hammer2_version() -> u32 {
    let mut version = libhammer2::fs::HAMMER2_VOL_VERSION_DEFAULT;
    let mut version_size = std::mem::size_of_val(&version);
    if unsafe {
        libhammer2::os::sysctlbyname(
            c"vfs.hammer2.supported_version".as_ptr(),
            std::ptr::from_mut::<u32>(&mut version).cast::<libc::c_void>(),
            std::ptr::from_mut::<libc::size_t>(&mut version_size),
            std::ptr::null_mut(),
            0,
        )
    } == 0
    {
        if version >= libhammer2::fs::HAMMER2_VOL_VERSION_WIP {
            version = libhammer2::fs::HAMMER2_VOL_VERSION_WIP - 1;
            log::warn!(
                "HAMMER2 VFS supports higher version than I understand.\n\
                Using default version {version}"
            );
        }
    } else {
        log::warn!(
            "HAMMER2 VFS not loaded, cannot get version info.\n\
            Using default version {version}"
        );
    }
    version
}

#[derive(Debug, Default)]
pub(crate) struct Opt {
    pub(crate) hammer2_version: u32,
    pub(crate) fstype: uuid::Uuid,
    pub(crate) volfsid: uuid::Uuid,
    pub(crate) supclid: uuid::Uuid,
    pub(crate) supfsid: uuid::Uuid,
    pub(crate) pfsclid: Vec<uuid::Uuid>,
    pub(crate) pfsfsid: Vec<uuid::Uuid>,
    pub(crate) boot_area_size: u64,
    pub(crate) aux_area_size: u64,
    pub(crate) fs_size: Vec<u64>,
    pub(crate) label: Vec<String>,
    pub(crate) comp_type: u8,
    pub(crate) check_type: u8,
    pub(crate) default_label_type: Option<Label>,
    pub(crate) debug: bool,
}

impl Opt {
    pub(crate) fn new() -> Self {
        Self {
            hammer2_version: get_hammer2_version(),
            label: vec!["LOCAL".to_string()],
            comp_type: libhammer2::fs::HAMMER2_COMP_DEFAULT, // default LZ4
            check_type: libhammer2::fs::HAMMER2_CHECK_DEFAULT, // default xxhash64
            default_label_type: None,
            volfsid: uuid::Uuid::new_v4(),
            supclid: uuid::Uuid::new_v4(),
            supfsid: uuid::Uuid::new_v4(),
            fstype: libhammer2::subs::get_uuid_from_str(libhammer2::fs::HAMMER2_UUID_STRING)
                .unwrap(),
            ..Default::default()
        }
    }

    pub(crate) fn parse_fs_size(&mut self, arg: &str) -> nix::Result<()> {
        for s in &arg.split(':').collect::<Vec<&str>>() {
            // XXX 0x7fffffffffffffff isn't limitation of HAMMER2
            self.fs_size.push(get_size(
                s,
                libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE,
                0x7fff_ffff_ffff_ffff,
                2,
            )?);
            if self.fs_size.len() >= libhammer2::fs::HAMMER2_MAX_VOLUMES.into() {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn adjust(&mut self, total_size: u64) {
        // Adjust Label[].
        match self.default_label_type {
            Some(Label::Boot) => self.label.push("BOOT".to_string()),
            Some(Label::Root) => self.label.push("ROOT".to_string()),
            Some(Label::Data) => self.label.push("DATA".to_string()),
            None => (),
        }

        // Calculate defaults for the boot area size and round to the
        // volume alignment boundary.
        //
        // Note: These areas are currently not used for booting but are
        // reserved for future filesystem expansion.
        let mut x = self.boot_area_size;
        if x == 0 {
            x = libhammer2::fs::HAMMER2_BOOT_NOM_BYTES;
            while x > total_size / 20 {
                x >>= 1;
            }
            if x < libhammer2::fs::HAMMER2_BOOT_MIN_BYTES {
                x = libhammer2::fs::HAMMER2_BOOT_MIN_BYTES;
            }
        } else if x < libhammer2::fs::HAMMER2_BOOT_MIN_BYTES {
            x = libhammer2::fs::HAMMER2_BOOT_MIN_BYTES;
        }
        self.boot_area_size = (x + libhammer2::fs::HAMMER2_VOLUME_ALIGNMASK)
            & !libhammer2::fs::HAMMER2_VOLUME_ALIGNMASK;

        // Calculate defaults for the aux area size and round to the
        // volume alignment boundary.
        //
        // Note: These areas are currently not used for logging but are
        // reserved for future filesystem expansion.
        let mut x = self.aux_area_size;
        if x == 0 {
            x = libhammer2::fs::HAMMER2_AUX_NOM_BYTES;
            while x > total_size / 20 {
                x >>= 1;
            }
            if x < libhammer2::fs::HAMMER2_AUX_MIN_BYTES {
                x = libhammer2::fs::HAMMER2_AUX_MIN_BYTES;
            }
        } else if x < libhammer2::fs::HAMMER2_AUX_MIN_BYTES {
            x = libhammer2::fs::HAMMER2_AUX_MIN_BYTES;
        }
        self.aux_area_size = (x + libhammer2::fs::HAMMER2_VOLUME_ALIGNMASK)
            & !libhammer2::fs::HAMMER2_VOLUME_ALIGNMASK;
    }
}

// Convert a string to a 64 bit signed integer with various requirements.
pub(crate) fn get_size(s: &str, minval: u64, maxval: u64, powerof2: i32) -> nix::Result<u64> {
    let (a, b) = s.split_at(s.len() - 1);
    let mut val = match a.parse::<u64>() {
        Ok(v) => v,
        Err(e) => {
            log::error!("{s}: {e}");
            return Err(nix::errno::Errno::EINVAL);
        }
    };
    match b {
        "t" | "T" => val <<= 40,
        "g" | "G" => val <<= 30,
        "m" | "M" => val <<= 20,
        "k" | "K" => val <<= 10,
        _ => {
            log::error!("Unknown suffix in number '{s}'");
            return Err(nix::errno::Errno::EINVAL);
        }
    }

    if val < minval {
        log::error!(
            "Value too small: {s}, min is {}",
            libhammer2::subs::get_size_string(minval)
        );
        return Err(nix::errno::Errno::EINVAL);
    }
    if val > maxval {
        log::error!(
            "Value too large: {s}, min is {}",
            libhammer2::subs::get_size_string(maxval)
        );
        return Err(nix::errno::Errno::EINVAL);
    }
    if (powerof2 & 1) != 0 && (val ^ (val - 1)) != ((val << 1) - 1) {
        log::error!("Value not power of 2: {s}");
        return Err(nix::errno::Errno::EINVAL);
    }
    if (powerof2 & 2) != 0 && (val & libhammer2::fs::HAMMER2_NEWFS_ALIGNMASK) != 0 {
        log::error!(
            "Value not an integral multiple of {}K: {s}",
            libhammer2::fs::HAMMER2_NEWFS_ALIGN / 1024
        );
        return Err(nix::errno::Errno::EINVAL);
    }
    Ok(val)
}

fn get_current_time() -> Result<u64, std::time::SystemTimeError> {
    Ok(libhammer2::util::get_current_time()? * 1_000_000)
}

fn get_buffer() -> hammer2_utils::Result<Vec<u8>> {
    Ok(vec![0; libhammer2::fs::HAMMER2_PBUFSIZE.try_into()?])
}

fn format_misc(
    vol: &mut libhammer2::volume::Volume,
    opt: &Opt,
    boot_base: u64,
    aux_base: u64,
) -> hammer2_utils::Result<u64> {
    // Clear the entire 4MB reserve for the first 2G zone.
    let mut tmp_base = 0;
    for _ in 0..libhammer2::fs::HAMMER2_ZONE_BLOCKS_SEG {
        vol.pwrite(&get_buffer()?, tmp_base)?;
        tmp_base += libhammer2::fs::HAMMER2_PBUFSIZE;
    }

    // Make sure alloc_base won't cross the reserved area at the
    // beginning of each 1GB.
    //
    // Reserve space for the super-root inode and the root inode.
    // Make sure they are in the same 64K block to simplify our code.
    let alloc_base = aux_base + opt.aux_area_size;
    assert_eq!(alloc_base & libhammer2::fs::HAMMER2_PBUFMASK, 0);
    assert!(alloc_base < libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE);

    // Clear the boot/aux area.
    let mut tmp_base = boot_base;
    while tmp_base < alloc_base {
        vol.pwrite(&get_buffer()?, tmp_base)?;
        tmp_base += libhammer2::fs::HAMMER2_PBUFSIZE;
    }
    Ok(alloc_base)
}

#[allow(clippy::too_many_lines)]
fn format_inode(
    vol: &mut libhammer2::volume::Volume,
    opt: &mut Opt,
    alloc_base: u64,
) -> hammer2_utils::Result<(u64, libhammer2::fs::Hammer2Blockref)> {
    let now = get_current_time()?;

    let mut buf = get_buffer()?;
    let mut root_blockref = vec![];
    let mut alloc_base = alloc_base;
    alloc_base &= !libhammer2::fs::HAMMER2_PBUFMASK;

    let t = alloc_direct(alloc_base, libhammer2::fs::HAMMER2_INODE_BYTES)?;
    alloc_base = t.0;
    let mut sroot_blockref = t.1;

    for s in &opt.label {
        let t = alloc_direct(alloc_base, libhammer2::fs::HAMMER2_INODE_BYTES)?;
        alloc_base = t.0;
        let mut bref = t.1;
        assert_eq!(
            (sroot_blockref.data_off ^ bref.data_off) & !libhammer2::fs::HAMMER2_PBUFMASK,
            0
        );

        let mut rawip = libhammer2::fs::Hammer2InodeData::new();
        rawip.meta.version = libhammer2::fs::HAMMER2_INODE_VERSION_ONE;
        rawip.meta.ctime = now;
        rawip.meta.mtime = now;
        // rawip.meta.atime = now; NOT IMPL MUST BE ZERO
        rawip.meta.btime = now;
        rawip.meta.typ = libhammer2::fs::HAMMER2_OBJTYPE_DIRECTORY;
        rawip.meta.mode = 0o755;
        rawip.meta.inum = 1; // root inode, inumber 1
        rawip.meta.nlinks = 1; // directory link count compat

        rawip.meta.name_len = s.len().try_into()?;
        rawip.filename[..s.len()].copy_from_slice(s.as_bytes());
        rawip.meta.name_key = libhammer2::subs::dirhash(&rawip.filename[..s.len()]);

        // Compression mode and supported copyids.
        //
        // Do not allow compression when creating any "BOOT" label
        // (pfs-create also does the same if the pfs is named "BOOT")
        if s.to_uppercase() == "BOOT" {
            rawip.meta.comp_algo = libhammer2::fs::enc_algo(libhammer2::fs::HAMMER2_COMP_AUTOZERO);
            rawip.meta.check_algo =
                libhammer2::fs::enc_algo(libhammer2::fs::HAMMER2_CHECK_XXHASH64);
        } else {
            rawip.meta.comp_algo = libhammer2::fs::enc_algo(opt.comp_type);
            rawip.meta.check_algo = libhammer2::fs::enc_algo(opt.check_type);
        }

        // Note: We leave nmasters set to 0, which means that we
        // don't know how many masters there are.  The quorum
        // calculation will effectively be 1 ( 0 / 2 + 1 ).
        let pfs_clid = uuid::Uuid::new_v4();
        let pfs_fsid = uuid::Uuid::new_v4();
        rawip
            .meta
            .pfs_clid
            .copy_from_slice(libhammer2::util::any_as_u8_slice(&pfs_clid));
        rawip
            .meta
            .pfs_fsid
            .copy_from_slice(libhammer2::util::any_as_u8_slice(&pfs_fsid));
        opt.pfsclid.push(pfs_clid);
        opt.pfsfsid.push(pfs_fsid);
        rawip.meta.pfs_type = libhammer2::fs::HAMMER2_PFSTYPE_MASTER;
        rawip.meta.op_flags |= libhammer2::fs::HAMMER2_OPFLAG_PFSROOT;

        // first allocatable inode number
        rawip.meta.pfs_inum = 16;
        // rawip.u.blockset is left empty

        // The root blockref will be stored in the super-root inode as
        // one of the ~4 PFS root directories.  The copyid here is the
        // actual copyid of the storage ref.
        //
        // The key field for a PFS root directory's blockref is
        // essentially the name key for the entry.
        bref.key = rawip.meta.name_key;
        bref.copyid = libhammer2::fs::HAMMER2_COPYID_LOCAL;
        bref.keybits = 0;
        bref.check_as_mut::<libhammer2::fs::Hammer2BlockrefCheckXxhash64>()
            .value = libhammer2::xxhash::xxh64(libhammer2::util::any_as_u8_slice(&rawip));
        bref.typ = libhammer2::fs::HAMMER2_BREF_TYPE_INODE;
        bref.methods = libhammer2::fs::enc_check(libhammer2::fs::HAMMER2_CHECK_XXHASH64)
            | libhammer2::fs::enc_comp(libhammer2::fs::HAMMER2_COMP_NONE);
        bref.mirror_tid = 16;
        bref.flags = libhammer2::fs::HAMMER2_BREF_FLAG_PFSROOT;

        copy_inode_to_buffer(&mut buf, &bref, &rawip)?;
        root_blockref.push(bref);
    }

    // Format the super-root directory inode, giving it ~4 PFS root
    // directories (root_blockref).
    //
    // The superroot contains ~4 directories pointing at the PFS root
    // inodes (named via the label).  Inodes contain one blockset which
    // is fully associative so we can put the entry anywhere without
    // having to worry about the hash.  Use index 0.
    let mut rawip = libhammer2::fs::Hammer2InodeData::new();
    rawip.meta.version = libhammer2::fs::HAMMER2_INODE_VERSION_ONE;
    rawip.meta.ctime = now;
    rawip.meta.mtime = now;
    // rawip.meta.atime = now; NOT IMPL MUST BE ZERO
    rawip.meta.btime = now;
    rawip.meta.typ = libhammer2::fs::HAMMER2_OBJTYPE_DIRECTORY;
    rawip.meta.mode = 0o700; // super-root - root only
    rawip.meta.inum = 0; // super root inode, inumber 0
    rawip.meta.nlinks = 2; // directory link count compat

    rawip.meta.comp_algo = libhammer2::fs::enc_algo(libhammer2::fs::HAMMER2_COMP_AUTOZERO);
    rawip.meta.check_algo = libhammer2::fs::enc_algo(libhammer2::fs::HAMMER2_CHECK_XXHASH64);

    // The super-root is flagged as a PFS and typically given its own
    // random FSID, making it possible to mirror an entire HAMMER2 disk
    // snapshots and all if desired.  PFS ids are used to match up
    // mirror sources and targets and cluster copy sources and targets.
    //
    // (XXX whole-disk logical mirroring is not really supported in
    //  the first attempt because each PFS is in its own modify/mirror
    //  transaction id domain, so normal mechanics cannot cross a PFS
    //  boundary).
    rawip
        .meta
        .pfs_clid
        .copy_from_slice(libhammer2::util::any_as_u8_slice(&opt.supclid));
    rawip
        .meta
        .pfs_fsid
        .copy_from_slice(libhammer2::util::any_as_u8_slice(&opt.supfsid));
    rawip.meta.pfs_type = libhammer2::fs::HAMMER2_PFSTYPE_SUPROOT;
    let filename = "SUPROOT";
    let name_len = filename.len();
    rawip.filename[..name_len].copy_from_slice(filename.as_bytes());
    rawip.meta.name_key = 0;
    rawip.meta.name_len = name_len.try_into()?;

    // The super-root has an inode number of 0
    rawip.meta.pfs_inum = 0;

    // Currently newfs_hammer2 just throws the PFS inodes into the
    // top-level block table at the volume root and doesn't try to
    // create an indirect block, so we are limited to ~4 at filesystem
    // creation time.  More can be added after mounting.
    let blockset = rawip.u_as_mut::<libhammer2::fs::Hammer2Blockset>();
    root_blockref.sort_by_key(|bref| bref.key);
    for i in 0..root_blockref.len() {
        if i != root_blockref.len() - 1 {
            assert!(root_blockref[i].key <= root_blockref[i + 1].key);
        }
        blockset.blockref[i] = root_blockref[i];
    }

    // The sroot blockref will be stored in the volume header.
    sroot_blockref.copyid = libhammer2::fs::HAMMER2_COPYID_LOCAL;
    sroot_blockref.keybits = 0;
    sroot_blockref
        .check_as_mut::<libhammer2::fs::Hammer2BlockrefCheckXxhash64>()
        .value = libhammer2::xxhash::xxh64(libhammer2::util::any_as_u8_slice(&rawip));
    sroot_blockref.typ = libhammer2::fs::HAMMER2_BREF_TYPE_INODE;
    sroot_blockref.methods = libhammer2::fs::enc_check(libhammer2::fs::HAMMER2_CHECK_XXHASH64)
        | libhammer2::fs::enc_comp(libhammer2::fs::HAMMER2_COMP_AUTOZERO);
    sroot_blockref.mirror_tid = 16;

    copy_inode_to_buffer(&mut buf, &sroot_blockref, &rawip)?;

    // Write out the 64K HAMMER2 block containing the root and sroot.
    assert_eq!(
        sroot_blockref.data_off & !libhammer2::fs::HAMMER2_PBUFMASK,
        (alloc_base - 1) & !libhammer2::fs::HAMMER2_PBUFMASK
    );
    vol.pwrite(
        &buf,
        sroot_blockref.data_off & !libhammer2::fs::HAMMER2_PBUFMASK,
    )?;

    Ok((alloc_base, sroot_blockref))
}

fn copy_inode_to_buffer(
    buf: &mut [u8],
    bref: &libhammer2::fs::Hammer2Blockref,
    rawip: &libhammer2::fs::Hammer2InodeData,
) -> hammer2_utils::Result<()> {
    let rawip = libhammer2::util::any_as_u8_slice(rawip);
    let offset = (bref.data_off & libhammer2::fs::HAMMER2_OFF_MASK_LO).try_into()?;
    let beg = offset;
    let end = offset + rawip.len();
    buf[beg..end].copy_from_slice(rawip);
    Ok(())
}

// Create the volume header, the super-root directory inode, and
// the writable snapshot subdirectory (named via the label) which
// is to be the initial mount point, or at least the first mount point.
// newfs_hammer2 doesn't format the freemap bitmaps for these.
//
// 0                      4MB
// [----reserved_area----][boot_area][aux_area]
// [[vol_hdr][freemap]...]                     [sroot][root][root]...
//     \                                        ^\     ^     ^
//      \--------------------------------------/  \---/-----/---...
//
// Note: The total size is 8MB-aligned to avoid edge cases.
fn format(
    fso: &mut libhammer2::ondisk::Ondisk,
    opt: &mut Opt,
    index: usize,
    free_size: u64,
) -> hammer2_utils::Result<()> {
    const DMSG_PEER_HAMMER2: u8 = 3; // server: h2 mounted volume
    let boot_base = libhammer2::fs::HAMMER2_ZONE_SEG;
    let aux_base = boot_base + opt.boot_area_size;
    let mut alloc_base;

    // Make sure we can write to the last usable block.
    let vol = &mut fso[index];
    vol.pwrite(
        &get_buffer()?,
        vol.get_size() - libhammer2::fs::HAMMER2_PBUFSIZE,
    )?;

    // Format misc area and sroot/root inodes for the root volume.
    let mut sroot_blockset = libhammer2::fs::Hammer2Blockset::new();
    if vol.get_id() == libhammer2::fs::HAMMER2_ROOT_VOLUME.into() {
        alloc_base = format_misc(vol, opt, boot_base, aux_base)?;
        let t = format_inode(vol, opt, alloc_base)?;
        alloc_base = t.0;
        sroot_blockset.blockref[0] = t.1;
    } else {
        alloc_base = 0;
        for i in 0..libhammer2::fs::HAMMER2_SET_COUNT {
            sroot_blockset.blockref[i].typ = libhammer2::fs::HAMMER2_BREF_TYPE_INVALID;
        }
    }

    // Format the volume header.
    //
    // The volume header points to sroot_blockset.  Also be absolutely
    // sure that allocator_beg is set for the root volume.
    let vol = &fso[index];
    let mut voldata = libhammer2::fs::Hammer2VolumeData::new();

    voldata.magic = libhammer2::fs::HAMMER2_VOLUME_ID_HBO;
    if vol.get_id() == libhammer2::fs::HAMMER2_ROOT_VOLUME.into() {
        voldata.boot_beg = boot_base;
        voldata.boot_end = boot_base + opt.boot_area_size;
        voldata.aux_beg = aux_base;
        voldata.aux_end = aux_base + opt.aux_area_size;
    }
    voldata.volu_size = vol.get_size();
    voldata.version = opt.hammer2_version;
    voldata.flags = 0;

    if voldata.version >= libhammer2::fs::HAMMER2_VOL_VERSION_MULTI_VOLUMES {
        voldata.volu_id = vol.get_id().try_into()?;
        voldata.nvolumes = fso.get_nvolumes().try_into()?;
        voldata.total_size = fso.get_total_size();
        for i in 0..libhammer2::fs::HAMMER2_MAX_VOLUMES.into() {
            voldata.volu_loff[i] = if i < fso.get_nvolumes() {
                fso[i].get_offset()
            } else {
                u64::MAX
            };
        }
    }

    voldata
        .fsid
        .copy_from_slice(libhammer2::util::any_as_u8_slice(&opt.volfsid));
    voldata
        .fstype
        .copy_from_slice(libhammer2::util::any_as_u8_slice(&opt.fstype));

    voldata.peer_type = DMSG_PEER_HAMMER2; // LNK_CONN identification

    assert!(vol.get_id() == libhammer2::fs::HAMMER2_ROOT_VOLUME.into() || alloc_base == 0);
    voldata.allocator_size = free_size;
    if vol.get_id() == libhammer2::fs::HAMMER2_ROOT_VOLUME.into() {
        voldata.allocator_free = free_size;
        voldata.allocator_beg = alloc_base;
    }

    voldata.sroot_blockset = sroot_blockset;
    voldata.mirror_tid = 16; // all blockref mirror TIDs set to 16
    voldata.freemap_tid = 16; // all blockref mirror TIDs set to 16
    voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT1] = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRC1_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRC1_SIZE,
    );

    // Set ICRC_SECT0 after all remaining elements of sect0 have been
    // populated in the volume header.  Note hat ICRC_SECT* (except for
    // SECT0) are part of sect0.
    voldata.icrc_sects[libhammer2::fs::HAMMER2_VOL_ICRC_SECT0] = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRC0_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRC0_SIZE,
    );
    voldata.icrc_volheader = voldata.get_crc(
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_OFF,
        libhammer2::fs::HAMMER2_VOLUME_ICRCVH_SIZE,
    );

    // Write the volume header and all alternates.
    let vol = &mut fso[index];
    for i in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
        let offset = libhammer2::volume::get_volume_data_offset(i);
        if offset >= vol.get_size() {
            break;
        }
        vol.pwrite(libhammer2::util::any_as_u8_slice(&voldata), offset)?;
    }
    Ok(vol.fsync()?)
}

fn alloc_direct(
    base: u64,
    bytes: u64,
) -> hammer2_utils::Result<(u64, libhammer2::fs::Hammer2Blockref)> {
    assert!(bytes > 0);
    let mut bytes = bytes;
    let mut radix = 0u8;

    while (bytes & 1) == 0 {
        bytes >>= 1;
        radix += 1;
    }
    assert_eq!(bytes, 1);

    if radix < libhammer2::fs::HAMMER2_RADIX_MIN.try_into()? {
        radix = libhammer2::fs::HAMMER2_RADIX_MIN.try_into()?;
    }

    let mut bref = libhammer2::fs::Hammer2Blockref::new_empty();
    bref.data_off = base | u64::from(radix);
    bref.vradix = radix;
    Ok((base + (1 << radix), bref))
}

#[allow(clippy::too_many_lines)]
pub(crate) fn mkfs(args: &[&str], opt: &mut Opt) -> hammer2_utils::Result<()> {
    let nvolumes = args.len();
    assert!(nvolumes >= 1);
    assert!(nvolumes <= libhammer2::fs::HAMMER2_MAX_VOLUMES.into());

    // Construct volumes information.
    // 1GB alignment (level1 freemap size) for volumes except for the last.
    // For the last volume, typically 8MB alignment to avoid edge cases for
    // reserved blocks and so raid stripes (if any) operate efficiently.
    let mut fso = libhammer2::ondisk::Ondisk::new(Some(opt.hammer2_version));

    let mut resid = 0;
    let n = opt.fs_size.len();
    if n == 1 {
        resid = opt.fs_size[0];
        assert!(resid >= libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE);
    } else if n > 1 && nvolumes != n {
        log::error!("Invalid filesystem size count {n} vs {nvolumes}");
        return Err(Box::new(nix::errno::Errno::EINVAL));
    }

    for (i, f) in args.iter().enumerate().take(nvolumes) {
        let mut size = libhammer2::subs::get_volume_size_from_path(f)?;
        // Limit size if a smaller filesystem size is specified.
        match n.cmp(&1) {
            std::cmp::Ordering::Equal => {
                if resid == 0 {
                    log::error!("No remaining filesystem size for {f}");
                    return Err(Box::new(nix::errno::Errno::EINVAL));
                }
                if size > resid {
                    size = resid;
                }
                resid -= size;
            }
            std::cmp::Ordering::Greater => {
                resid = opt.fs_size[i];
                assert!(resid >= libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE);
                if size > resid {
                    size = resid;
                }
            }
            std::cmp::Ordering::Less => (),
        }
        if i == nvolumes - 1 {
            size &= !libhammer2::fs::HAMMER2_VOLUME_ALIGNMASK;
        } else {
            size &= !libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_MASK;
        }
        if size == 0 {
            log::error!("{f} has aligned size of 0");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        fso.install_volume(i.try_into()?, f, false, fso.get_total_size(), size)?;
    }

    // Verify volumes constructed above.
    for i in 0..nvolumes {
        let vol = &fso[i];
        println!(
            "Volume {:<15} size {}",
            vol.get_path(),
            libhammer2::subs::get_size_string(vol.get_size())
        );
    }
    fso.verify_volumes(false)?;

    // Adjust options.
    opt.adjust(fso.get_total_size());

    // Calculate the amount of reserved space.  HAMMER2_ZONE_SEG (4MB)
    // is reserved at the beginning of every 1GB of storage, rounded up.
    // Thus a 200MB filesystem will still have a 4MB reserve area.
    //
    // We also include the boot and aux areas in the reserve.  The
    // reserve is used to help 'df' calculate the amount of available
    // space.
    //
    // XXX I kinda screwed up and made the reserved area on the LEVEL1
    //     boundary rather than the ZONE boundary.  LEVEL1 is on 1GB
    //     boundaries rather than 2GB boundaries.  Stick with the LEVEL1
    //     boundary.
    let reserved_size = ((fso.get_total_size() + libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_MASK)
        / libhammer2::fs::HAMMER2_FREEMAP_LEVEL1_SIZE)
        * libhammer2::fs::HAMMER2_ZONE_SEG;

    let x = reserved_size + opt.boot_area_size + opt.aux_area_size;
    if fso.get_total_size() < x {
        log::error!("Not enough free space");
        return Err(Box::new(nix::errno::Errno::EINVAL));
    }
    let free_size = fso.get_total_size() - x;

    // Format HAMMER2 volumes.
    for i in 0..nvolumes {
        format(&mut fso, opt, i, free_size)?;
    }

    println!("---------------------------------------------");
    println!("version:          {}", opt.hammer2_version);
    println!(
        "total-size:       {} ({} bytes)",
        libhammer2::subs::get_size_string(fso.get_total_size()),
        fso.get_total_size()
    );
    println!(
        "boot-area-size:   {} ({} bytes)",
        libhammer2::subs::get_size_string(opt.boot_area_size),
        opt.boot_area_size
    );
    println!(
        "aux-area-size:    {} ({} bytes)",
        libhammer2::subs::get_size_string(opt.aux_area_size),
        opt.aux_area_size
    );
    println!(
        "topo-reserved:    {} ({} bytes)",
        libhammer2::subs::get_size_string(reserved_size),
        reserved_size
    );
    println!(
        "free-size:        {} ({} bytes)",
        libhammer2::subs::get_size_string(free_size),
        free_size
    );
    println!("vol-fsid:         {}", opt.volfsid);
    println!("sup-clid:         {}", opt.supclid);
    println!("sup-fsid:         {}", opt.supfsid);
    for i in 0..opt.label.len() {
        println!("PFS \"{}\"", opt.label[i]);
        println!("    clid {}", opt.pfsclid[i]);
        println!("    fsid {}", opt.pfsfsid[i]);
    }
    if opt.debug {
        println!("---------------------------------------------");
        for s in &fso.fmt_volumes() {
            println!("{s}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_hammer2_version() {
        let version = super::get_hammer2_version();
        println!("{version}");
        assert_eq!(version, libhammer2::fs::HAMMER2_VOL_VERSION_MULTI_VOLUMES);
    }

    #[test]
    fn test_get_size_1() {
        assert!(super::get_size("0", u64::MIN, u64::MAX, 1).is_err());
        assert!(super::get_size("1", u64::MIN, u64::MAX, 1).is_err());
        assert!(super::get_size("k", u64::MIN, u64::MAX, 1).is_err());
        assert!(super::get_size("K", u64::MIN, u64::MAX, 1).is_err());
        assert!(super::get_size("m", u64::MIN, u64::MAX, 1).is_err());
        assert!(super::get_size("M", u64::MIN, u64::MAX, 1).is_err());

        assert_eq!(super::get_size("0k", u64::MIN, u64::MAX, 1), Ok(0));
        assert_eq!(super::get_size("0K", u64::MIN, u64::MAX, 1), Ok(0));
        assert_eq!(super::get_size("1k", u64::MIN, u64::MAX, 1), Ok(1 << 10));
        assert_eq!(super::get_size("1K", u64::MIN, u64::MAX, 1), Ok(1 << 10));
        assert_eq!(super::get_size("2k", u64::MIN, u64::MAX, 1), Ok(2 << 10));
        assert_eq!(super::get_size("2K", u64::MIN, u64::MAX, 1), Ok(2 << 10));

        assert_eq!(super::get_size("0m", u64::MIN, u64::MAX, 1), Ok(0));
        assert_eq!(super::get_size("0M", u64::MIN, u64::MAX, 1), Ok(0));
        assert_eq!(super::get_size("1m", u64::MIN, u64::MAX, 1), Ok(1 << 20));
        assert_eq!(super::get_size("1M", u64::MIN, u64::MAX, 1), Ok(1 << 20));
        assert_eq!(super::get_size("2m", u64::MIN, u64::MAX, 1), Ok(2 << 20));
        assert_eq!(super::get_size("2M", u64::MIN, u64::MAX, 1), Ok(2 << 20));
    }

    #[test]
    fn test_get_size_2() {
        assert!(super::get_size("0", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("1", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("k", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("K", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("m", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("M", u64::MIN, u64::MAX, 2).is_err());

        assert_eq!(super::get_size("0k", u64::MIN, u64::MAX, 2), Ok(0));
        assert_eq!(super::get_size("0K", u64::MIN, u64::MAX, 2), Ok(0));
        assert!(super::get_size("8k", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("8K", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("16k", u64::MIN, u64::MAX, 2).is_err());
        assert!(super::get_size("16K", u64::MIN, u64::MAX, 2).is_err());

        assert_eq!(super::get_size("0m", u64::MIN, u64::MAX, 2), Ok(0));
        assert_eq!(super::get_size("0M", u64::MIN, u64::MAX, 2), Ok(0));
        assert_eq!(super::get_size("8m", u64::MIN, u64::MAX, 2), Ok(8 << 20));
        assert_eq!(super::get_size("8M", u64::MIN, u64::MAX, 2), Ok(8 << 20));
        assert_eq!(super::get_size("16m", u64::MIN, u64::MAX, 2), Ok(16 << 20));
        assert_eq!(super::get_size("16M", u64::MIN, u64::MAX, 2), Ok(16 << 20));
    }
}
