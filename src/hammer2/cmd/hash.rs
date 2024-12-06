pub(crate) fn run(args: &[&str]) {
    for s in args {
        println!("{:016x} {s}", libhammer2::subs::dirhash(s.as_bytes()));
    }
}
