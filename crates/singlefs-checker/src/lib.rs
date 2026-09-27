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
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES,
    JOURNAL_RECORD_BYTES, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES,
    ROOT_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, WIDE_CHECKSUM_BYTES,
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
    /// 单元头或系统配置槽偏移 4 那 2 字节格式版本不是这一版 checker 读得了的那一版
    /// （`UNIT_FORMAT_VERSION_THIS_CHECKER_READS`、`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）：
    /// 版本号定的是后面每个字段按哪张表读，不认得就不按今天的表往下解。
    FormatVersionNotRecognized,
    /// 单元类身份段之后那 29 字节 nonce / MAC / 算法类型预留位有非 0 字节。它们不在头校验和覆盖内（在载荷 CRC 覆盖内），
    /// 由「恒 0、读者遇到非 0 一律判该结构损坏」这条规则守（I-2.4（头校验和覆盖范围）；D18（块里携带什么信息） 已定项 16 / 已定项 17）。
    EncryptionReservedBytesNotZero,
    /// 码 2 节点头里条目宽是 0、条目数不是 0：声明长度 0 过得了「声明长度 = 条目数 × 条目宽」（D18（块里携带什么信息） 已定项 18），
    /// key 宽也是 0 时「条目宽 ≥ key 宽」同样过，切出来的条目却一条都没有，与头里自述的条目数对不上。
    EntryWidthZeroWithEntries,
    /// 码 3 单元头里记录宽是 0、记录数不是 0：与 [`Self::EntryWidthZeroWithEntries`] 同一个坏法。
    RecordWidthZeroWithRecords,
    /// 系统配置偏移 219 的加密类型不是 0（关）：加密不进第一个可运行版本（D9（加密） 已定项 10），
    /// 登记表里的 1、2 是算法（D22（单元原子性怎么合成） 已定项 17），别的码不认得——都不是这一版读得了的卷。
    EncryptionTypeNotOff,
    /// 系统配置自述的固定结构槽距落在格式允许的区间之外：小于 4096（D2（RAID 条带策略） 已定项 19），
    /// 或槽 1（设备内偏移 = 槽距，D22（单元原子性怎么合成） 已定项 16）整槽不在同一槽自述的根环起点之前。
    FixedStructureSlotSpacingOutsideTheFormatRange,
    /// 系统配置自述的 `physical_block_size`（根槽宽，D22（单元原子性怎么合成） 已定项 2）装不下一条根记录（已定项 7），
    /// 或宽过固定结构槽距（根槽 j 在区域起点 + j × 槽距，已定项 16 第 2 句）。读根槽按它开缓冲。
    PhysicalBlockSizeOutsideTheRootSlotBounds,
    /// 系统配置自述的 journal 环长装不下 F 条记录（在飞记录数上限 = 环槽数 ÷ F 是 0，D23（journal 的角色与格式） 已定项 18），
    /// 或环的末端越过同一槽自述的单元区起点（已定项 19 ③：单元区起始槽号随环长走）。
    JournalRingBytesOutsideTheSupportedRange,
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
    /// 系统配置里另记的那一份回退下界 F（D22（单元原子性怎么合成） 已定项 9 字段表最后一行；D16（发布语义） 已定项 1「生效」取
    /// max(根上带的, 系统配置里读得出的)）。C556（checker 与层 0 不读系统配置里的 F） 那一半读它。
    pub rollback_floor: u64,
}

/// 根记录 flags 位 0 的卸载记号（D22（单元原子性怎么合成） 已定项 7）照实读出来：池级 checker 按它给
/// I-7.9（回退下界 F 不高于抬 F 的上限） 分两支。checker 自己写一份，不从 `singlefs-core` 引。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnmountMarkerView {
    /// 位 0 = 1：正常卸载那一串写的根。
    WrittenByTheUnmountSequence,
    /// 位 0 = 0：别的根。
    NotWrittenByTheUnmountSequence,
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
/// 系统配置里回退下界 F 那 8 字节：字段表里 journal tail 8、journal 实例代号 4 之后的那一行（D22（单元原子性怎么合成） 已定项 9）。
/// 按 checker 自己走字段表的读法从 tail 那一行推出来，不借实现那边的偏移。
const SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET: usize = SYSTEM_CONFIGURATION_TAIL_OFFSET + 8 + 4;
/// incompat 位 1 = 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号的那一版）；位 0 退役，见到按不认识的位判
/// （D15（格式冻结政策） 已定项 4，用户 2026-09-26）。位 n 住第 n div 8 个字节的第 n mod 8 低位（D22（单元原子性怎么合成） 已定项 13）。
/// checker 自己写一份，不从 `singlefs-core` 引。
const INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT: u8 = 0x02;
const ROOT_CHECKSUM_OFFSET: usize = 138;
/// 根记录 flags 那 4 字节的偏移：magic 4 + fsid 16。
const ROOT_FLAGS_OFFSET: usize = 20;
/// 根记录 flags 位 0 = 卸载记号（D22（单元原子性怎么合成） 已定项 7）；其余位第一版恒 0、非 0 拒收（已定项 17）。
const ROOT_FLAG_UNMOUNT_MARKER: u32 = 1 << 0;
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
/// 格式版本那 2 字节的偏移：单元头共同前缀与系统配置自举头都是紧跟 magic 4（D18（块里携带什么信息） 已定项 7 的共同前缀表；
/// D22（单元原子性怎么合成） 已定项 9 的字段表）。
pub(crate) const FORMAT_VERSION_OFFSET: usize = 4;
/// 这一版 checker 读得了的单元格式版本：`.claude/kb/layout/01-first-txn.md` 二「单元头 | 格式版本 | 2 | 1」（D18（块里携带什么信息） 已定项 7）。
/// checker 自己写一份，不从 `singlefs-core` 引。单元级（`check_unit`）与池级走读（`walk` 的 `judge_unit_header`）共用这一个数。
pub(crate) const UNIT_FORMAT_VERSION_THIS_CHECKER_READS: u16 = 1;
/// 这一版 checker 读得了的系统配置格式版本：D22（单元原子性怎么合成） 已定项 9 的字段表（`.claude/kb/layout/01-first-txn.md` 一）
/// 「自举头 | 格式版本 | 2 | 1」。与单元头那一份是两个字段、各记一个数。
const SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS: u16 = 1;
/// 系统配置里加密类型那 1 字节的偏移：整槽校验和 32 之后是 MAC 16、nonce 水位 12、KDF 标识 4，再是它（字段表偏移 219）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 155 + 32 + 16 + 12 + 4;
/// 加密类型登记表里的「关」（未加密；D9（加密） 已定项 10，登记位 D22（单元原子性怎么合成） 已定项 17）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF: u8 = 0;

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

/// 判一个系统配置槽（4096 字节）：magic、整槽校验和、格式版本、incompat 位、加密类型。
/// 几何字段的上下界在 [`image::geometry_of`] 判（R、S、槽距、`physical_block_size`、环长）。
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
    if read_u16(slot, FORMAT_VERSION_OFFSET)
        != SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS
    {
        return Err(Verdict::FormatVersionNotRecognized);
    }
    let incompat = &slot
        [SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 32];
    if incompat[0] & !INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT != 0
        || incompat[1..].iter().any(|byte| *byte != 0)
        || incompat[0] & INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT == 0
    {
        return Err(Verdict::UnknownIncompatBit);
    }
    if slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF
    {
        return Err(Verdict::EncryptionTypeNotOff);
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
        rollback_floor: read_u64(slot, SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET),
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
    pub unmount_marker: UnmountMarkerView,
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
    let flags = read_u32(slot, ROOT_FLAGS_OFFSET);
    if flags & !ROOT_FLAG_UNMOUNT_MARKER != 0 {
        return Err(Verdict::NonZeroFlags);
    }
    let unmount_marker = if flags & ROOT_FLAG_UNMOUNT_MARKER == 0 {
        UnmountMarkerView::NotWrittenByTheUnmountSequence
    } else {
        UnmountMarkerView::WrittenByTheUnmountSequence
    };
    Ok(RootView {
        instance: read_u32(slot, 24),
        checkpoint_txg: read_u64(slot, 28),
        tree_identifier_watermark: read_u64(slot, 36 + 86),
        rollback_floor: read_u64(slot, 36 + 86 + 8),
        unmount_marker,
        record_bytes: slot[..record_bytes].to_vec(),
    })
}

/// 码 2 索引节点含 key 区间与预留位的头宽，也是条目区的起点：`86 + 2 × key 宽 + 29`，key 宽取盘上偏移 51 那 1 字节原样
/// （D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 / 已定项 18）。
///
/// checker 自己的一份式子，不调实现、也不调理想模型的那一份（D13（验证路线） 已定项 5：`singlefs-format` 只放标量；
/// 2026-09-27 用户定案「三方各算一份 + 交叉断言」）；三份在 key 宽字段全部 256 个取值上连起来比的是
/// `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`。
#[must_use]
pub fn index_node_header_bytes(key_width_byte_at_offset_51: u8) -> usize {
    let key_range_bytes =
        INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT * u64::from(key_width_byte_at_offset_51);
    usize::try_from(
        INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE
            + key_range_bytes
            + NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    )
    .expect("key 宽至多 255，头宽至多 86 + 2 × 255 + 29 = 625，装得进 usize")
}

/// 判一个单元的头：magic、flags、类标签合法（1 / 2 / 3）、头校验和、载荷 CRC，再判格式版本认得、
/// 类身份段之后那 29 字节 nonce / MAC / 算法类型预留位恒 0（I-2.4（头校验和覆盖范围））；返回类标签。
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
            let header_end = index_node_header_bytes(unit[51])
                - usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
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
    // 两道校验和之后才判这两样：版本号在头校验和里、预留位在载荷 CRC 里，校验和过了，它们就是写这个单元的那一方写下的值，
    // 不认得、非 0 是格式上的拒收，不是一个字节坏了（坏了的那一种上面已经报成校验和对不上）。
    if read_u16(unit, FORMAT_VERSION_OFFSET) != UNIT_FORMAT_VERSION_THIS_CHECKER_READS {
        return Err(Verdict::FormatVersionNotRecognized);
    }
    // 预留位不看系统配置里的加密类型就判恒 0：这一版 checker 不收加密类型非 0 的系统配置（`check_system_configuration_slot`），
    // 它判的池都是加密关着的，那 29 字节只能是 0（D9（加密） 已定项 10）。
    let reserved_bytes = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    if unit[header_end..header_end + reserved_bytes]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(Verdict::EncryptionReservedBytesNotZero);
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

/// 解一个码 2 节点：头与载荷两道校验和先过，再核声明长度 = 条目数 × 条目宽、条目宽 ≥ key 宽、条目宽 0 时条目数也是 0、条目区装得下。
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
    if entry_width == 0 && entry_count != 0 {
        return Err(Verdict::EntryWidthZeroWithEntries);
    }
    let entries_start = offset + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    if entries_start + declared_length > unit.len() {
        return Err(Verdict::TooShort);
    }
    // 条目区恰好是 条目数 × 条目宽 字节（上面判过），按条目宽切正好切出条目数那么多条；
    // 条目宽是 0 只剩条目数也是 0 这一种，那时条目区是空的、一刀都不用切。
    let entries = if entry_count == 0 {
        Vec::new()
    } else {
        unit[entries_start..entries_start + declared_length]
            .chunks(entry_width)
            .map(<[u8]>::to_vec)
            .collect()
    };
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
    if record_width == 0 && record_count != 0 {
        return Err(Verdict::RecordWidthZeroWithRecords);
    }
    let records_start =
        usize::try_from(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES)
            .expect("136");
    if records_start + declared_length > unit.len() {
        return Err(Verdict::TooShort);
    }
    // 与码 2 条目区同一个切法：记录区恰好 记录数 × 记录宽 字节，记录宽 0 只剩记录数也是 0。
    let records = if record_count == 0 {
        Vec::new()
    } else {
        unit[records_start..records_start + declared_length]
            .chunks(record_width)
            .map(<[u8]>::to_vec)
            .collect()
    };
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
