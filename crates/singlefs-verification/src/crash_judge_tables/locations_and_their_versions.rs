//! 落点与它们的版本：从基线里择出的系统配置读几何、枚举判器读的结构槽落点，
//! 按段算盖住每个落点的那几次写可达的组合，照 `CrashImage::read` 同一条规则叠出每个版本的字节与逐扇区等不等。

use super::*;

/// 基线里择出的那一份系统配置记的几何（[`system_configuration_the_locations_are_laid_out_by`]）：落点从它来。
pub(super) struct BaseGeometry {
    fixed_structure_slot_spacing: u64,
    physical_block_size: u64,
    root_ring_slots_per_region: u64,
    region_devices: [u32; 3],
    journal_ring_bytes: u64,
    pub(super) unit_area_start_slot: u64,
}

/// 基线里 `device` 上偏移 `offset` 那一个系统配置槽自证得过（`SystemConfiguration::parse_slot` 交 Ok）时解出的那一份；读不出、解不开交 None。
fn self_verified_system_configuration_slot_at(
    base: &MemoryPool,
    device: singlefs_core::address::DeviceIdentity,
    offset: u64,
) -> Option<SystemConfiguration> {
    let slot_bytes_of_system_configuration =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let bytes = PoolReader::read(
        base,
        device,
        DeviceOffsetInBytes(offset),
        slot_bytes_of_system_configuration,
    )?;
    SystemConfiguration::parse_slot(&bytes).ok()
}

/// 格式允许的每一档槽距上槽 1 的偏移（槽 i 在 i × 槽距，槽 1 的偏移就是槽距）：不小于 [`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`]、
/// 扇区宽 [`SECTOR_BYTES`] 的整数倍、槽 1 整槽落在根环基址之前，同 core 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`
/// 与 GPU 两个内核逐档读槽 1 的那两个循环（`slot_one_by_trying_every_spacing`、`checker_slot_one_by_reading_every_spacing`）。
/// (1 MiB − 4096 − 4096) ÷ 512 + 1 = 2033 档。择落点几何（[`system_configuration_the_locations_are_laid_out_by`]）与建系统配置落点
/// （[`enumerate_locations`]）都按这一份试。
fn slot_one_offsets_of_every_spacing_the_format_allows() -> impl Iterator<Item = u64> {
    let largest_spacing = region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES;
    (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)
        .step_by(usize::try_from(SECTOR_BYTES).expect("512"))
}

/// 落点几何取基线里哪一份系统配置：恢复择得出就取恢复择的那一份（`singlefs_core::recovery::choose_system_configuration`：每块盘的槽 0、
/// 槽 1 比世代号，全池没有一个槽 0 可择时逐档读槽 1、取世代号最大的；GPU 两个内核逐档读时也取世代号最大的，`crash_judge_common.wgsl` 的
/// `slot_one_by_trying_every_spacing`），几何与 GPU 读根槽的落点同源（hand_out 第二位报告「交主 agent 定的」第 2 条：取第一份自证得过的、
/// 同一块盘上另有一份记别的槽距、世代号更大的槽时两边几何不同）。恢复择不出（整池拒：带读者不收之值、每区槽数越界）时退回下面那一种找法，
/// 取第一份自证得过的；一份都没有才交 None。CPU 恢复与池级 checker 判得了的镜像（一块盘两槽全废、或每块盘的槽 0 都抹掉而槽 1 在），
/// GPU 判表照样建得出。次序：先按池里盘的次序读每块盘的槽 0；全池没有一个槽 0 自证得过时槽距无处可读，按池里盘的次序在格式允许的每一档槽距
/// （[`slot_one_offsets_of_every_spacing_the_format_allows`]）上各读一槽，只收自己记的槽距正好等于它所在偏移的那一槽。
/// 几何那几样（槽距、根槽宽、每区槽数、区域盘、环长）是池级不可变字段，哪块盘哪一槽交出来都一样；这里只取几何，不判读者收不收那几个池级值
/// （那一判在 GPU 恢复内核与池级 checker 内核里，与 CPU 同判）：带读者不收之值的那一份照样取它的几何——读者遇到它整池拒（恢复报
/// `SystemConfigurationValueRefused`、池级 checker 只交 I-7.13），几何不进结论；跳过它去取更远一档槽距上的槽，那一份记的槽距与写表里
/// 根槽写的落点对不上，判表反而建不出（`WriteNotAlignedToALocation`）。几何取哪一份都不决定 GPU 读不读得到逐档那几处槽：
/// 那几处另建落点（[`enumerate_locations`]）。
fn system_configuration_the_locations_are_laid_out_by(
    base: &MemoryPool,
) -> Option<SystemConfiguration> {
    if let Ok(chosen_by_the_recovery_reader) =
        singlefs_core::recovery::choose_system_configuration(base)
    {
        return Some(chosen_by_the_recovery_reader);
    }
    let devices = PoolReader::device_identities(base);
    if let Some(slot_zero) = devices
        .iter()
        .find_map(|device| self_verified_system_configuration_slot_at(base, *device, 0))
    {
        return Some(slot_zero);
    }
    // 迭代上界：盘数 × 档数（2033 档）；找到第一份就交回
    devices.iter().find_map(|device| {
        slot_one_offsets_of_every_spacing_the_format_allows().find_map(|candidate_spacing| {
            self_verified_system_configuration_slot_at(base, *device, candidate_spacing).filter(
                |slot_one| {
                    u64::from(slot_one.immutable.sizes.fixed_structure_slot_spacing)
                        == candidate_spacing
                },
            )
        })
    })
}

pub(super) fn base_geometry(base: &MemoryPool) -> Result<BaseGeometry, JudgeTablesRefusal> {
    let system_configuration = system_configuration_the_locations_are_laid_out_by(base)
        .ok_or(JudgeTablesRefusal::BaseSystemConfigurationUnreadable)?;
    let sizes = &system_configuration.immutable.sizes;
    Ok(BaseGeometry {
        fixed_structure_slot_spacing: u64::from(sizes.fixed_structure_slot_spacing),
        physical_block_size: u64::from(sizes.physical_block_size),
        root_ring_slots_per_region: sizes.root_ring_slots_per_region.count(),
        region_devices: system_configuration
            .immutable
            .region_devices
            .map(|device| device.0),
        journal_ring_bytes: sizes.journal_ring_bytes,
        unit_area_start_slot: slot_after_the_journal_ring(sizes.journal_ring_bytes).0,
    })
}

/// 全部落点（还没填写与版本）：系统配置每盘两槽、根环全部槽、journal 环里基线或枚举写碰过的记录槽、单元区里基线或枚举写碰过的 16 KiB 槽；
/// 「碰过」不算清零写（[`StepKind::ZeroFill`]，连同它的撕裂镜像），理由在碰过的范围那一段。
/// 只在池里的盘（`devices`）上建落点：系统配置记的区域盘可以不在池里（卸载掉电之后只剩一块盘的镜像，丢掉的那块住着根环的两个区域），
/// CPU 的读者读那块盘交 None（`MemoryPool::read`，恢复与池级 checker 都当这一槽读不出），GPU 在那块盘上找不到落点、`read_at` 交没有版本的切片，
/// 同一个结局；给那块盘建落点就要从基线读它的字节，读不出。
pub(super) fn enumerate_locations(
    base: &MemoryPool,
    enumerated: &[RetainedWrite],
    devices: &[u32],
    device_bytes: u64,
    geometry: &BaseGeometry,
) -> Vec<JudgeLocation> {
    let mut locations: Vec<JudgeLocation> = Vec::new();
    let mut starts: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut push = |device: u32, offset: u64, length: u64, kind: JudgeLocationKind| {
        if !devices.contains(&device)
            || offset + length > device_bytes
            || !starts.insert((device, offset))
        {
            return;
        }
        locations.push(JudgeLocation {
            device,
            offset,
            length: u32::try_from(length).expect("落点长度装得进 u32"),
            kind,
            writes: Vec::new(),
            versions: Vec::new(),
        });
    };
    for device in devices {
        push(
            *device,
            0,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
            JudgeLocationKind::SystemConfiguration,
        );
        push(
            *device,
            geometry.fixed_structure_slot_spacing,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
            JudgeLocationKind::SystemConfiguration,
        );
    }
    for region in 0..ROOT_RING_REGIONS {
        let device = geometry.region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..geometry.root_ring_slots_per_region {
            push(
                device,
                region_start(region).0 + slot * geometry.fixed_structure_slot_spacing,
                geometry.physical_block_size,
                JudgeLocationKind::RootSlot,
            );
        }
    }
    // 碰过的字节范围：基线写过的扇区，加枚举写表里每次不是清零写的写的范围。清零写不算碰过：mkfs 一次清整个 journal 环（第一版 768 MiB，
    // 每块盘 196 608 个记录槽），照算就给每一槽建一个落点，GPU 每个状态逐个落点扫环（`crash_judge_common.wgsl` 的 `scan_journal`）。
    // 不算它不少判一个字节：清零写写的全是 0，它的撕裂镜像每个字节不是 0 就是「这次写之前的写全落了」时的旧字节（`crash::torn_image_of_in_place_overwrite`），
    // 旧字节非零的扇区是基线写过的、或录制流里更早一次带字节的写写的，都已算碰过；只被清零写碰过的槽在每个崩溃状态里都读出全 0。
    // GPU 读没有落点的槽，结局与读全 0 相同：单元区里交全零单元（`zero_unit_version_over_a_slot_nothing_wrote`），journal 环里那一槽不进扫环、
    // 全 0 的记录本来也自证不过，逐档读槽 1 读成不自证、全 0 的槽本来也自证不过；系统配置每盘两槽与根环全部槽照建落点，不看碰没碰过。
    // 清零写照样记进它罩住的每个落点的写（下面「每次写盖住哪几个落点」按字节范围记），落点的版本照叠法叠出来。
    let mut touched_ranges: BTreeMap<u32, Vec<Range<u64>>> = BTreeMap::new();
    for (device, sparse) in &base.devices {
        let ranges = touched_ranges.entry(device.0).or_default();
        for sector in sparse.written_sectors().keys() {
            ranges.push(sector * SECTOR_BYTES..(sector + 1) * SECTOR_BYTES);
        }
    }
    for write in enumerated
        .iter()
        .filter(|write| write_puts_bytes_of_its_own(write.kind))
    {
        touched_ranges
            .entry(write.device.0)
            .or_default()
            .push(write.offset.0..write.offset.0 + write.length_in_bytes());
    }
    let ring_start = JOURNAL_RING_START_SLOT * slot_bytes();
    let ring_end = ring_start + geometry.journal_ring_bytes;
    let unit_area_start = geometry.unit_area_start_slot * slot_bytes();
    for (device, ranges) in &touched_ranges {
        let mut journal_records: BTreeSet<u64> = BTreeSet::new();
        let mut unit_slots: BTreeSet<u64> = BTreeSet::new();
        for range in ranges {
            let journal_overlap = range.start.max(ring_start)..range.end.min(ring_end);
            if journal_overlap.start < journal_overlap.end {
                let first = (journal_overlap.start - ring_start) / JOURNAL_RECORD_BYTES;
                let last = (journal_overlap.end - 1 - ring_start) / JOURNAL_RECORD_BYTES;
                journal_records.extend(first..=last);
            }
            let unit_overlap = range.start.max(unit_area_start)..range.end;
            if unit_overlap.start < unit_overlap.end {
                let first = unit_overlap.start / slot_bytes();
                let last = (unit_overlap.end - 1) / slot_bytes();
                unit_slots.extend(first..=last);
            }
        }
        for record in journal_records {
            push(
                *device,
                ring_start + record * JOURNAL_RECORD_BYTES,
                JOURNAL_RECORD_BYTES,
                JudgeLocationKind::JournalRecord,
            );
        }
        for slot in unit_slots {
            let offset = slot * slot_bytes();
            let length = if offset + DATA_UNIT_BYTES <= device_bytes {
                DATA_UNIT_BYTES
            } else {
                NODE_BYTES
            };
            push(*device, offset, length, JudgeLocationKind::UnitSlot);
        }
    }
    // 逐档读到的槽 1：全池没有一个可择的槽 0 时，GPU 两个内核照 CPU 在每一档槽距上各读一槽
    // （[`slot_one_offsets_of_every_spacing_the_format_allows`]），偏移 0 与几何槽距两处之外没有落点的那几档 `read_at` 交没有版本的切片、
    // 读成不自证，而 CPU 在那里读到自证得过的槽（逐档读到 8192 处带读者不收之值、12288 处在飞上限加一的槽，CPU 报 I-7.13）。
    // 每一档上基线或枚举写碰过那 4096 字节的建一个系统配置落点；哪一版都不自证的建完版本再去掉（[`location_is_read_by_the_judge`]）。
    // 碰过的范围只取根环基址之前的那几段（基线写过的扇区可以上十万个，逐档比的只是系统配置那 1 MiB 里的几个）
    let root_ring_base = region_start(0).0;
    for (device, ranges) in &touched_ranges {
        let ranges_before_the_root_ring: Vec<&Range<u64>> = ranges
            .iter()
            .filter(|range| range.start < root_ring_base)
            .collect();
        for offset in slot_one_offsets_of_every_spacing_the_format_allows() {
            if ranges_before_the_root_ring.iter().any(|range| {
                range.start < offset + SYSTEM_CONFIGURATION_SLOT_BYTES && offset < range.end
            }) {
                push(
                    *device,
                    offset,
                    SYSTEM_CONFIGURATION_SLOT_BYTES,
                    JudgeLocationKind::SystemConfiguration,
                );
            }
        }
    }
    locations
}

/// 一次写会不会写下自己的非零字节（[`enumerate_locations`] 按它算碰过的范围）：清零写（连同它的撕裂镜像，种类照抄被撕的那次写）不会，
/// 别的种类会。discard 写的也是 0，照旧算碰过：它罩住的每个单元槽各建一个落点（discard 进判表那一批的写法，
/// `tests::discard_over_a_data_unit_lands_in_every_unit_slot_location_it_covers_in_its_three_states` 钉着），这里不动它。
fn write_puts_bytes_of_its_own(kind: StepKind) -> bool {
    match kind {
        StepKind::ZeroFill => false,
        StepKind::Discard
        | StepKind::UnitWrite
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => true,
    }
}

/// 建完版本之后这个落点留不留在判表里：系统配置落点在偏移 0 与几何槽距上的照留（每块盘两槽，判器按它们读槽 0 与槽 1）；
/// 逐档建的别的那几处至少一版自证得过（magic 对、整槽校验和过，与 GPU 两个内核判「自证」同一条：恢复内核 `parse_system_configuration`、
/// 走读内核 `checker_read_system_configuration_slot` 开头那一判）才留，一版都不自证的留不留 GPU 都读成不自证。别的种类照留。
pub(super) fn location_is_read_by_the_judge(
    location: &JudgeLocation,
    geometry: &BaseGeometry,
) -> bool {
    match location.kind {
        JudgeLocationKind::SystemConfiguration => {
            location.offset == 0
                || location.offset == geometry.fixed_structure_slot_spacing
                || location.versions.iter().any(|version| {
                    version.bytes.starts_with(&SYSTEM_CONFIGURATION_MAGIC)
                        && version.checks[SYSTEM_CONFIGURATION_SLOT_CHECKSUM_HOLDS_CHECK] == 1
                })
        }
        JudgeLocationKind::RootSlot
        | JudgeLocationKind::JournalRecord
        | JudgeLocationKind::UnitSlot => true,
    }
}

/// 枚举写表里录制流之外的那几条（撕裂镜像、重放）各自跟着哪次录制写：撕裂镜像跟着被撕的那次写，重放跟着被撕的那次写与被重放的那次写。
pub(super) struct ExtendedWriteOrigins {
    /// 枚举写表下标 → （被撕的那次写，被重放的那次写（撕裂镜像自己没有））。
    origin: BTreeMap<usize, (usize, Option<usize>)>,
}

impl ExtendedWriteOrigins {
    pub(super) fn of(tables: &CrashStateEnumerationTables, recorded_write_count: usize) -> Self {
        let mut origin = BTreeMap::new();
        for torn in &tables.torn_images {
            origin.insert(torn.torn_image_index, (torn.write_index, None));
            for (replay_index, replayed) in &torn.replays {
                origin.insert(*replay_index, (torn.write_index, Some(*replayed)));
            }
        }
        for index in origin.keys() {
            assert!(
                *index >= recorded_write_count,
                "撕裂镜像与重放都接在录制流之后"
            );
        }
        Self { origin }
    }
}

/// 一个落点在全部崩溃状态里可达的组合：按崩溃在哪一段（含最后全部持久那一个）逐段枚举自由写的落地、算组合、去重。
/// 展开了的段里每次写都当自由写（甲二只取其中一部分组合，多出来的版本没有状态查它，不改判定）。
pub(super) fn reachable_combinations_of_the_expanded_segments(
    segment_of_write: &[usize],
    extended: &ExtendedWriteOrigins,
    segment_count: usize,
    expanded_segments: &BTreeSet<usize>,
    location: &JudgeLocation,
) -> Result<BTreeSet<CombinationOfTheWritesOnALocation>, JudgeTablesRefusal> {
    let recorded_write_count = segment_of_write.len();
    let mut combinations: BTreeSet<CombinationOfTheWritesOnALocation> = BTreeSet::new();
    // 一次写的落地按它在哪一段与这个组合里给它的落地算
    let landing_of = |write_index: usize, cut: usize, free: &BTreeMap<usize, Landing>| -> Landing {
        if let Some(landing) = free.get(&write_index) {
            return *landing;
        }
        if segment_of_write[write_index] < cut {
            Landing::Persisted
        } else {
            Landing::NotPersisted
        }
    };
    let persisted_in = |entry: usize, cut: usize, free: &BTreeMap<usize, Landing>| -> bool {
        if entry < recorded_write_count {
            return landing_of(entry, cut, free) == Landing::Persisted;
        }
        let (torn_write, replayed) = extended.origin[&entry];
        if landing_of(torn_write, cut, free) != Landing::Torn {
            return false;
        }
        match replayed {
            None => true,
            Some(replayed) => landing_of(replayed, cut, free) == Landing::Persisted,
        }
    };
    let combination_of =
        |cut: usize, free: &BTreeMap<usize, Landing>| -> CombinationOfTheWritesOnALocation {
            let mut combination = CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
            for (position, entry) in location.writes.iter().enumerate() {
                if persisted_in(usize::try_from(*entry).expect("下标"), cut, free) {
                    combination.mark_persisted(position);
                }
            }
            combination
        };
    let mut cuts: Vec<usize> = expanded_segments.iter().copied().collect();
    cuts.push(segment_count);
    for cut in cuts {
        // 这一段里的自由写：盖住落点的录制写在这一段的，加上扩展写的来路里在这一段的
        let mut free_writes: BTreeSet<usize> = BTreeSet::new();
        for entry in &location.writes {
            let entry = usize::try_from(*entry).expect("下标");
            if entry < recorded_write_count {
                if segment_of_write[entry] == cut {
                    free_writes.insert(entry);
                }
            } else {
                let (torn_write, replayed) = extended.origin[&entry];
                if segment_of_write[torn_write] == cut {
                    free_writes.insert(torn_write);
                }
                if let Some(replayed) = replayed {
                    if segment_of_write[replayed] == cut {
                        free_writes.insert(replayed);
                    }
                }
            }
        }
        let free_writes: Vec<usize> = free_writes.into_iter().collect();
        let tearable = |write_index: usize| {
            extended
                .origin
                .values()
                .any(|(torn_write, replayed)| *torn_write == write_index && replayed.is_none())
        };
        let choices: Vec<usize> = free_writes
            .iter()
            .map(|write_index| if tearable(*write_index) { 3 } else { 2 })
            .collect();
        let total: usize = choices.iter().product();
        if total > MAXIMUM_VERSIONS_PER_LOCATION {
            return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                device: location.device,
                offset: location.offset,
            });
        }
        for mut assignment in 0..total {
            let mut free: BTreeMap<usize, Landing> = BTreeMap::new();
            for (write_index, choice_count) in free_writes.iter().zip(&choices) {
                let digit = assignment % choice_count;
                assignment /= choice_count;
                let landing = match (choice_count, digit) {
                    (2, 0) | (3, 0) => Landing::NotPersisted,
                    (2, 1) | (3, 2) => Landing::Persisted,
                    (3, 1) => Landing::Torn,
                    _ => unreachable!("数字不超过态数"),
                };
                free.insert(*write_index, landing);
            }
            combinations.insert(combination_of(cut, &free));
            if combinations.len() > MAXIMUM_VERSIONS_PER_LOCATION {
                return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                    device: location.device,
                    offset: location.offset,
                });
            }
        }
    }
    Ok(combinations)
}

/// 给定状态表上一个落点的组合：每个点名的持久掩码在这个落点的写上各取一个组合、去重。
pub(super) fn combinations_of_the_given_persisted_sets(
    location: &JudgeLocation,
    given_persisted_sets: &[Vec<bool>],
) -> Result<BTreeSet<CombinationOfTheWritesOnALocation>, JudgeTablesRefusal> {
    let mut combinations: BTreeSet<CombinationOfTheWritesOnALocation> = BTreeSet::new();
    for persisted in given_persisted_sets {
        let mut combination = CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted[usize::try_from(*entry).expect("下标")] {
                combination.mark_persisted(position);
            }
        }
        combinations.insert(combination);
        if combinations.len() > MAXIMUM_VERSIONS_PER_LOCATION {
            return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                device: location.device,
                offset: location.offset,
            });
        }
    }
    Ok(combinations)
}

/// 一个落点的每个组合叠出字节、算检查字与逐扇区等不等，按组合升序记进它的版本表（与 `CrashImage::read` 同一条叠法）。
pub(super) fn push_the_versions_of_the_combinations(
    base: &MemoryPool,
    enumerated: &[RetainedWrite],
    location: &mut JudgeLocation,
    combinations: BTreeSet<CombinationOfTheWritesOnALocation>,
) {
    let base_bytes = PoolReader::read(
        base,
        singlefs_core::address::DeviceIdentity(location.device),
        DeviceOffsetInBytes(location.offset),
        usize::try_from(location.length).expect("落点长度装得进 usize"),
    )
    .expect("落点在池里的盘上（enumerate_locations 只给池里的盘建落点）、在盘内、按扇区对齐");
    let location_start = location.offset;
    let location_length = u64::from(location.length);
    for combination in combinations {
        let mut bytes = base_bytes.clone();
        for (position, entry) in location.writes.iter().enumerate() {
            if !combination.is_persisted(position) {
                continue;
            }
            let write = &enumerated[usize::try_from(*entry).expect("下标")];
            overlay(&mut bytes, location_start, write);
        }
        let checks = version_checks(location.kind, &bytes);
        let sector_equals = location
            .writes
            .iter()
            .map(|entry| {
                sector_equality(
                    &bytes,
                    location_start,
                    location_length,
                    &enumerated[usize::try_from(*entry).expect("下标")],
                )
            })
            .collect();
        location.versions.push(JudgeLocationVersion {
            combination,
            bytes,
            checks,
            sector_equals,
        });
    }
}

/// 把一次写盖到落点的字节上（与 `CrashImage::read` 同一条叠法）。
fn overlay(bytes: &mut [u8], location_start: u64, write: &RetainedWrite) {
    let location_end = location_start + u64::try_from(bytes.len()).expect("长度");
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    let overlap_start = location_start.max(write_start);
    let overlap_end = location_end.min(write_end);
    if overlap_start >= overlap_end {
        return;
    }
    let destination = usize::try_from(overlap_start - location_start).expect("偏移");
    let source = usize::try_from(overlap_start - write_start).expect("偏移");
    let overlap_length = usize::try_from(overlap_end - overlap_start).expect("长度");
    write.contents.copy_range_into(
        source,
        &mut bytes[destination..destination + overlap_length],
    );
}

/// 落点每个扇区上的字节是不是这次写写下的（这次写没盖到的扇区算不是）。
fn sector_equality(
    bytes: &[u8],
    location_start: u64,
    location_length: u64,
    write: &RetainedWrite,
) -> u64 {
    let sector_count = usize::try_from(location_length / SECTOR_BYTES).expect("扇区数");
    assert!(
        sector_count <= MAXIMUM_LOCATION_SECTORS,
        "落点不超过 64 个扇区"
    );
    let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    let mut mask = 0u64;
    for sector in 0..sector_count {
        let sector_start = location_start + u64::try_from(sector).expect("扇区号") * SECTOR_BYTES;
        if sector_start < write_start || sector_start + SECTOR_BYTES > write_end {
            continue;
        }
        let in_write = usize::try_from(sector_start - write_start).expect("偏移");
        let on_disk = &bytes[sector * sector_bytes..(sector + 1) * sector_bytes];
        if write.contents.range_still_on_disk(in_write, on_disk) {
            mask |= 1u64 << sector;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use crate::crash::CrashPointReplaySegmentExpansion;
    use crate::crash_judge_tables::tests::{
        formatted_pool_of_two_devices,
        formatted_pool_of_two_devices_with_minimum_input_output_bytes, FORMATTED_DEVICE_BYTES,
    };
    use crate::crash_judge_tables::{
        build_judge_tables, build_judge_tables_for_the_given_persisted_sets, JudgeFlow,
        JudgeLocationKind, JudgeTables, JudgeTablesRefusal,
    };
    use crate::memory_pool::{
        writes_and_segments, CrashImage, MemoryPool, RetainedWrite, SparseBlockDevice,
        WrittenContents, SECTOR_BYTES,
    };
    use crate::segments::{FixedGeometry, StepKind};
    use crate::{RecordingBlockDevice, SharedStream};
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::journal::slot_after_the_journal_ring;
    use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
    use singlefs_core::recovery::PoolReader;
    use singlefs_core::root_ring::RootRingSlotsPerRegion;
    use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
    use singlefs_format::{
        JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
    };

    /// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（与 harness 场景
    /// `system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的同名常量同一个值）。
    const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;

    /// 一张镜像建成的给定状态表（写表为空、一个空的持久集合：harness 场景交出来的镜像在给定状态引擎里就是这样建表的）。
    fn judge_tables_of_one_image(image: &MemoryPool) -> Result<JudgeTables, JudgeTablesRefusal> {
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        build_judge_tables_for_the_given_persisted_sets(
            &JudgeFlow {
                base: image,
                writes: &[],
                segments: &[],
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            vec![Vec::new()],
        )
    }

    /// 系统配置槽那一类落点在哪几处（盘、偏移），升序。
    fn system_configuration_locations_of(tables: &JudgeTables) -> Vec<(u32, u64)> {
        tables
            .locations
            .iter()
            .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
            .map(|location| (location.device, location.offset))
            .collect()
    }

    /// 盘 0 两槽与盘 1 的槽 0 都抹成 0、只剩盘 1 的槽 1 自证得过（CPU 恢复与池级 checker 都择它、照常判）：判表照样建得出，
    /// 落点几何取的就是那一槽——单元区起点是 journal 环末尾的下一槽，两块盘各有偏移 0 与槽距那两个系统配置槽的落点。
    /// 只读第一块盘槽 0 的取法在这里交 `BaseSystemConfigurationUnreadable`，这条红。
    #[test]
    fn the_judge_tables_take_their_geometry_from_the_one_self_verified_slot_one_of_the_second_device(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for (device, offset) in [
            (DeviceIdentity(0), 0),
            (DeviceIdentity(0), spacing),
            (DeviceIdentity(1), 0),
        ] {
            image
                .devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes]);
        }
        let tables = judge_tables_of_one_image(&image)
            .expect("盘 1 的槽 1 自证得过：判表建得出，不报基线系统配置读不出");
        assert_eq!(
            tables.unit_area_start_in_bytes,
            slot_after_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES).0 * SLOT_BYTES,
            "单元区起点取自盘 1 槽 1 记的环长"
        );
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
            "两块盘各有偏移 0 与槽距（盘 1 槽 1 记的 {spacing}）两个系统配置槽的落点"
        );
    }

    /// 两块盘的槽 0 都抹成 0、槽距处本池的槽 1 照旧，两倍槽距处另有一份世代号更大、记的槽距就是两倍槽距的槽 1（自证得过、读者收）：
    /// 恢复逐档读槽 1、取世代号最大的那一份（GPU 两个内核同），落点几何取的就是它（槽距是两倍槽距），与恢复择的逐项相同。
    /// 判别力（`crates/mutations.tsv`）：落点几何照旧取第一份自证得过的（槽距处那一份），几何与恢复、GPU 不同源，这条红。
    #[test]
    fn the_judge_tables_take_their_geometry_from_the_slot_one_the_recovery_reader_chooses_by_its_generation(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let farther_spacing = 2 * spacing;
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut farther = SystemConfiguration::parse_slot(
                &PoolReader::read(&image, device, DeviceOffsetInBytes(spacing), slot_bytes)
                    .expect("槽 1 读得到"),
            )
            .expect("mkfs 写下的槽 1 自证得过");
            farther.immutable.sizes.fixed_structure_slot_spacing =
                u32::try_from(farther_spacing).expect("两倍槽距装得进 4 字节");
            farther.quantities.slot_generation += 10;
            let pool_device = image.devices.get_mut(&device).expect("池里有这块盘");
            pool_device.write(DeviceOffsetInBytes(farther_spacing), &farther.to_slot());
            pool_device.write(DeviceOffsetInBytes(0), &vec![0u8; slot_bytes]);
        }
        let chosen_by_the_recovery_reader =
            singlefs_core::recovery::choose_system_configuration(&image)
                .expect("恢复逐档读槽 1 择得出");
        assert_eq!(
            u64::from(
                chosen_by_the_recovery_reader
                    .immutable
                    .sizes
                    .fixed_structure_slot_spacing
            ),
            farther_spacing,
            "前提：恢复取世代号更大的那一份（两倍槽距那一槽）"
        );
        let geometry = super::base_geometry(&image).expect("基线的落点几何取得出");
        assert_eq!(
            geometry.fixed_structure_slot_spacing, farther_spacing,
            "落点几何取恢复择的那一份：槽距是两倍槽距"
        );
    }

    /// 两块盘各两槽都抹成 0：一份自证得过的系统配置都没有，判表交 `BaseSystemConfigurationUnreadable`、不建表。
    #[test]
    fn the_judge_tables_are_refused_when_no_system_configuration_slot_of_any_device_verifies_itself(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for offset in [0, spacing] {
                image
                    .devices
                    .get_mut(&device)
                    .expect("池里有这块盘")
                    .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes]);
            }
        }
        assert_eq!(
            judge_tables_of_one_image(&image).err(),
            Some(JudgeTablesRefusal::BaseSystemConfigurationUnreadable),
            "每块盘两槽都自证不过：落点几何无从取"
        );
    }

    /// 只剩盘 1 的池（盘 0 整块不在池里，而系统配置记的根环区域 0 与 2 在盘 0 上）：判表建得出、不 panic，
    /// 盘 0 上一个落点都不建（CPU 读那块盘交 None，GPU 在那块盘上找不到落点、读到没有版本的切片），盘 1 上的根环区域 1 照建。
    /// 给区域盘一律建落点的取法从基线读盘 0 的字节读不出，在「落点在盘内、按扇区对齐」那一处 panic，这条红。
    #[test]
    fn pool_without_a_device_that_holds_root_ring_regions_builds_its_tables_with_no_location_on_that_device(
    ) {
        let mut image = formatted_pool_of_two_devices();
        image
            .devices
            .remove(&DeviceIdentity(0))
            .expect("mkfs 之后池里有盘 0");
        let tables = judge_tables_of_one_image(&image)
            .expect("只剩盘 1 的池：判表建得出（盘 1 的系统配置槽自证得过）");
        assert_eq!(tables.devices, vec![1], "池里只有盘 1");
        assert!(
            tables.locations.iter().all(|location| location.device == 1),
            "落点都在盘 1 上：{:?}",
            tables
                .locations
                .iter()
                .map(|location| (location.device, location.offset, location.kind))
                .filter(|(device, _, _)| *device != 1)
                .collect::<Vec<_>>()
        );
        assert!(
            tables
                .locations
                .iter()
                .any(|location| location.kind == JudgeLocationKind::RootSlot),
            "盘 1 上的根环区域 1 照建落点"
        );
    }

    /// 枚举写表里一次 discard 按 64 槽整段发、罩住它之前写下的一份数据单元（C829（discard 与 mkfs 清零在段种类串里同名） 把它从清零里分出来之后，
    /// GPU 判器的落点模型接它）：判表建得出；discard 那一行不落在任何一个落点上，它罩住的 64 个单元槽各是一个落点、各把它记进自己的写；
    /// 那份数据单元的落点照 D13（验证路线） 已定项 4 的三态各有一个版本——discard 持久那一版读回全 0、没持久那一版是数据单元的字节、
    /// 撕裂那一版是撕裂镜像（`crash::torn_image_of_in_place_overwrite` 按「这次写之前的写全落了」时的旧字节算的）在这一槽上的那一截。
    /// 照旧在建表时拒 discard，或把 discard 当成落在它起点那一个落点上（长过那个落点就拒），这条在建表那一步就红。
    #[test]
    fn discard_over_a_data_unit_lands_in_every_unit_slot_location_it_covers_in_its_three_states() {
        let base = formatted_pool_of_two_devices();
        let unit_area_start_slot = slot_after_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES).0;
        let discard_slots: u64 = 64;
        let discard_start = (unit_area_start_slot + discard_slots) * SLOT_BYTES;
        let data_unit_bytes: Vec<u8> = (0..32768_u32)
            .map(|index| u8::try_from(index % 251 + 1).expect("小于 256"))
            .collect();
        let writes = vec![
            RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(discard_start),
                contents: WrittenContents::Bytes(data_unit_bytes.clone()),
            },
            RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::Discard,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(discard_start),
                contents: WrittenContents::Zeros {
                    length: discard_slots * SLOT_BYTES,
                },
            },
        ];
        let segments = vec![vec![0], vec![1]];
        let tables = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &crate::crash::full_expansion,
        })
        .expect("带 discard 的流建得出判表");
        assert_eq!(tables.writes[1].kind, StepKind::Discard);
        assert_eq!(
            tables.writes[1].location, None,
            "discard 不落在任何一个落点上"
        );
        assert!(
            tables.is_tearable[1],
            "长于一个扇区、罩住非零旧内容的 discard 取第三态"
        );
        let torn_image_index = u32::try_from(writes.len()).expect("装得进 u32");
        assert_eq!(
            tables.writes[2].kind,
            StepKind::Discard,
            "枚举写表里接在录制流后面的那一条是 discard 的撕裂镜像"
        );
        let unit_slot_locations_the_discard_covers: Vec<u64> = tables
            .locations
            .iter()
            .filter(|location| {
                location.kind == JudgeLocationKind::UnitSlot
                    && location.device == 0
                    && location.offset >= discard_start
                    && location.writes.contains(&1)
            })
            .map(|location| location.offset)
            .collect();
        let slots_the_discard_starts_in: Vec<u64> = (0..discard_slots)
            .map(|slot| discard_start + slot * SLOT_BYTES)
            .collect();
        assert_eq!(
            unit_slot_locations_the_discard_covers, slots_the_discard_starts_in,
            "discard 罩住的每个单元槽各是一个落点、各把它记进自己的写"
        );
        let data_unit_location = tables
            .locations
            .iter()
            .find(|location| location.device == 0 && location.offset == discard_start)
            .expect("数据单元那一槽是一个落点");
        assert_eq!(
            data_unit_location.writes,
            vec![0, 1, torn_image_index],
            "数据单元那一槽的写：数据单元、discard、discard 的撕裂镜像"
        );
        let version_with = |persisted_bits: u64| {
            let mut combination =
                crate::crash_judge_tables::CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
            for position in 0..data_unit_location.writes.len() {
                if (persisted_bits >> position) & 1 == 1 {
                    combination.mark_persisted(position);
                }
            }
            data_unit_location
                .versions
                .iter()
                .find(|version| version.combination == combination)
                .unwrap_or_else(|| panic!("数据单元那一槽有组合 {persisted_bits:#b} 的版本"))
        };
        assert!(
            version_with(0b011).bytes.iter().all(|byte| *byte == 0),
            "discard 持久那一态读回全 0"
        );
        assert_eq!(
            version_with(0b001).bytes,
            data_unit_bytes,
            "discard 没持久那一态是数据单元的字节"
        );
        let mut old_bytes_under_the_discard = data_unit_bytes.clone();
        old_bytes_under_the_discard.resize(
            usize::try_from(discard_slots * SLOT_BYTES).expect("装得进 usize"),
            0,
        );
        let torn_image = crate::crash::torn_image_of_in_place_overwrite(
            &old_bytes_under_the_discard,
            &WrittenContents::Zeros {
                length: discard_slots * SLOT_BYTES,
            },
        );
        assert_eq!(
            version_with(0b101).bytes,
            torn_image[..data_unit_bytes.len()],
            "撕裂那一态是撕裂镜像在这一槽上的那一截"
        );
    }

    /// 两块盘的槽 0 都抹成 0、4096 处本池的槽 1 照旧，8192 处放一份 journal 环起点写成 1025 的槽（自证得过、读者不收；
    /// harness 场景「8192 处那份槽带不收之值」的造法）：全池没有一个可择的槽 0，GPU 两个内核逐档读槽 1，读得到 8192 处那一份才判得出 I-7.13。
    /// 判表在偏移 0 与几何槽距 4096 之外，给每块盘的 8192 建一个系统配置落点；4096 与 8192 那两槽之间各档（4608 … 7680、8704 … 11776）
    /// 碰过而哪一版都不自证，建完版本就去掉。只建偏移 0 与几何槽距两处的取法少了 8192，这条红；碰过的都留着的取法多出那几档，这条也红。
    #[test]
    fn slot_one_carrying_a_refused_value_found_while_reading_every_spacing_gets_a_system_configuration_location(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let refused_offset = 2 * spacing;
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut refused = SystemConfiguration::parse_slot(
                &PoolReader::read(&image, device, DeviceOffsetInBytes(spacing), slot_bytes)
                    .expect("槽 1 读得到"),
            )
            .expect("mkfs 写下的槽 1 自证得过");
            refused.immutable.journal_ring_start_slot =
                SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
            let pool_device = image.devices.get_mut(&device).expect("池里有这块盘");
            pool_device.write(DeviceOffsetInBytes(refused_offset), &refused.to_slot());
            pool_device.write(DeviceOffsetInBytes(0), &vec![0u8; slot_bytes]);
        }
        let tables = judge_tables_of_one_image(&image).expect("4096 处的槽 1 自证得过：判表建得出");
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![
                (0, 0),
                (0, spacing),
                (0, refused_offset),
                (1, 0),
                (1, spacing),
                (1, refused_offset)
            ],
            "两块盘各有偏移 0、槽距 {spacing} 与逐档读到的 {refused_offset} 三个系统配置槽的落点，别的几档不留"
        );
    }

    /// io_min 8192（槽距 8192）的池，盘 1 槽 1 那 4096 字节整段清掉（稀疏盘上删掉那几个扇区：基线没写过、读出全 0），别处照 mkfs：
    /// 判表的几何取恢复择的那一份（槽距 8192），每块盘偏移 0 与几何槽距两处系统配置落点照建（[`super::location_is_read_by_the_judge`]：
    /// 这两处照留，判器按它们读槽 0 与槽 1），盘 1 的 8192 那一处没写过、不由逐档那一圈补出，只由几何槽距那一处建。
    /// 判别力（`crates/mutations.tsv`「E 崩溃点重放 J14 判表的系统配置槽 1 照旧摆在 4096」那一行）：槽 1 照旧摆在 4096，盘 1 的 8192 没有落点、
    /// 4096 那一处一版都不自证被去掉，这条红。盘上两槽都写过的池上逐档那一圈补出 8192，那一处改法看不出来。
    #[test]
    fn pool_with_minimum_input_output_bytes_8192_whose_slot_one_on_one_device_was_never_written_keeps_a_system_configuration_location_at_the_slot_spacing(
    ) {
        let mut image = formatted_pool_of_two_devices_with_minimum_input_output_bytes(8192);
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(8192));
        assert_eq!(spacing, 8192, "io_min 8192 的池槽距 8192");
        let device_one = image
            .devices
            .get_mut(&DeviceIdentity(1))
            .expect("池里有盘 1");
        device_one.zero_fill(
            DeviceOffsetInBytes(spacing),
            SYSTEM_CONFIGURATION_SLOT_BYTES,
        );
        assert!(
            device_one
                .written_sectors_in(
                    DeviceOffsetInBytes(spacing),
                    SYSTEM_CONFIGURATION_SLOT_BYTES
                )
                .is_empty(),
            "盘 1 槽 1 那 4096 字节没有写过的扇区"
        );
        let tables = judge_tables_of_one_image(&image)
            .expect("两块盘的槽 0 与盘 0 的槽 1 自证得过：判表建得出");
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
            "两块盘各有偏移 0 与几何槽距 {spacing} 两个系统配置槽的落点，盘 1 那一槽没写过也建"
        );
    }

    /// 用过的盘上重做 mkfs 那条流的 journal 环长：16 MiB（每块盘 4096 个记录槽），第一版默认的 768 MiB 照样建得出表，只是撕裂镜像大。
    const SMALL_JOURNAL_RING_BYTES: u64 = 16 << 20;

    /// journal 环 16 MiB、io_min 512（槽距 4096、物理块 512；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上）的 mkfs 参数。
    fn make_filesystem_parameters_with_a_small_journal_ring(
        filesystem_identifier: [u8; 16],
    ) -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
                journal_ring_bytes: SMALL_JOURNAL_RING_BYTES,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
        }
    }

    /// 旧池：两块空盘按 [`make_filesystem_parameters_with_a_small_journal_ring`] mkfs 一次（fsid `singlefs-tables!`），再在两块盘
    /// journal 环的头一个记录槽里各写一扇区非零字节（旧池写过的 journal 记录：只要非零，不要求自证）。
    fn used_pool_with_a_small_journal_ring() -> MemoryPool {
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        SparseBlockDevice::new(
                            FORMATTED_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        ),
                    )
                })
                .collect();
        make_filesystem(
            &make_filesystem_parameters_with_a_small_journal_ring(*b"singlefs-tables!"),
            &mut devices,
        )
        .expect("两块空盘上 mkfs 做得成");
        let mut base = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: FORMATTED_DEVICE_BYTES,
        };
        let old_journal_record_sector: Vec<u8> = (0..512_u32)
            .map(|index| u8::try_from(index % 251 + 1).expect("小于 256"))
            .collect();
        for device in base.devices.values_mut() {
            device.write(
                DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
                &old_journal_record_sector,
            );
        }
        base
    }

    /// 在 `base` 上经内层入口拿另一个 fsid（`remade-tables!!!`，前 8 字节与旧池不同）重做 mkfs，交回录下的写与段。
    fn make_filesystem_over_the_used_pool(
        base: &MemoryPool,
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = base
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device =
                    SparseBlockDevice::new(FORMATTED_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                device.image = sparse.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
                )
            })
            .collect();
        let parameters = make_filesystem_parameters_with_a_small_journal_ring(*b"remade-tables!!!");
        make_filesystem(&parameters, &mut devices).expect("用过的盘上重做 mkfs 做得成");
        writes_and_segments(
            &stream.retained_operations(),
            &FixedGeometry {
                fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
                journal_ring_bytes: parameters.geometry.journal_ring_bytes,
                root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
            },
        )
    }

    /// 用过的盘上重做 mkfs（两趟写序：两块盘的 [0, 根环基址) 清零自成一段，其余八次清零与四个单元写一段，三条第 0 代根各一段，四个系统配置槽一段）
    /// 按甲二展开建判表：清零写不落在哪一个落点上；它罩住的每个落点各把它记进自己的写；它不给自己建落点（journal 环里只有旧池写过的那一槽是落点）；
    /// 任一个状态里读得出非零字节的扇区（基线写过的、或枚举写表里哪一次写写下了非零字节的）都在某个落点里；每个状态、每个落点按持久掩码查到的版本
    /// 与崩溃镜像在那里读出的字节逐字节相同。
    /// 判别力（`crates/mutations.tsv`「C656 GPU 判表收清零写」那两行）：建落点那一步漏记清零写罩住的落点，作废之后系统配置槽那一版照旧是旧池的字节，
    /// 逐字节比那一句红；清零写照旧算碰过，journal 环里每一槽都建落点，落点那一句红。照旧在建表时拒清零写，建表那一步就红。
    #[test]
    fn zero_fills_of_a_make_filesystem_over_a_used_pool_land_in_every_location_they_cover_and_add_no_location_of_their_own(
    ) {
        let base = used_pool_with_a_small_journal_ring();
        let (writes, segments) = make_filesystem_over_the_used_pool(&base);
        assert_eq!(
            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
            vec![2, 12, 1, 1, 1, 4],
            "两趟写序的段序列"
        );
        let zero_fills: Vec<usize> = writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::ZeroFill)
            .map(|(write_index, _)| write_index)
            .collect();
        assert_eq!(zero_fills.len(), 10, "每块盘五次清零，两块盘十次");
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("mkfs 写第 0 代根");
        let tables = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index,
            versions: &[],
            expansion: &crate::crash::in_place_subsets_with_copy_on_write_none_or_all_expansion,
        })
        .expect("用过的盘上重做 mkfs 那条流建得出判表");
        let enumerated = crate::crash::crash_state_enumeration_tables(
            &base,
            &writes,
            &segments,
            &crate::crash::in_place_subsets_with_copy_on_write_none_or_all_expansion,
        )
        .enumerated_writes;
        let zero_fill_entries: Vec<usize> = enumerated
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::ZeroFill)
            .map(|(entry, _)| entry)
            .collect();
        assert!(
            zero_fill_entries.len() > zero_fills.len(),
            "罩住旧内容的清零写取第三态，枚举写表里接着它们的撕裂镜像"
        );
        for entry in &zero_fill_entries {
            assert_eq!(
                tables.writes[*entry].location, None,
                "清零写（或它的撕裂镜像）{entry} 不落在任何一个落点上"
            );
        }
        for location in &tables.locations {
            let location_end = location.offset + u64::from(location.length);
            for entry in &zero_fill_entries {
                let write = &enumerated[*entry];
                let covers = write.device.0 == location.device
                    && write.offset.0 < location_end
                    && location.offset < write.offset.0 + write.length_in_bytes();
                assert_eq!(
                    location
                        .writes
                        .contains(&u32::try_from(*entry).expect("装得进 u32")),
                    covers,
                    "落点 (盘 {}, 偏移 {}, {:?}) 的写里有没有清零写 {entry}",
                    location.device,
                    location.offset,
                    location.kind
                );
            }
        }
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let journal_record_locations: Vec<(u32, u64)> = tables
            .locations
            .iter()
            .filter(|location| location.kind == JudgeLocationKind::JournalRecord)
            .map(|location| (location.device, location.offset))
            .collect();
        assert_eq!(
            journal_record_locations,
            vec![(0, ring_start), (1, ring_start)],
            "清零写不给自己建落点：journal 环 {} 个记录槽里只有旧池写过的那一槽是落点",
            SMALL_JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES
        );
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        let is_inside_some_location = |device: u32, sector_start: u64| {
            tables.locations.iter().any(|location| {
                location.device == device
                    && location.offset <= sector_start
                    && sector_start + SECTOR_BYTES <= location.offset + u64::from(location.length)
            })
        };
        for (device, sparse) in &base.devices {
            for (sector, bytes) in sparse.written_sectors() {
                if bytes.iter().any(|byte| *byte != 0) {
                    assert!(
                        is_inside_some_location(device.0, sector * SECTOR_BYTES),
                        "基线在盘 {} 第 {sector} 扇区有非零字节，它在某个落点里",
                        device.0
                    );
                }
            }
        }
        for (entry, write) in enumerated.iter().enumerate() {
            let WrittenContents::Bytes(bytes) = &write.contents else {
                continue;
            };
            for (sector_in_the_write, sector) in bytes.chunks(sector_bytes).enumerate() {
                if sector.iter().any(|byte| *byte != 0) {
                    let sector_start = write.offset.0
                        + u64::try_from(sector_in_the_write).expect("扇区号") * SECTOR_BYTES;
                    assert!(
                        is_inside_some_location(write.device.0, sector_start),
                        "枚举写 {entry}（{:?}）在盘 {} 偏移 {sector_start} 写下非零字节，那一扇区在某个落点里",
                        write.kind,
                        write.device.0
                    );
                }
            }
        }
        let mut versions_checked = 0u64;
        for ordinal in 0..tables.state_count {
            let persisted = tables.persisted_writes_of_state(ordinal);
            let image = CrashImage {
                base: &base,
                writes: &enumerated,
                persisted: persisted.clone(),
            };
            for location in &tables.locations {
                let version = tables.version_of(location, &persisted).unwrap_or_else(|| {
                    panic!(
                        "状态 {ordinal} 在落点 (盘 {}, 偏移 {}) 上的组合没有版本",
                        location.device, location.offset
                    )
                });
                let on_disk = PoolReader::read(
                    &image,
                    DeviceIdentity(location.device),
                    DeviceOffsetInBytes(location.offset),
                    usize::try_from(location.length).expect("长度"),
                )
                .expect("落点在盘内");
                assert!(
                    version.bytes == on_disk,
                    "状态 {ordinal} 落点 (盘 {}, 偏移 {}, {:?}) 的版本字节与崩溃镜像不同",
                    location.device,
                    location.offset,
                    location.kind
                );
                versions_checked += 1;
            }
        }
        assert!(versions_checked > 0, "真比过版本");
        println!(
            "CRASH_JUDGE_TABLES_ZERO_FILLS states={} locations={} versions_checked={versions_checked} most_writes_on_one_location={}",
            tables.state_count,
            tables.locations.len(),
            tables
                .locations
                .iter()
                .map(|location| location.writes.len())
                .max()
                .unwrap_or(0)
        );
    }
}
