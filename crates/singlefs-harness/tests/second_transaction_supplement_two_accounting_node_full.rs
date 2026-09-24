//! 里程碑「第二个事务」增补 2 第 28 行的报错那一半（记账树）：记账树第一版只有一个节点（16384 − 头 159 = 16225 字节，每行 34 字节，装 477 行），
//! 一次发布写的记账行 = 池级 3 行 + 每块盘 6 行，只随盘数变：79 块盘正好 477 行、照常发布；80 块盘要 483 行 ⇒ 在动分配器之前报
//! `AccountingEntriesExceedOneNode`，一个写、一道屏障都不发，不走到装节点的断言（代码三方第二轮攻方腿 Y3）。分裂不做。
//!
//! mkfs 第一版只收 2 块盘，这里绕过 mkfs、直接在 79 / 80 块稀疏内存盘上调发布路径：今天的可写挂载与 mkfs 都走不到 80 块盘。

mod common;

use common::{parameters, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::pointer::NodePointer;
use singlefs_core::transaction::{
    publish_version, FileVersionPlan, InstanceTablePlan, PoolWriter, PublishError, PublishPlan,
    TransactionOutput, FIRST_TRANSACTION_NUMBER,
};
use singlefs_core::unit::index_node_entry_capacity;
use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, FIRST_TRANSACTION_TXG,
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
};
use singlefs_harness::crash::{SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const FILE_CONTENT: [u8; 3000] = [0x42; 3000];

struct ManyDevicePool {
    stream: SharedStream,
    devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    allocator: PoolAllocator,
}

/// `device_count` 块全新的稀疏内存盘（录制器共用一条流），分配器每块盘一张空闲图、一条记录都没有。
fn many_device_pool(device_count: u32) -> ManyDevicePool {
    let stream = SharedStream::new();
    let devices = (0..device_count)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let allocator = PoolAllocator::new(
        (0..device_count)
            .map(|device_number| DeviceFreeMap::new(DeviceIdentity(device_number), IMAGE_BYTES))
            .collect(),
    );
    ManyDevicePool {
        stream,
        devices,
        allocator,
    }
}

/// 第一个文件版本形态的一次发布（txg 3、jsn 3、事务号 1、八个角色全写）。
fn publish_first_file_version(
    pool: &mut ManyDevicePool,
) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    publish_version(
        &mut writer,
        &mut pool.allocator,
        PublishPlan {
            txg,
            counter: FIRST_TRANSACTION_TXG,
            transaction: FIRST_TRANSACTION_NUMBER,
            highest_transaction_number_before_this_publish: 0,
            instance: InstanceGeneration(1),
            back_chain: 0,
            file: Some(FileVersionPlan {
                content: &FILE_CONTENT,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
                inode_object_birth: txg,
                // 改动计数 = 这次发布的 checkpoint_txg（增补 2 第 11 行）：第一个文件版本形态就是 txg 3。
                change_count: txg.0,
            }),
            // 这条路径只建第一个文件那一个 inode。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(NodePointer::empty_root()),
            tree_birth_txg: txg,
            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            rollback_floor: CheckpointTxg(0),
        },
        None,
    )
}

fn accounting_node_capacity() -> usize {
    index_node_entry_capacity(
        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    )
}

#[test]
fn seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish() {
    assert_eq!(accounting_node_capacity(), 477);
    let mut pool = many_device_pool(79);
    let output =
        publish_first_file_version(&mut pool).expect("79 块盘：3 + 6 × 79 = 477 行，正好装满");
    assert_eq!(output.accounting_entries.len(), 477);
}

#[test]
fn eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written(
) {
    let mut pool = many_device_pool(80);
    let images_before: Vec<SparseDevice> = pool
        .devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    let result = publish_first_file_version(&mut pool);
    assert!(
        matches!(
            result,
            Err(PublishError::AccountingEntriesExceedOneNode {
                entries: 483,
                capacity: 477
            })
        ),
        "80 块盘要 3 + 6 × 80 = 483 行：{result:?}"
    );
    assert_eq!(
        pool.stream.operations().len(),
        0,
        "录制流一步都没有：一个写、一道屏障都没发"
    );
    let images_after: Vec<SparseDevice> = pool
        .devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    assert!(images_after == images_before, "80 块盘上逐字节不变");
    assert!(
        pool.allocator.records().is_empty(),
        "报错在动分配器之前：一条分配记录都没有"
    );
    assert!(
        pool.allocator
            .devices
            .iter()
            .all(|device_map| device_map.allocated_slots() == 0),
        "每块盘一个槽都没分出去"
    );
}
