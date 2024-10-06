use crate::hammer2fs;
use crate::util;

use std::os::unix::fs::FileTypeExt;

pub const K: usize = 1024;
pub const M: usize = K * 1024;
pub const G: usize = M * 1024;
pub const T: usize = G * 1024;

pub const K_U64: u64 = K as u64;
pub const M_U64: u64 = M as u64;
pub const G_U64: u64 = G as u64;
pub const T_U64: u64 = T as u64;

pub const K_F64: f64 = K as f64;
pub const M_F64: f64 = M as f64;
pub const G_F64: f64 = G as f64;
pub const T_F64: f64 = T as f64;

const FORCE_STD_EPOCH: &str = "HAMMER2_FORCE_STD_EPOCH";

/// # Panics
#[must_use]
pub fn get_local_time_string(t: u64) -> String {
    let mut d = i64::from(
        time::UtcOffset::current_local_offset()
            .unwrap()
            .whole_seconds(),
    );
    if t == 0 && std::env::var(FORCE_STD_EPOCH).is_ok() {
        d -= 3600;
    }
    get_time_string_impl(t, d)
}

#[must_use]
pub fn get_time_string(t: u64) -> String {
    get_time_string_impl(t, 0)
}

/// # Panics
#[must_use]
pub fn get_time_string_impl(t: u64, d: i64) -> String {
    let t = i64::try_from(t / 1_000_000).unwrap() + d;
    let t = if t < 0 {
        std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_secs((-t).try_into().unwrap())
    } else {
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(t.try_into().unwrap())
    };
    let fmt =
        time::format_description::parse("[day]-[month repr:short]-[year] [hour]:[minute]:[second]")
            .unwrap();
    time::OffsetDateTime::from(t).format(&fmt).unwrap()
}

#[must_use]
pub fn get_inode_type_string(typ: u8) -> String {
    match typ {
        hammer2fs::HAMMER2_OBJTYPE_UNKNOWN => "UNKNOWN",
        hammer2fs::HAMMER2_OBJTYPE_DIRECTORY => "DIR",
        hammer2fs::HAMMER2_OBJTYPE_REGFILE => "FILE",
        hammer2fs::HAMMER2_OBJTYPE_FIFO => "FIFO",
        hammer2fs::HAMMER2_OBJTYPE_CDEV => "CDEV",
        hammer2fs::HAMMER2_OBJTYPE_BDEV => "BDEV",
        hammer2fs::HAMMER2_OBJTYPE_SOFTLINK => "SOFTLINK",
        hammer2fs::HAMMER2_OBJTYPE_SOCKET => "SOCKET",
        hammer2fs::HAMMER2_OBJTYPE_WHITEOUT => "WHITEOUT",
        _ => "ILLEGAL",
    }
    .to_string()
}

#[must_use]
pub fn get_pfs_type_string(typ: u8) -> String {
    match typ {
        hammer2fs::HAMMER2_PFSTYPE_NONE => "NONE",
        hammer2fs::HAMMER2_PFSTYPE_SUPROOT => "SUPROOT",
        hammer2fs::HAMMER2_PFSTYPE_DUMMY => "DUMMY",
        hammer2fs::HAMMER2_PFSTYPE_CACHE => "CACHE",
        hammer2fs::HAMMER2_PFSTYPE_SLAVE => "SLAVE",
        hammer2fs::HAMMER2_PFSTYPE_SOFT_SLAVE => "SOFT_SLAVE",
        hammer2fs::HAMMER2_PFSTYPE_SOFT_MASTER => "SOFT_MASTER",
        hammer2fs::HAMMER2_PFSTYPE_MASTER => "MASTER",
        _ => "ILLEGAL",
    }
    .to_string()
}

#[must_use]
pub fn get_pfs_subtype_string(typ: u8) -> String {
    match typ {
        hammer2fs::HAMMER2_PFSSUBTYPE_NONE => "NONE",
        hammer2fs::HAMMER2_PFSSUBTYPE_SNAPSHOT => "SNAPSHOT",
        hammer2fs::HAMMER2_PFSSUBTYPE_AUTOSNAP => "AUTOSNAP",
        _ => "ILLEGAL",
    }
    .to_string()
}

#[must_use]
pub fn get_blockref_type_string(typ: u8) -> String {
    match typ {
        hammer2fs::HAMMER2_BREF_TYPE_EMPTY => "empty",
        hammer2fs::HAMMER2_BREF_TYPE_INODE => "inode",
        hammer2fs::HAMMER2_BREF_TYPE_INDIRECT => "indirect",
        hammer2fs::HAMMER2_BREF_TYPE_DATA => "data",
        hammer2fs::HAMMER2_BREF_TYPE_DIRENT => "dirent",
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_NODE => "freemap_node",
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP_LEAF => "freemap_leaf",
        hammer2fs::HAMMER2_BREF_TYPE_INVALID => "invalid",
        hammer2fs::HAMMER2_BREF_TYPE_FREEMAP => "freemap",
        hammer2fs::HAMMER2_BREF_TYPE_VOLUME => "volume",
        _ => "unknown",
    }
    .to_string()
}

const HAMMER2_CHECK_STRINGS: [&str; 5] = ["none", "disabled", "crc32", "xxhash64", "sha192"];
const HAMMER2_COMP_STRINGS: [&str; 4] = ["none", "autozero", "lz4", "zlib"];

// Note: Check algorithms normally do not encode any level.
#[must_use]
pub fn get_check_mode_string(x: u8) -> String {
    let check = usize::from(hammer2fs::dec_algo(x));
    let level = hammer2fs::dec_level(x);
    if level != 0 {
        if check < HAMMER2_CHECK_STRINGS.len() {
            format!("{}:{level}", HAMMER2_CHECK_STRINGS[check])
        } else {
            format!("unknown({check}):{level}")
        }
    } else if true {
        if check < HAMMER2_CHECK_STRINGS.len() {
            HAMMER2_CHECK_STRINGS[check].to_string()
        } else {
            format!("unknown({check})")
        }
    } else {
        unreachable!();
    }
    .to_string()
}

#[must_use]
pub fn get_comp_mode_string(x: u8) -> String {
    let comp = usize::from(hammer2fs::dec_algo(x));
    let level = hammer2fs::dec_level(x);
    if level != 0 {
        if comp < HAMMER2_COMP_STRINGS.len() {
            format!("{}:{level}", HAMMER2_COMP_STRINGS[comp])
        } else {
            format!("unknown({comp}):{level}")
        }
    } else if true {
        if comp < HAMMER2_COMP_STRINGS.len() {
            format!("{}:default", HAMMER2_COMP_STRINGS[comp])
        } else {
            format!("unknown({comp}):default")
        }
    } else {
        unreachable!();
    }
    .to_string()
}

#[must_use]
pub fn get_size_string(size: u64) -> String {
    if size < K_U64 / 2 {
        format!("{:6.2}B", size as f64)
    } else if size < M_U64 / 2 {
        format!("{:6.2}KB", size as f64 / K_F64)
    } else if size < G_U64 / 2 {
        format!("{:6.2}MB", size as f64 / M_F64)
    } else if size < T_U64 / 2 {
        format!("{:6.2}GB", size as f64 / G_F64)
    } else {
        format!("{:6.2}TB", size as f64 / T_F64)
    }
}

/// # Errors
pub fn get_volume_size_from_path(f: &str) -> std::io::Result<u64> {
    get_volume_size(&mut std::fs::File::open(f)?)
}

/// # Errors
pub fn get_volume_size(fp: &mut std::fs::File) -> std::io::Result<u64> {
    let t = fp.metadata()?.file_type();
    if !t.is_block_device() && !t.is_char_device() && !t.is_file() {
        log::error!("{fp:?}: unsupported type {t:?}");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    if util::is_linux() || util::is_freebsd() || util::is_solaris() {
        let size = util::seek_end(fp, 0)?;
        if size == 0 {
            log::error!("{fp:?}: failed to get size");
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }
        util::seek_set(fp, 0)?;
        Ok(size)
    } else {
        // XXX other platforms use ioctl(2)
        log::error!("{} is unsupported", util::get_os_name());
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

// Borrow HAMMER1's directory hash algorithm #1 with a few modifications.
// The filename is split into fields which are hashed separately and then
// added together.
//
// Differences include: bit 63 must be set to 1 for HAMMER2 (HAMMER1 sets
// it to 0), this is because bit63=0 is used for hidden hardlinked inodes.
// (This means we do not need to do a 0-check/or-with-0x100000000 either).
//
// Also, the iscsi crc code is used instead of the old crc32 code.
#[must_use]
pub fn dirhash(aname: &[u8]) -> u64 {
    // m32
    let mut crcx = 0;
    let mut i = 0;
    let mut j = 0;
    while i < aname.len() {
        let x = aname[i] as char;
        if x == '.' || x == '-' || x == '_' || x == '~' {
            if i != j {
                crcx += icrc32::iscsi_crc32(&aname[j..i]);
            }
            j = i + 1;
        }
        i += 1;
    }
    if i != j {
        crcx += icrc32::iscsi_crc32(&aname[j..i]);
    }

    // The directory hash utilizes the top 32 bits of the 64-bit key.
    // Bit 63 must be set to 1.
    crcx |= 0x8000_0000;
    let mut key = u64::from(crcx) << 32;

    // l16 - crc of entire filename
    // This crc reduces degenerate hash collision conditions.
    let mut crcx = icrc32::iscsi_crc32(aname);
    crcx = crcx ^ (crcx << 16);
    key |= u64::from(crcx) & 0xFFFF_0000;

    // Set bit 15.  This allows readdir to strip bit 63 so a positive
    // 64-bit cookie/offset can always be returned, and still guarantee
    // that the values 0x0000-0x7FFF are available for artificial entries
    // ('.' and '..').
    key | 0x8000
}

/// # Panics
#[must_use]
pub fn get_uuid_from_str(s: &str) -> uuid::Uuid {
    let src = *uuid::Uuid::parse_str(s).unwrap().as_bytes();
    let mut dst = src;
    dst[0] = src[3]; // 4
    dst[1] = src[2];
    dst[2] = src[1];
    dst[3] = src[0];
    dst[4] = src[5]; // 2
    dst[5] = src[4];
    dst[6] = src[7]; // 2
    dst[7] = src[6];
    uuid::Uuid::from_bytes(dst)
}

#[must_use]
pub fn get_uuid_string(u: &uuid::Uuid) -> String {
    get_uuid_string_from_bytes(u.as_bytes())
}

#[must_use]
pub fn get_uuid_string_from_bytes(b: &[u8]) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[3], b[2], b[1], b[0], // 4
        b[5], b[4], // 2
        b[7], b[6], // 2
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15])
}

#[cfg(test)]
mod tests {
    use crate::hammer2fs;
    use crate::util;

    #[test]
    fn test_get_time_string() {
        assert_eq!(super::get_time_string(0), "01-Jan-1970 00:00:00");
        assert_eq!(super::get_time_string(1_000_000), "01-Jan-1970 00:00:01");
    }

    #[test]
    fn test_get_time_string_impl() {
        assert_eq!(
            super::get_time_string_impl(0, -25200),
            "31-Dec-1969 17:00:00".to_string()
        ); // -7
        assert_eq!(
            super::get_time_string_impl(0, 32400),
            "01-Jan-1970 09:00:00".to_string()
        ); // +9
    }

    #[test]
    fn test_get_check_mode_string() {
        let l1 = hammer2fs::enc_level(1);
        let l0 = hammer2fs::enc_level(0);
        let def_algo = hammer2fs::enc_algo(hammer2fs::HAMMER2_CHECK_DEFAULT);
        assert_eq!(super::get_check_mode_string(l1 | def_algo), "xxhash64:1");
        assert_eq!(super::get_check_mode_string(l1 | 0xf), "unknown(15):1");
        assert_eq!(super::get_check_mode_string(l0 | def_algo), "xxhash64");
        assert_eq!(super::get_check_mode_string(l0 | 0xf), "unknown(15)");
    }

    #[test]
    fn test_get_comp_mode_string() {
        let l1 = hammer2fs::enc_level(1);
        let l0 = hammer2fs::enc_level(0);
        let def_algo = hammer2fs::enc_algo(hammer2fs::HAMMER2_COMP_DEFAULT);
        assert_eq!(super::get_comp_mode_string(l1 | def_algo), "lz4:1");
        assert_eq!(super::get_comp_mode_string(l1 | 0xf), "unknown(15):1");
        assert_eq!(super::get_comp_mode_string(l0 | def_algo), "lz4:default");
        assert_eq!(super::get_comp_mode_string(l0 | 0xf), "unknown(15):default");
    }

    #[test]
    fn test_get_size_string() {
        let l = [
            (0, "  0.00B"),
            (1, "  1.00B"),
            (512, "  0.50KB"),
            (1024, "  1.00KB"),
            (524_288, "  0.50MB"),
            (1_048_576, "  1.00MB"),
        ];
        for t in &l {
            assert_eq!(super::get_size_string(t.0), t.1, "{}", t.0);
        }
    }

    #[test]
    fn test_dirhash() {
        let l = [
            ("", 0x8000_0000_0000_8000),
            (".", 0x8000_0000_bc10_8000),
            ("-", 0x8000_0000_5cb4_8000),
            ("_", 0x8000_0000_e8ed_8000),
            ("~", 0x8000_0000_37e6_8000),
            ("A", 0xe16d_cdee_2c83_8000),
            ("hammer2", 0x9f2f_13b5_8c9a_8000),
            (
                "This code is derived from software contributed to The DragonFly Project",
                0xf8df_95ed_6d32_8000,
            ),
        ];
        for t in &l {
            assert_eq!(super::dirhash(t.0.as_bytes()), t.1, "{}", t.0);
        }
    }

    #[test]
    fn test_uuid() {
        let u = match uuid::Uuid::parse_str(hammer2fs::HAMMER2_UUID_STRING) {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(u.to_string(), hammer2fs::HAMMER2_UUID_STRING);
    }

    #[test]
    fn test_uuid_wrapper() {
        let u = super::get_uuid_from_str(hammer2fs::HAMMER2_UUID_STRING);
        assert_eq!(super::get_uuid_string(&u), hammer2fs::HAMMER2_UUID_STRING);
        assert_eq!(
            super::get_uuid_string_from_bytes(util::any_as_u8_slice(&u)),
            hammer2fs::HAMMER2_UUID_STRING
        );
    }
}
