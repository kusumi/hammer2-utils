mod cmd_dhash;
mod cmd_freemap;
mod cmd_hash;
mod cmd_show;
mod cmd_volhdr;
mod env;
mod show;

use hammer2_utils::util;

#[derive(Debug, Default)]
pub(crate) struct Hammer2Options {
    pub(crate) verbose: bool,
    pub(crate) quiet: bool,
}

impl Hammer2Options {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

fn usage(prog: &str, gopt: &getopts::Options) {
    let s = "    ";
    print!(
        "{}",
        gopt.usage(&format!(
            "usage: {prog} [options] command [argument ...]\n\
            {s}hash [<filename>...]              \
            Print directory hash (key) for name \n\
            {s}dhash [<filename>...]             \
            Print data hash for long directory entry \n\
            {s}show <devpath>                    \
            Raw hammer2 media dump for topology\n\
            {s}freemap <devpath>                 \
            Raw hammer2 media dump for freemap\n\
            {s}volhdr <devpath>                  \
            Raw hammer2 media dump for the volume header(s)"
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

    let args = &matches.free;
    if args.is_empty() {
        log::error!("Missing command");
        usage(prog, &gopt);
        std::process::exit(1);
    }

    let cmd = &args[0];
    let args = &args[1..];
    if cmd == "hash" {
        cmd_hash::run(args);
    } else if cmd == "dhash" {
        cmd_dhash::run(args);
    } else if cmd == "show" {
        if args.is_empty() {
            log::error!("Requires device path");
            std::process::exit(1);
        }
        if let Err(e) = cmd_show::run(&args[0], &opt) {
            log::error!("{e}");
            std::process::exit(1);
        }
    } else if cmd == "freemap" {
        if args.is_empty() {
            log::error!("Requires device path");
            std::process::exit(1);
        }
        if let Err(e) = cmd_freemap::run(&args[0], &opt) {
            log::error!("{e}");
            std::process::exit(1);
        }
    } else if cmd == "volhdr" {
        if args.is_empty() {
            log::error!("Requires device path");
            std::process::exit(1);
        }
        if let Err(e) = cmd_volhdr::run(&args[0], &opt) {
            log::error!("{e}");
            std::process::exit(1);
        }
    } else {
        log::error!("Unrecognized command: {cmd}");
        usage(prog, &gopt);
        std::process::exit(1);
    }
}
