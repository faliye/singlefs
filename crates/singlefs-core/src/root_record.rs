//! 根记录（D22（单元原子性怎么合成） 已定项 7）：466 字节住一个判定宽度（physical_block_size）的根槽，
//! 自证校验和罩整槽含补齐、自身按 0 参与。行序即盘上顺序，偏移按行累加。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）：[`RootRecord::to_slot`] 交出的就是
//! [`RootRecord::spec_slot_bytes`]，[`RootRecord::parse_slot`] 交出的就是 [`RootRecord::spec_parse_slot`]，往返引理
//! [`RootRecord::lemma_parse_slot_round_trip`] 证写出来的槽读回同一条根记录。

use singlefs_format::{
    NODE_POINTER_BYTES, ROOT_RECORD_BYTES, ROOT_RECORD_NO_RING_SLOT_PINNED, WIDE_CHECKSUM_BYTES,
};
use vstd::prelude::*;

use crate::address::{CheckpointTxg, InstanceGeneration};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::checksum::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::pointer::*;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::ROOT_RECORD_BYTES ] -> (root_record_bytes: u64) returns 466u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::ROOT_RECORD_NO_RING_SLOT_PINNED ] -> (no_slot_pinned: u8) returns 255u8;

/// 根记录的 magic「SFSR」（写成四个字节值：Verus 不认字节串字面量）。
pub const ROOT_MAGIC: [u8; 4] = [0x53, 0x46, 0x53, 0x52];
/// magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 138。
pub const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8;
/// flags 位 0 = 卸载记号：正常卸载那一串写的根置 1，其余根写 0（D22（单元原子性怎么合成） 已定项 7）；
/// 位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 13）。其余位第一版恒 0、非 0 拒收（已定项 17）。
/// 内存里是 [`RootRecord::unmount_marker`]（[`UnmountMarker`]），只有 `mount::unmount` 那一串写 1。
pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1;

/// 根记录 flags 位 0 的卸载记号（D22（单元原子性怎么合成） 已定项 7）：这条根是不是正常卸载那一串空发布写的
/// （D16（发布语义） 已定项 1「正常卸载」）。池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分两支判；
/// 「记号只由卸载入口打」盘上没有别的东西核它，归崩溃点重放录制流（C557（卸载记号只由卸载入口打没有检查））。
/// 由记录施加出来的那一版不归下面哪一个成员固定：它取那条记录新根段里的记号（D23（journal 的角色与格式） 已定项 15，`recovery` 施加处），
/// 记录的新根段照抄那次发布写的根上的记号——卸载那一串发布的记录带 1、施加出来是 [`UnmountMarker::WrittenByTheUnmountSequence`]，
/// 别的发布的记录带 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnmountMarker {
    /// flags 位 0 = 1：正常卸载那一串写的根（`mount::unmount`）。
    WrittenByTheUnmountSequence,
    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退。
    NotWrittenByTheUnmountSequence,
}

impl UnmountMarker {
    /// 写进 flags 那 4 字节的值（[`Self::flags_bits`] 的规格）。
    pub open spec fn spec_flags_bits(self) -> u32 {
        match self {
            UnmountMarker::WrittenByTheUnmountSequence => ROOT_RECORD_FLAG_UNMOUNT_MARKER,
            UnmountMarker::NotWrittenByTheUnmountSequence => 0,
        }
    }

    /// 只带记号那一位的 flags 读回的记号（[`Self::of_flags_bits_carrying_only_the_marker`] 的规格）。
    pub open spec fn spec_of_flags_bits_carrying_only_the_marker(flags: u32) -> Self {
        if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
            UnmountMarker::NotWrittenByTheUnmountSequence
        } else {
            UnmountMarker::WrittenByTheUnmountSequence
        }
    }

    /// 记号那 4 字节读回的记号：位 0 之外有位为 1 ⇒ `None`（[`Self::of_unmount_marker_field_of_a_new_root_segment`] 的规格）。
    pub open spec fn spec_of_unmount_marker_field(field: u32) -> Option<Self> {
        if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
            None
        } else {
            Some(Self::spec_of_flags_bits_carrying_only_the_marker(field))
        }
    }

    /// 往返：写进 flags 的值读回同一个记号。
    pub proof fn lemma_flags_bits_round_trip(self)
        ensures
            Self::spec_of_unmount_marker_field(self.spec_flags_bits()) == Some(self),
    {
        assert(1u32 & !1u32 == 0u32 && 1u32 & 1u32 != 0u32 && 0u32 & !1u32 == 0u32 && 0u32 & 1u32 == 0u32)
            by (bit_vector);
    }

    /// 写进 flags 那 4 字节的值：记号那一位，其余位恒 0。
    #[must_use]
    pub const fn flags_bits(self) -> (bits: u32)
        ensures
            bits == self.spec_flags_bits(),
    {
        match self {
            UnmountMarker::WrittenByTheUnmountSequence => ROOT_RECORD_FLAG_UNMOUNT_MARKER,
            UnmountMarker::NotWrittenByTheUnmountSequence => 0,
        }
    }

    /// journal 记录新根段里的卸载记号那 4 字节（D23（journal 的角色与格式） 已定项 15）：与根记录 flags 同一个编码，位 0 之外有位为 1 ⇒ `None`
    /// （读者把那条记录当损坏，与根记录 flags、记录标志「其余位读到非 0」同一个处置）。
    #[must_use]
    pub const fn of_unmount_marker_field_of_a_new_root_segment(field: u32) -> (marker: Option<Self>)
        ensures
            marker == Self::spec_of_unmount_marker_field(field),
    {
        if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
            return None;
        }
        Some(Self::of_flags_bits_carrying_only_the_marker(field))
    }

    /// 从已经判过「记号之外的位全是 0」的 flags 读回记号。
    #[must_use]
    const fn of_flags_bits_carrying_only_the_marker(flags: u32) -> (marker: Self)
        ensures
            marker == Self::spec_of_flags_bits_carrying_only_the_marker(flags),
    {
        if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
            UnmountMarker::NotWrittenByTheUnmountSequence
        } else {
            UnmountMarker::WrittenByTheUnmountSequence
        }
    }
}

/// 根记录与 journal 新根段里的钉槽 p（D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」，C773（读坏与全零根槽的回收门槛钉只在内存里））：
/// 建这一版的账时还钉着的根环槽里按（钉值, 槽）最小的那一槽，存那一槽的编码 `3·槽 + 区域`
/// （0 到 3S 之内与 `root_ring::target_for_publish` 落到它的最小 txg 相同，一槽一个编码）。钉值是那一槽可能住过的最旧 txg：槽号为 0 的槽
/// （mkfs 种第 0 代根的那三槽）是第 0 代根的 txg 0，别的槽就是编码本身
/// 三个槽 0 的钉值都是 0，p 存钉值就还原不出钉的是哪一槽，所以存槽不存钉值。
/// 没有钉着的槽写 [`ROOT_RECORD_NO_RING_SLOT_PINNED`]。
///
/// 盘上那 1 字节原样：core 的读者不判它的取值（core 只在施加记录拼根时把它原样带进拼出的根，不拿它建状态），
/// 取值判在池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinnedRingSlot(pub u8);

impl PinnedRingSlot {
    /// 建账时一个槽都没钉着；mkfs 的第 0 代也写它。
    pub const NO_SLOT_PINNED: Self = Self(ROOT_RECORD_NO_RING_SLOT_PINNED);

    /// 钉着的槽里按（钉值, 槽）最小的那一槽的编码 `3·槽 + 区域`（`None` = 一个都没钉着）写成的字节（`allocator::LowestPinnedRingSlot`）。
    ///
    /// # Panics
    /// 编码装不进一个字节、或撞上「没有钉着」那个值：编码小于环长 3S（S ≤ 16，`singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM`），
    /// 装得进也撞不上；装不进说明传进来的不是槽的编码。
    #[must_use]
    ///
    /// `requires` 就是这两种 panic 的反面。
    pub fn of_the_lowest_pin(lowest_pinned_ring_slot_encoding: Option<u64>) -> (pin: Self)
        requires
            lowest_pinned_ring_slot_encoding matches Some(encoding) ==> encoding < ROOT_RECORD_NO_RING_SLOT_PINNED,
        ensures
            match lowest_pinned_ring_slot_encoding {
                None => pin.0 == ROOT_RECORD_NO_RING_SLOT_PINNED,
                Some(encoding) => pin.0 == encoding,
            },
    {
        match lowest_pinned_ring_slot_encoding {
            None => Self::NO_SLOT_PINNED,
            Some(encoding) => {
                // Option::filter 带闭包没有 Verus 规格：装得进一个字节与撞不上 255 拆成两步判，panic 的那一句不变。
                let pin = u8::try_from(encoding)
                    .expect("钉着的槽的编码 3·槽 + 区域小于环长 3S ≤ 48：装得进一个字节、撞不上「没有钉着」的 255");
                assert!(
                    pin != ROOT_RECORD_NO_RING_SLOT_PINNED,
                    "钉着的槽的编码 3·槽 + 区域小于环长 3S ≤ 48：装得进一个字节、撞不上「没有钉着」的 255"
                );
                Self(pin)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRecord {
    pub filesystem_identifier: [u8; 16],
    /// flags 位 0（D22（单元原子性怎么合成） 已定项 7）；flags 其余位第一版恒 0，不进内存。
    pub unmount_marker: UnmountMarker,
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
    pub tree_table: NodePointer,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
    pub instance_table: NodePointer,
    /// 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
    pub mapping_root: NodePointer,
    /// 树表 0 条的那一版的分配记录树的根（随 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 加）。
    ///
    /// **只有树表 0 条的那一版用这一项**：它没有树表条目可放（往树表里写一条，`tree_table_has_no_entries` 当场翻面，
    /// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变）。带文件的一版恒 [`NodePointer::empty_root`]
    /// ——那一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8），两处都写就成了同一个量的两份手抄。
    /// mkfs 的第 0 代也恒全零：那一版的账由 `mount::format_time_allocator` 从实例表与树表两条指针直接算。
    pub allocation_record_tree_root: NodePointer,
    /// 回收门槛 t（D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」，C773（读坏与全零根槽的回收门槛钉只在内存里））：
    /// 带单元的发布写 max(这条根自己带的 F, 分配器记这条根时的回收门槛)，采样在回收之后、装根之前
    /// （`PoolAllocator::record_root_written_by_this_process` 交回的那一对）；零单元发布照抄上一版的；mkfs 第 0 代写 0；
    /// 由记录施加出来的那一版取那条记录新根段里的。池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 在建账的发布上求等。
    pub reclaim_threshold: CheckpointTxg,
    /// 钉槽 p，写法同 `reclaim_threshold`（同一处采样、同样照抄、同样从记录新根段带进拼出的根）。
    pub pinned_ring_slot: PinnedRingSlot,
}

impl RootRecord {
    /// 四条指针的槽号都装得进 6 字节、位置条目的次序写者肯写：[`Self::to_slot`] 写得出它。
    pub open spec fn is_encodable(self) -> bool {
        &&& self.tree_table.is_encodable() && self.tree_table.has_location_entries_the_writer_accepts()
        &&& self.instance_table.is_encodable() && self.instance_table.has_location_entries_the_writer_accepts()
        &&& self.mapping_root.is_encodable() && self.mapping_root.has_location_entries_the_writer_accepts()
        &&& self.allocation_record_tree_root.is_encodable()
            && self.allocation_record_tree_root.has_location_entries_the_writer_accepts()
    }

    /// 自证校验和之前那 138 字节：magic、fsid、flags、实例代号、checkpoint_txg、树表指针、树 ID 水位、回退下界 F。
    pub open spec fn spec_bytes_before_the_checksum(self) -> Seq<u8> {
        ROOT_MAGIC@ + self.filesystem_identifier@ + little_endian_bytes(self.unmount_marker.spec_flags_bits() as nat, 4)
            + little_endian_bytes(self.instance.0 as nat, 4) + little_endian_bytes(self.checkpoint_txg.0 as nat, 8)
            + self.tree_table.spec_bytes() + little_endian_bytes(self.tree_identifier_watermark as nat, 8)
            + little_endian_bytes(self.rollback_floor.0 as nat, 8)
    }

    /// 自证校验和之后那 296 字节：实例表、中央映射树、分配记录树三条指针，回收门槛 t、钉槽 p、算法类型 0、nonce 与 MAC 28 字节 0。
    pub open spec fn spec_bytes_after_the_checksum(self) -> Seq<u8> {
        self.instance_table.spec_bytes() + self.mapping_root.spec_bytes() + self.allocation_record_tree_root.spec_bytes()
            + little_endian_bytes(self.reclaim_threshold.0 as nat, 8) + little_endian_bytes(self.pinned_ring_slot.0 as nat, 1)
            + little_endian_bytes(0, 1) + zero_bytes(28)
    }

    /// 校验和字段还是 0 的整槽：记录 466 字节、之后补 0 到槽宽。
    pub open spec fn spec_unsealed_slot_bytes(self, slot_bytes: nat) -> Seq<u8> {
        self.spec_bytes_before_the_checksum() + zero_bytes(WIDE_CHECKSUM_BYTES as nat) + self.spec_bytes_after_the_checksum()
            + zero_bytes((slot_bytes - ROOT_RECORD_BYTES) as nat)
    }

    /// [`Self::to_slot`] 写出的整槽：校验和字段换成罩整槽、自身按 0 算出来的那 32 字节。
    pub open spec fn spec_slot_bytes(self, slot_bytes: nat) -> Seq<u8> {
        let unsealed = self.spec_unsealed_slot_bytes(slot_bytes);
        unsealed.subrange(0, ROOT_CHECKSUM_OFFSET as int) + spec_wide_checksum_field(
            unsealed,
            slot_bytes as int,
            ROOT_CHECKSUM_OFFSET as int,
        ) + unsealed.subrange(ROOT_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES, slot_bytes as int)
    }

    /// [`Self::parse_slot`] 的判法：magic、整槽校验和、fsid、flags、四条指针头部五关，过了才按字段表读出来。
    pub open spec fn spec_parse_slot(bytes: Seq<u8>, expected_filesystem_identifier: [u8; 16]) -> Option<Self> {
        let flags = little_endian_value(bytes.subrange(20, 24)) as u32;
        if bytes.len() < ROOT_RECORD_BYTES || bytes.subrange(0, 4) != ROOT_MAGIC@ {
            None
        } else if bytes.subrange(ROOT_CHECKSUM_OFFSET as int, ROOT_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES)
            != spec_wide_checksum_field(bytes, bytes.len() as int, ROOT_CHECKSUM_OFFSET as int) {
            None
        } else if bytes.subrange(4, 20) != expected_filesystem_identifier@ {
            None
        } else if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
            None
        } else if NodePointer::spec_read_judging_at(bytes, 36) is Err
            || NodePointer::spec_read_judging_at(bytes, 170) is Err
            || NodePointer::spec_read_judging_at(bytes, 256) is Err
            || NodePointer::spec_read_judging_at(bytes, 342) is Err {
            None
        } else {
            Some(RootRecord {
                filesystem_identifier: expected_filesystem_identifier,
                unmount_marker: UnmountMarker::spec_of_flags_bits_carrying_only_the_marker(flags),
                instance: InstanceGeneration(little_endian_value(bytes.subrange(24, 28)) as u32),
                checkpoint_txg: CheckpointTxg(little_endian_value(bytes.subrange(28, 36)) as u64),
                tree_table: NodePointer::spec_decode_at(bytes, 36),
                tree_identifier_watermark: little_endian_value(bytes.subrange(122, 130)) as u64,
                rollback_floor: CheckpointTxg(little_endian_value(bytes.subrange(130, 138)) as u64),
                instance_table: NodePointer::spec_decode_at(bytes, 170),
                mapping_root: NodePointer::spec_decode_at(bytes, 256),
                allocation_record_tree_root: NodePointer::spec_decode_at(bytes, 342),
                reclaim_threshold: CheckpointTxg(little_endian_value(bytes.subrange(428, 436)) as u64),
                pinned_ring_slot: PinnedRingSlot(little_endian_value(bytes.subrange(436, 437)) as u8),
            })
        }
    }

    /// 往返：写出的槽带着这条根记录的 fsid 读回同一条根记录。
    pub proof fn lemma_parse_slot_round_trip(self, slot_bytes: nat)
        requires
            self.is_encodable(),
            slot_bytes >= ROOT_RECORD_BYTES,
        ensures
            Self::spec_parse_slot(self.spec_slot_bytes(slot_bytes), self.filesystem_identifier) == Some(self),
    {
        lemma_little_endian_capacities();
        self.unmount_marker.lemma_flags_bits_round_trip();
        let unsealed = self.spec_unsealed_slot_bytes(slot_bytes);
        let sealed = self.spec_slot_bytes(slot_bytes);
        let before = self.spec_bytes_before_the_checksum();
        let after = self.spec_bytes_after_the_checksum();
        assert(before.len() == 138);
        assert(after.len() == 296);
        assert(unsealed.len() == slot_bytes);
        let digest = spec_wide_checksum_field(unsealed, slot_bytes as int, 138);
        assert(digest.len() == 32);
        assert(sealed.len() == slot_bytes);
        // 校验和字段之外两截与没封的那一份逐字节相同，所以罩住的那一截（字段按 0）两份相同、算出来的校验和相同。
        assert(sealed.subrange(0, 138) =~= before);
        assert(sealed.subrange(138, 170) =~= digest);
        assert(sealed.subrange(170, 466) =~= after);
        assert(spec_covered_with_the_field_zeroed(sealed, slot_bytes as int, 138)
            =~= spec_covered_with_the_field_zeroed(unsealed, slot_bytes as int, 138));
        // 前 138 字节逐段剥开。
        let magic = ROOT_MAGIC@;
        let fsid = self.filesystem_identifier@;
        let flags = little_endian_bytes(self.unmount_marker.spec_flags_bits() as nat, 4);
        let instance = little_endian_bytes(self.instance.0 as nat, 4);
        let checkpoint = little_endian_bytes(self.checkpoint_txg.0 as nat, 8);
        let tree_table = self.tree_table.spec_bytes();
        let watermark = little_endian_bytes(self.tree_identifier_watermark as nat, 8);
        let rollback = little_endian_bytes(self.rollback_floor.0 as nat, 8);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint + tree_table + watermark, rollback);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint + tree_table, watermark);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint, tree_table);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance, checkpoint);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags, instance);
        lemma_split_a_concatenated_range(sealed, 0, magic + fsid, flags);
        lemma_split_a_concatenated_range(sealed, 0, magic, fsid);
        // 后 296 字节逐段剥开。
        let instance_table = self.instance_table.spec_bytes();
        let mapping_root = self.mapping_root.spec_bytes();
        let allocation_root = self.allocation_record_tree_root.spec_bytes();
        let reclaim = little_endian_bytes(self.reclaim_threshold.0 as nat, 8);
        let pinned = little_endian_bytes(self.pinned_ring_slot.0 as nat, 1);
        let algorithm_type = little_endian_bytes(0, 1);
        lemma_split_a_concatenated_range(
            sealed,
            170,
            instance_table + mapping_root + allocation_root + reclaim + pinned + algorithm_type,
            zero_bytes(28),
        );
        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root + reclaim + pinned, algorithm_type);
        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root + reclaim, pinned);
        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root, reclaim);
        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root, allocation_root);
        lemma_split_a_concatenated_range(sealed, 170, instance_table, mapping_root);
        self.tree_table.lemma_decode_at_round_trip(sealed, 36);
        self.instance_table.lemma_decode_at_round_trip(sealed, 170);
        self.mapping_root.lemma_decode_at_round_trip(sealed, 256);
        self.allocation_record_tree_root.lemma_decode_at_round_trip(sealed, 342);
        assert(Self::spec_parse_slot(sealed, self.filesystem_identifier).unwrap().filesystem_identifier
            == self.filesystem_identifier);
    }

    /// 写成一个 `slot_bytes` 宽的槽；记录 466 字节，其余补 0。
    ///
    /// # Panics
    /// 槽宽装不下 466 字节，或四条指针有一条写者不肯写（`requires` 排除的两种）。
    #[must_use]
    pub fn to_slot(&self, slot_bytes: usize) -> (bytes: Vec<u8>)
        requires
            slot_bytes >= ROOT_RECORD_BYTES,
            self.is_encodable(),
        ensures
            bytes@ == self.spec_slot_bytes(slot_bytes as nat),
    {
        assert!(
            slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("466"),
            "根槽装不下根记录"
        );
        let mut writer = ByteWriter::new(slot_bytes);
        writer.put(&ROOT_MAGIC);
        writer.put(&self.filesystem_identifier);
        writer.put_u32(self.unmount_marker.flags_bits());
        writer.put_u32(self.instance.0);
        writer.put_u64(self.checkpoint_txg.0);
        self.tree_table.write_to(&mut writer);
        writer.put_u64(self.tree_identifier_watermark);
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(
            u64::try_from(ROOT_CHECKSUM_OFFSET).expect("138"),
            "根记录自证校验和",
        );
        writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        self.instance_table.write_to(&mut writer);
        self.mapping_root.write_to(&mut writer);
        self.allocation_record_tree_root.write_to(&mut writer);
        // 回收门槛 t 与钉槽 p 接在分配记录树根指针之后、算法类型之前：偏移 428、436。
        writer.put_u64(self.reclaim_threshold.0);
        writer.put_u8(self.pinned_ring_slot.0);
        writer.put_u8(0); // 算法类型：未加密
        writer.skip(12 + 16); // nonce、MAC：第一版留位全 0
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_unsealed_slot_bytes(slot_bytes as nat));
        let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_CHECKSUM_OFFSET);
        copy_bytes_into(&mut bytes, ROOT_CHECKSUM_OFFSET, &digest);
        assert(bytes@ =~= self.spec_slot_bytes(slot_bytes as nat));
        bytes
    }

    /// 读者：magic、整槽校验和、fsid、flags、四条指针头部五关。flags 只收卸载记号那一位（[`ROOT_RECORD_FLAG_UNMOUNT_MARKER`]），
    /// 别的位非 0 拒收；记号照实读进 [`RootRecord::unmount_marker`]（挂载与恢复不按它分支，按它分支的是池级 checker 的 I-7.9）。
    /// 树表、实例表、中央映射树、分配记录树四条指针头部里加密、压缩与 extent 偏移那几段按第一版的取值判，与 journal 新根段那四条指针
    /// 同一判法（`NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from`）：MAC / nonce 非 0、算法类型不是未加密、
    /// 压缩算法码不是 0、码 0 而压后长度非 0、extent 偏移非 0，任一条判这条根记录损坏、拒收（I-2.4（头校验和覆盖范围）「加密关着时恒 0、
    /// 读者遇到非 0 一律判该结构损坏」，D19（块指针的结构与宽度预算） 已定项 3，C581（extent 偏移非 0 没有判据），
    /// `pointer::PointerHeadFieldOutsideTheFirstVersion`）：拒收的根与自证不过的根同一个处置，择根时不可择。
    #[must_use]
    pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> (root: Option<Self>)
        ensures
            root == Self::spec_parse_slot(bytes@, *expected_filesystem_identifier),
    {
        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("466")
            || bytes[..4] != ROOT_MAGIC
        {
            proof {
                if bytes@.len() >= 466 {
                    assert(bytes@.subrange(0, 4) != ROOT_MAGIC@);
                }
            }
            return None;
        }
        assert(bytes@.subrange(0, 4) =~= ROOT_MAGIC@);
        if !wide_checksum_field_holds(bytes, bytes.len(), ROOT_CHECKSUM_OFFSET) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let filesystem_identifier: [u8; 16] = copied_into_an_array(reader.take(16));
        if filesystem_identifier != *expected_filesystem_identifier {
            proof {
                assert(bytes@.subrange(4, 20) != expected_filesystem_identifier@);
            }
            return None;
        }
        assert(bytes@.subrange(4, 20) =~= expected_filesystem_identifier@);
        let flags = reader.get_u32();
        if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
            return None;
        }
        let unmount_marker = UnmountMarker::of_flags_bits_carrying_only_the_marker(flags);
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        // Result::ok 接 ? 写成 match：拒收的那一种与校验和不过同一个处置（交回 None）。
        let tree_table =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        let instance_table =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let mapping_root =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let allocation_record_tree_root =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        assert!(
            reader.position()
                == ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86"),
            "根记录读到回收门槛 t 之前停在偏移 428"
        );
        let reclaim_threshold = CheckpointTxg(reader.get_u64());
        let pinned_ring_slot = PinnedRingSlot(reader.get_u8());
        Some(Self {
            filesystem_identifier: *expected_filesystem_identifier,
            unmount_marker,
            instance,
            checkpoint_txg,
            tree_table,
            tree_identifier_watermark,
            rollback_floor,
            instance_table,
            mapping_root,
            allocation_record_tree_root,
            reclaim_threshold,
            pinned_ring_slot,
        })
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_root_record() -> RootRecord {
        RootRecord {
            filesystem_identifier: [5u8; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: CheckpointTxg(0),
            pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    #[test]
    fn root_record_is_466_bytes_with_the_checksum_at_138_and_round_trips() {
        let slot = sample_root_record().to_slot(512);
        assert_eq!(slot.len(), 512);
        assert!(slot[466..].iter().all(|byte| *byte == 0));
        assert_eq!(&slot[..4], b"SFSR");
        assert_eq!(
            RootRecord::parse_slot(&slot, &[5u8; 16]),
            Some(sample_root_record())
        );
        assert_eq!(
            RootRecord::parse_slot(&slot, &[6u8; 16]),
            None,
            "fsid 不符不可择"
        );
        let mut torn = slot.clone();
        torn[500] ^= 1;
        assert_eq!(
            RootRecord::parse_slot(&torn, &[5u8; 16]),
            None,
            "补齐区改一字节：自证校验和罩整槽"
        );
    }

    /// 回收门槛 t 与钉槽 p（C773（读坏与全零根槽的回收门槛钉只在内存里））：t 8 字节小端在 [428, 436)、
    /// p 1 字节在 436，算法类型挪到 437、nonce [438, 450)、MAC [450, 466) 照旧全 0；两项各取一个与别的字段都不同的值，写得出、读得回。
    /// 偏移按字段表手算、写成字面量，不从写者用的常量推。
    #[test]
    fn the_reclaim_threshold_and_the_pinned_ring_slot_sit_after_the_allocation_record_tree_root_and_round_trip(
    ) {
        let root = RootRecord {
            reclaim_threshold: CheckpointTxg(0x0102_0304_0506_0708),
            pinned_ring_slot: PinnedRingSlot(7),
            ..sample_root_record()
        };
        let slot = root.to_slot(512);
        assert_eq!(
            &slot[428..436],
            &0x0102_0304_0506_0708u64.to_le_bytes(),
            "回收门槛 t 紧跟分配记录树根指针（[342, 428)），在 [428, 436)"
        );
        assert_eq!(slot[436], 7, "钉槽 p 在 436");
        assert!(
            slot[437..466].iter().all(|byte| *byte == 0),
            "算法类型 437、nonce [438, 450)、MAC [450, 466) 第一版全 0"
        );
        assert_eq!(RootRecord::parse_slot(&slot, &[5u8; 16]), Some(root));
        assert_eq!(
            sample_root_record().to_slot(512)[436],
            255,
            "没有钉着的槽写 255"
        );
    }

    /// 钉着的槽的编码写成钉槽字节：没有钉着写 255，区域 0 槽 2 写 6，区域 1 槽 0（钉值是第 0 代根的 0）写 1。
    #[test]
    fn the_pinned_ring_slot_byte_is_the_encoding_of_the_pinned_slot_or_255_when_nothing_is_pinned()
    {
        assert_eq!(PinnedRingSlot::of_the_lowest_pin(None), PinnedRingSlot(255));
        assert_eq!(
            PinnedRingSlot::of_the_lowest_pin(Some(6)),
            PinnedRingSlot(6)
        );
        assert_eq!(
            PinnedRingSlot::of_the_lowest_pin(Some(1)),
            PinnedRingSlot(1)
        );
    }

    /// 把槽里 flags 那 4 字节（偏移 20）改成 `flags`，自证校验和重封：magic、校验和、fsid 三关照样过，拒不拒只看 flags 判。
    fn slot_with_flags(flags: u32) -> Vec<u8> {
        let mut slot = sample_root_record().to_slot(512);
        slot[20..24].copy_from_slice(&flags.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&slot, 512, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        slot
    }

    /// flags 位 0 是卸载记号（D22（单元原子性怎么合成） 已定项 7）：带它的根照收、记号照实读进 `unmount_marker`、其余字段原样读回；
    /// 别的位（位 1、位 31、带着记号又多一位）非 0 照旧拒收。
    #[test]
    fn root_carrying_the_unmount_marker_is_accepted_and_any_other_flag_bit_is_refused() {
        assert_eq!(
            ROOT_RECORD_FLAG_UNMOUNT_MARKER, 0x0000_0001,
            "卸载记号是位 0"
        );
        let marked = slot_with_flags(ROOT_RECORD_FLAG_UNMOUNT_MARKER);
        assert_eq!(
            marked[20], 0x01,
            "位 0 住 flags 第一个字节的最低位（已定项 13）"
        );
        assert_eq!(
            RootRecord::parse_slot(&marked, &[5u8; 16]),
            Some(RootRecord {
                unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
                ..sample_root_record()
            }),
            "带卸载记号的根照收，记号读成卸载那一串写的，别的字段一个不差"
        );
        for refused_flags in [
            1u32 << 1,
            1u32 << 31,
            ROOT_RECORD_FLAG_UNMOUNT_MARKER | (1 << 1),
        ] {
            assert_eq!(
                RootRecord::parse_slot(&slot_with_flags(refused_flags), &[5u8; 16]),
                None,
                "flags {refused_flags:#010x}：卸载记号之外的位非 0 拒收"
            );
        }
    }

    /// 写者照 `unmount_marker` 写 flags：卸载那一串写的根 flags 是 1（只有位 0），别的根 flags 是 0；两样都原样读回。
    #[test]
    fn the_writer_puts_the_unmount_marker_into_flag_bit_zero_and_leaves_it_clear_on_every_other_root(
    ) {
        let marked_root = RootRecord {
            unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
            ..sample_root_record()
        };
        let marked_slot = marked_root.to_slot(512);
        assert_eq!(
            marked_slot[20..24],
            [0x01, 0x00, 0x00, 0x00],
            "卸载那一串写的根：flags 只有位 0"
        );
        assert_eq!(
            RootRecord::parse_slot(&marked_slot, &[5u8; 16]),
            Some(marked_root),
            "带记号写出去的根带记号读回来"
        );
        let unmarked_slot = sample_root_record().to_slot(512);
        assert_eq!(
            unmarked_slot[20..24],
            [0x00, 0x00, 0x00, 0x00],
            "别的根：flags 全 0"
        );
        assert_eq!(
            RootRecord::parse_slot(&unmarked_slot, &[5u8; 16]),
            Some(sample_root_record()),
            "不带记号写出去的根不带记号读回来"
        );
    }
}
