//! 三·6 干跑那两段删除历史（`scenario::run_deletion_history`，D8（核心索引结构） 已定项 16）在录制流里怎么分段：
//! 每次删除类发布重写几个单元（每盘）、按什么次序发，以及卸载与挂载那两截按 core 的发布写账切开之后每段对不对得上。
//! 进程里那三批的单元数照 `unlink_single_file_publishes.rs` 现钉的角色集合：意图 13、3 → 2 那一批 12、最后一批 10；2 → 1 那一批 11。
//! 挂载续做那三批在卸载、写行与暖机之后发，分配记录树根之下多改一对叶（`t5@0.0.62`、`t5@0.1.62`），后两批各多 2 个；
//! 分配记录树根之下那几片之外的角色数两段历史逐批相同（9、8、7、6）。
//! H3（E172 跑前登记第五节 5.3）只钉它自己的形状：2 号 size 0 的 inode 先建、与 1 号同一片叶，意图时间晚顺序写 60 秒、各批照抄。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::records::InodeRecord;
use singlefs_core::transaction::{PoolVersion, TransactionOutput, TransactionUnit};
use singlefs_core::write_accounting::WrittenStructureKind;
use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{
    e142_parameters, run_deletion_history, spans_of_consecutive_parts_by_their_write_calls,
    DeletionHistory, DeletionHistoryPublishKind, DeletionHistoryRun, DeletionHistorySegment,
};
use singlefs_harness::{
    RecordedOperation, RecordedOperationKind, RecordingBlockDevice, SharedStream,
};

/// 两块 4 GiB 内存盘（E142 那套几何）上跑一段删除历史，连同它的录制流交回。
fn run_on_two_memory_devices(
    history: DeletionHistory,
) -> (DeletionHistoryRun, Vec<RecordedOperation>) {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = [0, 1]
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .into();
    let run = run_deletion_history(history, &parameters, &mut devices, &stream)
        .unwrap_or_else(|failure| panic!("{}：{failure:?}", history.name()));
    (run, stream.operations())
}

fn publishes(run: &DeletionHistoryRun) -> Vec<(DeletionHistoryPublishKind, &TransactionOutput)> {
    run.segments
        .iter()
        .filter_map(|segment| match segment {
            DeletionHistorySegment::Publish { kind, version, .. } => {
                Some((*kind, version.as_ref()))
            }
            DeletionHistorySegment::WritesOutsideAnyPublish { .. } => None,
        })
        .collect()
}

fn is_a_deletion_publish(kind: DeletionHistoryPublishKind) -> bool {
    match kind {
        DeletionHistoryPublishKind::UnlinkIntent
        | DeletionHistoryPublishKind::UnlinkBatchInProcess
        | DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation => true,
        DeletionHistoryPublishKind::UnmountFloorRaise
        | DeletionHistoryPublishKind::MountInstanceRow
        | DeletionHistoryPublishKind::MountWarmUp => false,
    }
}

/// 重写的角色里分配记录树根之下那几片之外的个数：那几片按槽号按位置寻址，改哪几片随这段历史之前用过哪些槽变。
fn rewritten_units_outside_the_allocation_record_tree_below_its_root(
    version: &TransactionOutput,
) -> usize {
    version
        .rewritten
        .iter()
        .filter(|role| !matches!(role, TransactionUnit::AllocationTreeNodeBelowTheRoot(_)))
        .count()
}

/// 每次删除类发布：哪一种、重写的单元数（每盘）、其中分配记录树根之下那几片之外的个数、这一版之后剩几个数据单元。
fn deletion_publishes_with_units_per_device_and_data_units_after(
    run: &DeletionHistoryRun,
) -> Vec<(DeletionHistoryPublishKind, usize, usize, usize)> {
    publishes(run)
        .into_iter()
        .filter(|(kind, _)| is_a_deletion_publish(*kind))
        .map(|(kind, version)| {
            (
                kind,
                version.rewritten.len(),
                rewritten_units_outside_the_allocation_record_tree_below_its_root(version),
                version.data_pointers.len(),
            )
        })
        .collect()
}

/// 一次发布的写账里单元写的次数（journal 记录、根槽、系统配置槽之外的那几种），两块盘各一次。
fn unit_write_calls(version: &TransactionOutput) -> u64 {
    WrittenStructureKind::IN_REPORT_ORDER
        .into_iter()
        .filter(|kind| {
            !matches!(
                kind,
                WrittenStructureKind::JournalRecord
                    | WrittenStructureKind::RootSlot
                    | WrittenStructureKind::SystemConfigurationSlot
            )
        })
        .map(|kind| version.writes.of(kind).write_calls)
        .sum()
}

/// 每段对不对得上：各段首尾相接铺满窗口；每次发布那一段的写数等于它自己的写账、恰有一次根槽 FUA、单元写 = 重写角色数 × 2 盘。
fn segment_mismatches(run: &DeletionHistoryRun, operations: &[RecordedOperation]) -> Vec<String> {
    let mut mismatches = Vec::new();
    let mut expected_start = run.operations_before_the_unlink;
    for segment in &run.segments {
        let span = segment.operations();
        if span.start != expected_start {
            mismatches.push(format!("{span:?} 不接在 {expected_start} 上"));
        }
        expected_start = span.end;
        let DeletionHistorySegment::Publish { kind, version, .. } = segment else {
            continue;
        };
        let writes = operations[span.clone()]
            .iter()
            .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
            .count();
        let force_unit_access_writes = operations[span.clone()]
            .iter()
            .filter(|operation| operation.kind == RecordedOperationKind::WriteForceUnitAccess)
            .count();
        let observed = (
            u64::try_from(writes).expect("段长"),
            force_unit_access_writes,
            unit_write_calls(version),
        );
        let expected = (
            version.writes.total().write_calls,
            1,
            2 * u64::try_from(version.rewritten.len()).expect("角色数"),
        );
        if observed != expected {
            mismatches.push(format!(
                "{kind:?} txg {:?} {span:?}：（写数、FUA 数、单元写）= {observed:?}，应为 {expected:?}",
                version.root.checkpoint_txg
            ));
        }
    }
    if expected_start != operations.len() {
        mismatches.push(format!(
            "各段收在 {expected_start}，流尾在 {}",
            operations.len()
        ));
    }
    mismatches
}

fn write_at(device: u32) -> RecordedOperation {
    RecordedOperation {
        device: DeviceIdentity(device),
        kind: RecordedOperationKind::Write,
        offset: DeviceOffsetInBytes(0),
        length: 512,
        content_hash: 0,
    }
}

fn barrier_on(device: u32) -> RecordedOperation {
    RecordedOperation {
        device: DeviceIdentity(device),
        kind: RecordedOperationKind::Barrier,
        offset: DeviceOffsetInBytes(0),
        length: 0,
        content_hash: 0,
    }
}

#[test]
fn spans_of_consecutive_parts_give_each_part_the_barriers_after_its_last_write_and_the_first_part_the_leading_barriers(
) {
    let operations = [
        barrier_on(0),
        write_at(0),
        write_at(1),
        barrier_on(0),
        barrier_on(1),
        write_at(0),
        barrier_on(0),
        write_at(1),
    ];
    assert_eq!(
        spans_of_consecutive_parts_by_their_write_calls(&operations, &[2, 1, 1]),
        Ok(vec![0..5, 5..7, 7..8]),
        "开头那道屏障归第一段；每段收在下一段第一次写之前"
    );
}

#[test]
fn spans_of_consecutive_parts_refuse_a_part_without_writes_and_accounts_that_do_not_add_up() {
    let operations = [write_at(0), barrier_on(0), write_at(1)];
    for write_calls in [&[2, 0][..], &[1, 0, 1][..], &[1][..], &[1, 2][..]] {
        assert!(
            spans_of_consecutive_parts_by_their_write_calls(&operations, write_calls).is_err(),
            "写账 {write_calls:?} 切不开两次写，要拒"
        );
    }
}

#[test]
fn the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_twelve_units_per_device(
) {
    let (run, operations) =
        run_on_two_memory_devices(DeletionHistory::IntentAndEveryBatchInProcess);
    assert_eq!(
        deletion_publishes_with_units_per_device_and_data_units_after(&run),
        [
            (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 12, 8, 2),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 11, 7, 1),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 12, 8, 0),
        ],
        "意图 13、3 → 2 那一批 12、2 → 1 那一批 11、最后一批 12（每盘：此前 10，唯一那片叶容器身份消失、退役树多写容器与根两个单元），批大小 1 走三批"
    );
    let txgs: Vec<u64> = publishes(&run)
        .iter()
        .map(|(_, version)| version.root.checkpoint_txg.0)
        .collect();
    assert_eq!(
        txgs,
        (txgs[0]..txgs[0] + 4).collect::<Vec<_>>(),
        "四次发布按 txg 连着、没有别的发布插进来"
    );
    assert!(
        matches!(&run.final_version, PoolVersion::WithTreesAndNoFile(version) if version.deleted_inodes.is_empty()),
        "最后一批之后是删空的一版、队列空"
    );
    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
}

#[test]
fn the_mount_continuation_history_publishes_its_three_batches_after_the_unmount_the_row_and_the_warm_up(
) {
    let (run, operations) =
        run_on_two_memory_devices(DeletionHistory::IntentThenUnmountThenMountContinuation);
    assert_eq!(
        deletion_publishes_with_units_per_device_and_data_units_after(&run),
        [
            (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 12, 8, 2),
            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 13, 7, 1),
            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 14, 8, 0),
        ],
        "意图在进程里发，三批都由挂载续做发：分配记录树根之下那几片之外与进程里那三批相同，后两批多改一对分配记录树的叶"
    );
    let kinds_and_txgs: Vec<(DeletionHistoryPublishKind, u64)> = publishes(&run)
        .iter()
        .map(|(kind, version)| (*kind, version.root.checkpoint_txg.0))
        .collect();
    let intent_txg = kinds_and_txgs[0].1;
    assert_eq!(
        kinds_and_txgs,
        [
            (DeletionHistoryPublishKind::UnlinkIntent, intent_txg),
            (
                DeletionHistoryPublishKind::UnmountFloorRaise,
                intent_txg + 1
            ),
            (
                DeletionHistoryPublishKind::UnmountFloorRaise,
                intent_txg + 2
            ),
            (DeletionHistoryPublishKind::MountInstanceRow, intent_txg + 3),
            (DeletionHistoryPublishKind::MountWarmUp, intent_txg + 4),
            (DeletionHistoryPublishKind::MountWarmUp, intent_txg + 5),
            (
                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
                intent_txg + 6
            ),
            (
                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
                intent_txg + 7
            ),
            (
                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
                intent_txg + 8
            ),
        ],
        "意图 → 卸载两次抬 F → 写行 → 暖机两次 → 续做三批，按 txg 连着"
    );
    assert!(
        matches!(&run.final_version, PoolVersion::WithTreesAndNoFile(version) if version.deleted_inodes.is_empty()),
        "挂载交回删空的一版、队列空"
    );
    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
}

/// 一版 inode 树里号是 `inode` 的那条记录，连同它住的那片叶容器在左起第几片。
fn leaf_container_index_and_record(
    version: &TransactionOutput,
    inode: u64,
) -> Option<(usize, InodeRecord)> {
    version
        .inode_leaf_containers
        .iter()
        .enumerate()
        .find_map(|(container_index, container)| {
            container
                .contents
                .records
                .iter()
                .find(|record| record.inode == inode)
                .map(|record| (container_index, *record))
        })
}

#[test]
fn the_history_beside_a_size_zero_inode_creates_inode_two_before_the_sequential_write_and_gives_the_intent_sixty_seconds_after_the_write_which_every_batch_copies(
) {
    let (run, operations) = run_on_two_memory_devices(
        DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
    );
    let deletion_publishes: Vec<(DeletionHistoryPublishKind, &TransactionOutput)> = publishes(&run);
    assert_eq!(
        deletion_publishes
            .iter()
            .map(|(kind, version)| (*kind, version.data_pointers.len()))
            .collect::<Vec<_>>(),
        [
            (DeletionHistoryPublishKind::UnlinkIntent, 3),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 2),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 1),
            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 0),
        ],
        "同 H1：意图之后三批都在这个进程里发，批大小 1"
    );
    let (_, intent) = deletion_publishes[0];
    let intent_txg = intent.root.checkpoint_txg.0;
    assert_eq!(
        deletion_publishes
            .iter()
            .map(|(_, version)| version.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        (intent_txg..intent_txg + 4).collect::<Vec<_>>(),
        "四次发布按 txg 连着"
    );

    let (container_of_the_file, file_at_the_intent) =
        leaf_container_index_and_record(intent, 1).expect("意图那一版 1 号在树里");
    let (container_of_the_size_zero_inode, size_zero_inode) =
        leaf_container_index_and_record(intent, 2).expect("意图那一版 2 号在树里");
    assert_eq!(
        container_of_the_size_zero_inode, container_of_the_file,
        "2 号与 1 号在同一片 inode 叶容器里"
    );
    assert_eq!(
        (size_zero_inode.nlink, size_zero_inode.size),
        (1, 0),
        "2 号是活着的、size 0"
    );
    assert_eq!(
        size_zero_inode.object_birth.0 + 2,
        intent_txg,
        "2 号建在顺序写之前：建 2 号、顺序写、意图三次发布连着"
    );
    assert_eq!(
        size_zero_inode.write_time_seconds, 1_788_000_000,
        "2 号建出来时取新池新建文件的时间"
    );

    assert_eq!(
        run.intent_write_time_seconds, 1_788_000_120,
        "交给意图的调用方时间 = 1 788 000 000 + 60（顺序写）+ 60"
    );
    assert_eq!(
        (
            file_at_the_intent.nlink,
            file_at_the_intent.write_time_seconds
        ),
        (0, 1_788_000_120),
        "意图把 1 号写成 nlink 0，时间是调用方给的那一个、不是顺序写写进去的 1 788 000 060"
    );
    for (_, batch) in &deletion_publishes[1..3] {
        let (_, file_after_the_batch) =
            leaf_container_index_and_record(batch, 1).expect("最后一批之前 1 号还在树里");
        assert_eq!(
            file_after_the_batch.write_time_seconds, 1_788_000_120,
            "txg {} 那一批照抄意图那次写进去的时间",
            batch.root.checkpoint_txg.0
        );
    }
    let (_, final_batch) = deletion_publishes[3];
    assert_eq!(
        leaf_container_index_and_record(final_batch, 1),
        None,
        "最后一批删掉 1 号那条记录"
    );
    assert_eq!(
        leaf_container_index_and_record(final_batch, 2).map(|(_, record)| record),
        Some(size_zero_inode),
        "2 号那条记录从意图到最后一批一个字段都不变"
    );
    assert!(
        matches!(&run.final_version, PoolVersion::WithFile(version) if version.deleted_inodes.is_empty() && version.inode_record.is_none()),
        "最后一批之后队列空；还剩 2 号，仍是带文件的一版（D8 已定项 16 第 5 条）"
    );
    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
}
