//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的三态、系统配置与根环（I-7 类）、指针与位置条目。
//! 与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5）：偏移在这里按字段表各写一份。

use std::collections::BTreeMap;

use singlefs_format::{
    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR,
    ROOT_RECORD_BYTES, ROOT_RING_SLOTS_PER_REGION_MAXIMUM, ROOT_RING_SLOTS_PER_REGION_MINIMUM,
    SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::{
    check_root_slot, check_system_configuration_slot, read_six_byte_unsigned, read_u32, read_u64,
    RootView, SystemConfigurationView,
};

/// checker 读镜像的口子。它不依赖实现的块设备抽象：harness 的崩溃镜像、宿主上的盘镜像各自接一份。
pub trait ImageReader {
    fn devices(&self) -> Vec<u32>;
    fn device_bytes(&self, device: u32) -> Option<u64>;
    /// 读不到（没有这块盘、越界）返回 None。
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>>;
    /// 扫描方向的候选：可能有单元头的 16 KiB 槽号。None = 从单元区起点扫到盘尾。
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>>;
    /// 扫描方向的候选：可能有 journal 记录的环内槽号（从环起点数，每槽一条 4096 字节的记录）。None = 整条环都扫。
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>>;
}

/// 一条不变量在一个镜像上的判定。「不适用」必须带理由，不许静默算成立。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvariantVerdict {
    Holds,
    Violated(String),
    NotApplicable(&'static str),
}

/// 「中央映射条目的 key 与它指的单元头相符」那一条（用户 2026-09-27 定新立，实审 B2），
/// 编号 I-1.11（映射 key 与单元头相符），登记在 `.claude/kb/invariants.md`。
pub const MAPPING_KEY_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";

/// 「自证过的系统配置槽里，池级字段都在这一版读者收的范围里」那一条（实审 A3-checker-2，用户 2026-09-27 定系统配置越界整池拒）：
/// 格式版本 = 1、加密类型 = 0（关）、固定结构槽距、`physical_block_size`、journal 环长在各自的上下界里（[`crate::Verdict`] 的
/// `FormatVersionNotRecognized` / `EncryptionTypeNotOff` / `FixedStructureSlotSpacingOutsideTheFormatRange` /
/// `PhysicalBlockSizeOutsideTheRootSlotBounds` / `JournalRingBytesOutsideTheSupportedRange`）。`invariants.md` 今天没有一条罩得住它，
/// **编号是暂立的，待 kb 第六批登记时定**；改号只改这一处。
pub const SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS: &str = "I-7.13";

/// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
///
/// I-2.4（头校验和覆盖范围） 的射程从「单元头 29 字节预留位」扩到「单元头与指针头部的加密预留位，加密关着时恒 0」
/// （主 agent 2026-09-27 定，实审 A3-checker-2；`invariants.md` 那一行由 kb 第六批改）：池级走读在单元头上判格式版本认得、
/// 29 字节预留位全 0（`walk` 的 `judge_unit_header`），在每条跟随的指针上判头部 MAC 16 + nonce 12 全 0
/// （[`judge_pointer_mac_and_nonce_are_zero`]，D19（块指针的结构与宽度预算） 已定项 3 射程）。
///
/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：实审 A3b Q2 交上来，用户 2026-09-27 JST 17:4x 定「立不变量并同步」；
/// `invariants.md` 那一行由 kb 第八批写。池级走读每读一版的树表判一次（`walk` 的 `judge_tree_table_entries_ordering`）。
pub const IMPLEMENTED_INVARIANTS: [&str; 48] = [
    "I-1.1",
    "I-1.2",
    "I-1.3",
    "I-1.4",
    "I-1.6",
    "I-1.7",
    "I-1.8",
    "I-1.10",
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    "I-2.1",
    "I-2.3",
    "I-2.4",
    "I-2.5",
    "I-3.1",
    "I-3.8",
    "I-3.9",
    "I-3.10",
    "I-3.11",
    "I-4.2",
    "I-4.8",
    "I-5.1",
    "I-5.2",
    "I-5.4",
    "I-7.1",
    "I-7.2",
    "I-7.3",
    "I-7.4",
    "I-7.6",
    "I-7.7",
    "I-7.8",
    "I-7.9",
    "I-7.12",
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    "I-8.6",
    "I-8.7",
    "I-8.8",
    "I-8.9",
    "I-9.1",
    "I-9.2",
    "I-9.4",
    "I-9.6",
    "I-9.7",
    "I-9.10",
    "I-9.12",
    "I-9.13",
    "I-9.14",
    "I-9.15",
    "I-9.16",
];

/// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
#[derive(Default)]
pub struct Judgements {
    evaluated: BTreeMap<&'static str, u64>,
    violations: BTreeMap<&'static str, u64>,
    first_violation: BTreeMap<&'static str, String>,
    not_applicable: BTreeMap<&'static str, &'static str>,
    part_of_the_range_not_judged: BTreeMap<&'static str, &'static str>,
}

impl Judgements {
    /// 评估一次：条件不成立就记下这一处（只留第一处）。
    pub fn judge(&mut self, invariant: &'static str, holds: bool, detail: impl FnOnce() -> String) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        *self.evaluated.entry(invariant).or_insert(0) += 1;
        if !holds {
            *self.violations.entry(invariant).or_insert(0) += 1;
            self.first_violation.entry(invariant).or_insert_with(detail);
        }
    }

    /// 到此为止判了几次违例：走读一条候选根前后各读一次，差就是这条根引用的单元里对不上的那些（I-7.4、I-4.8 按根判）。
    #[must_use]
    pub fn violation_count(&self, invariant: &'static str) -> u64 {
        self.violations.get(invariant).copied().unwrap_or(0)
    }
    /// 这个镜像上整条不适用（例如第 0 代根下面没有记账树）。已经评估过的不改。
    pub fn not_applicable(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.not_applicable.entry(invariant).or_insert(reason);
    }
    /// 这条不变量射程里有一部分在这个镜像上判不了（例如 I-7.4（近 K 代块未被复用） 被抛弃根那一半：根环有读不出的槽）：
    /// 判了的那一部分成立不等于整条成立，没有违例时整条报「不适用」带这个理由、不报成立；判出了违例照报违例。
    pub fn part_of_the_range_not_judged(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.part_of_the_range_not_judged
            .entry(invariant)
            .or_insert(reason);
    }
    /// 这条不变量到此为止的第一处违例说明：一次走读前后各看一眼，走读里第一处对不上的是什么（I-7.4 被抛弃根那一半报它）。
    #[must_use]
    pub fn first_violation_of(&self, invariant: &'static str) -> Option<&str> {
        self.first_violation.get(invariant).map(String::as_str)
    }
    #[must_use]
    pub fn into_report(self) -> Vec<(&'static str, InvariantVerdict)> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                let verdict = if let Some(detail) = self.first_violation.get(invariant) {
                    InvariantVerdict::Violated(detail.clone())
                } else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {
                    InvariantVerdict::NotApplicable(reason)
                } else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {
                    InvariantVerdict::Holds
                } else {
                    InvariantVerdict::NotApplicable(
                        self.not_applicable
                            .get(invariant)
                            .copied()
                            .unwrap_or("这个镜像上没有它判的对象"),
                    )
                };
                (*invariant, verdict)
            })
            .collect()
    }
}

/// 系统配置里 checker 要用的几何（偏移按系统配置字段表各写一份：物理块 317、journal 环起点 325 与长度 333、R 361、S 362、P 363、
/// chunk 367、根环基址 371、逐区域设备 379、单元区起点 417、io_min 425、槽距 429）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolGeometry {
    pub filesystem_identifier: [u8; 16],
    pub physical_block_size: u32,
    pub journal_ring_start_slot: u64,
    pub journal_ring_bytes: u64,
    pub regions: u64,
    pub slots_per_region: u64,
    pub prime_step: u64,
    pub chunk_bytes: u64,
    pub base_slot: u64,
    pub region_devices: [u32; 3],
    pub unit_area_start_slot: u64,
    pub slot_spacing: u64,
}

/// 系统配置字段表给逐区域设备身份留了几个字段：偏移 379 / 383 / 387，一共三个。
/// R（偏移 361 那一字节）是**盘上读来的值**，可以是 0..255 里的任何一个；它大于这个数时，
/// 第 3 个及以后的区域在字段表里根本没有设备身份可读——这份系统配置这个格式版本读不了。
pub const REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION: usize = 3;

/// 从一个自证过的系统配置槽里取几何。
///
/// # Errors
/// R 大于 [`REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION`] ⇒
/// [`crate::Verdict::RegionCountPastTheRegionDeviceFields`]：不夹成 3 往下走（那是静默跳过几个区域，
/// 根环少读几个槽而没人说），也不 panic（panic 面普查 R12：`root_slot_positions` 拿它下标一个长 3 的数组）。
///
/// S（偏移 362 那一字节）不在格式承诺的区间里 ⇒
/// [`crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval`]，同一条理由：
/// 夹一下往下走就是按一个不存在的几何走读。两条判据都在盘上那一字节进 [`PoolGeometry`] 的唯一入口上。
///
/// 固定结构槽距、`physical_block_size`、journal 环长同样是盘上读来的数，同一个入口上判
/// （`fixed_structure_slot_spacing_lies_in_the_format_range`、`physical_block_size_fits_a_root_slot`、
/// `journal_ring_bytes_lie_in_the_supported_range`），越界各报一个成员。这三个成员与 R、S 那两个处置不同：池级读法把它们认成
/// 「带这一版读者不收的值」（[`value_refused_by_this_reader`]），报违例、整池不作保（用户 2026-09-27 定系统配置越界整池拒）；
/// R、S 越界照旧是这一槽不可择。槽距先判：它是根槽宽的上界。
pub fn geometry_of(
    slot: &[u8],
    view: &SystemConfigurationView,
) -> Result<PoolGeometry, crate::Verdict> {
    let regions = u64::from(slot[361]);
    if regions
        > u64::try_from(REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION).expect("三个区域字段")
    {
        return Err(crate::Verdict::RegionCountPastTheRegionDeviceFields);
    }
    let slots_per_region = u64::from(slot[362]);
    if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)
        .contains(&slots_per_region)
    {
        return Err(crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval);
    }
    let journal_ring_start_slot = read_u64(slot, 325);
    let journal_ring_bytes = read_u64(slot, 333);
    let base_slot = read_u64(slot, 371);
    let unit_area_start_slot = read_u64(slot, 417);
    let slot_spacing = read_u32(slot, 429);
    if !fixed_structure_slot_spacing_lies_in_the_format_range(slot_spacing, base_slot) {
        return Err(crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange);
    }
    if !physical_block_size_fits_a_root_slot(view.declared_physical_block_size, slot_spacing) {
        return Err(crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds);
    }
    if !journal_ring_bytes_lie_in_the_supported_range(
        journal_ring_start_slot,
        journal_ring_bytes,
        unit_area_start_slot,
    ) {
        return Err(crate::Verdict::JournalRingBytesOutsideTheSupportedRange);
    }
    Ok(PoolGeometry {
        filesystem_identifier: view.filesystem_identifier,
        physical_block_size: view.declared_physical_block_size,
        journal_ring_start_slot,
        journal_ring_bytes,
        regions,
        slots_per_region,
        prime_step: u64::from(read_u32(slot, 363)),
        chunk_bytes: u64::from(read_u32(slot, 367)),
        base_slot,
        region_devices: [
            read_u32(slot, 379),
            read_u32(slot, 383),
            read_u32(slot, 387),
        ],
        unit_area_start_slot,
        slot_spacing: u64::from(slot_spacing),
    })
}

/// 固定结构槽距在格式允许的区间里：不小于 4096（D2（RAID 条带策略） 已定项 19「槽距 = 4096 向上取整到 io_min 的整数倍」），
/// 且系统配置槽 1（设备内偏移 = 槽距、宽 4096，D22（单元原子性怎么合成） 已定项 16）整槽落在根环起点之前——
/// 根环起点取同一槽「根环起点」字段 × 16384（已定项 16 第 1 句），不取编译期常量：checker 走读根环也按这个字段。
/// 那个字段是盘上读来的 8 字节，乘出来超过 u64 时根环起点在任何设备偏移之外，槽 1 自然落在它之前。
fn fixed_structure_slot_spacing_lies_in_the_format_range(
    slot_spacing: u32,
    root_ring_base_slot: u64,
) -> bool {
    let slot_one_end = u64::from(slot_spacing) + SYSTEM_CONFIGURATION_SLOT_BYTES;
    u64::from(slot_spacing) >= FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        && root_ring_base_slot
            .checked_mul(SLOT_BYTES)
            .is_none_or(|root_ring_base_offset| slot_one_end <= root_ring_base_offset)
}

/// `physical_block_size` 是根槽宽（D22（单元原子性怎么合成） 已定项 2「根槽宽等于挂载时探测的 physical_block_size」）：
/// 装得下一条根记录（`ROOT_RECORD_BYTES`，已定项 7），且不宽过固定结构槽距（根槽 j 在区域起点 + j × 槽距，已定项 16 第 2 句；
/// 宽过槽距相邻两槽就叠在一起）。读根槽按它开缓冲，不设上界时盘上 4 字节就定得了每一次读要多少内存。
fn physical_block_size_fits_a_root_slot(physical_block_size: u32, slot_spacing: u32) -> bool {
    u64::from(physical_block_size) >= ROOT_RECORD_BYTES && physical_block_size <= slot_spacing
}

/// journal 环长装得下 F 条记录（在飞记录数上限 = 环槽数 ÷ F 不是 0，D23（journal 的角色与格式） 已定项 18；F 取格式常量
/// `JOURNAL_SAFETY_FACTOR`，不读槽里那 4 字节，那一格没有上下界可判），且环的末端（同一槽「journal 环起点」字段 × 16384 + 环长）
/// 不越过同一槽的「单元区起始槽号」字段 × 16384（已定项 19 ③：单元区起始槽号随环长走；checker 扫单元区也按这个字段）。
/// 两个槽号字段都是盘上读来的 8 字节：环末端乘加溢出算越界；单元区起点乘出来超过 u64 时它在任何设备偏移之外。
fn journal_ring_bytes_lie_in_the_supported_range(
    journal_ring_start_slot: u64,
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
) -> bool {
    let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;
    let journal_ring_end = journal_ring_start_slot
        .checked_mul(SLOT_BYTES)
        .and_then(|journal_ring_start| journal_ring_start.checked_add(journal_ring_bytes));
    in_flight_record_limit >= 1
        && journal_ring_end.is_some_and(|journal_ring_end| {
            unit_area_start_slot
                .checked_mul(SLOT_BYTES)
                .is_none_or(|unit_area_start| journal_ring_end <= unit_area_start)
        })
}

/// 一个系统配置槽读下来是什么样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemConfigurationSlotReading {
    /// 自证过、incompat 位认得、池级字段都在这一版读者收的范围里：可择。
    Usable(SystemConfigurationView, PoolGeometry),
    /// 自证过（magic 与整槽校验和都对）、incompat 位认得、R 与 S 在区间里，而一个池级字段越出这一版读者收的范围
    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员）。实现遇到这一槽整池拒绝挂载
    /// （`singlefs_core::recovery::RecoveryFailure::SystemConfigurationValueRefused`）；checker 同一个处置：报
    /// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`] 违例、不替这个池作保，不退成「这一槽不可择」静默跳过
    /// （用户 2026-09-27 定系统配置越界整池拒）。
    CarriesAValueThisReaderRefuses {
        offset_in_bytes: u64,
        refused: crate::Verdict,
    },
    /// 不自证（读不到、magic 或整槽校验和不对）、incompat 位不认得、R 或 S 越界：这一槽不可择（R、S 的既有处置，不动）。
    NotChoosable,
}

/// 一个自证过的系统配置槽被拒的理由是不是「池级字段越出这一版读者收的范围」。穷举写全：新加一个 [`crate::Verdict`] 成员时这里编译不过，
/// 逼着判它归哪一边。
#[must_use]
pub const fn value_refused_by_this_reader(verdict: &crate::Verdict) -> bool {
    match verdict {
        crate::Verdict::FormatVersionNotRecognized
        | crate::Verdict::EncryptionTypeNotOff
        | crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange
        | crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds
        | crate::Verdict::JournalRingBytesOutsideTheSupportedRange => true,
        crate::Verdict::Valid
        | crate::Verdict::BadMagic
        | crate::Verdict::ChecksumMismatch
        | crate::Verdict::UnknownIncompatBit
        | crate::Verdict::FilesystemIdentifierMismatch
        | crate::Verdict::NonZeroFlags
        | crate::Verdict::TooShort
        | crate::Verdict::WrongUnitClass
        | crate::Verdict::DeclaredLengthMismatch
        | crate::Verdict::KeyWidthMismatch
        | crate::Verdict::KeysNotStrictlyAscending
        | crate::Verdict::KeyOutsideDeclaredRange
        | crate::Verdict::UnknownRecordType
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords => false,
    }
}

fn read_system_configuration_slot(
    reader: &dyn ImageReader,
    device: u32,
    offset_in_bytes: u64,
) -> SystemConfigurationSlotReading {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let Some(bytes) = reader.read(device, offset_in_bytes, slot_bytes) else {
        return SystemConfigurationSlotReading::NotChoosable;
    };
    let refused_or_not_choosable = |verdict: crate::Verdict| {
        if value_refused_by_this_reader(&verdict) {
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes,
                refused: verdict,
            }
        } else {
            SystemConfigurationSlotReading::NotChoosable
        }
    };
    let view = match check_system_configuration_slot(&bytes) {
        Ok(view) => view,
        Err(verdict) => return refused_or_not_choosable(verdict),
    };
    match geometry_of(&bytes, &view) {
        Ok(geometry) => SystemConfigurationSlotReading::Usable(view, geometry),
        Err(verdict) => refused_or_not_choosable(verdict),
    }
}

/// 每盘两槽各读成一个 [`SystemConfigurationSlotReading`]：槽 0 在偏移 0；槽 1 的偏移按这块盘槽 0 记的槽距，这块盘的槽 0 不可用时
/// 按池里（`reader.devices()` 的次序）第一块槽 0 可用的盘记的槽距，全池的槽 0 都不可用时才按最小槽距 4096（2026-09-14 用户收尾弹窗定甲；
/// 借别的盘那一档与恢复 `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration` 同一个读法，实审 A2a 第 37 条改的，这里跟着对齐）。
#[must_use]
pub fn system_configuration_slot_readings(
    reader: &dyn ImageReader,
) -> Vec<(u32, [SystemConfigurationSlotReading; 2])> {
    let slot_zero_of_each_device: Vec<(u32, SystemConfigurationSlotReading)> = reader
        .devices()
        .into_iter()
        .map(|device| (device, read_system_configuration_slot(reader, device, 0)))
        .collect();
    let spacing_recorded_by_slot_zero = |reading: &SystemConfigurationSlotReading| match reading {
        SystemConfigurationSlotReading::Usable(_, geometry) => Some(geometry.slot_spacing),
        SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::NotChoosable => None,
    };
    let spacing_recorded_by_the_first_valid_slot_zero = slot_zero_of_each_device
        .iter()
        .find_map(|(_, slot_zero)| spacing_recorded_by_slot_zero(slot_zero));
    slot_zero_of_each_device
        .into_iter()
        .map(|(device, slot_zero)| {
            let spacing = spacing_recorded_by_slot_zero(&slot_zero)
                .or(spacing_recorded_by_the_first_valid_slot_zero)
                .unwrap_or(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES);
            let slot_one = read_system_configuration_slot(reader, device, spacing);
            (device, [slot_zero, slot_one])
        })
        .collect()
}

/// 每盘两槽里可择的系统配置（[`SystemConfigurationSlotReading::Usable`]），槽的读法见 [`system_configuration_slot_readings`]。
/// I-7.7（系统配置实例代号不低于根环） 按这个读法取每盘的实例代号（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_system_configuration_slots(
    reader: &dyn ImageReader,
) -> Vec<(u32, Vec<(SystemConfigurationView, PoolGeometry)>)> {
    system_configuration_slot_readings(reader)
        .into_iter()
        .map(|(device, readings)| {
            let usable = readings
                .into_iter()
                .filter_map(|reading| match reading {
                    SystemConfigurationSlotReading::Usable(view, geometry) => {
                        Some((view, geometry))
                    }
                    SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                    | SystemConfigurationSlotReading::NotChoosable => None,
                })
                .collect();
            (device, usable)
        })
        .collect()
}

/// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`]：每个可择的槽判一次成立，每个带越界值的槽判一次违例。
/// 交回有没有带越界值的槽——有就是实现整池拒绝挂载的镜像（`RecoveryFailure::SystemConfigurationValueRefused`，任一盘任一槽）。
pub fn judge_system_configuration_values_the_reader_accepts(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) -> bool {
    let mut any_slot_carries_a_refused_value = false;
    for (device, slots) in readings {
        for slot in slots {
            match slot {
                SystemConfigurationSlotReading::Usable(..) => judgements.judge(
                    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                    true,
                    String::new,
                ),
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                    offset_in_bytes,
                    refused,
                } => {
                    any_slot_carries_a_refused_value = true;
                    judgements.judge(
                        SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                        false,
                        || {
                            format!(
                                "盘 {device} 偏移 {offset_in_bytes} 的系统配置槽自证过，而池级字段越出这一版读者收的范围（{refused:?}）：实现整池拒绝挂载"
                            )
                        },
                    );
                }
                SystemConfigurationSlotReading::NotChoosable => {}
            }
        }
    }
    any_slot_carries_a_refused_value
}

/// 每盘择一个系统配置：两槽里世代号大的那一份（相等取槽 0）。
#[must_use]
pub fn chosen_system_configurations(
    reader: &dyn ImageReader,
) -> Vec<(u32, Option<(SystemConfigurationView, PoolGeometry)>)> {
    verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            let chosen = slots.into_iter().fold(
                None,
                |best: Option<(SystemConfigurationView, PoolGeometry)>, candidate| match best {
                    Some(current) if candidate.0.slot_generation <= current.0.slot_generation => {
                        Some(current)
                    }
                    Some(_) | None => Some(candidate),
                },
            );
            (device, chosen)
        })
        .collect()
}

/// 根环全部槽：(区域, 槽, 盘, 偏移)，择根与 I-7.7（系统配置实例代号不低于根环） 的「读不全」判定共用这一个算式。
#[must_use]
pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {
    let mut positions = Vec::new();
    for region in 0..geometry.regions {
        // 下标在范围内不是这里判的：`geometry_of` 是盘上那一字节进 `PoolGeometry` 的唯一入口，
        // 它已经把 R > 3 的槽拒成 `Verdict::RegionCountPastTheRegionDeviceFields`、不交出 `PoolGeometry`。
        let device = geometry.region_devices[usize::try_from(region).expect(
            "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
        )];
        for slot in 0..geometry.slots_per_region {
            let offset = geometry.base_slot * SLOT_BYTES
                + region * geometry.prime_step * geometry.chunk_bytes
                + slot * geometry.slot_spacing;
            positions.push((region, slot, device, offset));
        }
    }
    positions
}

/// 根环全部槽里自证过的根（I-7.1 的集合 S），带它住的区域与槽号。
#[must_use]
pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    root_slot_positions(geometry)
        .into_iter()
        .filter_map(|(region, slot, device, offset)| {
            reader
                .read(device, offset, slot_bytes)
                .and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())
                .map(|view| (region, slot, view))
        })
        .collect()
}

/// 位置条目：设备身份 4 + 槽号 6 + 整单元校验和 4。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerLocation {
    pub device: u32,
    pub slot: u64,
    pub checksum: u32,
}

fn location_at(bytes: &[u8], offset: usize) -> PointerLocation {
    PointerLocation {
        device: read_u32(bytes, offset),
        slot: read_six_byte_unsigned(bytes, offset + 4),
        checksum: read_u32(bytes, offset + 10),
    }
}

/// 指针的公共部分：头部 50（出生树在 34、出生 txg 在 42）+ 位置条目 14 × 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerView {
    pub birth_tree: u64,
    pub birth_txg: u64,
    pub locations: [PointerLocation; 2],
    /// 码 2 / 码 3 指针：实例代号 + 出生序号；码 1 指针：写序的实例代号 + 事务号。
    pub instance: u32,
    pub tail: u64,
    pub all_zero: bool,
    /// 头部 MAC 16（偏移 0）与 nonce 12（偏移 16）那 28 字节全是 0（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
    /// 加密关着时它们恒 0，读者遇到非 0 判该指针所在的结构损坏（已定项 3 射程）；池级走读在每条跟随的指针上按 I-2.4 判
    /// （[`judge_pointer_mac_and_nonce_are_zero`]）。
    pub is_mac_and_nonce_all_zero: bool,
}

/// 指针头部 MAC 16 + nonce 12：从偏移 0 起连着的 28 字节（D19（块指针的结构与宽度预算） 已定项 11）。
const POINTER_MAC_AND_NONCE_BYTES: usize = 16 + 12;

fn mac_and_nonce_are_all_zero(pointer_bytes: &[u8]) -> bool {
    pointer_bytes[..POINTER_MAC_AND_NONCE_BYTES]
        .iter()
        .all(|byte| *byte == 0)
}

/// 指向码 2 / 码 3 的指针 86：尾段 = 实例代号 4 + 出生序号 4。
#[must_use]
pub fn parse_node_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: u64::from(read_u32(bytes, 82)),
        all_zero: bytes[..86].iter().all(|byte| *byte == 0),
        is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
    }
}

/// 指向码 1 的指针 88：尾段 = 写序（实例代号 4 + 事务号 6）。
#[must_use]
pub fn parse_data_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: read_six_byte_unsigned(bytes, 82),
        all_zero: bytes[..88].iter().all(|byte| *byte == 0),
        is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
    }
}

/// I-2.4（头校验和覆盖范围） 射程里指针那一半（主 agent 2026-09-27 把射程从单元头扩到指针头部）：走读跟随的一条指针，头部 MAC 16 +
/// nonce 12 全是 0。这一版 checker 只收加密类型 0 的系统配置（`check_system_configuration_slot`），它判的池都是加密关着的，
/// 那 28 字节只能是 0（D19（块指针的结构与宽度预算） 已定项 3 射程；实现那一侧是 `crates/singlefs-core/src/pointer.rs` 的
/// `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`，checker 自己按偏移表判一份）。只判不断：指针指的单元照走。
pub fn judge_pointer_mac_and_nonce_are_zero(
    judgements: &mut Judgements,
    pointer: &PointerView,
    what: &str,
) {
    judgements.judge("I-2.4", pointer.is_mac_and_nonce_all_zero, || {
        format!("{what}：指针头部 MAC 16 / nonce 12 有非 0 字节，加密关着时它们恒 0")
    });
}

/// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免。
pub fn judge_location_order(judgements: &mut Judgements, pointer: &PointerView, what: &str) {
    if pointer.all_zero {
        return;
    }
    judge_location_entries_order(judgements, &pointer.locations, what);
}

/// I-2.5 对一个位置条目数组判一次（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17）：
/// 设备身份严格升序，相同或逆序判损坏。不带「整条全零」的豁免——那一条射程只写给 86 字节全零的指针（「注册了但还没有根」），
/// journal 点名项点的是这次发布写出的单元，没有「还没有」的形态。
pub fn judge_location_entries_order(
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    what: &str,
) {
    judgements.judge("I-2.5", locations[0].device < locations[1].device, || {
        format!(
            "{what} 的位置条目设备身份 {} / {} 不是严格升序",
            locations[0].device, locations[1].device
        )
    });
}

/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。
pub fn read_referenced_unit(
    reader: &dyn ImageReader,
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    unit_bytes: usize,
    what: &str,
) -> Option<Vec<u8>> {
    let mut first_good = None;
    for location in locations {
        let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);
        let matches = bytes
            .as_ref()
            .is_some_and(|content| crate::crc32_castagnoli_table(content) == location.checksum);
        judgements.judge("I-2.1", matches, || {
            format!(
                "{what} 在盘 {} 槽 {} 的那一份与位置条目里的校验和对不上",
                location.device, location.slot
            )
        });
        if matches && first_good.is_none() {
            first_good = bytes;
        }
    }
    first_good
}
