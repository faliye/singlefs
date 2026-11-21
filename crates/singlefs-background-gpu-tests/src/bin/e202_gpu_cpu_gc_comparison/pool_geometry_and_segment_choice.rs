//! 造起步镜像的池参数与几何：分配记录、单元区、碎片段，与产品 greedy 会挑的段。

use super::*;

// ---- 造起步镜像（登记第五节「负载：池怎么造」）----

/// 默认环长：从第一版的环起点到默认环长下单元区起点之间那么多字节（照核心层的两个起点算，同产品 GPU 包测试共用模块）。
fn default_journal_ring_bytes() -> u64 {
    UnitAreaStart::of_the_default_journal_ring()
        .slot()
        .to_device_offset()
        .0
        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
            .to_device_offset()
            .0
}

/// mkfs 参数：照产品 GPU 包测试共用模块的 `parameters()`（region 设备 0、1、0，默认环长，512 字节扇区），fsid 换成本实验的。
pub(super) fn make_filesystem_parameters_of_the_pool() -> MakeFilesystemParameters {
    let sector = u32::try_from(SECTOR_BYTES).expect("512");
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: sector,
            minimum_input_output_bytes: sector,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(sector),
            journal_ring_bytes: default_journal_ring_bytes(),
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 有树的那一版（第一个文件发过之后每一版都有树）。
pub(super) fn version_of_trees(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("第一个文件发过之后，会话的现行那一版都有树")
}

/// mkfs → 取号 → 暖机 → 新池新建文件（同产品 GPU 包测试共用模块 `pool_after_a_sequential_write_in_an_earlier_mount` 那一串），块照 `storage` 存。
pub(super) fn new_pool_with_the_first_file(
    device_bytes: u64,
    storage: ChunkStorage,
) -> PoolDevices {
    let parameters = make_filesystem_parameters_of_the_pool();
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
    let mut devices = empty_pool_devices_storing_the_chunks(device_bytes, &recording, storage);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    allocator.mark_format_time_units(
        genesis.instance_table_placement(),
        genesis.tree_table_genesis_placement(),
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let first_file_content: Vec<u8> = (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let mut writer = PoolWriter::new(&parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    publish_first_file(
        &mut writer,
        &mut allocator,
        warmed_up.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &first_file_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed_up.last_record_bytes,
    )
    .expect("新池新建文件");
    drop(writer);
    devices
}

/// 现行那一版里 inode 号落在 `[first_inode, first_inode + inode_count)` 的文件每个数据单元的槽（盘 0；两盘同槽）：
/// （inode 号，文件里的单元序号）→ 槽。只在造池的几个节点上读一次（扫整份 `units`），不逐次发布读。
pub(super) fn data_unit_slots_of_the_inodes(
    version: &TransactionOutput,
    first_inode: u64,
    inode_count: u64,
) -> BTreeMap<(u64, u64), u64> {
    let inodes = first_inode..first_inode + inode_count;
    version
        .units
        .iter()
        .filter_map(|published| {
            if let TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } =
                published.identity
            {
                inodes
                    .contains(&inode.0)
                    .then_some(((inode.0, unit.0), published.slot.0))
            } else {
                None
            }
        })
        .collect()
}

/// 盘 0 上被未释放记录罩住的槽。
pub(super) fn slots_under_an_unreleased_record_on_device_zero(
    records: &[AllocationRecord],
) -> BTreeSet<u64> {
    records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 盘 0 的单元区（量挂载时选段与活槽段看的那一块）。
pub(super) fn unit_area_of_device_zero(session: &MountedSession) -> UnitAreaOfOneDevice {
    let device_zero = &session.allocator.devices[0];
    UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: device_zero.unit_area_start(),
        slots: device_zero.unit_area_slots(),
    }
}

/// 段的起点槽（单元区起点在聚簇段边界上）。
pub(super) const fn segment_start_of(slot: u64, unit_area_start_slot: u64) -> u64 {
    unit_area_start_slot
        + (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_CLUSTER_SEGMENT_SLOTS
}

/// 几何核的读数（登记第五节「负载」第 4 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GeometryReading {
    /// 造数据挂载里垫片文件与活数据单元交错写满的段（碎片段，P ÷ 32 段），与其中活槽数恰好 32 的段数。
    pub(super) written_segments: u64,
    pub(super) written_segments_with_half_live: u64,
    /// 量挂载重挂之后 greedy 要选的那一段（照核心层签发那一步重放，[`segment_the_greedy_choice_picks`]）：只报，不进判法。
    pub(super) greedy: SegmentOfTheChoice,
    /// 强制选段后备头一条意图点名的那一段（[`fragment_segment_to_force`]，登记第十二节修订第 8 条）：几何核后一半判它。
    pub(super) forced: SegmentOfTheChoice,
    /// 每段活槽数的直方图：活槽数 → 段数。
    pub(super) live_slot_histogram: BTreeMap<u64, u64>,
}

impl GeometryReading {
    /// 几何核的两半都成立（登记第五节「负载」第 4 条，后一半照第十二节修订第 8 条换成强制选定的那一段）：碎片段造成了，
    /// 强制选段后备点名的那一段只装活数据单元、活槽不多于 32。
    pub(super) fn is_formed(&self) -> bool {
        self.fragment_segments_are_formed() && self.forced.is_fragment_like()
    }

    /// 前一半：碎片段里至少九成恰好 32 槽活。
    fn fragment_segments_are_formed(&self) -> bool {
        self.written_segments > 0
            && count_as_float(self.written_segments_with_half_live)
                >= SHARE_OF_WRITTEN_SEGMENTS_WITH_HALF_LIVE_AT_LEAST
                    * count_as_float(self.written_segments)
    }
}

/// 选中的一段的读数（greedy 照核心层重放要选的，或强制选段后备点名的）：起点（一段都没选中是 `None`）、活槽数、未释放记录是不是全是活数据单元（码 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SegmentOfTheChoice {
    pub(super) start_slot: Option<u64>,
    pub(super) live_slots: u64,
    pub(super) holds_only_data_units: bool,
}

impl SegmentOfTheChoice {
    /// 登记第五节「负载」第 4 条的判法：选中了一段，它活槽不多于 32、未释放记录全是活数据单元。
    pub(super) fn is_fragment_like(self) -> bool {
        self.start_slot.is_some()
            && self.live_slots <= LIVE_SLOTS_OF_A_FRAGMENTED_SEGMENT
            && self.holds_only_data_units
    }
}

/// 单元区里装着不是活数据单元的未释放记录的段（按段起点），一遍扫完全部记录。
pub(super) fn segments_holding_a_record_other_than_a_data_unit(
    records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    data_slots_now: &BTreeSet<u64>,
) -> BTreeSet<u64> {
    let unit_area_start_slot = unit_area.start.slot().0;
    records
        .iter()
        .filter(|record| {
            record.device == unit_area.device
                && !record.is_released
                && record.slot.0 >= unit_area_start_slot
                && !data_slots_now.contains(&record.slot.0)
        })
        .map(|record| segment_start_of(record.slot.0, unit_area_start_slot))
        .collect()
}

/// 起点 `start_slot` 那一段的读数。`live_slots_of_each_segment` 按段起点从小到大一段一个（`live_slots_per_cluster_segment_by_the_records`）。
pub(super) fn segment_of_the_choice(
    start_slot: Option<u64>,
    unit_area_start_slot: u64,
    live_slots_of_each_segment: &[u64],
    segments_holding_other_records: &BTreeSet<u64>,
) -> SegmentOfTheChoice {
    let live_slots = start_slot
        .and_then(|start| start.checked_sub(unit_area_start_slot))
        .and_then(|offset| usize::try_from(offset / LOCAL_CLUSTER_SEGMENT_SLOTS).ok())
        .and_then(|segment_index| live_slots_of_each_segment.get(segment_index).copied())
        .unwrap_or(0);
    SegmentOfTheChoice {
        start_slot,
        live_slots,
        holds_only_data_units: start_slot
            .is_some_and(|start| !segments_holding_other_records.contains(&start)),
    }
}

/// 强制选段后备（登记第十二节修订第 8 条）这一次点名的段：碎片段里还没点过名的、只装活数据单元、有活槽而不多于 32 的段里起点最小的那一段
/// （判法同第五节「负载」第 4 条；没有活槽的段核心层签发时照「一个可搬的单元都没有」拒，不点它）。一段都没有（碎片段用完）交回 `None`。
pub(super) fn fragment_segment_to_force(
    fragment_segment_starts: &BTreeSet<u64>,
    segments_already_forced: &BTreeSet<u64>,
    unit_area_start_slot: u64,
    live_slots_of_each_segment: &[u64],
    segments_holding_other_records: &BTreeSet<u64>,
) -> Option<SegmentOfTheChoice> {
    fragment_segment_starts
        .iter()
        .filter(|start| !segments_already_forced.contains(start))
        .map(|start| {
            segment_of_the_choice(
                Some(*start),
                unit_area_start_slot,
                live_slots_of_each_segment,
                segments_holding_other_records,
            )
        })
        .find(|segment| segment.live_slots > 0 && segment.is_fragment_like())
}

/// 几何核的纯函数：盘 0 的记录、单元区、碎片段（起点）、此刻活数据单元的槽、greedy 要选的那一段与强制选段后备点名的那一段（起点）。
pub(super) fn geometry_of_the_records(
    records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    written_segment_starts: &BTreeSet<u64>,
    data_slots_now: &BTreeSet<u64>,
    greedy_segment_start_slot: Option<u64>,
    forced_segment_start_slot: Option<u64>,
) -> GeometryReading {
    let unit_area_start_slot = unit_area.start.slot().0;
    let live_slots = live_slots_per_cluster_segment_by_the_records(records, unit_area);
    let segments_holding_other_records =
        segments_holding_a_record_other_than_a_data_unit(records, unit_area, data_slots_now);
    let mut live_slot_histogram: BTreeMap<u64, u64> = BTreeMap::new();
    let mut written_segments_with_half_live = 0_u64;
    for (segment_start_slot, live) in (0_u64..)
        .map(|segment_index| unit_area_start_slot + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS)
        .zip(live_slots.iter().copied())
    {
        *live_slot_histogram.entry(live).or_insert(0) += 1;
        if written_segment_starts.contains(&segment_start_slot)
            && live == LIVE_SLOTS_OF_A_FRAGMENTED_SEGMENT
        {
            written_segments_with_half_live += 1;
        }
    }
    GeometryReading {
        written_segments: u64::try_from(written_segment_starts.len()).expect("段数"),
        written_segments_with_half_live,
        greedy: segment_of_the_choice(
            greedy_segment_start_slot,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        ),
        forced: segment_of_the_choice(
            forced_segment_start_slot,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        ),
        live_slot_histogram,
    }
}

/// 整理不许搬、罩住它就不选那一段的槽：现行那一版里中央映射树各节点、树表、实例表链各片（核心层 `mounted_session.rs` 私有的
/// `version_as_the_compaction_reads_it` 那一份 `pinned_slots`，这里另写一份）。
fn slots_exempt_from_the_mapping(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version
        .units
        .iter()
        .filter(|published| {
            matches!(
                published.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
                    | TransactionUnit::MappingTree
                    | TransactionUnit::TreeTable
                    | TransactionUnit::InstanceTable
                    | TransactionUnit::InstanceTablePageAfterTheFirst(_)
            )
        })
        .flat_map(|published| {
            (published.slot.0..published.slot.0 + published.identity.span_slots()).map(SlotNumber)
        })
        .collect()
}

/// greedy 此刻要选的那一段（登记第五节「负载」第 4 条；第十二节修订第 8 条要每条意图照打）：照核心层签发那一步重放 `choose_region` 与「先扫」那一圈
/// （`mounted_session.rs` 签发那一支，判据 Greedy）。`segments_opened_by_this_mount` 是这次挂载开过的聚簇段（重挂之后还没开过的取空），
/// `refused` 是会话此刻记着的扫描拒过的段；钉住的槽与常驻隔离集合起来是不许选的槽；读单元从两块内存盘读。一段都没资格交回 `None`。
pub(super) fn segment_the_greedy_choice_picks(
    version: &TransactionOutput,
    unit_area: UnitAreaOfOneDevice,
    segments_opened_by_this_mount: &BTreeSet<SlotNumber>,
    refused: SegmentsRefusedByTheRegionScan,
    quarantined_slots: &BTreeSet<SlotNumber>,
    devices: &PoolDevices,
) -> Option<u64> {
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
    let pinned_slots = slots_exempt_from_the_mapping(version);
    let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> =
        pinned_slots.union(quarantined_slots).copied().collect();
    let reader = ReaderOfTheMemoryPool {
        devices: devices.as_slice(),
    };
    let mut region_scans = region_scans_of_the_segments_without_an_unreleased_record(
        &version.allocation_records,
        unit_area,
    );
    let mut refused = refused;
    // 迭代上界：每答一次「先扫」下一次就多一段有结论，不超过单元区的整段数（同核心层那一圈）。
    for _choice in 0..=unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS {
        match choose_region(
            &version.allocation_records,
            unit_area,
            segments_opened_by_this_mount,
            None,
            &slots_that_disqualify_a_segment,
            &region_scans,
            &refused,
            RegionChoiceCriterion::Greedy,
        ) {
            RegionChoice::Chosen(region) => return Some(region.start_slot().0),
            RegionChoice::ScanBeforeChoosing(region) => match scan_region(
                region,
                &version.allocation_records,
                &mapping_entries,
                &pinned_slots,
                quarantined_slots,
                |device, slot: SlotNumber, length| {
                    reader.read(device, slot.to_device_offset(), length)
                },
            ) {
                Ok(scan) => region_scans.push(scan),
                Err(refusal) => refused.note_refusal(region, refusal),
            },
            RegionChoice::NoEligibleSegment => return None,
        }
    }
    None
}

/// 一档的起步镜像：两块内存盘（关掉、没卸载）、活数据文件那 P ÷ K 个 inode 从几号起、每个单元最后一次写的是第几次、碎片段，与造的经过里要报的数。
/// 活数据单元第 u 个住 inode `lowest_live_data_inode + u ÷ K`，是那个文件里第 u mod K 个单元（[`file_and_index_of_a_live_data_unit`]）。
pub(super) struct StartingImage {
    pub(super) tier: UnitAreaTier,
    pub(super) devices: PoolDevices,
    pub(super) lowest_live_data_inode: u64,
    /// 交错那一段写满的段（碎片段，按段起点）：强制选段后备从这里点名（登记第十二节修订第 8 条）。
    pub(super) fragment_segment_starts: BTreeSet<u64>,
    pub(super) write_generation_of_each_unit: Vec<u64>,
    pub(super) next_write_time_seconds: u64,
    pub(super) construction: ConstructionReport,
}

#[derive(Clone, Debug)]
pub(super) struct ConstructionReport {
    pub(super) construction_seconds: f64,
    pub(super) publishes: u64,
    pub(super) floor_raises: u64,
    pub(super) release_rounds: u64,
    /// 垫片文件删掉之后，它每个单元的旧落点都按记录已释放。
    pub(super) is_every_spacer_slot_released: bool,
    /// 垫片文件的单元数（补齐文件的 a 个加交错垫片文件的 P ÷ 2 个），其中补齐文件的 a 个（不算进碎片段）。
    pub(super) spacer_units: u64,
    pub(super) alignment_spacer_units: u64,
    /// 文件的条数（活数据文件与交错垫片文件各 K 个单元，补齐文件 a 个，一层 inode 树装得下）：活数据文件 P ÷ K 个、
    /// 垫片文件 P ÷ 32 + 1 个（交错垫片文件加补齐文件）、造数据挂载之前树里已有的 inode 记录，与造数据挂载写完那一刻（造池的峰）树里的记录条数与叶容器片数。
    pub(super) live_data_files: u64,
    pub(super) spacer_files: u64,
    pub(super) inode_records_before_the_data_mount: u64,
    pub(super) inode_records_at_the_peak: u64,
    pub(super) inode_leaf_containers_at_the_peak: u64,
    /// 造数据挂载里每一次数据写类调用写进的单元数 → 这样的调用有几次（登记第十二节修订第 14 条：一次发布写满一个文件，K 个或补齐文件的 a 个）。
    pub(super) data_writes_by_units: BTreeMap<u64, u64>,
    /// 造数据挂载各阶段登记的发布数（停机条款第 8 条核过，各阶段量到的在 [`Self::phases`] 里）。
    pub(super) registered_publishes_of_the_data_mount: RegisteredPublishesOfTheDataMount,
    /// 造完那一版里数到的活数据单元个数（应等于 P）。
    pub(super) live_data_units_found: u64,
    pub(super) empty_segments: u64,
    pub(super) mapped_units: u64,
    pub(super) geometry: GeometryReading,
    /// 碎片段里活槽怎么排（修订第 14 条「几何核」追加的读数，只报不判）。
    pub(super) fragment_live_slot_layout: LiveSlotLayoutOfTheFragmentSegments,
    /// 造完那一刻本进程常驻内存的峰（`/proc/self/status` 的 `VmHWM`，修订第 14 条量 B 用时与内存那一行）；读不到是 `None`。
    pub(super) peak_resident_bytes_of_the_process: Option<u64>,
    /// 各阶段量到的（[`ConstructionClock`]），按造的先后一阶段一项。
    pub(super) phases: Vec<TimingOfAConstructionPhase>,
}

#[cfg(test)]
mod tests;
