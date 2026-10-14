//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 38 行（实例表第二片）的写路径：真写者写多于一片的实例表链。
//! 两条写法（书记员规格第四节）：
//! - D3（空间分配） 已定项 10 ⑤：一次发布重写实例表单元时实例表单元最前，**多于一片时尾片先**（第 k 片当第 k + 1 片的父，照「先叶后根」）——
//!   这个次序同时是各片的取落点次序与出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）；
//! - D18（块里携带什么信息） 已定项 11：**行一片写满 369 行再开下一片**，最后一片装剩下的。
//!
//! 链指针记录「有无下一片」1 = 有、0 = 无（实四的读法，主 agent 认）。每次写行 COW 重写整条链，被换下的旧链逐片释放。
//!
//! 「取号之后崩溃」（取号写完、写行那次发布之前掉电）是今天就走得到的历史：下一次挂载要给中间那些实例各补一行 (i, 0, 0)，
//! 一次挂载就能写很多行。本文件用它把表快速推过一片（`acquire_instance` 连取 k 次号 = 连着 k 次取号之后崩溃）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/instance_table_second_page_write.rs"]
mod scenarios_of_instance_table_second_page_write;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_instance_table_second_page_write::{
    instances_of, page_on_disk, pool_whose_row_publish_opened_the_second_page, DISKS, SECOND_PAGE,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::instance_table::{InstanceTableChainRecord, InstanceTablePageIndex};
use singlefs_core::pointer::BirthSequence;
use singlefs_core::transaction::TransactionUnit;

const SECOND_PAGE_ROLE: TransactionUnit =
    TransactionUnit::InstanceTablePageAfterTheFirst(SECOND_PAGE);

/// 分配记录树根之下 (层级, 盘, 同盘同层序号) 那个节点的角色。
fn allocation_record_tree_node(level: u8, device: u32, index_in_device: u64) -> TransactionUnit {
    TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device: DeviceIdentity(device),
        index_in_device,
    })
}

/// 带文件的一版上写 370 行：两片，尾片先。
/// - 角色与落点（bump 次序）：这次重写的角色是 [第 1 片, 第 0 片, 分配记录树（根之下四个节点与根）, 记账树, 中央映射树, 树表]，
///   落点按这个次序在新开的段里递增；
/// - 出生序号（树 0 在 (txg, 实例) 上连着发）：第 1 片 0、第 0 片 1、树表 2；
/// - 行：第 0 片写满 369 行（实例 1..=369），第 1 片装剩下的 1 行（实例 370）；第 0 片的链指针记录是「有下一片」、指着第 1 片
///   （落点、整单元校验和、出生序号都对得上），第 1 片的是「无下一片」；
/// - 点名项按同一个次序：第 1 片、第 0 片在最前；两片的分配记录都是这次发布的 txg、已分配。
#[test]
fn row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it() {
    let (devices, mounted) = pool_whose_row_publish_opened_the_second_page();
    let row_publish = mounted
        .output
        .row_publish
        .file_version()
        .expect("新池新建文件之后的写行发布带文件");
    let txg = row_publish.root.checkpoint_txg;
    assert_eq!(
        row_publish.rewritten,
        vec![
            SECOND_PAGE_ROLE,
            TransactionUnit::InstanceTable,
            allocation_record_tree_node(0, 0, 61),
            allocation_record_tree_node(0, 1, 61),
            allocation_record_tree_node(1, 0, 0),
            allocation_record_tree_node(1, 1, 0),
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "尾片先：第 1 片、第 0 片，之后才是固定点单元（分配记录树按位置寻址，D8（核心索引结构） 已定项 14：\
         4 GiB 两块盘上每盘叶 61、第 1 层 0 与根，先叶后根）"
    );
    let slots: Vec<SlotNumber> = row_publish
        .rewritten
        .iter()
        .map(|role| row_publish.unit(*role).slot)
        .collect();
    assert!(
        slots.windows(2).all(|pair| pair[0] < pair[1]),
        "落点按 bump 次序在开放段里递增：{slots:?}"
    );

    let first_page_pointer = row_publish.root.instance_table;
    let first_page = page_on_disk(&devices, &first_page_pointer, InstanceTablePageIndex::FIRST);
    let InstanceTableChainRecord::NextPage(second_page_pointer) = first_page.chain else {
        panic!(
            "第 0 片的链指针记录要是「有下一片」：{:?}",
            first_page.chain
        )
    };
    let second_page = page_on_disk(&devices, &second_page_pointer, SECOND_PAGE);
    assert_eq!(second_page.chain, InstanceTableChainRecord::LastPage);
    assert_eq!(
        second_page_pointer.locations[0].slot,
        row_publish.unit(SECOND_PAGE_ROLE).slot,
        "第 0 片的链指针指着这次写的第 1 片"
    );
    assert_eq!(
        (
            second_page_pointer.birth_sequence,
            first_page_pointer.birth_sequence,
            row_publish.root.tree_table.birth_sequence
        ),
        (BirthSequence(0), BirthSequence(1), BirthSequence(2)),
        "树 0 的出生序号按 bump 次序发：第 1 片、第 0 片、树表"
    );
    assert_eq!(
        (
            second_page_pointer.head.birth_txg,
            first_page_pointer.head.birth_txg
        ),
        (txg, txg)
    );
    assert_eq!(
        instances_of(&first_page.rows),
        (1..=369).collect::<Vec<u32>>(),
        "第 0 片写满 369 行"
    );
    assert_eq!(
        instances_of(&second_page.rows),
        vec![370],
        "第 1 片装剩下的那一行"
    );
    assert_eq!(
        first_page.rows[0].selected_root_txg,
        CheckpointTxg(3),
        "A 那一行记它的根 txg 3"
    );

    let named_slots: Vec<SlotNumber> = row_publish
        .record
        .named
        .iter()
        .take(2)
        .map(|named| named.locations[0].slot)
        .collect();
    assert_eq!(
        named_slots,
        vec![
            second_page_pointer.locations[0].slot,
            first_page_pointer.locations[0].slot
        ],
        "点名项按 bump 次序：第 1 片、第 0 片"
    );
    for pointer in [first_page_pointer, second_page_pointer] {
        for device in DISKS {
            let record = mounted
                .allocator
                .record_for(device, pointer.locations[0].slot)
                .expect("两片都有分配记录");
            assert_eq!(
                (record.is_released, record.generation, record.span_slots),
                (false, txg, 2),
                "盘 {device:?} 上那一片的分配记录：已分配、分配代是这次发布的 txg、跨两槽"
            );
        }
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_second_page_write::cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green() {
    scenarios_of_instance_table_second_page_write::cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_second_page_write::the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages() {
    scenarios_of_instance_table_second_page_write::the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_second_page_write::version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both() {
    scenarios_of_instance_table_second_page_write::version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both(&mut PoolCheckerImages::left_to_the_checker_tier());
}
