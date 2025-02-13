use std::os::fd::AsRawFd;

fn parse_comp_algo(s: &str) -> hammer2_utils::Result<(u8, &str)> {
    if let Ok(v) = s.parse::<u8>() {
        Ok((v, s))
    } else {
        for (i, x) in libhammer2::subs::HAMMER2_COMP_STRINGS.iter().enumerate() {
            if s == *x {
                return Ok((i.try_into()?, s));
            }
        }
        if s == "default" {
            return Ok((libhammer2::fs::HAMMER2_COMP_LZ4, "lz4"));
        } else if s == "disabled" {
            return Ok((libhammer2::fs::HAMMER2_COMP_AUTOZERO, "autozero"));
        }
        log::error!("Unknown compression type: {s}");
        Err(Box::new(nix::errno::Errno::ENOSYS))
    }
}

fn parse_comp_level(s: Option<&str>) -> hammer2_utils::Result<u8> {
    if let Some(v) = s {
        if let Ok(v) = v.parse::<u8>() {
            Ok(v)
        } else if v == "default" {
            Ok(0)
        } else {
            log::error!("Unknown compression level: {v}");
            Err(Box::new(nix::errno::Errno::ENOSYS))
        }
    } else {
        Ok(0)
    }
}

fn setcomp(
    comp_algo: u8,
    f: &str,
    m: &std::fs::Metadata,
    opt: &crate::Opt,
) -> hammer2_utils::Result<()> {
    let mut ino = libhammer2::ioctl::IocInode::new();
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::inode_get(fp.as_raw_fd(), &mut ino) }?;
    println!("{f}\tcomp_algo={comp_algo:#04x}");
    ino.flags |= libhammer2::ioctl::INODE_FLAGS_COMP;
    ino.ip_data.meta.comp_algo = comp_algo;
    unsafe { libhammer2::ioctl::inode_set(fp.as_raw_fd(), &mut ino) }?;
    if opt.recurse && m.file_type().is_dir() {
        for entry in std::fs::read_dir(f)? {
            let entry = entry?;
            let name = entry.file_name();
            if name == "." || name == ".." {
                continue;
            }
            setcomp(
                comp_algo,
                entry.path().to_str().ok_or(nix::errno::Errno::ENOENT)?,
                &entry.metadata()?,
                opt,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn run(comp_str: &str, paths: &[&str], opt: &crate::Opt) -> hammer2_utils::Result<()> {
    let v = &comp_str.split(':').collect::<Vec<&str>>();
    let v: Vec<String> = v.iter().map(|s| s.to_lowercase()).collect();
    let (comp_algo, comp_algo_str) = parse_comp_algo(v[0].as_str())?;
    let comp_level = parse_comp_level(if v.len() > 1 {
        Some(v[1].as_str())
    } else {
        None
    })?;
    if comp_level != 0 {
        if comp_algo == libhammer2::fs::HAMMER2_COMP_ZLIB {
            if !(6..=9).contains(&comp_level) {
                log::error!("Unsupported comp_level {comp_level} for {comp_algo_str}");
                return Err(Box::new(nix::errno::Errno::ENOSYS));
            }
        } else {
            log::error!("Unsupported comp_level {comp_level} for {comp_algo_str}");
            return Err(Box::new(nix::errno::Errno::ENOSYS));
        }
    }
    for f in paths {
        match std::fs::symlink_metadata(f) {
            Ok(v) => setcomp(
                libhammer2::fs::enc_algo(comp_algo) | libhammer2::fs::enc_level(comp_level),
                f,
                &v,
                opt,
            )?,
            Err(e) => {
                log::error!("{f}: {e}");
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}
