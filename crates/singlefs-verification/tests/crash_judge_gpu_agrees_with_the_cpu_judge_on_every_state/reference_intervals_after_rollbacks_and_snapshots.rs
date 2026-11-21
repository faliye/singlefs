//! 引用区间（I-3.5）：单元出生不晚于上一版、无快照回退之后建快照与快照下覆盖写、攻方种子缩出的建删快照历史。

use super::*;

/// 列出这条流上不一致的状态（前几个），没有就是空串。
pub(super) fn mismatch_listing(comparison: &Comparison) -> String {
    comparison
        .mismatches
        .iter()
        .filter(|line| !line.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

/// I-3.5：一版引用、前一版不引用的单元诞生代号不大于前一版的 txg、也不在整棵继承下来的用户可见树里——CPU 红 I-3.5，GPU 在同样的状态上红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version",
        &record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
            "crash-judge-gpu-i35",
        ),
        "I-3.5",
    );
}

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 2022：只当这段历史的名字，执行不读它。同 harness
/// `pool_checker_appearance_rule_labels_each_version_by_its_own_user_visible_trees.rs` 的同名常量（测试目标之间互相引不到，这里另写一份）。
const SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_2022: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 2022;

/// 攻方种子下标 2022 缩出的 5 步（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 T4、F9）：挂载、一个数据单元以内的覆盖写、
/// 回退到根环里第三新的那条根（覆盖写之前那一版）、建快照、跨 291 个数据单元的顺序写。取值照抄 harness 那一份。
fn rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_2022),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 7_879_751_949_923_199_194,
                },
                fill_seed: 17_840_163_520_990_143_491,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 291,
                    selector: 7_664_500_909_322_504_686,
                },
                fill_seed: 4_676_315_349_287_084_229,
            }),
        ],
    }
}

/// 攻方种子下标 2022 那段历史从 mkfs 之后录起的整条流（[`record_the_flow_of_a_history_from_the_make_filesystem`]）。
fn record_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it(
) -> (RecordedFlow, Vec<PublishedVersionWithoutAFile>) {
    record_the_flow_of_a_history_from_the_make_filesystem(
        &rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it(),
        "rollback-without-snapshots-then-a-snapshot-and-an-overwrite-under-it",
    )
}

/// 一段历史从 mkfs 之后录起的整条流，流名是 `flow_name`（起点镜像是空盘上叠 mkfs 那几步，mkfs 不是事务、不枚举；被判的根是流里最后一条根槽写）。
/// 版本表取理想模型每一步之后根环里的每一版（`IdealModel::committed_versions`，连同历史停下时那一份）：有文件的进 `versions`，
/// 树表 0 条或没有文件的进交回的那几版。
///
/// # Panics
/// 历史没有每一步都做成（或按前提不适用）、模型对不上；流里一条根槽写都没有；实例代号装不进 u32。
fn record_the_flow_of_a_history_from_the_make_filesystem(
    history: &GeneratedHistory,
    flow_name: &'static str,
) -> (RecordedFlow, Vec<PublishedVersionWithoutAFile>) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut contents_of_each_committed_version: BTreeMap<(u64, u64), Option<Vec<u8>>> =
        BTreeMap::new();
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            contents_of_each_committed_version
                .entry((key.instance.0, key.checkpoint_txg.0))
                .or_insert_with(|| file.map(|content| content.to_vec()));
        }
    });
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "流 {flow_name} 那段历史每一步都做成、模型对得上：{:?}",
        run.outcomes
    );
    for (key, file) in &run.committed_versions_at_the_end {
        contents_of_each_committed_version
            .entry((key.instance.0, key.checkpoint_txg.0))
            .or_insert_with(|| file.clone());
    }
    let operations = stream.retained_operations();
    let mkfs_operations = run.operations_written_by_make_filesystem;
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base.apply(&operations[..mkfs_operations]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[mkfs_operations..],
        &execution.device_width.fixed_geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or_else(|| panic!("流 {flow_name}：历史里最后那次发布有一条根槽写"));
    let mut versions = Vec::new();
    let mut versions_without_a_file = Vec::new();
    for ((instance, checkpoint_txg), file) in contents_of_each_committed_version {
        let instance = InstanceGeneration(
            u32::try_from(instance)
                .unwrap_or_else(|_| panic!("流 {flow_name}：这段历史的实例代号装得进 u32")),
        );
        let checkpoint_txg = CheckpointTxg(checkpoint_txg);
        match file {
            Some(content) => versions.push(PublishedVersion {
                instance,
                checkpoint_txg,
                content,
            }),
            None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance,
                checkpoint_txg,
            }),
        }
    }
    (
        RecordedFlow {
            name: flow_name,
            base,
            writes,
            segments,
            judged_root_index,
            versions,
        },
        versions_without_a_file,
    )
}

/// I-3.5 的用户可见树标签按版本记（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9 的 GPU 一侧）：攻方种子下标 2022
/// 那段历史里，回退那一版复活的单元在它自己的活头 extent 树里；最新那一版先走、经快照那两棵树走到它（快照树不是用户可见树），回退那一版共享这片子树、
/// 不重走。CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.5 判成立（不是不适用）。GPU 走读内核按版本记标签（并进来的共享子树落点按这一版此刻
/// 在哪棵树里打）之后每个状态逐位相同；内核照旧跨版本按落点取标签时，回退那一版的 (b) 款落空、GPU 判红 I-3.5 而 CPU 不红，对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_after_a_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_after_a_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it";
    let (flow, versions_without_a_file) =
        record_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it();
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.5"),
        InvariantVerdict::Holds,
        "整条落盘的镜像上 I-3.5 判到了单元、成立：回退那一版复活的单元按这一版自己的标签落在它的活头 extent 树里"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
        &[],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 至少在一个状态上判过 I-3.5 而成立（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 1032：只当这段历史的名字，执行不读它。同 harness
/// `pool_checker_shared_subtree_below_a_snapshot_only_unit_keeps_the_children_it_shares_with_the_head.rs` 的 `SEED_OF_THE_ATTACK_HISTORY`（测试目标之间互相引不到，这里另写一份）。
const SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_1032: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 1032;

/// 攻方种子下标 1032 缩出的 16 步（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9「种子下标 1032 那一形」，攻方 T4）：
/// 去掉回退那一组比重生成 40 步，留第 0、1、4、5、8、9、24、25、26、27、28、29、35、36、37、38 步。没有回退；有建快照、unlink、几次可写挂载、
/// 批间崩溃再挂载的整理、删快照。取值照抄 harness 那一份。
fn snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions(
) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_1032),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 4_908_652_545_014_035_923,
                },
                fill_seed: 1_175_750_450_067_266_269,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 152,
                    selector: 1_112_582_400_669_913_327,
                },
                fill_seed: 4_144_227_704_568_475_653,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::Unlink,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: 9_802_755_752_694_277_730,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 16_953_606_325_372_812_583,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
            }),
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 1_939_626_785_022_617_838,
            }),
            HistoryOperation::CreateInode,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 10_045_164_462_792_776_581,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 849_968_957_743_593_146,
            }),
            HistoryOperation::CreateInode,
        ],
    }
}

/// 快照独有的父单元下面与活头共享的孩子照样挂到它下面（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9
/// 「种子下标 1032 那一形」的 GPU 一侧，池级 checker 那一侧是 `walk.rs` 的 `note_references_of_a_pointer_followed_by_the_walk`）：
/// 攻方种子下标 1032 那段历史里，建快照之后活头改写了一片 extent 叶，改写之前那片叶只从快照那两棵树走得到、它下面的孩子活头也引用；
/// 根环转过一圈之后改写那一版先走、建快照之前那一版后走，后走那一版不重走那片叶、只按「单元 → 它下面的落点」那张表并进它的孩子。
/// CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.5 判成立（不是不适用）。GPU 走读内核把孩子一律挂到父单元下面之后每个状态逐位相同；
/// 内核照旧对这一版已经从活头记过的孩子不挂时（`crates/mutations.tsv` 里「m4 修复批三丁三」那一行），后走那一版少一个落点、
/// GPU 判红 I-3.5 而 CPU 不红，对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_when_a_snapshot_only_unit_shares_its_children_with_the_head(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_when_a_snapshot_only_unit_shares_its_children_with_the_head";
    let (flow, versions_without_a_file) = record_the_flow_of_a_history_from_the_make_filesystem(
        &snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions(),
        "snapshots-created-and-destroyed-around-writable-mounts-and-crashed-compactions",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.5"),
        InvariantVerdict::Holds,
        "整条落盘的镜像上 I-3.5 判到了单元、成立：快照独有的那片叶下面与活头共享的孩子并进建快照之前那一版，那一版的引用集合不少落点"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
        &[],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 至少在一个状态上判过 I-3.5 而成立（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-3.5 判成立的那一支：固定脚本到回退 D 的流上，回退那次发布引用回退目标那一版（txg 3）的单元，诞生代号都不大于前一版的 txg，
/// 它们落在根指针生在 txg 3 的 extent / inode 树里、整棵继承下来（(b)）。CPU 在至少一个状态上判过 I-3.5 而成立、一个状态都不红它，
/// GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule(
) {
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-i35-holds");
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5")
            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 判过 I-3.5 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 恢复内核与 core 同一口径读记录（D23（journal 的角色与格式） 已定项 15，`root_record.rs` 的 `of_unmount_marker_field_of_a_new_root_segment`）：
/// 新根段卸载记号位 0 之外有位为 1 的记录当损坏、不施加。发布那条记录落了而根没落的状态上 CPU 恢复停在暖机那条根，
/// GPU 收了这条记录就落到发布那一版、两遍恢复的结论对不上；池级 checker 那一边 GPU 也照 RR1 不收它。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied(
) {
    let flow = record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
        "crash-judge-gpu-rr1",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 恢复施加一条记录取它新根段里的实例表指针（已定项 15，`recovery.rs` 的 `replay_journal`），不照抄所选根的：记录里的实例表指针读不对时
/// CPU 恢复走到文件之前就读不出，GPU 照抄所选根就读得出；池级 checker 走施加出来那一版也从记录取（I-4.1），根落了的状态上红 I-8.10 第五段。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root",
        &record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
            "crash-judge-gpu-applied-instance-table",
        ),
        "I-8.10",
    );
}

/// 恢复内核判记录新根段四条指针的头与 core 同一口径：实例表指针头的 extent 偏移非 0 的记录当损坏、不施加；
/// CPU 池级 checker 不判记录里的指针头、照收，根落了的状态上红 I-8.10 第五段。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied",
        &record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
            "crash-judge-gpu-record-pointer-head",
        ),
        "I-8.10",
    );
}

/// 走读内核走由记录施加出来的那一版时，分配记录树根从记录新根段取（walk.rs 的 `walk_version_applied_only_by_records`），
/// 不取它施加在其上的那条根的：记录里的分配记录树根是外来的、读不对时 CPU 判 I-4.1 红，GPU 取那条根（全零、不走）就不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on",
        &record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
            "crash-judge-gpu-applied-allocation-root",
        ),
        "I-4.1",
    );
}

/// I-8.10 第七段：发布那条记录的卸载记号与同代的根不同（记录 1、根 0）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root",
        &record_new_pool_with_a_last_record_carrying_the_unmount_marker(
            "crash-judge-gpu-i810-unmount",
        ),
        "I-8.10",
    );
}

/// 两个池级 checker 内核把违例位的每个字都写出来：结论缓冲派活前整个填全 1，没被写的字读出来是全 1。
/// 新池新建文件那条流每段展开到 8 次写的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
#[test]
fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count() {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
    ) else {
        return;
    };
    // 判的是内核写没写全违例位的每个字，不是崩溃域：每段固定展开到 8 次写（53 个状态），不跟展开上限变量走全量域（这条流全量域 1677 万个状态）
    const EXPAND_UP_TO: usize = 8;
    let flow = record_new_pool_file_creation("crash-judge-gpu-violated-words-written", "new-pool");
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        &tables,
    ) else {
        return;
    };
    let Some(lease) = cards.lease_card_or_report_not_run(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        mebibytes_a_judge_needs(&packed, usize::MAX),
    ) else {
        return;
    };
    let Some(verdicts) = on_the_card_or_report_not_run_when_out_of_memory(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        lease.card(),
        "建判器与判这条流",
        || {
            GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, usize::MAX)
                .judge_states(0..state_count)
        },
    ) else {
        return;
    };
    drop(lease);
    let invariant_count = IMPLEMENTED_INVARIANTS.len();
    let states_with_bits_past_the_invariant_count: Vec<String> = verdicts
        .iter()
        .enumerate()
        .filter(|(_, words)| {
            words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
                || pool_checker_violated_bits(words) >> invariant_count != 0
        })
        .map(|(ordinal, words)| format!("状态 {ordinal}：{words:?}"))
        .collect();
    assert!(
        states_with_bits_past_the_invariant_count.is_empty(),
        "{} 个状态的违例位越过池级 checker 的 {invariant_count} 条或判不了（前几个）：\n{}",
        states_with_bits_past_the_invariant_count.len(),
        states_with_bits_past_the_invariant_count
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
