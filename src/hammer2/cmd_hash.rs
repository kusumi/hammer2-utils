use hammer2_utils::subs;

pub(crate) fn run(args: &[String]) {
    for s in args {
        println!("{:016x} {s}", subs::dirhash(s.as_bytes()));
    }
}
