mod os;

use hammer2_utils::util;

fn main() {
    if let Err(e) = util::init_std_logger() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let prog = &util::get_basename(&args[0]);

    let mut gopt = os::get_getopts();
    gopt.optflag("", "version", "Print version and exit");
    gopt.optflag("", "help", "Print usage and exit");

    let matches = match gopt.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            log::error!("{e}");
            os::usage(prog, &gopt);
            std::process::exit(1);
        }
    };
    if matches.opt_present("version") {
        util::print_version();
        std::process::exit(0);
    }
    if matches.opt_present("help") {
        os::usage(prog, &gopt);
        std::process::exit(0);
    }

    if let Err(e) = os::mount(&matches) {
        log::error!("{e}");
        std::process::exit(1);
    }
}
