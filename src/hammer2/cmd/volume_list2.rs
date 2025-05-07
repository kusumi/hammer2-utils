use std::os::fd::AsRawFd;

pub(crate) fn run(args: &[&str], opt: &crate::Opt) -> hammer2_utils::Result<()> {
    let (args, all) = if args.len() == 1 && args[0].is_empty() {
        (libhammer2::subs::get_hammer2_mounts()?, true)
    } else {
        let mut v = vec![];
        for &f in args {
            v.push(f.to_string());
        }
        (v, false)
    };
    for (i, f) in args.iter().enumerate() {
        if i != 0 {
            println!();
        }
        if args.len() > 1 || all {
            println!("{f}");
        }
        let mut vol = libhammer2::ioctl::IocVolumeList2::new();
        vol.nvolumes = libhammer2::fs::HAMMER2_MAX_VOLUMES.into();
        let fp = super::get_ioctl_handle(f)?;
        unsafe { libhammer2::ioctl::volume_list2(fp.as_raw_fd(), &mut vol) }?;
        let nvolumes = vol.nvolumes.try_into()?;
        let mut w = 0;
        for entry in vol.volumes.iter().take(nvolumes) {
            let n = libfs::string::b2s(&entry.path)?.len();
            if n > w {
                w = n;
            }
        }
        if opt.quiet {
            for entry in vol.volumes.iter().take(nvolumes) {
                println!("{}", libfs::string::b2s(&entry.path)?);
            }
        } else {
            println!("version {}", vol.version);
            println!("@{}", libfs::string::b2s(&vol.pfs_name)?);
            for entry in vol.volumes.iter().take(nvolumes) {
                print!(
                    "volume{:<2} {:<w$} {}",
                    entry.id,
                    libfs::string::b2s(&entry.path)?,
                    libhammer2::subs::get_size_string(entry.size)
                );
                if opt.verbose {
                    print!(" {:#018x} {:#018x}", entry.offset, entry.size);
                }
                println!();
            }
        }
    }
    Ok(())
}
