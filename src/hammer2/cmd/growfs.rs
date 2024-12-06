use std::os::fd::AsRawFd;

pub(crate) fn run(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for f in args {
        let mut gfs = libhammer2::ioctl::Hammer2IocGrowfs::new();
        let fp = libhammer2::subs::get_ioctl_handle(f)?;
        nix::ioctl_readwrite!(
            growfs,
            libhammer2::ioctl::HAMMER2IOC,
            libhammer2::ioctl::HAMMER2IOC_GROWFS,
            libhammer2::ioctl::Hammer2IocGrowfs
        );
        unsafe { growfs(fp.as_raw_fd(), &mut gfs) }?;
        if gfs.modified != 0 {
            println!("{f} grown to {:016x}", gfs.size);
        } else {
            println!("{f} no size change - {:016x}", gfs.size);
        }
    }
    Ok(())
}
