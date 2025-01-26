// DragonFly doesn't support "comma separated string of options"
// despite description below from mount_hammer2(8), but Rust does.
pub(crate) fn get_getopts() -> getopts::Options {
    let mut gopt = getopts::Options::new();
    gopt.optopt(
        "o",
        "",
        "Options are specified with a -o flag followed by a comma separated \
        string of options. See the mount(8) man page for possible options \
        and their meanings.",
        "<option>",
    );
    gopt.optflag(
        "u",
        "",
        "Update the mount point. This is used to upgrade a mount to read-write.",
    );
    gopt
}

pub(crate) fn usage(prog: &str, gopt: &getopts::Options) {
    print!(
        "{}",
        gopt.usage(&format!(
            "usage: {prog} [-o options] special[@label] node\n\
            usage: {prog} [-o options] @label node\n\
            usage: {prog} -u [-o options] node"
        ))
    );
}

pub(crate) fn mount(matches: &getopts::Matches) -> Result<(), Box<dyn std::error::Error>> {
    let mut mntflags = nix::mount::MntFlags::empty();
    let mut opts = vec![];

    if let Some(v) = matches.opt_str("o") {
        // Rust only supports "comma separated string of options",
        // since getopts can't take -o more than once.
        for &s in &v.split(',').collect::<Vec<&str>>() {
            if s.find('=').is_none() {
                // Ignore if None (e.g. "rw").
                if let Some(v) = libhammer2::os::get_mount_flag(s) {
                    mntflags.insert(v);
                }
                // e.g. hammer2_remount() uses "ro", not MNT_RDONLY.
                opts.push(libhammer2::util::new_cstring!(s)?);
            } else {
                return Err(Box::new(nix::errno::Errno::EINVAL));
            }
        }
    }
    let mut initflags = nix::mount::MntFlags::empty();
    if matches.opt_present("u") {
        initflags.insert(nix::mount::MntFlags::MNT_UPDATE);
        mntflags.insert(initflags);
    }

    let args: Vec<&str> = matches.free.iter().map(String::as_str).collect();
    // Only the mount point need be specified in update mode.
    let (cdev, cdir) = if initflags.contains(nix::mount::MntFlags::MNT_UPDATE) {
        if args.len() != 1 {
            log::error!("missing parameter (node)");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        (None, args[0])
    } else {
        if args.len() != 2 {
            log::error!("missing parameter(s) (special[@label] node)");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        (Some(args[0]), args[1])
    };

    let cdev = if let Some(cdev) = cdev {
        // Remove unnecessary slashes from the device path if any.
        let mut cdev = cdev.trim_end_matches('/').to_string();
        while cdev.contains("//") {
            cdev = cdev.replace("//", "/");
        }
        // Automatically add @DATA if no label specified.
        if cdev.find('@').is_none() {
            cdev = format!("{cdev}@DATA");
        }
        // Prefix if necessary.
        if !cdev.contains(':') && !cdev.starts_with('/') && !cdev.starts_with('@') {
            cdev = format!("/dev/{cdev}");
        }
        cdev
    } else {
        String::new()
    };
    // Resolve the mountpoint with realpath(3).
    let cdir = std::fs::canonicalize(cdir)?;

    let from = libhammer2::util::new_cstring!("from")?;
    let cdev = libhammer2::util::new_cstring!(&*cdev)?;
    // FreeBSD doesn't support hflags, but hflags itself is required.
    let hflags_str = libhammer2::util::new_cstring!("hflags")?;
    let mut hflags: u32 = 0;

    let mut m = nix::mount::Nmount::new();
    m.str_opt_owned("fstype", "hammer2")
        .str_opt_owned("fspath", &cdir)
        .str_opt(&from, &cdev);
    for s in &opts {
        m.null_opt(s);
    }
    unsafe {
        m.mut_ptr_opt(
            &hflags_str,
            std::ptr::from_mut::<u32>(&mut hflags).cast::<libc::c_void>(),
            std::mem::size_of_val(&hflags),
        )
    };
    Ok(m.nmount(mntflags)?)
}
