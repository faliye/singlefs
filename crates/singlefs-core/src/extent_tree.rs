//! extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14「extent 树」，用户 2026-09-24 定 K2）：两段，都按位置寻址，不分裂。
//!
//! - **上段**按 inode 号的位置寻址：叶 j 罩 inode 号 `[j × U, (j + 1) × U)`，U = [`EXTENT_TREE_UPPER_LEAF_INODES`]；层级 L 上第 j 个节点罩
//!   `[j × U × F^L, (j + 1) × U × F^L)`，F = [`EXTENT_TREE_INTERNAL_FANOUT`]。上段的根是它最高那一层的第 0 个节点（罩从 inode 0 起的一段），
//!   层级是罩得住有条目的最大 inode 号的最低一层。上段叶条目（[`ExtentUpperLeafEntry`]，113 字节）带一个标签字节区分
//!   「没有单元（0）/ 下段根指针（1）/ 内联数据指针（2）」。
//! - **下段**一个文件一棵，按数据单元号的位置寻址：叶罩 [`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`] 个单元，层级 L 上一个节点罩 144 × F^L 个单元，
//!   根是罩得住这个文件全部单元的最低一层的第 0 个节点。叶的条目是 extent 叶记录 112（key (0, inode, 文件字节偏移) + 数据指针 88），
//!   内部条目是「孩子那一段的起点 key + 子指针」110。洞就是缺席：没有单元的一段不写节点，父节点那一格留空。
//! - **只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针（标签 2）。写路径只写标签 1 与 2（今天每个文件版本从偏移 0 顺序写、
//!   长度 0 的内容也写一个声明长度 0 的数据单元，一个单元都没有的文件写不出来）；标签 0 由读者与 checker 认（「这个 inode 没有单元」）。
//!
//! 节点头的 key 区间写这个节点按位置规定罩的那一段（D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句），key 宽恒是 24
//! （上段与下段同一棵树、同一个 key 宽，头宽 163）：上段节点罩 `[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`，
//! 下段节点罩 `[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`（P = 数据单元净荷容量，offset 段是文件字节偏移，D8 已定项 3）。
//!
//! 上段叶条目的字段表、上段一片叶罩几个 inode、上段扇出是实现员的取法（D8 已定项 14 交给实现员，交回里写明）。
//! 这里只管结构：几何、字节怎么装、从盘上读回来并逐节点核位置。取落点、发出生序号、写盘在 `crate::transaction` 那一侧。

use std::collections::BTreeSet;

use singlefs_format::{
    DATA_POINTER_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES,
    EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_INTERNAL_FANOUT,
    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
    EXTENT_TREE_UPPER_LEAF_INODES, NODE_POINTER_BYTES,
};

use crate::address::{CheckpointTxg, InstanceGeneration, SlotNumber, TreeIdentifier};
use crate::bytes::{ByteReader, ByteWriter};
use crate::pointer::{BirthSequence, DataPointer, NodePointer};
use crate::records::{build_extent_record, parse_extent_record};
use crate::recovery::RecoveryFailure;
use crate::root_record::RootRecord;
use crate::unit::{
    build_index_node, data_unit_payload_capacity, parse_index_node, IndexNodeHeader,
};

/// 上段叶条目的标签：这个 inode 没有数据单元（D8（核心索引结构） 已定项 14 的「没有单元」）。
pub const EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT: u8 = 0;
/// 上段叶条目的标签：载荷是这个文件下段根的节点指针。
pub const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
/// 上段叶条目的标签：载荷是这个文件唯一那个数据单元的数据指针（内联）。
pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;

fn key_width() -> usize {
    usize::try_from(EXTENT_KEY_BYTES).expect("24")
}

/// extent 树的 key：(locality 8, inode 8, offset 8)，盘上小端（D8（核心索引结构） 已定项 3）。第一版 locality 恒 0。
#[must_use]
pub fn extent_key_bytes(inode: u64, offset_in_bytes: u64) -> Vec<u8> {
    let mut writer = ByteWriter::new(key_width());
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset_in_bytes);
    writer.into_bytes()
}

/// 一把 extent key 的三段：(locality, inode, offset)。
#[must_use]
pub fn extent_key_fields(key: &[u8]) -> (u64, u64, u64) {
    let mut reader = ByteReader::at(key, 0);
    (reader.get_u64(), reader.get_u64(), reader.get_u64())
}

fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 上段层级 L 上一个节点罩几个 inode 号：U × F^L，乘到装不进 u64 就停在 u64 的最大值。
#[must_use]
pub fn upper_span_in_inodes(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// 下段层级 L 上一个节点罩几个数据单元：144 × F^L，乘到装不进 u64 就停在 u64 的最大值。
#[must_use]
pub fn lower_span_in_data_units(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// 上段罩得住 inode 号 `largest_inode` 的最低一层（上段根的层级）。
#[must_use]
pub fn upper_root_level_for(largest_inode: u64) -> u8 {
    (0..=u8::MAX)
        .find(|level| upper_span_in_inodes(*level) > largest_inode)
        .expect("第 8 层起罩的 inode 号已超过 u64 的上界")
}

/// 下段罩得住 `data_units` 个单元（单元号 0 .. data_units − 1）的最低一层（下段根的层级）。
#[must_use]
pub fn lower_root_level_for(data_units: u64) -> u8 {
    (0..=u8::MAX)
        .find(|level| lower_span_in_data_units(*level) >= data_units)
        .expect("第 8 层起罩的单元数已超过 u64 的上界")
}

/// 上段一个节点的位置：层级（0 是叶）与同层从 inode 0 起数第几个。派生的全序（层级升序、同层按序号升序 = 按 key 升序）
/// 是上段里的 bump 次序（先叶后根）；上段的根是最高那一层的第 0 个。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentUpperNodePosition {
    pub level: u8,
    pub index: u64,
}

impl ExtentUpperNodePosition {
    /// 这个位置罩的 inode 号：首个与末个。
    #[must_use]
    pub fn inode_range(self) -> (u64, u64) {
        let span = upper_span_in_inodes(self.level);
        let first = self.index.saturating_mul(span);
        (first, first.saturating_add(span - 1))
    }

    /// 节点头里写的 key 区间：`[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`。
    #[must_use]
    pub fn key_range(self) -> (Vec<u8>, Vec<u8>) {
        let (first_inode, last_inode) = self.inode_range();
        (
            extent_key_bytes(first_inode, 0),
            extent_key_bytes(last_inode, u64::MAX),
        )
    }

    /// 父节点（上一层）的位置。
    #[must_use]
    pub fn parent(self) -> Self {
        Self {
            level: self.level + 1,
            index: self.index / EXTENT_TREE_INTERNAL_FANOUT,
        }
    }
}

/// 上段从 inode `inode` 所在的叶到根（层级 `root_level` 的第 0 个）那一串位置，叶在前、根在末。
///
/// # Panics
/// `root_level` 罩不住这个 inode：调用方按 [`upper_root_level_for`] 取的层级。
#[must_use]
pub fn upper_path_of_inode(inode: u64, root_level: u8) -> Vec<ExtentUpperNodePosition> {
    assert!(
        upper_span_in_inodes(root_level) > inode,
        "上段根的层级按有条目的最大 inode 号取，罩得住它"
    );
    let mut path = vec![ExtentUpperNodePosition {
        level: 0,
        index: inode / EXTENT_TREE_UPPER_LEAF_INODES,
    }];
    // 迭代上界是根的层级：每一轮往上一层。
    while path.last().expect("至少叶那一个").level < root_level {
        let parent = path.last().expect("至少叶那一个").parent();
        path.push(parent);
    }
    path
}

/// 下段一个节点的位置（一个文件的下段里）：层级（0 是叶）与同层从单元 0 起数第几个。派生的全序是下段里的 bump 次序（先叶后根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentLowerNodePosition {
    pub level: u8,
    pub index: u64,
}

impl ExtentLowerNodePosition {
    /// 这个位置罩的数据单元号：首个与末个。
    #[must_use]
    pub fn data_unit_range(self) -> (u64, u64) {
        let span = lower_span_in_data_units(self.level);
        let first = self.index.saturating_mul(span);
        (first, first.saturating_add(span - 1))
    }

    /// 节点头里写的 key 区间：`[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`，乘到装不进 u64 就停在最大值。
    #[must_use]
    pub fn key_range(self, inode: u64) -> (Vec<u8>, Vec<u8>) {
        let (first_unit, last_unit) = self.data_unit_range();
        let payload = payload_capacity_in_bytes();
        let first_offset = first_unit.saturating_mul(payload);
        let last_offset = last_unit
            .saturating_add(1)
            .saturating_mul(payload)
            .saturating_sub(1);
        (
            extent_key_bytes(inode, first_offset),
            extent_key_bytes(inode, last_offset),
        )
    }

    /// 这个位置下面那一层的孩子里，罩着单元号 `data_unit` 的那一个。
    #[must_use]
    pub fn child_holding_data_unit(self, data_unit: u64) -> Self {
        Self {
            level: self.level - 1,
            index: data_unit / lower_span_in_data_units(self.level - 1),
        }
    }
}

/// 一个 `data_units` 个单元（单元号 0 .. data_units − 1、没有洞）的文件，下段的全部节点，按 bump 次序（先叶后根、同层按位置升序）。
/// 只有一个单元的文件不建下段（内联），交回空的。
#[must_use]
pub fn lower_segment_nodes_of_a_file_without_holes(
    data_units: u64,
) -> Vec<ExtentLowerNodePosition> {
    if data_units <= 1 {
        return Vec::new();
    }
    let root_level = lower_root_level_for(data_units);
    let mut nodes = Vec::new();
    for level in 0..=root_level {
        let nodes_in_this_level = data_units.div_ceil(lower_span_in_data_units(level));
        nodes
            .extend((0..nodes_in_this_level).map(|index| ExtentLowerNodePosition { level, index }));
    }
    nodes
}

/// 下段一个内部节点这一版的孩子（没有洞的文件里）：罩得着单元 0 .. data_units − 1 的那几个，按位置升序。
#[must_use]
pub fn lower_children_of_a_file_without_holes(
    parent: ExtentLowerNodePosition,
    data_units: u64,
) -> Vec<ExtentLowerNodePosition> {
    let child_level = parent.level - 1;
    let child_span = lower_span_in_data_units(child_level);
    let (first_unit, last_unit) = parent.data_unit_range();
    let last_unit_with_data = last_unit.min(data_units - 1);
    (first_unit / child_span..=last_unit_with_data / child_span)
        .map(|index| ExtentLowerNodePosition {
            level: child_level,
            index,
        })
        .collect()
}

/// 上段叶条目的载荷：标签 0 / 1 / 2 三种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentUpperLeafTarget {
    NoDataUnit,
    LowerSegmentRoot(NodePointer),
    InlineDataUnit(DataPointer),
}

/// 上段叶条目 113：key 24（0, inode, 0）+ 标签 1 + 载荷 88（[`EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtentUpperLeafEntry {
    pub inode: u64,
    pub target: ExtentUpperLeafTarget,
}

/// 一条上段叶条目解不开：盘上读来的字节可以是任何值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentUpperLeafEntryMalformed {
    NarrowerThanItsFieldTable,
    /// 标签不是 0 / 1 / 2。
    UnrecognizedTag(u8),
    /// key 的 locality 段或 offset 段不是 0，或载荷里该是零的字节不是零。
    NonZeroBytesWhereTheFieldTableSaysZero,
}

impl ExtentUpperLeafEntry {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer =
            ByteWriter::new(usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113"));
        writer.put(&extent_key_bytes(self.inode, 0));
        match self.target {
            ExtentUpperLeafTarget::NoDataUnit => {
                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT);
                writer.skip(usize::try_from(DATA_POINTER_BYTES).expect("88"));
            }
            ExtentUpperLeafTarget::LowerSegmentRoot(pointer) => {
                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT);
                pointer.write_to(&mut writer);
                writer.skip(usize::try_from(DATA_POINTER_BYTES - NODE_POINTER_BYTES).expect("2"));
            }
            ExtentUpperLeafTarget::InlineDataUnit(pointer) => {
                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT);
                pointer.write_to(&mut writer);
            }
        }
        writer.assert_position(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, "extent 上段叶条目");
        writer.into_bytes()
    }

    /// 上段叶条目的读者：这里是盘上字节进字段表的边界。
    ///
    /// # Errors
    /// 窄于 113、标签不认识、该是零的字节不是零。
    pub fn parse(bytes: &[u8]) -> Result<Self, ExtentUpperLeafEntryMalformed> {
        if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113") {
            return Err(ExtentUpperLeafEntryMalformed::NarrowerThanItsFieldTable);
        }
        let (locality, inode, offset) = extent_key_fields(&bytes[..key_width()]);
        if locality != 0 || offset != 0 {
            return Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero);
        }
        let tag = bytes[key_width()];
        let payload = &bytes
            [key_width() + 1..usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")];
        let node_pointer_bytes = usize::try_from(NODE_POINTER_BYTES).expect("86");
        let target = match tag {
            EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT => {
                if payload.iter().any(|byte| *byte != 0) {
                    return Err(
                        ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero,
                    );
                }
                ExtentUpperLeafTarget::NoDataUnit
            }
            EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT => {
                if payload[node_pointer_bytes..].iter().any(|byte| *byte != 0) {
                    return Err(
                        ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero,
                    );
                }
                ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer::read_from(
                    &mut ByteReader::at(payload, 0),
                ))
            }
            EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT => ExtentUpperLeafTarget::InlineDataUnit(
                DataPointer::read_from(&mut ByteReader::at(payload, 0)),
            ),
            unrecognized => {
                return Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(unrecognized))
            }
        };
        Ok(Self { inode, target })
    }
}

/// 内部节点条目 110：孩子那一段的起点 key 24 + 子指针 86。
#[must_use]
pub fn build_extent_internal_entry(child_start_key: &[u8], child: &NodePointer) -> Vec<u8> {
    let mut writer =
        ByteWriter::new(usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"));
    writer.put(child_start_key);
    child.write_to(&mut writer);
    writer.into_bytes()
}

/// 装 extent 树一个节点时整次发布共用的身份字段。
#[derive(Clone, Copy, Debug)]
pub struct ExtentTreeNodeIdentity<'identity> {
    pub tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: &'identity [u8; 16],
    pub instance: InstanceGeneration,
}

/// 装下段一片叶：这片叶罩的那几个单元的 extent 叶记录（key 升序）。`data_pointers` 是这个文件全部单元的指针（第 i 项是单元 i）。
///
/// # Panics
/// 这片叶罩的单元一个都不在这个文件里：规划只交出罩着单元的叶。
#[must_use]
pub fn build_lower_leaf(
    identity: &ExtentTreeNodeIdentity<'_>,
    inode: u64,
    position: ExtentLowerNodePosition,
    data_pointers: &[DataPointer],
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let (first_unit, last_unit) = position.data_unit_range();
    let payload = payload_capacity_in_bytes();
    let records: Vec<Vec<u8>> = data_pointers
        .iter()
        .enumerate()
        .map(|(unit, pointer)| (u64::try_from(unit).expect("单元序号"), pointer))
        .filter(|(unit, _)| (first_unit..=last_unit).contains(unit))
        .map(|(unit, pointer)| build_extent_record(inode, unit * payload, *pointer))
        .collect();
    assert!(!records.is_empty(), "规划只交出罩着单元的叶");
    let (smallest_key, largest_key) = position.key_range(inode);
    build_index_node(
        identity.tree,
        0,
        key_width(),
        &smallest_key,
        &largest_key,
        identity.birth_txg,
        identity.filesystem_identifier,
        identity.instance,
        birth_sequence,
        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
        &records,
    )
}

/// 装下段一个内部节点：每个孩子一条「孩子那一段的起点 key + 孩子这一版的指针」。
///
/// # Panics
/// 一个孩子都没有：规划只交出罩着单元的节点。
#[must_use]
pub fn build_lower_internal_node(
    identity: &ExtentTreeNodeIdentity<'_>,
    inode: u64,
    position: ExtentLowerNodePosition,
    children: &[(ExtentLowerNodePosition, NodePointer)],
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    assert!(!children.is_empty(), "规划只交出罩着单元的节点");
    let entries: Vec<Vec<u8>> = children
        .iter()
        .map(|(child, pointer)| build_extent_internal_entry(&child.key_range(inode).0, pointer))
        .collect();
    let (smallest_key, largest_key) = position.key_range(inode);
    build_index_node(
        identity.tree,
        position.level,
        key_width(),
        &smallest_key,
        &largest_key,
        identity.birth_txg,
        identity.filesystem_identifier,
        identity.instance,
        birth_sequence,
        u16::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"),
        &entries,
    )
}

/// 装上段一片叶：条目按 inode 号升序。
///
/// # Panics
/// 一条条目都没有，或有条目的 inode 号不在这片叶罩的那一段里：规划只把条目放进罩着它的那片叶。
#[must_use]
pub fn build_upper_leaf(
    identity: &ExtentTreeNodeIdentity<'_>,
    position: ExtentUpperNodePosition,
    entries: &[ExtentUpperLeafEntry],
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let (first_inode, last_inode) = position.inode_range();
    assert!(
        !entries.is_empty()
            && entries
                .iter()
                .all(|entry| (first_inode..=last_inode).contains(&entry.inode)),
        "上段叶里的条目都落在它罩的那一段 inode 号里"
    );
    let (smallest_key, largest_key) = position.key_range();
    build_index_node(
        identity.tree,
        0,
        key_width(),
        &smallest_key,
        &largest_key,
        identity.birth_txg,
        identity.filesystem_identifier,
        identity.instance,
        birth_sequence,
        u16::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113"),
        &entries
            .iter()
            .map(ExtentUpperLeafEntry::to_bytes)
            .collect::<Vec<_>>(),
    )
}

/// 装上段一个内部节点：每个孩子一条「孩子那一段的起点 key + 孩子这一版的指针」。
///
/// # Panics
/// 一个孩子都没有：规划只交出有条目的叶的祖先。
#[must_use]
pub fn build_upper_internal_node(
    identity: &ExtentTreeNodeIdentity<'_>,
    position: ExtentUpperNodePosition,
    children: &[(ExtentUpperNodePosition, NodePointer)],
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    assert!(!children.is_empty(), "规划只交出有条目的叶的祖先");
    let entries: Vec<Vec<u8>> = children
        .iter()
        .map(|(child, pointer)| build_extent_internal_entry(&child.key_range().0, pointer))
        .collect();
    let (smallest_key, largest_key) = position.key_range();
    build_index_node(
        identity.tree,
        position.level,
        key_width(),
        &smallest_key,
        &largest_key,
        identity.birth_txg,
        identity.filesystem_identifier,
        identity.instance,
        birth_sequence,
        u16::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"),
        &entries,
    )
}

/// 一版 extent 树在内存里的样子（第一版只有第一个文件有内容，下段只有它那一棵）：上段每个节点与它这一版的指针（bump 次序，根在最末）、
/// 第一个文件下段每个节点与它这一版的指针（bump 次序；内联时为空）。节点的字节住 `transaction::TransactionOutput::units`。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtentTreeVersion {
    pub upper_nodes: Vec<(ExtentUpperNodePosition, NodePointer)>,
    pub lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)>,
}

impl ExtentTreeVersion {
    /// 上段根的位置（最高那一层的第 0 个）。
    ///
    /// # Panics
    /// 上段一个节点都没有：带文件的一版里第一个文件恒有一条上段叶条目。
    #[must_use]
    pub fn upper_root(&self) -> ExtentUpperNodePosition {
        self.upper_nodes.last().expect("带文件的一版里上段恒有根").0
    }

    /// 上段的高（上段根的层级 + 1，D8（核心索引结构） 已定项 11 ⑤ 的读法）。
    #[must_use]
    pub fn upper_height(&self) -> u64 {
        u64::from(self.upper_root().level) + 1
    }

    /// 第一个文件下段的高；内联（没有下段）时 0。
    #[must_use]
    pub fn lower_height(&self) -> u64 {
        self.lower_nodes
            .last()
            .map_or(0, |(position, _)| u64::from(position.level) + 1)
    }

    #[must_use]
    pub fn upper_pointer_of(&self, position: ExtentUpperNodePosition) -> Option<NodePointer> {
        self.upper_nodes
            .iter()
            .find(|(candidate, _)| *candidate == position)
            .map(|(_, pointer)| *pointer)
    }

    #[must_use]
    pub fn lower_pointer_of(&self, position: ExtentLowerNodePosition) -> Option<NodePointer> {
        self.lower_nodes
            .iter()
            .find(|(candidate, _)| *candidate == position)
            .map(|(_, pointer)| *pointer)
    }
}

/// 从盘上读 extent 树时核到哪一步（同分配记录树那两档）：**位置那几样两档都核**——层级、节点头的 key 区间是位置规定的那一段、
/// 内部条目的 key 是一个孩子那一段的起点、按位置严格递增、叶条目落在叶罩的那一段里、按 key 严格递增。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentTreeHeaderJudgement {
    /// 另核每个节点头的树 ID、fsid、诞生不晚于根、出生序号与指着它的指针相同。
    EveryHeaderAgainstItsReference,
    /// 头里的出生身份不核（从盘上重建上一版）。
    OnlyWhatThePositionsNeed,
}

/// 读 extent 树时一路不变的那几样。
#[derive(Clone, Copy)]
pub struct ExtentTreeReading<'reading> {
    pub tree: TreeIdentifier,
    pub judgement: ExtentTreeHeaderJudgement,
    pub root: &'reading RootRecord,
    pub expected_filesystem_identifier: u64,
}

fn violated(detail: &'static str) -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-1.1",
        detail,
    }
}

impl ExtentTreeReading<'_> {
    /// 读一个节点、核头与位置：交回解开的头。`level_and_key_range` 是这个位置的层级与它按位置规定罩的那一段；
    /// `already_read` 是调用方为了先认出根的层级已经读过一遍的字节（根那一个），不再读第二次；
    /// `already_read_is_the_root_of_the_tree` 说这是不是整棵树的根（上段的根）。
    fn read_and_judge_node(
        &self,
        pointer: &NodePointer,
        level_and_key_range: (u8, &(Vec<u8>, Vec<u8>)),
        already_read: Option<Vec<u8>>,
        already_read_is_the_root_of_the_tree: bool,
        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
        units_seen: &mut BTreeSet<SlotNumber>,
    ) -> Result<(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
        let (expected_level, key_range) = level_and_key_range;
        if !units_seen.insert(pointer.locations[0].slot) {
            return Err(violated("同一个 extent 树节点被两条父条目引用"));
        }
        let bytes = match already_read {
            Some(bytes) => bytes,
            None => read_node(pointer)?,
        };
        // 整棵树的根（上段的根，树表条目指着它）那一个的报错逐字沿用 `recovery` 核树根的那几句（同 `code_two_tree` 的做法）。
        let is_the_root_of_the_tree = already_read_is_the_root_of_the_tree;
        let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
            if is_the_root_of_the_tree {
                of_the_root
            } else {
                of_a_node_below_the_root
            }
        };
        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: detail("树根节点", "extent 树节点"),
        })?;
        match self.judgement {
            ExtentTreeHeaderJudgement::OnlyWhatThePositionsNeed => {}
            ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference => {
                if header.tree != self.tree {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "I-1.3",
                        detail: detail(
                            "根头里的树 ID 与树表不符",
                            "extent 树节点头里的树 ID 与引用它的树不符",
                        ),
                    });
                }
                if header.key_width != key_width() {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "E142 走读同款",
                        detail: detail(
                            "根自述 key 宽与树的种类不符",
                            "extent 树节点自述 key 宽不是 24",
                        ),
                    });
                }
                if header.birth_txg > self.root.checkpoint_txg
                    || header.instance > self.root.instance
                {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "I-1.2",
                        detail: detail("树根诞生于根之后", "extent 树节点诞生于根之后"),
                    });
                }
                if header.filesystem_identifier != self.expected_filesystem_identifier {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "I-1.4",
                        detail: detail("树根 fsid 不符", "extent 树节点 fsid 不符"),
                    });
                }
                if header.birth_sequence != pointer.birth_sequence {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "I-1.2",
                        detail: detail("树根出生序号与指针不符", "extent 树节点出生序号与指针不符"),
                    });
                }
            }
        }
        if header.key_width != key_width() {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "extent 树节点自述 key 宽不是 24",
            });
        }
        if header.level != expected_level {
            return Err(violated("extent 树节点的层级不是它的位置规定的那一层"));
        }
        if header.smallest_key != key_range.0 || header.largest_key != key_range.1 {
            return Err(violated(
                "extent 树节点头里的 key 区间不是它的位置规定罩的那一段",
            ));
        }
        if header.entries.is_empty() {
            return Err(violated(
                "extent 树节点一条条目都没有（缺席的一段不写节点）",
            ));
        }
        Ok((bytes, header))
    }

    /// 解一个内部节点的条目：每条的 key 是一个孩子那一段的起点，孩子按位置严格递增。交回 (孩子那一段的起点 key, 子指针)。
    fn internal_entries(
        header: &IndexNodeHeader,
    ) -> Result<Vec<(Vec<u8>, NodePointer)>, RecoveryFailure> {
        let internal_width = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
        if header.entry_width < internal_width {
            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "extent 树内部条目",
                entry_bytes: header.entry_width,
                field_table_bytes: internal_width,
            });
        }
        Ok(header
            .entries
            .iter()
            .map(|entry| {
                (
                    entry[..key_width()].to_vec(),
                    NodePointer::read_from(&mut ByteReader::at(entry, key_width())),
                )
            })
            .collect())
    }
}

/// 从盘上读回来的一个文件的下段：每个节点、它的指针与字节（bump 次序），与全部数据指针（按单元号升序，带单元号）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentLowerSegmentReadFromDisk {
    pub nodes: Vec<(ExtentLowerNodePosition, NodePointer, Vec<u8>)>,
    pub data_pointers: Vec<(u64, DataPointer)>,
}

/// 从下段根指针往下读一个文件的整个下段，逐节点按位置核：根的层级由根节点头自述、它罩的那一段要是从单元 0 起的那一格；
/// 每个节点头的 key 区间是它的位置规定的那一段；内部条目的 key 是一个孩子那一段的起点（inode 段是这个文件、offset 段落在孩子那一层的格点上、
/// 在父节点那一段里）、按位置严格递增；孩子的层级是父层级减一；叶里的记录 inode 段是这个文件、offset 段是单元号 × 净荷容量、
/// 落在叶罩的那一段里、按 key 严格递增；同一个单元不被两条父条目引用。
///
/// # Errors
/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；条目窄于字段表 ⇒ `EntryNarrowerThanItsFieldTable`；别的核不过 ⇒ `InvariantViolated`。
pub fn read_lower_segment(
    reading: &ExtentTreeReading<'_>,
    inode: u64,
    root_pointer: &NodePointer,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
    units_seen: &mut BTreeSet<SlotNumber>,
) -> Result<ExtentLowerSegmentReadFromDisk, RecoveryFailure> {
    // 根的层级只有它自己的头说得出：先读出来认层级，再按那一层第 0 格核它（字节交下去，不读第二次）。
    let bytes = read_node(root_pointer)?;
    let root_level = parse_index_node(&bytes)
        .map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "extent 树下段根",
        })?
        .level;
    let mut segment = ExtentLowerSegmentReadFromDisk {
        nodes: Vec::new(),
        data_pointers: Vec::new(),
    };
    read_lower_node_and_its_subtree(
        reading,
        inode,
        ExtentLowerNodePosition {
            level: root_level,
            index: 0,
        },
        root_pointer,
        Some(bytes),
        read_node,
        units_seen,
        &mut segment,
    )?;
    segment.nodes.sort_by_key(|(position, _, _)| *position);
    Ok(segment)
}

#[allow(
    clippy::too_many_arguments,
    reason = "读一棵子树要的：一路不变的那几样、哪个文件、哪个位置、指针、已经读过的字节、读节点的口子、见过的单元、读出来的东西"
)]
fn read_lower_node_and_its_subtree(
    reading: &ExtentTreeReading<'_>,
    inode: u64,
    position: ExtentLowerNodePosition,
    pointer: &NodePointer,
    already_read: Option<Vec<u8>>,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
    units_seen: &mut BTreeSet<SlotNumber>,
    segment: &mut ExtentLowerSegmentReadFromDisk,
) -> Result<(), RecoveryFailure> {
    let (bytes, header) = reading.read_and_judge_node(
        pointer,
        (position.level, &position.key_range(inode)),
        already_read,
        false,
        read_node,
        units_seen,
    )?;
    if position.level == 0 {
        let (first_unit, last_unit) = position.data_unit_range();
        let payload = payload_capacity_in_bytes();
        let mut previous_unit: Option<u64> = None;
        for entry in &header.entries {
            let (key, data_pointer) = parse_extent_record(entry).ok_or(
                RecoveryFailure::EntryNarrowerThanItsFieldTable {
                    what: "extent 叶记录",
                    entry_bytes: entry.len(),
                    field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
                },
            )?;
            let (locality, record_inode, offset) = extent_key_fields(&key);
            if locality != 0 || record_inode != inode || !offset.is_multiple_of(payload) {
                return Err(violated(
                    "extent 叶记录的 key 不是 (0, 这个文件, 单元号 × 净荷容量)",
                ));
            }
            let unit = offset / payload;
            if !(first_unit..=last_unit).contains(&unit)
                || previous_unit.is_some_and(|previous| previous >= unit)
            {
                return Err(violated(
                    "extent 叶记录不在叶罩的那一段里，或不按 key 严格递增",
                ));
            }
            previous_unit = Some(unit);
            segment.data_pointers.push((unit, data_pointer));
        }
        segment.nodes.push((position, *pointer, bytes));
        return Ok(());
    }
    let mut previous_child: Option<ExtentLowerNodePosition> = None;
    // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
    for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
        let (locality, child_inode, offset) = extent_key_fields(&child_start_key);
        let payload = payload_capacity_in_bytes();
        let child_span = lower_span_in_data_units(position.level - 1);
        let child_offset_span = child_span.saturating_mul(payload);
        if locality != 0 || child_inode != inode || !offset.is_multiple_of(child_offset_span) {
            return Err(violated(
                "extent 下段内部条目的 key 不是这个文件里一个孩子那一段的起点",
            ));
        }
        let child = ExtentLowerNodePosition {
            level: position.level - 1,
            index: offset / child_offset_span,
        };
        if child.data_unit_range().0 / lower_span_in_data_units(position.level) != position.index
            || previous_child.is_some_and(|previous| previous >= child)
        {
            return Err(violated(
                "extent 下段内部条目的孩子不在父节点那一段里，或不按位置严格递增",
            ));
        }
        previous_child = Some(child);
        read_lower_node_and_its_subtree(
            reading,
            inode,
            child,
            &child_pointer,
            None,
            read_node,
            units_seen,
            segment,
        )?;
    }
    segment.nodes.push((position, *pointer, bytes));
    Ok(())
}

/// 上段一个内部节点的条目指的孩子：条目 key 是孩子那一段的起点 (0, 首个 inode, 0)，首个 inode 落在孩子那一层的格点上、在父节点那一段里。
fn upper_child_named_by_entry_key(
    parent: ExtentUpperNodePosition,
    child_start_key: &[u8],
) -> Option<ExtentUpperNodePosition> {
    let (locality, first_inode, offset) = extent_key_fields(child_start_key);
    let child_level = parent.level.checked_sub(1)?;
    let child_span = upper_span_in_inodes(child_level);
    if locality != 0 || offset != 0 || !first_inode.is_multiple_of(child_span) {
        return None;
    }
    let child = ExtentUpperNodePosition {
        level: child_level,
        index: first_inode / child_span,
    };
    (child.parent() == parent).then_some(child)
}

/// 上段一片叶的条目：每条落在叶罩的那一段 inode 号里、按 inode 号严格递增。
fn upper_leaf_entries(
    position: ExtentUpperNodePosition,
    header: &IndexNodeHeader,
) -> Result<Vec<ExtentUpperLeafEntry>, RecoveryFailure> {
    let (first_inode, last_inode) = position.inode_range();
    let mut entries: Vec<ExtentUpperLeafEntry> = Vec::with_capacity(header.entries.len());
    for entry_bytes in &header.entries {
        let entry =
            ExtentUpperLeafEntry::parse(entry_bytes).map_err(|malformed| match malformed {
                ExtentUpperLeafEntryMalformed::NarrowerThanItsFieldTable => {
                    RecoveryFailure::EntryNarrowerThanItsFieldTable {
                        what: "extent 上段叶条目",
                        entry_bytes: entry_bytes.len(),
                        field_table_bytes: usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES)
                            .expect("113"),
                    }
                }
                ExtentUpperLeafEntryMalformed::UnrecognizedTag(_) => {
                    RecoveryFailure::InvariantViolated {
                        invariant: "E142 走读同款",
                        detail: "extent 上段叶条目的标签不是 0 / 1 / 2",
                    }
                }
                ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero => {
                    RecoveryFailure::InvariantViolated {
                        invariant: "E142 走读同款",
                        detail: "extent 上段叶条目里字段表写零的字节不是零",
                    }
                }
            })?;
        if !(first_inode..=last_inode).contains(&entry.inode)
            || entries
                .last()
                .is_some_and(|previous| previous.inode >= entry.inode)
        {
            return Err(violated(
                "extent 上段叶条目不在叶罩的那一段 inode 号里，或不按 inode 号严格递增",
            ));
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// 从盘上读回来的 extent 树上段：每个节点、它的指针与字节（bump 次序，根在最末），与全部叶条目（按 inode 号升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentUpperSegmentReadFromDisk {
    pub nodes: Vec<(ExtentUpperNodePosition, NodePointer, Vec<u8>)>,
    pub entries: Vec<ExtentUpperLeafEntry>,
}

/// 从 extent 树根指针往下读整个上段，逐节点按位置核（同下段那几样，inode 号代单元号）；叶条目解开、核标签。不读下段。
///
/// # Errors
/// 同 [`read_lower_segment`]；叶条目标签不认识、字段表写零的字节不是零 ⇒ `InvariantViolated`（E142 走读同款）。
pub fn read_upper_segment(
    reading: &ExtentTreeReading<'_>,
    root_pointer: &NodePointer,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
    units_seen: &mut BTreeSet<SlotNumber>,
) -> Result<ExtentUpperSegmentReadFromDisk, RecoveryFailure> {
    let bytes = read_node(root_pointer)?;
    let root_level = parse_index_node(&bytes)
        .map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "extent 树根",
        })?
        .level;
    let mut segment = ExtentUpperSegmentReadFromDisk {
        nodes: Vec::new(),
        entries: Vec::new(),
    };
    read_upper_node_and_its_subtree(
        reading,
        ExtentUpperNodePosition {
            level: root_level,
            index: 0,
        },
        root_pointer,
        Some(bytes),
        read_node,
        units_seen,
        &mut segment,
    )?;
    segment.nodes.sort_by_key(|(position, _, _)| *position);
    Ok(segment)
}

#[allow(
    clippy::too_many_arguments,
    reason = "读一棵子树要的：一路不变的那几样、哪个位置、指针、已经读过的字节、读节点的口子、见过的单元、读出来的东西"
)]
fn read_upper_node_and_its_subtree(
    reading: &ExtentTreeReading<'_>,
    position: ExtentUpperNodePosition,
    pointer: &NodePointer,
    already_read: Option<Vec<u8>>,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
    units_seen: &mut BTreeSet<SlotNumber>,
    segment: &mut ExtentUpperSegmentReadFromDisk,
) -> Result<(), RecoveryFailure> {
    let is_the_root_of_the_tree = already_read.is_some();
    let (bytes, header) = reading.read_and_judge_node(
        pointer,
        (position.level, &position.key_range()),
        already_read,
        is_the_root_of_the_tree,
        read_node,
        units_seen,
    )?;
    if position.level == 0 {
        segment
            .entries
            .extend(upper_leaf_entries(position, &header)?);
        segment.nodes.push((position, *pointer, bytes));
        return Ok(());
    }
    let mut previous_child: Option<ExtentUpperNodePosition> = None;
    // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
    for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
        let child = upper_child_named_by_entry_key(position, &child_start_key)
            .filter(|child| previous_child.is_none_or(|previous| previous < *child))
            .ok_or(violated(
                "extent 上段内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增",
            ))?;
        previous_child = Some(child);
        read_upper_node_and_its_subtree(
            reading,
            child,
            &child_pointer,
            None,
            read_node,
            units_seen,
            segment,
        )?;
    }
    segment.nodes.push((position, *pointer, bytes));
    Ok(())
}

/// 按需读（D8（核心索引结构） 已定项 14「挂载怎么读」：extent 树按需，打开文件时按位置走下去，用户 K4）：从上段根按 inode 号的位置
/// 走到罩着它的那片叶，交回它那一条叶条目与这一趟读了几个节点；那一段缺席（父节点那一格留空）或叶里没有这个 inode 交回 `None`。
/// 只读这一条路径上的节点，每个节点照样按位置核。
///
/// # Errors
/// 同 [`read_upper_segment`]。
pub fn find_upper_leaf_entry(
    reading: &ExtentTreeReading<'_>,
    root_pointer: &NodePointer,
    inode: u64,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<(Option<ExtentUpperLeafEntry>, u64), RecoveryFailure> {
    let mut units_seen = BTreeSet::new();
    let bytes = read_node(root_pointer)?;
    let root_level = parse_index_node(&bytes)
        .map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "extent 树根",
        })?
        .level;
    let mut position = ExtentUpperNodePosition {
        level: root_level,
        index: 0,
    };
    let mut pointer = *root_pointer;
    let mut already_read = Some(bytes);
    let mut nodes_read = 0u64;
    // 迭代上界是根的层级 + 1：每一轮往下一层，叶那一轮交回。
    loop {
        let is_the_root_of_the_tree = already_read.is_some();
        let (_, header) = reading.read_and_judge_node(
            &pointer,
            (position.level, &position.key_range()),
            already_read.take(),
            is_the_root_of_the_tree,
            read_node,
            &mut units_seen,
        )?;
        nodes_read += 1;
        let (first_inode, last_inode) = position.inode_range();
        if !(first_inode..=last_inode).contains(&inode) {
            return Ok((None, nodes_read));
        }
        if position.level == 0 {
            let entry = upper_leaf_entries(position, &header)?
                .into_iter()
                .find(|entry| entry.inode == inode);
            return Ok((entry, nodes_read));
        }
        let mut next = None;
        let mut previous_child: Option<ExtentUpperNodePosition> = None;
        for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
            let child = upper_child_named_by_entry_key(position, &child_start_key)
                .filter(|child| previous_child.is_none_or(|previous| previous < *child))
                .ok_or(violated(
                    "extent 上段内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增",
                ))?;
            previous_child = Some(child);
            let (child_first_inode, child_last_inode) = child.inode_range();
            if (child_first_inode..=child_last_inode).contains(&inode) {
                next = Some((child, child_pointer));
            }
        }
        let Some((child, child_pointer)) = next else {
            return Ok((None, nodes_read));
        };
        position = child;
        pointer = child_pointer;
    }
}

/// 一个文件在 extent 树里的样子：没有单元、内联的那一个单元、或下段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtentsOfAFileReadFromDisk {
    NoDataUnit,
    Inline(DataPointer),
    LowerSegment(ExtentLowerSegmentReadFromDisk),
}

impl ExtentsOfAFileReadFromDisk {
    /// 这个文件的数据指针，带单元号，按单元号升序。
    #[must_use]
    pub fn data_pointers(&self) -> Vec<(u64, DataPointer)> {
        match self {
            ExtentsOfAFileReadFromDisk::NoDataUnit => Vec::new(),
            ExtentsOfAFileReadFromDisk::Inline(pointer) => vec![(0, *pointer)],
            ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.data_pointers.clone(),
        }
    }
}

/// 从盘上读回来的整棵 extent 树：上段，与每条标签 1 的叶条目指的下段（按 inode 号升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentTreeReadFromDisk {
    pub upper: ExtentUpperSegmentReadFromDisk,
    pub files: Vec<(u64, ExtentsOfAFileReadFromDisk)>,
}

/// 从根指针往下读整棵 extent 树（冷走读、从盘上重建上一版）：上段整段，再按每条叶条目读那个文件的下段；每个节点按位置核，
/// 同一个单元不被两条父条目引用（上段与各文件的下段之间也不许共用）。
///
/// # Errors
/// 同 [`read_upper_segment`] 与 [`read_lower_segment`]。
pub fn read_extent_tree(
    reading: &ExtentTreeReading<'_>,
    root_pointer: &NodePointer,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<ExtentTreeReadFromDisk, RecoveryFailure> {
    let mut units_seen = BTreeSet::new();
    let upper = read_upper_segment(reading, root_pointer, read_node, &mut units_seen)?;
    let mut files = Vec::with_capacity(upper.entries.len());
    for entry in &upper.entries {
        let extents = match entry.target {
            ExtentUpperLeafTarget::NoDataUnit => ExtentsOfAFileReadFromDisk::NoDataUnit,
            ExtentUpperLeafTarget::InlineDataUnit(pointer) => {
                ExtentsOfAFileReadFromDisk::Inline(pointer)
            }
            ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) => {
                ExtentsOfAFileReadFromDisk::LowerSegment(read_lower_segment(
                    reading,
                    entry.inode,
                    &lower_root,
                    read_node,
                    &mut units_seen,
                )?)
            }
        };
        files.push((entry.inode, extents));
    }
    Ok(ExtentTreeReadFromDisk { upper, files })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 下段：两个单元就要建下段（根兼叶），144 个还是一片叶，145 个长出层级 1（两片叶加一个根）。
    #[test]
    fn a_lower_segment_grows_a_level_past_one_hundred_and_forty_four_data_units() {
        assert!(lower_segment_nodes_of_a_file_without_holes(1).is_empty());
        assert_eq!(
            lower_segment_nodes_of_a_file_without_holes(2),
            vec![ExtentLowerNodePosition { level: 0, index: 0 }]
        );
        assert_eq!(lower_segment_nodes_of_a_file_without_holes(144).len(), 1);
        assert_eq!(
            lower_segment_nodes_of_a_file_without_holes(145),
            vec![
                ExtentLowerNodePosition { level: 0, index: 0 },
                ExtentLowerNodePosition { level: 0, index: 1 },
                ExtentLowerNodePosition { level: 1, index: 0 },
            ]
        );
        assert_eq!(
            lower_children_of_a_file_without_holes(
                ExtentLowerNodePosition { level: 1, index: 0 },
                145
            ),
            vec![
                ExtentLowerNodePosition { level: 0, index: 0 },
                ExtentLowerNodePosition { level: 0, index: 1 },
            ]
        );
    }

    /// 上段：inode 1 落在第 0 片叶，上段根就是那片叶；inode 143 起要长一层。
    #[test]
    fn the_upper_segment_of_inode_one_is_a_single_leaf() {
        assert_eq!(upper_root_level_for(1), 0);
        assert_eq!(upper_root_level_for(142), 0);
        assert_eq!(upper_root_level_for(143), 1);
        assert_eq!(
            upper_path_of_inode(143, 1),
            vec![
                ExtentUpperNodePosition { level: 0, index: 1 },
                ExtentUpperNodePosition { level: 1, index: 0 },
            ]
        );
    }

    /// 上段叶条目 113 字节来回：三种标签各解回自己；不认识的标签、标签 1 的补齐非零都解不开。
    #[test]
    fn upper_leaf_entries_round_trip_and_refuse_an_unknown_tag() {
        let no_unit = ExtentUpperLeafEntry {
            inode: 7,
            target: ExtentUpperLeafTarget::NoDataUnit,
        };
        let lower = ExtentUpperLeafEntry {
            inode: 8,
            target: ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer::empty_root()),
        };
        for entry in [no_unit, lower] {
            let bytes = entry.to_bytes();
            assert_eq!(bytes.len(), 113);
            assert_eq!(ExtentUpperLeafEntry::parse(&bytes), Ok(entry));
        }
        let mut unknown = no_unit.to_bytes();
        unknown[24] = 3;
        assert_eq!(
            ExtentUpperLeafEntry::parse(&unknown),
            Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(3))
        );
        let mut padding = lower.to_bytes();
        padding[112] = 1;
        assert_eq!(
            ExtentUpperLeafEntry::parse(&padding),
            Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero)
        );
    }
}
