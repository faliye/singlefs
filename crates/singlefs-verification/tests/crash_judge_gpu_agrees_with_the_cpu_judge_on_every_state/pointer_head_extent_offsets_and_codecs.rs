//! 指针头部的 extent 偏移、MAC 与算法类型（I-2.4）：树表、根记录、inode、extent、分配记录树与容器树各处的指针，种类不小于 32 的重复。

use super::*;

// ───────────── 指针头部 extent 偏移，恢复那一半照 core 判、池级 checker 那一半补三处 ─────────────

/// inode 内部条目里子指针的偏移（D8（核心索引结构） 的字段表：分隔 key 8、身份引用 26 之后）。
pub(super) const INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET: usize = 34;

/// extent 树的 key 宽（locality 8、inode 8、文件偏移 8）。
pub(super) const EXTENT_KEY_BYTES: usize = 24;

/// extent 上段叶条目宽：key 24、标签 1、指针 88。
const EXTENT_UPPER_LEAF_ENTRY_BYTES: u16 = 113;

/// extent 下段叶记录宽：key 24、数据指针 88。
pub(super) const EXTENT_LOWER_LEAF_RECORD_BYTES: u16 = 112;

/// extent 上段叶条目的标签：1 指一个下段根、2 内联一个数据指针（标签之后就是那条指针）。
pub(super) const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT: u8 = 1;

const EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT: u8 = 2;

/// 分配记录树的 key 宽（盘 4、槽 6）与内部条目宽（key 10、子指针 86）。
pub(super) const ALLOCATION_RECORD_TREE_KEY_BYTES: usize = 10;

const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u16 = 96;

/// inode 树根的 key 宽与内部条目宽（D8（核心索引结构） 的 inode 内部条目 120）。
pub(super) const INODE_TREE_KEY_BYTES: usize = 8;

pub(super) const INODE_INTERNAL_ENTRY_BYTES: u16 = 120;

/// 偏移 `pointer` 起那条指针头部 extent 偏移那 2 字节（本来 0）写成 1。
pub(super) fn set_the_extent_offset_of_the_pointer_at(bytes: &mut [u8], pointer: usize) {
    let extent_offset = pointer + POINTER_EXTENT_OFFSET_OFFSET;
    assert_eq!(
        bytes[extent_offset..extent_offset + 2],
        [0u8; 2],
        "第一版指针的 extent 偏移恒 0"
    );
    bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
}

/// 指针头部里写成非第一版取值的那一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表）：extent 偏移（偏移 32 的 2 字节）写 1，
/// 或 MAC 第 0 字节写 1（加密关着时 MAC 恒 0，D19 已定项 3），或算法类型（偏移 28 的 1 字节）写 1（第一版不加密，D9（加密） 已定项 10）。
/// core 读走读跟随的指针都经 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 几段一并判，GPU 恢复内核各段都判；
/// 池级 checker 的 CPU 与 GPU 走读内核判算法类型那一段。
#[derive(Clone, Copy, Debug)]
pub(super) enum PointerHeadFieldWrittenNonZero {
    ExtentOffset,
    FirstMacByte,
    AlgorithmType,
}

/// 偏移 `pointer` 起那条指针头部 `field` 那一段（本来 0）写成非 0。
pub(super) fn write_the_pointer_head_field_non_zero_at(
    bytes: &mut [u8],
    pointer: usize,
    field: PointerHeadFieldWrittenNonZero,
) {
    match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            assert_eq!(bytes[pointer], 0, "加密关着时指针头部的 MAC 恒 0");
            bytes[pointer] = 1;
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            let algorithm_type = pointer + POINTER_ALGORITHM_TYPE_OFFSET;
            assert_eq!(
                bytes[algorithm_type], 0,
                "第一版指针头部的算法类型恒 0（不加密）"
            );
            bytes[algorithm_type] = 1;
        }
    }
}

/// 根记录实例表指针那一处：发布的根记录里实例表指针头部 extent 偏移写 1、重封根记录。池级 checker 走实例表链第 0 片时判它
/// （`walk_instance_table_chain`），CPU 与 GPU 走读内核都红 I-2.4。core 择根时把这条根当损坏拒（`root_record.rs` 的 `parse_slot` 第五关），
/// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointer_heads_are_all_the_first_version`）。
fn record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-root-record-instance-table-pointer-extent-offset-one",
    );
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        set_the_extent_offset_of_the_pointer_at(bytes, ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET);
    });
    flow
}

/// inode 内部条目子指针那一处：发布那一版 inode 树根（码 2、key 宽 8、条目宽 120）第一条内部条目的子指针头部 `field` 那一段写成非第一版的取值、
/// 重封到根。池级 checker 走 inode 内部条目时判它，CPU 与 GPU 走读内核都红 I-2.4；core 解这条内部条目时拒
/// （`records::parse_inode_internal_entry_or_refusal`），GPU 恢复内核在 `walk_to_file` 读 inode 树那一处同拒。
fn record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inode-internal-entry-child-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inode-internal-entry-child-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inode-internal-entry-child-pointer-algorithm-type-one"
        }
    };
    let inode_tree_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
    };
    record_new_pool_with_a_resealed_field(tag, name, &inode_tree_root, false, &|bytes| {
        let first_entry = index_node_entries_start(INODE_TREE_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            first_entry + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
            field,
        );
    })
}

/// extent 上段叶条目里的那条指针（标签之后）在单元里的偏移；条目的标签要是 `expected_tag`。
pub(super) fn pointer_of_the_first_extent_upper_leaf_entry(
    bytes: &[u8],
    expected_tag: u8,
) -> usize {
    let first_entry = index_node_entries_start(EXTENT_KEY_BYTES);
    assert_eq!(
        bytes[first_entry + EXTENT_KEY_BYTES],
        expected_tag,
        "上段叶第一条的标签"
    );
    first_entry + EXTENT_KEY_BYTES + 1
}

pub(super) fn is_an_extent_upper_leaf(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
        && index_node_entry_width(bytes) == EXTENT_UPPER_LEAF_ENTRY_BYTES
}

/// extent 数据指针那一处（内联在上段叶里的那条）：发布那一版唯一的数据单元由上段叶第一条内联指着（标签 2），这条数据指针头部 `field` 那一段
/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；
/// core 解上段叶条目时拒（`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
fn record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inline-data-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inline-data-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inline-data-pointer-algorithm-type-one"
        }
    };
    record_new_pool_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, false, &|bytes| {
        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT,
        );
        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
    })
}

/// 分配记录树内部条目子指针那一处：发布那一版分配记录树的根（码 2、key 宽 10、条目宽 96；一棵树里最后写出的内部节点）第一条内部条目的子指针
/// 头部 `field` 那一段写成非第一版的取值、重封到根。池级 checker 读分配记录树的码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；
/// core 读这条内部条目时拒（`allocation_record_tree` 读子指针那一处），GPU 恢复内核 `read_allocation_tree` 同拒。
fn record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-allocation-record-tree-child-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-allocation-record-tree-child-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-allocation-record-tree-child-pointer-algorithm-type-one"
        }
    };
    let allocation_internal_node = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == ALLOCATION_RECORD_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES
    };
    record_new_pool_with_a_resealed_field(tag, name, &allocation_internal_node, false, &|bytes| {
        let first_entry = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            first_entry + ALLOCATION_RECORD_TREE_KEY_BYTES,
            field,
        );
    })
}

/// 新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）：txg 4 那一版 extent 上段叶那一条指一个下段根（标签 1），
/// 下段叶记三条数据指针。搭建用 `common_deletion_in_progress` 的 `pool_with_a_three_unit_file`，不往下走删除。
pub(super) fn record_new_pool_through_a_three_unit_file(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let pool = pool_with_a_three_unit_file(tag);
    recorded_flow_of_the_pool(
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
                content: content_of_three_data_units(),
            },
        ],
    )
}

/// 三个数据单元那条流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
pub(super) fn record_three_unit_file_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// extent 下段根指针那一处：txg 4 那一版上段叶第一条（标签 1）指着的下段根指针头部 `field` 那一段写成非第一版的取值、重封到根。
/// 池级 checker 读下段根这个码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒
/// （`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
fn record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "three-unit-file-lower-segment-root-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "three-unit-file-lower-segment-root-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "three-unit-file-lower-segment-root-pointer-algorithm-type-one"
        }
    };
    record_three_unit_file_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, &|bytes| {
        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT,
        );
        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
    })
}

/// extent 数据指针那一处（下段叶记录里的那条）：txg 4 那一版下段叶（码 2、key 宽 24、条目宽 112）第一条记录的数据指针头部 `field` 那一段
/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解这条叶记录时拒
/// （`records::parse_extent_record`），GPU 恢复内核 `read_lower_segment` 读叶那一处同拒。
fn record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "three-unit-file-lower-leaf-data-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "three-unit-file-lower-leaf-data-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "three-unit-file-lower-leaf-data-pointer-algorithm-type-one"
        }
    };
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    record_three_unit_file_with_a_resealed_field(tag, name, &extent_lower_leaf, &|bytes| {
        let first_record = index_node_entries_start(EXTENT_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(bytes, first_record + EXTENT_KEY_BYTES, field);
    })
}

// ───────────── 容器树子指针 extent 偏移（E1）、恢复内核种类 ≥ 32 的重复、恢复内核指针头部加密压缩那几段 ─────────────

/// 墓碑树与谱系树码 2 节点条目里子指针的偏移（分隔 key 8、身份引用 26 之后）与条目宽，key 宽是容器号 8 字节。
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY: usize = 34;

pub(super) const CONTAINER_TREE_ENTRY_BYTES: u16 = 120;

pub(super) const CONTAINER_NUMBER_KEY_BYTES: usize = 8;

/// 码 2 节点条目里身份引用的类型段（分隔 key 8、出生树 8 之后的 2 字节）：指码 3 容器的条目写容器的记录类型。
pub(super) const CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET: usize = 16;

/// 谱系容器的打包记录类型（`singlefs_core::unit::PACKED_TYPE_LINEAGE`）。
pub(super) const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;

/// 一个墓碑树或谱系树的码 2 根（层级 1、key 宽 8、条目宽 120），第一条条目指一片记录类型 `record_type` 的容器。
/// inode 树根形状相同，靠第一条条目的类型段分开（inode 叶容器的类型是 2）。
pub(super) fn is_a_container_tree_root_over_containers_of_record_type(
    bytes: &[u8],
    record_type: u16,
) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
        && read_u16_at(
            bytes,
            index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == record_type
}

/// 容器树根第一条条目里指向容器的子指针头部 extent 偏移写 1。
fn set_the_extent_offset_of_the_first_container_pointer(bytes: &mut [u8]) {
    set_the_extent_offset_of_the_pointer_at(
        bytes,
        index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
            + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY,
    );
}

/// E1 墓碑树那一处：删除中间态的流上墓碑树根第一条条目里指向墓碑容器的子指针头部 extent 偏移写 1、重封到根。
/// 池级 checker 走墓碑树时判它（`walk_container_tree_root`），CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针
/// （`read_container_tree_index_nodes`），GPU 恢复内核 `read_container_tree_index_nodes` 同拒，都在找第一个文件那条记录之前。
fn record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-container-pointer-extent-offset-one",
        &|bytes| {
            is_a_container_tree_root_over_containers_of_record_type(
                bytes,
                PACKED_RECORD_TYPE_TOMBSTONE,
            )
        },
        &set_the_extent_offset_of_the_first_container_pointer,
    )
}

/// E1 谱系树那一处：固定脚本到回退 D 的流上谱系树根第一条条目里指向谱系容器的子指针头部 extent 偏移写 1、重封到根。
/// 池级 checker 走谱系树时判它，CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针（`read_container_tree_index_nodes`），
/// GPU 恢复内核 `read_container_tree_index_nodes` 同拒（`record_pointer_head_is_the_first_version` 连 extent 偏移一起判）。
fn record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
    tag: &str,
) -> RecordedFlow {
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-container-pointer-extent-offset-one",
        &|bytes| {
            is_a_container_tree_root_over_containers_of_record_type(
                bytes,
                PACKED_RECORD_TYPE_LINEAGE,
            )
        },
        &set_the_extent_offset_of_the_first_container_pointer,
    )
}

/// 一个第一版写者写不出的种类码（≥ 32）：GPU 恢复内核的种类位图只装得下 0..31，这一档逐条与前面各条比。
const TREE_KIND_NOT_BELOW_THIRTY_TWO: u16 = 40;

/// I-9.16 ③ 种类 ≥ 32 那一档（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第二行）：发布那张树表里
/// 稀疏旁表与 deadlist 两条（根指针都全零）的种类都改成 40——种类 40 两条、7 与 8 一条都没有。core 的只读恢复在读任何一棵树之前拒
/// （`tree_table_entries_each_kind_at_most_once` 比整个 u16），GPU 恢复内核对种类 ≥ 32 的条目逐条与前面比、同拒。
pub(super) fn record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-two-entries-of-kind-forty",
        &is_a_tree_table_unit,
        false,
        &|bytes| {
            for from_kind in [TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_DEADLIST] {
                let entry = only_tree_table_entry_of_kind(bytes, from_kind);
                let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
                assert_eq!(
                    bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
                    [0u8; NODE_POINTER_BYTES],
                    "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
                );
                let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
                bytes[kind..kind + 2]
                    .copy_from_slice(&TREE_KIND_NOT_BELOW_THIRTY_TWO.to_le_bytes());
            }
        },
    )
}

/// E1 墓碑树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset",
        &record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
            "crash-judge-gpu-i24-tombstone-container-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// E1 谱系树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset",
        &record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
            "crash-judge-gpu-i24-lineage-container-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// I-2.4：树表条目里 inode 树根指针的 MAC 第 0 字节非 0（core 解树表条目时拒、GPU 恢复内核同拒）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
            "crash-judge-gpu-i24-tree-table-root-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：inode 内部条目子指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
            "crash-judge-gpu-i24-inode-child-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：分配记录树内部条目子指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
            "crash-judge-gpu-i24-allocation-child-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶指着的下段根指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte",
        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
            "crash-judge-gpu-i24-lower-segment-root-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的算法类型写 1（core 解上段叶条目时拒、GPU 恢复内核同拒；池级 checker 的 CPU 与 GPU 走读内核
/// 判算法类型那一段）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-algorithm-type",
            PointerHeadFieldWrittenNonZero::AlgorithmType,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 下段叶记录的数据指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte",
        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
            "crash-judge-gpu-i24-lower-leaf-data-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：树表条目里码 2 节点的指针头部 extent 偏移非 0（池级 checker 与恢复两半都比）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
            "crash-judge-gpu-i24-node-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：根记录里实例表指针的头部 extent 偏移非 0（core 择根拒这条根）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset",
        &record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
            "crash-judge-gpu-i24-root-instance-table-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// I-2.4：inode 内部条目子指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
            "crash-judge-gpu-i24-inode-child-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：分配记录树内部条目子指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
            "crash-judge-gpu-i24-allocation-child-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶指着的下段根指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset",
        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
            "crash-judge-gpu-i24-lower-segment-root-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 下段叶记录的数据指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset",
        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
            "crash-judge-gpu-i24-lower-leaf-data-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}
