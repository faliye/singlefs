//! 只读快照三条崩溃流共用的搭建（里程碑四一·4，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S6）：
//! 公共前缀是 `common::build_pool` 那一段（mkfs → 取号 → 暖机 → 新池新建文件 A，第一个文件 3000 字节、一个数据单元），
//! 三条流都从 A 之后建一个快照 S 起头，再按流各接几步：
//! - 建快照流：A → S；
//! - 快照下覆盖写与删文件流：A → S → 覆盖写 W（换下的数据单元、extent 根、inode 叶容器与根都生在 S 之前，进 deadlist）→ unlink 意图 → 最后一批；
//! - 删快照续做流（`S W0 X0 C`）：A → S → 覆盖写 W → 删快照意图 → 三批 → 最后一批（批大小 1，deadlist 四条一批放一条，最后一批删那一对条目）；
//! - 快照下删文件流：A → S → unlink 意图 → 最后一批（A 的四个用户可见单元都生在 S 之前，unlink 把它们放进 deadlist：覆盖写流走不到的那条路）。
//!
//! 这一份只录流、交回每一步发出的那段录制流（下标区间）与它的输出，再给观察者用的两样：每一版在快照这几样上的样子（可写挂载续做之后与录流时
//! 那一版比），以及在一张崩溃镜像上可写挂载一次、续做走到空。崩溃点重放与崩溃放量那两路都调这里的同一份录法，写表逐字节相同。
//! 这一份只引 harness 档与实现，不引 checker 档：崩溃点的设点、观察者的计数与判定都在 checker 档的用例里。
#![allow(
    dead_code,
    reason = "共用的搭建模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use std::ops::Range;

use super::common::{
    build_pool, file_content, geometry, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration, TreeIdentifier};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    Mounted, ShadowLedger,
};
use singlefs_core::records::{DeadlistEntry, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::recovery::{read_the_first_file_of_a_snapshot, PoolReader, SnapshotFileReading};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_create_snapshot, publish_snapshot_destruction_intent,
    publish_snapshot_destruction_step, publish_unlink_intent, publish_unlink_step, PoolVersion,
    PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::scenario::{deletion_history_batch_size, deletion_history_switches};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这一份的仓内路径（与 `file!()` 归一之后相同）。
pub const THIS_FILE: &str = "crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs";

/// 三条流的发布都在实例 1 上（`build_pool` 取的号）。
pub const INSTANCE_OF_THE_FLOWS: InstanceGeneration = InstanceGeneration(1);

/// unlink 的是第一个文件。
pub const UNLINKED_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// unlink 意图那次的调用方时间比覆盖写晚：与那条记录现有的时间不同，意图写的是不是调用方给的那一个分辨得出。
pub const SECONDS_FROM_THE_OVERWRITE_TO_THE_UNLINK_INTENT: u64 = 60;

/// 快照下覆盖写换下的、被快照引用的用户可见单元：第一个文件一个数据单元，数据单元、extent 根、inode 叶容器、inode 根（判决 S3 的六类角色里
/// 这个文件用到的四类）。删快照按批大小 1 放它们：三批各放一条，最后一批放最后一条、删那一对条目。
pub const UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT: usize = 4;

/// 删快照的批数（不含最后一批）：deadlist 四条、批大小 1（`deletion_history_batch_size`），留 3、留 2、留 1 三批，再最后一批。
/// 步数按计划定死，不按「快照删完了没有」停：最后一批漏放一条（必红世界）时录流照样录完，红在判器上，不红在搭建上。
pub const SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH: usize =
    UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT - 1;

/// 覆盖写的内容：一个数据单元装得下、与第一个文件（`common::file_content`）不同的一份。
#[must_use]
pub fn overwrite_content() -> Vec<u8> {
    let content: Vec<u8> = (0..2700_usize)
        .map(|index| u8::try_from((index * 13 + 5) % 239).expect("小于 256"))
        .collect();
    assert!(
        content.len() <= data_unit_payload_capacity(),
        "覆盖写的内容一个数据单元装得下（三条流的文件都取一个数据单元，判决 S6）"
    );
    content
}

/// 三条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotCrashFlow {
    /// A → S。
    SnapshotCreation,
    /// A → S → 覆盖写 → unlink 意图 → 最后一批。
    OverwriteAndUnlinkUnderASnapshot,
    /// A → S → 覆盖写 → 删快照意图 → 三批 → 最后一批（`S W0 X0 C`，崩溃之后可写挂载续做）。
    SnapshotDestructionAfterAnOverwrite,
    /// A → S → unlink 意图 → 最后一批：unlink 换下的四个单元都生在 S 之前，删除批把它们写进 deadlist、不释放。
    UnlinkUnderASnapshot,
}

/// 流里一步是哪一种发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotFlowStepKind {
    CreateSnapshot,
    OverwriteUnderTheSnapshot,
    UnlinkIntent,
    /// unlink 一个数据单元的文件：意图之后只有最后一批（批大小 1，「剩下的不多于一批」就是最后一批，没有中间的批）。
    UnlinkFinalBatch,
    SnapshotDestructionIntent,
    SnapshotDestructionBatch,
    SnapshotDestructionFinalBatch,
}

impl SnapshotFlowStepKind {
    /// 入口的名字：崩溃放量那一路按它设点（`<流程>.<周期>.<步>` 的流程那一段）。
    #[must_use]
    pub const fn entry_name(self) -> &'static str {
        match self {
            Self::CreateSnapshot => "publish_create_snapshot",
            Self::OverwriteUnderTheSnapshot => "publish_overwrite_under_the_snapshot",
            Self::UnlinkIntent => "publish_unlink_intent_under_the_snapshot",
            Self::UnlinkFinalBatch => "publish_unlink_step_under_the_snapshot",
            Self::SnapshotDestructionIntent => "publish_snapshot_destruction_intent",
            Self::SnapshotDestructionBatch | Self::SnapshotDestructionFinalBatch => {
                "publish_snapshot_destruction_step"
            }
        }
    }

    /// 计数的名字里这一种发布那一段（观察者按恢复落到哪一种发布的那一版分别计数）。
    #[must_use]
    pub const fn counter_name(self) -> &'static str {
        match self {
            Self::CreateSnapshot => "create_snapshot",
            Self::OverwriteUnderTheSnapshot => "overwrite_under_the_snapshot",
            Self::UnlinkIntent => "unlink_intent",
            Self::UnlinkFinalBatch => "unlink_final_batch",
            Self::SnapshotDestructionIntent => "snapshot_destruction_intent",
            Self::SnapshotDestructionBatch => "snapshot_destruction_batch",
            Self::SnapshotDestructionFinalBatch => "snapshot_destruction_final_batch",
        }
    }

    /// 这一版落在一串删除（unlink 或删快照）的中间：意图或不是最后一批的一批，可写挂载的续做要接着走。
    #[must_use]
    pub const fn leaves_a_deletion_unfinished(self) -> bool {
        match self {
            Self::UnlinkIntent
            | Self::SnapshotDestructionIntent
            | Self::SnapshotDestructionBatch => true,
            Self::CreateSnapshot
            | Self::OverwriteUnderTheSnapshot
            | Self::UnlinkFinalBatch
            | Self::SnapshotDestructionFinalBatch => false,
        }
    }
}

/// 流里一步：哪一种、它发出的那段录制流（绝对下标的半开区间）、它的输出。
#[derive(Clone, Debug)]
pub struct SnapshotFlowStep {
    pub kind: SnapshotFlowStepKind,
    pub operations: Range<usize>,
    pub output: TransactionOutput,
}

/// 录好的一条流：池（录制流在 `pool.stream` 上）、A 的输出、A 之后的每一步。
pub struct RecordedSnapshotFlow {
    pub flow: SnapshotCrashFlow,
    pub pool: BuiltPool,
    pub first_file: TransactionOutput,
    pub steps: Vec<SnapshotFlowStep>,
}

impl RecordedSnapshotFlow {
    /// 建快照那一步建的那个快照（码 14 的树号，判决第七节岔路 5 按它点名）。
    #[must_use]
    pub fn snapshot(&self) -> TreeIdentifier {
        let created = &self.steps.first().expect("三条流都从建快照起头").output;
        snapshot_trees_of(created)
            .last()
            .copied()
            .expect("建快照那一版的树表里有它那条码 14")
    }

    /// 某一种发布的每一步的输出，按发布的次序。
    #[must_use]
    pub fn outputs_of(&self, kind: SnapshotFlowStepKind) -> Vec<&TransactionOutput> {
        self.steps
            .iter()
            .filter(|step| step.kind == kind)
            .map(|step| &step.output)
            .collect()
    }

    /// A 之后每一步的入口名与它发出的那段录制流，按发布的次序：崩溃放量那一路照它设点（设点在 checker 档的公共模块
    /// `common_crash_points::crash_points_of_a_flow_after_the_first_file`，几条快照流与生命周期闭环流共用同一批节点）。
    #[must_use]
    pub fn crash_point_entries_of_the_steps(&self) -> Vec<(&'static str, Range<usize>)> {
        self.steps
            .iter()
            .map(|step| (step.kind.entry_name(), step.operations.clone()))
            .collect()
    }

    /// A 与之后每一步各一版，按发布的次序（观察者另读的输入，[`VersionOfTheFlow`]）。
    #[must_use]
    pub fn versions_of_the_flow(&self) -> Vec<VersionOfTheFlow> {
        let mut outputs: Vec<(Option<SnapshotFlowStepKind>, &TransactionOutput)> =
            vec![(None, &self.first_file)];
        outputs.extend(
            self.steps
                .iter()
                .map(|step| (Some(step.kind), &step.output)),
        );
        let mut versions = Vec::with_capacity(outputs.len());
        for (index, (kind, output)) in outputs.iter().enumerate() {
            // 落在一串删除中间的一版：续做走到这一串的最后一批，步数是从它往后到最后一批（含）还有几步。
            let mut settled_index = index;
            while outputs[settled_index]
                .0
                .is_some_and(SnapshotFlowStepKind::leaves_a_deletion_unfinished)
            {
                settled_index += 1;
                assert!(
                    settled_index < outputs.len(),
                    "一串删除录到了它的最后一批（步数按计划定死）"
                );
            }
            versions.push(VersionOfTheFlow {
                version: (output.root.instance, output.root.checkpoint_txg),
                kind: *kind,
                root: output.root,
                live_snapshots: live_snapshot_trees_of(output),
                continuation_steps_from_this_version: settled_index - index,
                shape_after_the_continuation: SnapshotShapeOfAVersion::of_a_version_of_trees(
                    outputs[settled_index].1,
                ),
            });
        }
        versions
    }
}

/// 一版的树表里码 14 条目的树号（快照按它点名），按树表的次序。
#[must_use]
pub fn snapshot_trees_of(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .collect()
}

/// 一版里不带「销毁中」记号的快照（活快照，I-3.7）。
#[must_use]
pub fn live_snapshot_trees_of(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    snapshot_trees_of(version)
        .into_iter()
        .filter(|tree| !version.snapshots_being_destroyed.contains(tree))
        .collect()
}

/// 录一条流（`tag` 是镜像文件名里那一段）。
///
/// # Panics
/// 任何一步发布被拒：三条流的每一步按条款都发得出去。
#[must_use]
pub fn record_the_snapshot_crash_flow(tag: &str, flow: SnapshotCrashFlow) -> RecordedSnapshotFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = vec![record_one_step(
        &mut pool,
        SnapshotFlowStepKind::CreateSnapshot,
        create_snapshot,
    )];
    match flow {
        SnapshotCrashFlow::SnapshotCreation => {}
        SnapshotCrashFlow::OverwriteAndUnlinkUnderASnapshot => {
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::OverwriteUnderTheSnapshot,
                overwrite_under_the_snapshot,
            ));
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::UnlinkIntent,
                unlink_intent,
            ));
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::UnlinkFinalBatch,
                unlink_step,
            ));
        }
        SnapshotCrashFlow::UnlinkUnderASnapshot => {
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::UnlinkIntent,
                unlink_intent,
            ));
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::UnlinkFinalBatch,
                unlink_step,
            ));
        }
        SnapshotCrashFlow::SnapshotDestructionAfterAnOverwrite => {
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::OverwriteUnderTheSnapshot,
                overwrite_under_the_snapshot,
            ));
            let snapshot = snapshot_trees_of(&steps[0].output)
                .last()
                .copied()
                .expect("建快照那一版的树表里有它那条码 14");
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::SnapshotDestructionIntent,
                |pool| snapshot_destruction_intent(pool, snapshot),
            ));
            for _batch_index in 0..SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH {
                steps.push(record_one_step(
                    &mut pool,
                    SnapshotFlowStepKind::SnapshotDestructionBatch,
                    |pool| snapshot_destruction_step(pool, snapshot),
                ));
            }
            steps.push(record_one_step(
                &mut pool,
                SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
                |pool| snapshot_destruction_step(pool, snapshot),
            ));
        }
    }
    RecordedSnapshotFlow {
        flow,
        pool,
        first_file,
        steps,
    }
}

/// 发一步、记下它发出的那段录制流，池接到它那一版上。
pub fn record_one_step(
    pool: &mut BuiltPool,
    kind: SnapshotFlowStepKind,
    publish: impl FnOnce(&mut BuiltPool) -> TransactionOutput,
) -> SnapshotFlowStep {
    let start = pool.stream.operation_count();
    let output = publish(pool);
    let end = pool.stream.operation_count();
    pool.output = output.clone();
    SnapshotFlowStep {
        kind,
        operations: start..end,
        output,
    }
}

/// 建快照（实例 1，接在池的现行那一版后面）。生命周期闭环流也走这一步。
pub fn create_snapshot(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("建快照")
}

/// 快照下覆盖写（内容是 [`overwrite_content`]）。生命周期闭环流的「再覆盖写」也走这一步。
pub fn overwrite_under_the_snapshot(pool: &mut BuiltPool) -> TransactionOutput {
    let previous = pool.output.clone();
    publish_overwrite_in_process(
        pool,
        &previous,
        &overwrite_content(),
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("快照下覆盖写")
}

/// unlink 第一个文件的意图（实例 1）。生命周期闭环流的「删文件」也走这一步。
pub fn unlink_intent(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        FIXED_WRITE_TIME_SECONDS
            + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE
            + SECONDS_FROM_THE_OVERWRITE_TO_THE_UNLINK_INTENT,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("快照下 unlink 意图")
}

/// unlink 的下一批（批大小 1，`deletion_history_batch_size`）。
pub fn unlink_step(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        deletion_history_batch_size(),
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("快照下 unlink 的一批")
}

fn snapshot_destruction_intent(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("删快照的意图")
}

fn snapshot_destruction_step(pool: &mut BuiltPool, snapshot: TreeIdentifier) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_snapshot_destruction_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        deletion_history_batch_size(),
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("删快照的一批")
}

// ============================== 崩溃点重放要的那几样 ==============================

/// 一条录好的流按崩溃点重放要的样子摆好：起点镜像（mkfs 之后）、写表与段、每次写在录制流里的下标（崩溃放量换崩溃点用）、
/// 被判的根槽写（最后一条）、版本表（有文件的那几版与文件从那一版起没有了的那几版，`crash::CrashPointReplayVersionTable`）、
/// 观察者另读的流里每一版（[`VersionOfTheFlow`]）。
pub struct PreparedSnapshotCrashFlow {
    pub recorded: RecordedSnapshotFlow,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    pub versions_of_the_flow: Vec<VersionOfTheFlow>,
}

/// 录一条流并摆好（[`PreparedSnapshotCrashFlow`]）。版本表逐版现推：A 与建快照那一版是第一个文件（`common::file_content`），
/// 覆盖写起是 [`overwrite_content`]（删快照那几版不动文件内容），unlink 意图那一版起没有文件。
/// 这里只摆、不核形状：写数、段形、deadlist 条数这些在用例里判器判完之后再核，改坏实现时先红在判器上。
#[must_use]
pub fn prepare_the_snapshot_crash_flow(
    tag: &str,
    flow: SnapshotCrashFlow,
) -> PreparedSnapshotCrashFlow {
    let recorded = record_the_snapshot_crash_flow(tag, flow);
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let mut versions_with_a_file = vec![PublishedVersion {
        instance: recorded.first_file.root.instance,
        checkpoint_txg: recorded.first_file.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut versions_without_a_file = Vec::new();
    let mut content_of_the_first_file = Some(file_content());
    for step in &recorded.steps {
        content_of_the_first_file = match step.kind {
            SnapshotFlowStepKind::CreateSnapshot
            | SnapshotFlowStepKind::SnapshotDestructionIntent
            | SnapshotFlowStepKind::SnapshotDestructionBatch
            | SnapshotFlowStepKind::SnapshotDestructionFinalBatch => content_of_the_first_file,
            SnapshotFlowStepKind::OverwriteUnderTheSnapshot => Some(overwrite_content()),
            SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch => None,
        };
        match &content_of_the_first_file {
            Some(content) => versions_with_a_file.push(PublishedVersion {
                instance: step.output.root.instance,
                checkpoint_txg: step.output.root.checkpoint_txg,
                content: content.clone(),
            }),
            None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance: step.output.root.instance,
                checkpoint_txg: step.output.root.checkpoint_txg,
            }),
        }
    }
    let versions_of_the_flow = recorded.versions_of_the_flow();
    PreparedSnapshotCrashFlow {
        recorded,
        base,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_flow,
    }
}

impl PreparedSnapshotCrashFlow {
    /// 观察者另读的输入的摘要（流里每一版，[`text_of_the_versions_of_the_flow`]）：进观察者的版本，录制流一变它就变。
    #[must_use]
    pub fn digest_of_the_versions_of_the_flow(
        &self,
    ) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        singlefs_harness::sha256::sha256_digest(
            text_of_the_versions_of_the_flow(&self.versions_of_the_flow).as_bytes(),
        )
    }

    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段（快档展开的段）。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }
}

/// 判段在可写挂载续做之后的盘上判池级 checker、一条不变量都不判违例时加一的计数（每个状态挂完的盘都判过：计数等于状态数）。
pub const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

// ============================== 观察者另读的输入 ==============================

/// 第一个文件在这一版里在不在（有树的一版：`PoolVersion::WithFile` 或 `WithTreesAndNoFile`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstFilePresence {
    Present,
    Absent,
}

/// 一版有树的版本在快照这几样上的样子：第一个文件在不在、树表里的快照（码 14 的树号）与其中销毁中的、删除队列、deadlist 条目与留存的映射条目
/// （都按盘上 key 的字节，集合比）。可写挂载续做之后从盘上重建的那一版与录流时同一串删除的最后一批那一版逐项相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotShapeOfAVersion {
    pub first_file: FirstFilePresence,
    pub snapshot_trees: Vec<TreeIdentifier>,
    pub snapshots_being_destroyed: Vec<TreeIdentifier>,
    pub files_being_deleted: Vec<InodeNumber>,
    pub deadlist_entry_keys: Vec<Vec<u8>>,
    pub kept_mapping_entry_keys: Vec<Vec<u8>>,
}

impl SnapshotShapeOfAVersion {
    /// 有树的一版（发布交回的输出，或挂载交回的现行版本里的那一份）。
    #[must_use]
    pub fn of_a_version_of_trees(version: &TransactionOutput) -> Self {
        // 与 `PoolVersion::of_a_version_of_trees` 同一个判据：有树、文件集合为空的一版没有第一个文件。
        let first_file = if version.is_a_version_with_trees_and_no_file() {
            FirstFilePresence::Absent
        } else {
            FirstFilePresence::Present
        };
        let mut deadlist_entry_keys: Vec<Vec<u8>> = version
            .deadlist_entries
            .iter()
            .map(DeadlistEntry::key_bytes)
            .collect();
        deadlist_entry_keys.sort();
        let mut kept_mapping_entry_keys: Vec<Vec<u8>> = version
            .mapping_entries_kept_for_the_snapshots
            .iter()
            .map(|(key, _locations)| key.clone())
            .collect();
        kept_mapping_entry_keys.sort();
        Self {
            first_file,
            snapshot_trees: snapshot_trees_of(version),
            snapshots_being_destroyed: version.snapshots_being_destroyed.clone(),
            files_being_deleted: version.deleted_inodes.clone(),
            deadlist_entry_keys,
            kept_mapping_entry_keys,
        }
    }

    /// 挂载交回的现行版本：没有树的一版（落到 A 之前）交回 `None`。
    #[must_use]
    pub fn of_the_pool_version(version: &PoolVersion) -> Option<Self> {
        match version {
            PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
                Some(Self::of_a_version_of_trees(output))
            }
            PoolVersion::WithoutFile(_) => None,
        }
    }
}

/// 流里的一版（A 与之后每一步）：根（实例，txg）、哪一种发布（A 是 `None`）、根记录（读快照从它的树表起）、这一版的活快照、
/// 恢复落到这一版之后可写挂载的续做还差几步、续做走完之后快照这几样该是什么样子。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheFlow {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub kind: Option<SnapshotFlowStepKind>,
    pub root: RootRecord,
    pub live_snapshots: Vec<TreeIdentifier>,
    pub continuation_steps_from_this_version: usize,
    pub shape_after_the_continuation: SnapshotShapeOfAVersion,
}

/// 观察者另读的输入写成文字（取摘要进观察者的版本：录制流一变它就变，看过的块不按旧的复用）。
#[must_use]
pub fn text_of_the_versions_of_the_flow(versions: &[VersionOfTheFlow]) -> String {
    let mut text = String::from("snapshot crash flow versions 1\n");
    for version in versions {
        text.push_str(&format!(
            "version instance={} txg={} kind={:?} live_snapshots={:?} continuation_steps={} shape_after_the_continuation={:?}\n",
            version.version.0 .0,
            version.version.1 .0,
            version.kind,
            version.live_snapshots,
            version.continuation_steps_from_this_version,
            version.shape_after_the_continuation
        ));
    }
    text
}

/// 恢复落到的根在流里是哪一版。
#[derive(Clone, Copy, Debug)]
pub enum LandedVersionOfTheFlow<'flow> {
    /// 落到 A 之前（mkfs 或暖机那几条根）：根下面没有树、没有快照。
    BeforeTheFirstFile,
    Version(&'flow VersionOfTheFlow),
}

/// 恢复落到的根（实例，txg）在流里是哪一版：不是流里任何一版的根都在 A 之前（流里每一版的根都在表里）。
#[must_use]
pub fn version_landed_on(
    versions: &[VersionOfTheFlow],
    effective_root: (InstanceGeneration, CheckpointTxg),
) -> LandedVersionOfTheFlow<'_> {
    match versions
        .iter()
        .find(|version| version.version == effective_root)
    {
        Some(version) => LandedVersionOfTheFlow::Version(version),
        None => {
            let version_of_the_first_file = versions.first().expect("流里至少 A 一版");
            assert!(
                (effective_root.1, effective_root.0)
                    < (
                        version_of_the_first_file.version.1,
                        version_of_the_first_file.version.0,
                    ),
                "恢复落到的根 {effective_root:?} 不是流里任何一版，又不在 A 之前"
            );
            LandedVersionOfTheFlow::BeforeTheFirstFile
        }
    }
}

/// 在 `reader`（一张崩溃镜像）上读恢复落到的那一版里每一个活快照：读出的要是建它那一刻的内容（三条流都在 A 之后建快照，即 A 的内容）。
/// 交回读了几个快照。
///
/// # Panics
/// 有一个活快照读不出、读成没有文件或读出别的内容。
pub fn assert_every_live_snapshot_reads_its_creation_content(
    reader: &dyn PoolReader,
    version: &VersionOfTheFlow,
) -> usize {
    let content_at_the_creation = file_content();
    for snapshot in &version.live_snapshots {
        let reading = read_the_first_file_of_a_snapshot(reader, &version.root, *snapshot)
            .unwrap_or_else(|failure| {
                panic!(
                    "落到 {:?}（{:?}）：活快照 {snapshot:?} 读不出：{failure:?}",
                    version.version, version.kind
                )
            });
        match reading {
            SnapshotFileReading::File(content) => assert!(
                content == content_at_the_creation,
                "落到 {:?}（{:?}）：活快照 {snapshot:?} 读出建它那一刻的内容（{} 字节），实际 {} 字节",
                version.version,
                version.kind,
                content_at_the_creation.len(),
                content.len()
            ),
            SnapshotFileReading::NoFile | SnapshotFileReading::NoSnapshotWithThisTree => panic!(
                "落到 {:?}（{:?}）：活快照 {snapshot:?} 要读出建它那一刻的文件，实际 {reading:?}",
                version.version, version.kind
            ),
        }
    }
    version.live_snapshots.len()
}

// ============================== 可写挂载一次、续做走到空 ==============================

/// 在一张崩溃镜像落成的盘上可写挂载一次之后：挂载交回的东西、挂载发的每一次写（落在崩溃镜像上，按发的次序）、挂完的盘。
pub struct MountedAfterTheCrash {
    pub mounted: Mounted,
    pub writes_of_the_mount: Vec<RetainedWrite>,
    pub disks_after_the_mount: MemoryPool,
    /// 挂载读过的每一段（连同 journal 记录槽提示问到的段，按发出次序）：交回镜像的观察者按它算读集见证。
    pub read_ranges_of_the_mount: Vec<singlefs_harness::RecordedReadRange>,
}

/// 在 `disks`（一张崩溃镜像落成的池）的副本上可写挂载一次：删除开关同录流（批大小 1，`deletion_history_switches`），准入按式子、影子账开着；
/// 续做把 unlink 与删快照都走到空。挂载发的写录在一条新的录制流上，连同挂完的盘交回。
///
/// # Panics
/// 可写挂载做不成，或续做没走到空：三条流的每个崩溃状态按条款都挂得上、续做都走得完。
#[must_use]
pub fn mount_writable_on_the_crash_disks(disks: &MemoryPool) -> MountedAfterTheCrash {
    let mount_parameters = parameters();
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = disks
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                disks.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &mount_parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_history_switches(),
    )
    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做把 unlink 与删快照都走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    let disks_after_the_mount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: disks.device_size_in_bytes,
    };
    let (writes_of_the_mount, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    MountedAfterTheCrash {
        mounted,
        writes_of_the_mount,
        disks_after_the_mount,
        read_ranges_of_the_mount: stream.read_ranges(),
    }
}

/// 可写挂载之后核续做（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」、判决 S4「挂载续做同一个走空」）：续做的步数等于恢复落到的
/// 那一版还差几步，每一步落在这次挂载的实例上；挂完之后快照这几样与录流时同一串删除的最后一批那一版逐项相同（快照删完、deadlist 与留存的映射条目
/// 随它放空；unlink 删完、快照与 deadlist 原样留着）；落到 A 之前的挂完照旧没有树。交回续做的步数。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_the_mount_continued_to_the_settled_shape(
    landed: LandedVersionOfTheFlow<'_>,
    mounted: &Mounted,
) -> usize {
    let continuation = &mounted.output.deletion_queue_continuation;
    for step in continuation {
        assert_eq!(
            step.root.instance, mounted.output.instance,
            "续做的每一步落在这次挂载的实例上（txg {:?}）",
            step.root.checkpoint_txg
        );
    }
    let shape_after_the_mount = SnapshotShapeOfAVersion::of_the_pool_version(&mounted.current);
    match landed {
        LandedVersionOfTheFlow::BeforeTheFirstFile => {
            assert!(
                continuation.is_empty(),
                "落到 A 之前：没有要续做的，实际续做 {} 步",
                continuation.len()
            );
            assert_eq!(shape_after_the_mount, None, "落到 A 之前：挂完照旧没有树");
        }
        LandedVersionOfTheFlow::Version(version) => {
            assert_eq!(
                continuation.len(),
                version.continuation_steps_from_this_version,
                "落到 {:?}（{:?}）：续做还差几步就发几步",
                version.version,
                version.kind
            );
            assert_eq!(
                shape_after_the_mount.as_ref(),
                Some(&version.shape_after_the_continuation),
                "落到 {:?}（{:?}）：续做走完之后快照、删除队列、deadlist 与留存的映射条目与录流时这一串删除的最后一批那一版相同",
                version.version,
                version.kind
            );
        }
    }
    continuation.len()
}
