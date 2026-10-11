//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification
//! 删除类发布崩在中间（K-U6 F1：接在多记录流后面 unlink 那个文件）：多记录流「取号 → 暖机 × 2 → A → B → C」之后
//! unlink 第一个文件（C 之后三个数据单元）：意图那次发布 → 按只供测试的批大小 1 删两批 → 最后一批（D8（核心索引结构） 已定项 16），
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；
//! 版本表是 A、B、C 三版有文件，意图那一版起没有文件（D8（核心索引结构） 已定项 16 第 4 条，`crash::Layer0VersionTable`），
//! 两批与最后一批不进表、照比它旧的最新那一项（意图那一版）判。另逐状态核三样（快档与全量都核）：
//! - 多记录发布的发布边界（照多记录流）；
//! - 删除那几次发布崩在中间：这次发布的单元写落了至少一个、记录一份都没落、根没落 ⇒ 实际走的根是上一版
//!   （意图那次的上一版是 C，读回 C 的内容；某一批的上一版是意图或上一批，nlink = 0，答没有文件）；
//! - 可写挂载一次：续做把队列走到空（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」、已定项 16 第 6 条），
//!   续做的步数等于恢复落到的那一版还差几步；续做的每一批落在这次挂载的实例上，txg 与事务号都是这个实例的下一个、连号
//!   （已定项 16 第 2 条末句）；挂完的盘再跑一遍池级 checker，一条不变量都不许判违例（照 `assert_clean` 的口径，已定项 16 第 7 条 ③；
//!   判决 `research/prompts/m3-batch33r-code-r1-main-verification.md` 第三节第 2 条；这一个观察者与删除流的录法住在公共模块
//!   `tests/common_crash_point_observers/mod.rs`、删除流的录法住在公共模块 `tests/common_crash_points/mod.rs`，崩溃放量那一路照登记表把它挂在标准前缀的节点上，
//!   崩溃点重放合并第三轮判决 J4）。挂载是每个状态上的重活，观察者放在工作线程上跑
//!   （`crash::Layer0StateObserving::OnTheWorkerThreads`），一个工作线程、四个工作线程与观察者在调用线程上三种跑法的计数逐项相同（快档对拍）。
//!
//! **比已有的流多罩了什么**：已有的层 0 流上没有删除类发布；这里意图那次（13 个单元：墓碑容器、`deleted_inodes` 根、inode 叶容器与根……）、
//! 两批（12、11 个单元）、最后一批（12 个单元：inode 叶容器删空不写，退役树开第一片容器、写根）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；
//! 多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做），与挂完的盘上那一遍池级 checker。
//!
//! 快档只展开写数小于 10 的段（七个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（层 0 的枚举域 1463812278 个状态，
//! 七个单元段占 1463812089；每次写只取两态时的闭式 1463812228），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;

use std::num::NonZeroUsize;

use common::{build_pool, file_content, geometry, BuiltPool};
use common_crash_point_observers::{
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state,
    observe_the_mount_continuation, MountContinuationInputs,
};
use common_crash_points::{
    content_needing, publish_the_next_unlink_step, publish_the_unlink_intent, sequential_write,
    DELETION_BATCHES_BEFORE_THE_FINAL_BATCH, UNLINKED_FILE,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_against_the_version_table_or_one_shard, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, Layer0EnumerationOutcome,
    Layer0ObserverCounts, Layer0Parallelism, Layer0SegmentExpansion, Layer0SliceLength,
    Layer0StateObserving, Layer0Tally, Layer0VersionTable, Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, publishes_in, writes_and_segments, CrashImage, MemoryPool,
    PublishedVersion, PublishedVersionWithoutAFile, RecordStreamContinuity, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 全量那一趟的流名：进断点续跑的进度文件名与双机分片的账本名（`singlefs_checker_tier::layer0_progress`）。
const FULL_ENUMERATION_STREAM_NAME: &str = "unlink_after_the_multi_record_publish";

/// 快档对拍里多线程那一趟的工作线程数。
const WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON: usize = 4;

/// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
        .map(|(index, _)| index)
        .collect()
}

fn version_of(output: &TransactionOutput) -> (InstanceGeneration, CheckpointTxg) {
    (output.root.instance, output.root.checkpoint_txg)
}

/// 一次多条记录的发布（照多记录流）：记录按 jsn 升序、每条两盘各一份的写表下标，它与上一版的根，上一版的内容。
struct MultiRecordPublish {
    record_writes_in_order: Vec<Vec<usize>>,
    version: (InstanceGeneration, CheckpointTxg),
    version_before: (InstanceGeneration, CheckpointTxg),
    content_before: Vec<u8>,
}

impl MultiRecordPublish {
    fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
        copies.iter().any(|copy| persisted[*copy])
    }

    fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
        let first = self.record_writes_in_order.first().expect("至少两条记录");
        let last = self.record_writes_in_order.last().expect("至少两条记录");
        Self::is_any_copy_persisted(persisted, first)
            && !Self::is_any_copy_persisted(persisted, last)
    }
}

fn multi_record_publish(
    writes: &[RetainedWrite],
    output: &TransactionOutput,
    before: &TransactionOutput,
    content_before: Vec<u8>,
) -> MultiRecordPublish {
    let mut record_writes_in_order: Vec<Vec<usize>> = output
        .earlier_records_of_this_publish
        .iter()
        .map(|earlier| writes_to_record(writes, earlier.record.counter))
        .collect();
    record_writes_in_order.push(writes_to_record(writes, output.record.counter));
    for copies in &record_writes_in_order {
        assert_eq!(copies.len(), 2, "每条记录两盘各一份");
    }
    MultiRecordPublish {
        record_writes_in_order,
        version: version_of(output),
        version_before: version_of(before),
        content_before,
    }
}

/// 删除那几次发布是哪一种：上一版带不带文件分得开读回什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletionPublishKind {
    /// 意图那次：上一版是 C，第一个文件活着。
    Intent,
    /// 不是最后一批的一批：上一版是意图或上一批，第一个文件 nlink = 0。
    Batch,
    /// 最后一批：上一版是最后一个不是最后一批的批。
    FinalBatch,
}

/// 删除的一次发布在写表里的样子：它的单元写、它那一条记录的两份写、它的根槽写，它上一版的根（它自己那一版落到之后续做还差几步，
/// 在可写挂载续做观察者另读的输入里：`common_crash_point_observers::MountContinuationInputs`）。
#[derive(Debug)]
struct DeletionPublish {
    kind: DeletionPublishKind,
    unit_writes: Vec<usize>,
    record_copies: Vec<usize>,
    root_write: usize,
    version_before: (InstanceGeneration, CheckpointTxg),
}

impl DeletionPublish {
    /// 这一状态停在这次发布中间：单元写落了至少一个，记录一份都没落，根没落。
    fn is_cut_after_some_unit_and_before_its_record(&self, persisted: &[bool]) -> bool {
        self.unit_writes.iter().any(|write| persisted[*write])
            && !self.record_copies.iter().any(|copy| persisted[*copy])
            && !persisted[self.root_write]
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    /// A、B、C 三版有文件。
    versions: Vec<PublishedVersion>,
    /// 意图那一版起没有文件（两批与最后一批不进表，照比它旧的最新那一项判）。
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    multi_record_publishes: Vec<MultiRecordPublish>,
    deletion_publishes: Vec<DeletionPublish>,
    /// C 那一版：意图那次的上一版，读回它的内容。
    content_before_the_deletion: Vec<u8>,
    /// 可写挂载续做观察者另读的输入（公共模块 `MountContinuationInputs`）：带文件的 A、B、C，删除那几步各自落到之后续做还差几步。
    mount_continuation_inputs: MountContinuationInputs,
}

impl Prepared {
    fn version_table(&self) -> Layer0VersionTable<'_> {
        Layer0VersionTable::new(&self.versions, &self.versions_without_a_file)
    }
}

/// 多记录流的三次带文件的发布（A 在 `build_pool` 里）之后 unlink 第一个文件走到最后一批：交回 B、C、意图、各批的输出。
struct RecordedStream {
    pool: BuiltPool,
    first: TransactionOutput,
    second: TransactionOutput,
    second_content: Vec<u8>,
    third: TransactionOutput,
    third_content: Vec<u8>,
    deletion_steps: Vec<TransactionOutput>,
}

fn record_the_stream(tag: &str) -> RecordedStream {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(third.data_pointers.len(), 3, "C 之后第一个文件三个数据单元");
    // 步数按计划定死（意图、两批、最后一批），不按「队列空了没有」停：最后一批留着队列里的号时（改坏「最后一批与删意图同事务」），
    // 录流照样录完，红在判器上（池级 checker I-9.5、可写挂载续做），不红在搭建上。
    let mut deletion_steps = vec![publish_the_unlink_intent(&mut pool)];
    for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
        deletion_steps.push(publish_the_next_unlink_step(&mut pool));
    }
    RecordedStream {
        pool,
        first,
        second,
        second_content,
        third,
        third_content,
        deletion_steps,
    }
}

/// 删除那几步的形状：意图之后 nlink = 0、单元照旧；两批各少一个单元、nlink 仍 0、号还在队列里；最后一批删记录与最后一个单元。
/// 最后一批之后队列空不空不在这里判：那是「最后一批与删意图同事务」，留给判器（池级 checker I-9.5、可写挂载续做）红。
fn assert_the_shape_of_the_deletion_steps(deletion_steps: &[TransactionOutput]) {
    assert_eq!(
        deletion_steps.len(),
        1 + DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1,
        "意图、两批、最后一批"
    );
    for (step_index, step) in deletion_steps
        .iter()
        .enumerate()
        .take(1 + DELETION_BATCHES_BEFORE_THE_FINAL_BATCH)
    {
        let record = step
            .inode_record
            .expect("最后一批之前第一个文件那条记录还在");
        assert_eq!(
            record.nlink, NLINK_OF_AN_INODE_BEING_DELETED,
            "第 {step_index} 步之后 nlink = 0"
        );
        assert_eq!(
            step.deleted_inodes,
            vec![UNLINKED_FILE],
            "第 {step_index} 步之后号在队列里"
        );
        assert_eq!(
            step.data_pointers.len(),
            3 - step_index,
            "意图不动单元，之后每批释放尾部一个"
        );
    }
    let final_batch = deletion_steps.last().expect("至少一步");
    assert!(
        final_batch.inode_record.is_none() && final_batch.data_pointers.is_empty(),
        "最后一批删记录、释放最后一个单元"
    );
}

fn prepare(tag: &str) -> Prepared {
    let recorded = record_the_stream(tag);
    assert_the_shape_of_the_deletion_steps(&recorded.deletion_steps);
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![
            2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2,
            22, 2, 1, 2, 24, 2, 1, 2
        ],
        "前十九段同多记录流；之后意图那次 26 个单元写（13 个单元两盘各一份）、记录 2、根 1、轮换 2；两批 24、22 个单元写；\
         最后一批 24 个单元写（12 个单元：extent 根、inode 根、分配记录树五个节点、记账树、退役容器与退役树根、映射树、树表）；\
         各自记录 2、根 1、轮换 2"
    );
    let every_write_persisted = vec![true; writes.len()];
    let publishes = publishes_in(
        &writes,
        &every_write_persisted,
        RecordStreamContinuity::OneRecording,
    );
    let mut deletion_publishes = Vec::new();
    let mut version_before = version_of(&recorded.third);
    for (step_index, step) in recorded.deletion_steps.iter().enumerate() {
        let version = version_of(step);
        let publish = publishes
            .iter()
            .find(|publish| {
                publish.instance == version.0 .0 && publish.checkpoint_txg == version.1 .0
            })
            .expect("删除的每一步在写表里有它的根槽写");
        let record_copies = writes_to_record(&writes, step.record.counter);
        assert_eq!(record_copies.len(), 2, "删除的每一步一条记录、两盘各一份");
        assert_eq!(publish.records, record_copies, "删除的一步恰一条记录");
        let kind = match step_index {
            0 => DeletionPublishKind::Intent,
            index if index == recorded.deletion_steps.len() - 1 => DeletionPublishKind::FinalBatch,
            _ => DeletionPublishKind::Batch,
        };
        deletion_publishes.push(DeletionPublish {
            kind,
            unit_writes: publish.units.clone(),
            record_copies,
            root_write: publish.root,
            version_before,
        });
        version_before = version;
    }
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indexes.len(),
        9,
        "暖机两次、A、B、C、意图、两批、最后一批各一条根槽写"
    );
    let versions = vec![
        PublishedVersion {
            instance: recorded.first.root.instance,
            checkpoint_txg: recorded.first.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: recorded.second.root.instance,
            checkpoint_txg: recorded.second.root.checkpoint_txg,
            content: recorded.second_content.clone(),
        },
        PublishedVersion {
            instance: recorded.third.root.instance,
            checkpoint_txg: recorded.third.root.checkpoint_txg,
            content: recorded.third_content.clone(),
        },
    ];
    let intent = recorded.deletion_steps.first().expect("意图那一步");
    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
        instance: intent.root.instance,
        checkpoint_txg: intent.root.checkpoint_txg,
    }];
    let multi_record_publishes = vec![
        multi_record_publish(&writes, &recorded.second, &recorded.first, file_content()),
        multi_record_publish(
            &writes,
            &recorded.third,
            &recorded.second,
            recorded.second_content.clone(),
        ),
    ];
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        versions_without_a_file,
        multi_record_publishes,
        deletion_publishes,
        mount_continuation_inputs: MountContinuationInputs::of_the_publishes(
            &[&recorded.first, &recorded.second, &recorded.third],
            &recorded.deletion_steps,
        ),
        content_before_the_deletion: recorded.third_content,
    }
}

/// 每个状态上核发布边界（照多记录流）：一次发布的第一条记录落了、末条没落 ⇒ 走的是上一版的根、读回上一版。
fn observe_the_publish_boundary(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut Layer0ObserverCounts,
) {
    for (publish_index, publish) in prepared.multi_record_publishes.iter().enumerate() {
        if !publish.is_cut_before_its_last_record(&image.persisted) {
            continue;
        }
        counts.add(&format!("cut_in_multi_record_publish_{publish_index}"), 1);
        assert_eq!(
            report.effective_root,
            Some(publish.version_before),
            "第 {publish_index} 次多记录发布的末条没落：那次发布整体不施加，走的是上一版的根（{:?}）",
            publish.version
        );
        let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
            panic!("要读回上一版，实际 {:?}", report.outcome);
        };
        assert!(
            *content == publish.content_before,
            "读回上一版：文件是旧长度 {} 字节，实际 {} 字节",
            publish.content_before.len(),
            content.len()
        );
    }
}

/// 每个状态上核删除的中间态：删除的一次发布停在中间（单元写落了、记录没落、根没落）⇒ 走的是上一版的根；
/// 意图那次的上一版是 C，读回 C 的内容；某一批的上一版第一个文件 nlink = 0，答没有文件（D8（核心索引结构） 已定项 16 第 4 条）。
fn observe_the_deletion_cut_in_the_middle(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut Layer0ObserverCounts,
) {
    for (publish_index, publish) in prepared.deletion_publishes.iter().enumerate() {
        if !publish.is_cut_after_some_unit_and_before_its_record(&image.persisted) {
            continue;
        }
        counts.add(&format!("cut_in_deletion_publish_{publish_index}"), 1);
        assert_eq!(
            report.effective_root,
            Some(publish.version_before),
            "删除的第 {publish_index} 步（{:?}）停在中间：那次发布不施加，走的是上一版的根",
            publish.kind
        );
        match (publish.kind, &report.outcome) {
            (DeletionPublishKind::Intent, RecoveryOutcome::FileRead { content, .. }) => {
                assert!(
                    *content == prepared.content_before_the_deletion,
                    "崩在意图那次中间读回删之前那一版：{} 字节，实际 {} 字节",
                    prepared.content_before_the_deletion.len(),
                    content.len()
                );
            }
            (
                DeletionPublishKind::Batch | DeletionPublishKind::FinalBatch,
                RecoveryOutcome::NoFile { .. },
            ) => {}
            (kind, outcome) => panic!(
                "删除的第 {publish_index} 步（{kind:?}）停在中间：意图那次要读回 C、各批要答没有文件，实际 {outcome:?}"
            ),
        }
    }
}

/// 每个状态上核的三样：发布边界、删除的中间态、可写挂载续做（连同续做各步的代与事务号、挂完的盘过池级 checker）。
/// 只读 `prepared`，各工作线程共用一份。
fn observe_every_state(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut Layer0ObserverCounts,
) {
    observe_the_publish_boundary(prepared, image, report, counts);
    observe_the_deletion_cut_in_the_middle(prepared, image, report, counts);
    observe_the_mount_continuation(&prepared.mount_continuation_inputs, image, report, counts);
}

fn assert_clean(tally: &Layer0Tally) {
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
) -> Layer0SegmentExpansion {
    if segment.len() < 10 {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
    }
}

/// 逐状态的观察者在哪个线程上看。
#[derive(Clone, Copy, Debug)]
enum ObserverThread {
    WorkerThreads,
    CallingThread,
}

/// 快档那一趟：七个单元段不展开，每个状态交 [`observe_every_state`]，不留进度文件。
fn enumerate_the_quick_tier(
    prepared: &Prepared,
    parallelism: Layer0Parallelism,
    observer_thread: ObserverThread,
) -> Layer0Tally {
    let observe_on_the_worker_threads: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts)
          + Sync) = &|image, report, counts| observe_every_state(prepared, image, report, counts);
    let observe_on_the_calling_thread: &mut dyn FnMut(
        &CrashImage<'_>,
        &RecoveryReport,
        &mut Layer0ObserverCounts,
    ) = &mut |image, report, counts| observe_every_state(prepared, image, report, counts);
    let observing = match observer_thread {
        ObserverThread::WorkerThreads => {
            Layer0StateObserving::OnTheWorkerThreads(observe_on_the_worker_threads)
        }
        ObserverThread::CallingThread => {
            Layer0StateObserving::OnTheCallingThread(observe_on_the_calling_thread)
        }
    };
    match enumerate_layer0_against_the_version_table_or_one_shard(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &expansion_of_the_segments_shorter_than_ten,
        parallelism,
        observing,
        &Layer0Resume::NoProgressFile,
    ) {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => panic!(
            "不留进度文件、不分片的一趟交回整条流的计数，实际写了第 {} 片的账本",
            written.shard.text()
        ),
    }
}

fn worker_threads_given_by_the_test(worker_threads: usize) -> Layer0Parallelism {
    Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("工作线程数不是 0"),
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: Layer0SliceLength::ScaledToWorkerThreads,
    }
}

/// 平时跑的那一份：七个单元段不展开，其余每段任意真子集；每个状态核发布边界、删除的中间态，再在工作线程上可写挂载一次核续做走到空。
#[test]
fn every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty(
) {
    let prepared = prepare("unlink-after-multi-record-layer0-fast");
    let tally = enumerate_the_quick_tier(
        &prepared,
        Layer0Parallelism::from_environment(),
        ObserverThread::WorkerThreads,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        closed_form_state_count(&expanded),
        1 + 10 * 3 + 7 * 3 + 9 + 15 + 63,
        "每次写只取两态时展开的段按闭式数：1 + 十个系统配置槽 2 写段各 3 + 七个记录 2 写段各 3 + 九个 1 写段各 1 + 4 写段 15 + 6 写段 63"
    );
    assert_eq!(
        tally.states,
        layer0_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按层 0 的枚举域数（原地覆写取三态）"
    );
    assert_eq!(
        tally.states,
        1 + 10 * (3 * 3 - 1) + 7 * 3 + 9 + 15 + 63,
        "1 + 十个系统配置槽 2 写段各 3² − 1 + 七个记录 2 写段各 3 + 九个 1 写段各 1 + 4 写段 15 + 6 写段 63"
    );
    assert_eq!(
        tally.observed_states, tally.states,
        "每个状态都在工作线程上看过"
    );
    let counts = &tally.observer_counts;
    assert_eq!(
        [
            counts.count_named("cut_in_multi_record_publish_0"),
            counts.count_named("cut_in_multi_record_publish_1"),
        ],
        [3, 12],
        "崩在多记录发布的记录之间的状态真跑到了（同多记录流）"
    );
    // 单元段不展开：每次删除发布停在中间的状态只有「单元全落、记录一份没落」那一个（记录段的空子集）。
    assert_eq!(
        [0, 1, 2, 3]
            .map(|publish_index| counts
                .count_named(&format!("cut_in_deletion_publish_{publish_index}"))),
        [1, 1, 1, 1],
        "意图、两批、最后一批各自停在中间的状态真跑到了"
    );
    assert_eq!(
        (0..=3)
            .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
            .sum::<u64>(),
        tally.states,
        "每个状态都可写挂载过一次"
    );
    for steps in 1..=3 {
        assert!(
            counts.count_named(&format!("mount_continuation_steps_{steps}")) > 0,
            "恢复落到还差 {steps} 步的那一版、挂载续做 {steps} 步的状态真跑到了"
        );
    }
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
        counts,
        tally.states,
    );
    println!(
        "LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
}

/// 观察者挪进工作线程之后判定不变：同一趟快档，一个工作线程、四个工作线程（观察者都在工作线程上）与观察者在调用线程上，
/// 三份计数（状态数、违例、checker 逐条、发现表、观察者计数）逐项相同。
#[test]
fn the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and_with_the_observer_on_the_calling_thread(
) {
    let prepared = prepare("unlink-after-multi-record-layer0-threads");
    let on_one_worker_thread = enumerate_the_quick_tier(
        &prepared,
        worker_threads_given_by_the_test(1),
        ObserverThread::WorkerThreads,
    );
    let on_several_worker_threads = enumerate_the_quick_tier(
        &prepared,
        worker_threads_given_by_the_test(WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON),
        ObserverThread::WorkerThreads,
    );
    let observed_on_the_calling_thread = enumerate_the_quick_tier(
        &prepared,
        worker_threads_given_by_the_test(WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON),
        ObserverThread::CallingThread,
    );
    assert_eq!(
        on_several_worker_threads, on_one_worker_thread,
        "观察者在工作线程上：四个工作线程与一个工作线程的计数逐项相同"
    );
    assert_eq!(
        on_several_worker_threads, observed_on_the_calling_thread,
        "观察者在工作线程上与在调用线程上的计数逐项相同"
    );
    assert_eq!(
        on_several_worker_threads.observed_states, on_several_worker_threads.states,
        "每个状态都看过"
    );
}

/// 全量：35 段。七个单元段各 2^n − 1（n = 26、24、22、24 是删除那四次），每个状态两遍恢复 + checker，逐状态核发布边界、删除的中间态，
/// 并在工作线程上可写挂载一次核续做走到空。登记成崩溃枚举用例，checker-tier-layer0-replay 认下面打印的
/// `LAYER0_UNLINK_AFTER_MULTI_RECORD` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
#[test]
#[ignore = "全量 1463812278 个状态（十四亿多）、每个两遍恢复 + checker + 一次可写挂载；门禁 checker-tier-layer0-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhaustive_and_clean() {
    let prepared = prepare("unlink-after-multi-record-layer0-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 10 * 3
            + 7 * 3
            + 9
            + ((1 << 24) - 1)
            + ((1 << 28) - 1)
            + 15
            + ((1 << 30) - 1)
            + 63
            + ((1 << 26) - 1)
            + ((1 << 24) - 1)
            + ((1 << 22) - 1)
            + ((1 << 24) - 1),
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，三十五段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 1_463_812_228);
    let closed_form = layer0_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 10 * (3 * 3 - 1)
            + 7 * 3
            + 9
            + ((1 << 24) - 1)
            + ((1 << 28) - 1)
            + 15
            + ((1 << 30) - 1)
            + 63
            + ((1 << 26) - 1)
            + ((1 << 24) - 1)
            + ((1 << 22) - 1)
            + ((1 << 24) - 1),
        "枚举域的闭式：十个系统配置槽 2 写段各 3² − 1、七个记录 2 写段各 3、九个 1 写段、七个单元段各 2^n − 1、4 写段 15、6 写段 63"
    );
    assert_eq!(closed_form, 1_463_812_278);
    let observe_on_the_worker_threads: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts)
          + Sync) = &|image, report, counts| observe_every_state(&prepared, image, report, counts);
    let tally = match enumerate_layer0_against_the_version_table_or_one_shard(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &full_expansion,
        Layer0Parallelism::from_environment(),
        Layer0StateObserving::OnTheWorkerThreads(observe_on_the_worker_threads),
        &Layer0Resume::from_environment(FULL_ENUMERATION_STREAM_NAME),
    ) {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_written) => return,
    };
    println!(
        "LAYER0_UNLINK_AFTER_MULTI_RECORD states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form);
    assert_eq!(tally.observed_states, closed_form, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        [
            counts.count_named("cut_in_multi_record_publish_0"),
            counts.count_named("cut_in_multi_record_publish_1"),
        ],
        [3, 12]
    );
    // 单元段 n 个写里至少落一个的真子集 2^n − 2，加记录段的空子集（单元全落、记录一份没落）。
    assert_eq!(
        [0, 1, 2, 3]
            .map(|publish_index| counts
                .count_named(&format!("cut_in_deletion_publish_{publish_index}"))),
        [(1 << 26) - 1, (1 << 24) - 1, (1 << 22) - 1, (1 << 24) - 1],
        "意图、两批、最后一批各自停在中间的每个状态都核到了"
    );
    assert_eq!(
        (0..=3)
            .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
            .sum::<u64>(),
        closed_form,
        "每个状态都可写挂载过一次"
    );
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
        counts,
        closed_form,
    );
    assert_clean(&tally);
}

/// 录制器那一份造法录下的删除流：起点镜像、写表、段、崩溃点、被判的根槽写（最后一批的那一条）、版本表与「没有文件的那几版」。
#[cfg(feature = "verdict-store")]
struct RecordedThroughTheCrashPointRecorder {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

/// 录制器那一份造法：取号加暖机、新池新建文件、B、C 四个点由公共模块设（与多记录流同一个函数，共享到 C，崩溃点重放合并第三轮判决 J3），
/// 意图与每一批各一个点；`stop_after_the_multi_record_publishes` 为真时录到 C 就停（多记录流那一份，比共享用）。
#[cfg(feature = "verdict-store")]
fn record_through_the_crash_point_recorder(
    tag: &str,
    stop_after_the_multi_record_publishes: bool,
) -> RecordedThroughTheCrashPointRecorder {
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    let publishes =
        common_crash_points::record_the_multi_record_publishes(&mut recorder, &mut pool);
    let mut versions_without_a_file = Vec::new();
    if !stop_after_the_multi_record_publishes {
        let mut deletion_steps = vec![recorder.step(file!(), "publish_unlink_intent", || {
            publish_the_unlink_intent(&mut pool)
        })];
        for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
            deletion_steps.push(recorder.step(file!(), "publish_unlink_step", || {
                publish_the_next_unlink_step(&mut pool)
            }));
        }
        assert_the_shape_of_the_deletion_steps(&deletion_steps);
        let intent = deletion_steps.first().expect("意图那一步");
        versions_without_a_file.push(PublishedVersionWithoutAFile {
            instance: intent.root.instance,
            checkpoint_txg: intent.root.checkpoint_txg,
        });
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) =
        singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(
            &operations[pool.mkfs_operation_count..],
            &geometry(),
        );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedThroughTheCrashPointRecorder {
        base,
        writes,
        segments,
        crash_points,
        judged_root_index,
        versions: publishes.versions(),
        versions_without_a_file,
    }
}

/// 崩溃放量的形态（起点是第 32 批 Q 的草稿）：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。
/// 版本表带「没有文件的那几版」（意图那一版，`CrashFlow::versions_without_a_file`，第 33 批 R 报告「停下交主 agent」第一项），
/// 两批与最后一批不进表、照比它旧的最新那一项判：不带它时各批报没有文件判成违例、放量红。展开上限默认 6。
/// 登记表挂在标准前缀节点上的观察者（可写挂载续做）照表跑（崩溃点重放合并第三轮判决 J4），与多记录流那一项在同一批块上共享。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean() {
    let recorded = record_through_the_crash_point_recorder(
        "crash-amplification-unlink-after-multi-record",
        false,
    );
    common_crash_point_observers::amplify_a_flow_through_the_standard_prefix(
        "unlink-after-the-multi-record-publish",
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &recorded.versions_without_a_file,
        recorded.crash_points,
        6,
    );
}

/// 两份造法写表摘要相等（崩溃点重放合并第三轮判决 J3，第一轮判决 K2-2）：层 0 那一份（[`prepare`]，不录崩溃点）与录制器那一份
/// （B、C 两次顺序写挪进公共模块之后）录下的起点镜像与整张带屏障的写表逐字节相同、段一样切、被判的根槽写与版本表相同。
/// 两份都调公共模块里同一个 `sequential_write`；层 0 那一份的写法与挪之前逐字相同。摘要打一行 `CRASH_POINT_REPLAY_WRITE_TABLE`。
#[cfg(feature = "verdict-store")]
#[test]
fn the_unlink_stream_writes_the_same_table_through_the_crash_point_recorder_as_without_it() {
    use singlefs_checker_tier::crash_identity::{step_write_table_with_barriers, SectorImage};
    let without_the_recorder = prepare("unlink-write-table-without-the-recorder");
    let through_the_recorder =
        record_through_the_crash_point_recorder("unlink-write-table-through-the-recorder", false);
    let digests_of = |base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>]| {
        (
            SectorImage::of_base(base).digest(),
            step_write_table_with_barriers(writes, segments, 0..writes.len()),
        )
    };
    let (base_digest, write_table_digest) = digests_of(
        &without_the_recorder.base,
        &without_the_recorder.writes,
        &without_the_recorder.segments,
    );
    assert_eq!(
        digests_of(
            &through_the_recorder.base,
            &through_the_recorder.writes,
            &through_the_recorder.segments
        ),
        (base_digest, write_table_digest),
        "两份造法的起点镜像与整张带屏障的写表逐字节相同"
    );
    assert_eq!(
        through_the_recorder.segments, without_the_recorder.segments,
        "段一样切"
    );
    assert_eq!(
        through_the_recorder.judged_root_index, without_the_recorder.judged_root_index,
        "被判的根槽写都是最后一批的那一条"
    );
    assert_eq!(
        (
            &through_the_recorder.versions,
            &through_the_recorder.versions_without_a_file
        ),
        (
            &without_the_recorder.versions,
            &without_the_recorder.versions_without_a_file
        ),
        "版本表与「没有文件的那几版」相同"
    );
    let hexadecimal =
        |digest: &[u8]| -> String { digest.iter().map(|byte| format!("{byte:02x}")).collect() };
    println!(
        "CRASH_POINT_REPLAY_WRITE_TABLE flow=unlink-after-multi-record-publish base_digest={} write_table_digest={}",
        hexadecimal(&base_digest),
        hexadecimal(&write_table_digest)
    );
}

/// 删除流与多记录流共享到 C（崩溃点重放合并第三轮正文 H3，主 agent 定）：计划里的点细到每个屏障段（`<入口>.<周期>.<步>`），按点名的前缀认
/// 公共模块设的四个节点——取号加暖机、新池新建文件、B、C（B、C 都是 `sequential_write`：同一个入口里点名重复出现，就是下一个节点起头）。
/// 两条流这四个节点的每一个点身份、判器另读的输入、计划哈希、状态序号区间与块逐个相同（块键全同、判定共享）；整条流最后全部持久那一个状态
/// 不归 C，各自另立成流自己的末节点 `end_of_flow`（`crash_amplification::plan_crash_points`）：多记录流在 C 之后只剩它，删除流在 C 之后
/// 还有意图、各批与它自己的末节点；两条流的末节点身份与计划哈希各不相同。
#[cfg(feature = "verdict-store")]
#[test]
fn the_unlink_stream_shares_the_nodes_through_c_with_the_multi_record_publish_stream_and_each_stream_ends_in_its_own_node(
) {
    use singlefs_checker_tier::crash_amplification::{
        plan_crash_points, CrashFlow, PlannedCrashPoint, END_OF_FLOW_NODE_NAME,
    };
    let unlink = record_through_the_crash_point_recorder("unlink-shares-the-prefix", false);
    let multi_record =
        record_through_the_crash_point_recorder("multi-record-shares-the-prefix", true);
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 6 {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let plan_of = |recorded: &RecordedThroughTheCrashPointRecorder| {
        plan_crash_points(&CrashFlow {
            base: &recorded.base,
            writes: &recorded.writes,
            segments: &recorded.segments,
            judged_root_index: recorded.judged_root_index,
            versions: &recorded.versions,
            versions_without_a_file: &recorded.versions_without_a_file,
            expansion: &expansion,
            crash_points: recorded.crash_points.clone(),
        })
    };
    let unlink_plan = plan_of(&unlink);
    let multi_record_plan = plan_of(&multi_record);
    // 公共模块设的节点：身份以公共模块的路径起头、不是末节点；入口名是点名第一段，同一个入口里点名重复出现就是下一个节点起头
    let common_identity_prefix = format!("{}::", common_crash_points::THIS_FILE);
    let nodes_set_in_the_common_module =
        |points: &[PlannedCrashPoint]| -> Vec<(String, Vec<usize>)> {
            let mut nodes: Vec<(String, Vec<usize>, Vec<String>)> = Vec::new();
            for (index, point) in points.iter().enumerate() {
                if !point.identity.starts_with(&common_identity_prefix)
                    || point.name == END_OF_FLOW_NODE_NAME
                {
                    break;
                }
                let entry = point
                    .name
                    .split('.')
                    .next()
                    .expect("点名以入口名起头")
                    .to_string();
                let starts_a_node = nodes.last().is_none_or(|(last_entry, _, names)| {
                    *last_entry != entry || names.contains(&point.name)
                });
                if starts_a_node {
                    nodes.push((entry, Vec::new(), Vec::new()));
                }
                let (_entry, indexes, names) = nodes.last_mut().expect("刚放进去一个节点");
                indexes.push(index);
                names.push(point.name.clone());
            }
            nodes
                .into_iter()
                .map(|(entry, indexes, _names)| (entry, indexes))
                .collect()
        };
    let multi_record_nodes = nodes_set_in_the_common_module(&multi_record_plan.points);
    assert_eq!(
        multi_record_nodes
            .iter()
            .map(|(entry, _indexes)| entry.as_str())
            .collect::<Vec<&str>>(),
        vec![
            "acquire_instance_and_warm_up",
            "publish_first_file",
            "sequential_write",
            "sequential_write"
        ],
        "多记录流公共模块设的四个节点：取号加暖机、新池新建文件、B、C"
    );
    assert_eq!(
        nodes_set_in_the_common_module(&unlink_plan.points),
        multi_record_nodes,
        "删除流经过同样四个节点、每个节点同样几个点"
    );
    let points_through_c = 1 + *multi_record_nodes
        .last()
        .and_then(|(_entry, indexes)| indexes.last())
        .expect("C 至少一个点");
    assert_eq!(
        multi_record_plan.points.len(),
        points_through_c + 1,
        "多记录流在 C 之后只剩它自己的末节点"
    );
    assert!(
        unlink_plan.points.len() > points_through_c + 1,
        "删除流在 C 之后还有意图、各批与它自己的末节点"
    );
    for index in 0..points_through_c {
        let (unlink_point, multi_record_point) =
            (&unlink_plan.points[index], &multi_record_plan.points[index]);
        let name = &multi_record_point.name;
        assert_eq!(
            unlink_point.identity, multi_record_point.identity,
            "第 {index} 个点（{name}）两条流是树上同一个节点：设点文件、名字、复用键都相同"
        );
        assert_eq!(
            unlink_point.judge_inputs, multi_record_point.judge_inputs,
            "第 {index} 个点（{name}）判器另读的输入相同"
        );
        assert_eq!(
            unlink_point.plan_hash, multi_record_point.plan_hash,
            "第 {index} 个点（{name}）计划哈希相同：C 也是，整条流最后那一个状态不归它了"
        );
        assert_eq!(
            (unlink_point.ordinals.clone(), unlink_point.blocks()),
            (
                multi_record_point.ordinals.clone(),
                multi_record_point.blocks()
            ),
            "第 {index} 个点（{name}）状态序号区间与块相同：块键全同、判定共享"
        );
    }
    let unlink_end = unlink_plan.points.last().expect("计划至少一个点");
    let multi_record_end = multi_record_plan.points.last().expect("计划至少一个点");
    for end in [unlink_end, multi_record_end] {
        assert_eq!(end.name, END_OF_FLOW_NODE_NAME, "计划最后一项是末节点");
        assert_eq!(
            end.ordinals.end - end.ordinals.start,
            1,
            "末节点只管整条流最后全部持久那一个状态"
        );
    }
    assert_ne!(
        unlink_end.identity, multi_record_end.identity,
        "两条流的末节点各是各的：整张写表叠完的镜像不同"
    );
    assert_ne!(
        unlink_end.plan_hash, multi_record_end.plan_hash,
        "两条流的末节点计划哈希不同"
    );
}
