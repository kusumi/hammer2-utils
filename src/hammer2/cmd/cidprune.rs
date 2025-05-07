use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str) -> hammer2_utils::Result<()> {
    let mut ioc = libhammer2::ioctl::IocCidPrune::new();
    let fp = super::get_ioctl_handle(f)?;
    match unsafe { libhammer2::ioctl::cidprune(fp.as_raw_fd(), &mut ioc) } {
        Ok(_) => println!("{:?}", [ioc.vchain_total, ioc.fchain_total]),
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}
