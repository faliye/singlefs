//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 整理暂停删意图的崩溃流（I-5.6「活意图的 R 内没有分配代 > `c0` 的记录」；暂停时意图连同 R 排他一起撤，下一次整理从头签发）：
//! 「取号 → 暖机 × 2 → A → B、C 两次顺序写 → 重挂（取号、写行、暖机）→ 整理签发 → 一批 →（压准入余量）最后一批被准入拒、暂停，单发一次只删意图的发布
//! →（解开余量）在同一个 R 上重新签发 → 最后一批（同一次发布删意图）」
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 与批大小同整理搬迁流（第一个文件那次发布的提交内生块所在的段，一批两个）。
//! 压余量只动会话分配器的隔离位、不写盘：录制流里没有它，崩溃镜像上的可写挂载不带它。
//! 每个状态跑恢复 + 多版本 oracle、池级 checker（含 I-5.6）、记录核对器；观察者同整理搬迁流：抹掉这一版之前整理搬走的单元的源落点再读、
//! 可写挂载一次看整理续做推到意图删了（落到删意图那一版之后、重新签发之前的，挂载一次整理都不发）。
//!
//! **比已有的流多罩了什么**：已有的整理流里删意图都跟着 R 按记录全空（最后一批同发布删、或 R 已空只删），丢意图只在 R 里有隔离槽或无可搬时发，
//! 这几条流都没走到。这条流第一次枚举「R 里还有未释放的记录时单发一次只删意图的发布」那几个崩溃状态：崩在它之前落到意图活着、R 排他着的那一版
//! （可写挂载续做把这条意图做完），崩在它途中与之后落到意图已不在的那一版（挂载不续做、不签）；另枚举同一个 R 上第二条意图的签发与最后一批
//! （`c0` 比第一条大，R 里留着第一批没搬的单元）。多跑的步：删意图那一版、重新签发那一版各走一遍两个观察者。前缀到第一批与整理搬迁流逐字节相同。
//!
//! 一批出错停下时删意图（D26（后台整理与放置回收） 已定项 13 第 4 条，那一批放弃、单发一次只删意图的发布、整理停下）那条流
//! （`CompactionCrashFlow::CompactionStoppedAfterAComputeErrorDroppingTheIntent`）与这条流的录制流逐步相同：被拒的那一批与注入出错的那一批都一个写都不发，
//! 删意图那一次、重新签发与最后一批写的都一样。所以出错停下那一次删意图的崩溃状态就是这条流枚举的那几个，不另开一条枚举，
//! 只加一条小用例钉住「相同」（`the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes`）。
//!
//! 只剩 R 有空槽时暂停删意图那一条流（`CompactionCrashFlow::CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion`）：
//! 前缀到第一批同这条流，之后在会话的分配器上把 R 之外的每一个空槽隔离（不写盘、不进录制流），最后一批在 R 之外取不到落点、暂停，
//! 删意图那一次取落点时放开 R、新落点落在 R 里，流到这里为止。**它比这条流多罩的**：删意图那一次自己的写落在活意图的 R 里的那几个崩溃状态
//! ——崩在它途中落到意图活着、R 排他着的那一版（R 里那几个槽在那一版里是空槽，盘上却已有删意图那一次写了一半的字节），可写挂载续做照常把这条意图做完；
//! 崩在它之后落到删意图那一版（R 里有分配代是这次发布的记录、意图已不在）。每个状态照样跑池级 checker（含 I-5.6）。
//! 多跑的步：删意图那一版走一遍两个观察者。前缀到第一批与这条流逐字节相同，之后录制流不同（删意图那一次的落点不同），另开一条枚举。
//!
//! 每段都展开的两条登记成崩溃枚举用例，带断点续跑、可双机分片。
//! 两条流的同一组崩溃状态各另有一条在 CPU 判器上判的崩溃枚举用例（只开 CPU 的多机配置，先录进库、再从库读回判；两个观察者同上，挂完的盘同样在判段过池级 checker）：
//! 多机配置开着 GPU 时那两条全量的判段交 GPU 判器，这两条的判段一律交 CPU 判器；每条流的两条经同一个枚举函数录同一条流、挂同两个观察者，块键相同，
//! 库里有全量那一条判过的块就照块复用。改坏池级 checker 判反的变异行点名两条全量（改 GPU 判器那一份），这两条的变异行在录入段之前就红。
//! 它们不多罩崩溃状态，多跑的是 CPU 判器那一遍判。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_counter_of, landed_version_observer,
    mount_continuation_observer, observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction,
};
use common_compaction_prefix::{
    prepare_the_compaction_crash_flow, record_the_compaction_crash_flow,
    region_holds_a_placement_of_the_publish_of_the_version, region_of_the_crash_flows,
    CompactionCrashFlow, CompactionFlowStepKind, PreparedCompactionCrashFlow,
};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::logged_operations::live_compaction_intent_of_the_entries;
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
use singlefs_verification::crash_library::{
    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of, ExclusiveTemporaryDirectory,
    TheOneLibraryKeptForThisProcess,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::CrashImage;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs";

/// B、C 两次顺序写的内容（多记录流那两份）。
fn contents_of_the_sequential_writes() -> [Vec<u8>; 2] {
    [
        common_crash_points::content_needing(2, 1),
        common_crash_points::content_needing(3, 2),
    ]
}

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionPausedShortOfSpaceDroppingTheIntent,
        &contents_of_the_sequential_writes(),
    )
}

fn prepare_with_only_the_region_free(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion,
        &contents_of_the_sequential_writes(),
    )
}

fn version_table(prepared: &PreparedCompactionCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

/// 流里每一种那一版的计数名，每一种都要有状态。
fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 整理暂停删意图那条流录流的形状（下面那条形状用例与 CPU 判器那一条在枚举之前都核）。
fn assert_the_shape_of_the_paused_flow(prepared: &PreparedCompactionCrashFlow) {
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    let position_of_the_mount = kinds
        .iter()
        .position(|kind| *kind == CompactionFlowStepKind::MountWritable)
        .expect("流里有那一次重挂");
    assert_eq!(
        kinds[position_of_the_mount + 1..],
        [
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionIntentDroppedOnAStop,
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionFinalBatch
        ],
        "重挂之后：签发、一批、暂停删意图、重新签发、最后一批：{kinds:?}"
    );
    let region = region_of_the_crash_flows();
    let only_version_of = |step_index: usize| {
        let [version] = recorded.steps[step_index].versions.as_slice() else {
            panic!("{:?}：一步一次发布", recorded.steps[step_index].kind);
        };
        version
    };
    let first_issue = only_version_of(position_of_the_mount + 1);
    let dropped = only_version_of(position_of_the_mount + 3);
    let second_issue = only_version_of(position_of_the_mount + 4);
    let final_batch = only_version_of(position_of_the_mount + 5);
    assert!(
        dropped.logged_operations.is_empty()
            && dropped.units_relocated_by_this_publish.is_empty()
            && !region_is_empty_by_the_records(&dropped.allocation_records, region),
        "暂停那一次只删意图、不搬单元，R 里还有未释放的记录"
    );
    let (_, first_intent) = live_compaction_intent_of_the_entries(&first_issue.logged_operations)
        .expect("签发那一版有活意图");
    let (_, second_intent) = live_compaction_intent_of_the_entries(&second_issue.logged_operations)
        .expect("重新签发那一版有活意图");
    assert_eq!(first_intent.region, region, "第一条意图在 R 上");
    assert_eq!(second_intent.region, region, "重新签发的意图在同一个 R 上");
    assert!(
        second_intent.allocation_generation_at_issue.0
            > first_intent.allocation_generation_at_issue.0,
        "重新签发是从头来：c0 取签发那一刻，比第一条大"
    );
    assert!(
        final_batch.logged_operations.is_empty()
            && !final_batch.units_relocated_by_this_publish.is_empty()
            && region_is_empty_by_the_records(&final_batch.allocation_records, region),
        "最后一批搬了单元、同一次发布删意图、R 按记录全空"
    );
}

/// 录流的形状：重挂之后是签发、一批、暂停时只删意图的那一次、重新签发、最后一批；删意图那一版 logged_ops 空、不搬单元、R 里还有未释放的记录；
/// 重新签发的意图在同一个 R 上、`c0` 比第一条大；最后一批 R 按记录全空、同一次发布删意图。
#[test]
fn the_flow_drops_the_intent_on_the_pause_and_issues_it_again_on_the_same_region() {
    let prepared = prepare("compaction-paused-short-of-space-shape");
    assert_the_shape_of_the_paused_flow(&prepared);
}

/// 出错停下删意图那条流（第一批之后，下一批注入一次算着出错：那一批放弃、单发一次只删意图的发布、整理停下；切换解除停下之后重新签发、最后一批）
/// 与这条流写的一样：每一步的种类相同，录制流逐步相同（基线镜像、写表与段序列因此逐项相同）。两条流各自核删意图那一步恰好发一次只删意图的发布。
#[test]
fn the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes() {
    let contents = contents_of_the_sequential_writes();
    let paused = record_the_compaction_crash_flow(
        "compaction-paused-short-of-space-against-the-stop",
        CompactionCrashFlow::CompactionPausedShortOfSpaceDroppingTheIntent,
        &contents,
    );
    let stopped = record_the_compaction_crash_flow(
        "compaction-stopped-after-a-compute-error-against-the-pause",
        CompactionCrashFlow::CompactionStoppedAfterAComputeErrorDroppingTheIntent,
        &contents,
    );
    let kinds_of = |steps: &[common_compaction_prefix::CompactionFlowStep]| {
        steps.iter().map(|step| step.kind).collect::<Vec<_>>()
    };
    assert_eq!(
        kinds_of(&stopped.steps),
        kinds_of(&paused.steps),
        "两条流每一步的种类相同（删意图那一步都是只删意图的发布）"
    );
    assert!(
        kinds_of(&stopped.steps).contains(&CompactionFlowStepKind::CompactionIntentDroppedOnAStop),
        "出错停下那条流里有只删意图的那一步"
    );
    assert!(
        stopped.pool.retained_operations() == paused.pool.retained_operations(),
        "两条流的录制流逐步相同：出错停下那一次删意图写的就是暂停那一次删意图写的"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并可写挂载一次核整理续做。登记成崩溃枚举用例。
#[test]
fn full_enumeration_of_the_compaction_paused_short_of_space_dropping_the_intent_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-paused-short-of-space-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_paused_short_of_space_dropping_the_intent::full",
        &full_expansion,
        JudgeOfTheEnumeration::ConfiguredByTheCaller,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_PAUSED_SHORT_OF_SPACE_DROPPING_THE_INTENT states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
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
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "compaction_paused_short_of_space_dropping_the_intent_stream：用例自己的展开下的闭式",
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 只剩 R 有空槽那一条流录流的形状（下面那条形状用例与 CPU 判器那一条在枚举之前都核）。
fn assert_the_shape_of_the_flow_with_only_the_region_free(prepared: &PreparedCompactionCrashFlow) {
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    let position_of_the_mount = kinds
        .iter()
        .position(|kind| *kind == CompactionFlowStepKind::MountWritable)
        .expect("流里有那一次重挂");
    assert_eq!(
        kinds[position_of_the_mount + 1..],
        [
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionIntentDroppedOnAStop
        ],
        "重挂之后：签发、一批、暂停删意图：{kinds:?}"
    );
    let region = region_of_the_crash_flows();
    let only_version_of = |step_index: usize| {
        let [version] = recorded.steps[step_index].versions.as_slice() else {
            panic!("{:?}：一步一次发布", recorded.steps[step_index].kind);
        };
        version
    };
    let issue = only_version_of(position_of_the_mount + 1);
    let dropped = only_version_of(position_of_the_mount + 3);
    let (_, intent) = live_compaction_intent_of_the_entries(&issue.logged_operations)
        .expect("签发那一版有活意图");
    assert!(
        dropped.logged_operations.is_empty()
            && dropped.units_relocated_by_this_publish.is_empty()
            && !region_is_empty_by_the_records(&dropped.allocation_records, region),
        "暂停那一次只删意图、不搬单元，R 里还有未释放的记录"
    );
    assert!(
        region_holds_a_placement_of_the_publish_of_the_version(dropped)
            && dropped.root.checkpoint_txg.0 > intent.allocation_generation_at_issue.0,
        "删意图那一次自己的新落点落在 R 里，分配代比意图的 c0 大"
    );
}

/// 只剩 R 有空槽那一条流录流的形状：重挂之后是签发、一批、暂停时只删意图的那一次，到此为止；删意图那一版 logged_ops 空、不搬单元、
/// R 里还有未释放的记录，它自己的新落点落在 R 里，分配代比那条意图的 `c0` 大（意图要是还活着，这就是 I-5.6 的违例）。
#[test]
fn the_flow_with_only_the_region_free_drops_the_intent_with_its_placements_in_the_region() {
    let prepared = prepare_with_only_the_region_free("compaction-drop-placed-in-the-region-shape");
    assert_the_shape_of_the_flow_with_only_the_region_free(&prepared);
}

/// 只剩 R 有空槽那一条流的全量：每段都展开，每个状态两遍恢复 + checker（含 I-5.6）+ 抹掉源落点再读，并可写挂载一次核整理续做。登记成崩溃枚举用例。
#[test]
fn full_enumeration_of_the_compaction_paused_with_only_the_region_free_dropping_the_intent_in_the_region_is_exhaustive_and_clean(
) {
    let prepared = prepare_with_only_the_region_free("compaction-drop-placed-in-the-region-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_paused_short_of_space_dropping_the_intent::dropping_in_the_region",
        &full_expansion,
        JudgeOfTheEnumeration::ConfiguredByTheCaller,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_DROP_PLACED_IN_THE_REGION states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
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
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "compaction_paused_with_only_the_region_free_dropping_the_intent_in_the_region：用例自己的展开下的闭式",
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 在只开 CPU 的多机配置下跑 `enumerate`：照调用方的多机配置写一份只开 CPU 的（`crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of`：
/// 不开 GPU，判段交 CPU 判器；照抄 prune-server 的地址，块照块键与判器组的单元键进全仓那一份库），环境守卫里把多机配置指过去；
/// 调用方设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（指定范围的那几趟可缩域）照它缩，守卫清掉之前记下、清完设回（别的变量照清，只换多机配置）。
/// 只开 CPU 的配置写在 `tag` 起名的测试自己的临时目录里，`enumerate` 跑完连同两个守卫一起放掉。
///
/// 换上之后、`enumerate` 之前（录入段与判段都还没起）核这个进程读到的就是那一份、它定的跑法不上卡：换不上的当场红，
/// 不照调用方的配置往下判（`crates/mutations.tsv` 里每份文件一行把换上的那一份改回调用方的，红在这里）。
///
/// # Panics
/// 调用方的多机配置取不到、临时目录建不了、只开 CPU 的配置写不进；换上之后这个进程读到的不是那一份，或那一份定的跑法上卡。
// gpu-exempt: cpu-judge-mutation-rows
fn with_only_the_cpu_judge_configured<Outcome>(
    tag: &str,
    enumerate: impl FnOnce() -> Outcome,
) -> Outcome {
    // 调用方的多机配置在守卫清掉 SINGLEFS_MULTI_HOST_CONFIG 之前取（仓副本里只经这个变量交进来）
    let configuration_of_the_caller =
        singlefs_verification::crash_amplification::multi_host_configuration_of_this_process()
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .configuration_file;
    let configuration_directory =
        ExclusiveTemporaryDirectory::create(tag).unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration_on_the_cpu = configuration_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(
        &configuration_on_the_cpu,
        multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
            &configuration_of_the_caller,
        )
        .unwrap_or_else(|refusal| panic!("{refusal}")),
    )
    .expect("测试自己的临时目录里写得了只开 CPU 的多机配置");
    // 调用方缩过域的那一趟照它缩：守卫清掉一趟读的全部变量之前记下，清完设回
    let narrowing_of_the_caller = std::env::var_os(
        singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
    );
    // 先拿全仓那一份的守卫、再拿改环境的锁（`crash_library::TheOneLibraryKeptForThisProcess` 文档里的次序）；丢的时候倒过来
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration_on_the_cpu.as_os_str(),
    )]);
    if let Some(expand_up_to) = &narrowing_of_the_caller {
        environment.set_variable(
            singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            expand_up_to,
        );
    }
    let configuration_in_effect =
        singlefs_verification::crash_amplification::multi_host_configuration_of_this_process()
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(
        configuration_in_effect.configuration_file, configuration_on_the_cpu,
        "环境守卫里换上的多机配置是测试自己写的那一份只开 CPU 的"
    );
    assert!(
        !configuration_in_effect.mode.uses_gpu(),
        "只开 CPU 的多机配置定的跑法不上卡：{}",
        configuration_in_effect.mode.name()
    );
    enumerate()
}

/// 判段交哪一个判器（[`enumerate_with_both_observers`] 的最后一个参数）。全量那两条只差这一样：录的是同一条流、挂的是同两个观察者，块键相同，
/// CPU 判器那一条从库里复用另一条判过的块。
enum JudgeOfTheEnumeration {
    /// 照调用方的多机配置：开着 GPU 交 GPU 判器，没开交 CPU 判器。
    ConfiguredByTheCaller,
    /// 只开 CPU 的多机配置（[`with_only_the_cpu_judge_configured`]，那一份写进 `configuration_label` 起名的临时目录）：判段一律交 CPU 判器。
    TheCpuJudgeOnly { configuration_label: &'static str },
}

/// 按 `expansion` 枚举一趟，项名 `item_name`，每个状态交两个观察者（抹掉源落点再读；可写挂载续做，交回挂完的盘），判段交 `judge` 点名的判器。
/// 两条流的全量与各自的 CPU 判器那一条全量都经它。
fn enumerate_with_both_observers(
    prepared: &PreparedCompactionCrashFlow,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    judge: JudgeOfTheEnumeration,
) -> Option<CrashPointReplayTally> {
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_through_the_mapping(
            versions_of_the_flow,
            image,
            report,
            counts,
        );
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_compaction(versions_of_the_flow, image, report, counts)
        };
    let enumerate = || {
        enumerate_crash_states_with_every_kind_of_observer(
            CrashPointReplayItem {
                name: item_name,
                setting_file: PATH_OF_THIS_TEST_FILE,
            },
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            prepared.judged_root_index,
            version_table(prepared),
            expansion,
            &[landed_version_observer(prepared, &observe)],
            &[mount_continuation_observer(prepared, &observe_the_mount)],
        )
        .judged()
    };
    match judge {
        JudgeOfTheEnumeration::ConfiguredByTheCaller => enumerate(),
        JudgeOfTheEnumeration::TheCpuJudgeOnly {
            configuration_label,
        } => with_only_the_cpu_judge_configured(configuration_label, enumerate),
    }
}

/// 判器零红（CPU 判器那两条用，判据同两条全量里逐条写的那几样）：两遍恢复都过 oracle、走读一次都不失败、记录核对器两条判据、
/// 池级 checker（含 I-5.6）每条已实现的不变量都没有违例的状态。
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

/// 两条流各自的 CPU 判器那一条（下面两条用例调它）：同一条流、同一组崩溃状态（每段都展开，同那条流的全量）、同两个观察者，
/// 在只开 CPU 的多机配置下先录进库、再从库读回，逐状态在 CPU 判器上判（两遍恢复 + 多版本 oracle、池级 checker（含 I-5.6）、记录核对器；
/// 挂完的盘在判段同样交 CPU 判器过池级 checker）。判完打计数行 `count_line_name`；状态数等于闭式（缩过域照缩域的判），判器零红，
/// 每个状态都落到流里某一版或 A 之前、观察者计数同那条流的全量。与那条流的全量经同一个 [`enumerate_with_both_observers`]，只差判段交哪一个判器：
/// 块键相同，库里有全量那一条判过的块就照块复用、不在 CPU 上重判。
///
/// # Panics
/// 上面任何一样对不上；只开 CPU 的多机配置写不成或换不上（[`with_only_the_cpu_judge_configured`]）。
// gpu-exempt: cpu-judge-mutation-rows
fn every_crash_state_of_the_flow_judged_on_the_cpu_judge_is_clean(
    prepared: &PreparedCompactionCrashFlow,
    configuration_label: &'static str,
    item_name: &str,
    count_line_name: &str,
) {
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        prepared,
        item_name,
        &full_expansion,
        JudgeOfTheEnumeration::TheCpuJudgeOnly {
            configuration_label,
        },
    ) else {
        return;
    };
    println!(
        "{count_line_name} states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        &format!("{item_name}（CPU 判器）：用例自己的展开下的闭式"),
    );
    assert_every_state_was_observed(prepared, &tally, &landed_counters(prepared));
}

/// 整理暂停删意图那条流的 CPU 判器那一条（[`every_crash_state_of_the_flow_judged_on_the_cpu_judge_is_clean`]）。
/// 判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_PAUSED_SHORT_OF_SPACE_DROPPING_THE_INTENT_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-paused-short-of-space-dropping-the-intent-on-the-cpu-judge`）。
/// 枚举之前先核录流的形状（[`assert_the_shape_of_the_paused_flow`]）。
/// 判别力（`crates/mutations.tsv`「整理暂停删意图流 CPU 判器那一条：环境守卫里换上的多机配置改回调用方那一份」）：换不上只开 CPU 的配置，
/// [`with_only_the_cpu_judge_configured`] 在录入段之前就红。池级 checker 判反的判别力由点名那条流全量的变异行担（改 GPU 判器那一份）。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_of_the_compaction_paused_short_of_space_dropping_the_intent_stream_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-paused-short-of-space-full-on-the-cpu-judge");
    assert_the_shape_of_the_paused_flow(&prepared);
    every_crash_state_of_the_flow_judged_on_the_cpu_judge_is_clean(
        &prepared,
        "compaction-paused-short-of-space-on-the-cpu-judge",
        "compaction_paused_short_of_space_dropping_the_intent::full_on_the_cpu_judge",
        "CRASH_POINT_REPLAY_COMPACTION_PAUSED_SHORT_OF_SPACE_DROPPING_THE_INTENT_ON_THE_CPU_JUDGE",
    );
}

/// 只剩 R 有空槽那一条流的 CPU 判器那一条（[`every_crash_state_of_the_flow_judged_on_the_cpu_judge_is_clean`]）。
/// 判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_DROP_PLACED_IN_THE_REGION_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-paused-with-only-the-region-free-dropping-the-intent-in-the-region-on-the-cpu-judge`）。
/// 枚举之前先核录流的形状（[`assert_the_shape_of_the_flow_with_only_the_region_free`]）。
/// 判别力（`crates/mutations.tsv`「只剩 R 有空槽暂停删意图流 CPU 判器那一条：录成整理暂停删意图那条流」）：`prepare_with_only_the_region_free`
/// 录成另一条流，重挂之后多出重新签发与最后一批，形状那一核在录入段之前就红。池级 checker 判反的判别力由点名那条流全量的变异行担（改 GPU 判器那一份）。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_of_the_compaction_paused_with_only_the_region_free_dropping_the_intent_in_the_region_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    let prepared = prepare_with_only_the_region_free(
        "compaction-drop-placed-in-the-region-full-on-the-cpu-judge",
    );
    assert_the_shape_of_the_flow_with_only_the_region_free(&prepared);
    every_crash_state_of_the_flow_judged_on_the_cpu_judge_is_clean(
        &prepared,
        "compaction-drop-placed-in-the-region-on-the-cpu-judge",
        "compaction_paused_short_of_space_dropping_the_intent::dropping_in_the_region_on_the_cpu_judge",
        "CRASH_POINT_REPLAY_COMPACTION_DROP_PLACED_IN_THE_REGION_ON_THE_CPU_JUDGE",
    );
}
