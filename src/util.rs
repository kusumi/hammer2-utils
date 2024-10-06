use std::io::Seek;

pub const VERSION: [i32; 3] = [0, 1, 0];

#[must_use]
pub fn get_version_string() -> String {
    format!("{}.{}.{}", VERSION[0], VERSION[1], VERSION[2])
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

/// # Panics
#[must_use]
pub fn get_current_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// # Errors
pub fn open(path: &str, readonly: bool) -> std::io::Result<std::fs::File> {
    if readonly {
        std::fs::File::open(path)
    } else {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
    }
}

/// # Errors
pub fn seek_set(fp: &mut std::fs::File, offset: u64) -> std::io::Result<u64> {
    fp.seek(std::io::SeekFrom::Start(offset))
}

/// # Errors
pub fn seek_end(fp: &mut std::fs::File, offset: i64) -> std::io::Result<u64> {
    fp.seek(std::io::SeekFrom::End(offset))
}

/// # Errors
pub fn seek_cur(fp: &mut std::fs::File, offset: i64) -> std::io::Result<u64> {
    fp.seek(std::io::SeekFrom::Current(offset))
}

/// # Panics
#[must_use]
pub fn align_to<T>(buf: &[u8]) -> &T {
    let (prefix, body, suffix) = unsafe { buf.align_to::<T>() };
    assert!(prefix.is_empty());
    assert!(suffix.is_empty());
    &body[0]
}

/// # Panics
pub fn align_to_mut<T>(buf: &mut [u8]) -> &mut T {
    let (prefix, body, suffix) = unsafe { buf.align_to_mut::<T>() };
    assert!(prefix.is_empty());
    assert!(suffix.is_empty());
    &mut body[0]
}

/// # Safety
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            std::ptr::from_ref::<T>(p).cast::<u8>(),
            ::core::mem::size_of::<T>(),
        )
    }
}

#[must_use]
pub fn notfound() -> std::io::Error {
    std::io::Error::from(std::io::ErrorKind::NotFound)
}

#[must_use]
pub fn get_os_name() -> &'static str {
    std::env::consts::OS
}

#[must_use]
pub fn is_os_supported() -> bool {
    is_linux() || is_freebsd()
}

#[must_use]
pub fn is_linux() -> bool {
    get_os_name() == "linux"
}

#[must_use]
pub fn is_freebsd() -> bool {
    get_os_name() == "freebsd"
}

#[must_use]
pub fn is_solaris() -> bool {
    get_os_name() == "solaris"
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
