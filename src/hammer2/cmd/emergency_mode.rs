use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut enable = u32::from(enable);
    let fp = libhammer2::subs::get_ioctl_handle(f)?;
    nix::ioctl_readwrite!(
        emerg_mode,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_EMERG_MODE,
        u32
    );
    unsafe { emerg_mode(fp.as_raw_fd(), &mut enable) }?;
    println!(
        "Emergency mode on \"{f}\" {}d",
        if enable == 1 { "enable" } else { "disable" }
    );
    Ok(())
}
