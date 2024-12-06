use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, name: &str, privateid: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut pfs = libhammer2::ioctl::Hammer2IocPfs::new();
    pfs.copy_name(name.as_bytes());
    let fp = libhammer2::subs::get_ioctl_handle(f)?;
    nix::ioctl_readwrite!(
        pfs_lookup,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_PFS_LOOKUP,
        libhammer2::ioctl::Hammer2IocPfs
    );
    unsafe { pfs_lookup(fp.as_raw_fd(), &mut pfs) }?;
    let pfs_id_str = libhammer2::subs::get_uuid_string_from_bytes(if privateid {
        &pfs.pfs_fsid
    } else {
        &pfs.pfs_clid
    });
    println!("{pfs_id_str}");
    Ok(())
}
