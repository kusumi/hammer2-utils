use std::io::Write;
use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("deleting inodes on {f}");
    for s in args {
        let mut des = libhammer2::ioctl::Hammer2IocDestroy::new();
        des.cmd = libhammer2::ioctl::HAMMER2_DELETE_INUM;
        des.inum = s.parse()?;
        print!("{:16} ", des.inum);
        std::io::stdout().flush()?;
        let fp = libhammer2::subs::get_ioctl_handle(f)?;
        nix::ioctl_readwrite!(
            destroy,
            libhammer2::ioctl::HAMMER2IOC,
            libhammer2::ioctl::HAMMER2IOC_DESTROY,
            libhammer2::ioctl::Hammer2IocDestroy
        );
        match unsafe { destroy(fp.as_raw_fd(), &mut des) } {
            Ok(_) => println!("ok"),
            Err(e) => {
                println!("{e}");
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}
