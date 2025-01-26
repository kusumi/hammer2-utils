use std::os::fd::AsRawFd;

fn format_prefix(name: &str) -> String {
    format!("pfs_delete({name})")
}

fn get_mount_handle(
    pfs: &mut libhammer2::ioctl::Hammer2IocPfs,
) -> Result<Vec<std::fs::File>, Box<dyn std::error::Error>> {
    let mut v = vec![];
    for f in &libhammer2::subs::get_hammer2_mounts()? {
        let fp = libhammer2::subs::get_ioctl_handle(f)?;
        nix::ioctl_readwrite!(
            pfs_lookup,
            libhammer2::ioctl::HAMMER2IOC,
            libhammer2::ioctl::HAMMER2IOC_PFS_LOOKUP,
            libhammer2::ioctl::Hammer2IocPfs
        );
        if unsafe { pfs_lookup(fp.as_raw_fd(), pfs) }.is_ok() {
            v.push(fp);
        }
    }
    Ok(v)
}

pub(crate) fn run(f: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for name in args {
        let mut pfs = libhammer2::ioctl::Hammer2IocPfs::new();
        pfs.copy_name(name.as_bytes());
        let v = if f.is_empty() {
            let v = get_mount_handle(&mut pfs)?;
            if v.is_empty() {
                log::error!("{}: {name} not found", format_prefix(name));
                return Err(Box::new(nix::errno::Errno::ENOENT));
            }
            if v.len() > 1 {
                log::error!(
                    "{}: Duplicate PFS name, must specify mount",
                    format_prefix(name)
                );
                return Err(Box::new(nix::errno::Errno::EEXIST));
            }
            v
        } else {
            vec![libhammer2::subs::get_ioctl_handle(f)?]
        };
        nix::ioctl_readwrite!(
            pfs_delete,
            libhammer2::ioctl::HAMMER2IOC,
            libhammer2::ioctl::HAMMER2IOC_PFS_DELETE,
            libhammer2::ioctl::Hammer2IocPfs
        );
        unsafe { pfs_delete(v[0].as_raw_fd(), &mut pfs) }?;
        println!("{}: SUCCESS", format_prefix(name));
    }
    Ok(())
}
