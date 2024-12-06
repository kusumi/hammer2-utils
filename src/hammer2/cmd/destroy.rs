use std::io::Write;
use std::os::fd::AsRawFd;

fn split(f: &str) -> (&str, &str) {
    let f = f.trim_end_matches('/');
    let mut n = f.len();
    for c in f.chars().rev() {
        if c == '/' {
            break;
        }
        n -= 1;
    }
    if n == 0 {
        (".", f)
    } else {
        (&f[..n], &f[n..])
    }
}

pub(crate) fn run(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for &f in args {
        let mut des = libhammer2::ioctl::Hammer2IocDestroy::new();
        des.cmd = libhammer2::ioctl::HAMMER2_DELETE_FILE;
        let (dir, base) = split(f);
        des.copy_path(base.as_bytes());
        print!("{f}\t");
        std::io::stdout().flush()?;
        let fp = libhammer2::subs::get_ioctl_handle(dir)?;
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
