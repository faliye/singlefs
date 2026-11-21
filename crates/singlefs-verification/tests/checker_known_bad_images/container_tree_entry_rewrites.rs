//! 改容器树根条目的辅助：条目类型、身份字段、子指针与根节点字节，墓碑树、谱系树、inode 树几组用例都用。

use super::*;

/// 码 2 节点头里的层级（偏移 50）与 key 宽（偏移 51），条目区从 115 + 2 × key 宽 起（D18（块里携带什么信息） 已定项 18）；
/// 墓碑树、谱系树的根条目里类型段在条目偏移 16、身份引用的容器号在 18（分隔 key 8 + 出生树 8 + 类型段 2 之后，D8（核心索引结构） 已定项 6 的条目形态）。
pub(super) const INDEX_NODE_LEVEL_OFFSET: usize = 50;

pub(super) const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

pub(super) const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;

/// 容器树与 inode 树的码 2 节点条目宽（分隔 key 8 + 身份引用 26 + 子指针 86），身份引用里出生树在条目偏移 8、容器号与出生代在 18 起那 16 字节。
const INTERNAL_ENTRY_BYTES: usize = 120;

pub(super) const INTERNAL_ENTRY_IDENTITY_BIRTH_TREE: std::ops::Range<usize> = 8..16;

const INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH: std::ops::Range<usize> = 18..34;

/// 墓碑树、谱系树根长高之后根的条目类型段（指下一层码 2 节点）；墓碑容器、谱系容器的打包记录类型（D18（块里携带什么信息） 已定项 24）。
pub(super) const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;

pub(super) const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;

pub(super) const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;

pub(super) const PACKED_RECORD_TYPE_INODE: u16 = 2;

/// 码 3 单元头里容器号的偏移（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
pub(super) const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;

/// 最新的根那一版树表里种类 `kind` 那棵树（墓碑树、谱系树、inode 树）的码 2 根第 `entry_index` 条条目改一处（`change` 拿到那 120 字节），
/// 根重封、新的整单元校验和沿引用链补到根槽；交回根的层级。
pub(super) fn change_an_entry_of_the_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    entry_index: usize,
    change: impl FnOnce(&mut [u8]),
) -> u8 {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    let mut level_of_the_root = 0;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            level_of_the_root = root[INDEX_NODE_LEVEL_OFFSET];
            let start = 115
                + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET])
                + entry_index * INTERNAL_ENTRY_BYTES;
            change(&mut root[start..start + INTERNAL_ENTRY_BYTES]);
        },
    );
    level_of_the_root
}

pub(super) fn entry_type_of(entry: &[u8]) -> u16 {
    u16::from_le_bytes([
        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1],
    ])
}

/// 条目类型段从 `from` 改成 `to`（先认它是 `from`）。
pub(super) fn change_the_entry_type(entry: &mut [u8], from: u16, to: u16) {
    assert_eq!(entry_type_of(entry), from, "改之前条目类型段是 {from}");
    set_u16(entry, CONTAINER_TREE_ENTRY_TYPE_OFFSET, to);
}

/// 条目身份引用的出生树、容器号、出生代三段清零（类型段 0 的条目该有的样子），子指针不动。
pub(super) fn zero_the_three_identity_fields(entry: &mut [u8]) {
    entry[INTERNAL_ENTRY_IDENTITY_BIRTH_TREE].fill(0);
    entry[INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH].fill(0);
}

/// 判红的说明里点名孩子头里的类与类型段说的那一类不符（`walk` 按孩子头里的类标签定读多长）。
pub(super) fn assert_the_violation_names_the_child_class(
    image: &MemoryPool,
    invariant: &str,
    entry_type: u16,
    child_class_in_the_header: &str,
) {
    assert!(
        matches!(
            verdict(image, invariant),
            InvariantVerdict::Violated(detail)
                if detail.contains(&format!("类型段 {entry_type} 说孩子是"))
                    && detail.contains(&format!("孩子头里的类标签是 {child_class_in_the_header}"))
        ),
        "{invariant} 的违例说明写出类型段 {entry_type} 与孩子头里的类 {child_class_in_the_header}：{:?}",
        verdict(image, invariant)
    );
}

/// 码 2 节点与码 3 打包记录单元头里的类标签（单元头偏移 6）。
pub(super) const UNIT_CLASS_INDEX_NODE: u8 = 2;

pub(super) const UNIT_CLASS_PACKED_RECORD_UNIT: u8 = 3;

/// 单元头里类标签的偏移。
pub(super) const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;

/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根第 `entry_index` 条条目的子指针。
pub(super) fn child_pointer_of_an_entry_of_the_root_of_kind(
    image: &MemoryPool,
    kind: u16,
    entry_index: usize,
) -> singlefs_core::pointer::NodePointer {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("根解得开");
    singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
        &root.entries[entry_index],
        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
    ))
}

/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根（16 KiB 原样字节）：拿来当别处一个码 2 单元的陈旧副本。
pub(super) fn root_node_bytes_of_the_tree_of_kind(image: &MemoryPool, kind: u16) -> Vec<u8> {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    common_pointer_head_corruption::unit_bytes_at(
        image,
        common_pointer_head_corruption::the_slot_of(&root_pointer),
        common_pointer_head_corruption::node_bytes(),
    )
}

/// 一份副本陈旧：盘 `device` 上孩子的槽起写进 `stale_bytes`（别处一个单元的原样字节，不补任何校验和），另一块盘那一份不动。
/// 父条目位置条目里的整单元校验和还是孩子原来那一份的：陈旧那一份按哪一类的长度都自证不过。
pub(super) fn make_the_copy_on_one_device_stale(
    image: &mut MemoryPool,
    device: u32,
    child: &singlefs_core::pointer::NodePointer,
    stale_bytes: &[u8],
) {
    let slot = common_pointer_head_corruption::the_slot_of(child);
    write(image, device, slot.0 * SLOT, stale_bytes);
}
