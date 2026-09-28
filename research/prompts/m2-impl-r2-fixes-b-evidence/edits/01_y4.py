from edit_lib import edit

SC = "crates/singlefs-core/src/system_configuration.rs"
edit(SC, """const ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362;
const REGION_DEVICES_OFFSET: u64 = 379;""", """const ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362;
/// 几何段里「journal 环起点」那 8 字节（16 KiB 槽号，字段表 `layout/01-first-txn.md` 一）。写侧写它之前断言位置、读侧按它切。
const JOURNAL_RING_START_SLOT_OFFSET: u64 = 325;
/// 几何段里「根环起点」那 8 字节（16 KiB 槽号，D22（单元原子性怎么合成） 已定项 16 第 1 句）。写侧写它之前断言位置、读侧按它切。
const ROOT_RING_BASE_SLOT_OFFSET: u64 = 371;
const REGION_DEVICES_OFFSET: u64 = 379;""")
edit(SC, """/// D2（RAID 条带策略） 已定项 18：第一版 w_max 与 g 都写 4。""", """/// 第一版写进、也只收的 journal 环起点（16 KiB 槽号 1024，D22（单元原子性怎么合成） 已定项 16 第 1 句）：mkfs 与写入口照它写，
/// 读者择系统配置时拿槽里读到的与它比，不等整池拒（`recovery` 的 `system_configuration_values_this_reader_accepts`）。
/// core 读写 journal 环按格式常量走，这一判保证盘上那一份与它相同。
pub const JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION: SlotNumber = SlotNumber(JOURNAL_RING_START_SLOT);
/// 第一版写进、也只收的根环起点（16 KiB 槽号 64 = 1 MiB，D22（单元原子性怎么合成） 已定项 16 第 1 句）：同上，core 读写根环按格式常量走
/// （`root_ring::region_start`）。
pub const ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION: SlotNumber = SlotNumber(ROOT_RING_BASE_SLOT);
/// D2（RAID 条带策略） 已定项 18：第一版 w_max 与 g 都写 4。""")
edit(SC, """/// 23 行 128。这 37 行里只有下面 5 个字段（`sizes` 里 4 个值，合起来 8 个值）在内存里存着，
/// 其余由 [`SystemConfiguration::to_slot`] 照格式常量写死。""", """/// 23 行 128。这 37 行里只有下面 7 个字段（`sizes` 里 5 个值，合起来 11 个值）在内存里存着，
/// 其余由 [`SystemConfiguration::to_slot`] 照格式常量写死。journal 环起点与根环起点两个字段读进来（代码三方 m2-closeout-code-r2
/// 「core 与 checker 对系统配置几何字段的读法」：此前解槽跳过这两个字段、按编译期常量走，checker 按字段读，两边判不一致；
/// 用户 2026-09-27 定「读字段，不等于常量就整池拒」）。""")
edit(SC, """    pub region_devices: [DeviceIdentity; 3],
    pub sizes: SystemImmutableSizes,
}

impl SystemImmutableConfiguration {""", """    pub region_devices: [DeviceIdentity; 3],
    pub sizes: SystemImmutableSizes,
    /// journal 环起点（偏移 325，16 KiB 槽号）。写者写 [`JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION`]；读者收的只有它。
    pub journal_ring_start_slot: SlotNumber,
    /// 根环起点（偏移 371，16 KiB 槽号；基址 = 该字段 × 16384，D22（单元原子性怎么合成） 已定项 16 第 1 句）。
    /// 写者写 [`ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION`]；读者收的只有它，判固定结构槽距的上界用它（同一槽自述的根环起点）。
    pub root_ring_base_slot: SlotNumber,
}

impl SystemImmutableConfiguration {""")
edit(SC, """            writer.put_u32(0); // 扩展点声明值 N：第一版 0（D21（权威态与派生态的分界） 已定项 8）
            writer.put_u64(JOURNAL_RING_START_SLOT);""", """            writer.put_u32(0); // 扩展点声明值 N：第一版 0（D21（权威态与派生态的分界） 已定项 8）
            writer.assert_position(JOURNAL_RING_START_SLOT_OFFSET, "journal 环起点");
            writer.put_u64(self.immutable.journal_ring_start_slot.0);""")
edit(SC, """            writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));
            writer.put_u64(ROOT_RING_BASE_SLOT);""", """            writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));
            writer.assert_position(ROOT_RING_BASE_SLOT_OFFSET, "根环起点");
            writer.put_u64(self.immutable.root_ring_base_slot.0);""")
edit(SC, """        let physical_block_size = reader.get_u32();
        reader.skip(4 + 8);
        let journal_ring_bytes = reader.get_u64();""", """        let physical_block_size = reader.get_u32();
        let journal_ring_start_slot = SlotNumber(
            ByteReader::at(
                bytes,
                usize::try_from(JOURNAL_RING_START_SLOT_OFFSET).expect("325"),
            )
            .get_u64(),
        );
        reader.skip(4 + 8);
        let journal_ring_bytes = reader.get_u64();
        let root_ring_base_slot = SlotNumber(
            ByteReader::at(
                bytes,
                usize::try_from(ROOT_RING_BASE_SLOT_OFFSET).expect("371"),
            )
            .get_u64(),
        );""")
edit(SC, """                    journal_ring_bytes,
                    root_ring_slots_per_region,
                },
            },
            // 节点大小那 4 字节不读回来""", """                    journal_ring_bytes,
                    root_ring_slots_per_region,
                },
                journal_ring_start_slot,
                root_ring_base_slot,
            },
            // 节点大小那 4 字节不读回来""")
edit(SC, """                    root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
                },
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation: 1,""", """                    root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
                },
                journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation: 1,""")

MK = "crates/singlefs-core/src/make_filesystem.rs"
edit(MK, """                region_devices: parameters.region_devices,
                sizes: parameters.geometry,
            },""", """                region_devices: parameters.region_devices,
                sizes: parameters.geometry,
                journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },""")
edit(MK, """    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};""", """    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};""")
TX = "crates/singlefs-core/src/transaction.rs"
edit(TX, """                region_devices: self.parameters.region_devices,
                sizes: self.parameters.geometry,
            },""", """                region_devices: self.parameters.region_devices,
                sizes: self.parameters.geometry,
                journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },""")
edit(TX, """    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};""", """    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};""")
MU = "crates/singlefs-harness/tests/system_configuration_mutability_classes.rs"
edit(MU, """    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};""", """    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};""")
edit(MU, """                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
        },""", """                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
            root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
        },""", count=2)

RC = "crates/singlefs-core/src/recovery.rs"
edit(RC, """    unit_area_start_slot_recorded_in_the_slot, IncompatBitmap, SystemConfiguration,
    SystemConfigurationSlotRefusal, SystemImmutableSizes,
};""", """    unit_area_start_slot_recorded_in_the_slot, IncompatBitmap, SystemConfiguration,
    SystemConfigurationSlotRefusal, SystemImmutableSizes,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};""")
edit(RC, """    /// 固定结构槽距落在格式允许的区间之外：不小于 4096、槽 1 整槽落在根环基址之前（与逐档找槽 1 同一个区间，
    /// [`largest_fixed_structure_slot_spacing_the_format_allows`]）。槽距是根槽宽的上界，这一条先于根槽宽判。
    FixedStructureSlotSpacingOutsideTheFormatRange { fixed_structure_slot_spacing: u32 },""", """    /// 固定结构槽距落在格式允许的区间之外：不小于 4096、槽 1 整槽落在同一槽自述的根环起点之前（D22（单元原子性怎么合成） 已定项 16
    /// 第 5 句；根环起点字段 × 16384 − 4096 是上界，[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]；根环起点不到一个
    /// 系统配置槽宽时一档槽距都不收）。槽距是根槽宽的上界，这一条先于根槽宽判；也先于根环起点等不等第一版常量那一判，
    /// 根环起点落在槽 1 之前的槽报在这里（与 checker 的 I-7.13 报同一格）。
    FixedStructureSlotSpacingOutsideTheFormatRange { fixed_structure_slot_spacing: u32 },""")
edit(RC, """    /// journal 环长装不下 F 条记录""", """    /// 同一槽记着的根环起点（偏移 371 的 8 字节，16 KiB 槽号）不是第一版写的那个（64 = 1 MiB，
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
    /// journal 环长装不下 F 条记录""")
edit(RC, """/// 格式允许的最大固定结构槽距：槽 1 整槽落在根环基址（区域 0 的起点）之前。逐档找槽 1 与读者判槽距共用这一个上界。
fn largest_fixed_structure_slot_spacing_the_format_allows() -> u64 {
    region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES
}""", """/// 格式允许的最大固定结构槽距：槽 1 整槽落在第一版根环基址（区域 0 的起点）之前。逐档找槽 1 用它：那时一槽可择的都没有，
/// 没有哪一槽自述的根环起点可读。读者判一槽的槽距用那一槽自述的根环起点（[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]）。
fn largest_fixed_structure_slot_spacing_the_format_allows() -> u64 {
    region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES
}

/// 一槽自述的根环起点下格式允许的最大固定结构槽距：槽 1 整槽落在根环基址（该字段 × 16384）之前（D22（单元原子性怎么合成） 已定项 16
/// 第 1、5 句）。根环起点是盘上读来的 8 字节：乘出来溢出时上界取 `u64::MAX`（基址在任何设备偏移之外，槽 1 落在它之前）；
/// 基址不到一个系统配置槽宽时一档都不收（`None`）。
fn largest_fixed_structure_slot_spacing_under_the_root_ring_base(
    root_ring_base_slot: SlotNumber,
) -> Option<u64> {
    root_ring_base_slot.0.checked_mul(SLOT_BYTES).map_or(
        Some(u64::MAX),
        |root_ring_base_in_bytes| root_ring_base_in_bytes.checked_sub(SYSTEM_CONFIGURATION_SLOT_BYTES),
    )
}""")
edit(RC, """/// 次序：格式版本、加密类型、槽距、根槽宽、环长、单元区起点——根槽宽的上界是槽距，所以槽距先判；环末端对单元区起点那一判先于起点是不是
/// 环长现算的那个，与 checker 的环长判法（`journal_ring_bytes_lie_in_the_supported_range`）报同一格。""", """/// 次序：格式版本、加密类型、槽距、根槽宽、根环起点、journal 环起点、环长、单元区起点——根槽宽的上界是槽距，所以槽距先判；
/// 槽距的上界取同一槽自述的根环起点，所以根环起点落在槽 1 之前的那一格报成槽距越界、与 checker 的 I-7.13 同一格，之后才判它等不等
/// 第一版常量；环末端对单元区起点那一判先于起点是不是环长现算的那个，与 checker 的环长判法（`journal_ring_bytes_lie_in_the_supported_range`）报同一格。""")
edit(RC, """    let sizes = &system_configuration.immutable.sizes;
    let fixed_structure_slot_spacing = u64::from(sizes.fixed_structure_slot_spacing);
    if fixed_structure_slot_spacing < FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        || fixed_structure_slot_spacing > largest_fixed_structure_slot_spacing_the_format_allows()
    {""", """    let sizes = &system_configuration.immutable.sizes;
    let fixed_structure_slot_spacing = u64::from(sizes.fixed_structure_slot_spacing);
    if fixed_structure_slot_spacing < FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        || largest_fixed_structure_slot_spacing_under_the_root_ring_base(
            system_configuration.immutable.root_ring_base_slot,
        )
        .is_none_or(|largest_spacing| fixed_structure_slot_spacing > largest_spacing)
    {""")
edit(RC, """    let recorded_unit_area_start_slot = unit_area_start_slot_recorded_in_the_slot(slot);
    // 环末端 = 环起点 + 环长；""", """    let recorded_root_ring_base_slot = system_configuration.immutable.root_ring_base_slot;
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
    let recorded_unit_area_start_slot = unit_area_start_slot_recorded_in_the_slot(slot);
    // 环末端 = 环起点 + 环长；""")
print("01_y4 done")
