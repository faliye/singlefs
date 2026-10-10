//! checker 档模块：crash、layer0_progress、writable_mount_phases、crash_amplification
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
//!   判决 `research/prompts/m3-batch33r-code-r1-main-verification.md` 第三节第 2 条）。挂载是每个状态上的重活，观察者放在工作线程上跑
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
#[cfg(feature = "verdict-store")]
mod common_crash_points;

use std::num::NonZeroUsize;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    enumerate_layer0_against_the_version_table_or_one_shard, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, Layer0EnumerationOutcome,
    Layer0ObserverCounts, Layer0Parallelism, Layer0SegmentExpansion, Layer0SliceLength,
    Layer0StateObserving, Layer0Tally, Layer0VersionTable, Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    MountOutput, ShadowLedger,
};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    publish_sequential_write, publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, publishes_in, writes_and_segments, CrashImage, MemoryPool,
    PublishedVersion, PublishedVersionWithoutAFile, RecordStreamContinuity, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::scenario::{deletion_history_batch_size, deletion_history_switches};
use singlefs_harness::segments::StepKind;

/// 全量那一趟的流名：进断点续跑的进度文件名与双机分片的账本名（`singlefs_checker_tier::layer0_progress`）。
const FULL_ENUMERATION_STREAM_NAME: &str = "unlink_after_the_multi_record_publish";

/// unlink 的是第一个文件。
const UNLINKED_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 意图那次的调用方时间比顺序写晚：与那条记录现有的时间不同，意图写的是不是调用方给的那一个分辨得出。
const SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT: u64 = 120;

/// 删除段的批数（不含最后一批）：三个数据单元按批大小 1 走「留 2、留 1、最后一批」（K-U6 翻面观测：删除段少于两批不算数）。
const DELETION_BATCHES_BEFORE_THE_FINAL_BATCH: usize = 2;

/// 快档对拍里多线程那一趟的工作线程数。
const WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON: usize = 4;

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半（与多记录流同一份字节）。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 7) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

fn publish_the_unlink_intent(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT,
        InstanceGeneration(1),
    )
    .expect("unlink 意图");
    pool.output = output.clone();
    output
}

fn publish_the_next_unlink_step(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        deletion_history_batch_size(),
        InstanceGeneration(1),
    )
    .expect("unlink 的一批");
    pool.output = output.clone();
    output
}

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

/// 删除的一次发布在写表里的样子：它的单元写、它那一条记录的两份写、它的根槽写，它与上一版的根。
#[derive(Debug)]
struct DeletionPublish {
    kind: DeletionPublishKind,
    unit_writes: Vec<usize>,
    record_copies: Vec<usize>,
    root_write: usize,
    version: (InstanceGeneration, CheckpointTxg),
    version_before: (InstanceGeneration, CheckpointTxg),
    /// 恢复落到这一版之后，可写挂载的续做还要发几步才走到空。
    continuation_steps_from_this_version: usize,
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
            version,
            version_before,
            continuation_steps_from_this_version: recorded.deletion_steps.len() - 1 - step_index,
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

/// 恢复落到的是哪一类版本：挂载之后该是哪一种池版本、续做还差几步按它定。
#[derive(Clone, Copy, Debug)]
enum LandedVersion<'prepared> {
    /// mkfs 或两次暖机：A 之前，根下面没有树。
    BeforeTheFirstFile,
    /// A、B、C 之一：有文件、队列空。
    WithAFile,
    /// 意图、两批、最后一批之一。
    OneOfTheDeletionPublishes(&'prepared DeletionPublish),
}

fn landed_version<'prepared>(
    prepared: &'prepared Prepared,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> LandedVersion<'prepared> {
    let effective_root = effective_root.expect("快档与全量的每个状态都择到根（oracle 另判）");
    if let Some(deletion_publish) = prepared
        .deletion_publishes
        .iter()
        .find(|publish| publish.version == effective_root)
    {
        return LandedVersion::OneOfTheDeletionPublishes(deletion_publish);
    }
    if prepared
        .versions
        .iter()
        .any(|version| (version.instance, version.checkpoint_txg) == effective_root)
    {
        return LandedVersion::WithAFile;
    }
    LandedVersion::BeforeTheFirstFile
}

/// 这一状态上可写挂载一次（批大小同录流，三个容量照产品路径）：续做把队列走到空，步数等于恢复落到的那一版还差几步；
/// 落到删除那几版上的，挂完是「有树、文件集合为空」那一版；落到 A、B、C 上的挂完照旧有文件；落到 A 之前的挂完照旧没有树。
fn observe_the_mount_continuation(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut Layer0ObserverCounts,
) {
    let materialized = materialized_crash_image(image);
    let mount_parameters = parameters();
    let mut devices: Vec<_> = materialized
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                materialized.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &mount_parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_history_switches(),
    )
    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做把队列走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    let landed = landed_version(prepared, report.effective_root);
    let expected_continuation_steps = match landed {
        LandedVersion::OneOfTheDeletionPublishes(publish) => {
            publish.continuation_steps_from_this_version
        }
        LandedVersion::BeforeTheFirstFile | LandedVersion::WithAFile => 0,
    };
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        expected_continuation_steps,
        "恢复落到 {:?}：续做还差几步就发几步",
        report.effective_root
    );
    counts.add(
        &format!("mount_continuation_steps_{expected_continuation_steps}"),
        1,
    );
    match (&mounted.current, landed) {
        (PoolVersion::WithTreesAndNoFile(current), LandedVersion::OneOfTheDeletionPublishes(_)) => {
            assert!(current.deleted_inodes.is_empty(), "挂完队列是空的");
        }
        (PoolVersion::WithFile(current), LandedVersion::WithAFile) => {
            assert!(
                current.deleted_inodes.is_empty(),
                "删除之前的版本队列本来就空"
            );
        }
        (PoolVersion::WithoutFile(_), LandedVersion::BeforeTheFirstFile) => {
            counts.add("mount_before_the_first_file", 1);
        }
        (current, landed) => panic!(
            "落到删除那几版上的挂完是有树、文件集合为空那一版，落到 A、B、C 上的带文件，落到 A 之前的没有树：落到 {landed:?}，挂完 {current:?}"
        ),
    }
    assert_the_continuation_publishes_take_the_next_numbers_of_the_instance(&mounted.output);
    counts.add(
        "continuation_steps_numbered_next_in_the_instance",
        u64::try_from(mounted.output.deletion_queue_continuation.len()).expect("步数装得进 u64"),
    );
    let mounted_disks = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let violated_after_the_mount: Vec<(&str, String)> = check_pool_image(&mounted_disks)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(reason) => Some((invariant, reason)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    assert!(
        violated_after_the_mount.is_empty(),
        "恢复落到 {:?}、可写挂载续做之后的盘，池级 checker 一条都不许判违例（同 assert_clean 的口径）：{violated_after_the_mount:?}",
        report.effective_root
    );
    counts.add("pool_checker_clean_after_the_mount", 1);
}

/// 这次挂载里一次发布：续做的一步，或续做里为空间推的抬 F 那一串里的一次空发布。
#[derive(Clone, Copy)]
enum PublishOfTheContinuation<'output> {
    DeletionStep(&'output TransactionOutput),
    FloorRaise(&'output TransactionOutput),
}

impl<'output> PublishOfTheContinuation<'output> {
    fn output(self) -> &'output TransactionOutput {
        match self {
            PublishOfTheContinuation::DeletionStep(output)
            | PublishOfTheContinuation::FloorRaise(output) => output,
        }
    }
}

/// 有树的一版（写行、暖机那几次交回的）：续做有步数时它们都有树。
fn version_of_trees_before_the_continuation(version: &PoolVersion) -> &TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => panic!(
            "续做有步数的挂载，续做之前那几次发布都在有树的一版上（写行、暖机交回有树的一版）"
        ),
    }
}

/// D8（核心索引结构） 已定项 16 第 2 条末句：续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
/// 续做的每一步与续做里为空间推的抬 F 按 txg 交错排好，从这次挂载在续做之前发的最后一次（写行、暖机、取号之后推的抬 F，按先后）接起：
/// 每一次都落在这次挂载的实例上、txg 是前一次加一（连号）；续做的每一步的事务号是前一次发布之后这个实例用过的最大事务号加一。
/// 续做没有步数时什么都不判。
fn assert_the_continuation_publishes_take_the_next_numbers_of_the_instance(output: &MountOutput) {
    if output.deletion_queue_continuation.is_empty() {
        return;
    }
    let last_publish_before_the_continuation = output
        .space_admission
        .floor_raises()
        .iter()
        .flat_map(|raised| raised.publishes.iter())
        .last()
        .or_else(|| {
            output
                .warm_up_publishes
                .last()
                .map(version_of_trees_before_the_continuation)
        })
        .unwrap_or_else(|| version_of_trees_before_the_continuation(&output.row_publish));
    assert_eq!(
        last_publish_before_the_continuation.root.instance, output.instance,
        "续做之前那一次发布是这次挂载发的"
    );
    let mut publishes_of_the_continuation: Vec<PublishOfTheContinuation<'_>> = output
        .deletion_queue_continuation
        .iter()
        .map(PublishOfTheContinuation::DeletionStep)
        .chain(
            output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .flat_map(|raised| raised.publishes.iter())
                .map(PublishOfTheContinuation::FloorRaise),
        )
        .collect();
    publishes_of_the_continuation.sort_by_key(|publish| publish.output().root.checkpoint_txg);
    let mut previous = last_publish_before_the_continuation;
    for publish in publishes_of_the_continuation {
        let this_publish = publish.output();
        assert_eq!(
            this_publish.root.instance, output.instance,
            "续做里的每一次发布落在这次挂载的实例上（txg {:?}）",
            this_publish.root.checkpoint_txg
        );
        assert_eq!(
            this_publish.root.checkpoint_txg.0,
            previous.root.checkpoint_txg.0 + 1,
            "续做里的每一次发布的 txg 是前一次加一（连号）"
        );
        match publish {
            PublishOfTheContinuation::DeletionStep(step) => assert_eq!(
                step.record.transaction,
                previous.highest_transaction_number_in_this_instance + 1,
                "续做的一步（txg {:?}）的事务号是这个实例的下一个",
                step.root.checkpoint_txg
            ),
            PublishOfTheContinuation::FloorRaise(_) => {}
        }
        previous = this_publish;
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
    observe_the_mount_continuation(prepared, image, report, counts);
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

/// 每个状态挂完的盘都过了池级 checker；续做发成的每一步都核了代与事务号（续做 k 步的状态各核 k 步）。
fn assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
    counts: &Layer0ObserverCounts,
    states: u64,
) {
    assert_eq!(
        counts.count_named("pool_checker_clean_after_the_mount"),
        states,
        "每个状态挂完的盘都过了池级 checker"
    );
    assert_eq!(
        counts.count_named("continuation_steps_numbered_next_in_the_instance"),
        (1..=3)
            .map(|steps| steps * counts.count_named(&format!("mount_continuation_steps_{steps}")))
            .sum::<u64>(),
        "续做发成的每一步都核了代与事务号"
    );
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

/// 崩溃放量的形态（照多记录流：取号加暖机、新池新建文件两个点由公共模块设，B、C 两次顺序写，意图与每一批各一个点；起点是第 32 批 Q 的草稿）：
/// 按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判。版本表带「没有文件的那几版」（意图那一版，`CrashFlow::versions_without_a_file`，
/// 第 33 批 R 报告「停下交主 agent」第一项），两批与最后一批不进表、照比它旧的最新那一项判：不带它时各批报没有文件判成违例、放量红。
/// 展开上限默认 6。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean() {
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = build_pool("crash-amplification-unlink-after-multi-record");
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    common_crash_points::record_pool_build(&mut recorder, &pool);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = recorder.step(file!(), "sequential_write", || {
        sequential_write(&mut pool, &second_content)
    });
    let third_content = content_needing(3, 2);
    let third = recorder.step(file!(), "sequential_write", || {
        sequential_write(&mut pool, &third_content)
    });
    let mut deletion_steps = vec![recorder.step(file!(), "publish_unlink_intent", || {
        publish_the_unlink_intent(&mut pool)
    })];
    for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
        deletion_steps.push(recorder.step(file!(), "publish_unlink_step", || {
            publish_the_next_unlink_step(&mut pool)
        }));
    }
    assert_the_shape_of_the_deletion_steps(&deletion_steps);
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
    let versions = vec![
        PublishedVersion {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: second.root.instance,
            checkpoint_txg: second.root.checkpoint_txg,
            content: second_content,
        },
        PublishedVersion {
            instance: third.root.instance,
            checkpoint_txg: third.root.checkpoint_txg,
            content: third_content,
        },
    ];
    let intent = deletion_steps.first().expect("意图那一步");
    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
        instance: intent.root.instance,
        checkpoint_txg: intent.root.checkpoint_txg,
    }];
    common_crash_points::amplify_prepared_flow_against_the_version_table(
        "unlink-after-multi-record-publish",
        &base,
        &writes,
        &segments,
        judged_root_index,
        &versions,
        &versions_without_a_file,
        crash_points,
        6,
    );
}
