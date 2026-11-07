//! 验证包模块：crash、crash_state_enumeration、writable_mount_phases、crash_amplification
//! 删除与整理交错、回退跨过整理意图的崩溃流（里程碑四二·3，欠账 C621 那一条流；条款：D8（核心索引结构） 已定项 16 第 6 条续做、
//! 判决 `research/prompts/m4-trunc-r1-main-verification.md` K1「回退时整理条目照旧随现行」、D23（journal 的角色与格式） 已定项 14 管理员回退）：
//! 「取号 → 暖机 × 2 → A → 实例 1 里建 inode 2、顺序写它（两个数据单元）→ 重挂（取号、写行、暖机）→ 整理签发 → 一批（两个单元）
//! → 会话里 unlink inode 2（意图 → 一批 → 最后一批；意图与那一批两版里删除意图与整理意图同时活着）→ 管理员回退到签发之前那一版（重挂最后那次暖机）
//! → 整理一批 → 最后一批（同一次发布删意图）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 与批大小同整理搬迁流
//! （R 是第一个文件那次发布的提交内生块所在的段，建 inode 与顺序写的提交内生块也落在它里面；一批两个）。
//! 每个状态跑恢复 + 多版本 oracle（每一版都有第一个文件、内容不变）、池级 checker、记录核对器；另逐状态核两样（每个状态都核）：
//! - 读：恢复落到流里哪一版；那一版没人用的源落点整槽抹成 0 之后，沿它的根第一个文件读出它的内容、inode 2 读出这一版该读出的
//!   （还没建答没有文件、建了没写是空的、删除中答没有文件、回退之后回来了），读路径照映射走到目的地；
//! - 可写挂载一次（整理开关同录流）：删除续做先把队列走到空（落到 unlink 意图与那一批的状态上把 inode 2 删完），落到整理意图活着的那几版上
//!   整理续做推到意图删了、续做之后 R 按记录全空，落到别的版本不发；挂完 logged_ops 与删除队列都空、两个文件读出续做走完之后该读出的；
//!   挂完的盘在判段过池级 checker。
//!
//! 流的搭建住在公共模块 `crates/singlefs-verification/tests/common_compaction_deletion_prefix/mod.rs`（整理开关、R、挂载的写法借
//! `common_compaction_prefix`，不改它）；那一份在 `compaction_and_deletion_interleaved_across_a_rollback.rs` 里单独核录出来的形状与两张盘上的续做。
//!
//! **比已有的流多罩了什么**：已有的整理流（整理搬迁、搬共享块、生命周期闭环）上整理意图活着的时候没有删除意图，也没有回退；这条流第一次枚举
//! 删除意图与整理意图同时活着的那几版（unlink 意图那次与它的一批：`deleted_inodes` 与 logged_ops 都不空），崩在它们中间的每个状态上可写挂载
//! 先走删除续做、再走整理续做（判决 G8 的次序）；第一次枚举回退跨过一条活整理意图的那次发布（新根的 inode 树、extent 树与 `deleted_inodes`
//! 取签发之前那一版的，logged_ops 随现行，回退之前那一批搬走的单元复活在 R 里的源落点上、映射换回源落点）与回退之后整理重扫 R、把复活的单元
//! 连同剩下的一起搬走的两批。前十二段到 A 与新池新建文件流逐字节相同；重挂那一段同整理搬迁流的形态。多跑的步：逐状态抹掉源落点再沿根读两个文件、
//! 每个状态上的可写挂载续做（删除在前、整理在后）与挂完的盘上那一遍池级 checker。这条流上没有 discard。
//!
//! 每段都展开的那一条登记成崩溃枚举用例，可双机分片。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_compaction_deletion_prefix;
mod common_compaction_prefix;
mod common_snapshot_prefix;

use common_compaction_deletion_prefix::{
    assert_the_mount_continued_the_deletion_then_the_compaction,
    assert_the_version_reads_both_files, prepare_the_deletion_compaction_crash_flow,
    version_of_the_deletion_compaction_flow_landed_on, DeletionCompactionFlowStepKind,
    LandedVersionOfTheDeletionCompactionFlow, PreparedDeletionCompactionCrashFlow,
    PublishOfTheDeletionCompactionFlow, VersionOfTheDeletionCompactionFlow,
    INODE_OF_THE_OTHER_FILE,
};
use common_compaction_prefix::{
    image_with_the_relocation_sources_zeroed,
    mount_writable_with_the_compaction_switches_on_the_crash_disks, region_of_the_crash_flows,
    COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::logged_operations::live_compaction_intent_of_the_entries;
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver,
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication, ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, CrashImage,
};
use singlefs_verification::writable_mount_phases::materialized_crash_image;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取），也是两个观察者的代码文件。
const THIS_FILE: &str = "crates/singlefs-verification/tests/crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs";

/// 读的那个观察者的名字。
const LANDED_VERSION_READING_BOTH_FILES_THROUGH_THE_MAPPING: &str =
    "landed_version_reading_both_files_through_the_mapping";

/// 可写挂载续做那个观察者的名字。
const MOUNT_CONTINUING_THE_DELETION_AND_THE_COMPACTION: &str =
    "mount_continuing_the_deletion_and_the_compaction";

/// 恢复落到 A 之前的状态计数名。
const LANDED_BEFORE_THE_FIRST_FILE: &str = "landed_before_the_first_file";

/// 恢复落到删除意图与整理意图同时活着的那一版的状态计数名。
const LANDED_WITH_BOTH_INTENTS_LIVE: &str =
    "landed_with_the_deletion_and_the_compaction_intents_both_live";

/// 读的那个观察者抹过源落点再读的状态计数名。
const READ_AFTER_THE_RELOCATION_SOURCES_ARE_ZEROED: &str =
    "read_through_the_mapping_after_the_relocation_sources_are_zeroed";

/// 判段在可写挂载续做之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 一条意图的整理续做至多发几次（各批与最后一批；R 一段 64 槽、批大小同录流）。
const COMPACTION_CONTINUATION_PUBLISHES_AT_MOST: usize =
    64 / COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS;

/// 段形（录流按真写表数出来的，`compaction_and_deletion_interleaved_across_a_rollback.rs` 钉同一份）：前十二段到 A 同新池新建文件流；
/// 实例 1 里建 inode 20、顺序写 28 个单元写；重挂：取号两轮系统配置各 2、写行 18、暖机 16；签发 18、一批 20；unlink 意图 30、一批 26、最后一批 26；
/// 回退 24；回退之后一批 24、最后一批 24 个单元写，各自记录、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 62] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 4, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    18, 2, 1, 2, 20, 2, 1, 2, 30, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 24,
    2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（`memory_pool::closed_form_state_count_of_write_index_segments`）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 1_546_190_965;

fn prepare(tag: &str) -> PreparedDeletionCompactionCrashFlow {
    prepare_the_deletion_compaction_crash_flow(tag)
}

fn version_table(
    prepared: &PreparedDeletionCompactionCrashFlow,
) -> CrashPointReplayVersionTable<'_> {
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

/// 恢复落到流里那一版的计数名：`landed_on_<那一版的发布种类>`。
fn landed_counter_of(version: &VersionOfTheDeletionCompactionFlow) -> String {
    format!("landed_on_{}", version.publish.counter_name())
}

/// 流里每一种那一版的计数名，每一种都要有状态。
fn landed_counters(prepared: &PreparedDeletionCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 读的那个观察者看一个状态（文件头「读」那一样）。只读流里每一版，各工作线程共用一份。
fn observe_the_landed_version_reading_both_files_through_the_mapping(
    versions_of_the_flow: &[VersionOfTheDeletionCompactionFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let effective_root = report
        .effective_root
        .expect("每个状态都择到根（oracle 另判）");
    let LandedVersionOfTheDeletionCompactionFlow::Version(version) =
        version_of_the_deletion_compaction_flow_landed_on(versions_of_the_flow, effective_root)
    else {
        counts.add(LANDED_BEFORE_THE_FIRST_FILE, 1);
        return;
    };
    counts.add(&landed_counter_of(version), 1);
    if version.is_the_other_file_being_deleted && version.has_a_live_compaction_intent {
        counts.add(LANDED_WITH_BOTH_INTENTS_LIVE, 1);
    }
    if version.relocation_sources_unused_by_this_version.is_empty() {
        assert_the_version_reads_both_files(image, version, "崩溃镜像上");
        return;
    }
    let zeroed = image_with_the_relocation_sources_zeroed(
        &materialized_crash_image(image),
        &version.relocation_sources_unused_by_this_version,
    );
    assert_the_version_reads_both_files(&zeroed, version, "这一版没人用的源落点抹成 0 之后");
    counts.add(READ_AFTER_THE_RELOCATION_SOURCES_ARE_ZEROED, 1);
}

/// 可写挂载续做那个观察者看一个状态（文件头「可写挂载一次」那一样）。可写挂载与续做真跑实现，只能在 CPU 上
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
fn observe_the_mount_continuing_the_deletion_and_the_compaction(
    versions_of_the_flow: &[VersionOfTheDeletionCompactionFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("每个状态都择到根（oracle 另判）");
    let landed =
        version_of_the_deletion_compaction_flow_landed_on(versions_of_the_flow, effective_root);
    let mounted = mount_writable_with_the_compaction_switches_on_the_crash_disks(
        &materialized_crash_image(image),
    );
    let compaction_publishes =
        assert_the_mount_continued_the_deletion_then_the_compaction(landed, &mounted);
    counts.add(
        &format!("compaction_continuation_publishes_{compaction_publishes}"),
        1,
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

/// 录流的形状（判器判完之后再核）：步的次序；unlink 意图与不是最后一批的那几批两条意图同时活着；回退那一版整理意图还在、inode 2 回来了、
/// 删除队列空；整理搬的单元源落点都在 R 里、目的落点都在 R 外（R 排他）；最后一批同一次发布删意图、R 按记录全空；整条流没有 discard。
fn assert_the_shape_of_the_flow(prepared: &PreparedDeletionCompactionCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前十二段到 A 同新池新建文件流；之后建 inode、顺序写、重挂、签发、一批、unlink 三次、回退、两批"
    );
    let recorded = &prepared.recorded;
    assert_eq!(
        recorded
            .steps
            .iter()
            .map(|step| step.kind)
            .collect::<Vec<_>>(),
        vec![
            DeletionCompactionFlowStepKind::NewInodeInProcess,
            DeletionCompactionFlowStepKind::SequentialWriteOfTheOtherFileInProcess,
            DeletionCompactionFlowStepKind::MountWritable,
            DeletionCompactionFlowStepKind::CompactionIssue,
            DeletionCompactionFlowStepKind::CompactionBatch,
            DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession,
            DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue,
            DeletionCompactionFlowStepKind::CompactionBatch,
            DeletionCompactionFlowStepKind::CompactionFinalBatch,
        ],
        "建 inode → 顺序写 → 重挂 → 签发 → 一批 → unlink → 回退 → 一批 → 最后一批"
    );
    let unlink = recorded
        .steps_of(DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession)
        .first()
        .copied()
        .expect("unlink 那一步");
    assert_eq!(
        unlink
            .versions
            .iter()
            .map(|(publish, _output)| *publish)
            .collect::<Vec<_>>(),
        vec![
            PublishOfTheDeletionCompactionFlow::UnlinkIntent,
            PublishOfTheDeletionCompactionFlow::UnlinkBatch,
            PublishOfTheDeletionCompactionFlow::UnlinkFinalBatch,
        ],
        "两个数据单元、删除批大小 1：意图、一批、最后一批"
    );
    for (publish, output) in &unlink.versions[..2] {
        assert!(
            output.deleted_inodes.contains(&INODE_OF_THE_OTHER_FILE)
                && live_compaction_intent_of_the_entries(&output.logged_operations).is_some(),
            "{publish:?}：删除意图与整理意图同时活着"
        );
    }
    let rolled_back = &recorded
        .steps_of(DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue)
        .first()
        .copied()
        .expect("回退那一步")
        .versions[0]
        .1;
    assert!(
        live_compaction_intent_of_the_entries(&rolled_back.logged_operations).is_some()
            && rolled_back.deleted_inodes.is_empty()
            && rolled_back
                .files_other_than_the_first
                .iter()
                .any(|file| file.inode == INODE_OF_THE_OTHER_FILE),
        "回退那一版：整理意图随现行留着，删除队列与 inode 2 取签发之前那一版的"
    );
    let region = region_of_the_crash_flows();
    let relocated: Vec<_> = recorded
        .steps
        .iter()
        .flat_map(|step| step.versions.iter())
        .flat_map(|(_publish, output)| output.units_relocated_by_this_publish.iter())
        .collect();
    assert_eq!(
        relocated.len(),
        3 * COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
        "三批各搬两个单元（回退之前一批，回退之后复活的两个连同剩下的两个）"
    );
    for unit in relocated {
        assert!(
            unit.source
                .iter()
                .all(|location| region.contains_slot(location.slot)),
            "源落点在 R 里：{unit:?}"
        );
        assert!(
            unit.destination
                .iter()
                .all(|location| !region.contains_slot(location.slot)),
            "目的落点在 R 外（R 排他）：{unit:?}"
        );
    }
    let (_publish, final_batch) = recorded
        .steps
        .last()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert!(
        final_batch.logged_operations.is_empty()
            && region_is_empty_by_the_records(&final_batch.allocation_records, region),
        "最后一批同一次发布删意图、R 按记录全空"
    );
    assert_eq!(prepared.discards_in_the_stream(), 0, "这条流上没有 discard");
}

/// 崩溃点重放那一路的读观察者（另读的输入是流里每一版的摘要）。
fn landed_version_observer<'prepared>(
    prepared: &'prepared PreparedDeletionCompactionCrashFlow,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: LANDED_VERSION_READING_BOTH_FILES_THROUGH_THE_MAPPING,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
    }
}

/// 崩溃点重放那一路的可写挂载续做观察者（交回挂完的盘）：录入段在 CPU 上挂、按恢复镜像的字节去重，判段在挂完的盘上判池级 checker。
fn mount_continuation_observer<'prepared>(
    prepared: &'prepared PreparedDeletionCompactionCrashFlow,
    observe: &'prepared ObserveACrashStateHandingBackAnImage<'prepared>,
) -> CrashPointReplayObserverHandingBackAnImage<'prepared> {
    CrashPointReplayObserverHandingBackAnImage {
        name: MOUNT_CONTINUING_THE_DELETION_AND_THE_COMPACTION,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
    }
}

/// 两个观察者都看过每个状态：每个状态都落到流里某一版或 A 之前，流里每一种那一版都有状态；有状态落到两条意图同时活着的那几版上；
/// 落到搬过单元之后那几版的状态抹掉源落点读过；每个状态都可写挂载过一次、有状态上整理续做真发过、挂完的盘都过了池级 checker。
fn assert_every_state_was_observed(
    prepared: &PreparedDeletionCompactionCrashFlow,
    tally: &CrashPointReplayTally,
) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    let landed_counters = landed_counters(prepared);
    assert_eq!(
        counts.count_named(LANDED_BEFORE_THE_FIRST_FILE)
            + landed_counters
                .iter()
                .map(|name| counts.count_named(name))
                .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版或 A 之前：{counts:?}"
    );
    for name in &landed_counters {
        singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
            counts.count_named(name),
            &format!("{name} 的状态真跑到了：{counts:?}"),
        );
    }
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        counts.count_named(LANDED_WITH_BOTH_INTENTS_LIVE),
        &format!("落到删除意图与整理意图同时活着的那几版的状态真跑到了：{counts:?}"),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        counts.count_named(READ_AFTER_THE_RELOCATION_SOURCES_ARE_ZEROED),
        &format!("落到整理搬过单元之后那几版的状态抹掉源落点读过：{counts:?}"),
    );
    assert_eq!(
        (0..=COMPACTION_CONTINUATION_PUBLISHES_AT_MOST)
            .map(|publishes| counts
                .count_named(&format!("compaction_continuation_publishes_{publishes}")))
            .sum::<u64>(),
        tally.states,
        "每个状态都可写挂载过一次"
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        (1..=COMPACTION_CONTINUATION_PUBLISHES_AT_MOST)
            .map(|publishes| {
                counts.count_named(&format!("compaction_continuation_publishes_{publishes}"))
            })
            .sum::<u64>(),
        &format!("落到整理意图活着那几版的状态上整理续做真发过：{counts:?}"),
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "每个状态挂完的盘都过了池级 checker（判段在挂完的盘上判，配了 GPU 上卡）"
    );
}

/// 只展开写数小于 10 的段那一趟（测从库里重判计数相同）：单元段不展开，每个状态交两个观察者。
fn enumerate_the_segments_shorter_than_ten(
    prepared: &PreparedDeletionCompactionCrashFlow,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_both_files_through_the_mapping(
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
            observe_the_mount_continuing_the_deletion_and_the_compaction(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let item_name =
        format!("deletion_interleaved_with_compaction_across_a_rollback::segments_shorter_than_ten::{item_label}");
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
        &[landed_version_observer(prepared, &observe)],
        &[mount_continuation_observer(prepared, &observe_the_mount)],
    )
    .judged()
}

/// 复用不改判定：只展开写数小于 10 的段跑两遍，后跑的那一遍块与观察者结论照库里的（键拒算的那几组照判），两份计数逐项相同。
#[test]
fn the_deletion_interleaved_with_the_compaction_with_the_segments_shorter_than_ten_expanded_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("deletion-interleaved-with-compaction-crash-point-replay-rerun");
    let Some(run_filling_the_library) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun")
    else {
        return;
    };
    let Some(run_reusing_the_library) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun")
    else {
        return;
    };
    assert_eq!(
        run_reusing_the_library, run_filling_the_library,
        "后跑的那一遍照库里复用，计数逐项相同"
    );
    assert_eq!(
        run_filling_the_library.observed_states, run_filling_the_library.states,
        "每个状态都看过"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读两个文件，并在工作线程上可写挂载一次核删除续做与整理续做。登记成崩溃枚举用例，
/// crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_DELETION_INTERLEAVED_WITH_COMPACTION_ACROSS_A_ROLLBACK` 行里 `exhaustive=true`；可双机分片。
#[test]
fn full_enumeration_of_the_deletion_interleaved_with_compaction_across_a_rollback_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("deletion-interleaved-with-compaction-crash-point-replay-full");
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
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_both_files_through_the_mapping(
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
            observe_the_mount_continuing_the_deletion_and_the_compaction(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "deletion_interleaved_with_compaction_across_a_rollback::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[landed_version_observer(&prepared, &observe)],
        &[mount_continuation_observer(&prepared, &observe_the_mount)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_DELETION_INTERLEAVED_WITH_COMPACTION_ACROSS_A_ROLLBACK states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
        "deletion_interleaved_with_compaction_across_a_rollback_stream：用例自己的展开下的闭式",
    );
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&prepared, &tally);
}
