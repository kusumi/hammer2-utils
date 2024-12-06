use std::os::fd::AsRawFd;

pub(crate) fn run(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut w = 0;
    for f in args {
        if w < f.len() {
            w = f.len();
        }
    }
    if w < 16 {
        w = 16;
    }
    println!(
        "{:<w$} ncp  data-use inode-use comp               check        quota",
        "PATH"
    );
    for f in args {
        let mut ino = libhammer2::ioctl::Hammer2IocInode::new();
        let fp = libhammer2::subs::get_ioctl_handle(f)?;
        nix::ioctl_readwrite!(
            inode_get,
            libhammer2::ioctl::HAMMER2IOC,
            libhammer2::ioctl::HAMMER2IOC_INODE_GET,
            libhammer2::ioctl::Hammer2IocInode
        );
        unsafe { inode_get(fp.as_raw_fd(), &mut ino) }?;
        print!("{f:<w$} ");
        print!("{:>3} ", ino.ip_data.meta.ncopies);
        print!("{:>9} ", libhammer2::subs::get_size_string(ino.data_count));
        print!(
            "{:>9} ",
            libhammer2::subs::get_count_string(ino.inode_count)
        );
        print!(
            "{:<18} ",
            libhammer2::subs::get_comp_mode_string(ino.ip_data.meta.comp_algo)
        );
        print!(
            "{:<12} ",
            libhammer2::subs::get_check_mode_string(ino.ip_data.meta.check_algo)
        );
        if ino.ip_data.meta.data_quota != 0 || ino.ip_data.meta.inode_quota != 0 {
            print!(
                "{}",
                libhammer2::subs::get_size_string(ino.ip_data.meta.data_quota)
            );
            print!(
                "/{:<12}",
                libhammer2::subs::get_count_string(ino.ip_data.meta.inode_quota)
            );
        }
        println!();
    }
    Ok(())
}