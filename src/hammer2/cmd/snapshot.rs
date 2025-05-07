use std::os::fd::AsRawFd;

// The snapshot is named <PFSNAME>_<YYYYMMDD.HHMMSS.TRANSID> unless
// overridden by a label.
//
// When local non-cache media is involved the media is
// first synchronized and the snapshot is then based on
// the media.
//
// If the media is remote the snapshot is created on the remote
// end (if you have sufficient administrative rights) and a local
// ADMIN or CACHE PFS is created with a connection to the snapshot
// on the remote.
//
// If the client has snapshot rights to multiple remotes then TBD.

pub(crate) fn run(f: &str, args: &[&str], flags: u32) -> hammer2_utils::Result<()> {
    let f = if args.is_empty() { f } else { args[0] };
    let label = if args.len() > 1 {
        args[1]
    } else {
        let mut pfs = libhammer2::ioctl::IocPfs::new();
        pfs.name_key = u64::MAX;
        let fp = super::get_ioctl_handle(f)?;
        unsafe { libhammer2::ioctl::pfs_get(fp.as_raw_fd(), &mut pfs) }?;
        // XXX want local time
        let dt: time::OffsetDateTime = std::time::SystemTime::now().into();
        let fmt = time::format_description::parse("[year][month][day].[hour][minute][second]")?;
        &format!("{}.{}", libfs::string::b2s(&pfs.name)?, dt.format(&fmt)?)
    };
    let mut pfs = libhammer2::ioctl::IocPfs::new();
    pfs.copy_name(label.as_bytes());
    pfs.pfs_flags = flags;
    let fp = super::get_ioctl_handle(f)?;
    unsafe { libhammer2::ioctl::pfs_snapshot(fp.as_raw_fd(), &mut pfs) }?;
    println!("created snapshot {label}");
    Ok(())
}
