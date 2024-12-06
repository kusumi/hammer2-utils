mod cmd;
mod env;
mod show;

use hammer2_utils::util;

#[derive(Debug, Default)]
pub(crate) struct Hammer2Options {
    pub(crate) verbose: bool,
    pub(crate) quiet: bool,
    pub(crate) recurse: bool,
    pub(crate) pfs_type: u8,
    pub(crate) uuid_str: Option<String>,
    pub(crate) mem: usize,
}

impl Hammer2Options {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

fn get_string_size(v: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let s = &v[v.len() - 1..];
    let unit = match s {
        "k" | "K" => libhammer2::subs::K,
        "m" | "M" => libhammer2::subs::M,
        "g" | "G" => libhammer2::subs::G,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => 1,
        _ => {
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
    };
    let s = if unit > 1 { &v[..v.len() - 1] } else { v };
    match s.parse::<usize>() {
        Ok(v) => Ok(v * unit),
        Err(e) => Err(Box::new(e)),
    }
}

fn usage(prog: &str, gopt: &getopts::Options) {
    let indent = "    ";
    let ampersand = "&";
    print!(
        "{}",
        gopt.usage(&format!(
            "{prog} [options] command [argument ...]\n\
            {indent}cleanup [<path>]                  \
            Run cleanup passes\n\
            {indent}destroy <path>...                 \
            Destroy directory entries (only use if inode bad)\n\
            {indent}destroy-inum <inum>...            \
            Destroy inodes (only use if inode bad)\n\
            {indent}emergency-mode-enable <target>    \
            Enable emergency operations mode on filesystem\n\
            {indent}                                  \
            THIS IS A VERY DANGEROUS MODE\n\
            {indent}emergency-mode-disable <target>   \
            Disable emergency operations mode on filesystem\n\
            {indent}hash [<filename>...]              \
            Print directory hash (key) for name\n\
            {indent}dhash [<filename>...]             \
            Print data hash for long directory entry\n\
            {indent}pfs-list [<path>...]              \
            List PFSs\n\
            {indent}pfs-clid <label>                  \
            Print cluster id for specific PFS\n\
            {indent}pfs-fsid <label>                  \
            Print private id for specific PFS\n\
            {indent}pfs-create <label>                \
            Create a PFS\n\
            {indent}pfs-delete <label>                \
            Destroy a PFS\n\
            {indent}recover <devpath> <path> <destdir> \
            Recover deleted or corrupt files or trees\n\
            {indent}recover-relaxed <devpath> <path> <destdir> \
            Recover deleted or corrupt files or trees\n\
            {indent}recover-file <devpath> <path> <destdir> \
            Recover, target is explicitly a regular file\n\
            {indent}snapshot <path> [<label>]         \
            Snapshot a PFS or directory\n\
            {indent}snapshot-debug <path> [<label>]   \
            Snapshot without filesystem sync\n\
            {indent}stat [<path>...]                  \
            Return inode quota & config\n\
            {indent}growfs [<path>...]                \
            Grow a filesystem into resized partition\n\
            {indent}show <devpath>                    \
            Raw hammer2 media dump for topology\n\
            {indent}freemap <devpath>                 \
            Raw hammer2 media dump for freemap\n\
            {indent}volhdr <devpath>                  \
            Raw hammer2 media dump for the volume header(s)\n\
            {indent}volume-list [<path>...]           \
            List volumes\n\
            {indent}setcomp <comp[:level]> <path>...  \
            Set comp algo {{none, autozero, lz4, zlib}} {ampersand} level\n\
            {indent}setcheck <check> <path>...        \
            Set check algo {{none, crc32, xxhash64, sha192}}\n\
            {indent}clrcheck [<path>...]              \
            Clear check code override\n\
            {indent}setcrc32 [<path>...]              \
            Set check algo to crc32\n\
            {indent}setxxhash64 [<path>...]           \
            Set check algo to xxhash64\n\
            {indent}setsha192 [<path>...]             \
            Set check algo to sha192\n\
            {indent}bulkfree <path>                   \
            Run bulkfree pass\n\
            {indent}printinode <path>                 \
            Dump inode\n\
            {indent}dumpchain [<path>]                \
            Dump in-memory chain topology"
        ))
    );
}

fn main() {
    if let Err(e) = util::init_std_logger() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let prog = &util::get_basename(&args[0]);

    let mut gopt = getopts::Options::new();
    gopt.optflag("v", "", "Enable verbose flag");
    gopt.optflag("q", "", "Enable quiet flag");
    gopt.optflag("r", "", "Enable recurse flag");
    gopt.optopt("s", "", "Select filesystem", "<path>");
    gopt.optopt("t", "", "PFS type for pfs-create", "<type>");
    gopt.optopt("u", "", "uuid for pfs-create", "<uuid>");
    gopt.optopt("m", "", "buffer memory (bulkfree)", "<mem[k,m,g]>");
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
        util::print_version();
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

    let mut opt = Hammer2Options::new();
    if matches.opt_present("v") {
        if opt.quiet {
            opt.quiet = false;
        } else {
            opt.verbose = true;
        }
    }
    if matches.opt_present("q") {
        if opt.verbose {
            opt.verbose = false;
        } else {
            opt.quiet = true;
        }
    }
    opt.recurse = matches.opt_present("r");
    let sel_path_binding = matches.opt_str("s").unwrap_or_default();
    let sel_path = sel_path_binding.as_str();
    opt.pfs_type = if let Some(v) = matches.opt_str("t") {
        match v.to_uppercase().as_str() {
            "CACHE" => libhammer2::fs::HAMMER2_PFSTYPE_CACHE,
            "SLAVE" => libhammer2::fs::HAMMER2_PFSTYPE_SLAVE,
            "SOFT_SLAVE" => libhammer2::fs::HAMMER2_PFSTYPE_SOFT_SLAVE,
            "SOFT_MASTER" => libhammer2::fs::HAMMER2_PFSTYPE_SOFT_MASTER,
            "MASTER" => libhammer2::fs::HAMMER2_PFSTYPE_MASTER,
            "DUMMY" => libhammer2::fs::HAMMER2_PFSTYPE_DUMMY,
            _ => {
                log::error!("Unrecognized node type {v}");
                std::process::exit(1);
            }
        }
    } else {
        libhammer2::fs::HAMMER2_PFSTYPE_NONE
    };
    opt.uuid_str = matches.opt_str("u");
    opt.mem = match matches.opt_str("m") {
        Some(v) => match get_string_size(&v) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{e}");
                std::process::exit(1);
            }
        },
        None => 0,
    };

    let args: Vec<&str> = matches.free.iter().map(String::as_str).collect();
    if args.is_empty() {
        log::error!("Missing command");
        usage(prog, &gopt);
        std::process::exit(1);
    }
    if let Err(e) = cmd_run(args[0], &args[1..], sel_path, &opt) {
        log::error!("{e}");
        if let Ok(v) = e.downcast::<nix::errno::Errno>() {
            if *v == nix::errno::Errno::EINVAL {
                usage(prog, &gopt);
            }
        }
        std::process::exit(1);
    }
}

fn cmd_run(
    cmd: &str,
    args: &[&str],
    sel_path: &str,
    opt: &Hammer2Options,
) -> Result<(), Box<dyn std::error::Error>> {
    if cmd == "cleanup" {
        let f = if args.is_empty() { None } else { Some(args[0]) };
        cmd::cleanup::run(f, opt)
    } else if cmd == "destroy" {
        if args.is_empty() {
            log::error!("Specify one or more paths to destroy");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::destroy::run(args)
    } else if cmd == "destroy-inum" {
        if args.is_empty() {
            log::error!("Specify one or more inode numbers to destroy");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::destroy_inum::run(sel_path, args)
    } else if cmd == "emergency-mode-enable" {
        if args.is_empty() {
            log::error!("Requires filesystem path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::emergency_mode::run(args[0], true)
    } else if cmd == "emergency-mode-disable" {
        if args.is_empty() {
            log::error!("Requires filesystem path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::emergency_mode::run(args[0], false)
    } else if cmd == "hash" {
        cmd::hash::run(args);
        Ok(())
    } else if cmd == "dhash" {
        cmd::dhash::run(args);
        Ok(())
    } else if cmd == "pfs-clid" {
        if args.is_empty() {
            log::error!("Requires name");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::pfs_id::run(sel_path, args[0], false)
    } else if cmd == "pfs-fsid" {
        if args.is_empty() {
            log::error!("Requires name");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::pfs_id::run(sel_path, args[0], true)
    } else if cmd == "pfs-list" {
        let args = if args.is_empty() { &[sel_path] } else { args };
        cmd::pfs_list::run(args)
    } else if cmd == "pfs-create" {
        if args.is_empty() {
            log::error!("Requires name");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::pfs_create::run(sel_path, args[0], opt)
    } else if cmd == "pfs-delete" {
        if args.is_empty() {
            log::error!("Requires name");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::pfs_delete::run(sel_path, args)
    } else if cmd == "recover" || cmd == "recover-relaxed" || cmd == "recover-file" {
        if args.len() != 3 {
            log::error!("Recover device [/]path destdir");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::recover::run(
            args[0],
            args[1],
            args[2],
            cmd != "recover-relaxed",
            cmd == "recover-file",
            opt,
        )
    } else if cmd == "snapshot" || cmd == "snapshot-debug" {
        let flags = if cmd == "snapshot-debug" {
            libhammer2::ioctl::HAMMER2_PFSFLAGS_NOSYNC
        } else {
            0
        };
        if args.len() > 2 {
            log::error!("Too many arguments");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::snapshot::run(sel_path, args, flags)
    } else if cmd == "stat" {
        let args = if args.is_empty() { &["."] } else { args };
        cmd::stat::run(args)
    } else if cmd == "growfs" {
        let args = if args.is_empty() { &[sel_path] } else { args };
        cmd::growfs::run(args)
    } else if cmd == "show" {
        if args.len() != 1 {
            log::error!("Requires device path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::show::run(args[0], opt)
    } else if cmd == "freemap" {
        if args.len() != 1 {
            log::error!("Requires device path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::freemap::run(args[0], opt)
    } else if cmd == "volhdr" {
        if args.len() != 1 {
            log::error!("Requires device path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::volhdr::run(args[0], opt)
    } else if cmd == "volume-list" {
        let args = if args.is_empty() { &[sel_path] } else { args };
        cmd::volume_list::run(args, opt)
    } else if cmd == "setcomp" {
        if args.len() < 2 {
            log::error!("Requires compression method and directory/file path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::setcomp::run(args[0], &args[1..], opt)
    } else if cmd == "setcheck" {
        if args.len() < 2 {
            log::error!("Requires check code method and directory/file path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::setcheck::run(args[0], &args[1..], opt)
    } else if cmd == "clrcheck" {
        cmd::setcheck::run("none", &args[1..], opt)
    } else if cmd == "setcrc32" {
        cmd::setcheck::run("crc32", &args[1..], opt)
    } else if cmd == "setxxhash64" {
        cmd::setcheck::run("xxhash64", &args[1..], opt)
    } else if cmd == "setsha192" {
        cmd::setcheck::run("sha192", &args[1..], opt)
    } else if cmd == "bulkfree" {
        if args.len() != 1 {
            log::error!("Requires path to mount");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::bulkfree::run(args[0], opt)
    } else if cmd == "printinode" {
        if args.len() != 1 {
            log::error!("Requires directory/file path");
            return Err(Box::new(nix::errno::Errno::EINVAL));
        }
        cmd::printinode::run(args[0], opt)
    } else if cmd == "dumpchain" {
        let f = if args.is_empty() { "." } else { args[0] };
        cmd::dumpchain::run(f)
    } else {
        log::error!("Unrecognized command: {cmd}");
        Err(Box::new(nix::errno::Errno::EINVAL))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_string_size() {
        // 0
        match super::get_string_size("0") {
            Ok(v) => assert_eq!(v, 0),
            Err(e) => panic!("{e}"),
        }
        match super::get_string_size("00") {
            Ok(v) => assert_eq!(v, 0),
            Err(e) => panic!("{e}"),
        }

        // 1
        match super::get_string_size("1") {
            Ok(v) => assert_eq!(v, 1),
            Err(e) => panic!("{e}"),
        }
        match super::get_string_size("01") {
            Ok(v) => assert_eq!(v, 1),
            Err(e) => panic!("{e}"),
        }

        // K
        match super::get_string_size("1k") {
            Ok(v) => assert_eq!(v, 1 << 10),
            Err(e) => panic!("{e}"),
        }
        match super::get_string_size("2K") {
            Ok(v) => assert_eq!(v, (1 << 10) * 2),
            Err(e) => panic!("{e}"),
        }
        assert!(super::get_string_size("k").is_err());
        assert!(super::get_string_size("k1").is_err());

        // M
        match super::get_string_size("1m") {
            Ok(v) => assert_eq!(v, 1 << 20),
            Err(e) => panic!("{e}"),
        }
        match super::get_string_size("2M") {
            Ok(v) => assert_eq!(v, (1 << 20) * 2),
            Err(e) => panic!("{e}"),
        }
        assert!(super::get_string_size("m").is_err());
        assert!(super::get_string_size("m1").is_err());

        // G
        match super::get_string_size("1g") {
            Ok(v) => assert_eq!(v, 1 << 30),
            Err(e) => panic!("{e}"),
        }
        match super::get_string_size("2G") {
            Ok(v) => assert_eq!(v, (1 << 30) * 2),
            Err(e) => panic!("{e}"),
        }
        assert!(super::get_string_size("g").is_err());
        assert!(super::get_string_size("g1").is_err());

        // other
        assert!(super::get_string_size("xxx").is_err());
    }
}
