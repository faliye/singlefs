use super::*;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::pointer::{NodePointer, PointerHead};
use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
use singlefs_core::region_scan::{
    MappedUnitClass, MovableUnit, RegionScanInconsistency, RegionScanRefusal, ScannedAllocation,
    UnitHeaderUnreadable,
};
use singlefs_core::unit::{parse_data_unit, parse_index_node, parse_packed_unit, UnitError};
use std::arch::x86_64::{_mm_crc32_u64, _mm_crc32_u8};
use std::collections::HashMap;

/// CRC-32C 的反射多项式与初值（与 crates `checksum.rs` 同一个算法；这一份只给写法乙丁计时，值由停机 K-S1 每次重放与 crates 对拍）。
pub(super) const CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F6_3B78;
pub(super) const CRC_INITIAL_REGISTER: u32 = 0xFFFF_FFFF;
/// 一条 `crc32q` 吃几字节。
pub(super) const BYTES_PER_HARDWARE_CRC_STEP: usize = 8;
const BITS_PER_BYTE: usize = 8;
const REGISTER_BITS: usize = 32;

/// 硬件指令那一圈：从 `register` 起，每 8 字节一条 `crc32q`，尾巴逐字节 `crc32b`；交回原样的寄存器（不取反）。
#[target_feature(enable = "sse4.2")]
fn hardware_register_over(register: u32, bytes: &[u8]) -> u32 {
    let mut wide = u64::from(register);
    let (words, tail) = bytes.as_chunks::<BYTES_PER_HARDWARE_CRC_STEP>();
    for word in words {
        wide = _mm_crc32_u64(wide, u64::from_le_bytes(*word));
    }
    let mut narrow = u32::try_from(wide).expect("crc32q 只写低 32 位");
    for byte in tail {
        narrow = _mm_crc32_u8(narrow, *byte);
    }
    narrow
}

/// 单路（登记第一节写法乙「单路」）：初值 `0xFFFFFFFF`、出来取反。
#[target_feature(enable = "sse4.2")]
fn hardware_single_crc(bytes: &[u8]) -> u32 {
    hardware_register_over(CRC_INITIAL_REGISTER, bytes) ^ CRC_INITIAL_REGISTER
}

/// 三路切段（登记第一节写法乙「三路交错」）：段长取 ⌊长度 ÷ 3 ÷ 8⌋ × 8、前两段同值、余下的给第三段。
pub(super) fn three_way_lanes(length: usize) -> [usize; 3] {
    let lane = length / 3 / BYTES_PER_HARDWARE_CRC_STEP * BYTES_PER_HARDWARE_CRC_STEP;
    [lane, lane, length - 2 * lane]
}

/// 往 CRC 寄存器后面接 n 个零字节的 32×32 移位矩阵（GF(2) 上；第 j 列是只有第 j 位为 1 的寄存器接完零字节之后的样子）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ZeroByteShift {
    columns: [u32; REGISTER_BITS],
}

impl ZeroByteShift {
    fn identity() -> ZeroByteShift {
        let mut columns = [0_u32; REGISTER_BITS];
        for (bit, column) in columns.iter_mut().enumerate() {
            *column = 1_u32 << bit;
        }
        ZeroByteShift { columns }
    }

    /// 接一个零位：寄存器右移一位，移出的是 1 就异或多项式（反射 CRC 的一步）。
    fn one_zero_bit() -> ZeroByteShift {
        let mut columns = [0_u32; REGISTER_BITS];
        columns[0] = CASTAGNOLI_REFLECTED_POLYNOMIAL;
        for (bit, column) in columns.iter_mut().enumerate().skip(1) {
            *column = 1_u32 << (bit - 1);
        }
        ZeroByteShift { columns }
    }

    /// 矩阵乘向量：寄存器里为 1 的那几位对应的列异或起来。
    pub(super) fn applied_to(&self, register: u32) -> u32 {
        self.columns
            .iter()
            .enumerate()
            .filter(|(bit, _column)| register & (1_u32 << bit) != 0)
            .fold(0_u32, |folded, (_bit, column)| folded ^ column)
    }

    /// 先接 `self`、再接 `later`。
    fn followed_by(&self, later: &ZeroByteShift) -> ZeroByteShift {
        let mut columns = [0_u32; REGISTER_BITS];
        for (column, own) in columns.iter_mut().zip(self.columns) {
            *column = later.applied_to(own);
        }
        ZeroByteShift { columns }
    }

    /// 接 `bytes` 个零字节（一位的矩阵平方着乘）。
    pub(super) fn over_zero_bytes(bytes: usize) -> ZeroByteShift {
        let mut result = ZeroByteShift::identity();
        let mut power = ZeroByteShift::one_zero_bit();
        let mut remaining_bits = bytes * BITS_PER_BYTE;
        // 迭代上界：remaining_bits 每圈右移一位，至多 usize 的位数圈
        while remaining_bits > 0 {
            if remaining_bits & 1 == 1 {
                result = result.followed_by(&power);
            }
            power = power.followed_by(&power);
            remaining_bits >>= 1;
        }
        result
    }
}

/// 三路合并要的移位矩阵：开跑时按要量的单元长度把各段长的矩阵预算好，量的时候只查表、不现算（登记第一节写法乙）。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ThreeWayShifts {
    by_lane_bytes: BTreeMap<usize, ZeroByteShift>,
}

impl ThreeWayShifts {
    pub(super) fn for_unit_lengths(unit_lengths: &[usize]) -> ThreeWayShifts {
        let by_lane_bytes = unit_lengths
            .iter()
            .flat_map(|length| three_way_lanes(*length))
            .map(|lane_bytes| (lane_bytes, ZeroByteShift::over_zero_bytes(lane_bytes)))
            .collect();
        ThreeWayShifts { by_lane_bytes }
    }

    /// 接这么多零字节的矩阵；没预算的长度交 `None`（那一次三路读数作废，不现算）。
    pub(super) fn over(&self, lane_bytes: usize) -> Option<&ZeroByteShift> {
        self.by_lane_bytes.get(&lane_bytes)
    }
}

fn word_at(lane: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        lane[offset..offset + BYTES_PER_HARDWARE_CRC_STEP]
            .try_into()
            .expect("三段共有的字数之内，每段都还有 8 字节"),
    )
}

fn low_half(wide: u64) -> u32 {
    u32::try_from(wide).expect("crc32q 只写低 32 位")
}

/// 三路交错：三段在同一个循环里各走一条 `crc32q`，各自走完余下的，再按「第一段移过第二段长度的零字节、异或第二段，再移过第三段长度、
/// 异或第三段」合并。第一段初值 `0xFFFFFFFF`，后两段初值 0，出来取反。移位矩阵没预算的交 `None`。
#[target_feature(enable = "sse4.2")]
fn hardware_three_way_crc(bytes: &[u8], shifts: &ThreeWayShifts) -> Option<u32> {
    let [first_length, second_length, third_length] = three_way_lanes(bytes.len());
    let (first, rest) = bytes.split_at(first_length);
    let (second, third) = rest.split_at(second_length);
    let common_words =
        first_length.min(second_length).min(third_length) / BYTES_PER_HARDWARE_CRC_STEP;
    let mut registers = [u64::from(CRC_INITIAL_REGISTER), 0_u64, 0_u64];
    for word_index in 0..common_words {
        let offset = word_index * BYTES_PER_HARDWARE_CRC_STEP;
        registers[0] = _mm_crc32_u64(registers[0], word_at(first, offset));
        registers[1] = _mm_crc32_u64(registers[1], word_at(second, offset));
        registers[2] = _mm_crc32_u64(registers[2], word_at(third, offset));
    }
    let done = common_words * BYTES_PER_HARDWARE_CRC_STEP;
    let first_register = hardware_register_over(low_half(registers[0]), &first[done..]);
    let second_register = hardware_register_over(low_half(registers[1]), &second[done..]);
    let third_register = hardware_register_over(low_half(registers[2]), &third[done..]);
    let combined = shifts.over(second_length)?.applied_to(first_register) ^ second_register;
    let combined = shifts.over(third_length)?.applied_to(combined) ^ third_register;
    Some(combined ^ CRC_INITIAL_REGISTER)
}

/// 硬件 CRC 的两种写法：只能在核过 `sse4.2` 之后造出来（[`HardwareCrc::when_the_instructions_are_present`]），造出来就信它。
pub(super) struct HardwareCrc {
    shifts: ThreeWayShifts,
}

/// 写法乙的两端（登记第一节：单路、三路交错）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HardwareCrcLanes {
    Single,
    ThreeWay,
}

impl HardwareCrcLanes {
    pub(super) const ALL: [HardwareCrcLanes; 2] =
        [HardwareCrcLanes::Single, HardwareCrcLanes::ThreeWay];

    pub(super) const fn label(self) -> &'static str {
        match self {
            HardwareCrcLanes::Single => "hardware_single",
            HardwareCrcLanes::ThreeWay => "hardware_three_way",
        }
    }
}

impl HardwareCrc {
    /// 这台机器有 `sse4.2` 才交回（写法与拆分段开跑核一次，不在退 78）；移位矩阵按 `unit_lengths` 预算。
    pub(super) fn when_the_instructions_are_present(unit_lengths: &[usize]) -> Option<HardwareCrc> {
        std::arch::is_x86_feature_detected!("sse4.2").then(|| HardwareCrc {
            shifts: ThreeWayShifts::for_unit_lengths(unit_lengths),
        })
    }

    /// 一份字节的 CRC-32C；三路那一端的段长没预算交 `None`。
    #[allow(
        unsafe_code,
        reason = "写法乙要量装置自己的单路与三路交错两种 crc32q 写法，两个函数带 target_feature 属性（sse4.2），从没开这个特性的函数里调只能写 unsafe；std 没有查过特性再调的安全封装，crates 的 crc32_castagnoli 只有一种写法、替不了这两臂"
    )]
    pub(super) fn crc(&self, lanes: HardwareCrcLanes, bytes: &[u8]) -> Option<u32> {
        match lanes {
            // SAFETY: HardwareCrc 只在 is_x86_feature_detected!("sse4.2") 为真时造得出来（when_the_instructions_are_present），这台机器有这条指令。
            HardwareCrcLanes::Single => Some(unsafe { hardware_single_crc(bytes) }),
            // SAFETY: HardwareCrc 只在 is_x86_feature_detected!("sse4.2") 为真时造得出来（when_the_instructions_are_present），
            // 这台机器有 hardware_three_way_crc 要的这条指令。
            HardwareCrcLanes::ThreeWay => unsafe { hardware_three_way_crc(bytes, &self.shifts) },
        }
    }

    /// 甲段换成硬件写法：对每个单元算 CRC，交回（算过的字节、各段 CRC 的异或）；有一个单元三路段长没预算交 `None`。
    pub(super) fn stage_over_the_units(
        &self,
        lanes: HardwareCrcLanes,
        units: &[Vec<u8>],
    ) -> Option<(u64, u32)> {
        let mut covered_bytes = 0_u64;
        let mut folded = 0_u32;
        for unit in units {
            covered_bytes += length_in_bytes(unit);
            folded ^= self.crc(lanes, std::hint::black_box(unit))?;
        }
        Some((covered_bytes, folded))
    }
}

/// 停机 K-S1 的核法：每个单元两种硬件写法与 crates `crc32_castagnoli` 逐个比，交回对不上的个数（三路段长没预算也算对不上）。
pub(super) fn hardware_crc_mismatches(hardware: &HardwareCrc, units: &[Vec<u8>]) -> usize {
    units
        .iter()
        .map(|unit| {
            let expected = crc32_castagnoli(unit);
            HardwareCrcLanes::ALL
                .iter()
                .filter(|lanes| hardware.crc(**lanes, unit) != Some(expected))
                .count()
        })
        .sum()
}

/// PW3 的锚点（登记第七节乙类）：「第 i 字节 = i mod 251」的两种长度与它的 CRC-32C。
pub(super) const MODULUS_ANCHORS: [(usize, u32); 2] =
    [(32_768, 0x2AD6_0827), (16_384, 0xEAFC_A51D)];
pub(super) const RANDOM_LENGTH_CHECKS: usize = 1000;
pub(super) const RANDOM_LENGTH_AT_MOST: usize = 64;
pub(super) const RANDOM_LENGTH_SEED: u64 = 0xE185_0004_0000_0001;

/// 「第 i 字节 = i mod 251」那份字节。
pub(super) fn modulus_bytes(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from(index % CALIBRATION_BYTE_MODULUS).expect("251 以内装得进 u8"))
        .collect()
}

/// 写死种子的 0 到 64 字节随机串（splitmix64），PW3 第三项用。
pub(super) fn random_length_strings() -> Vec<Vec<u8>> {
    let mut state = RANDOM_LENGTH_SEED;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    };
    (0..RANDOM_LENGTH_CHECKS)
        .map(|_string| {
            let length =
                usize::try_from(next() % u64::try_from(RANDOM_LENGTH_AT_MOST + 1).expect("65"))
                    .expect("64 以内");
            (0..length).map(|_byte| next().to_le_bytes()[0]).collect()
        })
        .collect()
}

/// 写法与拆分段开跑时要预算的单元长度：数据单元、节点、i mod 251 的两种、0 到 64 字节。
pub(super) fn unit_lengths_of_the_segment() -> Vec<usize> {
    let mut lengths = vec![
        usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768"),
        usize::try_from(LOCAL_NODE_BYTES).expect("16384"),
    ];
    lengths.extend(MODULUS_ANCHORS.map(|(length, _crc)| length));
    lengths.extend(0..=RANDOM_LENGTH_AT_MOST);
    lengths.push(CHECK_STRING.len());
    lengths
}

/// PW3（登记第五节阳性对照表）：校验串、i mod 251 两种长度、随机长度三项，两种硬件写法各自与锚点、与 crates 逐个相同。
/// 交回（项、写法、对不上的个数）；全对交空表。
pub(super) fn hardware_crc_check_failures(
    hardware: &HardwareCrc,
) -> Vec<(String, &'static str, usize)> {
    let mut failures = Vec::new();
    for lanes in HardwareCrcLanes::ALL {
        let mut check = |item: String, bytes: &[u8], expected: u32| {
            let differs = usize::from(hardware.crc(lanes, bytes) != Some(expected))
                + usize::from(crc32_castagnoli(bytes) != expected);
            if differs > 0 {
                failures.push((item, lanes.label(), differs));
            }
        };
        check(
            "check_string".to_string(),
            CHECK_STRING,
            CHECK_STRING_CRC32_CASTAGNOLI,
        );
        for (length, anchor) in MODULUS_ANCHORS {
            check(
                format!("modulus_bytes_{length}"),
                &modulus_bytes(length),
                anchor,
            );
        }
        let random = random_length_strings();
        let random_mismatches = random
            .iter()
            .filter(|bytes| hardware.crc(lanes, bytes) != Some(crc32_castagnoli(bytes)))
            .count();
        if random_mismatches > 0 {
            failures.push((
                "random_lengths".to_string(),
                lanes.label(),
                random_mismatches,
            ));
        }
    }
    failures
}

// ───── 写法丙：抄的一份区域扫描，两处线性找换成按身份查（逐句照 crates `region_scan.rs` 的 `scan_region`、`classify_allocation`、`read_unit_header`） ─────

// 抄的扫描用到的三个类标签是顶层的本地常量（`LOCAL_UNIT_CLASS_DATA` 那三个，开跑与 crates 回比，[`constants_disagreeing_with_the_crates`]）。

fn local_tag_of(unit_class: MappedUnitClass) -> u8 {
    match unit_class {
        MappedUnitClass::Data => LOCAL_UNIT_CLASS_DATA,
        MappedUnitClass::IndexNode => LOCAL_UNIT_CLASS_INDEX_NODE,
        MappedUnitClass::Packed => LOCAL_UNIT_CLASS_PACKED,
    }
}

/// 一个位置项的键：盘与槽一起。
fn location_key_of(
    device_of_the_location: DeviceIdentity,
    slot_of_the_location: SlotNumber,
) -> (DeviceIdentity, SlotNumber) {
    (device_of_the_location, slot_of_the_location)
}

/// 这一版映射条目的两张索引（登记第一节写法丙）：（盘、槽）→ 条目号、key → 条目号，都用 std 的 `HashMap`，条目号按映射条目的次序存。
pub(super) struct IdentityIndexOfTheMapping {
    entries_by_location: HashMap<(DeviceIdentity, SlotNumber), Vec<usize>>,
    entries_by_key: HashMap<Vec<u8>, Vec<usize>>,
}

impl IdentityIndexOfTheMapping {
    /// 两条位置项各进一次：先全部条目的第一条、再全部条目的第二条，所以一个槽的条目号在表里不一定按映射条目的次序，查的时候再排。
    pub(super) fn of_the_entries(entries: &[CentralMappingEntry]) -> IdentityIndexOfTheMapping {
        let mut entries_by_location: HashMap<(DeviceIdentity, SlotNumber), Vec<usize>> =
            HashMap::new();
        for location_position in 0..2 {
            for (entry_index, entry) in entries.iter().enumerate() {
                let location = entry.locations[location_position];
                let indices = entries_by_location
                    .entry(location_key_of(location.device, location.slot))
                    .or_default();
                if !indices.contains(&entry_index) {
                    indices.push(entry_index);
                }
            }
        }
        let mut entries_by_key: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        for (entry_index, entry) in entries.iter().enumerate() {
            entries_by_key
                .entry(entry.key.clone())
                .or_default()
                .push(entry_index);
        }
        IdentityIndexOfTheMapping {
            entries_by_location,
            entries_by_key,
        }
    }

    /// 指着这一槽的条目号，按映射条目的次序（与线性找交回的次序相同）。
    fn entries_pointing_at(&self, device: DeviceIdentity, slot: SlotNumber) -> Vec<usize> {
        let mut indices = self
            .entries_by_location
            .get(&location_key_of(device, slot))
            .cloned()
            .unwrap_or_default();
        indices.sort_unstable();
        indices
    }

    fn is_the_key_mapped(&self, key: &[u8]) -> bool {
        self.entries_by_key.contains_key(key)
    }

    /// `identity_index_entries`：key 那张表的键数（为 0 照「读不到」作废那一次重放）。
    pub(super) fn keys(&self) -> usize {
        self.entries_by_key.len()
    }

    /// `identity_index_location_keys`：（盘、槽）那张表的键数。
    pub(super) fn location_keys(&self) -> usize {
        self.entries_by_location.len()
    }
}

/// 单元头里反查出来的东西（crates `UnitHeaderIdentity` 的同形，那一份是私有的）。
struct CopiedUnitHeaderIdentity {
    unit_class: MappedUnitClass,
    mapping_key: Vec<u8>,
    birth_txg: CheckpointTxg,
}

/// 抄的 `scan_region`：R 那块盘上起点在 R 里、未释放的每条记录分一类，按槽升序交回；跨边界、两条同 key 的拒法照 crates。
pub(super) fn scan_region_with_the_identity_index<ReadUnitBytes>(
    region: CompactionRegion,
    allocation_records: &[AllocationRecord],
    mapping_entries: &[CentralMappingEntry],
    index: &IdentityIndexOfTheMapping,
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    mut read_unit_bytes: ReadUnitBytes,
) -> Result<RegionScan, RegionScanRefusal>
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let device = region.device();
    let mut records_in_region: Vec<&AllocationRecord> = Vec::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == device && !record.is_released)
    {
        let record_end = record.slot.0 + u64::from(record.span_slots);
        let starts_inside = region.contains_slot(record.slot);
        let reaches_in_from_below =
            record.slot.0 < region.start_slot().0 && record_end > region.start_slot().0;
        let reaches_out_above = starts_inside && record_end > region.end_slot().0;
        if reaches_in_from_below || reaches_out_above {
            return Err(
                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                    device,
                    slot: record.slot,
                    span_slots: record.span_slots,
                },
            );
        }
        if starts_inside {
            records_in_region.push(record);
        }
    }
    records_in_region.sort_by_key(|record| record.slot);
    let allocations = records_in_region
        .into_iter()
        .map(|record| ScannedAllocation {
            placement: Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            },
            class: classify_allocation_with_the_identity_index(
                record,
                mapping_entries,
                index,
                pinned_slots,
                quarantined_slots,
                &mut read_unit_bytes,
            ),
        })
        .collect();
    let scan = RegionScan {
        region,
        allocations,
    };
    if let Some(slots) = scan.first_two_movable_units_sharing_a_mapping_key() {
        return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });
    }
    Ok(scan)
}

/// 抄的 `classify_allocation`：次序照 crates（钉住 → 隔离 → 读头 → 类标签 → key 与位置），只把两处线性找换成查索引。
fn classify_allocation_with_the_identity_index<ReadUnitBytes>(
    record: &AllocationRecord,
    mapping_entries: &[CentralMappingEntry],
    index: &IdentityIndexOfTheMapping,
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    read_unit_bytes: &mut ReadUnitBytes,
) -> AllocationInRegion
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let covered_slots = record.slot..SlotNumber(record.slot.0 + u64::from(record.span_slots));
    if pinned_slots.range(covered_slots.clone()).next().is_some() {
        return AllocationInRegion::Pinned;
    }
    if quarantined_slots.range(covered_slots).next().is_some() {
        return AllocationInRegion::Quarantined;
    }
    let entries_pointing_here: Vec<&CentralMappingEntry> = index
        .entries_pointing_at(record.device, record.slot)
        .into_iter()
        .map(|entry_index| &mapping_entries[entry_index])
        .collect();
    let header = match copied_read_unit_header(record, read_unit_bytes) {
        Ok(header) => header,
        Err(unreadable) => {
            return AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                unreadable,
            ))
        }
    };
    if let Some(entry) = entries_pointing_here
        .iter()
        .find(|entry| entry.key.first() != Some(&local_tag_of(header.unit_class)))
    {
        return AllocationInRegion::Inconsistent(
            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                header_unit_class: header.unit_class,
                mapping_key_class_tag: entry.key.first().copied().unwrap_or_default(),
            },
        );
    }
    let header_key_is_mapped = index.is_the_key_mapped(&header.mapping_key);
    if entries_pointing_here.is_empty() && !header_key_is_mapped {
        return AllocationInRegion::Dead;
    }
    let every_entry_pointing_here_carries_the_header_key = entries_pointing_here
        .iter()
        .all(|entry| entry.key == header.mapping_key);
    if !entries_pointing_here.is_empty() && every_entry_pointing_here_carries_the_header_key {
        return AllocationInRegion::Movable(MovableUnit {
            mapping_key: header.mapping_key,
            unit_class: header.unit_class,
            birth_txg: header.birth_txg,
        });
    }
    AllocationInRegion::Inconsistent(
        RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
    )
}

/// 抄的 `read_unit_header`：跨度 1 当码 2 读 16 KiB，跨度 2 读 32 KiB 先当码 1、类标签是 3 再当码 3；解头与 key 调 crates 公开的那几个函数。
fn copied_read_unit_header<ReadUnitBytes>(
    record: &AllocationRecord,
    read_unit_bytes: &mut ReadUnitBytes,
) -> Result<CopiedUnitHeaderIdentity, UnitHeaderUnreadable>
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let node_pointer_of = |birth_tree, birth_txg, instance, birth_sequence| NodePointer {
        head: PointerHead {
            birth_tree,
            birth_txg,
        },
        locations: NodePointer::empty_root().locations,
        instance,
        birth_sequence,
    };
    match record.span_slots {
        1 => {
            let bytes = read_unit_bytes(
                record.device,
                record.slot,
                usize::try_from(LOCAL_NODE_BYTES).expect("16384"),
            )
            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
            let header =
                parse_index_node(&bytes).map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
            Ok(CopiedUnitHeaderIdentity {
                unit_class: MappedUnitClass::IndexNode,
                mapping_key: mapping_key_for_node(
                    LOCAL_UNIT_CLASS_INDEX_NODE,
                    node_pointer_of(
                        header.tree,
                        header.birth_txg,
                        header.instance,
                        header.birth_sequence,
                    ),
                ),
                birth_txg: header.birth_txg,
            })
        }
        2 => {
            let bytes = read_unit_bytes(
                record.device,
                record.slot,
                usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768"),
            )
            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
            match parse_data_unit(&bytes) {
                Ok(header) => Ok(CopiedUnitHeaderIdentity {
                    unit_class: MappedUnitClass::Data,
                    mapping_key: mapping_key_for_data(
                        PointerHead {
                            birth_tree: header.identity.tree,
                            birth_txg: header.birth_txg,
                        },
                        header.write_order,
                    ),
                    birth_txg: header.birth_txg,
                }),
                Err(UnitError::WrongUnitClass {
                    found: LOCAL_UNIT_CLASS_PACKED,
                    ..
                }) => {
                    let header = parse_packed_unit(&bytes)
                        .map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
                    Ok(CopiedUnitHeaderIdentity {
                        unit_class: MappedUnitClass::Packed,
                        mapping_key: mapping_key_for_node(
                            LOCAL_UNIT_CLASS_PACKED,
                            node_pointer_of(
                                header.identity.birth_tree,
                                header.birth_txg,
                                header.write_order.instance,
                                header.birth_sequence,
                            ),
                        ),
                        birth_txg: header.birth_txg,
                    })
                }
                Err(refusal) => Err(UnitHeaderUnreadable::RefusedByTheParser(refusal)),
            }
        }
        span_slots => Err(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots }),
    }
}

// ───── 九种写法与它们在三样阶段上的取值（登记第一节「产物里的写法名」、B1、B2） ─────

/// CRC 那两项用哪一种写法。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrcWriting {
    Today,
    Hardware(HardwareCrcLanes),
}

/// 区域扫描用哪一份、建索引算不算。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanWriting {
    Today,
    IdentityIndexWithTheBuild,
    IdentityIndexWithoutTheBuild,
}

/// 九种写法（甲一种、乙两端、丙两端、丁四种组合）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CpuWriting {
    Today,
    HardwareCrcSingle,
    HardwareCrcThreeWay,
    IdentityIndexWithBuild,
    IdentityIndexWithoutBuild,
    BothSingleWithBuild,
    BothSingleWithoutBuild,
    BothThreeWayWithBuild,
    BothThreeWayWithoutBuild,
}

impl CpuWriting {
    pub(super) const ALL: [CpuWriting; 9] = [
        CpuWriting::Today,
        CpuWriting::HardwareCrcSingle,
        CpuWriting::HardwareCrcThreeWay,
        CpuWriting::IdentityIndexWithBuild,
        CpuWriting::IdentityIndexWithoutBuild,
        CpuWriting::BothSingleWithBuild,
        CpuWriting::BothSingleWithoutBuild,
        CpuWriting::BothThreeWayWithBuild,
        CpuWriting::BothThreeWayWithoutBuild,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            CpuWriting::Today => "today",
            CpuWriting::HardwareCrcSingle => "hardware_crc_single",
            CpuWriting::HardwareCrcThreeWay => "hardware_crc_three_way",
            CpuWriting::IdentityIndexWithBuild => "identity_index_with_build",
            CpuWriting::IdentityIndexWithoutBuild => "identity_index_without_build",
            CpuWriting::BothSingleWithBuild => "both_single_with_build",
            CpuWriting::BothSingleWithoutBuild => "both_single_without_build",
            CpuWriting::BothThreeWayWithBuild => "both_three_way_with_build",
            CpuWriting::BothThreeWayWithoutBuild => "both_three_way_without_build",
        }
    }

    const fn parts(self) -> (CrcWriting, ScanWriting) {
        let single = CrcWriting::Hardware(HardwareCrcLanes::Single);
        let three_way = CrcWriting::Hardware(HardwareCrcLanes::ThreeWay);
        match self {
            CpuWriting::Today => (CrcWriting::Today, ScanWriting::Today),
            CpuWriting::HardwareCrcSingle => (single, ScanWriting::Today),
            CpuWriting::HardwareCrcThreeWay => (three_way, ScanWriting::Today),
            CpuWriting::IdentityIndexWithBuild => {
                (CrcWriting::Today, ScanWriting::IdentityIndexWithTheBuild)
            }
            CpuWriting::IdentityIndexWithoutBuild => {
                (CrcWriting::Today, ScanWriting::IdentityIndexWithoutTheBuild)
            }
            CpuWriting::BothSingleWithBuild => (single, ScanWriting::IdentityIndexWithTheBuild),
            CpuWriting::BothSingleWithoutBuild => {
                (single, ScanWriting::IdentityIndexWithoutTheBuild)
            }
            CpuWriting::BothThreeWayWithBuild => {
                (three_way, ScanWriting::IdentityIndexWithTheBuild)
            }
            CpuWriting::BothThreeWayWithoutBuild => {
                (three_way, ScanWriting::IdentityIndexWithoutTheBuild)
            }
        }
    }
}

/// 一次调用之后量的九样计时件（纳秒；读不到的是 `None`），`name=cpu_writing_stage` 那一行的数。今天写法那三样取同一次重放
/// （[`replay_the_stages_of_a_call`]）的读数。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CpuWritingPieces {
    pub(super) crc_today: Option<f64>,
    pub(super) crc_hardware_single: Option<f64>,
    pub(super) crc_hardware_three_way: Option<f64>,
    pub(super) release_check_crc_today: Option<f64>,
    pub(super) release_check_crc_hardware_single: Option<f64>,
    pub(super) release_check_crc_hardware_three_way: Option<f64>,
    pub(super) scan_today: Option<f64>,
    pub(super) scan_identity_index_lookup: Option<f64>,
    pub(super) identity_index_build: Option<f64>,
}

/// 三样阶段（登记第一节「候选 A–D 的门槛 T_X」：写法 X 只换这三样）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum WritingStage {
    Crc,
    ScanIncrement,
    ReleaseCheckCrc,
}

impl WritingStage {
    pub(super) const ALL: [WritingStage; 3] = [
        WritingStage::Crc,
        WritingStage::ScanIncrement,
        WritingStage::ReleaseCheckCrc,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            WritingStage::Crc => "crc",
            WritingStage::ScanIncrement => "scan_increment",
            WritingStage::ReleaseCheckCrc => "release_check_crc",
        }
    }
}

impl CpuWritingPieces {
    fn crc_of(&self, writing: CrcWriting) -> Option<f64> {
        match writing {
            CrcWriting::Today => self.crc_today,
            CrcWriting::Hardware(HardwareCrcLanes::Single) => self.crc_hardware_single,
            CrcWriting::Hardware(HardwareCrcLanes::ThreeWay) => self.crc_hardware_three_way,
        }
    }

    fn release_check_crc_of(&self, writing: CrcWriting) -> Option<f64> {
        match writing {
            CrcWriting::Today => self.release_check_crc_today,
            CrcWriting::Hardware(HardwareCrcLanes::Single) => {
                self.release_check_crc_hardware_single
            }
            CrcWriting::Hardware(HardwareCrcLanes::ThreeWay) => {
                self.release_check_crc_hardware_three_way
            }
        }
    }

    /// 写法 X 下一样阶段的值（登记第一节写法乙丙丁）：`crc`、`release_check_crc` 取那一种 CRC 写法；`scan_increment` = 那一份扫描
    /// （丙丁按端加不加建索引）− 今天写法的 `crc`（乙段增量里那一份 CRC 是 `parse_data_unit` 调的 crates CRC，乙不改它）。
    pub(super) fn stage_of_the_writing(
        &self,
        writing: CpuWriting,
        stage: WritingStage,
    ) -> Option<f64> {
        let (crc_writing, scan_writing) = writing.parts();
        match stage {
            WritingStage::Crc => self.crc_of(crc_writing),
            WritingStage::ReleaseCheckCrc => self.release_check_crc_of(crc_writing),
            WritingStage::ScanIncrement => {
                let scan = match scan_writing {
                    ScanWriting::Today => self.scan_today?,
                    ScanWriting::IdentityIndexWithTheBuild => {
                        self.scan_identity_index_lookup? + self.identity_index_build?
                    }
                    ScanWriting::IdentityIndexWithoutTheBuild => self.scan_identity_index_lookup?,
                };
                Some(scan - self.crc_today?)
            }
        }
    }
}

/// 一条意图里进候选式子的那几次调用的一样阶段之和（登记第一节「门槛来源」）：`crc` 与 `scan_increment` 取签发加各批（A、B、C 的式子），
/// `release_check_crc` 只取各批（D 的式子）；「签发」照门槛账的取法是 `kind=issue` 那一次，其余都算批。有一次读不到交 `None`。
pub(super) fn stage_sum_of_the_intent(
    calls: &[(CallKind, CpuWritingPieces)],
    writing: CpuWriting,
    stage: WritingStage,
) -> Option<f64> {
    calls
        .iter()
        .filter(|(kind, _pieces)| match stage {
            WritingStage::Crc | WritingStage::ScanIncrement => true,
            WritingStage::ReleaseCheckCrc => *kind != CallKind::Issue,
        })
        .map(|(_kind, pieces)| pieces.stage_of_the_writing(writing, stage))
        .sum()
}

/// 一条意图上写法 X 的系数：X 那几次之和 ÷ 今天写法同几次之和；分母读不到或是 0 交 `None`。写法甲恒为 1。
pub(super) fn writing_factor_of_the_intent(
    calls: &[(CallKind, CpuWritingPieces)],
    writing: CpuWriting,
    stage: WritingStage,
) -> Option<f64> {
    let today = stage_sum_of_the_intent(calls, CpuWriting::Today, stage)?;
    let this_writing = stage_sum_of_the_intent(calls, writing, stage)?;
    (today != 0.0).then(|| this_writing / today)
}

/// 系数 φ（B2）：各有效轮的系数取中位；读不到的轮不收，一轮都没有交 `None`。
pub(super) fn writing_factor_of_the_rounds(factor_per_round: &[Option<f64>]) -> Option<f64> {
    let read: Vec<f64> = factor_per_round.iter().flatten().copied().collect();
    median_of(&read)
}

// ───── 一次调用之后、计时窗口外量八样新的计时件（B1）与对拍（K-S1、K-S2） ─────

/// 一次调用的新读数：九样计时件、字节、索引的大小、对拍结果。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CpuWritingsOfTheCall {
    pub(super) pieces: CpuWritingPieces,
    pub(super) choose_cpu_nanoseconds: Option<f64>,
    pub(super) crc_bytes: u64,
    pub(super) release_check_crc_bytes: u64,
    pub(super) hardware_crc_bytes: [u64; 2],
    pub(super) hardware_release_check_crc_bytes: [u64; 2],
    pub(super) identity_index_entries: usize,
    pub(super) identity_index_location_keys: usize,
    /// 两种硬件写法与 crates 对不上的单元个数（K-S1，扫描读到的与释放前读盘核的合起来）。
    pub(super) hardware_crc_mismatches: usize,
    /// 抄的扫描交回的整个 `Result` 与 crates `scan_region` 的相不相等（K-S2；签发与批之外的调用没有 R 时是 `None`）。
    pub(super) identity_index_scan_matches: Option<bool>,
}

impl CpuWritingsOfTheCall {
    /// 停机条款：K-S1、K-S2 与第七节乙类「一次重放里写法乙丙丁的字节」；对上交 `None`，对不上交（条款编号、一句）。
    pub(super) fn stop_finding(&self, call_index: usize) -> Option<(&'static str, String)> {
        if self.hardware_crc_mismatches > 0 {
            return Some((
                "K-S1",
                format!(
                    "clause_k_s1_hardware_crc_differs_from_the_crates_at_call_{call_index}_units_{}",
                    self.hardware_crc_mismatches
                ),
            ));
        }
        if self.identity_index_scan_matches == Some(false) {
            return Some((
                "K-S2",
                format!(
                    "clause_k_s2_identity_index_scan_differs_from_the_crates_at_call_{call_index}"
                ),
            ));
        }
        (!self.do_the_hardware_crc_bytes_hold()).then(|| {
            (
                "K-S1",
                format!("clause_k_s1_hardware_crc_bytes_differ_from_the_scan_and_release_check_reads_at_call_{call_index}"),
            )
        })
    }

    /// 第七节乙类：两种硬件写法算过的字节与今天写法的 `crc_bytes`、`release_check_crc_bytes` 相同。
    pub(super) fn do_the_hardware_crc_bytes_hold(&self) -> bool {
        self.hardware_crc_bytes
            .iter()
            .all(|bytes| *bytes == self.crc_bytes)
            && self
                .hardware_release_check_crc_bytes
                .iter()
                .all(|bytes| *bytes == self.release_check_crc_bytes)
    }

    /// 读不到（登记第十一节）：索引里一条都没有，那一次重放作废。
    pub(super) fn is_unreadable(&self) -> bool {
        self.identity_index_entries == 0
    }
}

/// 两种硬件写法在甲段两项上的（CPU 中位、算过的字节），次序同 [`HardwareCrcLanes::ALL`]：`crc` 算扫描读到的单元，`release_check_crc` 算释放前读盘核
/// 读的那几份副本（字节锚点照核，第七节乙类）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HardwareCrcPieces {
    pub(super) crc: [(Option<f64>, u64); 2],
    pub(super) release_check_crc: [(Option<f64>, u64); 2],
}

pub(super) fn hardware_crc_pieces(
    hardware: &HardwareCrc,
    units: &[Vec<u8>],
    copies: &[Vec<u8>],
) -> HardwareCrcPieces {
    let piece = |lanes: HardwareCrcLanes, bytes: &[Vec<u8>]| {
        let covered = hardware
            .stage_over_the_units(lanes, bytes)
            .map_or(0, |(covered_bytes, _folded)| covered_bytes);
        let timing = median_stage_timing(|| {
            std::hint::black_box(hardware.stage_over_the_units(lanes, bytes));
        });
        (
            timing.map(|read| read.cpu_nanoseconds_per_repetition),
            covered,
        )
    };
    HardwareCrcPieces {
        crc: HardwareCrcLanes::ALL.map(|lanes| piece(lanes, units)),
        release_check_crc: HardwareCrcLanes::ALL.map(|lanes| piece(lanes, copies)),
    }
}

/// 一次调用之后量新的计时件（各 5 次取中位，[`median_stage_timing`]）：两种硬件写法的 `crc` 与 `release_check_crc`、抄的扫描（索引在计时之外建好）、
/// 建索引；今天写法那三样与选段取 `replay`（同一次调用的阶段重放）。另做 K-S1、K-S2（计时之外）。
pub(super) fn cpu_writings_of_the_call(
    material: &CallReplayMaterial,
    region: Option<CompactionRegion>,
    replay: &CallStageReplay,
    hardware: &HardwareCrc,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> CpuWritingsOfTheCall {
    let units = units_read_by_the_scan(material);
    let copies = &material.release_check_copies;
    let cpu_of =
        |timing: Option<StageTiming>| timing.map(|read| read.cpu_nanoseconds_per_repetition);
    let hardware_pieces = hardware_crc_pieces(hardware, &units, copies);
    let version = &material.version;
    let index = IdentityIndexOfTheMapping::of_the_entries(&version.mapping_entries);
    let identity_index_build = cpu_of(median_stage_timing(|| {
        std::hint::black_box(IdentityIndexOfTheMapping::of_the_entries(
            &version.mapping_entries,
        ));
    }));
    let identity_scan_of = |scanned_region: CompactionRegion| {
        scan_region_with_the_identity_index(
            scanned_region,
            &version.allocation_records,
            &version.mapping_entries,
            &index,
            &version.pinned_slots,
            &material.state.quarantined_slots,
            replay_reader(material, pool_devices),
        )
    };
    let identity_index_scan_matches = region.map(|scanned_region| {
        let crates_scan = scan_region(
            scanned_region,
            &version.allocation_records,
            &version.mapping_entries,
            &version.pinned_slots,
            &material.state.quarantined_slots,
            replay_reader(material, pool_devices),
        );
        identity_scan_of(scanned_region) == crates_scan
    });
    let scan_identity_index_lookup = region.and_then(|scanned_region| {
        cpu_of(median_stage_timing(|| {
            std::hint::black_box(identity_scan_of(scanned_region).is_ok());
        }))
    });
    let mut checked_units = units.clone();
    checked_units.extend(copies.iter().cloned());
    CpuWritingsOfTheCall {
        pieces: CpuWritingPieces {
            crc_today: cpu_of(replay.crc),
            crc_hardware_single: hardware_pieces.crc[0].0,
            crc_hardware_three_way: hardware_pieces.crc[1].0,
            release_check_crc_today: cpu_of(replay.release_check_crc),
            release_check_crc_hardware_single: hardware_pieces.release_check_crc[0].0,
            release_check_crc_hardware_three_way: hardware_pieces.release_check_crc[1].0,
            scan_today: cpu_of(replay.scan),
            scan_identity_index_lookup,
            identity_index_build,
        },
        choose_cpu_nanoseconds: cpu_of(replay.choose),
        crc_bytes: replay.crc_bytes,
        release_check_crc_bytes: replay.release_check_crc_bytes,
        hardware_crc_bytes: hardware_pieces.crc.map(|(_cpu, bytes)| bytes),
        hardware_release_check_crc_bytes: hardware_pieces
            .release_check_crc
            .map(|(_cpu, bytes)| bytes),
        identity_index_entries: index.keys(),
        identity_index_location_keys: index.location_keys(),
        hardware_crc_mismatches: hardware_crc_mismatches(hardware, &checked_units),
        identity_index_scan_matches,
    }
}

// ───── PW1、PW2：五个计时件各自罩得住 2 ms、分得出两倍的活（每个点每轮第一批） ─────

/// 五个计时件（登记第五节阳性对照表 PW1）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum TimedPiece {
    TodayCrc,
    HardwareCrc(HardwareCrcLanes),
    IdentityIndexLookupScan,
    IdentityIndexBuild,
}

impl TimedPiece {
    pub(super) const ALL: [TimedPiece; 5] = [
        TimedPiece::TodayCrc,
        TimedPiece::HardwareCrc(HardwareCrcLanes::Single),
        TimedPiece::HardwareCrc(HardwareCrcLanes::ThreeWay),
        TimedPiece::IdentityIndexLookupScan,
        TimedPiece::IdentityIndexBuild,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            TimedPiece::TodayCrc => "today_crc",
            TimedPiece::HardwareCrc(HardwareCrcLanes::Single) => "hardware_crc_single",
            TimedPiece::HardwareCrc(HardwareCrcLanes::ThreeWay) => "hardware_crc_three_way",
            TimedPiece::IdentityIndexLookupScan => "identity_index_lookup_scan",
            TimedPiece::IdentityIndexBuild => "identity_index_build",
        }
    }

    /// 这个计时件红了作废哪几种写法（登记第十一节「阳性对照红」：那一种写法整段作废）：今天的 CRC 进每种写法的乙段增量，九种都作废。
    pub(super) fn writings_voided(self) -> Vec<CpuWriting> {
        CpuWriting::ALL
            .into_iter()
            .filter(|writing| {
                let (crc_writing, scan_writing) = writing.parts();
                match self {
                    TimedPiece::TodayCrc => true,
                    TimedPiece::HardwareCrc(lanes) => crc_writing == CrcWriting::Hardware(lanes),
                    TimedPiece::IdentityIndexLookupScan => scan_writing != ScanWriting::Today,
                    TimedPiece::IdentityIndexBuild => {
                        scan_writing == ScanWriting::IdentityIndexWithTheBuild
                    }
                }
            })
            .collect()
    }
}

/// 一个计时件跑一遍（`doubled` 时同一批输入喂两遍：单元表接两份、扫描连调两次、索引建两次）。
fn run_the_timed_piece(piece: TimedPiece, is_doubled: bool, inputs: &TimedPieceInputs<'_>) {
    let units = if is_doubled {
        inputs.doubled_units
    } else {
        inputs.units
    };
    let repetitions = if is_doubled { 2 } else { 1 };
    match piece {
        TimedPiece::TodayCrc => {
            std::hint::black_box(crc_stage_over_the_units(units));
        }
        TimedPiece::HardwareCrc(lanes) => {
            std::hint::black_box(inputs.hardware.stage_over_the_units(lanes, units));
        }
        TimedPiece::IdentityIndexLookupScan => {
            for _repetition in 0..repetitions {
                std::hint::black_box((inputs.lookup_scan)().is_ok());
            }
        }
        TimedPiece::IdentityIndexBuild => {
            for _repetition in 0..repetitions {
                std::hint::black_box(IdentityIndexOfTheMapping::of_the_entries(
                    inputs.mapping_entries,
                ));
            }
        }
    }
}

/// 五个计时件共用的输入。
struct TimedPieceInputs<'inputs> {
    units: &'inputs [Vec<u8>],
    doubled_units: &'inputs [Vec<u8>],
    hardware: &'inputs HardwareCrc,
    mapping_entries: &'inputs [CentralMappingEntry],
    lookup_scan: &'inputs dyn Fn() -> Result<RegionScan, RegionScanRefusal>,
}

/// PW1 的判法（2 ms 的 [0.9, 1.1] 倍）与 PW2 的判法（两遍 ÷ 一遍在 [1.8, 2.2]）在一个计时件的三个读数上：交回（自旋增量、两遍 ÷ 一遍、两条各过没过）。
pub(super) fn cpu_writing_control_judgements(
    plain: Option<f64>,
    spun: Option<f64>,
    doubled: Option<f64>,
) -> (Option<f64>, Option<f64>, bool, bool) {
    let increment = plain.zip(spun).map(|(without, with)| with - without);
    let doubling = plain.zip(doubled).map(|(single, double)| double / single);
    let spin_holds = increment.is_some_and(spin_control_holds);
    let double_holds = plain
        .zip(doubled)
        .is_some_and(|(single, double)| double_crc_control_holds(double, single));
    (increment, doubling, spin_holds, double_holds)
}

/// 一次调用上五个计时件各一次 PW1（计时区间里加 2 ms 按线程钟自旋）与 PW2（同一批输入喂两遍），各打一行
/// `name=positive_control row=cpu_writing`；交回没过的计时件。
pub(super) fn emit_the_cpu_writing_controls(
    emitter: &mut Emitter,
    prefix: &str,
    material: &CallReplayMaterial,
    region: CompactionRegion,
    hardware: &HardwareCrc,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<TimedPiece> {
    let units = units_read_by_the_scan(material);
    let doubled_units: Vec<Vec<u8>> = units.iter().chain(units.iter()).cloned().collect();
    let version = &material.version;
    let index = IdentityIndexOfTheMapping::of_the_entries(&version.mapping_entries);
    let lookup_scan = || {
        scan_region_with_the_identity_index(
            region,
            &version.allocation_records,
            &version.mapping_entries,
            &index,
            &version.pinned_slots,
            &material.state.quarantined_slots,
            replay_reader(material, pool_devices),
        )
    };
    let inputs = TimedPieceInputs {
        units: &units,
        doubled_units: &doubled_units,
        hardware,
        mapping_entries: &version.mapping_entries,
        lookup_scan: &lookup_scan,
    };
    let cpu_of =
        |timing: Option<StageTiming>| timing.map(|read| read.cpu_nanoseconds_per_repetition);
    let mut failures = Vec::new();
    for piece in TimedPiece::ALL {
        let plain = cpu_of(median_stage_timing(|| {
            run_the_timed_piece(piece, false, &inputs)
        }));
        let spun = cpu_of(median_stage_timing(|| {
            run_the_timed_piece(piece, false, &inputs);
            let _spun = spin_until_the_thread_cpu_advances(INSERTED_NANOSECONDS);
        }));
        let doubled = cpu_of(median_stage_timing(|| {
            run_the_timed_piece(piece, true, &inputs)
        }));
        let (increment, doubling, spin_holds, double_holds) =
            cpu_writing_control_judgements(plain, spun, doubled);
        for (control, holds, value) in [
            ("spin_two_milliseconds", spin_holds, increment),
            ("doubled_input", double_holds, doubling),
        ] {
            emitter.line(&format!(
                "name=positive_control row=cpu_writing {prefix} piece={} control={control} value={} holds={holds}",
                piece.label(),
                optional_number(value),
            ));
        }
        if !(spin_holds && double_holds) {
            failures.push(piece);
        }
    }
    failures
}

#[cfg(test)]
mod tests;
