use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, name: &str, privateid: bool) -> hammer2_utils::Result<()> {
    let mut pfs = libhammer2::ioctl::IocPfs::new();
    pfs.copy_name(name.as_bytes());
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::pfs_lookup(fp.as_raw_fd(), &mut pfs) }?;
    let pfs_id_str = libhammer2::subs::get_uuid_string_from_bytes(if privateid {
        &pfs.pfs_fsid
    } else {
        &pfs.pfs_clid
    });
    println!("{pfs_id_str}");
    Ok(())
}
