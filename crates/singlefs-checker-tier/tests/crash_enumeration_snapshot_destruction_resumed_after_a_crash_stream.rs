//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 删快照崩溃续做流（里程碑四一·4 F3，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4、S6 的 `S W0 X0 C`）：
//! 「取号 → 暖机 × 2 → A（一个数据单元）→ S（建快照）→ W（覆盖写：四个被快照引用的单元进 deadlist，纪元 = S.txg）→ 删快照意图
//! （码 14 条目 flags 位 0 写成销毁中、前驱 P 传给活头）→ 三批（批大小 1，每批放一条纪元落在 [lo, hi) 里、birth > P 的 deadlist 条目，
//! 删它、删留存的映射条目、落点进 defer）→ 最后一批（放最后一条、同一次发布删掉那一对码 14、15 条目）」整条录制流按 D13（验证路线） 已定项 4
//! 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle（每一版都有文件：删快照不动文件内容）、池级 checker、记录核对器；另逐状态核两样（快档与全量都核）：
//! - 恢复落到 S、W 那两版时活快照读得出、读出建它那一刻（A）的内容；落到意图起的那几版没有活快照（销毁中的不算，判决 S9「从写意图那一刻起不再钉住块」）；
//! - 可写挂载一次：续做把删快照走到空，步数等于恢复落到的那一版还差几步（意图 4、三批 3 / 2 / 1）；挂完之后码 14、15 条目、deadlist 与留存的映射条目
//!   都没了、文件照旧（与录流时最后一批那一版逐项相同）；挂完的盘在判段过池级 checker（一条不变量都不许判违例）。
//!
//! 边界（判决 S6「`birth == S.txg` 那一行」）：攻方模型里的 S.txg 是捕获那一版的 txg，实现里 S.txg 是建快照那次独立发布的 txg（判决 S2）、
//! 建快照不重写用户可见单元，被快照引用的单元的 birth 落在 S 捕获的最新那一版（A 的 txg = S.txg − 1）；deadlist 条目的纪元 = S.txg = 删快照放的区间
//! [lo, hi) 的下端 lo，这条流核的是「纪元 == lo 的条目删快照时放」这一边。
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs`（三条快照流共用一份录法）。
//!
//! **比已有的流多罩了什么**：已有的流上没有删快照；这里意图（8 个单元、16 写：只换树表与池级元数据）、三批（9 个单元、18 写：多一片 deadlist 树根）
//! 与最后一批（8 个单元、16 写：deadlist 删空回零根）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；崩在意图与最后一批之间的
//! 每个状态第一次由可写挂载的续做走完。多跑的两步同快照下覆盖写与删文件流：逐状态读快照、每个状态上的可写挂载与挂完的盘上那一遍池级 checker。
//!
//! 快档只展开写数小于 10 的段（八个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（崩溃点重放的枚举域 84869249 个状态，
//! 八个单元段占 84869112；每次写只取两态时的闭式 84869189），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_crash_point_observers;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;

use common_snapshot_crash_point_observers::{
    mount_continuation_observer_under_a_snapshot, observe_the_mount_continuation_under_a_snapshot,
    observe_the_snapshot_of_the_landed_version, snapshot_observer_of_every_state,
};
use common_snapshot_prefix::{
    prepare_the_snapshot_crash_flow, PreparedSnapshotCrashFlow, SnapshotCrashFlow,
    SnapshotFlowStepKind, POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER,
    SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH,
    UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs";

/// 段形：前二十段同快照下覆盖写与删文件流（到 W 为止；取号两轮各一段）；意图 16 个单元写、三批各 18、最后一批 16，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 40] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 26, 2, 1, 2, 16, 2, 1, 2, 18, 2, 1, 2, 18, 2,
    1, 2, 18, 2, 1, 2, 16, 2, 1, 2,
];

/// 恢复落到的那一版是哪一种（计数名），每一种都要有状态。
const LANDED_COUNTERS: [&str; 7] = [
    "landed_before_the_first_file",
    "landed_on_first_file",
    "landed_on_create_snapshot",
    "landed_on_overwrite_under_the_snapshot",
    "landed_on_snapshot_destruction_intent",
    "landed_on_snapshot_destruction_batch",
    "landed_on_snapshot_destruction_final_batch",
];

/// 可写挂载续做的步数至多几步：落到意图那一版，三批加最后一批。
const MOST_CONTINUATION_STEPS: usize = SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH + 1;

fn prepare(tag: &str) -> PreparedSnapshotCrashFlow {
    prepare_the_snapshot_crash_flow(tag, SnapshotCrashFlow::SnapshotDestructionAfterAnOverwrite)
}

fn version_table(prepared: &PreparedSnapshotCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

fn expansion_of_the_segments_shorter_than_ten(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < 10 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 两个观察者都看过每个状态：恢复落到的每一版都有状态；落到 S、W 的每个状态都读了它那一个活快照、落到意图起的没有活快照；
/// 每个状态都可写挂载过一次，续做 0 到 4 步每一种都跑到了，续做 k 步的状态数等于落到还差 k 步那几版的状态数；挂完的盘都过了池级 checker。
fn assert_every_state_was_observed(tally: &CrashPointReplayTally) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        LANDED_COUNTERS
            .iter()
            .map(|name| counts.count_named(name))
            .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版或 A 之前：{counts:?}"
    );
    for name in LANDED_COUNTERS {
        assert!(
            counts.count_named(name) > 0,
            "{name} 的状态真跑到了：{counts:?}"
        );
    }
    assert_eq!(
        counts.count_named("live_snapshots_read_at_their_creation_content"),
        counts.count_named("landed_on_create_snapshot")
            + counts.count_named("landed_on_overwrite_under_the_snapshot"),
        "落到 S、W 的每个状态都读了它那一个活快照；落到意图起的那几版快照销毁中、不读"
    );
    let continuation_counts: Vec<u64> = (0..=MOST_CONTINUATION_STEPS)
        .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
        .collect();
    assert_eq!(
        continuation_counts.iter().sum::<u64>(),
        tally.states,
        "每个状态都可写挂载过一次，续做至多 {MOST_CONTINUATION_STEPS} 步：{continuation_counts:?}"
    );
    for (steps, count) in continuation_counts.iter().enumerate() {
        assert!(
            *count > 0,
            "恢复落到还差 {steps} 步的那一版、挂载续做 {steps} 步的状态真跑到了：{continuation_counts:?}"
        );
    }
    assert_eq!(
        continuation_counts[MOST_CONTINUATION_STEPS],
        counts.count_named("landed_on_snapshot_destruction_intent"),
        "落到意图那一版的续做三批加最后一批"
    );
    assert_eq!(
        continuation_counts[1..MOST_CONTINUATION_STEPS]
            .iter()
            .sum::<u64>(),
        counts.count_named("landed_on_snapshot_destruction_batch"),
        "落到三批之一的续做余下的批与最后一批"
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "每个状态挂完的盘都过了池级 checker（判段在挂完的盘上判，配了 GPU 上卡）"
    );
}

/// 录流的形状（判器判完之后再核）：W 之后 deadlist 四条（纪元 = S.txg = 码 14 的诞生 txg = 删快照放的区间下端 lo）；意图把快照记成销毁中、
/// 活头前驱换成 P = 0、一条都不放；三批各放一条；最后一批放最后一条、删掉那一对条目，deadlist 与留存的映射条目都空了。
fn assert_the_shape_of_the_flow(prepared: &PreparedSnapshotCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前二十段同快照下覆盖写与删文件流；意图 16、三批各 18、最后一批 16 个单元写，各自记录 2、根 1、轮换 2"
    );
    let recorded = &prepared.recorded;
    let snapshot = recorded.snapshot();
    let snapshot_txg = recorded
        .outputs_of(SnapshotFlowStepKind::CreateSnapshot)
        .first()
        .expect("建快照那一步")
        .root
        .checkpoint_txg;
    let overwritten = recorded
        .outputs_of(SnapshotFlowStepKind::OverwriteUnderTheSnapshot)
        .first()
        .copied()
        .expect("覆盖写那一步");
    assert_eq!(
        overwritten.deadlist_entries.len(),
        UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
        "W 把四个被快照引用的单元写进 deadlist"
    );
    for entry in &overwritten.deadlist_entries {
        assert_eq!(
            entry.epoch, snapshot_txg,
            "纪元 = S.txg = 删快照放的区间下端 lo（边界那一行）"
        );
        assert!(
            singlefs_core::records::birth_txg_of_a_mapping_key(&entry.mapping_key)
                .is_some_and(|birth| birth < snapshot_txg),
            "进 deadlist 的单元都生在 S 之前、birth > P = 0：删快照时放"
        );
    }
    let intent = recorded
        .outputs_of(SnapshotFlowStepKind::SnapshotDestructionIntent)
        .first()
        .copied()
        .expect("意图那一步");
    assert_eq!(
        (
            intent.snapshots_being_destroyed.clone(),
            intent.previous_snapshot_txg_of_the_live_head(),
            intent.deadlist_entries.len()
        ),
        (
            vec![snapshot],
            CheckpointTxg(0),
            UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT
        ),
        "意图：快照记成销毁中、它的前驱 0 传给活头、一条都不放"
    );
    let batches = recorded.outputs_of(SnapshotFlowStepKind::SnapshotDestructionBatch);
    assert_eq!(
        batches
            .iter()
            .map(|batch| batch.deadlist_entries.len())
            .collect::<Vec<usize>>(),
        (1..=SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH)
            .rev()
            .collect::<Vec<usize>>(),
        "三批各放一条：剩 3、2、1"
    );
    let final_batch = recorded
        .outputs_of(SnapshotFlowStepKind::SnapshotDestructionFinalBatch)
        .first()
        .copied()
        .expect("最后一批");
    assert!(
        common_snapshot_prefix::snapshot_trees_of(final_batch).is_empty()
            && final_batch.snapshots_being_destroyed.is_empty()
            && final_batch.deadlist_entries.is_empty()
            && final_batch
                .mapping_entries_kept_for_the_snapshots
                .is_empty(),
        "最后一批放掉最后一条、删掉那一对码 14、15 条目，deadlist 与留存的映射条目都空了"
    );
}

/// 快档那一趟：八个单元段不展开，每个状态交两个观察者。
fn enumerate_the_quick_tier(
    prepared: &PreparedSnapshotCrashFlow,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_under_a_snapshot(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let item_name = format!("snapshot_destruction_resumed_after_a_crash::quick_tier::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(prepared),
        &expansion_of_the_segments_shorter_than_ten,
        &[snapshot_observer_of_every_state(prepared, &observe)],
        &[mount_continuation_observer_under_a_snapshot(
            prepared,
            &observe_the_mount,
        )],
    )
    .judged()
}

/// 平时跑的那一份：八个单元段不展开，其余每段任意真子集；每个状态核活快照，再可写挂载一次核续做把删快照走到空。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_between_the_snapshot_destruction_intent_and_its_final_batch_is_resumed_to_empty_by_the_next_writable_mount(
) {
    let prepared = prepare("snapshot-destruction-resumed-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SNAPSHOT_DESTRUCTION_RESUMED_AFTER_A_CRASH_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_the_shape_of_the_flow(&prepared);
    assert_eq!(
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        1 + 12 * 3 + 10 * 3 + 10,
        "每次写只取两态时展开的段按闭式数：1 + 十二个系统配置槽 2 写段（取号两轮、十次轮换）各 3 + 十个记录 2 写段各 3 + 十个根槽 1 写段各 1"
    );
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_eq!(
        tally.states,
        1 + 12 * (3 * 3 - 1) + 10 * 3 + 10,
        "1 + 十二个系统配置槽 2 写段各 3² − 1 + 十个记录 2 写段各 3 + 十个根槽 1 写段各 1"
    );
    assert_every_state_was_observed(&tally);
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_snapshot_destruction_resumed_after_a_crash_quick_tier_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("snapshot-destruction-resumed-crash-point-replay-rerun");
    let Some(first_run) = enumerate_the_quick_tier(&prepared, "rerun") else {
        return;
    };
    let Some(second_run) = enumerate_the_quick_tier(&prepared, "rerun") else {
        return;
    };
    assert_eq!(second_run, first_run, "第二遍全照库里复用，计数逐项相同");
    assert_eq!(
        first_run.observed_states, first_run.states,
        "每个状态都看过"
    );
}

/// 全量：40 段。八个单元段各 2^n − 1（n = 24、16、26、16、18、18、18、16：A、S、W、意图、三批、最后一批），每个状态两遍恢复 + checker + 读快照，
/// 并在工作线程上可写挂载一次核续做。登记成崩溃枚举用例，checker-tier-crash-point-replay 认下面打印的
/// `CRASH_POINT_REPLAY_SNAPSHOT_DESTRUCTION_RESUMED_AFTER_A_CRASH` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
#[test]
#[ignore = "全量 84869249 个状态（八千四百多万）、每个两遍恢复 + checker + 读快照 + 一次可写挂载；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_snapshot_destruction_resumed_after_a_crash_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("snapshot-destruction-resumed-crash-point-replay-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 12 * 3
            + 10 * 3
            + 10
            + ((1 << 24) - 1)
            + ((1 << 16) - 1)
            + ((1 << 26) - 1)
            + ((1 << 16) - 1)
            + 3 * ((1 << 18) - 1)
            + ((1 << 16) - 1),
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，四十段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 84_869_189);
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 12 * (3 * 3 - 1)
            + 10 * 3
            + 10
            + ((1 << 24) - 1)
            + ((1 << 16) - 1)
            + ((1 << 26) - 1)
            + ((1 << 16) - 1)
            + 3 * ((1 << 18) - 1)
            + ((1 << 16) - 1),
        "枚举域的闭式：十二个系统配置槽 2 写段（取号两轮、十次轮换）各 3² − 1、十个记录 2 写段各 3、十个根槽 1 写段、八个单元段各 2^n − 1"
    );
    assert_eq!(closed_form, 84_869_249);
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_under_a_snapshot(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "snapshot_destruction_resumed_after_a_crash::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[snapshot_observer_of_every_state(&prepared, &observe)],
        &[mount_continuation_observer_under_a_snapshot(
            &prepared,
            &observe_the_mount,
        )],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SNAPSHOT_DESTRUCTION_RESUMED_AFTER_A_CRASH states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&tally);
}

/// 录制器那一份的崩溃点（节点按 `<流程>.<周期>.<步>` 编码，`crash_amplification::CrashPointRecorder`）：取号加暖机、新池新建文件两个点照公共模块
/// `common_crash_points` 设（与别的流共享标准前缀，同 `common_crash_points::record_pool_build` 的两段），之后每一步设在这份文件。
/// 流已经录完，按每一步记下的录制流区间补记（记录器只认区间首尾相接、罩住整条流）。
#[cfg(feature = "verdict-store")]
fn crash_points_of_the_flow(
    prepared: &PreparedSnapshotCrashFlow,
) -> Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan> {
    common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    )
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。展开上限默认 6。
/// 两张登记表上的观察者照表跑：标准前缀节点上的可写挂载续做，快照那几步节点上的读快照与可写挂载续做
/// （`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_snapshot_destruction_resumed_after_a_crash_stream_is_clean() {
    let prepared = prepare("crash-amplification-snapshot-destruction-resumed");
    let crash_points = crash_points_of_the_flow(&prepared);
    common_snapshot_crash_point_observers::amplify_a_snapshot_flow(
        "snapshot-destruction-resumed-after-a-crash",
        &prepared,
        crash_points,
        6,
    );
}
