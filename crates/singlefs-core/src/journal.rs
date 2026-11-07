//! journal 记录 4096（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 15 / 已定项 17 / 已定项 18 / 已定项 19）：
//! 头 496 + 点名项 56 × N；`header_csum` 罩整条 [0, 4096) 含补齐、自身按 0 参与；`payload_csum` 罩点名项数组；
//! 反向链 = CRC-32C(本实例内逻辑前一条记录的 496 字节头，`header_csum` 按 0 参与)，本实例第一条恒 0。
//!
//! 除 [`JournalBytesToWrite`] 之外整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）：
//! [`JournalRecord::to_bytes`] 交出的就是写法规格 [`JournalRecord::spec_bytes`]；[`JournalRecord::parse`] 收下一条记录当且仅当
//! 读法规格 [`JournalRecord::spec_parse_accepts`] 成立，交回的记录逐字段等于那段字节按字段表读出来的样子
//! （[`JournalRecord::spec_matches_the_bytes`]）；往返引理 [`JournalRecord::lemma_parse_round_trip`] 证写出来的记录读者收、读回同一条。
//! 点名项、反向链、环内偏移与环末尾的下一个槽各有自己的规格。

use singlefs_format::{
    JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_NEW_ROOT_SEGMENT_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, MAPPING_KEY_BYTES, SLOT_BYTES,
    WIDE_CHECKSUM_BYTES,
};
use vstd::prelude::*;

use crate::address::{
    CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration, SlotNumber, TreeIdentifier,
};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::checksum::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::pointer::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::root_record::*;
// unit.rs 的规格与引理（spec_concatenation 一族）只在规格与证明里用，平时的构建把它们连同用处一起擦掉，所以按全路径写、不 use。

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::JOURNAL_RECORD_BYTES ] -> (record_bytes: u64) returns 4096u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::JOURNAL_HEADER_BYTES ] -> (header_bytes: u64) returns 496u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::JOURNAL_NAMED_ENTRY_BYTES ] -> (named_entry_bytes: u64) returns 56u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::JOURNAL_NEW_ROOT_SEGMENT_BYTES ] -> (new_root_segment_bytes: u64) returns 373u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::JOURNAL_RING_START_SLOT ] -> (ring_start_slot: u64) returns 1024u64;

/// journal 记录的 magic「SFSJ」（写成四个字节值：Verus 不认字节串字面量）。
pub const JOURNAL_MAGIC: [u8; 4] = [0x53, 0x46, 0x53, 0x4A];
/// 记录类型登记表（D23（journal 的角色与格式） 已定项 1）：0 无效、1 普通记录。
pub const JOURNAL_RECORD_TYPE_ORDINARY: u16 = 1;
/// magic 4 + 类型 2 + 算法类型 1 + 记录标志 1 + 记录长度 4 + 点名项数 4 + jsn 10 + checkpoint_txg 8 + nonce 12。
pub const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;
/// 记录标志 1 字节在 magic 4 + 类型 2 + 算法类型 1 之后（D23（journal 的角色与格式） 已定项 4，原「填充 1」那个字节）。
pub const JOURNAL_RECORD_FLAGS_OFFSET: usize = 4 + 2 + 1;
/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位写 0，读到非 0 当损坏（已定项 4）。
pub const JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;

/// 这条记录在它那次发布里是不是末条：记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
/// 一次发布的末条之外的记录写 0；每次只有一条记录的发布（含空发布记录）那一条也写 1。
/// 恢复按它认一次发布的边界（已定项 14 第六条），所选根覆盖的最后一条也按它认（已定项 14 注 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRecordPlaceInPublish {
    /// 位 0 = 1：本次发布末条。
    LastRecordOfThePublish,
    /// 位 0 = 0：这次发布在它之后还有记录。
    MoreRecordsOfThePublishFollow,
}

impl JournalRecordPlaceInPublish {
    /// 写进记录标志那 1 字节的值（[`Self::record_flags_byte`] 的规格）。
    pub open spec fn spec_record_flags_byte(self) -> u8 {
        match self {
            JournalRecordPlaceInPublish::LastRecordOfThePublish => JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH,
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => 0,
        }
    }

    /// 盘上那 1 字节读回的位置（[`Self::from_record_flags_byte`] 的规格）：0 与 1 之外 ⇒ `None`。
    pub open spec fn spec_of_record_flags_byte(record_flags_byte: u8) -> Option<Self> {
        if record_flags_byte == 0 {
            Some(JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow)
        } else if record_flags_byte == JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH {
            Some(JournalRecordPlaceInPublish::LastRecordOfThePublish)
        } else {
            None
        }
    }

    /// 往返：写进记录标志的值读回同一个位置。
    pub proof fn lemma_record_flags_byte_round_trip(self)
        ensures
            Self::spec_of_record_flags_byte(self.spec_record_flags_byte()) == Some(self),
    {
    }

    /// 写进记录标志那 1 字节的值：只用位 0，其余位恒 0。
    #[must_use]
    pub const fn record_flags_byte(self) -> (record_flags_byte: u8)
        ensures
            record_flags_byte == self.spec_record_flags_byte(),
    {
        match self {
            JournalRecordPlaceInPublish::LastRecordOfThePublish => {
                JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH
            }
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => 0,
        }
    }

    /// 从盘上读来的记录标志那 1 字节：位 0 之外有位为 1 ⇒ `None`（已定项 4「其余位写 0，读到非 0 当损坏」）。
    #[must_use]
    pub const fn from_record_flags_byte(record_flags_byte: u8) -> (place: Option<Self>)
        ensures
            place == Self::spec_of_record_flags_byte(record_flags_byte),
    {
        // 按常量的 match 臂写成 if / else if：0 与位 0 两个取值收，其余照旧当损坏。
        if record_flags_byte == 0 {
            Some(JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow)
        } else if record_flags_byte == JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH {
            Some(JournalRecordPlaceInPublish::LastRecordOfThePublish)
        } else {
            None
        }
    }
}

/// 本次发布内序号紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：头部校验和 32 之后。
pub const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize =
    JOURNAL_HEADER_CHECKSUM_OFFSET + 32 + 8 + 1;

/// 本次发布内序号（D23（journal 的角色与格式） 已定项 4，无符号 32 位，从 1 起）：一次发布切成 N 条记录时依次是 1..N，
/// 只有一条时是 1，空发布记录也写 1。所选根覆盖的最后一条读不出时，恢复靠它认出下一次发布的第一条（已定项 14 注 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalRecordOrdinalWithinPublish(pub u32);

impl JournalRecordOrdinalWithinPublish {
    /// 一次发布的第一条；只有一条记录的发布（空发布、写行、只有一个数据单元的写）那一条就是它。
    pub const FIRST: Self = Self(1);

    /// 一次发布里从 0 数第 `record_offset_in_this_publish` 条记录的序号：偏移 + 1。
    ///
    /// # Panics
    /// 偏移 + 1 装不进 32 位：各条发布路径在任何落盘动作之前按上限拒掉切出来的记录多于上限的发布
    /// （`transaction::journal_record_limit_of_one_publish`：在飞记录数上限与 2³² − 1 取小，代码审阅第 26 条），偏移 + 1 不超过它。
    /// `requires` 就是这一种 panic 的反面。
    #[must_use]
    pub fn of_record_at_offset(record_offset_in_this_publish: usize) -> (ordinal: Self)
        requires
            record_offset_in_this_publish < u32::MAX,
        ensures
            ordinal.0 == record_offset_in_this_publish + 1,
    {
        Self(u32::try_from(record_offset_in_this_publish + 1).expect(
            "一次发布切出来的记录条数在落盘之前按 transaction::journal_record_limit_of_one_publish（≤ 2³² − 1）拒过，序号装得进 32 位",
        ))
    }
}

/// 点名项 56（D23（journal 的角色与格式） 已定项 17）：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NamedUnit {
    pub locations: [LocationEntry; 2],
    pub unit_class: u8,
    pub birth_tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
    pub key_tail: [u8; 10],
}

/// 10 个字节拼成的定长数组（规格里从一段字节取回 key 尾段）：`bytes` 恰好 10 个字节时就是视图等于它的那一个数组。
pub open spec fn spec_ten_byte_array_of(bytes: Seq<u8>) -> [u8; 10] {
    choose|array: [u8; 10]| array@ == bytes
}

/// 一个 10 字节数组的视图取回的就是它自己（数组按逐元素相等判相等）。
pub proof fn lemma_ten_byte_array_of_a_view(array: [u8; 10])
    ensures
        spec_ten_byte_array_of(array@) == array,
{
    broadcast use vstd::array::group_array_axioms;
    let chosen = spec_ten_byte_array_of(array@);
    assert(exists|candidate: [u8; 10]| candidate@ == array@);
    assert(chosen@ == array@);
    assert forall|position: int| 0 <= position < 10 implies chosen[position] == array[position] by {
        assert(chosen[position] == chosen@[position]);
        assert(array[position] == array@[position]);
    }
    assert(chosen =~= array);
}

impl NamedUnit {
    /// 两条位置条目的槽号都装得进 6 字节：写者能把它写成盘上那 56 字节。
    pub open spec fn is_encodable(self) -> bool {
        location_entries_are_encodable(self.locations)
    }

    /// 盘上 56 字节：两条位置条目 28、类标签 1、出生树 8、出生 txg 8、key 尾段 10、flags 1（写 0）。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        spec_location_entries_bytes(self.locations) + little_endian_bytes(self.unit_class as nat, 1) + little_endian_bytes(
            self.birth_tree.0 as nat,
            8,
        ) + little_endian_bytes(self.birth_txg.0 as nat, 8) + self.key_tail@ + little_endian_bytes(0, 1)
    }

    /// 从 `source` 的 `position` 起读 56 字节解成的点名项（flags 那 1 字节读者不看）。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        NamedUnit {
            locations: [
                LocationEntry::spec_decode_at(source, position),
                LocationEntry::spec_decode_at(source, position + 14),
            ],
            unit_class: little_endian_value(source.subrange(position + 28, position + 29)) as u8,
            birth_tree: TreeIdentifier(little_endian_value(source.subrange(position + 29, position + 37)) as u64),
            birth_txg: CheckpointTxg(little_endian_value(source.subrange(position + 37, position + 45)) as u64),
            key_tail: spec_ten_byte_array_of(source.subrange(position + 45, position + 55)),
        }
    }

    /// 写成的点名项恰好 56 字节。
    pub proof fn lemma_bytes_have_the_entry_width(self)
        ensures
            self.spec_bytes().len() == JOURNAL_NAMED_ENTRY_BYTES,
    {
    }

    /// 往返：`position` 起那 56 字节是这个点名项写成的，就解回它。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            self.is_encodable(),
            0 <= position,
            position + 56 <= source.len(),
            source.subrange(position, position + 56) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
    {
        lemma_little_endian_capacities();
        let locations = spec_location_entries_bytes(self.locations);
        let class = little_endian_bytes(self.unit_class as nat, 1);
        let birth_tree = little_endian_bytes(self.birth_tree.0 as nat, 8);
        let birth_txg = little_endian_bytes(self.birth_txg.0 as nat, 8);
        let key_tail = self.key_tail@;
        let flags = little_endian_bytes(0, 1);
        lemma_split_a_concatenated_range(source, position, locations + class + birth_tree + birth_txg + key_tail, flags);
        lemma_split_a_concatenated_range(source, position, locations + class + birth_tree + birth_txg, key_tail);
        lemma_split_a_concatenated_range(source, position, locations + class + birth_tree, birth_txg);
        lemma_split_a_concatenated_range(source, position, locations + class, birth_tree);
        lemma_split_a_concatenated_range(source, position, locations, class);
        lemma_split_a_concatenated_range(source, position, self.locations[0].spec_bytes(), self.locations[1].spec_bytes());
        self.locations[0].lemma_decode_at_round_trip(source, position);
        self.locations[1].lemma_decode_at_round_trip(source, position + 14);
        lemma_ten_byte_array_of_a_view(self.key_tail);
        assert(Self::spec_decode_at(source, position).locations =~= self.locations);
    }

    /// 重放凑出的中央映射 key（[`Self::mapping_key`] 的规格）：类标签 1、出生树 8、出生 txg 8、key 尾段 10，共 27 字节。
    pub open spec fn spec_mapping_key(self) -> Seq<u8> {
        little_endian_bytes(self.unit_class as nat, 1) + little_endian_bytes(self.birth_tree.0 as nat, 8) + little_endian_bytes(
            self.birth_txg.0 as nat,
            8,
        ) + self.key_tail@
    }

    /// # Panics
    /// 写过缓冲末尾或位置条目的槽号装不进 6 字节（`requires` 排除的两种）。
    pub fn write_to(&self, writer: &mut ByteWriter)
        requires
            old(writer).has_room_for(JOURNAL_NAMED_ENTRY_BYTES as nat),
            self.is_encodable(),
        ensures
            final(writer).is_well_formed(),
            final(writer).written() == old(writer).written() + self.spec_bytes(),
            final(writer).total_bytes() == old(writer).total_bytes(),
    {
        let start = writer.position();
        // 两条位置条目的 for 循环写成两句（Verus 对数组迭代器的规格不全），次序不变。
        self.locations[0].write_to(writer);
        self.locations[1].write_to(writer);
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.put_u8(0);
        assert(writer.written() =~= old(writer).written() + self.spec_bytes());
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加一句话。
        assert!(
            writer.position() - start == usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56"),
            "点名项写了 56 字节"
        );
    }

    /// `requires` 里「被读的那一段不长过 usize」只为让末尾跳过 flags 那一字节时游标不越过 usize：被读的是一个切片，恒成立。
    pub fn read_from(reader: &mut ByteReader<'_>) -> (named: Self)
        requires
            old(reader).has_left(JOURNAL_NAMED_ENTRY_BYTES as nat),
            old(reader).source().len() <= usize::MAX,
        ensures
            final(reader).advanced_by(old(reader), JOURNAL_NAMED_ENTRY_BYTES as nat),
            named == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            named.is_encodable(),
    {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        let unit_class = reader.get_u8();
        let birth_tree = TreeIdentifier(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        // 切片转数组的 try_into 没有 Verus 规格：copied_into_an_array 拷成同样的 10 字节。
        let key_tail: [u8; 10] = copied_into_an_array(reader.take(10));
        reader.skip(1);
        proof {
            lemma_ten_byte_array_of_a_view(key_tail);
        }
        let named = Self {
            locations,
            unit_class,
            birth_tree,
            birth_txg,
            key_tail,
        };
        assert(named.locations =~= Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int).locations);
        named
    }

    /// 重放不查单元就能凑出这一项的中央映射 key（D23（journal 的角色与格式） 已定项 17 末句）。
    #[must_use]
    pub fn mapping_key(&self) -> (key: Vec<u8>)
        ensures
            key@ == self.spec_mapping_key(),
    {
        let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        let key = writer.into_bytes();
        assert(key@ =~= self.spec_mapping_key());
        key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalRecord {
    pub instance: InstanceGeneration,
    pub counter: u64,
    pub checkpoint_txg: CheckpointTxg,
    pub transaction: u64,
    pub is_commit: bool,
    pub ordinal_within_publish: JournalRecordOrdinalWithinPublish,
    /// 记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
    pub place_in_publish: JournalRecordPlaceInPublish,
    pub back_chain: u32,
    pub filesystem_identifier: u64,
    pub new_tree_table: NodePointer,
    pub new_mapping_root: NodePointer,
    pub new_tree_identifier_watermark: u64,
    pub new_rollback_floor: CheckpointTxg,
    /// 新根段后三项（D23（journal 的角色与格式） 已定项 15）：这次发布写的根上的实例表指针、
    /// 分配记录树根指针与卸载记号。施加一条记录时这三项与前四项一样换掉所选根的（`recovery::replay_journal` 拼新根那一截），不再照抄所选根。
    pub new_instance_table: NodePointer,
    pub new_allocation_record_tree_root: NodePointer,
    pub new_unmount_marker: UnmountMarker,
    /// 新根段末尾两项（C773（读坏与全零根槽的回收门槛钉只在内存里））：这次发布写的根上的回收门槛 t 与钉槽 p
    /// （`RootRecord::reclaim_threshold`、`RootRecord::pinned_ring_slot`）。施加一条记录拼根时与前七项一样原样带进拼出的根。
    pub new_reclaim_threshold: CheckpointTxg,
    pub new_pinned_ring_slot: PinnedRingSlot,
    pub named: Vec<NamedUnit>,
}

/// 反向链：前一条记录的整个记录头（[`JOURNAL_HEADER_BYTES`] = 496 字节），`header_csum` 那 32 字节按 0 参与。
///
/// # Panics
/// `previous_record_bytes` 短于一个记录头（`requires` 排除的那一种：调用方交的是读出来的整条前一条记录）。
#[must_use]
pub fn back_chain_of(previous_record_bytes: &[u8]) -> (back_chain: u32)
    requires
        previous_record_bytes@.len() >= JOURNAL_HEADER_BYTES,
    ensures
        back_chain == spec_crc32_castagnoli(
            spec_covered_with_the_field_zeroed(
                previous_record_bytes@,
                JOURNAL_HEADER_BYTES as int,
                JOURNAL_HEADER_CHECKSUM_OFFSET as int,
            ),
        ),
{
    // 原来是切出 496 字节 to_vec、再把校验和那 32 字节 fill(0)：to_vec 与 fill 没有 Verus 规格，经 ByteWriter 写成同样的 496 字节
    // （字段之前照抄、字段 32 字节写者 new 出来就是 0、字段之后照抄到记录头末尾）。
    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("496");
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    let mut header_writer = ByteWriter::new(header_bytes);
    header_writer.put(&previous_record_bytes[..JOURNAL_HEADER_CHECKSUM_OFFSET]);
    header_writer.skip(field_width);
    header_writer.put(&previous_record_bytes[JOURNAL_HEADER_CHECKSUM_OFFSET + field_width..header_bytes]);
    let header = header_writer.into_bytes();
    proof {
        assert(header@ =~= spec_covered_with_the_field_zeroed(
            previous_record_bytes@,
            JOURNAL_HEADER_BYTES as int,
            JOURNAL_HEADER_CHECKSUM_OFFSET as int,
        ));
    }
    crc32_castagnoli(&header)
}

/// jsn 计数器从 1 起、全池接着走（D23（journal 的角色与格式） 已定项 9 / 已定项 19）：计数器 0 没有记录。
pub const FIRST_JOURNAL_COUNTER: u64 = 1;

} // verus!

/// 一次或几次发布要写进 journal 环的字节（D16（发布语义） 已定项 5：脏量「按待发改动要写的 journal 字节算」）：
/// 记录条数乘一条记录的定长 [`JOURNAL_RECORD_BYTES`]，与有效 `T_dirty` = min(字段, 环长 ÷ F) 同一个量纲（环长也是 journal 字节）。
/// 数据字节、单元字节不是这个量，不进这个类型。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalBytesToWrite(pub u64);

impl JournalBytesToWrite {
    pub const ZERO: JournalBytesToWrite = JournalBytesToWrite(0);

    /// `records` 条定长记录占的 journal 字节。
    ///
    /// # Panics
    /// 乘回字节装不进 u64：一次发布的记录条数受在飞上限（环槽数 ÷ F）压着，环长本身装得进 u64，装不下说明条数不是一次发布的量。
    #[must_use]
    pub fn of_records(records: u64) -> Self {
        Self(
            records
                .checked_mul(JOURNAL_RECORD_BYTES)
                .expect("记录条数受在飞上限压着，乘 4096 装得进 u64"),
        )
    }

    /// 两份加起来（几次改动各自要写的 journal 字节之和）；装不下 u64 时停在 u64 的上沿：比阈值只看「到了没有」，上沿已经到了。
    #[must_use]
    pub fn plus(self, other: JournalBytesToWrite) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

verus! {

/// journal 环末尾的下一个 16 KiB 槽：环从 [`JOURNAL_RING_START_SLOT`] 起、长 `journal_ring_bytes`，末尾不落在槽边界上时取下一个整槽
/// （D3（空间分配） 已定项 10 ④「单元区起始槽号……第一版 = journal 环末尾的下一个槽」；D23（journal 的角色与格式） 已定项 19 ③
/// 「单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）」）。单元区起点由环长算只有这一处（`allocator::UnitAreaStart`）。
/// 环长是盘上读来的 8 字节时也不溢出：槽数 ≤ 2⁶⁴ ÷ 16384，加 1024 装得下。
///
/// `ensures` 说的就是「向上取整到槽」：从环起点数到交回的槽，罩得住整个环，少一个槽就罩不住（环长 0 时就是环起点）。
#[must_use]
pub fn slot_after_the_journal_ring(journal_ring_bytes: u64) -> (slot: SlotNumber)
    ensures
        slot.0 >= JOURNAL_RING_START_SLOT,
        (slot.0 - JOURNAL_RING_START_SLOT) * SLOT_BYTES >= journal_ring_bytes,
        slot.0 == JOURNAL_RING_START_SLOT || (slot.0 - JOURNAL_RING_START_SLOT - 1) * SLOT_BYTES < journal_ring_bytes,
{
    // 槽数 = ⌈环长 ÷ 16384⌉：u64::div_ceil 没有 Verus 规格，写成取商、有余数再加一，同一个数。
    let ring_slots = if journal_ring_bytes.is_multiple_of(SLOT_BYTES) {
        journal_ring_bytes / SLOT_BYTES
    } else {
        journal_ring_bytes / SLOT_BYTES + 1
    };
    proof {
        let quotient = journal_ring_bytes / SLOT_BYTES;
        let remainder = journal_ring_bytes % SLOT_BYTES;
        assert(quotient * 16384 + remainder == journal_ring_bytes && remainder < 16384) by (nonlinear_arith)
            requires
                quotient == journal_ring_bytes / 16384,
                remainder == journal_ring_bytes % 16384,
        ;
        assert(quotient <= 0x4_0000_0000_0000) by (nonlinear_arith)
            requires
                quotient == journal_ring_bytes / 16384,
                journal_ring_bytes < 0x1_0000_0000_0000_0000,
        ;
        if remainder == 0 {
            assert(ring_slots * 16384 == journal_ring_bytes);
            if ring_slots > 0 {
                assert((ring_slots - 1) * 16384 == ring_slots * 16384 - 16384) by (nonlinear_arith);
            }
        } else {
            assert(ring_slots * 16384 == quotient * 16384 + 16384) by (nonlinear_arith)
                requires
                    ring_slots == quotient + 1,
            ;
            assert((ring_slots - 1) * 16384 == quotient * 16384) by (nonlinear_arith)
                requires
                    ring_slots == quotient + 1,
            ;
        }
    }
    SlotNumber(JOURNAL_RING_START_SLOT + ring_slots)
}

/// 记录 n 在设备上的偏移（[`record_offset`] 的规格）：环起点之后 `(计数器 − 1) mod 槽数 × 4096`。
pub open spec fn spec_record_offset(counter: u64, ring_bytes: u64) -> int {
    JOURNAL_RING_START_SLOT * SLOT_BYTES + ((counter - FIRST_JOURNAL_COUNTER) as int % (ring_bytes / JOURNAL_RECORD_BYTES) as int)
        * JOURNAL_RECORD_BYTES
}

/// 记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`（D23（journal 的角色与格式） 已定项 18）。
///
/// # Panics
/// 环短于一条记录（环槽数 0）：mkfs 拒这种环长（`make_filesystem` 的 `JournalRingShorterThanOneRecord`），读者一侧择系统配置时
/// 拒装不下 F 条记录的环长（`recovery::choose_system_configuration` 报 `SystemConfigurationValueRefused`，代码审阅第 38 条）；
/// 发布写记录那一步拿的环长是 `PoolWriter` 的参数，可写挂载按择到的那份系统配置建它，与读者判过的是同一个值。
/// `counter` 是 0（计数器 0 没有记录，[`FIRST_JOURNAL_COUNTER`]）：核心层里拿盘上读来的计数器调这里的两处都先判过它不小于
/// [`FIRST_JOURNAL_COUNTER`]——`recovery` 判反向链时取「计数器小 1」的前一条（小 1 之后小于它就当前一条不在盘上），`mount` 读那一版末条记录
/// 所在那一槽时那一版计数器是 0 就不读；写者发的计数器是上一条加一。
/// 偏移装不进 u64（环长贴着 u64 上沿时环内偏移加上环起点会越界）。`requires` 就是这三种 panic 的反面。
#[must_use]
pub fn record_offset(counter: u64, ring_bytes: u64) -> (offset: DeviceOffsetInBytes)
    requires
        counter >= FIRST_JOURNAL_COUNTER,
        ring_bytes >= JOURNAL_RECORD_BYTES,
        spec_record_offset(counter, ring_bytes) <= u64::MAX,
    ensures
        offset.0 == spec_record_offset(counter, ring_bytes),
{
    let ring_slots = ring_bytes / JOURNAL_RECORD_BYTES;
    let records_before_this_one = counter.checked_sub(FIRST_JOURNAL_COUNTER).expect(
        "计数器从 1 起：拿盘上读来的计数器调这里的两处（recovery 判反向链、mount 读那一版末条记录）先判过它不小于 FIRST_JOURNAL_COUNTER",
    );
    let slot_in_ring = records_before_this_one.checked_rem(ring_slots).expect(
        "环至少装得下一条记录：mkfs 的 check_geometry 拒短于一条记录的环（JournalRingShorterThanOneRecord），择系统配置时读者拒装不下 F 条记录的环",
    );
    DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES + slot_in_ring * JOURNAL_RECORD_BYTES)
}

// ===== journal 记录的写法与读法 =====

/// 一条记录装得下的点名项数：(4096 − 496) ÷ 56 = 64。
pub open spec fn spec_named_units_one_record_holds() -> int {
    (JOURNAL_RECORD_BYTES as int - JOURNAL_HEADER_BYTES as int) / JOURNAL_NAMED_ENTRY_BYTES as int
}

/// 一串点名项各自写成的 56 字节。
pub open spec fn spec_named_unit_bytes_of(named: Seq<NamedUnit>) -> Seq<Seq<u8>> {
    named.map_values(|named_unit: NamedUnit| named_unit.spec_bytes())
}

/// 一串点名项写成的那几段都是 56 字节。
pub proof fn lemma_named_unit_bytes_have_the_entry_width(named: Seq<NamedUnit>)
    ensures
        crate::unit::spec_every_piece_has_width(spec_named_unit_bytes_of(named), JOURNAL_NAMED_ENTRY_BYTES as nat),
{
    // 点名项的写法规格在这里不展开：每一段的宽只取单项那条宽度引理交回的事实，少了那一步就当场报证明错。
    hide(NamedUnit::spec_bytes);
    assert forall|piece_index: int| 0 <= piece_index < named.len() implies (#[trigger] spec_named_unit_bytes_of(
        named,
    )[piece_index]).len() == JOURNAL_NAMED_ENTRY_BYTES by {
        named[piece_index].lemma_bytes_have_the_entry_width();
    }
}

impl JournalRecord {
    /// 写者写得出它：计数器装得进 6 字节、点名项不多于一条记录装得下的、点名项与新根段四条指针的位置条目都写得出。
    pub open spec fn is_encodable(self) -> bool {
        &&& self.counter < 0x1_0000_0000_0000
        &&& self.named@.len() <= spec_named_units_one_record_holds()
        &&& forall|named_index: int| 0 <= named_index < self.named@.len() ==> (#[trigger] self.named@[named_index]).is_encodable()
        &&& self.new_tree_table.is_encodable() && self.new_tree_table.has_location_entries_the_writer_accepts()
        &&& self.new_mapping_root.is_encodable() && self.new_mapping_root.has_location_entries_the_writer_accepts()
        &&& self.new_instance_table.is_encodable() && self.new_instance_table.has_location_entries_the_writer_accepts()
        &&& self.new_allocation_record_tree_root.is_encodable()
            && self.new_allocation_record_tree_root.has_location_entries_the_writer_accepts()
    }

    /// 提交标记那 1 字节：带写 1、不带写 0（D23（journal 的角色与格式） 已定项 7）。
    pub open spec fn spec_commit_marker(self) -> u8 {
        if self.is_commit {
            1
        } else {
            0
        }
    }

    /// 记录头 496 字节（两个校验和字段写 0），照写者的次序逐段接：magic 4、类型 2、算法类型 1、记录标志 1、记录长度 4、点名项数 4、
    /// 实例代号 4、计数器 6、checkpoint_txg 8、nonce 12、头部校验和 32、事务号 8、提交标记 1、本次发布内序号 4、反向链 4、载荷校验和 4、
    /// 新根段 373（树表指针 86、中央映射树根指针 86、树 ID 水位 8、回退下界 8、实例表指针 86、分配记录树根指针 86、卸载记号 4、
    /// 回收门槛 8、钉槽 1）、fsid 8、MAC 16。
    pub open spec fn spec_unsealed_header(self) -> Seq<u8> {
        JOURNAL_MAGIC@ + little_endian_bytes(JOURNAL_RECORD_TYPE_ORDINARY as nat, 2) + little_endian_bytes(0, 1)
            + little_endian_bytes(self.place_in_publish.spec_record_flags_byte() as nat, 1) + little_endian_bytes(
            JOURNAL_RECORD_BYTES as nat,
            4,
        ) + little_endian_bytes(self.named@.len(), 4) + little_endian_bytes(self.instance.0 as nat, 4)
            + little_endian_bytes(self.counter as nat, 6) + little_endian_bytes(self.checkpoint_txg.0 as nat, 8)
            + zero_bytes(12) + zero_bytes(WIDE_CHECKSUM_BYTES as nat) + little_endian_bytes(self.transaction as nat, 8)
            + little_endian_bytes(self.spec_commit_marker() as nat, 1) + little_endian_bytes(
            self.ordinal_within_publish.0 as nat,
            4,
        ) + little_endian_bytes(self.back_chain as nat, 4) + zero_bytes(4) + self.new_tree_table.spec_bytes()
            + self.new_mapping_root.spec_bytes() + little_endian_bytes(self.new_tree_identifier_watermark as nat, 8)
            + little_endian_bytes(self.new_rollback_floor.0 as nat, 8) + self.new_instance_table.spec_bytes()
            + self.new_allocation_record_tree_root.spec_bytes() + little_endian_bytes(
            self.new_unmount_marker.spec_flags_bits() as nat,
            4,
        ) + little_endian_bytes(self.new_reclaim_threshold.0 as nat, 8) + little_endian_bytes(
            self.new_pinned_ring_slot.0 as nat,
            1,
        ) + little_endian_bytes(self.filesystem_identifier as nat, 8) + zero_bytes(16)
    }

    /// 点名项那一截：各项的 56 字节依次接起来。
    pub open spec fn spec_named_area(self) -> Seq<u8> {
        crate::unit::spec_concatenation(spec_named_unit_bytes_of(self.named@))
    }

    /// 没封的整条：记录头、点名项、补 0 到 4096。
    pub open spec fn spec_unsealed_bytes(self) -> Seq<u8> {
        let header = self.spec_unsealed_header();
        let named_area = self.spec_named_area();
        header + named_area + zero_bytes((JOURNAL_RECORD_BYTES - header.len() - named_area.len()) as nat)
    }

    /// 写了载荷校验和的整条：偏移 95 那 4 字节换成罩点名项那一截的 CRC-32C 小端。
    pub open spec fn spec_bytes_with_the_payload_checksum(self) -> Seq<u8> {
        let unsealed = self.spec_unsealed_bytes();
        unsealed.subrange(0, 95) + little_endian_bytes(spec_crc32_castagnoli(self.spec_named_area()) as nat, 4)
            + unsealed.subrange(99, JOURNAL_RECORD_BYTES as int)
    }

    /// [`Self::to_bytes`] 写出的整条：再把偏移 46 那 32 字节换成罩整条 4096、自身按 0 算出来的头部校验和。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        let with_payload_checksum = self.spec_bytes_with_the_payload_checksum();
        with_payload_checksum.subrange(0, JOURNAL_HEADER_CHECKSUM_OFFSET as int) + spec_wide_checksum_field(
            with_payload_checksum,
            JOURNAL_RECORD_BYTES as int,
            JOURNAL_HEADER_CHECKSUM_OFFSET as int,
        ) + with_payload_checksum.subrange(
            JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES,
            JOURNAL_RECORD_BYTES as int,
        )
    }

    /// 头里自述的点名项数（偏移 12 的 4 字节）。
    pub open spec fn spec_named_count_of(bytes: Seq<u8>) -> nat {
        little_endian_value(bytes.subrange(12, 16))
    }

    /// [`Self::parse`] 收下这段字节的判法：magic、整条校验和、类型、记录标志、记录长度、提交标记、序号、新根段四条指针头部、
    /// 卸载记号、fsid、点名项区不越过记录末尾、载荷校验和。
    pub open spec fn spec_parse_accepts(bytes: Seq<u8>, expected_filesystem_identifier: u64) -> bool {
        let named_count = Self::spec_named_count_of(bytes);
        &&& bytes.len() >= JOURNAL_RECORD_BYTES
        &&& bytes.subrange(0, 4) == JOURNAL_MAGIC@
        &&& bytes.subrange(JOURNAL_HEADER_CHECKSUM_OFFSET as int, JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES)
            == spec_wide_checksum_field(bytes, JOURNAL_RECORD_BYTES as int, JOURNAL_HEADER_CHECKSUM_OFFSET as int)
        &&& little_endian_value(bytes.subrange(4, 6)) == JOURNAL_RECORD_TYPE_ORDINARY
        &&& JournalRecordPlaceInPublish::spec_of_record_flags_byte(little_endian_value(bytes.subrange(7, 8)) as u8) is Some
        &&& little_endian_value(bytes.subrange(8, 12)) == JOURNAL_RECORD_BYTES
        &&& little_endian_value(bytes.subrange(86, 87)) <= 1
        &&& little_endian_value(bytes.subrange(87, 91)) != 0
        &&& NodePointer::spec_read_judging_at(bytes, 99) is Ok
        &&& NodePointer::spec_read_judging_at(bytes, 185) is Ok
        &&& NodePointer::spec_read_judging_at(bytes, 287) is Ok
        &&& NodePointer::spec_read_judging_at(bytes, 373) is Ok
        &&& UnmountMarker::spec_of_unmount_marker_field(little_endian_value(bytes.subrange(459, 463)) as u32) is Some
        &&& little_endian_value(bytes.subrange(472, 480)) == expected_filesystem_identifier
        &&& JOURNAL_HEADER_BYTES + named_count * JOURNAL_NAMED_ENTRY_BYTES <= JOURNAL_RECORD_BYTES
        &&& little_endian_value(bytes.subrange(95, 99)) == spec_crc32_castagnoli(
            bytes.subrange(
                JOURNAL_HEADER_BYTES as int,
                JOURNAL_HEADER_BYTES + named_count * JOURNAL_NAMED_ENTRY_BYTES,
            ),
        )
    }

    /// 这条记录逐字段等于那段字节按字段表读出来的样子（[`Self::parse`] 交回的记录满足它）。
    pub open spec fn spec_matches_the_bytes(self, bytes: Seq<u8>) -> bool {
        let named_count = Self::spec_named_count_of(bytes);
        &&& JournalRecordPlaceInPublish::spec_of_record_flags_byte(little_endian_value(bytes.subrange(7, 8)) as u8) == Some(
            self.place_in_publish,
        )
        &&& self.instance.0 == little_endian_value(bytes.subrange(16, 20))
        &&& self.counter == little_endian_value(bytes.subrange(20, 26))
        &&& self.checkpoint_txg.0 == little_endian_value(bytes.subrange(26, 34))
        &&& self.transaction == little_endian_value(bytes.subrange(78, 86))
        &&& self.is_commit == (little_endian_value(bytes.subrange(86, 87)) == 1)
        &&& self.ordinal_within_publish.0 == little_endian_value(bytes.subrange(87, 91))
        &&& self.back_chain == little_endian_value(bytes.subrange(91, 95))
        &&& self.new_tree_table == NodePointer::spec_decode_at(bytes, 99)
        &&& self.new_mapping_root == NodePointer::spec_decode_at(bytes, 185)
        &&& self.new_tree_identifier_watermark == little_endian_value(bytes.subrange(271, 279))
        &&& self.new_rollback_floor.0 == little_endian_value(bytes.subrange(279, 287))
        &&& self.new_instance_table == NodePointer::spec_decode_at(bytes, 287)
        &&& self.new_allocation_record_tree_root == NodePointer::spec_decode_at(bytes, 373)
        &&& UnmountMarker::spec_of_unmount_marker_field(little_endian_value(bytes.subrange(459, 463)) as u32) == Some(
            self.new_unmount_marker,
        )
        &&& self.new_reclaim_threshold.0 == little_endian_value(bytes.subrange(463, 471))
        &&& self.new_pinned_ring_slot.0 == little_endian_value(bytes.subrange(471, 472))
        &&& self.filesystem_identifier == little_endian_value(bytes.subrange(472, 480))
        &&& self.named@ == Seq::new(
            named_count,
            |named_index: int| NamedUnit::spec_decode_at(
                bytes,
                JOURNAL_HEADER_BYTES + named_index * JOURNAL_NAMED_ENTRY_BYTES,
            ),
        )
    }

    /// # Panics
    /// 计数器装不进 6 字节、点名项多于一条记录装得下的、或有一条指针或点名项的位置条目写者不肯写（`requires` 排除的几种：
    /// 发布路径按一条记录装得下的点名项切记录，指针与位置条目来自写者现算的或读者判过的）。
    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        requires
            self.is_encodable(),
        ensures
            bytes@ == self.spec_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let mut writer = ByteWriter::new(record_bytes);
        writer.put(&JOURNAL_MAGIC);
        writer.put_u16(JOURNAL_RECORD_TYPE_ORDINARY);
        writer.put_u8(0); // 算法类型
        writer.assert_position(
            u64::try_from(JOURNAL_RECORD_FLAGS_OFFSET).expect("偏移"),
            "记录标志",
        );
        writer.put_u8(self.place_in_publish.record_flags_byte());
        writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES).expect("记录长度"));
        writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
        writer.put_u32(self.instance.0);
        writer.put_six_byte_unsigned(self.counter);
        writer.put_u64(self.checkpoint_txg.0);
        writer.skip(12); // nonce
        writer.assert_position(
            u64::try_from(JOURNAL_HEADER_CHECKSUM_OFFSET).expect("偏移"),
            "记录头校验和",
        );
        writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        writer.put_u64(self.transaction);
        // u8::from(bool) 没有 Verus 规格：带写 1、不带写 0，同一个字节。
        writer.put_u8(if self.is_commit { 1 } else { 0 });
        writer.assert_position(
            u64::try_from(JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET).expect("偏移"),
            "本次发布内序号",
        );
        writer.put_u32(self.ordinal_within_publish.0);
        writer.put_u32(self.back_chain);
        let payload_checksum_offset = writer.position();
        writer.skip(4);
        let new_root_segment_start = writer.position();
        self.new_tree_table.write_to(&mut writer);
        self.new_mapping_root.write_to(&mut writer);
        writer.put_u64(self.new_tree_identifier_watermark);
        writer.put_u64(self.new_rollback_floor.0);
        self.new_instance_table.write_to(&mut writer);
        self.new_allocation_record_tree_root.write_to(&mut writer);
        // 卸载记号 4 字节与根记录 flags 同一个编码（位 0 = 卸载那一串写的，其余位恒 0；D22（单元原子性怎么合成） 已定项 7 / 已定项 13）。
        writer.put_u32(self.new_unmount_marker.flags_bits());
        // 回收门槛 t 8 字节与钉槽 p 1 字节接在卸载记号之后（C773（读坏与全零根槽的回收门槛钉只在内存里）），与根记录同一对字段。
        writer.put_u64(self.new_reclaim_threshold.0);
        writer.put_u8(self.new_pinned_ring_slot.0);
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加一句话。
        assert!(
            writer.position() - new_root_segment_start
                == usize::try_from(JOURNAL_NEW_ROOT_SEGMENT_BYTES).expect("373"),
            "新根段写了 373 字节"
        );
        writer.put_u64(self.filesystem_identifier);
        writer.skip(16); // MAC：第一版留位全 0
        writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");
        let ghost header = writer.written();
        assert(header =~= self.spec_unsealed_header());
        let ghost named_pieces = spec_named_unit_bytes_of(self.named@);
        proof {
            lemma_named_unit_bytes_have_the_entry_width(self.named@);
        }
        // for 循环换成下标循环（Verus 对切片迭代器的规格不全），逐项写，次序不变。
        let mut named_index: usize = 0;
        while named_index < self.named.len()
            invariant
                writer.is_well_formed(),
                writer.total_bytes() == JOURNAL_RECORD_BYTES,
                header.len() == JOURNAL_HEADER_BYTES,
                header == self.spec_unsealed_header(),
                named_pieces == spec_named_unit_bytes_of(self.named@),
                named_index <= self.named@.len(),
                self.is_encodable(),
                crate::unit::spec_every_piece_has_width(named_pieces, JOURNAL_NAMED_ENTRY_BYTES as nat),
                writer.written() == header + crate::unit::spec_concatenation(named_pieces.subrange(0, named_index as int)),
            decreases self.named@.len() - named_index,
        {
            proof {
                let written_pieces = named_pieces.subrange(0, named_index as int);
                assert(crate::unit::spec_every_piece_has_width(written_pieces, JOURNAL_NAMED_ENTRY_BYTES as nat));
                crate::unit::lemma_concatenation_of_pieces_of_one_width(written_pieces, JOURNAL_NAMED_ENTRY_BYTES as nat);
                assert(named_index * 56 + 56 <= self.named@.len() * 56) by (nonlinear_arith)
                    requires
                        named_index < self.named@.len(),
                ;
                assert(self.named@[named_index as int].is_encodable());
            }
            self.named[named_index].write_to(&mut writer);
            proof {
                let longer = named_pieces.subrange(0, named_index + 1);
                assert(longer.drop_last() =~= named_pieces.subrange(0, named_index as int));
                assert(longer.last() == self.named@[named_index as int].spec_bytes());
            }
            named_index += 1;
        }
        proof {
            assert(named_pieces.subrange(0, self.named@.len() as int) =~= named_pieces);
            crate::unit::lemma_concatenation_of_pieces_of_one_width(named_pieces, JOURNAL_NAMED_ENTRY_BYTES as nat);
        }
        let payload_end = writer.position();
        let mut bytes = writer.into_bytes();
        proof {
            let unsealed = self.spec_unsealed_bytes();
            assert(bytes@ =~= unsealed);
            assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
            lemma_split_a_concatenated_range(
                unsealed,
                0,
                header + self.spec_named_area(),
                zero_bytes((JOURNAL_RECORD_BYTES - header.len() - self.spec_named_area().len()) as nat),
            );
            lemma_split_a_concatenated_range(unsealed, 0, header, self.spec_named_area());
        }
        let payload_checksum = crc32_castagnoli(
            &bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("496")..payload_end],
        );
        // copy_from_slice(&payload_checksum.to_le_bytes())：to_le_bytes 没有 Verus 规格，经 ByteWriter 写成同样的 4 字节，再成块拷进去。
        let mut checksum_writer = ByteWriter::new(4);
        checksum_writer.put_u32(payload_checksum);
        let checksum_bytes = checksum_writer.into_bytes();
        copy_bytes_into(&mut bytes, payload_checksum_offset, checksum_bytes.as_slice());
        assert(bytes@ =~= self.spec_bytes_with_the_payload_checksum());
        let digest =
            wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        copy_bytes_into(&mut bytes, JOURNAL_HEADER_CHECKSUM_OFFSET, &digest);
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// 读者：magic、整条校验和、类型、记录长度字段（等于 4096）、fsid、载荷校验和（连同点名项不越过记录末尾）六关，
    /// 外加五条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7）：记录标志位 0 之外有位为 1、提交标记不是 0 也不是 1、
    /// 本次发布内序号为 0、新根段四条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值（代码审阅第 29 条），
    /// 以及新根段的卸载记号那 4 字节位 0 之外有位为 1（与根记录 flags 同判，`root_record::parse_slot`；条款没写这一格，
    /// 照根记录与记录标志「其余位读到非 0 当损坏」的同一个处置取）。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
    /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
    #[must_use]
    pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> (record: Option<Self>)
        ensures
            match record {
                Some(parsed) => Self::spec_parse_accepts(bytes@, expected_filesystem_identifier)
                    && parsed.spec_matches_the_bytes(bytes@),
                None => !Self::spec_parse_accepts(bytes@, expected_filesystem_identifier),
            },
    {
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        if bytes.len() < record_bytes || bytes[..4] != JOURNAL_MAGIC {
            proof {
                if bytes@.len() >= 4096 {
                    assert(bytes@.subrange(0, 4) != JOURNAL_MAGIC@);
                }
            }
            return None;
        }
        assert(bytes@.subrange(0, 4) =~= JOURNAL_MAGIC@);
        if !wide_checksum_field_holds(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        if reader.get_u16() != JOURNAL_RECORD_TYPE_ORDINARY {
            return None;
        }
        reader.skip(1); // 算法类型
        let place_in_publish =
            JournalRecordPlaceInPublish::from_record_flags_byte(reader.get_u8())?;
        if u64::from(reader.get_u32()) != JOURNAL_RECORD_BYTES {
            return None;
        }
        let named_count = usize::try_from(reader.get_u32()).expect("点名项数");
        let instance = InstanceGeneration(reader.get_u32());
        let counter = reader.get_six_byte_unsigned();
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + 32);
        let transaction = reader.get_u64();
        // 提交标记只有 0（不带）与 1（带）两个取值（D23（journal 的角色与格式） 已定项 7）：读到 2..=255 当这条记录损坏、断链即止，
        // 与已定项 4 读者规则那几格同一处置。
        let commit_marker = reader.get_u8();
        let is_commit = if commit_marker == 0 {
            false
        } else if commit_marker == 1 {
            true
        } else {
            return None;
        };
        let ordinal_within_publish = JournalRecordOrdinalWithinPublish(reader.get_u32());
        // 序号从 1 起（已定项 4）：读到 0 当这条记录损坏。
        if ordinal_within_publish.0 == 0 {
            return None;
        }
        let back_chain = reader.get_u32();
        let payload_checksum = reader.get_u32();
        // 新根段那两条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值（代码审阅第 29 条，`pointer::PointerHeadFieldOutsideTheFirstVersion`）：
        // 与上面几格同一个结局，这条记录不算在、前缀在它之前断。Result::ok 接 ? 写成 match，同一个结局。
        let new_tree_table =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let new_mapping_root =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let new_tree_identifier_watermark = reader.get_u64();
        let new_rollback_floor = CheckpointTxg(reader.get_u64());
        let new_instance_table =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let new_allocation_record_tree_root =
            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut reader,
            ) {
                Ok(pointer) => pointer,
                Err(_refusal) => return None,
            };
        let new_unmount_marker =
            UnmountMarker::of_unmount_marker_field_of_a_new_root_segment(reader.get_u32())?;
        let new_reclaim_threshold = CheckpointTxg(reader.get_u64());
        let new_pinned_ring_slot = PinnedRingSlot(reader.get_u8());
        let filesystem_identifier = reader.get_u64();
        if filesystem_identifier != expected_filesystem_identifier {
            return None;
        }
        reader.skip(16);
        let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("496");
        let named_entry_bytes = usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加一句话。
        assert!(reader.position() == header_bytes, "记录头读到偏移 496");
        // 点名项区越过记录末尾的不收（496 + 56 × 项数 > 4096）。项数是盘上读来的 4 字节，先跟一条记录装得下的项数
        // （(4096 − 496) ÷ 56 = 64）比，过了才乘：与先乘再比同一个判法，乘法不越过 usize。
        if named_count > (record_bytes - header_bytes) / named_entry_bytes {
            proof {
                assert(496 + named_count * 56 > 4096) by (nonlinear_arith)
                    requires
                        named_count > 64,
                ;
            }
            return None;
        }
        proof {
            assert(496 + named_count * 56 <= 4096) by (nonlinear_arith)
                requires
                    named_count <= 64,
            ;
        }
        let payload_end = reader.position() + named_count * named_entry_bytes;
        if crc32_castagnoli(&bytes[reader.position()..payload_end]) != payload_checksum {
            return None;
        }
        // (0..项数).map(|_| NamedUnit::read_from(&mut reader)).collect() 写成下标循环：闭包与迭代器没有 Verus 规格，逐项读，次序不变。
        // 按项数预分配，与 collect 那一份一样只开一次、恰好项数：逐项 push 从空 Vec 起第一次就开 4 项，
        // 项数 1 的记录每条多占 3 项的堆，扫环收下的每条记录都带着这份多出来的堆（E197（扫环胶水内存随盘数）的分配器峰值预测按恰好项数算）。
        assert(bytes@.len() == bytes.len());
        let mut named: Vec<NamedUnit> = Vec::with_capacity(named_count);
        let mut named_index: usize = 0;
        while named_index < named_count
            invariant
                reader.source() == bytes@,
                bytes@.len() == bytes.len(),
                reader.spec_position() == 496 + named_index * 56,
                496 + named_count * 56 <= 4096,
                bytes@.len() >= 4096,
                named_index <= named_count,
                named@.len() == named_index,
                forall|read_index: int|
                    0 <= read_index < named_index ==> #[trigger] named@[read_index] == NamedUnit::spec_decode_at(
                        bytes@,
                        496 + read_index * 56,
                    ),
            decreases named_count - named_index,
        {
            proof {
                assert(496 + named_index * 56 + 56 <= 496 + named_count * 56) by (nonlinear_arith)
                    requires
                        named_index < named_count,
                ;
                assert((named_index + 1) * 56 == named_index * 56 + 56) by (nonlinear_arith);
            }
            named.push(NamedUnit::read_from(&mut reader));
            named_index += 1;
        }
        let record = Self {
            instance,
            counter,
            checkpoint_txg,
            transaction,
            is_commit,
            ordinal_within_publish,
            place_in_publish,
            back_chain,
            filesystem_identifier,
            new_tree_table,
            new_mapping_root,
            new_tree_identifier_watermark,
            new_rollback_floor,
            new_instance_table,
            new_allocation_record_tree_root,
            new_unmount_marker,
            new_reclaim_threshold,
            new_pinned_ring_slot,
            named,
        };
        assert(record.named@ =~= Seq::new(
            named_count as nat,
            |named_index: int| NamedUnit::spec_decode_at(
                bytes@,
                JOURNAL_HEADER_BYTES + named_index * JOURNAL_NAMED_ENTRY_BYTES,
            ),
        ));
        Some(record)
    }
}

// ===== 往返：写者写出来的记录，读者收下、读回同一条 =====

impl JournalRecord {
    /// 没封的整条逐段剥开（[`Self::lemma_parse_round_trip`] 拆出来的前一半）：照写者的次序，每个字段落在字段表上的那一截。
    pub proof fn lemma_unsealed_fields(self)
        requires
            self.is_encodable(),
        ensures
            ({
                let unsealed = self.spec_unsealed_bytes();
                let named_end = JOURNAL_HEADER_BYTES + self.named@.len() * JOURNAL_NAMED_ENTRY_BYTES;
                &&& unsealed.len() == JOURNAL_RECORD_BYTES
                &&& self.spec_named_area().len() == self.named@.len() * JOURNAL_NAMED_ENTRY_BYTES
                &&& named_end <= JOURNAL_RECORD_BYTES
                &&& unsealed.subrange(0, 4) == JOURNAL_MAGIC@
                &&& unsealed.subrange(4, 6) == little_endian_bytes(JOURNAL_RECORD_TYPE_ORDINARY as nat, 2)
                &&& unsealed.subrange(7, 8) == little_endian_bytes(self.place_in_publish.spec_record_flags_byte() as nat, 1)
                &&& unsealed.subrange(8, 12) == little_endian_bytes(JOURNAL_RECORD_BYTES as nat, 4)
                &&& unsealed.subrange(12, 16) == little_endian_bytes(self.named@.len(), 4)
                &&& unsealed.subrange(16, 20) == little_endian_bytes(self.instance.0 as nat, 4)
                &&& unsealed.subrange(20, 26) == little_endian_bytes(self.counter as nat, 6)
                &&& unsealed.subrange(26, 34) == little_endian_bytes(self.checkpoint_txg.0 as nat, 8)
                &&& unsealed.subrange(78, 86) == little_endian_bytes(self.transaction as nat, 8)
                &&& unsealed.subrange(86, 87) == little_endian_bytes(self.spec_commit_marker() as nat, 1)
                &&& unsealed.subrange(87, 91) == little_endian_bytes(self.ordinal_within_publish.0 as nat, 4)
                &&& unsealed.subrange(91, 95) == little_endian_bytes(self.back_chain as nat, 4)
                &&& unsealed.subrange(99, 185) == self.new_tree_table.spec_bytes()
                &&& unsealed.subrange(185, 271) == self.new_mapping_root.spec_bytes()
                &&& unsealed.subrange(271, 279) == little_endian_bytes(self.new_tree_identifier_watermark as nat, 8)
                &&& unsealed.subrange(279, 287) == little_endian_bytes(self.new_rollback_floor.0 as nat, 8)
                &&& unsealed.subrange(287, 373) == self.new_instance_table.spec_bytes()
                &&& unsealed.subrange(373, 459) == self.new_allocation_record_tree_root.spec_bytes()
                &&& unsealed.subrange(459, 463) == little_endian_bytes(self.new_unmount_marker.spec_flags_bits() as nat, 4)
                &&& unsealed.subrange(463, 471) == little_endian_bytes(self.new_reclaim_threshold.0 as nat, 8)
                &&& unsealed.subrange(471, 472) == little_endian_bytes(self.new_pinned_ring_slot.0 as nat, 1)
                &&& unsealed.subrange(472, 480) == little_endian_bytes(self.filesystem_identifier as nat, 8)
                &&& unsealed.subrange(JOURNAL_HEADER_BYTES as int, named_end) == self.spec_named_area()
            }),
    {
        lemma_little_endian_capacities();
        lemma_named_unit_bytes_have_the_entry_width(self.named@);
        crate::unit::lemma_concatenation_of_pieces_of_one_width(
            spec_named_unit_bytes_of(self.named@),
            JOURNAL_NAMED_ENTRY_BYTES as nat,
        );
        assert(self.named@.len() * 56 <= 64 * 56) by (nonlinear_arith)
            requires
                self.named@.len() <= 64,
        ;
        let header = self.spec_unsealed_header();
        let named_area = self.spec_named_area();
        let unsealed = self.spec_unsealed_bytes();
        let padding = zero_bytes((JOURNAL_RECORD_BYTES - header.len() - named_area.len()) as nat);
        assert(header.len() == 496);
        assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
        lemma_split_a_concatenated_range(unsealed, 0, header + named_area, padding);
        lemma_split_a_concatenated_range(unsealed, 0, header, named_area);
        let magic = JOURNAL_MAGIC@;
        let record_type = little_endian_bytes(JOURNAL_RECORD_TYPE_ORDINARY as nat, 2);
        let algorithm = little_endian_bytes(0, 1);
        let flags = little_endian_bytes(self.place_in_publish.spec_record_flags_byte() as nat, 1);
        let length = little_endian_bytes(JOURNAL_RECORD_BYTES as nat, 4);
        let count = little_endian_bytes(self.named@.len(), 4);
        let instance = little_endian_bytes(self.instance.0 as nat, 4);
        let counter = little_endian_bytes(self.counter as nat, 6);
        let checkpoint = little_endian_bytes(self.checkpoint_txg.0 as nat, 8);
        let nonce = zero_bytes(12);
        let header_checksum = zero_bytes(WIDE_CHECKSUM_BYTES as nat);
        let transaction = little_endian_bytes(self.transaction as nat, 8);
        let commit = little_endian_bytes(self.spec_commit_marker() as nat, 1);
        let ordinal = little_endian_bytes(self.ordinal_within_publish.0 as nat, 4);
        let back_chain = little_endian_bytes(self.back_chain as nat, 4);
        let payload_checksum = zero_bytes(4);
        let tree_table = self.new_tree_table.spec_bytes();
        let mapping_root = self.new_mapping_root.spec_bytes();
        let watermark = little_endian_bytes(self.new_tree_identifier_watermark as nat, 8);
        let floor = little_endian_bytes(self.new_rollback_floor.0 as nat, 8);
        let instance_table = self.new_instance_table.spec_bytes();
        let allocation_root = self.new_allocation_record_tree_root.spec_bytes();
        let unmount = little_endian_bytes(self.new_unmount_marker.spec_flags_bits() as nat, 4);
        let reclaim = little_endian_bytes(self.new_reclaim_threshold.0 as nat, 8);
        let pinned = little_endian_bytes(self.new_pinned_ring_slot.0 as nat, 1);
        let fsid = little_endian_bytes(self.filesystem_identifier as nat, 8);
        let mac = zero_bytes(16);
        let through_nonce = magic + record_type + algorithm + flags + length + count + instance + counter + checkpoint + nonce;
        let through_payload_checksum = through_nonce + header_checksum + transaction + commit + ordinal + back_chain
            + payload_checksum;
        let through_floor = through_payload_checksum + tree_table + mapping_root + watermark + floor;
        let through_pinned = through_floor + instance_table + allocation_root + unmount + reclaim + pinned;
        lemma_split_a_concatenated_range(unsealed, 0, through_pinned + fsid, mac);
        lemma_split_a_concatenated_range(unsealed, 0, through_pinned, fsid);
        lemma_split_a_concatenated_range(unsealed, 0, through_floor + instance_table + allocation_root + unmount + reclaim, pinned);
        lemma_split_a_concatenated_range(unsealed, 0, through_floor + instance_table + allocation_root + unmount, reclaim);
        lemma_split_a_concatenated_range(unsealed, 0, through_floor + instance_table + allocation_root, unmount);
        lemma_split_a_concatenated_range(unsealed, 0, through_floor + instance_table, allocation_root);
        lemma_split_a_concatenated_range(unsealed, 0, through_floor, instance_table);
        lemma_split_a_concatenated_range(unsealed, 0, through_payload_checksum + tree_table + mapping_root + watermark, floor);
        lemma_split_a_concatenated_range(unsealed, 0, through_payload_checksum + tree_table + mapping_root, watermark);
        lemma_split_a_concatenated_range(unsealed, 0, through_payload_checksum + tree_table, mapping_root);
        lemma_split_a_concatenated_range(unsealed, 0, through_payload_checksum, tree_table);
        lemma_split_a_concatenated_range(
            unsealed,
            0,
            through_nonce + header_checksum + transaction + commit + ordinal + back_chain,
            payload_checksum,
        );
        lemma_split_a_concatenated_range(unsealed, 0, through_nonce + header_checksum + transaction + commit + ordinal, back_chain);
        lemma_split_a_concatenated_range(unsealed, 0, through_nonce + header_checksum + transaction + commit, ordinal);
        lemma_split_a_concatenated_range(unsealed, 0, through_nonce + header_checksum + transaction, commit);
        lemma_split_a_concatenated_range(unsealed, 0, through_nonce + header_checksum, transaction);
        lemma_split_a_concatenated_range(unsealed, 0, through_nonce, header_checksum);
        lemma_split_a_concatenated_range(
            unsealed,
            0,
            magic + record_type + algorithm + flags + length + count + instance + counter + checkpoint,
            nonce,
        );
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm + flags + length + count + instance + counter, checkpoint);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm + flags + length + count + instance, counter);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm + flags + length + count, instance);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm + flags + length, count);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm + flags, length);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type + algorithm, flags);
        lemma_split_a_concatenated_range(unsealed, 0, magic + record_type, algorithm);
        lemma_split_a_concatenated_range(unsealed, 0, magic, record_type);
    }

    /// 封好的整条在两个校验和字段之外与没封的那一份逐字节相同，两个校验和两判都过（[`Self::lemma_parse_round_trip`] 拆出来的中间一段）。
    pub proof fn lemma_sealed_keeps_everything_outside_the_two_checksums(self)
        requires
            self.is_encodable(),
        ensures
            ({
                let unsealed = self.spec_unsealed_bytes();
                let sealed = self.spec_bytes();
                let named_end = JOURNAL_HEADER_BYTES + self.named@.len() * JOURNAL_NAMED_ENTRY_BYTES;
                &&& sealed.len() == JOURNAL_RECORD_BYTES
                &&& forall|position: int|
                    0 <= position < JOURNAL_RECORD_BYTES && !(JOURNAL_HEADER_CHECKSUM_OFFSET <= position
                        < JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES) && !(95 <= position < 99) ==> #[trigger] sealed[position]
                        == unsealed[position]
                &&& sealed.subrange(JOURNAL_HEADER_CHECKSUM_OFFSET as int, JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES)
                    == spec_wide_checksum_field(sealed, JOURNAL_RECORD_BYTES as int, JOURNAL_HEADER_CHECKSUM_OFFSET as int)
                &&& named_end <= JOURNAL_RECORD_BYTES
                &&& sealed.subrange(JOURNAL_HEADER_BYTES as int, named_end) == self.spec_named_area()
                &&& little_endian_value(sealed.subrange(95, 99)) == spec_crc32_castagnoli(
                    sealed.subrange(JOURNAL_HEADER_BYTES as int, named_end),
                )
            }),
    {
        lemma_little_endian_capacities();
        self.lemma_unsealed_fields();
        let unsealed = self.spec_unsealed_bytes();
        let with_payload_checksum = self.spec_bytes_with_the_payload_checksum();
        let crc = spec_crc32_castagnoli(self.spec_named_area());
        let field = spec_wide_checksum_field(with_payload_checksum, 4096, 46);
        let sealed = self.spec_bytes();
        let named_end = JOURNAL_HEADER_BYTES + self.named@.len() * JOURNAL_NAMED_ENTRY_BYTES;
        assert(with_payload_checksum.len() == 4096);
        assert(field.len() == 32);
        assert(sealed.len() == 4096);
        assert forall|position: int|
            0 <= position < 4096 && !(95 <= position < 99) implies #[trigger] with_payload_checksum[position]
                == unsealed[position] by {
            if position < 95 {
                assert(with_payload_checksum[position] == unsealed.subrange(0, 95)[position]);
            } else {
                assert(with_payload_checksum[position] == unsealed.subrange(99, 4096)[position - 99]);
            }
        }
        assert forall|position: int| 0 <= position < 4096 && !(46 <= position < 78) implies #[trigger] sealed[position]
            == with_payload_checksum[position] by {
            if position < 46 {
                assert(sealed[position] == with_payload_checksum.subrange(0, 46)[position]);
            } else {
                assert(sealed[position] == with_payload_checksum.subrange(78, 4096)[position - 78]);
            }
        }
        // 罩住的那一截（字段按 0）两份相同，所以封进去的字段就是读者现算的字段。
        assert(spec_covered_with_the_field_zeroed(sealed, 4096, 46) =~= spec_covered_with_the_field_zeroed(
            with_payload_checksum,
            4096,
            46,
        ));
        assert(sealed.subrange(46, 78) =~= field);
        // 点名项那一截在封好的整条里没变，载荷校验和那 4 字节是写进去的那个数。
        assert(sealed.subrange(496, named_end) =~= unsealed.subrange(496, named_end));
        assert(sealed.subrange(95, 99) =~= little_endian_bytes(crc as nat, 4)) by {
            assert forall|position: int| 0 <= position < 4 implies sealed.subrange(95, 99)[position] == little_endian_bytes(
                crc as nat,
                4,
            )[position] by {
                assert(sealed[95 + position] == with_payload_checksum[95 + position]);
                assert(with_payload_checksum[95 + position] == (unsealed.subrange(0, 95) + little_endian_bytes(
                    crc as nat,
                    4,
                ))[95 + position]);
            }
        }
    }

    /// 封好的整条里新根段之前的各字段还在字段表上的那一截（[`Self::lemma_parse_round_trip`] 拆出来的一段）。
    pub proof fn lemma_sealed_header_fields_before_the_new_root_segment(self)
        requires
            self.is_encodable(),
        ensures
            ({
                let sealed = self.spec_bytes();
                &&& sealed.subrange(0, 4) == JOURNAL_MAGIC@
                &&& sealed.subrange(4, 6) == little_endian_bytes(JOURNAL_RECORD_TYPE_ORDINARY as nat, 2)
                &&& sealed.subrange(7, 8) == little_endian_bytes(self.place_in_publish.spec_record_flags_byte() as nat, 1)
                &&& sealed.subrange(8, 12) == little_endian_bytes(JOURNAL_RECORD_BYTES as nat, 4)
                &&& sealed.subrange(12, 16) == little_endian_bytes(self.named@.len(), 4)
                &&& sealed.subrange(16, 20) == little_endian_bytes(self.instance.0 as nat, 4)
                &&& sealed.subrange(20, 26) == little_endian_bytes(self.counter as nat, 6)
                &&& sealed.subrange(26, 34) == little_endian_bytes(self.checkpoint_txg.0 as nat, 8)
                &&& sealed.subrange(78, 86) == little_endian_bytes(self.transaction as nat, 8)
                &&& sealed.subrange(86, 87) == little_endian_bytes(self.spec_commit_marker() as nat, 1)
                &&& sealed.subrange(87, 91) == little_endian_bytes(self.ordinal_within_publish.0 as nat, 4)
                &&& sealed.subrange(91, 95) == little_endian_bytes(self.back_chain as nat, 4)
            }),
    {
        self.lemma_unsealed_fields();
        self.lemma_sealed_keeps_everything_outside_the_two_checksums();
        let unsealed = self.spec_unsealed_bytes();
        let sealed = self.spec_bytes();
        assert(sealed.subrange(0, 4) =~= unsealed.subrange(0, 4));
        assert(sealed.subrange(4, 6) =~= unsealed.subrange(4, 6));
        assert(sealed.subrange(7, 8) =~= unsealed.subrange(7, 8));
        assert(sealed.subrange(8, 12) =~= unsealed.subrange(8, 12));
        assert(sealed.subrange(12, 16) =~= unsealed.subrange(12, 16));
        assert(sealed.subrange(16, 20) =~= unsealed.subrange(16, 20));
        assert(sealed.subrange(20, 26) =~= unsealed.subrange(20, 26));
        assert(sealed.subrange(26, 34) =~= unsealed.subrange(26, 34));
        assert(sealed.subrange(78, 86) =~= unsealed.subrange(78, 86));
        assert(sealed.subrange(86, 87) =~= unsealed.subrange(86, 87));
        assert(sealed.subrange(87, 91) =~= unsealed.subrange(87, 91));
        assert(sealed.subrange(91, 95) =~= unsealed.subrange(91, 95));
    }

    /// 封好的整条里新根段与 fsid 各字段还在字段表上的那一截（[`Self::lemma_parse_round_trip`] 拆出来的一段）。
    pub proof fn lemma_sealed_new_root_segment_fields(self)
        requires
            self.is_encodable(),
        ensures
            ({
                let sealed = self.spec_bytes();
                &&& sealed.subrange(99, 185) == self.new_tree_table.spec_bytes()
                &&& sealed.subrange(185, 271) == self.new_mapping_root.spec_bytes()
                &&& sealed.subrange(271, 279) == little_endian_bytes(self.new_tree_identifier_watermark as nat, 8)
                &&& sealed.subrange(279, 287) == little_endian_bytes(self.new_rollback_floor.0 as nat, 8)
                &&& sealed.subrange(287, 373) == self.new_instance_table.spec_bytes()
                &&& sealed.subrange(373, 459) == self.new_allocation_record_tree_root.spec_bytes()
                &&& sealed.subrange(459, 463) == little_endian_bytes(self.new_unmount_marker.spec_flags_bits() as nat, 4)
                &&& sealed.subrange(463, 471) == little_endian_bytes(self.new_reclaim_threshold.0 as nat, 8)
                &&& sealed.subrange(471, 472) == little_endian_bytes(self.new_pinned_ring_slot.0 as nat, 1)
                &&& sealed.subrange(472, 480) == little_endian_bytes(self.filesystem_identifier as nat, 8)
            }),
    {
        self.lemma_unsealed_fields();
        self.lemma_sealed_keeps_everything_outside_the_two_checksums();
        let unsealed = self.spec_unsealed_bytes();
        let sealed = self.spec_bytes();
        assert(sealed.subrange(99, 185) =~= unsealed.subrange(99, 185));
        assert(sealed.subrange(185, 271) =~= unsealed.subrange(185, 271));
        assert(sealed.subrange(271, 279) =~= unsealed.subrange(271, 279));
        assert(sealed.subrange(279, 287) =~= unsealed.subrange(279, 287));
        assert(sealed.subrange(287, 373) =~= unsealed.subrange(287, 373));
        assert(sealed.subrange(373, 459) =~= unsealed.subrange(373, 459));
        assert(sealed.subrange(459, 463) =~= unsealed.subrange(459, 463));
        assert(sealed.subrange(463, 471) =~= unsealed.subrange(463, 471));
        assert(sealed.subrange(471, 472) =~= unsealed.subrange(471, 472));
        assert(sealed.subrange(472, 480) =~= unsealed.subrange(472, 480));
    }

    /// 封好的整条里点名项逐项读回（[`Self::lemma_parse_round_trip`] 拆出来的一段）：第 named_index 项落在
    /// [496 + 56 × named_index, 496 + 56 × (named_index + 1))，解回的就是这一项。
    pub proof fn lemma_named_units_read_back(self)
        requires
            self.is_encodable(),
        ensures
            Self::spec_named_count_of(self.spec_bytes()) == self.named@.len(),
            self.named@ == Seq::new(
                Self::spec_named_count_of(self.spec_bytes()),
                |named_index: int| NamedUnit::spec_decode_at(
                    self.spec_bytes(),
                    JOURNAL_HEADER_BYTES + named_index * JOURNAL_NAMED_ENTRY_BYTES,
                ),
            ),
    {
        lemma_little_endian_capacities();
        self.lemma_sealed_keeps_everything_outside_the_two_checksums();
        self.lemma_sealed_header_fields_before_the_new_root_segment();
        let sealed = self.spec_bytes();
        let named_count = self.named@.len();
        let named_end = 496 + named_count as int * 56;
        let pieces = spec_named_unit_bytes_of(self.named@);
        lemma_named_unit_bytes_have_the_entry_width(self.named@);
        let decoded = Seq::new(
            Self::spec_named_count_of(sealed),
            |named_index: int| NamedUnit::spec_decode_at(sealed, JOURNAL_HEADER_BYTES + named_index * JOURNAL_NAMED_ENTRY_BYTES),
        );
        assert(Self::spec_named_count_of(sealed) == named_count);
        assert forall|named_index: int| 0 <= named_index < named_count implies #[trigger] decoded[named_index]
            == self.named@[named_index] by {
            crate::unit::lemma_piece_of_a_concatenation(pieces, 56, named_index);
            assert((named_index + 1) * 56 <= named_count * 56) by (nonlinear_arith)
                requires
                    named_index + 1 <= named_count,
            ;
            assert(0 <= named_index * 56) by (nonlinear_arith)
                requires
                    0 <= named_index,
            ;
            assert((named_index + 1) * 56 == named_index * 56 + 56) by (nonlinear_arith);
            let piece_start = 496 + named_index * 56;
            assert(sealed.subrange(piece_start, piece_start + 56) =~= sealed.subrange(496, named_end).subrange(
                named_index * 56,
                (named_index + 1) * 56,
            ));
            assert(pieces[named_index] == self.named@[named_index].spec_bytes());
            assert(self.named@[named_index].is_encodable());
            self.named@[named_index].lemma_decode_at_round_trip(sealed, piece_start);
        }
        assert(decoded =~= self.named@);
    }

    /// 往返：写者写出来的整条（[`Self::spec_bytes`]），读者按这条记录自己的 fsid 收下，读回的每个字段都是这条记录的
    /// （[`Self::spec_matches_the_bytes`]）。读者拒「本次发布内序号为 0」，所以这里要序号不为 0；写者不判它。
    pub proof fn lemma_parse_round_trip(self)
        requires
            self.is_encodable(),
            self.ordinal_within_publish.0 != 0,
        ensures
            Self::spec_parse_accepts(self.spec_bytes(), self.filesystem_identifier),
            self.spec_matches_the_bytes(self.spec_bytes()),
    {
        // 整条字节的两份写法规格与记录标志的读法规格在这里不展开：只用下面几段引理交回的事实，少了哪一段就在这里当场报证明错，
        // 不会去展开 4096 字节的写法、撞 rlimit。
        hide(JournalRecord::spec_bytes);
        hide(JournalRecord::spec_unsealed_bytes);
        hide(JournalRecordPlaceInPublish::spec_of_record_flags_byte);
        lemma_little_endian_capacities();
        self.lemma_sealed_keeps_everything_outside_the_two_checksums();
        self.lemma_sealed_header_fields_before_the_new_root_segment();
        self.lemma_sealed_new_root_segment_fields();
        self.lemma_named_units_read_back();
        let sealed = self.spec_bytes();
        self.place_in_publish.lemma_record_flags_byte_round_trip();
        self.new_unmount_marker.lemma_flags_bits_round_trip();
        self.new_tree_table.lemma_decode_at_round_trip(sealed, 99);
        self.new_mapping_root.lemma_decode_at_round_trip(sealed, 185);
        self.new_instance_table.lemma_decode_at_round_trip(sealed, 287);
        self.new_allocation_record_tree_root.lemma_decode_at_round_trip(sealed, 373);
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

    fn empty_record(counter: u64) -> JournalRecord {
        JournalRecord {
            instance: InstanceGeneration(1),
            counter,
            checkpoint_txg: CheckpointTxg(counter),
            transaction: 0,
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
            back_chain: 0,
            filesystem_identifier: 7,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            new_rollback_floor: CheckpointTxg(0),
            new_instance_table: NodePointer::empty_root(),
            new_allocation_record_tree_root: NodePointer::empty_root(),
            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            new_reclaim_threshold: CheckpointTxg(0),
            new_pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
            named: Vec::new(),
        }
    }

    /// 本次发布内序号 4 字节紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：落在 [87, 91)、小端、读得回；
    /// 它之后的反向链、载荷校验和、新根段依次后挪 4 字节；新根段 373 字节之后是 fsid [472, 480)，MAC 16 占记录头最后的 [480, 496)。偏移按字段表手算、写成字面量，
    /// 不从写者用的常量推（已定项 26 第 1 条：覆盖范围是显式的盘上常量）。
    #[test]
    fn the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips() {
        let record = JournalRecord {
            transaction: 5,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(3),
            back_chain: 0x0102_0304,
            ..empty_record(9)
        };
        let bytes = record.to_bytes();
        assert_eq!(&bytes[78..86], &5u64.to_le_bytes(), "事务号在 [78, 86)");
        assert_eq!(bytes[86], 1, "提交标记在 86");
        assert_eq!(
            &bytes[87..91],
            &3u32.to_le_bytes(),
            "本次发布内序号紧跟提交标记，在 [87, 91)"
        );
        assert_eq!(
            &bytes[91..95],
            &0x0102_0304u32.to_le_bytes(),
            "反向链后挪到 [91, 95)"
        );
        assert_eq!(
            &bytes[95..99],
            &crc32_castagnoli(&[]).to_le_bytes(),
            "载荷校验和后挪到 [95, 99)：没有点名项时罩的是空串"
        );
        assert_eq!(
            &bytes[99 + 172..99 + 180],
            &singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS.to_le_bytes(),
            "新根段从 99 起：两条指针 86 × 2 之后是树 ID 水位"
        );
        assert_eq!(
            &bytes[472..480],
            &7u64.to_le_bytes(),
            "fsid 在新根段之后，[472, 480)"
        );
        assert!(
            bytes[480..496].iter().all(|byte| *byte == 0),
            "MAC 16 占 [480, 496)，第一版全 0"
        );
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
    }

    /// 新根段后三项（D23（journal 的角色与格式） 已定项 4 / 已定项 15）：F 之后依次是实例表指针 [287, 373)、分配记录树根指针 [373, 459)、
    /// 卸载记号 [459, 463)（与根记录 flags 同一个编码，位 0）；三项各取一个与别的字段都不同的值，写得出、读得回。
    /// 偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
    #[test]
    fn the_instance_table_allocation_record_tree_root_and_unmount_marker_follow_the_floor_in_the_new_root_segment_and_round_trip(
    ) {
        let instance_table_location = |device: u32| LocationEntry {
            device: crate::address::DeviceIdentity(device),
            slot: crate::address::SlotNumber(60_001),
            unit_checksum: 0x0a0b_0c0d,
        };
        let allocation_record_tree_location = |device: u32| LocationEntry {
            device: crate::address::DeviceIdentity(device),
            slot: crate::address::SlotNumber(60_002),
            unit_checksum: 0x1a1b_1c1d,
        };
        let pointer_with = |locations: [LocationEntry; 2], birth_txg: u64| NodePointer {
            head: crate::pointer::PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(birth_txg),
            },
            locations,
            instance: InstanceGeneration(1),
            birth_sequence: crate::pointer::BirthSequence(3),
        };
        let record = JournalRecord {
            new_instance_table: pointer_with(
                [instance_table_location(0), instance_table_location(1)],
                5,
            ),
            new_allocation_record_tree_root: pointer_with(
                [
                    allocation_record_tree_location(0),
                    allocation_record_tree_location(1),
                ],
                6,
            ),
            new_unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
            ..empty_record(9)
        };
        let bytes = record.to_bytes();
        let mut instance_table_bytes = ByteWriter::new(86);
        record
            .new_instance_table
            .write_to(&mut instance_table_bytes);
        assert_eq!(
            &bytes[287..373],
            instance_table_bytes.into_bytes().as_slice(),
            "实例表指针紧跟 F（99 + 188），在 [287, 373)"
        );
        let mut allocation_record_tree_bytes = ByteWriter::new(86);
        record
            .new_allocation_record_tree_root
            .write_to(&mut allocation_record_tree_bytes);
        assert_eq!(
            &bytes[373..459],
            allocation_record_tree_bytes.into_bytes().as_slice(),
            "分配记录树根指针在 [373, 459)"
        );
        assert_eq!(
            &bytes[459..463],
            &[0x01, 0x00, 0x00, 0x00],
            "卸载记号在 [459, 463)：卸载那一串写的，位 0 = 1"
        );
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
    }

    /// 新根段末尾两项（C773（读坏与全零根槽的回收门槛钉只在内存里））：卸载记号之后是回收门槛 t [463, 471)、钉槽 p 471，fsid 后挪到 [472, 480)。
    /// 两项各取一个与别的字段都不同的值，写得出、读得回。偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
    #[test]
    fn the_reclaim_threshold_and_the_pinned_ring_slot_close_the_new_root_segment_and_round_trip() {
        let record = JournalRecord {
            new_reclaim_threshold: CheckpointTxg(0x1112_1314_1516_1718),
            new_pinned_ring_slot: PinnedRingSlot(10),
            ..empty_record(9)
        };
        let bytes = record.to_bytes();
        assert_eq!(
            &bytes[463..471],
            &0x1112_1314_1516_1718u64.to_le_bytes(),
            "回收门槛 t 紧跟卸载记号，在 [463, 471)"
        );
        assert_eq!(bytes[471], 10, "钉槽 p 在 471，新根段到 472 为止");
        assert_eq!(&bytes[472..480], &7u64.to_le_bytes(), "fsid 紧跟新根段");
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
    }

    /// 新根段的卸载记号那 4 字节位 0 之外有位为 1 的记录，读者当损坏（与根记录 flags、记录标志「其余位读到非 0」同一个处置）。
    /// 头部校验和按改过的字节重封过，拦住它的只有卸载记号那一判。
    #[test]
    fn record_whose_new_root_segment_unmount_marker_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser(
    ) {
        for unmount_marker_field in [0x0000_0002u32, 0x0000_0003, 0x8000_0000] {
            let mut bytes = empty_record(4).to_bytes();
            bytes[459..463].copy_from_slice(&unmount_marker_field.to_le_bytes());
            reseal_header_checksum(&mut bytes);
            assert_eq!(
                JournalRecord::parse(&bytes, 7),
                None,
                "卸载记号 {unmount_marker_field:#010x}：位 0 之外有位为 1，当损坏"
            );
        }
    }

    /// 改过记录头某几个字节之后重封头部校验和（罩整条 4096、自身按 0 参与，D23（journal 的角色与格式） 已定项 13）：
    /// 拦得住那条记录的只剩被改的字段本身。
    fn reseal_header_checksum(bytes: &mut [u8]) {
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let digest =
            wide_checksum_with_field_zeroed(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
    }

    /// 记录标志 1 字节在偏移 7（magic 4 + 类型 2 + 算法类型 1，原「填充 1」那个字节，D23（journal 的角色与格式） 已定项 4）：
    /// 本次发布末条写 1、不是末条写 0，两种都读得回。偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
    #[test]
    fn the_record_flags_byte_at_offset_seven_carries_bit_zero_for_the_last_record_of_the_publish_and_round_trips(
    ) {
        let last = empty_record(3);
        let last_bytes = last.to_bytes();
        assert_eq!(last_bytes[7], 1, "本次发布末条：记录标志位 0 = 1，其余位 0");
        assert_eq!(JournalRecord::parse(&last_bytes, 7), Some(last));
        let not_last = JournalRecord {
            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
            ..empty_record(3)
        };
        let not_last_bytes = not_last.to_bytes();
        assert_eq!(not_last_bytes[7], 0, "不是末条：记录标志整字节 0");
        assert_eq!(JournalRecord::parse(&not_last_bytes, 7), Some(not_last));
        let mut differing: Vec<usize> = (0..last_bytes.len())
            .filter(|offset| last_bytes[*offset] != not_last_bytes[*offset])
            .collect();
        differing.retain(|offset| !(46..78).contains(offset));
        assert_eq!(
            differing,
            vec![7],
            "两条记录只差记录标志那 1 字节（与头部校验和）"
        );
    }

    /// 记录标志位 0 之外有位为 1 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4「其余位写 0，读到非 0 当损坏」）：
    /// 位 0 本身是不是 1 都一样拒。头部校验和按改过的字节重封过，拦住它的只有记录标志那一判。
    #[test]
    fn record_whose_flags_byte_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser() {
        for record_flags_byte in [0b0000_0010u8, 0b0000_0011, 0b1000_0001, 0xff] {
            let mut bytes = empty_record(4).to_bytes();
            bytes[7] = record_flags_byte;
            reseal_header_checksum(&mut bytes);
            assert_eq!(
                JournalRecord::parse(&bytes, 7),
                None,
                "记录标志 {record_flags_byte:#010b}：位 0 之外有位为 1，当损坏"
            );
        }
    }

    /// 本次发布内序号为 0 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4：序号从 1 起，读到 0 当那条记录损坏、断链即止）。
    /// 头部校验和按改过的字节重封过，拦住它的只有序号那一判。
    #[test]
    fn record_whose_ordinal_within_publish_is_zero_is_refused_by_the_parser() {
        let mut bytes = empty_record(5).to_bytes();
        bytes[87..91].copy_from_slice(&0u32.to_le_bytes());
        reseal_header_checksum(&mut bytes);
        assert_eq!(
            JournalRecord::parse(&bytes, 7),
            None,
            "序号 0：当这条记录损坏"
        );
    }

    /// 一次发布里从 0 数第 k 条记录的序号是 k + 1：第一条是 1（D23（journal 的角色与格式） 已定项 4「从 1 起」）。
    #[test]
    fn the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one() {
        assert_eq!(
            JournalRecordOrdinalWithinPublish::of_record_at_offset(0),
            JournalRecordOrdinalWithinPublish::FIRST
        );
        assert_eq!(JournalRecordOrdinalWithinPublish::FIRST.0, 1);
        assert_eq!(
            JournalRecordOrdinalWithinPublish::of_record_at_offset(143),
            JournalRecordOrdinalWithinPublish(144)
        );
    }

    /// 一条 4096 字节记录装 64 个点名项（(4096 − 496) ÷ 56 = 3600 ÷ 56 取整）：64 项的记录写得出、读得回；头里自述 65 项的记录
    /// （点名项区越过记录末尾）读者拒收，不按自述的项数切到 4096 之外（里程碑「覆盖写、释放、回退与复用」并行线一验收第 4 条第三个变异：
    /// 点名项超过一条记录装得下的仍塞进一条记录 ⇒ 记录解析拒收）。头校验和按改过的字节重算过，拦住它的只有项数那一判。
    #[test]
    fn record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser()
    {
        use singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD;
        let named_unit = NamedUnit {
            locations: [
                LocationEntry {
                    device: crate::address::DeviceIdentity(0),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
                LocationEntry {
                    device: crate::address::DeviceIdentity(1),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
            ],
            unit_class: 1,
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
            birth_txg: CheckpointTxg(3),
            key_tail: [0; 10],
        };
        let capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");
        let full = JournalRecord {
            named: vec![named_unit; capacity],
            ..empty_record(1)
        };
        let full_bytes = full.to_bytes();
        assert_eq!(
            JournalRecord::parse(&full_bytes, 7),
            Some(full),
            "64 项正好装满一条记录"
        );
        let mut claims_one_more = full_bytes;
        let named_count_offset = 4 + 2 + 1 + 1 + 4;
        claims_one_more[named_count_offset..named_count_offset + 4]
            .copy_from_slice(&u32::try_from(capacity + 1).expect("65").to_le_bytes());
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let digest = wide_checksum_with_field_zeroed(
            &claims_one_more,
            record_bytes,
            JOURNAL_HEADER_CHECKSUM_OFFSET,
        );
        claims_one_more[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        assert_eq!(
            JournalRecord::parse(&claims_one_more, 7),
            None,
            "自述 65 个点名项的记录：点名项区越过 4096，读者拒收"
        );
    }

    /// 读者收下的点名项 Vec 按项数预分配：一项的记录读回来容量恰好 1，不是逐项 push 从空 Vec 起第一次开的 4。
    /// 扫环把每条读到的记录都留到交回为止，多开的那 3 项堆按记录条数累加（E197（扫环胶水内存随盘数）分配器峰值的预测按恰好项数算）。
    #[test]
    fn a_parsed_record_holds_its_named_units_in_a_vector_of_exactly_the_named_count() {
        let named_unit = NamedUnit {
            locations: [
                LocationEntry {
                    device: crate::address::DeviceIdentity(0),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
                LocationEntry {
                    device: crate::address::DeviceIdentity(1),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
            ],
            unit_class: 1,
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
            birth_txg: CheckpointTxg(3),
            key_tail: [0; 10],
        };
        let record = JournalRecord {
            named: vec![named_unit],
            ..empty_record(1)
        };
        let parsed = JournalRecord::parse(&record.to_bytes(), 7)
            .expect("写者刚写出的一项记录，fsid 相同，读者收下");
        assert_eq!(parsed, record, "一项的记录读得回");
        assert_eq!(
            parsed.named.capacity(),
            1,
            "点名项 Vec 按项数 1 预分配，不多开"
        );
    }

    #[test]
    fn record_is_4096_with_a_496_byte_header_and_round_trips() {
        let record = empty_record(1);
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 4096);
        assert!(
            bytes[496..].iter().all(|byte| *byte == 0),
            "空记录 496 之后全 0"
        );
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record.clone()));
        assert_eq!(
            JournalRecord::parse(&bytes, 8),
            None,
            "fsid 不符的记录不进前缀"
        );
        let mut torn = bytes.clone();
        torn[4000] ^= 1;
        assert_eq!(
            JournalRecord::parse(&torn, 7),
            None,
            "header_csum 罩整条 4096"
        );
        let second = JournalRecord {
            back_chain: back_chain_of(&bytes),
            ..empty_record(2)
        };
        assert_ne!(second.back_chain, 0);
        assert_eq!(
            record_offset(1, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384)
        );
        assert_eq!(
            record_offset(3, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384 + 8192)
        );
        assert_eq!(
            record_offset(196_609, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384),
            "绕回环首"
        );
    }
}
