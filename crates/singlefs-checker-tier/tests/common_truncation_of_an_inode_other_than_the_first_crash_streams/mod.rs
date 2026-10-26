//! 第一个文件之外的 inode 截断三条崩溃流共用的判法（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第五节「别的 inode 的截断
//! 崩溃流（照六·3 的三条）」，截断判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9 那三条推到别的 inode）。录法在
//! `tests/common_truncation_of_an_inode_other_than_the_first_recording/mod.rs`（只引 harness 档与实现）。
//!
//! 每个崩溃状态跑恢复 + 多版本 oracle（第一个文件恒是 A 的内容，每一版都在版本表里）、池级 checker（别的 inode 截断那几步 I-4.4 收截断条目、I-4.6
//! 每条记录都判、I-4.10 与 I-9.38 判 size 之外挂着的单元）、记录核对器；另挂一个观察者（[`observe_the_landed_version_and_the_truncated_inode`]）：
//! 记恢复落到哪一版，并在崩溃镜像上只读挂载一次、按号读 inode 2，读出的要等于落到的那一版里 inode 2 的内容（截前或截后，判决 K2「先落目标」推到
//! 别的 inode；判决第二节 S5「崩溃读回按 inode 给允许集合」）。
#![allow(
    dead_code,
    reason = "三条流的测试文件各只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use crate::common_truncation_of_an_inode_other_than_the_first_recording::{
    record_the_truncation_of_an_inode_other_than_the_first, writes_of_the_recorded_stream,
    RecordedTruncationOfAnInodeOtherThanTheFirst, TruncationOfAnInodeOtherThanTheFirstCrashStream,
    VersionOfTheStream, THE_TRUNCATED_INODE,
};
use singlefs_checker_tier::crash::{
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::CrashPointReplayObserver;
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{CrashImage, MemoryPool, PublishedVersion, RetainedWrite};

/// 崩溃点重放那一路要的几样：起点镜像、写表、段、被判的根槽写、版本表、观察者另读的落点表与每一版 inode 2 的内容。
pub struct PreparedTruncationOfAnInodeOtherThanTheFirst {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub first_file_versions: Vec<PublishedVersion>,
    pub versions: Vec<VersionOfTheStream>,
    pub landed_versions: Vec<((InstanceGeneration, CheckpointTxg), &'static str)>,
    pub recorded: RecordedTruncationOfAnInodeOtherThanTheFirst,
}

impl PreparedTruncationOfAnInodeOtherThanTheFirst {
    #[must_use]
    pub fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::with_a_file_only(&self.first_file_versions)
    }

    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段（快档展开的段）。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }

    /// 观察者另读的输入（落点表与每一版 inode 2 的内容）的摘要：进观察者的版本与去重键。
    #[must_use]
    pub fn digest_of_the_observer_inputs(&self) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        singlefs_harness::sha256::sha256_digest(
            format!("{:?}\n{:?}", self.landed_versions, self.versions).as_bytes(),
        )
    }
}

/// 录一条流（不录崩溃点）、切段。
#[must_use]
pub fn prepare_the_truncation_of_an_inode_other_than_the_first(
    tag: &str,
    stream: TruncationOfAnInodeOtherThanTheFirstCrashStream,
) -> PreparedTruncationOfAnInodeOtherThanTheFirst {
    let recorded = record_the_truncation_of_an_inode_other_than_the_first(tag, stream);
    let writes = writes_of_the_recorded_stream(&recorded);
    PreparedTruncationOfAnInodeOtherThanTheFirst {
        base: writes.base,
        writes: writes.writes,
        segments: writes.segments,
        judged_root_index: writes.judged_root_index,
        first_file_versions: recorded.first_file_versions(),
        versions: recorded.versions(),
        landed_versions: recorded.landed_versions(),
        recorded,
    }
}

/// 落到建 inode 2、按号写它或更早（A、暖机的根）时加一的计数名。
pub const LANDED_BEFORE_THE_STEPS_COUNTER: &str = "landed_before_the_truncation_stream_steps";
/// 只读挂载落到流里的一版、按号读出的 inode 2 等于那一版的内容时加一的计数名。
pub const READ_BACK_OF_THE_TRUNCATED_INODE_MATCHES_COUNTER: &str =
    "truncated_inode_read_back_matches_the_landed_version";
/// 只读挂载落到还没建 inode 2 的那一版（A）、按号打开它拒时加一的计数名。
pub const TRUNCATED_INODE_NOT_YET_CREATED_COUNTER: &str = "truncated_inode_not_yet_created";
/// 恢复落到流里那一版之外（A 之前：mkfs 或两次暖机），不读 inode 2 时加一的计数名。
pub const LANDED_BEFORE_THE_FIRST_FILE_COUNTER: &str = "landed_before_the_first_file";

/// 观察者看一个状态：记恢复落到按号写完 inode 2 之后哪一步那一版（计数名，落到它之前的记 [`LANDED_BEFORE_THE_STEPS_COUNTER`]）；落到流里有文件的一版时，
/// 在崩溃镜像上只读挂载一次（择根、扫环、施加前缀，`mounted_read::mount_read_only`），按号读 inode 2，读出的要等于挂载落到的那一版里 inode 2 的内容
/// （判决 m4-multi-inode-size-r1 第二节 S5「崩溃读回按 inode 给允许集合」：崩溃后 inode 2 只读到截前或截后，K2 推到别的 inode）。
///
/// # Panics
/// 恢复落到流里有文件的一版而只读挂载挂不上；读出的 inode 2 与那一版的不同；那一版还没建 inode 2 而按号打开它照收。
pub fn observe_the_landed_version_and_the_truncated_inode(
    versions: &[VersionOfTheStream],
    landed_versions: &[((InstanceGeneration, CheckpointTxg), &'static str)],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let landed_by_the_recovery = report.effective_root;
    let counter = landed_by_the_recovery
        .and_then(|root| {
            landed_versions
                .iter()
                .find(|(version, _counter)| *version == root)
                .map(|(_version, counter)| *counter)
        })
        .unwrap_or(LANDED_BEFORE_THE_STEPS_COUNTER);
    counts.add(counter, 1);
    let is_a_version_of_the_stream = landed_by_the_recovery
        .is_some_and(|root| versions.iter().any(|version| version.version == root));
    if !is_a_version_of_the_stream {
        counts.add(LANDED_BEFORE_THE_FIRST_FILE_COUNTER, 1);
        return;
    }
    let crash_image = materialized_crash_image(image);
    let mounted = mount_read_only(&crash_image).unwrap_or_else(|failure| {
        panic!(
            "恢复落到 {landed_by_the_recovery:?}（流里有文件的一版）：只读挂载挂得上：{failure:?}"
        )
    });
    let landed_by_the_mount = (
        mounted.effective_root.instance,
        mounted.effective_root.checkpoint_txg,
    );
    let Some(landed) = versions
        .iter()
        .find(|version| version.version == landed_by_the_mount)
    else {
        counts.add(LANDED_BEFORE_THE_FIRST_FILE_COUNTER, 1);
        return;
    };
    let opened = mounted
        .mounted
        .open_file(&crash_image, InodeNumber(THE_TRUNCATED_INODE));
    match (&landed.truncated_inode_content, opened) {
        (Some(expected), Ok(file)) => {
            let size = file.inode_record().size;
            let read = file
                .read_at(&crash_image, FileOffsetInBytes(0), size)
                .unwrap_or_else(|failure| {
                    panic!("挂载落到 {landed_by_the_mount:?}：inode {THE_TRUNCATED_INODE} 从 0 读到 size：{failure:?}")
                })
                .bytes;
            assert!(
                read == *expected,
                "挂载落到 {landed_by_the_mount:?}：inode {THE_TRUNCATED_INODE} 读出 {} 字节，那一版是 {} 字节（首个不同在 {:?}）",
                read.len(),
                expected.len(),
                read.iter().zip(expected).position(|(got, wanted)| got != wanted)
            );
            counts.add(READ_BACK_OF_THE_TRUNCATED_INODE_MATCHES_COUNTER, 1);
        }
        (Some(_), Err(failure)) => panic!(
            "挂载落到 {landed_by_the_mount:?}（那一版有 inode {THE_TRUNCATED_INODE}）：按号打开它拒：{failure:?}"
        ),
        (None, Ok(_)) => panic!(
            "挂载落到 {landed_by_the_mount:?}（那一版还没建 inode {THE_TRUNCATED_INODE}）：按号打开它照收"
        ),
        (None, Err(_)) => counts.add(TRUNCATED_INODE_NOT_YET_CREATED_COUNTER, 1),
    }
}

/// 观察者：[`observe_the_landed_version_and_the_truncated_inode`]，另读的输入是落点表与每一版 inode 2 的内容。
pub fn observer_of_the_truncated_inode<'prepared>(
    prepared: &'prepared PreparedTruncationOfAnInodeOtherThanTheFirst,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: "landed_version_and_the_truncated_inode_read_back",
        extra_inputs_digest: prepared.digest_of_the_observer_inputs(),
        observe,
    }
}

/// 每个状态都看过；落到每一步那一版的、落到它们之前的都有状态，加起来是状态数；落到流里有文件的一版的状态上 inode 2 都读对了，而且真读过。
pub fn assert_every_state_was_observed(
    prepared: &PreparedTruncationOfAnInodeOtherThanTheFirst,
    tally: &CrashPointReplayTally,
) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    let mut counters: Vec<&'static str> = vec![LANDED_BEFORE_THE_STEPS_COUNTER];
    for (_version, counter) in &prepared.landed_versions {
        if !counters.contains(counter) {
            counters.push(counter);
        }
    }
    assert_eq!(
        counters
            .iter()
            .map(|counter| counts.count_named(counter))
            .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一步、或那几步之前：{counts:?}"
    );
    for counter in counters {
        assert!(
            counts.count_named(counter) > 0,
            "{counter} 的状态真跑到了：{counts:?}"
        );
    }
    assert_eq!(
        counts.count_named(READ_BACK_OF_THE_TRUNCATED_INODE_MATCHES_COUNTER)
            + counts.count_named(TRUNCATED_INODE_NOT_YET_CREATED_COUNTER)
            + counts.count_named(LANDED_BEFORE_THE_FIRST_FILE_COUNTER),
        tally.states,
        "每个状态要么读对了 inode 2、要么落在还没建它或第一个文件之前的一版：{counts:?}"
    );
    assert!(
        counts.count_named(READ_BACK_OF_THE_TRUNCATED_INODE_MATCHES_COUNTER) > 0,
        "inode 2 真读过：{counts:?}"
    );
}

/// 判器判的几样一个都不红：oracle 两遍、走读、记录核对器两条判据、池级 checker 的每一条不变量。
pub fn assert_clean(tally: &CrashPointReplayTally) {
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

/// 快档的展开：写数小于 10 的段任意真子集，别的段不展开（只以整段持久进入后面的状态）。
#[must_use]
pub fn expansion_of_the_segments_shorter_than_ten(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < 10 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}
