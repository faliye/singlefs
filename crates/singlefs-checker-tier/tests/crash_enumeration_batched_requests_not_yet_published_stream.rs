//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 攒了几个请求还没发布时崩（判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」那一格：新加经会话、带攒批的崩溃流；
//! D16（发布语义） 已定项 10）：第一个文件之后崩了再可写挂载（第二个实例），会话接上待发（产品路径的阈值：`T_time` 5 s，`T_dirty` 夹成环长 ÷ F），
//! 钟由用例给、走不到 5 s：收下覆盖写第一个文件、建两个 inode、再覆盖写第一个文件三次请求，一个写都不发；之后立即排空，三次请求并成一次发布
//! （同一 inode 的两次写折叠成后一次、建 inode 并在同一次发布里，`transaction::publish_data_changes_published_together`）。只录排空那一次发布，
//! 按崩溃点重放的枚举域枚举它的每个崩溃状态：每个状态跑恢复 + 多版本 oracle（第一个文件每一版的内容）+ 池级 checker + 记录核对器；
//! 另在每个状态上可写挂载一次（空间准入判着），挂完的盘过池级 checker。
//!
//! 收下的三次请求一个写都不发（录制流在收下前后步数相同，用例核）：崩在待发期间的状态就是排空那一次发布的第一个写之前的那一个——读出挂载那一版、
//! 待发的三次请求一个都不出现；排空那一次的根槽写持久之后读出并成的那一版（最后那次覆盖写的内容、两个新 inode），中间没有「只施加了一部分请求」的版本。
//!
//! **比已有的流多罩了什么**：已有的流里每次发布只装一个用户改动（覆盖写、顺序写、建 inode 各发各的）；这一条第一次枚举一次发布同时写一个文件的数据单元
//! 与新建 inode 的记录（inode 树那一片叶里改一条、追加两条，同一组记录点名两样）的单元段、记录段、根槽写与之后的系统配置槽轮换，且接在会话收下而没发布的
//! 三次请求之后。多跑的步是每个状态上的可写挂载（取号、写行、暖机）与挂完的盘上那一遍池级 checker。与自动发布中途崩那一条
//! （`crash_enumeration_automatic_publish_interrupted_stream.rs`）起点相同，写表不同（那一条是两次自动触发的发布），各立全量。
//!
//! 快档是甲二（每段原地写各取它的几态、任意组合 × 单元写全不落或全落）；全量标 ignored，登记成崩溃枚举用例。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;

use std::time::Duration;

use common_admission_pool::{
    content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, quick_tier_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::compaction_plan::CallerMonotonicClockReading;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::mounted_session::{MountedSessionWithPendingChanges, UserChange};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::FirstFile;
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_batched_requests_not_yet_published_stream.rs";

/// 这条流的名字（崩溃点重放项名第一个 `::` 之前那一截，来历表按它查）。
const FLOW_NAME: &str = "batched_requests_not_yet_published";

/// 挂载之后观察者判干净时加的计数器名。
const MOUNTED_AND_CHECKER_CLEAN_COUNTER: &str = "mounted_and_checker_clean";

/// 收下的三次请求各自交的读数（毫秒）与立即排空那一次的读数：都在 `T_time` 5 s 以内，三次收下一个都不触发。
const READINGS_OF_THE_ACCEPTS_IN_MILLISECONDS: [u64; 3] = [0, 100, 200];
const READING_OF_THE_DRAIN_IN_MILLISECONDS: u64 = 300;

/// 录好的那一次排空：排空之前的镜像（挂载之后、收下三次请求之后——两者逐字节相同）、排空那一次发布的写表与段、最后一次根槽写、每一版的第一个文件。
struct RecordedDrain {
    pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    last_root_write: usize,
    versions: Vec<PublishedVersion>,
}

fn reading(milliseconds: u64) -> CallerMonotonicClockReading {
    CallerMonotonicClockReading(Duration::from_millis(milliseconds))
}

/// 起池（第一个文件）、崩了再可写挂载、会话接上待发（产品路径的阈值），收下三次请求（不发），再立即排空；只录排空那一次。
fn record_the_drain_of_three_batched_requests() -> RecordedDrain {
    let stream = SharedStream::retaining_contents();
    let stream_for_the_devices = stream.clone();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        move |identity, device| {
            RecordingBlockDevice::with_shared_stream(
                identity,
                device,
                stream_for_the_devices.clone(),
            )
        },
    );
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    let mut session = MountedSessionWithPendingChanges::of_the_session(
        pool.session.take().expect("挂载之后的会话"),
    );
    let base = pool.image();
    let operations_before_the_accepts = stream.operation_count();
    let first_content = content_of(OVERWRITE_BYTES, 501);
    let last_content = content_of(OVERWRITE_BYTES, 503);
    let accepts = [
        UserChange::Overwrite(FirstFile {
            content: &first_content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 501,
        }),
        UserChange::NewInodes {
            count: 2,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 502,
        },
        UserChange::Overwrite(FirstFile {
            content: &last_content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 503,
        }),
    ];
    for (change, milliseconds) in accepts
        .into_iter()
        .zip(READINGS_OF_THE_ACCEPTS_IN_MILLISECONDS)
    {
        let accepted = session
            .accept(&mut pool.devices, change, reading(milliseconds))
            .expect("收下");
        assert!(
            accepted.publishes.is_empty(),
            "钟在 T_time 以内、脏量远在阈值下：收下不发"
        );
    }
    assert_eq!(
        stream.operation_count(),
        operations_before_the_accepts,
        "收下三次请求一个写都不发"
    );
    assert!(pool.image() == base, "收下之后盘上逐字节不变");
    let drained = session
        .publish_pending_now(
            &mut pool.devices,
            reading(READING_OF_THE_DRAIN_IN_MILLISECONDS),
        )
        .expect("立即排空")
        .expect("待发非空：发");
    assert_eq!(
        drained.published.versions_published.len(),
        1,
        "三次请求并成一次发布"
    );
    pool.content_of_the_current_version = last_content;
    let root = *session.session().current.root();
    pool.note_root_of_the_current_content(&root);
    let recorded = stream.retained_operations()[operations_before_the_accepts..].to_vec();
    let (writes, segments) =
        writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    let last_root_write = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .last()
        .expect("排空那一次写了一次根槽");
    let versions = pool.published_versions();
    RecordedDrain {
        pool,
        base,
        writes,
        segments,
        last_root_write,
        versions,
    }
}

/// 观察者「崩溃之后可写挂载」：把这个状态持久的写落到基线上、可写挂载一次（空间准入判着），挂不上当场 panic；挂完的盘交回，池级 checker 在判段判。
// gpu-exempt: writable-mount
fn mount_writable_after_a_crash_in_the_drain(
    base: &MemoryPool,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: &CrashImage<'_>,
) -> ImageAfterTheObserver {
    let mut materialized = base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            write.contents.apply_to(
                materialized
                    .devices
                    .get_mut(&write.device)
                    .expect("写落在池里的盘上"),
                write.offset,
            );
        }
    }
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = plain_devices_on(&materialized)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    match mount_writable_with_space_admission(
        parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(_) => {
            let disks_after_the_observer = MemoryPool {
                devices: devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect(),
                device_size_in_bytes: materialized.device_size_in_bytes,
            };
            let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            ImageAfterTheObserver {
                writes_on_the_crash_image,
                disks_after_the_observer,
                ranges_read_on_the_crash_image:
                    ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(&stream.read_ranges()),
                invariants_judged:
                    InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                counter_when_the_pool_checker_is_clean: MOUNTED_AND_CHECKER_CLEAN_COUNTER
                    .to_string(),
            }
        }
        Err(error) => panic!("排空途中崩溃之后可写挂载失败：{error:?}"),
    }
}

/// 按 `expansion` 枚举排空那一次的崩溃状态，挂可写挂载那个观察者；只录那一趟交回 `None`。
fn enumerate_the_drain(
    recorded: &RecordedDrain,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    // 观察者要在几个工作线程上共用：只借它读的那两样（基镜像与挂载参数），不借整份录制（盘里带着单线程的录制流）。
    let base = &recorded.base;
    let parameters = &recorded.pool.parameters;
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   _counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        mount_writable_after_a_crash_in_the_drain(base, parameters, image)
    };
    let item_name = format!("{FLOW_NAME}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.last_root_write,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[],
        // 观察者另读的输入：挂载参数（不随状态变，进版本与去重键时只取它的调试写法的摘要）
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "writable_mount_after_the_crash",
            extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                format!("{parameters:?}").as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

/// 判器先判、计数后核：恢复之后 oracle 与池级 checker 0 违例、每个状态都挂得上且挂完干净。
fn assert_every_state_recovers_a_published_version_and_remounts_clean(
    tally: &CrashPointReplayTally,
    states: u64,
) {
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后 oracle 0 违例：读出的是挂载那一版或并成的那一版，没有只施加了一部分请求的版本"
    );
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(MOUNTED_AND_CHECKER_CLEAN_COUNTER),
        states,
        "每个崩溃状态都挂得上、挂载之后池级 checker 0 违例"
    );
}

/// 平时跑的那一份（甲二）：排空那一次的每段原地写取它的几态、任意组合 × 单元写全不落或全落。
#[test]
fn crash_states_of_the_drain_of_batched_requests_in_the_quick_tier_recover_a_published_version_and_remount_clean(
) {
    let recorded = record_the_drain_of_three_batched_requests();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &quick_tier_expansion,
    );
    let Some(tally) = enumerate_the_drain(&recorded, "quick_tier::fast", &quick_tier_expansion)
    else {
        return;
    };
    assert_eq!(tally.states, states, "甲二的状态数按崩溃点重放的口径另算");
    assert_every_state_recovers_a_published_version_and_remounts_clean(&tally, states);
}

/// 全量：排空那一次的每一段都展开任意真子集（原地覆写取三态），每个状态恢复、oracle、池级 checker、记录核对器与一次可写挂载。
/// 判完打一行计数行 `CRASH_POINT_REPLAY_BATCHED_REQUESTS_NOT_YET_PUBLISHED`（状态数等于每一段都展开时按崩溃点重放口径另算的闭式时带 `exhaustive=true`），
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:batched-requests-not-yet-published`）的 `count-line=`、`exhaustive=`、`threads=` 都按它判。
#[test]
#[ignore = "崩溃枚举：用户要求时由主 agent 经门禁 checker-tier-crash-point-replay --full 按输入哈希跑（release），提交时的快档只核它的全绿标记，平时不跑"]
fn every_crash_state_of_the_drain_of_batched_requests_recovers_a_published_version_and_remounts_clean(
) {
    let recorded = record_the_drain_of_three_batched_requests();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_the_drain(&recorded, "full", &full_expansion) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_BATCHED_REQUESTS_NOT_YET_PUBLISHED states={} closed_form={states} exhaustive={} violations={} failed={} checker_violations={} mounted_and_checker_clean={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == states,
        tally.violations,
        tally.failed_states,
        tally.checker_violated_states.values().sum::<u64>(),
        tally
            .observer_counts
            .count_named(MOUNTED_AND_CHECKER_CLEAN_COUNTER),
        tally.checker_counts_by_invariant()
    );
    assert_eq!(
        tally.states, states,
        "每一段都展开，状态数与按崩溃点重放口径另算的相同（原地覆写三态、其余两态）"
    );
    assert_every_state_recovers_a_published_version_and_remounts_clean(&tally, states);
}
