//! 完整流水线与日志记录被改坏的各种场景、结构集合的比较、窗口里的写。

use e7_index_bench::usize_from_u64;
use super::*;

/// 整条路（mkfs → 取号 → 暖机两次 → 新池新建文件）跑一遍，返回落盘后的整份镜像与这次发布的产出。
/// E142 第十一次跑用它跑出 M_A（change_count = NEW_POOL_FILE_CREATION_TXG）、M_B（真实基线，change_count = 1）与量 8 的两个几何取样点。
pub(super) fn run_full_pipeline(parameters: &PoolParameters, change_count: u64, file_bytes: &[u8]) -> (Pool, TransactionOutput) {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), change_count);
    (recording.pool.clone(), output)
}

/// P6（E142 第十四次跑第五节 5.2）用：跑到「t9 之前（含 t9 两份）全部持久、t10 根槽与 t11 系统配置槽都没持久」
/// 那个定点状态 S*，返回 (暖机之后的池, 新池新建文件的全部写请求, 这次发布的产出)。`persisted_mask_up_to_the_journal_record_fixed_state` 配它用：
/// 根槽 FUA 与系统配置槽两类写标 false，其余（8 个单元 × 2 盘 + journal 记录 × 2 盘）标 true。
pub(super) fn writes_up_to_the_journal_record_fixed_state(parameters: &PoolParameters, file_bytes: &[u8]) -> (Pool, Vec<WriteRequest>, TransactionOutput) {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let base = recording.pool.clone();
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    let (writes, _segments) = split_into_segments(&recording.operations[warm_up_operation_count..], false);
    (base, writes, output)
}

pub(super) fn persisted_mask_up_to_the_journal_record_fixed_state(writes: &[WriteRequest]) -> Vec<bool> {
    writes.iter().map(|write| !matches!(write.kind, StepKind::RootRecordFua | StepKind::SystemConfigurationSlot)).collect()
}

/// 把 `writes` 里两份 t9 记录（两盘各一份 `StepKind::JournalRecord`）解析回 `JournalRecord`、按 `mutate` 改一个字段、
/// 重封头校验和（D23 已定项 13）再写回字节——P6 的「两份改写」都靠它，不手改字节偏移。
fn corrupt_the_journal_record_copies(writes: &[WriteRequest], mutate: impl Fn(&mut JournalRecord)) -> Vec<WriteRequest> {
    writes
        .iter()
        .map(|write| {
            if write.kind == StepKind::JournalRecord {
                let mut record = JournalRecord::parse(&write.bytes).expect("t9 记录自检要过");
                mutate(&mut record);
                WriteRequest { bytes: record.to_bytes(), ..write.clone() }
            } else {
                write.clone()
            }
        })
        .collect()
}

/// P6「t9 之后在计数器 4 的槽里加一条同 txg 3 的记录」：两盘各一份，序号 2、标志 0x00、提交标记 1、
/// 反向链 = CRC32C(t9 的头)、不点名（重跑登记第五节 5.2 那一行）。
fn append_trailing_same_txg_record(writes: &[WriteRequest], journal_record: &JournalRecord, journal_record_bytes: &[u8], parameters: &PoolParameters) -> Vec<WriteRequest> {
    let extra = JournalRecord {
        instance: journal_record.instance,
        counter: JournalCounter(4),
        checkpoint_txg: journal_record.checkpoint_txg,
        transaction: TransactionNumber(0),
        is_commit: true,
        ordinal_within_publish: 2,
        record_flags: 0x00,
        back_chain: journal_back_chain(journal_record_bytes),
        fsid: journal_record.fsid,
        new_tree_table: journal_record.new_tree_table,
        new_mapping_root: journal_record.new_mapping_root,
        new_tree_identifier_watermark: journal_record.new_tree_identifier_watermark,
        new_rollback_floor: journal_record.new_rollback_floor,
        new_instance_table: journal_record.new_instance_table,
        new_allocation_record_tree_root: journal_record.new_allocation_record_tree_root,
        new_unmount_marker: journal_record.new_unmount_marker,
        named: Vec::new(),
    };
    let extra_bytes = extra.to_bytes();
    let extra_offset = journal_record_offset(JournalCounter(4));
    let mut extended = writes.to_vec();
    for device in parameters.devices() {
        extended.push(WriteRequest { device, offset: extra_offset, bytes: extra_bytes.clone(), kind: StepKind::JournalRecord });
    }
    extended
}

/// 已定项 4 的①：标志字节位 0 之外有值 ⇒ 当损坏。`replay_journal_clause` 的候选过滤与
/// `readable_record_count` 共用这一个判据，不各写一份——各写一份会让改坏其中一处的变异
/// 只影响一个观测点，另一个观测点看不出来（M84 踩过这个坑）。
pub(super) fn record_flag_byte_is_corrupted(record: &JournalRecord) -> bool {
    record.record_flags & !0x01 != 0
}

/// 已定项 4 的①：本次发布内序号为 0 ⇒ 当损坏。同上，与 `replay_journal_clause` 共用。
pub(super) fn record_ordinal_is_corrupted(record: &JournalRecord) -> bool {
    record.ordinal_within_publish == 0
}

/// P6 读者读了标志与序号一格里，「读得出的记录条数」（按 jsn 去重，已定项 4 的①判「当损坏」的那两条）。
fn readable_record_count(records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>) -> usize {
    records.values().filter(|record| !record_flag_byte_is_corrupted(record) && !record_ordinal_is_corrupted(record)).count()
}

pub(super) fn journal_without_root_mutate_flag_0x03(record: &mut JournalRecord) {
    record.record_flags = 0x03;
}
pub(super) fn journal_without_root_mutate_ordinal_0(record: &mut JournalRecord) {
    record.ordinal_within_publish = 0;
}
pub(super) fn journal_without_root_mutate_flag_0x00(record: &mut JournalRecord) {
    record.record_flags = 0x00;
}
pub(super) fn journal_without_root_mutate_ordinal_2(record: &mut JournalRecord) {
    record.ordinal_within_publish = 2;
}
pub(super) fn journal_without_root_mutate_txg_2(record: &mut JournalRecord) {
    record.checkpoint_txg = CheckpointTxg(2);
}

/// `recover()` 的 `outcome.root` 恒是**所选**的那条根（施加记录之前），不是施加之后的——`recover` 函数上的注释
/// 与 `eight_byte_flip_probes_recover_as_the_e142_product_records` 那条已有测试都是这个口径。P6 的预期表问的是「施加记录之后」
/// 的 (实例代号, checkpoint_txg)（重跑登记第五节 5.2 表头），S* 的 `above` 至多 1 条（t9），
/// 所以用 `journal.prefix_applied` 是否 ≥ 1 就等价于「重建出来的根有没有推进到 t9 那次发布」。
#[derive(Debug, Clone)]
pub(super) struct JournalWithoutRootOutcome {
    pub(super) primary_prefix_applied: usize,
    pub(super) primary_readable: usize,
    pub(super) clause_prefix_applied: usize,
    pub(super) clause_readable: usize,
    pub(super) clause_fatal: bool,
}

/// P6（第五节 5.2）跑一格：在 S*（t9 之前含 t9 两份全部持久、t10 根槽与 t11 系统配置槽都没持久）上，
/// 可选地改写 t9（`mutate`）或在 t9 之后加一条同 txg 的记录（`append_trailing`），两种读者各跑一次。
pub(super) fn run_journal_without_root_scenario(parameters: &PoolParameters, file_bytes: &[u8], mutate: Option<fn(&mut JournalRecord)>, append_trailing: bool) -> JournalWithoutRootOutcome {
    let (base, base_writes, output) = writes_up_to_the_journal_record_fixed_state(parameters, file_bytes);
    let mut writes = base_writes;
    if let Some(mutate) = mutate {
        writes = corrupt_the_journal_record_copies(&writes, mutate);
    }
    if append_trailing {
        writes = append_trailing_same_txg_record(&writes, &output.record, &output.record_bytes, parameters);
    }
    let persisted = persisted_mask_up_to_the_journal_record_fixed_state(&writes);
    let image = CrashImage { base: &base, writes: &writes, persisted };
    let primary = recover(&image, JournalPolicy::Consult, ReaderMode::Primary);
    let clause = recover(&image, JournalPolicy::Consult, ReaderMode::Clause);
    // 读者甲不判标志/序号是否合法，「读得出」= 扫描到、校验和过的全部记录；读者乙额外按①滤掉当损坏的那些。
    let records = scan_journal(&image, unit_fsid(&parameters.fsid));
    JournalWithoutRootOutcome {
        primary_prefix_applied: primary.journal.prefix_applied,
        primary_readable: records.len(),
        clause_prefix_applied: clause.journal.prefix_applied,
        clause_readable: readable_record_count(&records),
        clause_fatal: clause.clause_fatal,
    }
}

/// 差异段所属结构名去重（量 8 的「结构集合」）。
pub(super) fn structure_name_set(differences: &[DiffSegment], catalog: &StructureCatalog) -> std::collections::BTreeSet<&'static str> {
    differences.iter().map(|segment| classify_offset(catalog, segment.device, segment.offset).map_or("unregistered", |(structure, _)| structure)).collect()
}

/// 差异段的完整位置集合（设备 + 结构 + 结构内偏移，不含值——量 8 的判据要求「不含值」）。
pub(super) fn structure_position_set(differences: &[DiffSegment], catalog: &StructureCatalog) -> std::collections::BTreeSet<(u32, &'static str, u64)> {
    differences
        .iter()
        .map(|segment| {
            let (structure, offset) = classify_offset(catalog, segment.device, segment.offset).unwrap_or(("unregistered", segment.offset));
            (segment.device, structure, offset)
        })
        .collect()
}

/// 量 8 的判据：结构集合逐项相等（不是「相差 ≤ 1 段」的容差版——V6 变异测的就是这一句）。
pub(super) fn structure_sets_match(left: &std::collections::BTreeSet<&str>, right: &std::collections::BTreeSet<&str>) -> bool {
    left == right
}

// ═════════ E142 第十五次跑步④（`research/prompts/e142-r15-prereg.md` 第六节）：Q142.1–Q142.8 的支撑函数 ═════════
// 窗口 = `transaction_operations`（暖机最后一次写之后到发布结束），与比对侧
// `crates/singlefs-implementation-experiments/src/bin/e142_new_pool_file_creation_write_dump.rs` 的窗口取法同一个切点（P5）。
// 配对键是 (设备, 偏移, 长度)，不按名字配对（R3）；`region=`/`unit=` 只是给人看的标签，来自 `classify_offset`。

/// 窗口里模型自己的一次写：给 Q142.1、Q142.3、Q142.8 共用。
pub(super) struct ModelWindowWrite {
    pub(super) step: usize,
    pub(super) region: &'static str,
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) bytes: Vec<u8>,
}

pub(super) fn model_window_writes(catalog: &StructureCatalog, transaction_operations: &[RecordedOperation]) -> Vec<ModelWindowWrite> {
    let mut out = Vec::new();
    for (step, operation) in transaction_operations.iter().enumerate() {
        if let RecordedOperation::Write(write) = operation {
            let region = classify_offset(catalog, write.device.0, write.offset.0).map_or("unregistered", |(label, _)| label);
            out.push(ModelWindowWrite { step, region, device: write.device.0, offset: write.offset.0, bytes: write.bytes.clone() });
        }
    }
    out
}

/// 第八节 G4 用：跑一遍 mkfs → 取号 → 暖机 → 发布，返回窗口（暖机最后一次写之后到发布结束）里模型自己的
/// 写清单——与 `main()` 里对主几何跑的那一遍是同一条逻辑，只是几何参数不同（G4 传一盘的参数）。
pub(super) fn model_window_writes_for_parameters(parameters: &PoolParameters, file_bytes: &[u8]) -> Vec<ModelWindowWrite> {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let (root_region, root_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let root_device = parameters.region_devices[usize_from_u64(root_region)].0;
    let root_offset = ring_slot_offset(root_region, root_slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    model_window_writes(&catalog, transaction_operations)
}

#[cfg(test)]
#[path = "journal_without_root_scenarios_and_window_writes_tests.rs"]
mod tests;
