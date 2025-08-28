// Much of this file is taken from the uutils coreutils package,
// distributed under the MIT License:
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed there:
// https://github.com/uutils/coreutils/blob/main/LICENSE

use uucore::fsext::{FsUsage, read_fs_list, statfs};

use std::path::Path;

/// Remaining space on a partition.
///
/// Uses (vendored) uu_df from uutils under the hood.
pub struct FreeSpace {
    /// Number of remaining GiB.
    free_space_gib: u64,
    /// Number of total GiB.
    total_space_gib: u64,
    /// Percentage of remaining available space.
    free_space_percent: u64,
}

impl FreeSpace {
    // TODO: errors
    pub fn from_path(path: &Path) -> FreeSpace {
        let path = path.canonicalize().unwrap();

        // Copied from uutils df package (MIT license)
        let mounts: Vec<_> = read_fs_list().unwrap();
        let maybe_mount_point = mounts
            .iter()
            .map(|m| (m, std::fs::canonicalize(&m.dev_name)))
            .filter(|m| m.1.is_ok())
            .map(|m| (m.0, m.1.ok().unwrap()))
            .find(|m| m.1.eq(&path))
            .map(|m| m.0);
        let mount_info = maybe_mount_point
            .or_else(|| {
                mounts
                    .iter()
                    .filter(|mi| path.starts_with(&mi.mount_dir))
                    .max_by_key(|mi| mi.mount_dir.len())
            })
            .unwrap();
        let stat_path = if mount_info.mount_dir.is_empty() {
            mount_info.dev_name.clone()
        } else {
            mount_info.mount_dir.clone()
        };
        let usage = FsUsage::new(statfs(stat_path).unwrap());

        // Calculate used/free space on partition
        let blocks_used = usage.blocks.saturating_sub(usage.bfree);
        let bytes_free = usage.blocksize * usage.bavail;

        let percent_used = if usage.blocks == 0 {
            0.0
        } else {
            blocks_used as f64 / (blocks_used + usage.bavail) as f64
        } * 100.0;
        // Round up to the higher percent
        let percent_used = percent_used.ceil();

        let gib_free = bytes_free as f64 / 1024.0 / 1024.0 / 1024.0;
        // Round down to the lower GiB
        let gib_free = gib_free.floor();

        let gib_total = (usage.blocks * usage.blocksize) as f64 / 1024.0 / 1024.0 / 1024.0;

        Self {
            free_space_gib: gib_free as u64,
            total_space_gib: gib_total as u64,
            free_space_percent: 100 - percent_used as u64,
        }
    }
}

impl std::fmt::Display for FreeSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}% ({} / {} GiB)",
            self.free_space_percent, self.free_space_gib, self.total_space_gib
        )
    }
}
