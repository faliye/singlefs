//! 删除中间态的搭建与池级 checker 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半，墓碑与谱系那几条 I-9.18–I-9.20、I-9.24、
//! I-9.10 墓碑那一个等号）的坏镜像：
//! 新池新建文件之后把第一个文件顺序写成三个数据单元、按只供测试的批大小 1 发意图与几批（D8（核心索引结构） 已定项 16），
//! 或建几个空 inode；坏镜像在发布路径写出的一版上找到要改的单元（从最新的根读树表往下找），改一处、重封，
//! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::change_the_unit_and_propagate_its_checksum_on`）。
//! 用这一份的测试文件同时声明 `mod common;` 与 `mod common_pointer_head_corruption;`（照 `common_tree_split` 的做法，两份测试文件各 `mod` 一次）。
#![allow(dead_code, reason = "两份测试文件各自只用到其中一部分")]

use singlefs_checker::image::{
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration, SlotNumber};
use singlefs_core::bytes::ByteReader;
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_LIVELIST, TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
};
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step,
    publish_unlink_steps_until_the_queue_is_empty, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
    BuiltPool, PoolImageGeometry, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_pointer_head_corruption::{
    change_the_unit_and_propagate_its_checksum_on, data_unit_bytes, newest_root, node_bytes,
    the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
};

/// 删除与建 inode 都在新池新建文件那个实例里发。
pub const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);
/// 第一个文件的 inode 号。
pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
/// 建空 inode 那次的写入时间。
pub const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;

/// 坏镜像：在一份干净镜像上改一处。
pub type Mutation = Box<dyn Fn(&mut MemoryPool)>;

/// 码 3 单元头里记录数（偏移 69）、声明长度（偏移 8）与记录区起点（含 29 字节预留位的头 136，D18（块里携带什么信息） 已定项 16）。
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_UNIT_RECORDS_START: usize = 136;
/// inode 记录 140 字节：inode 号 0、nlink 36（4 字节）、size 40、blocks 48（D8（核心索引结构） 已定项 6 字段表）。
const INODE_RECORD_BYTES: usize = 140;
const INODE_RECORD_NLINK_OFFSET: usize = 36;
const INODE_RECORD_SIZE_OFFSET: usize = 40;
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
/// `blocks` = ⌈size ÷ 512⌉（I-9.15）。
const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
/// inode 树与墓碑树的码 2 内部条目：分隔 key 8 + 身份引用 26 + 子指针 86（子指针从 34 起）。
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
/// 码 2 节点头（key 宽 k）：最大 key 在 52 + k、条目数在 82 + 2k、条目区从 115 + 2k 起（D18（块里携带什么信息） 已定项 18）。
const INDEX_NODE_SMALLEST_KEY_OFFSET: usize = 52;
fn index_node_entry_count_offset(key_width: usize) -> usize {
    82 + 2 * key_width
}
fn index_node_entries_start(key_width: usize) -> usize {
    115 + 2 * key_width
}
/// `deleted_inodes` 根：key 宽 8、条目宽 8（D8（核心索引结构） 已定项 16）。
const DELETED_INODES_KEY_BYTES: usize = 8;
/// 记账树叶：key 宽 22、条目 34（统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
const ACCOUNTING_KEY_BYTES: usize = 22;
const ACCOUNTING_ROW_BYTES: usize = 34;
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 10;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项），不带设备维的设备段。
pub const STATISTIC_PENDING_DELETION_OCCUPANCY: u16 = 4;
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
/// 树表（码 2，key = 树 ID 8 字节）：条目 200 字节，树的种类在条目偏移 10、头 ID 在 116（D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_KEY_BYTES: usize = 8;
const TREE_TABLE_ENTRY_BYTES: usize = 200;
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
/// 树表条目里根指针 86 字节占 [14, 100)（`walk.rs` 读 `entry[14..100]`）。
const TREE_TABLE_ENTRY_ROOT_POINTER_START: usize = 14;
const TREE_TABLE_ENTRY_ROOT_POINTER_END: usize = 100;
/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码（攻方 L1a 把 `deleted_inodes` 那一条的种类 9 改成它）：
/// 13 第 50 批登记给了退役树、14 / 15 里程碑四登记给了只读快照的两种条目（快照 inode 根、快照 extent 根），改用 16。
const TREE_KIND_NOT_REGISTERED: u16 = 16;
/// 码 2 节点头里条目宽 2 字节在 84 + 2k、最大 key 在 52 + k（D18（块里携带什么信息） 已定项 7、18）。
fn index_node_entry_width_offset(key_width: usize) -> usize {
    84 + 2 * key_width
}
fn index_node_largest_key_offset(key_width: usize) -> usize {
    INDEX_NODE_SMALLEST_KEY_OFFSET + key_width
}
/// 码 3 单元头里写序的实例代号（4 字节，偏移 93）：已发布谓词按它与诞生代号判（I-1.2）。
const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
/// 墓碑记录 40 字节：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32（D18（块里携带什么信息） 已定项 24 第 2 条）。
const TOMBSTONE_RECORD_BYTES: usize = 40;
const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
pub const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
pub const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
pub const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
/// 码 3 单元头里记录宽（2 字节，偏移 71，D18（块里携带什么信息） 已定项 16）：I-1.7 比它与登记表的宽。
pub const PACKED_UNIT_RECORD_WIDTH_OFFSET: usize = 71;
/// 根环一圈的根槽数（三个区域、每区 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 8 槽，`common::parameters`）再多几次：
/// 发这么多次之后，更早的版本都不在回退候选集里（根槽被盖掉）。攻方 `research/prompts/m3-delete-checker-r2-opus-output.md` 第五节
/// 「再建 30 次」那一造法：坏镜像要改的墓碑容器被意图那一版共用时，改它会让意图那一版连带别的条红，先让那一版转出候选集再改。
pub const PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER: u64 = 30;

pub fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
}

/// 三个数据单元的内容：最后一个单元装半个净荷（与 `unlink_single_file_publishes.rs` 那一份同形）。
pub fn content_of_three_data_units() -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..2 * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件之后把第一个文件顺序写成三个数据单元。
pub fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
    pool
}

/// unlink 第一个文件的意图那次发布：inode 1 的记录 nlink 写成 0、size 不变，`deleted_inodes` 加 1，墓碑树开放容器追加一条。
pub fn publish_the_intent(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let write_time_seconds = previous
        .inode_record
        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        write_time_seconds,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("意图");
    pool.output = output.clone();
    output
}

/// 按批大小 1 删下一批（不是最后一批的：放掉尾部一个单元、size 改成剩下单元数 × 净荷容量；最后一批删记录与队列里的号）。
pub fn publish_the_next_step_in_batches_of_one(pool: &mut BuiltPool) -> TransactionOutput {
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
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("删除的下一步");
    pool.output = output.clone();
    output
}

/// 三个单元的文件发完意图：inode 1 nlink 0、size 2.5 个净荷、三个单元都在，队列 {1}，墓碑树一条墓碑。
pub fn image_after_the_intent_of_a_three_unit_file(tag: &str) -> MemoryPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    pool.memory_pool()
}

/// 三个单元的文件发完意图与第一批：inode 1 nlink 0、size 2 个净荷、剩两个单元，队列 {1}。
pub fn image_after_the_first_batch_of_a_three_unit_file(tag: &str) -> MemoryPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    pool.memory_pool()
}

/// 新池新建文件之后建 `count` 个空 inode（号从 2 起，size 0、没有数据单元，与第一个文件同在最左那片叶里）。
pub fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, count);
    pool
}

/// 在池上再发一次建 `count` 个空 inode（号从记账水位起），交回这一次发布。
pub fn publish_new_inodes_and_advance_the_pool(
    pool: &mut BuiltPool,
    count: u64,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("建空 inode");
    pool.output = output.clone();
    output
}

/// 新池新建文件之后把第一个文件顺序写成空内容：长度 0 的顺序写照样写一个数据单元（`write_request_split` 的
/// `data_unit_count_of_a_sequential_write`，长度 0 时是 1），inode 1 size 0、一个数据单元。
pub fn pool_with_a_first_file_of_size_zero_and_one_data_unit(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &[]);
    assert_eq!(
        grown.data_pointers.len(),
        1,
        "长度 0 的顺序写写一个数据单元"
    );
    pool
}

/// 在池上发 `inode` 的 unlink 意图（意图那一次：记录 nlink 写成 0、队列加这个号、墓碑树开放容器追加一条），交回这一次发布。
pub fn publish_the_intent_of(pool: &mut BuiltPool, inode: u64) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    pool.output = output.clone();
    output
}

/// 把 `inode` unlink 到底：意图那一次（交回它），再按产品批大小发到删除队列空（记录删掉，墓碑留在墓碑树里）。
pub fn unlink_to_the_end(pool: &mut BuiltPool, inode: u64) -> TransactionOutput {
    let intent = publish_the_intent_of(pool, inode);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let mut current = pool.output.clone();
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = current;
    intent
}

/// 挂着的时候回退到 `target` 那一版（一次向前发布：新根的 inode 树、extent 树、`deleted_inodes` 取 `target` 的，墓碑树随现行那一版，
/// 谱系树追加 (这次发布的 txg, `target` 的 txg)，D23（journal 的角色与格式） 已定项 14、D18（块里携带什么信息） 已定项 24 第 5 条）。
/// 交回回退那次发布。
pub fn roll_back_to(pool: &mut BuiltPool, target: &TransactionOutput) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    )
    .unwrap_or_else(|refusal| {
        panic!(
            "回退到 txg {} 那一版：{refusal:?}",
            target.root.checkpoint_txg.0
        )
    });
    pool.output.clone()
}

/// 接着发 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次建一个空 inode：之前那几版都转出回退候选集。
pub fn turn_the_root_ring_over_by_creating_inodes(pool: &mut BuiltPool) {
    // 迭代上界是常量 30。
    for _ in 0..PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER {
        publish_new_inodes_and_advance_the_pool(pool, 1);
    }
}

/// 这一版 inode 树里 `inode` 那条记录的对象出生代；没有这条记录交回 `None`。
pub fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<CheckpointTxg> {
    output
        .inode_leaf_container_contents()
        .iter()
        .flat_map(|container| container.records.iter())
        .find(|record| record.inode == inode)
        .map(|record| record.object_birth)
}

/// 新池新建文件之后建一个空 inode（2 号），再发它的 unlink 意图：队列 {2}、2 号 nlink 0、size 0、没有数据单元，墓碑树一条墓碑。
pub fn image_after_the_intent_of_an_empty_inode(tag: &str) -> MemoryPool {
    let mut pool = pool_with_new_inodes(tag, 1);
    publish_the_intent_of(&mut pool, 2);
    pool.memory_pool()
}

/// 新池新建文件之后分两次各建一个空 inode（2 号、3 号，对象出生代不同），把 3 号 unlink 到底（记录删掉，墓碑 (3, 3 号的出生代) 在谱系上），
/// 再建 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次：3 号的记录还在的那几版（含意图那一版，它与删完之后各版共用那片墓碑容器）都转出回退候选集。
pub fn pool_with_inode_three_unlinked_and_the_root_ring_turned_over(tag: &str) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, 1);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_to_the_end(&mut pool, 3);
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    pool
}

/// 新池新建文件之后建一个空 inode（2 号），把它 unlink 到底，再回退到发意图之前那一版（2 号又活着、nlink 1；墓碑树随现行那一版，
/// 那条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再建 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次：
/// 那条墓碑还在谱系上的几版（意图与删完那两版）都转出回退候选集。
pub fn pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over(
    tag: &str,
) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, 1);
    let before_the_intent = pool.output.clone();
    unlink_to_the_end(&mut pool, 2);
    roll_back_to(&mut pool, &before_the_intent);
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    pool
}

/// `slot` 那个单元（两盘同槽，`length_in_bytes` 宽）改一处、重封，新的整单元校验和沿引用链补到根槽。镜像是 `common::build_pool` 这一类
/// 4 GiB 盘、默认 journal 环上的：照这组盘的几何调 `common_pointer_head_corruption::change_the_unit_and_propagate_its_checksum_on`。
pub fn change_the_unit_and_propagate_its_checksum(
    image: &mut MemoryPool,
    slot: SlotNumber,
    length_in_bytes: usize,
    change: impl FnOnce(&mut Vec<u8>),
) {
    change_the_unit_and_propagate_its_checksum_on(
        image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        slot,
        length_in_bytes,
        change,
    );
}

fn get_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}
fn get_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}
fn get_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}
fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 一个码 2 内部条目（inode 树、墓碑树：120 字节）里子指针指的那个槽。
fn child_slot_of_a_container_tree_entry(entry: &[u8]) -> SlotNumber {
    let child = NodePointer::read_from(&mut ByteReader::at(
        entry,
        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
    ));
    assert_eq!(
        child.locations[0].slot, child.locations[1].slot,
        "第一版两盘同槽"
    );
    child.locations[0].slot
}

/// `leaf_slot` 那片 inode 叶容器里 `inode` 那条记录的下标。
fn index_of_the_inode_record_in_the_leaf_at(
    image: &MemoryPool,
    leaf_slot: SlotNumber,
    inode: u64,
) -> Option<usize> {
    let leaf = unit_bytes_at(image, leaf_slot, data_unit_bytes());
    let record_count = usize::from(get_u16(&leaf, PACKED_UNIT_RECORD_COUNT_OFFSET));
    (0..record_count).find(|index| {
        get_u64(
            &leaf,
            PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES,
        ) == inode
    })
}

/// 最新的根那一版 inode 树里装着 `inode` 那条记录的叶容器的槽（根是层级 1、孩子都是叶容器的那一档）与记录在容器里的下标。
fn inode_leaf_holding(image: &MemoryPool, inode: u64) -> (SlotNumber, usize) {
    let inode_root = tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
    let root = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&inode_root),
        node_bytes(),
    ))
    .expect("inode 树根解得开");
    assert_eq!(root.level, 1, "这几份镜像上 inode 树根直接管叶容器");
    root.entries
        .iter()
        .map(|entry| child_slot_of_a_container_tree_entry(entry))
        .find_map(|leaf_slot| {
            index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, inode)
                .map(|index| (leaf_slot, index))
        })
        .unwrap_or_else(|| panic!("最新的根那一版 inode 树里有 inode {inode} 的记录"))
}

/// 最新的根那一版里 `inode` 那条记录改一处（`change` 拿到的是那 140 字节），重封叶、补校验和。
pub fn change_the_inode_record(image: &mut MemoryPool, inode: u64, change: impl FnOnce(&mut [u8])) {
    let (leaf_slot, _) = inode_leaf_holding(image, inode);
    change_the_inode_record_in_the_leaf_at(image, leaf_slot, inode, change);
}

/// `leaf_slot` 那片 inode 叶容器（哪一版的都行：只被一版引用的那片改了只动那一版）里 `inode` 那条记录改一处，重封叶、补校验和。
pub fn change_the_inode_record_in_the_leaf_at(
    image: &mut MemoryPool,
    leaf_slot: SlotNumber,
    inode: u64,
    change: impl FnOnce(&mut [u8]),
) {
    let index = index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, inode)
        .unwrap_or_else(|| panic!("槽 {} 那片叶里有 inode {inode} 的记录", leaf_slot.0));
    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
        let start = PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES;
        change(&mut leaf[start..start + INODE_RECORD_BYTES]);
    });
}

/// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：`inode` 的记录 nlink 1 → 0，队列不动。
pub fn set_the_nlink_of_an_inode_record_to_zero(image: &mut MemoryPool, inode: u64) {
    let (leaf_slot, _) = inode_leaf_holding(image, inode);
    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(image, leaf_slot, inode);
}

/// [`set_the_nlink_of_an_inode_record_to_zero`] 改的是 `leaf_slot` 那片叶（哪一版的都行）。
pub fn set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(
    image: &mut MemoryPool,
    leaf_slot: SlotNumber,
    inode: u64,
) {
    change_the_inode_record_in_the_leaf_at(image, leaf_slot, inode, |record| {
        assert_eq!(
            get_u32(record, INODE_RECORD_NLINK_OFFSET),
            1,
            "改之前 inode {inode} 的 nlink 是 1"
        );
        set_u32(record, INODE_RECORD_NLINK_OFFSET, 0);
    });
}

/// I-9.5 正方向「队列里的号在树里 nlink ≠ 0」那一形（C708（nlink 为 0 却不在删除队列，checker 判不出） 还差的那一形）：
/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1（size、单元都不动）。
pub fn set_the_nlink_of_the_queued_first_file_back_to_one(image: &mut MemoryPool) {
    change_the_inode_record(image, FIRST_INODE_NUMBER, |record| {
        assert_eq!(
            get_u32(record, INODE_RECORD_NLINK_OFFSET),
            0,
            "意图之后 1 号的 nlink 是 0"
        );
        set_u32(record, INODE_RECORD_NLINK_OFFSET, 1);
    });
}

/// I-9.5 正方向「队列里的号没有记录」那一形（判决第二节最后一行：只红 I-9.5）：空 inode 2 发完意图之后的 `deleted_inodes` 根 {2} 改成 {7}
/// （条目与头里的最小、最大 key 2 → 7），7 号在 inode 树里没有记录、extent 树里也没有单元。队列只一个号（I-9.24 照旧成立）；
/// 2 号 nlink 0 而不在队列里，反方向也红在 I-9.5 上；2 号没有数据单元、size 0，I-4.7、I-4.6 照旧成立；
/// 7 号没有记录，I-9.18 那一个号判不了（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.18 那一行）。
pub fn replace_the_queued_inode_with_a_number_without_a_record(image: &mut MemoryPool) {
    const INODE_NUMBER_WITHOUT_A_RECORD: u64 = 7;
    const QUEUED_EMPTY_INODE: u64 = 2;
    let deleted_inodes_root = tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root;
    change_the_unit_and_propagate_its_checksum(
        image,
        the_slot_of(&deleted_inodes_root),
        node_bytes(),
        |root| {
            let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
            let entries_start = index_node_entries_start(DELETED_INODES_KEY_BYTES);
            assert_eq!(get_u16(root, count_offset), 1, "意图之后队列里一个号");
            assert_eq!(
                get_u64(root, entries_start),
                QUEUED_EMPTY_INODE,
                "队列里是 2"
            );
            set_u64(
                root,
                INDEX_NODE_SMALLEST_KEY_OFFSET,
                INODE_NUMBER_WITHOUT_A_RECORD,
            );
            set_u64(
                root,
                index_node_largest_key_offset(DELETED_INODES_KEY_BYTES),
                INODE_NUMBER_WITHOUT_A_RECORD,
            );
            set_u64(root, entries_start, INODE_NUMBER_WITHOUT_A_RECORD);
        },
    );
}

/// I-9.24（删除队列至多一条 key）只红自己的那一形：一次发布建了两个空 inode（2、3 号，对象出生代相同）、发完 2 号的意图之后，
/// 手改三处让 3 号也成一个合法的删除中号——`deleted_inodes` 根 {2} 加 3（条目数 1 → 2、声明长度 8 → 16、头里最大 key 2 → 3）、
/// 3 号记录 nlink 1 → 0、墓碑容器追加 (3, 3 号的对象出生代, 整对象, 意图那次的 txg)（记录数 1 → 2、声明长度 40 → 80）。
/// 少改哪一处都会连带别的条（I-9.5、I-9.18、I-9.19）。
pub fn put_a_second_inode_being_deleted_into_the_deletion_queue(image: &mut MemoryPool) {
    const SECOND_QUEUED_INODE: u64 = 3;
    let deleted_inodes_root = tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root;
    change_the_unit_and_propagate_its_checksum(
        image,
        the_slot_of(&deleted_inodes_root),
        node_bytes(),
        |root| {
            let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
            let entries_start = index_node_entries_start(DELETED_INODES_KEY_BYTES);
            assert_eq!(get_u16(root, count_offset), 1, "意图之后队列里一个号");
            assert_eq!(get_u64(root, entries_start), 2, "队列里是 2");
            set_u16(root, count_offset, 2);
            set_u16(root, UNIT_DECLARED_LENGTH_OFFSET, 2 * 8);
            set_u64(
                root,
                index_node_largest_key_offset(DELETED_INODES_KEY_BYTES),
                SECOND_QUEUED_INODE,
            );
            set_u64(root, entries_start + 8, SECOND_QUEUED_INODE);
        },
    );
    change_the_inode_record(image, SECOND_QUEUED_INODE, |record| {
        assert_eq!(
            get_u32(record, INODE_RECORD_NLINK_OFFSET),
            1,
            "改之前 3 号的 nlink 是 1"
        );
        set_u32(record, INODE_RECORD_NLINK_OFFSET, 0);
    });
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
                1,
                "意图之后墓碑容器里一条墓碑"
            );
            assert_eq!(
                get_u64(container, PACKED_UNIT_RECORDS_START),
                2,
                "那条墓碑的对象 ID 是 2"
            );
            let first = PACKED_UNIT_RECORDS_START;
            let second = first + TOMBSTONE_RECORD_BYTES;
            let mut tombstone_of_the_second = container[first..second].to_vec();
            set_u64(
                &mut tombstone_of_the_second,
                TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
                SECOND_QUEUED_INODE,
            );
            container[second..second + TOMBSTONE_RECORD_BYTES]
                .copy_from_slice(&tombstone_of_the_second);
            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
            set_u16(
                container,
                UNIT_DECLARED_LENGTH_OFFSET,
                u16::try_from(2 * TOMBSTONE_RECORD_BYTES).expect("80"),
            );
        },
    );
}

/// 最新的根那一版墓碑树第一片容器里第一条墓碑改一处（`change` 拿到的是那 40 字节），重封、补校验和。
pub fn change_the_first_tombstone_of_the_newest_root(
    image: &mut MemoryPool,
    change: impl FnOnce(&mut [u8]),
) {
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            let first = PACKED_UNIT_RECORDS_START;
            change(&mut container[first..first + TOMBSTONE_RECORD_BYTES]);
        },
    );
}

/// 最新的根那一版墓碑树第一片容器里第一条墓碑原样再追加一条（记录数 1 → 2、声明长度 40 → 80）：同一对 (号, 对象出生代)、同一个死亡 txg 两条。
pub fn append_a_copy_of_the_first_tombstone_of_the_newest_root(image: &mut MemoryPool) {
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
                1,
                "墓碑容器里一条墓碑"
            );
            let first = PACKED_UNIT_RECORDS_START;
            let second = first + TOMBSTONE_RECORD_BYTES;
            let tombstone = container[first..second].to_vec();
            container[second..second + TOMBSTONE_RECORD_BYTES].copy_from_slice(&tombstone);
            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
            set_u16(
                container,
                UNIT_DECLARED_LENGTH_OFFSET,
                u16::try_from(2 * TOMBSTONE_RECORD_BYTES).expect("80"),
            );
        },
    );
}

/// 退役记录 34 字节：被退役身份（出生树 0、打包记录类型 8、容器号 10、容器出生代 18，共 26 字节）、死亡 txg 26
/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）。
pub const RETIREMENT_RECORD_BYTES: usize = 34;
pub const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
pub const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
pub const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;
pub const RETIREMENT_RECORD_IDENTITY_BYTES: usize = 26;
/// inode 树内部条目里身份引用的起点（分隔 key 8 字节之后，26 字节，排法同退役记录的被退役身份）。
const INODE_INTERNAL_ENTRY_IDENTITY_OFFSET: usize = 8;

/// 最新的根那一版退役树第一片容器里第一条退役记录改一处（`change` 拿到的是那 34 字节），重封、补校验和。
pub fn change_the_first_retirement_record_of_the_newest_root(
    image: &mut MemoryPool,
    change: impl FnOnce(&mut [u8]),
) {
    let container_slot =
        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            let first = PACKED_UNIT_RECORDS_START;
            change(&mut container[first..first + RETIREMENT_RECORD_BYTES]);
        },
    );
}

/// 最新的根那一版退役树第一片容器（这几份镜像上只一条记录）末尾再追加一条：`record_of` 拿第一条那 34 字节、交回要追加的那 34 字节；
/// 记录数 1 → 2、声明长度 34 → 68，重封、补校验和。
pub fn append_a_retirement_record_to_the_first_container_of_the_newest_root(
    image: &mut MemoryPool,
    record_of: impl FnOnce(&[u8]) -> Vec<u8>,
) {
    let container_slot =
        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
                1,
                "退役容器里一条退役记录"
            );
            let first = PACKED_UNIT_RECORDS_START;
            let second = first + RETIREMENT_RECORD_BYTES;
            let appended = record_of(&container[first..second]);
            assert_eq!(appended.len(), RETIREMENT_RECORD_BYTES, "退役记录 34 字节");
            container[second..second + RETIREMENT_RECORD_BYTES].copy_from_slice(&appended);
            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
            set_u16(
                container,
                UNIT_DECLARED_LENGTH_OFFSET,
                u16::try_from(2 * RETIREMENT_RECORD_BYTES).expect("68"),
            );
        },
    );
}

/// 最新的根那一版 inode 树根（层级 1，直接管叶容器）第一条条目的身份引用那 26 字节（排法同退役记录的被退役身份）。
pub fn identity_of_the_first_inode_leaf_of_the_newest_root(image: &MemoryPool) -> Vec<u8> {
    let inode_tree_root = tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
    let root = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&inode_tree_root),
        node_bytes(),
    ))
    .expect("inode 树根解得开");
    assert_eq!(root.level, 1, "这几份镜像上 inode 树根直接管叶容器");
    root.entries.first().expect("inode 树根至少一条条目")[INODE_INTERNAL_ENTRY_IDENTITY_OFFSET
        ..INODE_INTERNAL_ENTRY_IDENTITY_OFFSET + RETIREMENT_RECORD_IDENTITY_BYTES]
        .to_vec()
}

/// 退役记录（34 字节）里 `offset` 那 8 字节读成数。
pub fn retirement_record_field(record: &[u8], offset: usize) -> u64 {
    get_u64(record, offset)
}

/// 退役记录（34 字节）里 `offset` 那 8 字节改成 `value`。
pub fn set_retirement_record_field(record: &mut [u8], offset: usize, value: u64) {
    set_u64(record, offset, value);
}

/// 退役记录（34 字节）里被退役身份的打包记录类型改成 `record_type`。
pub fn set_the_retired_record_type(record: &mut [u8], record_type: u16) {
    set_u16(record, RETIREMENT_RECORD_RECORD_TYPE_OFFSET, record_type);
}

/// 一条墓碑（40 字节）里 `offset` 那 8 字节读成数。
pub fn tombstone_field(tombstone: &[u8], offset: usize) -> u64 {
    get_u64(tombstone, offset)
}

/// 一条墓碑（40 字节）里 `offset` 那 8 字节改成 `value`。
pub fn set_tombstone_field(tombstone: &mut [u8], offset: usize, value: u64) {
    set_u64(tombstone, offset, value);
}

/// I-9.4 第四句（同一片打包记录类型 2 容器里记录号沿记录序严格递增）只红自己的那一形（S4a，代码轮判决
/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节）：新池新建文件之后建一个空 inode 的那一版，叶 [1, 2] 改成
/// [1 nlink 0, 1 nlink 1]——第二条整条换成 1 号那条原样，第一条 nlink 改 0；队列空。扫描里同号后者盖前者（1 号 nlink 1），
/// I-9.5、I-4.7 照旧成立。
pub fn turn_the_leaf_into_two_records_of_the_first_file(image: &mut MemoryPool) {
    let (leaf_slot, first_index) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
    assert_eq!(first_index, 0, "1 号是叶里第 0 条");
    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
        assert_eq!(
            get_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET),
            2,
            "叶里两条：1 号与空 inode 2"
        );
        let first = PACKED_UNIT_RECORDS_START;
        let second = first + INODE_RECORD_BYTES;
        let record_of_the_first_file = leaf[first..second].to_vec();
        leaf[second..second + INODE_RECORD_BYTES].copy_from_slice(&record_of_the_first_file);
        set_u32(leaf, first + INODE_RECORD_NLINK_OFFSET, 0);
    });
}

/// 最新的根那一版种类 `kind`（墓碑树或谱系树）的树根第一条条目指的那片容器头里的记录宽改成 `record_width`（I-1.7 只看头字段，
/// 声明长度、记录区不动）。
pub fn set_the_record_width_of_the_first_container_of_the_tree_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    record_width: u16,
) {
    let container_slot = first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            set_u16(container, PACKED_UNIT_RECORD_WIDTH_OFFSET, record_width);
        },
    );
}

/// I-4.7 只红自己的那一形（判决第二节 F1：孤儿单元，今天的实现走不到，在 checker 侧直接改字节造）：第一个文件（inode 1，有一个内联数据单元）
/// 与空 inode 2 同在最左那片叶里，把 1 号那条记录从叶里拿掉（2 号挪到第 0 条、记录数 2 → 1、声明长度 280 → 140），extent 树不动：
/// 1 号在 extent 树里还有数据单元、inode 树里没有它的记录、号也不在队列里。
pub fn drop_the_record_of_the_first_file_and_keep_its_data_units(image: &mut MemoryPool) {
    let (leaf_slot, _) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(image, leaf_slot);
}

/// [`drop_the_record_of_the_first_file_and_keep_its_data_units`] 改的是 `leaf_slot` 那片叶（哪一版的都行，叶里恰好 1 号与 2 号两条）。
pub fn drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(
    image: &mut MemoryPool,
    leaf_slot: SlotNumber,
) {
    assert_eq!(
        index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, FIRST_INODE_NUMBER),
        Some(0),
        "1 号是叶里第 0 条"
    );
    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
        assert_eq!(
            get_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET),
            2,
            "叶里两条：1 号与空 inode 2"
        );
        let second = PACKED_UNIT_RECORDS_START + INODE_RECORD_BYTES;
        let second_record = leaf[second..second + INODE_RECORD_BYTES].to_vec();
        leaf[PACKED_UNIT_RECORDS_START..second].copy_from_slice(&second_record);
        leaf[second..second + INODE_RECORD_BYTES].fill(0);
        set_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET, 1);
        set_u16(
            leaf,
            UNIT_DECLARED_LENGTH_OFFSET,
            u16::try_from(INODE_RECORD_BYTES).expect("140"),
        );
    });
}

/// I-4.6 只红自己的那一形：删除中间那一版 inode 1 的 size 多加一个净荷容量（blocks 跟着按 ⌈size ÷ 512⌉ 改，I-9.15 照旧成立），
/// extent 树里的单元数不动。
pub fn raise_the_size_of_the_first_file_by_one_data_unit_payload(image: &mut MemoryPool) {
    let (leaf_slot, _) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
    raise_the_size_of_the_first_file_in_the_leaf_at(image, leaf_slot);
}

/// [`raise_the_size_of_the_first_file_by_one_data_unit_payload`] 改的是 `leaf_slot` 那片叶（删除中间某一版的叶）。
pub fn raise_the_size_of_the_first_file_in_the_leaf_at(
    image: &mut MemoryPool,
    leaf_slot: SlotNumber,
) {
    change_the_inode_record_in_the_leaf_at(image, leaf_slot, FIRST_INODE_NUMBER, |record| {
        assert_eq!(
            get_u32(record, INODE_RECORD_NLINK_OFFSET),
            0,
            "1 号处于删除中"
        );
        let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量");
        let size = get_u64(record, INODE_RECORD_SIZE_OFFSET) + payload_capacity;
        set_u64(record, INODE_RECORD_SIZE_OFFSET, size);
        set_u64(
            record,
            INODE_RECORD_BLOCKS_OFFSET,
            size.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES),
        );
    });
}

/// 记账树（`slot` 那个根兼叶，层级 0）里待删除但意图未完成占用那一行改一处：`Some(value)` 改值，`None` 把这一行从叶里拿掉
/// （后面的行往前挪、条目数与声明长度各减一行；这一行不在首末，头里的 key 区间不变）。
pub fn change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(
    image: &mut MemoryPool,
    slot: SlotNumber,
    value: Option<u64>,
) {
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        assert_eq!(node[50], 0, "这几份镜像上记账树只有根兼叶");
        let count_offset = index_node_entry_count_offset(ACCOUNTING_KEY_BYTES);
        let entries_start = index_node_entries_start(ACCOUNTING_KEY_BYTES);
        let row_count = usize::from(get_u16(node, count_offset));
        let row_index = (0..row_count)
            .find(|index| {
                let row = entries_start + index * ACCOUNTING_ROW_BYTES;
                get_u16(node, row) == STATISTIC_PENDING_DELETION_OCCUPANCY
                    && get_u32(node, row + ACCOUNTING_ROW_DEVICE_OFFSET)
                        == STATISTIC_NO_DEVICE_DIMENSION
            })
            .expect("记账树里有待删除但意图未完成占用那一行");
        let row = entries_start + row_index * ACCOUNTING_ROW_BYTES;
        match value {
            Some(value) => set_u64(node, row + ACCOUNTING_ROW_VALUE_OFFSET, value),
            None => {
                assert!(
                    row_index > 0 && row_index + 1 < row_count,
                    "这一行不在首末：拿掉它头里的 key 区间不变"
                );
                let end = entries_start + row_count * ACCOUNTING_ROW_BYTES;
                node.copy_within(row + ACCOUNTING_ROW_BYTES..end, row);
                node[end - ACCOUNTING_ROW_BYTES..end].fill(0);
                set_u16(
                    node,
                    count_offset,
                    u16::try_from(row_count - 1).expect("行数"),
                );
                set_u16(
                    node,
                    UNIT_DECLARED_LENGTH_OFFSET,
                    u16::try_from((row_count - 1) * ACCOUNTING_ROW_BYTES).expect("声明长度"),
                );
            }
        }
    });
}

/// 最新的根那一版的记账树根的槽。
pub fn accounting_tree_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
    the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_ACCOUNTING).root)
}

/// I-3.4：最新的根那一版的记账树里待删占用那一行 0 → 1。
pub fn set_the_pending_deletion_occupancy_row_to_one(image: &mut MemoryPool) {
    let slot = accounting_tree_slot_of_the_newest_root(image);
    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(image, slot, Some(1));
}

/// I-3.4：最新的根那一版的记账树里拿掉待删占用那一行。
pub fn drop_the_pending_deletion_occupancy_row(image: &mut MemoryPool) {
    let slot = accounting_tree_slot_of_the_newest_root(image);
    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(image, slot, None);
}

/// I-9.6 墓碑那一半：3 号删完、根环转过一圈之后（`pool_with_inode_three_unlinked_and_the_root_ring_turned_over`）墓碑树第一片容器里
/// 那条墓碑的对象 ID 3 → `inode_number_watermark`（记账里的 inode 号水位）：水位不再大于全部墓碑的对象 ID，inode 树里最大的 key 仍小于水位。
/// 那条墓碑所在的容器被意图那一版共用，意图那一版要先转出候选集，不然它的队列 {3} 找不到 3 号的墓碑、I-9.18 连带红。
pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(
    image: &mut MemoryPool,
    inode_number_watermark: u64,
) {
    change_the_first_tombstone_of_the_newest_root(image, |tombstone| {
        assert_eq!(
            get_u64(tombstone, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
            3,
            "那条墓碑的对象 ID 是 3"
        );
        set_u64(
            tombstone,
            TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
            inode_number_watermark,
        );
    });
}

/// 意图之后记账里的 inode 号水位（第一个文件用掉 1 号）。
const INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 2;

/// 最新的根那一版墓碑树根第一条条目指的那片墓碑容器的槽（这几份镜像上墓碑树根是层级 1、孩子是容器）。
pub fn first_tombstone_container_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
    first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_TOMBSTONE)
}

/// 最新的根那一版种类 `kind`（墓碑树或谱系树）的树根第一条条目指的那片容器的槽（这几份镜像上树根是层级 1、孩子是容器）。
pub fn first_container_slot_of_the_tree_of_kind_of_the_newest_root(
    image: &MemoryPool,
    kind: u16,
) -> SlotNumber {
    let container_tree_root = tree_table_entry_of_kind(image, kind).root;
    let first_entry = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&container_tree_root),
        node_bytes(),
    ))
    .expect("容器树根解得开")
    .entries
    .first()
    .expect("容器树根至少一条条目")
    .clone();
    child_slot_of_a_container_tree_entry(&first_entry)
}

/// I-9.6 墓碑那一半「已发布」那一支（`walk.rs` 的 `largest_published_tombstone_object_identifier` 里按已发布谓词滤掉的容器）：
/// 意图之后那片墓碑容器头里写序的实例代号 1 → 2（大于挂载根的实例 1，已发布谓词判它不是已发布，I-1.2 那一行状态列），
/// 那条墓碑的对象 ID 也抬到水位 2。这片容器不算进墓碑那一半，I-9.6 照旧成立；容器被所选根引用而写序不是已发布，由别的条判红。
pub fn move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(
    image: &mut MemoryPool,
) {
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                get_u32(container, PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET),
                INSTANCE_OF_THE_PUBLISHES.0,
                "墓碑容器是挂载根那个实例写的"
            );
            set_u32(
                container,
                PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET,
                INSTANCE_OF_THE_PUBLISHES.0 + 1,
            );
            let object_identifier =
                PACKED_UNIT_RECORDS_START + TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET;
            assert_eq!(
                get_u64(container, object_identifier),
                FIRST_INODE_NUMBER,
                "那条墓碑的对象 ID 是 1"
            );
            set_u64(
                container,
                object_identifier,
                INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE,
            );
        },
    );
}

/// 一个单元两份全零（两盘同槽，不补引用链）：那个单元读不出。
pub fn zero_both_copies_of_the_unit_at(
    image: &mut MemoryPool,
    slot: SlotNumber,
    length_in_bytes: usize,
) {
    let zeroed = vec![0_u8; length_in_bytes];
    for device in image.devices.values_mut() {
        device.write(slot.to_device_offset(), &zeroed);
    }
}

/// 最新的根那一版树表单元的槽。
pub fn tree_table_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
    the_slot_of(&newest_root(image).tree_table)
}

fn tree_table_entry_count(tree_table: &[u8]) -> usize {
    usize::from(get_u16(
        tree_table,
        index_node_entry_count_offset(TREE_TABLE_KEY_BYTES),
    ))
}

fn tree_table_entry_start(index: usize) -> usize {
    index_node_entries_start(TREE_TABLE_KEY_BYTES) + index * TREE_TABLE_ENTRY_BYTES
}

/// 树表里种类是 `kind` 的那一条（恰好一条）的下标。
fn index_of_the_only_tree_table_entry_of_kind(tree_table: &[u8], kind: u16) -> usize {
    let indexes: Vec<usize> = (0..tree_table_entry_count(tree_table))
        .filter(|index| {
            get_u16(
                tree_table,
                tree_table_entry_start(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
            ) == kind
        })
        .collect();
    let [index] = indexes.as_slice() else {
        panic!("树表里种类 {kind} 的条目恰好一条：{indexes:?}")
    };
    *index
}

/// L1a（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，攻方报告第二节）：
/// `tree_table_slot` 那张树表（哪一版的都行）里 `deleted_inodes` 那一条的种类码 9 改成没登记的 13，树 ID、头 ID、根指针都不动：
/// 那一版树表里没有指着 inode 树的 `deleted_inodes` 条目。
pub fn give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
    image: &mut MemoryPool,
    tree_table_slot: SlotNumber,
) {
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let index =
                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_DELETED_INODES);
            set_u16(
                tree_table,
                tree_table_entry_start(index) + TREE_TABLE_ENTRY_KIND_OFFSET,
                TREE_KIND_NOT_REGISTERED,
            );
        },
    );
}

/// L1b：`tree_table_slot` 那张树表里整条拿掉 `deleted_inodes` 那一条（后面的条目往前挪、末尾清零、条目数与声明长度各减一条；
/// 那一条不在首末，头里的 key 区间不变）。
pub fn remove_the_deleted_inodes_entry_from_the_tree_table_at(
    image: &mut MemoryPool,
    tree_table_slot: SlotNumber,
) {
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let count = tree_table_entry_count(tree_table);
            let index =
                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_DELETED_INODES);
            assert!(
                index > 0 && index + 1 < count,
                "deleted_inodes 那一条不在首末：拿掉它头里的 key 区间不变"
            );
            let start = tree_table_entry_start(index);
            let end = tree_table_entry_start(count);
            tree_table.copy_within(start + TREE_TABLE_ENTRY_BYTES..end, start);
            tree_table[end - TREE_TABLE_ENTRY_BYTES..end].fill(0);
            set_u16(
                tree_table,
                index_node_entry_count_offset(TREE_TABLE_KEY_BYTES),
                u16::try_from(count - 1).expect("条目数"),
            );
            let declared_length = get_u16(tree_table, UNIT_DECLARED_LENGTH_OFFSET);
            set_u16(
                tree_table,
                UNIT_DECLARED_LENGTH_OFFSET,
                declared_length - u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
            );
        },
    );
}

/// `tree_table_slot` 那张树表里种类是 `kind` 的那一条的头 ID（条目偏移 116）改成 `head_identifier`，别的字段不动。
pub fn set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
    image: &mut MemoryPool,
    tree_table_slot: SlotNumber,
    kind: u16,
    head_identifier: u64,
) {
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let index = index_of_the_only_tree_table_entry_of_kind(tree_table, kind);
            set_u64(
                tree_table,
                tree_table_entry_start(index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
                head_identifier,
            );
        },
    );
}

/// S1（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节，攻方报告第二节 2.2）：`tree_table_slot` 那张树表里
/// 根指针全零的 livelist 条目（种类 6）种类改成 1、头 ID 改成 inode 树的树 ID，冒充这棵 inode 树的 extent 树；真 extent 树那一条的头 ID 改成 0。
/// 树 ID、根指针都不动。只从 inode 树一侧按头 ID 配 extent 树的话，配上的是那条空根，真 extent 树一次都不读。
pub fn let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(
    image: &mut MemoryPool,
    tree_table_slot: SlotNumber,
) {
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let inode_tree_index =
                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_INODE);
            let inode_tree = get_u64(tree_table, tree_table_entry_start(inode_tree_index));
            let extent_tree_index =
                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_EXTENT);
            let livelist_index =
                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_LIVELIST);
            let livelist_entry = tree_table_entry_start(livelist_index);
            assert!(
                tree_table[livelist_entry + TREE_TABLE_ENTRY_ROOT_POINTER_START
                    ..livelist_entry + TREE_TABLE_ENTRY_ROOT_POINTER_END]
                    .iter()
                    .all(|byte| *byte == 0),
                "livelist 条目的根指针全零（空根）"
            );
            set_u64(
                tree_table,
                tree_table_entry_start(extent_tree_index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
                0,
            );
            set_u16(
                tree_table,
                livelist_entry + TREE_TABLE_ENTRY_KIND_OFFSET,
                TREE_KIND_EXTENT,
            );
            set_u64(
                tree_table,
                livelist_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
                inode_tree,
            );
        },
    );
}

/// `tree_table_slot` 那张树表的条目宽收窄到 key 宽（8）：条目数改 1、声明长度改 8、最大 key 贴到剩下那一条的 key 上、
/// 条目区之后清零（照 `checker_known_bad_images.rs` 的 `narrow_the_entry_width_to_the_key_width`）。别的自描述字段自洽，
/// 只是条目比字段表的 200 字节窄。
pub fn narrow_the_entries_of_the_tree_table_at_to_the_key_width(
    image: &mut MemoryPool,
    tree_table_slot: SlotNumber,
) {
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let key_width = TREE_TABLE_KEY_BYTES;
            let key_width_field = u16::try_from(key_width).expect("8");
            let entries_start = index_node_entries_start(key_width);
            let first_key = tree_table[entries_start..entries_start + key_width].to_vec();
            set_u16(tree_table, UNIT_DECLARED_LENGTH_OFFSET, key_width_field);
            set_u16(tree_table, index_node_entry_count_offset(key_width), 1);
            set_u16(
                tree_table,
                index_node_entry_width_offset(key_width),
                key_width_field,
            );
            let largest_key = index_node_largest_key_offset(key_width);
            tree_table[largest_key..largest_key + key_width].copy_from_slice(&first_key);
            tree_table[entries_start + key_width..].fill(0);
        },
    );
}

/// 最新的根那一版树表里种类是 `kind` 的那一条指的根（`deleted_inodes`、extent 树）的槽。
pub fn root_slot_of_the_tree_of_kind_of_the_newest_root(
    image: &MemoryPool,
    kind: u16,
) -> SlotNumber {
    the_slot_of(&tree_table_entry_of_kind(image, kind).root)
}

/// 意图之后那一版上的坏镜像（`image_after_the_intent_of_a_three_unit_file`）。1 号 nlink 0 → 1 那一份必与 I-9.19（谱系上的墓碑不指活对象）
/// 同红（墓碑 (1, 1 号的出生代) 在谱系上而记录活着），登记在 I-9.5 名下。
pub fn known_bad_images_after_the_intent_of_a_three_unit_file() -> Vec<(&'static str, Mutation)> {
    vec![(
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
        Box::new(set_the_nlink_of_the_queued_first_file_back_to_one),
    )]
}

/// 空 inode 2 发完意图之后那一版上的坏镜像（`image_after_the_intent_of_an_empty_inode`）。
pub fn known_bad_images_after_the_intent_of_an_empty_inode() -> Vec<(&'static str, Mutation)> {
    vec![(
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
        Box::new(replace_the_queued_inode_with_a_number_without_a_record),
    )]
}

/// 意图与第一批之后那一版上的坏镜像（`image_after_the_first_batch_of_a_three_unit_file`）。
pub fn known_bad_images_after_the_first_batch_of_a_three_unit_file() -> Vec<(&'static str, Mutation)>
{
    vec![(
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        Box::new(raise_the_size_of_the_first_file_by_one_data_unit_payload),
    )]
}

/// 第一个文件之后建了一个空 inode（2 号）那一版上的坏镜像（`pool_with_new_inodes(…, 1)`）。
pub fn known_bad_images_after_one_new_inode() -> Vec<(&'static str, Mutation)> {
    vec![
        (
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
            Box::new(|image: &mut MemoryPool| set_the_nlink_of_an_inode_record_to_zero(image, 2)),
        ),
        (
            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
            Box::new(drop_the_record_of_the_first_file_and_keep_its_data_units),
        ),
    ]
}

/// 写完新池新建文件那一版（`build_pool`）上的坏镜像：记账标签 4 非 0、标签 4 缺行。
pub fn known_bad_images_of_the_pending_deletion_occupancy() -> Vec<(&'static str, Mutation)> {
    vec![
        (
            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
            Box::new(set_the_pending_deletion_occupancy_row_to_one),
        ),
        (
            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
            Box::new(drop_the_pending_deletion_occupancy_row),
        ),
    ]
}
