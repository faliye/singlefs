//! 读码核对与锚点核对、微型历史输出行，以及微型历史那一路的运行。

use super::*;

/// 停机条款（登记第十一节）的格式常量那一半：装置里的常量在 `crates/` 源码原文里逐字找得到。
/// 拆成「模块根加同名目录」的一族源码，每份文件的原文各占一项，找的时候整族一起找。
const CRATES_FORMAT_SOURCE: &str = include_str!("../../../crates/singlefs-format/src/lib.rs");
const CRATES_ALLOCATOR_SOURCES: [&str; 10] = [
    include_str!("../../../crates/singlefs-core/src/allocator.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/allocation_record.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/allocation_record/tests.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/device_free_map.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/device_free_map/tests.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/pool_allocator.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/pool_allocator/tests.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/pools_built_for_tests.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/root_ring_occupancy.rs"),
    include_str!("../../../crates/singlefs-core/src/allocator/root_ring_occupancy/tests.rs"),
];
const CRATES_CHECKER_WALK_SOURCES: [&str; 44] = [
    include_str!("../../../crates/singlefs-checker/src/walk.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/allocation_record_and_tree_birth_judgements.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/allocation_record_and_tree_birth_judgements/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/capacity_sum_and_committed_reservation.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/capacity_sum_and_committed_reservation/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/central_mapping_against_allocation_records.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/deletion_state.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/deletion_state/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/extent_and_allocation_record_tree_traversal.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/inode_and_container_tree_traversal.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/instance_first_txgs_and_high_water.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/instance_first_txgs_and_high_water/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/instance_table.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/journal_publishes_and_replay.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/journal_publishes_and_replay/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/journal_ring_scan.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/journal_ring_scan/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/logged_operation_intents_and_fragmentation.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/logged_operation_intents_and_fragmentation/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/pool_image_judgement.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/published_predicate_and_total_order.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/published_predicate_and_total_order/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/reference_interval_appearance_rule.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/reference_interval_appearance_rule/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/referenced_unit_identity.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/referenced_unit_identity/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/references_without_judging.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/references_without_judging/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/rollback_floor_raises.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/rollback_floor_raises/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/scan_rebuild_comparison.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/snapshots_and_deadlists.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/snapshots_and_deadlists/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/space_accounting_of_the_newest_root.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/space_accounting_of_the_newest_root/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/synthetic_images_for_tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tombstone_lineage_and_retirement_judgements.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tombstone_lineage_and_retirement_judgements/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tombstone_lineage_and_retirement_scan.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tree_table_and_central_mapping_traversal.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tree_table_and_central_mapping_traversal/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tree_table_entry_judgements.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/tree_table_entry_judgements/tests.rs"),
    include_str!("../../../crates/singlefs-checker/src/walk/unit_header_scan_and_instance_carriers.rs"),
];
const CRATES_GPU_JUDGE_SOURCES: [&str; 3] = [
    include_str!("../../../crates/singlefs-verification/src/crash_judge_common.wgsl"),
    include_str!("../../../crates/singlefs-verification/src/crash_judge_common_unit_parsing_mapping_and_index_trees.wgsl"),
    include_str!("../../../crates/singlefs-verification/src/crash_judge_common_extent_tree_instance_table_and_first_file_walk.wgsl"),
];

/// 一族源码里恰好一行（去掉首尾空白之后）被 `is_the_line` 认下，才算逐字找到。
fn written_exactly_once_in(sources: &[&str], is_the_line: impl Fn(&str) -> bool) -> bool {
    sources.iter().flat_map(|source| source.lines()).filter(|source_line| is_the_line(source_line.trim())).count() == 1
}

pub(super) fn crates_source_checks() -> Vec<(&'static str, &'static str, String, bool)> {
    let format_lines = [
        format!("pub const ALLOCATION_RECORD_BYTES: u64 = {ALLOCATION_RECORD_BYTES};"),
        format!("pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = {ALLOCATION_RECORD_TREE_LEAF_SLOTS};"),
        format!("pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = {ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES};"),
        format!("pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = {ALLOCATION_RECORD_TREE_INTERNAL_FANOUT};"),
        format!("pub const ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES: u64 = {ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES};"),
        format!("pub const CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES: u64 = {MAPPING_NODE_HEADER_BYTES};"),
        format!("pub const MAPPING_ENTRY_BYTES: u64 = {MAPPING_ENTRY_BYTES};"),
        format!("pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = {MAPPING_INTERNAL_ENTRY_BYTES};"),
        format!("pub const JOURNAL_RECORD_BYTES: u64 = {JOURNAL_RECORD_BYTES};"),
        format!("pub const JOURNAL_HEADER_BYTES: u64 = {JOURNAL_HEADER_BYTES};"),
        format!("pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = {JOURNAL_NAMED_ENTRY_BYTES};"),
        format!("pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = {JOURNAL_NAMED_ENTRIES_PER_RECORD};"),
        format!("pub const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = {SYSTEM_CONFIGURATION_SLOT_BYTES};"),
        format!("pub const UNIT_AREA_START_SLOT: u64 = {UNIT_AREA_START_SLOT};"),
        format!("pub const CLUSTER_SEGMENT_SLOTS: u64 = {CLUSTER_SEGMENT_SLOTS};"),
        format!("pub const TREE_TABLE_ENTRY_BYTES: u64 = {TREE_TABLE_ENTRY_BYTES};"),
        format!("pub const TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES: u64 = {TREE_TABLE_HEADER_BYTES};"),
        format!("pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = {NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES};"),
        format!("pub const EXTENT_TREE_INDEX_NODE_HEADER_BYTES: u64 = {EXTENT_NODE_HEADER_BYTES};"),
        format!("pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = {EXTENT_INTERNAL_ENTRY_BYTES};"),
        format!("pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = {EXTENT_LEAF_ENTRIES};"),
        format!("pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = {EXTENT_INTERNAL_ENTRIES};"),
        format!("pub const TREE_IDENTIFIER_EXTENT: u64 = {EXTENT_TREE};"),
        format!("pub const TREE_IDENTIFIER_INODE: u64 = {INODE_TREE};"),
        format!("pub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = {ALLOCATION_RECORD_TREE};"),
        format!("pub const TREE_IDENTIFIER_ACCOUNTING: u64 = {ACCOUNTING_TREE};"),
        format!("pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = {QUARANTINE_TREE};"),
    ];
    let mut checks: Vec<(&'static str, &'static str, String, bool)> = format_lines
        .into_iter()
        .map(|line| {
            let found = CRATES_FORMAT_SOURCE.lines().any(|source_line| source_line.trim() == line);
            ("format_constant", "crates/singlefs-format/src/lib.rs", line, found)
        })
        .collect();
    let released_flag_line = format!("const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x{ALLOCATION_RECORD_RELEASED_FLAG:04X};");
    checks.push((
        "released_flag",
        "crates/singlefs-core/src/allocator.rs",
        format!("pub {released_flag_line}"),
        written_exactly_once_in(&CRATES_ALLOCATOR_SOURCES, |source_line| source_line == format!("pub {released_flag_line}")),
    ));
    checks.push((
        "released_flag",
        "crates/singlefs-checker/src/walk.rs",
        released_flag_line.clone(),
        // 走读判定在子模块里，这条常量带 `pub(super)` 给兄弟模块用，其余一字不差。
        written_exactly_once_in(&CRATES_CHECKER_WALK_SOURCES, |source_line| {
            source_line == released_flag_line || source_line.strip_prefix("pub(super) ") == Some(released_flag_line.as_str())
        }),
    ));
    let gpu_span_line = format!("let span = span_field & 0x{:04X}u;", !(ALLOCATION_RECORD_RELEASED_FLAG | QUARANTINE_MARK_BIT));
    checks.push((
        "gpu_span_mask",
        "crates/singlefs-verification/src/crash_judge_common.wgsl",
        gpu_span_line.clone(),
        written_exactly_once_in(&CRATES_GPU_JUDGE_SOURCES, |source_line| source_line == gpu_span_line),
    ));
    checks
}

/// 停机条款的读码那一半（登记第十一节逐行）：执行员读工作区的 `crates/` 写下的事实，行号是现取的。
/// 模型照这些写；任一条对不上就停机交回，不跑产物。
const CODE_READING_STOP_CHECKS: [(&str, &str, &str); 13] = [
    ("quarantine_leaves_both_records_untouched", "crates/singlefs-core/src/allocator.rs:2109 release_leaving_the_record_allocated_on; crates/singlefs-core/src/transaction.rs:12937; transaction.rs:4876 copies_failing_the_release_checksum_check", "devices_whose_record_stays_allocated_are_skipped_record_unchanged_both_copies_returned_when_either_fails_mapping_entry_dropped"),
    ("arms_share_accounting_mapping_and_tree_table_changes", "crates/singlefs-core/src/transaction.rs:8230; transaction.rs:12928-12948", "accounting_tree_rewritten_every_publish_quarantined_copies_stay_allocated_in_the_same_allocator"),
    ("allocation_record_tree_by_position", "crates/singlefs-core/src/allocation_record_tree.rs:1-15 273 365; crates/singlefs-format/src/lib.rs:163 169", "leaf_812_fanout_169_root_level_formula_changed_leaf_plus_all_ancestors_and_root_empty_leaves_not_written"),
    ("allocation_record_tree_enters_the_mapping", "crates/singlefs-core/src/transaction.rs:3226", "data_extent_inode_allocation_and_accounting_nodes_mapped_mapping_tree_and_tree_table_exempt"),
    ("mapping_key_order", "crates/singlefs-core/src/records.rs:488 mapping_key_sort_key", "class_birth_tree_birth_txg_instance_tail"),
    ("code_two_tree_rules", "crates/singlefs-core/src/code_two_tree.rs:1-17 637 plan_the_tree_after_this_publish; crates/singlefs-core/src/unit.rs:155", "deletes_then_inserts_in_key_order_split_in_the_middle_left_ceiling_half_only_empty_nodes_removed_root_with_one_child_collapses_capacity_node_minus_header_over_entry"),
    ("journal_named_roles", "crates/singlefs-core/src/transaction.rs:11578 roles_named_by_each_record_of_the_publish; crates/singlefs-format/src/lib.rs:253", "one_record_per_data_unit_but_the_last_shared_roles_named_in_the_last_transaction_64_per_record"),
    ("placements", "crates/singlefs-core/src/allocator.rs:2193 2245 2327 933 1737", "data_lowest_two_free_slots_even_outside_every_cluster_segment_opened_in_this_mount_commit_generated_bumped_in_the_open_segment_new_segment_lowest_with_no_occupied_slot"),
    ("reclaim_floor_and_defer", "crates/singlefs-core/src/allocator.rs:2374 reclaim_released_up_to", "released_records_with_release_generation_up_to_the_floor_return_to_free_record_kept_until_reallocated"),
    ("rollback_revives_only_released_records", "crates/singlefs-core/src/mount.rs:9237 roll_back_by_a_forward_publish", "current_only_units_released_target_units_released_in_the_current_ledger_resurrected_extent_and_inode_trees_taken_from_the_target"),
    ("writes_per_device", "crates/singlefs-core/src/transaction.rs:1439 persist_publish_writes 1593", "unit_to_every_device_journal_record_to_every_device_root_slot_to_one_region_device_system_configuration_slot_to_every_device"),
    ("checker_isolation_criterion", "crates/singlefs-checker/src/walk.rs:16223 some_copy_of_the_unit_fails_its_location_entries", "exempt_when_some_copy_is_unreadable_or_fails_its_location_entry_checksum_falls_back_to_self_verification_without_one"),
    ("old_readers", "allocator.rs:80 parse; allocation_record_tree.rs:273 738; recovery.rs:2361 2512; walk.rs:4394 5293 5752 6171; crash_judge_common.wgsl:2566", "see_old_reader_field_table_lines"),
];

/// 落点甲借位之后五处旧读者的读法（登记第六节 `old_reader_field_table`，读码核）：跨度段 0x4001 / 0x4002 时读出的五个字段与接着做什么。
const OLD_READER_FIELD_TABLE: [(&str, &str, &str, &str); 5] = [
    (
        "core_record_parser",
        "crates/singlefs-core/src/allocator.rs:80 AllocationRecord::parse",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written_allocation_generation released=false",
        "returns_the_record_to_its_caller_no_judgement_here",
    ),
    (
        "core_allocation_record_tree_reader",
        "crates/singlefs-core/src/allocation_record_tree.rs:273 record_fits_in_its_leaf 738",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written released=false",
        "refuses_the_tree_with_AllocationRecordOutsideThePoolGeometry_mount_and_recovery_fail",
    ),
    (
        "core_recovery_geometry",
        "crates/singlefs-core/src/recovery.rs:2361 allocation_records_fit_the_pool_geometry 2512",
        "device=as_written slot=as_written span=16385_or_16386",
        "reached_only_after_the_tree_reader_accepted_the_records_so_never_reached_for_these_records_standalone_it_refuses_only_on_an_overlap_or_past_the_unit_area_end",
    ),
    (
        "pool_checker",
        "crates/singlefs-checker/src/walk.rs:5293 parse_allocation_record 4394 6171 5752",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written released=false",
        "judges_I-1.1_red_in_the_judging_walk_and_the_non_judging_reader_keeps_the_record_with_span_16385_in_the_references_used_by_the_accounting_judgements",
    ),
    (
        "gpu_crash_judge",
        "crates/singlefs-verification/src/crash_judge_common.wgsl:2566",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written span_field_kept_raw",
        "stops_the_walk_at_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY",
    ),
];

/// 第七节「出自被测条款本身」的钉值（不符走 `clause_anchor_mismatch`，不作废）。
fn clause_anchor_checks() -> Vec<(&'static str, bool, String)> {
    let first_version_spans_leave_the_mark_bit_clear = FIRST_VERSION_SPANS.iter().all(|span| span & QUARANTINE_MARK_BIT == 0);
    let new_pool = new_pool_file_creation_open_segment();
    vec![
        (
            "allocation_record_bytes",
            ALLOCATION_RECORD_BYTES == ALLOCATION_RECORD_KEY_BYTES + 10 && ALLOCATION_RECORD_SPAN_FIELD_OFFSET == 10,
            format!("record={ALLOCATION_RECORD_BYTES} key={ALLOCATION_RECORD_KEY_BYTES} span_field_offset={ALLOCATION_RECORD_SPAN_FIELD_OFFSET}"),
        ),
        ("released_flag", ALLOCATION_RECORD_RELEASED_FLAG == 0x8000, format!("flag=0x{ALLOCATION_RECORD_RELEASED_FLAG:04X}")),
        ("span_values_of_the_first_version", first_version_spans_leave_the_mark_bit_clear, format!("spans={FIRST_VERSION_SPANS:?} mark_bit=0x{QUARANTINE_MARK_BIT:04X}")),
        ("records_kept_per_quarantine", quarantine_entries_of(0).len() == 2, "records_kept=2".to_string()),
        ("mapping_entry_removed_on_quarantine", mapping_entry_removed_on_quarantine_in_every_arm(), "every_fork_arm_drops_the_victim_key".to_string()),
        ("segment_slots", CLUSTER_SEGMENT_SLOTS == 64 && DATA_UNIT_ALIGNMENT_SLOTS == 2, format!("segment={CLUSTER_SEGMENT_SLOTS} data_unit_alignment_slots={DATA_UNIT_ALIGNMENT_SLOTS}")),
        (
            "new_pool_open_segment",
            new_pool == (50240, 50304, 2),
            format!("open_segment=[{},{}) root_level={}", new_pool.0, new_pool.1, new_pool.2),
        ),
    ]
}

/// 32768 字节对齐 = 两个 16 KiB 槽（D3（空间分配） 已定项 10 ③）。
const DATA_UNIT_ALIGNMENT_SLOTS: u64 = 32768 / SLOT_BYTES;

/// 新池新建文件（一个数据单元）之后模型开的段与分配记录树根层级（D3（空间分配） 已定项 10 ⑤ 那张表的段与 t9）。
pub(super) fn new_pool_file_creation_open_segment() -> (u64, u64, u8) {
    let mut state = PoolState::after_make_filesystem(PoolGeometry::DefaultFourGibibyteDevices, MICRO_CANDIDATE_WINDOW, false);
    publish(&mut state, &PublishRequest::plain(PublishKind::CreateFile { data_unit_count: 1 }), Arm::NoMark);
    let start = state.open_segment_start.expect("新建文件开了一段装提交内生块");
    (start, start + CLUSTER_SEGMENT_SLOTS, state.root_level)
}

/// 三条臂隔离那一次发布之后，被隔离单元那把映射 key 都不在映射里（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
pub(super) fn mapping_entry_removed_on_quarantine_in_every_arm() -> bool {
    FORK_ARMS.iter().all(|arm| {
        let (mut state, request) = prepare_micro_history(MicroHistory::CleanLeaf, *arm);
        let victim = state.version.as_ref().expect("文件建好了").data_units[usize_of(u64::from(MICRO_VICTIM))];
        let victim_key = state.unit(victim).mapping_key.expect("数据单元进映射");
        let had_the_key = tree_contains(&state.mapping, &victim_key);
        publish(&mut state, &request, *arm);
        had_the_key && !tree_contains(&state.mapping, &victim_key) && state.quarantined.contains_key(&victim)
    })
}

/// 第七节「独立算出、用命令核过的」钉值（不符就作废）。
fn independent_anchor_checks() -> Vec<(&'static str, bool, String)> {
    let span_with_high_bit = [old_reader_arithmetic(QUARANTINE_MARK_BIT, 1), old_reader_arithmetic(QUARANTINE_MARK_BIT, 2)];
    let span_with_low_bit = [old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 1), old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 2)];
    let leaf_range = |leaf: u64| (leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS, (leaf + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS);
    vec![
        (
            "allocation_leaf_capacity",
            ALLOCATION_RECORD_TREE_LEAF_ENTRIES == 812 && ALLOCATION_RECORD_TREE_INTERNAL_FANOUT == 169,
            format!("leaf={ALLOCATION_RECORD_TREE_LEAF_ENTRIES} fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT}"),
        ),
        ("mapping_capacities", MAPPING_LEAF_ENTRIES == 294 && MAPPING_INTERNAL_ENTRIES == 143, format!("leaf={MAPPING_LEAF_ENTRIES} internal={MAPPING_INTERNAL_ENTRIES}")),
        ("quarantine_tree_leaf_capacity", QUARANTINE_TREE_LEAF_ENTRIES == 812, format!("leaf={QUARANTINE_TREE_LEAF_ENTRIES}")),
        ("tree_table_unit_capacity", TREE_TABLE_UNIT_ENTRIES == 81, format!("entries={TREE_TABLE_UNIT_ENTRIES}")),
        (
            "extent_capacities",
            EXTENT_LEAF_ENTRIES == 144 && EXTENT_INTERNAL_ENTRIES == 147 && EXTENT_LEAF_ENTRIES * EXTENT_INTERNAL_ENTRIES == 21168,
            format!("leaf={EXTENT_LEAF_ENTRIES} internal={EXTENT_INTERNAL_ENTRIES} two_levels={}", EXTENT_LEAF_ENTRIES * EXTENT_INTERNAL_ENTRIES),
        ),
        ("journal_names_per_record", JOURNAL_NAMED_ENTRIES_PER_RECORD == 64, format!("names={JOURNAL_NAMED_ENTRIES_PER_RECORD}")),
        (
            "root_levels",
            allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()) == 2
                && allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device()) == 3,
            format!(
                "default={} wide={}",
                allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()),
                allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device())
            ),
        ),
        (
            "leaf_ranges",
            leaf_range(62) == (50344, 51156)
                && leaf_range(70) == (56840, 57652)
                && leaf_range(75) == (60900, 61712)
                && leaf_of_slot(140000) == 172
                && span_in_slots_at_level(1) == 137228,
            format!("leaf62={:?} leaf70={:?} leaf75={:?} leaf_of_140000={} level_one_span={}", leaf_range(62), leaf_range(70), leaf_range(75), leaf_of_slot(140000), span_in_slots_at_level(1)),
        ),
        (
            "old_reader_span_with_high_bit",
            span_with_high_bit[0].span_read == 16385 && span_with_high_bit[1].span_read == 16386 && span_with_high_bit.iter().all(|arithmetic| arithmetic.accepted == 0),
            format!("spans_read={}/{} accepted={}/{}", span_with_high_bit[0].span_read, span_with_high_bit[1].span_read, span_with_high_bit[0].accepted, span_with_high_bit[1].accepted),
        ),
        (
            "old_reader_span_with_low_bit",
            span_with_low_bit[0].span_read == 5
                && span_with_low_bit[1].span_read == 6
                && span_with_low_bit[0].legal_starts == 812
                && span_with_low_bit[0].accepted == 808
                && span_with_low_bit[1].legal_starts == 406
                && span_with_low_bit[1].accepted == 404,
            format!(
                "spans_read={}/{} accepted={}_of_{}/{}_of_{}",
                span_with_low_bit[0].span_read,
                span_with_low_bit[1].span_read,
                span_with_low_bit[0].accepted,
                span_with_low_bit[0].legal_starts,
                span_with_low_bit[1].accepted,
                span_with_low_bit[1].legal_starts
            ),
        ),
        (
            "logical_bytes_per_quarantine",
            logical_bytes_changed_by_the_span_bit() == 2 && 2 * QUARANTINE_TREE_ENTRY_BYTES == 40 && TREE_TABLE_ENTRY_BYTES == 200,
            format!("span_bit={} quarantine_tree={}+first_time_{}", logical_bytes_changed_by_the_span_bit(), 2 * QUARANTINE_TREE_ENTRY_BYTES, TREE_TABLE_ENTRY_BYTES),
        ),
    ]
}

/// 甲改的逻辑字节：两条记录跨度段小端的高字节由 0x00 变 0x40。
pub(super) fn logical_bytes_changed_by_the_span_bit() -> u64 {
    let record = AllocationRecord { span_slots: 2, is_released: false, generation: 7, carries_the_quarantine_mark: false };
    let marked = AllocationRecord { span_slots: record.span_slots, is_released: record.is_released, generation: record.generation, carries_the_quarantine_mark: true };
    let changed_bytes_per_record = record.span_field().to_le_bytes().iter().zip(marked.span_field().to_le_bytes().iter()).filter(|(before, after)| before != after).count();
    DEVICE_COUNT * u64_of(changed_bytes_per_record)
}

/// 读码核的结论（`walk.rs` 5752、6171 那两处）：池级 checker 判 I-1.1 红的同时，不判的那一遍照收这条记录、按读错的跨度给记账判据用。
const POOL_CHECKER_KEEPS_THE_MISREAD_SPAN_FOR_THE_ACCOUNTING_JUDGEMENTS: bool = true;

fn micro_lines(micro: MicroHistory, measurements: &[MicroArmMeasurement]) -> Vec<String> {
    let no_mark = measurements.iter().find(|measurement| measurement.arm == Arm::NoMark).expect("每段微历史都跑 no_mark");
    measurements
        .iter()
        .map(|measurement| {
            let total = measurement.writes.without_root_and_system_configuration();
            let extra = micro_extra(measurements, measurement.arm);
            format!(
                "name=micro history={} arm={} device_writes={} bytes={} journal_records={} last_transaction_named_roles={} padding_named_roles={} victim_leaf_rewritten={} \
                 extra_device_writes={} extra_bytes={} extra_bytes_single_device={} {} logical_tree_table_bytes_added={} mark_agreement_red_after_revival={}",
                micro.label(),
                measurement.arm.label(),
                total.device_writes,
                total.bytes_on_both_devices,
                measurement.writes.journal_records,
                measurement.writes.last_transaction_named_roles,
                measurement.padding_named_roles,
                measurement.victim_leaf_rewritten,
                extra.device_writes,
                extra.bytes_on_both_devices,
                extra.bytes_on_both_devices / 2,
                structure_differences(&measurement.writes, &no_mark.writes),
                measurement.writes.logical_tree_table_bytes_added,
                measurement.mark_agreement_red_after_revival.map_or("not_a_revival".to_string(), |red| red.to_string())
            )
        })
        .collect()
}

/// 第一段的判定（登记第六节、第十节、第十一节）。
pub(super) struct SegmentOneJudgements {
    pub(super) stop_checks_hold: bool,
    clause_anchors_hold: bool,
    independent_anchors_hold: bool,
    micro_pins_hold: bool,
    positive_controls_hold: bool,
    revival_clears_the_mark: bool,
    old_readers_reject_the_mark_holds: bool,
    mark_hides_a_leak_holds: bool,
    mark_removes_the_false_red_holds: bool,
    span_bit_writes_nothing_extra_on_the_micro_histories_holds: bool,
}

pub(super) fn run_segment_one() -> (Vec<String>, SegmentOneJudgements) {
    let mut lines = Vec::new();
    let mut stop_checks_hold = true;
    for (item, origin, fact) in CODE_READING_STOP_CHECKS {
        lines.push(format!("name=stop_check item={item} kind=code_reading where={} observed={fact} result=pass", origin.replace(' ', "_")));
    }
    for (item, origin, line, found) in crates_source_checks() {
        stop_checks_hold &= found;
        lines.push(format!("name=stop_check item={item} kind=source_text where={origin} expected_line={} result={}", line.replace(' ', "_"), if found { "pass" } else { "fail" }));
    }
    let mut clause_anchors_hold = true;
    for (name, holds, detail) in clause_anchor_checks() {
        clause_anchors_hold &= holds;
        lines.push(format!("name=clause_anchor anchor={name} holds={holds} {}", detail.replace(' ', "_")));
    }
    let mut independent_anchors_hold = true;
    for (name, holds, detail) in independent_anchor_checks() {
        independent_anchors_hold &= holds;
        lines.push(format!("name=independent_anchor anchor={name} holds={holds} {}", detail.replace(' ', "_")));
    }
    // 八段微历史与写量的阳性对照。
    let mut micro_pins_hold = true;
    let mut positive_controls_hold = true;
    let mut revival_clears_the_mark = true;
    let mut span_bit_writes_nothing_extra_on_the_micro_histories_holds = true;
    for micro in MICRO_HISTORIES {
        let measurements = measure_micro_history(micro);
        lines.extend(micro_lines(micro, &measurements));
        let span_bit = micro_extra(&measurements, Arm::SpanBit);
        let quarantine_tree = micro_extra(&measurements, Arm::QuarantineTree);
        let (span_bit_writes, span_bit_bytes, quarantine_tree_writes, quarantine_tree_bytes) = micro.pinned_extra_writes();
        let holds = span_bit == WriteDifference { device_writes: signed(span_bit_writes), bytes_on_both_devices: signed(span_bit_bytes) }
            && quarantine_tree == WriteDifference { device_writes: signed(quarantine_tree_writes), bytes_on_both_devices: signed(quarantine_tree_bytes) };
        micro_pins_hold &= holds;
        span_bit_writes_nothing_extra_on_the_micro_histories_holds &= span_bit.device_writes <= 0;
        for measurement in &measurements {
            revival_clears_the_mark &= measurement.mark_agreement_red_after_revival != Some(true);
        }
        lines.push(format!(
            "name=micro_pin history={} span_bit_extra={}/{} span_bit_expected={span_bit_writes}/{span_bit_bytes} quarantine_tree_extra={}/{} quarantine_tree_expected={quarantine_tree_writes}/{quarantine_tree_bytes} holds={holds}",
            micro.label(),
            span_bit.device_writes,
            span_bit.bytes_on_both_devices,
            quarantine_tree.device_writes,
            quarantine_tree.bytes_on_both_devices
        ));
        if micro == MicroHistory::CleanLeaf {
            let plain = micro_extra(&measurements, Arm::PlainRelease);
            let counted = plain == WriteDifference { device_writes: 4, bytes_on_both_devices: 65536 } && span_bit.device_writes == 4 && quarantine_tree.device_writes == 2;
            positive_controls_hold &= counted;
            lines.push(format!(
                "name=positive_control control=write_counting history={} plain_release_minus_no_mark={}/{} expected=4/65536 span_bit_minus_no_mark={} quarantine_tree_minus_no_mark={} span_bit_more_than_quarantine_tree={} holds={counted}",
                micro.label(),
                plain.device_writes,
                plain.bytes_on_both_devices,
                span_bit.device_writes,
                quarantine_tree.device_writes,
                span_bit.device_writes > quarantine_tree.device_writes
            ));
        }
        if micro == MicroHistory::SecondQuarantineTreeCold {
            lines.push(format!(
                "name=pairwise_with_absolute history={} quarantine_tree_more_than_span_bit={} quarantine_tree={} span_bit={}",
                micro.label(),
                quarantine_tree.device_writes > span_bit.device_writes,
                quarantine_tree.device_writes,
                span_bit.device_writes
            ));
        }
    }
    // checker 的阳性对照：每条臂、每个变体，泄漏判红、正常判绿（两版都判）。
    for arm in FORK_ARMS {
        for checker in checkers_of(arm) {
            for version in JUDGED_VERSIONS {
                let leak = judge_record(*checker, &constructed_view(arm, RecordClass::Leak, MarkState::WrittenCorrectly, version));
                let normal = judge_record(*checker, &constructed_view(arm, RecordClass::Normal, MarkState::WrittenCorrectly, version));
                let holds = leak.is_red() && !normal.is_red();
                positive_controls_hold &= holds;
                lines.push(format!(
                    "name=positive_control control=checker arm={} checker={} version={} leak={} normal={} holds={holds}",
                    arm.label(),
                    checker.label(),
                    version.label(),
                    leak.label(),
                    normal.label()
                ));
            }
        }
    }
    let positive_control_old_reader = !old_reader_accepts(OLD_READER_POSITIVE_CONTROL_SLOT % ALLOCATION_RECORD_TREE_LEAF_SLOTS, old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD));
    positive_controls_hold &= positive_control_old_reader;
    lines.push(format!(
        "name=positive_control control=old_reader span_field=0x{OLD_READER_POSITIVE_CONTROL_SPAN_FIELD:04X} slot={OLD_READER_POSITIVE_CONTROL_SLOT} span_read={} judged_outside_the_leaf={positive_control_old_reader} holds={positive_control_old_reader}",
        old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD)
    ));
    // 位翻转分类表：臂 × 变体 × 记录类 × 记号状态 × 版本；放过与误红按两个宇宙数。
    let mut constructed_counts: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts> = BTreeMap::new();
    let mut mark_removes_the_false_red_holds = true;
    for arm in FORK_ARMS {
        for checker in checkers_of(arm) {
            for class in RECORD_CLASSES {
                for mark_state in MARK_STATES {
                    for version in JUDGED_VERSIONS {
                        let verdict = judge_record(*checker, &constructed_view(arm, class, mark_state, version));
                        let legal = is_a_legal_disk(class, mark_state);
                        let outcome = match (legal, verdict.is_red()) {
                            (true, true) => "false_red",
                            (true, false) => "correct_green",
                            (false, true) => "caught",
                            (false, false) => "passed_defect",
                        };
                        constructed_counts.entry((arm, *checker, universe_of(mark_state))).or_default().count(legal, &verdict, 1);
                        if arm != Arm::NoMark && class == RecordClass::QuarantineIntact && mark_state == MarkState::WrittenCorrectly && verdict.is_red() {
                            mark_removes_the_false_red_holds = false;
                        }
                        lines.push(format!(
                            "name=flip_classification arm={} checker={} class={} mark={} version={} verdict={} isolation_criteria_red={} legal_disk={legal} outcome={outcome}",
                            arm.label(),
                            checker.label(),
                            class.label(),
                            mark_state.label(),
                            version.label(),
                            verdict.label(),
                            verdict.isolation_criteria_red_count()
                        ));
                    }
                }
            }
        }
    }
    lines.push("name=flip_classification_medium arms=no_mark,span_bit,quarantine_tree verdict=I-2.1 note=node_checksum_mismatch_reddens_every_arm_alike_not_counted_in_passes_or_false_reds".to_string());
    let mut mark_hides_a_leak_holds = true;
    for ((arm, checker, universe), counts) in &constructed_counts {
        let baseline = constructed_counts[&(Arm::NoMark, CheckerUnderTest::Today, *universe)];
        let reading = match arm {
            Arm::SpanBit | Arm::QuarantineTree => {
                if counts.passed_defects > baseline.passed_defects {
                    mark_hides_a_leak_holds = false;
                }
                pass_and_false_red_reading(*counts, baseline)
            }
            Arm::NoMark | Arm::PlainRelease => "baseline",
        };
        lines.push(format!(
            "name=constructed_pass_and_false_red arm={} checker={} universe={} passed_defects={} false_reds={} judged_disks={} reading={reading}",
            arm.label(),
            checker.label(),
            UNIVERSE_LABELS[*universe],
            counts.passed_defects,
            counts.false_reds,
            counts.judged_disks
        ));
    }
    // 落点甲的旧读者：算术（装置）与读码（五处）。
    let mut arithmetic_accepts_nothing_with_the_high_bit = true;
    for borrowed_bit in [QUARANTINE_MARK_BIT, LOW_SPAN_BIT_PROBE] {
        for record_span in FIRST_VERSION_SPANS {
            let arithmetic = old_reader_arithmetic(borrowed_bit, record_span);
            if borrowed_bit == QUARANTINE_MARK_BIT {
                arithmetic_accepts_nothing_with_the_high_bit &= arithmetic.accepted == 0;
            }
            lines.push(format!(
                "name=old_reader_arithmetic borrowed_bit=0x{:04X} record_span={} span_field=0x{:04X} span_read={} legal_starts_per_leaf={} accepted={}",
                arithmetic.borrowed_bit,
                arithmetic.record_span,
                arithmetic.record_span | arithmetic.borrowed_bit,
                arithmetic.span_read,
                arithmetic.legal_starts,
                arithmetic.accepted
            ));
        }
    }
    for (reader, origin, fields, action) in OLD_READER_FIELD_TABLE {
        lines.push(format!("name=old_reader_field_table reader={reader} where={} fields={} next={action}", origin.replace(' ', "_"), fields.replace(' ', ",")));
    }
    let old_readers_reject_the_mark_holds = arithmetic_accepts_nothing_with_the_high_bit && !POOL_CHECKER_KEEPS_THE_MISREAD_SPAN_FOR_THE_ACCOUNTING_JUDGEMENTS;
    let judgements = SegmentOneJudgements {
        stop_checks_hold,
        clause_anchors_hold,
        independent_anchors_hold,
        micro_pins_hold,
        positive_controls_hold,
        revival_clears_the_mark,
        old_readers_reject_the_mark_holds,
        mark_hides_a_leak_holds,
        mark_removes_the_false_red_holds,
        span_bit_writes_nothing_extra_on_the_micro_histories_holds,
    };
    lines.push(format!(
        "name=verdict segment=one stop_checks_hold={} clause_anchors_hold={} independent_anchors_hold={} micro_pins_hold={} positive_controls_hold={} \
         revival_clears_the_mark={} old_readers_reject_the_mark_holds={} mark_hides_a_leak_holds={} mark_removes_the_false_red_holds={} \
         span_bit_writes_nothing_extra_on_the_micro_histories_holds={}",
        judgements.stop_checks_hold,
        judgements.clause_anchors_hold,
        judgements.independent_anchors_hold,
        judgements.micro_pins_hold,
        judgements.positive_controls_hold,
        judgements.revival_clears_the_mark,
        judgements.old_readers_reject_the_mark_holds,
        judgements.mark_hides_a_leak_holds,
        judgements.mark_removes_the_false_red_holds,
        judgements.span_bit_writes_nothing_extra_on_the_micro_histories_holds
    ));
    (lines, judgements)
}
