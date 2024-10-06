use sha2::Digest;

#[must_use]
pub fn sha256(buf: &[u8]) -> Vec<u8> {
    let mut h = sha2::Sha256::new();
    h.update(buf);
    h.finalize()[..].to_vec()
}

#[cfg(test)]
mod tests {
    const SHA256_LIST: [(&str, &str); 8] = [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            ".",
            "cdb4ee2aea69cc6a83331bbe96dc2caa9a299d21329efb0336fc02a82e1839a8",
        ),
        (
            "-",
            "3973e022e93220f9212c18d0d0c543ae7c309e46640da93a4a0314de999f5112",
        ),
        (
            "_",
            "d2e2adf7177b7a8afddbc12d1634cf23ea1a71020f6a1308070a16400fb68fde",
        ),
        (
            "~",
            "7ace431cb61584cb9b8dc7ec08cf38ac0a2d649660be86d349fb43108b542fa4",
        ),
        (
            "A",
            "559aead08264d5795d3909718cdd05abd49572e84fe55590eef31a88a08fdffd",
        ),
        (
            "hammer2",
            "8a6606c9532c846c617512bce93e4c5496dde608b985d1aefcc9156314928c6f",
        ),
        (
            "This code is derived from software contributed to The DragonFly Project",
            "51fef6ec893b56802dcdee6e59aa9d41fc3df6e353f99d8f5a2098dfa2ec19bd",
        ),
    ];

    #[test]
    fn test_sha256() {
        for t in &SHA256_LIST {
            let v = super::sha256(t.0.as_bytes());
            assert_eq!(v.len(), 256 / 8);
            assert_eq!(hex::encode(v), t.1, "{}", t.0);
        }
    }
}
