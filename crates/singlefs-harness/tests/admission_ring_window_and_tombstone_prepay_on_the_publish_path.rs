//! 准入式子里的环窗口与记账标签 6 的墓碑预付接进发布路径与可写挂载（D28（挂载期承诺量） 已定项 1「环窗口」那一条；
//! D3（空间分配） 已定项 2：标签 6 = 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)，写入类发布按发布之后的标签判，
//! 环窗口从第一个文件版本那次发布起计，删除队列非空时可写挂载前那一判把环窗口算作可动用）。
//! 池是两块 4 GiB 盘、mkfs 的每区槽数 S = 8：产品批大小 144 时环窗口每盘 (3 × 8 + 1) × 368 = 9200 槽；
//! 第二个几何是随机历史单元区 384 槽那一档小盘（环窗口补足之后单元区 1199 槽）：每盘 25 × 30 = 750 槽。

mod common;
mod common_admission;

use common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use common_admission::{plain_device, PoolUnderTest};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::{
    admission_reading_before_a_publish_on_a_ring_of, admission_reading_of_a_writable_mount,
    committed_reservation_recorded_in, AdmissionReading, BytesOnOneDevice, DeviceAdmissionTerms,
    RingWindowOfTheReading,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::history::HistoryDeviceWidth;

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 进程内那几次发布都在新池新建文件那个实例里发。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

/// 产品批大小 144、S = 8、两块 4 GiB 盘：W = 24 + min(524, 340) + min(4, 340) = 368，环窗口 = 25 × 368 槽。
const RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH: u64 = 25 * 368;

fn slots_per_region() -> RootRingSlotsPerRegion {
    parameters().geometry.root_ring_slots_per_region
}

fn tag_six_of(version: &TransactionOutput) -> u64 {
    version
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_COMMITTED_RESERVATION_BYTES)
        .expect("池级三行之一")
        .value
}

fn ring_window_slots_on_each_device(reading: &AdmissionReading) -> Vec<u64> {
    reading
        .per_device()
        .iter()
        .map(|terms| terms.ring_window.0 / singlefs_format::SLOT_BYTES)
        .collect()
}

/// 同一份读数把环窗口那一项换成 0 之后，可用(d) 比原来多出的字节，逐盘。
fn available_bytes_the_ring_window_takes_on_each_device(reading: &AdmissionReading) -> Vec<i128> {
    let available_with_the_window = reading.available_on_each_device();
    let available_without_the_window = AdmissionReading::new(
        reading
            .per_device()
            .iter()
            .map(|terms| DeviceAdmissionTerms {
                device: terms.device,
                capacity: terms.capacity,
                allocated: terms.allocated,
                unreclaimable: terms.unreclaimable,
                mount_time_commitment: terms.mount_time_commitment,
                abandoned_root_exclusive: terms.abandoned_root_exclusive,
                ring_window: BytesOnOneDevice::ZERO,
            })
            .collect(),
        reading.pool_wide(),
        reading.replicas(),
    )
    .available_on_each_device();
    available_without_the_window
        .iter()
        .zip(&available_with_the_window)
        .map(|((_, without), (_, with))| without.0 - with.0)
        .collect()
}

fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
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

fn publish_new_inodes_in_process(pool: &mut BuiltPool, count: u64) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        FIXED_WRITE_TIME_SECONDS + 60,
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("建 inode");
    pool.output = output.clone();
    output
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

/// 新池新建文件那一版 inode 树只有第一个文件（n_live = 1）、墓碑树零根（H = 0）：
/// 标签 6 = 2 × ((2 + 0 + 1) × 16384 + 41 × 1) = 98386；再建两个 inode（n_live = 3）：2 × (49152 + 123) = 98550。
#[test]
fn tag_six_of_the_first_file_version_is_three_slots_and_one_live_object_and_two_new_inodes_add_two_more(
) {
    let mut pool = build_pool("tag-six-first-file-and-new-inodes");
    let first_file_version = pool.output.clone();
    let with_two_new_inodes = publish_new_inodes_in_process(&mut pool, 2);
    assert_eq!(
        (
            tag_six_of(&first_file_version),
            tag_six_of(&with_two_new_inodes)
        ),
        (98_386, 98_550),
        "标签 6 按每一版现算：两份副本 × (3 槽常驻 + 41 字节 × n_live)"
    );
}

/// 发布之前的读数：环窗口每块盘各扣 9200 槽，已承诺预留取那一版记账行里的标签 6。
#[test]
fn the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_device() {
    let pool = build_pool("ring-window-in-the-publish-reading");
    assert_eq!(slots_per_region().count(), 8, "mkfs 写的 S");
    let reading = admission_reading_before_a_publish_on_a_ring_of(
        &pool.allocator,
        Some(&pool.output),
        committed_reservation_recorded_in(Some(&pool.output)),
        slots_per_region(),
    );
    assert_eq!(
        (
            ring_window_slots_on_each_device(&reading),
            reading.pool_wide().committed_reservation.0,
        ),
        (vec![RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH; 2], 98_386),
        "每块盘扣 (3S + 1) × W 槽；全池那一项读到标签 6"
    );
    assert_eq!(
        available_bytes_the_ring_window_takes_on_each_device(&reading),
        vec![i128::from(RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH * 16384); 2],
        "可用(d) 正好少了环窗口那么多字节"
    );
}

/// 第二个几何：随机历史单元区 384 槽那一档小盘（环窗口补足 815 槽之后单元区 [1408, 2607)、分配记录树根层级 1），
/// 每块盘罩叶 ⌊2606 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1 = 3 片 ⇒ N_0 = 6、W = 24 + 6 = 30，环窗口 = 25 × 30 = 750 槽；可用(d) 正好少这么多字节。
#[test]
fn the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_slots() {
    let pool = PoolUnderTest::start_after_the_first_file(
        HistoryDeviceWidth::UnitAreaOf384Slots,
        plain_device,
    );
    let session = pool.session();
    let first_file_version = pool.current_file_version();
    let reading = admission_reading_before_a_publish_on_a_ring_of(
        &session.allocator,
        Some(first_file_version),
        committed_reservation_recorded_in(Some(first_file_version)),
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    assert_eq!(
        (
            ring_window_slots_on_each_device(&reading),
            available_bytes_the_ring_window_takes_on_each_device(&reading),
        ),
        (vec![25 * 30; 2], vec![i128::from(25 * 30 * 16384_u64); 2]),
        "每块盘扣 (3 × 8 + 1) × 30 槽，可用(d) 正好少这么多字节"
    );
}

/// 批大小 K 取分配器上装的删除开关：K = 1 时 W = 24 + min(524, 54) + min(4, 54) = 82，环窗口 = 25 × 82 = 2050 槽。
#[test]
fn the_ring_window_follows_the_batch_size_switch_installed_on_the_allocator() {
    let mut pool = build_pool("ring-window-follows-the-batch-switch");
    pool.allocator.set_deletion_switches(DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    });
    assert_eq!(
        RingWindowOfTheReading::DeductedOnEachDevice {
            slots_per_region: slots_per_region()
        }
        .on_each_device(&pool.allocator),
        BytesOnOneDevice::of_slots(25 * 82)
    );
}

/// 可写挂载那一判：要接的那一版删除队列非空（意图之后、各批之间）把环窗口算作可动用、不扣；队列走空之后照扣；
/// 树表 0 条的一版（还没有第一个文件版本）不起计。
#[test]
fn the_writable_mount_reading_counts_the_ring_window_as_usable_only_while_the_deletion_queue_is_not_empty(
) {
    let mut pool = pool_with_a_three_unit_file("ring-window-usable-while-the-queue-is-not-empty");
    let intent = publish_the_intent_in_process(&mut pool);
    let first_batch = publish_the_next_step_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let emptied = publish_the_next_step_in_process(&mut pool);
    let mount_reading_of = |version: Option<&TransactionOutput>| {
        ring_window_slots_on_each_device(&admission_reading_of_a_writable_mount(
            &pool.allocator,
            version,
            slots_per_region(),
        ))
    };
    assert_eq!(
        (
            mount_reading_of(Some(&intent)),
            mount_reading_of(Some(&first_batch)),
            mount_reading_of(Some(&emptied)),
            mount_reading_of(None),
        ),
        (
            vec![0, 0],
            vec![0, 0],
            vec![RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH; 2],
            vec![0, 0],
        ),
        "队列非空：可动用；队列空：扣 9200 槽；没有第一个文件版本：不起计"
    );
}
