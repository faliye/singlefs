//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 取号写写到一半崩溃的结局（代码轮 m3-b44-code-r3 判决 `research/prompts/m3-b44-code-r3-main-verification.md`「推的、没造、进下一轮清单」那一句：
//! L1 ② 取号写写到一半崩溃会怎样，要在 checker 档枚举；`records/2026-10-13-代码欠账批.md`「收拢表（2026-10-18）」第 7 行）。
//!
//! 流：mkfs → 取号 1 → 暖机 → 第一个文件版本 A（同一个进程，`build_pool`）→ 进程退出、重开可写挂载（取号 2 → 写实例 1 那一行 → 暖机）。
//! 基线是重开之前的盘面（A 那次发布连同它的轮换全落），枚举的是重开那次可写挂载的写。取号写（D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」；
//! D22（单元原子性怎么合成） 已定项 8 落地之后写两轮：每轮逐盘写一次系统配置槽、轮后一道池屏障，取号之前另有一道）每轮自成一段，两块盘各一写；
//! 系统配置槽写是原地覆写，每写取三态（落了、没落、新旧都读不出），每段的真子集 3² − 1 = 8 个，两段 16 个，
//! 就是「取号写写到一半」的全部崩溃状态：第一轮一块盘落了一块没落、一块撕了一块没落、两块都撕……第一轮全落之后第二轮同样八种。
//! 每个状态再当交回镜像的观察者重开可写挂载一次，判结局：挂得上（取号之前那几道读核过得去）、取到的号比崩溃镜像上读得出的每一个实例代号都大
//! （D23 已定项 16「新号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1」：取号写只落了一块盘时，那块盘上已经是 2，
//! 重开要取 3，再取 2 就是重号）。实例代号由池级 checker 那一份读法读（`singlefs_checker::image` 的系统配置槽与根环），不用实现的读法。
//! 挂完的盘交判段跑池级 checker。
//!
//! **比已有的流多罩了什么**：取号那一段在 `crash_enumeration_acquisition_barrier.rs` 里也展开，那一条只跑恢复与 I-7.7，崩溃状态上不重开挂载；
//! 崩溃注入（`crash_injection_writable_mount_after_the_crash.rs`）在随机历史上抽样重开，不保证摆到取号那一段的每个组合。
//! **多跑的一步**：取号写写到一半的每个崩溃状态上（全量是重开那次挂载的每个崩溃状态上）重开可写挂载，判取号不重号。
//! 基线与写表与已有的流都不相同（`crash_enumeration_acquisition_barrier.rs` 重开之前多一次覆盖写 B），各自枚举。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, crash_state_devices, file_content, geometry, memory_pool_of_sparse_devices,
    parameters,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
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

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs";
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 重开之前那段历史只有实例 1（`build_pool` 取号 1）。
const INSTANCE_BEFORE_THE_REMOUNT: InstanceGeneration = InstanceGeneration(1);
/// 观察者每个状态加一的计数：重开可写挂载做成、取到的号比崩溃镜像上读得出的每一个实例代号都大。
const REMOUNTED_ABOVE_EVERY_INSTANCE_ON_THE_DISKS: &str =
    "remounted_above_every_instance_on_the_disks";
/// 崩溃镜像上读得出重开那次取到的号（取号写至少在一块盘上落成了）的状态数：这些状态上重开要越过它。
const ACQUIRED_INSTANCE_READABLE_ON_THE_CRASH_IMAGE: &str =
    "acquired_instance_readable_on_the_crash_image";
/// 判段在观察者交回的盘上判干净时加一的计数。
const CHECKER_CLEAN_AFTER_THE_REMOUNT: &str = "checker_clean_after_the_remount";
/// 重开那次挂载的录制流里取号写占的段数：取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地），每轮之后一道池屏障，各成一段，排在最前面。
const ACQUISITION_SEGMENTS: usize = 2;

/// 录好的重开：重开之前的基线、重开那次挂载的写表与段、被判的根槽写（暖机最后那一次）、允许的版本、取号那几写。
struct RecordedRemountAfterTheFirstFile {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    acquisition_writes: Vec<usize>,
}

fn record_the_remount_after_the_first_file(tag: &str) -> RecordedRemountAfterTheFirstFile {
    let mut pool = build_pool(tag);
    let base = pool.memory_pool();
    let operations_before_the_remount = pool.stream.operations().len();
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable(&parameters(), &mut devices).expect("第一个文件版本之后进程重开、可写挂载");
    pool.devices = Some(devices);
    assert_eq!(
        mounted.output.instance,
        InstanceGeneration(INSTANCE_BEFORE_THE_REMOUNT.0 + 1),
        "重开取号 2"
    );
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[operations_before_the_remount..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 18, 2, 1, 2, 16, 2, 1, 2],
        "取号第一轮两写 | 取号第二轮两写 | 写行那次的十八个单元写 | 记录两写 | 根 | 系统配置两写 | 暖机那次的十六个单元写 | 记录两写 | 根 | 系统配置两写"
    );
    assert_eq!(
        closed_form_state_count(&segments),
        1 + 4 * 3 + ((1 << 18) - 1) + 2 * 3 + 2 + ((1 << 16) - 1),
        "每次写只取两态时：1 + 四个系统配置 2 写段各 3 + 18 写段 2^18 − 1 + 两个记录 2 写段各 3 + 两个根槽 1 写段各 1 + 16 写段 2^16 − 1"
    );
    // 取号那几写：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写（`writable_mount_phases::WritableMountPhase::Acquisition` 同一个划法）。
    let acquisition_writes: Vec<usize> = writes
        .iter()
        .take_while(|write| write.kind == StepKind::SystemConfigurationSlot)
        .enumerate()
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        acquisition_writes
            .iter()
            .map(|index| writes[*index].device)
            .collect::<Vec<_>>(),
        vec![
            DeviceIdentity(0),
            DeviceIdentity(1),
            DeviceIdentity(0),
            DeviceIdentity(1)
        ],
        "取号写两轮，每轮逐盘写一次系统配置槽"
    );
    assert_eq!(
        segments[..ACQUISITION_SEGMENTS].concat(),
        acquisition_writes,
        "取号写之前一道屏障、每轮之后一道池屏障：取号两轮各两写、各自成一段（D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」）"
    );
    let filesystem_identifier = parameters().filesystem_identifier;
    let mut versions = vec![PublishedVersion {
        instance: INSTANCE_BEFORE_THE_REMOUNT,
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    for write in writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let root =
            RootRecord::parse_slot(write.bytes().expect("根槽写带字节"), &filesystem_identifier)
                .expect("重开那次挂载写的根自证得过");
        versions.push(PublishedVersion {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            content: file_content(),
        });
    }
    RecordedRemountAfterTheFirstFile {
        base,
        judged_root_index: writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("写行与暖机写了根槽"),
        writes,
        segments,
        versions,
        acquisition_writes,
    }
}

/// 盘上读得出的最大实例代号：每块盘两槽里可择的系统配置的实例代号与根环里自证过的根的实例代号取最大（池级 checker 那一份读法）。
fn highest_instance_readable_on(image: &MemoryPool) -> u32 {
    let slots = singlefs_checker::image::verified_system_configuration_slots(image);
    let geometry = slots
        .iter()
        .find_map(|(_, usable)| usable.first())
        .map(|(_, geometry)| geometry)
        .expect("取号写只覆写较旧那一槽：每块盘较新那一槽一直读得出，至少一块盘有可择的系统配置");
    let highest_in_the_slots = slots
        .iter()
        .flat_map(|(_, usable)| usable.iter().map(|(view, _)| view.journal_instance))
        .max()
        .expect("至少一个可择的槽");
    let highest_in_the_ring = singlefs_checker::image::valid_roots(image, geometry)
        .iter()
        .map(|(_, _, root)| root.instance)
        .max()
        .unwrap_or(0);
    highest_in_the_slots.max(highest_in_the_ring)
}

/// 观察者「取号写到一半之后重开」（`remount_after_the_crash`）：把这个状态持久的写落到基线上，可写挂载一次；挂不上当场 panic，
/// 取到的号不比崩溃镜像上读得出的每一个实例代号都大也当场 panic（重号）。挂完的盘交回，池级 checker 在判段判。
/// 观察者的身份是这个 fn 的闭包键（入口要登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`；没登记时取键拒算、照判不入库）。
// gpu-exempt: writable-mount
fn remount_after_the_crash_with_an_instance_above_every_one_on_the_disks(
    base: &MemoryPool,
    image: &CrashImage<'_>,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices = crash_state_devices(base, image.writes, &image.persisted, &stream);
    let highest_on_the_crash_image =
        highest_instance_readable_on(&memory_pool_of_sparse_devices(&devices));
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("取号写写到一半崩溃之后重开可写挂载被拒：{error:?}"));
    assert!(
        mounted.output.instance.0 > highest_on_the_crash_image,
        "重号：重开取到 {:?}，崩溃镜像上已经读得出实例 {highest_on_the_crash_image}（D23（journal 的角色与格式） 已定项 16 新号取盘上最大的加一）",
        mounted.output.instance
    );
    counts.add(REMOUNTED_ABOVE_EVERY_INSTANCE_ON_THE_DISKS, 1);
    if highest_on_the_crash_image > INSTANCE_BEFORE_THE_REMOUNT.0 {
        counts.add(ACQUIRED_INSTANCE_READABLE_ON_THE_CRASH_IMAGE, 1);
    }
    let (writes_on_the_crash_image, _segments_of_the_remount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer: memory_pool_of_sparse_devices(&devices),
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: CHECKER_CLEAN_AFTER_THE_REMOUNT.to_string(),
    }
}

/// 按 `expansion` 枚举重开那次挂载的写，每个状态挂上重开挂载的观察者。只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）流写进判定库就交回 `None`。
fn enumerate_remounting_after_each_crash(
    recorded: &RecordedRemountAfterTheFirstFile,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        remount_after_the_crash_with_an_instance_above_every_one_on_the_disks(
            &recorded.base,
            image,
            counts,
        )
    };
    let item_name = format!("crash_in_the_middle_of_the_acquisition_write::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[],
        // 观察者另读的输入：挂载的参数与重开之前那段历史的实例代号——不随状态变，进版本与去重键时只取它们的摘要
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "remount_after_the_crash",
            extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                format!(
                    "parameters={:?}\ninstance_before_the_remount={}\n",
                    parameters(),
                    INSTANCE_BEFORE_THE_REMOUNT.0
                )
                .as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

fn assert_recovery_checker_record_checker_and_remounts_clean(tally: &CrashPointReplayTally) {
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后多版本 oracle 0 违例"
    );
    assert_eq!(
        (
            tally.ignored_violations,
            tally.first_ignored_violation.clone()
        ),
        (0, None),
        "不看 journal 那一遍恢复同样过 oracle"
    );
    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(REMOUNTED_ABOVE_EVERY_INSTANCE_ON_THE_DISKS),
        tally.states,
        "每个崩溃状态之后都重开得了可写挂载，取到的号比盘上读得出的每一个都大"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT),
        tally.states,
        "每个崩溃状态重开挂载之后的盘，池级 checker 判段一条都不判违例"
    );
}

/// 快档：只展开取号那两段（两轮，每轮两块盘各一次系统配置槽写，各取三态），别的段只以整段持久进入后面的状态：1 + 2 × (3² − 1) = 17 个状态，
/// 前 16 个是取号写写到一半，最后一个是重开那次挂载全落（取号写一轮时只有一段、9 个）。每个状态恢复、oracle、池级 checker、记录核对器，
/// 再重开可写挂载、判取号不重号。
/// 判别力（`crates/mutations.tsv`「取号写到一半：取号只取每块盘两槽系统配置里实例代号的最小值」）：第一轮只落了一块盘时，盘上已有的 2
/// 被同盘另一槽上的 1 压住，重开又取 2，观察者判重号，这条红；不崩在取号写中途的历史上每块盘两槽的号都相同，这一改看不出来。
#[test]
fn every_crash_in_the_middle_of_the_acquisition_writes_remounts_with_an_instance_above_every_one_on_the_disks(
) {
    let recorded =
        record_the_remount_after_the_first_file("crash-in-the-middle-of-the-acquisition-quick");
    let expand_the_acquisition_segments_only = |segment_index: usize, _segment: &[usize]| {
        if segment_index < ACQUISITION_SEGMENTS {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    assert_eq!(
        recorded.segments[..ACQUISITION_SEGMENTS].concat(),
        recorded.acquisition_writes
    );
    let Some(tally) = enumerate_remounting_after_each_crash(
        &recorded,
        "acquisition_segment",
        &expand_the_acquisition_segments_only,
    ) else {
        return;
    };
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expand_the_acquisition_segments_only,
    );
    assert_eq!(
        tally.states, states,
        "只展开取号那两段的状态数照崩溃点重放的口径另算"
    );
    assert_eq!(
        states,
        1 + 2 * (3 * 3 - 1),
        "取号两轮各两写、各取三态（系统配置槽写是原地覆写），每轮的真子集 3² − 1，再加重开那次挂载全落那一个"
    );
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
    let readable = tally
        .observer_counts
        .count_named(ACQUIRED_INSTANCE_READABLE_ON_THE_CRASH_IMAGE);
    assert!(
        readable > 0 && readable < tally.states,
        "取号那两段里既有盘上读得出新号的状态（取号写至少在一块盘上落成），也有读不出的（第一轮都没落或都撕了）：{readable} / {}",
        tally.states
    );
}

/// 全量：重开那次挂载的每一段都展开（D13（验证路线） 已定项 4），闭式 1 + 四个系统配置 2 写段各 3² − 1 + 18 写段 2^18 − 1 + 两个记录 2 写段各 3 +
/// 两个根槽 1 写段各 1 + 16 写段 2^16 − 1 = 327719 个状态（取号写一轮时三个系统配置 2 写段、327711）；每个状态恢复、oracle、池级 checker、记录核对器，再重开可写挂载、判取号不重号
/// ——取号之后新号的单元已有落盘、根还没落的那些状态上，重开取号不重号最要紧。
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:crash-in-the-middle-of-the-acquisition-write-stream`），门禁
/// checker-tier-crash-point-replay `--full` 在 release 下跑，认下面打的 `CRASH_POINT_REPLAY_CRASH_IN_THE_MIDDLE_OF_THE_ACQUISITION_WRITE` 行里 `exhaustive=true`。
/// 判别力（`crates/mutations.tsv`「取号写到一半全量：取号的号只看根环、不看系统配置」）：取号写落了、根还没落的状态上重开又取 2，观察者判重号，这条红。
#[test]
#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:crash-in-the-middle-of-the-acquisition-write-stream，由 checker-tier-crash-point-replay --full 在 release 下跑"]
fn every_crash_state_of_the_remount_after_the_first_file_remounts_with_an_instance_above_every_one_on_the_disks(
) {
    let recorded =
        record_the_remount_after_the_first_file("crash-in-the-middle-of-the-acquisition-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 4 * (3 * 3 - 1) + ((1 << 18) - 1) + 2 * 3 + 2 + ((1 << 16) - 1),
        "闭式：每段 3^m · 2^(n−m) − 1 之和再加 1"
    );
    let Some(tally) = enumerate_remounting_after_each_crash(&recorded, "full", &full_expansion)
    else {
        return;
    };
    // 门禁 checker-tier-crash-point-replay 只认行首 `CRASH_POINT_REPLAY_CRASH_IN_THE_MIDDLE_OF_THE_ACQUISITION_WRITE ` 与 `exhaustive=true`，
    // 线程按 states= 找 CRASH_POINT_REPLAY_PARALLEL_FINISHED 那一行判；整行原样报出来。
    println!(
        "CRASH_POINT_REPLAY_CRASH_IN_THE_MIDDLE_OF_THE_ACQUISITION_WRITE states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={} {REMOUNTED_ABOVE_EVERY_INSTANCE_ON_THE_DISKS}={} {ACQUIRED_INSTANCE_READABLE_ON_THE_CRASH_IMAGE}={} {CHECKER_CLEAN_AFTER_THE_REMOUNT}={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_violated_states.values().sum::<u64>(),
        tally
            .observer_counts
            .count_named(REMOUNTED_ABOVE_EVERY_INSTANCE_ON_THE_DISKS),
        tally
            .observer_counts
            .count_named(ACQUIRED_INSTANCE_READABLE_ON_THE_CRASH_IMAGE),
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT),
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
}
