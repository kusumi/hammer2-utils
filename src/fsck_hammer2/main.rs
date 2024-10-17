mod fsck;

use hammer2_utils::util;

#[derive(Debug, Default)]
struct Hammer2FsckOptions {
    verbose: bool,
    quiet: bool,
    debug: bool,
    force: bool,
    count_empty: bool,
    scan_best: bool,
    scan_pfs: bool,
    print_pfs: bool,
    pfs_names: Vec<String>,
    blockref_cache_count: usize,
}

impl Hammer2FsckOptions {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

fn usage(prog: &str, gopt: &getopts::Options) {
    print!(
        "{}",
        gopt.usage(&format!(
            "usage: {prog} [-f] [-v] [-q] [-e] [-b] [-p] [-P] \
            [-l pfs_names] [-c cache_count] special"
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
    gopt.optflag("d", "", "Enable debug flag");
    gopt.optflag("f", "", "Enable force flag");
    gopt.optflag("e", "", "Count empty blockrefs");
    gopt.optflag("b", "", "Scan only best zone");
    gopt.optflag("p", "", "Scan each PFS separately");
    gopt.optflag("P", "", "Print PFS information");
    gopt.optopt("l", "", "Specify PFS names when -p is used", "<pfs_names>");
    gopt.optopt("c", "", "Specify blockref cache count", "<cache_count>");
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

    if !util::is_os_supported() {
        log::error!("{} is unsupported", util::get_os_name());
        std::process::exit(1);
    }

    let mut opt = Hammer2FsckOptions::new();
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
    opt.debug = matches.opt_present("d");
    opt.force = matches.opt_present("f");
    opt.count_empty = matches.opt_present("e");
    opt.scan_best = matches.opt_present("b");
    opt.scan_pfs = matches.opt_present("p");
    opt.print_pfs = matches.opt_present("P");
    if let Some(v) = matches.opt_str("l") {
        for (i, s) in v.split(',').collect::<Vec<&str>>().iter().enumerate() {
            if opt.debug {
                println!("PFSNames[{i}]=\"{s}\"");
            }
            opt.pfs_names.push((*s).to_string());
        }
    }
    if let Some(v) = matches.opt_str("c") {
        opt.blockref_cache_count = match v.parse() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{v}: {e}");
                std::process::exit(1);
            }
        }
    }

    let args = &matches.free;
    if args.is_empty() {
        usage(prog, &gopt);
        std::process::exit(1);
    }

    for (i, s) in args.iter().enumerate() {
        if args.len() != 1 {
            println!("{s}");
        }
        if let Err(e) = fsck::fsck(s, &opt) {
            log::error!("{e}");
            std::process::exit(1);
        }
        if i != args.len() - 1 {
            println!(
                "----------------------------------------\
		        ----------------------------------------"
            );
        }
    }
}
