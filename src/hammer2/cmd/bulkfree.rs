use crate::Hammer2Options;

use std::os::fd::AsRawFd;

fn get_usermem() -> u64 {
    let mut usermem = 0;
    let mut usermem_size = std::mem::size_of_val(&usermem);
    if unsafe {
        libhammer2::os::sysctlbyname(
            c"hw.usermem".as_ptr().cast::<libc::c_char>(),
            std::ptr::from_mut::<u64>(&mut usermem).cast::<libc::c_void>(),
            std::ptr::from_mut::<libc::size_t>(&mut usermem_size),
            std::ptr::null_mut(),
            0,
        )
    } == 0
    {
        usermem
    } else {
        0
    }
}

pub(crate) fn run(f: &str, opt: &Hammer2Options) -> Result<(), Box<dyn std::error::Error>> {
    const UNIT: u64 = 8 * libhammer2::subs::M_U64;
    let mut bfi = libhammer2::ioctl::Hammer2IocBulkfree::new();
    bfi.size = get_usermem() / 16;
    if bfi.size < UNIT {
        bfi.size = UNIT;
    }
    if opt.mem != 0 {
        bfi.size = (u64::try_from(opt.mem)? + UNIT - 1) & !(UNIT - 1);
    }
    let fp = libhammer2::subs::get_ioctl_handle(f)?;
    nix::ioctl_readwrite!(
        bulkfree_scan,
        libhammer2::ioctl::HAMMER2IOC,
        libhammer2::ioctl::HAMMER2IOC_BULKFREE_SCAN,
        libhammer2::ioctl::Hammer2IocBulkfree
    );
    unsafe { bulkfree_scan(fp.as_raw_fd(), &mut bfi) }?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_usermem() {
        let usermem = super::get_usermem();
        println!("{usermem}");
        #[cfg(target_os = "freebsd")]
        assert!(usermem > 0);
        #[cfg(target_os = "linux")]
        assert_eq!(usermem, 0);
    }
}
