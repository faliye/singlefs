//! 择系统配置：一个槽读成什么、这一版读者认得的取值、几何字段与环的跨度、按格式许可的槽距找第二个槽，
//! 择到的那一槽的盘上原样，每块盘两槽各读一次与系统配置见证判不判得出。

use super::*;

/// 一个系统配置槽读回来之后可择不可择（[`choose_system_configuration`] 按它分流）。封闭集合，`match` 不写通配臂。
enum SystemConfigurationSlotReading {
    /// 三关都过、S 在区间里：这一槽可择。
    Mountable(SystemConfiguration),
    /// 读不到，或 magic / 整槽校验和不过，或节点大小不是格式常量（当自证不过，C840）：这一槽不可择，换一槽换一盘还可以试。
    NotSelfDescribing,
    /// magic 与整槽校验和都过、incompat 位图不认识：这一槽同样不可择，记下它，池里一份可择的都没有时报布局不认识。
    IncompatBitsNotRecognized(IncompatBitmap),
}

impl SystemConfigurationSlotReading {
    fn into_mountable(self) -> Option<SystemConfiguration> {
        match self {
            Self::Mountable(system_configuration) => Some(system_configuration),
            Self::NotSelfDescribing | Self::IncompatBitsNotRecognized(_) => None,
        }
    }

    fn incompat_bitmap_not_recognized(&self) -> Option<IncompatBitmap> {
        match self {
            Self::IncompatBitsNotRecognized(incompat_bitmap) => Some(*incompat_bitmap),
            Self::Mountable(_) | Self::NotSelfDescribing => None,
        }
    }

    /// 可择的这一槽记的固定结构槽距；自证不过或布局不认识的槽里记的不能信。
    fn recorded_fixed_structure_slot_spacing(&self) -> Option<u64> {
        match self {
            Self::Mountable(system_configuration) => Some(u64::from(
                system_configuration
                    .immutable
                    .sizes
                    .fixed_structure_slot_spacing,
            )),
            Self::NotSelfDescribing | Self::IncompatBitsNotRecognized(_) => None,
        }
    }
}

/// 系统配置里这个读者不收的那一个池级不可变字段与它的值（[`RecoveryFailure::SystemConfigurationValueRefused`]）。封闭集合，`match` 不写通配臂。
/// 上下界取 mkfs 判几何用的同一组条款（`make_filesystem::check_geometry`），读者这一侧的判法条款没有写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemConfigurationValueOutsideWhatThisReaderAccepts {
    /// 格式版本不是这个读者写的那一版（`system_configuration::FORMAT_VERSION`）。
    FormatVersionNotRecognized { format_version: u16 },
    /// 加密类型不是 0（关）：加密不进第一个可运行版本（D9（加密） 已定项 10），1、2 是登记过的算法、别的码不认识，都不收。
    EncryptionTypeNotOff { encryption_type: u8 },
    /// 同一槽记着的单元大小（偏移 305 的 4 字节）不是格式常量 `DATA_UNIT_BYTES`（32768）：core 读写单元按格式常量走，字段与常量不等时
    /// 两边说的不是同一个单元几何（D21（权威态与派生态的分界） 已定项 16「单元几何在挂载时求值一次」）。几何四格（单元大小、落点粒度、
    /// 位置条目宽、扩展点声明值 N）排在加密类型之后、槽距之前，按字段表次序判，第一个不等的报出来（[`unit_geometry_fields_of_the_format`]）。
    UnitBytesNotTheFormatConstant {
        recorded_unit_bytes: u32,
        format_unit_bytes: u64,
    },
    /// 同一槽记着的落点粒度（偏移 309 的 4 字节）不是格式常量 `SLOT_BYTES`（16384），处置同 [`Self::UnitBytesNotTheFormatConstant`]。
    SlotGranularityBytesNotTheFormatConstant {
        recorded_slot_granularity_bytes: u32,
        format_slot_granularity_bytes: u64,
    },
    /// 同一槽记着的位置条目宽（偏移 313 的 4 字节）不是格式常量 `LOC_ENTRY`（14），处置同 [`Self::UnitBytesNotTheFormatConstant`]。
    LocationEntryBytesNotTheFormatConstant {
        recorded_location_entry_bytes: u32,
        format_location_entry_bytes: u64,
    },
    /// 同一槽记着的扩展点声明值 N（偏移 321 的 4 字节）不是第一版写的 0
    /// （`system_configuration::EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION`，D21（权威态与派生态的分界） 已定项 8），
    /// 处置同 [`Self::UnitBytesNotTheFormatConstant`]。
    ExtensionPointDeclaredBytesNotTheFirstVersionValue {
        recorded_extension_point_declared_bytes: u32,
        first_version_extension_point_declared_bytes: u32,
    },
    /// 固定结构槽距落在格式允许的区间之外：不小于 4096、槽 1 整槽落在同一槽自述的根环起点之前（D22（单元原子性怎么合成） 已定项 16
    /// 第 5 句；根环起点字段 × 16384 − 4096 是上界，[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]；根环起点不到一个
    /// 系统配置槽宽时一档槽距都不收）。槽距是根槽宽的上界，这一条先于根槽宽判；也先于根环起点等不等第一版常量那一判，
    /// 根环起点落在槽 1 之前的槽报在这里（与 checker 的 I-7.13 报同一格）。
    FixedStructureSlotSpacingOutsideTheFormatRange { fixed_structure_slot_spacing: u32 },
    /// `physical_block_size`（根槽宽，D22（单元原子性怎么合成） 已定项 2）装不下根记录（466 字节，已定项 7），或大于固定结构槽距
    /// （槽 j 在区域起点 + j × 槽距，已定项 16）。读根槽按它开缓冲，不设上界时一个盘上的 4 字节就定得了每一次读的内存。
    PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: u32 },
    /// 同一槽记着的根环起点（偏移 371 的 8 字节，16 KiB 槽号）不是第一版写的那个（64 = 1 MiB，
    /// `system_configuration::ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION`）：core 读写根环按格式常量走（`root_ring::region_start`），
    /// 字段与常量不等时两边读的不是同一个环；条款定的是按字段读、第一版只收常量，不等整池拒（D22（单元原子性怎么合成） 已定项 16 第 1 句，
    /// 用户 2026-09-27 定「读字段，不等于常量就整池拒」；代码三方 m2-closeout-code-r2「core 与 checker 对系统配置几何字段的读法」）。
    RootRingBaseNotTheFirstVersionSlot {
        recorded_root_ring_base_slot: SlotNumber,
        first_version_root_ring_base_slot: SlotNumber,
    },
    /// 同一槽记着的 journal 环起点（偏移 325 的 8 字节，16 KiB 槽号）不是第一版写的那个（1024，
    /// `system_configuration::JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION`）：core 读写 journal 环按格式常量走，处置同
    /// [`Self::RootRingBaseNotTheFirstVersionSlot`]（同一条款、同一次用户定案）。
    JournalRingStartNotTheFirstVersionSlot {
        recorded_journal_ring_start_slot: SlotNumber,
        first_version_journal_ring_start_slot: SlotNumber,
    },
    /// 同一槽记着的根环区域数 R（偏移 361 那 1 字节）不是第一版写的那个（`ROOT_RING_REGIONS`，3）：core 读写根环按格式常量 R 走
    /// （择根、`root_ring::target_for_publish` 按 txg 对 R 取模），字段与常量不等时两边读的不是同一个环。处置与根环起点、journal 环起点同一类，
    /// 不等整池拒（C663（系统配置槽越界与不收之值两读者分类不同），用户 2026-10-20 定「S 照条款整池拒，R 读并整池拒」）。
    RootRingRegionCountNotTheFirstVersionValue {
        recorded_root_ring_region_count: u8,
        first_version_root_ring_region_count: u64,
    },
    /// 同一槽记着的根环区域步长 P（偏移 363 的 4 字节）不是第一版常量（`ROOT_RING_PRIME_STEP`，3）：core 按格式常量放区域
    /// （`root_ring::region_start`），字段与常量不等时两边读的不是同一个环。不等整池拒（D22（单元原子性怎么合成） 已定项 16 第 1 句、
    /// I-7.13，m3-close-c506 三轮判决、用户 2026-10-22 定「四件照写回并实现」）。排在区域数 R 之后、环长之前。
    RootRingPrimeStepNotTheFirstVersionValue {
        recorded_root_ring_prime_step: u32,
        first_version_root_ring_prime_step: u64,
    },
    /// 同一槽记着的根环 chunk（偏移 367 的 4 字节）不是第一版常量（`ROOT_RING_CHUNK_BYTES`，1 MiB），处置同
    /// [`Self::RootRingPrimeStepNotTheFirstVersionValue`]（同一句条款）。
    RootRingChunkBytesNotTheFirstVersionValue {
        recorded_root_ring_chunk_bytes: u32,
        first_version_root_ring_chunk_bytes: u64,
    },
    /// P 与 chunk 都等于第一版常量，而 S × 固定结构槽距大于区域间距 P × chunk（3 MiB）：一个区域的最后几槽落进下一个区域
    /// （D22（单元原子性怎么合成） 已定项 16 第 1 句「写 ≤ 收」、I-7.13；mkfs 判固定结构两两不重叠拒同一形，首尾相接收）。
    /// S 本身在 [4, 16] 里（越界先报 [`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]），槽距本身在格式允许的区间里（先报
    /// [`Self::FixedStructureSlotSpacingOutsideTheFormatRange`]）。
    RootRingRegionLongerThanTheRegionStride {
        slots_per_region: u64,
        fixed_structure_slot_spacing: u32,
        region_stride_bytes: u64,
    },
    /// journal 环长装不下 F 条记录（在飞上限 = 环槽数 ÷ F 是 0，D23（journal 的角色与格式） 已定项 18），在飞上限装不进系统配置里它那 4 字节
    /// （mkfs 的 `JournalInFlightRecordLimitWiderThanItsFourByteField`；不拒的话挂载之后轮换系统配置槽时写这 4 字节那一句 panic），
    /// 或环的末端越过同一槽记着的单元区起点（偏移 417 的 8 字节；与 checker 同一个判法，C475（非默认环长下单元区起点取编译期常量）之后
    /// 单元区起点随环长走、不再有编译期的上界）。写记录按它取模，不设界时一个盘上的 8 字节就定得了算术溢不溢出；扫环逐槽列偏移只列到
    /// 那块盘的末尾（[`scan_journal`]）。
    JournalRingBytesOutsideTheSupportedRange { journal_ring_bytes: u64 },
    /// 同一槽记着的单元区起始槽号（偏移 417 的 8 字节）不是环长现算的那个（journal 环末尾的下一个槽，D3（空间分配） 已定项 10 ④
    /// 「第一版 = journal 环末尾的下一个槽」）：挂载建空闲图、恢复判分配记录落点都按环长现算的起点走（`allocator::UnitAreaStart`，
    /// 由环长算只有 `journal::slot_after_the_journal_ring` 一处），两个数不一样时按哪一个条款没写，第一版不收。
    UnitAreaStartNotTheSlotAfterTheJournalRing {
        recorded_unit_area_start_slot: SlotNumber,
        slot_after_the_journal_ring: SlotNumber,
    },
    /// 单元区起点（环长现算、与偏移 417 那 8 字节相等）不落在 64 槽聚簇段边界上：这样的单元区怎么分段条款没写，第一版不支持
    /// （`allocator::UnitAreaStart` 的不变量；mkfs 同一格拒成 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`）。
    UnitAreaStartOffTheClusterSegmentBoundaryUnsupported { unit_area_start_slot: SlotNumber },
    /// 同一槽记着的安全系数 F（偏移 357 的 4 字节）不是这个读者写的那个（`singlefs_format::JOURNAL_SAFETY_FACTOR`，第一版 3）。
    /// I-8.1（环几何够大） 的三条等式之一，core 与 checker 同判（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行）：
    /// 恢复取前缀、发布切记录都按格式常量算在飞上限，盘上的 F 与它不等时两边说的不是同一个环几何。F ≥ 2 那一半落在格式常量上
    /// （`system_configuration` 里的编译期断言）。
    JournalSafetyFactorNotTheFirstVersionValue {
        recorded_safety_factor: u32,
        first_version_safety_factor: u64,
    },
    /// 同一槽记着的在飞记录数上限（偏移 345 的 4 字节）不等于按同一槽的环长现算的 ⌊环长 ÷ 4096 ÷ F⌋
    /// （`system_configuration::journal_in_flight_record_limit`；I-8.1（环几何够大））。
    JournalInFlightRecordLimitNotRecomputedFromTheRingLength {
        recorded_in_flight_record_limit: u32,
        recomputed_in_flight_record_limit: u64,
    },
    /// 同一槽记着的最坏占用（偏移 349 的 8 字节）不等于现算的在飞记录数上限 × 4096（I-8.1（环几何够大））。
    JournalWorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes {
        recorded_worst_case_occupancy_bytes: u64,
        recomputed_worst_case_occupancy_bytes: u64,
    },
}

/// 系统配置槽里格式版本那 2 字节的偏移：紧跟 magic（`system_configuration::SystemConfiguration::to_slot` 的写法）。
const SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET: usize =
    crate::system_configuration::SYSTEM_CONFIGURATION_MAGIC.len();
/// 系统配置槽里加密类型那 1 字节的偏移：整槽校验和 32 之后是 MAC 16、nonce 水位 12、KDF 标识 4，再是它（字段表偏移 219）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize =
    crate::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32 + 16 + 12 + 4;
/// 加密类型登记表里的「关」（未加密，D9（加密） 已定项 10；D22（单元原子性怎么合成） 已定项 17）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF: u8 = 0;

/// 格式允许的最大固定结构槽距：槽 1 整槽落在第一版根环基址（区域 0 的起点）之前。逐档找槽 1 用它：那时一槽可择的都没有，
/// 没有哪一槽自述的根环起点可读。读者判一槽的槽距用那一槽自述的根环起点（[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]）。
fn largest_fixed_structure_slot_spacing_the_format_allows() -> u64 {
    region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES
}

/// 一槽系统配置里记着的格式版本（紧跟 magic 的 2 字节，小端）。只切字节、不判值。
///
/// # Panics
/// 槽短于两字节加 magic：调用方读的是整槽 4096 字节，magic 与整槽校验和那两关过了才调这里。
fn format_version_recorded_in_the_slot(slot: &[u8]) -> u16 {
    u16::from_le_bytes([
        slot[SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET],
        slot[SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET + 1],
    ])
}

/// 择系统配置时收哪个池的槽（[`mountable_slot_or_refusal`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotsOfWhichPool {
    /// 读者手里没有本池的 fsid（恢复、只读挂载、用例与装置）：各盘择到的 fsid 对不上整池拒（[`RecoveryFailure::SystemConfigurationsDisagree`]）。
    AnyPool,
    /// 调用方给了本池的 fsid（可写挂载、离线出路）：自述别的 fsid 的槽当自证不过，择这块盘的另一槽（S3，C707）。
    OnlyThePool([u8; 16]),
}

/// 一槽自述的根环起点下格式允许的最大固定结构槽距：槽 1 整槽落在根环基址（该字段 × 16384）之前（D22（单元原子性怎么合成） 已定项 16
/// 第 1、5 句）。根环起点是盘上读来的 8 字节：乘出来溢出时上界取 `u64::MAX`（基址在任何设备偏移之外，槽 1 落在它之前）；
/// 基址不到一个系统配置槽宽时一档都不收（`None`）。
fn largest_fixed_structure_slot_spacing_under_the_root_ring_base(
    root_ring_base_slot: SlotNumber,
) -> Option<u64> {
    root_ring_base_slot.0.checked_mul(SLOT_BYTES).map_or(
        Some(u64::MAX),
        |root_ring_base_in_bytes| {
            root_ring_base_in_bytes.checked_sub(SYSTEM_CONFIGURATION_SLOT_BYTES)
        },
    )
}

/// 一槽自证得过的系统配置，逐个判这个读者收不收它的池级不可变字段（[`SystemConfigurationValueOutsideWhatThisReaderAccepts`] 各成员的出处）。
/// 次序：格式版本、加密类型、单元几何四格（[`unit_geometry_fields_of_the_format`]）、槽距、根槽宽、根环起点、journal 环起点、根环区域数 R、根环区域步长 P、chunk、S × 槽距（[`root_ring_regions_fit_their_stride`]）、
/// 环长、单元区起点——根槽宽的上界是槽距，所以槽距先判；
/// 槽距的上界取同一槽自述的根环起点，所以根环起点落在槽 1 之前的那一格报成槽距越界、与 checker 的 I-7.13 同一格，之后才判它等不等
/// 第一版常量；环末端对单元区起点那一判先于起点是不是环长现算的那个，与 checker 的环长判法（`journal_ring_bytes_lie_in_the_supported_range`）报同一格。
fn system_configuration_values_this_reader_accepts(
    slot: &[u8],
    system_configuration: &SystemConfiguration,
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let format_version = format_version_recorded_in_the_slot(slot);
    if format_version != crate::system_configuration::FORMAT_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
                format_version,
            },
        );
    }
    let encryption_type = slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET];
    if encryption_type != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
                encryption_type,
            },
        );
    }
    unit_geometry_fields_of_the_format(slot)?;
    let sizes = &system_configuration.immutable.sizes;
    let fixed_structure_slot_spacing = u64::from(sizes.fixed_structure_slot_spacing);
    if fixed_structure_slot_spacing < FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        || largest_fixed_structure_slot_spacing_under_the_root_ring_base(
            system_configuration.immutable.root_ring_base_slot,
        )
        .is_none_or(|largest_spacing| fixed_structure_slot_spacing > largest_spacing)
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
                fixed_structure_slot_spacing: sizes.fixed_structure_slot_spacing,
            },
        );
    }
    if u64::from(sizes.physical_block_size) < ROOT_RECORD_BYTES
        || sizes.physical_block_size > sizes.fixed_structure_slot_spacing
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds {
                physical_block_size: sizes.physical_block_size,
            },
        );
    }
    let recorded_root_ring_base_slot = system_configuration.immutable.root_ring_base_slot;
    if recorded_root_ring_base_slot != ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot {
                recorded_root_ring_base_slot,
                first_version_root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },
        );
    }
    let recorded_journal_ring_start_slot = system_configuration.immutable.journal_ring_start_slot;
    if recorded_journal_ring_start_slot != JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingStartNotTheFirstVersionSlot {
                recorded_journal_ring_start_slot,
                first_version_journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
            },
        );
    }
    let recorded_root_ring_region_count = root_ring_region_count_recorded_in_the_slot(slot);
    if u64::from(recorded_root_ring_region_count) != ROOT_RING_REGIONS {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingRegionCountNotTheFirstVersionValue {
                recorded_root_ring_region_count,
                first_version_root_ring_region_count: ROOT_RING_REGIONS,
            },
        );
    }
    root_ring_regions_fit_their_stride(slot, sizes)?;
    let recorded_unit_area_start_slot = unit_area_start_slot_recorded_in_the_slot(slot);
    // 环末端 = 环起点 + 环长；单元区起点 × 16384。两个都是盘上读来的数：乘加溢出时环末端算越界，起点乘出来溢出时它在任何设备偏移之外。
    let journal_ring_end_in_bytes =
        (JOURNAL_RING_START_SLOT * SLOT_BYTES).checked_add(sizes.journal_ring_bytes);
    let journal_ring_ends_by_the_recorded_unit_area_start =
        journal_ring_end_in_bytes.is_some_and(|journal_ring_end| {
            recorded_unit_area_start_slot
                .0
                .checked_mul(SLOT_BYTES)
                .is_none_or(|unit_area_start| journal_ring_end <= unit_area_start)
        });
    if journal_in_flight_record_limit(sizes.journal_ring_bytes) == 0
        || !journal_in_flight_record_limit_fits_its_four_byte_field(sizes.journal_ring_bytes)
        || !journal_ring_ends_by_the_recorded_unit_area_start
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange {
                journal_ring_bytes: sizes.journal_ring_bytes,
            },
        );
    }
    let slot_after_the_ring = slot_after_the_journal_ring(sizes.journal_ring_bytes);
    if recorded_unit_area_start_slot != slot_after_the_ring {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartNotTheSlotAfterTheJournalRing {
                recorded_unit_area_start_slot,
                slot_after_the_journal_ring: slot_after_the_ring,
            },
        );
    }
    UnitAreaStart::following_the_journal_ring(sizes.journal_ring_bytes).map_err(|off_the_boundary| {
        SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
            unit_area_start_slot: off_the_boundary.slot_after_the_journal_ring,
        }
    })?;
    journal_ring_geometry_fields_recompute_from_the_ring_length(slot, sizes.journal_ring_bytes)
}

/// 单元几何四格（D21（权威态与派生态的分界） 已定项 16、已定项 8；C1041）：单元大小、落点粒度、位置条目宽各等于格式常量，扩展点声明值 N
/// 等于第一版的 0，按字段表次序判，第一个不等的报出来。排在加密类型之后、槽距之前（[`system_configuration_values_this_reader_accepts`]）。
fn unit_geometry_fields_of_the_format(
    slot: &[u8],
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let recorded = unit_geometry_fields_recorded_in_the_slot(slot);
    if u64::from(recorded.unit_bytes) != DATA_UNIT_BYTES {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitBytesNotTheFormatConstant {
                recorded_unit_bytes: recorded.unit_bytes,
                format_unit_bytes: DATA_UNIT_BYTES,
            },
        );
    }
    if u64::from(recorded.slot_granularity_bytes) != SLOT_BYTES {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::SlotGranularityBytesNotTheFormatConstant {
                recorded_slot_granularity_bytes: recorded.slot_granularity_bytes,
                format_slot_granularity_bytes: SLOT_BYTES,
            },
        );
    }
    if u64::from(recorded.location_entry_bytes) != LOC_ENTRY {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::LocationEntryBytesNotTheFormatConstant {
                recorded_location_entry_bytes: recorded.location_entry_bytes,
                format_location_entry_bytes: LOC_ENTRY,
            },
        );
    }
    if recorded.extension_point_declared_bytes
        != EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::ExtensionPointDeclaredBytesNotTheFirstVersionValue {
                recorded_extension_point_declared_bytes: recorded.extension_point_declared_bytes,
                first_version_extension_point_declared_bytes:
                    EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION,
            },
        );
    }
    Ok(())
}

/// 根环区域间距那三判（D22（单元原子性怎么合成） 已定项 16 第 1 句、I-7.13，用户 2026-10-22 定）：P、chunk 先判等于第一版常量，
/// 再判 S × 固定结构槽距 ≤ P × chunk（写 ≤ 收，首尾相接的区域收）。P × chunk 取格式常量：前两判过了，盘上那两个字段与它们相等。
/// 排在区域数 R 之后、环长之前（[`system_configuration_values_this_reader_accepts`]）；S × 槽距不溢出：S ≤ 16，槽距是 4 字节。
fn root_ring_regions_fit_their_stride(
    slot: &[u8],
    sizes: &SystemImmutableSizes,
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let recorded = root_ring_region_stride_fields_recorded_in_the_slot(slot);
    if u64::from(recorded.prime_step) != ROOT_RING_PRIME_STEP {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingPrimeStepNotTheFirstVersionValue {
                recorded_root_ring_prime_step: recorded.prime_step,
                first_version_root_ring_prime_step: ROOT_RING_PRIME_STEP,
            },
        );
    }
    if u64::from(recorded.chunk_bytes) != ROOT_RING_CHUNK_BYTES {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingChunkBytesNotTheFirstVersionValue {
                recorded_root_ring_chunk_bytes: recorded.chunk_bytes,
                first_version_root_ring_chunk_bytes: ROOT_RING_CHUNK_BYTES,
            },
        );
    }
    let region_stride_bytes = ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES;
    let slots_per_region = sizes.root_ring_slots_per_region.count();
    if slots_per_region * u64::from(sizes.fixed_structure_slot_spacing) > region_stride_bytes {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingRegionLongerThanTheRegionStride {
                slots_per_region,
                fixed_structure_slot_spacing: sizes.fixed_structure_slot_spacing,
                region_stride_bytes,
            },
        );
    }
    Ok(())
}

/// I-8.1（环几何够大） 的三条等式，挂载读系统配置时复算（core 同判，`research/prompts/m3-inv-a-r1-main-verification.md`
/// 第二节第 4 行、第三节）：F = 格式常量、在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096，按这个次序判，第一个不等的报出来。
/// 放在 [`system_configuration_values_this_reader_accepts`] 的最后：环长与单元区起点不收的槽先按那几格报，这三个字段是环长的函数，
/// 环长本身不收时比它们没有意义。
fn journal_ring_geometry_fields_recompute_from_the_ring_length(
    slot: &[u8],
    journal_ring_bytes: u64,
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let recorded = journal_ring_geometry_fields_recorded_in_the_slot(slot);
    if u64::from(recorded.safety_factor) != JOURNAL_SAFETY_FACTOR {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalSafetyFactorNotTheFirstVersionValue {
                recorded_safety_factor: recorded.safety_factor,
                first_version_safety_factor: JOURNAL_SAFETY_FACTOR,
            },
        );
    }
    let recomputed_in_flight_record_limit = journal_in_flight_record_limit(journal_ring_bytes);
    if u64::from(recorded.in_flight_record_limit) != recomputed_in_flight_record_limit {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalInFlightRecordLimitNotRecomputedFromTheRingLength {
                recorded_in_flight_record_limit: recorded.in_flight_record_limit,
                recomputed_in_flight_record_limit,
            },
        );
    }
    let recomputed_worst_case_occupancy_bytes =
        recomputed_in_flight_record_limit * JOURNAL_RECORD_BYTES;
    if recorded.worst_case_occupancy_bytes != recomputed_worst_case_occupancy_bytes {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalWorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes {
                recorded_worst_case_occupancy_bytes: recorded.worst_case_occupancy_bytes,
                recomputed_worst_case_occupancy_bytes,
            },
        );
    }
    Ok(())
}

/// 择到的系统配置下的单元区起点：journal 环末尾的下一个槽（`allocator::UnitAreaStart::following_the_journal_ring`）。
/// 挂载建空闲图、判分配记录与被抛弃根的落点在不在单元区里，都按它（C475（非默认环长下单元区起点取编译期常量））。
///
/// # Panics
/// 环长算出的起点不在聚簇段边界上：`system_configuration` 必须是 [`choose_system_configuration`] 交回的——择的时候
/// `system_configuration_values_this_reader_accepts` 判过起点在段边界上、且与槽里偏移 417 那 8 字节相等。
pub(crate) fn unit_area_start_of_the_chosen_system_configuration(
    system_configuration: &SystemConfiguration,
) -> UnitAreaStart {
    UnitAreaStart::following_the_journal_ring(system_configuration.immutable.sizes.journal_ring_bytes)
        .expect("择系统配置时 system_configuration_values_this_reader_accepts 判过环长算出的单元区起点在聚簇段边界上")
}

/// 这个池的单元区起点：先择系统配置（[`choose_system_configuration`]），再按它的环长算。给不带单元区起点的那几个公开入口
/// （[`rebuild_version`]、[`walk_to_file`]、[`allocation_records_under_root`]、[`allocation_records_of_version_without_file`]）用：
/// 它们的调用方多是用例与实验装置，手里没有择好的系统配置；挂载与冷走读手里有，走带起点的那一份，不多读系统配置槽。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）。
pub(super) fn unit_area_start_of_the_pool(
    reader: &dyn PoolReader,
) -> Result<UnitAreaStart, RecoveryFailure> {
    choose_system_configuration(reader).map(|system_configuration| {
        unit_area_start_of_the_chosen_system_configuration(&system_configuration)
    })
}

/// 一个槽读回来之后分五路：读不到 / 自证不过 ⇒ 不可择（换一槽换一盘还可以试）；自证得过、incompat 位图不认识 ⇒ 同样不可择、
/// 另记一笔；自述的每区槽数 S 越界、或别的池级不可变字段这个读者不收 ⇒ 整池拒绝挂载，把点名的成员交回去；都过 ⇒ 可择。
///
/// 单拎成一个函数是因为 [`choose_system_configuration`] 每块盘要做两次（槽 0、槽 1），
/// 而两次的分流必须一模一样：其中一次把越界悄悄当成「这一槽不可择」，越界的池就会靠另一槽挂上去。
///
/// 先于这五路两判：调用方给了本池的 fsid（[`SlotsOfWhichPool::OnlyThePool`]）时，magic 与整槽校验和都过而自述的 fsid 不是本池的槽当自证不过
/// （S3，C707，用户 2026-10-19 定「先按 fsid 筛，算自证不过」）；incompat 位图不认识的槽格式版本又不是这个读者写的那一版时，按格式版本整池拒
/// （C663，用户 2026-10-20 定「格式版本先」）。
fn mountable_slot_or_refusal(
    device: DeviceIdentity,
    bytes: Option<Vec<u8>>,
    slots_of_which_pool: SlotsOfWhichPool,
) -> Result<SystemConfigurationSlotReading, RecoveryFailure> {
    let Some(bytes) = bytes else {
        return Ok(SystemConfigurationSlotReading::NotSelfDescribing);
    };
    match slots_of_which_pool {
        SlotsOfWhichPool::AnyPool => {}
        SlotsOfWhichPool::OnlyThePool(filesystem_identifier_of_the_pool) => {
            if filesystem_identifier_self_described_by_the_slot(&bytes)
                .is_some_and(|self_described| self_described != filesystem_identifier_of_the_pool)
            {
                return Ok(SystemConfigurationSlotReading::NotSelfDescribing);
            }
        }
    }
    match SystemConfiguration::parse_slot(&bytes) {
        Ok(system_configuration) => {
            system_configuration_values_this_reader_accepts(&bytes, &system_configuration)
                .map_err(|value| RecoveryFailure::SystemConfigurationValueRefused {
                    device,
                    value,
                })?;
            Ok(SystemConfigurationSlotReading::Mountable(
                system_configuration,
            ))
        }
        // 节点大小不是格式常量的槽当自证不过（C840，用户 2026-10-15 定）：与整槽校验和不过同一路，换一槽换一盘还可以试。
        Err(
            SystemConfigurationSlotRefusal::NotSelfDescribing
            | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
        ) => Ok(SystemConfigurationSlotReading::NotSelfDescribing),
        Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { incompat_bitmap }) => {
            let format_version = format_version_recorded_in_the_slot(&bytes);
            if crate::system_configuration::FORMAT_VERSION != format_version {
                return Err(RecoveryFailure::SystemConfigurationValueRefused {
                    device,
                    value:
                        SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
                            format_version,
                        },
                });
            }
            Ok(SystemConfigurationSlotReading::IncompatBitsNotRecognized(
                incompat_bitmap,
            ))
        }
        Err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(out_of_range)) => {
            Err(RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
                device,
                out_of_range,
            })
        }
    }
}

/// 逐档找槽 1 时相邻两档槽距之差：槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19），
/// io_min 是逻辑块宽的整数倍、逻辑块宽最小 512 字节（Linux 块层的扇区），所以格式允许的槽距都是 512 的整数倍。
const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 512;

/// 全池没有一个槽 0 可择时找一块盘的槽 1：槽距无处可读，在格式允许的每一档槽距上各读一槽，
/// 只收可择、且自己记的固定结构槽距正好等于它所在偏移的那一槽；几档都收得到时取世代号最大的
/// （D22（单元原子性怎么合成） 已定项 16「择槽取校验和过且世代号最大的」，逐盘计），世代号相同取偏移小的。
/// 各盘择到的之间 fsid 与设备数要对得上，由 [`choose_system_configuration`] 照常判。
///
/// 格式允许的槽距：不小于 4096（[`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`]）、[`SLOT_SPACING_SEARCH_STEP_BYTES`] 的整数倍，
/// 槽 1 整槽落在根环基址（[`region_start`] 的区域 0，1 MiB）之前——mkfs 判固定结构两两不重叠（`make_filesystem::check_geometry`）。
/// 上下界之间 (1 MiB − 4096 − 4096) ÷ 512 + 1 = 2033 档，每档读一次。
///
/// 风险：同一块盘上更早一次 mkfs 用别的槽距写下的槽 1，这一次 mkfs 不清（它只写本次的两槽），它自证得过、记的槽距也等于它所在的偏移，
/// 与本池的槽 1 分不出。它的世代号更大时择到的就是它：fsid 不同时各盘择到的对不上、整池报 [`RecoveryFailure::SystemConfigurationsDisagree`]；
/// fsid 相同（调用方拿同一个 fsid 重做 mkfs）时按旧池的几何挂。只按最小槽距 4096 试一档时，旧池槽距是 4096 那一格上同一个风险也在。
///
/// 一档读回来的分流与槽 0、槽 1 同一个（[`mountable_slot_or_refusal`]）。一槽可择的都没有时，交回试到的第一个 incompat 位图不认识的槽
/// （按偏移升序；它的槽距记在哪一格读不出来，不核偏移），再没有就是自证不过。
///
/// # Errors
/// 试到的某一槽自证得过、每区槽数 S 越界 ⇒ [`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]，整池拒，与槽 0、槽 1 同一个判法。
fn slot_one_found_by_trying_every_slot_spacing_the_format_allows(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
    slots_of_which_pool: SlotsOfWhichPool,
) -> Result<SystemConfigurationSlotReading, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let largest_spacing = largest_fixed_structure_slot_spacing_the_format_allows();
    let mut highest_generation_found: Option<SystemConfiguration> = None;
    let mut first_slot_with_incompat_bits_not_recognized: Option<IncompatBitmap> = None;
    // 迭代次数的上界是档数（2033）；跨轮带着的是至今世代号最大的那一槽与第一个 incompat 位图，都只换不删。
    // 这一档槽距下槽 1 的偏移就是槽距本身（槽 i 在 i × 槽距）。
    for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)
        .step_by(usize::try_from(SLOT_SPACING_SEARCH_STEP_BYTES).expect("512"))
    {
        match mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(candidate_spacing), slot_bytes),
            slots_of_which_pool,
        )? {
            SystemConfigurationSlotReading::Mountable(system_configuration) => {
                let recorded_spacing = u64::from(
                    system_configuration
                        .immutable
                        .sizes
                        .fixed_structure_slot_spacing,
                );
                if recorded_spacing != candidate_spacing {
                    continue;
                }
                if highest_generation_found.is_none_or(|found| {
                    system_configuration.quantities.slot_generation
                        > found.quantities.slot_generation
                }) {
                    highest_generation_found = Some(system_configuration);
                }
            }
            SystemConfigurationSlotReading::NotSelfDescribing => {}
            SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap) => {
                first_slot_with_incompat_bits_not_recognized =
                    first_slot_with_incompat_bits_not_recognized.or(Some(incompat_bitmap));
            }
        }
    }
    Ok(
        match (
            highest_generation_found,
            first_slot_with_incompat_bits_not_recognized,
        ) {
            (Some(system_configuration), _) => {
                SystemConfigurationSlotReading::Mountable(system_configuration)
            }
            (None, Some(incompat_bitmap)) => {
                SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap)
            }
            (None, None) => SystemConfigurationSlotReading::NotSelfDescribing,
        },
    )
}

/// 每盘两槽：槽 0 在偏移 0；槽 1 在固定结构槽距处（D22（单元原子性怎么合成） 已定项 16）。槽距是池级字段（两盘四槽同值）：
/// 先读遍每块盘的槽 0，一块盘的槽 0 可择就按它自己记的槽距找它的槽 1；自己的槽 0 不可择，按池里第一块槽 0 可择的盘记的槽距找；
/// 全池没有一个槽 0 可择时槽距无处可读，在格式允许的每一档槽距上试（[`slot_one_found_by_trying_every_slot_spacing_the_format_allows`]，
/// 那里写着它的风险：更早一次 mkfs 留下的槽 1 分不出来）。
/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余
/// （D22（单元原子性怎么合成） 已定项 8 第 1 条；E87 的 8 种失效组合里「掉了那块盘」这一格要可挂）。
///
/// # Errors
/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；
/// 池里一槽可择的都没有、而至少一槽自证得过只是 incompat 位图不认识（[`RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized`]，
/// 先于上一条判）；
/// 交得出系统配置的几块盘之间 fsid 或设备数对不上（[`RecoveryFailure::SystemConfigurationsDisagree`]）；
/// 池里一块盘都没有（[`RecoveryFailure::NoDeviceInThePool`]）；
/// 某一槽自证得过、而自述的每区槽数 S 越界（[`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]）——
/// **这一条是挂载时的区间判**：S 越界的池在这里就拒，后面一步不走、盘上一个字节不动。
pub fn choose_system_configuration(
    reader: &dyn PoolReader,
) -> Result<SystemConfiguration, RecoveryFailure> {
    choose_system_configuration_naming_devices_without_a_mountable_slot(reader)
        .map(|choice| choice.system_configuration)
}

/// 择到的系统配置，连带池里两个槽都不可择（读不出、自证不过、或 incompat 位图不认识）而被跳过的那几块盘
/// （[`choose_system_configuration_naming_devices_without_a_mountable_slot`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationChoice {
    pub system_configuration: SystemConfiguration,
    /// 两个系统配置槽都不可择的盘，按 [`PoolReader::device_identities`] 的次序；冗余完好时为空。择系统配置时跳过它们、照常择别的盘那一份
    /// （D22（单元原子性怎么合成） 已定项 8 第 1 条），这张表把「冗余被吃掉一份」报给只读与恢复路径的调用方（C463（一块盘的系统配置两份全废之后静默成功），
    /// `research/prompts/m3-ring-r1-main-verification.md` 第二节第 11 行「只读与恢复路径报出来」）。只报、不重写：自愈随那一轮第四节第 3 条定。
    pub devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity>,
}

/// 与 [`choose_system_configuration`] 同一个择法、同一组错误（它就是调这一个），另交回两个槽都不可择而被跳过的盘
/// （[`SystemConfigurationChoice::devices_without_a_mountable_system_configuration_slot`]）。
/// 单开一个函数、不改 [`choose_system_configuration`] 的返回类型：后者的调用点遍布 core 与验证包，
/// 要这张表的只有只读挂载（`mounted_read::MountedReadOnly`）与想看冗余的恢复调用方。
///
/// # Errors
/// 同 [`choose_system_configuration`]。
pub fn choose_system_configuration_naming_devices_without_a_mountable_slot(
    reader: &dyn PoolReader,
) -> Result<SystemConfigurationChoice, RecoveryFailure> {
    choose_system_configuration_among_the_slots_of(reader, SlotsOfWhichPool::AnyPool)
}

/// 可写挂载与离线出路择系统配置（S3，C707；三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U5 S3 那一行，
/// 用户 2026-10-19 定「先按 fsid 筛，算自证不过」）：与 [`choose_system_configuration`] 同一个择法，只是自述的 fsid 不是
/// `filesystem_identifier_of_the_pool`（调用方交进来的本池 fsid）的槽当自证不过、择这块盘的另一槽——一块盘最新那一槽换成别的池的
/// （单故障）不再让各盘择到的 fsid 对不上、整池拒。
///
/// # Errors
/// 同 [`choose_system_configuration`]；别的池的槽筛掉之后某块盘两槽都不可择，照「两个槽都无效的盘」跳过。
pub fn choose_system_configuration_of_the_pool(
    reader: &dyn PoolReader,
    filesystem_identifier_of_the_pool: &[u8; 16],
) -> Result<SystemConfiguration, RecoveryFailure> {
    choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
        reader,
        filesystem_identifier_of_the_pool,
    )
    .map(|choice| choice.system_configuration)
}

/// 与 [`choose_system_configuration_of_the_pool`] 同一个筛法，另交回两个槽都不可择而被跳过的盘（别的池的槽筛掉之后两槽都不可择的盘也在内）：
/// 带本池 fsid 的只读挂载与恢复（`mounted_read::mount_read_only_of_the_pool`、[`recover_of_the_pool`]）用。
///
/// # Errors
/// 同 [`choose_system_configuration_of_the_pool`]；盘上没有一槽是这个 fsid 的池的报 [`RecoveryFailure::NoValidSystemConfiguration`]。
pub fn choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
    reader: &dyn PoolReader,
    filesystem_identifier_of_the_pool: &[u8; 16],
) -> Result<SystemConfigurationChoice, RecoveryFailure> {
    choose_system_configuration_among_the_slots_of(
        reader,
        SlotsOfWhichPool::OnlyThePool(*filesystem_identifier_of_the_pool),
    )
}

/// [`choose_system_configuration_naming_devices_without_a_mountable_slot`] 与 [`choose_system_configuration_of_the_pool`] 共用的择法，
/// `slots_of_which_pool` 说收哪个池的槽（[`SlotsOfWhichPool`]）。
fn choose_system_configuration_among_the_slots_of(
    reader: &dyn PoolReader,
    slots_of_which_pool: SlotsOfWhichPool,
) -> Result<SystemConfigurationChoice, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut chosen: Option<SystemConfiguration> = None;
    let mut devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity> = Vec::new();
    let mut first_slot_with_incompat_bits_not_recognized: Option<(DeviceIdentity, IncompatBitmap)> =
        None;
    let mut slot_zero_of_each_device: Vec<(DeviceIdentity, SystemConfigurationSlotReading)> =
        Vec::new();
    for device in reader.device_identities() {
        let slot_zero = mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(0), slot_bytes),
            slots_of_which_pool,
        )?;
        slot_zero_of_each_device.push((device, slot_zero));
    }
    let slot_spacing_recorded_by_the_first_mountable_slot_zero = slot_zero_of_each_device
        .iter()
        .find_map(|(_, slot_zero)| slot_zero.recorded_fixed_structure_slot_spacing());
    for (device, slot_zero) in slot_zero_of_each_device {
        // 自己的槽 0 不可择时它记的槽距不能信，用池里别的盘槽 0 记的；全池一个都没有时逐档试——按 D2（RAID 条带策略） 已定项 19
        // 的式子拿挂载时探得的 io_min 算，要 `PoolReader` 交得出探测值，今天交不出。
        let slot_one = match slot_zero
            .recorded_fixed_structure_slot_spacing()
            .or(slot_spacing_recorded_by_the_first_mountable_slot_zero)
        {
            Some(spacing) => mountable_slot_or_refusal(
                device,
                reader.read(device, DeviceOffsetInBytes(spacing), slot_bytes),
                slots_of_which_pool,
            )?,
            None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(
                reader,
                device,
                slots_of_which_pool,
            )?,
        };
        if first_slot_with_incompat_bits_not_recognized.is_none() {
            first_slot_with_incompat_bits_not_recognized = slot_zero
                .incompat_bitmap_not_recognized()
                .or_else(|| slot_one.incompat_bitmap_not_recognized())
                .map(|incompat_bitmap| (device, incompat_bitmap));
        }
        let best_on_device = match (slot_zero.into_mountable(), slot_one.into_mountable()) {
            (None, None) => {
                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上；记下来交给调用方。
                devices_without_a_mountable_system_configuration_slot.push(device);
                continue;
            }
            (Some(only), None) | (None, Some(only)) => only,
            (Some(zero), Some(one)) => {
                if one.quantities.slot_generation > zero.quantities.slot_generation {
                    one
                } else {
                    zero
                }
            }
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.immutable.filesystem_identifier
                    != best_on_device.immutable.filesystem_identifier
                    || previous.immutable.device_count != best_on_device.immutable.device_count
                {
                    return Err(RecoveryFailure::SystemConfigurationsDisagree);
                }
            }
        }
    }
    if let Some(system_configuration) = chosen {
        return Ok(SystemConfigurationChoice {
            system_configuration,
            devices_without_a_mountable_system_configuration_slot,
        });
    }
    // 一份可择的都没有：有一槽自证得过、只是布局不认识，就报布局不认识（不是数据坏了）；否则才是每块盘两槽都坏。
    match (
        first_slot_with_incompat_bits_not_recognized,
        devices_without_a_mountable_system_configuration_slot
            .first()
            .copied(),
    ) {
        (Some((device, incompat_bitmap)), _) => Err(
            RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
                first_device_carrying_them: device,
                incompat_bitmap,
            },
        ),
        (None, Some(device)) => Err(RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: device,
        }),
        // 一块盘交得出系统配置就落在上面的 `chosen`，一块交不出就记进那张表：两样都空只在池里一块盘都没有时。
        (None, None) => Err(RecoveryFailure::NoDeviceInThePool),
    }
}

/// 择到的系统配置（[`choose_system_configuration`]）那一槽的盘上原样（整槽 `SYSTEM_CONFIGURATION_SLOT_BYTES`）：在择到的那份自述的本盘
/// （`immutable.this_device`）两个槽里找解出来与它相同的那一槽；择不到系统配置、或两槽都对不上时 None。只读。
/// 挂载透明的判官按字节区间比 R1 与 R2 的系统配置用它（系统运行配置那 36 字节 `parse_slot` 不进内存，只有原字节里有）。
#[must_use]
pub fn chosen_system_configuration_slot_bytes(reader: &dyn PoolReader) -> Option<Vec<u8>> {
    let chosen = choose_system_configuration(reader).ok()?;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let slot_spacing = u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing);
    (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).find_map(|slot_index| {
        let bytes = reader.read(
            chosen.immutable.this_device,
            DeviceOffsetInBytes(slot_index * slot_spacing),
            slot_bytes,
        )?;
        (SystemConfiguration::parse_slot(&bytes).as_ref() == Ok(&chosen)).then_some(bytes)
    })
}

/// 一块盘两槽里自证过、fsid 与本池相同的系统配置。取号的 max（D18（块里携带什么信息） 已定项 11）与系统配置槽写的世代号
/// （D22（单元原子性怎么合成） 已定项 16，逐盘计）都按这个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_system_configuration_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> Vec<SystemConfiguration> {
    system_configuration_slots_of_one_device_read_once(
        reader,
        device,
        slot_spacing_in_bytes,
        filesystem_identifier,
    )
    .self_verified
}

/// 一块盘两个系统配置槽各读一次的结局（D22（单元原子性怎么合成） 已定项 8 射程「可写挂载读系统配置的见证按盘读」，S-辨：
/// 一槽撕裂、另一槽自证过，与设备读不出分开）：自证过、fsid 与本池相同的那几份，设备报读错（`PoolReader::read` 交空）的槽数，
/// 与整槽校验和过而 fsid 不是本池的槽数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationSlotsOfOneDevice {
    pub device: DeviceIdentity,
    pub self_verified: Vec<SystemConfiguration>,
    pub slots_reporting_a_read_error: u64,
    /// 别的池的槽：magic、整槽校验和、布局位、每区槽数都过，自述的 fsid 不是本池的。「自证过」要 fsid 与本池相同
    /// （`research/prompts/e205-preregistration.md`「乙入判不出」下一行「两种读法共用」，C707（系统配置判不出里 fsid 不符落哪一边没写）
    /// 定「fsid 不是本池的槽按自证不过处置」），这几槽不进 `self_verified`；读的一方不报读坏（不进读坏账）。
    /// R2 本身不看它；可写挂载读阶段见证读的乙入判不出（`mount::newer_publish_witness`）把它算读得出而自证不过的槽。
    pub slots_of_another_pool_whose_checksum_verifies: u64,
}

impl SystemConfigurationSlotsOfOneDevice {
    /// 两槽都自证过（R2 里「有一块盘两槽都自证过」那一半）。
    #[must_use]
    pub fn both_slots_self_verified(&self) -> bool {
        u64::try_from(self.self_verified.len()).expect("一块盘两个槽，装得进 u64")
            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
    }
}

/// 一块盘两槽各读一次：读得出的按整槽校验和与 fsid 判（每区槽数 S 越界的槽与自证不过的槽一样丢掉：走到这一步之前
/// `choose_system_configuration` 已经在同一个池上判过区间、越界的池根本挂不上，这里再报一次越界也没有第二种处置），读不出的计数，
/// 解得开而 fsid 不是本池的计数。
#[must_use]
pub fn system_configuration_slots_of_one_device_read_once<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> SystemConfigurationSlotsOfOneDevice {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut self_verified = Vec::new();
    let mut slots_reporting_a_read_error = 0;
    let mut slots_of_another_pool_whose_checksum_verifies = 0;
    for offset in [0, slot_spacing_in_bytes] {
        match reader.read(device, DeviceOffsetInBytes(offset), slot_bytes) {
            None => slots_reporting_a_read_error += 1,
            Some(bytes) => match SystemConfiguration::parse_slot(&bytes) {
                Ok(system_configuration) => {
                    if system_configuration.immutable.filesystem_identifier
                        == *filesystem_identifier
                    {
                        self_verified.push(system_configuration);
                    } else {
                        slots_of_another_pool_whose_checksum_verifies += 1;
                    }
                }
                // 读成而自证不过（magic 与整槽校验和、节点大小，C840 当自证不过）：报给读者（可写挂载这次挂载共用的那一份记进读坏账、
                // 丢掉这一槽，下一次读它穿到盘上）。mkfs 把每块盘两槽都写过，这两样不过的槽不是「本来就空」。
                Err(
                    SystemConfigurationSlotRefusal::NotSelfDescribing
                    | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
                ) => reader.note_that_the_bytes_read_fail_self_verification(
                    device,
                    DeviceOffsetInBytes(offset),
                    slot_bytes,
                ),
                // 布局不认识、每区槽数越界：是一份完整的系统配置、不是坏槽（择系统配置那一步已按它们判过整池），不算读坏。
                Err(
                    SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { .. }
                    | SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(_),
                ) => {}
            },
        }
    }
    SystemConfigurationSlotsOfOneDevice {
        device,
        self_verified,
        slots_reporting_a_read_error,
        slots_of_another_pool_whose_checksum_verifies,
    }
}

/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」，零轮）：
/// 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。可写挂载读阶段的见证读（`mount::newer_publish_witness`）与
/// 取号那一刻读见证值（`transaction::self_verified_system_configurations_of_every_device`）都按它判，一处定义。
/// 撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
#[must_use]
pub fn system_configuration_witness_is_undecidable(
    slots_of_every_device: &[SystemConfigurationSlotsOfOneDevice],
) -> bool {
    slots_of_every_device
        .iter()
        .any(|slots| slots.slots_reporting_a_read_error > 0)
        && !slots_of_every_device
            .iter()
            .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)
}
