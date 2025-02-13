use std::os::fd::AsRawFd;

pub(crate) fn run(args: &[&str]) -> hammer2_utils::Result<()> {
    for f in args {
        let mut gfs = libhammer2::ioctl::IocGrowfs::new();
        let fp = super::get_ioctl_handle(f)?;
        unsafe { libhammer2::ioctl::growfs(fp.as_raw_fd(), &mut gfs) }?;
        if gfs.modified != 0 {
            println!("{f} grown to {:016x}", gfs.size);
        } else {
            println!("{f} no size change - {:016x}", gfs.size);
        }
    }
    Ok(())
}
