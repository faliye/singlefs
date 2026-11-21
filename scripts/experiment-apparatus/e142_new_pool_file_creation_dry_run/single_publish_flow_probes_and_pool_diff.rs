//! 单次发布的流、预期违例的式子、阳性对照、探针、池之间的差异段与十六进制。

use e7_index_bench::{usize_from_u64, usize_from_u64_in_const};
use super::*;

/// 一次「单次发布」流：mkfs → 直接发布（阳性对照、G6 都是这个形状：没有取号、没有暖机）。
pub(super) struct SinglePublishFlow {
    pub(super) base: Pool,
    pub(super) writes: Vec<WriteRequest>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) closed_form: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) root_index: usize,
    pub(super) journal_index: usize,
    /// 依赖集甲：这一窗口里全部单元写，每个单元一组（一盘时组内恰一个下标）。
    pub(super) dependency_groups: Vec<Vec<usize>>,
    /// 依赖集乙要减掉的那一部分：分配记录树根以下的节点数（第一节 R10 乙档）。
    below_root_count: usize,
    pub(super) role_names: Vec<String>,
}

pub(super) fn build_single_publish_flow(parameters: &PoolParameters, file_bytes: &[u8], fua_is_boundary: bool) -> SinglePublishFlow {
    let (mut recording, genesis) = mkfs(parameters);
    let mkfs_operation_count = recording.operations.len();
    let base = recording.pool.clone();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], fua_is_boundary);
    let closed_form = closed_form_state_count(&segments);
    let root_index = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条");
    let journal_index = writes.iter().position(|write| write.kind == StepKind::JournalRecord).expect("写流里有 journal 记录那一条");
    let dependency_groups: Vec<Vec<usize>> = (0..output.units_by_slot.len()).map(|index| vec![index]).collect();
    let below_root_count = output
        .units_by_slot
        .iter()
        .filter(|(_, unit, _)| matches!(unit, TransactionUnit::Allocation(AllocationNodeRole::Leaf { .. }) | TransactionUnit::Allocation(AllocationNodeRole::Internal { .. })))
        .count();
    let role_names = single_publish_role_names(writes.len(), &output.units_by_slot);
    SinglePublishFlow { base, writes, segments, closed_form, root_index, journal_index, dependency_groups, below_root_count, role_names }
}

/// R10：`(1 << (N-1)) - (1 << (N-1-|依赖集|))`。
fn expected_violations_formula(write_count: usize, dependency_count: usize) -> u64 {
    (1u64 << (write_count - 1)) - (1u64 << (write_count - 1 - dependency_count))
}

/// 阳性对照期望违例数的三档读法（R10 甲/乙/丙），从这一窗口的写清单现算，不写字面
/// （M149 的会红检查：把甲档的依赖计数换成乙档会把 4092 改成 4080）。
pub(super) fn control_expected_violations_three_readings(flow: &SinglePublishFlow) -> (u64, u64, u64) {
    let write_count = flow.writes.len();
    let unit_writes = flow.dependency_groups.len();
    (
        expected_violations_formula(write_count, unit_writes),
        expected_violations_formula(write_count, unit_writes - flow.below_root_count),
        expected_violations_formula(write_count, unit_writes - 1),
    )
}

/// PC2（第五节 5.2）：主臂几何上，全部持久（1 个状态）与逐个单元的两份都不持久（每个单元 1 个状态）。
/// 不走枚举——只构造这 `units_by_slot.len() + 1` 个状态，直接调 `recover` + `oracle_violation`。
pub(super) struct Pc2Report {
    pub(super) all_persisted_legal: bool,
    pub(super) all_persisted_content_ok: bool,
    pub(super) caught: u64,
    pub(super) per_unit: Vec<(String, String, bool, String)>, // (label=descriptive_tag, tag=tN, caught, outcome_kind)
}

pub(super) fn run_pc2(base: &Pool, writes: &[WriteRequest], units_by_slot: &[(SlotNumber, TransactionUnit, Vec<u8>)], expected_content: &[u8]) -> Pc2Report {
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条（PC2）");
    let evaluate = |persisted: Vec<bool>| -> (RecoveryOutcome, bool) {
        let root_persisted = persisted[root_index];
        let image = CrashImage { base, writes, persisted };
        let report = recover(&image, JournalPolicy::Consult, ReaderMode::Clause);
        let is_violation = oracle_violation(&report.outcome, root_persisted, expected_content).is_some();
        (report.outcome, is_violation)
    };
    let all_true = vec![true; writes.len()];
    let (all_outcome, all_violation) = evaluate(all_true.clone());
    let all_persisted_content_ok = matches!(&all_outcome, RecoveryOutcome::FileRead { content, .. } if content == expected_content);
    let mut caught = 0u64;
    let mut per_unit = Vec::new();
    for (slot, unit, _) in units_by_slot {
        let offset = slot.device_offset();
        let indices: Vec<usize> = writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::UnitWrite && write.offset == offset).map(|(index, _)| index).collect();
        assert!(!indices.is_empty(), "PC2：单元 {} 在写清单里一份镜像都找不到", unit.descriptive_tag());
        let mut persisted = all_true.clone();
        for index in &indices {
            persisted[*index] = false;
        }
        let (outcome, is_violation) = evaluate(persisted);
        if is_violation {
            caught += 1;
        }
        per_unit.push((unit.descriptive_tag().to_string(), unit.tag().to_string(), is_violation, outcome_kind(&outcome).to_string()));
    }
    Pc2Report { all_persisted_legal: !all_violation, all_persisted_content_ok, caught, per_unit }
}

// ───────────────────────── 坏字节探针（里程碑步 6 验收） ─────────────────────────

pub(super) fn flip_byte(pool: &mut Pool, device: DeviceIdentity, offset: DeviceOffset, byte_index: u64) {
    let sector_offset = DeviceOffset(offset.0 + byte_index - byte_index % PHYSICAL_BLOCK_BYTES);
    let mut sector = pool.devices[device.0 as usize].read(sector_offset, usize_from_u64(PHYSICAL_BLOCK_BYTES));
    sector[(byte_index % PHYSICAL_BLOCK_BYTES) as usize] ^= 0xff;
    pool.devices[device.0 as usize].write(sector_offset, &sector);
}

pub(super) fn outcome_kind(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::Failed { .. } => "failed",
    }
}

pub(super) fn outcome_root(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: Some(root), .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: None, .. } => "none".to_string(),
    }
}

pub(super) struct Probe {
    pub(super) name: &'static str,
    pub(super) flips: Vec<(DeviceIdentity, DeviceOffset, u64)>,
}

pub(super) fn probes(parameters: &PoolParameters) -> Vec<Probe> {
    let (newest_root_region, newest_root_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let newest_root_device = parameters.region_devices[usize_from_u64(newest_root_region)];
    let journal_start = DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES + (NEW_POOL_FILE_CREATION_TXG - 1) * JOURNAL_RECORD_BYTES);
    let data_offset = SlotNumber(SLOT_DATA_UNIT).device_offset();
    let tree_table_offset = SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH).device_offset();
    let both = |offset: DeviceOffset, byte: u64| vec![(DeviceIdentity(0), offset, byte), (DeviceIdentity(1), offset, byte)];
    vec![
        Probe { name: "newest_root_slot_one_byte", flips: vec![(newest_root_device, ring_slot_offset(newest_root_region, newest_root_slot), 100)] },
        Probe { name: "journal_record_both_copies", flips: both(journal_start, 200) },
        Probe { name: "journal_record_one_copy", flips: vec![(DeviceIdentity(0), journal_start, 200)] },
        Probe { name: "data_payload_one_copy", flips: vec![(DeviceIdentity(0), data_offset, 200)] },
        Probe { name: "data_payload_both_copies", flips: both(data_offset, 200) },
        Probe { name: "data_header_last_byte_both_copies", flips: both(data_offset, DATA_UNIT_HEADER_BYTES - 1) },
        Probe { name: "tree_table_both_copies", flips: both(tree_table_offset, 300) },
        Probe { name: "system_configuration_slot_one_both_devices", flips: both(DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), 50) },
    ]
}

pub(super) fn run_probe(full: &Pool, probe: &Probe) -> RecoveryReport {
    let mut damaged = full.clone();
    for (device, offset, byte_index) in &probe.flips {
        flip_byte(&mut damaged, *device, *offset, *byte_index);
    }
    recover(&damaged, JournalPolicy::Consult, ReaderMode::Clause)
}

// ───────────────────────── E142 第十一次跑：改动计数从 1 改成 3，逐字节相减 ─────────────────────────
// 重跑登记 research/prompts/e142-r11-prereg.md 第一节第 3 条：分母是两块盘的全部字节，不是只看那 8 个单元。
// 没写过的扇区两边都读 0，差不出字节，所以「两份镜像里写过的扇区并集」与「全部字节逐字节比对」等价（第 3 步跑之前已在这里写死这条推理）。

/// 量 1 的六元组，去掉「所属结构 + 结构内偏移」——那两列由 `classify_offset` 事后算，不是逐字节比对本身要用的。
#[derive(Clone, Debug)]
pub(super) struct DiffSegment {
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) old: Vec<u8>,
    pub(super) new: Vec<u8>,
}

/// 两份镜像（同设备数）逐字节相减，合并相邻差异字节成段。
pub(super) fn diff_pools(old: &Pool, new: &Pool) -> Vec<DiffSegment> {
    assert_eq!(old.devices.len(), new.devices.len(), "两份镜像设备数必须一致才能相减");
    let mut segments = Vec::new();
    let zero_sector = [0u8; usize_from_u64_in_const(PHYSICAL_BLOCK_BYTES)];
    for device_index in 0..old.devices.len() {
        let device = u32::try_from(device_index).expect("设备数");
        let mut sectors: std::collections::BTreeSet<u64> = old.devices[device_index].sectors.keys().copied().collect();
        sectors.extend(new.devices[device_index].sectors.keys().copied());
        let mut open: Option<(u64, Vec<u8>, Vec<u8>)> = None;
        for sector in sectors {
            let sector_offset = sector * PHYSICAL_BLOCK_BYTES;
            let old_sector = old.devices[device_index].sectors.get(&sector).unwrap_or(&zero_sector);
            let new_sector = new.devices[device_index].sectors.get(&sector).unwrap_or(&zero_sector);
            for byte_index in 0..usize_from_u64(PHYSICAL_BLOCK_BYTES) {
                let absolute = sector_offset + byte_index as u64;
                let old_byte = old_sector[byte_index];
                let new_byte = new_sector[byte_index];
                if old_byte == new_byte {
                    if let Some((start, old_bytes, new_bytes)) = open.take() {
                        segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
                    }
                    continue;
                }
                let extends = matches!(&open, Some((start, old_bytes, _)) if *start + old_bytes.len() as u64 == absolute);
                if extends {
                    let (_, old_bytes, new_bytes) = open.as_mut().expect("刚判过 extends");
                    old_bytes.push(old_byte);
                    new_bytes.push(new_byte);
                } else {
                    if let Some((start, old_bytes, new_bytes)) = open.take() {
                        segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
                    }
                    open = Some((absolute, vec![old_byte], vec![new_byte]));
                }
            }
        }
        if let Some((start, old_bytes, new_bytes)) = open.take() {
            segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
        }
    }
    segments
}

pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join("")
}

#[cfg(test)]
#[path = "single_publish_flow_probes_and_pool_diff_tests.rs"]
mod tests;
