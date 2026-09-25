//! checker：与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5），
//! CRC-32C、解析、校验各写一份，不从 `singlefs-core` 引任何东西。
//! 步 1 能判的：系统配置槽（magic / 整槽校验和 / incompat 位 / 世代号）、根记录槽（magic / 整槽校验和 / fsid / flags）、
//! 单元头（共同前缀 / 头校验和 / 载荷 CRC），以及「三个区域的槽 0 是不是同一份第 0 代根」。
//! 步 3 / 步 4 / 步 5 加的：码 2 节点的头与条目（声明长度、key 宽、key 按字段序严格递增、区间贴紧）、码 3 容器的记录、
//! journal 记录（`header_csum` 罩整条 4096、载荷 CRC、点名项、反向链口径）。
#![forbid(unsafe_code)]

pub mod image;
pub mod position_addressed;
pub mod walk;

use singlefs_format::{
    index_node_header_bytes, DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, JOURNAL_HEADER_BYTES,
    JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_RECORD_BYTES, NODE_BYTES,
    NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES, ROLLBACK_WITNESS_COUNT_BYTES,
    ROLLBACK_WITNESS_ENTRIES_MAXIMUM, ROLLBACK_WITNESS_ENTRY_BYTES,
    ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT, ROOT_RECORD_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES, WIDE_CHECKSUM_BYTES,
};

/// checker 自己解析出来的判定宽度：探测到的，或系统配置声明的（探不到时报「声明值，未探测」，不许当成探到的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionWidth {
    Probed { bytes: u32 },
    DeclaredOnly { bytes: u32 },
}

impl DecisionWidth {
    #[must_use]
    pub const fn bytes(self) -> u32 {
        match self {
            DecisionWidth::Probed { bytes } | DecisionWidth::DeclaredOnly { bytes } => bytes,
        }
    }
}

/// 按位的 CRC-32C（Castagnoli，反射，初值与输出异或全 1）——故意不用查表：与实现那份 slicing-by-8 独立。
#[must_use]
pub fn crc32_castagnoli_bitwise(bytes: &[u8]) -> u32 {
    const POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;
    let mut remainder = !0u32;
    for byte in bytes {
        remainder ^= u32::from(*byte);
        for _bit in 0..8 {
            remainder = if remainder & 1 == 1 {
                (remainder >> 1) ^ POLYNOMIAL_REFLECTED
            } else {
                remainder >> 1
            };
        }
    }
    !remainder
}

/// 查表的 CRC-32C（反射多项式 0x82F63B78，一次一个字节、一张 256 项的表）：与按位那一份逐字节同值（单测对拍），
/// 与实现那份 slicing-by-8 仍是两种写法；池级 checker 在层 0 的每个崩溃状态上要算几十个单元，按位算太慢。
#[must_use]
pub fn crc32_castagnoli_table(bytes: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut entries = [0u32; 256];
        for (index, entry) in entries.iter_mut().enumerate() {
            let mut remainder = u32::try_from(index).expect("小于 256");
            for _bit in 0..8 {
                remainder = if remainder & 1 == 1 {
                    (remainder >> 1) ^ 0x82F6_3B78
                } else {
                    remainder >> 1
                };
            }
            *entry = remainder;
        }
        entries
    });
    let mut remainder = !0u32;
    for byte in bytes {
        remainder = table[usize::from(u8::try_from(remainder & 0xFF).expect("低 8 位") ^ *byte)]
            ^ (remainder >> 8);
    }
    !remainder
}

pub(crate) fn checksum_field_holds(bytes: &[u8], cover_end: usize, field_offset: usize) -> bool {
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    if bytes.len() < cover_end || field_offset + field_width > cover_end {
        return false;
    }
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + field_width].fill(0);
    let expected = crc32_castagnoli_bitwise(&covered).to_le_bytes();
    let stored = &bytes[field_offset..field_offset + field_width];
    stored[..4] == expected && stored[4..].iter().all(|byte| *byte == 0)
}

/// 一个槽 / 单元判下来的结论；每个成员是一条读者能据以行动的理由。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Valid,
    BadMagic,
    ChecksumMismatch,
    UnknownIncompatBit,
    FilesystemIdentifierMismatch,
    NonZeroFlags,
    TooShort,
    /// 要的是这一类单元，读到的是另一类（类标签或类标签副本对不上）。
    WrongUnitClass,
    /// 声明长度 ≠ 条目数 × 条目宽，或记录长度字段与 4096 不符（D8（核心索引结构） 已定项 11）。
    DeclaredLengthMismatch,
    /// 节点自述的 key 宽与这棵树的 key 形态不符。
    KeyWidthMismatch,
    /// 条目 key 没有按字段序严格递增（D8（核心索引结构） 已定项 11）。
    KeysNotStrictlyAscending,
    /// 条目 key 落在头里声明的 key 区间之外，或区间没有贴紧首末条目。
    KeyOutsideDeclaredRange,
    /// journal 记录类型不在登记表里。
    UnknownRecordType,
    /// 系统配置自述的区域数 R 大于字段表给逐区域设备身份留的字段数
    /// （[`crate::image::REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION`]）：
    /// 第 3 个及以后的区域没有设备身份可读，这份系统配置这个格式版本读不了，这一槽不可择。
    RegionCountPastTheRegionDeviceFields,
    /// 系统配置自述的每区槽数 S（偏移 362 那一字节）落在格式承诺的区间之外
    /// （`singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM`..`_MAXIMUM`，D22（单元原子性怎么合成） 已定项 1 的字段表）：
    /// S 是**盘上读来的一字节**，可以是 0..255 里的任何一个，而根环的每一处走读都按它算
    /// （`image::root_slot_positions` 枚举 R × S 个槽、`walk` 按 R × S 算环长）。
    /// 不夹到区间里往下走（那是按一个池里根本不存在的几何走读，少读或多读几个槽而没人说），
    /// 也不 panic：S = 0 会让环长变 0、实现侧取模除零。挂载侧同一条判定是
    /// `singlefs_core::recovery::RecoveryFailure::RootRingSlotsPerRegionOutOfRange`。
    RootRingSlotsPerRegionOutsideTheFormatInterval,
}

/// 回退见证表的一个条目（D23（journal 的角色与格式） 已定项 14「回退见证」）：新实例代号 4 + 回退目标 R_old 的实例代号 4 + txg 8，小端。
/// 偏移与宽度只从 `singlefs-format` 取，解析在 checker 这边另写一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RollbackWitnessEntryView {
    pub new_instance: u32,
    pub rollback_target_instance: u32,
    pub rollback_target_txg: u64,
}

impl RollbackWitnessEntryView {
    /// (实例代号, txg) 这一处被这次回退抛弃：(r_old, T_old) < (i, T)（实例代号为主比）且 i < N。
    #[must_use]
    pub fn abandons(&self, instance: u32, checkpoint_txg: u64) -> bool {
        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)
            && instance < self.new_instance
    }
}

/// 一个自证过的系统配置槽里的回退见证表：槽内偏移 481 起，条数 1 字节 + 47 个 16 字节的条目位。`capacity` 是这个池的条数上限
/// （R × S − 1，R、S 读自同一槽）。条数不超过上限、条目按 (N, r_old, T_old) 严格升序、每一条 r_old < N、条数之后的条目位全 0，
/// 四样都满足才交回条目；不满足交回哪一样不满足（I-7.10（回退见证表各槽自洽、各盘一致） 的违例说明要它）。
///
/// # Errors
/// 上面四样里第一样不满足的，一句话。
pub fn rollback_witness_of_system_configuration_slot(
    slot: &[u8],
    capacity: u64,
) -> Result<Vec<RollbackWitnessEntryView>, &'static str> {
    let start = usize::try_from(ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT)
        .expect("481");
    let count_bytes = usize::try_from(ROLLBACK_WITNESS_COUNT_BYTES).expect("1");
    let entry_bytes = usize::try_from(ROLLBACK_WITNESS_ENTRY_BYTES).expect("16");
    let positions = usize::try_from(ROLLBACK_WITNESS_ENTRIES_MAXIMUM).expect("47");
    if slot.len() < start + count_bytes + positions * entry_bytes {
        return Err("槽比见证表的末尾短");
    }
    let count = u64::from(slot[start]);
    if count > capacity.min(ROLLBACK_WITNESS_ENTRIES_MAXIMUM) {
        return Err("条数超过这个池的上限 R × S − 1");
    }
    let mut entries: Vec<RollbackWitnessEntryView> = Vec::new();
    for position in 0..positions {
        let base = start + count_bytes + position * entry_bytes;
        let entry = RollbackWitnessEntryView {
            new_instance: read_u32(slot, base),
            rollback_target_instance: read_u32(slot, base + 4),
            rollback_target_txg: read_u64(slot, base + 8),
        };
        if u64::try_from(position).expect("47 以内") < count {
            if entry.rollback_target_instance >= entry.new_instance {
                return Err("有一条的回退目标实例代号不小于新实例代号");
            }
            if entries.last().is_some_and(|previous| *previous >= entry) {
                return Err("条目不按 (新实例, 目标实例, 目标 txg) 严格升序");
            }
            entries.push(entry);
        } else if slot[base..base + entry_bytes].iter().any(|byte| *byte != 0) {
            return Err("条数之后的条目位不全是 0");
        }
    }
    Ok(entries)
}

/// 系统配置槽解出来的几个要紧字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationView {
    pub filesystem_identifier: [u8; 16],
    pub this_device: u32,
    pub device_count: u32,
    pub slot_generation: u64,
    pub declared_physical_block_size: u32,
    pub journal_tail: u64,
    pub journal_instance: u32,
}

const SYSTEM_CONFIGURATION_MAGIC: &[u8; 4] = b"SFSB";
const ROOT_MAGIC: &[u8; 4] = b"SFSR";
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET: usize = 6;
const SYSTEM_CONFIGURATION_FSID_OFFSET: usize = 102;
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
const SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET: usize =
    155 + 32 + 16 + 12 + 4 + 1 + 1 + 80 + 16;
const SYSTEM_CONFIGURATION_TAIL_OFFSET: usize = 469;
const ROOT_CHECKSUM_OFFSET: usize = 138;
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}
pub(crate) fn read_six_byte_unsigned(bytes: &[u8], offset: usize) -> u64 {
    let mut widened = [0u8; 8];
    widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(widened)
}
pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}
pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 判一个系统配置槽（4096 字节）。
pub fn check_system_configuration_slot(slot: &[u8]) -> Result<SystemConfigurationView, Verdict> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    if slot.len() < slot_bytes {
        return Err(Verdict::TooShort);
    }
    if &slot[..4] != SYSTEM_CONFIGURATION_MAGIC {
        return Err(Verdict::BadMagic);
    }
    if !checksum_field_holds(slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) {
        return Err(Verdict::ChecksumMismatch);
    }
    let incompat = &slot
        [SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 32];
    if incompat[0] & !0x01 != 0
        || incompat[1..].iter().any(|byte| *byte != 0)
        || incompat[0] & 0x01 == 0
    {
        return Err(Verdict::UnknownIncompatBit);
    }
    Ok(SystemConfigurationView {
        filesystem_identifier: slot
            [SYSTEM_CONFIGURATION_FSID_OFFSET..SYSTEM_CONFIGURATION_FSID_OFFSET + 16]
            .try_into()
            .expect("16 字节"),
        this_device: read_u32(slot, SYSTEM_CONFIGURATION_FSID_OFFSET + 16 + 20 + 1),
        device_count: read_u32(slot, SYSTEM_CONFIGURATION_FSID_OFFSET + 16 + 20 + 1 + 4),
        slot_generation: read_u64(slot, SYSTEM_CONFIGURATION_FSID_OFFSET + 16 + 20 + 1 + 8),
        declared_physical_block_size: read_u32(
            slot,
            SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET,
        ),
        journal_tail: read_u64(slot, SYSTEM_CONFIGURATION_TAIL_OFFSET),
        journal_instance: read_u32(slot, SYSTEM_CONFIGURATION_TAIL_OFFSET + 8),
    })
}

/// 择槽（D22（单元原子性怎么合成） 已定项 16）：校验和过且世代号最大的那个。
#[must_use]
pub fn choose_system_configuration(slots: &[&[u8]]) -> Option<(usize, SystemConfigurationView)> {
    slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| {
            check_system_configuration_slot(slot)
                .ok()
                .map(|view| (index, view))
        })
        .max_by_key(|(_, view)| view.slot_generation)
}

/// 根记录槽解出来的要紧字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootView {
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: u64,
    /// 457 字节记录本身（含校验和字段），三个区域比对用。
    pub record_bytes: Vec<u8>,
}

/// 判一个根槽（判定宽度那么宽）。
pub fn check_root_slot(
    slot: &[u8],
    expected_filesystem_identifier: &[u8; 16],
) -> Result<RootView, Verdict> {
    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("457");
    if slot.len() < record_bytes {
        return Err(Verdict::TooShort);
    }
    if &slot[..4] != ROOT_MAGIC {
        return Err(Verdict::BadMagic);
    }
    if !checksum_field_holds(slot, slot.len(), ROOT_CHECKSUM_OFFSET) {
        return Err(Verdict::ChecksumMismatch);
    }
    if &slot[4..20] != expected_filesystem_identifier {
        return Err(Verdict::FilesystemIdentifierMismatch);
    }
    if read_u32(slot, 20) != 0 {
        return Err(Verdict::NonZeroFlags);
    }
    Ok(RootView {
        instance: read_u32(slot, 24),
        checkpoint_txg: read_u64(slot, 28),
        tree_identifier_watermark: read_u64(slot, 36 + 86),
        rollback_floor: read_u64(slot, 36 + 86 + 8),
        record_bytes: slot[..record_bytes].to_vec(),
    })
}

/// 判一个单元的头：magic、flags、类标签合法（1 / 2 / 3）、头校验和、载荷 CRC；返回类标签。
pub fn check_unit(unit: &[u8]) -> Result<u8, Verdict> {
    if unit.len() < 42 {
        return Err(Verdict::TooShort);
    }
    if &unit[..4] != UNIT_MAGIC {
        return Err(Verdict::BadMagic);
    }
    if unit[7] != 0 {
        return Err(Verdict::NonZeroFlags);
    }
    let unit_class = unit[6];
    let (header_end, expected_length_in_bytes, payload_crc_offset) = match unit_class {
        1 => (
            usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105"),
            DATA_UNIT_BYTES,
            101,
        ),
        2 => {
            let header_end = usize::try_from(
                index_node_header_bytes(u64::from(unit[51])) - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
            )
            .expect("头宽");
            (header_end, NODE_BYTES, header_end - 10)
        }
        3 => (
            usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),
            DATA_UNIT_BYTES,
            89,
        ),
        _ => return Err(Verdict::BadMagic),
    };
    if unit.len() != usize::try_from(expected_length_in_bytes).expect("单元长度") {
        return Err(Verdict::TooShort);
    }
    if !checksum_field_holds(unit, header_end, UNIT_HEADER_CHECKSUM_OFFSET) {
        return Err(Verdict::ChecksumMismatch);
    }
    if read_u32(unit, payload_crc_offset) != crc32_castagnoli_bitwise(&unit[header_end..]) {
        return Err(Verdict::ChecksumMismatch);
    }
    Ok(unit_class)
}

/// 码 2 索引节点解出来的头与条目（D18（块里携带什么信息） 已定项 18 的偏移表：42 树 ID / 50 层级 / 51 key 宽 / 52 key 区间 /
/// 52+2k 诞生代号 / 60+2k fsid / 68+2k 写序 4 / 72+2k 出生序号 / 76+2k 载荷 CRC / 80+2k 预留 2 / 82+2k 条目数 / 84+2k 条目宽 / 115+2k 条目区）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexNodeView {
    pub tree_identifier: u64,
    pub level: u8,
    pub key_width: usize,
    pub smallest_key: Vec<u8>,
    pub largest_key: Vec<u8>,
    pub birth_txg: u64,
    pub filesystem_identifier: u64,
    pub instance: u32,
    pub birth_sequence: u32,
    pub entry_width: usize,
    pub entries: Vec<Vec<u8>>,
}

/// 解一个码 2 节点：头与载荷两道校验和先过，再核声明长度 = 条目数 × 条目宽、条目宽 ≥ key 宽、条目区装得下。
pub fn index_node_view(unit: &[u8]) -> Result<IndexNodeView, Verdict> {
    if check_unit(unit)? != 2 {
        return Err(Verdict::WrongUnitClass);
    }
    let key_width = usize::from(unit[51]);
    let mut offset = 52;
    let smallest_key = unit[offset..offset + key_width].to_vec();
    offset += key_width;
    let largest_key = unit[offset..offset + key_width].to_vec();
    offset += key_width;
    let birth_txg = read_u64(unit, offset);
    let node_filesystem_identifier = read_u64(unit, offset + 8);
    offset += 8 + 8;
    let instance = read_u32(unit, offset);
    offset += 4;
    let birth_sequence = read_u32(unit, offset);
    offset += 4 + 4 + 2;
    let entry_count = usize::from(read_u16(unit, offset));
    offset += 2;
    let entry_width = usize::from(read_u16(unit, offset));
    offset += 2;
    assert_eq!(offset, 86 + 2 * key_width, "明文头末尾 = 86 + 2k");
    let declared_length = usize::from(read_u16(unit, 8));
    if declared_length != entry_count * entry_width || entry_width < key_width {
        return Err(Verdict::DeclaredLengthMismatch);
    }
    let entries_start = offset + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    if entries_start + declared_length > unit.len() {
        return Err(Verdict::TooShort);
    }
    let entries = unit[entries_start..entries_start + declared_length]
        .chunks(entry_width.max(1))
        .take(entry_count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(IndexNodeView {
        tree_identifier: read_u64(unit, 42),
        level: unit[50],
        key_width,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier: node_filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    })
}

/// key 的字段宽度表：按字段自左向右比无符号整数（D8（核心索引结构） 已定项 11），小端存储不构成 memcmp 序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeySchema {
    pub field_widths: &'static [usize],
}

pub const KEY_SCHEMA_EXTENT: KeySchema = KeySchema {
    field_widths: &[8, 8, 8],
};
pub const KEY_SCHEMA_INODE: KeySchema = KeySchema { field_widths: &[8] };
pub const KEY_SCHEMA_ALLOCATION: KeySchema = KeySchema {
    field_widths: &[4, 6],
};
pub const KEY_SCHEMA_ACCOUNTING: KeySchema = KeySchema {
    field_widths: &[2, 8, 4, 8],
};
/// 映射 key 27：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6（码 1 是事务号低 48 位；码 2 / 码 3 是出生序号 4 + 补零 2，按 6 字节整数比与按出生序号比同序）。
pub const KEY_SCHEMA_MAPPING: KeySchema = KeySchema {
    field_widths: &[1, 8, 8, 4, 6],
};
pub const KEY_SCHEMA_TREE_TABLE: KeySchema = KeySchema { field_widths: &[8] };

impl KeySchema {
    #[must_use]
    pub fn width(self) -> usize {
        self.field_widths.iter().sum()
    }
    #[must_use]
    pub fn fields(self, key: &[u8]) -> Vec<u64> {
        let mut offset = 0;
        self.field_widths
            .iter()
            .map(|width| {
                let value = match *width {
                    1 => u64::from(key[offset]),
                    2 => u64::from(read_u16(key, offset)),
                    4 => u64::from(read_u32(key, offset)),
                    6 => read_six_byte_unsigned(key, offset),
                    8 => read_u64(key, offset),
                    other => panic!("key 字段宽 {other} 不在登记表里"),
                };
                offset += width;
                value
            })
            .collect()
    }
}

/// 树的种类码 → key 形态（D8（核心索引结构） 已定项 9 / 已定项 11）；livelist 6、稀疏旁表 7、deadlist 8 三棵 day-1 只注册、没有节点要解。
#[must_use]
pub fn key_schema_for_tree_kind(kind: u16) -> Option<KeySchema> {
    match kind {
        1 => Some(KEY_SCHEMA_EXTENT),
        2 => Some(KEY_SCHEMA_INODE),
        3 => Some(KEY_SCHEMA_ALLOCATION),
        4 => Some(KEY_SCHEMA_ACCOUNTING),
        5 => Some(KEY_SCHEMA_MAPPING),
        _ => None,
    }
}

/// 判一个码 2 节点的 key：自述 key 宽等于形态宽、条目 key 按字段序严格递增、区间贴紧首末条目（空节点区间任意）。
pub fn check_index_node_keys(view: &IndexNodeView, schema: KeySchema) -> Result<(), Verdict> {
    if schema.width() != view.key_width {
        return Err(Verdict::KeyWidthMismatch);
    }
    let key_of = |entry: &Vec<u8>| schema.fields(&entry[..view.key_width]);
    for pair in view.entries.windows(2) {
        if key_of(&pair[0]) >= key_of(&pair[1]) {
            return Err(Verdict::KeysNotStrictlyAscending);
        }
    }
    if let (Some(first), Some(last)) = (view.entries.first(), view.entries.last()) {
        if key_of(first) != schema.fields(&view.smallest_key)
            || key_of(last) != schema.fields(&view.largest_key)
        {
            return Err(Verdict::KeyOutsideDeclaredRange);
        }
    }
    Ok(())
}

/// 判一个多层码 2 树内部节点的 key（D8（核心索引结构） 已定项 11）：自述 key 宽等于形态宽、条目按分隔 key 严格递增、节点不空。
/// 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），不贴紧首末两条分隔 key——它要等孩子读回来才判得了，
/// 由走读的一方判，这里不判。
pub fn check_internal_node_separators(
    view: &IndexNodeView,
    schema: KeySchema,
) -> Result<(), Verdict> {
    if schema.width() != view.key_width {
        return Err(Verdict::KeyWidthMismatch);
    }
    if view.entries.is_empty() {
        return Err(Verdict::KeyOutsideDeclaredRange);
    }
    let key_of = |entry: &Vec<u8>| schema.fields(&entry[..view.key_width]);
    for pair in view.entries.windows(2) {
        if key_of(&pair[0]) >= key_of(&pair[1]) {
            return Err(Verdict::KeysNotStrictlyAscending);
        }
    }
    Ok(())
}

/// 码 3 打包记录单元解出来的头与记录（D18（块里携带什么信息） 已定项 11 / 已定项 16：记录区从 136 起）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedUnitView {
    pub birth_tree: u64,
    pub record_type: u16,
    pub container: u64,
    pub container_birth: u64,
    pub birth_txg: u64,
    pub filesystem_identifier: u64,
    pub record_width: usize,
    pub records: Vec<Vec<u8>>,
}

pub fn packed_unit_view(unit: &[u8]) -> Result<PackedUnitView, Verdict> {
    if check_unit(unit)? != 3 || unit[42] != 3 {
        return Err(Verdict::WrongUnitClass);
    }
    let record_count = usize::from(read_u16(unit, 69));
    let record_width = usize::from(read_u16(unit, 71));
    let declared_length = usize::from(read_u16(unit, 8));
    if declared_length != record_count * record_width {
        return Err(Verdict::DeclaredLengthMismatch);
    }
    let records_start =
        usize::try_from(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES)
            .expect("136");
    if records_start + declared_length > unit.len() {
        return Err(Verdict::TooShort);
    }
    let records = unit[records_start..records_start + declared_length]
        .chunks(record_width.max(1))
        .take(record_count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(PackedUnitView {
        birth_tree: read_u64(unit, 43),
        record_type: read_u16(unit, 51),
        container: read_u64(unit, 53),
        container_birth: read_u64(unit, 61),
        birth_txg: read_u64(unit, 73),
        filesystem_identifier: read_u64(unit, 81),
        record_width,
        records,
    })
}

/// journal 记录头的偏移（D23（journal 的角色与格式） 已定项 4 的字段表）：magic 4 + 类型 2 + 算法类型 1 之后是记录标志 1；
/// 事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，再是反向链 4、载荷校验和 4、新根段 188、fsid 8、MAC 16，头到 311 为止。
const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;
const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 46;
const JOURNAL_TRANSACTION_OFFSET: usize = 78;
const JOURNAL_COMMIT_MARKER_OFFSET: usize = 86;
const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize = 87;
const JOURNAL_BACK_CHAIN_OFFSET: usize = 91;
const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: usize = 287;

/// 点名项解出来的样子（D23（journal 的角色与格式） 已定项 17）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedEntryView {
    /// (设备身份, 槽号, 整单元校验和) × 2。
    pub locations: [(u32, u64, u32); 2],
    pub unit_class: u8,
    pub birth_tree: u64,
    pub birth_txg: u64,
    pub key_tail: [u8; 10],
}

impl NamedEntryView {
    /// 重放不查单元就能凑出的 27 字节映射 key。
    #[must_use]
    pub fn mapping_key(&self) -> Vec<u8> {
        let mut key = Vec::with_capacity(27);
        key.push(self.unit_class);
        key.extend_from_slice(&self.birth_tree.to_le_bytes());
        key.extend_from_slice(&self.birth_txg.to_le_bytes());
        key.extend_from_slice(&self.key_tail);
        key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalRecordView {
    pub instance: u32,
    pub counter: u64,
    pub checkpoint_txg: u64,
    pub transaction: u64,
    /// 提交标记那 1 字节等于 1。
    pub is_commit: bool,
    /// 提交标记那 1 字节原样（D23（journal 的角色与格式） 已定项 7）：只许 0 或 1，别的值 I-8.8（前缀里的事务不被切开） 判红，
    /// 所以不能只留 `is_commit`——那一步把 2..=255 静默读成「不带」。
    pub commit_marker_byte: u8,
    /// 本次发布内序号（D23（journal 的角色与格式） 已定项 4）：一次发布 N 条记录依次是 1..N，只有一条时是 1。
    /// 读到的原样：序号 0 在这里不拒，I-8.9（一次发布的记录序号连续且只有末条带标志） 判红。
    pub ordinal_within_publish: u32,
    /// 记录标志那 1 字节原样（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：位 0 = 本次发布末条，其余位只许 0。
    /// 不在这里拒其余位非 0 的记录，I-8.9（一次发布的记录序号连续且只有末条带标志） 判红——拒了它就看不见。
    pub record_flags_byte: u8,
    pub back_chain: u32,
    /// 新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
    pub new_root_segment: Vec<u8>,
    pub new_tree_identifier_watermark: u64,
    pub new_rollback_floor: u64,
    pub named: Vec<NamedEntryView>,
}

/// 反向链的口径（D23（journal 的角色与格式） 已定项 19 ②）：前一条记录 311 字节头的 CRC-32C，`header_csum` 那 32 字节按 0 参与。
#[must_use]
pub fn back_chain_of_record_header(record: &[u8]) -> u32 {
    let mut header = record[..usize::try_from(JOURNAL_HEADER_BYTES).expect("311")].to_vec();
    header[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].fill(0);
    crc32_castagnoli_bitwise(&header)
}

/// 判一条 journal 记录：magic、`header_csum` 罩整条 4096（D23（journal 的角色与格式） 已定项 13）、类型、记录长度、fsid、载荷 CRC、点名项 flags。
pub fn check_journal_record(
    record: &[u8],
    expected_filesystem_identifier: u64,
) -> Result<JournalRecordView, Verdict> {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    if record.len() < record_bytes {
        return Err(Verdict::TooShort);
    }
    if &record[..4] != JOURNAL_MAGIC {
        return Err(Verdict::BadMagic);
    }
    if !checksum_field_holds(record, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET) {
        return Err(Verdict::ChecksumMismatch);
    }
    if read_u16(record, 4) != 1 {
        return Err(Verdict::UnknownRecordType);
    }
    if u64::from(read_u32(record, 8)) != JOURNAL_RECORD_BYTES {
        return Err(Verdict::DeclaredLengthMismatch);
    }
    if read_u64(record, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET) != expected_filesystem_identifier {
        return Err(Verdict::FilesystemIdentifierMismatch);
    }
    let named_count = usize::try_from(read_u32(record, 12)).expect("点名项数");
    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("311");
    let entry_bytes = usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
    let payload_end = header_bytes + named_count * entry_bytes;
    if payload_end > record_bytes {
        return Err(Verdict::DeclaredLengthMismatch);
    }
    if read_u32(record, JOURNAL_PAYLOAD_CHECKSUM_OFFSET)
        != crc32_castagnoli_bitwise(&record[header_bytes..payload_end])
    {
        return Err(Verdict::ChecksumMismatch);
    }
    let mut named = Vec::with_capacity(named_count);
    for index in 0..named_count {
        let entry =
            &record[header_bytes + index * entry_bytes..header_bytes + (index + 1) * entry_bytes];
        if entry[55] != 0 {
            return Err(Verdict::NonZeroFlags);
        }
        let location = |start: usize| {
            (
                read_u32(entry, start),
                read_six_byte_unsigned(entry, start + 4),
                read_u32(entry, start + 10),
            )
        };
        named.push(NamedEntryView {
            locations: [location(0), location(14)],
            unit_class: entry[28],
            birth_tree: read_u64(entry, 29),
            birth_txg: read_u64(entry, 37),
            key_tail: entry[45..55].try_into().expect("10 字节"),
        });
    }
    let new_root_segment =
        record[JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET].to_vec();
    Ok(JournalRecordView {
        instance: read_u32(record, 16),
        counter: read_six_byte_unsigned(record, 20),
        checkpoint_txg: read_u64(record, 26),
        transaction: read_u64(record, JOURNAL_TRANSACTION_OFFSET),
        is_commit: record[JOURNAL_COMMIT_MARKER_OFFSET] == 1,
        commit_marker_byte: record[JOURNAL_COMMIT_MARKER_OFFSET],
        ordinal_within_publish: read_u32(record, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET),
        record_flags_byte: record[JOURNAL_RECORD_FLAGS_OFFSET],
        back_chain: read_u32(record, JOURNAL_BACK_CHAIN_OFFSET),
        new_tree_identifier_watermark: read_u64(record, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86),
        new_rollback_floor: read_u64(record, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86 + 8),
        new_root_segment,
        named,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_only_width_stays_distinguishable_from_probed() {
        let declared = DecisionWidth::DeclaredOnly { bytes: 512 };
        let probed = DecisionWidth::Probed { bytes: 512 };
        assert_ne!(declared, probed, "同一个数、两种来源，checker 必须分得开");
        assert_eq!(declared.bytes(), probed.bytes());
    }

    #[test]
    fn bitwise_crc32c_matches_the_published_check_value() {
        assert_eq!(crc32_castagnoli_bitwise(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn garbage_slots_are_rejected_with_the_right_reason() {
        assert_eq!(
            check_system_configuration_slot(&[0u8; 4096]).unwrap_err(),
            Verdict::BadMagic
        );
        assert_eq!(
            check_system_configuration_slot(&[0u8; 10]).unwrap_err(),
            Verdict::TooShort
        );
        assert_eq!(
            check_root_slot(&[0u8; 512], &[0u8; 16]).unwrap_err(),
            Verdict::BadMagic
        );
        assert_eq!(check_unit(&[0u8; 16384]).unwrap_err(), Verdict::BadMagic);
        assert_eq!(choose_system_configuration(&[&[0u8; 4096][..]]), None);
    }
}
