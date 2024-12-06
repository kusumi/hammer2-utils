use crate::Hammer2Options;

pub(crate) fn run(
    _devpath: &str,
    _pathname: &str,
    _destdir: &str,
    _strict: bool,
    _isafile: bool,
    _opt: &Hammer2Options,
) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::new(nix::errno::Errno::EOPNOTSUPP)) // XXX
}
