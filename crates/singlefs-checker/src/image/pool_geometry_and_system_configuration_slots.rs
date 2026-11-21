//! 系统配置槽：每块盘两槽的读法、这个读者认不认得槽里的值、从择定的系统配置读出的池几何，
//! 与系统配置那几条判定（取值读者认得、自己的设备号、代号随轮换、字段表之后恒 0、journal 环几何三格、决定宽度与根槽对齐、至少一块盘有自证过的一槽）。

use super::*;

/// 系统配置里 checker 要用的几何（偏移按系统配置字段表各写一份：物理块 317、journal 环起点 325 与长度 333、记录尺寸 341、
/// 在飞上限 345、最坏占用 349、安全系数 F 357、R 361、S 362、P 363、chunk 367、根环基址 371、逐区域设备 379、单元区起点 417、
/// io_min 425、槽距 429）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolGeometry {
    pub filesystem_identifier: [u8; 16],
    pub physical_block_size: u32,
    pub journal_ring_start_slot: u64,
    pub journal_ring_bytes: u64,
    /// 槽里自述的 journal 在飞记录数上限（偏移 345，4 字节）：I-8.1 拿它与 ⌊环长 ÷ 4096 ÷ F⌋ 比、拿它数同一次发布的记录条数。
    /// I-4.1 的重放前缀不按它截，按环长与格式常量 F 现算（D23（journal 的角色与格式） 已定项 18，与实现恢复同一个读法）。
    pub declared_in_flight_record_limit: u64,
    /// 槽里自述的 journal 最坏占用（偏移 349，8 字节，单位字节）：I-8.1 拿它与在飞上限 × 4096 比。
    pub declared_worst_case_occupancy_bytes: u64,
    /// 槽里自述的安全系数 F（偏移 357，4 字节）：I-8.1 判它等于格式常量、拿它算在飞上限。
    pub declared_safety_factor: u64,
    /// 槽里自述的 io_min（偏移 425，4 字节；0 = 没声明）：I-7.5 的判定宽度取 max(physical_block_size, io_min)。
    pub declared_minimum_input_output_bytes: u32,
    pub regions: u64,
    pub slots_per_region: u64,
    /// 槽里自述的区域步长 P（偏移 363）与 chunk（偏移 367）：[`geometry_of`] 判过两样都等于第一版常量才交出 [`PoolGeometry`]。
    /// 放区域不读这两个字段，按格式常量放（[`root_slot_positions`]，与实现的 `root_ring::region_start` 一致）。
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
/// R 是 0 ⇒ [`crate::Verdict::RegionCountZero`]；R 是 1 或 2 ⇒ [`crate::Verdict::RegionCountBelowTheFirstVersionConstant`]
/// （R 与第一版常量 `ROOT_RING_REGIONS` 不等，C663（系统配置槽越界与不收之值两读者分类不同）：R 读并整池拒）。
///
/// S（偏移 362 那一字节）不在格式承诺的区间里 ⇒
/// [`crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval`]，同一条理由：
/// 夹一下往下走就是按一个不存在的几何走读。两条判据都在盘上那一字节进 [`PoolGeometry`] 的唯一入口上。
///
/// 固定结构槽距、`physical_block_size`、journal 环长同样是盘上读来的数，同一个入口上判
/// （`fixed_structure_slot_spacing_lies_in_the_format_range`、`physical_block_size_fits_a_root_slot`、
/// `journal_ring_bytes_lie_in_the_supported_range`），越界各报一个成员。R、S 那四个成员与这三个处置相同：池级读法都认成
/// 「带这一版读者不收的值」（[`value_refused_by_this_reader`]），报违例、整池不作保（系统配置越界整池拒；
/// R、S 两样的分类是 C663（系统配置槽越界与不收之值两读者分类不同）：S 照条款整池拒，R 读并整池拒；S 那一半照 D22（单元原子性怎么合成） 已定项 2
/// 「越界整池拒绝挂载，与每区槽数 S 越界同一个处置」）。槽距先判：它是根槽宽的上界。
///
/// 根环区域步长 P（363）、chunk（367）不等于第一版的格式常量 ⇒ [`crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant`] /
/// [`crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant`]；两样都等而 S × 槽距大于 P × chunk ⇒
/// [`crate::Verdict::RootRingRegionLongerThanTheRegionStride`]（D22（单元原子性怎么合成） 已定项 16 第 1 句）。
/// 这三判排在根槽宽之后、环长之前，与实现的 `system_configuration_values_this_reader_accepts` 同在 R 之后、环长之前；
/// 区域按常量放之后根槽偏移装得进 u64（基槽等于 64、R 等于 3、S ≤ 16、槽距是 4 字节），不另判溢出。
///
/// journal 环起点（325）、根环起点（371）不等于第一版的格式常量，单元区起始槽号（417）不是环长现算的那个槽或不在聚簇段边界上，
/// 同样各报一个成员、同样认成「带这一版读者不收的值」（实审 Y4-a）。
/// 两个起点先于槽距判：槽距的上界按同一槽自述的根环起点算，起点不对时报起点、不报槽距。
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
    if regions == 0 {
        return Err(crate::Verdict::RegionCountZero);
    }
    if regions != ROOT_RING_REGIONS {
        return Err(crate::Verdict::RegionCountBelowTheFirstVersionConstant);
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
    if journal_ring_start_slot != JOURNAL_RING_START_SLOT {
        return Err(crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant);
    }
    if base_slot != ROOT_RING_BASE_SLOT {
        return Err(crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant);
    }
    if !fixed_structure_slot_spacing_lies_in_the_format_range(slot_spacing, base_slot) {
        return Err(crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange);
    }
    if !physical_block_size_fits_a_root_slot(view.declared_physical_block_size, slot_spacing) {
        return Err(crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds);
    }
    let prime_step = u64::from(read_u32(slot, 363));
    let chunk_bytes = u64::from(read_u32(slot, 367));
    if prime_step != ROOT_RING_PRIME_STEP {
        return Err(crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant);
    }
    if chunk_bytes != ROOT_RING_CHUNK_BYTES {
        return Err(crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant);
    }
    if slots_per_region * u64::from(slot_spacing) > ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES {
        return Err(crate::Verdict::RootRingRegionLongerThanTheRegionStride);
    }
    if !journal_ring_bytes_lie_in_the_supported_range(
        journal_ring_start_slot,
        journal_ring_bytes,
        unit_area_start_slot,
    ) {
        return Err(crate::Verdict::JournalRingBytesOutsideTheSupportedRange);
    }
    if !unit_area_start_slot_is_the_slot_after_the_journal_ring(
        journal_ring_start_slot,
        journal_ring_bytes,
        unit_area_start_slot,
    ) {
        return Err(crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing);
    }
    if !unit_area_start_slot.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
        return Err(crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary);
    }
    Ok(PoolGeometry {
        filesystem_identifier: view.filesystem_identifier,
        physical_block_size: view.declared_physical_block_size,
        journal_ring_start_slot,
        journal_ring_bytes,
        declared_in_flight_record_limit: u64::from(read_u32(slot, 345)),
        declared_worst_case_occupancy_bytes: read_u64(slot, 349),
        declared_safety_factor: u64::from(read_u32(slot, 357)),
        declared_minimum_input_output_bytes: read_u32(slot, 425),
        regions,
        slots_per_region,
        prime_step,
        chunk_bytes,
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

/// journal 环长装得下 F 条记录（在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋ 不是 0，D23（journal 的角色与格式） 已定项 18；F 取格式常量
/// `JOURNAL_SAFETY_FACTOR`，不读槽里那 4 字节，那一格没有上下界可判），在飞上限装得进系统配置里它那 4 字节一格（`u32`，
/// `.claude/kb/layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」；装不进时 I-8.1 那一格在飞上限的等式必不成立，
/// 这一格先拒，归 I-7.13、与实现收同一张表），且环的末端（同一槽「journal 环起点」字段 × 16384 + 环长）
/// 不越过同一槽的「单元区起始槽号」字段 × 16384（已定项 19 ③：单元区起始槽号随环长走；checker 扫单元区也按这个字段）。
/// 两个槽号字段都是盘上读来的 8 字节：环末端乘加溢出算越界；单元区起点乘出来超过 u64 时它在任何设备偏移之外。
/// 算式 checker 自己写，不引实现（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 checker 不引实现）。
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
        && u32::try_from(in_flight_record_limit).is_ok()
        && journal_ring_end.is_some_and(|journal_ring_end| {
            unit_area_start_slot
                .checked_mul(SLOT_BYTES)
                .is_none_or(|unit_area_start| journal_ring_end <= unit_area_start)
        })
}

/// 单元区起始槽号是 journal 环末尾的下一个槽：同一槽自述的环起点 + 环长向上取整到 16 KiB 槽（D3（空间分配） 已定项 10 ④
/// 「第一版 = journal 环末尾的下一个槽」；D23（journal 的角色与格式） 已定项 19 ③「单元区起始槽号随环长走」）。
/// checker 自己按条款算一份，不引实现的 `journal::slot_after_the_journal_ring`（门禁 checker-independence-and-sync 的
/// checker-implementation-disjoint 格判 checker 不引实现）。环长是盘上读来的 8 字节：
/// 槽数不超过 2⁶⁴ ÷ 16384，加环起点溢出时算不相等。
fn unit_area_start_slot_is_the_slot_after_the_journal_ring(
    journal_ring_start_slot: u64,
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
) -> bool {
    journal_ring_start_slot
        .checked_add(journal_ring_bytes.div_ceil(SLOT_BYTES))
        .is_some_and(|slot_after_the_journal_ring| {
            unit_area_start_slot == slot_after_the_journal_ring
        })
}

/// 一个系统配置槽读下来是什么样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemConfigurationSlotReading {
    /// 自证过、incompat 位认得、池级字段都在这一版读者收的范围里：可择。
    Usable(SystemConfigurationView, PoolGeometry),
    /// 自证过（magic 与整槽校验和都对）、incompat 位认得，而一个池级字段越出这一版读者收的范围（R 与 S 也算）
    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员；逐档读槽 1 时还有 I-8.1 三个字段等式，
    /// [`RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees`]）。实现遇到这一槽整池拒绝挂载
    /// （`singlefs_core::recovery::RecoveryFailure::SystemConfigurationValueRefused`）；checker 同一个处置：报
    /// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`] 违例、不替这个池作保，不退成「这一槽不可择」静默跳过
    CarriesAValueThisReaderRefuses {
        offset_in_bytes: u64,
        refused: RefusedSystemConfigurationValue,
    },
    /// 不自证：读不到、不足 4096 字节、magic 或整槽校验和不对（I-8.5（tail 槽有效集非空） 的「自证过」= magic 与整槽校验和都对）；
    /// 节点大小不是格式常量的槽也归这里（C840（系统配置节点大小字段没人判），当自证不过，[`crate::Verdict::NodeBytesNotTheFormatConstant`]）。
    NotSelfVerified,
    /// 自证过（magic 与整槽校验和都对），而 incompat 位不认得：这一槽不可择。它仍是 I-8.5 数的「自证过的槽」：tail 住在自证过的槽里（D23（journal 的角色与格式） 已定项 3）。
    /// R、S 越界不在这里，归 [`Self::CarriesAValueThisReaderRefuses`]（C663）。
    SelfVerifiedButNotChoosable,
}

/// 一个带这一版读者不收之值的系统配置槽（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）拒在哪一格。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefusedSystemConfigurationValue {
    /// 自证与几何那几道（[`crate::check_system_configuration_slot`]、[`geometry_of`]）拒的，是 [`value_refused_by_this_reader`] 认的那几个成员之一。
    RefusedByTheSlotChecks(crate::Verdict),
    /// I-8.1（环几何够大） 的三个字段等式里第一个不成立的那一格（[`journal_ring_geometry_field_that_disagrees`]）。只在逐档读槽 1 时这样交回
    /// （[`slot_one_read_at_every_slot_spacing_the_format_allows`]）：实现那张「读者不收之值」表的最后三格就是这三个等式
    /// （`singlefs_core::recovery` 的 `system_configuration_values_this_reader_accepts`），逐档读到它整池拒；
    /// 别的读法上可择的槽照旧由 [`judge_journal_ring_geometry_fields_agree`] 按 I-8.1 判。
    JournalRingGeometryFieldDisagrees(JournalRingGeometryFieldThatDisagrees),
    /// 单元几何四格里第一个与格式常量不等的那一格（[`unit_geometry_field_that_disagrees`]）：自证过的槽在 [`geometry_of`] 之前判，
    /// 实现那张「读者不收之值」表里排在加密类型之后的同四格（`singlefs_core::recovery` 的 `system_configuration_values_this_reader_accepts`）。
    UnitGeometryFieldDisagrees(UnitGeometryFieldThatDisagrees),
}

/// 系统配置几何段紧跟节点大小（301）的单元几何字段：单元大小 305、落点粒度 309、位置条目宽 313 各 4 字节；
/// `physical_block_size`（317）之后是扩展点声明值 N 321，4 字节（`.claude/kb/layout/01-first-txn.md` 一的字段表）。
const SYSTEM_CONFIGURATION_UNIT_BYTES_OFFSET: usize = 305;
const SYSTEM_CONFIGURATION_SLOT_GRANULARITY_BYTES_OFFSET: usize = 309;
const SYSTEM_CONFIGURATION_LOCATION_ENTRY_BYTES_OFFSET: usize = 313;
const SYSTEM_CONFIGURATION_EXTENSION_POINT_DECLARED_BYTES_OFFSET: usize = 321;
/// 这一版读者收的扩展点声明值 N：第一版 N = 0（D21（权威态与派生态的分界） 已定项 8）。checker 自己写一份，不引实现。
const EXTENSION_POINT_DECLARED_BYTES_THIS_CHECKER_READS: u32 = 0;

/// 单元几何四格里哪一格与格式常量不等，次序与实现判这四格的次序相同（字段表次序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitGeometryFieldThatDisagrees {
    /// 槽里的单元大小不等于格式常量 `DATA_UNIT_BYTES`。
    UnitBytesNotTheFormatConstant,
    /// 槽里的落点粒度不等于格式常量 `SLOT_BYTES`。
    SlotGranularityBytesNotTheFormatConstant,
    /// 槽里的位置条目宽不等于格式常量 `LOC_ENTRY`。
    LocationEntryBytesNotTheFormatConstant,
    /// 槽里的扩展点声明值 N 不是第一版的 0。
    ExtensionPointDeclaredBytesNotTheFirstVersionValue,
}

/// 单元几何四格（D21（权威态与派生态的分界） 已定项 16「单元几何在挂载时求值一次」、已定项 8「N 住系统配置，第一版 N = 0」；C1041）：
/// 单元大小、落点粒度、位置条目宽各等于格式常量，扩展点声明值 N 等于 0，按字段表次序判，交回第一个不等的；四格都等交回 None。
/// checker 走读单元、按落点算偏移、切位置条目都按格式常量走，槽里这四格与常量不等时这份系统配置说的不是 checker 判的那个几何。
///
/// # Panics
/// 槽短于偏移 325：调用方读的是整槽 4096 字节，[`crate::check_system_configuration_slot`] 判过槽宽才调这里。
#[must_use]
pub fn unit_geometry_field_that_disagrees(slot: &[u8]) -> Option<UnitGeometryFieldThatDisagrees> {
    if u64::from(read_u32(slot, SYSTEM_CONFIGURATION_UNIT_BYTES_OFFSET)) != DATA_UNIT_BYTES {
        return Some(UnitGeometryFieldThatDisagrees::UnitBytesNotTheFormatConstant);
    }
    if u64::from(read_u32(
        slot,
        SYSTEM_CONFIGURATION_SLOT_GRANULARITY_BYTES_OFFSET,
    )) != SLOT_BYTES
    {
        return Some(UnitGeometryFieldThatDisagrees::SlotGranularityBytesNotTheFormatConstant);
    }
    if u64::from(read_u32(
        slot,
        SYSTEM_CONFIGURATION_LOCATION_ENTRY_BYTES_OFFSET,
    )) != LOC_ENTRY
    {
        return Some(UnitGeometryFieldThatDisagrees::LocationEntryBytesNotTheFormatConstant);
    }
    if read_u32(
        slot,
        SYSTEM_CONFIGURATION_EXTENSION_POINT_DECLARED_BYTES_OFFSET,
    ) != EXTENSION_POINT_DECLARED_BYTES_THIS_CHECKER_READS
    {
        return Some(
            UnitGeometryFieldThatDisagrees::ExtensionPointDeclaredBytesNotTheFirstVersionValue,
        );
    }
    None
}

/// I-8.1（环几何够大） 三个字段等式里的哪一格不成立，次序与实现判这三格的次序相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRingGeometryFieldThatDisagrees {
    /// 槽里的安全系数 F 不等于格式常量 `JOURNAL_SAFETY_FACTOR`。
    SafetyFactorNotTheFormatConstant,
    /// 槽里记的在飞上限不等于 ⌊环长 ÷ 4096 ÷ F⌋。
    InFlightRecordLimitNotRecomputedFromTheRingLength,
    /// 槽里记的最坏占用不等于在飞上限 × 4096（乘出来溢出也算不等）。
    WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
}

/// I-8.1 第二格右边：按同一槽的环长与 F 现算的在飞上限 ⌊环长 ÷ 4096 ÷ F⌋；F 为 0 时没有这个数（None，那一格判不等、不除）。
const fn in_flight_record_limit_recomputed_from_the_ring(geometry: &PoolGeometry) -> Option<u64> {
    (geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES)
        .checked_div(geometry.declared_safety_factor)
}

/// I-8.1 第三格右边：槽里记的在飞上限 × 4096；乘出来溢出时没有这个数（None，那一格判不等）。
const fn worst_case_occupancy_recomputed_from_the_limit(geometry: &PoolGeometry) -> Option<u64> {
    geometry
        .declared_in_flight_record_limit
        .checked_mul(JOURNAL_RECORD_BYTES)
}

/// I-8.1（环几何够大） 的三个字段等式按 F、在飞上限、最坏占用的次序判，交回第一个不成立的；三格都成立交回 None。
/// 三格与 [`judge_journal_ring_geometry_fields_agree`] 判的是同三个等式（后两格的右边共用上面两个算式）。
#[must_use]
pub fn journal_ring_geometry_field_that_disagrees(
    geometry: &PoolGeometry,
) -> Option<JournalRingGeometryFieldThatDisagrees> {
    if geometry.declared_safety_factor != JOURNAL_SAFETY_FACTOR {
        return Some(JournalRingGeometryFieldThatDisagrees::SafetyFactorNotTheFormatConstant);
    }
    if in_flight_record_limit_recomputed_from_the_ring(geometry)
        != Some(geometry.declared_in_flight_record_limit)
    {
        return Some(
            JournalRingGeometryFieldThatDisagrees::InFlightRecordLimitNotRecomputedFromTheRingLength,
        );
    }
    if worst_case_occupancy_recomputed_from_the_limit(geometry)
        != Some(geometry.declared_worst_case_occupancy_bytes)
    {
        return Some(
            JournalRingGeometryFieldThatDisagrees::WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
        );
    }
    None
}

/// 一个自证过的系统配置槽被拒的理由是不是「池级字段越出这一版读者收的范围」。穷举写全：新加一个 [`crate::Verdict`] 成员时这里编译不过，
/// 逼着判它归哪一边。
#[must_use]
pub const fn value_refused_by_this_reader(verdict: &crate::Verdict) -> bool {
    match verdict {
        // R 与第一版常量 3 不等、S 越界整池拒：C663（系统配置槽越界与不收之值两读者分类不同）：S 照条款整池拒，R 读并整池拒
        crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RegionCountZero
        | crate::Verdict::RegionCountBelowTheFirstVersionConstant => true,
        // P、chunk 与第一版常量不等、S × 槽距大于 P × chunk 整池拒：D22（单元原子性怎么合成） 已定项 16 第 1 句
        crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant
        | crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant
        | crate::Verdict::RootRingRegionLongerThanTheRegionStride => true,
        crate::Verdict::FormatVersionNotRecognized
        | crate::Verdict::EncryptionTypeNotOff
        | crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange
        | crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds
        | crate::Verdict::JournalRingBytesOutsideTheSupportedRange
        | crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant
        | crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant
        | crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing
        | crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary => true,
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
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords
        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero
        | crate::Verdict::NodeBytesNotTheFormatConstant => false,
    }
}

/// 一个系统配置槽被拒的理由是不是「它自己证不了自己」：不足 4096 字节、magic 不对、整槽校验和不对
/// （[`crate::check_system_configuration_slot`] 头三道），以及节点大小不是格式常量（C840（系统配置节点大小字段没人判），当自证不过，
/// 那一道排在格式版本与 incompat 位之后，但处置与头三道相同）。别的理由拒的都是自证过的槽。穷举写全，理由同 [`value_refused_by_this_reader`]。
#[must_use]
pub const fn verdict_says_the_slot_does_not_verify_itself(verdict: &crate::Verdict) -> bool {
    match verdict {
        crate::Verdict::TooShort
        | crate::Verdict::BadMagic
        | crate::Verdict::ChecksumMismatch
        | crate::Verdict::NodeBytesNotTheFormatConstant => true,
        crate::Verdict::Valid
        | crate::Verdict::FormatVersionNotRecognized
        | crate::Verdict::EncryptionTypeNotOff
        | crate::Verdict::FixedStructureSlotSpacingOutsideTheFormatRange
        | crate::Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds
        | crate::Verdict::JournalRingBytesOutsideTheSupportedRange
        | crate::Verdict::JournalRingStartSlotNotTheFirstVersionConstant
        | crate::Verdict::RootRingBaseSlotNotTheFirstVersionConstant
        | crate::Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing
        | crate::Verdict::UnitAreaStartOffTheClusterSegmentBoundary
        | crate::Verdict::UnknownIncompatBit
        | crate::Verdict::FilesystemIdentifierMismatch
        | crate::Verdict::NonZeroFlags
        | crate::Verdict::WrongUnitClass
        | crate::Verdict::DeclaredLengthMismatch
        | crate::Verdict::KeyWidthMismatch
        | crate::Verdict::KeysNotStrictlyAscending
        | crate::Verdict::KeyOutsideDeclaredRange
        | crate::Verdict::UnknownRecordType
        | crate::Verdict::RegionCountPastTheRegionDeviceFields
        | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
        | crate::Verdict::RegionCountZero
        | crate::Verdict::RegionCountBelowTheFirstVersionConstant
        | crate::Verdict::RootRingPrimeStepNotTheFirstVersionConstant
        | crate::Verdict::RootRingChunkBytesNotTheFirstVersionConstant
        | crate::Verdict::RootRingRegionLongerThanTheRegionStride
        | crate::Verdict::EncryptionReservedBytesNotZero
        | crate::Verdict::EntryWidthZeroWithEntries
        | crate::Verdict::RecordWidthZeroWithRecords
        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero => false,
    }
}

fn read_system_configuration_slot(
    reader: &dyn ImageReader,
    device: u32,
    offset_in_bytes: u64,
) -> SystemConfigurationSlotReading {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let Some(bytes) = reader.read(device, offset_in_bytes, slot_bytes) else {
        return SystemConfigurationSlotReading::NotSelfVerified;
    };
    let refused_or_not_choosable = |verdict: crate::Verdict| {
        if value_refused_by_this_reader(&verdict) {
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes,
                refused: RefusedSystemConfigurationValue::RefusedByTheSlotChecks(verdict),
            }
        } else if verdict_says_the_slot_does_not_verify_itself(&verdict) {
            SystemConfigurationSlotReading::NotSelfVerified
        } else {
            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable
        }
    };
    let view = match check_system_configuration_slot(&bytes) {
        Ok(view) => view,
        Err(verdict) => return refused_or_not_choosable(verdict),
    };
    if let Some(disagreeing_field) = unit_geometry_field_that_disagrees(&bytes) {
        return SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
            offset_in_bytes,
            refused: RefusedSystemConfigurationValue::UnitGeometryFieldDisagrees(disagreeing_field),
        };
    }
    match geometry_of(&bytes, &view) {
        Ok(geometry) => SystemConfigurationSlotReading::Usable(view, geometry),
        Err(verdict) => refused_or_not_choosable(verdict),
    }
}

/// 逐档读槽 1 时相邻两档槽距之差：槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID 条带策略） 已定项 19），
/// io_min 是逻辑块宽的整数倍、逻辑块宽最小 512 字节，所以格式允许的槽距都是 512 的整数倍。
const SLOT_SPACING_STEP_BYTES: u64 = 512;

/// 逐档读槽 1 时槽距的上界：槽 1 整槽落在根环起点之前（[`fixed_structure_slot_spacing_lies_in_the_format_range`]），
/// 根环起点取这一版读者收的那一个（「根环起点」字段只收第一版常量 [`ROOT_RING_BASE_SLOT`]，[`geometry_of`] 判），× 16384 − 4096。
const LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS: u64 =
    ROOT_RING_BASE_SLOT * SLOT_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;

/// 全池没有一个可用的槽 0 时，一块盘的槽 1：槽距无处可读，在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、
/// 不越过 [`LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS`]），只收可用、且自述的固定结构槽距等于所在偏移的槽
/// （槽 i 的设备内偏移 = i × 固定结构槽距，D22（单元原子性怎么合成） 已定项 16）；几档都收得到时取世代号最大的，
/// 世代号相同取偏移小的（C651（没有可用槽 0 时 checker 与实现找槽 1 读法不同））。
///
/// 读到的某一档带这一版读者不收的值（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）：交回按偏移升序读到的第一个，
/// 由 I-7.13（系统配置池级字段在读者收的范围里） 报违例、整池不作保——D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」，
/// 与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。
/// 读到可用的一档，先按实现那张「读者不收之值」表剩下的三格判（I-8.1（环几何够大） 三个字段等式，[`journal_ring_geometry_field_that_disagrees`]），
/// 有一格不成立就同样当场交回带不收之值，不再核偏移、比世代号：已定项 16「逐档读到自证过而池级字段越界的槽整池拒」不要求
/// 槽距等于偏移，实现照字面先判值后核偏移（`singlefs_core::recovery` 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`）。
/// 一档可用的都没收到时，有自证过而不可择的就交回 [`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]
/// （I-8.5（tail 槽有效集非空） 数它；它记的槽距可能读不出来，不核偏移），再没有就是 [`SystemConfigurationSlotReading::NotSelfVerified`]。
fn slot_one_read_at_every_slot_spacing_the_format_allows(
    reader: &dyn ImageReader,
    device: u32,
) -> SystemConfigurationSlotReading {
    let mut highest_generation_accepted: Option<(SystemConfigurationView, PoolGeometry)> = None;
    let mut some_slot_verifies_itself_but_is_not_choosable = false;
    // 迭代次数的上界是档数 (LARGEST − 4096) ÷ 512 + 1 = 2033；跨轮带着的是至今世代号最大的那一槽与「见过自证过而不可择的」，
    // 前者只在世代号严格更大时换（同号留偏移小的），后者只置不清；提前出口只有读到带不收之值的那一档。
    for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        ..=LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS)
        .step_by(usize::try_from(SLOT_SPACING_STEP_BYTES).expect("512"))
    {
        match read_system_configuration_slot(reader, device, candidate_spacing) {
            SystemConfigurationSlotReading::Usable(view, geometry) => {
                if let Some(disagreeing_field) =
                    journal_ring_geometry_field_that_disagrees(&geometry)
                {
                    return SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                        offset_in_bytes: candidate_spacing,
                        refused: RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees(
                            disagreeing_field,
                        ),
                    };
                }
                if geometry.slot_spacing != candidate_spacing {
                    continue;
                }
                if highest_generation_accepted
                    .as_ref()
                    .is_none_or(|(accepted, _)| view.slot_generation > accepted.slot_generation)
                {
                    highest_generation_accepted = Some((view, geometry));
                }
            }
            refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {
                return refused;
            }
            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {
                some_slot_verifies_itself_but_is_not_choosable = true;
            }
            SystemConfigurationSlotReading::NotSelfVerified => {}
        }
    }
    match (
        highest_generation_accepted,
        some_slot_verifies_itself_but_is_not_choosable,
    ) {
        (Some((view, geometry)), _) => SystemConfigurationSlotReading::Usable(view, geometry),
        (None, true) => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,
        (None, false) => SystemConfigurationSlotReading::NotSelfVerified,
    }
}

/// 每盘两槽各读成一个 [`SystemConfigurationSlotReading`]：槽 0 在偏移 0；槽 1 的偏移按这块盘槽 0 记的槽距，这块盘的槽 0 不可用时
/// 按池里（`reader.devices` 的次序）第一块槽 0 可用的盘记的槽距（借别的盘那一档与恢复
/// `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration` 同一个读法）；
/// 全池的槽 0 都不可用时逐档读（[`slot_one_read_at_every_slot_spacing_the_format_allows`]，C651）。
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
        | SystemConfigurationSlotReading::NotSelfVerified
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
    };
    let spacing_recorded_by_the_first_valid_slot_zero = slot_zero_of_each_device
        .iter()
        .find_map(|(_, slot_zero)| spacing_recorded_by_slot_zero(slot_zero));
    slot_zero_of_each_device
        .into_iter()
        .map(|(device, slot_zero)| {
            let slot_one = match spacing_recorded_by_slot_zero(&slot_zero)
                .or(spacing_recorded_by_the_first_valid_slot_zero)
            {
                Some(spacing) => read_system_configuration_slot(reader, device, spacing),
                None => slot_one_read_at_every_slot_spacing_the_format_allows(reader, device),
            };
            (device, [slot_zero, slot_one])
        })
        .collect()
}

/// 每盘两槽里可择的系统配置（[`SystemConfigurationSlotReading::Usable`]），槽的读法见 [`system_configuration_slot_readings`]。
/// I-7.7（系统配置实例代号不低于根环） 按这个读法取每盘的实例代号（C322（取号那一步的屏障怎么放没有条款））。
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
                    | SystemConfigurationSlotReading::NotSelfVerified
                    | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
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
                SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {}
            }
        }
    }
    any_slot_carries_a_refused_value
}

/// [`SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY`]：每块盘两槽里每个自证过、属于本池（fsid 相同）的系统配置槽，
/// 自述的本盘设备号等于这块盘在池里的身份（`ImageReader::devices` 给的那个号），每槽判一次。读法与实现可写挂载比盘表时同一个：
/// 自证过的槽、fsid 与择到的那份相同的槽（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`）；
/// 别的池的槽归 I-1.4 判，不在这里。盘体对调（身份 0 的位置上装着自述 1 的盘）时两块盘各红一处。
pub fn judge_own_device_number_is_the_device_identity(
    verified_system_configurations_of_this_pool: &[(u32, Vec<crate::SystemConfigurationView>)],
    judgements: &mut Judgements,
) {
    for (device_identity, slots) in verified_system_configurations_of_this_pool {
        for view in slots {
            judgements.judge(
                SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
                view.this_device == *device_identity,
                || {
                    format!(
                        "盘 {device_identity} 上世代 {} 的系统配置槽自述本盘设备号 {}：与这块盘在池里的身份不同",
                        view.slot_generation, view.this_device
                    )
                },
            );
        }
    }
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
        // 它已经把 R > 3 的槽拒成 `Verdict::RegionCountPastTheRegionDeviceFields`、R = 0 拒成 `Verdict::RegionCountZero`、
        // 基槽、R、S、槽距都过了它的判（基槽等于 64、R 等于 3、S ≤ 16、槽距是 4 字节），下面这几步乘加装得进 u64。
        let device = geometry.region_devices[usize::try_from(region).expect(
            "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
        )];
        for slot in 0..geometry.slots_per_region {
            let offset = geometry.base_slot * SLOT_BYTES
                + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
                + slot * geometry.slot_spacing;
            positions.push((region, slot, device, offset));
        }
    }
    positions
}

/// mkfs 写下的两个系统配置槽的世代号（D22（单元原子性怎么合成） 已定项 16）：两槽相等只许都是它。
pub const SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS: u64 = 1;

/// 每块盘的系统配置槽数（D22（单元原子性怎么合成） 已定项 8：每盘两份）：世代号 ≥ 2 的那一份住「世代号 mod 这个数」号槽。
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;

/// [`SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION`]（I-7.16）：每块盘上，可择的系统配置槽 i（[`SystemConfigurationSlotReading::Usable`]，
/// 带越界值的槽归 I-7.13 整池拒、不进这一条）世代号 ≥ 2 时世代号 mod 2 == i，每槽判一格；两槽都可择时世代号相差恰为 1，
/// 或两槽都是 mkfs 写下的 1，每盘判一格（D22（单元原子性怎么合成） 已定项 16）。
///
/// 同一处调用顺带判 I-2.4 的系统配置槽尾（[`judge_system_configuration_slot_bytes_past_the_field_table_are_zero`]，同一组可择的槽）：
/// 那一判现在挂在这一处调用上，等它在 `walk::check_pool_image` 里自己占一个调用点，再把调用点挪过去、删掉这一句。
pub fn judge_system_configuration_slot_generations_follow_the_rotation(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    judge_system_configuration_slot_bytes_past_the_field_table_are_zero(readings, judgements);
    let generation_of_a_choosable_slot = |reading: &SystemConfigurationSlotReading| match reading {
        SystemConfigurationSlotReading::Usable(view, _) => Some(view.slot_generation),
        SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::NotSelfVerified
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
    };
    for (device, slots) in readings {
        let generations = [
            generation_of_a_choosable_slot(&slots[0]),
            generation_of_a_choosable_slot(&slots[1]),
        ];
        for (slot_index, generation) in (0u64..).zip(generations) {
            let Some(generation) = generation else {
                continue;
            };
            if generation > SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS {
                judgements.judge(
                    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
                    generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE == slot_index,
                    || {
                        format!(
                            "盘 {device} 槽 {slot_index} 的系统配置世代号 {generation}：世代号 ≥ 2 的那一份该住槽 {}",
                            generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                        )
                    },
                );
            }
        }
        if let [Some(slot_zero_generation), Some(slot_one_generation)] = generations {
            let both_written_by_mkfs = slot_zero_generation
                == SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS
                && slot_one_generation == SYSTEM_CONFIGURATION_GENERATION_WRITTEN_BY_MKFS;
            judgements.judge(
                SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
                both_written_by_mkfs || slot_zero_generation.abs_diff(slot_one_generation) == 1,
                || {
                    format!(
                        "盘 {device} 两个系统配置槽的世代号 {slot_zero_generation} / {slot_one_generation}：相差不是 1，也不是 mkfs 写下的两个 1"
                    )
                },
            );
        }
    }
}

/// I-2.4（头校验和覆盖范围） 射程里系统配置槽尾那一处：每块盘两槽里每个可择的系统配置槽（[`SystemConfigurationSlotReading::Usable`]），
/// 字段表之后到槽末（[489, 4096)，D22（单元原子性怎么合成） 已定项 9「槽内余 3607」那一段）全是 0，每槽判一格。
/// 第一版写 0、core 的读者不读这一段，往余量里加字段时同一次改动改这里的端点（C728（根槽与系统配置槽尾部字节没有条款）：恒 0，由 checker 判）。
/// 带越界值的槽归 I-7.13 整池拒、不判；incompat 位不认得的槽（[`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]）按别的布局写，
/// 这一版读者不知道它的字段表到哪为止，也不判。只判不断。
pub fn judge_system_configuration_slot_bytes_past_the_field_table_are_zero(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    for (device, slots) in readings {
        for (slot_index, reading) in (0u64..).zip(slots) {
            let view = match reading {
                SystemConfigurationSlotReading::Usable(view, _) => view,
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                | SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => continue,
            };
            let first_non_zero_byte = view.first_non_zero_byte_past_the_field_table;
            judgements.judge("I-2.4", first_non_zero_byte.is_none(), || {
                let crate::FirstNonZeroByteInTheSystemConfigurationSlot {
                    offset_in_the_slot,
                    value,
                } = first_non_zero_byte.expect("判红时字段表之后有非 0 字节");
                format!(
                    "盘 {device} 系统配置槽 {slot_index}（世代 {}）：槽内偏移 {offset_in_the_slot} 的字节是 {value:#04x}，字段表之后那一段（{SYSTEM_CONFIGURATION_BYTES}..{SYSTEM_CONFIGURATION_SLOT_BYTES}）第一版恒 0",
                    view.slot_generation
                )
            });
        }
    }
}

/// [`ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES`]（I-7.17）：根环里每条自证过的根，`checkpoint_txg` ≥ 1 时住
/// (区域 txg mod R, 槽 (txg div R) mod S)；第 0 代（mkfs 把它种进全部区域，D22（单元原子性怎么合成） 已定项 8）只判槽 0。
/// 公式是 D22（单元原子性怎么合成） 已定项 16。
pub fn judge_root_records_live_in_the_slot_their_txg_names(
    roots: &[(u64, u64, RootView)],
    geometry: &PoolGeometry,
    judgements: &mut Judgements,
) {
    for (region, slot, root) in roots {
        let txg = root.checkpoint_txg;
        let (expected_region, expected_slot) = if txg == GENESIS_ROOT_CHECKPOINT_TXG {
            (None, 0)
        } else {
            let region_of_the_txg = txg
                .checked_rem(geometry.regions)
                .expect("根环里读出了根 ⇒ `root_slot_positions` 至少排出一个区域，R ≥ 1");
            let slot_of_the_txg = (txg / geometry.regions)
                .checked_rem(geometry.slots_per_region)
                .expect("根环里读出了根 ⇒ 每个区域至少一个槽，S ≥ 1");
            (Some(region_of_the_txg), slot_of_the_txg)
        };
        let holds =
            expected_region.is_none_or(|expected| expected == *region) && expected_slot == *slot;
        judgements.judge(ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES, holds, || {
            format!(
                "实例 {}、txg {txg} 的根住在区域 {region} 槽 {slot}：按 txg 算该在区域 {expected_region:?}（第 0 代不判区域）槽 {expected_slot}",
                root.instance
            )
        });
    }
}

/// [`JOURNAL_RING_GEOMETRY_FIELDS_AGREE`]（I-8.1）的三个字段等式：每块盘每个可择的系统配置槽（[`SystemConfigurationSlotReading::Usable`]；
/// 带越界值的槽归 I-7.13 整池拒、不可择的槽 checker 不读它的几何），三格各判一次——
/// 槽里的 F 等于格式常量 `JOURNAL_SAFETY_FACTOR`（`crates/singlefs-checker/invariants/03-journal.md` I-8.1 那一行「F = 格式常量，任一不等判红」，
/// 与实现择系统配置时同判，`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3 行）；
/// 在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋（F 取同一槽里的那个字段，F 为 0 时这一格判红、不除）；最坏占用 = 在飞上限 × 4096
/// （乘出来溢出判红）。「环长 ≥ F × 最坏占用」是这三格的推论，不另判（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行）。
/// 三格有一格不成立，实现整池拒绝挂载，`walk::check_pool_image` 照 I-7.13 那一支整池不作保。
/// 同一次发布的记录数不多于在飞上限那一格读 journal 环，在 `walk` 里判。
pub fn judge_journal_ring_geometry_fields_agree(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    for (device, slots) in readings {
        for (slot_index, reading) in (0u64..).zip(slots) {
            let geometry = match reading {
                SystemConfigurationSlotReading::Usable(_, geometry) => geometry,
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
                | SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => continue,
            };
            let safety_factor = geometry.declared_safety_factor;
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                safety_factor == JOURNAL_SAFETY_FACTOR,
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 的安全系数 F = {safety_factor}，不等于格式常量 {JOURNAL_SAFETY_FACTOR}"
                    )
                },
            );
            let in_flight_record_limit_from_the_ring =
                in_flight_record_limit_recomputed_from_the_ring(geometry);
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                in_flight_record_limit_from_the_ring
                    == Some(geometry.declared_in_flight_record_limit),
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 记的在飞上限 {}，按环长 {} ÷ 4096 ÷ F {safety_factor} 算是 {in_flight_record_limit_from_the_ring:?}",
                        geometry.declared_in_flight_record_limit, geometry.journal_ring_bytes
                    )
                },
            );
            let worst_case_occupancy_from_the_limit =
                worst_case_occupancy_recomputed_from_the_limit(geometry);
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                worst_case_occupancy_from_the_limit
                    == Some(geometry.declared_worst_case_occupancy_bytes),
                || {
                    format!(
                        "盘 {device} 系统配置槽 {slot_index} 记的最坏占用 {} 字节，按在飞上限 {} × 4096 算是 {worst_case_occupancy_from_the_limit:?}",
                        geometry.declared_worst_case_occupancy_bytes,
                        geometry.declared_in_flight_record_limit
                    )
                },
            );
        }
    }
}

/// I-7.5 的判定宽度：各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的那个（`io_min` 为 0 视为没声明，
/// 取 max 时自然落到 `physical_block_size`；异构池取最大）。
/// checker 不探测设备，这个宽度一律是声明值（[`crate::DecisionWidth::DeclaredOnly`]，探测比对仍欠 C388（checker 的判定宽度不自己探测））。
/// 没有一块盘择得到系统配置时交 None。
#[must_use]
pub fn decision_width_declared_by_the_chosen_system_configurations(
    chosen_geometries: &[PoolGeometry],
) -> Option<crate::DecisionWidth> {
    chosen_geometries
        .iter()
        .map(|geometry| {
            geometry
                .physical_block_size
                .max(geometry.declared_minimum_input_output_bytes)
        })
        .max()
        .map(|bytes| crate::DecisionWidth::DeclaredOnly { bytes })
}

/// [`ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH`]（I-7.5）：根环 R × S 个槽逐个判，起始偏移（设备内字节）是判定宽度的整数倍。
/// 只判第 0 个不够：偏移半个槽的槽照样跨两个原子单元（E41（根环槽几何 2×4×256））。判定宽度 0 不会出现
/// （`geometry_of` 判过 `physical_block_size` ≥ 根记录宽），出现了这一格判红、不除。
pub fn judge_root_slots_aligned_to_the_decision_width(
    geometry: &PoolGeometry,
    decision_width: crate::DecisionWidth,
    judgements: &mut Judgements,
) {
    let width_in_bytes = u64::from(decision_width.bytes());
    for (region, slot, device, offset) in root_slot_positions(geometry) {
        judgements.judge(
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            offset
                .checked_rem(width_in_bytes)
                .is_some_and(|remainder| remainder == 0),
            || {
                format!(
                    "根环区域 {region} 槽 {slot}（盘 {device} 偏移 {offset}）不是判定宽度 {decision_width:?}（声明值，未探测）的整数倍"
                )
            },
        );
    }
}

/// 一个系统配置槽是不是自证过（magic 与整槽校验和都对，节点大小是格式常量）：I-8.5（tail 槽有效集非空） 与旁路报告都按这个数。
const fn slot_verifies_itself(reading: &SystemConfigurationSlotReading) -> bool {
    match reading {
        SystemConfigurationSlotReading::Usable(..)
        | SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => true,
        SystemConfigurationSlotReading::NotSelfVerified => false,
    }
}

/// [`SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION`]（I-8.5，V-池）判一格：全池至少一块盘的两个系统配置槽里有一个自证过。
/// 对着只读恢复判：单盘全废只读恢复照样挂得上，不判违例，
/// 走旁路报告 [`system_configuration_redundancy_report`]。
pub fn judge_some_device_carries_a_self_verified_system_configuration(
    readings: &[(u32, [SystemConfigurationSlotReading; 2])],
    judgements: &mut Judgements,
) {
    let some_slot_verifies_itself = readings
        .iter()
        .any(|(_, slots)| slots.iter().any(slot_verifies_itself));
    judgements.judge(
        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
        some_slot_verifies_itself,
        || {
            format!(
                "池里 {} 块盘的系统配置槽一个都没自证过：tail 没有见证者",
                readings.len()
            )
        },
    );
}

/// 系统配置冗余的旁路报告：不进违例通道（I-8.5 对着只读恢复判，这两样只读恢复都挂得上）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationRedundancyReport {
    /// 两个槽都没自证过的盘（单盘全废）。
    pub devices_without_a_self_verified_slot: Vec<u32>,
    /// 少于两个槽自证过的盘（少一槽自证过；单盘全废的盘也在这里）。这类盘的可写挂载带读故障时可能判不出而拒（判不出要有槽设备报读错，
    /// D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」）；没有读故障时挂得上，取号写两轮补回那一槽。
    pub devices_with_fewer_than_two_self_verified_slots: Vec<u32>,
}

/// 逐盘数自证过的系统配置槽（magic 与整槽校验和都对），交回 [`SystemConfigurationRedundancyReport`]。槽的读法同
/// [`system_configuration_slot_readings`]。「本池自证不过」那一半（fsid 是别的池的槽）这里不分：数的是 I-8.5 那一行的「自证过」。
#[must_use]
pub fn system_configuration_redundancy_report(
    reader: &dyn ImageReader,
) -> SystemConfigurationRedundancyReport {
    let mut report = SystemConfigurationRedundancyReport {
        devices_without_a_self_verified_slot: Vec::new(),
        devices_with_fewer_than_two_self_verified_slots: Vec::new(),
    };
    for (device, slots) in system_configuration_slot_readings(reader) {
        let self_verified_slots = slots
            .iter()
            .filter(|reading| slot_verifies_itself(reading))
            .count();
        if self_verified_slots == 0 {
            report.devices_without_a_self_verified_slot.push(device);
        }
        if self_verified_slots < slots.len() {
            report
                .devices_with_fewer_than_two_self_verified_slots
                .push(device);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    //! 系统配置槽 R、S 越界与格式版本、incompat 同在一槽的分类（C663），根槽与系统配置槽尾部恒 0（C728）：
    //! 两块盘的池按字段表手写四个系统配置槽（与一条第 0 代根），整槽校验和照 checker 自己的算法封上，交给 `walk::check_pool_image`。
    //! 池级 checker 只依赖 `singlefs-format`、没有 dev-dependencies，造不出验证包那种真池，坏镜像在这里按字节写。

    use super::*;

    /// 每块盘的字节数：装得下系统配置两槽、整条根环（区域 2 在 1 MiB + 2 × 3 × 1 MiB 起）与 journal 环（槽 1024 起 1 MiB）。
    const DEVICE_BYTES: usize = 18 << 20;
    /// 两块盘的身份。
    const DEVICES: [u32; 2] = [0, 1];
    /// 这个池的 fsid。
    const POOL_FILESYSTEM_IDENTIFIER: [u8; 16] = [0x5A; 16];
    /// 系统配置槽 1 的偏移（固定结构槽距，第一版 4096，D2（RAID 条带策略） 已定项 19）。
    const SLOT_SPACING_BYTES: u32 = 4096;
    /// 根槽宽（`physical_block_size`）。
    const ROOT_SLOT_BYTES: u32 = 512;
    /// journal 环长：1 MiB，单元区起始槽号 = 1024 + 64 = 1088，落在 64 槽段边界上。
    const JOURNAL_RING_BYTES: u64 = 1 << 20;
    /// mkfs 时的每区槽数（D22（单元原子性怎么合成） 已定项 2 的区间里）。
    const SLOTS_PER_REGION: u8 = 8;
    /// 系统配置字段表里 R、S 两字节的偏移（`geometry_of` 读 `slot[361]`、`slot[362]`）。
    const REGIONS_OFFSET: usize = 361;
    const SLOTS_PER_REGION_OFFSET: usize = 362;
    /// 系统配置几何段单元几何四格的偏移：单元大小、落点粒度、位置条目宽、扩展点声明值 N（用例自己按字段表写一份，不取被测的常量）。
    const UNIT_BYTES_OFFSET: usize = 305;
    const SLOT_GRANULARITY_BYTES_OFFSET: usize = 309;
    const LOCATION_ENTRY_BYTES_OFFSET: usize = 313;
    const EXTENSION_POINT_DECLARED_BYTES_OFFSET: usize = 321;
    /// 系统配置自举头里 incompat 位图第一个字节的偏移（magic 4、格式版本 2 之后）。
    const INCOMPAT_FIRST_BYTE_OFFSET: usize = 6;
    /// 系统配置整槽校验和与根记录自证校验和的偏移（D22（单元原子性怎么合成） 已定项 7 / 已定项 9 的字段表）。
    const FIELD_TABLE_SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
    const FIELD_TABLE_ROOT_CHECKSUM_OFFSET: usize = 138;

    struct DevicesInMemory {
        bytes_of_each_device: BTreeMap<u32, Vec<u8>>,
    }

    impl ImageReader for DevicesInMemory {
        fn devices(&self) -> Vec<u32> {
            self.bytes_of_each_device.keys().copied().collect()
        }
        fn device_bytes(&self, device: u32) -> Option<u64> {
            self.bytes_of_each_device
                .get(&device)
                .map(|bytes| u64::try_from(bytes.len()).expect("18 MiB"))
        }
        fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
            let bytes = self.bytes_of_each_device.get(&device)?;
            let start = usize::try_from(offset).ok()?;
            bytes
                .get(start..start.checked_add(length)?)
                .map(<[u8]>::to_vec)
        }
        fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
            Some(Vec::new())
        }
        fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
            Some(Vec::new())
        }
    }

    fn put_u32(slot: &mut [u8], offset: usize, value: u32) {
        slot[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put_u64(slot: &mut [u8], offset: usize, value: u64) {
        slot[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    /// 校验和字段先清零、对 `cover_end` 之前的字节算 CRC-32C、写进字段的前 4 字节（`checksum_field_holds` 的读法）。
    fn seal(bytes: &mut [u8], checksum_offset: usize) {
        bytes[checksum_offset..checksum_offset + 32].fill(0);
        let checksum = crate::crc32_castagnoli_table(bytes);
        bytes[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
    }

    /// 盘 `device` 上世代号 `slot_generation` 的系统配置槽：每个字段都取第一版读者收的值（偏移按 `.claude/kb/layout/01-first-txn.md` 一的字段表），
    /// 字段表之后全 0，整槽校验和封上。
    fn first_version_system_configuration_slot(device: u32, slot_generation: u64) -> Vec<u8> {
        let mut slot = vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
        slot[..4].copy_from_slice(b"SFSB");
        slot[4..6].copy_from_slice(&1u16.to_le_bytes());
        slot[INCOMPAT_FIRST_BYTE_OFFSET] = 0x06;
        slot[102..118].copy_from_slice(&POOL_FILESYSTEM_IDENTIFIER);
        put_u32(&mut slot, 139, device);
        put_u32(
            &mut slot,
            143,
            u32::try_from(DEVICES.len()).expect("两块盘"),
        );
        put_u64(&mut slot, 147, slot_generation);
        put_u32(
            &mut slot,
            301,
            u32::try_from(singlefs_format::NODE_BYTES).expect("16384"),
        );
        put_u32(
            &mut slot,
            UNIT_BYTES_OFFSET,
            u32::try_from(DATA_UNIT_BYTES).expect("32768"),
        );
        put_u32(
            &mut slot,
            SLOT_GRANULARITY_BYTES_OFFSET,
            u32::try_from(SLOT_BYTES).expect("16384"),
        );
        put_u32(
            &mut slot,
            LOCATION_ENTRY_BYTES_OFFSET,
            u32::try_from(LOC_ENTRY).expect("14"),
        );
        put_u32(&mut slot, 317, ROOT_SLOT_BYTES);
        put_u64(&mut slot, 325, JOURNAL_RING_START_SLOT);
        put_u64(&mut slot, 333, JOURNAL_RING_BYTES);
        let in_flight_record_limit =
            JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;
        put_u32(
            &mut slot,
            345,
            u32::try_from(in_flight_record_limit).expect("85"),
        );
        put_u64(
            &mut slot,
            349,
            in_flight_record_limit * JOURNAL_RECORD_BYTES,
        );
        put_u32(
            &mut slot,
            357,
            u32::try_from(JOURNAL_SAFETY_FACTOR).expect("3"),
        );
        slot[REGIONS_OFFSET] = u8::try_from(ROOT_RING_REGIONS).expect("3");
        slot[SLOTS_PER_REGION_OFFSET] = SLOTS_PER_REGION;
        put_u32(
            &mut slot,
            363,
            u32::try_from(singlefs_format::ROOT_RING_PRIME_STEP).expect("3"),
        );
        put_u32(
            &mut slot,
            367,
            u32::try_from(singlefs_format::ROOT_RING_CHUNK_BYTES).expect("1 MiB"),
        );
        put_u64(&mut slot, 371, ROOT_RING_BASE_SLOT);
        for (region, region_device) in singlefs_format::ROOT_RING_REGION_DEVICES.iter().enumerate()
        {
            put_u32(&mut slot, 379 + 4 * region, *region_device);
        }
        put_u64(
            &mut slot,
            417,
            JOURNAL_RING_START_SLOT + JOURNAL_RING_BYTES.div_ceil(SLOT_BYTES),
        );
        put_u32(&mut slot, 429, SLOT_SPACING_BYTES);
        seal(&mut slot, FIELD_TABLE_SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        slot
    }

    /// 第 0 代根（mkfs 种在区域 0 槽 0 的那一条）：magic、fsid、四条指针全零，槽宽 [`ROOT_SLOT_BYTES`]、根记录之后全 0，自证校验和封上。
    fn genesis_root_slot() -> Vec<u8> {
        let mut slot = vec![0u8; usize::try_from(ROOT_SLOT_BYTES).expect("512")];
        slot[..4].copy_from_slice(b"SFSR");
        slot[4..20].copy_from_slice(&POOL_FILESYSTEM_IDENTIFIER);
        seal(&mut slot, FIELD_TABLE_ROOT_CHECKSUM_OFFSET);
        slot
    }

    /// 两块盘、每盘槽 0（世代 2）与槽 1（世代 1）都是第一版的值，区域 0 槽 0 有一条第 0 代根。
    fn pool_holding_only_first_version_values() -> DevicesInMemory {
        let mut bytes_of_each_device = BTreeMap::new();
        for device in DEVICES {
            let mut bytes = vec![0u8; DEVICE_BYTES];
            let slot_spacing = usize::try_from(SLOT_SPACING_BYTES).expect("4096");
            bytes[..slot_spacing]
                .copy_from_slice(&first_version_system_configuration_slot(device, 2));
            bytes[slot_spacing..2 * slot_spacing]
                .copy_from_slice(&first_version_system_configuration_slot(device, 1));
            bytes_of_each_device.insert(device, bytes);
        }
        let mut pool = DevicesInMemory {
            bytes_of_each_device,
        };
        write_root_slot_at_region_zero_slot_zero(&mut pool, &genesis_root_slot());
        pool
    }

    fn write_root_slot_at_region_zero_slot_zero(pool: &mut DevicesInMemory, root_slot: &[u8]) {
        let offset = usize::try_from(ROOT_RING_BASE_SLOT * SLOT_BYTES).expect("1 MiB");
        let device = singlefs_format::ROOT_RING_REGION_DEVICES[0];
        pool.bytes_of_each_device
            .get_mut(&device)
            .expect("区域 0 的盘在池里")[offset..offset + root_slot.len()]
            .copy_from_slice(root_slot);
    }

    /// 盘 1 槽 0 按 `rewrite` 改、整槽校验和重封（别的三槽照原样）。
    fn rewrite_and_reseal_device_one_slot_zero(
        pool: &mut DevicesInMemory,
        rewrite: impl FnOnce(&mut [u8]),
    ) {
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        let slot = &mut pool.bytes_of_each_device.get_mut(&1).expect("盘 1 在池里")[..slot_bytes];
        rewrite(slot);
        seal(slot, FIELD_TABLE_SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    }

    fn verdict_of_the_invariant_in_the_report(
        report: &[(&'static str, InvariantVerdict)],
        invariant: &str,
    ) -> InvariantVerdict {
        report
            .iter()
            .find(|(name, _)| *name == invariant)
            .map(|(_, verdict)| verdict.clone())
            .unwrap_or_else(|| panic!("报告里有 {invariant}"))
    }

    /// 池级 checker 对这个池只报 I-7.13 违例、其余每一条都报不适用（整池不作保），盘 1 槽 0 读成带 `refused` 那个成员的不收之值。
    fn assert_device_one_slot_zero_refuses_the_whole_pool_naming(
        pool: &DevicesInMemory,
        refused: crate::Verdict,
    ) {
        assert_device_one_slot_zero_refuses_the_whole_pool_as(
            pool,
            &RefusedSystemConfigurationValue::RefusedByTheSlotChecks(refused),
        );
    }

    /// 同 [`assert_device_one_slot_zero_refuses_the_whole_pool_naming`]，盘 1 槽 0 读成带 `refused` 的不收之值（拒在哪一格由调用方给全）。
    fn assert_device_one_slot_zero_refuses_the_whole_pool_as(
        pool: &DevicesInMemory,
        refused: &RefusedSystemConfigurationValue,
    ) {
        let readings = system_configuration_slot_readings(pool);
        assert_eq!(
            &readings[1].1[0],
            &SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes: 0,
                refused: refused.clone(),
            },
            "盘 1 槽 0 自证过、带这一版读者不收的值，点名 {refused:?}，不是这一槽不可择"
        );
        let report = crate::walk::check_pool_image(pool);
        let judged: Vec<&(&str, InvariantVerdict)> = report
            .iter()
            .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
            .collect();
        assert!(
            matches!(
                judged.as_slice(),
                [(
                    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                    InvariantVerdict::Violated(_)
                )]
            ),
            "整池拒：只有 I-7.13 判红、别的每一条都报不适用，实际判了的是 {judged:?}"
        );
    }

    /// 阳性对照：四槽都是第一版的值、尾部全 0，第 0 代根的槽尾全 0——四槽都可择，I-7.13 成立，I-2.4 不红。
    #[test]
    fn pool_holding_only_first_version_values_reads_every_slot_as_usable_and_keeps_header_checksum_coverage(
    ) {
        let pool = pool_holding_only_first_version_values();
        for (device, slots) in system_configuration_slot_readings(&pool) {
            for slot in slots {
                assert!(
                    matches!(slot, SystemConfigurationSlotReading::Usable(..)),
                    "盘 {device} 的槽读成 {slot:?}，该是可择"
                );
            }
        }
        let report = crate::walk::check_pool_image(&pool);
        assert_eq!(
            verdict_of_the_invariant_in_the_report(
                &report,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Holds,
            "四槽都只带第一版的值"
        );
        assert!(
            !matches!(
                verdict_of_the_invariant_in_the_report(&report, "I-2.4"),
                InvariantVerdict::Violated(_)
            ),
            "槽尾全 0 的池上 I-2.4 不红：{:?}",
            verdict_of_the_invariant_in_the_report(&report, "I-2.4")
        );
    }

    /// C1041：盘 1 槽 0 的单元几何四格之一改一个值、重封，读成拒在那一格的不收之值、整池不作保。
    fn assert_one_unit_geometry_field_rewritten_refuses_the_whole_pool(
        offset_in_the_slot: usize,
        rewritten_value: u32,
        expected: UnitGeometryFieldThatDisagrees,
    ) {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| {
            put_u32(slot, offset_in_the_slot, rewritten_value);
        });
        assert_device_one_slot_zero_refuses_the_whole_pool_as(
            &pool,
            &RefusedSystemConfigurationValue::UnitGeometryFieldDisagrees(expected),
        );
    }

    /// C1041 单元大小：盘 1 槽 0 偏移 305 写 65536（格式常量 32768 的两倍）。
    #[test]
    fn slot_declaring_a_unit_size_other_than_the_format_constant_refuses_the_whole_pool() {
        assert_one_unit_geometry_field_rewritten_refuses_the_whole_pool(
            UNIT_BYTES_OFFSET,
            65536,
            UnitGeometryFieldThatDisagrees::UnitBytesNotTheFormatConstant,
        );
    }

    /// C1041 落点粒度：盘 1 槽 0 偏移 309 写 4096（格式常量 16384 之外）。
    #[test]
    fn slot_declaring_a_slot_granularity_other_than_the_format_constant_refuses_the_whole_pool() {
        assert_one_unit_geometry_field_rewritten_refuses_the_whole_pool(
            SLOT_GRANULARITY_BYTES_OFFSET,
            4096,
            UnitGeometryFieldThatDisagrees::SlotGranularityBytesNotTheFormatConstant,
        );
    }

    /// C1041 位置条目宽：盘 1 槽 0 偏移 313 写 15（格式常量 14 加一）。
    #[test]
    fn slot_declaring_a_location_entry_width_other_than_the_format_constant_refuses_the_whole_pool()
    {
        assert_one_unit_geometry_field_rewritten_refuses_the_whole_pool(
            LOCATION_ENTRY_BYTES_OFFSET,
            15,
            UnitGeometryFieldThatDisagrees::LocationEntryBytesNotTheFormatConstant,
        );
    }

    /// C1041 扩展点声明值 N：盘 1 槽 0 偏移 321 写 1（第一版只收 0）。
    #[test]
    fn slot_declaring_a_non_zero_extension_point_size_refuses_the_whole_pool() {
        assert_one_unit_geometry_field_rewritten_refuses_the_whole_pool(
            EXTENSION_POINT_DECLARED_BYTES_OFFSET,
            1,
            UnitGeometryFieldThatDisagrees::ExtensionPointDeclaredBytesNotTheFirstVersionValue,
        );
    }

    /// C663 S：盘 1 槽 0 的每区槽数写 17（区间 [4, 16] 之外）、重封——D22（单元原子性怎么合成） 已定项 2「越界整池拒绝挂载」，
    /// 不再是「这一槽不可择」、别的槽照择。
    #[test]
    fn slot_declaring_seventeen_slots_per_region_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| {
            slot[SLOTS_PER_REGION_OFFSET] = 17
        });
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval,
        );
    }

    /// C663 R：盘 1 槽 0 的区域数写 2（与第一版常量 3 不等）、重封。
    #[test]
    fn slot_declaring_two_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 2);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountBelowTheFirstVersionConstant,
        );
    }

    /// C663 R：区域数写 0，报 0 那一头自己的成员，同一个处置（整池拒）。
    #[test]
    fn slot_declaring_zero_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 0);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountZero,
        );
    }

    /// C663 R：区域数写 4（比字段表留的三个逐区域设备字段多一个），报大于 3 那一头自己的成员，同一个处置（整池拒）。
    #[test]
    fn slot_declaring_four_regions_refuses_the_whole_pool() {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[REGIONS_OFFSET] = 4);
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::RegionCountPastTheRegionDeviceFields,
        );
    }

    /// C663 次序：盘 1 槽 0 格式版本写 2、incompat 第一个字节写 0x03（不是这一版布局的 0x06，不认识）、重封——两样同在一槽时先判格式版本
    /// 读成带不收之值、整池拒，不是 incompat 不认得那一支的「这一槽不可择」。
    #[test]
    fn slot_with_an_unrecognized_format_version_and_an_unknown_incompat_bit_refuses_the_whole_pool_by_its_format_version(
    ) {
        let mut pool = pool_holding_only_first_version_values();
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| {
            slot[4..6].copy_from_slice(&2u16.to_le_bytes());
            slot[INCOMPAT_FIRST_BYTE_OFFSET] = 0x03;
        });
        assert_device_one_slot_zero_refuses_the_whole_pool_naming(
            &pool,
            crate::Verdict::FormatVersionNotRecognized,
        );
    }

    /// C728 系统配置槽尾：盘 1 槽 0 字段表之后的头一个字节（偏移 489）写 1、重封——整槽校验和罩着它、core 不读它，
    /// 池级 checker 判 I-2.4 红、点名那个偏移；它不是读者不收之值，I-7.13 照旧成立。
    #[test]
    fn non_zero_byte_right_past_the_system_configuration_field_table_reddens_header_checksum_coverage(
    ) {
        let mut pool = pool_holding_only_first_version_values();
        let field_table_end = usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("489");
        rewrite_and_reseal_device_one_slot_zero(&mut pool, |slot| slot[field_table_end] = 1);
        let report = crate::walk::check_pool_image(&pool);
        let InvariantVerdict::Violated(detail) =
            verdict_of_the_invariant_in_the_report(&report, "I-2.4")
        else {
            panic!(
                "字段表之后有非 0 字节：I-2.4 该红，实际 {:?}",
                verdict_of_the_invariant_in_the_report(&report, "I-2.4")
            );
        };
        assert!(
            detail.contains("系统配置槽") && detail.contains(&format!("偏移 {field_table_end} ")),
            "违例点名系统配置槽偏移 {field_table_end}：{detail}"
        );
        assert_eq!(
            verdict_of_the_invariant_in_the_report(
                &report,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Holds,
            "槽尾非 0 不是读者不收之值，不整池拒"
        );
    }

    /// C728 根槽尾：第 0 代根的根记录之后头一个字节（偏移 466）写 1、重封自证校验和——池级 checker 判 I-2.4 红、点名那个偏移。
    #[test]
    fn non_zero_byte_right_past_the_root_record_reddens_header_checksum_coverage() {
        let mut pool = pool_holding_only_first_version_values();
        let record_end = usize::try_from(ROOT_RECORD_BYTES).expect("466");
        let mut root_slot = genesis_root_slot();
        root_slot[record_end] = 1;
        seal(&mut root_slot, FIELD_TABLE_ROOT_CHECKSUM_OFFSET);
        write_root_slot_at_region_zero_slot_zero(&mut pool, &root_slot);
        let report = crate::walk::check_pool_image(&pool);
        let InvariantVerdict::Violated(detail) =
            verdict_of_the_invariant_in_the_report(&report, "I-2.4")
        else {
            panic!(
                "根记录之后有非 0 字节：I-2.4 该红，实际 {:?}",
                verdict_of_the_invariant_in_the_report(&report, "I-2.4")
            );
        };
        assert!(
            detail.contains("根槽偏移 466 "),
            "违例点名根槽偏移 466：{detail}"
        );
    }

    /// 在飞上限那 4 字节一格（`.claude/kb/layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」）：环长 `3 << 44` 字节时
    /// 在飞上限 = 2³²，装不进 `u32`，环长判据判假；小一个 12288 字节（F 条记录）时在飞上限 = 2³² − 1，这一格不拒。
    /// 两个环长的单元区起点都照环长现算（同一个槽号 1024 + 3 × 2³⁰），环末端都不越过它——这一对取样点上只有装不装得进 4 字节那一格分得开。
    #[test]
    fn journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_lies_outside_the_supported_range(
    ) {
        let overflowing_journal_ring_bytes: u64 = 3 << 44;
        let largest_fitting_journal_ring_bytes =
            overflowing_journal_ring_bytes - JOURNAL_SAFETY_FACTOR * JOURNAL_RECORD_BYTES;
        assert_eq!(
            overflowing_journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR,
            u64::from(u32::MAX) + 1,
            "取样点在 4 字节一格的边界上：大的那个环长的在飞上限恰好比 u32::MAX 多 1"
        );
        for journal_ring_bytes in [
            overflowing_journal_ring_bytes,
            largest_fitting_journal_ring_bytes,
        ] {
            let unit_area_start_slot =
                JOURNAL_RING_START_SLOT + journal_ring_bytes.div_ceil(SLOT_BYTES);
            assert_eq!(
                unit_area_start_slot,
                JOURNAL_RING_START_SLOT + (3 << 30),
                "环长 {journal_ring_bytes} 现算的单元区起点"
            );
            assert_eq!(
                journal_ring_bytes_lie_in_the_supported_range(
                    JOURNAL_RING_START_SLOT,
                    journal_ring_bytes,
                    unit_area_start_slot,
                ),
                journal_ring_bytes == largest_fitting_journal_ring_bytes,
                "环长 {journal_ring_bytes}：在飞上限 {} 装得进 4 字节才过环长判据",
                journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
            );
        }
    }
}
