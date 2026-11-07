//! 验证包模块：crash、crash_state_enumeration、crash_identity、crash_library、crash_amplification、verdict_store
//! 删除类发布崩在中间（K-U6 F1：接在多记录流后面 unlink 那个文件）：多记录流「取号 → 暖机 × 2 → A → B → C」之后
//! unlink 第一个文件（C 之后三个数据单元）：意图那次发布 → 按只供测试的批大小 1 删两批 → 最后一批（D8（核心索引结构） 已定项 16），
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；
//! 版本表是 A、B、C 三版有文件，意图那一版起没有文件（D8（核心索引结构） 已定项 16 第 4 条，`crash::CrashPointReplayVersionTable`），
//! 两批与最后一批不进表、照比它旧的最新那一项（意图那一版）判。另逐状态核三样，三个观察者各按功能挂（按功能挂到对应的段）：
//! - 多记录发布的发布边界：挂整条流，与多记录流的崩溃枚举用例挂的是同一个（公共模块 `tests/common_crash_point_observers/mod.rs`
//!   的 `observe_the_publish_boundary_of_the_multi_record_publishes`，代码文件点名那一份、另读的输入相同）；
//! - 删除那几次发布崩在中间：只挂删除那几步的段（`crash_state_enumeration::CrashPointReplayObserverPoints::PointsOverTheSegments`），
//!   这次发布的单元写落了至少一个、记录一份都没落、根没落 ⇒ 实际走的根是上一版
//!   （意图那次的上一版是 C，读回 C 的内容；某一批的上一版是意图或上一批，nlink = 0，答没有文件）；
//! - 可写挂载一次：续做把队列走到空（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」、已定项 16 第 6 条），
//!   续做的步数等于恢复落到的那一版还差几步；续做的每一批落在这次挂载的实例上，txg 与事务号都是这个实例的下一个、连号
//!   （已定项 16 第 2 条末句）；挂完的盘再跑一遍池级 checker，一条不变量都不许判违例（照 `assert_clean` 的口径，已定项 16 第 7 条 ③；
//!   判决 `research/prompts/m3-batch33r-code-r1-main-verification.md` 第三节第 2 条）。挂整条流，与多记录流的崩溃枚举用例、崩溃放量那一路挂在标准前缀上的
//!   是同一个（公共模块的 `observe_the_mount_continuation`，J4）。挂载是每个状态上的重活，观察者放在工作线程上跑；
//!   只展开写数小于 10 的段、同一趟跑两遍、第二遍全照判定库复用，两份计数逐项相同
//!   （`the_unlink_stream_with_the_segments_shorter_than_ten_expanded_counts_the_same_when_rerun_from_the_library`）。
//!
//! **比已有的流多罩了什么**：已有的崩溃点重放流上没有删除类发布；这里意图那次（13 个单元：墓碑容器、`deleted_inodes` 根、inode 叶容器与根……）、
//! 两批（12、11 个单元）、最后一批（12 个单元：inode 叶容器删空不写，退役树开第一片容器、写根）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；
//! 多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做），与挂完的盘上那一遍池级 checker。
//!
//! 按功能切项（要根据功能和模块来配置 item、必须要剪枝和复用）：整条流照录，一条崩溃枚举用例只展开崩溃点录制器的一个功能步的段
//! （取号与暖机、新建文件、两次顺序写、删除意图、两批、最后一批，[`FunctionalStepOfTheUnlinkStream`]），别的段只以整段持久进入后面的状态。
//! 一个点的块键只由这一步的写表、起步镜像与它自己的段展开定，单跑一步与整条流里同一个点的块逐块相同
//! （`each_functional_step_of_the_unlink_stream_judged_alone_plans_the_same_blocks_as_the_whole_stream`）：前缀那三步的块与多记录流、与别的步项共享，
//! 库里有就复用、只判一遍。六步各自的态数相加、整条流最后全部持久那一个状态六项共用一块只算一次，合计 1463812286 个状态，与整条流一项的枚举域相同。
//! CPU 判器的参照照同一把切法只留两批、最后一批两步（恢复会落到不在版本表里、没有文件的根的那两步）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_crash_point_observers;
mod common_crash_points;

use std::ops::Range;

use common::{build_pool, file_content, geometry, BuiltPool};
use common_crash_point_observers::{
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state,
    mount_continuation_inputs_of_the_unlink_stream,
    mount_continuation_observer_of_a_crash_enumeration_case, multi_record_publish,
    observe_the_mount_continuation, observe_the_publish_boundary_of_the_multi_record_publishes,
    publish_boundary_observer_of_a_crash_enumeration_case, writes_to_record,
    MountContinuationInputs, MultiRecordPublishBoundaryInputs,
};
use common_crash_points::{
    content_needing, publish_the_next_unlink_step, publish_the_unlink_intent, sequential_write,
    DELETION_BATCHES_BEFORE_THE_FINAL_BATCH, UNLINKED_FILE,
};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable, CrashStateJudgement,
};
use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
use singlefs_verification::crash_library::{
    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of, ExclusiveTemporaryDirectory,
    TheOneLibraryKeptForThisProcess,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_attached_observers,
    enumerate_crash_states_with_every_kind_of_observer,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table,
    AttachedCrashPointReplayObserver, CrashPointReplayItem, CrashPointReplayObserver,
    CrashPointReplayObserverAttachment, CrashPointReplayObserverCodeFile,
    CrashPointReplayObserverPoints,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const PATH_OF_THIS_TEST_FILE: &str = "crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs";
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::TransactionOutput;
use singlefs_verification::memory_pool::{
    publishes_in, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RecordStreamContinuity, RetainedWrite,
};
use singlefs_verification::segments::StepKind;

fn version_of(output: &TransactionOutput) -> (InstanceGeneration, CheckpointTxg) {
    (output.root.instance, output.root.checkpoint_txg)
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
    /// 发布边界观察者另读的输入（公共模块 `MultiRecordPublishBoundaryInputs`）：B、C 两次多条记录的发布，与多记录流录出的逐字相同。
    publish_boundary_inputs: MultiRecordPublishBoundaryInputs,
    deletion_publishes: Vec<DeletionPublish>,
    /// C 那一版：意图那次的上一版，读回它的内容。
    content_before_the_deletion: Vec<u8>,
    /// 可写挂载续做观察者另读的输入（公共模块 `MountContinuationInputs`）：带文件的 A、B、C，删除那几步各自落到之后续做还差几步。
    mount_continuation_inputs: MountContinuationInputs,
    /// 每个功能步管哪几段（照崩溃点录制器那一份造法的步名切，[`segments_of_each_functional_step`]），按流的次序。
    segments_of_each_step: Vec<(FunctionalStepOfTheUnlinkStream, Range<usize>)>,
}

impl Prepared {
    fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::new(&self.versions, &self.versions_without_a_file)
    }

    /// 这一功能步管的段。
    fn segments_of_the_step(&self, step: FunctionalStepOfTheUnlinkStream) -> Range<usize> {
        self.segments_of_each_step
            .iter()
            .find(|(step_of_the_range, _segments)| *step_of_the_range == step)
            .map(|(_step, segments)| segments.clone())
            .expect("每个功能步在录制器那一份里都有它的段（segments_of_each_functional_step 核过六步齐全）")
    }

    /// 删除那几步（意图、两批、最后一批）的段：删除停在中间观察者只挂这几段。
    fn segments_of_the_deletion_steps(&self) -> Vec<usize> {
        self.segments_of_each_step
            .iter()
            .filter(|(step, _segments)| step.is_a_deletion_step())
            .flat_map(|(_step, segments)| segments.clone())
            .collect()
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
            2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 22, 2, 1, 2, 24, 2, 1, 2
        ],
        "前二十段同多记录流（取号两轮各一段）；之后意图那次 26 个单元写（13 个单元两盘各一份）、记录 2、根 1、轮换 2；两批 24、22 个单元写；\
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
    let publish_boundary_inputs = MultiRecordPublishBoundaryInputs {
        publishes: vec![
            multi_record_publish(&writes, &recorded.second, &recorded.first, file_content()),
            multi_record_publish(
                &writes,
                &recorded.third,
                &recorded.second,
                recorded.second_content.clone(),
            ),
        ],
    };
    // 功能步照崩溃点录制器那一份造法的步名切：两份造法写表与段相同（这里先核段，写表逐字节相同另有
    // `the_unlink_stream_writes_the_same_table_through_the_crash_point_recorder_as_without_it`）
    let through_the_recorder =
        record_through_the_crash_point_recorder(&format!("{tag}-functional-steps"), false);
    assert_eq!(
        through_the_recorder.segments, segments,
        "录制器那一份造法的段与这一份一样切"
    );
    let segments_of_each_step =
        segments_of_each_functional_step(&through_the_recorder.crash_points);
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        versions_without_a_file,
        publish_boundary_inputs,
        deletion_publishes,
        mount_continuation_inputs: MountContinuationInputs::of_the_publishes(
            &[&recorded.first, &recorded.second, &recorded.third],
            &recorded.deletion_steps,
        ),
        content_before_the_deletion: recorded.third_content,
        segments_of_each_step,
    }
}

/// 每个状态上核删除的中间态：删除的一次发布停在中间（单元写落了、记录没落、根没落）⇒ 走的是上一版的根；
/// 意图那次的上一版是 C，读回 C 的内容；某一批的上一版第一个文件 nlink = 0，答没有文件（D8（核心索引结构） 已定项 16 第 4 条）。
fn observe_the_deletion_cut_in_the_middle(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
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

/// 删除停在中间观察者的名字（只挂删除那几步的段）。
const DELETION_CUT_IN_THE_MIDDLE_OBSERVER_NAME: &str = "deletion_cut_in_the_middle";

/// 删除停在中间观察者另读的输入的摘要（进它的版本）：它读的删除那几次发布（种类、单元写、记录两份、根槽写、上一版）与删之前那一版的内容，
/// 逐项一行写下、取 SHA-256；录制流一变它就变，看过的块不按旧的复用。
fn deletion_cut_inputs_digest(
    prepared: &Prepared,
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
    let mut text = String::from("deletion cut in the middle inputs\n");
    for publish in &prepared.deletion_publishes {
        text.push_str(&format!(
            "kind={:?} unit_writes={:?} record_copies={:?} root_write={} version_before={:?}\n",
            publish.kind,
            publish.unit_writes,
            publish.record_copies,
            publish.root_write,
            publish.version_before
        ));
    }
    text.push_str(&format!(
        "content_before_the_deletion={:?}\n",
        prepared.content_before_the_deletion
    ));
    singlefs_verification::sha256::sha256_digest(text.as_bytes())
}

/// 在删除流上照用例自己的展开（`expansion`）枚举一趟，挂三个观察者（按功能挂到对应的段）：发布边界挂整条流（与多记录流的崩溃枚举用例
/// 同一个）；删除停在中间只挂删除那几步的段（前缀点上不录、不判，块账在那几点上不要它的结论）；可写挂载续做挂整条流（与多记录流的崩溃枚举用例、
/// 崩溃放量那一路挂在标准前缀上的同一个）。前缀那几步的块与多记录流的同几个点块键相同、观察者版本相同，库里有就复用。只录那一趟交 `None`。
fn enumerate_the_unlink_stream_with_its_observers(
    prepared: &Prepared,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe_the_boundary =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_publish_boundary_of_the_multi_record_publishes(
                &prepared.publish_boundary_inputs,
                image,
                report,
                counts,
            );
        };
    let observe_the_deletion =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_deletion_cut_in_the_middle(prepared, image, report, counts);
        };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation(
                &prepared.mount_continuation_inputs,
                image,
                report,
                counts,
            )
        };
    let segments_of_the_deletion_steps = prepared.segments_of_the_deletion_steps();
    // 同一个二进制里在 CPU 判器上跑的那两条换多机配置：开库期间拿着全仓那一份的守卫
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    enumerate_crash_states_with_attached_observers(
        CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        expansion,
        &[
            publish_boundary_observer_of_a_crash_enumeration_case(
                &prepared.publish_boundary_inputs,
                &observe_the_boundary,
            ),
            AttachedCrashPointReplayObserver {
                observer: CrashPointReplayObserver {
                    name: DELETION_CUT_IN_THE_MIDDLE_OBSERVER_NAME,
                    extra_inputs_digest: deletion_cut_inputs_digest(prepared),
                    observe: &observe_the_deletion,
                },
                attachment: CrashPointReplayObserverAttachment {
                    points: CrashPointReplayObserverPoints::PointsOverTheSegments(
                        &segments_of_the_deletion_steps,
                    ),
                    code_file: CrashPointReplayObserverCodeFile::TheCaseFile,
                },
            },
        ],
        &[mount_continuation_observer_of_a_crash_enumeration_case(
            &prepared.mount_continuation_inputs,
            &observe_the_mount,
        )],
    )
    .judged()
}

/// 只展开写数小于 10 的段那一趟：七个单元段不展开，挂三个观察者（[`enumerate_the_unlink_stream_with_its_observers`]）。
fn enumerate_the_segments_shorter_than_ten(
    prepared: &Prepared,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    enumerate_the_unlink_stream_with_its_observers(
        prepared,
        &format!("unlink_after_multi_record::segments_shorter_than_ten::{item_label}"),
        &expansion_of_the_segments_shorter_than_ten,
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

/// 复用不改判定：只展开写数小于 10 的段，同一趟跑两遍，第二遍的块与观察者结论全照库里的，两份计数（状态数、违例、checker 逐条、发现表、观察者计数）逐项相同，
/// 每个状态都看过。
#[test]
fn the_unlink_stream_with_the_segments_shorter_than_ten_expanded_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("unlink-after-multi-record-crash-point-replay-rerun");
    let Some(first_run) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun") else {
        return;
    };
    let Some(second_run) = enumerate_the_segments_shorter_than_ten(&prepared, "rerun") else {
        return;
    };
    assert_eq!(second_run, first_run, "第二遍全照库里复用，计数逐项相同");
    assert_eq!(
        first_run.observed_states, first_run.states,
        "每个状态都看过"
    );
}

// ============================== 按功能切项：一个功能步一条崩溃枚举用例 ==============================

/// 删除流的功能步，照崩溃点录制器的步名切（`acquire_instance_and_warm_up`、`publish_first_file`、`sequential_write`、`publish_unlink_intent`、
/// `publish_unlink_step`）：两次顺序写同一个步名、连着发，并成一步；`publish_unlink_step` 里最后一批与前面几批发布种类不同（[`DeletionPublishKind`]），
/// 分成两步。每一步一条崩溃枚举用例，只展开自己的段；要测多少个状态由这一步自己的段定（这个模块要测多少、需要几个 item，
/// 它就合理的去自己配置）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FunctionalStepOfTheUnlinkStream {
    /// 取号与两次暖机（段 0–7）。
    InstanceAcquisitionAndWarmUp,
    /// 新建第一个文件 A（段 8–11）。
    FirstFileCreation,
    /// 两次顺序写 B、C，多条记录的发布（段 12–19）。
    SequentialWrites,
    /// 删除意图（段 20–23）。
    UnlinkIntent,
    /// 不是最后一批的两批（段 24–31）。
    UnlinkBatches,
    /// 最后一批（段 32–35）。
    UnlinkFinalBatch,
}

impl FunctionalStepOfTheUnlinkStream {
    /// 流里的次序。
    const IN_THE_ORDER_OF_THE_STREAM: [Self; 6] = [
        Self::InstanceAcquisitionAndWarmUp,
        Self::FirstFileCreation,
        Self::SequentialWrites,
        Self::UnlinkIntent,
        Self::UnlinkBatches,
        Self::UnlinkFinalBatch,
    ];

    /// 崩溃枚举用例的项名。一步一条流：名字里没有 `::`，整个就是来历表认的流名，`crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS`
    /// 里一步一行、登在崩溃点重放合表这一步的项上（跑的那一处按（崩溃枚举，流名）只认一行，六步共用一个流名就登不上六项）。
    fn item_name(self) -> &'static str {
        match self {
            Self::InstanceAcquisitionAndWarmUp => {
                "unlink_after_multi_record_instance_acquisition_and_warm_up"
            }
            Self::FirstFileCreation => "unlink_after_multi_record_first_file_creation",
            Self::SequentialWrites => "unlink_after_multi_record_sequential_writes",
            Self::UnlinkIntent => "unlink_after_multi_record_unlink_intent",
            Self::UnlinkBatches => "unlink_after_multi_record_unlink_batches",
            Self::UnlinkFinalBatch => "unlink_after_multi_record_unlink_final_batch",
        }
    }

    /// 录流用的标签（池镜像的名字）。
    fn label(self) -> &'static str {
        match self {
            Self::InstanceAcquisitionAndWarmUp => "instance-acquisition-and-warm-up",
            Self::FirstFileCreation => "first-file-creation",
            Self::SequentialWrites => "sequential-writes",
            Self::UnlinkIntent => "unlink-intent",
            Self::UnlinkBatches => "unlink-batches",
            Self::UnlinkFinalBatch => "unlink-final-batch",
        }
    }

    /// 删除那几步：删除停在中间观察者只挂它们的段。
    fn is_a_deletion_step(self) -> bool {
        match self {
            Self::InstanceAcquisitionAndWarmUp
            | Self::FirstFileCreation
            | Self::SequentialWrites => false,
            Self::UnlinkIntent | Self::UnlinkBatches | Self::UnlinkFinalBatch => true,
        }
    }

    /// 这一步那几段的全量域态数，照枚举域闭式逐段相加（推的；用例里另照这一步的展开现算闭式、与它比）：系统配置槽 2 写段 3² − 1、
    /// 记录 2 写段 3、4 写段 15、6 写段 63、根槽写段 1、单元段 2^n − 1。六步合计 1463812285，加整条流最后全部持久那一个状态是 1463812286。
    fn states_of_its_segments_in_the_full_domain(self) -> u64 {
        match self {
            // 取号两轮各一个系统配置槽段，两次暖机各记录、根槽写、轮换
            Self::InstanceAcquisitionAndWarmUp => 4 * (3 * 3 - 1) + 2 * 3 + 2,
            // A 的 24 个单元写、记录、根槽写、轮换
            Self::FirstFileCreation => ((1 << 24) - 1) + 3 + 1 + (3 * 3 - 1),
            // B 的 28 个单元写、两条记录 4 写、根槽写、轮换；C 的 30 个单元写、三条记录 6 写、根槽写、轮换
            Self::SequentialWrites => {
                ((1 << 28) - 1) + 15 + 1 + (3 * 3 - 1) + ((1 << 30) - 1) + 63 + 1 + (3 * 3 - 1)
            }
            // 意图的 26 个单元写、记录、根槽写、轮换
            Self::UnlinkIntent => ((1 << 26) - 1) + 3 + 1 + (3 * 3 - 1),
            // 两批各 24、22 个单元写、记录、根槽写、轮换
            Self::UnlinkBatches => {
                ((1 << 24) - 1) + 3 + 1 + (3 * 3 - 1) + ((1 << 22) - 1) + 3 + 1 + (3 * 3 - 1)
            }
            // 最后一批的 24 个单元写、记录、根槽写、轮换
            Self::UnlinkFinalBatch => ((1 << 24) - 1) + 3 + 1 + (3 * 3 - 1),
        }
    }

    /// 停在 B、C 两次多记录发布中间的状态数（全量域）：只在两次顺序写那一步，B 记录段 4 写里第一条落了、第二条一份没落 3 个，
    /// C 记录段 6 写里第一条落了、第三条一份没落 12 个。
    fn cut_states_in_the_multi_record_publishes(self) -> [u64; 2] {
        match self {
            Self::SequentialWrites => [3, 12],
            Self::InstanceAcquisitionAndWarmUp
            | Self::FirstFileCreation
            | Self::UnlinkIntent
            | Self::UnlinkBatches
            | Self::UnlinkFinalBatch => [0, 0],
        }
    }

    /// 意图、两批、最后一批各自停在中间的状态数（全量域）：单元段 n 个写里至少落一个的真子集 2^n − 2，加记录段的空子集（单元全落、记录一份没落）。
    fn cut_states_in_the_deletion_publishes(self) -> [u64; 4] {
        match self {
            Self::InstanceAcquisitionAndWarmUp
            | Self::FirstFileCreation
            | Self::SequentialWrites => [0, 0, 0, 0],
            Self::UnlinkIntent => [(1 << 26) - 1, 0, 0, 0],
            Self::UnlinkBatches => [0, (1 << 24) - 1, (1 << 22) - 1, 0],
            Self::UnlinkFinalBatch => [0, 0, 0, (1 << 24) - 1],
        }
    }

    /// 可写挂载续做还差 1、2、3 步的状态数（全量域；恢复落到第二批、头一批、意图那一版的状态）。落到意图：意图根槽写落了之后（意图的轮换段 8 个）
    /// 到头一批根槽写落之前（头一批单元段 2^24 − 1、记录段 3、根槽写段的空子集 1）；落到头一批、第二批同理；落到最后一批与之前那几版的续做 0 步。
    fn states_landing_with_continuation_steps_one_to_three(self) -> [u64; 3] {
        match self {
            Self::InstanceAcquisitionAndWarmUp
            | Self::FirstFileCreation
            | Self::SequentialWrites => [0, 0, 0],
            Self::UnlinkIntent => [0, 0, 3 * 3 - 1],
            Self::UnlinkBatches => [
                3 * 3 - 1,
                (3 * 3 - 1) + ((1 << 22) - 1) + 3 + 1,
                ((1 << 24) - 1) + 3 + 1,
            ],
            Self::UnlinkFinalBatch => [((1 << 24) - 1) + 3 + 1, 0, 0],
        }
    }
}

/// 照崩溃点录制器那一份造法的点切功能步：录制器每段一个点，点名 `<步名>.<周期>.<这一种周期在这一次调用里的第几段>`，步名变了、
/// 或同一次调用里点名重复出现（同一个步名的下一次调用）就是下一次调用起头；调用照步名归功能步，连着同一个功能步的几次调用并成一段区间。
/// 交回每个功能步管的段（按流的次序，首尾相接罩住全部段）。
fn segments_of_each_functional_step(
    crash_points: &[singlefs_verification::crash_amplification::CrashPointSpan],
) -> Vec<(FunctionalStepOfTheUnlinkStream, Range<usize>)> {
    // 每一次调用：（步名，段区间，这一次调用里出现过的点名）；迭代上界：段数
    let mut calls: Vec<(String, Range<usize>, Vec<String>)> = Vec::new();
    for (segment_index, point) in crash_points.iter().enumerate() {
        let entry = point
            .name
            .split('.')
            .next()
            .expect("点名以步名起头")
            .to_string();
        let starts_a_call = calls.last().is_none_or(|(last_entry, _segments, names)| {
            *last_entry != entry || names.contains(&point.name)
        });
        if starts_a_call {
            calls.push((entry, segment_index..segment_index, Vec::new()));
        }
        let (_entry, segments, names) = calls.last_mut().expect("刚放进去一次调用");
        segments.end = segment_index + 1;
        names.push(point.name.clone());
    }
    let unlink_step_calls = calls
        .iter()
        .filter(|(entry, _segments, _names)| entry == "publish_unlink_step")
        .count();
    assert_eq!(
        unlink_step_calls,
        DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1,
        "删除的批：两批加最后一批，各一次调用"
    );
    let mut unlink_step_calls_seen = 0usize;
    let mut steps: Vec<(FunctionalStepOfTheUnlinkStream, Range<usize>)> = Vec::new();
    for (entry, segments, _names) in calls {
        let step = match entry.as_str() {
            "acquire_instance_and_warm_up" => {
                FunctionalStepOfTheUnlinkStream::InstanceAcquisitionAndWarmUp
            }
            "publish_first_file" => FunctionalStepOfTheUnlinkStream::FirstFileCreation,
            "sequential_write" => FunctionalStepOfTheUnlinkStream::SequentialWrites,
            "publish_unlink_intent" => FunctionalStepOfTheUnlinkStream::UnlinkIntent,
            "publish_unlink_step" => {
                unlink_step_calls_seen += 1;
                if unlink_step_calls_seen == unlink_step_calls {
                    FunctionalStepOfTheUnlinkStream::UnlinkFinalBatch
                } else {
                    FunctionalStepOfTheUnlinkStream::UnlinkBatches
                }
            }
            unknown => panic!("录制器的步名 {unknown:?} 不在删除流的功能步里"),
        };
        match steps.last_mut() {
            Some((last_step, last_segments)) if *last_step == step => {
                last_segments.end = segments.end;
            }
            Some(_) | None => steps.push((step, segments)),
        }
    }
    assert_eq!(
        steps
            .iter()
            .map(|(step, _segments)| *step)
            .collect::<Vec<_>>(),
        FunctionalStepOfTheUnlinkStream::IN_THE_ORDER_OF_THE_STREAM.to_vec(),
        "录制器的步名切出六个功能步、照流的次序"
    );
    assert_eq!(
        steps.last().map(|(_step, segments)| segments.end),
        Some(crash_points.len()),
        "六步首尾相接罩住全部段"
    );
    steps
}

/// 一个功能步的展开：这一步的段每段全部真子集，别的段不展开（只以整段持久进入后面的状态）。
fn expansion_of_the_functional_step(
    segments_of_the_step: Range<usize>,
) -> impl Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync {
    move |segment_index: usize, _segment: &[usize]| {
        if segments_of_the_step.contains(&segment_index) {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }
}

/// 删除流的一个功能步当一条崩溃枚举用例跑一趟：整条流照录，只展开这一步的段，挂三个观察者（[`enumerate_the_unlink_stream_with_its_observers`]）；
/// 前缀那几步的块与多记录流的同几个点块键相同、观察者版本相同，几步之间末节点那一块相同，库里有就复用、不另判。逐项核：
/// 状态数等于这一步展开下的闭式（这一步的态数加整条流最后全部持久那一个状态），每个状态都看过；停在两次多记录发布中间、删除四次发布各自停在中间、
/// 续做还差 1、2、3 步的状态数照全量域钉（[`FunctionalStepOfTheUnlinkStream`] 那几张表）；每个状态都可写挂载过一次、挂完的盘过池级 checker、
/// 续做的每一步核了代与事务号；判器零红。打一行 `CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD step=<项名> …`（门禁 crash-point-replay 认它的 `exhaustive=true`）。
fn assert_the_functional_step_is_exhaustive_and_clean(step: FunctionalStepOfTheUnlinkStream) {
    let prepared = prepare(&format!("unlink-after-multi-record-{}", step.label()));
    let expansion = expansion_of_the_functional_step(prepared.segments_of_the_step(step));
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &expansion,
    );
    assert_eq!(
        closed_form,
        step.states_of_its_segments_in_the_full_domain() + 1,
        "{step:?}：这一步的展开下的枚举域，等于这一步那几段逐段相加、加整条流最后全部持久那一个状态"
    );
    let Some(tally) =
        enumerate_the_unlink_stream_with_its_observers(&prepared, step.item_name(), &expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD step={} states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        step.item_name(),
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        &format!("{step:?}：这一步的展开下的闭式"),
    );
    assert_eq!(
        tally.observed_states, tally.states,
        "{step:?}：发布边界与可写挂载续做挂整条流，每个状态都看过"
    );
    let counts = &tally.observer_counts;
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        [0, 1].map(|publish_index| {
            counts.count_named(&format!("cut_in_multi_record_publish_{publish_index}"))
        }),
        step.cut_states_in_the_multi_record_publishes(),
        &format!("{step:?}：停在 B、C 两次多记录发布中间的状态数"),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        [0, 1, 2, 3].map(|publish_index| {
            counts.count_named(&format!("cut_in_deletion_publish_{publish_index}"))
        }),
        step.cut_states_in_the_deletion_publishes(),
        &format!("{step:?}：意图、两批、最后一批各自停在中间的状态数"),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        [1, 2, 3].map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}"))),
        step.states_landing_with_continuation_steps_one_to_three(),
        &format!("{step:?}：续做还差 1、2、3 步的状态数"),
    );
    assert_eq!(
        (0..=3)
            .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
            .sum::<u64>(),
        tally.states,
        "{step:?}：每个状态都可写挂载过一次"
    );
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
        counts,
        tally.states,
    );
    assert_clean(&tally);
}

/// 全量：取号与两次暖机那一步（段 0–7，四个系统配置槽 2 写段、两个记录段、两个根槽写段，40 个状态加末节点那 1 个）。
/// 块与多记录流同几个点相同，多记录流判过就全复用。
#[test]
fn full_enumeration_of_the_instance_acquisition_and_warm_up_before_the_unlink_after_the_multi_record_publish_is_exhaustive_and_clean(
) {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::InstanceAcquisitionAndWarmUp,
    );
}

/// 全量：新建第一个文件 A 那一步（段 8–11，16777227 个状态加末节点那 1 个）。块与多记录流同几个点相同。
#[test]
fn full_enumeration_of_the_first_file_creation_before_the_unlink_after_the_multi_record_publish_is_exhaustive_and_clean(
) {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::FirstFileCreation,
    );
}

/// 全量：两次顺序写 B、C 那一步（段 12–19，1342177374 个状态加末节点那 1 个；B、C 的单元段各 2^28 − 1、2^30 − 1）。C 尾（C 根槽写之后的轮换，
/// 按发布分的态数往下游找到意图那次发布）块键与多记录流不同，其余与多记录流同几个点相同。
#[test]
fn full_enumeration_of_the_sequential_writes_before_the_unlink_after_the_multi_record_publish_is_exhaustive_and_clean(
) {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::SequentialWrites,
    );
}

/// 全量：删除意图那一步（段 20–23，67108875 个状态加末节点那 1 个）。
#[test]
fn full_enumeration_of_the_unlink_intent_after_the_multi_record_publish_is_exhaustive_and_clean() {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::UnlinkIntent,
    );
}

/// 全量：不是最后一批的两批那一步（段 24–31，20971542 个状态加末节点那 1 个）。
#[test]
fn full_enumeration_of_the_unlink_batches_after_the_multi_record_publish_is_exhaustive_and_clean() {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::UnlinkBatches,
    );
}

/// 全量：最后一批那一步（段 32–35，16777227 个状态加末节点那 1 个）。
#[test]
fn full_enumeration_of_the_unlink_final_batch_after_the_multi_record_publish_is_exhaustive_and_clean(
) {
    assert_the_functional_step_is_exhaustive_and_clean(
        FunctionalStepOfTheUnlinkStream::UnlinkFinalBatch,
    );
}

/// 录制器那一份造法录下的删除流：起点镜像、写表、段、崩溃点、被判的根槽写（最后一批的那一条）、版本表与「没有文件的那几版」。
struct RecordedThroughTheCrashPointRecorder {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

/// 录制器那一份造法：取号加暖机、新池新建文件、B、C 四个点由公共模块设（与多记录流同一个函数，共享到 C，J3），
/// 意图与每一批各一个点；`stop_after_the_multi_record_publishes` 为真时录到 C 就停（多记录流那一份，比共享用）。
fn record_through_the_crash_point_recorder(
    tag: &str,
    stop_after_the_multi_record_publishes: bool,
) -> RecordedThroughTheCrashPointRecorder {
    use singlefs_verification::crash_amplification::CrashPointRecorder;
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
        singlefs_verification::memory_pool::writes_and_segments_with_stream_indexes(
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

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。
/// 版本表带「没有文件的那几版」（意图那一版，`CrashFlow::versions_without_a_file`），
/// 两批与最后一批不进表、照比它旧的最新那一项判：不带它时各批报没有文件判成违例、放量红。展开上限取引擎常量（设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 照它缩）。
/// 登记表挂在标准前缀节点上的观察者（可写挂载续做）照表跑（J4），与多记录流那一项在同一批块上共享。
#[test]
fn crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean() {
    // 同一个二进制里在 CPU 判器上跑放量的那一条设一次性库：整趟拿着全仓那一份的守卫，它设不了一次性库，这一边开不到它那一份
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
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
    );
}

/// CPU 判器参照只留的两步：恢复会落到不在版本表里、没有文件的根的就这两步（意图那一版在「没有文件的那几版」里，两批与最后一批不进表）；
/// 前缀那几步的块键、判器版本与 GPU 那几条相同，另判一遍只是复用库里的块。一步一条流，项名里没有 `::`，来历表里各一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FunctionalStepJudgedOnTheCpuJudge {
    /// 不是最后一批的两批（段 24–31）。
    UnlinkBatches,
    /// 最后一批（段 32–35）。
    UnlinkFinalBatch,
}

impl FunctionalStepJudgedOnTheCpuJudge {
    /// 它是删除流的哪一个功能步（段与闭式照那一步取）。
    fn step(self) -> FunctionalStepOfTheUnlinkStream {
        match self {
            Self::UnlinkBatches => FunctionalStepOfTheUnlinkStream::UnlinkBatches,
            Self::UnlinkFinalBatch => FunctionalStepOfTheUnlinkStream::UnlinkFinalBatch,
        }
    }

    /// CPU 判器参照那一条的项名（整个就是来历表认的流名）。
    fn item_name(self) -> &'static str {
        match self {
            Self::UnlinkBatches => "unlink_after_multi_record_unlink_batches_on_the_cpu_judge",
            Self::UnlinkFinalBatch => {
                "unlink_after_multi_record_unlink_final_batch_on_the_cpu_judge"
            }
        }
    }
}

/// CPU 判器上判删除流的一个功能步：照调用方的多机配置写一份只开 CPU 的（`crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of`：
/// 不开 GPU，判段交 CPU 判器；照抄 prune-server 的地址，没有服务 socket 的那一台照它连同一处），只展开这一步的段（[`expansion_of_the_functional_step`]，
/// 用例自己不缩域：要测多少个状态由这一步自己的段定；调用方设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（指定范围的那几趟可缩域）照它缩，
/// 环境守卫清掉之前记下、清完设回），不挂用例自己的观察者（核的是 CPU 判器）。块进全仓那一份库、照块键与判器组的单元键复用；
/// 门禁开 GPU 时同一步的那一条崩溃枚举用例判段走 GPU 判器，CPU 逐块判的 `check_blocks_where_until_with_threads` 它跑不到；被改坏的那一处在
/// 那一处交给 CPU 判一段的版本表上，版本表不带「没有文件的那几版」时恢复落到意图、各批的根报没有文件判成违例（变异表「崩溃放量 CPU 判器不带
/// 「没有文件的那几版」」；变异那一趟连的是变异门禁自己起的临时 prune-server，库是空的，每块现判）。
/// 状态数等于这一步展开下的闭式（缩过域照缩域的判），报没有文件的状态真跑到了，判器零红。
// gpu-exempt: cpu-judge-mutation-rows
fn assert_the_functional_step_judged_on_the_cpu_judge_is_clean(
    step_on_the_cpu_judge: FunctionalStepJudgedOnTheCpuJudge,
) {
    let step = step_on_the_cpu_judge.step();
    let prepared = prepare(&format!(
        "unlink-after-multi-record-{}-on-the-cpu-judge",
        step.label()
    ));
    let expansion = expansion_of_the_functional_step(prepared.segments_of_the_step(step));
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &expansion,
    );
    assert_eq!(
        closed_form,
        step.states_of_its_segments_in_the_full_domain() + 1,
        "{step:?}：这一步的展开下的枚举域，等于这一步那几段逐段相加、加整条流最后全部持久那一个状态"
    );
    // 调用方的多机配置在守卫清掉 SINGLEFS_MULTI_HOST_CONFIG 之前取（副本里只经这个变量交进来）
    let configuration_of_the_caller =
        singlefs_verification::crash_amplification::multi_host_configuration_of_this_process()
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .configuration_file;
    let configuration_directory = ExclusiveTemporaryDirectory::create(&format!(
        "unlink-after-multi-record-{}-on-the-cpu-judge",
        step.label()
    ))
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration_on_the_cpu = configuration_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(
        &configuration_on_the_cpu,
        multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
            &configuration_of_the_caller,
        )
        .unwrap_or_else(|refusal| panic!("{refusal}")),
    )
    .expect("测试自己的临时目录里写得了只开 CPU 的多机配置");
    // 调用方缩过域的那一趟照它缩：守卫清掉一趟读的全部变量之前记下，清完设回（别的变量照清，用例只换多机配置）
    let narrowing_of_the_caller = std::env::var_os(
        singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
    );
    // 先拿全仓那一份的守卫、再拿改环境的锁（`crash_library::TheOneLibraryKeptForThisProcess` 文档里的次序）；丢的时候倒过来
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration_on_the_cpu.as_os_str(),
    )]);
    if let Some(expand_up_to) = &narrowing_of_the_caller {
        environment.set_variable(
            singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            expand_up_to,
        );
    }
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: step_on_the_cpu_judge.item_name(),
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &expansion,
        &[],
        &[],
    )
    .judged() else {
        return;
    };
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        &format!("{step:?}（CPU 判器）：这一步的展开下的闭式"),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        tally.no_file_states,
        &format!("{step:?}（CPU 判器）：恢复落到意图或某一批的根、报没有文件的状态真跑到了"),
    );
    assert_clean(&tally);
}

/// CPU 判器上判不是最后一批的两批那一步（段 24–31，恢复落到意图、头一批、第二批的根：意图那一版在「没有文件的那几版」里，两批不进表、照比它旧的最新那一项判）。
/// 判别力（`crates/mutations.tsv`）：CPU 判器的 oracle 认不出落在意图那一版上（`crash::CrashPointReplayVersionTable` 查根时不看「没有文件的那几版」），
/// 恢复落到意图的状态照 C 判成「有文件却报没有」，这一条红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn unlink_batches_after_the_multi_record_publish_judged_on_the_cpu_judge_are_clean() {
    assert_the_functional_step_judged_on_the_cpu_judge_is_clean(
        FunctionalStepJudgedOnTheCpuJudge::UnlinkBatches,
    );
}

/// CPU 判器上判最后一批那一步（段 32–35，恢复落到第二批与最后一批的根，两版都不进表、照意图那一版判）。
/// 判别力（`crates/mutations.tsv`）：CPU 判器的 oracle 找比根旧的「没有文件」那一版时比反了方向（找不到意图那一版），落到第二批与最后一批的根照 C 判成违例，这一条红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn unlink_final_batch_after_the_multi_record_publish_judged_on_the_cpu_judge_is_clean() {
    assert_the_functional_step_judged_on_the_cpu_judge_is_clean(
        FunctionalStepJudgedOnTheCpuJudge::UnlinkFinalBatch,
    );
}

/// 单跑一个功能步与整条流里判的块逐块相同（按功能切项要靠这一条才复用得上）：崩溃枚举用例的点按屏障段切（`crash_state_enumeration::crash_points_of_the_steps`，
/// 这条流一段一个点），一个点的块键只由这一步的写表、起步镜像与它自己的段展开定。六个功能步各自只展开自己的段排的计划里，这一步的点上的块与
/// 每段都展开排的整条流计划里同几个点的块逐块相同，别的点上一块都没有；末节点那一块（整条流最后全部持久那一个状态）六步与整条流相同；
/// 六步的块合起来、末节点那一块只算一次，恰好是整条流的块（态数之和因此等于整条流的枚举域）。
/// 判别力（`crates/mutations.tsv`）：功能步的切法与展开都只在这里排计划核，不进枚举：一步的展开把这一步之前的段也展开（别的点上有了块）、
/// 录制器的顺序写步名归到新建文件那一步（`prepare` 切不出六步）、连着同一步的几次调用并段时少并最后一段（这一步的态数少了一段），这一条各红一次。
// crash-case-check:not-a-crash-case 只排计划、不判一个状态
#[test]
fn each_functional_step_of_the_unlink_stream_judged_alone_plans_the_same_blocks_as_the_whole_stream(
) {
    use singlefs_verification::crash_amplification::{
        plan_crash_points, CrashFlow, END_OF_FLOW_NODE_NAME,
    };
    use singlefs_verification::crash_state_enumeration::crash_points_of_the_steps;
    use singlefs_verification::verdict_store::VerdictBlockKey;
    let prepared = prepare("unlink-after-multi-record-functional-steps-plan-the-same-blocks");
    // 每个点（连同末节点）上的块键，与这个点管的状态数
    let blocks_of_each_point = |item_name: &str,
                                expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion
                                      + Sync)|
     -> Vec<(String, Vec<VerdictBlockKey>, u64)> {
        let item = CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        };
        let flow = CrashFlow {
            base: &prepared.base,
            writes: &prepared.writes,
            segments: &prepared.segments,
            judged_root_index: prepared.judged_root_index,
            versions: &prepared.versions,
            versions_without_a_file: &prepared.versions_without_a_file,
            expansion,
            crash_points: crash_points_of_the_steps(item, &prepared.writes, &prepared.segments),
        };
        plan_crash_points(&flow)
            .points
            .iter()
            .map(|point| {
                (
                    point.name.clone(),
                    point
                        .blocks()
                        .into_iter()
                        .map(|local| point.block_key(&local))
                        .collect(),
                    point.ordinals.end - point.ordinals.start,
                )
            })
            .collect()
    };
    let whole_stream = blocks_of_each_point(
        "unlink_after_multi_record::whole_stream_plan",
        &full_expansion,
    );
    assert_eq!(
        whole_stream.len(),
        prepared.segments.len() + 1,
        "前提：这条流一段一个点，加末节点"
    );
    let end_of_flow = whole_stream.len() - 1;
    assert_eq!(
        whole_stream[end_of_flow].0, END_OF_FLOW_NODE_NAME,
        "计划最后一项是末节点"
    );
    assert_eq!(
        (
            whole_stream[end_of_flow].1.len(),
            whole_stream[end_of_flow].2
        ),
        (1, 1),
        "末节点一块、一个状态"
    );
    let mut blocks_of_the_steps: Vec<VerdictBlockKey> = whole_stream[end_of_flow].1.clone();
    let mut states_of_the_steps_without_the_end_of_flow = 0u64;
    for (step, segments_of_the_step) in &prepared.segments_of_each_step {
        let alone = blocks_of_each_point(
            step.item_name(),
            &expansion_of_the_functional_step(segments_of_the_step.clone()),
        );
        assert_eq!(alone.len(), whole_stream.len(), "{step:?}：点一样切");
        for (
            point_index,
            ((_name, blocks_alone, states_alone), (name, blocks_whole, states_whole)),
        ) in alone.iter().zip(&whole_stream).enumerate()
        {
            if point_index == end_of_flow || segments_of_the_step.contains(&point_index) {
                assert_eq!(
                    (blocks_alone, states_alone),
                    (blocks_whole, states_whole),
                    "{step:?}：第 {point_index} 个点（{name}）上的块与整条流里的逐块相同"
                );
            } else {
                assert!(
                    blocks_alone.is_empty(),
                    "{step:?}：第 {point_index} 个点（{name}）不归这一步，一块都没有"
                );
            }
            if point_index != end_of_flow && segments_of_the_step.contains(&point_index) {
                blocks_of_the_steps.extend(blocks_alone.iter().cloned());
                states_of_the_steps_without_the_end_of_flow += states_alone;
            }
        }
        assert_eq!(
            alone
                .iter()
                .enumerate()
                .filter(|(point_index, _point)| *point_index != end_of_flow)
                .map(|(_point_index, (_name, _blocks, states))| *states)
                .filter(|states| *states > 0)
                .sum::<u64>(),
            step.states_of_its_segments_in_the_full_domain(),
            "{step:?}：这一步的点上的状态数等于这一步那几段的全量域态数"
        );
    }
    blocks_of_the_steps.sort();
    let mut blocks_of_the_whole_stream: Vec<VerdictBlockKey> = whole_stream
        .iter()
        .flat_map(|(_name, blocks, _states)| blocks.iter().cloned())
        .collect();
    blocks_of_the_whole_stream.sort();
    assert_eq!(
        blocks_of_the_steps, blocks_of_the_whole_stream,
        "六步的块合起来（末节点那一块只算一次）恰好是整条流的块"
    );
    assert_eq!(
        states_of_the_steps_without_the_end_of_flow + 1,
        1_463_812_286,
        "六步的态数之和加末节点那 1 个，等于整条流的枚举域"
    );
}

/// 删除流与多记录流的崩溃枚举用例挂的发布边界、可写挂载续做两个观察者是同一个：代码文件都点名公共模块（登记表里一行、一把闭包键），
/// 另读的输入也要逐字相同，两条流共有的前缀块上观察者结论才一处判过、另一处复用。场景：删除流录出的这两份另读的输入，与多记录流的录法
/// （多记录流 `prepare` 的同一串：取号、暖机、A、两次顺序写，B、C 的记录写下标与上一版内容）、多记录流全量那一条取的续做输入
/// （`common_crash_point_observers::mount_continuation_inputs_of_the_unlink_stream`）各比一次。预期：两份摘要都相同。
/// 判别力（`crates/mutations.tsv`）：删除流交给发布边界观察者的 B 那一次上一版内容换成空，这一条红。
#[test]
fn the_unlink_stream_hands_its_shared_observers_the_same_inputs_as_the_multi_record_publish_stream()
{
    let prepared = prepare("unlink-after-multi-record-shared-observer-inputs");
    let mut pool = build_pool("unlink-after-multi-record-shared-observer-inputs-multi-record");
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    let third = sequential_write(&mut pool, &content_needing(3, 2));
    let operations = pool.retained_operations();
    let (writes_of_the_multi_record_stream, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let inputs_of_the_multi_record_stream = MultiRecordPublishBoundaryInputs {
        publishes: vec![
            multi_record_publish(
                &writes_of_the_multi_record_stream,
                &second,
                &first,
                file_content(),
            ),
            multi_record_publish(
                &writes_of_the_multi_record_stream,
                &third,
                &second,
                second_content,
            ),
        ],
    };
    assert_eq!(
        prepared.publish_boundary_inputs.digest(),
        inputs_of_the_multi_record_stream.digest(),
        "发布边界观察者另读的输入：删除流录出的与多记录流录出的逐字相同"
    );
    assert_eq!(
        prepared.mount_continuation_inputs.digest(),
        mount_continuation_inputs_of_the_unlink_stream(
            "unlink-after-multi-record-shared-observer-inputs-continuation"
        )
        .digest(),
        "可写挂载续做观察者另读的输入：删除流录出的与多记录流全量那一条取的逐字相同"
    );
}

/// 只展开写数小于 10 的段那一趟的展开（写数小于 10 的段任意真子集）整条在 CPU 判器上逐状态判：取样那一支（`sample_crash_states_on_the_cpu_with_threads_against_the_version_table`）
/// 不经选路，多机配置开不开 GPU、编没编进 gpu 特性都在 CPU 上判——每个状态两遍恢复，oracle 照这条流的版本表（A、B、C 有文件，意图那一版起没有）判。
/// 按功能步切的那几条崩溃枚举用例（[`assert_the_functional_step_is_exhaustive_and_clean`]）
/// 在门禁开 GPU 时判段走 GPU 判器，CPU 判器那一份 oracle 它跑不到，变异表里改坏它的行点名这里的用例。
/// 每个状态交 `see` 取出要核的那几样；交回整条流的状态数与按序号排好的 (序号, 取出的那几样)。只录那一趟（分片）交 `None`。线程数取每台默认重活线程数。
// gpu-exempt: cpu-judge-mutation-rows
fn segments_shorter_than_ten_judged_on_the_cpu_judge<Seen: Send>(
    prepared: &Prepared,
    see: &(dyn Fn(&CrashImage<'_>, &CrashStateJudgement) -> Seen + Sync),
) -> Option<(u64, Vec<(u64, Seen)>)> {
    let state_count = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &expansion_of_the_segments_shorter_than_ten,
    );
    let seen_by_ordinal: std::sync::Mutex<Vec<(u64, Seen)>> = std::sync::Mutex::new(Vec::new());
    let states_of_the_flow = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: "unlink_after_multi_record::segments_shorter_than_ten_on_the_cpu_judge",
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &expansion_of_the_segments_shorter_than_ten,
        0..state_count,
        singlefs_verification::heavy_threads::default_heavy_threads(),
        &|ordinal, image, judgement| {
            let seen = see(image, judgement);
            seen_by_ordinal
                .lock()
                .expect("判定线程不 panic")
                .push((ordinal, seen));
        },
    )
    .judged()?;
    let mut seen_by_ordinal = seen_by_ordinal.into_inner().expect("判定线程不 panic");
    seen_by_ordinal.sort_by_key(|(ordinal, _seen)| *ordinal);
    Some((states_of_the_flow, seen_by_ordinal))
}

/// CPU 判器一个状态上 oracle 那一半要核的几样：两遍恢复各自的 oracle 违例，与看 journal 那一遍是不是落到表里没有、比意图那一版新的根上、报没有文件。
struct OracleSideOfAnUnlinkState {
    consulted_violation: Option<String>,
    ignored_violation: Option<String>,
    landed_past_the_intent_without_a_file: bool,
}

/// 只展开写数小于 10 的段那一趟的展开在 CPU 判器上判，两遍恢复都过 oracle：恢复落到两批或最后一批的根（不在版本表里）报没有文件的状态真跑到了，
/// 照比它旧的最新那一项（意图那一版，文件从那一版起没有了）判、不是违例。
/// 被改坏的 `CrashPointReplayVersionTable::newest_older_version_if_it_has_a_file` 由 CPU 判器每个状态的 oracle 调（落到的根不在表里、报没有文件时）：
/// 只看有没有更旧的带文件的一版、不看比它旧的最新那一项是不是没有文件时，那几个状态判成违例（变异表「三·4 崩溃点重放 oracle：根不在版本表里时退回看任一更旧版有没有文件」）。
/// 同一份 oracle 的另两处也由这一条判：`version_at_the_root` 查根时不看「没有文件的那几版」（落到意图那一版的状态照 C 判成违例），
/// `newest_older_version_if_it_has_a_file` 找比根旧的「没有文件」那一版时比反了方向（落到两批、最后一批的根照 C 判成违例）；
/// 变异表「删除流两批 CPU 判器参照」「删除流最后一批 CPU 判器参照」两行点名这里：取样那一支不经选路，门禁开 GPU 时照样在 CPU 判器上判。
#[test]
fn unlink_stream_with_the_segments_shorter_than_ten_expanded_judged_on_the_cpu_judge_reads_no_file_past_the_intent_without_an_oracle_violation(
) {
    // 同一个二进制里在 CPU 判器上跑放量的那一条设一次性库：整趟拿着全仓那一份的守卫，它设不了一次性库，这一边开不到它那一份
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let prepared = prepare("unlink-after-multi-record-on-the-cpu-judge");
    let intent = prepared
        .versions_without_a_file
        .first()
        .map(|version| (version.checkpoint_txg, version.instance))
        .expect("意图那一版在没有文件的那几版里");
    let root_is_in_the_version_table =
        |instance: InstanceGeneration, checkpoint_txg: CheckpointTxg| {
            prepared.versions.iter().any(|version| {
                version.instance == instance && version.checkpoint_txg == checkpoint_txg
            }) || prepared.versions_without_a_file.iter().any(|version| {
                version.instance == instance && version.checkpoint_txg == checkpoint_txg
            })
        };
    let Some((states_of_the_flow, oracle_sides)) =
        segments_shorter_than_ten_judged_on_the_cpu_judge(&prepared, &|_image, judgement| {
            OracleSideOfAnUnlinkState {
                consulted_violation: judgement
                    .consulted_oracle
                    .as_ref()
                    .map(|violation| violation.reason.clone()),
                ignored_violation: judgement
                    .ignored_oracle
                    .as_ref()
                    .map(|violation| violation.reason.clone()),
                landed_past_the_intent_without_a_file: matches!(
                    judgement.consulted.outcome,
                    RecoveryOutcome::NoFile { .. }
                ) && judgement
                    .consulted
                    .effective_root
                    .is_some_and(|(instance, checkpoint_txg)| {
                        (checkpoint_txg, instance) > intent
                            && !root_is_in_the_version_table(instance, checkpoint_txg)
                    }),
            }
        })
    else {
        return;
    };
    assert_eq!(
        states_of_the_flow,
        1 + 11 * (3 * 3 - 1) + 7 * 3 + 9 + 15 + 63,
        "只展开写数小于 10 的段那一趟：1 + 十一个系统配置槽 2 写段各 3² − 1 + 七个记录 2 写段各 3 + 九个 1 写段各 1 + 4 写段 15 + 6 写段 63"
    );
    assert!(
        oracle_sides
            .iter()
            .any(|(_ordinal, state)| state.landed_past_the_intent_without_a_file),
        "恢复落到两批或最后一批的根、报没有文件的状态真跑到了（不在版本表里的根照比它旧的最新那一项判那一支）"
    );
    let violations: Vec<(u64, &Option<String>, &Option<String>)> = oracle_sides
        .iter()
        .filter(|(_ordinal, state)| {
            state.consulted_violation.is_some() || state.ignored_violation.is_some()
        })
        .map(|(ordinal, state)| {
            (
                *ordinal,
                &state.consulted_violation,
                &state.ignored_violation,
            )
        })
        .collect();
    assert!(
        violations.is_empty(),
        "两遍恢复都过 oracle（序号、看 journal 那一遍、不看 journal 那一遍）：{violations:?}"
    );
}

/// 两份造法写表摘要相等（J3、K2-2）：崩溃点重放那一份（[`prepare`]，不录崩溃点）与录制器那一份
/// （B、C 两次顺序写挪进公共模块之后）录下的起点镜像与整张带屏障的写表逐字节相同、段一样切、被判的根槽写与版本表相同。
/// 两份都调公共模块里同一个 `sequential_write`；崩溃点重放那一份的写法与挪之前逐字相同。摘要打一行 `CRASH_POINT_REPLAY_WRITE_TABLE`。
#[test]
fn the_unlink_stream_writes_the_same_table_through_the_crash_point_recorder_as_without_it() {
    use singlefs_verification::crash_identity::{step_write_table_with_barriers, SectorImage};
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

/// 删除流与多记录流共享到 C（H3）：计划里的点细到每个屏障段（`<入口>.<周期>.<步>`），按点名的前缀认
/// 公共模块设的四个节点——取号加暖机、新池新建文件、B、C（B、C 都是 `sequential_write`：同一个入口里点名重复出现，就是下一个节点起头）。
/// 两条流这四个节点的每一个点身份、判器另读的输入、状态序号区间与块逐个相同；计划哈希除 C 尾（C 那次发布根槽写之后的系统配置槽轮换，
/// 它按发布分的态数要往下游找：下游第一次根槽写进块键）之外逐个相同（块键全同、判定共享），C 尾两条流不同；整条流最后全部持久那一个状态
/// 不归 C，各自另立成流自己的末节点 `end_of_flow`（`crash_amplification::plan_crash_points`）：多记录流在 C 之后只剩它，删除流在 C 之后
/// 还有意图、各批与它自己的末节点；两条流的末节点身份与计划哈希各不相同。
#[test]
fn the_unlink_stream_shares_the_nodes_through_c_with_the_multi_record_publish_stream_and_each_stream_ends_in_its_own_node(
) {
    use singlefs_verification::crash_amplification::{
        plan_crash_points, CrashFlow, PlannedCrashPoint, END_OF_FLOW_NODE_NAME,
    };
    let unlink = record_through_the_crash_point_recorder("unlink-shares-the-prefix", false);
    let multi_record =
        record_through_the_crash_point_recorder("multi-record-shares-the-prefix", true);
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 6 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
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
    let tail_of_c = points_through_c - 1;
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
        if index == tail_of_c {
            assert_ne!(
                unlink_point.plan_hash, multi_record_point.plan_hash,
                "C 尾（第 {index} 个点 {name}）计划哈希不同：它按发布分的态数要往下游找，多记录流后面没有根槽写、删除流归意图那次发布，\
                 下游第一次根槽写进块键（用户 2026-10-11 定「乙」）"
            );
        } else {
            assert_eq!(
                unlink_point.plan_hash, multi_record_point.plan_hash,
                "第 {index} 个点（{name}）计划哈希相同：C 尾之前的点照旧共享，整条流最后那一个状态不归 C 了"
            );
        }
        assert_eq!(
            (unlink_point.ordinals.clone(), unlink_point.blocks()),
            (
                multi_record_point.ordinals.clone(),
                multi_record_point.blocks()
            ),
            "第 {index} 个点（{name}）状态序号区间与块相同：C 尾之前块键全同、判定共享"
        );
    }
    assert!(
        !multi_record_plan.points[tail_of_c].blocks().is_empty(),
        "C 尾（{}）有状态",
        multi_record_plan.points[tail_of_c].name
    );
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

/// 一份计划里的一块：点下标与点名、块键的四样（与 `verdict_store::VerdictBlockKey` 同：复用键、身份、计划哈希、块起点）、
/// 一块按发布分的态数（按段的序号区间现算：每段与这一块重叠的那几个归这一段的那次发布，最后全部持久那一个归 `every_write_persisted`；
/// 与崩溃放量 GPU 判器存计数向量的归法同一种）。
struct BlockCountedStateByState {
    point_index: usize,
    point_name: String,
    block_key: (
        singlefs_verification::crash_identity::ReuseKey,
        String,
        [u8; singlefs_verification::sha256::DIGEST_BYTES],
        u64,
    ),
    states_by_publish: std::collections::BTreeMap<
        singlefs_verification::crash::CrashPointReplayPublishOfState,
        u64,
    >,
}

/// 计划里的每一块按发布分的态数（按段的序号区间现算）；展开照计划的（`CrashPlan::segment_expansion`）。
fn blocks_of_the_plan_counted_state_by_state(
    flow: &singlefs_verification::crash_amplification::CrashFlow<'_>,
    plan: &singlefs_verification::crash_amplification::CrashPlan,
) -> Vec<BlockCountedStateByState> {
    use singlefs_verification::crash::{
        crash_state_ranges_by_segment, publish_of_each_segment, CrashPointReplayPublishOfState,
    };
    let point_expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let (ranges_by_segment, state_count) =
        crash_state_ranges_by_segment(flow.base, flow.writes, flow.segments, &point_expansion);
    let publish_of_segment = publish_of_each_segment(flow.writes, flow.segments);
    let mut blocks = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let mut states_by_publish = std::collections::BTreeMap::new();
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            for (range, publish) in ranges_by_segment.iter().zip(&publish_of_segment) {
                let overlap = range
                    .end
                    .min(ordinals.end)
                    .saturating_sub(range.start.max(ordinals.start));
                if overlap > 0 {
                    *states_by_publish.entry(*publish).or_insert(0) += overlap;
                }
            }
            if ordinals.start < state_count && ordinals.end >= state_count {
                *states_by_publish
                    .entry(CrashPointReplayPublishOfState::EveryWritePersisted)
                    .or_insert(0) += 1;
            }
            blocks.push(BlockCountedStateByState {
                point_index,
                point_name: point.name.clone(),
                block_key: (
                    point.reuse_key,
                    point.identity.clone(),
                    point.plan_hash,
                    local.start,
                ),
                states_by_publish,
            });
        }
    }
    blocks
}

/// 落在点下标 `points` 那几个点上的块，按计划里的次序。
fn blocks_of_the_points(
    blocks: &[BlockCountedStateByState],
    points: std::ops::Range<usize>,
) -> Vec<&BlockCountedStateByState> {
    blocks
        .iter()
        .filter(|block| points.contains(&block.point_index))
        .collect()
}

/// 崩溃点重放枚举（每个状态照判，不挂观察者）按计划的展开数出来的 `CrashPointReplayTally`。
fn crash_point_replay_tally_under_the_expansion_of_the_plan(
    recorded: &RecordedThroughTheCrashPointRecorder,
    item_label: &str,
    flow: &singlefs_verification::crash_amplification::CrashFlow<'_>,
    plan: &singlefs_verification::crash_amplification::CrashPlan,
) -> Option<CrashPointReplayTally> {
    let point_expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let item_name = format!("unlink_after_multi_record::under_the_plan::{item_label}");
    // 同一个二进制里在 CPU 判器上跑放量的那一条设一次性库：开库期间拿着全仓那一份的守卫，它设不了一次性库，这一边开不到它那一份
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::new(&recorded.versions, &recorded.versions_without_a_file),
        &point_expansion,
        &[],
        &[],
    )
    .judged()
}

/// 块上存的按发布分的态数跟着块键走（J7：每一趟照判复用块的计数向量，`count_vector_disagreements_of_the_flow`）：
/// 按发布分的态数要往下游找的块，下游第一次根槽写进块键（`crash_amplification::plan_crash_points`）。
/// 删除流与多记录流共享到 C：C 尾那一块（多记录流末节点之前最后一个点，C 那次发布根槽写之后的系统配置槽轮换）在多记录流里后面没有根槽写、
/// 归 `after_the_last_root`，在删除流里归意图那次发布，两条流块键不同；C 尾之前的每一块两条流块键相同、逐状态数归哪次发布也相同。
/// 每条流逐块按发布分的态数（按段的序号区间现算，CPU 判器存计数向量照的那一份），全部块合起来与崩溃点重放枚举的
/// `CrashPointReplayTally::states_by_publish` 逐项相同（同一种展开）。
// crash-case-check:not-a-crash-case 两条合成流只展开写数不超过 6 的段，一两百个状态，只比按发布分的计数，单跑几秒
#[test]
fn the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_crash_point_replay_tally(
) {
    use singlefs_verification::crash::CrashPointReplayPublishOfState;
    use singlefs_verification::crash_amplification::{
        plan_crash_points, CrashFlow, END_OF_FLOW_NODE_NAME,
    };
    let unlink = record_through_the_crash_point_recorder("unlink-keys-the-tail-of-c-apart", false);
    let multi_record =
        record_through_the_crash_point_recorder("multi-record-keys-the-tail-of-c-apart", true);
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 6 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let unlink_flow = CrashFlow {
        base: &unlink.base,
        writes: &unlink.writes,
        segments: &unlink.segments,
        judged_root_index: unlink.judged_root_index,
        versions: &unlink.versions,
        versions_without_a_file: &unlink.versions_without_a_file,
        expansion: &expansion,
        crash_points: unlink.crash_points.clone(),
    };
    let multi_record_flow = CrashFlow {
        base: &multi_record.base,
        writes: &multi_record.writes,
        segments: &multi_record.segments,
        judged_root_index: multi_record.judged_root_index,
        versions: &multi_record.versions,
        versions_without_a_file: &multi_record.versions_without_a_file,
        expansion: &expansion,
        crash_points: multi_record.crash_points.clone(),
    };
    let unlink_plan = plan_crash_points(&unlink_flow);
    let multi_record_plan = plan_crash_points(&multi_record_flow);
    let unlink_blocks = blocks_of_the_plan_counted_state_by_state(&unlink_flow, &unlink_plan);
    let multi_record_blocks =
        blocks_of_the_plan_counted_state_by_state(&multi_record_flow, &multi_record_plan);

    // 一、每条流：全部块逐状态数的按发布分的态数合起来，与崩溃点重放枚举的计数逐项相同
    for (stream, item_label, recorded, flow, plan, blocks) in [
        (
            "删除流",
            "unlink",
            &unlink,
            &unlink_flow,
            &unlink_plan,
            &unlink_blocks,
        ),
        (
            "多记录流",
            "multi_record",
            &multi_record,
            &multi_record_flow,
            &multi_record_plan,
            &multi_record_blocks,
        ),
    ] {
        let mut summed_over_the_blocks: std::collections::BTreeMap<
            singlefs_verification::crash::CrashPointReplayPublishOfState,
            u64,
        > = std::collections::BTreeMap::new();
        for block in blocks {
            for (publish, states) in &block.states_by_publish {
                *summed_over_the_blocks.entry(*publish).or_insert(0) += states;
            }
        }
        let Some(tally) = crash_point_replay_tally_under_the_expansion_of_the_plan(
            recorded, item_label, flow, plan,
        ) else {
            continue;
        };
        singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
            tally.states,
            &format!("{stream}：崩溃点重放枚举到了状态"),
        );
        // 块按计划排（用例自己的展开，全量域）：缩过域的那一趟枚举到的本来就比它少
        singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
            &tally.states_by_publish,
            &summed_over_the_blocks,
            &format!("{stream}：逐块逐状态数的按发布分的态数合起来，与崩溃点重放枚举的 CrashPointReplayTally::states_by_publish 逐项相同"),
        );
    }

    // 二、C 尾：两条流同一个树节点、同样几块，块键不同，各自归各自下游的那次发布
    let tail_of_c = multi_record_plan.points.len() - 2;
    assert_eq!(
        multi_record_plan.points[tail_of_c + 1].name,
        END_OF_FLOW_NODE_NAME,
        "多记录流 C 尾之后只剩末节点"
    );
    assert_eq!(
        unlink_plan.points[tail_of_c].identity, multi_record_plan.points[tail_of_c].identity,
        "C 尾在两条流里是树上同一个节点：设点文件、名字、复用键相同"
    );
    let intent = unlink
        .versions_without_a_file
        .first()
        .expect("删除流的版本表带意图那一版");
    let unlink_tail = blocks_of_the_points(&unlink_blocks, tail_of_c..tail_of_c + 1);
    let multi_record_tail = blocks_of_the_points(&multi_record_blocks, tail_of_c..tail_of_c + 1);
    assert!(!multi_record_tail.is_empty(), "C 尾至少一块");
    assert_eq!(
        unlink_tail.len(),
        multi_record_tail.len(),
        "C 尾两条流一样多块"
    );
    for (unlink_block, multi_record_block) in unlink_tail.iter().zip(&multi_record_tail) {
        let place = format!(
            "C 尾 {} 块 {}",
            multi_record_block.point_name, multi_record_block.block_key.3
        );
        assert_ne!(
            unlink_block.block_key, multi_record_block.block_key,
            "{place}：两条流块键不同（下游第一次根槽写进了计划哈希）"
        );
        assert_eq!(
            multi_record_block
                .states_by_publish
                .keys()
                .copied()
                .collect::<Vec<CrashPointReplayPublishOfState>>(),
            vec![CrashPointReplayPublishOfState::AfterTheLastRoot],
            "{place}：多记录流 C 尾后面没有根槽写"
        );
        assert!(
            !unlink_block.states_by_publish.is_empty()
                && unlink_block.states_by_publish.keys().all(|publish| matches!(
                    publish,
                    CrashPointReplayPublishOfState::UpToTheRootOf { instance, checkpoint_txg, .. }
                        if (*instance, *checkpoint_txg) == (intent.instance, intent.checkpoint_txg)
                )),
            "{place}：删除流 C 尾归意图那次发布，实际 {:?}",
            unlink_block.states_by_publish
        );
        assert_eq!(
            unlink_block.states_by_publish.values().sum::<u64>(),
            multi_record_block.states_by_publish.values().sum::<u64>(),
            "{place}：两条流这一块状态数相同"
        );
    }

    // 三、C 尾之前：两条流逐块块键相同，逐状态数归哪次发布也相同
    let unlink_before_the_tail = blocks_of_the_points(&unlink_blocks, 0..tail_of_c);
    let multi_record_before_the_tail = blocks_of_the_points(&multi_record_blocks, 0..tail_of_c);
    assert!(!multi_record_before_the_tail.is_empty(), "C 尾之前至少一块");
    assert_eq!(
        unlink_before_the_tail.len(),
        multi_record_before_the_tail.len(),
        "C 尾之前两条流一样多块"
    );
    let disagreements: Vec<String> = unlink_before_the_tail
        .iter()
        .zip(&multi_record_before_the_tail)
        .filter(|(unlink_block, multi_record_block)| {
            unlink_block.block_key != multi_record_block.block_key
                || unlink_block.states_by_publish != multi_record_block.states_by_publish
        })
        .map(|(unlink_block, multi_record_block)| {
            format!(
                "点 {} 块 {}：块键相同 {}，删除流里按发布分 {:?}，多记录流里 {:?}",
                multi_record_block.point_name,
                multi_record_block.block_key.3,
                unlink_block.block_key == multi_record_block.block_key,
                unlink_block.states_by_publish,
                multi_record_block.states_by_publish
            )
        })
        .collect();
    assert!(
        disagreements.is_empty(),
        "C 尾之前两条流逐块块键相同、按发布分的态数相同（判定共享）：{disagreements:?}"
    );
}
