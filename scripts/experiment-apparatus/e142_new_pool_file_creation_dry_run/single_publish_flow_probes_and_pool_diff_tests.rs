use e7_index_bench::usize_from_u64_in_const;
use super::*;

/// E142 第十五次跑（第七节 B10、B11）：一盘时分配记录树仍是叶 + 层级 1 + 根三个节点（B13「one_device
/// new_pool_file_creation_allocation_nodes=3」），加已有的 7 个单元 = 10 个单元、13 次写、2^13 = 8192 个状态。
/// 这个装置的读者总是递归到叶收全部记录（`read_allocation_tree`），对应 B11 的
/// 「读者整棵读分配记录树时」那一档：4096 − 4 = 4092。
#[test]
fn positive_control_without_barriers_has_4092_violations_out_of_8192() {
    let control = PoolParameters::control_one_device_no_barriers();
    let (mut recording, genesis) = mkfs(&control);
    let mkfs_operation_count = recording.operations.len();
    let _ = publish_first_file(&mut recording, &control, &genesis, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    let (base, _) = mkfs(&control);
    let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
    assert_eq!(writes.len(), 13, "第七节 B10「control_writes=13」");
    let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file(), ReaderMode::Clause);
    assert_eq!(tally.states, 8192, "第七节 B10「control_states=8192」");
    assert_eq!(tally.violations, 4092, "第七节 B11「violations_reader_reads_whole_allocation_tree=4092」");
    assert_eq!(tally.root_persisted_states, 4096);
}

// ═════════ E142 第十七次跑（重跑登记 research/prompts/e142-r17-prereg.md）：问题单第 6、8 行 ═════════

/// 第七节 B16：阳性对照的三档期望违例数都从写清单现算——单元数 10、总写数 13、分配记录树根以下 2 个。
/// M149 的会红检查：把 `control_expected_jia` 的依赖计数换成乙档（10 − 2 = 8）会把这一格从 4092 改成
/// 4080，与 R10 主读法（依赖集甲）对不上。
#[test]
fn control_expected_violations_three_readings_match_the_written_out_clause() {
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), false);
    assert_eq!(control_flow.writes.len(), 13, "第七节 B16「control writes=13」");
    assert_eq!(control_flow.dependency_groups.len(), 10, "第七节 B16「unit_writes=10」");
    assert_eq!(control_flow.below_root_count, 2, "第七节 B16「allocation_nodes_below_root=2」");
    assert_eq!(expected_violations_formula(13, 10), 4092, "第七节 B16「expected_violations jia=4092」");
    assert_eq!(expected_violations_formula(13, 10 - 2), 4080, "第七节 B16「expected_violations yi=4080」");
    assert_eq!(expected_violations_formula(13, 10 - 1), 4088, "第七节 B16「expected_violations bing=4088」");
    // M149 的会红检查：这条走 `main()` 真调的那个函数，不是重算一遍公式——甲档换成乙档的依赖计数
    // 会让这里从 4092 变 4080。
    assert_eq!(control_expected_violations_three_readings(&control_flow), (4092, 4080, 4088));
}

/// PC4：逐态预言器自己的闭式——原登记那一档（11 次写、8 个单元写）预言 2048 个状态、1020 个违例
/// （原登记第 20/41 行、附录判据 5），与这一次一盘 13 次写那一档（4092，上一条测试）不是同一个数，
/// 不许混着核。
#[test]
fn original_era_positive_control_predicts_2048_states_1020_violations() {
    assert_eq!(1u64 << 11, 2048, "原登记判据 5：states = 2^11");
    assert_eq!(expected_violations_formula(11, 8), 1020, "原登记判据 5：violations = 2^10 − 2^2 = 1020");
}

/// PC1／Q142.23：逐态预言器（依赖集甲）对阳性对照 8192 个状态逐一比较，checker 与预言器必须
/// 逐态一致——`missed = 0`（对照本身是好的）、`false_alarm = 0`（期望值没有算多）。
/// M150（预言器依赖集漏掉数据单元）、M151（依赖集退回乙档）、M155（取错写的种类）都会让这条变红。
#[test]
fn control_state_comparison_predictor_matches_checker_on_every_state() {
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), false);
    let (tally, comparison) = compare_layer0_predictions(
        &control_flow.base, &control_flow.writes, &control_flow.segments, &sample_file(), ReaderMode::Clause,
        control_flow.journal_index, &control_flow.dependency_groups, &control_flow.role_names,
    );
    assert_eq!(comparison.states, 8192);
    assert_eq!(tally.violations, 4092);
    assert_eq!(comparison.predicted_violations, 4092, "预言器自己数出来的违例数");
    assert_eq!(comparison.agree, 8192, "{:?} / {:?}", comparison.first_missed, comparison.first_false_alarm);
    assert_eq!(comparison.missed, 0, "对照本身坏了：{:?}", comparison.first_missed);
    assert_eq!(comparison.false_alarm, 0, "期望值算多了：{:?}", comparison.first_false_alarm);
}

/// Q142.24：阳性对照结果类矩阵——这个装置没有暖机、`journal` 的记录属于实例 1 而所选（旧）根停在
/// mkfs 的实例 0，重放「按严格前缀停在实例边界」（D23（journal 的角色与格式） 已定项 14 注 1）不跨实例
/// 施加记录：根没持久、journal 与全部单元都持久的 2 个状态，预言器按「记录能施加」预言
/// `legal_file_read`，checker 真的走 `recover` 判出 `no_file`——这不是违例（两边都同意「合法」，
/// 只是合法的哪一种），`off_diagonal = 2` 是这次新发现的正当结果（F13），不是这份自检的漏洞。
#[test]
fn control_outcome_matrix_off_diagonal_is_exactly_the_known_instance_boundary_gap() {
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), false);
    let (_, comparison) = compare_layer0_predictions(
        &control_flow.base, &control_flow.writes, &control_flow.segments, &sample_file(), ReaderMode::Clause,
        control_flow.journal_index, &control_flow.dependency_groups, &control_flow.role_names,
    );
    assert_eq!(comparison.off_diagonal, 2, "F13：一盘上、根没持久时，记录跨不过实例边界，2 个状态从「预言读得到文件」变成「实测读不到」");
    assert_eq!(
        comparison.outcome_matrix.get(&(PredictedOutcomeClass::LegalFileRead, ActualOutcomeClass::NoFile)).copied().unwrap_or(0),
        2,
        "唯一的越对角格：预言 legal_file_read、实测 no_file"
    );
}

/// 第八节 A17/B17：G6（一盘、把 D16（发布语义） 已定项 7 的两道屏障放回去、FUA 不切段）——
/// 屏障隔开了根槽与它前面的写，根槽照样自成一段：1028 个状态、0 个违例（方向与阳性对照相反）。
#[test]
fn barriers_restored_matches_the_registered_1028_states_and_zero_violations() {
    let barriers_restored_flow = build_single_publish_flow(&PoolParameters::one_device_with_barriers(), &sample_file(), false);
    assert_eq!(barriers_restored_flow.segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![10, 1, 2], "第七节 A17「段 [U],[1],[2]」");
    assert_eq!(barriers_restored_flow.closed_form, 1028, "第七节 A17「1028 个状态」");
    let (tally, comparison) = compare_layer0_predictions(
        &barriers_restored_flow.base, &barriers_restored_flow.writes, &barriers_restored_flow.segments, &sample_file(), ReaderMode::Clause, barriers_restored_flow.journal_index, &barriers_restored_flow.dependency_groups, &barriers_restored_flow.role_names,
    );
    assert_eq!(tally.states, 1028);
    assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
    assert_eq!(comparison.missed, 0);
    assert_eq!(comparison.false_alarm, 0);
}

/// 第八节 G7（停机 S10，见第十二节修订）：阳性对照的同一条写流只把 FUA 当段边界——这个装置的
/// `split_into_segments` 与 `crates/singlefs-verification/src/segments.rs`（doc comment「FUA 写关掉
/// 自己所在的那一段」）一致地把 FUA 写并进它前面没被屏障断开的写，不是登记 A18 假设的「FUA 恒自成
/// 一段」；这条流没有屏障，于是 [单元×10, journal] 与根槽合并成一段 [12]，状态数 4097、违例 2046，
/// 不是 A18 写的 2050 / 0。预言器与 checker 在这一点上逐态一致（`missed = false_alarm = 0`）——
/// 分歧只在「这个装置的切法是不是 D13（验证路线） 已定项 4 的字面」，不在依赖集读法上。
/// M154 把切法换成「FUA 自成一段」会让这条测试变红（4097→2050、2046→0），用来证「这条判别力来自
/// 切法，不是巧合」。
#[test]
fn fua_boundary_merges_with_preceding_unbarriered_writes_instead_of_the_registered_2050() {
    let fua_boundary_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), true);
    assert_eq!(fua_boundary_flow.segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![12, 1], "实测切法：单元+journal+根槽合并成一段，系统配置槽单独一段");
    assert_eq!(fua_boundary_flow.closed_form, 4097, "1 + (2^12 − 1) + (2^1 − 1)");
    let (tally, comparison) = compare_layer0_predictions(
        &fua_boundary_flow.base, &fua_boundary_flow.writes, &fua_boundary_flow.segments, &sample_file(), ReaderMode::Clause, fua_boundary_flow.journal_index, &fua_boundary_flow.dependency_groups, &fua_boundary_flow.role_names,
    );
    assert_eq!(tally.states, 4097);
    assert_eq!(tally.violations, 2046, "根被并进那个 12 项段之后，根持久而依赖集不全的子集都判违例：(2^12)/2 − 2 = 2046");
    assert_eq!(comparison.missed, 0);
    assert_eq!(comparison.false_alarm, 0, "预言器与 checker 在这个（有争议的）切法上仍逐态一致");
}

/// PC2（第五节 5.2、第七节 B22）：主臂几何上，全部持久合法，逐个单元的两份都不持久必被判违例——
/// 12 个单元、`caught = 12`。
#[test]
fn pc2_main_geometry_catches_every_single_unit_dependency() {
    let BuiltPool { recording, output, mkfs_operation_count, .. } = built_pool();
    // PC2 要「mkfs 之后、事务写全没持久那一态」当基线（同 main() 里的 `base_after_mkfs`）：
    // 同一份 `PoolParameters` 单独重新 mkfs 一次，取它的池，不复用 `built_pool()` 已经写完事务的池。
    let mkfs_pool = mkfs(&PoolParameters::settled_two_devices()).0.pool;
    let (writes, _) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    let pc2 = run_pc2(&mkfs_pool, &writes, &output.units_by_slot, &sample_file());
    assert_eq!(pc2.per_unit.len(), 12, "第七节 B22「pc2 units=12」");
    assert_eq!(pc2.caught, 12, "第七节 B22「caught=12」");
    assert!(pc2.all_persisted_legal && pc2.all_persisted_content_ok, "全部持久时判合法、读回内容对");
    for (label, _, caught, _) in &pc2.per_unit {
        assert!(*caught, "单元 {label} 两份都缺时该判违例");
    }
}

/// 第八节判别力自证第 1 条：把第 6 行的判定门槛挪到阳性对照（4092）与 G6（0）之间——
/// 例如「违例数 ≥ 1 才算对照有效」——G6 必须判「无效」、阳性对照必须判「有效」，
/// 证明这个门槛不是摆设。
#[test]
fn judgment_threshold_between_control_and_barriers_restored_discriminates_both_ways() {
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), false);
    let control_tally = enumerate_layer0(&control_flow.base, &control_flow.writes, &control_flow.segments, &sample_file(), ReaderMode::Clause);
    let barriers_restored_flow = build_single_publish_flow(&PoolParameters::one_device_with_barriers(), &sample_file(), false);
    let barriers_restored_tally = enumerate_layer0(&barriers_restored_flow.base, &barriers_restored_flow.writes, &barriers_restored_flow.segments, &sample_file(), ReaderMode::Clause);
    let threshold_says_effective = |violations: u64| violations >= 1;
    assert!(threshold_says_effective(control_tally.violations), "阳性对照必须判「有效」");
    assert!(!threshold_says_effective(barriers_restored_tally.violations), "G6 必须判「无效」");
}

#[test]
fn eight_byte_flip_probes_recover_as_the_e142_product_records() {
    let BuiltPool { recording, .. } = built_pool();
    let parameters = PoolParameters::settled_two_devices();
    let mut by_name = BTreeMap::new();
    for probe in probes(&parameters) {
        by_name.insert(probe.name, run_probe(&recording.pool, &probe));
    }
    // 最新根槽坏了 ⇒ 择回暖机第 2 代根，再施加 jsn 3 那条记录重建那次发布的根（D23 已定项 15）⇒ 照样读得到文件。
    assert!(matches!(by_name["newest_root_slot_one_byte"].outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(2)), .. }));
    assert_eq!(by_name["newest_root_slot_one_byte"].journal.prefix_applied, 1);
    assert!(matches!(by_name["journal_record_both_copies"].outcome, RecoveryOutcome::FileRead { .. }));
    assert_eq!(by_name["journal_record_both_copies"].journal.valid_records, 2, "事务记录两份都坏，暖机的两条空记录还在");
    assert_eq!(by_name["journal_record_one_copy"].journal.valid_records, 3);
    assert!(matches!(by_name["data_payload_one_copy"].outcome, RecoveryOutcome::FileRead { .. }));
    assert!(matches!(by_name["data_payload_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
    assert_eq!(by_name["data_payload_both_copies"].mapping_fallbacks, 1);
    assert!(matches!(by_name["data_header_last_byte_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
    assert!(matches!(by_name["tree_table_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
    assert!(matches!(by_name["system_configuration_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
}

/// 第八节判别力自证（M90）：主取样点 w1、G1 自己那条记录都没有 [91,95) 反向链差异段——两条都是本实例第一条，
/// 反向链两臂恒 0（已定项 10）。挪成「含」两条都必须由绿转红，否则这两格没有判别力（作废 V3）。
#[test]
fn journal_header_geometry_back_chain_absence_has_discriminating_power() {
    let flag_set_parameters = PoolParameters::settled_two_devices();
    let flag_cleared_parameters = flag_set_parameters.clone().with_last_of_publish_flag(0x00);
    let (flag_cleared_image, _) = run_full_pipeline(&flag_cleared_parameters, NEW_POOL_FILE_CREATION_TXG, &sample_file());
    let (flag_set_image, flag_set_output) = run_full_pipeline(&flag_set_parameters, NEW_POOL_FILE_CREATION_TXG, &sample_file());
    let main_journal_label = format!("t{}", flag_set_output.units_by_slot.len() + 1);
    let main_differences = diff_pools(&flag_cleared_image, &flag_set_image);
    let has_back_chain = |differences: &[DiffSegment], record: &str, label: &str| {
        differences.iter().any(|segment| {
            journal_record_name_for_offset(segment.offset, label).is_some_and(|(name, offset_in_record)| name == record && journal_header_field_tag(offset_in_record) == "back_chain")
        })
    };

    let unwarmed_single_device_flag_cleared_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
    let unwarmed_single_device_flag_set_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
    let (mut unwarmed_single_device_flag_cleared_recording, unwarmed_single_device_flag_cleared_genesis) = mkfs(&unwarmed_single_device_flag_cleared_parameters);
    let _ = publish_first_file(&mut unwarmed_single_device_flag_cleared_recording, &unwarmed_single_device_flag_cleared_parameters, &unwarmed_single_device_flag_cleared_genesis, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    let (mut unwarmed_single_device_flag_set_recording, unwarmed_single_device_flag_set_genesis) = mkfs(&unwarmed_single_device_flag_set_parameters);
    let unwarmed_single_device_flag_set_output = publish_first_file(&mut unwarmed_single_device_flag_set_recording, &unwarmed_single_device_flag_set_parameters, &unwarmed_single_device_flag_set_genesis, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    let unwarmed_single_device_journal_label = format!("c{}", unwarmed_single_device_flag_set_output.units_by_slot.len() + 1);
    let unwarmed_single_device_differences = diff_pools(&unwarmed_single_device_flag_cleared_recording.pool, &unwarmed_single_device_flag_set_recording.pool);

    assert!(!has_back_chain(&main_differences, "w1", &main_journal_label), "主取样点：w1 是本实例第一条，反向链两臂都是 0");
    assert!(!has_back_chain(&unwarmed_single_device_differences, &unwarmed_single_device_journal_label, &unwarmed_single_device_journal_label), "G1：本实例第一条就是它自己那条记录，反向链两臂都是 0");
}

#[test]
fn diff_pools_reports_a_single_byte_segment_when_only_one_byte_changes() {
    let mut old_sector = [0u8; usize_from_u64_in_const(PHYSICAL_BLOCK_BYTES)];
    old_sector[10] = 1;
    let mut new_sector = old_sector;
    new_sector[10] = 3;
    let old_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(5u64, old_sector)]) }], device_bytes: DEVICE_BYTES };
    let new_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(5u64, new_sector)]) }], device_bytes: DEVICE_BYTES };
    let differences = diff_pools(&old_pool, &new_pool);
    assert_eq!(differences.len(), 1);
    assert_eq!(differences[0].device, 0);
    assert_eq!(differences[0].offset, 5 * PHYSICAL_BLOCK_BYTES + 10);
    assert_eq!(differences[0].length, 1);
    assert_eq!(differences[0].old, vec![1]);
    assert_eq!(differences[0].new, vec![3]);
}

#[test]
fn diff_pools_merges_adjacent_differing_bytes_and_keeps_separated_ones_apart() {
    let mut old_sector = [0u8; usize_from_u64_in_const(PHYSICAL_BLOCK_BYTES)];
    let mut new_sector = old_sector;
    new_sector[20] = 9;
    new_sector[21] = 9; // 相邻两字节都变 ⇒ 合并成一段，长度 2
    new_sector[100] = 9; // 隔得远的一字节 ⇒ 单独一段
    old_sector[300] = 5;
    new_sector[300] = 5; // 这个字节没变，不该出现在任何段里
    let old_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(0u64, old_sector)]) }], device_bytes: DEVICE_BYTES };
    let new_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(0u64, new_sector)]) }], device_bytes: DEVICE_BYTES };
    let mut differences = diff_pools(&old_pool, &new_pool);
    differences.sort_by_key(|segment| segment.offset);
    assert_eq!(differences.len(), 2, "20-21 合并成一段，100 单独一段");
    assert_eq!((differences[0].offset, differences[0].length), (20, 2));
    assert_eq!((differences[1].offset, differences[1].length), (100, 1));
}

#[test]
fn diff_pools_detects_a_flipped_byte_at_the_exact_injected_position() {
    let BuiltPool { recording, .. } = built_pool();
    let base = recording.pool.clone();
    let mut mutated = base.clone();
    let leaf_base = SlotNumber(SLOT_INODE_LEAF).device_offset();
    flip_byte(&mut mutated, DeviceIdentity(0), leaf_base, 88);
    let differences = diff_pools(&base, &mutated);
    assert_eq!(differences.len(), 1, "阳性对照：注入一个字节只能测出一段");
    assert_eq!(differences[0].device, 0);
    assert_eq!(differences[0].offset, leaf_base.0 + 88);
    assert_eq!(differences[0].length, 1);
}
