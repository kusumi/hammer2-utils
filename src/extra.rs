use crate::hammer2fs;
use crate::util;

impl hammer2fs::Hammer2Blockref {
    #[must_use]
    pub fn embed_as_dirent(&self) -> &hammer2fs::Hammer2DirentHead {
        util::align_to(&self.embed)
    }

    pub fn embed_as_dirent_mut(&mut self) -> &mut hammer2fs::Hammer2DirentHead {
        util::align_to_mut(&mut self.embed)
    }

    #[must_use]
    pub fn embed_as_stats(&self) -> &hammer2fs::Hammer2BlockrefEmbedStats {
        util::align_to(&self.embed)
    }

    pub fn embed_as_stats_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefEmbedStats {
        util::align_to_mut(&mut self.embed)
    }

    #[must_use]
    pub fn check_as_iscsi32(&self) -> &hammer2fs::Hammer2BlockrefCheckIscsi {
        util::align_to(&self.check)
    }

    pub fn check_as_iscsi32_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckIscsi {
        util::align_to_mut(&mut self.check)
    }

    #[must_use]
    pub fn check_as_xxhash64(&self) -> &hammer2fs::Hammer2BlockrefCheckXxhash64 {
        util::align_to(&self.check)
    }

    pub fn check_as_xxhash64_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckXxhash64 {
        util::align_to_mut(&mut self.check)
    }

    #[must_use]
    pub fn check_as_sha192(&self) -> &hammer2fs::Hammer2BlockrefCheckSha192 {
        util::align_to(&self.check)
    }

    pub fn check_as_sha192_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckSha192 {
        util::align_to_mut(&mut self.check)
    }

    #[must_use]
    pub fn check_as_sha256(&self) -> &hammer2fs::Hammer2BlockrefCheckSha256 {
        util::align_to(&self.check)
    }

    pub fn check_as_sha256_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckSha256 {
        util::align_to_mut(&mut self.check)
    }

    #[must_use]
    pub fn check_as_sha512(&self) -> &hammer2fs::Hammer2BlockrefCheckSha512 {
        util::align_to(&self.check)
    }

    pub fn check_as_sha512_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckSha512 {
        util::align_to_mut(&mut self.check)
    }

    #[must_use]
    pub fn check_as_freemap(&self) -> &hammer2fs::Hammer2BlockrefCheckFreemap {
        util::align_to(&self.check)
    }

    pub fn check_as_freemap_mut(&mut self) -> &mut hammer2fs::Hammer2BlockrefCheckFreemap {
        util::align_to_mut(&mut self.check)
    }
}

impl hammer2fs::Hammer2InodeData {
    #[must_use]
    pub fn u_as_blockset(&self) -> &hammer2fs::Hammer2Blockset {
        util::align_to(&self.u)
    }

    pub fn u_as_blockset_mut(&mut self) -> &mut hammer2fs::Hammer2Blockset {
        util::align_to_mut(&mut self.u)
    }

    #[must_use]
    pub fn u_as_data(&self) -> &[u8] {
        &self.u
    }

    pub fn u_as_data_mut(&mut self) -> &mut [u8] {
        &mut self.u
    }

    /// # Panics
    #[must_use]
    pub fn get_filename_string(&self) -> String {
        let n = usize::from(self.meta.name_len);
        if n <= hammer2fs::HAMMER2_INODE_MAXNAME {
            std::str::from_utf8(&self.filename[..n]).unwrap()
        } else {
            ""
        }
        .to_string()
    }
}

impl hammer2fs::Hammer2VolumeData {
    /// # Panics
    #[must_use]
    pub fn get_crc(&self, offset: u64, size: u64) -> u32 {
        let voldata = util::any_as_u8_slice(self);
        let beg = offset.try_into().unwrap();
        let end = (offset + size).try_into().unwrap();
        icrc32::iscsi_crc32(&voldata[beg..end])
    }
}
