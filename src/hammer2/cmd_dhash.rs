use hammer2_utils::xxhash;

pub(crate) fn run(args: &[String]) {
    for s in args {
        let mut buf = vec![0; 1024]; // 1K extended directory record
        let b = s.as_bytes();
        buf[..b.len()].copy_from_slice(b);
        println!("{:016x} {s}", xxhash::xxh64(&buf));
    }
}
