//! 分配记录树重写集的固定点每次并进重写集都顺着释放链走到闭包（`transaction::settle_the_allocation_record_tree`）：释放链不占轮数，最终的重写集仍恰是内容变了的节点。
//! 历史是 E206（发布固定点轮数的代价）的取样点 `files_1000_units_10`（单元区 2¹⁸ 槽，大文件 10 个数据单元，1000 个小文件各 10 个数据单元，10 次建了就删），
//! 顺序建的文件把每片叶上一次重写的落点连成一条沿活跃叶的长链。这段历史之后的头一次覆盖写，不走闭包的那一臂量到 29 轮、重写 99 个角色、87 个节点，走闭包的那一臂量到 2 轮、角色与节点数相同
//! （`research/results/e206-fixed-point-cost-report-2026-11-09-r2.out` 的 `name=q1`、`name=peaks`、`name=q10_many_rounds` 行）。
//! 盘在进程里造，不读活树；量窗那几次经 `PoolWriter` 直接发（轮数读数住写入口上，会话每次改动另开写入口），发法与会话里的覆盖写同一个函数。

mod common_admission_pool;

use std::collections::BTreeSet;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocation_record_tree::{
    nodes_whose_contents_changed, AllocationRecordTreeGeometry, AllocationRecordTreeNode,
};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{
    publish_overwrite_of_an_inode, PartialWriteBytes, PoolVersion, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;

type Pool = PoolUnderTest<SparseBlockDevice>;

/// 6 MiB 环：单元区从槽 1408 起（E206 全部取样点同一份）。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const UNIT_AREA_SLOTS: u64 = 1 << 18;
const DATA_UNITS_OF_EACH_FILE: u64 = 10;
const SMALL_FILES: u64 = 1_000;
const INODES_PER_BATCH: u64 = 100;
const CREATE_AND_UNLINK_ROUNDS: u64 = 10;
const FIRST_FILE_OVERWRITES: u64 = 32;
const COLD_FILE_OVERWRITES: u64 = 32;
/// 冷文件按 inode 号均分成这么多份，每份取一个。
const SHARES_OF_THE_COLD_FILES: usize = 32;
/// 不走闭包的那一臂在这段历史之后的头一次覆盖写上量到的重写角色数与重写节点数：走闭包不改写什么，这两个数不变。
const ROLES_REWRITTEN_BY_THE_OVERWRITE_RIGHT_AFTER_THE_HISTORY: usize = 99;
const NODES_REWRITTEN_BY_THE_OVERWRITE_RIGHT_AFTER_THE_HISTORY: usize = 87;
/// 每次固定点至少两轮：头一轮重写集是空的，释放弄脏的叶罩不住。
const FEWEST_ROUNDS_OF_A_SETTLING: usize = 2;
/// 走闭包的那一臂在这个取样点上量到的峰值（`name=q1 item=files_1000_units_10__p2a peak_rounds=3`；E206 全部取样点的峰值是 5）。
/// 取落点那一处并进重写集的节点不走闭包时，量窗里覆盖写第 25 个冷文件那一次（它取落点碰到的叶走得到别的节点）多走一轮，超过这个数。
const MOST_ROUNDS_OF_A_SETTLING_ON_THIS_HISTORY: usize = 3;

fn parameters() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES;
    parameters
}

fn device_bytes() -> u64 {
    let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES)
        .expect("6 MiB 的环末尾落在聚簇段边界上")
        .slot()
        .0;
    (unit_area_start_slot + UNIT_AREA_SLOTS) * SLOT_BYTES
}

fn create_inodes(pool: &mut Pool, count: u64) -> Vec<InodeNumber> {
    let inode_number_watermark = pool.current_file_version().inode_number_watermark();
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        UserChange::NewInodes {
            count,
            write_time_seconds,
        },
        None,
    )
    .expect("单元区离写满还远，建 inode 做得成");
    (inode_number_watermark..inode_number_watermark + count)
        .map(InodeNumber)
        .collect()
}

fn fill_the_file_with_data_units(pool: &mut Pool, inode: InodeNumber) {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷装得进 u64");
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let bytes = content_of(
        usize::try_from(DATA_UNITS_OF_EACH_FILE * payload).expect("几百 KiB"),
        write_time_seconds,
    );
    pool.publish(
        UserChange::WriteAtOffset {
            inode,
            offset: FileOffsetInBytes(0),
            bytes: PartialWriteBytes::new(&bytes).expect("写的字节不是空的"),
            write_time_seconds,
        },
        None,
    )
    .expect("单元区离写满还远，写做得成");
}

fn create_and_unlink_an_inode(pool: &mut Pool) {
    let inode = create_inodes(pool, 1)[0];
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode,
            write_time_seconds,
        },
        None,
    )
    .expect("刚建的 inode 删得掉");
}

/// 取样点 `files_1000_units_10` 的历史，交回第一个文件之外还活着的文件。
fn history_of_small_files_laying_a_long_release_chain(pool: &mut Pool) -> Vec<InodeNumber> {
    let big_file = create_inodes(pool, 1)[0];
    fill_the_file_with_data_units(pool, big_file);
    let mut files = vec![big_file];
    let mut remaining = SMALL_FILES;
    while remaining > 0 {
        let count = remaining.min(INODES_PER_BATCH);
        for inode in create_inodes(pool, count) {
            fill_the_file_with_data_units(pool, inode);
            files.push(inode);
        }
        remaining -= count;
    }
    for _ in 0..CREATE_AND_UNLINK_ROUNDS {
        create_and_unlink_an_inode(pool);
    }
    files
}

/// 这一版比上一版重写了的节点：上一版没有的，或指针变了的（重写一定换新槽）。
fn nodes_rewritten_between(
    previous: &TransactionOutput,
    version: &TransactionOutput,
) -> BTreeSet<AllocationRecordTreeNode> {
    version
        .allocation_record_tree
        .nodes
        .iter()
        .filter(|(node, pointer)| {
            previous.allocation_record_tree.pointer_of(*node) != Some(*pointer)
        })
        .map(|(node, _)| *node)
        .collect()
}

/// 经一个新写入口覆盖写 `inode`，判这一次的重写集，交回（固定点轮数，重写的角色数，重写的节点数）。
/// 重写集要恰是内容变了的节点：罩不住是固定点没定下来，多罩是闭包走过了头、多写了单元。
fn overwrite_and_judge_the_rewrite_set(
    pool: &mut Pool,
    inode: InodeNumber,
) -> (usize, usize, usize) {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let PoolUnderTest {
        parameters,
        devices,
        session,
        ..
    } = pool;
    let session = session.as_mut().expect("这一刻有可写会话");
    let previous = session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    let version = publish_overwrite_of_an_inode(
        &mut writer,
        &mut session.allocator,
        &previous,
        inode,
        &content,
        write_time_seconds,
        session.instance,
    )
    .expect("单元区离写满还远，覆盖写做得成");
    let rounds = writer.rounds_of_the_last_allocation_record_tree_settling();
    let geometry = AllocationRecordTreeGeometry::of_allocator(&session.allocator);
    let rewritten = nodes_rewritten_between(&previous, &version);
    assert_eq!(
        rewritten,
        nodes_whose_contents_changed(
            &geometry,
            &previous.allocation_records,
            &version.allocation_records
        ),
        "覆盖写 inode {} 的重写集恰是内容变了的节点",
        inode.0
    );
    let roles = version.rewritten.len();
    session.current = PoolVersion::of_a_version_of_trees(version);
    (rounds, roles, rewritten.len())
}

fn assert_the_rounds_stay_within_the_peak_of_this_history(rounds: usize, inode: InodeNumber) {
    assert!(
        (FEWEST_ROUNDS_OF_A_SETTLING..=MOST_ROUNDS_OF_A_SETTLING_ON_THIS_HISTORY).contains(&rounds),
        "覆盖写 inode {} 的固定点走了 {rounds} 轮，这段历史上走闭包量到 {FEWEST_ROUNDS_OF_A_SETTLING} 到 {MOST_ROUNDS_OF_A_SETTLING_ON_THIS_HISTORY} 轮",
        inode.0
    );
}

/// 判别力：`crates/mutations.tsv`「分配记录树固定点」开头那几行。释放那一处不走闭包的一行红在轮数（重写集那几条断言照过，这就是两种做法重写集相同的量）。
// 一个场景：量窗那几次覆盖写接在同一段历史上按次序做，后一次读前一次留下的那一版，轮数本身就是被量的次序效应。
#[test]
fn overwrites_after_a_long_release_chain_settle_within_the_measured_rounds_and_rewrite_exactly_the_changed_nodes(
) {
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        device_bytes(),
        plain_device,
    );
    let cold_files = history_of_small_files_laying_a_long_release_chain(&mut pool);
    let first_file = InodeNumber(FIRST_INODE_NUMBER);

    let (
        rounds_right_after_the_history,
        roles_right_after_the_history,
        nodes_right_after_the_history,
    ) = overwrite_and_judge_the_rewrite_set(&mut pool, first_file);
    println!(
        "SETTLING right_after_the_history rounds={rounds_right_after_the_history} roles={roles_right_after_the_history} nodes={nodes_right_after_the_history}"
    );
    assert_eq!(
        (roles_right_after_the_history, nodes_right_after_the_history),
        (
            ROLES_REWRITTEN_BY_THE_OVERWRITE_RIGHT_AFTER_THE_HISTORY,
            NODES_REWRITTEN_BY_THE_OVERWRITE_RIGHT_AFTER_THE_HISTORY
        ),
        "头一次覆盖写重写的角色与节点数，与不走闭包的那一臂量到的相同"
    );
    assert_the_rounds_stay_within_the_peak_of_this_history(
        rounds_right_after_the_history,
        first_file,
    );

    for overwrite_index in 1..FIRST_FILE_OVERWRITES {
        let (rounds, roles, nodes) = overwrite_and_judge_the_rewrite_set(&mut pool, first_file);
        println!("SETTLING first_file index={overwrite_index} rounds={rounds} roles={roles} nodes={nodes}");
        assert_the_rounds_stay_within_the_peak_of_this_history(rounds, first_file);
    }
    for overwrite_index in 0..COLD_FILE_OVERWRITES {
        let share = usize::try_from(overwrite_index).expect("下标");
        let cold_file = cold_files[share * cold_files.len() / SHARES_OF_THE_COLD_FILES];
        let (rounds, roles, nodes) = overwrite_and_judge_the_rewrite_set(&mut pool, cold_file);
        println!("SETTLING cold_file index={overwrite_index} rounds={rounds} roles={roles} nodes={nodes}");
        assert_the_rounds_stay_within_the_peak_of_this_history(rounds, cold_file);
    }
}
