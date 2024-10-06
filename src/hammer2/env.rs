pub(crate) const SHOW_ALL_VOLUME_DATA: &str = "HAMMER2_SHOW_ALL_VOLUME_HEADERS";
pub(crate) const SHOW_TAB: &str = "HAMMER2_SHOW_TAB";
pub(crate) const SHOW_DEPTH: &str = "HAMMER2_SHOW_DEPTH";
pub(crate) const SHOW_MIN_MIRROR_TID: &str = "HAMMER2_SHOW_MIN_MIRROR_TID";
pub(crate) const SHOW_MIN_MODIFY_TID: &str = "HAMMER2_SHOW_MIN_MODIFY_TID";

pub(crate) fn get_show_all_volume_data() -> bool {
    is_non_zero(std::env::var(SHOW_ALL_VOLUME_DATA))
}

pub(crate) fn get_show_tab() -> usize {
    get_in_range(std::env::var(SHOW_TAB), 0, 8, 2)
}

pub(crate) fn get_show_depth() -> usize {
    get_in_range(std::env::var(SHOW_DEPTH), 0, usize::MAX, usize::MAX)
}

pub(crate) fn get_show_min_mirror_tid() -> u64 {
    get_in_radix(std::env::var(SHOW_MIN_MIRROR_TID), 16, 0)
}

pub(crate) fn get_show_min_modify_tid() -> u64 {
    get_in_radix(std::env::var(SHOW_MIN_MODIFY_TID), 16, 0)
}

fn is_non_zero(x: Result<String, std::env::VarError>) -> bool {
    if let Ok(v) = x {
        !matches!(v.parse::<isize>(), Ok(0) | Err(_))
    } else {
        false
    }
}

fn get_in_range<T>(x: Result<String, std::env::VarError>, min: T, max: T, def: T) -> T
where
    T: std::str::FromStr + PartialOrd,
{
    assert!(min <= max);
    if let Ok(v) = x {
        if let Ok(v) = v.parse() {
            if (min..=max).contains(&v) {
                v
            } else {
                def
            }
        } else {
            def
        }
    } else {
        def
    }
}

fn get_in_radix<T>(x: Result<String, std::env::VarError>, radix: u32, def: T) -> T
where
    T: num_traits::Num,
{
    if let Ok(v) = x {
        T::from_str_radix(&v, radix).unwrap_or(def)
    } else {
        def
    }
}

pub(crate) fn init() -> (bool, usize, usize, u64, u64) {
    (
        get_show_all_volume_data(),
        get_show_tab(),
        get_show_depth(),
        get_show_min_mirror_tid(),
        get_show_min_modify_tid(),
    )
}

#[cfg(test)]
mod tests {
    fn ok(s: &str) -> Result<String, std::env::VarError> {
        Ok(s.to_string())
    }

    fn err() -> Result<String, std::env::VarError> {
        Err(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_is_non_zero() {
        assert!(super::is_non_zero(ok("1")));
        assert!(super::is_non_zero(ok("10")));
        assert!(super::is_non_zero(ok("-1")));

        assert!(!super::is_non_zero(ok("")));
        assert!(!super::is_non_zero(ok("0")));
        assert!(!super::is_non_zero(ok("a")));

        assert!(!super::is_non_zero(err()));
    }

    #[test]
    fn test_get_in_range() {
        assert_eq!(super::get_in_range(ok(""), 0, 8, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_range(ok("0"), 0, 8, u64::MAX), 0);
        assert_eq!(super::get_in_range(ok("2"), 0, 8, u64::MAX), 2);
        assert_eq!(super::get_in_range(ok("8"), 0, 8, u64::MAX), 8);

        assert_eq!(super::get_in_range(ok(""), 0, 0, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_range(ok("0"), 0, 0, u64::MAX), 0);
        assert_eq!(super::get_in_range(ok("10"), 0, 8, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_range(ok("-10"), 0, 8, u64::MAX), u64::MAX);

        assert_eq!(super::get_in_range(err(), 0, 8, u64::MAX), u64::MAX);
    }

    #[test]
    fn test_get_in_radix() {
        assert_eq!(super::get_in_radix(ok(""), 8, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_radix(ok("0"), 8, u64::MAX), 0);
        assert_eq!(super::get_in_radix(ok("A"), 8, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_radix(ok("10"), 8, u64::MAX), 8);

        assert_eq!(super::get_in_radix(ok(""), 10, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_radix(ok("0"), 10, u64::MAX), 0);
        assert_eq!(super::get_in_radix(ok("A"), 10, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_radix(ok("10"), 10, u64::MAX), 10);

        assert_eq!(super::get_in_radix(ok(""), 16, u64::MAX), u64::MAX);
        assert_eq!(super::get_in_radix(ok("0"), 16, u64::MAX), 0);
        assert_eq!(super::get_in_radix(ok("A"), 16, u64::MAX), 10);
        assert_eq!(super::get_in_radix(ok("10"), 16, u64::MAX), 16);

        assert_eq!(super::get_in_radix(err(), 10, u64::MAX), u64::MAX);
    }
}
