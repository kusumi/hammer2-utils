pub(crate) fn run(args: &[&str]) {
    for s in args {
        let mut buf = vec![0; 1024]; // 1K extended directory record
        let b = s.as_bytes();
        buf[..b.len()].copy_from_slice(b);
        println!("{:016x} {s}", libhammer2::xxhash::xxh64(&buf));
    }
}
