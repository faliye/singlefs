//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! C492（树表 0 条的一版上写实例表行是误拒）在崩溃点重放上的那一条：固定走 R1 历史（2026-09-22 三方判决
//! `research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节 R1）——mkfs → 可写挂载（取号 1、不写行、零单元发布 txg 1 与暖机 txg 2）
//! → 不写文件 → 正常卸载（树表 0 条的一版上一个字节都不写）→ 进程重开、再可写挂载（取号 2、在树表 0 条的一版上写实例 1 那一行 txg 3、暖机 txg 4）。
//! 把 mkfs 之后的整条录制流按 D13（验证路线） 已定项 4 取崩溃状态，每个状态跑恢复 + 多版本 oracle（这条流上没有带文件的版本，只许报没有文件）、
//! 池级 checker、记录核对器；每个状态再当交回镜像的观察者重开可写挂载一次、接着发第一个文件版本，挂不上或发不出都当场 panic（C492 的验收：
//! 重开之后发得出布），挂完发完的盘交判段跑池级 checker。
//!
//! **比已有的流多罩了什么**：`crash_enumeration_writable_mount_of_a_formatted_pool.rs` 只罩「只做过 mkfs 的池重开一次可写挂载、同一个进程里发第一个
//! 文件版本」，那条流的写表与新池新建文件流逐项相同，崩溃状态上不重开挂载；这一条多罩第二次可写挂载那一串写（取号 2、树表 0 条的一版上写行那次
//! 十二个单元写、记录、根槽、轮换，与之后的暖机）的每个崩溃状态——树表 0 条的一版上写行在已有的崩溃点重放流里只有
//! `crash_enumeration_position_addressed_trees.rs` 那条 370 行的流（取号之后崩溃 369 次造出来的，只跑恢复）。
//! **多跑的一步**：每个崩溃状态之后重开可写挂载（取号、写行、暖机）再发第一个文件版本；已有的流在崩溃状态上不重开。
//! 与已有流的基线镜像相同（mkfs 之后），写表与段序列不同（第二次挂载那九段是新的），各自全量枚举。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    crash_state_devices, file_content, format_pool, geometry, memory_pool_of_sparse_devices,
    parameters, FIXED_WRITE_TIME_SECONDS,
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

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs";
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 观察者每个状态加一的计数：重开可写挂载做成、第一个文件版本发成。
const REMOUNTED_AND_PUBLISHED_THE_FIRST_FILE: &str = "remounted_and_published_the_first_file";
/// 判段在观察者交回的盘上判干净时加一的计数。
const CHECKER_CLEAN_AFTER_THE_REMOUNT_AND_THE_FIRST_FILE: &str =
    "checker_clean_after_the_remount_and_the_first_file";

/// 录好的 R1 历史：mkfs 之后的基线、整条录制流的写表与段、被判的根槽写（第二次挂载暖机那一次）。
struct RecordedFormattedPoolRemountedWithoutAFile {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
}

fn record_the_formatted_pool_remounted_without_a_file(
    tag: &str,
) -> RecordedFormattedPoolRemountedWithoutAFile {
    let mut formatted = format_pool(tag);

    let mut first_mount_devices = formatted.reopen_recorded();
    let mut first_mount =
        mount_writable(&parameters(), &mut first_mount_devices).expect("只做过 mkfs 的池可写挂载");
    assert_eq!(first_mount.output.instance, InstanceGeneration(1));
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：第一次可写挂载不写行"
    );
    // 不写文件，正常卸载：树表 0 条的一版上卸载一个字节都不写（`Unmounted::NothingWrittenOnAVersionWithoutFile`）。
    let unmounted = unmount(
        &parameters(),
        &mut first_mount_devices,
        &mut first_mount.allocator,
        &mut first_mount.current,
        ShadowLedger::On,
    )
    .expect("树表 0 条的一版上正常卸载");
    let Unmounted::NothingWrittenOnAVersionWithoutFile { current } = unmounted else {
        panic!("现行那一版树表 0 条：卸载不推空发布")
    };
    assert_eq!(
        (current.instance, current.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(2)),
        "卸载时现行那一版是第一次挂载的暖机根"
    );
    formatted.devices = Some(first_mount_devices);

    let mut second_mount_devices = formatted.reopen_recorded();
    let second_mount = mount_writable(&parameters(), &mut second_mount_devices)
        .expect("进程重开之后再可写挂载：树表 0 条的一版上写实例 1 那一行（R1，C492）");
    assert_eq!(second_mount.output.instance, InstanceGeneration(2));
    assert_eq!(
        second_mount.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
        }],
        "实例 1 那一行：选的根 txg 2、W = 0"
    );
    let PoolVersion::WithoutFile(_) = &second_mount.current else {
        panic!("写行与暖机之后现行那一版仍是树表 0 条的一版")
    };
    assert_eq!(
        second_mount.current.root().checkpoint_txg,
        CheckpointTxg(4),
        "写行 txg 3、暖机 txg 4"
    );
    formatted.devices = Some(second_mount_devices);

    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 2, 1, 2, 2, 1, 2, 2, 2, 12, 2, 1, 2, 2, 1, 2],
        "第一次挂载：取号两轮各两写（轮后各一道池屏障，各成一段）| 零单元发布的记录两写 | 根 | 系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写；\
         第二次挂载：取号两轮各两写 | 写行那次的十二个单元写（实例表一个单元、这一版自己那棵分配记录树五个节点，各两盘）| 记录两写 | 根 | \
         系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写"
    );
    assert_eq!(
        closed_form_state_count(&segments),
        1 + 8 * 3 + 4 * 3 + 4 + ((1 << 12) - 1),
        "每次写只取两态时：1 + 八个系统配置 2 写段（两次挂载各取号两轮、四次轮换）各 3 + 四个记录 2 写段各 3 + 四个根槽 1 写段各 1 + 12 写段 2^12 − 1"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 4, "txg 1、2、3、4 各一条根槽写");
    RecordedFormattedPoolRemountedWithoutAFile {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("四条根槽写"),
    }
}

/// 观察者「重开之后发第一个文件版本」（`remount_and_publish_the_first_file`）：把这个状态持久的写落到基线上，可写挂载一次、接着发第一个文件版本；
/// 挂不上、发不出都当场 panic（C492：树表 0 条的一版上必须有可写的出路，重开之后发得出布）。挂完发完的盘交回，池级 checker 在判段判。
/// 这条流上每个崩溃状态恢复都落在树表 0 条的一版上（没有带文件的版本），所以接着发的就是第一个文件版本。
/// 观察者的身份是这个 fn 的闭包键（入口要登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`；没登记时取键拒算、照判不入库）。
// gpu-exempt: writable-mount
fn remount_and_publish_the_first_file_after_the_crash(
    base: &MemoryPool,
    image: &CrashImage<'_>,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices = crash_state_devices(base, image.writes, &image.persisted, &stream);
    let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
        panic!("C492：崩溃之后重开可写挂载被拒（树表 0 条的一版上没有可写的出路）：{error:?}")
    });
    let mut allocator = mounted.allocator;
    let content = file_content();
    let published = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            mounted.current.record_bytes(),
        )
    }
    .unwrap_or_else(|error| {
        panic!("C492：崩溃之后重开可写挂载做成了，第一个文件版本发不出：{error:?}")
    });
    assert_eq!(
        published.root.instance, mounted.output.instance,
        "第一个文件版本是重开那次挂载取到的实例发的"
    );
    counts.add(REMOUNTED_AND_PUBLISHED_THE_FIRST_FILE, 1);
    let (writes_on_the_crash_image, _segments_of_the_remount_and_the_first_file) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer: memory_pool_of_sparse_devices(&devices),
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: CHECKER_CLEAN_AFTER_THE_REMOUNT_AND_THE_FIRST_FILE
            .to_string(),
    }
}

/// 按 `expansion` 枚举这条流，每个状态挂上重开再发第一个文件版本的观察者。只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）流写进判定库就交回 `None`。
fn enumerate_remounting_and_publishing_the_first_file(
    recorded: &RecordedFormattedPoolRemountedWithoutAFile,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        remount_and_publish_the_first_file_after_the_crash(&recorded.base, image, counts)
    };
    let item_name = format!("formatted_pool_remounted_without_a_file::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        // 这条流上没有带文件的版本：oracle 只许报没有文件。
        CrashPointReplayVersionTable::with_a_file_only(&[]),
        expansion,
        &[],
        // 观察者另读的输入：挂载与发布的参数、第一个文件版本的内容与时间——不随状态变，进版本与去重键时只取它们的摘要
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "remount_and_publish_the_first_file",
            extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                format!(
                    "parameters={:?}\nfirst_file_content_digest={:?}\nwrite_time_seconds={FIXED_WRITE_TIME_SECONDS}\n",
                    parameters(),
                    singlefs_harness::sha256::sha256_digest(&file_content()),
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
        "恢复之后多版本 oracle 0 违例（只许报没有文件）"
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
            .count_named(REMOUNTED_AND_PUBLISHED_THE_FIRST_FILE),
        tally.states,
        "C492：每个崩溃状态之后都重开得了可写挂载、发得出第一个文件版本"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT_AND_THE_FIRST_FILE),
        tally.states,
        "每个崩溃状态重开挂载、发完第一个文件版本之后的盘，池级 checker 判段一条都不判违例"
    );
}

/// 快档：整条流按甲二展开（系统配置槽写与记录写各取它的几态、任意组合，写行那次的十二个单元写只取全不落或全落），每个状态恢复、oracle、
/// 池级 checker、记录核对器，再重开可写挂载、发第一个文件版本。状态数 1 + 八个系统配置 2 写段（两次挂载各取号两轮、四次轮换）各 3² − 1（系统配置槽写是原地覆写、取三态） +
/// 四个记录 2 写段各 3 + 四个根槽 1 写段各 1 + 单元写段 1（全不落那一个；全落那一个算进下一段）= 82（取号写一轮时六个系统配置段、66）。
/// 判别力（`crates/mutations.tsv`「C492：树表 0 条的一版上写行在取号之前的预演里报拒」）：实现改回对树表 0 条那一版上的写行返回「无出路」，
/// 录这条流时第二次可写挂载就被拒，这条红。
#[test]
fn quick_tier_of_the_formatted_pool_remounted_without_a_file_remounts_and_publishes_the_first_file_in_every_crash_state(
) {
    let recorded = record_the_formatted_pool_remounted_without_a_file(
        "formatted-remounted-without-a-file-quick",
    );
    let Some(tally) = enumerate_remounting_and_publishing_the_first_file(
        &recorded,
        "quick_tier",
        &quick_tier_expansion,
    ) else {
        return;
    };
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &quick_tier_expansion,
    );
    assert_eq!(
        tally.states, states,
        "按甲二展开的状态数照崩溃点重放的口径另算"
    );
    assert_eq!(
        states,
        1 + 8 * (3 * 3 - 1) + 4 * 3 + 4 + 1,
        "1 + 八个系统配置 2 写段各 3² − 1 + 四个记录 2 写段各 3 + 四个根槽 1 写段各 1 + 单元写段只取全不落那一个 1"
    );
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
}

/// 全量：整条流每一段都展开（D13（验证路线） 已定项 4），闭式 1 + 八个系统配置 2 写段各 3² − 1 + 四个记录 2 写段各 3 + 四个根槽 1 写段各 1 +
/// 12 写段 2^12 − 1 = 4176 个状态（取号写一轮时 4160）；每个状态恢复、oracle、池级 checker、记录核对器，再重开可写挂载、发第一个文件版本。
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:formatted-pool-remounted-without-a-file-stream`），门禁
/// checker-tier-crash-point-replay `--full` 在 release 下跑，认下面打的 `CRASH_POINT_REPLAY_FORMATTED_POOL_REMOUNTED_WITHOUT_A_FILE` 行里 `exhaustive=true`。
/// 判别力（`crates/mutations.tsv`「C492 全量：树表 0 条的一版上写行那次真发报拒」）：取号之后写行那次真发返回拒绝，录这条流时第二次可写挂载就失败，这条红。
#[test]
#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:formatted-pool-remounted-without-a-file-stream，由 checker-tier-crash-point-replay --full 在 release 下跑"]
fn every_crash_state_of_the_formatted_pool_remounted_without_a_file_remounts_and_publishes_the_first_file(
) {
    let recorded = record_the_formatted_pool_remounted_without_a_file(
        "formatted-remounted-without-a-file-full",
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 8 * (3 * 3 - 1) + 4 * 3 + 4 + ((1 << 12) - 1),
        "闭式：每段 3^m · 2^(n−m) − 1 之和再加 1"
    );
    let Some(tally) =
        enumerate_remounting_and_publishing_the_first_file(&recorded, "full", &full_expansion)
    else {
        return;
    };
    // 门禁 checker-tier-crash-point-replay 只认行首 `CRASH_POINT_REPLAY_FORMATTED_POOL_REMOUNTED_WITHOUT_A_FILE ` 与 `exhaustive=true`，
    // 线程按 states= 找 CRASH_POINT_REPLAY_PARALLEL_FINISHED 那一行判；整行原样报出来。
    println!(
        "CRASH_POINT_REPLAY_FORMATTED_POOL_REMOUNTED_WITHOUT_A_FILE states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} no_file={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={} {REMOUNTED_AND_PUBLISHED_THE_FIRST_FILE}={} {CHECKER_CLEAN_AFTER_THE_REMOUNT_AND_THE_FIRST_FILE}={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.no_file_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_violated_states.values().sum::<u64>(),
        tally
            .observer_counts
            .count_named(REMOUNTED_AND_PUBLISHED_THE_FIRST_FILE),
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT_AND_THE_FIRST_FILE),
        tally.checker_counts_by_invariant()
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
}
