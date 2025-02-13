mod mkfs;

fn usage(prog: &str, gopt: &getopts::Options) {
    print!(
        "{}",
        gopt.usage(&format!(
            "usage: {prog} [-b bootsize] [-r auxsize] \
            [-V version] [-L label ...] [-s size] special ..."
        ))
    );
}

#[allow(clippy::too_many_lines)]
fn main() {
    if let Err(e) = hammer2_utils::util::init_std_logger() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let Some(prog) = &hammer2_utils::util::get_basename(&args[0]) else {
        log::error!("{args:?}");
        std::process::exit(1);
    };

    let mut gopt = getopts::Options::new();
    gopt.optopt(
        "b",
        "",
        "Specify a fixed area in which a boot related kernel and data can be \
        stored. The bootsize is specified in bytes. By default a boot area of \
        approximately 64MB will be created. This area is not currently used \
        for booting and may be repurposed in the future.",
        "<bootsize>",
    );
    gopt.optopt(
        "r",
        "",
        "Specify a fixed area in which an aux related kernel and data can be \
        stored. The auxsize is specified in bytes. By default an aux area of \
        approximately 256MB will be created. This area is not currently used \
        and may be repurposed in the future.",
        "<auxsize>",
    );
    gopt.optopt(
        "V",
        "",
        "Specify the HAMMER2 file system version to format. By default \
        newfs_hammer2 formats the file system using the highest production \
        version number supported by the HAMMER2 VFS by checking the \
        vfs.hammer2.supported_version sysctl. If you need to maintain \
        compatibility with an older version of HAMMER2 you may specify the \
        version with this option. If vfs.hammer2.supported_version sysctl \
        is unsupported, the current default version is selected.",
        "<version>",
    );
    gopt.optopt(
        "L",
        "",
        "By default newfs_hammer2 always creates a local master PFSs on the \
        new volume called \"LOCAL\" and \"DATA\". If you specify one or more \
        label options to create your own named local PFSs, newfs_hammer2 will \
        not create any conditional PFSs. However, \"LOCAL\" is still always \
        created and should not be specified with this option. If you don't \
        want any PFSs to be created (other than \"LOCAL\"), use -L none. \
        Use comma separated <label> to specify more than one labels.",
        "<label>",
    );
    gopt.optopt(
        "s",
        "",
        "The size of the file system in bytes. This value defaults to the \
        total size of the raw partitions specified in special (in other words, \
        newfs_hammer2 will use the entire partition for the file system). The \
        size must be 1GiB or larger.",
        "<size>",
    );
    gopt.optflag("d", "", "Enable debug flag");
    gopt.optflag("", "version", "Print version and exit");
    gopt.optflag("", "help", "Print usage and exit");

    let matches = match gopt.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            log::error!("{e}");
            usage(prog, &gopt);
            std::process::exit(1);
        }
    };
    if matches.opt_present("version") {
        hammer2_utils::util::print_version();
        std::process::exit(0);
    }
    if matches.opt_present("help") {
        usage(prog, &gopt);
        std::process::exit(0);
    }

    if !libhammer2::util::is_os_supported() {
        log::error!("{} is unsupported", libhammer2::util::get_os_name());
        std::process::exit(1);
    }

    let mut opt = mkfs::Opt::new();
    if let Some(v) = matches.opt_str("b") {
        opt.boot_area_size = match mkfs::get_size(
            &v,
            libhammer2::fs::HAMMER2_NEWFS_ALIGN,
            libhammer2::fs::HAMMER2_BOOT_MAX_BYTES,
            2,
        ) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{v}: {e}");
                std::process::exit(1);
            }
        };
    }
    if let Some(v) = matches.opt_str("r") {
        opt.aux_area_size = match mkfs::get_size(
            &v,
            libhammer2::fs::HAMMER2_NEWFS_ALIGN,
            libhammer2::fs::HAMMER2_AUX_MAX_BYTES,
            2,
        ) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{v}: {e}");
                std::process::exit(1);
            }
        };
    }
    if let Some(v) = matches.opt_str("V") {
        let v = match v.parse() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{v}: {e}");
                std::process::exit(1);
            }
        };
        if !(libhammer2::fs::HAMMER2_VOL_VERSION_MIN..libhammer2::fs::HAMMER2_VOL_VERSION_WIP)
            .contains(&v)
        {
            log::error!("I don't understand how to format HAMMER2 version {v}");
            std::process::exit(1);
        }
        opt.hammer2_version = v;
    }
    if let Some(v) = matches.opt_str("L") {
        if v.is_empty() {
            log::error!("Volume label '{v}' cannot be 0-length");
            std::process::exit(1);
        }
        // use comma separated labels as getopts can't take -L more than once
        for s in &v.split(',').collect::<Vec<&str>>() {
            if s.len() >= libhammer2::fs::HAMMER2_INODE_MAXNAME {
                log::error!(
                    "Volume label '{s}' is too long ({} chars max)",
                    libhammer2::fs::HAMMER2_INODE_MAXNAME - 1
                );
                std::process::exit(1);
            }
            if s.to_lowercase() != "none" {
                if opt.label.len() >= mkfs::MAXLABELS {
                    log::error!("Limit of {} local labels", mkfs::MAXLABELS - 1);
                    std::process::exit(1);
                }
                opt.label.push((*s).to_string());
            }
        }
    } else {
        assert!(opt.default_label_type.is_none());
        opt.default_label_type = Some(mkfs::Label::Data);
    }
    if let Some(v) = matches.opt_str("s") {
        if let Err(e) = opt.parse_fs_size(&v) {
            log::error!("{v}: {e}");
            std::process::exit(1);
        }
    }
    opt.debug = matches.opt_present("d");

    let args: Vec<&str> = matches.free.iter().map(String::as_str).collect();
    if args.is_empty() {
        log::error!("You must specify at least one disk device");
        std::process::exit(1);
    }
    if args.len() > libhammer2::fs::HAMMER2_MAX_VOLUMES.into() {
        log::error!(
            "The maximum number of volumes is {}",
            libhammer2::fs::HAMMER2_MAX_VOLUMES
        );
        std::process::exit(1);
    }

    assert_eq!(opt.comp_type, libhammer2::fs::HAMMER2_COMP_DEFAULT);
    assert_eq!(opt.check_type, libhammer2::fs::HAMMER2_CHECK_DEFAULT);
    if let Err(e) = mkfs::mkfs(&args, &mut opt) {
        log::error!("{e}");
        std::process::exit(1);
    }
}
