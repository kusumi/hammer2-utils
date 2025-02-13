use std::io::Write;
use std::os::fd::AsRawFd;

pub(crate) fn run(f: &str, args: &[&str]) -> hammer2_utils::Result<()> {
    println!("deleting inodes on {f}");
    for s in args {
        let mut des = libhammer2::ioctl::IocDestroy::new();
        des.cmd = libhammer2::ioctl::DESTROY_CMD_INUM;
        des.inum = s.parse()?;
        print!("{:16} ", des.inum);
        std::io::stdout().flush()?;
        let fp = super::get_ioctl_handle(f)?;
        match unsafe { libhammer2::ioctl::destroy(fp.as_raw_fd(), &mut des) } {
            Ok(_) => println!("ok"),
            Err(e) => {
                println!("{e}");
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}
