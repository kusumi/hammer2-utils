#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "freebsd")]
pub(crate) use freebsd::*;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
