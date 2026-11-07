//! 验证包模块：crash、crash_state_enumeration、writable_mount_phases、crash_amplification
//! GPU 模式下的整理搬迁流（里程碑四七·4，D13（验证路线） 已定项 4；后台计算设置照 D26（后台整理与放置回收） 已定项 13 第 1 条）：
//! 同一负载、同一池，重挂选 `background_compute=gpu`，签发、一批、最后一批都在 GPU 路径上算（GPU 那一侧是验证包里背后照核心层 CPU 写法算的替身，
//! 不起真卡）。录下的块层请求与全程 `cpu` 的整理搬迁流（`crash_enumeration_compaction_relocation_stream.rs` 枚举的那一条）逐字节相同；
//! 录法、比对与摆法在 `tests/common_compaction_relocation_on_the_background_compute/mod.rs`。
//! 这条流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器，另逐状态核两样：
//! - 恢复落到的那一版照映射读、抹掉整理搬走的源落点再读（观察者同全程 `cpu` 那一条，`tests/common_compaction_crash_point_observers/mod.rs`）；
//! - 选 `gpu` 可写挂载一次（递进一份新的替身，整理开关同录流）：落到签发与那一批的状态上整理续做推到意图删了，续做的每一批都走 GPU 路径、不出错；
//!   挂完 logged_ops 空、第一个文件读得回；挂完的盘在判段过池级 checker（这一份的观察者）。
//!
//! **比已有的流多罩了什么**：崩溃状态集合与全程 `cpu` 的整理搬迁流相同（基镜像、写表、段逐字节相同，块序列比对那一条用例钉住），不多罩崩溃状态；
//! 多跑的一步是每个状态上选 `gpu` 的可写挂载续做（全程 `cpu` 那一条的续做选 `cpu`）。
//!
//! 每段都展开，登记成崩溃枚举用例，带断点续跑、可双机分片。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_background_gpu_compute;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_compaction_relocation_on_the_background_compute;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use common::{geometry, parameters};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, shared_cpu_backed_gpu_compute,
};
use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_version_observer,
    observe_the_landed_version_reading_through_the_mapping,
};
use common_compaction_prefix::{
    assert_the_mount_continued_the_compaction_and_the_deletions,
    compaction_switches_of_the_crash_flows, prepare_the_compaction_crash_flow,
    version_of_the_compaction_flow_landed_on, CompactionCrashFlow,
    MountedWithTheCompactionSwitchesAfterTheCrash, PreparedCompactionCrashFlow,
    VersionOfTheCompactionFlow,
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
use common_crash_point_observers::POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER;
use singlefs_core::background_compute::{
    BackgroundComputePathTaken, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches;
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
};
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver,
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication, ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, writes_and_segments, CrashImage, MemoryPool,
    SparseBlockDevice,
};
use singlefs_verification::writable_mount_phases::materialized_crash_image;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs";

/// 崩溃点重放的流名：项名第一个 `::` 之前那一截（来历表 `crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS` 按它查）。
const FLOW_NAME: &str = "compaction_relocation_on_the_gpu_path";

/// 选 `gpu` 的可写挂载续做那个观察者的名字。
const MOUNT_CONTINUING_THE_COMPACTION_ON_THE_GPU_PATH: &str =
    "mount_continuing_the_compaction_on_the_gpu_path";

/// 选 `gpu` 的可写挂载续做里走 GPU 路径的批数（逐状态加）：不是 0 才说明续做真在 GPU 路径上发过批。
const CONTINUATION_BATCHES_ON_THE_GPU_PATH_COUNTER: &str =
    "compaction_continuation_batches_on_the_gpu_path";

fn prepare(tag: &str) -> PreparedOnTheBackgroundCompute {
    prepare_the_compaction_relocation_on_the_background_compute(
        tag,
        BackendsOfTheCompactionPublishes::GpuOnEveryPublish,
    )
}

/// 在 `disks`（一张崩溃镜像落成的池）的副本上选 `gpu` 可写挂载一次：后台计算设置点名替身的两张卡、递进一份新的替身（不注入故障），
/// 整理开关同录流（`common_compaction_prefix::compaction_switches_of_the_crash_flows`），GC 停着时在 CPU 上补跑的阈值取默认。
/// 挂载发的写录在一条新的录制流上，连同挂完的盘交回。同 `common_compaction_prefix::mount_writable_with_the_compaction_switches_on_the_crash_disks`，
/// 只是挂载参数选 `gpu`（那一份在共用模块里、只挂 `cpu`）。
///
/// # Panics
/// 可写挂载做不成：这条流的每个崩溃状态按条款都挂得上。
fn mount_writable_on_the_gpu_path_on_the_crash_disks(
    disks: &MemoryPool,
) -> MountedWithTheCompactionSwitchesAfterTheCrash {
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
    let (_gpu_compute_kept_by_the_handle, gpu_compute_handle) = shared_cpu_backed_gpu_compute();
    let mounted =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            &mount_parameters,
            &mut devices,
            admissible_gpu_settings_naming_both_cards(),
            Some(gpu_compute_handle),
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            compaction_switches_of_the_crash_flows(),
        )
        .unwrap_or_else(|refused| {
            panic!("每个崩溃状态上选 gpu 的可写挂载都做得成：{refused:?}")
        });
    let disks_after_the_mount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: disks.device_size_in_bytes,
    };
    let (writes_of_the_mount, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    MountedWithTheCompactionSwitchesAfterTheCrash {
        mounted,
        writes_of_the_mount,
        disks_after_the_mount,
        read_ranges_of_the_mount: stream.read_ranges(),
    }
}

/// 挂载续做那一趟的每一批都走了 GPU 路径、没有一批算着出错：交回几批（落到的那一版没有活的整理意图、一趟都没续做的是 0）。
///
/// # Panics
/// 有一批没走 GPU 路径，或算着出了错。
fn continuation_batches_all_on_the_gpu_path(
    mounted: &MountedWithTheCompactionSwitchesAfterTheCrash,
) -> u64 {
    let Some(run) = mounted.mounted.output.compaction_continuation.run.as_ref() else {
        return 0;
    };
    for batch in &run.background_compute_of_each_batch {
        assert!(
            matches!(batch.path_taken, BackgroundComputePathTaken::Gpu(_))
                && batch.compute_error_that_abandoned_the_batch.is_none(),
            "选 gpu 的挂载续做每一批都走 GPU 路径、不出错：{batch:?}"
        );
    }
    u64::try_from(run.background_compute_of_each_batch.len()).expect("批数装得进 u64")
}

/// 选 `gpu` 的可写挂载续做那个观察者看一个状态（模块文件头第二样）。可写挂载与续做真跑实现（GPU 那三段经替身算），只能在 CPU 上
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
fn observe_the_mount_continuing_the_compaction_on_the_gpu_path(
    versions_of_the_flow: &[VersionOfTheCompactionFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("每个状态都择到根（oracle 另判）");
    let landed = version_of_the_compaction_flow_landed_on(versions_of_the_flow, effective_root);
    let mounted =
        mount_writable_on_the_gpu_path_on_the_crash_disks(&materialized_crash_image(image));
    let compaction_publishes =
        assert_the_mount_continued_the_compaction_and_the_deletions(landed, &mounted);
    let continuation_batches_on_the_gpu_path = continuation_batches_all_on_the_gpu_path(&mounted);
    counts.add(
        &format!("compaction_continuation_publishes_{compaction_publishes}"),
        1,
    );
    counts.add(
        CONTINUATION_BATCHES_ON_THE_GPU_PATH_COUNTER,
        continuation_batches_on_the_gpu_path,
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image: mounted.writes_of_the_mount,
        disks_after_the_observer: mounted.disks_after_the_mount,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &mounted.read_ranges_of_the_mount,
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER
            .to_string(),
    }
}

/// 崩溃点重放那一路选 `gpu` 的可写挂载续做观察者（交回挂完的盘）：录入段在 CPU 上挂、按恢复镜像的字节去重，判段在挂完的盘上判池级 checker。
fn mount_continuation_on_the_gpu_path_observer<'prepared>(
    prepared: &'prepared PreparedCompactionCrashFlow,
    observe: &'prepared ObserveACrashStateHandingBackAnImage<'prepared>,
) -> CrashPointReplayObserverHandingBackAnImage<'prepared> {
    CrashPointReplayObserverHandingBackAnImage {
        name: MOUNT_CONTINUING_THE_COMPACTION_ON_THE_GPU_PATH,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
    }
}

/// 按 `expansion` 枚举一趟，每个状态交两个观察者（照映射读；选 `gpu` 的可写挂载续做）。
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
            observe_the_mount_continuing_the_compaction_on_the_gpu_path(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
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
        &[mount_continuation_on_the_gpu_path_observer(
            prepared,
            &observe_the_mount,
        )],
    )
    .judged()
}

/// 块序列比对（验收：同一负载、同一池，两种模式录下的写序列逐字节相同）：全程 `gpu` 录一条、全程 `cpu` 录一条，录下的块层请求逐字节相同，
/// 每一步发成的版本逐项相同；`gpu` 那一条整理的三次发布每一批都走 GPU 路径、替身被调过。必红世界（替身把一条记录的源槽算错）红在块序列比对那一条断言上。
#[test]
fn the_compaction_relocation_on_the_gpu_path_writes_the_block_sequence_of_the_cpu_flow_byte_for_byte(
) {
    let of_the_cpu = prepare_the_compaction_crash_flow(
        "compaction-relocation-block-sequence-of-the-cpu-flow-beside-the-gpu-path",
        CompactionCrashFlow::CompactionRelocation,
        &contents_of_the_sequential_writes(),
    );
    let on_the_gpu_path = record_the_compaction_relocation_on_the_background_compute(
        "compaction-relocation-block-sequence-on-the-gpu-path",
        BackendsOfTheCompactionPublishes::GpuOnEveryPublish,
    );
    assert_the_block_sequence_is_the_one_of_the_cpu_flow(
        &on_the_gpu_path.recorded,
        &of_the_cpu.recorded,
    );
    assert_the_steps_publish_the_versions_of_the_cpu_flow(
        &on_the_gpu_path.recorded,
        &of_the_cpu.recorded,
    );
    assert_every_compaction_publish_ran_on_the_planned_backend(
        on_the_gpu_path.backends,
        &on_the_gpu_path.compaction_publishes,
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并在工作线程上选 `gpu` 可写挂载一次核整理续做。登记成崩溃枚举用例，
/// crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_COMPACTION_RELOCATION_ON_THE_GPU_PATH` 行里 `exhaustive=true`；带断点续跑、可双机分片。
#[test]
fn full_enumeration_of_the_compaction_relocation_stream_on_the_gpu_path_is_exhaustive_and_clean() {
    let on_the_gpu_path = prepare("compaction-relocation-on-the-gpu-path-crash-point-replay-full");
    let prepared = &on_the_gpu_path.prepared;
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
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATION_ON_THE_GPU_PATH states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "compaction_relocation_stream_on_the_gpu_path：用例自己的展开下的闭式",
    );
    assert_every_compaction_publish_ran_on_the_planned_backend(
        on_the_gpu_path.backends,
        &on_the_gpu_path.compaction_publishes,
    );
    assert_every_state_was_observed(prepared, &tally, &landed_counters(prepared));
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        tally
            .observer_counts
            .count_named(CONTINUATION_BATCHES_ON_THE_GPU_PATH_COUNTER),
        &format!(
            "落到签发与那一批的状态上，选 gpu 的挂载续做真在 GPU 路径上发过批：{:?}",
            tally.observer_counts
        ),
    );
}
