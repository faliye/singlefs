//! 截断三条崩溃流共用的录法与判法（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9）。三条都接在多记录流
//! 「取号 → 暖机 × 2 → A → B → C」之后（C 是第一个文件三个数据单元、两个半净荷；前缀与多记录流、删除流同一份录法：
//! `common_crash_points` 的 `sequential_write` 与 `record_the_multi_record_publishes`），截断的各批按只供测试的批大小 1 放（判决 K9）：
//! - 缩到 0：截断意图（size 写成 0，单元 0 重写成声明长度 0，单元 1、2 在 size 之外挂着、截断条目点名第一个文件）→ 一批（放单元 2）
//!   → 最后一批（放单元 1、同一次发布删截断条目，上段叶条目换成内联那一个单元）；
//! - 缩到单元中间再变长：缩到一个半净荷的意图（单元 1 重写成半个净荷，单元 2 在 size 之外挂着）→ 最后一批（放单元 2、删条目）
//!   → 变长到三个半净荷（一次发布：单元 1 补零重写成一整个净荷，之后到 size 是洞，上段叶条目标签 3，判决 K5）；
//! - 快照下缩：建快照 S → 缩到半个净荷的意图（单元 0 重写，被快照引用、换下的单元进 deadlist、映射条目留存，判决 K4）→ 一批 → 最后一批
//!   （放掉的尾部单元被快照引用，同样进 deadlist、不释放）。
//!
//! 崩溃点重放那一路（[`prepare_the_truncation_crash_stream`]，不录崩溃点）与崩溃放量那一路（[`record_through_the_crash_point_recorder`]，
//! 录制器给每一步设一个点，设点文件是这一份）调同一份步骤函数（[`TruncationStreamStep::publish`]），写表逐字节相同。
//! 步数按计划定死，不按「截断条目删了没有」停：最后一批漏删条目（必红世界）时录流照样录完，红在判器上，不红在搭建上。
//!
//! 崩溃放量那一路挂两张登记表上的观察者（[`amplify_a_truncation_flow`]，同整理流的 `common_compaction_crash_point_observers::amplify_a_compaction_flow`）：
//! 标准前缀那几个节点上的可写挂载续做（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，代码在 `common_crash_point_observers`），
//! 与 C 之后截断那几步节点上的两个（[`OBSERVERS_ON_THE_TRUNCATION_STEPS`]）：
//! - 落点：恢复落到流里哪一版（[`observe_the_landed_version`]，只记计数；崩溃点重放那一路挂的也是它）；
//! - 可写挂载续做截断：每个状态上可写挂载一次（批大小 1，同录流），共用走空把截断的各批与最后一批走完（判决 K1「截断进共用走空」），
//!   续做的步数等于恢复落到的那一版之后录流还剩的截断批数，挂完 logged_ops 里没有截断条目、第一个文件读出落到的那一版的内容
//!   （[`observe_the_mount_continuing_the_truncation`]）；交回挂完的盘，判段在它上面跑池级 checker（配了 GPU 上卡）。
#![allow(
    dead_code,
    reason = "三条流的测试文件各只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use super::common::{build_pool, geometry, parameters, BuiltPool};
use super::common_crash_points::{content_needing, sequential_write};
use super::common_truncation::{
    deletion_switches_with_batches_of_one, first_file_read_by_the_mounted_reader,
};
use singlefs_checker_tier::crash::{
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_state_enumeration::CrashPointReplayObserver;
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::logged_operations::live_truncation_intent_of_the_entries;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    ShadowLedger,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这一份的仓内路径（与 `file!()` 归一之后相同）。
pub const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/common_truncation_crash_streams/mod.rs";

/// 三条截断流里的哪一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationCrashStream {
    /// 缩到 0。
    DownToZero,
    /// 缩到单元中间再变长。
    IntoAUnitThenExtended,
    /// 快照下缩。
    UnderASnapshot,
}

/// 接在 C 之后的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationStreamStep {
    /// 建快照 S（快照下缩那一条）。
    CreateSnapshot,
    /// 截断意图：一次发布写记录的 size = 目标、签截断条目、按需重写尾单元（判决 K2、K3）。
    TruncationIntent { new_size_in_bytes: u64 },
    /// 不是最后一批的一批：从尾部放一个单元，截断条目留着。
    TruncationBatch,
    /// 最后一批：放完剩下的、同一次发布删截断条目（判决 K2、K11）。
    TruncationLastBatch,
    /// 变长：一次发布做完，不立意图（判决 K5）。
    Extension { new_size_in_bytes: u64 },
}

impl TruncationStreamStep {
    /// 录制器设点用的名字，也是观察者落到这一版时加一的计数名去掉 `landed_on_` 那一段。
    #[must_use]
    pub fn entry_name(self) -> &'static str {
        match self {
            Self::CreateSnapshot => "create_snapshot",
            Self::TruncationIntent { .. } => "truncation_intent",
            Self::TruncationBatch => "truncation_batch",
            Self::TruncationLastBatch => "truncation_last_batch",
            Self::Extension { .. } => "extension",
        }
    }

    /// 观察者落到这一版时加一的计数名。
    #[must_use]
    pub fn landed_counter_name(self) -> &'static str {
        match self {
            Self::CreateSnapshot => "landed_on_create_snapshot",
            Self::TruncationIntent { .. } => "landed_on_truncation_intent",
            Self::TruncationBatch => "landed_on_truncation_batch",
            Self::TruncationLastBatch => "landed_on_truncation_last_batch",
            Self::Extension { .. } => "landed_on_extension",
        }
    }

    /// 接着池的现行那一版发这一步，池接到它那一版上。
    pub fn publish(self, pool: &mut BuiltPool) -> TransactionOutput {
        match self {
            Self::CreateSnapshot => {
                let created = super::common_snapshot_prefix::create_snapshot(pool);
                pool.output = created.clone();
                created
            }
            Self::TruncationIntent { new_size_in_bytes }
            | Self::Extension { new_size_in_bytes } => {
                super::common_truncation::change_the_size_of_the_first_file_and_advance(
                    pool,
                    new_size_in_bytes,
                )
            }
            Self::TruncationBatch | Self::TruncationLastBatch => {
                super::common_truncation::publish_the_next_truncation_step_and_advance(
                    pool,
                    super::common_truncation::batches_of_one(),
                )
            }
        }
    }

    /// 这一步之后第一个文件读出的内容（前一版是 `content_before`）：改 size 的截到它或补 0 到它，别的不动内容。
    fn content_after(self, content_before: &[u8]) -> Vec<u8> {
        match self {
            Self::TruncationIntent { new_size_in_bytes }
            | Self::Extension { new_size_in_bytes } => {
                let new_size =
                    usize::try_from(new_size_in_bytes).expect("这几条流的 size 装得进 usize");
                let mut content = content_before[..new_size.min(content_before.len())].to_vec();
                content.resize(new_size, 0);
                content
            }
            Self::CreateSnapshot | Self::TruncationBatch | Self::TruncationLastBatch => {
                content_before.to_vec()
            }
        }
    }
}

/// 净荷容量。
fn payload_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 这一条流接在 C 之后的各步（按计划定死）。
#[must_use]
pub fn steps_of(stream: TruncationCrashStream) -> Vec<TruncationStreamStep> {
    let payload = payload_in_bytes();
    match stream {
        TruncationCrashStream::DownToZero => vec![
            TruncationStreamStep::TruncationIntent {
                new_size_in_bytes: 0,
            },
            TruncationStreamStep::TruncationBatch,
            TruncationStreamStep::TruncationLastBatch,
        ],
        TruncationCrashStream::IntoAUnitThenExtended => vec![
            TruncationStreamStep::TruncationIntent {
                new_size_in_bytes: payload + payload / 2,
            },
            TruncationStreamStep::TruncationLastBatch,
            TruncationStreamStep::Extension {
                new_size_in_bytes: 3 * payload + payload / 2,
            },
        ],
        TruncationCrashStream::UnderASnapshot => vec![
            TruncationStreamStep::CreateSnapshot,
            TruncationStreamStep::TruncationIntent {
                new_size_in_bytes: payload / 2,
            },
            TruncationStreamStep::TruncationBatch,
            TruncationStreamStep::TruncationLastBatch,
        ],
    }
}

/// C 之前的三版带文件的发布：A（`build_pool` 里那一版）、B、C 的输出与内容。
pub struct MultiRecordPrefix {
    pub versions: Vec<PublishedVersion>,
    pub content_of_c: Vec<u8>,
}

/// 录完的一条流：池（录制流在它里面）、C 之前那一段、C 之后每一步与它的输出。
pub struct RecordedTruncationStream {
    pub pool: BuiltPool,
    pub prefix: MultiRecordPrefix,
    pub steps: Vec<(TruncationStreamStep, TransactionOutput)>,
}

impl RecordedTruncationStream {
    /// oracle 认的版本表：A、B、C，与 C 之后的每一版（各批的内容同意图那一版：意图那次已把 size 写成目标）。
    #[must_use]
    pub fn versions(&self) -> Vec<PublishedVersion> {
        let mut versions = self.prefix.versions.clone();
        let mut content = self.prefix.content_of_c.clone();
        for (step, output) in &self.steps {
            content = step.content_after(&content);
            versions.push(PublishedVersion {
                instance: output.root.instance,
                checkpoint_txg: output.root.checkpoint_txg,
                content: content.clone(),
            });
        }
        versions
    }

    /// 每一步那一版的根身份与落到它时加一的计数名（观察者另读的输入）。
    #[must_use]
    pub fn landed_versions(&self) -> Vec<((InstanceGeneration, CheckpointTxg), &'static str)> {
        self.steps
            .iter()
            .map(|(step, output)| {
                (
                    (output.root.instance, output.root.checkpoint_txg),
                    step.landed_counter_name(),
                )
            })
            .collect()
    }
}

/// 不录崩溃点那一路：`build_pool`（取号、暖机两次、A）、B、C，再按计划发 C 之后的各步。
#[must_use]
pub fn record_the_truncation_stream(
    tag: &str,
    stream: TruncationCrashStream,
) -> RecordedTruncationStream {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    let prefix = multi_record_prefix(&first, &second, second_content, &third, third_content);
    let steps = steps_of(stream)
        .into_iter()
        .map(|step| (step, step.publish(&mut pool)))
        .collect();
    RecordedTruncationStream {
        pool,
        prefix,
        steps,
    }
}

fn multi_record_prefix(
    first: &TransactionOutput,
    second: &TransactionOutput,
    second_content: Vec<u8>,
    third: &TransactionOutput,
    third_content: Vec<u8>,
) -> MultiRecordPrefix {
    assert_eq!(third.data_pointers.len(), 3, "C 之后第一个文件三个数据单元");
    let version = |output: &TransactionOutput, content: Vec<u8>| PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    };
    MultiRecordPrefix {
        versions: vec![
            version(first, super::common::file_content()),
            version(second, second_content),
            version(third, third_content.clone()),
        ],
        content_of_c: third_content,
    }
}

/// 每一步之后截断条目在不在、第一个文件还剩几个数据单元、上段叶条目的标签（录流的形状，判器判完之后核：改坏实现时先红在判器上）。
pub fn assert_the_shape_of_the_steps(
    stream: TruncationCrashStream,
    recorded: &RecordedTruncationStream,
) {
    let expected: Vec<(bool, usize, u8)> = match stream {
        // 意图：条目在、三个单元都还挂着、下段根；一批：放掉单元 2；最后一批：放掉单元 1、删条目，剩下的一个单元内联在上段叶里
        TruncationCrashStream::DownToZero => vec![(true, 3, 1), (true, 2, 1), (false, 1, 2)],
        // 意图：条目在、三个单元；最后一批放掉单元 2、删条目；变长：两个单元之后到 size 是洞，标签 3
        TruncationCrashStream::IntoAUnitThenExtended => {
            vec![(true, 3, 1), (false, 2, 1), (false, 2, 3)]
        }
        // 建快照：不动文件；意图、一批、最后一批同缩到 0 那一条的形状
        TruncationCrashStream::UnderASnapshot => {
            vec![(false, 3, 1), (true, 3, 1), (true, 2, 1), (false, 1, 2)]
        }
    };
    let found: Vec<(bool, usize, u8)> = recorded
        .steps
        .iter()
        .map(|(_step, output)| {
            (
                super::common_truncation::truncation_entry_of(output).is_some(),
                output.data_pointers.len(),
                super::common_truncation::upper_leaf_tag_of_the_first_file(output),
            )
        })
        .collect();
    assert_eq!(
        found, expected,
        "{stream:?} 每一步之后（截断条目在不在、数据单元数、上段叶条目标签）"
    );
}

/// 快照下缩那一条：意图与各批换下的、被快照引用的单元都进 deadlist（判决 K4），一条 deadlist 条目一条留存的映射条目，条数只增不减。
pub fn assert_the_units_truncated_under_the_snapshot_went_to_the_deadlist(
    recorded: &RecordedTruncationStream,
) {
    let mut deadlist_entries_before = 0;
    for (step, output) in &recorded.steps {
        if *step == TruncationStreamStep::CreateSnapshot {
            continue;
        }
        assert!(
            output.deadlist_entries.len() > deadlist_entries_before,
            "{step:?}：这一步换下的、被快照引用的单元进 deadlist（之前 {deadlist_entries_before} 条，之后 {} 条）",
            output.deadlist_entries.len()
        );
        assert_eq!(
            output.mapping_entries_kept_for_the_snapshots.len(),
            output.deadlist_entries.len(),
            "{step:?}：一条 deadlist 条目一条留存的映射条目"
        );
        deadlist_entries_before = output.deadlist_entries.len();
    }
}

/// 崩溃点重放那一路要的几样：起点镜像、写表、段、被判的根槽写、版本表、观察者另读的落点表。
pub struct PreparedTruncationCrashStream {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub landed_versions: Vec<((InstanceGeneration, CheckpointTxg), &'static str)>,
    pub recorded: RecordedTruncationStream,
}

impl PreparedTruncationCrashStream {
    #[must_use]
    pub fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::with_a_file_only(&self.versions)
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

    /// 观察者另读的输入（落点表）的摘要：进观察者的版本与去重键。
    #[must_use]
    pub fn digest_of_the_landed_versions(&self) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        digest_of_the_landed_versions(&self.landed_versions)
    }
}

/// 落点表的摘要（崩溃点重放那一路与崩溃放量那一路同一种算法）。
#[must_use]
pub fn digest_of_the_landed_versions(
    landed_versions: &[((InstanceGeneration, CheckpointTxg), &'static str)],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    singlefs_harness::sha256::sha256_digest(format!("{landed_versions:?}").as_bytes())
}

/// 录一条流（不录崩溃点）、切段。
#[must_use]
pub fn prepare_the_truncation_crash_stream(
    tag: &str,
    stream: TruncationCrashStream,
) -> PreparedTruncationCrashStream {
    let recorded = record_the_truncation_stream(tag, stream);
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("流里有根槽写");
    PreparedTruncationCrashStream {
        base,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions: recorded.versions(),
        landed_versions: recorded.landed_versions(),
        recorded,
    }
}

/// 恢复落到 C 之后的哪一步那一版（计数名）；落到 C 或更早（A、B、暖机的根）记 `landed_before_the_truncation_stream_steps`。
pub fn observe_the_landed_version(
    landed_versions: &[((InstanceGeneration, CheckpointTxg), &'static str)],
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let counter = report
        .effective_root
        .and_then(|root| {
            landed_versions
                .iter()
                .find(|(version, _counter)| *version == root)
                .map(|(_version, counter)| *counter)
        })
        .unwrap_or(LANDED_BEFORE_THE_STEPS_COUNTER);
    counts.add(counter, 1);
}

/// 落到 C 或更早时加一的计数名。
pub const LANDED_BEFORE_THE_STEPS_COUNTER: &str = "landed_before_the_truncation_stream_steps";

/// 观察者：逐状态记恢复落到哪一版（只记计数）。
pub fn landed_version_observer<'prepared>(
    prepared: &'prepared PreparedTruncationCrashStream,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: "landed_version",
        extra_inputs_digest: prepared.digest_of_the_landed_versions(),
        observe,
    }
}

/// 每个状态都看过；落到每一步那一版的、落到 C 或更早的都有状态，加起来是状态数。
pub fn assert_every_state_was_observed(
    prepared: &PreparedTruncationCrashStream,
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
        "每个状态都落到流里的某一版、C 或更早：{counts:?}"
    );
    for counter in counters {
        assert!(
            counts.count_named(counter) > 0,
            "{counter} 的状态真跑到了：{counts:?}"
        );
    }
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

/// 录制器那一份造法：取号加暖机、新池新建文件、B、C 四个点由公共模块 `common_crash_points` 设（与多记录流、删除流共享到 C），
/// C 之后每一步各一个点，设在这一份里。
/// 另带截断那几步的观察者另读的两样：落点表（[`observe_the_landed_version`]）与可写挂载续做截断的输入（[`TruncationMountContinuationInputs`]）。
#[cfg(feature = "verdict-store")]
pub struct RecordedThroughTheCrashPointRecorder {
    pub stream: TruncationCrashStream,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub landed_versions: Vec<((InstanceGeneration, CheckpointTxg), &'static str)>,
    pub truncation_mount_continuation_inputs: TruncationMountContinuationInputs,
}

#[cfg(feature = "verdict-store")]
#[must_use]
pub fn record_through_the_crash_point_recorder(
    tag: &str,
    stream: TruncationCrashStream,
) -> RecordedThroughTheCrashPointRecorder {
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    let publishes =
        super::common_crash_points::record_the_multi_record_publishes(&mut recorder, &mut pool);
    let prefix = multi_record_prefix(
        &publishes.first,
        &publishes.second,
        publishes.second_content.clone(),
        &publishes.third,
        publishes.third_content.clone(),
    );
    let steps = steps_of(stream)
        .into_iter()
        .map(|step| {
            let output = recorder.step(file!(), step.entry_name(), || step.publish(&mut pool));
            (step, output)
        })
        .collect();
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("流里有根槽写");
    let recorded = RecordedTruncationStream {
        pool,
        prefix,
        steps,
    };
    RecordedThroughTheCrashPointRecorder {
        stream,
        base,
        writes,
        segments,
        crash_points,
        judged_root_index,
        versions: recorded.versions(),
        landed_versions: recorded.landed_versions(),
        truncation_mount_continuation_inputs:
            TruncationMountContinuationInputs::of_the_recorded_stream(&recorded),
    }
}

// ============================== 截断那几步节点上的两个观察者（崩溃放量那一路） ==============================

/// 截断的一步之后录流还剩几次截断的发布（各批与最后一批，批大小 1）：意图与不是最后一批的一批数到这一条截断的最后一批为止（含），
/// 最后一批、变长、建快照是 0。
fn truncation_publishes_left_after(steps: &[TruncationStreamStep], step_index: usize) -> usize {
    match steps[step_index] {
        TruncationStreamStep::TruncationIntent { .. } | TruncationStreamStep::TruncationBatch => {
            steps[step_index + 1..]
                .iter()
                .position(|step| *step == TruncationStreamStep::TruncationLastBatch)
                .map(|offset_of_the_last_batch| offset_of_the_last_batch + 1)
                .expect("录流里每一条截断都以最后一批收尾（步数按计划定死）")
        }
        TruncationStreamStep::CreateSnapshot
        | TruncationStreamStep::TruncationLastBatch
        | TruncationStreamStep::Extension { .. } => 0,
    }
}

/// 截断流里有文件的一版：根身份、第一个文件那一刻的内容、落到它之后可写挂载的续做还差几次截断的发布。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheTruncationStream {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub first_file_content: Vec<u8>,
    pub truncation_publishes_left: usize,
}

/// 可写挂载续做截断那个观察者另读的流程输入（进观察者的版本与去重键）：A、B、C 与 C 之后每一步那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TruncationMountContinuationInputs {
    pub versions: Vec<VersionOfTheTruncationStream>,
}

impl TruncationMountContinuationInputs {
    /// A、B、C（续做 0 次）与 C 之后每一步那一版（[`truncation_publishes_left_after`]）。
    #[must_use]
    pub fn of_the_recorded_stream(recorded: &RecordedTruncationStream) -> Self {
        let steps: Vec<TruncationStreamStep> =
            recorded.steps.iter().map(|(step, _)| *step).collect();
        let published = recorded.versions();
        let versions = published
            .iter()
            .enumerate()
            .map(|(version_index, version)| {
                let truncation_publishes_left = version_index
                    .checked_sub(recorded.prefix.versions.len())
                    .map_or(0, |step_index| {
                        truncation_publishes_left_after(&steps, step_index)
                    });
                VersionOfTheTruncationStream {
                    version: (version.instance, version.checkpoint_txg),
                    first_file_content: version.content.clone(),
                    truncation_publishes_left,
                }
            })
            .collect();
        Self { versions }
    }

    /// 另读的输入的摘要：逐版一行（根身份、内容的 SHA-256、续做还差几次）写下、取 SHA-256。文字里只有逐版那几行，不带标签与
    /// 手写的版本号（D13（验证路线） 已定项 17：手写的版本常量不进键）；录法变了，变的是逐版的内容，摘要跟着变。
    #[must_use]
    pub fn digest(&self) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        let mut text = String::new();
        for version in &self.versions {
            text.push_str(&format!(
                "version instance={} txg={} content={:?} truncation_publishes_left={}\n",
                version.version.0 .0,
                version.version.1 .0,
                singlefs_harness::sha256::sha256_digest(&version.first_file_content),
                version.truncation_publishes_left
            ));
        }
        singlefs_harness::sha256::sha256_digest(text.as_bytes())
    }
}

/// 判段在可写挂载续做截断之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
pub const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 可写挂载续做截断的计数名：`truncation_continuation_publishes_<续做发了几次>`。
#[must_use]
pub fn truncation_continuation_counter(publishes: usize) -> String {
    format!("truncation_continuation_publishes_{publishes}")
}

/// 一条截断至多还剩几次截断的发布（三条流里截到 0 那两条：意图之后一批、最后一批）。
const TRUNCATION_CONTINUATION_PUBLISHES_AT_MOST: usize = 2;

/// 恢复落到流里那一版之外（A 之前：mkfs 或两次暖机）时加一的计数名。
pub const MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER: &str = "mount_landed_before_the_first_file";

/// 可写挂载续做截断那个观察者看一个状态（模块文件头「可写挂载续做截断」那一样）：在崩溃镜像落成的盘的副本上可写挂载一次（批大小 1，同录流），
/// 共用走空把队列走到空，续做发的次数等于恢复落到的那一版之后录流还剩的截断发布数；落到流里有文件的一版的，挂完 logged_ops 里没有截断条目、
/// 第一个文件经挂载态读者读出那一版的内容（截断意图那次已把 size 写成目标，续做不改内容）；落到 A 之前的挂完照旧没有树。
/// 挂载发的写录在一条新的录制流上，连同挂完的盘交回：判段在挂完的盘上跑池级 checker（判干净时 [`POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER`]
/// 加一；配了 GPU 上卡）。可写挂载与续做真跑实现，只能在 CPU 上（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
///
/// # Panics
/// 可写挂载做不成，或上面任何一样对不上。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn observe_the_mount_continuing_the_truncation(
    inputs: &TruncationMountContinuationInputs,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    let landed = inputs
        .versions
        .iter()
        .find(|version| version.version == effective_root);
    let materialized = materialized_crash_image(image);
    let mount_parameters = parameters();
    // 记读：挂载读过的每一段交回去，录入段按它算读集见证
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = materialized
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                materialized.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &mount_parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "共用走空把队列走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    let expected_publishes = landed.map_or(0, |version| version.truncation_publishes_left);
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        expected_publishes,
        "恢复落到 {effective_root:?}：录流在这一版之后还剩几次截断的发布，续做就发几次"
    );
    counts.add(&truncation_continuation_counter(expected_publishes), 1);
    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    match landed {
        Some(version) => {
            let current = mounted
                .current
                .version_of_trees()
                .expect("落到 A 起的一版：挂完有树");
            assert!(
                live_truncation_intent_of_the_entries(&current.logged_operations).is_none(),
                "恢复落到 {effective_root:?}：挂完 logged_ops 里没有截断条目"
            );
            let read =
                first_file_read_by_the_mounted_reader(&disks_after_the_observer, &current.root);
            assert!(
                read == version.first_file_content,
                "恢复落到 {effective_root:?}：挂完第一个文件读出那一版的内容（{} 字节），实际 {} 字节",
                version.first_file_content.len(),
                read.len()
            );
        }
        None => {
            assert!(
                mounted.current.version_of_trees().is_none(),
                "恢复落到 {effective_root:?}（A 之前）：挂完照旧没有树"
            );
            counts.add(MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER, 1);
        }
    }
    let (writes_on_the_crash_image, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER
            .to_string(),
    }
}

/// 挂在截断流 C 之后那几步节点上的观察者（封闭的一张：加一种就加一个成员，[`amplify_a_truncation_flow`] 里的 `match` 跟着补）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverOfTheTruncationSteps {
    /// 恢复落到流里哪一版（只记计数，[`observe_the_landed_version`]）。
    LandedVersion,
    /// 可写挂载一次、共用走空把截断走完，挂完的盘过池级 checker（[`observe_the_mount_continuing_the_truncation`]）。
    MountContinuingTheTruncation,
}

impl ObserverOfTheTruncationSteps {
    /// 观察者的名字（落点那一个与崩溃点重放那一路的同一个观察者同名）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::LandedVersion => "landed_version",
            Self::MountContinuingTheTruncation => "mount_continuing_the_truncation",
        }
    }
}

/// 两个观察者都挂。
const EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS: &[ObserverOfTheTruncationSteps] = &[
    ObserverOfTheTruncationSteps::LandedVersion,
    ObserverOfTheTruncationSteps::MountContinuingTheTruncation,
];

/// 登记表：C 之后那几步的节点（入口名照 [`TruncationStreamStep::entry_name`]，设点文件是这一份）→ 挂的观察者。
/// 标准前缀那几个节点（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`：取号加暖机、新池新建文件、B、C 两次顺序写）不在这里；
/// 三条流 C 之后的每一步都在这里：建快照、截断意图、一批、最后一批、变长。[`amplify_a_truncation_flow`] 开跑前核这条流 C 之后的每一步都在这张表里。
pub static OBSERVERS_ON_THE_TRUNCATION_STEPS: &[(&str, &[ObserverOfTheTruncationSteps])] = &[
    ("create_snapshot", EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS),
    ("truncation_intent", EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS),
    ("truncation_batch", EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS),
    (
        "truncation_last_batch",
        EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS,
    ),
    ("extension", EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS),
];

/// 这条流 C 之后每一步的入口名（不重复，按录流的次序）。
fn entry_names_of_the_steps_after_the_standard_prefix(
    stream: TruncationCrashStream,
) -> Vec<&'static str> {
    let mut entry_names: Vec<&'static str> = Vec::new();
    for step in steps_of(stream) {
        if !entry_names.contains(&step.entry_name()) {
            entry_names.push(step.entry_name());
        }
    }
    entry_names
}

/// 一个截断那几步的观察者挂在这条流的哪几个节点上：登记表里挂着它、这条流也经过的那几个入口（设点文件是这一份）。
///
/// # Panics
/// 这条流 C 之后有一步不在登记表里（那一步的节点就一个观察者都没挂）。
#[cfg(feature = "verdict-store")]
#[must_use]
pub fn nodes_of_the_truncation_steps_observer(
    stream: TruncationCrashStream,
    observer: ObserverOfTheTruncationSteps,
) -> Vec<singlefs_checker_tier::crash_amplification::NodeOfAnObserver> {
    let entry_names = entry_names_of_the_steps_after_the_standard_prefix(stream);
    for entry_name in &entry_names {
        assert!(
            OBSERVERS_ON_THE_TRUNCATION_STEPS
                .iter()
                .any(|(registered_entry_name, _observers)| registered_entry_name == entry_name),
            "流 {stream:?} C 之后的一步 {entry_name} 不在截断那几步的登记表里：它的节点一个观察者都没挂"
        );
    }
    OBSERVERS_ON_THE_TRUNCATION_STEPS
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

/// 截断那几步的两个观察者在这条流的放量计划里挂上了 C 之后的每一个点，别的点一个都不挂：计划（`crash_amplification::plan_crash_points`）
/// 里每个点，「两个观察者的节点里有一个装得下它」与「它的身份以这一份的路径起头、入口名（名字里第一个 `.` 之前那一段）是这条流 C 之后的一步」
/// 逐点相同，两个观察者各自挂上的点数等于另按入口名数出来的数，而且不是 0。交回那个点数。
///
/// # Panics
/// 上面任何一样对不上。
#[cfg(feature = "verdict-store")]
pub fn assert_the_truncation_observers_sit_on_every_planned_point_after_the_standard_prefix(
    flow_name: &str,
    recorded: RecordedThroughTheCrashPointRecorder,
    default_expand_up_to: usize,
) -> usize {
    use singlefs_checker_tier::crash_amplification::{
        plan_crash_points, CrashFlow, ExpandUpToWritesPerSegment,
    };
    let limit = ExpandUpToWritesPerSegment::from_environment_or(default_expand_up_to);
    let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: recorded.crash_points,
    };
    let plan = plan_crash_points(&flow);
    let entry_names = entry_names_of_the_steps_after_the_standard_prefix(recorded.stream);
    let is_after_the_standard_prefix =
        |point: &singlefs_checker_tier::crash_amplification::PlannedCrashPoint| {
            let entry_name = point.name.split('.').next().unwrap_or(point.name.as_str());
            point.identity.starts_with(THIS_FILE) && entry_names.contains(&entry_name)
        };
    let points_after_the_standard_prefix = plan
        .points
        .iter()
        .filter(|point| is_after_the_standard_prefix(point))
        .count();
    assert!(
        points_after_the_standard_prefix > 0,
        "流 {flow_name} 在标准前缀之后有点（截断那几步）"
    );
    for observer in EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS {
        let nodes = nodes_of_the_truncation_steps_observer(recorded.stream, *observer);
        let mut points_with_the_observer = 0usize;
        for point in &plan.points {
            let held = nodes.iter().any(|node| node.holds_the_point(point));
            assert_eq!(
                held,
                is_after_the_standard_prefix(point),
                "流 {flow_name}：点 {} 挂没挂观察者 {} 与它在不在 C 之后那几步上不一致",
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
            "流 {flow_name}：观察者 {} 挂上的点数等于 C 之后那几步的点数",
            observer.name()
        );
    }
    points_after_the_standard_prefix
}

/// 一条截断流按环境跑一趟崩溃放量（展开上限、库目录、判器照 `common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line`），
/// 挂两张登记表上的观察者：标准前缀节点上的（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，可写挂载续做，代码在 `common_crash_point_observers`，
/// 另读删除流的发布表）与截断那几步节点上的（[`OBSERVERS_ON_THE_TRUNCATION_STEPS`]，这一份的两个，另读落点表与这条流的每一版）。同整理流的
/// `common_compaction_crash_point_observers::amplify_a_compaction_flow`：观察者走两段，录入段在 CPU 上看（交回镜像的按恢复镜像去重），
/// 判段在挂完的盘上判池级 checker（配了 GPU 上卡）；两段各打一行。观察者计数的断言每一趟都判（这一趟拼的与库里复用的块读回来的合起来）：
/// 每个观察者至少看过一个状态、一个状态都不判红；落点那一个看的每个状态都落到流里某一版或 C 之前；两个可写挂载的观察者每个状态都挂过一次、
/// 挂完的盘都过了池级 checker。只录、只排派活计划、叫停、判段报本次未跑的那一趟不判。
///
/// # Panics
/// 这条流 C 之后有一步不在截断那几步的登记表里；或上面任何一条断言不成立。
#[cfg(feature = "verdict-store")]
pub fn amplify_a_truncation_flow(
    flow_name: &str,
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
    let landed_versions = recorded.landed_versions.as_slice();
    let observe_the_landed_version_of_the_flow =
        |_ordinal: u64,
         _image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_landed_version(landed_versions, report, counts);
        };
    let truncation_mount_continuation_inputs = &recorded.truncation_mount_continuation_inputs;
    let observe_the_mount_continuing_the_truncation_of_the_flow =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_truncation(
                truncation_mount_continuation_inputs,
                image,
                report,
                counts,
            )
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
    for truncation_observer in EVERY_OBSERVER_OF_THE_TRUNCATION_STEPS {
        observers.push(ObserverOnTheNodes {
            observer_name: observer_name(truncation_observer.name()),
            observer_code_file: THIS_FILE.to_string(),
            extra_inputs_digest: match truncation_observer {
                ObserverOfTheTruncationSteps::LandedVersion => {
                    digest_of_the_landed_versions(landed_versions)
                }
                ObserverOfTheTruncationSteps::MountContinuingTheTruncation => {
                    truncation_mount_continuation_inputs.digest()
                }
            },
            nodes: nodes_of_the_truncation_steps_observer(recorded.stream, *truncation_observer),
            observe: match truncation_observer {
                ObserverOfTheTruncationSteps::LandedVersion => {
                    ObserverCode::CountsOnly(&observe_the_landed_version_of_the_flow)
                }
                ObserverOfTheTruncationSteps::MountContinuingTheTruncation => {
                    ObserverCode::HandsBackAnImage {
                        observe: &observe_the_mount_continuing_the_truncation_of_the_flow,
                        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                    }
                }
            },
        });
    }
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            flow_name,
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            recorded.judged_root_index,
            &recorded.versions,
            &[],
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
        ObserverOfTheTruncationSteps::LandedVersion.name(),
        ObserverOfTheTruncationSteps::MountContinuingTheTruncation.name(),
    ];
    for name in observer_names {
        let states = observing
            .states_of_each_observer
            .get(name)
            .copied()
            .unwrap_or(0);
        assert!(
            states > 0,
            "流 {flow_name} 经过标准前缀与截断那几步，观察者 {name} 一个状态都没看"
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
            assert_eq!(
                count_named("continuation_steps_numbered_next_in_the_instance"),
                (1..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1)
                    .map(|steps| {
                        u64::try_from(steps).expect("步数装得进 u64")
                            * count_named(&format!("mount_continuation_steps_{steps}"))
                    })
                    .sum::<u64>(),
                "流 {flow_name}：续做发成的每一步都核了代与事务号"
            );
        } else if name == ObserverOfTheTruncationSteps::LandedVersion.name() {
            let mut landed_counters: Vec<&'static str> = vec![LANDED_BEFORE_THE_STEPS_COUNTER];
            for (_version, counter) in landed_versions {
                if !landed_counters.contains(counter) {
                    landed_counters.push(counter);
                }
            }
            assert_eq!(
                landed_counters
                    .iter()
                    .map(|counter| count_named(counter))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：截断那几步上的每个状态都落到流里某一版或 C 之前"
            );
        } else {
            assert_eq!(
                (0..=TRUNCATION_CONTINUATION_PUBLISHES_AT_MOST)
                    .map(|publishes| count_named(&truncation_continuation_counter(publishes)))
                    .sum::<u64>(),
                states,
                "流 {flow_name}：截断那几步上的每个状态都可写挂载过一次"
            );
            assert!(
                (1..=TRUNCATION_CONTINUATION_PUBLISHES_AT_MOST)
                    .map(|publishes| count_named(&truncation_continuation_counter(publishes)))
                    .sum::<u64>()
                    > 0,
                "流 {flow_name}：落到截断意图或不是最后一批的那几版的状态上截断续做真发过：{counts:?}"
            );
        }
        if name != ObserverOfTheTruncationSteps::LandedVersion.name() {
            assert_eq!(
                count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
                states,
                "流 {flow_name}：观察者 {name} 看的每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
            );
        }
    }
    run
}
