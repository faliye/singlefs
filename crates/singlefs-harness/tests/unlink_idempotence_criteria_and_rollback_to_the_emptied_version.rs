//! unlink 的幂等判据（D8（核心索引结构） 已定项 16 第 7 条）在发布层的三格：① 计划相等（从两批之间的镜像重挂载续做，
//! 续做的每一次发布与一路做下来的那几次逻辑内容逐次相同）、③ 一个写点截断之后重开续做到底（终态逻辑内容与不崩溃的终态相同、
//! 崩溃态与终态池级 checker 全绿；全量逐写点枚举归 checker 档）、④ 每一版的记账标签 4、6 等于从这一版镜像现算的值；
//! 另加回退到删空的一版（D23（journal 的角色与格式） 已定项 14 候选集「带文件（那一版写过文件，不是还没写过文件的那一版）」：
//! 删空的那一版写过文件，照样是候选）。三个单元的第一个文件、只供测试的批大小 1：意图 → 留 2 → 留 1 → 最后一批。

mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::records::{
    TombstoneRecord, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_INODE_WATERMARK,
    STATISTIC_PENDING_DELETE_BYTES,
};
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 进程内那几次发布都在新池新建文件那个实例里发。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
}

fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..2 * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(tag);
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
    pool
}

fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        previous
            .inode_record
            .expect("意图之前第一个文件那条记录在")
            .write_time_seconds,
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("意图");
    pool.output = output.clone();
    output
}

fn publish_the_next_step_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        batches_of_one_data_unit(),
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("删除的下一步");
    pool.output = output.clone();
    output
}

/// 一份崩溃镜像按批大小 1 的删除开关可写挂载一次（挂载里续做到空），交回挂载与它写到的那两块内存盘。
fn mount_a_copy_of_the_image_with_batches_of_one(
    image: &MemoryPool,
    stream: &SharedStream,
) -> (
    Mounted,
    Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    )>,
) {
    let mut devices = crash_state_devices(image, &[], &[], stream);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("这一份镜像可写挂载、续做");
    (mounted, devices)
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn accounting_value(version: &TransactionOutput, statistic: u16) -> u64 {
    version
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic)
        .expect("池级三行每一版都写")
        .value
}

/// 第 7 条的「逻辑内容」：inode 每条（号、对象出生代、size、nlink）、数据单元个数、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、
/// 标签 12 / 4 / 6。不含实例号、根与单元头里的 txg、改动计数、落点（第一版只删第一个文件：数据单元的内容随个数一起由那个文件定）。
#[derive(Debug, PartialEq, Eq)]
struct LogicalContent {
    inode_records: Vec<(u64, CheckpointTxg, u64, u32)>,
    data_units: usize,
    deleted_inodes: Vec<InodeNumber>,
    tombstones: Vec<TombstoneRecord>,
    inode_number_watermark: u64,
    pending_delete: u64,
    committed_reservation: u64,
}

fn logical_content_of(version: &TransactionOutput) -> LogicalContent {
    let mut inode_records: Vec<(u64, CheckpointTxg, u64, u32)> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (record.inode, record.object_birth, record.size, record.nlink))
        .collect();
    inode_records.sort_unstable();
    let mut tombstones: Vec<TombstoneRecord> = version
        .tombstone_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect();
    tombstones.sort_unstable_by_key(|record| record.to_bytes());
    LogicalContent {
        inode_records,
        data_units: version.data_pointers.len(),
        deleted_inodes: version.deleted_inodes.clone(),
        tombstones,
        inode_number_watermark: accounting_value(version, STATISTIC_INODE_WATERMARK),
        pending_delete: accounting_value(version, STATISTIC_PENDING_DELETE_BYTES),
        committed_reservation: accounting_value(version, STATISTIC_COMMITTED_RESERVATION_BYTES),
    }
}

fn rebuilt_version_of_trees(
    image: &MemoryPool,
    root: &singlefs_core::root_record::RootRecord,
    record_standing: &singlefs_core::journal::JournalRecord,
) -> TransactionOutput {
    match rebuild_version(image, root, Some(record_standing.clone()))
        .unwrap_or_else(|failure| panic!("txg {:?} 从盘上重建：{failure:?}", root.checkpoint_txg))
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("txg {:?} 这一版有树", root.checkpoint_txg),
    }
}

/// ①：两批之间（意图、留 2 之后）的镜像重挂载，挂载续做的两次发布（留 1、最后一批）与一路做下来的那两次逻辑内容逐次相同——
/// 续做的计划只由现行那一版定，从盘上重建的那一版与进程里那一版算出同一个计划。
#[test]
fn the_publishes_continued_by_a_remount_between_two_batches_equal_the_rest_of_the_uninterrupted_run(
) {
    let mut pool = pool_with_a_three_unit_file("idempotence-plan-equal-on-remount");
    publish_the_intent_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let image_between_two_batches = pool.memory_pool();
    let uninterrupted_rest = [
        publish_the_next_step_in_process(&mut pool),
        publish_the_next_step_in_process(&mut pool),
    ];
    let stream = SharedStream::new();
    let (mounted, devices) =
        mount_a_copy_of_the_image_with_batches_of_one(&image_between_two_batches, &stream);
    let last_row_or_warm_up_txg = mounted
        .output
        .warm_up_publishes
        .last()
        .unwrap_or(&mounted.output.row_publish)
        .root()
        .checkpoint_txg;
    let record_standing = mounted
        .current
        .version_of_trees()
        .expect("续做之后有树")
        .record
        .clone();
    let continued_image = memory_pool_of_sparse_devices(&devices);
    let mut continuation_roots: Vec<singlefs_core::root_record::RootRecord> =
        disk_snapshot(&continued_image, &stream)
            .readable_roots
            .into_iter()
            .filter(|root| root.checkpoint_txg > last_row_or_warm_up_txg)
            .collect();
    continuation_roots.sort_by_key(|root| root.checkpoint_txg);
    let continued: Vec<LogicalContent> = continuation_roots
        .iter()
        .map(|root| {
            logical_content_of(&rebuilt_version_of_trees(
                &continued_image,
                root,
                &record_standing,
            ))
        })
        .collect();
    assert_eq!(
        continued,
        uninterrupted_rest
            .iter()
            .map(logical_content_of)
            .collect::<Vec<LogicalContent>>(),
        "挂载续做两次（留 1、最后一批），逐次与一路做下来的那两次逻辑内容相同"
    );
}

/// ③（一个写点）：第一批的写发出一半时崩，重开可写挂载续做到底；终态逻辑内容与不崩溃的终态相同，崩溃态与终态池级 checker 全绿，
/// 两份终态的读者都答「没有文件」。逐写点的全量枚举是 checker 档的活。
#[test]
fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(
) {
    let mut pool = pool_with_a_three_unit_file("idempotence-crash-in-the-first-batch");
    publish_the_intent_in_process(&mut pool);
    let operations_before_the_first_batch = pool.retained_operations().len();
    publish_the_next_step_in_process(&mut pool);
    let operations_after_the_first_batch = pool.retained_operations().len();
    publish_the_next_step_in_process(&mut pool);
    let uninterrupted_end = publish_the_next_step_in_process(&mut pool);
    let operations = pool.retained_operations();
    let crash_point = operations_before_the_first_batch
        + (operations_after_the_first_batch - operations_before_the_first_batch) / 2;
    let mut crash_image =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    crash_image.apply(&operations[..crash_point]);
    assert_eq!(
        violations(&crash_image),
        Vec::<String>::new(),
        "崩溃态池级 checker 全绿"
    );
    let stream = SharedStream::new();
    let (mounted, devices) = mount_a_copy_of_the_image_with_batches_of_one(&crash_image, &stream);
    let continued_end = mounted
        .current
        .version_of_trees()
        .expect("续做到底之后有树")
        .clone();
    assert_eq!(
        logical_content_of(&continued_end),
        logical_content_of(&uninterrupted_end),
        "终态逻辑内容与不崩溃的终态相同"
    );
    let continued_image = memory_pool_of_sparse_devices(&devices);
    assert_eq!(
        violations(&continued_image),
        Vec::<String>::new(),
        "续做之后池级 checker 全绿"
    );
    let read_only = mount_read_only(&continued_image).expect("只读挂载");
    assert!(
        matches!(
            read_only.mounted.open_file(&continued_image, FIRST_FILE),
            Err(OpenFileFailure::NoSuchInode { inode }) if inode == FIRST_FILE
        ),
        "续做之后读者答「没有文件」"
    );
}

/// ④：删除前、意图、两批、最后一批每一版记账行里的标签 4 与 6，等于从这一版镜像现算的值：
/// 标签 4 恒 0；标签 6 = 2 × ((2 + H + 1) × 16384 + 41 × n_live)，n_live 数重建出来的 inode 树里 nlink ≥ 1 的记录，
/// H 从墓碑树根单元的码 2 头现读（零根时 0）。删除前 n_live = 1（98386），意图起 n_live = 0（98304）。
#[test]
fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image() {
    let mut pool = pool_with_a_three_unit_file("idempotence-tags-four-and-six");
    let mut published = vec![
        pool.output.clone(),
        publish_the_intent_in_process(&mut pool),
    ];
    for _ in 0..3 {
        published.push(publish_the_next_step_in_process(&mut pool));
    }
    let image = pool.memory_pool();
    let recorded_and_recomputed: Vec<((u64, u64), (u64, u64))> = published
        .iter()
        .map(|version| {
            let rebuilt = rebuilt_version_of_trees(&image, &version.root, &version.record);
            let live_objects = u64::try_from(
                rebuilt
                    .inode_leaf_containers
                    .iter()
                    .flat_map(|container| container.contents.records.iter())
                    .filter(|record| record.nlink >= 1)
                    .count(),
            )
            .expect("记录数");
            let tombstone_tree_internal_levels = rebuilt
                .units
                .iter()
                .find(|unit| unit.identity == TransactionUnit::TombstoneRoot)
                .map_or(0, |root| {
                    u64::from(parse_index_node(&root.bytes).expect("墓碑树根解得开").level) - 1
                });
            (
                (
                    accounting_value(version, STATISTIC_PENDING_DELETE_BYTES),
                    accounting_value(version, STATISTIC_COMMITTED_RESERVATION_BYTES),
                ),
                (
                    0,
                    2 * ((2 + tombstone_tree_internal_levels + 1) * 16384 + 41 * live_objects),
                ),
            )
        })
        .collect();
    assert_eq!(
        recorded_and_recomputed
            .iter()
            .map(|(recorded, _)| *recorded)
            .collect::<Vec<(u64, u64)>>(),
        recorded_and_recomputed
            .iter()
            .map(|(_, recomputed)| *recomputed)
            .collect::<Vec<(u64, u64)>>(),
        "每一版记账行里的标签 4、6 等于从镜像现算的"
    );
    assert_eq!(
        recorded_and_recomputed
            .iter()
            .map(|((_, recorded_tag_six), _)| *recorded_tag_six)
            .collect::<Vec<u64>>(),
        vec![98_386, 98_304, 98_304, 98_304, 98_304],
        "删除前一个活对象；从意图那次起第一个文件 nlink = 0、不再算活对象"
    );
}

/// 回退到删空的一版（Q-K）：删完唯一的文件之后再建一个 size 0 的 inode，回退目标取删空那一版——它写过文件，在候选集里；
/// 回退之后的一版是有树而文件集合为空的那一种，池级 checker 全绿。
#[test]
fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file()
{
    let mut pool = pool_with_a_three_unit_file("rollback-to-the-emptied-version");
    publish_the_intent_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let emptied = publish_the_next_step_in_process(&mut pool);
    assert!(
        emptied.is_a_version_with_trees_and_no_file(),
        "最后一批之后是删空的一版"
    );
    {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            1,
            FIXED_WRITE_TIME_SECONDS + 60,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("删空的一版上建一个 inode");
    }
    assert!(
        !pool.output.is_a_version_with_trees_and_no_file(),
        "建了 inode 之后那一版带文件"
    );
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: emptied.root.instance,
            checkpoint_txg: emptied.root.checkpoint_txg,
        },
    )
    .expect("删空的一版写过文件，是回退候选");
    assert!(
        pool.output.is_a_version_with_trees_and_no_file(),
        "回退之后的一版有树而文件集合为空"
    );
    assert_eq!(
        violations(&pool.memory_pool()),
        Vec::<String>::new(),
        "回退之后池级 checker 全绿"
    );
}

/// 回退那次发布的标签 6 按新根现算（D23（journal 的角色与格式） 已定项 14「准入与记账」那一行）：新根的 inode 树与 `deleted_inodes`
/// 取 R_old 的，n_live = R_old 那一版 inode 树的记录数减 `deleted_inodes` 的 key 数。删完唯一的文件（标签 6 = 98304，n_live = 0）之后
/// 回退到意图之前那一版（第一个文件活着）：新根的标签 6 = 2 × (49152 + 41 × 1) = 98386，被带回来的对象的墓碑份额照抬。
#[test]
fn rolling_back_to_the_version_before_the_intent_recomputes_tag_six_from_the_rollback_targets_inode_tree(
) {
    let mut pool = pool_with_a_three_unit_file("rollback-recomputes-tag-six");
    let before_the_intent = pool.output.clone();
    publish_the_intent_in_process(&mut pool);
    for _ in 0..3 {
        publish_the_next_step_in_process(&mut pool);
    }
    let tag_six_before_the_rollback =
        accounting_value(&pool.output, STATISTIC_COMMITTED_RESERVATION_BYTES);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: before_the_intent.root.instance,
            checkpoint_txg: before_the_intent.root.checkpoint_txg,
        },
    )
    .expect("回退到意图之前那一版");
    assert_eq!(
        (
            tag_six_before_the_rollback,
            accounting_value(&pool.output, STATISTIC_COMMITTED_RESERVATION_BYTES),
        ),
        (98_304, 98_386),
        "删完时没有活对象；回退新根按 R_old 的 inode 树现算，第一个文件又是一个活对象"
    );
}
