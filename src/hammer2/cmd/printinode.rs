use std::os::fd::AsRawFd;

fn hexdump_inode(meta: &libhammer2::fs::Hammer2InodeMeta, opt: &crate::Opt) {
    if !opt.verbose {
        return;
    }
    let data = libfs::cast::as_u8_slice(meta);
    for (i, x) in data.iter().enumerate() {
        print!("{x:02X}");
        if (i + 1) % 16 == 0 {
            println!();
        } else if i != data.len() - 1 {
            print!(" ");
        }
    }
    println!();
}

pub(crate) fn run(f: &str, opt: &crate::Opt) -> hammer2_utils::Result<()> {
    let mut ino = libhammer2::ioctl::IocInode::new();
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::inode_get(fp.as_raw_fd(), &mut ino) }?;
    let meta = &ino.ip_data.meta;
    hexdump_inode(meta, opt);
    println!("version = {}", meta.version);
    println!(
        "pfs_subtype = {} ({})",
        meta.pfs_subtype,
        libhammer2::subs::get_pfs_subtype_string(meta.pfs_subtype)
    );
    println!("uflags = {:#x}", meta.uflags);
    println!("rmajor = {}", meta.rmajor);
    println!("rminor = {}", meta.rminor);
    println!(
        "ctime = {}",
        libhammer2::subs::get_local_time_string(meta.ctime)
    );
    println!(
        "mtime = {}",
        libhammer2::subs::get_local_time_string(meta.mtime)
    );
    println!(
        "atime = {}",
        libhammer2::subs::get_local_time_string(meta.atime)
    );
    println!(
        "btime = {}",
        libhammer2::subs::get_local_time_string(meta.btime)
    );
    println!(
        "uid = {}",
        libhammer2::subs::get_uuid_string_from_bytes(&meta.uid)
    );
    println!(
        "gid = {}",
        libhammer2::subs::get_uuid_string_from_bytes(&meta.gid)
    );
    println!(
        "type = {} ({})",
        meta.typ,
        libhammer2::subs::get_inode_type_string(meta.typ)
    );
    println!("op_flags = {:#x}", meta.op_flags);
    println!("cap_flags = {:#x}", meta.cap_flags);
    println!("mode = 0{:o}", meta.mode); // :#o prints 0o (not 0)
    println!("inum = {:#x}", meta.inum);
    println!("size = {}", meta.size);
    println!("nlinks = {}", meta.nlinks);
    println!("iparent = {:#x}", meta.iparent);
    println!("name_key = {:#x}", meta.name_key);
    println!("name_len = {}", meta.name_len);
    println!("ncopies = {}", meta.ncopies);
    println!("comp_algo = {}", meta.comp_algo);
    println!("check_algo = {}", meta.check_algo);
    println!("pfs_nmasters = {}", meta.pfs_nmasters);
    println!(
        "pfs_type = {} ({})",
        meta.pfs_type,
        libhammer2::subs::get_pfs_type_string(meta.pfs_type)
    );
    println!("pfs_inum = {:#x}", meta.pfs_inum);
    println!(
        "pfs_clid = {}",
        libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_clid)
    );
    println!(
        "pfs_fsid = {}",
        libhammer2::subs::get_uuid_string_from_bytes(&meta.pfs_fsid)
    );
    println!("data_quota = {:#x}", meta.data_quota);
    println!("inode_quota = {:#x}", meta.inode_quota);
    println!("pfs_lsnap_tid = {:#x}", meta.pfs_lsnap_tid);
    println!("decrypt_check = {:#x}", meta.decrypt_check);
    Ok(())
}
