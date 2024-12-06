use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut dummy = 0;
    let fp = libhammer2::subs::get_ioctl_handle(f)?;
    nix::ioctl_readwrite!(
        debug_dump,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_DEBUG_DUMP,
        u32
    );
    unsafe { debug_dump(fp.as_raw_fd(), &mut dummy) }?;
    Ok(())
}
