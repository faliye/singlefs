//! 树里的记录与条目：inode 记录 140、inode 内部条目 120、extent 叶记录 112、记账条目 34、映射条目 55、树表条目 200。
//! key 的全序按逐字段无符号整数、字段自左向右比较（D8（核心索引结构） 已定项 11）——小端存储不构成 memcmp 序，所以各给 sort key。

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_POINTER_BYTES,
    TOMBSTONE_RECORD_BYTES, TREE_TABLE_ENTRY_BYTES,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, TreeIdentifier,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::pointer::{
    BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
    PointerHeadFieldOutsideTheFirstVersion,
};
use crate::unit::{PackedIdentity, WriteOrder, UNIT_CLASS_DATA};

/// 统计量标签 = D5（快照 / 空间记账机制） 已定项 4 的行号（已定项 10 的登记表）。
pub const STATISTIC_ALLOCATED_BYTES: u16 = 1;
pub const STATISTIC_FREE_BYTES: u16 = 2;
pub const STATISTIC_UNRECLAIMABLE_BYTES: u16 = 3;
pub const STATISTIC_PENDING_DELETE_BYTES: u16 = 4;
pub const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
pub const STATISTIC_COMMITTED_RESERVATION_BYTES: u16 = 6;
pub const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
pub const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
pub const STATISTIC_INODE_WATERMARK: u16 = 12;
/// 不带设备维的统计量，设备段写保留值（D5（快照 / 空间记账机制） 已定项 10）。
pub const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
/// 第一版直落叶，seq 一律 1（D8（核心索引结构） 已定项 10）。
pub const ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF: u32 = 1;

/// inode 记录偏移 48 那个 `blocks` 字段的计量单位：512 字节一块（D8（核心索引结构） 已定项 6）。
pub const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;

/// 第一版没有目录，活着的文件 nlink 恒 1（D8（核心索引结构） 已定项 6「第一版的唯一文件取 inode 1，nlink = 1」）。
pub const NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES: u32 = 1;

/// unlink 的意图那次发布把记录写成 nlink = 0、size 不变（D8（核心索引结构） 已定项 16 第 1 条）：从那一版起读者答「没有文件」、写入口拒它。
pub const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0;

/// inode 记录 140（D8（核心索引结构） 已定项 6 的偏移表）；时间是发布参数，不取系统时钟（可复现）。
///
/// 偏移 48 的 `blocks` **不是字段**：条款定它 = ⌈`size` ÷ 512⌉（逻辑长度的块数，不表示分到的空间），
/// 写的时候由 `size` 现算，于是「`blocks` 与 `size` 对不上」的记录从写路径造不出来；读的时候不交出它——
/// 盘上那 8 字节与 `size` 一不一致归 checker 判（C480（inode 记录的 blocks 怎么算全仓没有条款））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InodeRecord {
    pub inode: u64,
    pub object_birth: CheckpointTxg,
    /// 偏移 36 那 4 字节（D8（核心索引结构） 已定项 16 第 4 条：`parse` 读它、`to_bytes` 写结构里的值）：
    /// 活着的是 [`NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES`]，删除中的是 [`NLINK_OF_AN_INODE_BEING_DELETED`]。
    pub nlink: u32,
    pub size: u64,
    pub change_count: u64,
    pub write_time_seconds: u64,
}

impl InodeRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INODE_RECORD_BYTES).expect("140"));
        writer.put_u64(self.inode);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(0); // locality_id：第一版无父目录取 0
        writer.put_u32(0o100_644); // mode
        writer.put_u32(0); // uid
        writer.put_u32(0); // gid
        writer.put_u32(self.nlink);
        writer.assert_position(40, "size");
        writer.put_u64(self.size);
        writer.put_u64(self.size.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)); // blocks = ⌈size ÷ 512⌉
        writer.put_u64(0); // rdev
        writer.assert_position(64, "时间秒");
        for _clock in 0..3 {
            writer.put_u64(self.write_time_seconds);
        }
        writer.assert_position(88, "改动计数");
        writer.put_u64(self.change_count);
        writer.assert_position(96, "时间纳秒");
        writer.skip(12);
        writer.assert_position(108, "填充 / flags / 预留");
        writer.skip(4 + 8 + 20);
        writer.assert_position(INODE_RECORD_BYTES, "inode 记录");
        writer.into_bytes()
    }
    /// 读者：三段填充 / flags / 预留（偏移 108 起）非零判不可用（I-9.7）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INODE_RECORD_BYTES).expect("140")
            || bytes[108..].iter().any(|byte| *byte != 0)
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        let inode = reader.get_u64();
        let object_birth = CheckpointTxg(reader.get_u64());
        reader.skip(8 + 4 + 4 + 4); // locality_id、mode、uid、gid
        let nlink = reader.get_u32();
        let size = reader.get_u64();
        reader.skip(8 + 8); // blocks（由 size 定，不交出）、rdev
        let write_time_seconds = reader.get_u64();
        reader.skip(16);
        let change_count = reader.get_u64();
        Some(Self {
            inode,
            object_birth,
            nlink,
            size,
            change_count,
            write_time_seconds,
        })
    }
}

/// 墓碑记录 40（D18（块里携带什么信息） 已定项 24 第 2 条；字段表在 `.claude/kb/layout/03-third-txn.md`「二、墓碑记录」）：
/// 对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8，小端。第一版只写整对象，区间恒是 [0, 2⁶⁴ − 1]
/// （[`TombstoneRecord::of_the_whole_object`]）；死亡 txg = 意图那次发布的 checkpoint_txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TombstoneRecord {
    pub object: InodeNumber,
    pub object_birth: CheckpointTxg,
    pub range_start_in_bytes: u64,
    pub range_end_in_bytes: u64,
    pub death_txg: CheckpointTxg,
}

impl TombstoneRecord {
    /// 整对象墓碑的区间：起 0、止 2⁶⁴ − 1（D18（块里携带什么信息） 已定项 24 第 2 条「整对象写 [0, 2⁶⁴ − 1]」）。
    pub const WHOLE_OBJECT_RANGE_START_IN_BYTES: u64 = 0;
    pub const WHOLE_OBJECT_RANGE_END_IN_BYTES: u64 = u64::MAX;

    /// 意图那次发布给 `object` 写的整对象墓碑。
    #[must_use]
    pub const fn of_the_whole_object(
        object: InodeNumber,
        object_birth: CheckpointTxg,
        death_txg: CheckpointTxg,
    ) -> Self {
        Self {
            object,
            object_birth,
            range_start_in_bytes: Self::WHOLE_OBJECT_RANGE_START_IN_BYTES,
            range_end_in_bytes: Self::WHOLE_OBJECT_RANGE_END_IN_BYTES,
            death_txg,
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(TOMBSTONE_RECORD_BYTES).expect("40"));
        writer.put_u64(self.object.0);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(self.range_start_in_bytes);
        writer.put_u64(self.range_end_in_bytes);
        writer.put_u64(self.death_txg.0);
        writer.assert_position(TOMBSTONE_RECORD_BYTES, "墓碑记录");
        writer.into_bytes()
    }

    /// 墓碑记录的读者：长度不是 40 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 40 再切）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(TOMBSTONE_RECORD_BYTES).expect("40") {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        Some(Self {
            object: InodeNumber(reader.get_u64()),
            object_birth: CheckpointTxg(reader.get_u64()),
            range_start_in_bytes: reader.get_u64(),
            range_end_in_bytes: reader.get_u64(),
            death_txg: CheckpointTxg(reader.get_u64()),
        })
    }
}

/// 谱系记录 16（D18（块里携带什么信息） 已定项 24 第 5 条；字段表在 `.claude/kb/layout/03-third-txn.md`「三、谱系记录」）：
/// T_rb 8 + T_old 8，小端，读作「T_rb 的父是 T_old」。管理员回退那次发布追加一条：T_rb 是回退那次发布的 checkpoint_txg，
/// T_old 是回退目标 R_old 的 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineageRecord {
    pub rollback_txg: CheckpointTxg,
    pub rollback_target_txg: CheckpointTxg,
}

impl LineageRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(LINEAGE_RECORD_BYTES).expect("16"));
        writer.put_u64(self.rollback_txg.0);
        writer.put_u64(self.rollback_target_txg.0);
        writer.assert_position(LINEAGE_RECORD_BYTES, "谱系记录");
        writer.into_bytes()
    }

    /// 谱系记录的读者：长度不是 16 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 16 再切）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(LINEAGE_RECORD_BYTES).expect("16") {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        Some(Self {
            rollback_txg: CheckpointTxg(reader.get_u64()),
            rollback_target_txg: CheckpointTxg(reader.get_u64()),
        })
    }
}

/// inode 树内部节点条目 120：分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）+ 子指针 86。
#[must_use]
pub fn build_inode_internal_entry(
    separator_key: u64,
    child_identity: PackedIdentity,
    child: NodePointer,
) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(INODE_INTERNAL_ENTRY).expect("120"));
    writer.put_u64(separator_key);
    writer.put_u64(child_identity.birth_tree.0);
    writer.put_u16(child_identity.record_type);
    writer.put_u64(child_identity.container);
    writer.put_u64(child_identity.container_birth.0);
    child.write_to(&mut writer);
    writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");
    writer.into_bytes()
}

/// extent 叶记录 112：key (locality_id 8, inode 8, offset 8) + 数据指针 88。
#[must_use]
pub fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"));
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    writer.into_bytes()
}

/// 记账条目 34：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountingEntry {
    pub statistic: u16,
    pub tree: TreeIdentifier,
    pub device: DeviceIdentity,
    pub generation: CheckpointTxg,
    pub value: u64,
    pub sequence: u32,
}

impl AccountingEntry {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"));
        writer.put_u16(self.statistic);
        writer.put_u64(self.tree.0);
        writer.put_u32(self.device.0);
        writer.put_u64(self.generation.0);
        writer.put_u64(self.value);
        writer.put_u32(self.sequence);
        writer.assert_position(ACCOUNTING_ENTRY_BYTES, "记账条目");
        writer.into_bytes()
    }
    #[must_use]
    pub fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..22].to_vec()
    }
    #[must_use]
    pub fn sort_key(&self) -> (u16, u64, u32, u64) {
        (
            self.statistic,
            self.tree.0,
            self.device.0,
            self.generation.0,
        )
    }
    /// 记账条目 34 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 22），
    /// 窄于 34 时返回 `None`：这里是盘上字节进字段表的边界，往里就按 34 信它。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34") {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        Some(Self {
            statistic: reader.get_u16(),
            tree: TreeIdentifier(reader.get_u64()),
            device: DeviceIdentity(reader.get_u32()),
            generation: CheckpointTxg(reader.get_u64()),
            value: reader.get_u64(),
            sequence: reader.get_u32(),
        })
    }
}

/// 中央映射 key 一律 27（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10）：码 1 = 类标签 + 出生树 + 出生 txg + 写序 10；
/// 码 2 / 码 3 = 类标签 + 出生树 + 出生 txg + 实例代号 4 + 出生序号 4，末尾补零 2。
#[must_use]
pub fn mapping_key_for_data(head: PointerHead, write_order: WriteOrder) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(head.birth_tree.0);
    writer.put_u64(head.birth_txg.0);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(MAPPING_KEY_BYTES, "码 1 映射 key");
    writer.into_bytes()
}

#[must_use]
pub fn mapping_key_for_node(unit_class: u8, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
    writer.put_u8(unit_class);
    writer.put_u64(pointer.head.birth_tree.0);
    writer.put_u64(pointer.head.birth_txg.0);
    writer.put_u32(pointer.instance.0);
    writer.put_u32(pointer.birth_sequence.0);
    writer.assert_position(25, "码 2 / 码 3 映射 key 里有含义的那一段");
    writer.skip(2);
    writer.into_bytes()
}

/// 类标签 → 出生树 → 出生 txg → 尾段（码 1 的写序、码 2 / 码 3 的实例代号与出生序号）。
#[must_use]
pub fn mapping_key_sort_key(key: &[u8]) -> (u8, u64, u64, u32, u64) {
    let mut reader = ByteReader::at(key, 0);
    let unit_class = reader.get_u8();
    let birth_tree = reader.get_u64();
    let birth_txg = reader.get_u64();
    let instance = reader.get_u32();
    let tail = if unit_class == UNIT_CLASS_DATA {
        reader.get_six_byte_unsigned()
    } else {
        u64::from(reader.get_u32())
    };
    (unit_class, birth_tree, birth_txg, instance, tail)
}

/// 映射条目 55：key 27 + 位置条目 14 × 2。
#[must_use]
pub fn build_mapping_entry(key: &[u8], locations: [LocationEntry; 2]) -> Vec<u8> {
    assert_eq!(key.len(), usize::try_from(MAPPING_KEY_BYTES).expect("27"));
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_ENTRY_BYTES).expect("55"));
    writer.put(key);
    for location in &locations {
        location.write_to(&mut writer);
    }
    writer.assert_position(MAPPING_ENTRY_BYTES, "映射条目");
    writer.into_bytes()
}

/// journal 点名项与映射 key 共用的 10 字节尾段。
#[must_use]
pub fn data_key_tail(write_order: WriteOrder) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&write_order.instance.0.to_le_bytes());
    tail[4..].copy_from_slice(&write_order.transaction.to_le_bytes()[..6]);
    tail
}

#[must_use]
pub fn node_key_tail(instance: InstanceGeneration, birth_sequence: BirthSequence) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&instance.0.to_le_bytes());
    tail[4..8].copy_from_slice(&birth_sequence.0.to_le_bytes());
    tail
}

/// 树的种类码（D8（核心索引结构） 已定项 9 的登记表）。
pub const TREE_KIND_EXTENT: u16 = 1;
pub const TREE_KIND_INODE: u16 = 2;
pub const TREE_KIND_ALLOCATION: u16 = 3;
pub const TREE_KIND_ACCOUNTING: u16 = 4;
pub const TREE_KIND_LIVELIST: u16 = 6;
pub const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
pub const TREE_KIND_DEADLIST: u16 = 8;
/// 第一个文件版本那次零根注册的四棵树的种类码：D8（核心索引结构） 已定项 9 登记表的码 9..12（只追加不重用码，接在 8 之后，
/// 次序照 D18（块里携带什么信息） 已定项 24 第 6 条列的）。
pub const TREE_KIND_DELETED_INODES: u16 = 9;
pub const TREE_KIND_TOMBSTONE: u16 = 10;
pub const TREE_KIND_LINEAGE: u16 = 11;
pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;

/// 树表条目里根指针的起点：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（`singlefs_format::TREE_TABLE_ENTRY_BYTES` 的字段表）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES: usize = 8 + 2 + 2 + 2;

/// 树表条目 200：树 ID 8（打头 = key）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeTableEntry {
    pub kind: u16,
    pub tree: TreeIdentifier,
    pub root: NodePointer,
    pub birth_txg: CheckpointTxg,
    /// D5（快照 / 空间记账机制） 已定项 9：这棵树归哪个可写头；无归属写 0。
    pub head_identifier: u64,
}

impl TreeTableEntry {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
        writer.put_u64(self.tree.0);
        writer.put_u16(u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
        writer.put_u16(self.kind);
        writer.put_u16(0);
        writer.assert_position(
            u64::try_from(TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES).expect("14"),
            "树表条目的根指针",
        );
        self.root.write_to(&mut writer);
        writer.put_u64(0); // previous_snapshot_txg：0 = 不适用
        writer.put_u64(self.birth_txg.0);
        writer.put_u64(self.head_identifier);
        writer.skip(76);
        writer.assert_position(TREE_TABLE_ENTRY_BYTES, "树表条目");
        writer.into_bytes()
    }
    /// 一条条目盘上字节里根指针那 86 字节的原样（逐字节比两条根指针用；解析成 `NodePointer` 会丢掉第一版恒 0 的 MAC / nonce 那几段）。
    /// 调用方要先用 `parse` 核过这是一条合法的 200 字节条目。
    #[must_use]
    pub fn root_pointer_bytes(entry_bytes: &[u8]) -> &[u8] {
        &entry_bytes[TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES
            ..TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES
                + usize::try_from(NODE_POINTER_BYTES).expect("86")]
    }
    /// 读者：条目长度、flags 未知位（D8（核心索引结构） 已定项 8 ㊁：一律拒收）、预留 76 非零、根指针头部有一段不是第一版的取值
    /// （MAC / nonce 非 0、算法类型不是未加密、压缩算法码不认识、码 0 而压后长度非 0、extent 偏移非 0：D19（块指针的结构与宽度预算） 已定项 3、
    /// D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据），判该指针所在的结构——这条条目——损坏）都判不可用。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
            || bytes[bytes.len() - 76..].iter().any(|byte| *byte != 0)
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        let tree = TreeIdentifier(reader.get_u64());
        if u64::from(reader.get_u16()) != TREE_TABLE_ENTRY_BYTES {
            return None;
        }
        let kind = reader.get_u16();
        if reader.get_u16() != 0 {
            return None;
        }
        let root =
            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            )
            .ok()?;
        let _previous_snapshot_txg = reader.get_u64();
        let birth_txg = CheckpointTxg(reader.get_u64());
        let head_identifier = reader.get_u64();
        Some(Self {
            kind,
            tree,
            root,
            birth_txg,
            head_identifier,
        })
    }
}

/// 一条带指针的条目（inode 内部条目 120、extent 叶记录 112）解不开是哪一种。封闭集合，`match` 不写通配臂。
/// 两种都是盘上读来的字节、不是我们的不变量：调用方把这条条目当损坏处置，不拿里面的指针去解引用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerCarryingEntryRefusal {
    /// 条目宽（索引节点头里的盘上字段）窄于字段表：调用方报「条目窄于字段表」。
    NarrowerThanItsFieldTable,
    /// 条目里那条指针的头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移）：判该指针所在的结构
    /// （这条条目）损坏（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
    PointerHeadFieldOutsideTheFirstVersion(PointerHeadFieldOutsideTheFirstVersion),
}

/// inode 内部条目 120 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 8），
/// 窄于字段表、或子指针头部有一段不是第一版的取值时返回 `None`：这里是盘上字节进字段表的边界，往里就按 120 信它。
/// 要分清是哪一种的调用方（冷启动走读、只读挂载打开池）用 [`parse_inode_internal_entry_or_refusal`]；只要知道解不解得开的（用例）用这一个。
#[must_use]
pub fn parse_inode_internal_entry(bytes: &[u8]) -> Option<(u64, PackedIdentity, NodePointer)> {
    parse_inode_internal_entry_or_refusal(bytes).ok()
}

/// 同 [`parse_inode_internal_entry`]，解不开时交回是哪一种。
///
/// # Errors
/// 窄于 120 ⇒ `NarrowerThanItsFieldTable`；子指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
pub fn parse_inode_internal_entry_or_refusal(
    bytes: &[u8],
) -> Result<(u64, PackedIdentity, NodePointer), PointerCarryingEntryRefusal> {
    if bytes.len() < usize::try_from(INODE_INTERNAL_ENTRY).expect("120") {
        return Err(PointerCarryingEntryRefusal::NarrowerThanItsFieldTable);
    }
    let mut reader = ByteReader::at(bytes, 0);
    let separator_key = reader.get_u64();
    let child_identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let child = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
        &mut reader,
    )
    .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;
    Ok((separator_key, child_identity, child))
}

/// extent 叶记录 112 的读者：key 24 + 指针 88（同 [`parse_inode_internal_entry_or_refusal`]：`parse_index_node` 只判了条目宽 ≥ key 宽 24，
/// 切到偏移 112 之前要在这里判一次；数据指针头部第一版之外的取值也在这里判）。
///
/// # Errors
/// 窄于 112 ⇒ `NarrowerThanItsFieldTable`；数据指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
pub fn parse_extent_record(
    bytes: &[u8],
) -> Result<([u8; 24], DataPointer), PointerCarryingEntryRefusal> {
    if bytes.len() < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {
        return Err(PointerCarryingEntryRefusal::NarrowerThanItsFieldTable);
    }
    let key: [u8; 24] = bytes[..24].try_into().expect("24");
    let pointer =
        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(bytes, 24),
        )
        .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;
    Ok((key, pointer))
}

/// 映射条目 55 的读者：key 27 + 位置条目 14 × 2。条目宽窄于 55 时返回 `None`
/// （同 [`parse_inode_internal_entry`] 与 [`parse_extent_record`]：`parse_index_node` 只判了条目宽 ≥ key 宽，
/// 切到偏移 27 与 55 之前要在这里判一次；panic 面普查 R2）。
///
/// **判的是「今天这个解析器要几个字节」，不判条目宽是不是正好 55**：key 一律 27（码 2 / 码 3 的 25 字节末尾补零）、
/// 条目一律 55 已由 D19（块指针的结构与宽度预算） 已定项 10 定，落在 [`MAPPING_KEY_BYTES`] 与 [`MAPPING_ENTRY_BYTES`] 上，
/// 这一判跟着那两个常量走，不自己写死一个宽度。宽于 55 的条目这里不拒，拒它欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
#[must_use]
pub fn parse_mapping_entry(bytes: &[u8]) -> Option<(Vec<u8>, [LocationEntry; 2])> {
    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {
        return None;
    }
    let key = bytes[..usize::try_from(MAPPING_KEY_BYTES).expect("27")].to_vec();
    let mut reader = ByteReader::at(bytes, key.len());
    Some((
        key,
        [
            LocationEntry::read_from(&mut reader),
            LocationEntry::read_from(&mut reader),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::SlotNumber;
    use singlefs_format::{
        TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
        TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
        TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
        TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    };

    /// 树的种类码与树 ID 的值域不相交（D8（核心索引结构） 已定项 9 射程、已定项 11，用户 2026-10-01 定案「树 ID 整体挪开」）：
    /// 核心层的每个种类码都小于 65536（树的种类那 2 字节取不到的最小值），每个树 ID 与两个水位都不小于它。
    #[test]
    fn every_tree_kind_code_is_below_every_tree_identifier() {
        let first_value_the_kind_field_cannot_hold = u64::from(u16::MAX) + 1;
        for kind in [
            TREE_KIND_EXTENT,
            TREE_KIND_INODE,
            TREE_KIND_ALLOCATION,
            TREE_KIND_ACCOUNTING,
            TREE_KIND_LIVELIST,
            TREE_KIND_SPARSE_SIDE_TABLE,
            TREE_KIND_DEADLIST,
            TREE_KIND_DELETED_INODES,
            TREE_KIND_TOMBSTONE,
            TREE_KIND_LINEAGE,
            TREE_KIND_LOGGED_OPERATIONS,
        ] {
            assert!(
                u64::from(kind) < first_value_the_kind_field_cannot_hold,
                "种类码 {kind} 小于 65536"
            );
        }
        for identifier in [
            TREE_IDENTIFIER_EXTENT,
            TREE_IDENTIFIER_INODE,
            TREE_IDENTIFIER_ALLOCATION_RECORDS,
            TREE_IDENTIFIER_ACCOUNTING,
            TREE_IDENTIFIER_CENTRAL_MAPPING,
            TREE_IDENTIFIER_LIVELIST,
            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
            TREE_IDENTIFIER_DEADLIST,
            TREE_IDENTIFIER_DELETED_INODES,
            TREE_IDENTIFIER_TOMBSTONE,
            TREE_IDENTIFIER_LINEAGE,
            TREE_IDENTIFIER_LOGGED_OPERATIONS,
            TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        ] {
            assert!(
                identifier >= first_value_the_kind_field_cannot_hold,
                "树 ID（或水位）{identifier} 不小于 65536：与种类码的值域不相交"
            );
        }
    }

    #[test]
    fn widths_of_every_record_match_the_byte_table() {
        let record = InodeRecord {
            inode: 1,
            object_birth: CheckpointTxg(3),
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 3000,
            // 新池新建文件那条记录的取值：改动计数 = 这次发布的 checkpoint_txg 3（增补 2 第 11 行）。
            change_count: 3,
            write_time_seconds: 1_788_000_000,
        };
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 140);
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
            0o100_644,
            "mode 在偏移 24"
        );
        assert_eq!(
            u64::from_le_bytes(bytes[40..48].try_into().expect("8")),
            3000,
            "size 在偏移 40"
        );
        assert_eq!(
            u64::from_le_bytes(bytes[48..56].try_into().expect("8")),
            6,
            "blocks 在偏移 48：3000 字节 ⇒ ⌈3000 ÷ 512⌉ = 6（D8（核心索引结构） 已定项 6）"
        );
        assert_eq!(InodeRecord::parse(&bytes), Some(record));
        let mut tampered = bytes.clone();
        tampered[139] = 1;
        assert_eq!(InodeRecord::parse(&tampered), None, "预留段非零拒收");
        let pointer = NodePointer::empty_root();
        let identity = PackedIdentity {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
            record_type: 2,
            container: 1,
            container_birth: CheckpointTxg(3),
        };
        assert_eq!(build_inode_internal_entry(1, identity, pointer).len(), 120);
        let data_pointer = DataPointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                birth_txg: CheckpointTxg(3),
            },
            locations: [
                LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50180),
                    unit_checksum: 1,
                },
                LocationEntry {
                    device: DeviceIdentity(1),
                    slot: SlotNumber(50180),
                    unit_checksum: 1,
                },
            ],
            write_order: WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 1,
            },
        };
        assert_eq!(build_extent_record(1, 0, data_pointer).len(), 112);
        assert_eq!(
            parse_extent_record(&build_extent_record(1, 0, data_pointer))
                .expect("满宽的 112 字节记录解得开")
                .1,
            data_pointer
        );
        assert_eq!(
            parse_inode_internal_entry(&build_inode_internal_entry(1, identity, pointer)),
            Some((1, identity, pointer))
        );
        let entry = AccountingEntry {
            statistic: 1,
            tree: TreeIdentifier(0),
            device: DeviceIdentity(0),
            generation: CheckpointTxg(3),
            value: 212_992,
            sequence: 1,
        };
        assert_eq!(entry.to_bytes().len(), 34);
        assert_eq!(AccountingEntry::parse(&entry.to_bytes()), Some(entry));
        let key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
        assert_eq!(key.len(), 27);
        assert_eq!(build_mapping_entry(&key, data_pointer.locations).len(), 55);
        assert_eq!(
            parse_mapping_entry(&build_mapping_entry(&key, data_pointer.locations)),
            Some((key.clone(), data_pointer.locations))
        );
        assert_eq!(
            &key[17..27],
            &data_key_tail(data_pointer.write_order),
            "映射 key 的尾段就是点名项的尾段"
        );
        let node_key = mapping_key_for_node(2, pointer);
        assert_eq!(&node_key[25..], &[0, 0], "码 2 的 key 末尾补零到 27");
        let table_entry = TreeTableEntry {
            kind: TREE_KIND_EXTENT,
            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            root: pointer,
            birth_txg: CheckpointTxg(3),
            head_identifier: TREE_IDENTIFIER_INODE,
        };
        assert_eq!(table_entry.to_bytes().len(), 200);
        assert_eq!(
            TreeTableEntry::parse(&table_entry.to_bytes()),
            Some(table_entry)
        );
    }

    /// 墓碑记录五个字段落在 `.claude/kb/layout/03-third-txn.md`「二、墓碑记录」那张表的偏移上，读回来逐字段相等。
    #[test]
    fn tombstone_record_fields_sit_at_the_offsets_of_the_layout_table_and_read_back() {
        let record = TombstoneRecord::of_the_whole_object(
            InodeNumber(7),
            CheckpointTxg(5),
            CheckpointTxg(9),
        );
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 40, "墓碑记录 40 字节（D18 已定项 24 第 2 条）");
        let field_at = |offset: usize| {
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
        };
        assert_eq!(field_at(0), 7, "偏移 0：对象 ID = 被删的 inode 号");
        assert_eq!(field_at(8), 5, "偏移 8：对象出生代");
        assert_eq!(field_at(16), 0, "偏移 16：区间起 0（整对象）");
        assert_eq!(field_at(24), u64::MAX, "偏移 24：区间止 2⁶⁴ − 1（整对象）");
        assert_eq!(field_at(32), 9, "偏移 32：死亡 txg = 意图那次发布的 txg");
        assert_eq!(
            TombstoneRecord::parse(&bytes),
            Some(record),
            "读回来逐字段相等"
        );
        assert_eq!(
            TombstoneRecord::parse(&bytes[..39]),
            None,
            "窄于 40 字节不解"
        );
    }

    /// D8（核心索引结构） 已定项 16 第 4 条：nlink 由 `to_bytes` 照结构里的值写进偏移 36、由 `parse` 读回来——
    /// 删除中的记录（nlink = 0，size 不变）写出去读回来还是 0，读者与写入口才认得出它；活着的写 1。
    #[test]
    fn nlink_is_written_from_the_record_and_read_back_at_offset_thirty_six() {
        let record_of_nlink = |nlink: u32| InodeRecord {
            inode: 7,
            object_birth: CheckpointTxg(5),
            nlink,
            size: 40_000,
            change_count: 6,
            write_time_seconds: 1_788_000_000,
        };
        for nlink in [
            NLINK_OF_AN_INODE_BEING_DELETED,
            NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        ] {
            let bytes = record_of_nlink(nlink).to_bytes();
            assert_eq!(
                u32::from_le_bytes(bytes[36..40].try_into().expect("偏移 36 起 4 字节")),
                nlink,
                "nlink 写在偏移 36，写的是结构里的值 {nlink}"
            );
            assert_eq!(
                InodeRecord::parse(&bytes).map(|parsed| parsed.nlink),
                Some(nlink),
                "nlink {nlink} 读回来不变"
            );
            assert_eq!(
                InodeRecord::parse(&bytes).map(|parsed| parsed.size),
                Some(40_000),
                "size 不随 nlink 变（意图那次 size 不变）"
            );
        }
    }

    /// C480（inode 记录的 blocks 怎么算全仓没有条款） 定案（D8（核心索引结构） 已定项 6）：`blocks` = ⌈size ÷ 512⌉，
    /// 逻辑长度的块数，不表示分到的空间。期望值按条款手算，不从被测代码反推。
    /// 每个取样点旁边写着另外两种读法会给的数：「按分到几个 32 KiB 数据单元 × 64 填」（净荷 32634 一个单元）
    /// 与「向下取整」——除了 0 之外，每个点至少把其中一种分开。
    #[test]
    fn blocks_is_the_logical_length_in_512_byte_blocks_rounded_up_not_the_allocated_units() {
        let blocks_written_for_size = |size: u64| {
            let bytes = InodeRecord {
                inode: 1,
                object_birth: CheckpointTxg(3),
                nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
                size,
                change_count: 3,
                write_time_seconds: 1_788_000_000,
            }
            .to_bytes();
            u64::from_le_bytes(bytes[48..56].try_into().expect("偏移 48 起 8 字节"))
        };
        // (size, 条款给的 blocks, 按分到的单元填, 向下取整)
        let expected: [(u64, u64, u64, u64); 7] = [
            (0, 0, 0, 0),
            (1, 1, 64, 0),
            (512, 1, 64, 1),
            (513, 2, 64, 1),
            (3000, 6, 64, 5),
            (32_635, 64, 128, 63),
            (40_000, 79, 128, 78),
        ];
        for (size, blocks, by_allocated_units, rounded_down) in expected {
            assert_eq!(
                blocks_written_for_size(size),
                blocks,
                "size {size} 字节 ⇒ blocks = ⌈{size} ÷ 512⌉ = {blocks}（按分到的单元填是 {by_allocated_units}，向下取整是 {rounded_down}）"
            );
        }
    }

    /// 条目宽是索引节点头里的一个**盘上字段**（`parse_index_node` 只判了它 ≥ key 宽）：
    /// 窄到刚好等于 key 宽的条目，四个读者都要交回 `None`，不许按字段表的固定偏移切下去。
    /// key 宽：extent 24、inode 8、记账 22、映射 27（panic 面普查 R1 / R3 / R4 / R2 的坏法就是把条目宽缩到这四个数）。
    #[test]
    fn entry_readers_refuse_an_entry_narrower_than_its_field_table() {
        let entry_narrowed_to_the_key_width = |key_width: usize| vec![0u8; key_width];
        assert_eq!(
            parse_extent_record(&entry_narrowed_to_the_key_width(24)),
            Err(PointerCarryingEntryRefusal::NarrowerThanItsFieldTable),
            "extent 叶记录该有 112 字节，条目宽缩到 key 宽 24 时不解"
        );
        assert_eq!(
            parse_extent_record(&[0u8; 111]),
            Err(PointerCarryingEntryRefusal::NarrowerThanItsFieldTable),
            "差一个字节也不解"
        );
        assert!(
            parse_extent_record(&[0u8; 112]).is_ok(),
            "刚好 112 要解得开"
        );
        assert_eq!(
            parse_inode_internal_entry(&entry_narrowed_to_the_key_width(8)),
            None,
            "inode 内部条目该有 120 字节，条目宽缩到 key 宽 8 时不解"
        );
        assert!(
            parse_inode_internal_entry(&[0u8; 120]).is_some(),
            "刚好 120 要解得开"
        );
        assert_eq!(
            AccountingEntry::parse(&entry_narrowed_to_the_key_width(22)),
            None,
            "记账条目该有 34 字节，条目宽缩到 key 宽 22 时不解"
        );
        assert!(
            AccountingEntry::parse(&[0u8; 34]).is_some(),
            "刚好 34 要解得开"
        );
        assert_eq!(
            parse_mapping_entry(&entry_narrowed_to_the_key_width(27)),
            None,
            "映射条目该有 55 字节，条目宽缩到 key 宽 27 时不解（普查 R2）"
        );
        assert_eq!(parse_mapping_entry(&[0u8; 54]), None, "差一个字节也不解");
        assert!(
            parse_mapping_entry(&[0u8; 55]).is_some(),
            "刚好 55 要解得开"
        );
    }

    #[test]
    fn mapping_keys_order_by_field_not_by_bytes() {
        let pointer_with_sequence = |sequence: u32| NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
                birth_txg: CheckpointTxg(3),
            },
            locations: NodePointer::empty_root().locations,
            instance: InstanceGeneration(1),
            birth_sequence: BirthSequence(sequence),
        };
        let earlier = mapping_key_for_node(2, pointer_with_sequence(1));
        let later = mapping_key_for_node(2, pointer_with_sequence(256));
        assert!(
            mapping_key_sort_key(&earlier) < mapping_key_sort_key(&later),
            "1 < 256 按整数比，小端字节序会比反"
        );
        assert!(earlier > later, "memcmp 序确实是反的，所以不能用它");
    }
}
