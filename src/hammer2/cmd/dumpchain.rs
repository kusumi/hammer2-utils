use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str) -> hammer2_utils::Result<()> {
    let mut dummy = 0;
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::debug_dump(fp.as_raw_fd(), &mut dummy) }?;
    Ok(())
}
