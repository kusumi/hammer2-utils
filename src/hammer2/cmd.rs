pub(crate) mod bulkfree;
pub(crate) mod cleanup;
pub(crate) mod destroy;
pub(crate) mod destroy_inum;
pub(crate) mod dhash;
pub(crate) mod dumpchain;
pub(crate) mod emergency_mode;
pub(crate) mod freemap;
pub(crate) mod growfs;
pub(crate) mod hash;
pub(crate) mod pfs_create;
pub(crate) mod pfs_delete;
pub(crate) mod pfs_id;
pub(crate) mod pfs_list;
pub(crate) mod printinode;
pub(crate) mod recover;
pub(crate) mod setcheck;
pub(crate) mod setcomp;
pub(crate) mod show;
pub(crate) mod snapshot;
pub(crate) mod stat;
pub(crate) mod volhdr;
pub(crate) mod volume_list;

use std::os::fd::AsRawFd;

pub(crate) fn get_ioctl_handle(sel_path: &str) -> hammer2_utils::Result<std::fs::File> {
    let f = if sel_path.is_empty() { "." } else { sel_path };
    let fp = std::fs::File::open(f)?;
    let mut info = libhammer2::ioctl::IocVersion::new();
    if let Err(e) = unsafe { libhammer2::ioctl::version_get(fp.as_raw_fd(), &mut info) } {
        log::error!("'{f}' is not a hammer2 filesystem");
        return Err(Box::new(e));
    }
    Ok(fp)
}
