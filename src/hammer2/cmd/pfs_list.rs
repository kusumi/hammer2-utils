use std::os::fd::AsRawFd;

pub(crate) fn run(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let args = if args.len() == 1 && args[0].is_empty() {
        libhammer2::subs::get_hammer2_mounts()?
    } else {
        let mut v = vec![];
        for &f in args {
            v.push(f.to_string());
        }
        v
    };
    for (i, f) in args.iter().enumerate() {
        let mut pfs = libhammer2::ioctl::Hammer2IocPfs::new();
        let mut v = vec![];
        let fp = libhammer2::subs::get_ioctl_handle(f)?;
        if i != 0 {
            println!();
        }
        loop {
            pfs.name_key = pfs.name_next;
            if pfs.name_key == u64::MAX {
                break;
            }
            nix::ioctl_readwrite!(
                pfs_get,
                libhammer2::ioctl::HAMMER2IOC,
                libhammer2::ioctl::HAMMER2IOC_PFS_GET,
                libhammer2::ioctl::Hammer2IocPfs
            );
            unsafe { pfs_get(fp.as_raw_fd(), &mut pfs) }?;
            let pfs_id_str = libhammer2::subs::get_uuid_string_from_bytes(&pfs.pfs_clid);
            let type_str = if pfs.pfs_type == libhammer2::fs::HAMMER2_PFSTYPE_MASTER {
                if pfs.pfs_subtype == libhammer2::fs::HAMMER2_PFSSUBTYPE_NONE {
                    "MASTER"
                } else {
                    libhammer2::subs::get_pfs_subtype_string(pfs.pfs_subtype)
                }
            } else {
                libhammer2::subs::get_pfs_type_string(pfs.pfs_type)
            };
            v.push((pfs.name, format!("{type_str:<11} {pfs_id_str}")));
        }
        v.sort_by_key(|t| t.0);
        println!("Type        ClusterId (pfs_clid)                 Label on {f}");
        for p in &v {
            println!("{} {}", p.1, libhammer2::util::bin_to_string(&p.0)?);
        }
    }
    Ok(())
}
