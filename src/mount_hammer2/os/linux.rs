pub(crate) fn get_getopts() -> getopts::Options {
    getopts::Options::new()
}

pub(crate) fn usage(prog: &str, gopt: &getopts::Options) {
    print!("{}", gopt.usage(&format!("usage: {prog}")));
}

pub(crate) fn mount(_matches: &getopts::Matches) -> hammer2_utils::Result<()> {
    log::error!("{} is unsupported", libhammer2::util::get_os_name());
    Err(Box::new(nix::errno::Errno::EOPNOTSUPP))
}
