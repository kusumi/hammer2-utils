#[must_use]
pub fn get_version_string() -> String {
    format!(
        "{}.{}.{}",
        libhammer2::VERSION[0],
        libhammer2::VERSION[1],
        libhammer2::VERSION[2]
    )
}

pub fn print_version() {
    println!("{}", get_version_string());
}

/// # Errors
pub fn init_std_logger() -> Result<(), log::SetLoggerError> {
    let env = env_logger::Env::default().filter_or(
        "RUST_LOG",
        if libfs::is_debug_set() {
            "trace"
        } else {
            "info"
        },
    );
    env_logger::try_init_from_env(env)
}
