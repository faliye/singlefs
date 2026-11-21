//! 录好的流交给崩溃点重放之前备好的样子：流里每一版、版本文字、崩溃镜像落在哪一版上。

use super::*;

/// 生命周期闭环流建 inode 那几次的调用方时间比第一个文件晚这么多秒（再逐次加一）：与之前每一次写的时间都不同。
pub const LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES: u64 = 600;

// ============================== 崩溃点重放要的那几样 ==============================

/// 流里的一版（A 与之后每一步发成的每一版）：根（实例，txg）、哪一种（A 是 `None`）、根记录、这一版里第一个文件的内容（没有文件是 `None`）、
/// 这一版的活快照与它们建的那一刻第一个文件的内容、这一版有没有活的整理意图、这一版与它之前整理搬走的单元的源落点（读的时候先抹掉它们，
/// 核读路径照映射走到目的地）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheCompactionFlow {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub kind: Option<CompactionFlowStepKind>,
    pub root: RootRecord,
    pub first_file_content: Option<Vec<u8>>,
    pub live_snapshots: Vec<TreeIdentifier>,
    pub content_captured_by_the_snapshots: Option<Vec<u8>>,
    pub has_a_live_compaction_intent: bool,
    pub relocation_sources_up_to_this_version: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 一条录好的流按崩溃点重放要的样子摆好：起点镜像（mkfs 之后）、写表与段、每次写在录制流里的下标、被判的根槽写（最后一条）、
/// 版本表（有文件的与文件从那一版起没有了的）、观察者另读的流里每一版。
pub struct PreparedCompactionCrashFlow {
    pub recorded: RecordedCompactionFlow,
    pub base: MemoryPool,
    pub operations_after_mkfs: Vec<RetainedOperation>,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    pub versions_of_the_flow: Vec<VersionOfTheCompactionFlow>,
}

/// 录一条流并摆好。版本表逐版现推：每一步的每一版都进表（有文件的带它那一刻第一个文件的内容），重挂、整理、删快照、抬 F、建 inode 不动文件内容。
/// 这里只摆、不核形状：段形、搬走的单元、discard 这些在用例里判器判完之后再核。
#[must_use]
pub fn prepare_the_compaction_crash_flow(
    tag: &str,
    flow: CompactionCrashFlow,
    contents_of_the_sequential_writes: &[Vec<u8>],
) -> PreparedCompactionCrashFlow {
    let recorded = record_the_compaction_crash_flow(tag, flow, contents_of_the_sequential_writes);
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let operations_after_mkfs = operations[pool.mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_after_mkfs, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let mut versions_with_a_file = Vec::new();
    let mut versions_without_a_file = Vec::new();
    let mut versions_of_the_flow = Vec::new();
    let mut first_file_content = Some(file_content());
    let mut content_captured_by_the_snapshots = None;
    let mut relocation_sources = Vec::new();
    let mut sequential_writes_seen = 0usize;
    let mut note_version =
        |kind: Option<CompactionFlowStepKind>,
         output: &TransactionOutput,
         content_of_this_version: &Option<Vec<u8>>,
         captured_content_of_this_version: &Option<Vec<u8>>,
         sources_of_this_version: &[(DeviceIdentity, SlotNumber)]| {
            match content_of_this_version {
                Some(content) => versions_with_a_file.push(PublishedVersion {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                    content: content.clone(),
                }),
                None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                }),
            }
            let live_snapshots = live_snapshot_trees_of(output);
            versions_of_the_flow.push(VersionOfTheCompactionFlow {
                version: (output.root.instance, output.root.checkpoint_txg),
                kind,
                root: output.root,
                first_file_content: content_of_this_version.clone(),
                content_captured_by_the_snapshots: if live_snapshots.is_empty() {
                    None
                } else {
                    captured_content_of_this_version.clone()
                },
                live_snapshots,
                has_a_live_compaction_intent: !output.logged_operations.is_empty(),
                // 回收之后又分配出去的源落点（生命周期闭环流在搬出来的空段上再分配）这一版有人用，不抹
                relocation_sources_up_to_this_version: sources_of_this_version
                    .iter()
                    .filter(|(device, slot)| {
                        !output.allocation_records.iter().any(|record| {
                            record.device == *device
                                && !record.is_released
                                && record.slot.0 <= slot.0
                                && slot.0 < record.slot.0 + u64::from(record.span_slots)
                        })
                    })
                    .copied()
                    .collect(),
            });
        };
    note_version(
        None,
        &recorded.first_file,
        &first_file_content,
        &content_captured_by_the_snapshots,
        &relocation_sources,
    );
    for step in &recorded.steps {
        match step.kind {
            CompactionFlowStepKind::SequentialWrite => {
                first_file_content = Some(
                    contents_of_the_sequential_writes
                        .get(sequential_writes_seen)
                        .expect("每一次顺序写的内容都由调用方给了")
                        .clone(),
                );
                sequential_writes_seen += 1;
            }
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot => {
                first_file_content = Some(content_overwritten_before_the_snapshot());
            }
            CompactionFlowStepKind::OverwriteWhileTheIntentIsLive => {
                first_file_content = Some(content_overwritten_while_the_intent_is_live());
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot) => {
                content_captured_by_the_snapshots = first_file_content.clone();
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot) => {
                first_file_content = Some(overwrite_content());
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch,
            ) => {
                first_file_content = None;
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::SnapshotDestructionIntent
                | SnapshotFlowStepKind::SnapshotDestructionBatch
                | SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
            )
            | CompactionFlowStepKind::MountWritable
            | CompactionFlowStepKind::CompactionIssue
            | CompactionFlowStepKind::CompactionBatch
            | CompactionFlowStepKind::CompactionFinalBatch
            | CompactionFlowStepKind::DestroySnapshotThroughTheSession
            | CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling
            | CompactionFlowStepKind::NewInodesThroughTheSession
            | CompactionFlowStepKind::CompactionIntentDroppedOnAStop => {}
        }
        for output in &step.versions {
            relocation_sources.extend(output.units_relocated_by_this_publish.iter().flat_map(
                |relocated| {
                    relocated
                        .source
                        .iter()
                        .map(|location| (location.device, location.slot))
                },
            ));
            note_version(
                Some(step.kind),
                output,
                &first_file_content,
                &content_captured_by_the_snapshots,
                &relocation_sources,
            );
        }
    }
    PreparedCompactionCrashFlow {
        recorded,
        base,
        operations_after_mkfs,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_flow,
    }
}

impl PreparedCompactionCrashFlow {
    /// 观察者另读的输入的摘要（流里每一版，[`text_of_the_versions_of_the_compaction_flow`]）：进观察者的版本，录制流一变它就变。
    #[must_use]
    pub fn digest_of_the_versions_of_the_flow(
        &self,
    ) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
        singlefs_verification::sha256::sha256_digest(
            text_of_the_versions_of_the_compaction_flow(&self.versions_of_the_flow).as_bytes(),
        )
    }

    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }

    /// 录制流里的 discard（mkfs 之后的下标、盘、段起点）。
    #[must_use]
    pub fn discards_in_the_stream(&self) -> Vec<(usize, DeviceIdentity, SlotNumber)> {
        self.operations_after_mkfs
            .iter()
            .enumerate()
            .filter(|(_index, retained)| retained.operation.kind == RecordedOperationKind::Discard)
            .map(|(index, retained)| {
                (
                    index,
                    retained.operation.device,
                    SlotNumber(retained.operation.offset.0 / SLOT_BYTES),
                )
            })
            .collect()
    }
}

/// 观察者另读的输入写成文字（取摘要进观察者的版本）：只有逐版那几行，不带标签与手写的版本号（D13（验证路线） 已定项 17：
/// 手写的版本常量不进键）；录法变了，变的是逐版的内容，摘要跟着变。
#[must_use]
pub fn text_of_the_versions_of_the_compaction_flow(
    versions: &[VersionOfTheCompactionFlow],
) -> String {
    let mut text = String::new();
    for version in versions {
        text.push_str(&format!(
            "version instance={} txg={} kind={:?} first_file_bytes={:?} live_snapshots={:?} captured_bytes={:?} live_intent={} relocation_sources={:?}\n",
            version.version.0 .0,
            version.version.1 .0,
            version.kind,
            version.first_file_content.as_ref().map(|content| singlefs_verification::fnv1a_64(content)),
            version.live_snapshots,
            version
                .content_captured_by_the_snapshots
                .as_ref()
                .map(|content| singlefs_verification::fnv1a_64(content)),
            version.has_a_live_compaction_intent,
            version.relocation_sources_up_to_this_version
        ));
    }
    text
}

/// 恢复落到的根在流里是哪一版。
#[derive(Clone, Copy, Debug)]
pub enum LandedVersionOfTheCompactionFlow<'flow> {
    /// 落到 A 之前（mkfs 或暖机那几条根）：根下面没有树。
    BeforeTheFirstFile,
    Version(&'flow VersionOfTheCompactionFlow),
}

/// 恢复落到的根（实例，txg）在流里是哪一版：不是流里任何一版的根都在 A 之前（流里每一版的根都在表里）。
#[must_use]
pub fn version_of_the_compaction_flow_landed_on(
    versions: &[VersionOfTheCompactionFlow],
    effective_root: (InstanceGeneration, CheckpointTxg),
) -> LandedVersionOfTheCompactionFlow<'_> {
    match versions
        .iter()
        .find(|version| version.version == effective_root)
    {
        Some(version) => LandedVersionOfTheCompactionFlow::Version(version),
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
            LandedVersionOfTheCompactionFlow::BeforeTheFirstFile
        }
    }
}
