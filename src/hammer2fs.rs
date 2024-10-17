pub const HAMMER2_RADIX_MIN: u8 = 10; // minimum allocation size 2^N

pub const HAMMER2_SEGSIZE: u64 = 1 << HAMMER2_FREEMAP_LEVEL0_RADIX;

pub const HAMMER2_PBUFRADIX: usize = 16; // physical buf (1<<16) bytes
pub const HAMMER2_PBUFSIZE: u64 = 65536;
pub const HAMMER2_LBUFRADIX: usize = 14; // logical buf (1<<14) bytes
pub const HAMMER2_LBUFSIZE: u64 = 16384;

pub const HAMMER2_SET_RADIX: u8 = 2; // radix 2 = 4 entries
pub const HAMMER2_SET_COUNT: usize = 1 << HAMMER2_SET_RADIX;
pub const HAMMER2_EMBEDDED_BYTES: u64 = 512; // inode blockset/dd size

pub const HAMMER2_PBUFMASK: u64 = HAMMER2_PBUFSIZE - 1;
pub const HAMMER2_LBUFMASK: u64 = HAMMER2_LBUFSIZE - 1;
pub const HAMMER2_SEGMASK: u64 = HAMMER2_SEGSIZE - 1;

pub const HAMMER2_UUID_STRING: &str = "5cbb9ad1-862d-11dc-a94d-01301bb8a9f5";

pub const HAMMER2_VOLUME_ALIGN: u64 = 8 * 1024 * 1024;
pub const HAMMER2_VOLUME_ALIGNMASK: u64 = HAMMER2_VOLUME_ALIGN - 1;
pub const HAMMER2_NEWFS_ALIGN: u64 = HAMMER2_VOLUME_ALIGN;
pub const HAMMER2_NEWFS_ALIGNMASK: u64 = HAMMER2_VOLUME_ALIGN - 1;

pub const HAMMER2_ZONE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const HAMMER2_ZONE_SEG: u64 = 4 * 1024 * 1024;
pub const HAMMER2_ZONE_BLOCKS_SEG: usize = (HAMMER2_ZONE_SEG / HAMMER2_PBUFSIZE) as usize;

pub const HAMMER2_ZONE_FREEMAP_INC: usize = 5; // 5 deep

pub const HAMMER2_ZONE_VOLHDR: usize = 0; // volume header or backup
pub const HAMMER2_ZONE_FREEMAP_00: usize = 1; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_01: usize = 6; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_02: usize = 11; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_03: usize = 16; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_04: usize = 21; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_05: usize = 26; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_06: usize = 31; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_07: usize = 36; // normal freemap rotation
pub const HAMMER2_ZONE_FREEMAP_END: usize = 41; // non-inclusive
                                                // zone 41-63 unused
pub const HAMMER2_ZONE_END: usize = 64; // non-inclusive

pub const HAMMER2_FREEMAP_LEVEL6_RADIX: usize = 64; // 16EB (end)
pub const HAMMER2_FREEMAP_LEVEL5_RADIX: usize = 62; // 4EB
pub const HAMMER2_FREEMAP_LEVEL4_RADIX: usize = 54; // 16PB
pub const HAMMER2_FREEMAP_LEVEL3_RADIX: usize = 46; // 64TB
pub const HAMMER2_FREEMAP_LEVEL2_RADIX: usize = 38; // 256GB
pub const HAMMER2_FREEMAP_LEVEL1_RADIX: usize = 30; // 1GB
pub const HAMMER2_FREEMAP_LEVEL0_RADIX: usize = 22; // 4MB (x 256 in l-1 leaf)

pub const HAMMER2_FREEMAP_LEVELN_PSIZE: u64 = 32768; // physical bytes

pub const HAMMER2_FREEMAP_LEVEL5_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL5_RADIX;
pub const HAMMER2_FREEMAP_LEVEL4_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL4_RADIX;
pub const HAMMER2_FREEMAP_LEVEL3_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL3_RADIX;
pub const HAMMER2_FREEMAP_LEVEL2_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL2_RADIX;
pub const HAMMER2_FREEMAP_LEVEL1_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL1_RADIX;
pub const HAMMER2_FREEMAP_LEVEL0_SIZE: u64 = 1u64 << HAMMER2_FREEMAP_LEVEL0_RADIX;

pub const HAMMER2_FREEMAP_LEVEL5_MASK: u64 = HAMMER2_FREEMAP_LEVEL5_SIZE - 1;
pub const HAMMER2_FREEMAP_LEVEL4_MASK: u64 = HAMMER2_FREEMAP_LEVEL4_SIZE - 1;
pub const HAMMER2_FREEMAP_LEVEL3_MASK: u64 = HAMMER2_FREEMAP_LEVEL3_SIZE - 1;
pub const HAMMER2_FREEMAP_LEVEL2_MASK: u64 = HAMMER2_FREEMAP_LEVEL2_SIZE - 1;
pub const HAMMER2_FREEMAP_LEVEL1_MASK: u64 = HAMMER2_FREEMAP_LEVEL1_SIZE - 1;
pub const HAMMER2_FREEMAP_LEVEL0_MASK: u64 = HAMMER2_FREEMAP_LEVEL0_SIZE - 1;

pub const HAMMER2_FREEMAP_COUNT: usize =
    HAMMER2_FREEMAP_LEVELN_PSIZE as usize / std::mem::size_of::<Hammer2BmapData>();

pub const HAMMER2_BMAP_ELEMENTS: usize = 8;

pub const HAMMER2_BOOT_MIN_BYTES: u64 = HAMMER2_VOLUME_ALIGN;
pub const HAMMER2_BOOT_NOM_BYTES: u64 = 64 * 1024 * 1024;
pub const HAMMER2_BOOT_MAX_BYTES: u64 = 256 * 1024 * 1024;

pub const HAMMER2_AUX_MIN_BYTES: u64 = HAMMER2_VOLUME_ALIGN;
pub const HAMMER2_AUX_NOM_BYTES: u64 = 256 * 1024 * 1024;
pub const HAMMER2_AUX_MAX_BYTES: u64 = 1024 * 1024 * 1024;

pub const HAMMER2_OFF_MASK: u64 = 0xFFFF_FFFF_FFFF_FFC0;
pub const HAMMER2_OFF_MASK_LO: u64 = HAMMER2_OFF_MASK & HAMMER2_PBUFMASK;
pub const HAMMER2_OFF_MASK_RADIX: u64 = 0x0000_0000_0000_003F;

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2DirentHead {
    pub inum: u64,   // inode number
    pub namlen: u16, // name length
    pub typ: u8,     // OBJTYPE_*
    pub unused0b: u8,
    pub unused0c: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hammer2Blockref {
    pub typ: u8,         // type of underlying item
    pub methods: u8,     // check method & compression method
    pub copyid: u8,      // specify which copy this is
    pub keybits: u8,     // #of keybits masked off 0=leaf
    pub vradix: u8,      // virtual data/meta-data size
    pub flags: u8,       // blockref flags
    pub leaf_count: u16, // leaf aggregation count
    pub key: u64,        // key specification
    pub mirror_tid: u64, // media flush topology & freemap
    pub modify_tid: u64, // clc modify (not propagated)
    pub data_off: u64,   // low 6 bits is phys size (radix)
    pub update_tid: u64, // clc modify (propagated upward)
    pub embed: [u8; 16],
    pub check: [u8; 64],
}

impl Default for Hammer2Blockref {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Hammer2Blockref {
    #[must_use]
    pub fn new(typ: u8) -> Self {
        Self {
            typ,
            methods: 0,
            copyid: 0,
            keybits: 0,
            vradix: 0,
            flags: 0,
            leaf_count: 0,
            key: 0,
            mirror_tid: 0,
            modify_tid: 0,
            data_off: 0,
            update_tid: 0,
            embed: [0; 16],
            check: [0; 64],
        }
    }

    #[must_use]
    pub fn new_empty() -> Self {
        Self::new(HAMMER2_BREF_TYPE_EMPTY)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefEmbedStats {
    pub data_count: u64,
    pub inode_count: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckIscsi {
    pub value: u32,
    pub reserved: [u32; 15],
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckXxhash64 {
    pub value: u64,
    pub reserved: [u64; 7],
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckSha192 {
    pub data: [u8; 24],
    pub reserved: [u64; 5],
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckSha256 {
    pub data: [u8; 32],
    pub reserved: [u8; 32],
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckSha512 {
    pub data: [u8; 64],
}

#[repr(C)]
#[derive(Debug)]
pub struct Hammer2BlockrefCheckFreemap {
    pub icrc32: u32,
    pub bigmask: u32, // available radixes
    pub avail: u64,   // total available bytes
    pub reserved: [u8; 48],
}

pub const HAMMER2_BLOCKREF_BYTES: u64 = 128; // blockref struct in bytes

pub const HAMMER2_BREF_TYPE_EMPTY: u8 = 0;
pub const HAMMER2_BREF_TYPE_INODE: u8 = 1;
pub const HAMMER2_BREF_TYPE_INDIRECT: u8 = 2;
pub const HAMMER2_BREF_TYPE_DATA: u8 = 3;
pub const HAMMER2_BREF_TYPE_DIRENT: u8 = 4;
pub const HAMMER2_BREF_TYPE_FREEMAP_NODE: u8 = 5;
pub const HAMMER2_BREF_TYPE_FREEMAP_LEAF: u8 = 6;
pub const HAMMER2_BREF_TYPE_INVALID: u8 = 7;
pub const HAMMER2_BREF_TYPE_FREEMAP: u8 = 254; // pseudo-type
pub const HAMMER2_BREF_TYPE_VOLUME: u8 = 255; // pseudo-type

pub const HAMMER2_BREF_FLAG_PFSROOT: u8 = 0x01; // see also related opflag

pub const HAMMER2_CHECK_NONE: u8 = 0;
pub const HAMMER2_CHECK_DISABLED: u8 = 1;
pub const HAMMER2_CHECK_ISCSI32: u8 = 2;
pub const HAMMER2_CHECK_XXHASH64: u8 = 3;
pub const HAMMER2_CHECK_SHA192: u8 = 4;
pub const HAMMER2_CHECK_FREEMAP: u8 = 5;
pub const HAMMER2_CHECK_DEFAULT: u8 = HAMMER2_CHECK_XXHASH64;

pub const HAMMER2_COMP_NONE: u8 = 0;
pub const HAMMER2_COMP_AUTOZERO: u8 = 1;
pub const HAMMER2_COMP_LZ4: u8 = 2;
pub const HAMMER2_COMP_ZLIB: u8 = 3;
pub const HAMMER2_COMP_DEFAULT: u8 = HAMMER2_COMP_LZ4;

// Encode/decode check mode and compression mode for bref.methods.
// The compression level is not encoded in bref.methods.
#[must_use]
pub fn enc_check(n: u8) -> u8 {
    (n & 15) << 4
}

#[must_use]
pub fn dec_check(n: u8) -> u8 {
    (n >> 4) & 15
}

#[must_use]
pub fn enc_comp(n: u8) -> u8 {
    n & 15
}

#[must_use]
pub fn dec_comp(n: u8) -> u8 {
    n & 15
}

// Encode/decode check or compression algorithm request in
// ipdata->meta.check_algo and ipdata->meta.comp_algo.
#[must_use]
pub fn enc_algo(n: u8) -> u8 {
    n
}

#[must_use]
pub fn dec_algo(n: u8) -> u8 {
    n & 15
}

#[must_use]
pub fn enc_level(n: u8) -> u8 {
    n << 4
}

#[must_use]
pub fn dec_level(n: u8) -> u8 {
    (n >> 4) & 15
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Hammer2Blockset {
    pub blockref: [Hammer2Blockref; HAMMER2_SET_COUNT],
}

impl Hammer2Blockset {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Hammer2BmapData {
    pub linear: u32,                           // 00 linear sub-granular allocation offset
    pub class: u16,                            // 04-05 clustering class ((type<<8)|radix)
    pub reserved06: u8,                        // 06
    pub reserved07: u8,                        // 07
    pub reserved08: u32,                       // 08
    pub reserved0c: u32,                       // 0C
    pub reserved10: u32,                       // 10
    pub reserved14: u32,                       // 14
    pub reserved18: u32,                       // 18
    pub avail: u32,                            // 1C
    pub reserved20: [u32; 8],                  // 20-3F
    pub bitmapq: [u64; HAMMER2_BMAP_ELEMENTS], // 40-7F 512 bits manages 4MB of storage
}

impl Hammer2BmapData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

pub const HAMMER2_INODE_BYTES: u64 = 1024; // (asserted by code)
pub const HAMMER2_INODE_MAXNAME: usize = 256; // maximum name in bytes
pub const HAMMER2_INODE_VERSION_ONE: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Hammer2InodeMeta {
    pub version: u16,    // 0000 inode data version
    pub reserved02: u8,  // 0002
    pub pfs_subtype: u8, // 0003 pfs sub-type

    pub uflags: u32,   // 0004 chflags
    pub rmajor: u32,   // 0008 available for device nodes
    pub rminor: u32,   // 000C available for device nodes
    pub ctime: u64,    // 0010 inode change time
    pub mtime: u64,    // 0018 modified time
    pub atime: u64,    // 0020 access time (unsupported)
    pub btime: u64,    // 0028 birth time
    pub uid: [u8; 16], // 0030 uid / degenerate unix uid
    pub gid: [u8; 16], // 0040 gid / degenerate unix gid

    pub typ: u8,        // 0050 object type
    pub op_flags: u8,   // 0051 operational flags
    pub cap_flags: u16, // 0052 capability flags
    pub mode: u32,      // 0054 unix modes (typ low 16 bits)

    pub inum: u64,      // 0058 inode number
    pub size: u64,      // 0060 size of file
    pub nlinks: u64,    // 0068 hard links (typ only dirs)
    pub iparent: u64,   // 0070 nominal parent inum
    pub name_key: u64,  // 0078 full filename key
    pub name_len: u16,  // 0080 filename length
    pub ncopies: u8,    // 0082 ncopies to local media
    pub comp_algo: u8,  // 0083 compression request & algo
    pub unused84: u8,   // 0084
    pub check_algo: u8, // 0085 check code request & algo

    pub pfs_nmasters: u8,   // 0086 (if PFSROOT) if multi-master
    pub pfs_type: u8,       // 0087 (if PFSROOT) node type
    pub pfs_inum: u64,      // 0088 (if PFSROOT) inum allocator
    pub pfs_clid: [u8; 16], // 0090 (if PFSROOT) cluster uuid
    pub pfs_fsid: [u8; 16], // 00A0 (if PFSROOT) unique uuid

    pub data_quota: u64,  // 00B0 subtree quota in bytes
    pub unusedb8: u64,    // 00B8
    pub inode_quota: u64, // 00C0 subtree quota inode count
    pub unusedc8: u64,    // 00C8

    pub pfs_lsnap_tid: u64, // 00D0 last snapshot tid
    pub reservedd8: u64,    // 00D8 (avail)

    pub decrypt_check: u64,   // 00E0 decryption validator
    pub reservede8: [u64; 3], // 00E8/F0/F8
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Hammer2InodeData {
    pub meta: Hammer2InodeMeta,                   // 0000-00FF
    pub filename: [u8; HAMMER2_INODE_MAXNAME],    // 0100-01FF (256 char, unterminated)
    pub u: [u8; HAMMER2_EMBEDDED_BYTES as usize], // 0200-03FF (64x8 = 512 bytes)
}

impl Default for Hammer2InodeData {
    fn default() -> Self {
        Self::new()
    }
}

impl Hammer2InodeData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            meta: Hammer2InodeMeta {
                ..Default::default()
            },
            filename: [0; HAMMER2_INODE_MAXNAME],
            u: [0; HAMMER2_EMBEDDED_BYTES as usize],
        }
    }
}

pub const HAMMER2_OPFLAG_DIRECTDATA: u8 = 0x01;
pub const HAMMER2_OPFLAG_PFSROOT: u8 = 0x02;

pub const HAMMER2_OBJTYPE_UNKNOWN: u8 = 0;
pub const HAMMER2_OBJTYPE_DIRECTORY: u8 = 1;
pub const HAMMER2_OBJTYPE_REGFILE: u8 = 2;
pub const HAMMER2_OBJTYPE_FIFO: u8 = 4;
pub const HAMMER2_OBJTYPE_CDEV: u8 = 5;
pub const HAMMER2_OBJTYPE_BDEV: u8 = 6;
pub const HAMMER2_OBJTYPE_SOFTLINK: u8 = 7;
pub const HAMMER2_OBJTYPE_UNUSED08: u8 = 8;
pub const HAMMER2_OBJTYPE_SOCKET: u8 = 9;
pub const HAMMER2_OBJTYPE_WHITEOUT: u8 = 10;

pub const HAMMER2_COPYID_LOCAL: u8 = u8::MAX;

pub const HAMMER2_PFSTYPE_NONE: u8 = 0x00;
pub const HAMMER2_PFSTYPE_CACHE: u8 = 0x01;
pub const HAMMER2_PFSTYPE_SLAVE: u8 = 0x03;
pub const HAMMER2_PFSTYPE_SOFT_SLAVE: u8 = 0x04;
pub const HAMMER2_PFSTYPE_SOFT_MASTER: u8 = 0x05;
pub const HAMMER2_PFSTYPE_MASTER: u8 = 0x06;
pub const HAMMER2_PFSTYPE_SUPROOT: u8 = 0x08;
pub const HAMMER2_PFSTYPE_DUMMY: u8 = 0x09;
pub const HAMMER2_PFSTYPE_MAX: u8 = 16;

pub const HAMMER2_PFSSUBTYPE_NONE: u8 = 0;
pub const HAMMER2_PFSSUBTYPE_SNAPSHOT: u8 = 1; // manual/managed snapshot
pub const HAMMER2_PFSSUBTYPE_AUTOSNAP: u8 = 2; // automatic snapshot

pub const HAMMER2_VOLUME_ID_HBO: u64 = 0x4841_4d32_0517_2011;
pub const HAMMER2_VOLUME_ID_ABO: u64 = 0x1120_1705_324d_4148;

pub const HAMMER2_MAX_VOLUMES: u8 = 64;
pub const HAMMER2_ROOT_VOLUME: u8 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Hammer2VolumeData {
    // sector #0 - 512 bytes
    pub magic: u64,     // 0000 Signature
    pub boot_beg: u64,  // 0008 Boot area (future)
    pub boot_end: u64,  // 0010 (size = end - beg)
    pub aux_beg: u64,   // 0018 Aux area (future)
    pub aux_end: u64,   // 0020 (size = end - beg)
    pub volu_size: u64, // 0028 Volume size, bytes

    pub version: u32,        // 0030
    pub flags: u32,          // 0034
    pub copyid: u8,          // 0038 copyid of phys vol
    pub freemap_version: u8, // 0039 freemap algorithm
    pub peer_type: u8,       // 003A HAMMER2_PEER_xxx
    pub volu_id: u8,         // 003B
    pub nvolumes: u8,        // 003C
    pub reserved003d: u8,    // 003D
    pub reserved003e: u16,   // 003E

    pub fsid: [u8; 16],   // 0040
    pub fstype: [u8; 16], // 0050

    pub allocator_size: u64, // 0060 Total data space
    pub allocator_free: u64, // 0068    Free space
    pub allocator_beg: u64,  // 0070 Initial allocations

    pub mirror_tid: u64,        // 0078 committed tid (vol)
    pub reserved0080: u64,      // 0080
    pub reserved0088: u64,      // 0088
    pub freemap_tid: u64,       // 0090 committed tid (fmap)
    pub bulkfree_tid: u64,      // 0098 bulkfree incremental
    pub reserved00a0: [u64; 4], // 00A0-00BF

    pub total_size: u64, // 00C0 Total volume size, bytes

    pub copyexists: [u32; 8],    // 00C8-00E7 unused
    pub reserved00e8: [u8; 248], // 00E8-01DF

    pub icrc_sects: [u32; 8], // 01E0-01FF

    // sector #1 - 512 bytes
    pub sroot_blockset: Hammer2Blockset, // 0200-03FF Superroot dir

    // sector #2-6
    pub sector2: [u8; 512],                // 0400-05FF reserved
    pub sector3: [u8; 512],                // 0600-07FF reserved
    pub freemap_blockset: Hammer2Blockset, // 0800-09FF freemap
    pub sector5: [u8; 512],                // 0A00-0BFF reserved
    pub sector6: [u8; 512],                // 0C00-0DFF reserved

    // sector #7 - 512 bytes
    pub volu_loff: [u64; HAMMER2_MAX_VOLUMES as usize],

    // sector #8-71 - 32768 bytes for unused 256 volconf array.
    pub reserved_volconf: [u8; 0x8000], // 1000-8FFF reserved

    pub reserved9000: [u8; 0x6FFC], // 9000-FFFB reserved

    pub icrc_volheader: u32, // FFFC-FFFF full volume icrc
}

impl Default for Hammer2VolumeData {
    fn default() -> Self {
        Self::new()
    }
}

impl Hammer2VolumeData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            magic: 0,
            boot_beg: 0,
            boot_end: 0,
            aux_beg: 0,
            aux_end: 0,
            volu_size: 0,
            version: 0,
            flags: 0,
            copyid: 0,
            freemap_version: 0,
            peer_type: 0,
            volu_id: 0,
            nvolumes: 0,
            reserved003d: 0,
            reserved003e: 0,
            fsid: [0; 16],
            fstype: [0; 16],
            allocator_size: 0,
            allocator_free: 0,
            allocator_beg: 0,
            mirror_tid: 0,
            reserved0080: 0,
            reserved0088: 0,
            freemap_tid: 0,
            bulkfree_tid: 0,
            reserved00a0: [0; 4],
            total_size: 0,
            copyexists: [0; 8],
            reserved00e8: [0; 248],
            icrc_sects: [0; 8],
            sroot_blockset: Hammer2Blockset::new(),
            sector2: [0; 512],
            sector3: [0; 512],
            freemap_blockset: Hammer2Blockset::new(),
            sector5: [0; 512],
            sector6: [0; 512],
            volu_loff: [0; HAMMER2_MAX_VOLUMES as usize],
            reserved_volconf: [0; 0x8000],
            reserved9000: [0; 0x6FFC],
            icrc_volheader: 0,
        }
    }
}

pub const HAMMER2_VOL_ICRC_SECT0: usize = 7;
pub const HAMMER2_VOL_ICRC_SECT1: usize = 6;

pub const HAMMER2_VOLUME_BYTES: u64 = 65536;

pub const HAMMER2_VOLUME_ICRC0_OFF: u64 = 0;
pub const HAMMER2_VOLUME_ICRC1_OFF: u64 = 512;
pub const HAMMER2_VOLUME_ICRCVH_OFF: u64 = 0;

pub const HAMMER2_VOLUME_ICRC0_SIZE: u64 = 512 - 4;
pub const HAMMER2_VOLUME_ICRC1_SIZE: u64 = 512;
pub const HAMMER2_VOLUME_ICRCVH_SIZE: u64 = 65536 - 4;

pub const HAMMER2_VOL_VERSION_MULTI_VOLUMES: u32 = 2;

pub const HAMMER2_VOL_VERSION_MIN: u32 = 1;
pub const HAMMER2_VOL_VERSION_DEFAULT: u32 = HAMMER2_VOL_VERSION_MULTI_VOLUMES;
pub const HAMMER2_VOL_VERSION_WIP: u32 = HAMMER2_VOL_VERSION_MULTI_VOLUMES + 1;

pub const HAMMER2_NUM_VOLHDRS: usize = 4;

#[cfg(test)]
mod tests {
    #[test]
    fn test_struct_hammer2_dirent_head() {
        assert_eq!(std::mem::size_of::<super::Hammer2DirentHead>(), 16);
    }

    #[test]
    fn test_struct_hammer2_blockref() {
        assert_eq!(std::mem::size_of::<super::Hammer2Blockref>(), 128);
        assert_eq!(
            std::mem::size_of::<super::Hammer2Blockref>(),
            super::HAMMER2_BLOCKREF_BYTES.try_into().unwrap()
        );
        assert_eq!(super::HAMMER2_BREF_TYPE_EMPTY, 0);
    }

    #[test]
    fn test_struct_hammer2_blockref_embed() {
        assert_eq!(std::mem::size_of::<super::Hammer2DirentHead>(), 16);
        assert_eq!(std::mem::size_of::<super::Hammer2BlockrefEmbedStats>(), 16);
    }

    #[test]
    fn test_struct_hammer2_blockref_check() {
        assert_eq!(std::mem::size_of::<super::Hammer2BlockrefCheckIscsi>(), 64);
        assert_eq!(
            std::mem::size_of::<super::Hammer2BlockrefCheckXxhash64>(),
            64
        );
        assert_eq!(std::mem::size_of::<super::Hammer2BlockrefCheckSha192>(), 64);
        assert_eq!(std::mem::size_of::<super::Hammer2BlockrefCheckSha256>(), 64);
        assert_eq!(std::mem::size_of::<super::Hammer2BlockrefCheckSha512>(), 64);
        assert_eq!(
            std::mem::size_of::<super::Hammer2BlockrefCheckFreemap>(),
            64
        );
    }

    #[test]
    fn test_struct_hammer2_blockset() {
        assert_eq!(
            std::mem::size_of::<super::Hammer2Blockset>(),
            128 * super::HAMMER2_SET_COUNT
        );
    }

    #[test]
    fn test_struct_hammer2_bmap_data() {
        assert_eq!(std::mem::size_of::<super::Hammer2BmapData>(), 128);
    }

    #[test]
    fn test_struct_hammer2_inode_meta() {
        assert_eq!(std::mem::size_of::<super::Hammer2InodeMeta>(), 256);
    }

    #[test]
    fn test_struct_hammer2_inode_data() {
        assert_eq!(std::mem::size_of::<super::Hammer2InodeData>(), 1024);
        assert_eq!(
            std::mem::size_of::<super::Hammer2InodeData>(),
            super::HAMMER2_INODE_BYTES.try_into().unwrap()
        );
    }

    #[test]
    fn test_struct_hammer2_volume_data() {
        assert_eq!(std::mem::size_of::<super::Hammer2VolumeData>(), 65536);
        assert_eq!(
            std::mem::size_of::<super::Hammer2VolumeData>(),
            super::HAMMER2_VOLUME_BYTES.try_into().unwrap()
        );
    }
}
