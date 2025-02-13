use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, enable: bool) -> hammer2_utils::Result<()> {
    let mut enable = u32::from(enable);
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::emerg_mode(fp.as_raw_fd(), &mut enable) }?;
    println!(
        "Emergency mode on \"{f}\" {}d",
        if enable == 1 { "enable" } else { "disable" }
    );
    Ok(())
}
