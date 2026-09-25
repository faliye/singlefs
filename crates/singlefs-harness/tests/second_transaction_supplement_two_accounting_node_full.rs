//! 里程碑「第二个事务」增补 2 第 28 行的记账树那一半：记账树一个节点（16384 − 头 159 = 16225 字节，每行 34 字节）装 477 行，
//! 一次发布写的记账行 = 池级 3 行 + 每块盘 6 行，只随盘数变：79 块盘正好 477 行、一个节点；80 块盘要 483 行 ⇒ 改之前在动分配器之前报
//! `AccountingEntriesExceedOneNode`（代码三方第二轮攻方腿 Y3），现在照 D8（核心索引结构） 已定项 11 分裂：第 478 行插进去时
//! 根兼叶从中间切（239 + 239），剩下 5 行落在右叶，长出一个层级 1 的根——这是产品容量下（不压小）唯一走得到的记账树分裂。
//!
//! mkfs 第一版只收 2 块盘，这里绕过 mkfs、直接在 79 / 80 块稀疏内存盘上调发布路径：今天的可写挂载与 mkfs 都走不到 80 块盘。

mod common;

use common::{parameters, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::{parse_internal_entry, CodeTwoTreeNodeContents};
use singlefs_core::pointer::NodePointer;
use singlefs_core::transaction::{
    publish_version, FileVersionPlan, InstanceTablePlan, MultiLevelCodeTwoTree, PoolWriter,
    PublishError, PublishPlan, TransactionOutput, TransactionUnit, FIRST_TRANSACTION_NUMBER,
};
use singlefs_core::unit::{index_node_entry_capacity, parse_index_node};
use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, FIRST_TRANSACTION_TXG,
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const FILE_CONTENT: [u8; 3000] = [0x42; 3000];

struct ManyDevicePool {
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
    ManyDevicePool { devices, allocator }
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
fn eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing() {
    let mut pool = many_device_pool(80);
    let output = publish_first_file_version(&mut pool)
        .expect("80 块盘要 3 + 6 × 80 = 483 行：记账树分裂，不拒");
    assert_eq!(output.accounting_entries.len(), 483);
    assert_eq!(
        output.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::Accounting),
        2,
        "树高从根节点头现读：根的层级 1 + 1（D8 已定项 11 ⑤、D28 已定项 4）"
    );
    let leaf_row_counts: Vec<usize> = output
        .accounting_tree
        .shape
        .nodes()
        .iter()
        .filter_map(|node| match &node.contents {
            CodeTwoTreeNodeContents::Leaf { keys } => Some(keys.len()),
            CodeTwoTreeNodeContents::Internal { .. } => None,
        })
        .collect();
    assert_eq!(
        leaf_row_counts,
        vec![239, 244],
        "第 478 行插进去时 478 行从中间切成 239 + 239，剩下 5 行落在右叶"
    );
    assert!(
        output.rewritten.contains(&TransactionUnit::AccountingTree)
            && output
                .rewritten
                .iter()
                .filter(|identity| matches!(
                    identity,
                    TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
                ))
                .count()
                == 2,
        "两片叶与根都在这次写出的角色里：{:?}",
        output.rewritten
    );
    // 记账树每个节点各一条映射条目（D19 已定项 8：码 2 树节点进映射）。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：
    // 80 块 4 GiB 的盘上根在第 2 层（每块盘两个第 1 层节点那么宽，80 × 2 = 160 ≤ 169），每个节点每块盘各一份，
    // 这次写的两百多个节点把每块盘上用到的槽推过 50344 ⇒ 每块盘的记录落在叶 61 与叶 62 里，每块盘三个节点（两片叶、第 1 层节点 0），
    // 加根：1 + 80 × 3 = 241 个节点，各一条映射条目。
    assert_eq!(
        output.mapping_keys.len(),
        4 + (1 + 80 * 3) + 3,
        "数据单元、extent 根、inode 叶、inode 根各一条，分配记录树 241 个节点各一条，记账树三个节点各一条"
    );
    // 三个节点都落了盘（80 块盘每块一份）：盘上那一份解得开、层级是它在树里的层级，根的两条内部条目指着两片叶。
    let root = parse_index_node(&output.unit(TransactionUnit::AccountingTree).bytes).expect("根");
    assert_eq!(
        (root.level, root.entries.len(), root.entry_width),
        (1, 2, 108)
    );
    for (node, pointer) in output
        .accounting_tree
        .shape
        .nodes()
        .iter()
        .zip(&output.accounting_tree.pointers)
    {
        for (_, device) in &pool.devices {
            let on_disk = device
                .inner()
                .image
                .read(pointer.locations[0].slot.to_device_offset(), 16384);
            let header = parse_index_node(&on_disk).expect("盘上的节点解得开");
            assert_eq!(header.level, node.position.level);
        }
    }
    let leaf_slots: Vec<u64> = root
        .entries
        .iter()
        .map(|entry| {
            parse_internal_entry(entry, 22)
                .expect("内部条目 108")
                .1
                .locations[0]
                .slot
                .0
        })
        .collect();
    let expected_leaf_slots: Vec<u64> = output.accounting_tree.pointers[..2]
        .iter()
        .map(|pointer| pointer.locations[0].slot.0)
        .collect();
    assert_eq!(
        leaf_slots, expected_leaf_slots,
        "根的两条内部条目按 key 序指着两片叶"
    );
}
