//! 挂在标准前缀节点上的观察者（J4）：观察者的代码、它另读的流程输入怎么录、照登记表
//! （`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，名字住在公共模块）挂上观察者跑一趟崩溃放量的入口。
//! 这一份单独放：公共模块（标准前缀的设点文件）不引它，标准前缀上的点的 CPU 判器版本不收观察者代码（H4「观察者代码不进计划哈希、
//! 不进判器版本」）；观察者的身份是它的入口 fn（[`observe_the_mount_continuation`]）的闭包键，入口登记在
//! `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES`。
//! 可写挂载续做观察者从删除流的崩溃枚举用例搬来，那一份用例照旧调这里的同一个。
//! 两段（观察者走两段、池级 checker 上 GPU、按恢复结果去重）：观察者在录入段的 CPU 上真跑可写挂载与续做，
//! 交回挂载发的写与挂完的盘；挂完的盘上的池级 checker 不在这里跑，判段从库里读回来判（配了 GPU 上卡）。
#![allow(
    dead_code,
    reason = "共用的观察者模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use super::common::{build_pool, geometry, parameters};
use super::common_crash_points;
use super::common_crash_points::{
    content_needing, publish_the_next_unlink_step, publish_the_unlink_intent, sequential_write,
    ObserverOfTheStandardPrefix, DELETION_BATCHES_BEFORE_THE_FINAL_BATCH,
    OBSERVERS_ON_THE_STANDARD_PREFIX,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    MountOutput, ShadowLedger,
};
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_verification::crash::CrashPointReplayObserverCounts;
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserveACrashState,
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication, ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_state_enumeration::{
    AttachedCrashPointReplayObserver, AttachedCrashPointReplayObserverHandingBackAnImage,
    CrashPointReplayObserver, CrashPointReplayObserverAttachment, CrashPointReplayObserverCodeFile,
    CrashPointReplayObserverHandingBackAnImage, CrashPointReplayObserverPoints,
};
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::scenario::deletion_history_switches;
use singlefs_verification::segments::StepKind;
use singlefs_verification::writable_mount_phases::materialized_crash_image;
use singlefs_verification::{RecordingBlockDevice, SharedStream};
use std::sync::Arc;

/// 判段在可写挂载续做之后的盘上判池级 checker、一条不变量都不判违例时加一的计数（每个状态挂完的盘都判过：计数等于状态数）。
pub const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 这一份的仓内路径（观察者代码文件，与 `file!()` 归一之后相同）。
pub const FILE_OF_THE_CRASH_POINT_OBSERVERS: &str =
    "crates/singlefs-verification/tests/common_crash_point_observers/mod.rs";

// ============================== 多记录发布的发布边界观察者（从多记录流与删除流的崩溃枚举用例搬来，两条流挂同一个） ==============================

/// 发布边界观察者的名字：多记录流与删除流的崩溃枚举用例挂的是同一个（代码文件都点名这一份，登记表里一行、一把闭包键），
/// 两条流共有的前缀块上它的结论一处判过、另一处复用（按功能挂到对应的段：发布边界那一半两条流都挂）。
pub const MULTI_RECORD_PUBLISH_BOUNDARY_OBSERVER_NAME: &str = "multi_record_publish_boundary";

/// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
pub fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
        .map(|(index, _)| index)
        .collect()
}

/// 一次多条记录的发布在写表里的样子：它每一条记录的两份写各在写表的哪一格，它的根在哪一版、上一版是哪一版，上一版的内容。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiRecordPublish {
    /// 这次发布的记录按 jsn 升序，每条两盘各一份：写表下标。
    pub record_writes_in_order: Vec<Vec<usize>>,
    pub version: (InstanceGeneration, CheckpointTxg),
    pub version_before: (InstanceGeneration, CheckpointTxg),
    pub content_before: Vec<u8>,
}

impl MultiRecordPublish {
    fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
        copies.iter().any(|copy| persisted[*copy])
    }

    /// 这一状态里这次发布的第一条记录至少落了一份，而末条一份都没落：前缀停在这次发布中间。
    fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
        let first = self.record_writes_in_order.first().expect("至少两条记录");
        let last = self.record_writes_in_order.last().expect("至少两条记录");
        Self::is_any_copy_persisted(persisted, first)
            && !Self::is_any_copy_persisted(persisted, last)
    }
}

/// 一次多条记录的发布（`output`，上一版是 `before`、它的内容 `content_before`）在写表里的样子。
pub fn multi_record_publish(
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
        version: (output.root.instance, output.root.checkpoint_txg),
        version_before: (before.root.instance, before.root.checkpoint_txg),
        content_before,
    }
}

/// 发布边界观察者另读的流程输入（M2-3 乙：进观察者结果的键）：B、C 两次多条记录的发布。多记录流与删除流到 C 为止写表相同，
/// 两条流录出的这一份逐字相同、摘要相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiRecordPublishBoundaryInputs {
    pub publishes: Vec<MultiRecordPublish>,
}

impl MultiRecordPublishBoundaryInputs {
    /// 另读的输入的摘要（进观察者的版本）：每次发布一行（记录写的下标、它与它之前那一版的根、之前那一版的内容），取 SHA-256；录制流一变它就变，
    /// 看过的块不按旧的复用。
    pub fn digest(&self) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
        let mut text = String::new();
        for publish in &self.publishes {
            text.push_str(&format!(
                "record_writes_in_order={:?} version={:?} version_before={:?} content_before={:?}\n",
                publish.record_writes_in_order,
                publish.version,
                publish.version_before,
                publish.content_before
            ));
        }
        singlefs_verification::sha256::sha256_digest(text.as_bytes())
    }
}

/// 每个状态上核发布边界（D23（journal 的角色与格式） 已定项 14 第六条）：一次发布的第一条记录落了、末条没落 ⇒ 那次发布整体不施加，
/// 实际走的根是上一版的、读回上一版、文件是旧长度（I-4.3（提交原子））。这一条 oracle 判不出来：崩溃点重放按屏障切段，记录段里的状态上这次发布的
/// 单元全都落了，半次发布施加上去读回的也是完整的新内容。计数 `cut_in_multi_record_publish_<发布序号>`。
pub fn observe_the_publish_boundary_of_the_multi_record_publishes(
    inputs: &MultiRecordPublishBoundaryInputs,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    for (publish_index, publish) in inputs.publishes.iter().enumerate() {
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

/// 崩溃枚举用例挂发布边界观察者：名字、代码文件（这一份）、另读的输入两条流都一样，挂整条流。
pub fn publish_boundary_observer_of_a_crash_enumeration_case<'observer>(
    inputs: &MultiRecordPublishBoundaryInputs,
    observe: &'observer ObserveACrashState<'observer>,
) -> AttachedCrashPointReplayObserver<'observer> {
    AttachedCrashPointReplayObserver {
        observer: CrashPointReplayObserver {
            name: MULTI_RECORD_PUBLISH_BOUNDARY_OBSERVER_NAME,
            extra_inputs_digest: inputs.digest(),
            observe,
        },
        attachment: CrashPointReplayObserverAttachment {
            points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
            code_file: CrashPointReplayObserverCodeFile::SharedFile(
                FILE_OF_THE_CRASH_POINT_OBSERVERS,
            ),
        },
    }
}

/// 崩溃枚举用例挂可写挂载续做观察者：名字、代码文件与崩溃放量那一路挂在标准前缀上的同一个（`common_crash_points::ObserverOfTheStandardPrefix`，
/// 登记表里同一行、同一把闭包键），另读的输入是删除流的发布表（多记录流取 [`mount_continuation_inputs_of_the_unlink_stream`]，删除流取自己录的那一份，
/// 两份逐字相同；两条流挂同一个 mount_continuation），挂整条流，按恢复镜像去重。
pub fn mount_continuation_observer_of_a_crash_enumeration_case<'observer>(
    inputs: &MountContinuationInputs,
    observe: &'observer ObserveACrashStateHandingBackAnImage<'observer>,
) -> AttachedCrashPointReplayObserverHandingBackAnImage<'observer> {
    AttachedCrashPointReplayObserverHandingBackAnImage {
        observer: CrashPointReplayObserverHandingBackAnImage {
            name: ObserverOfTheStandardPrefix::WritableMountAndContinuation.name(),
            extra_inputs_digest: inputs.digest(),
            observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
        attachment: CrashPointReplayObserverAttachment {
            points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
            code_file: CrashPointReplayObserverCodeFile::SharedFile(
                FILE_OF_THE_CRASH_POINT_OBSERVERS,
            ),
        },
    }
}

// ============================== 可写挂载续做观察者（J4） ==============================

/// 删除的一次发布落到之后，可写挂载的续做还差几步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeletionPublishContinuation {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub continuation_steps_from_this_version: usize,
}

/// 可写挂载续做观察者另读的流程输入（M2-3 乙：进观察者结果的键）：带文件的几版、删除那几次发布各自落到它之后续做还差几步。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountContinuationInputs {
    pub versions_with_a_file: Vec<(InstanceGeneration, CheckpointTxg)>,
    pub deletion_publishes: Vec<DeletionPublishContinuation>,
}

impl MountContinuationInputs {
    /// 带文件的几版（A、B、C 的输出）与删除那几步（意图、各批、最后一批的输出，按发布的次序）：最后一批落到之后续做 0 步，往前每一步多一步。
    pub fn of_the_publishes(
        versions_with_a_file: &[&TransactionOutput],
        deletion_steps: &[TransactionOutput],
    ) -> Self {
        let version_of =
            |output: &TransactionOutput| (output.root.instance, output.root.checkpoint_txg);
        Self {
            versions_with_a_file: versions_with_a_file
                .iter()
                .map(|output| version_of(output))
                .collect(),
            deletion_publishes: deletion_steps
                .iter()
                .enumerate()
                .map(|(step_index, step)| DeletionPublishContinuation {
                    version: version_of(step),
                    continuation_steps_from_this_version: deletion_steps.len() - 1 - step_index,
                })
                .collect(),
        }
    }

    /// 另读的输入的摘要（进观察者的版本，`crash_amplification::ObserverOnTheNodes` 的 `extra_inputs_digest`）：逐版一行写下、取 SHA-256。
    pub fn digest(&self) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
        let mut text = String::from("mount continuation inputs 1\n");
        for (instance, checkpoint_txg) in &self.versions_with_a_file {
            text.push_str(&format!(
                "version_with_a_file instance={} txg={}\n",
                instance.0, checkpoint_txg.0
            ));
        }
        for publish in &self.deletion_publishes {
            text.push_str(&format!(
                "deletion_publish instance={} txg={} continuation_steps={}\n",
                publish.version.0 .0,
                publish.version.1 .0,
                publish.continuation_steps_from_this_version
            ));
        }
        singlefs_verification::sha256::sha256_digest(text.as_bytes())
    }
}

/// 录一遍删除流（多记录流到 C 之后 unlink 第一个文件：意图、两批、最后一批），交回续做观察者另读的输入。
pub fn mount_continuation_inputs_of_the_unlink_stream(tag: &str) -> MountContinuationInputs {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second = sequential_write(&mut pool, &content_needing(2, 1));
    let third = sequential_write(&mut pool, &content_needing(3, 2));
    let mut deletion_steps = vec![publish_the_unlink_intent(&mut pool)];
    for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
        deletion_steps.push(publish_the_next_unlink_step(&mut pool));
    }
    MountContinuationInputs::of_the_publishes(&[&first, &second, &third], &deletion_steps)
}

/// 恢复落到的是哪一类版本：挂载之后该是哪一种池版本、续做还差几步按它定。
#[derive(Clone, Copy, Debug)]
enum LandedVersion {
    /// mkfs 或两次暖机：A 之前，根下面没有树。
    BeforeTheFirstFile,
    /// A、B、C 之一：有文件、队列空。
    WithAFile,
    /// 意图、两批、最后一批之一。
    OneOfTheDeletionPublishes(DeletionPublishContinuation),
}

fn landed_version(
    inputs: &MountContinuationInputs,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> LandedVersion {
    let effective_root = effective_root.expect("每个状态都择到根（oracle 另判）");
    if let Some(deletion_publish) = inputs
        .deletion_publishes
        .iter()
        .find(|publish| publish.version == effective_root)
    {
        return LandedVersion::OneOfTheDeletionPublishes(*deletion_publish);
    }
    if inputs.versions_with_a_file.contains(&effective_root) {
        return LandedVersion::WithAFile;
    }
    LandedVersion::BeforeTheFirstFile
}

/// 这一状态上可写挂载一次（批大小同录流，三个容量照产品路径）：续做把队列走到空，步数等于恢复落到的那一版还差几步；
/// 落到删除那几版上的，挂完是「有树、文件集合为空」那一版；落到 A、B、C 上的挂完照旧有文件；落到 A 之前的挂完照旧没有树。
/// 挂载发的每一次写录在一条录制流上，连同挂完的盘交回：判段在挂完的盘上跑池级 checker（一条不变量都不许判违例，判干净时
/// [`POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER`] 加一；配了 GPU 上卡）。删除流的崩溃枚举用例与挂在标准前缀上的放量观察者
/// （`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`）调的都是这一份。可写挂载与续做真跑实现，只能在 CPU 上
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。它交回的只随镜像字节、恢复报告与另读的输入变，
/// 录入段按恢复镜像去重。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn observe_the_mount_continuation(
    inputs: &MountContinuationInputs,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let materialized = materialized_crash_image(image);
    let mount_parameters = parameters();
    // 记读：挂载读过的每一段（连同 journal 记录槽提示问到的段）交回去，录入段按它算读集见证
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
    let landed = landed_version(inputs, report.effective_root);
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
    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
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
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            Arc::as_ref(output)
        }
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
        .map(Arc::as_ref)
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

/// 每个状态挂完的盘都过了池级 checker；续做发成的每一步都核了代与事务号（续做 k 步的状态各核 k 步）。
pub fn assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
    counts: &CrashPointReplayObserverCounts,
    states: u64,
) {
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        states,
        "每个状态挂完的盘都过了池级 checker（判段在挂完的盘上判，配了 GPU 上卡）"
    );
    assert_eq!(
        counts.count_named("continuation_steps_numbered_next_in_the_instance"),
        (1..=3)
            .map(|steps| steps * counts.count_named(&format!("mount_continuation_steps_{steps}")))
            .sum::<u64>(),
        "续做发成的每一步都核了代与事务号"
    );
}

/// 一条走过标准前缀的流（多记录流、删除流：前缀由 `common_crash_points::record_the_multi_record_publishes` 设点）按环境跑一趟崩溃放量，连同登记表
/// `common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX` 上挂在它经过的标准前缀节点上的全部观察者。观察者另读的输入照录（M4 第 10 条：
/// 单跑一项要的别项流程照录）：可写挂载续做观察者读删除流的发布表，这里录一遍删除流取它（[`mount_continuation_inputs_of_the_unlink_stream`]），
/// 多记录流单跑也一样。观察者走两段：录入段在 CPU 上可写挂载、续做（按恢复镜像去重，同一张镜像只挂一次），
/// 判段在挂完的盘上判池级 checker（配了 GPU 上卡）；两段各打一行「现判 N、去重复用 M」。观察者计数的断言每一趟都判（J7，这一趟拼的与库里复用的
/// 块读回来的合起来）：经过标准前缀的每个状态都可写挂载过一次、挂完的盘都过了池级 checker、续做发成的每一步都核了代与事务号，一个状态都不判红。
/// 只录、只排派活计划、叫停、判段报本次未跑的那一趟不判。其余同 `common_crash_points::amplify_prepared_flow_against_the_version_table`。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的八样各自独立，收成一个结构体只是把八个字段挪个地方"
)]
pub fn amplify_a_flow_through_the_standard_prefix(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    versions_without_a_file: &[singlefs_verification::memory_pool::PublishedVersionWithoutAFile],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
) -> singlefs_verification::crash_amplification::EnvironmentRun {
    amplify_a_flow_through_the_standard_prefix_expanding_up_to(
        flow_name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        versions_without_a_file,
        crash_points,
        singlefs_verification::crash_amplification::EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
    )
}

/// 同 [`amplify_a_flow_through_the_standard_prefix`]，展开上限由调用方给（`expand_up_to_of_the_flow`：一段的写数不超过它就展开全部真子集，
/// 环境里缩域的变量照样只往小缩）。只给缩了规模的那几条用：全量那几条照引擎常量走 [`amplify_a_flow_through_the_standard_prefix`]。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的八样各自独立，另加这一趟的展开上限，收成一个结构体只是把九个字段挪个地方"
)]
pub fn amplify_a_flow_through_the_standard_prefix_expanding_up_to(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    versions_without_a_file: &[singlefs_verification::memory_pool::PublishedVersionWithoutAFile],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    expand_up_to_of_the_flow: usize,
) -> singlefs_verification::crash_amplification::EnvironmentRun {
    use singlefs_verification::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_verification::crash_identity::ObserverName;
    let mount_continuation_inputs =
        mount_continuation_inputs_of_the_unlink_stream(&format!("{flow_name}-observer-inputs"));
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation(&mount_continuation_inputs, image, report, counts)
        };
    let mut every_observer: Vec<ObserverOfTheStandardPrefix> = Vec::new();
    for (_entry_name, observers_of_the_node) in OBSERVERS_ON_THE_STANDARD_PREFIX.iter() {
        for observer in *observers_of_the_node {
            if !every_observer.contains(observer) {
                every_observer.push(*observer);
            }
        }
    }
    let observers: Vec<ObserverOnTheNodes<'_>> = every_observer
        .iter()
        .map(|observer| ObserverOnTheNodes {
            observer_name: ObserverName::new(observer.name()).expect("观察者的名字非空、不长"),
            observer_code_file: FILE_OF_THE_CRASH_POINT_OBSERVERS.to_string(),
            extra_inputs_digest: match observer {
                ObserverOfTheStandardPrefix::WritableMountAndContinuation => {
                    mount_continuation_inputs.digest()
                }
            },
            nodes: OBSERVERS_ON_THE_STANDARD_PREFIX
                .iter()
                .filter(|(_entry_name, observers_of_the_node)| {
                    observers_of_the_node.contains(observer)
                })
                .map(|(entry_name, _observers_of_the_node)| NodeOfAnObserver {
                    setting_file: common_crash_points::THIS_FILE.to_string(),
                    entry_name: (*entry_name).to_string(),
                })
                .collect(),
            observe: match observer {
                ObserverOfTheStandardPrefix::WritableMountAndContinuation => {
                    ObserverCode::HandsBackAnImage {
                        observe: &observe_the_mount,
                        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                    }
                }
            },
        })
        .collect();
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            flow_name,
            base,
            writes,
            segments,
            judged_root_index,
            versions,
            versions_without_a_file,
            crash_points,
            expand_up_to_of_the_flow,
            &observers,
        );
    if let Some(recording) = &run.observer_recording {
        // 录入段的「现判 N、去重复用 M」：这一趟真挂的恢复镜像张数、镜像已经看过（这一趟早先的状态或库里存着的）因而不再挂的状态数
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_RECORDING flow={flow_name} images_observed_fresh={} states_reusing_an_observed_image={} buckets={} witness_misses={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={}",
            recording.images_observed_here,
            recording.states_reusing_an_observed_image,
            recording.buckets,
            recording.witness_misses,
            recording.blocks_recorded_here,
            recording.states_recorded_here,
            recording.blocks_already_recorded
        );
    }
    if let Some(observing) = &run.observing {
        // 判段的「现判 N、去重复用 M」：这一趟在挂完的盘上判池级 checker 的张数（其中在卡上的）、库里这一把判器版本早先判过的张数
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
        for observer in &every_observer {
            let name = observer.name();
            let states = observing
                .states_of_each_observer
                .get(name)
                .copied()
                .unwrap_or(0);
            assert!(
                states > 0,
                "流 {flow_name} 经过标准前缀，观察者 {name} 一个状态都没看"
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
            match observer {
                ObserverOfTheStandardPrefix::WritableMountAndContinuation => {
                    assert_eq!(
                        (0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1)
                            .map(|steps| count_named(&format!("mount_continuation_steps_{steps}")))
                            .sum::<u64>(),
                        states,
                        "流 {flow_name}：经过标准前缀的每个状态都可写挂载过一次"
                    );
                    assert_eq!(
                        count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
                        states,
                        "流 {flow_name}：每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
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
                }
            }
        }
        assert_the_observers_attached_by_the_provenance_saw_the_flow(flow_name, observing);
    }
    run
}

/// 崩溃放量那一处按来历挂的观察者（`singlefs_verification::crash_point_replay_provenance`：登记位上、来历是 mkfs 返回之后的流挂两槽全废盘面观察者，
/// 它在判段从判器交回的逐盘自证槽数看）在这条流上挂上了、看过状态、一个都没判红，看过的状态数计数等于它看过的状态数。这一份不挂它，只核跑流那一处挂了。
fn assert_the_observers_attached_by_the_provenance_saw_the_flow(
    flow_name: &str,
    observing: &singlefs_verification::crash_amplification::ObservingOfTheRun,
) {
    use singlefs_verification::crash_point_replay_provenance::{
        observers_attached_to_the_flow, RunOfTheFlow,
    };
    let attached = observers_attached_to_the_flow(RunOfTheFlow::CrashAmplification, flow_name)
        .unwrap_or_else(|refusal| panic!("{}", refusal.sentence()));
    assert!(
        !attached.is_empty(),
        "流 {flow_name} 走过标准前缀、登在崩溃放量的 ITEMS 上、来历是 mkfs 返回之后：按来历挂两槽全废盘面观察者"
    );
    for observer in attached {
        let name = observer.observer_name();
        let states = observing
            .states_of_each_observer
            .get(name)
            .copied()
            .unwrap_or(0);
        assert!(
            states > 0,
            "流 {flow_name}：按来历挂的观察者 {name} 一个状态都没看（跑流那一处没挂上）"
        );
        assert_eq!(
            observing
                .red_states_of_each_observer
                .get(name)
                .copied()
                .unwrap_or(0),
            0,
            "流 {flow_name}：按来历挂的观察者 {name} 判红的状态（这一趟看出来的前几个）：{:?}",
            observing.red_texts
        );
        assert_eq!(
            observing
                .counts_of_each_observer
                .get(name)
                .and_then(|counts| counts.get(observer.states_seen_counter()))
                .copied()
                .unwrap_or(0),
            states,
            "流 {flow_name}：按来历挂的观察者 {name} 看过的每个状态都记了数"
        );
    }
}
