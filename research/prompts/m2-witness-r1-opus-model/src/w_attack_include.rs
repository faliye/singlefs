// ============================================================================
// m2-witness-r1 云端攻方腿原型驱动（只在副本上）。接在 E158 的 H3 枚举与 P332 判定上：
// 故障不只点名根槽，也点名见证副本（`singlefs_core::witness::witness_copy_locations`）；
// 物理块由 E158_PBS（512 / 4096）选；放处与写序由 SINGLEFS_WITNESS_PLACEMENT / SINGLEFS_WITNESS_ORDER 选。
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::witness;

fn physical_block_size_of_this_run() -> u32 {
    match env::var("E158_PBS").as_deref() {
        Ok("4096") => 4096,
        Ok("512") | Err(_) => 512,
        Ok(other) => panic!("E158_PBS={other} 不认识"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FaultItem {
    RootSlot { region: u64, slot: u64 },
    WitnessCopy { device: u32, offset: u64, length: u64 },
}

impl FaultItem {
    fn range(&self) -> (DeviceIdentity, u64, u64) {
        match *self {
            FaultItem::RootSlot { region, slot } => {
                let device = region_devices()[usize::try_from(region).expect("区域")];
                let start = singlefs_core::root_ring::slot_offset(RootRingSlot { region, slot }, 4096).0;
                (device, start, start + 4096)
            }
            FaultItem::WitnessCopy { device, offset, length } => (DeviceIdentity(device), offset, offset + length),
        }
    }
    fn label(&self) -> String {
        match *self {
            FaultItem::RootSlot { region, slot } => format!("R{region}.{slot}"),
            FaultItem::WitnessCopy { device, offset, .. } => format!("W{device}@{offset}"),
        }
    }
}

fn witness_fault_items() -> Vec<FaultItem> {
    witness::witness_copy_locations(&[DeviceIdentity(0), DeviceIdentity(1)], 4096)
        .into_iter()
        .map(|location| FaultItem::WitnessCopy { device: location.device.0, offset: location.offset, length: location.length })
        .collect()
}

fn overlaps(ranges: &[(DeviceIdentity, u64, u64)], device: DeviceIdentity, start: u64, end: u64) -> bool {
    ranges.iter().any(|(d, s, e)| *d == device && start < *e && *s < end)
}

struct RangeFaultReader<'a> {
    inner: &'a dyn singlefs_core::recovery::PoolReader,
    ranges: Vec<(DeviceIdentity, u64, u64)>,
}

impl singlefs_core::recovery::PoolReader for RangeFaultReader<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        if overlaps(&self.ranges, device, offset.0, offset.0 + length as u64) {
            return None;
        }
        self.inner.read(device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 可写挂载要块设备：读落在点名范围里就报错，写照常（写不修好读——与 E158 的根槽开关同义：点名的位置读不出）。
struct RangeFailingDevice {
    inner: SparseBlockDevice,
    identity: DeviceIdentity,
    ranges: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>>,
}

impl BlockDevice for RangeFailingDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if overlaps(&self.ranges.borrow(), self.identity, offset.0, offset.0 + buffer.len() as u64) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一个故障集合在一个节点上的 P332 三处判定（与 E158 `evaluate_violation_checkpoints` 同形，故障换成按范围点名）。
fn evaluate_with_fault_items(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    faults: &[FaultItem],
    baseline_chosen_root: Option<TimelineRoot>,
    pool_for_cold_recover: &MemoryPool,
) -> ViolationCheckpoints {
    let ranges: Vec<(DeviceIdentity, u64, u64)> = faults.iter().map(FaultItem::range).collect();
    let reader = RangeFaultReader { inner: pool_for_cold_recover, ranges: ranges.clone() };
    let cold_recover_outcome = recover(&reader, JournalPolicy::Consult).outcome;
    let cold_recover_chosen_root = chosen_root_of(&cold_recover_outcome);
    let cold_recover_with_fault = violation_at_checkpoint(node, &cold_recover_outcome);
    let cold_recover_with_fault_failed = cold_recover_with_fault.is_none();
    if !faults.is_empty() && cold_recover_chosen_root == baseline_chosen_root {
        return ViolationCheckpoints {
            cold_recover_with_fault,
            mount_then_recover_with_fault: Some(ViolationAtOneCheckpoint::default()),
            recover_after_fault_removed: Some(ViolationAtOneCheckpoint::default()),
            cold_recover_with_fault_failed,
            mount_writable_with_fault_failed: false,
            skipped_by_the_reduction_rule: true,
        };
    }
    let shared = Rc::new(RefCell::new(ranges));
    let mut faulted_devices: Vec<(DeviceIdentity, RangeFailingDevice)> = devices_from_pool(&node.pool, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, device)| (identity, RangeFailingDevice { inner: device, identity, ranges: shared.clone() }))
        .collect();
    let mount_result = mount_writable(parameters, &mut faulted_devices);
    let (mount_then_recover_with_fault, mount_writable_with_fault_failed) = match mount_result {
        Ok(_) => (violation_at_checkpoint(node, &recover(&faulted_devices, JournalPolicy::Consult).outcome), false),
        Err(_) => (None, true),
    };
    shared.borrow_mut().clear();
    let recover_after_fault_removed = violation_at_checkpoint(node, &recover(&faulted_devices, JournalPolicy::Consult).outcome);
    ViolationCheckpoints {
        cold_recover_with_fault,
        mount_then_recover_with_fault,
        recover_after_fault_removed,
        cold_recover_with_fault_failed,
        mount_writable_with_fault_failed,
        skipped_by_the_reduction_rule: false,
    }
}

fn violation_flags(checkpoints: &ViolationCheckpoints) -> String {
    let flag = |point: &Option<ViolationAtOneCheckpoint>| match point {
        None => "failed".to_string(),
        Some(p) => format!("{}{}", if p.chosen_root_abandoned { "V1" } else { "" }, if p.content_off_timeline { "V2" } else { "" }),
    };
    format!(
        "a={} b={} c={}",
        flag(&checkpoints.cold_recover_with_fault),
        flag(&checkpoints.mount_then_recover_with_fault),
        flag(&checkpoints.recover_after_fault_removed)
    )
}

fn any_violation(checkpoints: &ViolationCheckpoints) -> bool {
    [&checkpoints.cold_recover_with_fault, &checkpoints.mount_then_recover_with_fault, &checkpoints.recover_after_fault_removed]
        .iter()
        .any(|point| point.is_some_and(|p| p.chosen_root_abandoned || p.content_off_timeline))
}

#[allow(clippy::too_many_arguments)]
fn enumerate_witness_family(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
    maximum_weight: usize,
    overwrites_before_sigma: u64,
    summary: &mut FaultSetViolationSummary,
) {
    summary.nodes_visited += 1;
    let mut universe: Vec<FaultItem> = readable_roots_with_slots(&node.pool, geometry_view)
        .iter()
        .map(|(region, slot, _)| FaultItem::RootSlot { region: *region, slot: *slot })
        .collect();
    universe.extend(witness_fault_items());
    let has_rollback = !node.rollback_events.is_empty();
    let baseline_chosen_root = Some(*node.timeline.last().expect("非空"));
    for weight in 0..=maximum_weight {
        for combo in combinations_of_size(&universe, weight) {
            let checkpoints = evaluate_with_fault_items(node, parameters, &combo, baseline_chosen_root, &node.pool);
            record_violation_checkpoints_for_one_fault_set(summary, u32::try_from(weight).expect("w"), has_rollback, &checkpoints);
            if any_violation(&checkpoints) {
                let witness_faults = combo.iter().filter(|item| matches!(item, FaultItem::WitnessCopy { .. })).count();
                println!(
                    "VKEY ob={overwrites_before_sigma} path={:?} F=[{}] weight={weight} witness_faults={witness_faults} {}",
                    node.path,
                    combo.iter().map(FaultItem::label).collect::<Vec<_>>().join(","),
                    violation_flags(&checkpoints)
                );
            }
        }
    }
    if depth_remaining == 0 {
        return;
    }
    if node.session.is_some() {
        if let Ok(next) = apply_overwrite(node, parameters) {
            enumerate_witness_family(&next, depth_remaining - 1, parameters, geometry_view, maximum_weight, overwrites_before_sigma, summary);
        }
    }
    if let Ok(next) = apply_mount_writable(node, parameters) {
        enumerate_witness_family(&next, depth_remaining - 1, parameters, geometry_view, maximum_weight, overwrites_before_sigma, summary);
    }
    for readable_root in &readable_roots_independent(&node.pool, geometry_view) {
        let target = RollbackTarget { instance: InstanceGeneration(readable_root.0), checkpoint_txg: CheckpointTxg(readable_root.1) };
        if let Ok(next) = apply_mount_rollback(node, target, parameters) {
            enumerate_witness_family(&next, depth_remaining - 1, parameters, geometry_view, maximum_weight, overwrites_before_sigma, summary);
        }
    }
}

fn arm_label() -> String {
    format!(
        "placement={:?} order={:?} pbs={}",
        witness::placement(),
        witness::order(),
        physical_block_size_of_this_run()
    )
}

fn run_witness_family(geometry: &Geometry, initial_overwrite_counts: &[u64], sigma_length_limit: u64, maximum_weight: usize) {
    let parameters = parameters_for(geometry);
    let mut summary = FaultSetViolationSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = bootstrap(geometry, overwrites_before_sigma).expect("bootstrap");
        let view = independent_geometry(&root_node.pool).expect("几何");
        enumerate_witness_family(&root_node, sigma_length_limit, &parameters, &view, maximum_weight, overwrites_before_sigma, &mut summary);
    }
    emit_result(&format!(
        "name=witness_family_summary {} geometry={} max_weight={maximum_weight} nodes_visited={} pairs={} pairs_with_any_violation={} cold_recover_with_fault_failed={} mount_writable_with_fault_failed={} skipped_by_the_reduction_rule={}",
        arm_label(), geometry.label, summary.nodes_visited, summary.pairs, summary.pairs_with_any_violation,
        summary.cold_recover_with_fault_failed, summary.mount_writable_with_fault_failed, summary.skipped_by_the_reduction_rule
    ));
    for (weight, pairs) in &summary.pairs_by_weight {
        emit_result(&format!(
            "name=witness_family_by_weight {} weight={weight} pairs={pairs} violations={}",
            arm_label(), summary.violations_by_weight.get(weight).copied().unwrap_or(0)
        ));
    }
    emit_result(&format!(
        "name=witness_family_by_checkpoint {} a_V1={} a_V2={} b_V1={} b_V2={} c_V1={} c_V2={}",
        arm_label(),
        summary.violations_cold_recover_chosen_root_abandoned,
        summary.violations_cold_recover_content_off_timeline,
        summary.violations_mount_then_recover_chosen_root_abandoned,
        summary.violations_mount_then_recover_content_off_timeline,
        summary.violations_recover_after_fault_removed_chosen_root_abandoned,
        summary.violations_recover_after_fault_removed_content_off_timeline
    ));
}

// ---------------------------------------------------------------------------
// w-min：每个带可读被抛弃根的节点，构造让择根落进被抛弃集合所需的故障集合 F* = Hi ∪ W_eff，
// 真跑一次 (a) 坐实它打中，再撤掉见证那一半坐实见证在起作用。
// ---------------------------------------------------------------------------

fn key_of(root: TimelineRoot) -> (u64, u32) {
    (root.1, root.0)
}

/// 只留这一份见证副本、其余副本全读不出时，读得到的见证条目。
fn entries_visible_through_only(pool: &MemoryPool, keep: &FaultItem) -> BTreeSet<witness::WitnessEntry> {
    let ranges: Vec<(DeviceIdentity, u64, u64)> = witness_fault_items().iter().filter(|item| *item != keep).map(FaultItem::range).collect();
    let reader = RangeFaultReader { inner: pool, ranges };
    witness::read_witness(&reader, &FILESYSTEM_IDENTIFIER, 4096, u64::from(physical_block_size_of_this_run()))
}

#[derive(Default)]
struct MinimumFaultSummary {
    nodes_with_live_abandoned_root: u64,
    histogram_of_fault_count: BTreeMap<usize, u64>,
    constructed_set_hits: u64,
    constructed_set_misses: u64,
    without_witness_half_hits: u64,
}

fn walk_minimum_fault_family(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    view: &PoolGeometry,
    summary: &mut MinimumFaultSummary,
) {
    let with_slots = readable_roots_with_slots(&node.pool, view);
    let abandoned_readable: Vec<TimelineRoot> = with_slots.iter().map(|(_, _, root)| *root).filter(|root| is_abandoned(node, *root)).collect();
    if let Some(highest_abandoned) = abandoned_readable.iter().map(|root| key_of(*root)).max() {
        summary.nodes_with_live_abandoned_root += 1;
        let high: Vec<FaultItem> = with_slots
            .iter()
            .filter(|(_, _, root)| !is_abandoned(node, *root) && key_of(*root) > highest_abandoned)
            .map(|(region, slot, _)| FaultItem::RootSlot { region: *region, slot: *slot })
            .collect();
        let effective_witness: Vec<FaultItem> = witness_fault_items().into_iter().filter(|item| !entries_visible_through_only(&node.pool, item).is_empty()).collect();
        let mut constructed = high.clone();
        constructed.extend(effective_witness.iter().copied());
        *summary.histogram_of_fault_count.entry(constructed.len()).or_insert(0) += 1;
        let outcome = recover(&RangeFaultReader { inner: &node.pool, ranges: constructed.iter().map(FaultItem::range).collect() }, JournalPolicy::Consult).outcome;
        let hit = violation_at_checkpoint(node, &outcome).is_some_and(|p| p.chosen_root_abandoned || p.content_off_timeline);
        if hit { summary.constructed_set_hits += 1 } else { summary.constructed_set_misses += 1 }
        if !effective_witness.is_empty() {
            let outcome_without_witness = recover(&RangeFaultReader { inner: &node.pool, ranges: high.iter().map(FaultItem::range).collect() }, JournalPolicy::Consult).outcome;
            if violation_at_checkpoint(node, &outcome_without_witness).is_some_and(|p| p.chosen_root_abandoned || p.content_off_timeline) {
                summary.without_witness_half_hits += 1;
            }
        }
        if high.is_empty() && env::var("WMIN_PRINT_EMPTY_HIGH").is_ok() {
            println!("WMIN_EMPTY_HIGH path={:?} timeline={:?} readable={:?} abandoned_readable={:?}", node.path, node.timeline, with_slots.iter().map(|(_, _, r)| *r).collect::<Vec<_>>(), abandoned_readable);
        }
        if summary.nodes_with_live_abandoned_root <= 3 {
            println!(
                "WMIN_EXAMPLE path={:?} F*=[{}] hit={hit}",
                node.path,
                constructed.iter().map(FaultItem::label).collect::<Vec<_>>().join(",")
            );
        }
    }
    if depth_remaining == 0 {
        return;
    }
    if node.session.is_some() {
        if let Ok(next) = apply_overwrite(node, parameters) {
            walk_minimum_fault_family(&next, depth_remaining - 1, parameters, view, summary);
        }
    }
    if let Ok(next) = apply_mount_writable(node, parameters) {
        walk_minimum_fault_family(&next, depth_remaining - 1, parameters, view, summary);
    }
    for readable_root in &readable_roots_independent(&node.pool, view) {
        let target = RollbackTarget { instance: InstanceGeneration(readable_root.0), checkpoint_txg: CheckpointTxg(readable_root.1) };
        if let Ok(next) = apply_mount_rollback(node, target, parameters) {
            walk_minimum_fault_family(&next, depth_remaining - 1, parameters, view, summary);
        }
    }
}

fn run_minimum_fault_family(geometry: &Geometry, initial_overwrite_counts: &[u64], sigma_length_limit: u64) {
    let parameters = parameters_for(geometry);
    let mut summary = MinimumFaultSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = bootstrap(geometry, overwrites_before_sigma).expect("bootstrap");
        let view = independent_geometry(&root_node.pool).expect("几何");
        walk_minimum_fault_family(&root_node, sigma_length_limit, &parameters, &view, &mut summary);
    }
    emit_result(&format!(
        "name=witness_minimum_fault_summary {} geometry={} sigma_length_limit={sigma_length_limit} nodes_with_live_abandoned_root={} constructed_set_hits={} constructed_set_misses={} without_witness_half_hits={} histogram_of_fault_count={:?}",
        arm_label(), geometry.label, summary.nodes_with_live_abandoned_root, summary.constructed_set_hits,
        summary.constructed_set_misses, summary.without_witness_half_hits, summary.histogram_of_fault_count
    ));
}

// ---------------------------------------------------------------------------
// w-crash：回退那一次挂载的写流录下来，按屏障与 FUA 切段，层 0 同一个枚举域（更早的段整段持久、当前段任意子集、
// 更晚的段一个都没持久），另加撕裂：长于物理块的写，只持久它与底图不同的那几个物理块的非空真子集。
// 每个崩溃状态先不带故障冷启动定「回退算不算成立」（rb0），再对 U = {N 已落盘的根槽} ∪ 见证副本 ∪ {R_old 的槽, 回退前时间线尾的槽}
// 取 |F| ≤ 3 的全部子集跑 P332 三处判定，记最小打中的 |F|。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum RecordedOperation {
    Write { device: DeviceIdentity, offset: u64, bytes: Vec<u8>, force_unit_access: bool },
    Barrier,
}

struct RecordingDevice {
    inner: SparseBlockDevice,
    identity: DeviceIdentity,
    log: Rc<RefCell<Vec<RecordedOperation>>>,
}

impl BlockDevice for RecordingDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.log.borrow_mut().push(RecordedOperation::Write {
            device: self.identity,
            offset: offset.0,
            bytes: bytes.to_vec(),
            force_unit_access: matches!(durability, WriteDurability::ForceUnitAccess),
        });
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.log.borrow_mut().push(RecordedOperation::Write {
            device: self.identity,
            offset: offset.0,
            bytes: vec![0u8; usize::try_from(length).expect("长度")],
            force_unit_access: false,
        });
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut log = self.log.borrow_mut();
        if !matches!(log.last(), Some(RecordedOperation::Barrier)) {
            log.push(RecordedOperation::Barrier);
        }
        Ok(())
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

#[derive(Clone, Debug)]
struct PendingWrite {
    device: DeviceIdentity,
    offset: u64,
    bytes: Vec<u8>,
    force_unit_access: bool,
}

fn write_kind_label(write: &PendingWrite) -> String {
    let offset = write.offset;
    let device = write.device.0;
    if offset < 8192 {
        format!("SC{device}.{}", offset / 4096)
    } else if (512 * 1024..1024 * 1024).contains(&offset) {
        format!("WU{device}.{}", (offset - 512 * 1024) / 4096)
    } else if (1024 * 1024..16 * 1024 * 1024).contains(&offset) {
        format!("ROOT{device}")
    } else if (16 * 1024 * 1024..784 * 1024 * 1024).contains(&offset) {
        format!("REC{device}")
    } else {
        format!("UNIT{device}")
    }
}

fn split_into_segments(operations: &[RecordedOperation]) -> Vec<Vec<PendingWrite>> {
    let mut segments = Vec::new();
    let mut current: Vec<PendingWrite> = Vec::new();
    for operation in operations {
        match operation {
            RecordedOperation::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperation::Write { device, offset, bytes, force_unit_access } => {
                current.push(PendingWrite { device: *device, offset: *offset, bytes: bytes.clone(), force_unit_access: *force_unit_access });
                if *force_unit_access {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn apply_write(pool: &mut MemoryPool, write: &PendingWrite) {
    pool.devices.get_mut(&write.device).expect("盘").write(DeviceOffsetInBytes(write.offset), &write.bytes);
}

/// 撕裂：这次写与底图不同的物理块里只持久 `kept` 那几块。
fn apply_torn_write(pool: &mut MemoryPool, write: &PendingWrite, block_bytes: usize, kept: &[usize]) {
    for &block in kept {
        let start = block * block_bytes;
        apply_write(pool, &PendingWrite {
            device: write.device,
            offset: write.offset + start as u64,
            bytes: write.bytes[start..start + block_bytes].to_vec(),
            force_unit_access: false,
        });
    }
}

fn differing_blocks(pool: &MemoryPool, write: &PendingWrite, block_bytes: usize) -> Vec<usize> {
    let old = singlefs_core::recovery::PoolReader::read(pool, write.device, DeviceOffsetInBytes(write.offset), write.bytes.len()).expect("读底图");
    (0..write.bytes.len() / block_bytes)
        .filter(|block| old[block * block_bytes..(block + 1) * block_bytes] != write.bytes[block * block_bytes..(block + 1) * block_bytes])
        .collect()
}

struct CrashState {
    image: MemoryPool,
    description: String,
}

fn crash_states_of(pre_image: &MemoryPool, segments: &[Vec<PendingWrite>], block_bytes: usize) -> Vec<CrashState> {
    let mut states = Vec::new();
    let mut base = pre_image.clone();
    for (segment_index, segment) in segments.iter().enumerate() {
        let labels: Vec<String> = segment.iter().map(write_kind_label).collect();
        let subsets: Vec<Vec<usize>> = if segment.len() <= 6 {
            (0u32..(1u32 << segment.len()) - 1)
                .map(|mask| (0..segment.len()).filter(|bit| mask & (1 << bit) != 0).collect())
                .collect()
        } else {
            // 段里带单元（每盘几个单元）：单元一个都不持久，其余（系统配置、见证、记录、根）取任意子集。
            // 单元是写时复制的新槽，只有这一段之后那条根落盘才用得到它们。见报告「限度」。
            let others: Vec<usize> = (0..segment.len()).filter(|index| !write_kind_label(&segment[*index]).starts_with("UNIT")).collect();
            assert!(others.len() <= 6, "非单元写太多");
            (0u32..(1u32 << others.len()))
                .map(|mask| (0..others.len()).filter(|bit| mask & (1 << bit) != 0).map(|bit| others[bit]).collect())
                .collect()
        };
        for subset in &subsets {
            let mut image = base.clone();
            for &index in subset {
                apply_write(&mut image, &segment[index]);
            }
            states.push(CrashState {
                image,
                description: format!("seg={segment_index}/{} [{}] persisted={:?}", segments.len(), labels.join(","), subset),
            });
            // 撕裂：子集之外的一次写（它是这个状态里「在飞」的那一次）只持久它不同物理块的非空真子集。
            for (index, write) in segment.iter().enumerate() {
                if subset.contains(&index) || write.bytes.len() <= block_bytes {
                    continue;
                }
                let mut probe = base.clone();
                for &kept_index in subset {
                    apply_write(&mut probe, &segment[kept_index]);
                }
                let differing = differing_blocks(&probe, write, block_bytes);
                if differing.len() < 2 || differing.len() > 4 {
                    continue;
                }
                for mask in 1u32..(1u32 << differing.len()) - 1 {
                    let kept: Vec<usize> = (0..differing.len()).filter(|bit| mask & (1 << bit) != 0).map(|bit| differing[bit]).collect();
                    let mut image = probe.clone();
                    apply_torn_write(&mut image, write, block_bytes, &kept);
                    states.push(CrashState {
                        image,
                        description: format!(
                            "seg={segment_index}/{} [{}] persisted={:?} torn={}:blocks{:?}of{:?}",
                            segments.len(), labels.join(","), subset, labels[index], kept, differing
                        ),
                    });
                }
            }
        }
        for write in segment {
            apply_write(&mut base, write);
        }
    }
    states.push(CrashState { image: base, description: format!("seg=done/{}", segments.len()) });
    states
}

struct RollbackEdgeBookkeeping {
    abandoned: BTreeSet<TimelineRoot>,
    new_timeline_contents: Vec<Option<Vec<u8>>>,
    new_instance_roots: Vec<TimelineRoot>,
    target: TimelineRoot,
    pre_tip: TimelineRoot,
}

fn chosen_and_content(outcome: &RecoveryOutcome) -> Option<(TimelineRoot, Option<Vec<u8>>)> {
    match outcome {
        RecoveryOutcome::Failed { .. } => None,
        RecoveryOutcome::NoFile { root } => Some(((root.0 .0, root.1 .0), None)),
        RecoveryOutcome::FileRead { root, content } => Some(((root.0 .0, root.1 .0), Some(content.clone()))),
    }
}

/// 回退成立之后的判定：所选根落进被抛弃集合（V1），或读回的内容不在新时间线上（V2）。
fn revoked(book: &RollbackEdgeBookkeeping, outcome: &RecoveryOutcome) -> Option<bool> {
    let (chosen, content) = chosen_and_content(outcome)?;
    let v1 = book.abandoned.contains(&chosen);
    let v2 = content.is_some() && !book.new_timeline_contents.contains(&content);
    Some(v1 || v2)
}

fn root_slot_item_of(image: &MemoryPool, view: &PoolGeometry, root: TimelineRoot) -> Option<FaultItem> {
    readable_roots_with_slots(image, view)
        .into_iter()
        .find(|(_, _, candidate)| *candidate == root)
        .map(|(region, slot, _)| FaultItem::RootSlot { region, slot })
}

#[derive(Default)]
struct CrashSummary {
    edges: u64,
    states: u64,
    states_rolled_back: u64,
    states_rolled_back_without_new_root: u64,
    states_not_rolled_back_but_flip_to_rolled_back_under_fault: u64,
    minimum_fault_histogram: BTreeMap<String, u64>,
}

#[allow(clippy::too_many_arguments)]
fn evaluate_crash_state(
    state: &CrashState,
    book: &RollbackEdgeBookkeeping,
    parameters: &MakeFilesystemParameters,
    view: &PoolGeometry,
    edge_label: &str,
    summary: &mut CrashSummary,
) {
    summary.states += 1;
    let baseline = recover(&state.image, JournalPolicy::Consult).outcome;
    let Some((chosen_zero, _)) = chosen_and_content(&baseline) else {
        println!("CRASH {edge_label} {} baseline=failed", state.description);
        return;
    };
    let rolled_back = !book.abandoned.contains(&chosen_zero);
    let readable_now: BTreeSet<TimelineRoot> = readable_roots_independent(&state.image, view);
    let new_root_durable = book.new_instance_roots.iter().any(|root| readable_now.contains(root));
    let entries_now = witness::read_witness(&state.image, &FILESYSTEM_IDENTIFIER, 4096, u64::from(physical_block_size_of_this_run()));
    let copies_with_new_entry = witness_fault_items()
        .iter()
        .filter(|item| {
            entries_visible_through_only(&state.image, item).iter().any(|entry| entry.target_instance == book.target.0 && entry.target_txg == book.target.1 && book.new_instance_roots.first().is_none_or(|first| entry.new_instance == first.0))
        })
        .count();
    let mut universe: Vec<FaultItem> = book.new_instance_roots.iter().filter_map(|root| root_slot_item_of(&state.image, view, *root)).collect();
    universe.extend(witness_fault_items());
    for extra in [book.target, book.pre_tip] {
        if let Some(item) = root_slot_item_of(&state.image, view, extra) {
            if !universe.contains(&item) {
                universe.push(item);
            }
        }
    }
    let mut minimum: Option<(usize, String, String)> = None;
    let mut flip = false;
    'weights: for weight in 1..=3usize {
        for combo in combinations_of_size(&universe, weight) {
            let ranges: Vec<(DeviceIdentity, u64, u64)> = combo.iter().map(FaultItem::range).collect();
            let cold = recover(&RangeFaultReader { inner: &state.image, ranges: ranges.clone() }, JournalPolicy::Consult).outcome;
            if !rolled_back {
                if let Some((chosen, _)) = chosen_and_content(&cold) {
                    if book.new_instance_roots.contains(&chosen) || (!book.abandoned.contains(&chosen) && readable_now.iter().any(|root| book.abandoned.contains(root) && !combo.iter().any(|item| Some(*item) == root_slot_item_of(&state.image, view, *root)))) {
                        flip = true;
                    }
                }
                continue;
            }
            let mut hit_at = String::new();
            if revoked(book, &cold) == Some(true) {
                hit_at.push('a');
            }
            let cold_chosen = chosen_and_content(&cold).map(|(root, _)| root);
            if hit_at.is_empty() && cold_chosen != Some(chosen_zero) {
                let shared = Rc::new(RefCell::new(ranges));
                let mut devices: Vec<(DeviceIdentity, RangeFailingDevice)> = devices_from_pool(&state.image, IMAGE_BYTES)
                    .into_iter()
                    .map(|(identity, device)| (identity, RangeFailingDevice { inner: device, identity, ranges: shared.clone() }))
                    .collect();
                if mount_writable(parameters, &mut devices).is_ok() {
                    if revoked(book, &recover(&devices, JournalPolicy::Consult).outcome) == Some(true) {
                        hit_at.push('b');
                    }
                }
                shared.borrow_mut().clear();
                if revoked(book, &recover(&devices, JournalPolicy::Consult).outcome) == Some(true) {
                    hit_at.push('c');
                }
            }
            if !hit_at.is_empty() {
                minimum = Some((weight, combo.iter().map(FaultItem::label).collect::<Vec<_>>().join(","), hit_at));
                break 'weights;
            }
        }
    }
    if rolled_back {
        summary.states_rolled_back += 1;
        if !new_root_durable {
            summary.states_rolled_back_without_new_root += 1;
        }
    } else if flip {
        summary.states_not_rolled_back_but_flip_to_rolled_back_under_fault += 1;
    }
    let minimum_label = match &minimum {
        Some((weight, _, _)) => format!("{weight}"),
        None if rolled_back => ">3".to_string(),
        None => "n/a(not_rolled_back)".to_string(),
    };
    *summary.minimum_fault_histogram.entry(minimum_label.clone()).or_insert(0) += 1;
    println!(
        "CRASH {edge_label} {} rb0={rolled_back} chosen0={chosen_zero:?} new_root_durable={new_root_durable} witness_entries={} copies_with_new_entry={copies_with_new_entry} flip_to_rb_under_fault={flip} minF={minimum_label} example={}",
        state.description,
        entries_now.len(),
        minimum.as_ref().map(|(_, set, at)| format!("[{set}]@{at}")).unwrap_or_default()
    );
}

fn collect_nodes(node: &SimNode, depth_remaining: u64, parameters: &MakeFilesystemParameters, view: &PoolGeometry, out: &mut Vec<SimNode>) {
    out.push(node.clone());
    if depth_remaining == 0 {
        return;
    }
    if node.session.is_some() {
        if let Ok(next) = apply_overwrite(node, parameters) {
            collect_nodes(&next, depth_remaining - 1, parameters, view, out);
        }
    }
    if let Ok(next) = apply_mount_writable(node, parameters) {
        collect_nodes(&next, depth_remaining - 1, parameters, view, out);
    }
    for readable_root in &readable_roots_independent(&node.pool, view) {
        let target = RollbackTarget { instance: InstanceGeneration(readable_root.0), checkpoint_txg: CheckpointTxg(readable_root.1) };
        if let Ok(next) = apply_mount_rollback(node, target, parameters) {
            collect_nodes(&next, depth_remaining - 1, parameters, view, out);
        }
    }
}

fn run_crash_family(geometry: &Geometry, initial_overwrite_counts: &[u64], prefix_depth: u64, edge_stride: usize) {
    let parameters = parameters_for(geometry);
    let block_bytes = usize::try_from(physical_block_size_of_this_run()).expect("pbs");
    let mut summary = CrashSummary::default();
    let mut edge_index = 0usize;
    let mut stream_printed = false;
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = bootstrap(geometry, overwrites_before_sigma).expect("bootstrap");
        let view = independent_geometry(&root_node.pool).expect("几何");
        let mut nodes = Vec::new();
        collect_nodes(&root_node, prefix_depth, &parameters, &view, &mut nodes);
        for node in &nodes {
            for readable_root in &readable_roots_independent(&node.pool, &view) {
                let target = RollbackTarget { instance: InstanceGeneration(readable_root.0), checkpoint_txg: CheckpointTxg(readable_root.1) };
                let log = Rc::new(RefCell::new(Vec::new()));
                let mut devices: Vec<(DeviceIdentity, RecordingDevice)> = devices_from_pool(&node.pool, IMAGE_BYTES)
                    .into_iter()
                    .map(|(identity, device)| (identity, RecordingDevice { inner: device, identity, log: log.clone() }))
                    .collect();
                let Some(position_in_timeline) = node.timeline.iter().position(|root| *root == *readable_root) else {
                    // 读得出、但在被抛弃的时间线上：回退会被候选集拒绝，不算一条边。
                    continue;
                };
                let readable_before = readable_roots_independent(&node.pool, &view);
                if !node.timeline[position_in_timeline + 1..].iter().any(|root| readable_before.contains(root)) {
                    // 回退到时间线尾（或被抛弃的根一条都读不出）：没有可被撤销的东西，不算一条边。
                    continue;
                }
                let Ok(mounted) = mount_rollback(&parameters, &mut devices, target, ShadowLedger::On) else { continue };
                edge_index += 1;
                if edge_index % edge_stride != 0 {
                    continue;
                }
                summary.edges += 1;
                let position = position_in_timeline;
                let target_content = node.content_by_root.get(readable_root).cloned().unwrap_or(None);
                let mut new_instance_roots = vec![root_pair(mounted.output.row_publish.root())];
                new_instance_roots.extend(mounted.output.warm_up_publishes.iter().map(|publish| root_pair(publish.root())));
                let mut new_timeline_contents: Vec<Option<Vec<u8>>> = node.timeline[..=position].iter().map(|root| node.content_by_root.get(root).cloned().unwrap_or(None)).collect();
                new_timeline_contents.push(target_content);
                let book = RollbackEdgeBookkeeping {
                    abandoned: node.timeline[position + 1..].iter().copied().collect(),
                    new_timeline_contents,
                    new_instance_roots,
                    target: *readable_root,
                    pre_tip: *node.timeline.last().expect("非空"),
                };
                let operations = log.borrow().clone();
                let segments = split_into_segments(&operations);
                if !stream_printed {
                    stream_printed = true;
                    for (index, segment) in segments.iter().enumerate() {
                        println!("STREAM {} seg={index} [{}]", arm_label(), segment.iter().map(write_kind_label).collect::<Vec<_>>().join(","));
                    }
                }
                let edge_label = format!("edge={edge_index} ob={overwrites_before_sigma} path={:?} R({},{})", node.path, readable_root.0, readable_root.1);
                for state in crash_states_of(&node.pool, &segments, block_bytes) {
                    evaluate_crash_state(&state, &book, &parameters, &view, &edge_label, &mut summary);
                }
            }
        }
    }
    emit_result(&format!(
        "name=witness_crash_summary {} geometry={} prefix_depth={prefix_depth} edge_stride={edge_stride} edges={} states={} states_rolled_back={} states_rolled_back_without_new_root={} states_not_rolled_back_but_flip_to_rolled_back_under_fault={} minimum_fault_histogram={:?}",
        arm_label(), geometry.label, summary.edges, summary.states, summary.states_rolled_back,
        summary.states_rolled_back_without_new_root, summary.states_not_rolled_back_but_flip_to_rolled_back_under_fault,
        summary.minimum_fault_histogram
    ));
}

fn run_witness_modes(mode: &str, command_line_arguments: &[String]) {
    let counts = [1u64, 2, 3];
    match mode {
        "w-family" => {
            let maximum_weight = command_line_arguments.get(2).and_then(|v| v.parse().ok()).unwrap_or(2usize);
            run_witness_family(&GEOMETRY_PRIMARY, &counts, 3, maximum_weight);
        }
        "w-family-s4" => run_witness_family(&GEOMETRY_SMALLER_ROOT_RING, &counts, 4, 2),
        "w-min" => run_minimum_fault_family(&GEOMETRY_PRIMARY, &counts, 3),
        "w-min-s4" => run_minimum_fault_family(&GEOMETRY_SMALLER_ROOT_RING, &counts, 4),
        "w-crash" => {
            let prefix_depth = command_line_arguments.get(2).and_then(|v| v.parse().ok()).unwrap_or(1u64);
            let edge_stride = command_line_arguments.get(3).and_then(|v| v.parse().ok()).unwrap_or(1usize);
            run_crash_family(&GEOMETRY_PRIMARY, &counts, prefix_depth, edge_stride);
        }
        "w-targeted" => run_targeted(&GEOMETRY_PRIMARY),
        "w-stale" => run_stale_witness_probe(&GEOMETRY_PRIMARY),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// w-targeted（2026-09-24 主 agent 转用户令之后）：不做全量枚举，只跑挑的历史。
//   E1 = ob=1、path=[Overwrite] 之后第一条有可被撤销之物的回退（表 1 条）；
//   E2 = 第一个「已有一次回退」的节点上第一条有可被撤销之物的回退（表 2 条，(a) 的表越过 512）。
//   每条边：回退那次挂载的写流上每个崩溃点（含撕裂），|F| ≤ 3、U 同 w-crash。
//   T6：崩溃后见证只剩 1 份 + 那一份读不出时可写挂载，之后撤故障，再让新实例的根全读不出。
//   T5：盘上留一份 fsid 不同的旧见证（(b)(c)），核 fsid 与不核各跑一次。
// ---------------------------------------------------------------------------

struct ChosenEdge {
    label: String,
    node: SimNode,
    target: TimelineRoot,
}

fn choose_targeted_edges(geometry: &Geometry) -> Vec<ChosenEdge> {
    let parameters = parameters_for(geometry);
    let root_node = bootstrap(geometry, 1).expect("bootstrap");
    let view = independent_geometry(&root_node.pool).expect("几何");
    let mut nodes = Vec::new();
    collect_nodes(&root_node, 2, &parameters, &view, &mut nodes);
    let mut edges = Vec::new();
    let has_undoable_target = |node: &SimNode, target: &TimelineRoot| {
        let readable = readable_roots_independent(&node.pool, &view);
        node.timeline.iter().position(|root| root == target).is_some_and(|position| node.timeline[position + 1..].iter().any(|root| readable.contains(root)))
    };
    for (label, wanted_events) in [("E1", 0usize), ("E2", 1usize)] {
        'nodes: for node in &nodes {
            if node.rollback_events.len() != wanted_events || (wanted_events == 0 && node.path.len() != 1) {
                continue;
            }
            // 取 (实例, txg) 最大的那个可回退目标（「退一步」那种回退；目标 txg 非 0，表里那几个字节才不全是 0）。
            for target in readable_roots_independent(&node.pool, &view).into_iter().rev() {
                let has_file = |root: &TimelineRoot| matches!(node.content_by_root.get(root), Some(Some(_)));
                let tip = *node.timeline.last().expect("非空");
                if has_undoable_target(node, &target) && target.1 > 0 && has_file(&target) && has_file(&tip)
                    && node.content_by_root.get(&target) != node.content_by_root.get(&tip)
                {
                    edges.push(ChosenEdge { label: label.to_string(), node: node.clone(), target });
                    break 'nodes;
                }
            }
        }
    }
    edges
}

fn run_targeted(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let block_bytes = usize::try_from(physical_block_size_of_this_run()).expect("pbs");
    let root_node = bootstrap(geometry, 1).expect("bootstrap");
    let view = independent_geometry(&root_node.pool).expect("几何");
    for edge in choose_targeted_edges(geometry) {
        let node = &edge.node;
        let target = RollbackTarget { instance: InstanceGeneration(edge.target.0), checkpoint_txg: CheckpointTxg(edge.target.1) };
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut devices: Vec<(DeviceIdentity, RecordingDevice)> = devices_from_pool(&node.pool, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingDevice { inner: device, identity, log: log.clone() }))
            .collect();
        let mounted = mount_rollback(&parameters, &mut devices, target, ShadowLedger::On).expect("挑的回退边可做");
        let position = node.timeline.iter().position(|root| *root == edge.target).expect("在时间线上");
        let mut new_instance_roots = vec![root_pair(mounted.output.row_publish.root())];
        new_instance_roots.extend(mounted.output.warm_up_publishes.iter().map(|publish| root_pair(publish.root())));
        let mut new_timeline_contents: Vec<Option<Vec<u8>>> = node.timeline[..=position].iter().map(|root| node.content_by_root.get(root).cloned().unwrap_or(None)).collect();
        new_timeline_contents.push(node.content_by_root.get(&edge.target).cloned().unwrap_or(None));
        let book = RollbackEdgeBookkeeping {
            abandoned: node.timeline[position + 1..].iter().copied().collect(),
            new_timeline_contents,
            new_instance_roots: new_instance_roots.clone(),
            target: edge.target,
            pre_tip: *node.timeline.last().expect("非空"),
        };
        let segments = split_into_segments(&log.borrow().clone());
        println!("EDGE {} {} path={:?} target={:?} pre_tip={:?} abandoned={:?} new_instance_roots={:?}", edge.label, arm_label(), node.path, edge.target, book.pre_tip, book.abandoned, new_instance_roots);
        for (index, segment) in segments.iter().enumerate() {
            println!("STREAM {} {} seg={index} [{}]", edge.label, arm_label(), segment.iter().map(write_kind_label).collect::<Vec<_>>().join(","));
        }
        let mut summary = CrashSummary::default();
        let states = crash_states_of(&node.pool, &segments, block_bytes);
        for state in &states {
            evaluate_crash_state(state, &book, &parameters, &view, &edge.label, &mut summary);
        }
        emit_result(&format!(
            "name=witness_targeted_summary {} edge={} states={} states_rolled_back={} states_rolled_back_without_new_root={} flip_to_rolled_back_under_fault={} minimum_fault_histogram={:?}",
            arm_label(), edge.label, summary.states, summary.states_rolled_back, summary.states_rolled_back_without_new_root,
            summary.states_not_rolled_back_but_flip_to_rolled_back_under_fault, summary.minimum_fault_histogram
        ));
        // T6：挑「回退已成立、见证恰 1 份、新实例一条根都没落盘」的第一个崩溃状态。
        for state in &states {
            let chosen = chosen_and_content(&recover(&state.image, JournalPolicy::Consult).outcome).map(|(root, _)| root);
            let Some(chosen) = chosen else { continue };
            let readable = readable_roots_independent(&state.image, &view);
            if book.abandoned.contains(&chosen) || new_instance_roots.iter().any(|root| readable.contains(root)) {
                continue;
            }
            let copies: Vec<FaultItem> = witness_fault_items().into_iter().filter(|item| !entries_visible_through_only(&state.image, item).is_empty() && entries_visible_through_only(&state.image, item).iter().any(|entry| entry.target_instance == edge.target.0 && entry.target_txg == edge.target.1)).collect();
            if copies.len() != 1 {
                continue;
            }
            run_follow_on_history(edge.label.as_str(), state, &book, &parameters, &view, &copies);
            break;
        }
    }
    let _ = root_node;
}

/// T6：见证那一份读不出时可写挂载（新实例 M 建在哪），撤故障冷启动，再让 M 的根全读不出冷启动。
fn run_follow_on_history(edge_label: &str, state: &CrashState, book: &RollbackEdgeBookkeeping, parameters: &MakeFilesystemParameters, view: &PoolGeometry, copies: &[FaultItem]) {
    let before = recover(&state.image, JournalPolicy::Consult).outcome;
    let shared = Rc::new(RefCell::new(copies.iter().map(FaultItem::range).collect::<Vec<_>>()));
    let mut devices: Vec<(DeviceIdentity, RangeFailingDevice)> = devices_from_pool(&state.image, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, device)| (identity, RangeFailingDevice { inner: device, identity, ranges: shared.clone() }))
        .collect();
    let mounted = match mount_writable(parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            println!("T6 {edge_label} {} {} mount_writable_under_fault_failed={error:?}", arm_label(), state.description);
            return;
        }
    };
    let built_on = root_pair(&mounted.output.chosen_root);
    let mut m_roots = vec![root_pair(mounted.output.row_publish.root())];
    m_roots.extend(mounted.output.warm_up_publishes.iter().map(|publish| root_pair(publish.root())));
    shared.borrow_mut().clear();
    let after_fault_removed = recover(&devices, JournalPolicy::Consult).outcome;
    let image_after: MemoryPool = memory_pool_of(&devices.iter().map(|(identity, device)| (*identity, clone_sparse(&device.inner))).collect::<Vec<_>>(), IMAGE_BYTES);
    let m_items: Vec<FaultItem> = m_roots.iter().filter_map(|root| root_slot_item_of(&image_after, view, *root)).collect();
    let m_faulted = recover(&RangeFaultReader { inner: &image_after, ranges: m_items.iter().map(FaultItem::range).collect() }, JournalPolicy::Consult).outcome;
    let witness_after = witness::read_witness(&image_after, &FILESYSTEM_IDENTIFIER, 4096, u64::from(physical_block_size_of_this_run()));
    let show = |outcome: &RecoveryOutcome| match chosen_and_content(outcome) {
        None => "failed".to_string(),
        Some((root, content)) => format!(
            "{root:?} content={}",
            if content.is_none() { "none".to_string() } else if book.new_timeline_contents[book.new_timeline_contents.len() - 1] == content { "R_old".to_string() } else if book.abandoned.contains(&root) || !book.new_timeline_contents.contains(&content) { "abandoned-timeline".to_string() } else { "other-on-new-timeline".to_string() }
        ),
    };
    println!(
        "T6 {edge_label} {} {} faulted_witness=[{}] before={} m_built_on={built_on:?} m_roots={m_roots:?} after_fault_removed={} witness_after={:?} m_roots_faulted=[{}] recover_with_m_roots_faulted={}",
        arm_label(), state.description,
        copies.iter().map(FaultItem::label).collect::<Vec<_>>().join(","),
        show(&before), show(&after_fault_removed), witness_after,
        m_items.iter().map(FaultItem::label).collect::<Vec<_>>().join(","), show(&m_faulted)
    );
}

fn clone_sparse(device: &SparseBlockDevice) -> SparseBlockDevice {
    let mut out = SparseBlockDevice::new(IMAGE_BYTES, device.probe_physical_block_size());
    out.image = device.image.clone();
    out
}

/// T5：盘上留一份 fsid 不同、表里一条 (u32::MAX, 0, 0) 的旧见证（抛弃 (0,0) 之上的一切），看择根。
fn run_stale_witness_probe(geometry: &Geometry) {
    let node = bootstrap(geometry, 1).expect("bootstrap");
    let mut pool = node.pool.clone();
    let foreign_identifier = [0xEEu8; 16];
    let mut entries = BTreeSet::new();
    entries.insert(witness::WitnessEntry { new_instance: u32::MAX, target_instance: 0, target_txg: 0 });
    let pbs = usize::try_from(physical_block_size_of_this_run()).expect("pbs");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        match witness::placement() {
            witness::WitnessPlacement::IndependentUnit => {
                let unit = witness::build_independent_unit(&foreign_identifier, device, 7, &entries, pbs);
                pool.devices.get_mut(&device).expect("盘").write(DeviceOffsetInBytes(witness::INDEPENDENT_UNIT_OFFSETS[1]), &unit);
            }
            witness::WitnessPlacement::SectorPiece => {
                // 旧片放在系统配置槽 1 的第 2 扇区（这一池今天还没写过片）。
                let mut slot = singlefs_core::recovery::PoolReader::read(&pool, device, DeviceOffsetInBytes(4096), 4096).expect("读");
                witness::place_sector_piece(&mut slot, &foreign_identifier, device, 7, &entries);
                pool.devices.get_mut(&device).expect("盘").write(DeviceOffsetInBytes(4096), &slot);
            }
            _ => return,
        }
    }
    let before = chosen_root_of(&recover(&node.pool, JournalPolicy::Consult).outcome);
    let after = chosen_root_of(&recover(&pool, JournalPolicy::Consult).outcome);
    emit_result(&format!(
        "name=witness_stale_foreign_piece {} checks_fsid={} chosen_without_stale={before:?} chosen_with_stale={after:?}",
        arm_label(), witness::checks_filesystem_identifier()
    ));
}
