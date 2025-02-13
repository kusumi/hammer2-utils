use std::os::fd::AsRawFd;

fn format_prefix(name: &str) -> String {
    format!("pfs_create({name})")
}

pub(crate) fn run(f: &str, name: &str, opt: &crate::Opt) -> hammer2_utils::Result<()> {
    // Default to MASTER if no uuid was specified.
    // Default to SLAVE if a uuid was specified.
    //
    // When adding masters to a cluster, the new PFS must be added as
    // a slave and then upgraded to ensure proper synchronization.
    let pfs_type = if opt.pfs_type == libhammer2::fs::HAMMER2_PFSTYPE_NONE {
        if opt.uuid_str.is_some() {
            libhammer2::fs::HAMMER2_PFSTYPE_SLAVE
        } else {
            libhammer2::fs::HAMMER2_PFSTYPE_MASTER
        }
    } else {
        opt.pfs_type
    };
    let mut pfs = libhammer2::ioctl::IocPfs::new();
    pfs.copy_name(name.as_bytes());
    pfs.pfs_type = pfs_type;
    pfs.pfs_clid = *match &opt.uuid_str {
        Some(v) => libhammer2::subs::get_uuid_from_str(v.as_str())?,
        None => uuid::Uuid::new_v4(),
    }
    .as_bytes();
    pfs.pfs_fsid = *uuid::Uuid::new_v4().as_bytes();
    let fp = super::get_ioctl_handle(f)?;
    match unsafe { libhammer2::ioctl::pfs_create(fp.as_raw_fd(), &mut pfs) } {
        Err(nix::errno::Errno::EEXIST) => {
            log::error!(
                "NOTE: Typically the same name is used for cluster elements on \
                different mounts,\n      \
                but cluster elements on the same mount require unique names.\n\
                {}: already present",
                format_prefix(name)
            );
            return Err(Box::new(nix::errno::Errno::EEXIST));
        }
        Err(e) => return Err(Box::new(e)),
        _ => (),
    }
    println!("{}: SUCCESS", format_prefix(name));
    Ok(())
}
