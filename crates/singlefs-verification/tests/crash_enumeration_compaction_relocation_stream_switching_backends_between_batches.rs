//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 批间切 cpu ↔ gpu 的整理搬迁流（里程碑四七·4，D13（验证路线） 已定项 4；挂着切换照 D26（后台整理与放置回收） 已定项 13 第 1 条「下一批生效」）：
//! 同一负载、同一池，重挂选 `background_compute=cpu`、签发在 CPU 上算；签发之后经会话请求一次通过校验的切换到 `gpu`，那一批在 GPU 路径上算
//! （GPU 那一侧是验证包里背后照核心层 CPU 写法算的替身，不起真卡）；那一批之后再请求一次通过校验的切换回 `cpu`，最后一批在 CPU 上算——
//! 一条流来回切两次，两个方向各一次。录下的块层请求与全程 `cpu` 的整理搬迁流（`crash_enumeration_compaction_relocation_stream.rs` 枚举的那一条）
//! 逐字节相同；录法、比对与摆法在 `tests/common_compaction_relocation_on_the_background_compute/mod.rs`。
//! 这条流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器，另逐状态核两样（观察者同全程 `cpu`
//! 那一条，`tests/common_compaction_crash_point_observers/mod.rs`）：恢复落到的那一版照映射读、抹掉整理搬走的源落点再读；可写挂载一次（选 `cpu`，
//! 整理开关同录流）续做整理到意图删了，挂完的盘在判段过池级 checker。
//!
//! **比已有的流多罩了什么**：崩溃状态集合与全程 `cpu` 的整理搬迁流相同（基镜像、写表、段逐字节相同，块序列比对那一条用例钉住），观察者也是那一条的两个，
//! 不多罩崩溃状态、不多跑一步；多出来的只在录流那一边：一次整理里两次通过校验的批间切换，块序列与全程 `cpu` 的相同。
//!
//! 每段都展开，登记成崩溃枚举用例，带断点续跑、可双机分片。
//! 同一组崩溃状态另有一条在 CPU 判器上判的崩溃枚举用例（只开 CPU 的多机配置，先录进库、再从库读回判；两个观察者同上，挂完的盘同样在判段过池级 checker）：
//! 多机配置开着 GPU 时上面那一条的判段交 GPU 判器，这一条的判段一律交 CPU 判器；两条经同一个枚举函数录同一条流、挂同两个观察者，块键相同，
//! 库里有上面那一条判过的块就照块复用。改坏池级 checker 判反的变异行点名上面那一条（改 GPU 判器那一份），这一条的变异行在录入段之前就红。
//! 它不多罩崩溃状态，多跑的是 CPU 判器那一遍判。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_background_gpu_compute;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_compaction_relocation_on_the_background_compute;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_version_observer, mount_continuation_observer,
    observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction,
};
use common_compaction_prefix::{
    prepare_the_compaction_crash_flow, CompactionCrashFlow, PreparedCompactionCrashFlow,
};
use common_compaction_relocation_on_the_background_compute::{
    assert_clean, assert_every_compaction_publish_ran_on_the_planned_backend,
    assert_the_block_sequence_is_the_one_of_the_cpu_flow,
    assert_the_steps_publish_the_versions_of_the_cpu_flow, contents_of_the_sequential_writes,
    landed_counters, prepare_the_compaction_relocation_on_the_background_compute,
    record_the_compaction_relocation_on_the_background_compute, version_table,
    BackendsOfTheCompactionPublishes, PreparedOnTheBackgroundCompute,
    CLOSED_FORM_WITH_TWO_STATES_PER_WRITE, SEGMENTS_OF_THE_FLOW,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
};
use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
use singlefs_verification::crash_library::{
    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of, ExclusiveTemporaryDirectory,
    TheOneLibraryKeptForThisProcess,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, CrashImage,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_switching_backends_between_batches.rs";

/// 崩溃点重放的流名：项名第一个 `::` 之前那一截（来历表 `crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS` 按它查）。
const FLOW_NAME: &str = "compaction_relocation_switching_backends_between_batches";

fn prepare(tag: &str) -> PreparedOnTheBackgroundCompute {
    prepare_the_compaction_relocation_on_the_background_compute(
        tag,
        BackendsOfTheCompactionPublishes::CpuThenGpuForTheBatchThenCpuAgain,
    )
}

/// 按 `expansion` 枚举一趟，每个状态交两个观察者（照映射读；可写挂载续做，同全程 `cpu` 的整理搬迁流）。
fn enumerate_with_both_observers(
    prepared: &PreparedCompactionCrashFlow,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
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
}

/// 块序列比对（验收：批间来回切的写序列与全程 `cpu` 的那一条逐字节相同）：批间来回切录一条、全程 `cpu` 录一条，录下的块层请求逐字节相同，
/// 每一步发成的版本逐项相同；签发与最后一批在 CPU 上、那一批在 GPU 路径上（每一批开头读到的设置就是计划的后端），替身只在那一批被调过。
/// 必红世界（替身把一条记录的源槽算错）红在块序列比对那一条断言上。
#[test]
fn the_compaction_relocation_switching_from_cpu_to_gpu_and_back_between_batches_writes_the_block_sequence_of_the_cpu_flow_byte_for_byte(
) {
    let of_the_cpu = prepare_the_compaction_crash_flow(
        "compaction-relocation-block-sequence-of-the-cpu-flow-beside-the-backend-switches",
        CompactionCrashFlow::CompactionRelocation,
        &contents_of_the_sequential_writes(),
    );
    let switching = record_the_compaction_relocation_on_the_background_compute(
        "compaction-relocation-block-sequence-switching-backends-between-batches",
        BackendsOfTheCompactionPublishes::CpuThenGpuForTheBatchThenCpuAgain,
    );
    assert_the_block_sequence_is_the_one_of_the_cpu_flow(&switching.recorded, &of_the_cpu.recorded);
    assert_the_steps_publish_the_versions_of_the_cpu_flow(
        &switching.recorded,
        &of_the_cpu.recorded,
    );
    assert_every_compaction_publish_ran_on_the_planned_backend(
        switching.backends,
        &switching.compaction_publishes,
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并在工作线程上可写挂载一次核整理续做。登记成崩溃枚举用例，
/// crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_COMPACTION_RELOCATION_SWITCHING_BACKENDS_BETWEEN_BATCHES` 行里 `exhaustive=true`；
/// 带断点续跑、可双机分片。
#[test]
fn full_enumeration_of_the_compaction_relocation_stream_switching_backends_between_batches_is_exhaustive_and_clean(
) {
    let switching = prepare("compaction-relocation-switching-backends-crash-point-replay-full");
    let prepared = &switching.prepared;
    assert_eq!(
        prepared.segments.len(),
        SEGMENTS_OF_THE_FLOW,
        "段数同全程 cpu 的整理搬迁流"
    );
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
    let Some(tally) =
        enumerate_with_both_observers(prepared, &format!("{FLOW_NAME}::full"), &full_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATION_SWITCHING_BACKENDS_BETWEEN_BATCHES states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_relocation_stream_switching_backends_between_batches：用例自己的展开下的闭式",
    );
    assert_every_compaction_publish_ran_on_the_planned_backend(
        switching.backends,
        &switching.compaction_publishes,
    );
    assert_every_state_was_observed(prepared, &tally, &landed_counters(prepared));
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

/// 全量的 CPU 判器那一条：同一条流、同一组崩溃状态（每段都展开）、同两个观察者（照映射读；可写挂载续做，交回挂完的盘），
/// 在只开 CPU 的多机配置下先录进库、再从库读回，逐状态在 CPU 判器上判（两遍恢复 + 多版本 oracle、池级 checker、记录核对器；
/// 挂完的盘在判段同样交 CPU 判器过池级 checker）。签发与最后一批在 CPU 上、那一批在 GPU 路径上（替身在 CPU 上算）照录，只换判段。
/// 状态数等于闭式（缩过域照缩域的判），判器零红，观察者计数同上面那条全量。
/// 判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_RELOCATION_SWITCHING_BACKENDS_BETWEEN_BATCHES_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-relocation-switching-backends-between-batches-on-the-cpu-judge`）。
/// 与上面那条全量经同一个 [`enumerate_with_both_observers`]，只差判段交哪一个判器：块键相同，库里有上面那条判过的块就照块复用、不在 CPU 上重判。
/// 判别力（`crates/mutations.tsv`「批间切换整理搬迁流 CPU 判器那一条：环境守卫里换上的多机配置改回调用方那一份」）：换不上只开 CPU 的配置，
/// [`with_only_the_cpu_judge_configured`] 在录入段之前就红。池级 checker 判反的判别力由点名上面那条全量的变异行担（改 GPU 判器那一份）。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_of_the_compaction_relocation_stream_switching_backends_between_batches_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    let switching = prepare(
        "compaction-relocation-switching-backends-crash-point-replay-full-on-the-cpu-judge",
    );
    let prepared = &switching.prepared;
    assert_eq!(
        prepared.segments.len(),
        SEGMENTS_OF_THE_FLOW,
        "段数同全程 cpu 的整理搬迁流"
    );
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
    let Some(tally) = with_only_the_cpu_judge_configured(
        "compaction-relocation-switching-backends-on-the-cpu-judge",
        || {
            enumerate_with_both_observers(
                prepared,
                &format!("{FLOW_NAME}::full_on_the_cpu_judge"),
                &full_expansion,
            )
        },
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATION_SWITCHING_BACKENDS_BETWEEN_BATCHES_ON_THE_CPU_JUDGE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_relocation_stream_switching_backends_between_batches（CPU 判器）：用例自己的展开下的闭式",
    );
    assert_every_compaction_publish_ran_on_the_planned_backend(
        switching.backends,
        &switching.compaction_publishes,
    );
    assert_every_state_was_observed(prepared, &tally, &landed_counters(prepared));
}
