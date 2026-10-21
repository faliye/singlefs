//! 整理两条崩溃流与生命周期闭环流的两个观察者（里程碑四二·3、三·1；崩溃点重放那一路）：
//! - 读：恢复落到流里哪一版；那一版的活快照在崩溃镜像上读出建它那一刻的内容；再把那一版与它之前整理搬走的单元的源落点整槽抹成 0，
//!   第一个文件与活快照照样读出原来的内容（读路径照映射走到目的地，判决 G3「以映射为准」；判决 G6 必红世界「快照一侧读旧提示」在这一样上红）。只记计数。
//! - 可写挂载续做：每个状态上可写挂载一次（整理开关同录流），删除续做走到空，恢复落到的那一版有活的整理意图时整理续做推到意图删了为止，
//!   挂完 logged_ops 空、第一个文件与活快照读得回（`common_compaction_prefix::assert_the_mount_continued_the_compaction_and_the_deletions`）；
//!   交回挂完的盘，判段在它上面跑池级 checker（判干净时 [`POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER`] 加一）。
//!
//! 两个观察者另读的输入是这条流的每一版（`common_compaction_prefix::VersionOfTheCompactionFlow`）。三份测试文件调这里的同一份。
#![allow(
    dead_code,
    reason = "共用的观察者模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use super::common_compaction_prefix::{
    assert_every_live_snapshot_of_the_version_reads_its_creation_content,
    assert_the_first_file_of_the_version_reads_its_content,
    assert_the_mount_continued_the_compaction_and_the_deletions,
    image_with_the_relocation_sources_zeroed,
    mount_writable_with_the_compaction_switches_on_the_crash_disks,
    version_of_the_compaction_flow_landed_on, LandedVersionOfTheCompactionFlow,
    PreparedCompactionCrashFlow, VersionOfTheCompactionFlow,
    COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
};
use singlefs_checker_tier::crash::{CrashPointReplayObserverCounts, CrashPointReplayTally};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver,
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication, ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_state_enumeration::{
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::CrashImage;

/// 这一份的仓内路径（观察者代码文件，与 `file!()` 归一之后相同）：崩溃放量那一路两个观察者的版本从这一份出发（`judging_units_entries::OBSERVER_ENTRIES`）。
pub const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/common_compaction_crash_point_observers/mod.rs";

/// 判段在可写挂载续做之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
pub const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 读的那个观察者的名字。
pub const LANDED_VERSION_READ_THROUGH_THE_MAPPING: &str = "landed_version_read_through_the_mapping";

/// 可写挂载续做那个观察者的名字。
pub const MOUNT_CONTINUING_THE_COMPACTION: &str = "mount_continuing_the_compaction";

/// 恢复落到 A 之前的状态计数名。
pub const LANDED_BEFORE_THE_FIRST_FILE: &str = "landed_before_the_first_file";

/// 一条意图的整理续做至多发几次（签发之后的各批与最后一批；R 一段 64 槽、批大小同录流）。
const COMPACTION_CONTINUATION_PUBLISHES_AT_MOST: usize =
    64 / COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS;

/// 恢复落到流里那一版的计数名：`landed_on_<那一步的计数名>`，A 是 `landed_on_first_file`。
#[must_use]
pub fn landed_counter_of(version: &VersionOfTheCompactionFlow) -> String {
    format!(
        "landed_on_{}",
        version
            .kind
            .map_or("first_file", |kind| kind.counter_name())
    )
}

/// 读的那个观察者看一个状态（模块文件头第一样）。只读流里每一版，各工作线程共用一份。
pub fn observe_the_landed_version_reading_through_the_mapping(
    versions_of_the_flow: &[VersionOfTheCompactionFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let effective_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    let LandedVersionOfTheCompactionFlow::Version(version) =
        version_of_the_compaction_flow_landed_on(versions_of_the_flow, effective_root)
    else {
        counts.add(LANDED_BEFORE_THE_FIRST_FILE, 1);
        return;
    };
    counts.add(&landed_counter_of(version), 1);
    let snapshots_read = assert_every_live_snapshot_of_the_version_reads_its_creation_content(
        image,
        version,
        "崩溃镜像上",
    );
    counts.add(
        "live_snapshots_read_at_their_creation_content",
        u64::try_from(snapshots_read).expect("快照数装得进 u64"),
    );
    if version.relocation_sources_up_to_this_version.is_empty() {
        return;
    }
    let zeroed = image_with_the_relocation_sources_zeroed(
        &materialized_crash_image(image),
        &version.relocation_sources_up_to_this_version,
    );
    assert_the_first_file_of_the_version_reads_its_content(
        &zeroed,
        version,
        "整理搬走的源落点抹成 0 之后",
    );
    assert_every_live_snapshot_of_the_version_reads_its_creation_content(
        &zeroed,
        version,
        "整理搬走的源落点抹成 0 之后",
    );
    counts.add(
        "read_through_the_mapping_after_the_relocation_sources_are_zeroed",
        1,
    );
}

/// 可写挂载续做那个观察者看一个状态（模块文件头第二样）。可写挂载与续做真跑实现，只能在 CPU 上
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn observe_the_mount_continuing_the_compaction(
    versions_of_the_flow: &[VersionOfTheCompactionFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    let landed = version_of_the_compaction_flow_landed_on(versions_of_the_flow, effective_root);
    let mounted = mount_writable_with_the_compaction_switches_on_the_crash_disks(
        &materialized_crash_image(image),
    );
    let compaction_publishes =
        assert_the_mount_continued_the_compaction_and_the_deletions(landed, &mounted);
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

/// 崩溃点重放那一路的读观察者（另读的输入是流里每一版的摘要）。
pub fn landed_version_observer<'prepared>(
    prepared: &'prepared PreparedCompactionCrashFlow,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: LANDED_VERSION_READ_THROUGH_THE_MAPPING,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
    }
}

/// 崩溃点重放那一路的可写挂载续做观察者（交回挂完的盘）：录入段在 CPU 上挂、按恢复镜像的字节去重，判段在挂完的盘上判池级 checker。
pub fn mount_continuation_observer<'prepared>(
    prepared: &'prepared PreparedCompactionCrashFlow,
    observe: &'prepared ObserveACrashStateHandingBackAnImage<'prepared>,
) -> CrashPointReplayObserverHandingBackAnImage<'prepared> {
    CrashPointReplayObserverHandingBackAnImage {
        name: MOUNT_CONTINUING_THE_COMPACTION,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
    }
}

/// 两个观察者都看过每个状态：每个状态都落到流里某一版或 A 之前，流里每一种那一版都有状态（`landed_counters` 是这条流该有的那几个计数名）；
/// 落到的那一版搬过单元的每个状态都抹掉源落点读过一遍；每个状态都可写挂载过一次、挂完的盘都过了池级 checker。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_every_state_was_observed(
    prepared: &PreparedCompactionCrashFlow,
    tally: &CrashPointReplayTally,
    landed_counters: &[String],
) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        counts.count_named(LANDED_BEFORE_THE_FIRST_FILE)
            + landed_counters
                .iter()
                .map(|name| counts.count_named(name))
                .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版或 A 之前：{counts:?}"
    );
    for name in landed_counters {
        assert!(
            counts.count_named(name) > 0,
            "{name} 的状态真跑到了：{counts:?}"
        );
    }
    let landed_on_a_version_after_a_relocation: u64 = landed_counters
        .iter()
        .filter(|name| {
            prepared.versions_of_the_flow.iter().any(|version| {
                &landed_counter_of(version) == *name
                    && !version.relocation_sources_up_to_this_version.is_empty()
            })
        })
        .map(|name| counts.count_named(name))
        .sum();
    assert!(
        counts.count_named("read_through_the_mapping_after_the_relocation_sources_are_zeroed")
            >= landed_on_a_version_after_a_relocation.min(1),
        "落到整理搬过单元之后那几版的状态抹掉源落点读过：{counts:?}"
    );
    assert_eq!(
        (0..=COMPACTION_CONTINUATION_PUBLISHES_AT_MOST)
            .map(|publishes| counts
                .count_named(&format!("compaction_continuation_publishes_{publishes}")))
            .sum::<u64>(),
        tally.states,
        "每个状态都可写挂载过一次"
    );
    assert!(
        (1..=COMPACTION_CONTINUATION_PUBLISHES_AT_MOST)
            .map(|publishes| counts
                .count_named(&format!("compaction_continuation_publishes_{publishes}")))
            .sum::<u64>()
            > 0,
        "落到签发与各批那几版的状态上整理续做真发过：{counts:?}"
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "每个状态挂完的盘都过了池级 checker（判段在挂完的盘上判，配了 GPU 上卡）"
    );
}

// ============================== 崩溃放量那一路：挂在整理那几步节点上的两个观察者 ==============================

/// 挂在整理流与生命周期闭环流 A 之后那几步节点上的观察者（封闭的一张：加一种就加一个成员，[`amplify_a_compaction_flow`] 里的 `match` 跟着补）。
/// 代码就是上面两个，崩溃点重放那一路与崩溃放量那一路同一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverOfTheCompactionSteps {
    /// 模块文件头第一样：恢复落到的那一版照映射读，抹掉搬走的源落点再读一遍（只记计数）。
    LandedVersionReadThroughTheMapping,
    /// 模块文件头第二样：可写挂载一次、整理与删除续做走到底，挂完的盘过池级 checker。
    MountContinuingTheCompaction,
}

impl ObserverOfTheCompactionSteps {
    /// 观察者的名字：与崩溃点重放那一路的同一个观察者同名（[`LANDED_VERSION_READ_THROUGH_THE_MAPPING`]、[`MOUNT_CONTINUING_THE_COMPACTION`]）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::LandedVersionReadThroughTheMapping => LANDED_VERSION_READ_THROUGH_THE_MAPPING,
            Self::MountContinuingTheCompaction => MOUNT_CONTINUING_THE_COMPACTION,
        }
    }
}

/// 两个观察者都挂。
const EVERY_OBSERVER_OF_THE_COMPACTION_STEPS: &[ObserverOfTheCompactionSteps] = &[
    ObserverOfTheCompactionSteps::LandedVersionReadThroughTheMapping,
    ObserverOfTheCompactionSteps::MountContinuingTheCompaction,
];

/// 登记表：A 之后那几步的节点（入口名照 `common_compaction_prefix::CompactionFlowStepKind::entry_name`，设点都在 `common_crash_points`）→ 挂的观察者。
/// 标准前缀那几个节点（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`：取号加暖机、新池新建文件、B、C 两次顺序写）不在这里，挂的是可写挂载续做；
/// 三条流经过的其余每一步都在这里：建快照前那次覆盖写、快照那几步、重挂、整理签发、一批、最后一批、会话里删快照、抬 F、建空 inode。
/// [`amplify_a_compaction_flow`] 开跑前核这条流的每一步要么在标准前缀上、要么在这张表里，一步都不漏。
pub static OBSERVERS_ON_THE_COMPACTION_STEPS: &[(&str, &[ObserverOfTheCompactionSteps])] = &[
    ("publish_overwrite", EVERY_OBSERVER_OF_THE_COMPACTION_STEPS),
    (
        "publish_create_snapshot",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_overwrite_under_the_snapshot",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_unlink_intent_under_the_snapshot",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_unlink_step_under_the_snapshot",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_snapshot_destruction_intent",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_snapshot_destruction_step",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    ("mount_writable", EVERY_OBSERVER_OF_THE_COMPACTION_STEPS),
    (
        "publish_compaction_issue",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_compaction_batch",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_compaction_final_batch",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "destroy_snapshot_through_the_session",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "raise_rollback_floor_to_the_admission_ceiling",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
    (
        "publish_new_inodes_through_the_session",
        EVERY_OBSERVER_OF_THE_COMPACTION_STEPS,
    ),
];

/// 这条流 A 之后每一步的入口名，标准前缀上的那几个除外（B、C 两次顺序写挂的是标准前缀上的观察者）。
fn entry_names_of_the_steps_after_the_standard_prefix(
    prepared: &PreparedCompactionCrashFlow,
) -> Vec<&'static str> {
    use super::common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX;
    let mut entry_names: Vec<&'static str> = Vec::new();
    for (entry_name, _operations) in prepared.recorded.crash_point_entries_of_the_steps() {
        let on_the_standard_prefix = OBSERVERS_ON_THE_STANDARD_PREFIX
            .iter()
            .any(|(standard_entry_name, _observers)| *standard_entry_name == entry_name);
        if !on_the_standard_prefix && !entry_names.contains(&entry_name) {
            entry_names.push(entry_name);
        }
    }
    entry_names
}

/// 一个整理那几步的观察者挂在这条流的哪几个节点上：登记表里挂着它、这条流也经过的那几个入口（设点文件是 `common_crash_points`）。
///
/// # Panics
/// 这条流有一步既不在标准前缀上、也不在登记表里（那一步的节点就一个观察者都没挂）。
#[cfg(feature = "verdict-store")]
#[must_use]
pub fn nodes_of_the_compaction_steps_observer(
    prepared: &PreparedCompactionCrashFlow,
    observer: ObserverOfTheCompactionSteps,
) -> Vec<singlefs_checker_tier::crash_amplification::NodeOfAnObserver> {
    let entry_names = entry_names_of_the_steps_after_the_standard_prefix(prepared);
    for entry_name in &entry_names {
        assert!(
            OBSERVERS_ON_THE_COMPACTION_STEPS
                .iter()
                .any(|(registered_entry_name, _observers)| registered_entry_name == entry_name),
            "流里 A 之后的一步 {entry_name} 不在标准前缀上，也不在整理那几步的登记表里：它的节点一个观察者都没挂"
        );
    }
    OBSERVERS_ON_THE_COMPACTION_STEPS
        .iter()
        .filter(|(entry_name, observers_of_the_node)| {
            observers_of_the_node.contains(&observer) && entry_names.contains(entry_name)
        })
        .map(|(entry_name, _observers_of_the_node)| {
            singlefs_checker_tier::crash_amplification::NodeOfAnObserver {
                setting_file: super::common_crash_points::THIS_FILE.to_string(),
                entry_name: (*entry_name).to_string(),
            }
        })
        .collect()
}

/// 整理那几步的两个观察者在这条流的放量计划里挂上了 A 之后、标准前缀之外的每一个点，别的点一个都不挂：计划（`crash_amplification::plan_crash_points`）
/// 里每个点，「两个观察者的节点里有一个装得下它」与「它的入口名（名字里第一个 `.` 之前那一段）是这条流 A 之后不在标准前缀上的一步」逐点相同，
/// 两个观察者各自挂上的点数等于另按入口名数出来的数，而且不是 0。交回那个点数。
///
/// # Panics
/// 上面任何一样对不上。
#[cfg(feature = "verdict-store")]
pub fn assert_the_compaction_observers_sit_on_every_planned_point_after_the_standard_prefix(
    flow_name: &str,
    prepared: &PreparedCompactionCrashFlow,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    default_expand_up_to: usize,
) -> usize {
    use singlefs_checker_tier::crash_amplification::{
        plan_crash_points, CrashFlow, ExpandUpToWritesPerSegment,
    };
    let limit = ExpandUpToWritesPerSegment::from_environment_or(default_expand_up_to);
    let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &prepared.base,
        writes: &prepared.writes,
        segments: &prepared.segments,
        judged_root_index: prepared.judged_root_index,
        versions: &prepared.versions_with_a_file,
        versions_without_a_file: &prepared.versions_without_a_file,
        expansion: &expansion,
        crash_points,
    };
    let plan = plan_crash_points(&flow);
    let entry_names = entry_names_of_the_steps_after_the_standard_prefix(prepared);
    let points_after_the_standard_prefix = plan
        .points
        .iter()
        .filter(|point| {
            let entry_name = point.name.split('.').next().unwrap_or(point.name.as_str());
            point
                .identity
                .starts_with(super::common_crash_points::THIS_FILE)
                && entry_names.contains(&entry_name)
        })
        .count();
    assert!(
        points_after_the_standard_prefix > 0,
        "流 {flow_name} 在标准前缀之后有点（整理那几步）"
    );
    for observer in EVERY_OBSERVER_OF_THE_COMPACTION_STEPS {
        let nodes = nodes_of_the_compaction_steps_observer(prepared, *observer);
        let mut points_with_the_observer = 0usize;
        for point in &plan.points {
            let held = nodes.iter().any(|node| node.holds_the_point(point));
            let entry_name = point.name.split('.').next().unwrap_or(point.name.as_str());
            let after_the_standard_prefix = point
                .identity
                .starts_with(super::common_crash_points::THIS_FILE)
                && entry_names.contains(&entry_name);
            assert_eq!(
                held,
                after_the_standard_prefix,
                "流 {flow_name}：点 {} 挂没挂观察者 {} 与它在不在标准前缀之后的那几步上不一致",
                point.name,
                observer.name()
            );
            if held {
                points_with_the_observer += 1;
            }
        }
        assert_eq!(
            points_with_the_observer,
            points_after_the_standard_prefix,
            "流 {flow_name}：观察者 {} 挂上的点数等于标准前缀之后那几步的点数",
            observer.name()
        );
    }
    points_after_the_standard_prefix
}

/// 一条整理流（或生命周期闭环流）按环境跑一趟崩溃放量（展开上限、库目录、判器照 `common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line`），
/// 挂两张登记表上的观察者：标准前缀节点上的（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，可写挂载续做，代码在 `common_crash_point_observers`，
/// 另读删除流的发布表）与整理那几步节点上的（[`OBSERVERS_ON_THE_COMPACTION_STEPS`]，这一份的两个，另读这条流的每一版）。同快照流的
/// `common_snapshot_crash_point_observers::amplify_a_snapshot_flow`：观察者走两段，录入段在 CPU 上看（交回镜像的按恢复镜像去重），
/// 判段在挂完的盘上判池级 checker（配了 GPU 上卡）；两段各打一行。观察者计数的断言每一趟都判（这一趟拼的与库里复用的块读回来的合起来）：
/// 每个观察者至少看过一个状态、一个状态都不判红；两个可写挂载的观察者每个状态都挂过一次、挂完的盘都过了池级 checker。
/// 只录、只排派活计划、叫停、判段报本次未跑的那一趟不判。
///
/// # Panics
/// 这条流有一步既不在标准前缀上、也不在整理那几步的登记表里；或上面任何一条断言不成立。
#[cfg(feature = "verdict-store")]
pub fn amplify_a_compaction_flow(
    flow_name: &str,
    prepared: &PreparedCompactionCrashFlow,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    default_expand_up_to: usize,
) -> singlefs_checker_tier::crash_amplification::EnvironmentRun {
    use super::common_crash_point_observers;
    use super::common_crash_points::{
        self, ObserverOfTheStandardPrefix, DELETION_BATCHES_BEFORE_THE_FINAL_BATCH,
        OBSERVERS_ON_THE_STANDARD_PREFIX,
    };
    use singlefs_checker_tier::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverOnTheNodes,
    };
    use singlefs_checker_tier::crash_identity::ObserverName;
    let mount_continuation_inputs =
        common_crash_point_observers::mount_continuation_inputs_of_the_unlink_stream(&format!(
            "{flow_name}-observer-inputs"
        ));
    let observe_the_mount_on_the_standard_prefix =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            common_crash_point_observers::observe_the_mount_continuation(
                &mount_continuation_inputs,
                image,
                report,
                counts,
            )
        };
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe_the_landed_version =
        |_ordinal: u64,
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
    let observe_the_mount_continuing_the_compaction_of_the_flow =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_compaction(versions_of_the_flow, image, report, counts)
        };
    let observer_name = |name: &str| ObserverName::new(name).expect("观察者的名字非空、不长");
    let mut observers: Vec<ObserverOnTheNodes<'_>> = Vec::new();
    let standard_prefix_observer = ObserverOfTheStandardPrefix::WritableMountAndContinuation;
    observers.push(ObserverOnTheNodes {
        observer_name: observer_name(standard_prefix_observer.name()),
        observer_code_file: common_crash_point_observers::THIS_FILE.to_string(),
        extra_inputs_digest: mount_continuation_inputs.digest(),
        nodes: OBSERVERS_ON_THE_STANDARD_PREFIX
            .iter()
            .filter(|(_entry_name, observers_of_the_node)| {
                observers_of_the_node.contains(&standard_prefix_observer)
            })
            .map(|(entry_name, _observers_of_the_node)| NodeOfAnObserver {
                setting_file: common_crash_points::THIS_FILE.to_string(),
                entry_name: (*entry_name).to_string(),
            })
            .collect(),
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe_the_mount_on_the_standard_prefix,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
    });
    for compaction_observer in EVERY_OBSERVER_OF_THE_COMPACTION_STEPS {
        observers.push(ObserverOnTheNodes {
            observer_name: observer_name(compaction_observer.name()),
            observer_code_file: THIS_FILE.to_string(),
            extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
            nodes: nodes_of_the_compaction_steps_observer(prepared, *compaction_observer),
            observe: match compaction_observer {
                ObserverOfTheCompactionSteps::LandedVersionReadThroughTheMapping => {
                    ObserverCode::CountsOnly(&observe_the_landed_version)
                }
                ObserverOfTheCompactionSteps::MountContinuingTheCompaction => {
                    ObserverCode::HandsBackAnImage {
                        observe: &observe_the_mount_continuing_the_compaction_of_the_flow,
                        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                    }
                }
            },
        });
    }
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            flow_name,
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            prepared.judged_root_index,
            &prepared.versions_with_a_file,
            &prepared.versions_without_a_file,
            crash_points,
            default_expand_up_to,
            &observers,
        );
    if let Some(recording) = &run.observer_recording {
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_RECORDING flow={flow_name} images_observed_fresh={} states_reusing_an_observed_image={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={}",
            recording.images_observed_here,
            recording.states_reusing_an_observed_image,
            recording.blocks_recorded_here,
            recording.states_recorded_here,
            recording.blocks_already_recorded
        );
    }
    let Some(observing) = &run.observing else {
        return run;
    };
    println!(
        "CRASH_POINT_REPLAY_OBSERVERS flow={flow_name} images_judged_fresh={} images_judged_on_the_gpu_here={} image_judgements_reused={} blocks_observed_here={} states_observed_here={} states={:?} red={:?}",
        observing.images_judged_here,
        observing.images_judged_on_the_gpu_here,
        observing.image_judgements_reused,
        observing.blocks_observed_here,
        observing.states_observed_here,
        observing.states_of_each_observer,
        observing.red_states_of_each_observer
    );
    let observer_names = [
        standard_prefix_observer.name(),
        ObserverOfTheCompactionSteps::LandedVersionReadThroughTheMapping.name(),
        ObserverOfTheCompactionSteps::MountContinuingTheCompaction.name(),
    ];
    for name in observer_names {
        let states = observing
            .states_of_each_observer
            .get(name)
            .copied()
            .unwrap_or(0);
        assert!(
            states > 0,
            "流 {flow_name} 经过标准前缀与整理那几步，观察者 {name} 一个状态都没看"
        );
        assert_eq!(
            observing
                .red_states_of_each_observer
                .get(name)
                .copied()
                .unwrap_or(0),
            0,
            "流 {flow_name}：观察者 {name} 判红的状态（这一趟看出来的前几个）：{:?}",
            observing.red_texts
        );
        let counts = observing
            .counts_of_each_observer
            .get(name)
            .cloned()
            .unwrap_or_default();
        let count_named = |counter: &str| counts.get(counter).copied().unwrap_or(0);
        if name == standard_prefix_observer.name() {
            assert_eq!(
                (0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1)
                    .map(|steps| count_named(&format!("mount_continuation_steps_{steps}")))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：标准前缀上的每个状态都可写挂载过一次"
            );
        } else if name == ObserverOfTheCompactionSteps::MountContinuingTheCompaction.name() {
            assert_eq!(
                (0..=COMPACTION_CONTINUATION_PUBLISHES_AT_MOST)
                    .map(|publishes| count_named(&format!(
                        "compaction_continuation_publishes_{publishes}"
                    )))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：整理那几步上的每个状态都可写挂载过一次"
            );
        }
        if name != ObserverOfTheCompactionSteps::LandedVersionReadThroughTheMapping.name() {
            assert_eq!(
                count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
                states,
                "流 {flow_name}：观察者 {name} 看的每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
            );
        }
    }
    run
}
