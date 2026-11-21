//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 活意图期间写类改动的崩溃流（D26（后台整理与放置回收） 已定项 13 第 5 条挂载就起之后，整理批与前台写在一把锁上交错成的次序）：
//! 「取号 → 暖机 × 2 → A → B、C 两次顺序写 → 实例 1 里建空 inode 填满几个叶容器 → 重挂（取号、写行、暖机）→ 整理签发 → 一批 → 意图活着时会话里覆盖写第一个文件 → 一批 → 最后一批（同一次发布删意图）」
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 与批大小同整理搬迁流（第一个文件那次发布的提交内生块所在的段，一批两个）。
//! 建 inode 那一步是这条流站得住的前提：覆盖写第一个文件重写的单元把整理搬迁流 R 里第一批搬剩下的全放掉，R 按记录全空，之后只有一次不搬单元、只删意图的发布；
//! 那次发布的叶容器也落在 R 里，覆盖写只重写第一片，别的留给下一批。
//! 每个状态跑恢复 + 多版本 oracle（覆盖写之前读 C 那一份，之后读覆盖写那一份）、池级 checker、记录核对器；观察者同整理搬迁流：
//! 抹掉这一版之前整理搬走的单元的源落点再读、可写挂载一次看整理续做推到意图删了。
//!
//! **比已有的流多罩了什么**：已有的整理流上活意图期间只有 unlink（`crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs`），没有写类改动。
//! 这条流第一次枚举覆盖写放掉 R 里落点之后的状态：那次发布重写的节点在 R 里的旧落点进 defer，R 里剩下要搬的少了，下一批对 R 重扫；
//! 崩在那次发布与下一批之间的每个状态上，可写挂载续做读到的是覆盖写之后那一版的 R。多跑的步：覆盖写那一版也走一遍两个观察者。前缀到 C 与整理搬迁流逐字节相同。
//!
//! 每段都展开的那一条登记成崩溃枚举用例，带断点续跑、可双机分片。
//! 同一组崩溃状态另有一条在 CPU 判器上判的崩溃枚举用例（只开 CPU 的多机配置，先录进库、再从库读回判；两个观察者同上，挂完的盘同样在判段过池级 checker）：
//! 多机配置开着 GPU 时上面那一条的判段交 GPU 判器，这一条的判段一律交 CPU 判器；两条经同一个枚举函数录同一条流、挂同两个观察者，块键相同，
//! 库里有上面那一条判过的块就照块复用。改坏池级 checker 判反的变异行点名上面那一条（改 GPU 判器那一份），这一条的变异行在录入段之前就红。
//! 它不多罩崩溃状态，多跑的是 CPU 判器那一遍判。
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
    free_slots_in_the_region_of_the_crash_flows_on_each_device,
    isolate_every_free_slot_outside_the_region_of_the_crash_flows,
    prepare_the_compaction_crash_flow, region_holds_a_placement_of_the_publish_of_the_version,
    region_of_the_crash_flows,
    session_with_the_region_emptied_by_an_overwrite_while_the_intent_is_live, version_of_trees,
    CompactionCrashFlow, CompactionFlowStepKind, PreparedCompactionCrashFlow,
    PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE,
};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
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
    "crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs";

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive,
        &[
            common_crash_points::content_needing(2, 1),
            common_crash_points::content_needing(3, 2),
        ],
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

/// 录流的形状：覆盖写夹在第一批与下一批之间、那一版意图还活着；覆盖写放掉了 R 里的落点（R 里未释放的记录比上一批少，不只少这一批搬走的）；
/// 之后接着一批、最后一批，都搬 R 里的单元（下一批对 R 重扫，搬的是覆盖写没动的那几个）；最后一批 R 按记录全空。
#[test]
fn the_overwrite_of_the_flow_sits_between_two_batches_and_releases_placements_in_the_region() {
    let prepared = prepare("compaction-write-while-the-intent-is-live-shape");
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    let position_of_the_overwrite = kinds
        .iter()
        .position(|kind| *kind == CompactionFlowStepKind::OverwriteWhileTheIntentIsLive)
        .expect("流里有那一次覆盖写");
    assert_eq!(
        kinds[position_of_the_overwrite - 1],
        CompactionFlowStepKind::CompactionBatch,
        "覆盖写接在一批之后：{kinds:?}"
    );
    assert_eq!(
        kinds[position_of_the_overwrite + 1..],
        [
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionFinalBatch
        ],
        "覆盖写之后接着搬：一批照抄意图，再一批同一次发布删意图：{kinds:?}"
    );
    let region = region_of_the_crash_flows();
    for step in &recorded.steps[position_of_the_overwrite + 1..] {
        let relocated: Vec<_> = step
            .versions
            .iter()
            .flat_map(|version| version.units_relocated_by_this_publish.iter())
            .collect();
        assert!(!relocated.is_empty(), "{:?}：这一批真搬了单元", step.kind);
        assert!(
            relocated.iter().all(|unit| unit
                .source
                .iter()
                .all(|location| region.contains_slot(location.slot))),
            "{:?}：搬的是 R 里的单元（下一批对 R 重扫）",
            step.kind
        );
    }
    let unreleased_records_in_the_region = |step_index: usize| -> usize {
        let version = recorded.steps[step_index]
            .versions
            .last()
            .expect("这一步发成了一版");
        version
            .allocation_records
            .iter()
            .filter(|record| {
                record.device == region.device()
                    && !record.is_released
                    && region.contains_slot(record.slot)
            })
            .count()
    };
    assert!(
        unreleased_records_in_the_region(position_of_the_overwrite)
            < unreleased_records_in_the_region(position_of_the_overwrite - 1),
        "覆盖写重写的节点在 R 里的旧落点放掉了"
    );
    let overwrite_version = recorded.steps[position_of_the_overwrite]
        .versions
        .last()
        .expect("覆盖写发成了一版");
    assert_eq!(
        overwrite_version.logged_operations.len(),
        1,
        "覆盖写那一版意图还活着"
    );
    let last_version = recorded
        .steps
        .last()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert!(
        last_version.logged_operations.is_empty()
            && region_is_empty_by_the_records(&last_version.allocation_records, region),
        "最后一批同一次发布删意图、R 按记录全空"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并可写挂载一次核整理续做。登记成崩溃枚举用例。
#[test]
fn full_enumeration_of_the_compaction_with_a_write_while_the_intent_is_live_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-write-while-the-intent-is-live-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_with_a_write_while_the_intent_is_live::full",
        &full_expansion,
        JudgeOfTheEnumeration::ConfiguredByTheCaller,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_WITH_A_WRITE_WHILE_THE_INTENT_IS_LIVE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_with_a_write_while_the_intent_is_live_stream：用例自己的展开下的闭式",
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
/// 全量与 CPU 判器那一条全量都经它。
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

/// 判器零红（CPU 判器那一条用，判据同上面那条全量里逐条写的那几样）：两遍恢复都过 oracle、走读一次都不失败、记录核对器两条判据、
/// 池级 checker 每条已实现的不变量都没有违例的状态。
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

/// 全量的 CPU 判器那一条：同一条流、同一组崩溃状态（每段都展开）、同两个观察者，在只开 CPU 的多机配置下先录进库、再从库读回，
/// 逐状态在 CPU 判器上判（两遍恢复 + 多版本 oracle、池级 checker、记录核对器；挂完的盘在判段同样交 CPU 判器过池级 checker）。
/// 状态数等于闭式（缩过域照缩域的判），判器零红，观察者计数同上面那条全量。
/// 判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_WITH_A_WRITE_WHILE_THE_INTENT_IS_LIVE_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-with-a-write-while-the-intent-is-live-on-the-cpu-judge`）。
/// 与上面那条全量经同一个 [`enumerate_with_both_observers`]，只差判段交哪一个判器：块键相同，库里有上面那条判过的块就照块复用、不在 CPU 上重判。
/// 判别力（`crates/mutations.tsv`「活意图期间写类改动流 CPU 判器那一条：环境守卫里换上的多机配置改回调用方那一份」）：换不上只开 CPU 的配置，
/// [`with_only_the_cpu_judge_configured`] 在录入段之前就红。池级 checker 判反的判别力由点名上面那条全量的变异行担（改 GPU 判器那一份）。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_of_the_compaction_with_a_write_while_the_intent_is_live_stream_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-write-while-the-intent-is-live-full-on-the-cpu-judge");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_with_a_write_while_the_intent_is_live::full_on_the_cpu_judge",
        &full_expansion,
        JudgeOfTheEnumeration::TheCpuJudgeOnly {
            configuration_label: "compaction-write-while-the-intent-is-live-on-the-cpu-judge",
        },
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_WITH_A_WRITE_WHILE_THE_INTENT_IS_LIVE_ON_THE_CPU_JUDGE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_with_a_write_while_the_intent_is_live_stream（CPU 判器）：用例自己的展开下的闭式",
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// R 已按记录全空、不带一批只删意图的那一次取落点放开 R（D26（后台整理与放置回收） 已定项 2）：不建空 inode 的前缀上，第一批搬走 R 里两个、
/// 意图活着时覆盖写第一个文件放掉剩下的两个，R 按记录全空、意图还活着；再把 R 之外的每一个空槽隔离（不写盘），整理推一次：
/// 这一次不带一批、只删意图，新落点只能落在 R 里；删意图那一核不把这次发布自己在 R 里的新落点算成「R 不空」，发成，意图连同 R 排他撤了。
/// 判别力：`crates/mutations.tsv`「不带一批的删意图那一核数自己在 R 里的新落点」那一行。
#[test]
fn the_deletion_without_a_batch_takes_its_placements_in_the_region_once_only_the_region_has_free_slots(
) {
    let (mut pool, mut session) =
        session_with_the_region_emptied_by_an_overwrite_while_the_intent_is_live(
            "compaction-deletion-without-a-batch-placed-in-the-region",
            &[
                common_crash_points::content_needing(2, 1),
                common_crash_points::content_needing(3, 2),
            ],
        );
    let free_in_the_region = free_slots_in_the_region_of_the_crash_flows_on_each_device(&session);
    assert!(
        free_in_the_region.iter().all(|(_, free)| *free > 0),
        "前提：R 里每块盘都有空槽（盘、空槽数）{free_in_the_region:?}"
    );
    isolate_every_free_slot_outside_the_region_of_the_crash_flows(&mut session);
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let txg_before_the_run = version_of_trees(&session.current).root.checkpoint_txg;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let run = session
        .run_compaction(devices)
        .unwrap_or_else(|refusal| panic!("只删意图那一次发得成：{refusal:?}"));
    let [deleted] = run.versions_published.as_slice() else {
        panic!(
            "这一趟恰好发一次只删意图的发布，实际发了 {} 次，停在 {:?}",
            run.versions_published.len(),
            run.outcome
        );
    };
    assert!(
        deleted.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && deleted.logged_operations.is_empty()
            && deleted.units_relocated_by_this_publish.is_empty()
            && !region_holds_an_unreleased_record_from_before(deleted),
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图，R 里上一版就在的记录一条都没有"
    );
    assert!(
        region_holds_a_placement_of_the_publish_of_the_version(deleted),
        "删意图那一次自己的新落点落在 R 里（R 之外一个空槽都没有）"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "分配器这一侧意图连同 R 排他一起撤了"
    );
}

/// 这一版 R 里有一条分配代不是它自己的 txg 的未释放记录（上一版就在 R 里的）。
fn region_holds_an_unreleased_record_from_before(
    version: &singlefs_core::transaction::TransactionOutput,
) -> bool {
    let region = region_of_the_crash_flows();
    version.allocation_records.iter().any(|record| {
        !record.is_released
            && record.generation != version.root.checkpoint_txg
            && region.contains_slot(record.slot)
    })
}
