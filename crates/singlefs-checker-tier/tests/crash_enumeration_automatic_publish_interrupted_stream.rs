//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 自动发布中途崩（判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」那一格：新加经会话、带攒批的崩溃流；
//! D16（发布语义） 已定项 2、10）：第一个文件之后崩了再可写挂载（第二个实例），会话接上待发（产品路径的阈值：`T_time` 5 s，`T_dirty` 夹成环长 ÷ F），
//! 钟由用例给：0 毫秒收下建一个 inode（不发）；6000 毫秒收下一次覆盖写第一个文件，收下之后时间那一支到了，在 `accept` 里自动发布（两次请求并成一次）；
//! 6100 毫秒再收下一次覆盖写（不发）；12000 毫秒调用方按周期调 `publish_when_due`，时间那一支又到了，自动发布第二次。录两次自动发布，
//! 按崩溃点重放的枚举域枚举它们的每个崩溃状态：每个状态跑恢复 + 多版本 oracle（第一个文件每一版的内容）+ 池级 checker + 记录核对器；
//! 另在每个状态上可写挂载一次（空间准入判着），挂完的盘过池级 checker。
//!
//! 一个请求不被触发切开：触发只在收下一个完整改动之后判，自动发布写的是收下的每一次请求的整份——崩在自动发布里面读出的只有发布之前、之后两样，
//! 没有一次覆盖写只落了一部分的版本（攻方 P5 那四格的前提不出现）。
//!
//! **比已有的流多罩了什么**：已有的流里发布都是调用方点名的（每次改动一次，或卸载、抬 F、整理），没有一次是会话按时间自己触发的；这一条第一次枚举
//! `accept` 里触发的那一次（建 inode 与覆盖写并成一次）与 `publish_when_due` 里触发的那一次，崩在两次自动发布之间读出第一次那一版、待发的那次覆盖写不出现。
//! 多跑的步是每个状态上的可写挂载（取号、写行、暖机）与挂完的盘上那一遍池级 checker。与攒了几个请求还没发布时崩那一条
//! （`crash_enumeration_batched_requests_not_yet_published_stream.rs`）起点相同，写表不同（那一条是一次立即排空），各立全量。
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
    "crates/singlefs-checker-tier/tests/crash_enumeration_automatic_publish_interrupted_stream.rs";

/// 这条流的名字（崩溃点重放项名第一个 `::` 之前那一截，来历表按它查）。
const FLOW_NAME: &str = "automatic_publish_interrupted";

/// 挂载之后观察者判干净时加的计数器名。
const MOUNTED_AND_CHECKER_CLEAN_COUNTER: &str = "mounted_and_checker_clean";

/// 几次调用交的读数（毫秒）：建 inode 在 0；覆盖写在 6000（离基线 6 s，过了 `T_time` 5 s，收下之后自动发布）；再一次覆盖写在 6100（离上次发布 0.1 s，不发）；
/// 按周期判在 12000（离上次发布 6 s，自动发布第二次）。
const READING_OF_THE_INODE_CREATION_IN_MILLISECONDS: u64 = 0;
const READING_OF_THE_OVERWRITE_TRIGGERING_IN_MILLISECONDS: u64 = 6000;
const READING_OF_THE_OVERWRITE_LEFT_PENDING_IN_MILLISECONDS: u64 = 6100;
const READING_OF_THE_PUBLISH_WHEN_DUE_IN_MILLISECONDS: u64 = 12_000;

/// 录好的两次自动发布：第一次之前的镜像（挂载之后、收下建 inode 之后——两者逐字节相同）、两次发布的写表与段、最后一次根槽写、每一版的第一个文件。
struct RecordedAutomaticPublishes {
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

/// 起池（第一个文件）、崩了再可写挂载、会话接上待发（产品路径的阈值），照用例的钟收下、触发两次自动发布；录两次发布。
fn record_the_two_automatic_publishes() -> RecordedAutomaticPublishes {
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
    let created = session
        .accept(
            &mut pool.devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 601,
            },
            reading(READING_OF_THE_INODE_CREATION_IN_MILLISECONDS),
        )
        .expect("收下建 inode");
    assert!(created.publishes.is_empty(), "基线读数上收下：不发");
    assert_eq!(
        stream.operation_count(),
        operations_before_the_accepts,
        "收下一个写都不发"
    );
    let triggering_content = content_of(OVERWRITE_BYTES, 602);
    let triggered = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &triggering_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 602,
            }),
            reading(READING_OF_THE_OVERWRITE_TRIGGERING_IN_MILLISECONDS),
        )
        .expect("收下覆盖写");
    assert_eq!(
        triggered.publishes.len(),
        1,
        "离基线 6 s、过了 T_time：收下之后自动发布一次"
    );
    pool.content_of_the_current_version = triggering_content;
    let root_of_the_first_automatic_publish = *session.session().current.root();
    pool.note_root_of_the_current_content(&root_of_the_first_automatic_publish);
    let pending_content = content_of(OVERWRITE_BYTES, 603);
    let left_pending = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &pending_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 603,
            }),
            reading(READING_OF_THE_OVERWRITE_LEFT_PENDING_IN_MILLISECONDS),
        )
        .expect("收下覆盖写");
    assert!(left_pending.publishes.is_empty(), "离上次发布 0.1 s：不发");
    let when_due = session
        .publish_when_due(
            &mut pool.devices,
            reading(READING_OF_THE_PUBLISH_WHEN_DUE_IN_MILLISECONDS),
        )
        .expect("按周期判")
        .expect("离上次发布 6 s：自动发布第二次");
    assert_eq!(when_due.published.versions_published.len(), 1);
    pool.content_of_the_current_version = pending_content;
    let root_of_the_second_automatic_publish = *session.session().current.root();
    pool.note_root_of_the_current_content(&root_of_the_second_automatic_publish);
    let recorded = stream.retained_operations()[operations_before_the_accepts..].to_vec();
    let (writes, segments) =
        writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_writes.len(), 2, "两次自动发布各一次根槽写");
    let last_root_write = *root_writes.last().expect("两次根槽写");
    let versions = pool.published_versions();
    RecordedAutomaticPublishes {
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
fn mount_writable_after_a_crash_in_the_automatic_publishes(
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
        Err(error) => panic!("自动发布途中崩溃之后可写挂载失败：{error:?}"),
    }
}

/// 按 `expansion` 枚举两次自动发布的崩溃状态，挂可写挂载那个观察者；只录那一趟交回 `None`。
fn enumerate_the_automatic_publishes(
    recorded: &RecordedAutomaticPublishes,
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
        mount_writable_after_a_crash_in_the_automatic_publishes(base, parameters, image)
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
        "恢复之后 oracle 0 违例：读出的是挂载那一版或两次自动发布的某一版，没有一次请求只落了一部分的版本"
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

/// 平时跑的那一份（甲二）：两次自动发布的每段原地写取它的几态、任意组合 × 单元写全不落或全落。
#[test]
fn crash_states_of_the_automatic_publishes_in_the_quick_tier_recover_a_published_version_and_remount_clean(
) {
    let recorded = record_the_two_automatic_publishes();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &quick_tier_expansion,
    );
    let Some(tally) =
        enumerate_the_automatic_publishes(&recorded, "quick_tier::fast", &quick_tier_expansion)
    else {
        return;
    };
    assert_eq!(tally.states, states, "甲二的状态数按崩溃点重放的口径另算");
    assert_every_state_recovers_a_published_version_and_remounts_clean(&tally, states);
}

/// 全量：两次自动发布的每一段都展开任意真子集（原地覆写取三态），每个状态恢复、oracle、池级 checker、记录核对器与一次可写挂载。
/// 判完打一行计数行 `CRASH_POINT_REPLAY_AUTOMATIC_PUBLISH_INTERRUPTED`（状态数等于每一段都展开时按崩溃点重放口径另算的闭式时带 `exhaustive=true`），
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:automatic-publish-interrupted`）的 `count-line=`、`exhaustive=`、`threads=` 都按它判。
#[test]
#[ignore = "崩溃枚举：用户要求时由主 agent 经门禁 checker-tier-crash-point-replay --full 按输入哈希跑（release），提交时的快档只核它的全绿标记，平时不跑"]
fn every_crash_state_of_the_automatic_publishes_recovers_a_published_version_and_remounts_clean() {
    let recorded = record_the_two_automatic_publishes();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_the_automatic_publishes(&recorded, "full", &full_expansion) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_AUTOMATIC_PUBLISH_INTERRUPTED states={} closed_form={states} exhaustive={} violations={} failed={} checker_violations={} mounted_and_checker_clean={} checker_by_invariant(evaluated/violated/not_applicable) {}",
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
