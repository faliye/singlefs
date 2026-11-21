//! 日志策略与读者模式、恢复结局与报告、读单元、选系统配置与根、日志扫描与重放、读分配树、走到文件、恢复。

use e7_index_bench::usize_from_u64;
use super::*;

// ───────────────────────── 冷启动恢复（里程碑步 6） ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum JournalPolicy {
    /// 全环扫描、按 D23 已定项 14 的五条口径取前缀、施加前逐项验证点名单元。
    Consult,
    /// 根本不看 journal：与 Consult 逐状态比，问 journal 在新池新建文件里承不承重。
    Ignore,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ReaderMode {
    /// 读者甲：第九、十次跑起的装置规则（`replay_journal`）：锚点按 (实例, txg) 与所选根相同的那条认；
    /// 无锚点时只认 txg = 根 txg + 1；不带提交标记即停；一条一条施加。新字段（标志、序号）照读、不解释。
    Primary,
    /// 读者乙：照 D23 已定项 4 / 17 / 14 第六条 / 已定项 14「这一版的失败处置」字面写（`replay_journal_clause`），
    /// 与读者甲各自独立实现、不共用代码（E142 第十四次跑第五节 5.1）——这是这一版真实基线的读法。
    Clause,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum RecoveryOutcome {
    /// 择到的根下面没有文件（第 0 代）。
    NoFile { root: (InstanceGeneration, CheckpointTxg) },
    /// 沿树走到数据单元、校验和全过，读回内容。
    FileRead { root: (InstanceGeneration, CheckpointTxg), content: Vec<u8> },
    /// 走不下去：哪一步、为什么。
    Failed { root: Option<(InstanceGeneration, CheckpointTxg)>, reason: String },
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct JournalScanReport {
    pub(super) valid_records: usize,
    pub(super) above_water: usize,
    pub(super) prefix_applied: usize,
    pub(super) verification_passed: usize,
    pub(super) verification_failed: usize,
}

/// 读者乙 ①–④ 四类分支各走到几次（第五节 5.1、第八节 S5）。合法历史（未改写的层 0 状态）里全部恒 0——
/// 这几格只有 P6 的改写镜像才走得到（已定项 4 依据）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct ReaderClauseBranchCounts {
    /// ①标志字节位 0 之外有值 ⇒ 这条记录当损坏。
    pub(super) corrupted_flag: u64,
    /// ①本次发布内序号为 0 ⇒ 这条记录当损坏。
    pub(super) corrupted_ordinal_zero: u64,
    /// ②同一 (实例, txg) 带标志的记录多于一条 ⇒ 停下（已定项 14「这一版的失败处置」）。
    pub(super) multiple_last_flag: u64,
    /// ③一次发布之内序号不是上一条 + 1 ⇒ 断。
    pub(super) ordinal_gap: u64,
    /// ③锚点读得出 / 无锚点起链时，下一次发布首条序号不是 1 ⇒ 断在这一条。
    pub(super) ordinal_not_one_at_publish_start: u64,
    /// ④同一 (实例, txg) 带标志的那条之后还有记录 ⇒ 断在带标志的那条、这次发布不施加。
    pub(super) trailing_after_last_flag: u64,
}

impl ReaderClauseBranchCounts {
    /// `main()` 里 `name=journal_header_reader_branch_counts` 那一行的合计。
    pub(super) fn total(&self) -> u64 {
        self.corrupted_flag + self.corrupted_ordinal_zero + self.multiple_last_flag + self.ordinal_gap + self.ordinal_not_one_at_publish_start + self.trailing_after_last_flag
    }
    pub(super) fn add(&mut self, other: &Self) {
        self.corrupted_flag += other.corrupted_flag;
        self.corrupted_ordinal_zero += other.corrupted_ordinal_zero;
        self.multiple_last_flag += other.multiple_last_flag;
        self.ordinal_gap += other.ordinal_gap;
        self.ordinal_not_one_at_publish_start += other.ordinal_not_one_at_publish_start;
        self.trailing_after_last_flag += other.trailing_after_last_flag;
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct RecoveryReport {
    pub(super) outcome: RecoveryOutcome,
    pub(super) journal: JournalScanReport,
    pub(super) mapping_fallbacks: usize,
    /// 只在 `ReaderMode::Clause` 下非零；`Primary` 与 `Ignore` 恒是默认值（全 0、`fatal=false`）。
    pub(super) clause_branches: ReaderClauseBranchCounts,
    pub(super) clause_fatal: bool,
}

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC32C 等于条目里的校验和才算读到（字节表三：无加密时那 4 字节就是整单元 CRC）。
fn read_unit_via_locations(reader: &dyn BlockReader, locations: &[LocationEntry; 2], unit_bytes: usize) -> Result<Vec<u8>, String> {
    for location in locations {
        if location.device.0 as usize >= reader.device_count() {
            continue;
        }
        let bytes = reader.read(location.device, location.slot.device_offset(), unit_bytes);
        if castagnoli_crc32(&bytes) == location.unit_checksum {
            return Ok(bytes);
        }
    }
    Err(format!("槽 {} 的两条位置条目都读不到校验和相符的单元", locations[0].slot.0))
}

pub(super) fn choose_system_configuration(reader: &dyn BlockReader) -> Result<SystemConfiguration, String> {
    let mut chosen: Option<SystemConfiguration> = None;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        let mut best_on_device: Option<SystemConfiguration> = None;
        for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            let bytes = reader.read(device, DeviceOffset(slot_offset), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
            if let Some(candidate) = SystemConfiguration::parse_slot(&bytes) {
                if best_on_device.as_ref().is_none_or(|best| candidate.slot_generation > best.slot_generation) {
                    best_on_device = Some(candidate);
                }
            }
        }
        let Some(best_on_device) = best_on_device else {
            return Err(format!("盘 {device_index} 两个系统配置槽都无效"));
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.fsid != best_on_device.fsid || previous.device_count != best_on_device.device_count {
                    return Err("两盘的系统配置 fsid 或设备数对不上".to_string());
                }
            }
        }
    }
    chosen.ok_or_else(|| "池里没有设备".to_string())
}

fn choose_root(reader: &dyn BlockReader, system_configuration: &SystemConfiguration) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    for region in 0..RING_REGIONS {
        let device = system_configuration.region_devices[usize_from_u64(region)];
        if device.0 as usize >= reader.device_count() {
            continue;
        }
        for slot in 0..RING_SLOTS_PER_REGION {
            let bytes = reader.read(device, ring_slot_offset(region, slot), usize_from_u64(PHYSICAL_BLOCK_BYTES));
            if let Some(candidate) = RootRecord::parse_slot(&bytes, &system_configuration.fsid) {
                let candidate_key = (candidate.checkpoint_txg, candidate.instance);
                if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance)) {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

/// 全环扫描（D23 已定项 3：不先信 tail），两份镜像任一份合法即算记录在（gap G9）。
/// 记录头带 fsid（D23 已定项 4）⇒ fsid 不符的记录一律不算数（I-1.4（块头 fsid 一致））。
pub(super) fn scan_journal(reader: &dyn BlockReader, expected_fsid: u64) -> BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord> {
    let mut records = BTreeMap::new();
    let ring_start = JOURNAL_START_SLOT * SLOT_BYTES;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        for sector in reader.written_sectors(device, DeviceOffset(ring_start), JOURNAL_RING_BYTES) {
            let offset = sector * PHYSICAL_BLOCK_BYTES;
            if !(offset - ring_start).is_multiple_of(JOURNAL_RECORD_BYTES) {
                continue;
            }
            let bytes = reader.read(device, DeviceOffset(offset), usize_from_u64(JOURNAL_RECORD_BYTES));
            if let Some(record) = JournalRecord::parse(&bytes) {
                if record.fsid != expected_fsid {
                    continue;
                }
                records.entry((record.instance, record.counter)).or_insert(record);
            }
        }
    }
    records
}

/// D23 已定项 14 的五条口径里新池新建文件碰得到的三条：jsn 严格连续、(实例代号, checkpoint_txg) 大于根的水位、提交标记齐全；
/// 在飞记录数上限（D23 已定项 18，第一版 65536）也进前缀判定。
/// 「施加一条记录」= 把所选根的七个字段换成记录新根段里的那七个（D23 已定项 15）——
/// 树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F、实例表单元指针、分配记录树根指针、卸载记号；
/// 只有 fsid 取所选根，实例代号与 checkpoint_txg 照记录头（那次发布的身份就在头里，所以新根段里不重复）。
fn replay_journal(
    reader: &dyn BlockReader,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>,
) -> (JournalScanReport, RootRecord) {
    let mut report = JournalScanReport { valid_records: records.len(), ..JournalScanReport::default() };
    let water = (root.instance, root.checkpoint_txg);
    let mut rebuilt = *root;
    // 前缀规则不跨实例边界（D23 已定项 14 第 1 条）：只有所选根自己那个实例的记录是候选，链从所选根覆盖的最后一条之后接；
    // 所选根是 mkfs 的第 0 代根时一条都不施加。
    let mut above: Vec<&JournalRecord> = records.values().filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water).collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条读不出时链首只认 checkpoint_txg = 根 txg + 1 的那条
    // （一次发布一条记录、txg 每次加一；与 crates 同一条规则；无锚点时不能无条件接上）。
    let root_own_record_counter = records.values().find(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg).map(|record| record.counter.0);
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = root_own_record_counter.map(|counter| (root.instance, JournalCounter(counter + 1)));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
    for record in above.into_iter().take(usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限")) {
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
        }
        expected = Some((record.instance, JournalCounter(record.counter.0 + 1)));
        if !record.is_commit {
            break;
        }
        let mut all_verified = true;
        for named in &record.named {
            let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                all_verified = false;
                continue;
            };
            for location in &named.locations {
                if location.device.0 as usize >= reader.device_count() {
                    continue;
                }
                let bytes = reader.read(location.device, location.slot.device_offset(), usize::try_from(unit_bytes).expect("单元字节数"));
                if castagnoli_crc32(&bytes) != location.unit_checksum {
                    all_verified = false;
                }
            }
        }
        if all_verified {
            report.verification_passed += 1;
            report.prefix_applied += 1;
            rebuilt = RootRecord {
                fsid: rebuilt.fsid,
                // D23 已定项 15：卸载记号、实例表指针、分配记录树根指针与前四项一样从记录新根段来，
                // 不照抄被施加那条根；只有 fsid 取所选根。
                unmount_marker: record.new_unmount_marker,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: record.new_instance_table,
                mapping_root: record.new_mapping_root,
                allocation_record_tree_root: record.new_allocation_record_tree_root,
            };
        } else {
            report.verification_failed += 1;
            break;
        }
    }
    (report, rebuilt)
}

/// 读者乙（E142 第十四次跑第五节 5.1）：D23 已定项 4 / 17 / 14 第六条 / 已定项 14「这一版的失败处置」字面写，
/// 独立于读者甲（`replay_journal`），不共用代码。E142 的写路径每次发布恰一条记录（G18）：`above` 在自然
/// （未改写的）层 0 状态里至多 1 条，下面的多记录批处理只服务 P6 的改写镜像。
/// 返回 (扫描报告, 重建出来的根, 分支计数, 是否触发「这一版的失败处置」那个停机)。
fn replay_journal_clause(
    reader: &dyn BlockReader,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>,
) -> (JournalScanReport, RootRecord, ReaderClauseBranchCounts, bool) {
    let mut branch = ReaderClauseBranchCounts::default();
    let mut report = JournalScanReport { valid_records: records.len(), ..JournalScanReport::default() };
    let mut rebuilt = *root;

    // ① 逐条先滤掉「当损坏」的记录：标志字节位 0 之外有值，或本次发布内序号为 0（已定项 4）。
    // 这两条判据与 `readable_record_count`（P6「读得出的记录条数」）共用同一对函数：改坏其中一个，
    // 两处观测（这里的候选集合、那边报出来的可读条数）会一起变，不会只影响其中一处。
    let mut usable: Vec<&JournalRecord> = Vec::new();
    for record in records.values() {
        if record_flag_byte_is_corrupted(record) {
            branch.corrupted_flag += 1;
            continue;
        }
        if record_ordinal_is_corrupted(record) {
            branch.corrupted_ordinal_zero += 1;
            continue;
        }
        usable.push(record);
    }
    usable.sort_by_key(|record| (record.instance, record.counter));

    let water = (root.instance, root.checkpoint_txg);
    let above: Vec<&JournalRecord> = usable.iter().filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water).copied().collect();
    report.above_water = above.len();

    // ② 锚点 = 所选根自己那次发布（同实例、同 txg）里带标志的记录；带标志的多于一条 ⇒ 停下（已定项 14「这一版的失败处置」）。
    let same_publish_as_root: Vec<&JournalRecord> = usable.iter().filter(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg).copied().collect();
    let flagged_at_root: Vec<&JournalRecord> = same_publish_as_root.iter().filter(|record| record.record_flags & 0x01 == 1).copied().collect();
    if flagged_at_root.len() > 1 {
        branch.multiple_last_flag += 1;
        return (report, rebuilt, branch, true);
    }
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = flagged_at_root.first().map(|record| (root.instance, JournalCounter(record.counter.0 + 1)));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);

    let mut index = 0usize;
    let in_flight_limit = usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限");
    while index < above.len() && index < in_flight_limit {
        let record = above[index];
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
        }
        // ③ 这次发布的第一条（无论是锚点接上的还是「无锚点」起链的）序号必须是 1，否则断在这一条。
        if record.ordinal_within_publish != 1 {
            branch.ordinal_not_one_at_publish_start += 1;
            break;
        }
        // 收集这一次发布的全部记录，直到带标志的那一条（序号连续、counter 紧接）；候选用尽还没见到标志 ⇒ 这次发布还没写完，链断在这里。
        let mut batch_end = index;
        loop {
            if batch_end > index {
                let previous = above[batch_end - 1];
                let current = above[batch_end];
                if (current.instance, current.counter) != (previous.instance, JournalCounter(previous.counter.0 + 1)) {
                    break;
                }
                if current.ordinal_within_publish != previous.ordinal_within_publish + 1 {
                    branch.ordinal_gap += 1;
                    break;
                }
            }
            if above[batch_end].record_flags & 0x01 == 1 {
                break;
            }
            if batch_end + 1 >= above.len() {
                batch_end += 1; // 越出候选集合，下面按「没见到标志」处理
                break;
            }
            batch_end += 1;
        }
        if batch_end >= above.len() || above[batch_end].record_flags & 0x01 != 1 {
            break;
        }
        let flagged_record = above[batch_end];
        // ④ 同一 (实例, txg) 带标志的那条之后还有记录 ⇒ 断在带标志的那条，这次发布不施加。
        if batch_end + 1 < above.len() && above[batch_end + 1].instance == flagged_record.instance && above[batch_end + 1].checkpoint_txg == flagged_record.checkpoint_txg {
            branch.trailing_after_last_flag += 1;
            break;
        }
        // ④ 走到带标志的那一条且带提交标记才整体施加。
        if !flagged_record.is_commit {
            break;
        }
        let mut publish_verified = true;
        for batch_record in &above[index..=batch_end] {
            let mut record_verified = true;
            for named in &batch_record.named {
                let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                    record_verified = false;
                    continue;
                };
                for location in &named.locations {
                    if location.device.0 as usize >= reader.device_count() {
                        continue;
                    }
                    let bytes = reader.read(location.device, location.slot.device_offset(), usize::try_from(unit_bytes).expect("单元字节数"));
                    if castagnoli_crc32(&bytes) != location.unit_checksum {
                        record_verified = false;
                    }
                }
            }
            if record_verified {
                report.verification_passed += 1;
                report.prefix_applied += 1;
            } else {
                report.verification_failed += 1;
                publish_verified = false;
            }
        }
        if !publish_verified {
            break;
        }
        rebuilt = RootRecord {
            fsid: rebuilt.fsid,
            unmount_marker: flagged_record.new_unmount_marker, // 三项取记录新根段（D23 已定项 15），不再照抄被施加那条根
            instance: flagged_record.instance,
            checkpoint_txg: flagged_record.checkpoint_txg,
            tree_table: flagged_record.new_tree_table,
            tree_identifier_watermark: flagged_record.new_tree_identifier_watermark,
            rollback_floor: flagged_record.new_rollback_floor,
            instance_table: flagged_record.new_instance_table,
            mapping_root: flagged_record.new_mapping_root,
            allocation_record_tree_root: flagged_record.new_allocation_record_tree_root,
        };
        index = batch_end + 1;
        expected = Some((flagged_record.instance, JournalCounter(flagged_record.counter.0 + 1)));
    }
    (report, rebuilt, branch, false)
}

struct TreeRoots {
    extent: IndexNodeHeader,
    pub(super) inode: IndexNodeHeader,
    accounting: IndexNodeHeader,
    mapping: IndexNodeHeader,
}

/// 分配记录树递归下探到叶（D8 已定项 14，按位置寻址）：每个节点核「header 里的 key 区间 == 它按位置规定罩的
/// 那一段」（D18 已定项 2 射程），不核「首末条目 key」——稠密内部节点的最后一格常是空格，M111 的会红检查
/// 就是防止这里退回码 2 树那种「区间 = 首末条目 key」的检查（那个检查对稠密内部节点会错判）。
#[allow(clippy::too_many_arguments, reason = "位置寻址要传全部几何上下文，收成结构体只会多一层没人验的名字")]
fn read_allocation_records(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64, device: u32, span_start: u64, span_length: u64, level: u32) -> Result<Vec<Vec<u8>>, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, usize_from_u64(NODE_BYTES))?;
    let node = parse_index_node(&bytes).map_err(|error| format!("分配记录树节点 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("分配记录树节点头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != ALLOCATION_KEY_BYTES {
        return Err(format!("分配记录树节点自述 key 宽 {}，要 {ALLOCATION_KEY_BYTES}", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("分配记录树节点诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("分配记录树节点 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("分配记录树节点出生序号与指针不符".to_string());
    }
    if u32::from(node.level) != level {
        return Err(format!("分配记录树节点自述层级 {} 与位置期待的 {level} 不符", node.level));
    }
    let expected_smallest = allocation_position_key(device, span_start);
    let expected_largest = allocation_position_key(device, span_start + span_length - 1);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("分配记录树节点的 key 区间与它按位置规定罩的那一段不符".to_string()); // D18 已定项 2 射程
    }
    if level == 0 {
        return Ok(node.entries);
    }
    let child_span = allocation_record_tree_span_at_level(level - 1);
    let mut collected = Vec::new();
    let mut previous_key: Option<Vec<u8>> = None;
    for cell in &node.entries {
        let key = &cell[..ALLOCATION_KEY_BYTES];
        if let Some(previous) = &previous_key {
            if key <= previous.as_slice() {
                return Err("分配记录树内部条目的 key 没有严格递增（D8 已定项 14 第 395 行「按 key 排」）".to_string());
            }
        }
        previous_key = Some(key.to_vec());
        let (child_device, child_span_start, child_pointer) = allocation_internal_cell_child_position(cell);
        if child_device != device {
            return Err(format!("分配记录树内部条目 key 的设备 {child_device} 与节点自己的设备 {device} 不符"));
        }
        if child_span_start < span_start || child_span_start >= span_start + span_length || !(child_span_start - span_start).is_multiple_of(child_span) {
            return Err("分配记录树内部条目 key 的槽号不是这个节点段内某个孩子位置的起点（ι甲）".to_string());
        }
        collected.extend(read_allocation_records(reader, tree, &child_pointer, root, expected_fsid, device, child_span_start, child_span, level - 1)?);
    }
    Ok(collected)
}

/// 分配记录树内部条目：把 key 解析成 (设备, 这个孩子按位置罩的那一段的起点)（ι甲，D8 已定项 14 第 395 行）
/// 与它的子指针，供内部节点与根共用的读路径解码一条条目。
fn allocation_internal_cell_child_position(cell: &[u8]) -> (u32, u64, NodePointer) {
    let mut reader = ByteReader::new(cell);
    let device = reader.get_u32();
    let slot = reader.get_six_byte_unsigned();
    let pointer = NodePointer::read_from(&mut reader);
    (device, slot, pointer)
}

/// 读路径上一块盘在分配记录树根里占几格：按池自己的每盘字节数算（写路按 `PoolParameters::device_slots`，两边同一个 δ 甲读法）。
pub(super) fn root_cells_per_device_of_reader(reader: &dyn BlockReader, root_level: u32) -> u64 {
    allocation_record_tree_cells_per_device(slots_of_device_bytes(reader.device_bytes()), root_level)
}

/// 分配记录树的根（单一节点，按盘分流，D8 已定项 14）：读根、按 β 甲核它的 key 区间、再按格分流到各盘的子树。
fn read_allocation_tree(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64, device_count: usize) -> Result<Vec<Vec<u8>>, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, usize_from_u64(NODE_BYTES))?;
    let node = parse_index_node(&bytes).map_err(|error| format!("分配记录树根 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("分配记录树根头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != ALLOCATION_KEY_BYTES {
        return Err(format!("分配记录树根自述 key 宽 {}，要 {ALLOCATION_KEY_BYTES}", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("分配记录树根诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("分配记录树根 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("分配记录树根出生序号与指针不符".to_string());
    }
    let root_level = u32::from(node.level);
    if root_level == 0 {
        return Ok(node.entries); // 极小几何：根本身就是叶（这个装置的固定几何走不到，留通用性）
    }
    let cells_per_device = root_cells_per_device_of_reader(reader, root_level);
    let level_one_span = allocation_record_tree_span_at_level(root_level - 1);
    // D8 已定项 14 第 395 行（唯一写法）：根罩整个 key 空间，与盘数、盘大小无关（第七节 A6）。
    let expected_smallest = allocation_position_key(0, 0);
    let expected_largest = allocation_position_key(u32::MAX, (1u64 << 48) - 1);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("分配记录树根 key 区间不是整个 key 空间（D8 已定项 14 第 395 行）".to_string());
    }
    let mut collected = Vec::new();
    let mut previous_key: Option<Vec<u8>> = None;
    for cell in &node.entries {
        let key = &cell[..ALLOCATION_KEY_BYTES];
        if let Some(previous) = &previous_key {
            if key <= previous.as_slice() {
                return Err("分配记录树根条目的 key 没有严格递增（D8 已定项 14 第 395 行「按 key 排」）".to_string());
            }
        }
        previous_key = Some(key.to_vec());
        let (device, child_span_start, child_pointer) = allocation_internal_cell_child_position(cell);
        if device as usize >= device_count {
            return Err(format!("分配记录树根条目 key 的设备号 {device} 越出设备数范围"));
        }
        if child_span_start % level_one_span != 0 || child_span_start / level_one_span >= cells_per_device {
            return Err("分配记录树根条目 key 的槽号不是某个孩子位置的起点（ι甲）".to_string());
        }
        collected.extend(read_allocation_records(reader, tree, &child_pointer, root, expected_fsid, device, child_span_start, level_one_span, root_level - 1)?);
    }
    Ok(collected)
}

fn read_tree_root(reader: &dyn BlockReader, kind: u16, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64) -> Result<IndexNodeHeader, String> {
    let entry = TreeTableEntry { kind, tree, root: *pointer, birth_txg: CheckpointTxg(0), head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE };
    let key_width = key_width_for_kind(entry.kind).ok_or_else(|| format!("树的种类 {} 没登记", entry.kind))?;
    let bytes = read_unit_via_locations(reader, &entry.root.locations, usize_from_u64(NODE_BYTES))?;
    // key 宽住偏移 51（D18 已定项 18）⇒ 解析器自己读得出来，不用调用方先告诉它。
    let node = parse_index_node(&bytes).map_err(|error| format!("树 {} 的根 {error:?}", entry.tree.0))?;
    if node.tree != entry.tree {
        return Err(format!("树 {} 的根头里写的树 ID 是 {}", entry.tree.0, node.tree.0)); // I-1.3
    }
    if node.key_width != key_width {
        return Err(format!("树 {} 的根自述 key 宽 {}，而这棵树的种类 {} 要 {key_width}", entry.tree.0, node.key_width, entry.kind));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err(format!("树 {} 的根诞生于根之后", entry.tree.0)); // I-1.2 的第一版读法
    }
    if node.fsid != expected_fsid {
        return Err(format!("树 {} 的根 fsid 不符", entry.tree.0)); // I-1.4
    }
    if node.birth_sequence != entry.root.birth_sequence {
        return Err(format!("树 {} 的根出生序号与指针不符", entry.tree.0));
    }
    // D8 已定项 11：条目里 key 一律是条目的前 key 宽 个字节 ⇒ 区间与条目对得上是逐字节比较。
    if let (Some(first), Some(last)) = (node.entries.first(), node.entries.last()) {
        if first[..key_width] != node.smallest_key[..] || last[..key_width] != node.largest_key[..] {
            return Err(format!("树 {} 的根 key 区间与条目不符", entry.tree.0)); // I-1.1 索引节点那一半
        }
    }
    Ok(node)
}

/// extent 树上段叶（层级 0，5.2 ζ：这个装置的几何只有一个 inode ⇒ 根就是叶）：D18 已定项 2 射程——
/// 按位置核 key 区间，不核「首末条目 key」（那是 `read_tree_root` 给码 2 btree 树用的通用检查）。
fn read_extent_upper_leaf(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64) -> Result<IndexNodeHeader, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, usize_from_u64(NODE_BYTES))?;
    let node = parse_index_node(&bytes).map_err(|error| format!("extent 树上段叶 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("extent 树上段叶头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != 24 {
        return Err(format!("extent 树上段叶自述 key 宽 {}，要 24", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("extent 树上段叶诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("extent 树上段叶 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("extent 树上段叶出生序号与指针不符".to_string());
    }
    if node.level != 0 {
        return Err("extent 树这个装置的几何只有一个 inode，上段的根恒是层级 0 的叶（5.2 ζ）".to_string());
    }
    let first_entry = node.entries.first().ok_or_else(|| "extent 树上段叶没有条目".to_string())?;
    let inode = u64::from_le_bytes(first_entry[8..16].try_into().expect("key 前 24 字节里第二段是 inode 号"));
    let upper_leaf_index = extent_upper_leaf_index(inode);
    let (expected_smallest, expected_largest) = extent_upper_leaf_positional_key_range(upper_leaf_index);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("extent 树上段叶 key 区间与它按位置规定罩的那一段不符".to_string()); // D18 已定项 2 射程
    }
    Ok(node)
}

fn walk_to_file(reader: &dyn BlockReader, root: &RootRecord, fsid: &[u8; 16], mapping_fallbacks: &mut usize) -> Result<Option<Vec<u8>>, String> {
    let expected_fsid = unit_fsid(fsid);
    // 实例表单元由根记录直接持有（D22 已定项 7）。
    let instance_table_bytes = read_unit_via_locations(reader, &root.instance_table.locations, usize_from_u64(DATA_UNIT_BYTES))?;
    let instance_table = parse_packed_unit(&instance_table_bytes).map_err(|error| format!("实例表 {error:?}"))?;
    if instance_table.identity.record_type != PACKED_TYPE_INSTANCE_TABLE || instance_table.records.is_empty() {
        return Err("实例表单元不是类型 4 或没有链指针记录".to_string());
    }
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, usize_from_u64(NODE_BYTES))?;
    let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| format!("树表 {error:?}"))?;
    if tree_table.key_width != TREE_TABLE_KEY_WIDTH {
        return Err(format!("树表单元自述 key 宽 {}，而它的 key 是树 ID", tree_table.key_width));
    }
    if tree_table.entries.is_empty() {
        return Ok(None);
    }
    let entries: Vec<TreeTableEntry> = tree_table.entries.iter().map(|bytes| TreeTableEntry::parse(bytes)).collect::<Result<_, _>>().map_err(|error| format!("树表条目 {error:?}"))?;
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
    let mut allocation_records: Option<Vec<Vec<u8>>> = None;
    let mut extent_root: Option<IndexNodeHeader> = None;
    for entry in &entries {
        if entry.tree.0 >= root.tree_identifier_watermark {
            return Err("树 ID 不低于水位".to_string()); // I-7.8
        }
        if entry.root.is_empty_root() {
            // day-1 注册、还没有根的树（livelist、稀疏旁表、deadlist，以及种类 9..12 的 `deleted_inodes`、墓碑树、谱系树、logged_ops，
            // D8 已定项 9 登记表）：不读根、不报错——不论种类，零根一律跳过（M208 把这一句限成种类 ≤ 8）。
            continue;
        }
        if entry.kind == TREE_KIND_ALLOCATION {
            // 按位置寻址（D8 已定项 14），不走通用的 `read_tree_root`：根不是唯一节点，要递归下探到叶。
            allocation_records = Some(read_allocation_tree(reader, entry.tree, &entry.root, root, expected_fsid, reader.device_count())?);
            continue;
        }
        if entry.kind == TREE_KIND_EXTENT {
            // 按位置寻址（D8 已定项 14），核区间不核首末条目 key（D18 已定项 2 射程）。
            extent_root = Some(read_extent_upper_leaf(reader, entry.tree, &entry.root, root, expected_fsid)?);
            continue;
        }
        by_kind.insert(entry.kind, read_tree_root(reader, entry.kind, entry.tree, &entry.root, root, expected_fsid)?);
    }
    let mut take = |kind: u16, name: &str| by_kind.remove(&kind).ok_or_else(|| format!("树表里没有{name}"));
    let roots = TreeRoots {
        extent: extent_root.ok_or_else(|| "树表里没有 extent 树".to_string())?,
        inode: take(TREE_KIND_INODE, "inode 树")?,
        accounting: take(TREE_KIND_ACCOUNTING, "记账树")?,
        // 中央映射树的根住根记录，不进树表（D19 已定项 11）。
        mapping: read_tree_root(reader, TREE_KIND_MAPPING, TreeIdentifier(TREE_IDENTIFIER_MAPPING), &root.mapping_root, root, expected_fsid)?,
    };
    let allocation_records = allocation_records.ok_or_else(|| "树表里没有分配记录树".to_string())?;
    // 每盘 (10 + 2 × 盘数) 条：6 个既有单元 + 分配记录树自己 (2 × 盘数 + 1) 个节点 + 记账 / 映射 / 树表
    // （两块盘时 10 + 4 = 14，一块盘时 10 + 2 = 12，第七节 B4、B10）。
    let expected_allocation_records = (10 + 2 * reader.device_count()) * reader.device_count();
    if allocation_records.len() != expected_allocation_records {
        return Err(format!("分配记录数 {} 不是 (10 + 2 × 盘数) × 盘数 = {expected_allocation_records}", allocation_records.len()));
    }
    // D5 已定项 8：池级三行（待删占用、已承诺预留、inode 号水位）+ 每盘六行
    // （已分配字节、空闲字节、不可回收、defer 待释放、碎片度 runs、全空聚簇段数）⇒ 两盘 15 行。
    if roots.accounting.entries.len() != 3 + 6 * reader.device_count() {
        return Err(format!("记账条目数 {} 不是 3 + 6 × 盘数", roots.accounting.entries.len()));
    }
    // 6 个既有单元（数据、extent、inode 叶、inode 根、分配记录树根、记账）+ 分配记录树每盘一对（叶 + 层级 1 节点）；
    // 两块盘时 6 + 4 = 10（第七节 B7），一块盘时 6 + 2 = 8（第七节 B10 的对照臂）。
    let expected_mapping_entries = 6 + 2 * reader.device_count();
    if roots.mapping.entries.len() != expected_mapping_entries {
        return Err(format!("映射条目数 {} 不是 {expected_mapping_entries}", roots.mapping.entries.len()));
    }
    for record_bytes in &allocation_records {
        let record = AllocationRecord::parse(record_bytes);
        if record.span_slots & 0x8000 != 0 || record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err("分配记录带已释放标志、跨度为 0 或分配代晚于根".to_string()); // 新池新建文件没有释放
        }
    }
    for entry_bytes in &roots.accounting.entries {
        let entry = AccountingEntry::parse(entry_bytes);
        if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
            return Err("记账条目的代晚于根或 seq 为 0".to_string());
        }
    }

    // inode 树：根（码 2）→ 内部条目 → 叶容器（码 3）→ 记录。
    if roots.inode.level != 1 {
        return Err("inode 树根层级不是 1".to_string());
    }
    let mut inode_record: Option<InodeRecord> = None;
    for entry in &roots.inode.entries {
        let (separator_key, identity, child) = parse_inode_internal_entry(entry);
        if identity.record_type != PACKED_TYPE_INODE {
            return Err("inode 内部条目类型段不是 2".to_string());
        }
        let leaf_bytes = read_unit_via_locations(reader, &child.locations, usize_from_u64(DATA_UNIT_BYTES))?;
        let leaf = parse_packed_unit(&leaf_bytes).map_err(|error| format!("inode 叶 {error:?}"))?;
        if leaf.identity != identity || leaf.record_width as u64 != INODE_RECORD_BYTES || leaf.fsid != expected_fsid {
            return Err("inode 叶头与条目身份引用不符".to_string()); // I-9.2
        }
        if child.head.birth_tree != identity.birth_tree || leaf.birth_sequence != child.birth_sequence {
            return Err("inode 叶的出生身份与子指针不符".to_string());
        }
        if leaf.birth_txg > root.checkpoint_txg || leaf.write_order.instance != child.instance {
            return Err("inode 叶诞生于根之后或写序实例与子指针不符".to_string()); // I-1.2 码 3 那一半
        }
        for record_bytes in &leaf.records {
            let record = InodeRecord::parse(record_bytes).map_err(|error| format!("inode 记录 {error:?}"))?;
            if record.inode < separator_key || record.inode < identity.container {
                return Err("inode 记录号小于分隔 key 或容器号".to_string()); // I-9.4 / I-9.12
            }
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    let Some(inode_record) = inode_record else {
        return Ok(None);
    };

    // extent 树：上段叶按 (locality 0, inode 1, 第三分量 0) 找条目（5.2 ζ：根就是上段叶）；标签 2 才是内联数据指针，
    // 解引用先按位置提示、校验和不对再查映射（D19 已定项 5）。
    let mut wanted_key = [0u8; 24];
    wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    let mut data_unit: Option<(DataPointer, Vec<u8>)> = None;
    for record_bytes in &roots.extent.entries {
        let (key, tag, pointer) = parse_extent_upper_leaf_entry(record_bytes);
        if key != wanted_key {
            continue;
        }
        if tag != EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT {
            return Err(format!(
                "extent 上段叶条目标签 {tag} 不是内联数据指针（标签 {EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT} 没有单元、\
                 标签 {EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT} 要走下段，这个装置的新池新建文件走不到）"
            ));
        }
        let bytes = match read_unit_via_locations(reader, &pointer.locations, usize_from_u64(DATA_UNIT_BYTES)) {
            Ok(bytes) => bytes,
            Err(hint_error) => {
                *mapping_fallbacks += 1;
                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
                let mapped = roots.mapping.entries.iter().map(|entry| parse_mapping_entry(entry)).find(|(listed_key, _)| *listed_key == mapping_key);
                let Some((_, locations)) = mapped else {
                    return Err(format!("{hint_error}；映射里也没有这个 key"));
                };
                read_unit_via_locations(reader, &locations, usize_from_u64(DATA_UNIT_BYTES)).map_err(|error| format!("经映射仍读不到：{error}"))?
            }
        };
        data_unit = Some((pointer, bytes));
    }
    let Some((pointer, bytes)) = data_unit else {
        return Err("extent 树里没有这个文件的记录".to_string());
    };
    let header = parse_data_unit(&bytes).map_err(|error| format!("数据单元 {error:?}"))?;
    if header.identity.tree != TreeIdentifier(TREE_IDENTIFIER_EXTENT) || header.identity.object != FIRST_INODE_NUMBER || header.identity.anchor_offset != 0 {
        return Err("数据单元五元组与查找路径不符".to_string()); // I-1.1
    }
    if header.identity.object_birth != inode_record.object_birth {
        return Err("对象出生代与 inode 记录不符".to_string()); // I-9.10
    }
    if header.birth_txg != pointer.head.birth_txg || header.write_order != pointer.write_order || header.fsid != expected_fsid {
        return Err("数据单元头与指针的出生身份不符".to_string());
    }
    if u64::from(header.declared_length) != inode_record.size {
        return Err("声明长度与 inode size 不符".to_string());
    }
    if bytes[usize_from_u64(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) + header.declared_length as usize..].iter().any(|&byte| byte != 0) {
        return Err("补齐字节非零".to_string()); // I-2.3
    }
    Ok(Some(data_unit_payload(&bytes, header.declared_length).to_vec()))
}

#[allow(clippy::needless_pass_by_value, reason = "参数是 Copy 的策略枚举，按值取更贴调用点")]
pub(super) fn recover(reader: &dyn BlockReader, policy: JournalPolicy, mode: ReaderMode) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let system_configuration = match choose_system_configuration(reader) {
        Ok(system_configuration) => system_configuration,
        Err(reason) => {
            return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason }, journal: JournalScanReport::default(), mapping_fallbacks, clause_branches: ReaderClauseBranchCounts::default(), clause_fatal: false };
        }
    };
    let Some(root) = choose_root(reader, &system_configuration) else {
        return RecoveryReport {
            outcome: RecoveryOutcome::Failed { root: None, reason: "根环里一条合法根都没有".to_string() },
            journal: JournalScanReport::default(),
            mapping_fallbacks,
            clause_branches: ReaderClauseBranchCounts::default(),
            clause_fatal: false,
        };
    };
    // 报出去的 `root=` 恒是**所选**的那条根；施加记录之后走的是重建出来的根（D23 已定项 15）。
    let root_key = (root.instance, root.checkpoint_txg);
    let mut clause_branches = ReaderClauseBranchCounts::default();
    let mut clause_fatal = false;
    let (journal, effective_root) = match policy {
        JournalPolicy::Consult => match mode {
            ReaderMode::Primary => replay_journal(reader, &root, &scan_journal(reader, unit_fsid(&system_configuration.fsid))),
            ReaderMode::Clause => {
                let (journal, effective_root, branches, fatal) = replay_journal_clause(reader, &root, &scan_journal(reader, unit_fsid(&system_configuration.fsid)));
                clause_branches = branches;
                clause_fatal = fatal;
                (journal, effective_root)
            }
        },
        JournalPolicy::Ignore => (JournalScanReport::default(), root),
    };
    if clause_fatal {
        // 已定项 14「这一版的失败处置」：同一 (实例代号, checkpoint_txg) 带末条标志的记录多于一条，恢复在任何写之前停下
        // （`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。
        return RecoveryReport {
            outcome: RecoveryOutcome::Failed { root: Some(root_key), reason: "root_publish_carries_more_than_one_last_record_flag_whose_anchor_is_undecided".to_string() },
            journal,
            mapping_fallbacks,
            clause_branches,
            clause_fatal,
        };
    }
    let outcome = match walk_to_file(reader, &effective_root, &system_configuration.fsid, &mut mapping_fallbacks) {
        Ok(Some(content)) => RecoveryOutcome::FileRead { root: root_key, content },
        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
        Err(reason) => RecoveryOutcome::Failed { root: Some(root_key), reason },
    };
    RecoveryReport { outcome, journal, mapping_fallbacks, clause_branches, clause_fatal }
}

#[cfg(test)]
#[path = "recovery_and_journal_replay_tests.rs"]
mod tests;
