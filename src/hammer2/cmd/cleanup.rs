use crate::cmd;
use crate::Hammer2Options;

fn docleanup(f: &str, opt: &Hammer2Options) -> Result<(), Box<dyn std::error::Error>> {
    println!("hammer2 cleanup \"{f}\"");
    cmd::bulkfree::run(f, opt)
}

fn sameh2prefix(f: &str, h2prefixes: &[String]) -> bool {
    let f = geth2prefix(f);
    for x in h2prefixes {
        if *x == f {
            return true;
        }
    }
    false
}

fn saveh2prefix(f: &str, h2prefixes: &mut Vec<String>) {
    h2prefixes.push(geth2prefix(f).to_string());
}

fn geth2prefix(f: &str) -> &str {
    if let Some(i) = f.find('@') {
        &f[..i]
    } else {
        f
    }
}

pub(crate) fn run(f: Option<&str>, opt: &Hammer2Options) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(f) = f {
        return docleanup(f, opt);
    }
    let m = libhammer2::os::get_mnt_info()?;
    if m.is_empty() {
        println!("hammer2 cleanup: no HAMMER2 mounts");
        return Ok(());
    }
    let mut h2prefixes = vec![];
    for t in &m {
        let (fstype, path, from) = t;
        if fstype != "hammer2" {
            continue;
        }
        if sameh2prefix(from, &h2prefixes) {
            println!("hammer2 cleanup \"{path}\" (same partition)");
        } else {
            docleanup(path, opt)?;
            saveh2prefix(from, &mut h2prefixes);
        }
    }
    Ok(())
}
