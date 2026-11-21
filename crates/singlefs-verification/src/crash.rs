//! 崩溃点重放（D13（验证路线） 已定项 4）：拿录制流在内存里重建镜像，屏障与 FUA 按设备切段，
//! 段内每次写各取它的几态、任意组合：原地覆写的写多一态「新旧都读不出」（[`TearableInPlaceOverwrites`]），
//! 其余写的撕裂态并进「没持久」；每个状态跑恢复、oracle、池级 checker 与记录核对器（D13（验证路线） 已定项 7，故意不给编号）。
//! 内存池、保留写、崩溃镜像与两个结果类型在 `singlefs_verification::memory_pool`。

use crate::crash_judge_conclusions::recovery_passes_differ;
use std::collections::BTreeMap;

use std::ops::Range;
use std::sync::mpsc;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_format::NODE_POINTER_BYTES;

use crate::memory_pool::{
    newest_persisted_root, publishes_in, root_identity_of_write, root_identity_written_by,
    CrashImage, MemoryPool, PublishWrites, PublishedVersion, PublishedVersionWithoutAFile,
    RecordCheck, RecordStreamContinuity, RetainedWrite, WrittenContents, SECTOR_BYTES,
};
use crate::segments::StepKind;
use singlefs_core::system_configuration::SystemConfiguration;

mod crash_state_plan;
mod findings_and_tally;
mod oracle_and_version_table;
mod record_checker;
mod state_count;
mod state_judgement;
mod wall_clock_progress_lines;

pub use crash_state_plan::*;
pub use findings_and_tally::*;
pub use oracle_and_version_table::*;
pub use record_checker::*;
pub use state_count::*;
pub use state_judgement::*;
pub use wall_clock_progress_lines::*;

/// 判一个崩溃镜像（写表是枚举用的那一张，撕裂镜像与重放接在后面）；`judged_root_index` 是被判的那次根槽 FUA 写。
/// 文件从不消失的流：版本表是 [`CrashPointReplayVersionTable::with_a_file_only`]。
#[must_use]
pub fn judge_crash_image(
    image: &CrashImage<'_>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> CrashStateJudgement {
    judge_crash_image_against_the_version_table(
        image,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_pool::{closed_form_state_count_of_write_index_segments, SparseDevice};

    #[test]
    fn sparse_device_reads_zero_for_holes_and_flip_byte_only_touches_one_sector() {
        let mut device = SparseDevice::default();
        device.write(DeviceOffsetInBytes(1024), &[7u8; 1024]);
        assert_eq!(device.read(DeviceOffsetInBytes(512), 512), vec![0u8; 512]);
        assert_eq!(device.read(DeviceOffsetInBytes(1024), 512), vec![7u8; 512]);
        assert_eq!(
            device.written_sectors_in(DeviceOffsetInBytes(0), 4096),
            vec![2, 3]
        );
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 20);
        pool.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘")
            .write(DeviceOffsetInBytes(0), &[1u8; 1024]);
        pool.flip_byte(DeviceIdentity(0), DeviceOffsetInBytes(0), 600);
        let read_back =
            PoolReader::read(&pool, DeviceIdentity(0), DeviceOffsetInBytes(0), 1024).expect("读");
        assert_eq!(read_back[600], 0xfe);
        assert!(read_back
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 600 || *byte == 1));
        assert_eq!(
            closed_form_state_count_of_write_index_segments(&[vec![0, 1], vec![2]]),
            1 + 3 + 1
        );
    }
}
