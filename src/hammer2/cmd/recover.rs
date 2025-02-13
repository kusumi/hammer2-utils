use libhammer2::os::StatExt;
use std::io::Write;

// hammer2 recover <devpath> <path> <destdir>
//
// Recover files from corrupted media, recover deleted files from good
// media, generally ignoring the data structure topology outside of the
// structures that hang off of inodes and directory entries.  Files are
// validated during recovery and renamed to .corrupted when a version of
// a file cannot be completely recovered.
//
// This is a "try to get everything we can out of the filesystem"
// directive when you've done something terrible to the filesystem.
// The resulting <destdir> tree cannot directly replace the lost topology
// as many versions of the same file will be present, so filenames are
// suffixed.
//
// <path> may be a relative file, directory, directory path, or absolute
// (absolute is relative to the mount) file or directory path.
//
// For example, "hammer2 recover /dev/da0s1d /home/charlie /tmp/" will
// recover all possible versions of the /home/charlie sub-tree.  If
// you said "home/charlie" instead, then it would recover the same but
// also include any sub-directory paths that match home/charlie, not
// just root paths.  If you want a specific file, then e.g. ".cshrc"
// would recover every single .cshrc that can be found on the media.
//
// The command checks ALL PFSs and snapshots.  Redundant files with the same
// path are ignored.  You basically get everything that can possibly be
// recovered from the media.

const HTABLE_SIZE: u64 = 4 * 1024 * 1024;
const HTABLE_MASK: u64 = HTABLE_SIZE - 1;

const MAX_RADIX_MASK: u64 = 0x1F; // not HAMMER2_OFF_MASK_RADIX (0x3F)

// Rust: unique id's to substitute pointer comparison used in C
static mut INODE_ENTRY_ID_NEXT: u64 = 0;
static mut TOPOLOGY_ENTRY_ID_NEXT: u64 = 0;

#[derive(Clone, Debug, Default)]
struct InodeEntry {
    id: u64,   // Rust
    typ: u8,   // from inode meta
    inum: u64, // from bref or inode meta
    data_off: u64,
    crc: u32,
    encountered: bool, // copies limit w/REPINODEDEPTH
    loopcheck: bool,   // recursion loop check
    link_file_path: String,
}

impl InodeEntry {
    fn init() {
        unsafe {
            INODE_ENTRY_ID_NEXT = 0;
        }
    }

    fn new(typ: u8, inum: u64, data_off: u64, crc: u32) -> Self {
        let id;
        unsafe {
            id = INODE_ENTRY_ID_NEXT;
            INODE_ENTRY_ID_NEXT += 1;
        }
        Self {
            id,
            typ,
            inum,
            data_off,
            crc,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
struct TopologyEntry {
    id: u64, // Rust
    path: String,
    iterator: usize,
}

impl TopologyEntry {
    fn init() {
        unsafe {
            TOPOLOGY_ENTRY_ID_NEXT = 0;
        }
    }

    fn new(path: &str) -> Self {
        let id;
        unsafe {
            id = TOPOLOGY_ENTRY_ID_NEXT;
            TOPOLOGY_ENTRY_ID_NEXT += 1;
        }
        Self {
            id,
            path: path.to_string(),
            iterator: 1,
        }
    }
}

#[derive(Debug, Default)]
struct TopologyInodeEntry {
    topo_id: u64,  // Rust
    iscan_id: u64, // Rust
}

impl TopologyInodeEntry {
    fn new(topo_id: u64, iscan_id: u64) -> Self {
        Self { topo_id, iscan_id }
    }
}

#[derive(Debug, Default)]
struct TopologyBlockrefEntry {
    topo_id: u64, // Rust
    data_off: u64,
}

impl TopologyBlockrefEntry {
    fn new(topo_id: u64, data_off: u64) -> Self {
        Self { topo_id, data_off }
    }
}

#[derive(Debug, Default)]
struct NegativeEntry {
    bref: libhammer2::fs::Hammer2Blockref,
}

impl NegativeEntry {
    fn new(bref: &libhammer2::fs::Hammer2Blockref) -> Self {
        Self { bref: *bref }
    }
}

#[derive(Debug, Default)]
struct Stats {
    inode: usize,
    topology_inode: usize,
    topology_inode_dup: usize,
    topology_blockref: usize,
    topology_blockref_dup: usize,
    negative: usize,
    negative_hits: usize,
}

impl Stats {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

// DragonFly uses uint32_t for hash values
type InodeEntryHash = std::collections::HashMap<u64, Vec<InodeEntry>>;
type TopologyEntryHash = std::collections::HashMap<u64, Vec<TopologyEntry>>;
type TopologyInodeEntryHash = std::collections::HashMap<u64, Vec<TopologyInodeEntry>>;
type TopologyBlockrefEntryHash = std::collections::HashMap<u64, Vec<TopologyBlockrefEntry>>;
type NegativeEntryHash = std::collections::HashMap<u64, Vec<NegativeEntry>>;

type InodeEntryHashId = (u64, usize);

macro_rules! get_entry {
    ($h:expr, $hid:expr) => {
        &$h.get(&$hid.0).unwrap()[$hid.1]
    };
}

macro_rules! get_entry_mut {
    ($h:expr, $hid:expr) => {
        &mut $h.get_mut(&$hid.0).unwrap()[$hid.1]
    };
}

// Check for a matching filename, Directory entries can directly-embed
// filenames <= 64 bytes.  Otherwise the directory entry has a data
// reference to the location of the filename.
//
// If filename is NULL, check for a valid filename, and copy it into buf.
#[allow(dead_code)]
fn check_filename(
    fso: &mut libhammer2::ondisk::Ondisk,
    bref: &libhammer2::fs::Hammer2Blockref,
    filename: Option<&str>,
    flen: usize,
    strict: bool,
) -> hammer2_utils::Result<Option<String>> {
    // filename too long
    if flen > 1024 {
        return Ok(None);
    }
    if flen <= 64 {
        // Filename is embedded in bref.
        let buf = libhammer2::util::bin_to_string(&bref.check[..flen])?;
        if let Some(filename) = filename {
            if filename[..flen] == buf {
                Ok(Some(buf))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(buf))
        }
    } else {
        // Filename requires media access.
        // bref must represent a data reference to a 1KB block or smaller.
        if (bref.data_off & MAX_RADIX_MASK) == 0 || (bref.data_off & MAX_RADIX_MASK) > 10 {
            return Ok(None);
        }
        // Indirect block containing filename must be large enough
        // to contain the filename.
        let psize = 1 << (bref.data_off & MAX_RADIX_MASK);
        if flen > psize {
            return Ok(None);
        }
        // In strict mode we disallow bref's set to HAMMER2_CHECK_NON
        // or HAMMER2_CHECK_DISABLED.  Do this check before burning
        // time on an I/O.
        if strict {
            let check_algo = libhammer2::fs::dec_check(bref.methods);
            if check_algo == libhammer2::fs::HAMMER2_CHECK_NONE
                || check_algo == libhammer2::fs::HAMMER2_CHECK_DISABLED
            {
                return Ok(None);
            }
        }
        // Read the data, check CRC and such.
        let Some(vol) = fso.get_volume_mut(bref.data_off) else {
            return Ok(None);
        };
        let poff = (bref.data_off - vol.get_offset()) & !MAX_RADIX_MASK;
        let Ok(data) = vol.preadx(psize.try_into()?, poff) else {
            return Ok(None);
        };
        if !validate_crc(bref, &data, strict) {
            return Ok(None);
        }
        let buf = libhammer2::util::bin_to_string(&data[..flen])?;
        if let Some(filename) = filename {
            if filename[..flen] == buf {
                Ok(Some(buf))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(buf))
        }
    }
}

// Topology duplicate scan avoidance helpers.  We associate inodes and
// indirect block data offsets, allowing us to avoid re-scanning any
// duplicates that we see.  And there will be many due to how the COW
// process occurs.
//
// For example, when a large directory is modified the content update to
// the directory entries will cause the directory inode to be COWd, along
// with whatever is holding the bref(s) blocks that have undergone
// adjustment.  More likely than not, there will be MANY shared indirect
// blocks.
fn enter_topology(thash: &mut TopologyEntryHash, path: &str) -> (InodeEntryHashId, u64) {
    let mut hv = 0;
    for x in path.bytes() {
        hv = (hv << 5) ^ u64::from(x) ^ (hv >> 24);
    }
    hv = (hv ^ (hv >> 16)) & HTABLE_MASK;
    if let Some(v) = thash.get(&hv) {
        for (i, topo) in v.iter().enumerate() {
            if path == topo.path.as_str() {
                return ((hv, i), topo.id);
            }
        }
    }
    let topo = TopologyEntry::new(path);
    let topo_id = topo.id;
    let i = if let Some(v) = thash.get_mut(&hv) {
        v.push(topo);
        v.len() - 1
    } else {
        thash.insert(hv, vec![topo]);
        0
    };
    ((hv, i), topo_id)
}

// Determine if an inode at the current topology location is one that we
// have already dealt with.
fn topology_check_duplicate_inode(
    tihash: &mut TopologyInodeEntryHash,
    stats: &mut Stats,
    topo_id: u64,
    iscan: &InodeEntry,
) -> bool {
    let hv = ((topo_id ^ iscan.id) >> 6) & HTABLE_MASK;
    if let Some(v) = tihash.get(&hv) {
        for scan in v {
            if scan.topo_id == topo_id && scan.iscan_id == iscan.id {
                stats.topology_inode_dup += 1;
                return true;
            }
        }
    }
    let scan = TopologyInodeEntry::new(topo_id, iscan.id);
    if let Some(v) = tihash.get_mut(&hv) {
        v.push(scan);
    } else {
        tihash.insert(hv, vec![scan]);
    }
    stats.topology_inode += 1;
    false
}

// Determine if an indirect block (represented by the bref) at the current
// topology level is one that we have already dealt with.
fn topology_check_duplicate_indirect(
    tbhash: &mut TopologyBlockrefEntryHash,
    stats: &mut Stats,
    topo_id: u64,
    bref: &libhammer2::fs::Hammer2Blockref,
) -> bool {
    let hv = (topo_id ^ (bref.data_off >> 8)) & HTABLE_MASK;
    if let Some(v) = tbhash.get(&hv) {
        for scan in v {
            if scan.topo_id == topo_id && scan.data_off == bref.data_off {
                stats.topology_blockref_dup += 1;
                return true;
            }
        }
    }
    let scan = TopologyBlockrefEntry::new(topo_id, bref.data_off);
    if let Some(v) = tbhash.get_mut(&hv) {
        v.push(scan);
    } else {
        tbhash.insert(hv, vec![scan]);
    }
    stats.topology_blockref += 1;
    false
}

// Valid and record an inode found on media.  There can be many versions
// of the same inode number present on the media.
// Note: Modified DragonFly's inefficient hv2.
#[allow(clippy::too_many_arguments)]
fn enter_inode(
    fso: &mut libhammer2::ondisk::Ondisk,
    ihash1: &mut InodeEntryHash,
    ihash2: &mut InodeEntryHash,
    nhash: &mut NegativeEntryHash,
    stats: &mut Stats,
    sdc: &mut SdcCache,
    bref: &libhammer2::fs::Hammer2Blockref,
    strict: bool,
) -> hammer2_utils::Result<()> {
    let hv1 = (bref.key ^ (bref.key >> 16)) & HTABLE_MASK;
    let hv2 = ((bref.key ^ (bref.key >> 16)) | (bref.data_off >> 10)) & HTABLE_MASK;
    // Ignore duplicate inodes, use the secondary inode hash table's
    // better spread to reduce cpu consumption (there can be many
    // copies of the same inode so the primary hash table can have
    // very long chains in it).
    if let Some(v) = ihash2.get(&hv2) {
        for scan in v {
            if bref.key == scan.inum && bref.data_off == scan.data_off {
                return Ok(());
            }
        }
    }
    // Ignore brefs which we have already determined to be bad.
    if find_negative(nhash, stats, bref) {
        return Ok(());
    }
    // Validate the potential blockref.  Note that this might not be a
    // real blockref.  Don't trust anything, really.
    //
    // - Must be sized for an inode block
    // - Must be properly aligned for an inode block
    // - Keyspace is 1 (keybits == 0), i.e. a single inode number
    if (1 << (bref.data_off & MAX_RADIX_MASK)) != libhammer2::fs::HAMMER2_INODE_BYTES {
        return Ok(());
    }
    if ((bref.data_off & !MAX_RADIX_MASK) & (libhammer2::fs::HAMMER2_INODE_BYTES - 1)) != 0 {
        return Ok(());
    }
    if bref.keybits != 0 {
        return Ok(());
    }
    if bref.key == 0 {
        return Ok(());
    }
    let (data, psize) = sdc.cache_read(fso, bref.data_off)?;
    // Failure prior to I/O being performed.
    if psize == 0 {
        return Ok(());
    }
    // DragonFly missing this
    if data.is_empty() || psize != libhammer2::fs::HAMMER2_INODE_BYTES {
        return Ok(());
    }
    // Any failures which occur after the I/O has been performed
    // should enter the bref in the negative cache to avoid unnecessary
    // guaranteed-to-fil reissuances of the same (bref, data_off) combo.
    if data.is_empty() {
        enter_negative(nhash, stats, bref);
        return Ok(());
    }
    // The blockref looks ok but the real test is whether the
    // inode data it references passes the CRC check.  If it
    // does, it is highly likely that we have a valid inode.
    if !validate_crc(bref, &data[..psize.try_into()?], strict) {
        enter_negative(nhash, stats, bref);
        return Ok(());
    }
    let inode = libhammer2::extra::media_as::<libhammer2::fs::Hammer2InodeData>(data)[0];
    if inode.meta.inum != bref.key {
        enter_negative(nhash, stats, bref);
        return Ok(());
    }
    // Record the inode.  For now we do not record the actual content
    // of the inode because if there are more than few million of them
    // the memory consumption can get into the dozens of gigabytes.
    //
    // Instead, the inode will be re-read from media in the recovery
    // pass.
    let scan1 = InodeEntry::new(
        inode.meta.typ,
        bref.key,
        bref.data_off,
        icrc32::iscsi_crc32(&data[..psize.try_into()?]),
    );
    let scan2 = scan1.clone();

    if let Some(v) = ihash1.get_mut(&hv1) {
        v.push(scan1);
    } else {
        ihash1.insert(hv1, vec![scan1]);
    }
    if let Some(v) = ihash2.get_mut(&hv2) {
        v.push(scan2);
    } else {
        ihash2.insert(hv2, vec![scan2]);
    }
    stats.inode += 1;
    Ok(())
}

// This is used to enter possible root inodes.  Root inodes typically hang
// off the volume root and thus there might not be a bref reference to the
// many old copies of root inodes sitting around on the media.  Without a
// bref we can't really validate that the content is ok.  But we need
// these inodes as part of our path searches.
// Note: Modified DragonFly's inefficient hv2.
fn enter_inode_untested(
    ihash1: &mut InodeEntryHash,
    ihash2: &mut InodeEntryHash,
    stats: &mut Stats,
    inode: &libhammer2::fs::Hammer2InodeData,
    loff: u64,
) {
    let hv1 = (inode.meta.inum ^ (inode.meta.inum >> 16)) & HTABLE_MASK;
    let hv2 = ((inode.meta.inum ^ (inode.meta.inum >> 16)) | (loff >> 10)) & HTABLE_MASK;
    if let Some(v) = ihash2.get(&hv2) {
        for scan in v {
            if inode.meta.inum == scan.inum && loff == scan.data_off {
                return;
            }
        }
    }
    // Record the inode.  For now we do not record the actual content
    // of the inode because if there are more than few million of them
    // the memory consumption can get into the dozens of gigabytes.
    //
    // Instead, the inode will be re-read from media in the recovery
    // pass.
    let scan1 = InodeEntry::new(
        inode.meta.typ,
        inode.meta.inum,
        loff,
        icrc32::iscsi_crc32(libhammer2::util::any_as_u8_slice(inode)),
    );
    let scan2 = scan1.clone();

    if let Some(v) = ihash1.get_mut(&hv1) {
        v.push(scan1);
    } else {
        ihash1.insert(hv1, vec![scan1]);
    }
    if let Some(v) = ihash2.get_mut(&hv2) {
        v.push(scan2);
    } else {
        ihash2.insert(hv2, vec![scan2]);
    }
    stats.inode += 1;
}

fn find_first_inode(ihash1: &InodeEntryHash, inum: u64) -> Option<(InodeEntryHashId, usize)> {
    let hv1 = (inum ^ (inum >> 16)) & HTABLE_MASK;
    if let Some(v) = ihash1.get(&hv1) {
        for (i, entry) in v.iter().enumerate() {
            if entry.inum == inum {
                return Some(((hv1, i), v.len()));
            }
        }
    }
    None
}

// Negative bref cache.  A cache of brefs that we have determined
// to be invalid.  Used to reduce unnecessary disk I/O.
//
// Note: Checks must be reasonable and at least encompass checks
//   done in enter_inode() after it has decided to read the
//   block at data_off.
//
//   Adding a few more match fields in addition won't hurt either.
fn find_negative(
    nhash: &NegativeEntryHash,
    stats: &mut Stats,
    bref: &libhammer2::fs::Hammer2Blockref,
) -> bool {
    let hv = (bref.data_off >> 10) & HTABLE_MASK;
    if let Some(v) = nhash.get(&hv) {
        for neg in v {
            if bref.data_off == neg.bref.data_off
                && bref.typ == neg.bref.typ
                && bref.methods == neg.bref.methods
                && bref.key == neg.bref.key
                && bref.check == neg.bref.check
            {
                stats.negative_hits += 1;
                return true;
            }
        }
    }
    false
}

fn enter_negative(
    nhash: &mut NegativeEntryHash,
    stats: &mut Stats,
    bref: &libhammer2::fs::Hammer2Blockref,
) {
    let hv = (bref.data_off >> 10) & HTABLE_MASK;
    let neg = NegativeEntry::new(bref);
    if let Some(v) = nhash.get_mut(&hv) {
        v.push(neg);
    } else {
        nhash.insert(hv, vec![neg]);
    }
    stats.negative += 1;
}

// Dump the specified inode (file or directory)
//
// This function recurses via dump_dir_data().
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn dump_tree(
    fso: &mut libhammer2::ondisk::Ondisk,
    ihash1: &mut InodeEntryHash,
    thash: &mut TopologyEntryHash,
    tihash: &mut TopologyInodeEntryHash,
    tbhash: &mut TopologyBlockrefEntryHash,
    stats: &mut Stats,
    sdc: &mut SdcCache,
    hid: InodeEntryHashId,
    dest: &str,
    remain: &str,
    depth: usize,
    path_depth: usize,
    isafile: bool,
    strict: bool,
) -> hammer2_utils::Result<()> {
    const REPINODEDEPTH: usize = 256;
    let iscan = get_entry!(ihash1, hid);
    // Re-read the already-validated inode instead of saving it in
    // memory from the media pass.  Even though we already validated
    // it, the content may have changed if scanning live media, so
    // check against a simple crc we recorded earlier.
    let (data, psize) = sdc.cache_read(fso, iscan.data_off)?;
    if psize == 0 {
        return Ok(());
    }
    if data.is_empty() || psize != libhammer2::fs::HAMMER2_INODE_BYTES {
        return Ok(());
    }
    if iscan.crc != icrc32::iscsi_crc32(&data[..psize.try_into()?]) {
        return Ok(());
    }
    let inode = libhammer2::extra::media_as::<libhammer2::fs::Hammer2InodeData>(data)[0];
    // Try to limit potential infinite loops.
    if depth > REPINODEDEPTH && iscan.encountered {
        return Ok(());
    }
    // Get rid of any dividing slashes.
    let remain = remain.trim_start_matches('/');
    // Create/lookup destination path (without the iterator), to acquire
    // an iterator for different versions of the same file.
    //
    // Due to the COW mechanism, a lot of repeated snapshot-like
    // directories may be encountered so we use the topology tree
    // to weed out duplicates that attempt to use the same pathname.
    //
    // We also don't iterate copies of directories since this would
    // create a major mess due to the many versions that might be
    // laying around.  Directories use unextended names.
    let (topo_hid, topo_id) = enter_topology(thash, dest);
    if topology_check_duplicate_inode(tihash, stats, topo_id, iscan) {
        return Ok(());
    }
    match iscan.typ {
        libhammer2::fs::HAMMER2_OBJTYPE_DIRECTORY => {
            // If we have exhausted the path and isafile is TRUE, stop.
            if isafile && remain.is_empty() {
                return Ok(());
            }
            // Create / make usable the target directory.  Note that
            // it might already exist.
            //
            // Do not do this for the destination base directory
            // (depth 1).
            if depth != 1 {
                if std::fs::metadata(dest).is_err() {
                    std::fs::create_dir(dest)?;
                    get_entry_mut!(ihash1, hid).encountered = true;
                }
                let mut st = libhammer2::os::new_stat();
                unsafe {
                    let dest = libhammer2::util::new_cstring!(dest)?;
                    let pdest = dest.as_ptr();
                    if libhammer2::os::stat(pdest, &mut st) == 0 {
                        if st.get_flags() != 0 {
                            let _ = libhammer2::os::chflags(pdest, 0);
                        }
                        if (st.st_mode & 0o700) != 0o700 {
                            let _ = libhammer2::os::chmod(pdest, 0o755);
                        }
                    }
                }
            }
            // Dump directory contents (scan the directory).
            let inode = *inode; // dump_dir_data need to mut borrow sdc
            dump_dir_data(
                fso,
                ihash1,
                thash,
                tihash,
                tbhash,
                stats,
                sdc,
                dest,
                remain,
                inode
                    .u_as::<libhammer2::fs::Hammer2Blockset>()
                    .as_blockref()
                    .as_slice(),
                depth,
                path_depth + 1,
                isafile,
                strict,
            )?;
            // Final adjustment to directory inode.
            if depth != 1 {
                unsafe {
                    let dest = libhammer2::util::new_cstring!(dest)?;
                    let pdest = dest.as_ptr();
                    let tvs = inode.meta.get_utimes_timeval();
                    let error = libhammer2::os::lutimes(pdest, tvs.as_ptr());
                    if error != 0 {
                        log::error!("lutimes {dest:?} {tvs:?} {error}");
                    }
                    let _ = libhammer2::os::lchown(
                        pdest,
                        hammer2_to_unix_xid(&inode.meta.uid),
                        hammer2_to_unix_xid(&inode.meta.gid),
                    );
                    let _ = libhammer2::os::chmod(pdest, inode.meta.mode); // XXX lchmod
                    let _ = libhammer2::os::lchflags(pdest, inode.meta.uflags.into());
                }
            }
        }
        libhammer2::fs::HAMMER2_OBJTYPE_REGFILE => {
            // If no more path to match, dump the file contents.
            if remain.is_empty() {
                let topo = get_entry_mut!(thash, topo_hid);
                let path = format!("{}.{:05}", dest, topo.iterator);
                topo.iterator += 1;
                let mut st = libhammer2::os::new_stat();
                unsafe {
                    let path = libhammer2::util::new_cstring!(&*path)?;
                    let ppath = path.as_ptr();
                    if libhammer2::os::stat(ppath, &mut st) == 0 {
                        if st.get_flags() != 0 {
                            let _ = libhammer2::os::chflags(ppath, 0);
                        }
                        if (st.st_mode & 0o600) != 0o600 {
                            let _ = libhammer2::os::chmod(ppath, 0o644);
                        }
                    }
                }
                get_entry_mut!(ihash1, hid).encountered = true;
                dump_inum_file(fso, ihash1, hid, inode, &path, strict)?;
            }
        }
        libhammer2::fs::HAMMER2_OBJTYPE_SOFTLINK => {
            // If no more path to match, dump the file contents.
            if remain.is_empty() {
                let topo = get_entry_mut!(thash, topo_hid);
                let path = format!("{}.{:05}", dest, topo.iterator);
                topo.iterator += 1;
                let mut st = libhammer2::os::new_stat();
                unsafe {
                    let path = libhammer2::util::new_cstring!(&*path)?;
                    let ppath = path.as_ptr();
                    if libhammer2::os::stat(ppath, &mut st) == 0 {
                        if st.get_flags() != 0 {
                            let _ = libhammer2::os::chflags(ppath, 0);
                        }
                        if (st.st_mode & 0o600) != 0o600 {
                            let _ = libhammer2::os::chmod(ppath, 0o644);
                        }
                    }
                }
                // dump_inum_softlink (Not implemented)
            }
        }
        _ => (),
    }
    Ok(())
}

// Scan the directory for a match against the next component in
// the (remain) path.
//
// This function is part of the dump_tree() recursion mechanism.
#[allow(clippy::too_many_arguments)]
fn dump_dir_data(
    fso: &mut libhammer2::ondisk::Ondisk,
    ihash1: &mut InodeEntryHash,
    thash: &mut TopologyEntryHash,
    tihash: &mut TopologyInodeEntryHash,
    tbhash: &mut TopologyBlockrefEntryHash,
    stats: &mut Stats,
    sdc: &mut SdcCache,
    dest: &str,
    remain: &str,
    base: &[&libhammer2::fs::Hammer2Blockref],
    depth: usize,
    path_depth: usize,
    isafile: bool,
    strict: bool,
) -> hammer2_utils::Result<()> {
    // Scan the brefs associated with the directory.
    for bref in base {
        if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY {
            continue;
        }
        match bref.typ {
            libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT => 'indirect: {
                if (bref.data_off & MAX_RADIX_MASK) == 0 {
                    break 'indirect; // DragonFly missing this
                }
                let Some(vol) = fso.get_volume_mut(bref.data_off) else {
                    break 'indirect;
                };
                let poff = (bref.data_off - vol.get_offset()) & !MAX_RADIX_MASK;
                let psize = 1 << (bref.data_off & MAX_RADIX_MASK);
                if psize > libhammer2::fs::HAMMER2_PBUFSIZE {
                    break 'indirect;
                }
                if psize == 0 {
                    break 'indirect;
                }
                // Due to COW operations, even if the inode is
                // replicated, some of the indirect brefs might
                // still be shared and allow us to reject duplicate
                // scans.
                let (_, topo_id) = enter_topology(thash, dest);
                if topology_check_duplicate_indirect(tbhash, stats, topo_id, bref) {
                    // DragonFly sets res to 1, but res unused by top level caller
                    break 'indirect;
                }
                let Ok(data) = vol.preadx(psize, poff) else {
                    break 'indirect;
                };
                if !validate_crc(bref, &data, strict) {
                    break 'indirect;
                }
                dump_dir_data(
                    fso,
                    ihash1,
                    thash,
                    tihash,
                    tbhash,
                    stats,
                    sdc,
                    dest,
                    remain,
                    &libhammer2::extra::media_as(&data),
                    depth,
                    path_depth,
                    isafile,
                    strict,
                )?;
            }
            libhammer2::fs::HAMMER2_BREF_TYPE_DIRENT => {
                // Calculate length of next path component to match against.
                // A length of zero matches against the entire remaining
                // directory sub-tree.
                let flen = match remain.find('/') {
                    Some(v) => v,
                    None => remain.len(),
                };
                // Match the path element or match all directory
                // entries if we have exhausted (remain).
                //
                // Locate all matching inodes and continue the
                // traversal.
                //
                // Avoid traversal loops by recording path_depth
                // on the way down and clearing it on the way back up.
                let dirent = bref.embed_as::<libhammer2::fs::Hammer2DirentHead>();
                let filename_buf = if flen == 0 {
                    check_filename(fso, bref, None, dirent.namlen.into(), strict)?
                } else if flen == dirent.namlen.into() {
                    check_filename(fso, bref, Some(remain), flen, strict)?
                } else {
                    None
                };
                if let Some(filename_buf) = filename_buf {
                    let inum = dirent.inum;
                    let path = format!("{dest}/{filename_buf}");
                    if let Some(((hv, start), end)) = find_first_inode(ihash1, inum) {
                        for i in start..end {
                            let hid = (hv, i);
                            let iscan = get_entry!(ihash1, hid);
                            if iscan.inum == inum && !iscan.loopcheck && iscan.typ == dirent.typ {
                                get_entry_mut!(ihash1, hid).loopcheck = true;
                                dump_tree(
                                    fso,
                                    ihash1,
                                    thash,
                                    tihash,
                                    tbhash,
                                    stats,
                                    sdc,
                                    hid,
                                    &path,
                                    &remain[flen..],
                                    depth + 1,
                                    path_depth,
                                    isafile,
                                    strict,
                                )?;
                                // Clear loop check.
                                get_entry_mut!(ihash1, hid).loopcheck = false;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

// [re]create a regular file and attempt to restore the originanl perms,
// modes, flags, times, uid, and gid if successful.
//
// If the data block recursion fails the file will be renamed .corrupted.
fn dump_inum_file(
    fso: &mut libhammer2::ondisk::Ondisk,
    ihash1: &mut InodeEntryHash,
    hid: InodeEntryHashId,
    inode: &libhammer2::fs::Hammer2InodeData,
    path1: &str,
    strict: bool,
) -> hammer2_utils::Result<bool> {
    let iscan = get_entry!(ihash1, hid);
    // If this specific inode has already been generated, try to
    // hardlink it instead of regenerating the same file again.
    if !iscan.link_file_path.is_empty() {
        if std::fs::hard_link(&iscan.link_file_path, path1).is_ok() {
            return Ok(true);
        }
        let link_file_path = libhammer2::util::new_cstring!(&*iscan.link_file_path)?;
        let plink_file_path = link_file_path.as_ptr();
        unsafe {
            let _ = libhammer2::os::chflags(plink_file_path, 0);
            let _ = libhammer2::os::chmod(plink_file_path, 0o600);
        }
        if std::fs::hard_link(&iscan.link_file_path, path1).is_ok() {
            unsafe {
                let _ = libhammer2::os::chmod(plink_file_path, inode.meta.mode);
                let _ = libhammer2::os::chflags(plink_file_path, inode.meta.uflags.into());
            }
            return Ok(true);
        }
    }
    // Cleanup potential flags and modes to allow us to write out a
    // new file.
    unsafe {
        let path1 = libhammer2::util::new_cstring!(path1)?;
        let ppath1 = path1.as_ptr();
        let _ = libhammer2::os::chflags(ppath1, 0);
        let _ = libhammer2::os::chmod(ppath1, 0o600);
    }
    let mut fp = std::fs::File::create(path1)?;
    let res = if (inode.meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_DIRECTDATA) != 0 {
        // direct data case
        if inode.meta.size > 0 && inode.meta.size <= libhammer2::fs::HAMMER2_EMBEDDED_BYTES {
            fp.write_all(&inode.u).is_ok()
        } else {
            true
        }
    } else {
        // file content, indirect blockrefs
        dump_file_data(
            fso,
            &mut fp,
            inode.meta.size,
            &inode
                .u_as::<libhammer2::fs::Hammer2Blockset>()
                .as_blockref(),
            strict,
        )?
    };
    // On success, set perms, mtime, flags, etc.
    // On failure, rename file to .corrupted.
    fp.set_len(inode.meta.size)?;
    unsafe {
        let path1 = libhammer2::util::new_cstring!(path1)?;
        let ppath1 = path1.as_ptr();
        let tvs = inode.meta.get_utimes_timeval();
        let error = libhammer2::os::utimes(ppath1, tvs.as_ptr());
        if error != 0 {
            log::error!("utimes {path1:?} {tvs:?} {error}");
        }
        let _ = libhammer2::os::chown(
            ppath1,
            hammer2_to_unix_xid(&inode.meta.uid),
            hammer2_to_unix_xid(&inode.meta.gid),
        );
    }
    get_entry_mut!(ihash1, hid).link_file_path = if res {
        unsafe {
            let path1 = libhammer2::util::new_cstring!(path1)?;
            let ppath1 = path1.as_ptr();
            let _ = libhammer2::os::chmod(ppath1, inode.meta.mode);
            let _ = libhammer2::os::chflags(ppath1, inode.meta.uflags.into());
        }
        path1.to_string()
    } else {
        let path2 = format!("{path1}.corrupted");
        std::fs::rename(path1, &path2)?;
        path2
    };
    Ok(res)
}

// Dumps the data records for an inode to the target file, returns
// TRUE on success, FALSE if corruption was detected.
fn dump_file_data(
    fso: &mut libhammer2::ondisk::Ondisk,
    fp: &mut std::fs::File,
    fsize: u64,
    base: &[&libhammer2::fs::Hammer2Blockref],
    strict: bool,
) -> hammer2_utils::Result<bool> {
    let mut res = true;
    for bref in base {
        if bref.typ == libhammer2::fs::HAMMER2_BREF_TYPE_EMPTY || bref.data_off == 0 {
            continue;
        }
        let Some(vol) = fso.get_volume_mut(bref.data_off) else {
            continue;
        };
        // DragonFly has redundant EMPTY check here
        let poff = (bref.data_off - vol.get_offset()) & !MAX_RADIX_MASK;
        let psize = 1 << (bref.data_off & MAX_RADIX_MASK);
        if psize > libhammer2::fs::HAMMER2_PBUFSIZE {
            res = false;
            continue;
        }
        let Ok(data) = vol.preadx(psize, poff) else {
            res = false;
            continue;
        };
        if !validate_crc(bref, &data, strict) {
            res = false;
            continue;
        }
        match bref.typ {
            libhammer2::fs::HAMMER2_BREF_TYPE_INDIRECT => {
                dump_file_data(fso, fp, fsize, &libhammer2::extra::media_as(&data), strict)?;
            }
            libhammer2::fs::HAMMER2_BREF_TYPE_DATA => 'data: {
                let nsize = 1 << bref.keybits;
                if nsize > libhammer2::fs::HAMMER2_PBUFSIZE {
                    res = false;
                    break 'data;
                }
                let dbuf = match libhammer2::fs::dec_comp(bref.methods) {
                    libhammer2::fs::HAMMER2_COMP_LZ4 => {
                        libhammer2::lz4::decompress(&data, nsize.try_into()?)?
                    }
                    libhammer2::fs::HAMMER2_COMP_ZLIB => {
                        libhammer2::zlib::decompress(&data, nsize.try_into()?)?
                    }
                    _ => data, // leave in current form
                };
                libhammer2::util::seek_set(fp, bref.key)?;
                if bref.key + u64::try_from(dbuf.len())? > fsize {
                    fp.write_all(&dbuf[..(fsize - bref.key).try_into()?])?;
                } else {
                    fp.write_all(&dbuf)?;
                }
            }
            _ => (),
        }
    }
    Ok(res)
}

// Validate the bref data target.  The recovery scan will often attempt to
// validate invalid elements, so don't spew errors to stderr on failure.
fn validate_crc(bref: &libhammer2::fs::Hammer2Blockref, data: &[u8], strict: bool) -> bool {
    let check_algo = libhammer2::fs::dec_check(bref.methods);
    match check_algo {
        libhammer2::fs::HAMMER2_CHECK_NONE | libhammer2::fs::HAMMER2_CHECK_DISABLED => !strict,
        libhammer2::fs::HAMMER2_CHECK_ISCSI32 => {
            bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckIscsi>()
                .value
                == icrc32::iscsi_crc32(data)
        }
        libhammer2::fs::HAMMER2_CHECK_XXHASH64 => {
            bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckXxhash64>()
                .value
                == libhammer2::xxhash::xxh64(data)
        }
        libhammer2::fs::HAMMER2_CHECK_SHA192 => {
            bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckSha256>()
                .data
                == libhammer2::sha::sha256(data).as_slice()
        }
        libhammer2::fs::HAMMER2_CHECK_FREEMAP => {
            bref.check_as::<libhammer2::fs::Hammer2BlockrefCheckFreemap>()
                .icrc32
                == icrc32::iscsi_crc32(data)
        }
        _ => panic!("{check_algo}"),
    }
}

// Convert a hammer2 uuid to a uid or gid.
fn hammer2_to_unix_xid(uuid: &[u8]) -> u32 {
    uuid[12].into()
}

// Read from disk image, with caching to improve performance.
// Use a very simple LRU algo with 16 entries, linearly checked.
#[derive(Debug, Default)]
struct SdcCache {
    cache: Vec<SdcCacheEntry>,
    last: u64,
}

impl SdcCache {
    fn new() -> Self {
        Self {
            cache: vec![SdcCacheEntry::new(); 16],
            last: 0,
        }
    }

    fn get_last(&mut self) -> u64 {
        self.last += 1;
        self.last
    }

    fn cache_read<'a>(
        &'a mut self,
        fso: &mut libhammer2::ondisk::Ondisk,
        data_off: u64,
    ) -> hammer2_utils::Result<(&'a [u8], u64)> {
        // Translate logical offset to volume and physical offset.
        // Return NULL with *bytesp set to 0 to indicate pre-I/O
        // sanity check failure.
        let Some(vol) = fso.get_volume_mut(data_off) else {
            return Ok((&[], 0));
        };
        let poff = (data_off - vol.get_offset()) & !MAX_RADIX_MASK;
        let bytes = 1 << (data_off & MAX_RADIX_MASK);
        let pbase = poff & !libhammer2::fs::HAMMER2_PBUFMASK;
        // Must not straddle two full-sized hammer2 blocks.
        if ((poff ^ (poff + bytes - 1)) & !libhammer2::fs::HAMMER2_PBUFMASK) != 0 {
            return Ok((&[], 0));
        }
        // I/Os are powers of 2 in size and must be aligned to at least
        // the I/O size.
        if ((poff & !MAX_RADIX_MASK) & (bytes - 1)) != 0 {
            return Ok((&[], 0));
        }
        // LRU match lookup
        let mut worst = 0;
        for i in 0..self.cache.len() {
            let c = &self.cache[i];
            if c.volid == vol.get_id() && c.offset == pbase {
                let last = self.get_last();
                let c = &mut self.cache[i];
                c.last = last;
                return Ok((&c.buf[(poff - pbase).try_into()?..], bytes));
            }
            if worst == 0 || self.cache[worst].last > c.last {
                worst = i;
            }
        }
        // Fallback to I/O if not found, using oldest entry.
        //
        // On failure we leave (*bytesp) intact to indicate that an I/O
        // was attempted.
        let last = self.get_last();
        let c = &mut self.cache[worst];
        c.volid = vol.get_id();
        c.offset = pbase;
        c.last = last;
        let Ok(v) = vol.preadx(libhammer2::fs::HAMMER2_PBUFSIZE, pbase) else {
            c.offset = u64::MAX;
            c.last = 0;
            return Ok((&[], bytes));
        };
        c.buf.copy_from_slice(&v);
        Ok((&c.buf[(poff - pbase).try_into()?..], bytes))
    }
}

#[derive(Clone, Debug, Default)]
struct SdcCacheEntry {
    buf: Vec<u8>,
    volid: usize,
    offset: u64,
    last: u64,
}

impl SdcCacheEntry {
    fn new() -> Self {
        Self {
            buf: vec![0; libhammer2::fs::HAMMER2_PBUFSIZE as usize],
            volid: libhammer2::fs::HAMMER2_MAX_VOLUMES.into(),
            offset: 0,
            last: 0,
        }
    }
}

// Recover the specified file.
//
// Basically do a raw scan of the drive image looking for directory entries
// and inodes.  Index all inodes found, including copies, and filter
// directory entries for the requested filename to locate inode numbers.
//
// All copies that are located are written to destdir with a suffix .00001,
// .00002, etc.
#[allow(clippy::too_many_lines)]
pub(crate) fn run(
    devpath: &str,
    pathname: &str,
    destdir: &str,
    strict: bool,
    isafile: bool,
    opt: &crate::Opt,
) -> hammer2_utils::Result<()> {
    const INODES_PER_BLOCK: usize =
        (libhammer2::fs::HAMMER2_PBUFSIZE / libhammer2::fs::HAMMER2_INODE_BYTES) as usize;
    const DISPMODULO: u64 = HTABLE_SIZE / 32768;
    InodeEntry::init();
    TopologyEntry::init();
    let mut fso = libhammer2::ondisk::init(devpath, true)?;
    let mut ihash1 = InodeEntryHash::new();
    let mut ihash2 = InodeEntryHash::new();
    let mut thash = TopologyEntryHash::new();
    let mut tihash = TopologyInodeEntryHash::new();
    let mut tbhash = TopologyBlockrefEntryHash::new();
    let mut nhash = NegativeEntryHash::new();
    let mut stats = Stats::new();
    let mut sdc = SdcCache::new();

    // Media Pass
    //
    // Look for blockrefs that point to inodes.  The blockrefs could
    // be bogus since we aren't validating them, but the combination
    // of a CRC that matches the inode content is fairly robust in
    // finding actual inodes.
    //
    // We also enter unvalidated inodes for inode #1 (PFS roots),
    // because there might not be any blockrefs pointing to some of
    // them.  We need these to be able to locate directory entries
    // under the roots.
    //
    // At the moment we do not try to enter unvalidated directory
    // entries, since this will result in a massive number of false
    // hits.
    println!("MEDIA PASS");
    let mut media_bytes = 0;
    let mut loff = 0;
    while let Some(vol) = fso.get_volume(loff) {
        let offset = vol.get_offset();
        let size = vol.get_size();
        let next_loff = offset + size;
        let mut poff = loff - offset;
        let mut xdisp = 0;
        while poff < size {
            let vol = fso.get_volume_mut(loff).ok_or(nix::errno::Errno::ENOENT)?;
            let Ok(data) = vol.preadx(libhammer2::fs::HAMMER2_PBUFSIZE, poff) else {
                // Try to skip possible I/O error.
                poff += libhammer2::fs::HAMMER2_PBUFSIZE;
                continue;
            };
            let brefs = libhammer2::extra::media_as::<libhammer2::fs::Hammer2Blockref>(&data);
            assert_eq!(brefs.len(), libhammer2::fs::HAMMER2_IND_COUNT_MAX);
            for bref in brefs {
                // Found a possible inode.
                match bref.typ {
                    libhammer2::fs::HAMMER2_BREF_TYPE_INODE => {
                        // Note: preliminary bref filter is inside enter_inode().
                        enter_inode(
                            &mut fso,
                            &mut ihash1,
                            &mut ihash2,
                            &mut nhash,
                            &mut stats,
                            &mut sdc,
                            bref,
                            strict,
                        )?;
                    }
                    libhammer2::fs::HAMMER2_BREF_TYPE_DIRENT => {
                        // Go overboard and try to index
                        // anything that looks like a
                        // directory entry.  This might find
                        // entries whos inodes are no longer
                        // available, but will also generate
                        // a lot of false files.
                        // (Not implemented)
                    }
                    _ => (),
                }
            }
            // Look for possible root inodes.  We generally can't
            // find these by finding BREFs pointing to them because
            // the BREFs often hang off the volume header.
            //
            // These "inodes" could be seriously corrupt, but if
            // the bref tree is intact that is what we need to
            // get top-level directory entries.
            let inodes = libhammer2::extra::media_as::<libhammer2::fs::Hammer2InodeData>(&data);
            assert_eq!(inodes.len(), INODES_PER_BLOCK);
            for (i, &inode) in inodes.iter().enumerate() {
                if inode.meta.inum == 1
                    && inode.meta.iparent == 0
                    && inode.meta.typ == libhammer2::fs::HAMMER2_OBJTYPE_DIRECTORY
                    && (inode.meta.op_flags & libhammer2::fs::HAMMER2_OPFLAG_PFSROOT) != 0
                {
                    enter_inode_untested(
                        &mut ihash1,
                        &mut ihash2,
                        &mut stats,
                        inode,
                        poff + offset + u64::try_from(i * std::mem::size_of_val(inode))?,
                    );
                }
            }
            let n = u64::try_from(data.len())?;
            poff += n;
            media_bytes += n;
            // Update progress
            if !opt.quiet {
                let vol = fso.get_volume(loff).ok_or(nix::errno::Errno::ENOENT)?;
                xdisp += 1;
                if xdisp == DISPMODULO || poff == vol.get_size() - n {
                    xdisp = 0;
                    print!(
                        "{} inodes scanned, media {:6.2}/{:<3.2}G\r",
                        stats.inode,
                        media_bytes as f64 / 1_000_000_000_f64,
                        vol.get_size() as f64 / 1_000_000_000_f64
                    );
                    std::io::stdout().flush()?;
                }
            }
        }
        loff = next_loff;
    }

    // Restoration Pass
    //
    // Run through available directory inodes, which allows us to locate
    // and validate (crc check) their directory entry blockrefs and
    // construct absolute or relative paths through a recursion.
    //
    // When an absolute path is obtained the search is anchored on a
    // root inode.  When a relative path is obtained the search is
    // unanchored and will find all matching sub-paths.  For example,
    // if you look for ".cshrc" it will find ALL .cshrc's.  If you
    // look for "fubar/.cshsrc" it will find ALL .cshrc's residing
    // in a directory called fubar, however many there are.  But if
    // you look for "/fubar/srcs" it will only find the sub-tree
    // "/fubar/srcs" relative to PFS roots.
    //
    // We may not have indexed the PFS roots themselves, because they
    // often hang off of the volume header and might not have COW'd
    // references to them, so we use the "iparent" field in the inode
    // to detect top-level directories under those roots.
    println!(
        "\nInodes={}, Invalid_brefs={}, Invalid_hits={}",
        stats.inode, stats.negative, stats.negative_hits
    );
    println!("RESTORATION PASS");
    // Check for absolute path, else relative.
    let (pathname, abspath) = if pathname.starts_with('/') {
        (pathname.trim_start_matches('/'), true)
    } else {
        (pathname, false)
    };
    // Count root inodes.
    let mut root_max = 0;
    if let Some(v) = ihash1.get(&1) {
        for iscan in v {
            if iscan.inum == 1 {
                root_max += 1;
            }
        }
    }
    // Run through all directory inodes to locate validated
    // directory entries.  If an absolute path was specified
    // we start at root inodes.
    let mut root_count = 0;
    for i in 0..HTABLE_SIZE {
        if let Some(v) = ihash1.get(&i) {
            // Lists in DragonFly are front inserted, hence rev().
            // "scan roots..." is affected without rev().
            for j in (0..v.len()).rev() {
                let iscan = get_entry!(&ihash1, (i, j));
                // Absolute paths always start at root inodes,
                // otherwise we can start at any directory inode.
                if abspath && iscan.inum != 1 {
                    continue;
                }
                if iscan.typ != libhammer2::fs::HAMMER2_OBJTYPE_DIRECTORY {
                    continue;
                }
                // Progress down root inodes can be slow,
                // so print progress for each root inode.
                if i == 1 && iscan.inum == 1 && !opt.quiet {
                    root_count += 1;
                    print!(
                        "scan roots {:#x} {:#018x} (count {}/{})\r",
                        std::ptr::addr_of!(iscan) as u64,
                        iscan.data_off,
                        root_count,
                        root_max
                    );
                    std::io::stdout().flush()?;
                }
                // Primary match/recover recursion
                dump_tree(
                    &mut fso,
                    &mut ihash1,
                    &mut thash,
                    &mut tihash,
                    &mut tbhash,
                    &mut stats,
                    &mut sdc,
                    (i, j),
                    destdir,
                    pathname,
                    1,
                    1,
                    isafile,
                    strict,
                )?;
            }
        }
        if !opt.quiet && (i & (DISPMODULO - 1)) == DISPMODULO - 1 {
            if i == DISPMODULO - 1 {
                println!();
            }
            print!("Progress {i}/{HTABLE_SIZE}\r");
            std::io::stdout().flush()?;
        }
    }
    println!();
    println!("CLEANUP");
    println!(
        "TopoBRef stats: count={} dups={}",
        stats.topology_blockref, stats.topology_blockref_dup
    );
    println!(
        "TopoInode stats: count={} dups={}",
        stats.topology_inode, stats.topology_inode_dup
    );
    if opt.verbose {
        println!("ihash1: {} {:?}", ihash1.len(), get_vector_size(&ihash1));
        println!("ihash2: {} {:?}", ihash2.len(), get_vector_size(&ihash2));
        println!("thash: {} {:?}", thash.len(), get_vector_size(&thash));
        println!("tihash: {} {:?}", tihash.len(), get_vector_size(&tihash));
        println!("tbhash: {} {:?}", tbhash.len(), get_vector_size(&tbhash));
        println!("nhash: {} {:?}", nhash.len(), get_vector_size(&nhash));
        println!("{stats:#?}");
    }
    Ok(())
}

fn get_vector_size<T>(h: &std::collections::HashMap<u64, Vec<T>>) -> (usize, usize, usize) {
    let mut min = usize::MAX;
    let mut max = 0;
    let mut avg = 0;
    for v in h.values() {
        if v.len() < min {
            min = v.len();
        }
        if v.len() > max {
            max = v.len();
        }
        avg += v.len();
    }
    avg /= h.len();
    (min, max, avg)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_str_find() {
        assert_eq!("/".find(""), Some(0));
        assert_eq!("/".find('/'), Some(0));
        assert_eq!("/".find("//"), None);
        assert_eq!("/".find("xxx"), None);
    }

    #[test]
    fn test_str_trim_start_matches() {
        assert_eq!("/".trim_start_matches('/'), "");
        assert_eq!("//".trim_start_matches('/'), "");
        assert_eq!("/path/to/x".trim_start_matches('/'), "path/to/x");
        assert_eq!("//path/to/x".trim_start_matches('/'), "path/to/x");
        assert_eq!("//path/to/x/".trim_start_matches('/'), "path/to/x/");
        assert_eq!("//path//to//x".trim_start_matches('/'), "path//to//x");
    }

    #[test]
    fn test_inode_entry_id() {
        super::InodeEntry::init();
        let x = super::InodeEntry::new(0, 0, 0, 0);
        assert_eq!(x.id, 0);
        assert_eq!(x.data_off, 0);
        assert_eq!(x.inum, 0);
        assert_eq!(x.crc, 0);
        assert_eq!(x.typ, 0);
        assert!(!x.encountered);
        assert!(x.link_file_path.is_empty());
        assert_eq!(super::InodeEntry::new(0, 0, 0, 0).id, 1);
        assert_eq!(super::InodeEntry::new(0, 0, 0, 0).id, 2);
        super::InodeEntry::init();
        assert_eq!(super::InodeEntry::new(0, 0, 0, 0).id, 0);
    }

    #[test]
    fn test_topology_entry_id() {
        super::TopologyEntry::init();
        let x = super::TopologyEntry::new("");
        assert_eq!(x.id, 0);
        assert!(x.path.is_empty());
        assert_eq!(x.iterator, 1);
        assert_eq!(super::TopologyEntry::new("").id, 1);
        assert_eq!(super::TopologyEntry::new("").id, 2);
        super::TopologyEntry::init();
        assert_eq!(super::TopologyEntry::new("").id, 0);
    }
}
