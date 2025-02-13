#[macro_export]
macro_rules! format {
    ($n: expr, $($args: tt)*) => {
        format!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*))
    }
}
pub use format;

#[macro_export]
macro_rules! print {
    ($n: expr, $($args: tt)*) => {
        print!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use print;

#[macro_export]
macro_rules! println {
    ($n: expr, $($args: tt)*) => {
        println!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use println;

#[macro_export]
macro_rules! eprint {
    ($n: expr, $($args: tt)*) => {
        eprint!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use eprint;

#[macro_export]
macro_rules! eprintln {
    ($n: expr, $($args: tt)*) => {
        eprintln!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use eprintln;

#[macro_export]
macro_rules! warning {
    ($n: expr, $($args: tt)*) => {
        log::warn!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use warning;

#[macro_export]
macro_rules! error {
    ($n: expr, $($args: tt)*) => {
        log::error!("{}{}", " ".repeat($n * TAB_INDENT), format!($($args)*));
    }
}
pub use error;
