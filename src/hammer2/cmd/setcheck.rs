use std::os::fd::AsRawFd;

fn parse_check_algo(s: &str) -> hammer2_utils::Result<u8> {
    if let Ok(v) = s.parse::<u8>() {
        Ok(v)
    } else {
        for (i, x) in libhammer2::subs::HAMMER2_CHECK_STRINGS.iter().enumerate() {
            if s == *x {
                return Ok(i.try_into()?);
            }
        }
        if s == "default" {
            return Ok(libhammer2::fs::HAMMER2_CHECK_XXHASH64);
        } else if s == "disabled" {
            return Ok(libhammer2::fs::HAMMER2_CHECK_DISABLED);
        }
        log::error!("Unknown check code type: {s}");
        Err(Box::new(nix::errno::Errno::ENOSYS))
    }
}

fn setcheck(
    check_algo: u8,
    f: &str,
    m: &std::fs::Metadata,
    opt: &crate::Opt,
) -> hammer2_utils::Result<()> {
    let mut ino = libhammer2::ioctl::IocInode::new();
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::inode_get(fp.as_raw_fd(), &mut ino) }?;
    println!("{f}\tcheck_algo={check_algo:#04x}");
    ino.flags |= libhammer2::ioctl::INODE_FLAGS_CHECK;
    ino.ip_data.meta.check_algo = check_algo;
    unsafe { libhammer2::ioctl::inode_set(fp.as_raw_fd(), &mut ino) }?;
    if opt.recurse && m.file_type().is_dir() {
        for entry in std::fs::read_dir(f)? {
            let entry = entry?;
            let name = entry.file_name();
            if name == "." || name == ".." {
                continue;
            }
            setcheck(
                check_algo,
                entry.path().to_str().ok_or(nix::errno::Errno::ENOENT)?,
                &entry.metadata()?,
                opt,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn run(check_str: &str, paths: &[&str], opt: &crate::Opt) -> hammer2_utils::Result<()> {
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
