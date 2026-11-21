//! 验证包模块：crash、crash_state_enumeration、device_log、crash_library、crash_amplification
//! 整理搬迁流（里程碑四二·3，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G6 前缀 a）：
//! 「取号 → 暖机 × 2 → A → B、C 两次顺序写（多记录流那两次）→ 重挂（取号、写行、暖机）→ 整理签发（意图写进 logged_ops 树）→ 一批（两个单元）
//! → 最后一批（两个单元，同一次发布删意图）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 强制取第一个文件那次发布的提交内生块所在的段
//! （重挂之后不是本次挂载开过的段），一批两个（判决 G6：批大小 1–3，单元段写数 ≤ 30）。每个状态跑恢复 + 多版本 oracle（每一版都有文件，
//! 重挂与整理不动内容）、池级 checker、记录核对器；另逐状态核两样（每个状态都核，观察者在 `tests/common_compaction_crash_point_observers/mod.rs`）：
//! - 恢复落到的那一版与它之前整理搬走的单元的源落点整槽抹成 0 之后，第一个文件照样读出这一版的内容（读路径照映射走到目的地）；
//! - 可写挂载一次（整理开关同录流）：落到签发与那一批的状态上整理续做推到意图删了（最后一批同一次发布删意图、R 按记录全空），落到别的版本不发；
//!   挂完 logged_ops 空、第一个文件读得回；挂完的盘在判段过池级 checker。
//!
//! 流的搭建住在公共模块 `crates/singlefs-verification/tests/common_compaction_prefix/mod.rs`（两条整理流与生命周期闭环流共用一份录法）。
//!
//! **比已有的流多罩了什么**：已有的流上没有整理；签发那次发布第一次把 logged_ops 树写成非零根，两批第一次把单元原样拷到目的地、映射条目换位置、
//! 源落点同一次发布进 defer，崩在这几次的单元、记录、根槽写之间的每个状态第一次被枚举到。前缀到 C 与多记录流逐字节相同（标准前缀节点共享），
//! 重挂那一段同可写挂载流的形态。多跑的两步：逐状态抹掉源落点再读、每个状态上的可写挂载续做与挂完的盘上那一遍池级 checker。
//! 这条流上没有 discard（R 搬空之后源落点还在 defer，回收与 discard 在生命周期闭环流上）：设备日志判器在这条流上判 0 条，另有一条必红世界
//! 在最后一批之后插一段对 R 的 discard。
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
    prepare_the_compaction_crash_flow, region_of_the_crash_flows, CompactionCrashFlow,
    CompactionFlowStepKind, PreparedCompactionCrashFlow, COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
    REGION_START_SLOT_OF_THE_CRASH_FLOWS,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::recovery::RecoveryReport;
use singlefs_format::SLOT_BYTES;
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
use singlefs_verification::device_log::{
    expected_device_events, judge_discards_against_the_publishes, MisplacedDiscard,
};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, CrashImage,
};
use singlefs_verification::{RecordedOperation, RecordedOperationKind, RetainedOperation};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream.rs";

/// 段形（录流按真写表数出来的）：前二十段同多记录流（取号两轮、暖机、A 24、B 28、C 30 个单元写，各自记录、根、轮换）；
/// 重挂：取号两轮系统配置各 2、写行 18、暖机 16 个单元写；签发 18、一批 20、最后一批 24 个单元写，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 42] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    18, 2, 1, 2, 20, 2, 1, 2, 24, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（`memory_pool::closed_form_state_count_of_write_index_segments`；取号写一轮时 1_377_370_253，两次取号各多一段 2，各 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 1_377_370_259;

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionRelocation,
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

/// 流里每一种那一版的计数名（A、B 与 C、重挂、签发、一批、最后一批），每一种都要有状态。
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

/// 录流的形状（判器判完之后再核）：步的次序；两批各搬两个单元、没有一个只被快照引用，源落点都在 R 里、目的落点都在 R 外；签发与那一批
/// 那两版 logged_ops 有一条意图，最后一批那一版没有、R 按记录全空；整条流没有 discard。
fn assert_the_shape_of_the_flow(prepared: &PreparedCompactionCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前二十段同多记录流；重挂、签发 18、一批 20、最后一批 24 个单元写"
    );
    let recorded = &prepared.recorded;
    assert_eq!(
        recorded
            .steps
            .iter()
            .map(|step| step.kind)
            .collect::<Vec<_>>(),
        vec![
            CompactionFlowStepKind::SequentialWrite,
            CompactionFlowStepKind::SequentialWrite,
            CompactionFlowStepKind::MountWritable,
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionFinalBatch,
        ],
        "B、C → 重挂 → 签发 → 一批 → 最后一批"
    );
    let relocated = recorded.relocated_units();
    assert_eq!(
        relocated.len(),
        2 * COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
        "两批各搬两个单元"
    );
    let region = region_of_the_crash_flows();
    for unit in &relocated {
        assert!(
            !unit.is_kept_only_for_the_snapshots,
            "这条流上没有快照：{unit:?}"
        );
        assert!(
            unit.source
                .iter()
                .all(|(_device, slot)| region.contains_slot(*slot)),
            "源落点在 R 里：{unit:?}"
        );
        assert!(
            unit.destination
                .iter()
                .all(|(_device, slot)| !region.contains_slot(*slot)),
            "目的落点在 R 外（R 排他）：{unit:?}"
        );
    }
    let final_batch = recorded
        .steps_of(CompactionFlowStepKind::CompactionFinalBatch)
        .first()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert!(
        final_batch.logged_operations.is_empty()
            && region_is_empty_by_the_records(&final_batch.allocation_records, region),
        "最后一批同一次发布删意图、R 按记录全空"
    );
    for kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        for step in recorded.steps_of(kind) {
            assert!(
                step.versions
                    .iter()
                    .all(|version| version.logged_operations.len() == 1),
                "{kind:?}：logged_ops 里一条意图"
            );
        }
    }
    assert!(
        prepared.discards_in_the_stream().is_empty(),
        "R 搬空之后源落点还在 defer，这条流上没有 discard"
    );
}

/// 判段交哪一个判器（[`enumerate_with_both_observers`] 的最后一个参数）。全量那两条只差这一样：录的是同一条流、挂的是同两个观察者，块键相同，
/// CPU 判器那一条从库里复用另一条判过的块。
enum JudgeOfTheEnumeration {
    /// 照调用方的多机配置：开着 GPU 交 GPU 判器，没开交 CPU 判器。
    ConfiguredByTheCaller,
    /// 只开 CPU 的多机配置（[`with_only_the_cpu_judge_configured`]，那一份写进 `configuration_label` 起名的临时目录）：判段一律交 CPU 判器。
    TheCpuJudgeOnly { configuration_label: &'static str },
}

/// 按 `expansion` 枚举一趟，项名 `item_name`，每个状态交两个观察者（照映射读、抹掉源落点再读；可写挂载续做，交回挂完的盘），判段交 `judge` 点名的判器。
/// 这份文件里每一趟枚举都经它（全量、CPU 判器那一条全量、只展开写数小于 10 的段那一趟）。
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

/// 只展开写数小于 10 的段那一趟（测从库里重判计数相同）：单元段不展开，每个状态交两个观察者，照调用方的配置判。
fn enumerate_the_segments_shorter_than_ten(
    prepared: &PreparedCompactionCrashFlow,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    enumerate_with_both_observers(
        prepared,
        &format!("compaction_relocation::segments_shorter_than_ten::{item_label}"),
        &expansion_of_the_segments_shorter_than_ten,
        JudgeOfTheEnumeration::ConfiguredByTheCaller,
    )
}

/// 复用不改判定：只展开写数小于 10 的段跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_compaction_relocation_with_the_segments_shorter_than_ten_expanded_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("compaction-relocation-crash-point-replay-rerun");
    let Some(first_run) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun") else {
        return;
    };
    let Some(second_run) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun") else {
        return;
    };
    assert_eq!(second_run, first_run, "第二遍全照库里复用，计数逐项相同");
    assert_eq!(
        first_run.observed_states, first_run.states,
        "每个状态都看过"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并在工作线程上可写挂载一次核整理续做。登记成崩溃枚举用例，
/// crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_COMPACTION_RELOCATION` 行里 `exhaustive=true`；带断点续跑、可双机分片。
#[test]
fn full_enumeration_of_the_compaction_relocation_stream_is_exhaustive_and_clean() {
    let prepared = prepare("compaction-relocation-crash-point-replay-full");
    assert_eq!(
        closed_form_state_count_of_write_index_segments(&prepared.segments),
        CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_relocation::full",
        &full_expansion,
        JudgeOfTheEnumeration::ConfiguredByTheCaller,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATION states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_relocation_stream：用例自己的展开下的闭式",
    );
    assert_the_shape_of_the_flow(&prepared);
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

/// 全量的 CPU 判器那一条：同一条流、同一组崩溃状态（每段都展开）、同两个观察者，在只开 CPU 的多机配置下先录进库、再从库读回，
/// 逐状态在 CPU 判器上判（两遍恢复 + 多版本 oracle、池级 checker、记录核对器；挂完的盘在判段同样交 CPU 判器过池级 checker）。
/// 状态数等于闭式（缩过域照缩域的判），判器零红，录流的形状与观察者计数同上面那条全量。判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_RELOCATION_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-relocation-on-the-cpu-judge`）。
/// 与上面那条全量经同一个 [`enumerate_with_both_observers`]，只差判段交哪一个判器：块键相同，库里有上面那条判过的块就照块复用、不在 CPU 上重判。
/// 判别力（`crates/mutations.tsv`「整理搬迁流 CPU 判器那一条：环境守卫里换上的多机配置改回调用方那一份」）：换不上只开 CPU 的配置，
/// [`with_only_the_cpu_judge_configured`] 在录入段之前就红。池级 checker 判反的判别力由点名上面那条全量的变异行担（改 GPU 判器那一份）。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_of_the_compaction_relocation_stream_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-relocation-crash-point-replay-full-on-the-cpu-judge");
    assert_eq!(
        closed_form_state_count_of_write_index_segments(&prepared.segments),
        CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_with_both_observers(
        &prepared,
        "compaction_relocation::full_on_the_cpu_judge",
        &full_expansion,
        JudgeOfTheEnumeration::TheCpuJudgeOnly {
            configuration_label: "compaction-relocation-on-the-cpu-judge",
        },
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATION_ON_THE_CPU_JUDGE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_relocation_stream（CPU 判器）：用例自己的展开下的闭式",
    );
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 设备日志判器（判决 G5，可发区间由判器自己从录制流里的发布算）在这条流的录制流上：两块盘都判过 0 条 discard（这条流不发 discard）。
#[test]
fn the_device_log_of_the_compaction_relocation_stream_holds_no_discard_and_is_judged_clean() {
    let prepared = prepare("compaction-relocation-device-log");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let events = expected_device_events(&prepared.operations_after_mkfs, device);
        assert_eq!(
            judge_discards_against_the_publishes(
                &prepared.base,
                &prepared.operations_after_mkfs,
                &common::geometry(),
                device,
                &events,
            ),
            Ok(0),
            "盘 {device:?}：这条流不发 discard"
        );
    }
}

/// 必红世界（判决 G5：discard 只发在可再分配的整段上）：最后一批返回之后对 R 两块盘各插一段 discard——R 里的单元刚搬走，源落点还在 defer、
/// 近几代根仍引用它们，在任何一次发布之后都不可发，设备日志判器判它落在不可发的槽上。
#[test]
fn a_discard_of_the_region_right_after_the_final_batch_is_judged_outside_the_discardable_ranges() {
    let prepared = prepare("compaction-relocation-device-log-inserted-discard");
    let segment_bytes = 64 * SLOT_BYTES;
    let region_offset = REGION_START_SLOT_OF_THE_CRASH_FLOWS * SLOT_BYTES;
    let mut operations = prepared.operations_after_mkfs.clone();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        operations.push(RetainedOperation {
            operation: RecordedOperation {
                device,
                kind: RecordedOperationKind::Discard,
                offset: DeviceOffsetInBytes(region_offset),
                length: segment_bytes,
                content_hash: singlefs_verification::fnv1a_64_of_zeros(segment_bytes),
            },
            contents: None,
        });
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let events = expected_device_events(&operations, device);
        let verdict = judge_discards_against_the_publishes(
            &prepared.base,
            &operations,
            &common::geometry(),
            device,
            &events,
        );
        assert!(
            matches!(
                verdict,
                Err(MisplacedDiscard::OutsideTheDiscardableRanges { offset, length, .. })
                    if offset == DeviceOffsetInBytes(region_offset) && length == segment_bytes
            ),
            "盘 {device:?}：刚搬空的 R 不可发，判落在不可发的槽上：{verdict:?}"
        );
    }
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。展开上限默认 6。
/// 崩溃点照公共模块 `common_crash_points` 设（`crash_points_of_a_flow_after_the_first_file`）：取号加暖机、A、B、C 与多记录流同一批节点，
/// 重挂与整理那几步与搬共享块流、生命周期闭环流共享入口名。登记表挂在标准前缀节点上的观察者（可写挂载续做）与挂在整理那几步节点上的两个观察者（读、可写挂载续做，`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）照表跑。
#[test]
fn crash_amplification_of_the_compaction_relocation_stream_is_clean() {
    let prepared = prepare("crash-amplification-compaction-relocation");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    common_compaction_crash_point_observers::amplify_a_compaction_flow(
        "compaction-relocation",
        &prepared,
        crash_points,
    );
}

/// 放量计划上整理那几步的观察者（`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）：计划里 A 之后、标准前缀之外的每一个点
/// 都挂着读与可写挂载续做两个观察者，别的点不挂；两个观察者各挂的点数等于另按这条流每一步的入口名数出来的点数，不是 0。只排计划、不判。
#[test]
fn crash_amplification_plan_of_the_compaction_relocation_stream_hangs_both_compaction_observers_on_every_point_after_the_standard_prefix(
) {
    let prepared = prepare("crash-amplification-compaction-relocation-observer-nodes");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    let points = common_compaction_crash_point_observers::assert_the_compaction_observers_sit_on_every_planned_point_after_the_standard_prefix(
        "compaction-relocation",
        &prepared,
        crash_points,
    );
    println!("CRASH_AMPLIFICATION_COMPACTION_OBSERVER_NODES flow=compaction-relocation points_with_both_observers={points}");
}
