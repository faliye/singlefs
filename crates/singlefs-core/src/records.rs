//! 树里的记录与条目：inode 记录 140、inode 内部条目 120、extent 叶记录 112、记账条目 34、映射条目 55、树表条目 200。
//! key 的全序按逐字段无符号整数、字段自左向右比较（D8（核心索引结构） 已定项 11）——小端存储不构成 memcmp 序，所以各给 sort key。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）：每一种记录有一份写法的规格
//! （`spec_bytes`）、一份读法的规格（`spec_parse`）与一条往返引理（`lemma_parse_round_trip`：写出来的字节读回同一条记录）；
//! 写的函数交出的就是写法的规格，读的函数交出的就是读法的规格。

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_POINTER_BYTES,
    RETIREMENT_RECORD_BYTES, TOMBSTONE_RECORD_BYTES, TREE_TABLE_ENTRY_BYTES,
};
use vstd::prelude::*;

use crate::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, TreeIdentifier,
};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::pointer::*;
use crate::unit::{PackedIdentity, WriteOrder, UNIT_CLASS_DATA};

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段；MAPPING_KEY_BYTES 在 verified_model.rs 里）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::ACCOUNTING_ENTRY_BYTES ] -> (entry_bytes: u64) returns 34u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::EXTENT_LEAF_RECORD_BYTES ] -> (record_bytes: u64) returns 112u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::INODE_INTERNAL_ENTRY ] -> (entry_bytes: u64) returns 120u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::INODE_RECORD_BYTES ] -> (record_bytes: u64) returns 140u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::LINEAGE_RECORD_BYTES ] -> (record_bytes: u64) returns 24u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::MAPPING_ENTRY_BYTES ] -> (entry_bytes: u64) returns 55u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::RETIREMENT_RECORD_BYTES ] -> (record_bytes: u64) returns 34u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::TOMBSTONE_RECORD_BYTES ] -> (record_bytes: u64) returns 40u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::TREE_TABLE_ENTRY_BYTES ] -> (entry_bytes: u64) returns 200u64;

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

/// inode 记录里截断字段（T 8 字节、N_w 8 字节）的起点：之前 108–119、之后 136–139 两段仍恒 0。
pub const INODE_RECORD_TRUNCATION_FIELD_OFFSET: u64 = 120;

/// inode 记录偏移 48 那个 `blocks` 字段的计量单位：512 字节一块（D8（核心索引结构） 已定项 6）。
pub const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;

/// 第一版没有目录，活着的文件 nlink 恒 1（D8（核心索引结构） 已定项 6「第一版的唯一文件取 inode 1，nlink = 1」）。
pub const NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES: u32 = 1;

/// unlink 的意图那次发布把记录写成 nlink = 0、size 不变（D8（核心索引结构） 已定项 16 第 1 条）：从那一版起读者答「没有文件」、写入口拒它。
pub const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0;

/// inode 记录偏移 120 起的截断字段：T（截断字段的 txg，偏移 120 起
/// 8 字节小端，0 是没写过）与 N_w（截断字段的 size，偏移 128 起 8 字节小端）。存 size 不存单元数：单元数由 ⌈N_w ÷ 净荷⌉.max(1) 现算。
/// 合法性只判「T = 0 ⇒ N_w = 0」（截到 0 时 T ≠ 0、N_w = 0）：两段私有，只能经 [`Self::NOT_WRITTEN`]、
/// [`Self::written_by_the_truncation_intent_of`] 与 [`Self::from_the_two_fields_on_disk`] 进来，T = 0 而 N_w ≠ 0 的值造不出来
/// （这一条是类型不变量 [`Self::is_consistent`]）。
/// 什么时候写由 `truncate::truncation_field_after_the_truncation_intent` 判（R2 与第六节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InodeRecordTruncationField {
    truncation_txg: CheckpointTxg,
    truncated_size_in_bytes: u64,
}

impl InodeRecordTruncationField {
    /// 类型不变量：T = 0 ⇒ N_w = 0。
    #[verifier::type_invariant]
    spec fn is_consistent(self) -> bool {
        self.truncation_txg.0 == 0 ==> self.truncated_size_in_bytes == 0
    }

    /// T。
    pub closed spec fn spec_truncation_txg(self) -> CheckpointTxg {
        self.truncation_txg
    }

    /// N_w。
    pub closed spec fn spec_truncated_size_in_bytes(self) -> u64 {
        self.truncated_size_in_bytes
    }

    /// 类型不变量的公开说法：每一个造得出来的值都满足它，两个读字段的方法在 `ensures` 里交出来。
    pub open spec fn spec_is_consistent(self) -> bool {
        self.spec_truncation_txg().0 == 0 ==> self.spec_truncated_size_in_bytes() == 0
    }

    /// 盘上读来的两段 8 字节解成的字段（[`Self::from_the_two_fields_on_disk`] 的判法）。
    pub closed spec fn spec_from_the_two_fields_on_disk(truncation_txg: u64, truncated_size_in_bytes: u64) -> Option<Self> {
        if truncation_txg == 0 && truncated_size_in_bytes != 0 {
            None
        } else {
            Some(InodeRecordTruncationField { truncation_txg: CheckpointTxg(truncation_txg), truncated_size_in_bytes })
        }
    }

    /// 往返：合法的字段把它的两段再解一次就是它自己；解出来的两段就是给的那两段。
    pub proof fn lemma_from_the_two_fields_on_disk(self, truncation_txg: u64, truncated_size_in_bytes: u64)
        ensures
            self.spec_is_consistent() ==> Self::spec_from_the_two_fields_on_disk(
                self.spec_truncation_txg().0,
                self.spec_truncated_size_in_bytes(),
            ) == Some(self),
            Self::spec_from_the_two_fields_on_disk(truncation_txg, truncated_size_in_bytes) matches Some(field)
                ==> field.spec_truncation_txg().0 == truncation_txg && field.spec_truncated_size_in_bytes()
                == truncated_size_in_bytes && field.spec_is_consistent(),
    {
    }

    /// 截断意图那次写下的（这次 txg, N）。
    ///
    /// # Panics
    /// `truncation_txg` 是 0（`requires` 排除的那一种）：T = 0 是「没写过」，截断意图那次发布的 checkpoint_txg 恒大于 0
    /// （mkfs 那一版是第 0 代，之后每次发布加一）。
    #[must_use]
    pub fn written_by_the_truncation_intent_of(
        truncation_txg: CheckpointTxg,
        truncated_size_in_bytes: u64,
    ) -> (field: Self)
        requires
            truncation_txg.0 != 0,
        ensures
            field.spec_truncation_txg() == truncation_txg,
            field.spec_truncated_size_in_bytes() == truncated_size_in_bytes,
    {
        // assert_ne! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话。
        assert!(
            truncation_txg.0 != 0,
            "截断意图那次发布的 checkpoint_txg 不是 0（第 0 代是 mkfs 那一版，不发截断）"
        );
        Self {
            truncation_txg,
            truncated_size_in_bytes,
        }
    }

    /// 盘上读来的两段 8 字节：T = 0 而 N_w ≠ 0 不合法 ⇒ `None`（R1）。
    #[must_use]
    pub const fn from_the_two_fields_on_disk(
        truncation_txg: u64,
        truncated_size_in_bytes: u64,
    ) -> (field: Option<Self>)
        ensures
            field == Self::spec_from_the_two_fields_on_disk(truncation_txg, truncated_size_in_bytes),
    {
        if truncation_txg == 0 && truncated_size_in_bytes != 0 {
            return None;
        }
        Some(Self {
            truncation_txg: CheckpointTxg(truncation_txg),
            truncated_size_in_bytes,
        })
    }

    /// T：写下字段的那次截断意图的 checkpoint_txg；没写过是 0。
    #[must_use]
    pub const fn truncation_txg(self) -> (truncation_txg: CheckpointTxg)
        ensures
            truncation_txg == self.spec_truncation_txg(),
            self.spec_is_consistent(),
    {
        proof {
            use_type_invariant(self);
        }
        self.truncation_txg
    }

    /// N_w：写下字段的那次截断意图的目标 size；没写过是 0。
    #[must_use]
    pub const fn truncated_size_in_bytes(self) -> (truncated_size_in_bytes: u64)
        ensures
            truncated_size_in_bytes == self.spec_truncated_size_in_bytes(),
            self.spec_is_consistent(),
    {
        proof {
            use_type_invariant(self);
        }
        self.truncated_size_in_bytes
    }

    /// T ≠ 0：被一次截断意图写过。
    #[must_use]
    pub const fn is_written(self) -> (is_written: bool)
        ensures
            is_written == (self.spec_truncation_txg().0 != 0),
    {
        self.truncation_txg.0 != 0
    }
}

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
    /// 偏移 120 起两段 8 字节。
    pub truncation_field: InodeRecordTruncationField,
}

} // verus!

// 这一个关联常量在 verus! 外面：字段私有的类型在 verus! 里不许用结构体字面量写公开常量（Verus 报「constructor for an opaque datatype」）；
// T = 0、N_w = 0 满足类型不变量。
impl InodeRecordTruncationField {
    /// T = 0、N_w = 0：建出来之后没被截断意图写过，或整份写（顺序写、覆盖写）清零之后。
    pub const NOT_WRITTEN: Self = Self {
        truncation_txg: CheckpointTxg(0),
        truncated_size_in_bytes: 0,
    };
}

verus! {

/// `blocks` = ⌈`size` ÷ 512⌉（[`InodeRecord::to_bytes`] 写的那 8 字节）。
pub open spec fn spec_blocks_of_the_size(size: u64) -> u64 {
    if size % 512 == 0 {
        (size / 512) as u64
    } else {
        (size / 512 + 1) as u64
    }
}

impl InodeRecord {
    /// 截断字段满足类型不变量（每一个造得出来的字段都满足）：[`Self::lemma_parse_round_trip`] 的前提。
    pub open spec fn is_well_formed(self) -> bool {
        self.truncation_field.spec_is_consistent()
    }

    /// 盘上 140 字节（[`Self::to_bytes`] 交出的）。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.inode as nat, 8) + little_endian_bytes(self.object_birth.0 as nat, 8)
            + little_endian_bytes(0, 8) + little_endian_bytes(0o100_644, 4) + little_endian_bytes(0, 4)
            + little_endian_bytes(0, 4) + little_endian_bytes(self.nlink as nat, 4) + little_endian_bytes(
            self.size as nat,
            8,
        ) + little_endian_bytes(spec_blocks_of_the_size(self.size) as nat, 8) + little_endian_bytes(0, 8)
            + little_endian_bytes(self.write_time_seconds as nat, 8) + little_endian_bytes(
            self.write_time_seconds as nat,
            8,
        ) + little_endian_bytes(self.write_time_seconds as nat, 8) + little_endian_bytes(
            self.change_count as nat,
            8,
        ) + zero_bytes(12) + zero_bytes(12) + little_endian_bytes(
            self.truncation_field.spec_truncation_txg().0 as nat,
            8,
        ) + little_endian_bytes(self.truncation_field.spec_truncated_size_in_bytes() as nat, 8) + zero_bytes(4)
    }

    /// [`Self::parse`] 的判法。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Option<Self> {
        if bytes.len() != INODE_RECORD_BYTES || bytes.subrange(108, 120) != zero_bytes(12) || bytes.subrange(136, 140)
            != zero_bytes(4) {
            None
        } else {
            match InodeRecordTruncationField::spec_from_the_two_fields_on_disk(
                little_endian_value(bytes.subrange(120, 128)) as u64,
                little_endian_value(bytes.subrange(128, 136)) as u64,
            ) {
                None => None,
                Some(truncation_field) => Some(InodeRecord {
                    inode: little_endian_value(bytes.subrange(0, 8)) as u64,
                    object_birth: CheckpointTxg(little_endian_value(bytes.subrange(8, 16)) as u64),
                    nlink: little_endian_value(bytes.subrange(36, 40)) as u32,
                    size: little_endian_value(bytes.subrange(40, 48)) as u64,
                    change_count: little_endian_value(bytes.subrange(88, 96)) as u64,
                    write_time_seconds: little_endian_value(bytes.subrange(64, 72)) as u64,
                    truncation_field,
                }),
            }
        }
    }

    /// 往返：写出的 140 字节读回同一条记录。
    pub proof fn lemma_parse_round_trip(self)
        requires
            self.is_well_formed(),
        ensures
            Self::spec_parse(self.spec_bytes()) == Some(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let inode = little_endian_bytes(self.inode as nat, 8);
        let birth = little_endian_bytes(self.object_birth.0 as nat, 8);
        let locality = little_endian_bytes(0, 8);
        let mode = little_endian_bytes(0o100_644, 4);
        let uid = little_endian_bytes(0, 4);
        let gid = little_endian_bytes(0, 4);
        let nlink = little_endian_bytes(self.nlink as nat, 4);
        let size = little_endian_bytes(self.size as nat, 8);
        let blocks = little_endian_bytes(spec_blocks_of_the_size(self.size) as nat, 8);
        let rdev = little_endian_bytes(0, 8);
        let access_time = little_endian_bytes(self.write_time_seconds as nat, 8);
        let modify_time = little_endian_bytes(self.write_time_seconds as nat, 8);
        let change_time = little_endian_bytes(self.write_time_seconds as nat, 8);
        let change_count = little_endian_bytes(self.change_count as nat, 8);
        let nanoseconds = zero_bytes(12);
        let reserved = zero_bytes(12);
        let truncation_txg = little_endian_bytes(self.truncation_field.spec_truncation_txg().0 as nat, 8);
        let truncated_size = little_endian_bytes(self.truncation_field.spec_truncated_size_in_bytes() as nat, 8);
        let tail_reserved = zero_bytes(4);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time + change_count + nanoseconds + reserved + truncation_txg + truncated_size, tail_reserved);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time + change_count + nanoseconds + reserved + truncation_txg, truncated_size);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time + change_count + nanoseconds + reserved, truncation_txg);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time + change_count + nanoseconds, reserved);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time + change_count, nanoseconds);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time + change_time, change_count);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time + modify_time, change_time);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev + access_time, modify_time);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks + rdev, access_time);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size + blocks, rdev);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink + size, blocks);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid + nlink, size);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid + gid, nlink);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode + uid, gid);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality + mode, uid);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth + locality, mode);
        lemma_split_a_concatenated_range(bytes, 0, inode + birth, locality);
        lemma_split_a_concatenated_range(bytes, 0, inode, birth);
        self.truncation_field.lemma_from_the_two_fields_on_disk(0, 0);
    }

    /// # Panics
    /// 截断字段不满足类型不变量造不出来，这里不 panic；字段表偏移的断言由 Verus 证过对得上。
    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
            self.is_well_formed(),
    {
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
        // blocks = ⌈size ÷ 512⌉：u64::div_ceil 没有 Verus 规格，写成取商、有余数再加一，同一个数。
        let blocks = if self.size.is_multiple_of(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES) {
            self.size / INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES
        } else {
            self.size / INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES + 1
        };
        writer.put_u64(blocks);
        writer.put_u64(0); // rdev
        writer.assert_position(64, "时间秒");
        // 三个时间秒（atime、mtime、ctime）写同一个值。
        writer.put_u64(self.write_time_seconds);
        writer.put_u64(self.write_time_seconds);
        writer.put_u64(self.write_time_seconds);
        writer.assert_position(88, "改动计数");
        writer.put_u64(self.change_count);
        writer.assert_position(96, "时间纳秒");
        writer.skip(12);
        writer.assert_position(108, "填充 / flags / 预留");
        writer.skip(4 + 8);
        writer.assert_position(INODE_RECORD_TRUNCATION_FIELD_OFFSET, "截断字段 T 与 N_w");
        writer.put_u64(self.truncation_field.truncation_txg().0);
        writer.put_u64(self.truncation_field.truncated_size_in_bytes());
        writer.assert_position(136, "预留");
        writer.skip(4);
        writer.assert_position(INODE_RECORD_BYTES, "inode 记录");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// 读者：偏移 108–119 与 136–139 两段恒零区非零、截断字段「T = 0 而 N_w ≠ 0」都判不可用（I-9.7）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (record: Option<Self>)
        ensures
            record == Self::spec_parse(bytes@),
    {
        let truncation_field_offset =
            usize::try_from(INODE_RECORD_TRUNCATION_FIELD_OFFSET).expect("120");
        // 两段恒零区分开判（迭代器的 chain 与 any 没有 Verus 规格），两段都全零才收，与连起来判同一个结论。
        if bytes.len() != usize::try_from(INODE_RECORD_BYTES).expect("140")
            || !every_byte_is_zero(&bytes[108..truncation_field_offset])
            || !every_byte_is_zero(&bytes[truncation_field_offset + 16..])
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
        let mut truncation_field_reader = ByteReader::at(
            bytes,
            usize::try_from(INODE_RECORD_TRUNCATION_FIELD_OFFSET).expect("120"),
        );
        let truncation_txg = truncation_field_reader.get_u64();
        let truncated_size_in_bytes = truncation_field_reader.get_u64();
        let truncation_field = InodeRecordTruncationField::from_the_two_fields_on_disk(
            truncation_txg,
            truncated_size_in_bytes,
        )?;
        Some(Self {
            inode,
            object_birth,
            nlink,
            size,
            change_count,
            write_time_seconds,
            truncation_field,
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
    ) -> (record: Self)
        ensures
            record == (TombstoneRecord {
                object,
                object_birth,
                range_start_in_bytes: 0,
                range_end_in_bytes: u64::MAX,
                death_txg,
            }),
    {
        Self {
            object,
            object_birth,
            range_start_in_bytes: Self::WHOLE_OBJECT_RANGE_START_IN_BYTES,
            range_end_in_bytes: Self::WHOLE_OBJECT_RANGE_END_IN_BYTES,
            death_txg,
        }
    }

    /// 盘上 40 字节：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.object.0 as nat, 8) + little_endian_bytes(self.object_birth.0 as nat, 8)
            + little_endian_bytes(self.range_start_in_bytes as nat, 8) + little_endian_bytes(
            self.range_end_in_bytes as nat,
            8,
        ) + little_endian_bytes(self.death_txg.0 as nat, 8)
    }

    /// [`Self::parse`] 的判法：长度不是 40 交回 `None`。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Option<Self> {
        if bytes.len() != TOMBSTONE_RECORD_BYTES {
            None
        } else {
            Some(TombstoneRecord {
                object: InodeNumber(little_endian_value(bytes.subrange(0, 8)) as u64),
                object_birth: CheckpointTxg(little_endian_value(bytes.subrange(8, 16)) as u64),
                range_start_in_bytes: little_endian_value(bytes.subrange(16, 24)) as u64,
                range_end_in_bytes: little_endian_value(bytes.subrange(24, 32)) as u64,
                death_txg: CheckpointTxg(little_endian_value(bytes.subrange(32, 40)) as u64),
            })
        }
    }

    /// 往返：写出的 40 字节读回同一条墓碑。
    pub proof fn lemma_parse_round_trip(self)
        ensures
            Self::spec_parse(self.spec_bytes()) == Some(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let object = little_endian_bytes(self.object.0 as nat, 8);
        let birth = little_endian_bytes(self.object_birth.0 as nat, 8);
        let start = little_endian_bytes(self.range_start_in_bytes as nat, 8);
        let end = little_endian_bytes(self.range_end_in_bytes as nat, 8);
        let death = little_endian_bytes(self.death_txg.0 as nat, 8);
        lemma_split_a_concatenated_range(bytes, 0, object + birth + start + end, death);
        lemma_split_a_concatenated_range(bytes, 0, object + birth + start, end);
        lemma_split_a_concatenated_range(bytes, 0, object + birth, start);
        lemma_split_a_concatenated_range(bytes, 0, object, birth);
    }

    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
    {
        let mut writer = ByteWriter::new(usize::try_from(TOMBSTONE_RECORD_BYTES).expect("40"));
        writer.put_u64(self.object.0);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(self.range_start_in_bytes);
        writer.put_u64(self.range_end_in_bytes);
        writer.put_u64(self.death_txg.0);
        writer.assert_position(TOMBSTONE_RECORD_BYTES, "墓碑记录");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// 墓碑记录的读者：长度不是 40 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 40 再切）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (record: Option<Self>)
        ensures
            record == Self::spec_parse(bytes@),
    {
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

/// 谱系记录 24（D18（块里携带什么信息） 已定项 24 第 5 条；字段表在 `.claude/kb/layout/03-third-txn.md`「三、谱系记录」）：
/// T_rb 8 + T_old 8 + H_cur 8，小端，读作「T_rb 的父是 T_old」。管理员回退那次发布追加一条：T_rb 是回退那次发布的 checkpoint_txg，
/// T_old 是回退目标 R_old 的 txg，H_cur 是 cur（回退之前的现行那一版）的 inode 号水位，从 cur 那一版读、不从发布路径交来的「上一版」现算。
/// 三段的逐字节次序条款没定，这里取 T_rb | T_old | H_cur（新加的一段接在已有两段之后，已有两段的偏移不动）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineageRecord {
    pub rollback_txg: CheckpointTxg,
    pub rollback_target_txg: CheckpointTxg,
    pub inode_number_watermark_of_the_version_rolled_back_from: u64,
}

impl LineageRecord {
    /// 盘上 24 字节：T_rb 8 + T_old 8 + H_cur 8。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.rollback_txg.0 as nat, 8) + little_endian_bytes(self.rollback_target_txg.0 as nat, 8)
            + little_endian_bytes(self.inode_number_watermark_of_the_version_rolled_back_from as nat, 8)
    }

    /// [`Self::parse`] 的判法：长度不是 24 交回 `None`。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Option<Self> {
        if bytes.len() != LINEAGE_RECORD_BYTES {
            None
        } else {
            Some(LineageRecord {
                rollback_txg: CheckpointTxg(little_endian_value(bytes.subrange(0, 8)) as u64),
                rollback_target_txg: CheckpointTxg(little_endian_value(bytes.subrange(8, 16)) as u64),
                inode_number_watermark_of_the_version_rolled_back_from: little_endian_value(bytes.subrange(16, 24)) as u64,
            })
        }
    }

    /// 往返：写出的 24 字节读回同一条谱系记录。
    pub proof fn lemma_parse_round_trip(self)
        ensures
            Self::spec_parse(self.spec_bytes()) == Some(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let rollback = little_endian_bytes(self.rollback_txg.0 as nat, 8);
        let target = little_endian_bytes(self.rollback_target_txg.0 as nat, 8);
        let watermark = little_endian_bytes(self.inode_number_watermark_of_the_version_rolled_back_from as nat, 8);
        lemma_split_a_concatenated_range(bytes, 0, rollback + target, watermark);
        lemma_split_a_concatenated_range(bytes, 0, rollback, target);
    }

    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
    {
        let mut writer = ByteWriter::new(usize::try_from(LINEAGE_RECORD_BYTES).expect("24"));
        writer.put_u64(self.rollback_txg.0);
        writer.put_u64(self.rollback_target_txg.0);
        writer.put_u64(self.inode_number_watermark_of_the_version_rolled_back_from);
        writer.assert_position(LINEAGE_RECORD_BYTES, "谱系记录");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// 谱系记录的读者：长度不是 24 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 24 再切）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (record: Option<Self>)
        ensures
            record == Self::spec_parse(bytes@),
    {
        if bytes.len() != usize::try_from(LINEAGE_RECORD_BYTES).expect("24") {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        Some(Self {
            rollback_txg: CheckpointTxg(reader.get_u64()),
            rollback_target_txg: CheckpointTxg(reader.get_u64()),
            inode_number_watermark_of_the_version_rolled_back_from: reader.get_u64(),
        })
    }
}

/// 退役记录 34（D18（块里携带什么信息） 已定项 11 打包记录类型 3；第 2、3 条）：
/// 被退役身份的出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8 | 死亡 txg 8，小端，无填充无预留。身份四段与 inode 内部条目里的
/// 身份引用（[`build_inode_internal_entry`]）同序同宽。死亡 txg = 身份消失那次发布的 checkpoint_txg（删空摘掉、被左邻吸收那次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetirementRecord {
    pub retired: PackedIdentity,
    pub death_txg: CheckpointTxg,
}

impl RetirementRecord {
    /// 盘上 34 字节：被退役的身份 26 + 死亡 txg 8。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        self.retired.spec_bytes() + little_endian_bytes(self.death_txg.0 as nat, 8)
    }

    /// [`Self::parse`] 的判法：长度不是 34 交回 `None`。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Option<Self> {
        if bytes.len() != RETIREMENT_RECORD_BYTES {
            None
        } else {
            Some(RetirementRecord {
                retired: PackedIdentity::spec_decode_at(bytes, 0),
                death_txg: CheckpointTxg(little_endian_value(bytes.subrange(26, 34)) as u64),
            })
        }
    }

    /// 往返：写出的 34 字节读回同一条退役记录。
    pub proof fn lemma_parse_round_trip(self)
        ensures
            Self::spec_parse(self.spec_bytes()) == Some(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        lemma_split_a_concatenated_range(
            bytes,
            0,
            self.retired.spec_bytes(),
            little_endian_bytes(self.death_txg.0 as nat, 8),
        );
        self.retired.lemma_decode_at_round_trip(bytes, 0);
    }

    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
    {
        let mut writer = ByteWriter::new(usize::try_from(RETIREMENT_RECORD_BYTES).expect("34"));
        writer.put_u64(self.retired.birth_tree.0);
        writer.put_u16(self.retired.record_type);
        writer.put_u64(self.retired.container);
        writer.put_u64(self.retired.container_birth.0);
        writer.put_u64(self.death_txg.0);
        writer.assert_position(RETIREMENT_RECORD_BYTES, "退役记录");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// 退役记录的读者：长度不是 34 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 34 再切）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (record: Option<Self>)
        ensures
            record == Self::spec_parse(bytes@),
    {
        if bytes.len() != usize::try_from(RETIREMENT_RECORD_BYTES).expect("34") {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        Some(Self {
            retired: PackedIdentity {
                birth_tree: TreeIdentifier(reader.get_u64()),
                record_type: reader.get_u16(),
                container: reader.get_u64(),
                container_birth: CheckpointTxg(reader.get_u64()),
            },
            death_txg: CheckpointTxg(reader.get_u64()),
        })
    }
}

/// inode 内部条目 120 的写法：分隔 key 8 + 身份引用 26 + 子指针 86。
pub open spec fn spec_inode_internal_entry_bytes(
    separator_key: u64,
    child_identity: PackedIdentity,
    child: NodePointer,
) -> Seq<u8> {
    little_endian_bytes(separator_key as nat, 8) + child_identity.spec_bytes() + child.spec_bytes()
}

/// inode 树内部节点条目 120：分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）+ 子指针 86。
///
/// # Panics
/// 子指针的槽号装不进 6 字节或位置条目次序写者不肯写（`requires` 排除的那一种）。
#[must_use]
pub fn build_inode_internal_entry(
    separator_key: u64,
    child_identity: PackedIdentity,
    child: NodePointer,
) -> (entry: Vec<u8>)
    requires
        child.is_encodable(),
        child.has_location_entries_the_writer_accepts(),
    ensures
        entry@ == spec_inode_internal_entry_bytes(separator_key, child_identity, child),
{
    let mut writer = ByteWriter::new(usize::try_from(INODE_INTERNAL_ENTRY).expect("120"));
    writer.put_u64(separator_key);
    writer.put_u64(child_identity.birth_tree.0);
    writer.put_u16(child_identity.record_type);
    writer.put_u64(child_identity.container);
    writer.put_u64(child_identity.container_birth.0);
    child.write_to(&mut writer);
    writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");
    let entry = writer.into_bytes();
    assert(entry@ =~= spec_inode_internal_entry_bytes(separator_key, child_identity, child));
    entry
}

/// extent 叶记录 112 的写法：key (locality_id 0, inode 8, offset 8) + 数据指针 88。
pub open spec fn spec_extent_record_bytes(inode: u64, offset: u64, pointer: DataPointer) -> Seq<u8> {
    little_endian_bytes(0, 8) + little_endian_bytes(inode as nat, 8) + little_endian_bytes(offset as nat, 8)
        + pointer.spec_bytes()
}

/// extent 叶记录 112：key (locality_id 8, inode 8, offset 8) + 数据指针 88。
///
/// # Panics
/// 数据指针装不进或位置条目不按设备身份严格升序（`requires` 排除的那一种）。
#[must_use]
pub fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> (record: Vec<u8>)
    requires
        pointer.is_encodable(),
        spec_location_entries_ascend_by_device(pointer.locations),
    ensures
        record@ == spec_extent_record_bytes(inode, offset, pointer),
{
    let mut writer = ByteWriter::new(usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"));
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    let record = writer.into_bytes();
    assert(record@ =~= spec_extent_record_bytes(inode, offset, pointer));
    record
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
    /// 盘上 34 字节。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.statistic as nat, 2) + little_endian_bytes(self.tree.0 as nat, 8)
            + little_endian_bytes(self.device.0 as nat, 4) + little_endian_bytes(self.generation.0 as nat, 8)
            + little_endian_bytes(self.value as nat, 8) + little_endian_bytes(self.sequence as nat, 4)
    }

    /// [`Self::parse`] 的判法：宽不等于 34 交回 `None`。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Option<Self> {
        if bytes.len() != ACCOUNTING_ENTRY_BYTES {
            None
        } else {
            Some(AccountingEntry {
                statistic: little_endian_value(bytes.subrange(0, 2)) as u16,
                tree: TreeIdentifier(little_endian_value(bytes.subrange(2, 10)) as u64),
                device: DeviceIdentity(little_endian_value(bytes.subrange(10, 14)) as u32),
                generation: CheckpointTxg(little_endian_value(bytes.subrange(14, 22)) as u64),
                value: little_endian_value(bytes.subrange(22, 30)) as u64,
                sequence: little_endian_value(bytes.subrange(30, 34)) as u32,
            })
        }
    }

    /// 往返：写出的 34 字节读回同一条记账条目。
    pub proof fn lemma_parse_round_trip(self)
        ensures
            Self::spec_parse(self.spec_bytes()) == Some(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let statistic = little_endian_bytes(self.statistic as nat, 2);
        let tree = little_endian_bytes(self.tree.0 as nat, 8);
        let device = little_endian_bytes(self.device.0 as nat, 4);
        let generation = little_endian_bytes(self.generation.0 as nat, 8);
        let value = little_endian_bytes(self.value as nat, 8);
        let sequence = little_endian_bytes(self.sequence as nat, 4);
        lemma_split_a_concatenated_range(bytes, 0, statistic + tree + device + generation + value, sequence);
        lemma_split_a_concatenated_range(bytes, 0, statistic + tree + device + generation, value);
        lemma_split_a_concatenated_range(bytes, 0, statistic + tree + device, generation);
        lemma_split_a_concatenated_range(bytes, 0, statistic + tree, device);
        lemma_split_a_concatenated_range(bytes, 0, statistic, tree);
    }

    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
    {
        let mut writer = ByteWriter::new(usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"));
        writer.put_u16(self.statistic);
        writer.put_u64(self.tree.0);
        writer.put_u32(self.device.0);
        writer.put_u64(self.generation.0);
        writer.put_u64(self.value);
        writer.put_u32(self.sequence);
        writer.assert_position(ACCOUNTING_ENTRY_BYTES, "记账条目");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// key 那 22 字节（标签、树 ID、设备、代）：整条的前 22 字节。
    #[must_use]
    pub fn key_bytes(&self) -> (key: Vec<u8>)
        ensures
            key@ == self.spec_bytes().subrange(0, 22),
    {
        // `to_vec` 没有 Verus 规格，经 copied_into_a_vector 拷，同一个结果。
        copied_into_a_vector(&self.to_bytes()[..22])
    }

    #[must_use]
    pub fn sort_key(&self) -> (sort_key: (u16, u64, u32, u64))
        ensures
            sort_key == (self.statistic, self.tree.0, self.device.0, self.generation.0),
    {
        (
            self.statistic,
            self.tree.0,
            self.device.0,
            self.generation.0,
        )
    }

    /// 记账条目 34 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 22），
    /// 不等于 34 时返回 `None`：这里是盘上字节进字段表的边界，往里就按 34 信它。窄于 34 的切不出整条记录；
    /// 宽于 34 的同样不收（I-1.10（码 2 条目宽等于字段表宽） 写的是「等于」，欠账 C780）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (entry: Option<Self>)
        ensures
            entry == Self::spec_parse(bytes@),
    {
        if bytes.len() != usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34") {
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

/// 码 1 映射 key 的写法：类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10。
pub open spec fn spec_mapping_key_for_data_bytes(head: PointerHead, write_order: WriteOrder) -> Seq<u8> {
    little_endian_bytes(UNIT_CLASS_DATA as nat, 1) + little_endian_bytes(head.birth_tree.0 as nat, 8)
        + little_endian_bytes(head.birth_txg.0 as nat, 8) + write_order.spec_bytes()
}

/// 中央映射 key 一律 27（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10）：码 1 = 类标签 + 出生树 + 出生 txg + 写序 10；
/// 码 2 / 码 3 = 类标签 + 出生树 + 出生 txg + 实例代号 4 + 出生序号 4，末尾补零 2。
///
/// # Panics
/// 写序的事务号装不进 6 字节（`requires` 排除的那一种）。
#[must_use]
pub fn mapping_key_for_data(head: PointerHead, write_order: WriteOrder) -> (key: Vec<u8>)
    requires
        write_order.is_encodable(),
    ensures
        key@ == spec_mapping_key_for_data_bytes(head, write_order),
{
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(head.birth_tree.0);
    writer.put_u64(head.birth_txg.0);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(MAPPING_KEY_BYTES, "码 1 映射 key");
    let key = writer.into_bytes();
    assert(key@ =~= spec_mapping_key_for_data_bytes(head, write_order));
    key
}

/// 码 2 / 码 3 映射 key 的写法：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 + 补零 2。
pub open spec fn spec_mapping_key_for_node_bytes(unit_class: u8, pointer: NodePointer) -> Seq<u8> {
    little_endian_bytes(unit_class as nat, 1) + little_endian_bytes(pointer.head.birth_tree.0 as nat, 8)
        + little_endian_bytes(pointer.head.birth_txg.0 as nat, 8) + little_endian_bytes(pointer.instance.0 as nat, 4)
        + little_endian_bytes(pointer.birth_sequence.0 as nat, 4) + zero_bytes(2)
}

#[must_use]
pub fn mapping_key_for_node(unit_class: u8, pointer: NodePointer) -> (key: Vec<u8>)
    ensures
        key@ == spec_mapping_key_for_node_bytes(unit_class, pointer),
{
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
    writer.put_u8(unit_class);
    writer.put_u64(pointer.head.birth_tree.0);
    writer.put_u64(pointer.head.birth_txg.0);
    writer.put_u32(pointer.instance.0);
    writer.put_u32(pointer.birth_sequence.0);
    writer.assert_position(25, "码 2 / 码 3 映射 key 里有含义的那一段");
    writer.skip(2);
    let key = writer.into_bytes();
    assert(key@ =~= spec_mapping_key_for_node_bytes(unit_class, pointer));
    key
}

/// 类标签 → 出生树 → 出生 txg → 尾段（码 1 的写序、码 2 / 码 3 的实例代号与出生序号）。
///
/// # Panics
/// key 短于 27 字节（`requires` 排除的那一种：映射 key 一律 27）。
#[must_use]
pub fn mapping_key_sort_key(key: &[u8]) -> (sort_key: (u8, u64, u64, u32, u64))
    requires
        key@.len() >= MAPPING_KEY_BYTES,
    ensures
        sort_key.0 == little_endian_value(key@.subrange(0, 1)),
        sort_key.1 == little_endian_value(key@.subrange(1, 9)),
        sort_key.2 == little_endian_value(key@.subrange(9, 17)),
        sort_key.3 == little_endian_value(key@.subrange(17, 21)),
        sort_key.4 == if sort_key.0 == UNIT_CLASS_DATA {
            little_endian_value(key@.subrange(21, 27))
        } else {
            little_endian_value(key@.subrange(21, 25))
        },
{
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

/// 映射条目 55 的写法：key 27 + 两条位置条目 28。
pub open spec fn spec_mapping_entry_bytes(key: Seq<u8>, locations: [LocationEntry; 2]) -> Seq<u8> {
    key + spec_location_entries_bytes(locations)
}

/// 映射条目 55：key 27 + 位置条目 14 × 2。
///
/// # Panics
/// key 不是 27 字节、或位置条目的槽号装不进 6 字节（`requires` 排除的两种）。
#[must_use]
pub fn build_mapping_entry(key: &[u8], locations: [LocationEntry; 2]) -> (entry: Vec<u8>)
    requires
        key@.len() == MAPPING_KEY_BYTES,
        locations[0].is_encodable(),
        locations[1].is_encodable(),
    ensures
        entry@ == spec_mapping_entry_bytes(key@, locations),
{
    // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert!。
    assert!(
        key.len() == usize::try_from(MAPPING_KEY_BYTES).expect("27"),
        "映射 key 一律 27 字节"
    );
    let mut writer = ByteWriter::new(usize::try_from(MAPPING_ENTRY_BYTES).expect("55"));
    writer.put(key);
    // 两条位置条目照数组次序写（对数组的 for 循环没有 Verus 规格，逐条写，同一个次序）。
    locations[0].write_to(&mut writer);
    locations[1].write_to(&mut writer);
    writer.assert_position(MAPPING_ENTRY_BYTES, "映射条目");
    let entry = writer.into_bytes();
    assert(entry@ =~= spec_mapping_entry_bytes(key@, locations));
    entry
}

/// journal 点名项与映射 key 共用的 10 字节尾段。事务号只取低 6 字节（与 `to_le_bytes()[..6]` 同一个截法）。
#[must_use]
pub fn data_key_tail(write_order: WriteOrder) -> (tail: [u8; 10])
    ensures
        tail@ == little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(
            (write_order.transaction % 0x1_0000_0000_0000) as nat,
            6,
        ),
{
    // copy_from_slice 与 to_le_bytes 没有 Verus 规格：经 ByteWriter 写成 10 字节再拷成数组，同一组字节。
    let mut writer = ByteWriter::new(10);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction % 0x1_0000_0000_0000);
    let tail = writer.into_bytes();
    assert(tail@ =~= little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(
        (write_order.transaction % 0x1_0000_0000_0000) as nat,
        6,
    ));
    copied_into_an_array(&tail)
}

#[must_use]
pub fn node_key_tail(instance: InstanceGeneration, birth_sequence: BirthSequence) -> (tail: [u8; 10])
    ensures
        tail@ == little_endian_bytes(instance.0 as nat, 4) + little_endian_bytes(birth_sequence.0 as nat, 4)
            + zero_bytes(2),
{
    // 同 data_key_tail：经 ByteWriter 写成 10 字节再拷成数组，末尾两字节保持 0。
    let mut writer = ByteWriter::new(10);
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    writer.skip(2);
    let tail = writer.into_bytes();
    assert(tail@ =~= little_endian_bytes(instance.0 as nat, 4) + little_endian_bytes(birth_sequence.0 as nat, 4)
        + zero_bytes(2));
    copied_into_an_array(&tail)
}

/// 树的种类码（D8（核心索引结构） 已定项 9 的登记表）。
pub const TREE_KIND_EXTENT: u16 = 1;
pub const TREE_KIND_INODE: u16 = 2;
pub const TREE_KIND_ALLOCATION: u16 = 3;
pub const TREE_KIND_ACCOUNTING: u16 = 4;
pub const TREE_KIND_LIVELIST: u16 = 6;
pub const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
pub const TREE_KIND_DEADLIST: u16 = 8;
/// 第一个文件版本那次零根注册的四棵树的种类码：D8（核心索引结构） 已定项 9 登记表的码 9..12
/// （次序照 D18（块里携带什么信息） 已定项 24 第 6 条列的）。
pub const TREE_KIND_DELETED_INODES: u16 = 9;
pub const TREE_KIND_TOMBSTONE: u16 = 10;
pub const TREE_KIND_LINEAGE: u16 = 11;
pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
/// 退役树：码 13（D8（核心索引结构） 已定项 9 登记表；第 1 条）。
pub const TREE_KIND_RETIREMENT: u16 = 13;
/// 只读快照的 inode 根：码 14（D8（核心索引结构） 已定项 9 登记表；判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。一个快照一条，字段取值：根指针 = 建快照那一版活头 inode 树的根；
/// 诞生 txg = S.txg（建快照那次独立发布的 `checkpoint_txg`，同一判决 S2）；`previous_snapshot_txg` = 捕获那一刻活头 inode 树条目里的值；
/// 头 ID = 活头 inode 树的树 ID；flags 位 0 是「销毁中」（同一判决 S4：删快照的意图写在这里，[`SnapshotDestructionMark`]）。
/// 一张树表里可以有多条（每个快照一条），与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`] 一一配对。
pub const TREE_KIND_SNAPSHOT_INODE_ROOT: u16 = 14;
/// 只读快照的 extent 根：码 15（D8（核心索引结构） 已定项 9 登记表；同上 S1）。根指针 = 建快照那一版活头 extent 树的根；头 ID = 与它配对的那条
/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 条目的树 ID；`previous_snapshot_txg` 写 0（0 = 不适用，D5（快照 / 空间记账机制） 已定项 9）。
pub const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u16 = 15;
/// 第一版同时活着的只读快照至多几个（一张树表里码 14 的条数上界）：（判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1；单节点树表能装 34 个，取 10 是用户给的上界）。
pub const SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: usize = 10;

/// 树表条目 flags 的位 0「销毁中」：只在码 14 条目上有含义（ 与第七节岔路 2：
/// 删快照的意图写在被删那个快照自己的码 14 条目上，只占这一位，预留字节不写）。别的位、别的种类上的这一位仍按 D8（核心索引结构） 已定项 8 ㊁拒收。
pub const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u16 = 1;

/// 一条码 14 条目带不带「销毁中」的记号（flags 位 0，[`TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotDestructionMark {
    NotBeingDestroyed,
    BeingDestroyed,
}

impl SnapshotDestructionMark {
    /// 这个记号在 flags 那 2 字节里的值（[`Self::flags`] 的规格）。
    pub open spec fn spec_flags(self) -> u16 {
        match self {
            SnapshotDestructionMark::NotBeingDestroyed => 0,
            SnapshotDestructionMark::BeingDestroyed => TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED,
        }
    }

    /// 这个记号在 flags 那 2 字节里的值。
    #[must_use]
    pub const fn flags(self) -> (flags: u16)
        ensures
            flags == self.spec_flags(),
    {
        match self {
            SnapshotDestructionMark::NotBeingDestroyed => 0,
            SnapshotDestructionMark::BeingDestroyed => {
                TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
            }
        }
    }
}

/// deadlist 条目的 key 宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43，value 空（D5（快照 / 空间记账机制） 已定项 12；
/// deadlist 是第三棵多层码 2 树，种类 8）。
pub const DEADLIST_ENTRY_KEY_BYTES: usize = 8 + 8 + 27;

/// 中央映射 key 里出生 txg 那 8 字节的起点：类标签 1 + 出生树 8（[`mapping_key_for_data`]、[`mapping_key_for_node`]）。
const MAPPING_KEY_BIRTH_TXG_OFFSET_IN_BYTES: usize = 1 + 8;

/// 一把中央映射 key 里的出生 txg（D5（快照 / 空间记账机制） 已定项 19 的 `birth`：判决 S3「birth 从映射 key 的出生 txg 段取」）。
/// key 不是 27 字节时交回 `None`（盘上读来的 key 由调用方当取不到处置：D5 已定项 14「判据的输入取不到时默认进 deadlist」）。
#[must_use]
pub fn birth_txg_of_a_mapping_key(mapping_key: &[u8]) -> (birth_txg: Option<CheckpointTxg>)
    ensures
        birth_txg == if mapping_key@.len() == MAPPING_KEY_BYTES {
            Some(CheckpointTxg(little_endian_value(mapping_key@.subrange(9, 17)) as u64))
        } else {
            None
        },
{
    if mapping_key.len() != usize::try_from(MAPPING_KEY_BYTES).expect("27") {
        return None;
    }
    let mut reader = ByteReader::at(mapping_key, MAPPING_KEY_BIRTH_TXG_OFFSET_IN_BYTES);
    Some(CheckpointTxg(reader.get_u64()))
}

/// 一条 deadlist 条目（D5（快照 / 空间记账机制） 已定项 12「丙-区间」）：key = 头的树 ID + 纪元 + 中央映射 key，value 空——key 即事实。
/// 纪元 = 块被杀那一刻头的 `previous_snapshot_txg`；哪个快照的那一份按纪元落在哪一段区间认，建快照与删快照都不改条目的键。
/// 不派生全序：盘上的序是逐字段按小端无符号整数比（`code_two_tree::CodeTwoKeyFieldWidths::DEADLIST`），映射 key 那一段按字节比与它不同序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeadlistEntry {
    pub head: TreeIdentifier,
    pub epoch: CheckpointTxg,
    pub mapping_key: Vec<u8>,
}

impl DeadlistEntry {
    /// 盘上 43 字节：头的树 ID 8 + 纪元 8 + 映射 key 27。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.head.0 as nat, 8) + little_endian_bytes(self.epoch.0 as nat, 8) + self.mapping_key@
    }

    /// [`Self::parse`] 的判法：宽不等于 43 交回 `None`。
    pub open spec fn spec_parse_fields(bytes: Seq<u8>) -> Option<(TreeIdentifier, CheckpointTxg, Seq<u8>)> {
        if bytes.len() != DEADLIST_ENTRY_KEY_BYTES {
            None
        } else {
            Some(
                (
                    TreeIdentifier(little_endian_value(bytes.subrange(0, 8)) as u64),
                    CheckpointTxg(little_endian_value(bytes.subrange(8, 16)) as u64),
                    bytes.subrange(16, 43),
                ),
            )
        }
    }

    /// 往返：映射 key 是 27 字节的条目写出的 43 字节读回同样的三段。
    pub proof fn lemma_parse_round_trip(self)
        requires
            self.mapping_key@.len() == MAPPING_KEY_BYTES,
        ensures
            Self::spec_parse_fields(self.spec_bytes()) == Some((self.head, self.epoch, self.mapping_key@)),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let head = little_endian_bytes(self.head.0 as nat, 8);
        let epoch = little_endian_bytes(self.epoch.0 as nat, 8);
        let mapping_key = self.mapping_key@;
        lemma_split_a_concatenated_range(bytes, 0, head + epoch, mapping_key);
        lemma_split_a_concatenated_range(bytes, 0, head, epoch);
    }

    /// 盘上 43 字节（叶条目就是 key）。
    ///
    /// # Panics
    /// `mapping_key` 不是 27 字节（`requires` 排除的那一种）：条目的映射 key 取自一版的 `mapped_units`（写者现算的）或 [`Self::parse`] 判过宽度的盘上字节。
    #[must_use]
    pub fn key_bytes(&self) -> (key: Vec<u8>)
        requires
            self.mapping_key@.len() == MAPPING_KEY_BYTES,
        ensures
            key@ == self.spec_bytes(),
    {
        let mut writer = ByteWriter::new(DEADLIST_ENTRY_KEY_BYTES);
        writer.put_u64(self.head.0);
        writer.put_u64(self.epoch.0);
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话。
        assert!(
            self.mapping_key.len() == usize::try_from(MAPPING_KEY_BYTES).expect("27"),
            "deadlist 条目里的映射 key 是 27 字节：取自写者现算的 key 或读者判过宽度的字节"
        );
        writer.put(&self.mapping_key);
        writer.assert_position(
            u64::try_from(DEADLIST_ENTRY_KEY_BYTES).expect("43"),
            "deadlist 条目",
        );
        let key = writer.into_bytes();
        assert(key@ =~= self.spec_bytes());
        key
    }

    /// 读者：条目宽不等于 43 交回 `None`（`parse_index_node` 只判了条目宽 ≥ key 宽）。宽于 43 的同样不收
    /// （I-1.10（码 2 条目宽等于字段表宽） 写的是「等于」，欠账 C780、C867）。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> (entry: Option<Self>)
        ensures
            match entry {
                Some(parsed) => Self::spec_parse_fields(bytes@) == Some(
                    (parsed.head, parsed.epoch, parsed.mapping_key@),
                ),
                None => Self::spec_parse_fields(bytes@) is None,
            },
    {
        if bytes.len() != DEADLIST_ENTRY_KEY_BYTES {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        let head = TreeIdentifier(reader.get_u64());
        let epoch = CheckpointTxg(reader.get_u64());
        Some(Self {
            head,
            epoch,
            mapping_key: copied_into_a_vector(&bytes[16..DEADLIST_ENTRY_KEY_BYTES]),
        })
    }

    /// 这条条目指的那个单元的出生 txg（映射 key 的出生 txg 段）。
    ///
    /// # Panics
    /// 映射 key 不是 27 字节（`requires` 排除的那一种）：同 [`Self::key_bytes`]。
    #[must_use]
    pub fn birth_txg(&self) -> (birth_txg: CheckpointTxg)
        requires
            self.mapping_key@.len() == MAPPING_KEY_BYTES,
        ensures
            birth_txg.0 == little_endian_value(self.mapping_key@.subrange(9, 17)),
    {
        proof {
            lemma_little_endian_value_fits(self.mapping_key@.subrange(9, 17));
            lemma_little_endian_capacities();
        }
        birth_txg_of_a_mapping_key(&self.mapping_key)
            .expect("deadlist 条目里的映射 key 是 27 字节：取自写者现算的 key 或读者判过宽度的字节")
    }
}

/// 树表条目里根指针的起点：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（`singlefs_format::TREE_TABLE_ENTRY_BYTES` 的字段表）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES: usize = 8 + 2 + 2 + 2;

/// 树表条目 200：树 ID 8（打头 = key）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeTableEntry {
    pub kind: u16,
    pub tree: TreeIdentifier,
    pub root: NodePointer,
    /// D5（快照 / 空间记账机制） 已定项 9：写在两处——活头 inode 树那条条目（= 最近一个快照的 S.txg，还没打过快照写 0）与每条码 14
    /// 快照条目（= 捕获那一刻活头的值）；其余条目写 0（0 = 不适用）。`parse` 照盘上读进来、`to_bytes` 照它写。
    pub previous_snapshot_txg: CheckpointTxg,
    pub birth_txg: CheckpointTxg,
    /// D5（快照 / 空间记账机制） 已定项 9：这棵树归哪个可写头；无归属写 0。
    pub head_identifier: u64,
}

impl TreeTableEntry {
    /// 写得出的一条：根指针装得进、位置条目次序写者肯写，「销毁中」只写在码 14 上。
    pub open spec fn is_writable_with(self, mark: SnapshotDestructionMark) -> bool {
        &&& self.root.is_encodable()
        &&& self.root.has_location_entries_the_writer_accepts()
        &&& (mark == SnapshotDestructionMark::NotBeingDestroyed || self.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
    }

    /// 盘上 200 字节，flags 写 `mark` 的值。
    pub open spec fn spec_bytes_with_destruction_mark(self, mark: SnapshotDestructionMark) -> Seq<u8> {
        little_endian_bytes(self.tree.0 as nat, 8) + little_endian_bytes(200, 2) + little_endian_bytes(self.kind as nat, 2)
            + little_endian_bytes(mark.spec_flags() as nat, 2) + self.root.spec_bytes() + little_endian_bytes(
            self.previous_snapshot_txg.0 as nat,
            8,
        ) + little_endian_bytes(self.birth_txg.0 as nat, 8) + little_endian_bytes(self.head_identifier as nat, 8)
            + zero_bytes(76)
    }

    /// [`Self::parse_with_destruction_mark`] 的判法。
    pub open spec fn spec_parse_with_destruction_mark(bytes: Seq<u8>) -> Option<(Self, SnapshotDestructionMark)> {
        let kind = little_endian_value(bytes.subrange(10, 12)) as u16;
        let flags = little_endian_value(bytes.subrange(12, 14)) as u16;
        if bytes.len() != TREE_TABLE_ENTRY_BYTES || bytes.subrange(124, 200) != zero_bytes(76) {
            None
        } else if little_endian_value(bytes.subrange(8, 10)) != TREE_TABLE_ENTRY_BYTES {
            None
        } else if !(flags == 0 || (flags == TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED && kind
            == TREE_KIND_SNAPSHOT_INODE_ROOT)) {
            None
        } else {
            match NodePointer::spec_read_judging_at(bytes, 14) {
                Err(_refusal) => None,
                Ok(root) => Some(
                    (
                        TreeTableEntry {
                            kind,
                            tree: TreeIdentifier(little_endian_value(bytes.subrange(0, 8)) as u64),
                            root,
                            previous_snapshot_txg: CheckpointTxg(little_endian_value(bytes.subrange(100, 108)) as u64),
                            birth_txg: CheckpointTxg(little_endian_value(bytes.subrange(108, 116)) as u64),
                            head_identifier: little_endian_value(bytes.subrange(116, 124)) as u64,
                        },
                        if flags == 0 {
                            SnapshotDestructionMark::NotBeingDestroyed
                        } else {
                            SnapshotDestructionMark::BeingDestroyed
                        },
                    ),
                ),
            }
        }
    }

    /// 往返：带记号写出的 200 字节读回同一条条目与同一个记号。
    pub proof fn lemma_parse_round_trip(self, mark: SnapshotDestructionMark)
        requires
            self.is_writable_with(mark),
        ensures
            Self::spec_parse_with_destruction_mark(self.spec_bytes_with_destruction_mark(mark)) == Some((self, mark)),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes_with_destruction_mark(mark);
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        let tree = little_endian_bytes(self.tree.0 as nat, 8);
        let length = little_endian_bytes(200, 2);
        let kind = little_endian_bytes(self.kind as nat, 2);
        let flags = little_endian_bytes(mark.spec_flags() as nat, 2);
        let root = self.root.spec_bytes();
        let previous = little_endian_bytes(self.previous_snapshot_txg.0 as nat, 8);
        let birth = little_endian_bytes(self.birth_txg.0 as nat, 8);
        let head = little_endian_bytes(self.head_identifier as nat, 8);
        let reserved = zero_bytes(76);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind + flags + root + previous + birth + head, reserved);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind + flags + root + previous + birth, head);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind + flags + root + previous, birth);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind + flags + root, previous);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind + flags, root);
        lemma_split_a_concatenated_range(bytes, 0, tree + length + kind, flags);
        lemma_split_a_concatenated_range(bytes, 0, tree + length, kind);
        lemma_split_a_concatenated_range(bytes, 0, tree, length);
        self.root.lemma_decode_at_round_trip(bytes, 14);
    }

    /// flags 写 0（不带「销毁中」的记号）。
    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        requires
            self.is_writable_with(SnapshotDestructionMark::NotBeingDestroyed),
        ensures
            bytes@ == self.spec_bytes_with_destruction_mark(SnapshotDestructionMark::NotBeingDestroyed),
    {
        self.to_bytes_with_destruction_mark(SnapshotDestructionMark::NotBeingDestroyed)
    }

    /// flags 写 `mark` 的值。
    ///
    /// # Panics
    /// 给不是码 14 的条目写「销毁中」（`requires` 排除的那一种）：读者对别的种类上的这一位拒收（[`Self::parse_with_destruction_mark`]），
    /// 写者只给被删的那条码 14 写。
    #[must_use]
    pub fn to_bytes_with_destruction_mark(&self, mark: SnapshotDestructionMark) -> (bytes: Vec<u8>)
        requires
            self.is_writable_with(mark),
        ensures
            bytes@ == self.spec_bytes_with_destruction_mark(mark),
    {
        // 记号的 == 没有 Verus 规格，比 flags 的值：两种记号的值不同，同一个判法。
        assert!(
            mark.flags() == SnapshotDestructionMark::NotBeingDestroyed.flags()
                || self.kind == TREE_KIND_SNAPSHOT_INODE_ROOT,
            "「销毁中」只写在码 14 条目上（判决 S4）"
        );
        let mut writer = ByteWriter::new(usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
        writer.put_u64(self.tree.0);
        writer.put_u16(u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
        writer.put_u16(self.kind);
        writer.put_u16(mark.flags());
        writer.assert_position(
            u64::try_from(TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES).expect("14"),
            "树表条目的根指针",
        );
        self.root.write_to(&mut writer);
        writer.put_u64(self.previous_snapshot_txg.0);
        writer.put_u64(self.birth_txg.0);
        writer.put_u64(self.head_identifier);
        writer.skip(76);
        writer.assert_position(TREE_TABLE_ENTRY_BYTES, "树表条目");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes_with_destruction_mark(mark));
        bytes
    }

    /// 一条条目盘上字节里根指针那 86 字节的原样（逐字节比两条根指针用；解析成 `NodePointer` 会丢掉第一版恒 0 的 MAC / nonce 那几段）。
    /// 调用方要先用 `parse` 核过这是一条合法的 200 字节条目。
    ///
    /// # Panics
    /// 短于 100 字节（`requires` 排除的那一种）。
    #[must_use]
    pub fn root_pointer_bytes(entry_bytes: &[u8]) -> (root_pointer: &[u8])
        requires
            entry_bytes@.len() >= 100,
        ensures
            root_pointer@ == entry_bytes@.subrange(14, 100),
    {
        &entry_bytes[TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES
            ..TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES
                + usize::try_from(NODE_POINTER_BYTES).expect("86")]
    }

    /// 读者：条目长度、flags 未知位（D8（核心索引结构） 已定项 8 ㊁：一律拒收）、预留 76 非零、根指针头部有一段不是第一版的取值
    /// （MAC / nonce 非 0、算法类型不是未加密、压缩算法码不认识、码 0 而压后长度非 0、extent 偏移非 0：D19（块指针的结构与宽度预算） 已定项 3、
    /// D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据），判该指针所在的结构——这条条目——损坏）都判不可用。
    /// 码 14 条目 flags 位 0（销毁中）收下、丢掉记号；要记号的读者用 [`Self::parse_with_destruction_mark`]。
    #[must_use]
    #[allow(
        clippy::manual_map,
        reason = "Option::map 带闭包时 Verus 交不出结果的规格（vstd 的规格只说调了闭包），写成 match 才证得出交回的就是那条条目"
    )]
    pub fn parse(bytes: &[u8]) -> (entry: Option<Self>)
        ensures
            entry == match Self::spec_parse_with_destruction_mark(bytes@) {
                Some((parsed, _mark)) => Some(parsed),
                None => None,
            },
    {
        // Option::map 带闭包时 Verus 交不出结果的规格，写成 match：丢掉记号、照旧交回条目。
        match Self::parse_with_destruction_mark(bytes) {
            Some((entry, _mark)) => Some(entry),
            None => None,
        }
    }

    /// 同 [`Self::parse`]，连同 flags 读出来的「销毁中」记号：flags 是 0 ⇒ `NotBeingDestroyed`；码 14 条目上 flags 恰是位 0 ⇒ `BeingDestroyed`；
    /// 别的取值（别的位、别的种类上的位 0）照 D8（核心索引结构） 已定项 8 ㊁拒收。
    #[must_use]
    pub fn parse_with_destruction_mark(bytes: &[u8]) -> (entry: Option<(Self, SnapshotDestructionMark)>)
        ensures
            entry == Self::spec_parse_with_destruction_mark(bytes@),
    {
        if bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
            || !every_byte_is_zero(&bytes[bytes.len() - 76..])
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 0);
        let tree = TreeIdentifier(reader.get_u64());
        if u64::from(reader.get_u16()) != TREE_TABLE_ENTRY_BYTES {
            return None;
        }
        let kind = reader.get_u16();
        // 按 flags 的值分的 match（常量模式加守卫、余下的拒收）写成 if：Verus 不认按常量匹配，三种结论不变。
        let flags = reader.get_u16();
        let mark = if flags == 0 {
            SnapshotDestructionMark::NotBeingDestroyed
        } else if flags == TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
            && kind == TREE_KIND_SNAPSHOT_INODE_ROOT
        {
            SnapshotDestructionMark::BeingDestroyed
        } else {
            return None;
        };
        // Result::ok 接 ? 写成 match：头部判不过的根指针与别的拒收同一个处置。
        let root =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(root) => root,
                Err(_refusal) => return None,
            };
        let previous_snapshot_txg = CheckpointTxg(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        let head_identifier = reader.get_u64();
        Some((
            Self {
                kind,
                tree,
                root,
                previous_snapshot_txg,
                birth_txg,
                head_identifier,
            },
            mark,
        ))
    }
}
/// 一条带指针的条目（inode 内部条目 120、extent 叶记录 112）解不开是哪一种。封闭集合，`match` 不写通配臂。
/// 两种都是盘上读来的字节、不是我们的不变量：调用方把这条条目当损坏处置，不拿里面的指针去解引用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerCarryingEntryRefusal {
    /// 条目宽（索引节点头里的盘上字段）不等于字段表（窄于、宽于都算：I-1.10（码 2 条目宽等于字段表宽） 写的是「等于」，
    /// 欠账 C867）：调用方报「条目宽与字段表不符」。
    WidthOtherThanItsFieldTable,
    /// 条目里那条指针的头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移）：判该指针所在的结构
    /// （这条条目）损坏（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
    PointerHeadFieldOutsideTheFirstVersion(PointerHeadFieldOutsideTheFirstVersion),
}

/// [`parse_inode_internal_entry_or_refusal`] 的判法。
pub open spec fn spec_parse_inode_internal_entry(
    bytes: Seq<u8>,
) -> Result<(u64, PackedIdentity, NodePointer), PointerCarryingEntryRefusal> {
    if bytes.len() != INODE_INTERNAL_ENTRY {
        Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable)
    } else {
        match NodePointer::spec_read_judging_at(bytes, 34) {
            Err(refusal) => Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(refusal)),
            Ok(child) => Ok(
                (
                    little_endian_value(bytes.subrange(0, 8)) as u64,
                    PackedIdentity::spec_decode_at(bytes, 8),
                    child,
                ),
            ),
        }
    }
}

/// 往返：写出的 inode 内部条目读回同样的三样。
pub proof fn lemma_inode_internal_entry_round_trip(separator_key: u64, child_identity: PackedIdentity, child: NodePointer)
    requires
        child.is_encodable(),
    ensures
        spec_parse_inode_internal_entry(spec_inode_internal_entry_bytes(separator_key, child_identity, child)) == Ok::<
            (u64, PackedIdentity, NodePointer),
            PointerCarryingEntryRefusal,
        >((separator_key, child_identity, child)),
{
    lemma_little_endian_capacities();
    let bytes = spec_inode_internal_entry_bytes(separator_key, child_identity, child);
    assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
    let separator = little_endian_bytes(separator_key as nat, 8);
    lemma_split_a_concatenated_range(bytes, 0, separator + child_identity.spec_bytes(), child.spec_bytes());
    lemma_split_a_concatenated_range(bytes, 0, separator, child_identity.spec_bytes());
    child_identity.lemma_decode_at_round_trip(bytes, 8);
    child.lemma_decode_at_round_trip(bytes, 34);
}

/// inode 内部条目 120 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 8），
/// 不等于字段表宽（窄于、宽于都算）、或子指针头部有一段不是第一版的取值时返回 `None`：这里是盘上字节进字段表的边界，往里就按 120 信它。
/// 要分清是哪一种的调用方（冷启动走读、只读挂载打开池）用 [`parse_inode_internal_entry_or_refusal`]；只要知道解不解得开的（用例）用这一个。
#[must_use]
pub fn parse_inode_internal_entry(bytes: &[u8]) -> (entry: Option<(u64, PackedIdentity, NodePointer)>)
    ensures
        entry == match spec_parse_inode_internal_entry(bytes@) {
            Ok(parsed) => Some(parsed),
            Err(_refusal) => None,
        },
{
    parse_inode_internal_entry_or_refusal(bytes).ok()
}

/// 同 [`parse_inode_internal_entry`]，解不开时交回是哪一种。
///
/// # Errors
/// 不等于 120（窄于、宽于都算，欠账 C867）⇒ `WidthOtherThanItsFieldTable`；子指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
pub fn parse_inode_internal_entry_or_refusal(
    bytes: &[u8],
) -> (entry: Result<(u64, PackedIdentity, NodePointer), PointerCarryingEntryRefusal>)
    ensures
        entry == spec_parse_inode_internal_entry(bytes@),
{
    if bytes.len() != usize::try_from(INODE_INTERNAL_ENTRY).expect("120") {
        return Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable);
    }
    let mut reader = ByteReader::at(bytes, 0);
    let separator_key = reader.get_u64();
    let child_identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    // Result::map_err 接 ? 写成 match：头部判不过的照旧包成 PointerHeadFieldOutsideTheFirstVersion。
    let child = match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
        &mut reader,
    ) {
        Ok(child) => child,
        Err(refusal) => {
            return Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(refusal))
        }
    };
    Ok((separator_key, child_identity, child))
}

/// 往返：写出的 extent 叶记录里，key 那 24 字节是三段 key 的小端、指针读回同一条数据指针。
pub proof fn lemma_extent_record_round_trip(inode: u64, offset: u64, pointer: DataPointer)
    requires
        pointer.is_encodable(),
    ensures
        spec_extent_record_bytes(inode, offset, pointer).subrange(0, 24) == little_endian_bytes(0, 8)
            + little_endian_bytes(inode as nat, 8) + little_endian_bytes(offset as nat, 8),
        DataPointer::spec_read_judging_at(spec_extent_record_bytes(inode, offset, pointer), 24) == Ok::<
            DataPointer,
            PointerHeadFieldOutsideTheFirstVersion,
        >(pointer),
{
    let bytes = spec_extent_record_bytes(inode, offset, pointer);
    assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
    let key = little_endian_bytes(0, 8) + little_endian_bytes(inode as nat, 8) + little_endian_bytes(offset as nat, 8);
    lemma_split_a_concatenated_range(bytes, 0, key, pointer.spec_bytes());
    pointer.lemma_decode_at_round_trip(bytes, 24);
}

/// extent 叶记录 112 的读者：key 24 + 指针 88（同 [`parse_inode_internal_entry_or_refusal`]：`parse_index_node` 只判了条目宽 ≥ key 宽 24，
/// 切到偏移 112 之前要在这里判一次；数据指针头部第一版之外的取值也在这里判）。
///
/// # Errors
/// 不等于 112（窄于、宽于都算，欠账 C867）⇒ `WidthOtherThanItsFieldTable`；数据指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
pub fn parse_extent_record(
    bytes: &[u8],
) -> (record: Result<([u8; 24], DataPointer), PointerCarryingEntryRefusal>)
    ensures
        match record {
            Ok((key, pointer)) => {
                &&& bytes@.len() == EXTENT_LEAF_RECORD_BYTES
                &&& key@ == bytes@.subrange(0, 24)
                &&& DataPointer::spec_read_judging_at(bytes@, 24) == Ok::<DataPointer, PointerHeadFieldOutsideTheFirstVersion>(pointer)
            },
            Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable) => bytes@.len() != EXTENT_LEAF_RECORD_BYTES,
            Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(refusal)) => {
                &&& bytes@.len() == EXTENT_LEAF_RECORD_BYTES
                &&& DataPointer::spec_read_judging_at(bytes@, 24) == Err::<DataPointer, PointerHeadFieldOutsideTheFirstVersion>(refusal)
            },
        },
{
    if bytes.len() != usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {
        return Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable);
    }
    // 切片转定长数组的 try_into 没有 Verus 规格，经 copied_into_an_array 拷，同一个结果。
    let key: [u8; 24] = copied_into_an_array(&bytes[..24]);
    // Result::map_err 接 ? 写成 match：头部判不过的照旧包成 PointerHeadFieldOutsideTheFirstVersion。
    let pointer =
        match DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(bytes, 24),
        ) {
            Ok(pointer) => pointer,
            Err(refusal) => {
                return Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(refusal))
            }
        };
    Ok((key, pointer))
}

/// 往返：写出的映射条目读回同一把 key 与同两条位置条目。
pub proof fn lemma_mapping_entry_round_trip(key: Seq<u8>, locations: [LocationEntry; 2])
    requires
        key.len() == MAPPING_KEY_BYTES,
        locations[0].is_encodable(),
        locations[1].is_encodable(),
    ensures
        spec_mapping_entry_bytes(key, locations).subrange(0, 27) == key,
        LocationEntry::spec_decode_at(spec_mapping_entry_bytes(key, locations), 27) == locations[0],
        LocationEntry::spec_decode_at(spec_mapping_entry_bytes(key, locations), 41) == locations[1],
{
    let bytes = spec_mapping_entry_bytes(key, locations);
    assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
    lemma_split_a_concatenated_range(bytes, 0, key, spec_location_entries_bytes(locations));
    lemma_split_a_concatenated_range(bytes, 27, locations[0].spec_bytes(), locations[1].spec_bytes());
    locations[0].lemma_decode_at_round_trip(bytes, 27);
    locations[1].lemma_decode_at_round_trip(bytes, 41);
}

/// 映射条目 55 的读者：key 27 + 位置条目 14 × 2。条目宽窄于 55 时返回 `None`
/// （同 [`parse_inode_internal_entry`] 与 [`parse_extent_record`]：`parse_index_node` 只判了条目宽 ≥ key 宽，
/// 切到偏移 27 与 55 之前要在这里判一次；panic 面普查 R2）。
///
/// **判的是「今天这个解析器要几个字节」，不判条目宽是不是正好 55**：key 一律 27（码 2 / 码 3 的 25 字节末尾补零）、
/// 条目一律 55 已由 D19（块指针的结构与宽度预算） 已定项 10 定，落在 [`MAPPING_KEY_BYTES`] 与 [`MAPPING_ENTRY_BYTES`] 上，
/// 这一判跟着那两个常量走，不自己写死一个宽度。宽于 55 的条目这里不拒，拒它欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
#[must_use]
pub fn parse_mapping_entry(bytes: &[u8]) -> (entry: Option<(Vec<u8>, [LocationEntry; 2])>)
    ensures
        match entry {
            Some((key, locations)) => {
                &&& bytes@.len() >= MAPPING_ENTRY_BYTES
                &&& key@ == bytes@.subrange(0, 27)
                &&& locations[0] == LocationEntry::spec_decode_at(bytes@, 27)
                &&& locations[1] == LocationEntry::spec_decode_at(bytes@, 41)
            },
            None => bytes@.len() < MAPPING_ENTRY_BYTES,
        },
{
    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {
        return None;
    }
    // `to_vec` 没有 Verus 规格，经 copied_into_a_vector 拷，同一个结果。
    let key = copied_into_a_vector(&bytes[..usize::try_from(MAPPING_KEY_BYTES).expect("27")]);
    let mut reader = ByteReader::at(bytes, key.len());
    Some((
        key,
        [
            LocationEntry::read_from(&mut reader),
            LocationEntry::read_from(&mut reader),
        ],
    ))
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::SlotNumber;
    use singlefs_format::{
        TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
        TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
        TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS, TREE_IDENTIFIER_RETIREMENT,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
        TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    };

    /// 树的种类码与树 ID 的值域不相交（D8（核心索引结构） 已定项 9 射程、已定项 11）：
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
            TREE_KIND_RETIREMENT,
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
            TREE_IDENTIFIER_RETIREMENT,
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
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
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
            previous_snapshot_txg: CheckpointTxg(0),
            birth_txg: CheckpointTxg(3),
            head_identifier: TREE_IDENTIFIER_INODE,
        };
        assert_eq!(table_entry.to_bytes().len(), 200);
        assert_eq!(
            TreeTableEntry::parse(&table_entry.to_bytes()),
            Some(table_entry)
        );
    }

    /// D5（快照 / 空间记账机制） 已定项 9：`previous_snapshot_txg` 写在条目偏移 100 那 8 字节（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 +
    /// 根指针 86 之后、诞生 txg 之前），`to_bytes` 照结构里的值写、`parse` 照盘上读回来；两个 txg 字段各自独立，换一个不动另一个。
    #[test]
    fn previous_snapshot_txg_is_written_at_offset_one_hundred_and_read_back_apart_from_the_birth_txg(
    ) {
        let entry = TreeTableEntry {
            kind: TREE_KIND_INODE,
            tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
            root: NodePointer::empty_root(),
            previous_snapshot_txg: CheckpointTxg(0x0102_0304),
            birth_txg: CheckpointTxg(0x0506),
            head_identifier: TREE_IDENTIFIER_INODE,
        };
        let bytes = entry.to_bytes();
        let field_at = |offset: usize| {
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
        };
        assert_eq!(
            field_at(100),
            0x0102_0304,
            "偏移 100：previous_snapshot_txg"
        );
        assert_eq!(field_at(108), 0x0506, "偏移 108：诞生 txg");
        assert_eq!(
            TreeTableEntry::parse(&bytes).map(|parsed| parsed.previous_snapshot_txg),
            Some(CheckpointTxg(0x0102_0304)),
            "读回来是写进去的值，不是 0"
        );
        assert_eq!(TreeTableEntry::parse(&bytes), Some(entry), "逐字段读回相等");
    }

    /// 删快照的意图只占码 14 条目 flags 位 0：写进偏移 12 那 2 字节、
    /// 读回来记号相同、别的字段不动；码 14 上别的位、别的种类上的位 0 读者拒收。
    #[test]
    fn the_destruction_mark_sits_in_flags_bit_zero_of_a_snapshot_inode_root_entry_only() {
        let snapshot_inode_root = TreeTableEntry {
            kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
            tree: TreeIdentifier(65549),
            root: NodePointer::empty_root(),
            previous_snapshot_txg: CheckpointTxg(4),
            birth_txg: CheckpointTxg(9),
            head_identifier: TREE_IDENTIFIER_INODE,
        };
        let marked = snapshot_inode_root
            .to_bytes_with_destruction_mark(SnapshotDestructionMark::BeingDestroyed);
        assert_eq!(
            u16::from_le_bytes([marked[12], marked[13]]),
            TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED,
            "偏移 12：flags 位 0"
        );
        assert_eq!(
            TreeTableEntry::parse_with_destruction_mark(&marked),
            Some((snapshot_inode_root, SnapshotDestructionMark::BeingDestroyed)),
            "读回来带「销毁中」，别的字段与写进去的相同"
        );
        assert_eq!(
            TreeTableEntry::parse_with_destruction_mark(&snapshot_inode_root.to_bytes()),
            Some((
                snapshot_inode_root,
                SnapshotDestructionMark::NotBeingDestroyed
            )),
            "不带记号的那一份读回来不带"
        );
        let mut another_bit = marked.clone();
        another_bit[12] = 2;
        assert_eq!(
            TreeTableEntry::parse(&another_bit),
            None,
            "码 14 上 flags 别的位：拒收"
        );
        let snapshot_extent_root = TreeTableEntry {
            kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
            tree: TreeIdentifier(65550),
            root: NodePointer::empty_root(),
            previous_snapshot_txg: CheckpointTxg(0),
            birth_txg: CheckpointTxg(9),
            head_identifier: 65549,
        };
        let mut bit_zero_on_another_kind = snapshot_extent_root.to_bytes();
        bit_zero_on_another_kind[12] = 1;
        assert_eq!(
            TreeTableEntry::parse(&bit_zero_on_another_kind),
            None,
            "码 15 上 flags 位 0：拒收"
        );
    }

    /// deadlist 条目 43 字节：头的树 ID 8 + 纪元 8 + 中央映射 key 27（D5（快照 / 空间记账机制） 已定项 12），读回来逐字段相等；
    /// 出生 txg 取映射 key 偏移 9 那 8 字节，不取纪元。
    #[test]
    fn a_deadlist_entry_is_the_head_the_epoch_and_the_mapping_key_and_reads_back() {
        let mapping_key = mapping_key_for_node(
            2,
            NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                    birth_txg: CheckpointTxg(7),
                },
                locations: NodePointer::empty_root().locations,
                instance: InstanceGeneration(1),
                birth_sequence: BirthSequence(3),
            },
        );
        let entry = DeadlistEntry {
            head: TreeIdentifier(TREE_IDENTIFIER_INODE),
            epoch: CheckpointTxg(11),
            mapping_key: mapping_key.clone(),
        };
        let bytes = entry.key_bytes();
        assert_eq!(bytes.len(), DEADLIST_ENTRY_KEY_BYTES, "43 字节");
        assert_eq!(
            &bytes[..8],
            &TREE_IDENTIFIER_INODE.to_le_bytes(),
            "偏移 0：头的树 ID"
        );
        assert_eq!(&bytes[8..16], &11u64.to_le_bytes(), "偏移 8：纪元");
        assert_eq!(
            &bytes[16..],
            mapping_key.as_slice(),
            "偏移 16：中央映射 key"
        );
        assert_eq!(
            DeadlistEntry::parse(&bytes),
            Some(entry.clone()),
            "读回来逐字段相等"
        );
        assert_eq!(
            entry.birth_txg(),
            CheckpointTxg(7),
            "出生 txg 取映射 key 那一段"
        );
        assert_eq!(DeadlistEntry::parse(&bytes[..42]), None, "窄于 43：解不开");
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

    /// 退役记录 34 字节：出生树 0、打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26，小端、无填充；读回逐字段相等，
    /// 长度不是 34 读不出（短一字节、长一字节都不收）。身份四段与 inode 内部条目里身份引用那 26 字节逐字节相同。
    #[test]
    fn a_retirement_record_is_thirty_four_bytes_identity_in_entry_order_then_the_death_txg() {
        let record = RetirementRecord {
            retired: PackedIdentity {
                birth_tree: TreeIdentifier(0x0102),
                record_type: 2,
                container: 0x0304,
                container_birth: CheckpointTxg(0x0506),
            },
            death_txg: CheckpointTxg(0x0708),
        };
        let bytes = record.to_bytes();
        assert_eq!(
            bytes,
            [
                vec![0x02, 0x01, 0, 0, 0, 0, 0, 0],
                vec![2, 0],
                vec![0x04, 0x03, 0, 0, 0, 0, 0, 0],
                vec![0x06, 0x05, 0, 0, 0, 0, 0, 0],
                vec![0x08, 0x07, 0, 0, 0, 0, 0, 0],
            ]
            .concat(),
            "出生树 0、类型 8、容器号 10、出生代 18、死亡 txg 26，小端"
        );
        let entry = build_inode_internal_entry(9, record.retired, NodePointer::empty_root());
        assert_eq!(
            &bytes[..26],
            &entry[8..34],
            "身份四段与 inode 内部条目里的身份引用同序同宽"
        );
        assert_eq!(RetirementRecord::parse(&bytes), Some(record));
        assert_eq!(RetirementRecord::parse(&bytes[..33]), None);
        assert_eq!(
            RetirementRecord::parse(&[bytes.as_slice(), &[0]].concat()),
            None
        );
    }

    fn record_with_the_truncation_field(
        truncation_field: InodeRecordTruncationField,
    ) -> InodeRecord {
        InodeRecord {
            inode: 1,
            object_birth: CheckpointTxg(3),
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 100,
            change_count: 9,
            write_time_seconds: 1_788_000_000,
            truncation_field,
        }
    }

    /// T 写在偏移 120 起 8 字节、N_w 写在 128 起 8 字节，都是小端，读者从同一处读回来；两段各取一个每个字节都不同的值，
    /// 写错位置、读错位置、读成 0、两段对调都对不上。之前 108–119、之后 136–139 两段仍是 0。
    #[test]
    fn truncation_field_is_written_little_endian_at_offsets_120_and_128_and_read_back() {
        let record = record_with_the_truncation_field(
            InodeRecordTruncationField::written_by_the_truncation_intent_of(
                CheckpointTxg(0x0807_0605_0403_0201),
                0x1817_1615_1413_1211,
            ),
        );
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 140);
        assert_eq!(
            bytes[120..128],
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            "T 在偏移 120 起 8 字节、小端"
        );
        assert_eq!(
            bytes[128..136],
            [0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18],
            "N_w 在偏移 128 起 8 字节、小端"
        );
        assert!(
            bytes[108..120]
                .iter()
                .chain(&bytes[136..])
                .all(|byte| *byte == 0),
            "108–119 与 136–139 仍恒 0"
        );
        assert_eq!(InodeRecord::parse(&bytes), Some(record), "读回来逐字段相同");
        let not_written = record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN);
        assert!(
            not_written.to_bytes()[108..].iter().all(|byte| *byte == 0),
            "没写过的字段：偏移 108 起整段都是 0"
        );
    }

    /// 合法性只判「T = 0 ⇒ N_w = 0」。T = 0 而 N_w ≠ 0 拒；截到 0 是 T ≠ 0、N_w = 0，收。
    #[test]
    fn parse_refuses_a_truncation_field_whose_txg_is_zero_while_its_size_is_not() {
        let mut size_without_txg =
            record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
        size_without_txg[128] = 5;
        assert_eq!(
            InodeRecord::parse(&size_without_txg),
            None,
            "T = 0、N_w = 5：不合法"
        );
        let truncated_to_zero = record_with_the_truncation_field(
            InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(7), 0),
        );
        assert_eq!(
            InodeRecord::parse(&truncated_to_zero.to_bytes()).map(|parsed| parsed.truncation_field),
            Some(
                InodeRecordTruncationField::written_by_the_truncation_intent_of(
                    CheckpointTxg(7),
                    0
                )
            ),
            "截到 0：T = 7、N_w = 0 合法，读回来 T 仍是 7"
        );
        assert_eq!(
            InodeRecordTruncationField::from_the_two_fields_on_disk(0, 5),
            None
        );
    }

    /// 截断字段两边那两段恒零区（108–119、136–139）任一字节非零都拒，截断字段那 16 字节里的非零（T 非零）收。
    #[test]
    fn parse_refuses_a_nonzero_byte_in_either_always_zero_area_around_the_truncation_field() {
        let bytes =
            record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
        for offset in (108..120).chain(136..140) {
            let mut tampered = bytes.clone();
            tampered[offset] = 1;
            assert_eq!(
                InodeRecord::parse(&tampered),
                None,
                "偏移 {offset} 在恒零区里，非零拒收"
            );
        }
        for offset in 120..128 {
            let mut txg_byte_set = bytes.clone();
            txg_byte_set[offset] = 1;
            assert!(
                InodeRecord::parse(&txg_byte_set).is_some(),
                "偏移 {offset} 是 T 的一个字节：T 非零、N_w = 0 合法"
            );
        }
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
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
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

    /// C480（inode 记录的 blocks 怎么算全仓没有条款）（D8（核心索引结构） 已定项 6）：`blocks` = ⌈size ÷ 512⌉，
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
                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
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
            Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable),
            "extent 叶记录该有 112 字节，条目宽缩到 key 宽 24 时不解"
        );
        assert_eq!(
            parse_extent_record(&[0u8; 111]),
            Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable),
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
