//! 三段对拍的独立参照：在测试里另写、不出自产品的那一份（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：审计与被审计
//! 不用同一段代码）。产品 GPU 的两份核与产品 CPU 那一份（`cpu_reference_outcome`、`cpu_reference_live_slots`、`cpu_reference_classification`，
//! 后两份背后是核心层）照 D26（后台整理与放置回收） 已定项 13 彼此对拍；这一份另判对错，三方两两相等才算过。
//!
//! 这里的函数只用格式常量（`singlefs_format`，与核心层 `unit` 模块里那几个单元头的格式常量：magic、版本、类标签、头校验和的偏移），
//! 不调产品 GPU 包与核心层的任何计算函数；入参都是平常的数与字节（把产品的类型换成这里的入参在 `common_independent_reference_projections`）。
//! 三份各换一种算法写法：
//! - CRC-32C：不反射的正规式（最高位先进），多项式 0x1EDC6F41，每个字节先位反转、按字节查一张 256 项的表（表在这里逐位移现算），
//!   末了寄存器位反转再取反。产品 CPU 那一份是 SSE4.2 的 CRC 指令（三路并行）或 slicing-by-8（反射式、8 张表），产品的两份 WGSL 核一份是
//!   反射式按字节一张表、一份是反射式按 32 位字逐位移。
//! - 每段活槽数：单元区整段那一截上记一张「罩住的记录数在这一槽起变多少」的差分表，前缀和得每槽被几条未释放记录罩住，再按段加起来。
//!   核心层那一份逐条记录逐槽落段，两份 WGSL 核一份逐段与每条记录求区间交、一份在排好的序号上二分逐槽数。
//! - 区域记录分类：按单元头的字节表（D18（块里携带什么信息） 的偏移表）判「收不收」：三类单元各自全部条件同时成立才收，收下后取映射 key、
//!   类标签与诞生代号；拒因只给类别（读不出头、类标签与条目 key 不符、key 与位置不符、已死），不分是哪一项头字段坏了，不照产品的检查次序。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use singlefs_core::unit::{
    HEADER_CHECKSUM_OFFSET, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
    UNIT_FORMAT_VERSION, UNIT_MAGIC,
};
use singlefs_format::{
    CHECKSUM_CRC32_CASTAGNOLI_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
    DATA_UNIT_HEADER_BYTES, DATA_UNIT_PAYLOAD_OFFSET, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, MAPPING_KEY_BYTES, NODE_BYTES,
    NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES, PACKED_UNIT_RECORDS_OFFSET,
    WIDE_CHECKSUM_BYTES,
};

// ============================== CRC-32C ==============================

/// CRC-32C（Castagnoli）多项式的正规式（最高位先进；反射式写作 0x82F63B78）。
pub const CASTAGNOLI_POLYNOMIAL_MOST_SIGNIFICANT_BIT_FIRST: u32 = 0x1EDC_6F41;
/// 寄存器的起始值，也是末了取反的那个掩码。
const CASTAGNOLI_REGISTER_START_AND_FINAL_FLIP: u32 = 0xFFFF_FFFF;
/// 寄存器的最高位。
const REGISTER_TOP_BIT: u32 = 0x8000_0000;

/// 最高位先进的按字节余数表：第 `top_byte` 项是寄存器最高字节为 `top_byte`、其余为 0 时移出 8 位后剩下的余数。
/// 循环：256 项，每项移 8 位，没有提前出口。
fn remainder_table_most_significant_bit_first() -> [u32; 256] {
    std::array::from_fn(|top_byte| {
        let mut remainder = u32::try_from(top_byte).expect("表下标小于 256") << 24;
        for _shifted_bit in 0..8 {
            remainder = if remainder & REGISTER_TOP_BIT == 0 {
                remainder << 1
            } else {
                (remainder << 1) ^ CASTAGNOLI_POLYNOMIAL_MOST_SIGNIFICANT_BIT_FIRST
            };
        }
        remainder
    })
}

/// `bytes` 的 CRC-32C，照文件头说的不反射正规式查表算。
#[must_use]
pub fn crc32c_by_the_most_significant_bit_first_table(bytes: &[u8]) -> u32 {
    let table = remainder_table_most_significant_bit_first();
    let mut register = CASTAGNOLI_REGISTER_START_AND_FINAL_FLIP;
    for byte in bytes {
        let top_byte = u8::try_from(register >> 24).expect("右移 24 位只剩 8 位");
        register = (register << 8) ^ table[usize::from(top_byte ^ byte.reverse_bits())];
    }
    register.reverse_bits() ^ CASTAGNOLI_REGISTER_START_AND_FINAL_FLIP
}

// ============================== 每段活槽数 ==============================

/// 一条分配记录：盘号、头一个槽、跨几个槽、释放了没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationRecordWords {
    pub device: u32,
    pub first_slot: u64,
    pub span_slots: u64,
    pub is_released: bool,
}

/// 一块盘上的单元区：盘号、头一个槽、槽数（末尾凑不满一段的那几个槽也算在里面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitAreaWords {
    pub device: u32,
    pub first_slot: u64,
    pub slot_count: u64,
}

fn as_index(number: u64) -> usize {
    usize::try_from(number).expect("单元区一截的槽数与段数装得进 usize")
}

/// 单元区每个整段（从单元区起点按 [`CLUSTER_SEGMENT_SLOTS`] 切，末尾凑不满一段的不算）的活槽数：
/// 单元区那块盘上、未释放的记录罩住、落在这一段里的槽数，几条记录罩住同一个槽各算一次。按段起点从小到大。
///
/// 循环：记录一条一次；整段那一截一个槽一次。跨轮带着的是此刻罩住这一槽的记录数，恒不为负。
#[must_use]
pub fn live_slots_per_full_segment_by_coverage_changes(
    records: &[AllocationRecordWords],
    unit_area: UnitAreaWords,
) -> Vec<u64> {
    let full_segments = unit_area.slot_count / CLUSTER_SEGMENT_SLOTS;
    let slots_in_the_full_segments = full_segments * CLUSTER_SEGMENT_SLOTS;
    let end_of_the_full_segments = unit_area.first_slot + slots_in_the_full_segments;
    let mut coverage_changes = vec![0_i64; as_index(slots_in_the_full_segments) + 1];
    for record in records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        let first_covered = record.first_slot.max(unit_area.first_slot);
        let end_covered = (record.first_slot + record.span_slots).min(end_of_the_full_segments);
        if first_covered < end_covered {
            coverage_changes[as_index(first_covered - unit_area.first_slot)] += 1;
            coverage_changes[as_index(end_covered - unit_area.first_slot)] -= 1;
        }
    }
    let mut live_slots = vec![0_u64; as_index(full_segments)];
    let mut records_covering_the_slot: i64 = 0;
    for (slot_offset, change) in coverage_changes
        .iter()
        .take(as_index(slots_in_the_full_segments))
        .enumerate()
    {
        records_covering_the_slot += change;
        live_slots[slot_offset / as_index(CLUSTER_SEGMENT_SLOTS)] +=
            u64::try_from(records_covering_the_slot)
                .expect("罩住一个槽的记录数不为负：差分表里每条记录先加后减");
    }
    live_slots
}

// ============================== 区域记录分类 ==============================

/// 一条中央映射条目：key 与两个落点（盘号、槽号）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MappingEntryWords<'mapping> {
    pub key: &'mapping [u8],
    pub locations: [(u32, u64); 2],
}

/// 一条记录分进哪一类。封闭集合：收（可搬）带收下后的三样；不收的只给类别。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionRecordClassWorkedOutIndependently {
    Movable {
        mapping_key: Vec<u8>,
        unit_class_tag: u8,
        birth_txg: u64,
    },
    /// 头读得出、没有条目指着它、头里的 key 映射里也没有。
    Dead,
    /// 读不到、跨度不是 1 也不是 2、或头不收。
    HeaderUnreadable,
    /// 有条目指着它，key 打头的类标签不是单元头的类。
    UnitClassDisagreesWithTheMappingKey,
    /// 头读得出、类标签都对得上，而指着它的条目不全带头里的 key，或没有条目指着它而头里的 key 在映射里。
    HeaderKeyDisagreesWithTheMappingLocation,
}

/// 头里读出的身份：类标签、反查出的映射 key、诞生代号。
struct HeaderIdentity {
    unit_class_tag: u8,
    mapping_key: Vec<u8>,
    birth_txg: u64,
}

fn as_offset(number: u64) -> usize {
    usize::try_from(number).expect("单元里的偏移装得进 usize")
}

fn little_endian_u16(unit: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(unit[offset..offset + 2].try_into().expect("切了 2 字节"))
}

fn little_endian_u32(unit: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(unit[offset..offset + 4].try_into().expect("切了 4 字节"))
}

fn little_endian_u64(unit: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(unit[offset..offset + 8].try_into().expect("切了 8 字节"))
}

/// 头校验和字段（共同前缀里 32 字节）收不收：罩 `[0, header_end)`、字段自身按 0 算的 CRC-32C 小端写在字段开头，其余全 0。
fn header_checksum_field_holds(unit: &[u8], header_end: usize) -> bool {
    let field = HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + as_offset(WIDE_CHECKSUM_BYTES);
    let mut covered = unit[..header_end].to_vec();
    covered[field.clone()].fill(0);
    let crc_bytes = crc32c_by_the_most_significant_bit_first_table(&covered).to_le_bytes();
    let crc_width = as_offset(CHECKSUM_CRC32_CASTAGNOLI_BYTES);
    unit[field.start..field.start + crc_width] == crc_bytes
        && unit[field.start + crc_width..field.end]
            .iter()
            .all(|byte| *byte == 0)
}

/// 三类单元共有的几条收不收：magic、版本、类标签、flags、头校验和、载荷校验和（罩 `[header_end, 末尾)`，写在 `payload_checksum_offset`）、
/// 头之后的加密预留位全 0。
fn shared_header_conditions_hold(
    unit: &[u8],
    unit_class_tag: u8,
    header_end: usize,
    payload_checksum_offset: usize,
) -> bool {
    let reserved = header_end..header_end + as_offset(NONCE_MAC_ALGORITHM_RESERVED_BYTES);
    unit[..UNIT_MAGIC.len()] == UNIT_MAGIC
        && little_endian_u16(unit, UNIT_MAGIC.len()) == UNIT_FORMAT_VERSION
        && unit[6] == unit_class_tag
        && unit[7] == 0
        && header_checksum_field_holds(unit, header_end)
        && little_endian_u32(unit, payload_checksum_offset)
            == crc32c_by_the_most_significant_bit_first_table(&unit[header_end..])
        && unit[reserved].iter().all(|byte| *byte == 0)
}

/// 声明长度（偏移 8 的两字节）。
fn declared_length(unit: &[u8]) -> usize {
    usize::from(little_endian_u16(unit, 8))
}

/// 映射 key：类标签接几段头字节，码 2 与码 3 末尾补两个 0；长度恒是 [`MAPPING_KEY_BYTES`]。
fn mapping_key_of(unit_class_tag: u8, pieces: &[&[u8]], zero_padding: usize) -> Vec<u8> {
    let mut key = vec![unit_class_tag];
    for piece in pieces {
        key.extend_from_slice(piece);
    }
    key.extend(std::iter::repeat_n(0_u8, zero_padding));
    assert_eq!(
        key.len(),
        as_offset(MAPPING_KEY_BYTES),
        "映射 key 一律 27 字节"
    );
    key
}

/// 码 2 索引节点（16384 字节）收下时的身份。偏移（key 宽 k 在 51）：42 树号、52+2k 诞生代号、68+2k 实例代号、72+2k 出生序号、
/// 76+2k 载荷 CRC、82+2k 条目数、84+2k 条目宽，明文头到 86+2k，预留位之后是条目区。
fn identity_of_an_index_node(unit: &[u8]) -> Option<HeaderIdentity> {
    if unit.len() != as_offset(NODE_BYTES) {
        return None;
    }
    let key_width = usize::from(unit[51]);
    let header_end = as_offset(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE)
        + as_offset(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT) * key_width;
    let entry_count = usize::from(little_endian_u16(unit, header_end - 4));
    let entry_width = usize::from(little_endian_u16(unit, header_end - 2));
    let declared = declared_length(unit);
    let is_accepted =
        shared_header_conditions_hold(unit, UNIT_CLASS_INDEX_NODE, header_end, header_end - 10)
            && entry_count * entry_width == declared
            && (entry_width != 0 || entry_count == 0)
            && entry_width >= key_width
            && header_end + as_offset(NONCE_MAC_ALGORITHM_RESERVED_BYTES) + declared <= unit.len();
    let birth_txg_offset = header_end - 34;
    let instance_offset = header_end - 18;
    is_accepted.then(|| HeaderIdentity {
        unit_class_tag: UNIT_CLASS_INDEX_NODE,
        mapping_key: mapping_key_of(
            UNIT_CLASS_INDEX_NODE,
            &[
                &unit[42..50],
                &unit[birth_txg_offset..birth_txg_offset + 8],
                &unit[instance_offset..instance_offset + 8],
            ],
            2,
        ),
        birth_txg: little_endian_u64(unit, birth_txg_offset),
    })
}

/// 码 1 数据单元（32768 字节）收下时的身份。偏移：42 类标签副本、43 树号、75 诞生代号、91 写序（实例代号 4、事务号 6）、101 载荷 CRC，
/// 头到 105，预留位之后（134）是载荷，声明长度的载荷要装得下。
fn identity_of_a_data_unit(unit: &[u8]) -> Option<HeaderIdentity> {
    let header_end = as_offset(DATA_UNIT_HEADER_BYTES);
    let is_accepted = shared_header_conditions_hold(unit, UNIT_CLASS_DATA, header_end, 101)
        && unit[42] == UNIT_CLASS_DATA
        && as_offset(DATA_UNIT_PAYLOAD_OFFSET) + declared_length(unit) <= unit.len();
    is_accepted.then(|| HeaderIdentity {
        unit_class_tag: UNIT_CLASS_DATA,
        mapping_key: mapping_key_of(
            UNIT_CLASS_DATA,
            &[&unit[43..51], &unit[75..83], &unit[91..101]],
            0,
        ),
        birth_txg: little_endian_u64(unit, 75),
    })
}

/// 码 3 打包记录单元（32768 字节）收下时的身份。偏移：42 类标签副本、43 出生树、69 记录数、71 记录宽、73 诞生代号、89 载荷 CRC、
/// 93 写序（实例代号 4、事务号 6）、103 出生序号，头到 107，预留位之后（136）是记录区。
fn identity_of_a_packed_unit(unit: &[u8]) -> Option<HeaderIdentity> {
    let header_end = as_offset(PACKED_UNIT_HEADER_BYTES);
    let record_count = usize::from(little_endian_u16(unit, 69));
    let record_width = usize::from(little_endian_u16(unit, 71));
    let declared = declared_length(unit);
    let is_accepted = shared_header_conditions_hold(unit, UNIT_CLASS_PACKED, header_end, 89)
        && unit[42] == UNIT_CLASS_PACKED
        && record_count * record_width == declared
        && (record_width != 0 || record_count == 0)
        && as_offset(PACKED_UNIT_RECORDS_OFFSET) + declared <= unit.len();
    is_accepted.then(|| HeaderIdentity {
        unit_class_tag: UNIT_CLASS_PACKED,
        mapping_key: mapping_key_of(
            UNIT_CLASS_PACKED,
            &[&unit[43..51], &unit[73..81], &unit[93..97], &unit[103..107]],
            2,
        ),
        birth_txg: little_endian_u64(unit, 73),
    })
}

/// 跨两个槽的单元：32768 字节，类标签是 1 的当码 1、是 3 的当码 3，别的不收。
fn identity_of_a_two_slot_unit(unit: &[u8]) -> Option<HeaderIdentity> {
    if unit.len() != as_offset(DATA_UNIT_BYTES) {
        return None;
    }
    match unit[6] {
        tag if tag == UNIT_CLASS_DATA => identity_of_a_data_unit(unit),
        tag if tag == UNIT_CLASS_PACKED => identity_of_a_packed_unit(unit),
        _ => None,
    }
}

/// 落在盘 `record_device`、槽 `record_slot`、跨 `span_slots` 个槽的一条记录（读到的单元字节 `unit_bytes`，`None` 是读不到）
/// 在中央映射 `mapping_entries` 下分进哪一类。
#[must_use]
pub fn region_record_class_worked_out_independently(
    record_device: u32,
    record_slot: u64,
    span_slots: u64,
    unit_bytes: Option<&[u8]>,
    mapping_entries: &[MappingEntryWords<'_>],
) -> RegionRecordClassWorkedOutIndependently {
    let identity = unit_bytes.and_then(|unit| match span_slots {
        1 => identity_of_an_index_node(unit),
        2 => identity_of_a_two_slot_unit(unit),
        _ => None,
    });
    let Some(identity) = identity else {
        return RegionRecordClassWorkedOutIndependently::HeaderUnreadable;
    };
    let entries_pointing_here: Vec<&MappingEntryWords<'_>> = mapping_entries
        .iter()
        .filter(|entry| entry.locations.contains(&(record_device, record_slot)))
        .collect();
    let an_entry_names_another_class = entries_pointing_here
        .iter()
        .any(|entry| entry.key.first() != Some(&identity.unit_class_tag));
    let every_entry_carries_the_header_key = entries_pointing_here
        .iter()
        .all(|entry| entry.key == identity.mapping_key.as_slice());
    let header_key_is_mapped = mapping_entries
        .iter()
        .any(|entry| entry.key == identity.mapping_key.as_slice());
    match (
        an_entry_names_another_class,
        entries_pointing_here.is_empty(),
        every_entry_carries_the_header_key,
        header_key_is_mapped,
    ) {
        (true, _, _, _) => {
            RegionRecordClassWorkedOutIndependently::UnitClassDisagreesWithTheMappingKey
        }
        (false, true, _, false) => RegionRecordClassWorkedOutIndependently::Dead,
        (false, false, true, _) => RegionRecordClassWorkedOutIndependently::Movable {
            mapping_key: identity.mapping_key,
            unit_class_tag: identity.unit_class_tag,
            birth_txg: identity.birth_txg,
        },
        (false, true, _, true) | (false, false, false, _) => {
            RegionRecordClassWorkedOutIndependently::HeaderKeyDisagreesWithTheMappingLocation
        }
    }
}
