use crate::Hammer2Options;

use std::os::fd::AsRawFd;

fn parse_check_algo(s: &str) -> Result<u8, Box<dyn std::error::Error>> {
    if let Ok(v) = s.parse::<u8>() {
        Ok(v)
    } else {
        let mut i = libhammer2::subs::HAMMER2_CHECK_STRINGS.len() - 1;
        loop {
            if s == libhammer2::subs::HAMMER2_CHECK_STRINGS[i] {
                return Ok(i.try_into()?);
            }
            if i == 0 {
                if s == "default" {
                    return Ok(libhammer2::fs::HAMMER2_CHECK_XXHASH64);
                } else if s == "disabled" {
                    return Ok(libhammer2::fs::HAMMER2_CHECK_DISABLED);
                }
                log::error!("Unknown check code type: {s}");
                return Err(Box::new(nix::errno::Errno::ENOSYS));
            }
            i -= 1;
        }
    }
}

fn setcheck(
    check_algo: u8,
    f: &str,
    m: &std::fs::Metadata,
    opt: &Hammer2Options,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ino = libhammer2::ioctl::Hammer2IocInode::new();
    let fp = libhammer2::subs::get_ioctl_handle(f)?;
    nix::ioctl_readwrite!(
        inode_get,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_INODE_GET,
        libhammer2::ioctl::Hammer2IocInode
    );
    unsafe { inode_get(fp.as_raw_fd(), &mut ino) }?;
    println!("{f}\tcheck_algo={check_algo:#04x}");
    ino.flags |= libhammer2::ioctl::HAMMER2IOC_INODE_FLAG_CHECK;
    ino.ip_data.meta.check_algo = check_algo;
    nix::ioctl_readwrite!(
        inode_set,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_INODE_SET,
        libhammer2::ioctl::Hammer2IocInode
    );
    unsafe { inode_set(fp.as_raw_fd(), &mut ino) }?;
    if opt.recurse && m.file_type().is_dir() {
        for entry in std::fs::read_dir(f)? {
            let entry = entry?;
            let name = entry.file_name();
            if name == "." || name == ".." {
                continue;
            }
            setcheck(
                check_algo,
                entry
                    .path()
                    .to_str()
                    .ok_or_else(|| std::io::Error::from(nix::errno::Errno::ENOENT))?,
                &entry.metadata()?,
                opt,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn run(
    check_str: &str,
    paths: &[&str],
    opt: &Hammer2Options,
) -> Result<(), Box<dyn std::error::Error>> {
    let check_algo = parse_check_algo(check_str.to_lowercase().as_str())?;
    for f in paths {
        match std::fs::symlink_metadata(f) {
            Ok(v) => setcheck(libhammer2::fs::enc_algo(check_algo), f, &v, opt)?,
            Err(e) => {
                log::error!("{f}: {e}");
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}
