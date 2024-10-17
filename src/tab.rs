#[macro_export]
macro_rules! format {
    ($tab: expr, $($args: tt)*) => {
        format!("{}{}", " ".repeat($tab * TAB_INDENT), format!($($args)*))
    }
}
pub use format;

#[macro_export]
macro_rules! print {
    ($tab: expr, $($args: tt)*) => {
        print!("{}", " ".repeat($tab * TAB_INDENT));
        print!($($args)*);
    }
}
pub use print;

#[macro_export]
macro_rules! println {
    ($tab: expr, $($args: tt)*) => {
        print!("{}", " ".repeat($tab * TAB_INDENT));
        println!($($args)*);
    }
}
pub use println;

#[macro_export]
macro_rules! eprint {
    ($tab: expr, $($args: tt)*) => {
        eprint!("{}", " ".repeat($tab * TAB_INDENT));
        eprint!($($args)*);
    }
}
pub use eprint;

#[macro_export]
macro_rules! eprintln {
    ($tab: expr, $($args: tt)*) => {
        eprint!("{}", " ".repeat($tab * TAB_INDENT));
        eprintln!($($args)*);
    }
}
pub use eprintln;
