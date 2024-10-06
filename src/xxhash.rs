const XXH_HAMMER2_SEED: u64 = 0x4d61_7474_446c_6c6e;

#[must_use]
pub fn xxh64(buf: &[u8]) -> u64 {
    xxhash_rust::xxh64::xxh64(buf, XXH_HAMMER2_SEED)
}

#[cfg(test)]
mod tests {
    const XXH64_LIST: [(&str, u64); 8] = [
        ("", 0x8456_6ac0_f5a0_cb84),
        (".", 0x6bc3_43ee_b2a8_4f95),
        ("-", 0x47ba_29e5_be35_936a),
        ("_", 0xaca9_e940_ff02_a58f),
        ("~", 0xd46d_c5a2_172e_3e9b),
        ("A", 0xe1ce_5343_4b3a_ad24),
        ("hammer2", 0xa041_be26_8b56_74e5),
        (
            "This code is derived from software contributed to The DragonFly Project",
            0x103c_241e_eddc_8d32,
        ),
    ];

    #[test]
    fn test_xxh64() {
        for t in &XXH64_LIST {
            assert_eq!(super::xxh64(t.0.as_bytes()), t.1, "{}", t.0);
        }
    }

    #[test]
    fn test_struct_xxh64() {
        let mut h = xxhash_rust::xxh64::Xxh64::new(super::XXH_HAMMER2_SEED);
        for t in &XXH64_LIST {
            h.update(t.0.as_bytes());
            assert_eq!(h.digest(), t.1, "{}", t.0);
            h.reset(super::XXH_HAMMER2_SEED);
        }
    }
}
