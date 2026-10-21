//! 多次 COW 的几条崩溃流（里程碑三一·1 的三条：同一文件跨两次关闭再可写挂载覆盖写、多个 inode 交错覆盖写、跨下段叶的大文件覆盖写；
//! 与判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M7 定的另开那一条多文件删除流的两支）共用的前缀与判法。
//! 几条流都接在多记录流「取号 → 暖机 × 2 → A → B → C」之后（前缀的点由 `common_crash_points::record_the_multi_record_publishes` 设，
//! 与多记录流、删除流、截断流共享到 C）；C 之后每一步一个入口、设点文件是这一份（录法在同目录的 `steps.rs`，它只用实现与 harness 档）：
//! 节点名 `<入口>.<周期>.<步>`，入口是 `steps::StepAfterTheMultiRecordPrefix::entry_name`。多 inode 那条与删除那两支在 C 之后还共用
//! 「建 149 个空 inode → 写 inode 150」两步（`steps::steps_through_the_write_outside_the_upper_leaf_of_the_first_file`），这两步的节点几条流共享。
//!
//! 逐状态挂两个观察者（崩溃点重放那一路与崩溃放量那一路同一份代码）：
//! - 落点（[`observe_the_landed_version`]，只记计数）：恢复落到流里哪一步发的那一版，或 A、B、C，或 A 之前；
//! - 可写挂载续做、读回每个文件（[`observe_the_mount_continuation_and_the_files_read_back`]，交回挂完的盘）：每个状态上可写挂载一次（批大小 1，同录流），
//!   续做把删除走完；读回第一个文件与这条流写过、删过的号，都等于落到的那一版续做走完之后的样子（正推第 6.2 节幂等判据 ③ 的取样点）；
//!   落到删除中的那几版时同一张崩溃镜像在另一份副本上再挂一次、挂完的盘逐字节相同（判据 ①b 的取样点）；判段在挂完的盘上判池级 checker
//!   （配了 GPU 上卡）。oracle 只读第一个文件，第一个文件之外的文件读回什么由这个观察者核。
#![allow(
    dead_code,
    reason = "几条流的测试文件各只用其中几样，没用的在那个测试二进制里就是没用到"
)]

pub mod steps;

use steps::{
    mount_continue_and_read_back_every_file, version_of_the_stream_at,
    MultipleCopyOnWriteCrashStream, PreparedMultipleCopyOnWriteCrashStream, VersionOfTheStream,
    CONTINUATION_REPEATED_BYTE_IDENTICAL_COUNTER, DELETION_CONTINUATION_PUBLISHES_AT_MOST,
    FILES_READ_BACK_AFTER_THE_CONTINUATION_COUNTER, LANDED_BEFORE_THE_FIRST_FILE_COUNTER,
    LANDED_ON_THE_MULTI_RECORD_PREFIX_COUNTER, MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER,
};

use super::common::geometry;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{writes_and_segments, CrashImage};

/// 这一份的仓内路径（与 `file!()` 归一之后相同）：C 之后每一步的设点文件，也是崩溃放量那一路两个观察者的代码文件。
pub const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/common_multiple_copy_on_write_crash_streams/mod.rs";

/// 落点观察者的名字（崩溃点重放与崩溃放量两路同名）。
pub const LANDED_VERSION_OBSERVER_NAME: &str = "landed_version";

/// 可写挂载续做、读回每个文件那个观察者的名字。
pub const MOUNT_CONTINUATION_OBSERVER_NAME: &str = "mount_continuation_and_files_read_back";

/// 判段在可写挂载之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
pub const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 快档展开的段：写数小于它的展开全部真子集（同别的流的快档）。
pub const QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW: usize = 10;

/// 全量展开的段：写数小于它的展开全部真子集。2^|段| 装得进 u64 的段（`singlefs_harness::memory_pool::closed_form_state_count` 的口径）
/// 都在里面；大文件那条流里那几次 144 个单元以上的发布的单元段与记录段（288–324 次写）不在：它们只以整段持久进入后面的状态，
/// 段内的子集由崩溃放量那一路按段登记表（`.claude/gate.d/crash-point-replay-segments.tsv`）罩。
pub const FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW: usize = 64;

/// oracle 认的版本表：第一个文件读得出的那几版，与从删第一个文件的意图那次起答没有文件的那几版。
#[must_use]
pub fn version_table(
    prepared: &PreparedMultipleCopyOnWriteCrashStream,
) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

/// 两个观察者另读的输入（流里每一版、各自续做之后每个要读的号读回什么）的摘要：逐版一行（根身份、计数名、续做还剩几次、每个号内容的 SHA-256），
/// 取 SHA-256。不带标签与手写的版本号：录法变了，变的是逐版的内容，摘要跟着变。
#[must_use]
pub fn digest_of_the_versions_of_the_stream(
    versions: &[VersionOfTheStream],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    let mut text = String::new();
    for version in versions {
        text.push_str(&format!(
            "version instance={} txg={} counter={} deletion_publishes_left={}",
            version.root.0 .0,
            version.root.1 .0,
            version.landed_counter_name,
            version.deletion_publishes_left
        ));
        for (inode, content) in &version.contents_after_the_continuation {
            text.push_str(&format!(
                " inode={inode}:{:?}",
                content
                    .as_ref()
                    .map(|bytes| singlefs_harness::sha256::sha256_digest(bytes))
            ));
        }
        text.push('\n');
    }
    singlefs_harness::sha256::sha256_digest(text.as_bytes())
}

/// 落点观察者（只记计数）：恢复落到流里那一版就给它的计数名加一（A、B、C 是 [`LANDED_ON_THE_MULTI_RECORD_PREFIX_COUNTER`]），
/// 落到 A 之前（mkfs 或两次暖机）加 [`LANDED_BEFORE_THE_FIRST_FILE_COUNTER`]。
// gpu-exempt: observer-assertions-on-the-recovery-report
pub fn observe_the_landed_version(
    versions: &[VersionOfTheStream],
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let counter = report
        .effective_root
        .and_then(|root| version_of_the_stream_at(versions, root))
        .map_or(LANDED_BEFORE_THE_FIRST_FILE_COUNTER, |version| {
            version.landed_counter_name
        });
    counts.add(counter, 1);
}

/// 可写挂载续做、读回每个文件那个观察者（模块文件头那一样）：在崩溃镜像落成的盘的副本上做 `steps::mount_continue_and_read_back_every_file`，
/// 它要加一的计数照加，交回挂载发的写、挂完的盘与读过的段：判段在挂完的盘上判池级 checker（判干净时 [`POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER`]
/// 加一；配了 GPU 上卡）。可写挂载与续做真跑实现，只能在 CPU 上（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
///
/// # Panics
/// 恢复没择到根（oracle 另判），或 `steps::mount_continue_and_read_back_every_file` 判出不对。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn observe_the_mount_continuation_and_the_files_read_back(
    versions: &[VersionOfTheStream],
    inodes_read_back: &[u64],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let landed_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    let materialized = materialized_crash_image(image);
    let mounted = mount_continue_and_read_back_every_file(
        versions,
        inodes_read_back,
        landed_root,
        &materialized,
    );
    for counter in &mounted.counters {
        counts.add(counter, 1);
    }
    let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
        &mounted.stream_of_the_mount.retained_operations(),
        &geometry(),
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer: mounted.disks_after_the_mount,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &mounted.stream_of_the_mount.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER
            .to_string(),
    }
}

/// 快档的展开：写数小于 [`QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW`] 的段任意真子集，别的段不展开（只以整段持久进入后面的状态）。
#[must_use]
pub fn quick_tier_expansion(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 全量的展开：写数小于 [`FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW`] 的段任意真子集（除大文件那条之外每条流就是每一段），别的段不展开。
#[must_use]
pub fn full_tier_expansion(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 判器判的几样一个都不红：oracle 两遍、走读、记录核对器两条判据、池级 checker 的每一条不变量。
pub fn assert_clean(stream: MultipleCopyOnWriteCrashStream, tally: &CrashPointReplayTally) {
    assert_eq!(
        tally.violations, 0,
        "{stream:?} oracle：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "{stream:?} 不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "{stream:?} 走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "{stream:?} 记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{stream:?} {invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 两个观察者把每个状态都看过：落点那几个计数加起来、续做次数那几个计数加起来各等于状态数，挂完的盘每个都过了池级 checker；
/// 读回每个文件的与落到 A 之前的加起来是状态数；流里每一步发的那几版都有状态落到（每个计数名都不是 0）；落到删除中那几版的每个状态都挂了两份、
/// 两份逐字节相同（`deletion_continuation_publishes_1..` 之和等于判据 ①b 那个计数），流里有删除的那几条这个数不是 0。
pub fn assert_every_state_was_observed(
    prepared: &PreparedMultipleCopyOnWriteCrashStream,
    tally: &CrashPointReplayTally,
) {
    let stream = prepared.recorded.stream;
    let counts = &tally.observer_counts;
    assert_eq!(
        tally.observed_states, tally.states,
        "{stream:?}：每个状态都看过"
    );
    let mut landed_counters: Vec<&'static str> = vec![LANDED_BEFORE_THE_FIRST_FILE_COUNTER];
    for version in &prepared.versions_of_the_stream {
        if !landed_counters.contains(&version.landed_counter_name) {
            landed_counters.push(version.landed_counter_name);
        }
    }
    assert_eq!(
        landed_counters
            .iter()
            .map(|counter| counts.count_named(counter))
            .sum::<u64>(),
        tally.states,
        "{stream:?}：每个状态都落到流里某一版或 A 之前：{counts:?}"
    );
    for counter in &landed_counters {
        assert!(
            counts.count_named(counter) > 0,
            "{stream:?}：{counter} 的状态真跑到了：{counts:?}"
        );
    }
    assert!(
        counts.count_named(LANDED_ON_THE_MULTI_RECORD_PREFIX_COUNTER) > 0,
        "{stream:?}：落到 A、B、C 的状态真跑到了"
    );
    assert_eq!(
        (0..=DELETION_CONTINUATION_PUBLISHES_AT_MOST)
            .map(|publishes| counts.count_named(&steps::deletion_continuation_counter(publishes)))
            .sum::<u64>(),
        tally.states,
        "{stream:?}：每个状态都可写挂载过一次"
    );
    assert_eq!(
        counts.count_named(FILES_READ_BACK_AFTER_THE_CONTINUATION_COUNTER)
            + counts.count_named(MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER),
        tally.states,
        "{stream:?}：落到 A 起的每个状态都读回了每个文件"
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "{stream:?}：每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
    );
    let states_landed_while_deleting = (1..=DELETION_CONTINUATION_PUBLISHES_AT_MOST)
        .map(|publishes| counts.count_named(&steps::deletion_continuation_counter(publishes)))
        .sum::<u64>();
    assert_eq!(
        counts.count_named(CONTINUATION_REPEATED_BYTE_IDENTICAL_COUNTER),
        states_landed_while_deleting,
        "{stream:?}：落到删除中那几版的每个状态都在两份副本上续做、挂完逐字节相同"
    );
    let stream_deletes = prepared
        .versions_of_the_stream
        .iter()
        .any(|version| version.deletion_publishes_left > 0);
    assert_eq!(
        states_landed_while_deleting > 0,
        stream_deletes,
        "{stream:?}：有删除的流落到删除中那几版的状态真跑到了，没有删除的一个都没有"
    );
}

/// 展开的段按崩溃点重放的枚举域数（原地覆写取三态）。
#[must_use]
pub fn crash_state_count_under(
    prepared: &PreparedMultipleCopyOnWriteCrashStream,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        expansion,
    )
}

/// 不展开的段（全量里只以整段持久进入后面的状态的那几段）：`段号:写数`，用逗号接；一段都没有写 `-`。
#[must_use]
pub fn segments_left_whole_text(prepared: &PreparedMultipleCopyOnWriteCrashStream) -> String {
    let left_whole: Vec<String> = prepared
        .segments
        .iter()
        .enumerate()
        .filter(|(_segment_index, segment)| {
            segment.len() >= FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW
        })
        .map(|(segment_index, segment)| format!("{segment_index}:{}", segment.len()))
        .collect();
    if left_whole.is_empty() {
        "-".to_string()
    } else {
        left_whole.join(",")
    }
}

// ============================== 崩溃放量那一路（录制器设点、挂观察者） ==============================

/// 录制器那一份造法录下的一条流：起点镜像、写表、段、崩溃点、被判的根槽写与两样版本表，连同崩溃点重放那一路同一套「流里每一版」。
#[cfg(feature = "verdict-store")]
pub struct RecordedThroughTheCrashPointRecorder {
    pub crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    pub prepared: PreparedMultipleCopyOnWriteCrashStream,
}

/// 录制器那一份造法：取号加暖机、新池新建文件、B、C 四个点由 `common_crash_points` 设（与多记录流、删除流、截断流共享到 C），
/// C 之后每一步各一个点，设在这一份里（入口名 `steps::StepAfterTheMultiRecordPrefix::entry_name`）。
#[cfg(feature = "verdict-store")]
#[must_use]
pub fn record_through_the_crash_point_recorder(
    tag: &str,
    stream: MultipleCopyOnWriteCrashStream,
) -> RecordedThroughTheCrashPointRecorder {
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = super::common::build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    let publishes =
        super::common_crash_points::record_the_multi_record_publishes(&mut recorder, &mut pool);
    let prefix = steps::MultiRecordPrefixOfTheStream::of_the_three_publishes(
        publishes.first,
        publishes.second,
        publishes.second_content,
        publishes.third,
        publishes.third_content,
    );
    let recorded =
        steps::record_the_steps_after_the_prefix(stream, pool, prefix, |step, pool, instance| {
            recorder.step(file!(), step.entry_name(), || step.publish(pool, instance))
        });
    let prepared = steps::prepared_from_the_recorded_stream(recorded);
    let crash_points = recorder.into_crash_points(
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    RecordedThroughTheCrashPointRecorder {
        crash_points,
        prepared,
    }
}

/// 挂在 C 之后那几步节点上的观察者（封闭的一张：加一种就加一个成员，[`amplify_a_multiple_copy_on_write_flow`] 里的 `match` 跟着补）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverOfTheStepsAfterThePrefix {
    /// 恢复落到流里哪一版（只记计数，[`observe_the_landed_version`]）。
    LandedVersion,
    /// 可写挂载一次、续做走完、读回每个文件，挂完的盘过池级 checker（[`observe_the_mount_continuation_and_the_files_read_back`]）。
    MountContinuationAndFilesReadBack,
}

impl ObserverOfTheStepsAfterThePrefix {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::LandedVersion => LANDED_VERSION_OBSERVER_NAME,
            Self::MountContinuationAndFilesReadBack => MOUNT_CONTINUATION_OBSERVER_NAME,
        }
    }
}

/// 两个观察者都挂。
const EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX: &[ObserverOfTheStepsAfterThePrefix] = &[
    ObserverOfTheStepsAfterThePrefix::LandedVersion,
    ObserverOfTheStepsAfterThePrefix::MountContinuationAndFilesReadBack,
];

/// 登记表：C 之后那几步的节点（入口名照 `steps::StepAfterTheMultiRecordPrefix::entry_name`，设点文件是这一份）→ 挂的观察者。
/// 标准前缀那几个节点（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`）不在这里。[`amplify_a_multiple_copy_on_write_flow`] 开跑前核
/// 这条流 C 之后的每一步都在这张表里。
pub static OBSERVERS_ON_THE_STEPS_AFTER_THE_PREFIX: &[(
    &str,
    &[ObserverOfTheStepsAfterThePrefix],
)] = &[
    (
        "mount_writable",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
    (
        "publish_new_inodes",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
    (
        "publish_overwrite_of_an_inode",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
    (
        "publish_sequential_write_of_an_inode",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
    (
        "publish_unlink_intent_of_an_inode",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
    (
        "publish_unlink_step_of_an_inode",
        EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX,
    ),
];

/// 一个观察者挂在这条流的哪几个节点上：登记表里挂着它、这条流 C 之后也经过的那几个入口（设点文件是这一份）。
///
/// # Panics
/// 这条流 C 之后有一步不在登记表里（那一步的节点就一个观察者都没挂）。
#[cfg(feature = "verdict-store")]
#[must_use]
pub fn nodes_of_the_observer(
    stream: MultipleCopyOnWriteCrashStream,
    observer: ObserverOfTheStepsAfterThePrefix,
) -> Vec<singlefs_checker_tier::crash_amplification::NodeOfAnObserver> {
    let mut entry_names: Vec<&'static str> = Vec::new();
    for step in stream.steps() {
        if !entry_names.contains(&step.entry_name()) {
            entry_names.push(step.entry_name());
        }
    }
    for entry_name in &entry_names {
        assert!(
            OBSERVERS_ON_THE_STEPS_AFTER_THE_PREFIX
                .iter()
                .any(|(registered_entry_name, _observers)| registered_entry_name == entry_name),
            "流 {stream:?} C 之后的一步 {entry_name} 不在登记表里：它的节点一个观察者都没挂"
        );
    }
    OBSERVERS_ON_THE_STEPS_AFTER_THE_PREFIX
        .iter()
        .filter(|(entry_name, observers_of_the_node)| {
            observers_of_the_node.contains(&observer) && entry_names.contains(entry_name)
        })
        .map(|(entry_name, _observers_of_the_node)| {
            singlefs_checker_tier::crash_amplification::NodeOfAnObserver {
                setting_file: THIS_FILE.to_string(),
                entry_name: (*entry_name).to_string(),
            }
        })
        .collect()
}

/// 一条流按环境跑一趟崩溃放量（展开上限、库目录、判器照 `common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line`），
/// 挂两张登记表上的观察者：标准前缀节点上的（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，可写挂载续做，代码在
/// `common_crash_point_observers`，另读删除流的发布表）与 C 之后那几步节点上的（[`OBSERVERS_ON_THE_STEPS_AFTER_THE_PREFIX`]，这一份的两个，另读流里每一版）。
/// 观察者走两段：录入段在 CPU 上看（交回镜像的按恢复镜像去重），判段在挂完的盘上判池级 checker（配了 GPU 上卡）。观察者计数的断言每一趟都判：
/// 每个观察者至少看过一个状态、一个状态都不判红；落点那一个看的每个状态都落到流里某一版或 A 之前；两个可写挂载的观察者每个状态都挂过一次、
/// 挂完的盘都过了池级 checker。只录、只排派活计划、叫停、判段报本次未跑的那一趟不判。
///
/// # Panics
/// 这条流 C 之后有一步不在登记表里；或上面任何一条断言不成立。
#[cfg(feature = "verdict-store")]
pub fn amplify_a_multiple_copy_on_write_flow(
    recorded: RecordedThroughTheCrashPointRecorder,
    default_expand_up_to: usize,
) -> singlefs_checker_tier::crash_amplification::EnvironmentRun {
    use super::common_crash_point_observers;
    use super::common_crash_points::{
        self, ObserverOfTheStandardPrefix, DELETION_BATCHES_BEFORE_THE_FINAL_BATCH,
        OBSERVERS_ON_THE_STANDARD_PREFIX,
    };
    use singlefs_checker_tier::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_checker_tier::crash_identity::ObserverName;
    let stream = recorded.prepared.recorded.stream;
    let flow_name = stream.flow_name();
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
    let versions = recorded.prepared.versions_of_the_stream.as_slice();
    let inodes_read_back = stream.inodes_read_back();
    let observe_the_landing_of_the_flow =
        |_ordinal: u64,
         _image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_landed_version(versions, report, counts);
        };
    let observe_the_mount_of_the_flow =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_and_the_files_read_back(
                versions,
                &inodes_read_back,
                image,
                report,
                counts,
            )
        };
    let observer_name = |name: &str| ObserverName::new(name).expect("观察者的名字非空、不长");
    let digest = digest_of_the_versions_of_the_stream(versions);
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
    for observer in EVERY_OBSERVER_OF_THE_STEPS_AFTER_THE_PREFIX {
        observers.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name()),
            observer_code_file: THIS_FILE.to_string(),
            extra_inputs_digest: digest,
            nodes: nodes_of_the_observer(stream, *observer),
            observe: match observer {
                ObserverOfTheStepsAfterThePrefix::LandedVersion => {
                    ObserverCode::CountsOnly(&observe_the_landing_of_the_flow)
                }
                ObserverOfTheStepsAfterThePrefix::MountContinuationAndFilesReadBack => {
                    ObserverCode::HandsBackAnImage {
                        observe: &observe_the_mount_of_the_flow,
                        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                    }
                }
            },
        });
    }
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            flow_name,
            &recorded.prepared.base,
            &recorded.prepared.writes,
            &recorded.prepared.segments,
            recorded.prepared.judged_root_index,
            &recorded.prepared.versions_with_a_file,
            &recorded.prepared.versions_without_a_file,
            recorded.crash_points,
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
        LANDED_VERSION_OBSERVER_NAME,
        MOUNT_CONTINUATION_OBSERVER_NAME,
    ];
    for name in observer_names {
        let states = observing
            .states_of_each_observer
            .get(name)
            .copied()
            .unwrap_or(0);
        assert!(
            states > 0,
            "流 {flow_name} 经过标准前缀与 C 之后那几步，观察者 {name} 一个状态都没看"
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
        } else if name == LANDED_VERSION_OBSERVER_NAME {
            let mut landed_counters: Vec<&'static str> = vec![LANDED_BEFORE_THE_FIRST_FILE_COUNTER];
            for version in versions {
                if !landed_counters.contains(&version.landed_counter_name) {
                    landed_counters.push(version.landed_counter_name);
                }
            }
            assert_eq!(
                landed_counters
                    .iter()
                    .map(|counter| count_named(counter))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：C 之后那几步上的每个状态都落到流里某一版或 A 之前"
            );
        } else {
            assert_eq!(
                (0..=DELETION_CONTINUATION_PUBLISHES_AT_MOST)
                    .map(|publishes| count_named(&steps::deletion_continuation_counter(publishes)))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：C 之后那几步上的每个状态都可写挂载过一次"
            );
            assert_eq!(
                count_named(FILES_READ_BACK_AFTER_THE_CONTINUATION_COUNTER)
                    + count_named(MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER),
                states,
                "流 {flow_name}：C 之后那几步上落到 A 起的每个状态都读回了每个文件"
            );
        }
        if name != LANDED_VERSION_OBSERVER_NAME {
            assert_eq!(
                count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
                states,
                "流 {flow_name}：观察者 {name} 看的每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
            );
        }
    }
    run
}
