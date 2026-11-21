use e7_index_bench::usize_from_u64;
use super::*;

#[test]
fn cold_start_reads_the_file_back_and_chooses_instance_one_txg_three() {
    let BuiltPool { recording, .. } = built_pool();
    let report = recover(&recording.pool, JournalPolicy::Consult, ReaderMode::Clause);
    assert_eq!(report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(3)), content: sample_file() });
    assert_eq!(report.journal.valid_records, 3, "两条暖机空记录 + 事务记录");
    // 记录头带 fsid：把三条记录的 fsid 段全改成别的池，扫描一条都不认（I-1.4）。
    let mut alien = recording.pool.clone();
    for counter in 1..=3u64 {
        let offset = journal_record_offset(JournalCounter(counter));
        for device_index in 0..2u32 {
            let device = DeviceIdentity(device_index);
            let mut record = alien.read(device, offset, usize_from_u64(JOURNAL_RECORD_BYTES));
            record[463..471].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes()); // fsid 偏移 463（已定项 4，487 字节头的字段表）
            let digest = wide_checksum_with_field_zeroed(&record, usize_from_u64(JOURNAL_RECORD_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET);
            record[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            alien.devices[device.0 as usize].write(offset, &record);
        }
    }
    assert_eq!(scan_journal(&alien, unit_fsid(&FIXED_FSID)).len(), 0, "fsid 不符的记录不算数");
    assert_eq!(scan_journal(&alien, 0xDEAD_BEEF).len(), 3, "换成它们自己的 fsid 就全认——证明上一行不是因为校验和坏了");
    assert_eq!(report.journal.above_water, 0, "记录 txg 1..3 都不高于根的水位 3，不施加");
    assert_eq!(report.mapping_fallbacks, 0);
}

/// M152 的会红检查：D16（发布语义） 已定项 7「恢复重放在施加任何记录之前，必须逐项验证点名单元的
/// 校验和」——主臂根停在暖机第二次（txg 2），新池新建文件自己的记录（txg 3，与根同一个实例，重放不会
/// 卡在实例边界）已持久，但它点名的 12 个单元里 t1（数据单元）两份都没持久：验证不过就不该施加，
/// 应该停在暖机的旧态（没有文件）。
#[test]
fn main_arm_transaction_record_is_not_applied_when_a_named_unit_is_missing() {
    let (base, writes, output) = writes_up_to_the_journal_record_fixed_state(&PoolParameters::settled_two_devices(), &sample_file());
    let mut persisted = persisted_mask_up_to_the_journal_record_fixed_state(&writes);
    let (slot, unit, _) = &output.units_by_slot[0];
    assert_eq!(*unit, TransactionUnit::Data, "第 0 个单元是数据单元（t1）");
    let offset = slot.device_offset();
    let mut cleared = 0;
    for (index, write) in writes.iter().enumerate() {
        if write.kind == StepKind::UnitWrite && write.offset == offset {
            persisted[index] = false;
            cleared += 1;
        }
    }
    assert_eq!(cleared, 2, "t1 两份镜像都要抹掉");
    let image = CrashImage { base: &base, writes: &writes, persisted };
    let report = recover(&image, JournalPolicy::Consult, ReaderMode::Clause);
    match &report.outcome {
        RecoveryOutcome::NoFile { .. } => {}
        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::Failed { .. } => panic!("t1 缺一份都不该施加这条记录，应停在暖机旧态（NoFile），实际是 {:?}", report.outcome),
    }
}

/// M31 的会红检查：读者甲（`replay_journal`）施加记录时必须用记录里的新树表指针重建根，
/// 不能沿用旧根的树表指针——沿用旧的会让施加之后的走读找不到刚发布的文件。
#[test]
fn primary_reader_rebuilds_the_tree_table_pointer_when_applying_a_record() {
    let BuiltPool { recording, .. } = built_pool();
    let parameters = PoolParameters::settled_two_devices();
    let probe = probes(&parameters).into_iter().find(|probe| probe.name == "newest_root_slot_one_byte").expect("探针在（第三节 probes()）");
    let mut damaged = recording.pool.clone();
    for (device, offset, byte_index) in &probe.flips {
        flip_byte(&mut damaged, *device, *offset, *byte_index);
    }
    let report = recover(&damaged, JournalPolicy::Consult, ReaderMode::Primary);
    assert!(
        matches!(&report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(2)), content } if *content == sample_file()),
        "最新根槽坏了⇒择回暖机第 2 代根、施加 jsn 3 那条记录重建那次发布的根，用新树表指针才读得到文件：{:?}",
        report.outcome
    );
    assert_eq!(report.journal.prefix_applied, 1);
}

/// M67 的会红检查：D23 已定项 14 第 1 条「前缀规则不跨实例边界」——所选根是 mkfs 的第 0 代根
/// （实例代号 0）时，别的实例（哪怕 checkpoint_txg 更大）的记录一条都不许施加。
#[test]
fn primary_reader_does_not_apply_records_from_a_newer_instance_onto_the_genesis_root() {
    let parameters = PoolParameters::settled_two_devices();
    let (recording, genesis) = mkfs(&parameters);
    let base = recording.pool.clone();
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let record_from_a_newer_instance = JournalRecord {
        instance,
        counter: JournalCounter(1),
        checkpoint_txg: CheckpointTxg(1),
        transaction: TransactionNumber(0),
        is_commit: true,
        ordinal_within_publish: 1,
        record_flags: 0x01,
        back_chain: 0,
        fsid: unit_fsid(&parameters.fsid),
        new_tree_table: genesis.root.tree_table,
        new_mapping_root: genesis.root.mapping_root,
        new_tree_identifier_watermark: genesis.root.tree_identifier_watermark,
        new_rollback_floor: genesis.root.rollback_floor,
        new_instance_table: genesis.root.instance_table,
        new_allocation_record_tree_root: genesis.root.allocation_record_tree_root,
        new_unmount_marker: genesis.root.unmount_marker,
        named: Vec::new(),
    };
    let record_bytes = record_from_a_newer_instance.to_bytes();
    let writes: Vec<WriteRequest> = parameters
        .devices()
        .into_iter()
        .map(|device| WriteRequest { device, offset: journal_record_offset(JournalCounter(1)), bytes: record_bytes.clone(), kind: StepKind::JournalRecord })
        .collect();
    let persisted = vec![true; writes.len()];
    let image = CrashImage { base: &base, writes: &writes, persisted };
    let report = recover(&image, JournalPolicy::Consult, ReaderMode::Primary);
    assert_eq!(outcome_root(&report.outcome), "0:0", "所选根是 mkfs 第 0 代根（实例 0），实例 1 的记录不跨实例边界，一条都不施加");
    assert_eq!(report.journal.prefix_applied, 0);
    assert_eq!(report.journal.above_water, 0, "前缀规则不跨实例边界：above_water 在过滤之后就该是 0");
}

/// 施加的合成场景：所选根（实例 1、txg 2）与它之后的一条记录（txg 3），记录新根段的后三项与所选根的不同——
/// 施加出来的根要带记录里的（M214–M217 的会红检查；这条流里两者本来相同，主产物上看不出，要靠这条）。
fn synthetic_apply_case() -> (RootRecord, JournalRecord) {
    let parameters = PoolParameters::settled_two_devices();
    let (_, genesis) = mkfs(&parameters);
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let root = RootRecord { instance, checkpoint_txg: CheckpointTxg(2), ..genesis.root };
    let other_pointer = |slot: u64, checksum: u32| NodePointer {
        head: PointerHead { birth_tree: TreeIdentifier(65537), birth_txg: CheckpointTxg(2) },
        locations: [
            LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(slot), unit_checksum: checksum },
            LocationEntry { device: DeviceIdentity(1), slot: SlotNumber(slot), unit_checksum: checksum },
        ],
        instance,
        birth_sequence: BirthSequence(5),
    };
    let record = JournalRecord {
        instance,
        counter: JournalCounter(3),
        checkpoint_txg: CheckpointTxg(3),
        transaction: TransactionNumber(1),
        is_commit: true,
        ordinal_within_publish: 1,
        record_flags: 0x01,
        back_chain: 0,
        fsid: unit_fsid(&parameters.fsid),
        new_tree_table: other_pointer(60_000, 0x1111),
        new_mapping_root: other_pointer(60_001, 0x2222),
        new_tree_identifier_watermark: 65549,
        new_rollback_floor: CheckpointTxg(0),
        new_instance_table: other_pointer(60_002, 0x3333),
        new_allocation_record_tree_root: other_pointer(60_003, 0x4444),
        new_unmount_marker: true,
        named: Vec::new(),
    };
    assert_ne!(record.new_instance_table, root.instance_table);
    assert_ne!(record.new_allocation_record_tree_root, root.allocation_record_tree_root);
    assert!(record.new_unmount_marker && !root.unmount_marker);
    (root, record)
}

fn assert_the_seven_fields_come_from_the_record(rebuilt: &RootRecord, root: &RootRecord, record: &JournalRecord) {
    assert_eq!(rebuilt.fsid, root.fsid, "只有 fsid 取所选根");
    assert_eq!(rebuilt.instance, record.instance);
    assert_eq!(rebuilt.checkpoint_txg, record.checkpoint_txg);
    assert_eq!(rebuilt.tree_table, record.new_tree_table);
    assert_eq!(rebuilt.mapping_root, record.new_mapping_root);
    assert_eq!(rebuilt.tree_identifier_watermark, record.new_tree_identifier_watermark);
    assert_eq!(rebuilt.rollback_floor, record.new_rollback_floor);
    assert_eq!(rebuilt.instance_table, record.new_instance_table, "实例表指针从记录新根段来，不照抄所选根（D23 已定项 15）");
    assert_eq!(rebuilt.unmount_marker, record.new_unmount_marker, "卸载记号从记录新根段来，不照抄所选根");
    assert_eq!(rebuilt.allocation_record_tree_root, record.new_allocation_record_tree_root, "分配记录树根指针从记录新根段来，不照抄所选根");
}

/// 施加第一处（读者甲，`replay_journal`）：M214（实例表指针）、M215（卸载记号）、M216（分配记录树根指针）各自照抄所选根都会红。
#[test]
fn primary_reader_applies_all_seven_new_root_segment_fields_from_the_record() {
    let (root, record) = synthetic_apply_case();
    let records: BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord> = [((record.instance, record.counter), record.clone())].into_iter().collect();
    let empty_pool = Pool { devices: vec![SparseDevice::default(); 2], device_bytes: DEVICE_BYTES };
    let (report, rebuilt) = replay_journal(&empty_pool, &root, &records);
    assert_eq!(report.prefix_applied, 1, "不点名任何单元的记录，验证恒过、施加一条");
    assert_the_seven_fields_come_from_the_record(&rebuilt, &root, &record);
}

/// 施加第二处（读者乙，`replay_journal_clause`，按一次发布成批施加）：M217（三项一起照抄所选根）会红。
#[test]
fn clause_reader_applies_all_seven_new_root_segment_fields_from_the_record() {
    let (root, record) = synthetic_apply_case();
    let records: BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord> = [((record.instance, record.counter), record.clone())].into_iter().collect();
    let empty_pool = Pool { devices: vec![SparseDevice::default(); 2], device_bytes: DEVICE_BYTES };
    let (report, rebuilt, _, fatal) = replay_journal_clause(&empty_pool, &root, &records);
    assert!(!fatal);
    assert_eq!(report.prefix_applied, 1);
    assert_the_seven_fields_come_from_the_record(&rebuilt, &root, &record);
}
