//! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态的流、改坏一个字段的流与展开上限不展开的段只差一个写的状态。

use super::*;

// ───────────── 里程碑三·3 删除线：I-9.5、I-3.4、I-4.7、I-4.6 与 I-9.6 墓碑那一半（GPU 扫描内核的删除态扫描与 CPU 池级 checker 逐状态逐位对拍）─────────────

// 码 2 / 码 3 单元头的声明长度与记录区、inode 记录、码 2 节点头、`deleted_inodes` 根、记账叶的偏移与宽度，与待删除但意图未完成占用那一行的
// 两段（`UNIT_DECLARED_LENGTH_OFFSET`、`INODE_RECORD_*`、`ACCOUNTING_*`、`index_node_entry_count_offset` 这些）取删除中间态搭建模块
// `common_deletion_in_progress` 里那一份，这里不另抄：同一个测试目标里同名，单元图按裸名两份都连。
/// 打包记录类型：1 墓碑、2 inode。
pub(super) const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;

pub(super) const PACKED_RECORD_TYPE_INODE: u16 = 2;

/// I-9.5 正方向那一形塞进队列的号：inode 树里没有它的记录、extent 树里也没有它的单元。
const INODE_NUMBER_QUEUED_WITHOUT_A_RECORD: u64 = 7;

/// I-9.6 墓碑那一半那一形把墓碑的对象 ID 改成的值：远大于这条流的 inode 号水位（新池新建文件之后是 2）。
const TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK: u64 = 1 << 40;

pub(super) fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
}

pub(super) fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
}

pub(super) fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}

/// 删除线那几条流共用的收尾：`pool` 从 mkfs 之后录下的写切段，被判的是最后一次根槽写。
pub(super) fn recorded_flow_of_the_pool(
    pool: &BuiltPool,
    name: &'static str,
    versions: Vec<PublishedVersion>,
) -> RecordedFlow {
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// 删除中间态的流（`common_deletion_in_progress` 的搭建）：新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）、unlink 意图（txg 5：
/// inode 1 的记录 nlink 写成 0、号进 `deleted_inodes`、墓碑树开放容器追加一条）、按批大小 1 删第一批（txg 6：剩两个单元、size 两个净荷）。
/// 恢复从意图那一版起答「没有文件」，所以带文件的版本只有 txg 3、4。
pub(super) fn record_new_pool_through_the_first_deletion_batch(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    )
}

/// 删除线那几条流上读得出第一个文件的两版：新建文件（txg 3）与顺序写成三个数据单元（txg 4）；意图那一版起恢复答「没有文件」。
pub(super) fn versions_with_the_first_file_before_its_deletion() -> Vec<PublishedVersion> {
    vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: content_of_three_data_units(),
        },
    ]
}

/// 三个单元的文件按批大小 1 删几批删完：前两批各放掉尾部一个单元，最后一批删记录、队列里的号与最后一个单元
/// （`unlink_single_file_publishes.rs` 的 `the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish`）。
const BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE: usize = 3;

/// 删完的流：同 [`record_new_pool_through_the_first_deletion_batch`] 的搭建，再一批批删到最后一批（txg 6、7、8）。最后一批那一版起
/// extent 树只剩上段零条目根（D8（核心索引结构） 已定项 16 第 5 条）、没有下段与数据单元，inode 树里没有第一个文件那条记录，
/// 队列空；恢复从意图那一版起答「没有文件」，带文件的版本同样只有 txg 3、4。
///
/// # Panics
/// 最后一批那一版还有第一个文件的记录、队列里的号、数据单元或下段，或 extent 上段不是只剩一个节点：那这条流就没走到零条目根。
pub(super) fn record_new_pool_through_the_last_deletion_batch(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    let last_batch = (0..BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE)
        .map(|_batch_index| publish_the_next_step_in_batches_of_one(&mut pool))
        .last()
        .expect("删完要至少一批");
    assert!(
        last_batch.inode_record.is_none()
            && last_batch.deleted_inodes.is_empty()
            && last_batch.data_pointers.is_empty()
            && last_batch.extent_tree.lower_nodes.is_empty()
            && last_batch.extent_tree.upper_nodes.len() == 1,
        "第 {BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE} 批是最后一批：记录、队列里的号、数据单元与下段都不剩，extent 树只剩上段零条目根（{:?}）",
        last_batch.extent_tree
    );
    recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    )
}

/// I-3.5「走到没有条款的形状」（`pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 同一形）：删到意图、留 2、留 1，
/// 中间那一版（留 2）树表里 `deleted_inodes` 那一条（带根）的种类改成没登记的码、重封到根。
/// 主走读不往下走，中间那一版的引用集合数不全，它参与的那两对两边都不判；照判的话最新那一版整棵继承的 `deleted_inodes` 树红 I-3.5。
fn record_deletion_with_an_unregistered_kind_on_the_deleted_inodes_entry_of_an_intermediate_version(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-intermediate-version-tree-table-entry-of-an-unregistered-kind";
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    );
    let tree_table_copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite && write.bytes().is_some_and(is_a_tree_table_unit)
        })
        .map(|(index, _)| index)
        .collect();
    let intermediate_version_copies =
        &tree_table_copies[tree_table_copies.len() - 4..tree_table_copies.len() - 2];
    assert_eq!(
        flow.writes[intermediate_version_copies[0]].bytes(),
        flow.writes[intermediate_version_copies[1]].bytes(),
        "倒数第二张树表的两份副本字节相同（每一版重写树表，一张两份）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, intermediate_version_copies, &|bytes| {
        let entry = only_tree_table_entry_of_kind(bytes, TREE_KIND_DELETED_INODES);
        let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
        assert_ne!(
            bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "删除中间那一版的 deleted_inodes 树有根"
        );
        let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
        bytes[kind..kind + 2].copy_from_slice(&UNREGISTERED_TREE_KIND.to_le_bytes());
    });
    flow
}

/// CPU 红 I-9.16、不红 I-3.5，I-3.5 在整条落盘的镜像上报部分没判、点名那一条条目的种类；GPU 每个状态逐位相同。
/// GPU 不把这一版记成引用没收全时，最新那一版整棵继承的 `deleted_inodes` 树判红 I-3.5，这一位对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_intermediate_deletion_version_carries_a_tree_table_entry_of_an_unregistered_kind(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_when_an_intermediate_deletion_version_carries_a_tree_table_entry_of_an_unregistered_kind";
    let flow = record_deletion_with_an_unregistered_kind_on_the_deleted_inodes_entry_of_an_intermediate_version(
        "crash-judge-gpu-i35-intermediate-version-unregistered-kind",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.5") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains(&format!(
                "有一版树表里有种类 {UNREGISTERED_TREE_KIND} 的条目，走读走到没有条款的形状、引用没收全，这一对不判"
            )),
            "I-3.5 的部分没判点名那一条条目的种类：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("中间那一版的引用没收全：I-3.5 要报部分没判，实际 {other_verdict:?}")
        }
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16")
            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 红 I-9.16、一个状态都不红 I-3.5（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 删除中间态的流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
pub(super) fn record_deletion_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

fn is_a_deleted_inodes_root(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == DELETED_INODES_KEY_BYTES
        && usize::from(index_node_entry_width(bytes)) == DELETED_INODES_KEY_BYTES
}

pub(super) fn is_an_accounting_leaf(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == ACCOUNTING_KEY_BYTES
        && usize::from(index_node_entry_width(bytes)) == ACCOUNTING_ROW_BYTES
}

pub(super) fn is_an_inode_leaf_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
}

pub(super) fn is_a_tombstone_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_TOMBSTONE
}

/// inode 叶容器里号 `inode` 那条记录的起点。
pub(super) fn offset_of_the_inode_record(bytes: &[u8], inode: u64) -> usize {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    (0..record_count)
        .map(|index| PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES)
        .find(|offset| read_u64_at(bytes, *offset) == inode)
        .unwrap_or_else(|| panic!("这片 inode 叶容器里有 inode {inode} 的记录"))
}

/// I-9.5 正方向那一形：`deleted_inodes` 根 {1} 改成 {1, 7}（条目数加一、声明长度加 8、头里最大 key 改成 7），7 号在 inode 树里没有记录。
fn put_an_inode_number_without_a_record_into_the_deletion_queue_root(bytes: &mut [u8]) {
    let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
    let entry_count = usize::from(read_u16_at(bytes, count_offset));
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    assert_eq!(
        usize::from(declared_length),
        entry_count * DELETED_INODES_KEY_BYTES,
        "队列根的声明长度是条目数 × 8"
    );
    let new_entry =
        index_node_entries_start(DELETED_INODES_KEY_BYTES) + entry_count * DELETED_INODES_KEY_BYTES;
    bytes[new_entry..new_entry + DELETED_INODES_KEY_BYTES]
        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
    bytes[count_offset..count_offset + 2].copy_from_slice(
        &u16::try_from(entry_count + 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length + u16::try_from(DELETED_INODES_KEY_BYTES).expect("8")).to_le_bytes(),
    );
    let largest_key = INDEX_NODE_SMALLEST_KEY_OFFSET + DELETED_INODES_KEY_BYTES;
    bytes[largest_key..largest_key + DELETED_INODES_KEY_BYTES]
        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
}

/// I-3.4 那一形：记账叶里待删除但意图未完成占用（标签 4、设备段 0xFFFF_FFFF）那一行的值 0 → 1。
fn set_the_pending_deletion_occupancy_row_to_one_in_the_accounting_leaf_bytes(bytes: &mut [u8]) {
    let entry_count = usize::from(read_u16_at(
        bytes,
        index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
    ));
    let row = (0..entry_count)
        .map(|index| index_node_entries_start(ACCOUNTING_KEY_BYTES) + index * ACCOUNTING_ROW_BYTES)
        .find(|row| {
            read_u16_at(bytes, *row) == STATISTIC_PENDING_DELETION_OCCUPANCY
                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
                    == STATISTIC_NO_DEVICE_DIMENSION
        })
        .expect("记账叶里有待删除但意图未完成占用那一行");
    assert_eq!(
        read_u64_at(bytes, row + ACCOUNTING_ROW_VALUE_OFFSET),
        0,
        "那一行恒 0"
    );
    bytes[row + ACCOUNTING_ROW_VALUE_OFFSET..row + ACCOUNTING_ROW_VALUE_OFFSET + 8]
        .copy_from_slice(&1u64.to_le_bytes());
}

/// I-4.6 那一形：inode 叶容器里第一个文件（删除中、nlink = 0）的 size 多加一个净荷容量，blocks 跟着按 ⌈size ÷ 512⌉ 改（I-9.15 照旧成立），
/// extent 树里的单元数不动。
fn raise_the_size_of_the_first_file_by_one_data_unit_payload_in_the_inode_leaf_bytes(
    bytes: &mut [u8],
) {
    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
    let payload_capacity =
        u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("装得进 u64");
    let size = read_u64_at(bytes, record + INODE_RECORD_SIZE_OFFSET) + payload_capacity;
    bytes[record + INODE_RECORD_SIZE_OFFSET..record + INODE_RECORD_SIZE_OFFSET + 8]
        .copy_from_slice(&size.to_le_bytes());
    bytes[record + INODE_RECORD_BLOCKS_OFFSET..record + INODE_RECORD_BLOCKS_OFFSET + 8]
        .copy_from_slice(
            &size
                .div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)
                .to_le_bytes(),
        );
}

/// I-9.6 墓碑那一半那一形：墓碑容器第一条墓碑的对象 ID 改成远大于 inode 号水位的值。
fn raise_the_first_tombstone_object_identifier_past_the_watermark(bytes: &mut [u8]) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "墓碑容器里至少一条墓碑"
    );
    bytes[PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + 8]
        .copy_from_slice(&TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK.to_le_bytes());
}

/// I-3.4 缺行那一形：记账叶里待删除但意图未完成占用那一行整行拿掉（后面的行往前挪、条目数减一、声明长度减 34；这一行不在首末，头里的 key 区间不变）。
fn drop_the_pending_deletion_occupancy_row_from_the_accounting_leaf_bytes(bytes: &mut [u8]) {
    let count_offset = index_node_entry_count_offset(ACCOUNTING_KEY_BYTES);
    let entry_count = usize::from(read_u16_at(bytes, count_offset));
    let entries_start = index_node_entries_start(ACCOUNTING_KEY_BYTES);
    let row_index = (0..entry_count)
        .find(|index| {
            let row = entries_start + index * ACCOUNTING_ROW_BYTES;
            read_u16_at(bytes, row) == STATISTIC_PENDING_DELETION_OCCUPANCY
                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
                    == STATISTIC_NO_DEVICE_DIMENSION
        })
        .expect("记账叶里有待删除但意图未完成占用那一行");
    assert!(
        row_index > 0 && row_index + 1 < entry_count,
        "那一行不在首末：拿掉它头里的 key 区间不变"
    );
    let row = entries_start + row_index * ACCOUNTING_ROW_BYTES;
    let rows_end = entries_start + entry_count * ACCOUNTING_ROW_BYTES;
    bytes.copy_within(row + ACCOUNTING_ROW_BYTES..rows_end, row);
    bytes[rows_end - ACCOUNTING_ROW_BYTES..rows_end].fill(0);
    bytes[count_offset..count_offset + 2].copy_from_slice(
        &u16::try_from(entry_count - 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length - u16::try_from(ACCOUNTING_ROW_BYTES).expect("34")).to_le_bytes(),
    );
}

/// 新池新建文件之后建一个空 inode（txg 4，号 2，size 0、没有数据单元，与第一个文件同在最左那片叶里）的流：
/// 那片叶（最后一个 inode 叶容器的两份副本）照 `mutate` 改一处、重封到根。
pub(super) fn record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
    tag: &str,
    name: &'static str,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let pool = pool_with_new_inodes(tag, 1);
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: file_content(),
            },
        ],
    );
    let targets = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// I-4.7 那一形（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F1 的孤儿单元，在 checker 侧直接改字节造）：
/// 空 inode 那片叶里 1 号的记录拿掉（后面的记录往前挪、记录数减一、声明长度减 140），extent 树不动：1 号在 extent 树里还有数据单元、
/// inode 树里没有它的记录、号也不在队列里。
fn drop_the_record_of_the_first_file(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 2, "第一个文件与空 inode 同在这片叶里");
    let dropped = offset_of_the_inode_record(bytes, FIRST_FILE.0);
    let records_end = PACKED_UNIT_RECORDS_START + record_count * INODE_RECORD_BYTES;
    bytes.copy_within(dropped + INODE_RECORD_BYTES..records_end, dropped);
    bytes[records_end - INODE_RECORD_BYTES..records_end].fill(0);
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(record_count - 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length - u16::try_from(INODE_RECORD_BYTES).expect("140")).to_le_bytes(),
    );
}

/// I-9.5 反方向那一形（C708、判决里说的僵尸记录）：空 inode 2 的记录 nlink 1 → 0，号不进队列（它没有数据单元，I-4.7、I-4.6 照旧成立）。
fn set_the_nlink_of_the_new_inode_to_zero(bytes: &mut [u8]) {
    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
    assert_eq!(
        read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
        1,
        "空 inode 的 nlink 是 1"
    );
    bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
        .copy_from_slice(&0u32.to_le_bytes());
}

/// 删除中间态（意图之后、第一批之后）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局：意图那一版起
/// 两边都把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
/// （I-9.5 队列 {1} 与 nlink = 0 的号集合相同、I-3.4 标签 4 那一行在且为 0、I-4.7 号 1 在队列里、I-4.6 单元数等于 ⌈size ÷ 净荷容量⌉）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
    let flow = record_new_pool_through_the_first_deletion_batch(
        "crash-judge-gpu-deletion-in-progress",
        "deletion-through-the-first-batch",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删除中间态的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删除中间态的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 删到最后一批（意图之后三批）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局）：最后一批那一版起
/// extent 树只剩上段零条目根，CPU 冷走读把它当第一个文件删完之后的合法形态收下（`recovery.rs` 的 `extents_of_the_first_file`）、
/// inode 树里没有那条记录、答没有文件，GPU 恢复内核同判（`judge_extent_node` 只放过上段根兼叶的零条目，读第一个文件放过零条叶条目）；
/// 删除线四条在这条流上 CPU 判过而成立、一个状态都不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch() {
    let flow = record_new_pool_through_the_last_deletion_batch(
        "crash-judge-gpu-deletion-through-the-last-batch",
        "deletion-through-the-last-batch",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删到最后一批的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删到最后一批的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 一条流换一种切段，造出「之前的段全落、第 `segment_index` 段只差第 `missing_write_index` 次写」那一个状态：`segments` 把原来那一段拆成
/// 「这一段其余的写」与「只有这一次写」前后两段（别的段照旧），只展开后一段（下标 `segment_index + 1`）。持久集合与原来那一段全量展开
/// （[`CrashPointReplaySegmentExpansion::EveryProperSubset`]）枚举到的状态之一逐个相同；状态只有它与最后全落那一个。
struct SegmentsMissingOneWrite {
    segment_index: usize,
    missing_write_index: usize,
    pub(super) segments: Vec<Vec<usize>>,
}

/// [`the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress`] 拆哪几段：
/// 写数超过它的段。
const EXPAND_UP_TO_OF_THE_SEGMENTS_SPLIT_BY_ONE_MISSING_WRITE: usize = 8;

/// 写数超过 `limit`（按这个展开上限不展开）的每一段、段里每一次写各一种切法（[`SegmentsMissingOneWrite`]）。
///
/// # Panics
/// 这几段里有不是单元写的写：单元写是 COW、只取两态（`TearableInPlaceOverwrites` 认法第 1 条），拆段不改它落不落；原地覆写的撕裂镜像要重放
/// 同一段里更晚的重叠写，拆开之后那几次写归到前一段，造出的状态就不是原来那一段的了。
fn segments_missing_one_write_of_each_unexpanded_segment(
    flow: &RecordedFlow,
    limit: usize,
) -> Vec<SegmentsMissingOneWrite> {
    let mut variants = Vec::new();
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        if segment.len() <= limit {
            continue;
        }
        for write_index in segment {
            assert_eq!(
                flow.writes[*write_index].kind,
                StepKind::UnitWrite,
                "流 {} 不展开的第 {segment_index} 段里第 {write_index} 次写要是单元写，拆段才不改它的落地",
                flow.name
            );
        }
        for missing_write_index in segment {
            let rest_of_the_segment: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| write_index != missing_write_index)
                .collect();
            let mut segments = Vec::with_capacity(flow.segments.len() + 1);
            segments.extend(flow.segments[..segment_index].iter().cloned());
            segments.push(rest_of_the_segment);
            segments.push(vec![*missing_write_index]);
            segments.extend(flow.segments[segment_index + 1..].iter().cloned());
            variants.push(SegmentsMissingOneWrite {
                segment_index,
                missing_write_index: *missing_write_index,
                segments,
            });
        }
    }
    variants
}

/// 删除中间态那条流上，展开上限不展开的段（新建文件、顺序写三个单元、意图、第一批那几次发布的单元写，每段二三十个写）在
/// [`compare_on_every_state`] 里一个状态都不比：「这一次发布的单元写落了一半」时 GPU 的删除态扫描与恢复内核从没和 CPU 比过。
/// 这里每段每次写各比一次「之前的段全落、这一段只差这一次写」的状态（连同最后全落那一个），
/// GPU 与 CPU 整份相同；CPU 那一侧逐状态现判，不抄数。第一批那一段的这些状态上最新落盘的根是意图那一版，第一个文件 nlink = 0，
/// 两边都要答没有文件（C712（GPU 恢复内核不判 nlink = 0））。每种切法恰好两个状态在取卡之前判，没卡的机器上也判到。
// harness-test-granularity:one-scenario 同一条录好的流只录一次、不重建池，各轮是同一份写序列换一种切段、每种切法只有两个状态，合起来是这条流上展开上限不展开的段那一个场景
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress(
) {
    let flow = record_new_pool_through_the_first_deletion_batch(
        "crash-judge-gpu-deletion-missing-one-write",
        "deletion-through-the-first-batch-missing-one-write",
    );
    // 拆的是按 8 不展开的段（这条用例立的时候对拍的展开上限）：对拍没设变量时取全量域、这几段在 compare_on_every_state 里本来就展开，
    // 这里照旧按 8 拆，钉的状态不随对拍的展开上限变。
    let limit = EXPAND_UP_TO_OF_THE_SEGMENTS_SPLIT_BY_ONE_MISSING_WRITE;
    let variants = segments_missing_one_write_of_each_unexpanded_segment(&flow, limit);
    assert!(
        !variants.is_empty(),
        "流 {} 按展开上限 {limit} 要有不展开的段（发布的单元写），这条用例才比得到它们",
        flow.name
    );
    let expansion_of = |variant: &SegmentsMissingOneWrite| {
        let expanded_segment_index = variant.segment_index + 1;
        move |segment_index: usize, _segment: &[usize]| {
            if segment_index == expanded_segment_index {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        }
    };
    for variant in &variants {
        assert_eq!(
            crash_point_replay_plan_state_count(
                &flow.base,
                &flow.writes,
                &variant.segments,
                &expansion_of(variant)
            ),
            2,
            "第 {} 段只差第 {} 次写的切法只有那一个状态与最后全落那一个",
            variant.segment_index,
            variant.missing_write_index
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress",
    ) else {
        return;
    };
    let mut failures = Vec::new();
    for variant in &variants {
        // 每种切法一个项名：同一条流在两类库里按切法各录一份
        let item_name = format!(
            "{}.segment-{}-missing-write-{}",
            flow.name, variant.segment_index, variant.missing_write_index
        );
        let Some(comparison) = compare_on_every_state_of_these_segments_as_the_item(
            cards,
            &item_name,
            &flow,
            &variant.segments,
            &expansion_of(variant),
        ) else {
            return;
        };
        if !comparison.mismatches.is_empty() {
            failures.push(format!(
                "第 {} 段只差第 {} 次写：\n{}",
                variant.segment_index,
                variant.missing_write_index,
                mismatch_listing(&comparison)
            ));
        }
    }
    eprintln!(
        "CRASH_JUDGE_GPU_MISSING_ONE_WRITE flow={} limit={limit} unexpanded_segment_lengths={:?} states_missing_one_write={} failed={}",
        flow.name,
        flow.segments
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.len() > limit)
            .map(|(segment_index, segment)| (segment_index, segment.len()))
            .collect::<Vec<_>>(),
        variants.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "流 {}：{} 个只差一个写的状态 GPU 与 CPU 不一致：\n{}",
        flow.name,
        failures.len(),
        failures.join("\n\n")
    );
}

/// 队列 {1} → {1, 7}：7 号没有记录（I-9.5 正方向红），队列里两个不同的号（I-9.24 红，读队列在配对之前判）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i95",
        "deletion-queue-names-an-inode-without-a-record",
        &is_a_deleted_inodes_root,
        &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
        &flow,
        &["I-9.5", "I-9.24"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i34",
        "deletion-pending-occupancy-row-one",
        &is_an_accounting_leaf,
        &set_the_pending_deletion_occupancy_row_to_one_in_the_accounting_leaf_bytes,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
        &flow,
        "I-3.4",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i34-missing-row",
        "deletion-pending-occupancy-row-missing",
        &is_an_accounting_leaf,
        &drop_the_pending_deletion_occupancy_row_from_the_accounting_leaf_bytes,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
        &flow,
        "I-3.4",
    );
}

/// 这条流上没有 nlink = 0 的记录（第一个文件的记录整条拿掉了），恢复两边都答「没有文件」，所以连恢复一起逐状态比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue",
        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
            "crash-judge-gpu-i47",
            "new-pool-new-inode-first-file-record-dropped",
            &drop_the_record_of_the_first_file,
        ),
        "I-4.7",
    );
}

/// 这条流上 nlink = 0 的是空 inode 2，第一个文件照常可读，恢复两边都读出文件，所以连恢复一起逐状态比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue",
        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
            "crash-judge-gpu-i95-zombie",
            "new-pool-new-inode-nlink-zero-outside-the-queue",
            &set_the_nlink_of_the_new_inode_to_zero,
        ),
        "I-9.5",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units(
) {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i46",
        "deletion-size-past-the-data-units",
        &is_an_inode_leaf_container,
        &raise_the_size_of_the_first_file_by_one_data_unit_payload_in_the_inode_leaf_bytes,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
        &flow,
        "I-4.6",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark()
{
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i96-tombstone",
        "deletion-tombstone-past-the-watermark",
        &is_a_tombstone_container,
        &raise_the_first_tombstone_object_identifier_past_the_watermark,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
        &flow,
        "I-9.6",
    );
}
