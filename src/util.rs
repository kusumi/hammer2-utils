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

/// # Panics
#[must_use]
pub fn get_basename(f: &str) -> String {
    std::path::Path::new(&f)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

const DEBUG: &str = "DEBUG";

#[must_use]
pub fn get_debug_level() -> i32 {
    match std::env::var(DEBUG) {
        Ok(v) => v.parse().unwrap_or(-1),
        Err(_) => -1,
    }
}

#[must_use]
pub fn is_debug_set() -> bool {
    get_debug_level() > 0
}

/// # Errors
pub fn init_std_logger() -> Result<(), log::SetLoggerError> {
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", if is_debug_set() { "trace" } else { "info" });
    env_logger::try_init_from_env(env)
}
