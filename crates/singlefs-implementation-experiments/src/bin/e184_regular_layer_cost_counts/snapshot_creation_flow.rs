use singlefs_core::address::InstanceGeneration;
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
use singlefs_verification::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, PublishedVersionWithoutAFile, RetainedWrite,
};
use singlefs_verification::segments::StepKind;

use super::experiment_pool::{build_pool, file_content, geometry, parameters};

/// 建快照在新池新建文件那个实例里发。
const INSTANCE_OF_THE_SNAPSHOT_CREATION: InstanceGeneration = InstanceGeneration(1);

/// 起点镜像（mkfs 之后）、写表与段、被判的根槽写（最后一条）、版本表：A 与建快照那一版都是第一个文件，
/// 这条流里没有文件从哪一版起没有了的那几版（那一张是空的）。
pub struct PreparedSnapshotCreationFlow {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

/// 录建快照流并摆好（`tag` 是镜像文件名里那一段）：造池到 A，在 A 上建一个快照 S，从 mkfs 之后的录制流切写表与段。
///
/// # Panics
/// 建快照被拒；录制流里没有一条根槽写。
pub fn prepare_the_snapshot_creation_flow(tag: &str) -> PreparedSnapshotCreationFlow {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    let snapshot_version = {
        let parameters_of_the_pool = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters_of_the_pool, open_devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &first_file_version,
            INSTANCE_OF_THE_SNAPSHOT_CREATION,
        )
        .expect("建快照")
    };
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let versions_with_a_file = [&first_file_version, &snapshot_version]
        .into_iter()
        .map(|version| PublishedVersion {
            instance: version.root.instance,
            checkpoint_txg: version.root.checkpoint_txg,
            content: file_content(),
        })
        .collect();
    PreparedSnapshotCreationFlow {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file: Vec::new(),
    }
}
