//! 整理搬只被快照引用的单元、deadlist 跨两代实例记与清、回收的那一条流：录流、回收与再分配、备好交给崩溃点重放。

use super::*;

// ============================== 整理搬只被快照引用的单元、deadlist 跨两代实例记与清、回收的流 ==============================
//
// 生命周期闭环流那一串，多两样：建快照之前那次覆盖写连带建空 inode，把 inode 树填满几个叶容器（这几个容器、inode 根与 extent 根都落在 R 里）；
// 快照下除了覆盖写第一个文件，再逐个容器改一条 inode 记录（给那个 inode 写一个数据单元的内容），把快照抄下的每个容器都换下来。
// 换下来的这些单元只被快照引用，记进 deadlist（实例 1），整理把它们搬出 R（实例 2，几批加最后一批），删快照由实例 2 清空 deadlist。
// 流头（mkfs 到建快照）与整理、deadlist 无关，崩溃点重放的基镜像取在它之后、不枚举；崩溃放量那一路的基镜像照旧取在 mkfs 之后
// （标准前缀上的观察者要有状态可看）。

/// 快照抄下的 inode 树有几个叶容器（一个容器装 [`INODE_LEAF_RECORDS`] 条记录）。
pub const INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT: u64 = 6;

/// 建快照之前那次覆盖写连带建几个空 inode：连同第一个文件恰好填满 [`INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT`] 个容器。
pub const INODES_CREATED_WITH_THE_OVERWRITE_BEFORE_THE_SNAPSHOT: u64 =
    INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT * INODE_LEAF_RECORDS - 1;

/// 快照下改别的 inode 那几次的调用方时间比第一个文件晚这么多秒（再逐次加一）：与这条流里别的写都不同。
pub const SECONDS_FROM_THE_FIRST_WRITE_TO_THE_WRITES_OF_THE_OTHER_INODES: u64 = 300;

/// 快照下改的那个 inode：第 `container_index` 个叶容器里的第一个号（inode 号从第一个文件起连号，一个容器装满再开下一个）。
#[must_use]
pub fn inode_in_the_leaf_container(container_index: u64) -> InodeNumber {
    InodeNumber(FIRST_INODE_NUMBER + container_index * INODE_LEAF_RECORDS)
}

/// 快照下写给第 `container_index` 个叶容器里那个 inode 的内容：一个数据单元装得下、各不相同。
#[must_use]
pub fn content_written_under_the_snapshot_to_the_inode_in_the_leaf_container(
    container_index: u64,
) -> Vec<u8> {
    let seed = usize::try_from(container_index).expect("容器序号装得进 usize");
    (0..1500_usize)
        .map(|index| u8::try_from((index * 5 + seed * 17 + 1) % 233).expect("小于 256"))
        .collect()
}

/// 录好的这条流与流头有几步（建快照之前那次覆盖写、建快照；崩溃点重放的基镜像取在它们之后）。
pub struct RecordedCompactionOfSnapshotUnitsFlow {
    pub recorded: RecordedCompactionFlow,
    pub steps_of_the_stream_head: usize,
}

/// 录这条流（`tag` 是镜像文件名里那一段）：
/// A → 覆盖写连带建空 inode → 建快照（以上是流头）→ 快照下覆盖写第一个文件 → 快照下逐个容器给一个 inode 写一个单元 → unlink 意图、最后一批
/// → 重挂 → 整理签发、几批、最后一批 → 会话里删快照 → 回收（抬 F、建 inode，直到 R 整段回收）→ 在搬出来的空段上再分配。
/// `flow` 记成生命周期闭环流（同一串，只多了建 inode 与快照下逐个容器那几次写）。
///
/// # Panics
/// 任何一步发布被拒、整理停不下来、回收或再分配在上限之内做不成。
#[must_use]
pub fn record_the_compaction_of_snapshot_units_flow(
    tag: &str,
) -> RecordedCompactionOfSnapshotUnitsFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    let write_time_of_the_overwrite =
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE;
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
        |pool| {
            let previous = pool.output.clone();
            let content = content_overwritten_before_the_snapshot();
            let publish_parameters = parameters();
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_data_changes_published_together(
                &mut writer,
                &mut pool.allocator,
                &previous,
                DataChangesPublishedTogether {
                    whole_write: Some(WholeWriteOfOneInode {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        content: &content,
                        write_time_seconds: write_time_of_the_overwrite,
                    }),
                    new_inodes: &[NewInodesOfOneChange {
                        count: INODES_CREATED_WITH_THE_OVERWRITE_BEFORE_THE_SNAPSHOT,
                        write_time_seconds: write_time_of_the_overwrite,
                    }],
                },
                INSTANCE_OF_THE_FLOWS,
            )
            .expect("建快照之前覆盖写第一个文件、连带建空 inode")
        },
    ));
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
        create_snapshot,
    ));
    let snapshot = live_snapshot_trees_of(&steps.last().expect("刚发布快照").versions[0])
        .last()
        .copied();
    let steps_of_the_stream_head = steps.len();
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
        overwrite_under_the_snapshot,
    ));
    for container_index in 1..INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT {
        steps.push(record_one_publish_of_instance_one(
            &mut pool,
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
            |pool| {
                let previous = pool.output.clone();
                let content = content_written_under_the_snapshot_to_the_inode_in_the_leaf_container(
                    container_index,
                );
                let publish_parameters = parameters();
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
                publish_overwrite_of_an_inode(
                    &mut writer,
                    &mut pool.allocator,
                    &previous,
                    inode_in_the_leaf_container(container_index),
                    &content,
                    FIXED_WRITE_TIME_SECONDS
                        + SECONDS_FROM_THE_FIRST_WRITE_TO_THE_WRITES_OF_THE_OTHER_INODES
                        + container_index,
                    INSTANCE_OF_THE_FLOWS,
                )
                .unwrap_or_else(|refusal| {
                    panic!("快照下给第 {container_index} 个容器里的 inode 写一个单元：{refusal:?}")
                })
            },
        ));
    }
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent),
        unlink_intent,
    ));
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkFinalBatch),
        unlink_step,
    ));
    assert!(
        pool.output.deleted_inodes.is_empty(),
        "一个数据单元的文件：意图之后一批就删完"
    );
    let (mount_step, mut session) = remount_and_open_the_session(&mut pool);
    steps.push(mount_step);
    steps.extend(publish_the_compaction_until_the_intent_is_deleted(
        &mut pool,
        &mut session,
    ));
    steps.push(publish_a_user_change_through_the_session(
        &mut pool,
        &mut session,
        CompactionFlowStepKind::DestroySnapshotThroughTheSession,
        UserChange::DestroySnapshot {
            snapshot: snapshot.expect("流头建过快照"),
        },
    ));
    record_the_reclaim_and_the_reallocation_of_the_snapshot_units_flow(
        &mut pool,
        &mut session,
        &mut steps,
    );
    pool.output = version_of_trees(&session.current).clone();
    RecordedCompactionOfSnapshotUnitsFlow {
        recorded: RecordedCompactionFlow {
            flow: CompactionCrashFlow::LifecycleClosure,
            pool,
            first_file,
            steps,
            snapshot,
        },
        steps_of_the_stream_head,
    }
}

/// 这条流「回收」那一段至多几轮（抬 F 到准入上限、建一次 inode）。
pub const SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST: usize = 24;

/// 这条流「再分配」那一段至多建几次 inode。
pub const SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST: usize = 24;

/// 这条流的「回收」与「在搬出来的空段上再分配」：同 [`record_the_reclaim_and_the_reallocation`]，上限按这条流取
/// （[`SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST`]、[`SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST`]），做不成时报出 R 上的记录。
fn record_the_reclaim_and_the_reallocation_of_the_snapshot_units_flow(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    steps: &mut Vec<CompactionFlowStep>,
) {
    let mut new_inodes_published = 0_u64;
    let mut new_inodes_step =
        |pool_of_the_flow: &mut BuiltPool, session_of_the_flow: &mut MountedSession| {
            new_inodes_published += 1;
            publish_a_user_change_through_the_session(
                pool_of_the_flow,
                session_of_the_flow,
                CompactionFlowStepKind::NewInodesThroughTheSession,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS
                        + LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES
                        + new_inodes_published,
                },
            )
        };
    let mut is_the_region_discarded = false;
    let mut discards_of_each_round = Vec::new();
    for _round_index in 0..SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST {
        if let Some(raised) = raise_the_floor_to_the_admission_ceiling(pool, session) {
            steps.push(raised);
        }
        let step = new_inodes_step(pool, session);
        is_the_region_discarded = the_step_discards_the_region_on_both_devices(&step);
        discards_of_each_round.push(step.discards.clone());
        steps.push(step);
        if is_the_region_discarded {
            break;
        }
    }
    let current = version_of_trees(&session.current).clone();
    assert!(
        is_the_region_discarded,
        "{SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST} 轮抬 F 之内 R 整段回收、会话对它发了 discard：F {:?}，R 上的记录 {:?}，每轮的 discard {:?}",
        current.root.rollback_floor,
        allocation_records_over_the_region(&current),
        discards_of_each_round
    );
    for _publish_index in 0..SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST {
        let step = new_inodes_step(pool, session);
        let is_reallocated = step
            .versions
            .last()
            .is_some_and(region_holds_an_unreleased_record);
        steps.push(step);
        if is_reallocated {
            return;
        }
    }
    panic!("建 {SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST} 次 inode 之内分配到 R 上");
}

/// 这条流的基镜像取在哪。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseImageOfTheCompactionOfSnapshotUnitsFlow {
    /// mkfs 之后：崩溃放量那一路（标准前缀上的观察者要有状态可看）。
    AfterTheMakeFilesystem,
    /// 流头（mkfs 到建快照）之后：崩溃点重放那一路，流头不枚举。
    AfterTheStreamHead,
}

/// 录这条流并摆好（同 [`prepare_the_compaction_crash_flow`]，基镜像按 `base_image` 取）。写表、段与被判的根槽写从基镜像之后数；
/// `stream_indexes` 照旧是 mkfs 之后录制流里的下标，`operations_after_mkfs` 照旧是 mkfs 之后整条录制流。
/// 版本表与流里每一版收全（流头那几版也在：基镜像里的版本恢复落得到的只有最后一版，收全不改判法）。
#[must_use]
pub fn prepare_the_compaction_of_snapshot_units_flow(
    tag: &str,
    base_image: BaseImageOfTheCompactionOfSnapshotUnitsFlow,
) -> PreparedCompactionCrashFlow {
    let RecordedCompactionOfSnapshotUnitsFlow {
        recorded,
        steps_of_the_stream_head,
    } = record_the_compaction_of_snapshot_units_flow(tag);
    let pool = &recorded.pool;
    let operations = pool.retained_operations();
    let mkfs_operations = pool.mkfs_operation_count;
    let operations_in_the_base_image = match base_image {
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheMakeFilesystem => mkfs_operations,
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheStreamHead => {
            recorded.steps[steps_of_the_stream_head - 1].operations.end
        }
    };
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[mkfs_operations..operations_in_the_base_image]);
    let operations_after_mkfs = operations[mkfs_operations..].to_vec();
    let (writes, segments, stream_indexes_after_the_base_image) =
        writes_and_segments_with_stream_indexes(
            &operations[operations_in_the_base_image..],
            &geometry(),
        );
    let stream_indexes = stream_indexes_after_the_base_image
        .iter()
        .map(|index| index + (operations_in_the_base_image - mkfs_operations))
        .collect();
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
                // 回收之后又分配出去的源落点这一版有人用，不抹（同 `prepare_the_compaction_crash_flow`）
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
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot => {
                first_file_content = Some(content_overwritten_before_the_snapshot());
            }
            CompactionFlowStepKind::OverwriteWhileTheIntentIsLive => {
                first_file_content = Some(content_overwritten_while_the_intent_is_live());
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot) => {
                content_captured_by_the_snapshots = first_file_content.clone();
            }
            // 快照下给别的 inode 写的那几次也记成这一种：第一个文件照旧是快照下覆盖写那一份内容
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot) => {
                first_file_content = Some(overwrite_content());
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch,
            ) => {
                first_file_content = None;
            }
            CompactionFlowStepKind::SequentialWrite => {
                panic!("这条流没有顺序写：{:?}", step.kind)
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
    /// 每一段归流里哪一步（段第一个写在录制流里的下标落在那一步发出的那段录制流里）：`None` 是 A 与 A 之前那几段。
    ///
    /// # Panics
    /// 有一段一个写都没有。
    #[must_use]
    pub fn step_position_of_each_segment(&self) -> Vec<Option<usize>> {
        let mkfs_operations = self.recorded.pool.mkfs_operation_count;
        self.segments
            .iter()
            .map(|segment| {
                let write_opening_the_segment = *segment.first().expect("段至少一个写");
                let operation_in_the_stream =
                    mkfs_operations + self.stream_indexes[write_opening_the_segment];
                self.recorded
                    .steps
                    .iter()
                    .position(|step| step.operations.contains(&operation_in_the_stream))
            })
            .collect()
    }

    /// 崩溃状态落得到的流里那几版：根槽写在写表里的那几版，连同排在它们头一版之前的那一版（基镜像里最后那一版；基镜像取在 mkfs 之后时
    /// 写表里头一版就是 A，前面没有）。再早的几版在基镜像里，崩溃状态择不到。
    ///
    /// # Panics
    /// 写表里一条流里那几版的根槽写都没有。
    #[must_use]
    pub fn versions_landed_on_from_the_base_image(&self) -> Vec<&VersionOfTheCompactionFlow> {
        let roots_written_after_the_base_image: Vec<(InstanceGeneration, CheckpointTxg)> = self
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .map(singlefs_verification::memory_pool::root_identity_of_write)
            .collect();
        let earliest_version_written_after_the_base_image = self
            .versions_of_the_flow
            .iter()
            .position(|version| roots_written_after_the_base_image.contains(&version.version))
            .expect("写表里至少有流里一版的根槽写");
        self.versions_of_the_flow[earliest_version_written_after_the_base_image.saturating_sub(1)..]
            .iter()
            .collect()
    }
}
