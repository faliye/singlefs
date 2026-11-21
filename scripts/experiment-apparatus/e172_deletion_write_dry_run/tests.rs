use super::*;
use std::sync::OnceLock;

/// 入库的两份导出（`research/results/` 下的文件名）。跑前登记 e172-r2 第九节原写「找不到就判红，不跳过」；
/// `research/` 可以整个删掉，别的实验的产物缺了报本次未跑：读不到时用到它们的单测逐份报一行本次未跑、照常返回。
/// 路径写成字面的 `concat!`（写法与为什么见 `e7_index_bench::ARCHIVED_PRODUCT_NOT_RUN_MARKER`）。
/// 第 3 次重跑登记 `research/prompts/e172-r3-prereg.md` 第五节 5.7 第 1 步的两份导出（主 agent 跑、入库）。
const HARNESS_AND_MOUNT_CONTINUATION_EXPORT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/e172-r3-deletion-write-dump-h1h2-2026-11-02.out");
const RECORD_TWO_KEPT_EXPORT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/e172-r3-deletion-write-dump-h3-2026-11-02.out");

struct ExportFixture {
    export_files: Vec<ExportFile>,
    histories: Vec<HistoryExport>,
    judgements: Vec<HistoryJudgement>,
    lines: Vec<String>,
}

/// 两份导出读一次、解一次；读不到的交（文件名，错误）逐份列出。
fn loaded_exports() -> &'static Result<ExportFixture, Vec<(&'static str, String)>> {
    static FIXTURE: OnceLock<Result<ExportFixture, Vec<(&'static str, String)>>> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let read = |path: &'static str| std::fs::read_to_string(path).map_err(|error| (path, error.to_string()));
        let (first_text, second_text) = match (read(HARNESS_AND_MOUNT_CONTINUATION_EXPORT), read(RECORD_TWO_KEPT_EXPORT)) {
            (Ok(first_text), Ok(second_text)) => (first_text, second_text),
            (first, second) => return Err([first.err(), second.err()].into_iter().flatten().collect()),
        };
        let texts = [first_text.as_str(), second_text.as_str()];
        let export_files = parse_export_files(&texts);
        let histories: Vec<HistoryExport> = export_files.iter().flat_map(|export| export.histories.iter().cloned()).collect();
        let judgements = histories.iter().map(judge_history).collect();
        let lines = report_lines(&texts);
        Ok(ExportFixture { export_files, histories, judgements, lines })
    })
}

/// 用到两份导出的单测开头调：读得到交 true；读不到就替这一条单测逐份报一行本次未跑，交 false，调用方照常返回。
fn exports_present_or_report_not_run(test_name: &str) -> bool {
    match loaded_exports() {
        Ok(_) => true,
        Err(missing) => {
            for (file_name, error) in missing {
                e7_index_bench::report_not_run_line_on_stdout(&e7_index_bench::archived_product_not_run_line(test_name, file_name, error));
            }
            false
        }
    }
}

/// 单测开头一行：两份导出读不到就报本次未跑（用例名取所在函数的路径）、返回。
macro_rules! return_not_run_unless_exports_present {
    () => {{
        fn here() {}
        let here_path = std::any::type_name_of_val(&here);
        if !exports_present_or_report_not_run(here_path.strip_suffix("::here").unwrap_or(here_path)) {
            return;
        }
    }};
}

fn fixture() -> &'static ExportFixture {
    loaded_exports().as_ref().expect("用到导出的单测开头先写 return_not_run_unless_exports_present!()")
}

fn judgement_named(name: &str) -> &'static HistoryJudgement {
    fixture().judgements.iter().find(|judgement| judgement.name == name).expect("三段历史都在")
}

fn history_named(name: &str) -> &'static HistoryExport {
    fixture().histories.iter().find(|history| history.name == name).expect("三段历史都在")
}

fn cell_row(history: &str, txg: u64, cell: &str, field: &str) -> &'static CellRow {
    judgement_named(history)
        .cell_rows
        .iter()
        .find(|row| row.txg == txg && row.cell == cell && row.field == field)
        .unwrap_or_else(|| panic!("{history} txg {txg} 没有 {cell} {field} 这一行"))
}

fn line_starting_with(prefix: &str) -> &'static String {
    fixture().lines.iter().find(|line| line.starts_with(prefix)).unwrap_or_else(|| panic!("产物里没有以 {prefix} 起头的行"))
}

#[test]
fn castagnoli_crc32_of_the_standard_check_string_is_e3069283() {
    assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
}

#[test]
fn sha256_of_abc_matches_the_standard_vector() {
    assert_eq!(sha256_hexadecimal(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn registered_capacities_compute_to_815_2031_and_120() {
    assert_eq!(tombstone_container_capacity(), 815);
    assert_eq!(deleted_inodes_root_capacity(), 2031);
    assert_eq!(tombstone_root_entry_bytes(), 120);
}

/// 跑前登记 e172-r3 第七节 7.1 A-tag6 的四个值（带退役项），与判别臂那一份（去掉退役项，即第 2 次装置的 98304 / 98386）。
#[test]
fn committed_reservation_formula_with_retirement_gives_196678_196608_196830_and_196760() {
    assert_eq!(expected_committed_reservation(0, 0, 0, 1, RetirementShare::Included), 196_678);
    assert_eq!(expected_committed_reservation(0, 0, 0, 0, RetirementShare::Included), 196_608);
    assert_eq!(expected_committed_reservation(0, 0, 1, 2, RetirementShare::Included), 196_830);
    assert_eq!(expected_committed_reservation(0, 0, 1, 1, RetirementShare::Included), 196_760);
    assert_eq!(expected_committed_reservation(0, 0, 0, 1, RetirementShare::Removed), 98_304);
    assert_eq!(expected_committed_reservation(0, 0, 1, 2, RetirementShare::Removed), 98_386);
}

#[test]
fn expected_file_bytes_are_two_full_payloads_and_a_half() {
    assert_eq!(EXPECTED_FILE_BYTES, 81_585);
    assert_eq!(DATA_UNIT_PAYLOAD_CAPACITY, 32_634);
}

#[test]
fn root_slot_journal_and_system_configuration_anchors_follow_the_layout_formulas() {
    assert_eq!(expected_root_slot(5), (0, 7_344_128));
    assert_eq!(expected_root_slot(6), (0, 1_056_768));
    assert_eq!(expected_root_slot(7), (1, 4_202_496));
    assert_eq!(expected_root_slot(13), (1, 4_210_688));
    assert_eq!(expected_journal_offset_assuming_counter_equals_txg(5), 16_793_600);
    assert_eq!(expected_journal_offset_assuming_counter_equals_txg(8), 16_805_888);
    assert_eq!(expected_system_configuration_for_txg_in_the_first_instance(5), (8, 0));
    assert_eq!(expected_system_configuration_for_txg_in_the_first_instance(6), (9, 4096));
}

#[test]
fn layout_field_tables_tile_their_registered_totals() {
    let root_record_widths = [4usize, 16, 4, 4, 8, 86, 8, 8, 32, 86, 86, 86, 8, 1, 1, 12, 16];
    assert_eq!(root_record_widths.iter().sum::<usize>(), ROOT_RECORD_FIELD_BYTES);
    let index_node_header_widths_without_key_range = [4usize, 2, 1, 1, 2, 32, 8, 1, 1, 8, 8, 4, 4, 4, 2, 2, 2];
    assert_eq!(index_node_header_widths_without_key_range.iter().sum::<usize>(), INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE);
    let packed_header_widths = [4usize, 2, 1, 1, 2, 32, 1, 8, 2, 8, 8, 2, 2, 8, 8, 4, 10, 4];
    assert_eq!(packed_header_widths.iter().sum::<usize>(), PACKED_UNIT_PLAINTEXT_HEADER_BYTES);
    assert_eq!(index_node_header_bytes(8) + RESERVED_NONCE_MAC_ALGORITHM_BYTES, 131);
}

#[test]
fn key_range_reading_tells_entry_key_range_from_subtree_coverage() {
    assert_eq!(key_range_reading((1, 1), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::EntryKeyRange);
    assert_eq!(key_range_reading((0, u64::MAX), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::SubtreeCoverage);
    assert_eq!(key_range_reading((0, u64::MAX), (0, u64::MAX), &[(0, u64::MAX)]), KeyRangeReading::BothReadingsCoincide);
    assert_eq!(key_range_reading((2, 3), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::Neither);
}

#[test]
fn cutting_cuts_at_force_unit_access_and_counts_a_barrier_step_per_device() {
    let view = |region_kind: RegionKind, is_barrier: bool, is_force_unit_access: bool| OperationView { is_barrier, region_kind, is_force_unit_access };
    // 一个单元两块盘、一道屏障两步、journal 两块盘、一道屏障、根槽 FUA、系统配置两块盘、一道屏障。
    let operations = vec![
        view(RegionKind::Unit, false, false),
        view(RegionKind::Unit, false, false),
        view(RegionKind::Unrecognized, true, false),
        view(RegionKind::Unrecognized, true, false),
        view(RegionKind::JournalRecord, false, false),
        view(RegionKind::JournalRecord, false, false),
        view(RegionKind::Unrecognized, true, false),
        view(RegionKind::Unrecognized, true, false),
        view(RegionKind::RootRecordSlot, false, true),
        view(RegionKind::SystemConfigurationSlot, false, false),
        view(RegionKind::SystemConfigurationSlot, false, false),
        view(RegionKind::Unrecognized, true, false),
        view(RegionKind::Unrecognized, true, false),
    ];
    let pieces = cut_into_pieces(&operations);
    assert_eq!(segment_count_text(&pieces), expected_count_text(1));
    assert_eq!(segment_kind_text(&pieces), expected_kind_text(1));
}

#[test]
fn coverage_counts_a_nonzero_byte_in_constant_zero_padding() {
    let mut map = CoverageMap::new(8);
    map.claim(0, 4, ByteClass::LayoutOneCell);
    map.claim(4, 8, ByteClass::ConstantZeroPadding);
    let region = DecodedRegion {
        step: 0,
        device: 0,
        offset: 0,
        bytes: vec![1, 2, 3, 4, 0, 0, 7, 0],
        region_kind: RegionKind::Unrecognized,
        body: DecodedBody::Unrecognized,
        structure_failures: Vec::new(),
        coverage: map,
        locations_inside: Vec::new(),
    };
    let segment = SegmentAnalysis {
        start_step: 0,
        end_step: 1,
        label: SegmentLabel::Outside { segment_name: "synthetic".to_string() },
        is_deletion_publish: false,
        operations: Vec::new(),
        regions: vec![region],
        units: Vec::new(),
        root_records: Vec::new(),
        journal_records: Vec::new(),
        system_configurations: Vec::new(),
        structure_failures: Vec::new(),
        unreferenced_units: Vec::new(),
    };
    let analysis = HistoryAnalysis { name: "synthetic".to_string(), segments: vec![segment], states_before: Vec::new(), states_after: Vec::new() };
    let totals = coverage_of(&analysis);
    assert_eq!(totals.constant_zero_nonzero_bytes, 1);
    assert_eq!(totals.constant_zero_bytes, 4);
    assert_eq!(totals.layout_one_bytes, 4);
    assert_eq!(totals.unregistered_bytes, 0);
}

#[test]
fn export_lines_are_all_anchored_and_every_history_carries_the_registered_parameters() {
    return_not_run_unless_exports_present!();
    let export_files = &fixture().export_files;
    assert_eq!(export_files.len(), 2);
    for (export, expected_histories) in export_files.iter().zip(HISTORIES_OF_EACH_EXPORT_FILE) {
        assert_eq!(export.line_count, export.anchored_line_count);
        assert_eq!(export.done_emitted, Some(export.line_count));
        assert!(export.malformed_lines.is_empty(), "{:?}", export.malformed_lines);
        assert_eq!(export.histories.iter().map(|history| history.name.as_str()).collect::<Vec<_>>(), expected_histories);
    }
    assert_eq!(HISTORIES_OF_EACH_EXPORT_FILE, [&["h1", "h2"][..], &["h3"][..]]);
    for history in &fixture().histories {
        assert!(configuration_matches_registration(history), "{}", history.name);
        assert_eq!(history.configuration.get("file_bytes").map(String::as_str), Some("81585"));
    }
    let record_two_kept_history = history_named("h3");
    let sequential = field_number(&record_two_kept_history.configuration, "sequential_write_time_seconds").expect("H3 带顺序写的时间");
    assert_eq!(field_number(&record_two_kept_history.configuration, "intent_write_time_seconds"), Some(sequential + 60));
    assert!(!history_named("h1").configuration.contains_key("intent_write_time_seconds"));
    assert!(!history_named("h2").configuration.contains_key("intent_write_time_seconds"));
}

#[test]
fn decoded_structure_passes_every_stop_check_on_the_export() {
    return_not_run_unless_exports_present!();
    for judgement in &fixture().judgements {
        assert!(judgement.structure_failures.is_empty(), "{:?}", judgement.structure_failures);
        assert!(judgement.role_mismatches.is_empty(), "{:?}", judgement.role_mismatches);
        assert!(judgement.tiling_failures.is_empty(), "{:?}", judgement.tiling_failures);
    }
}

#[test]
fn intent_record_cells_read_the_registered_offsets() {
    return_not_run_unless_exports_present!();
    assert_eq!(cell_row("h1", 5, "L03-1", "nlink").actual, "0");
    assert_eq!(cell_row("h1", 5, "L03-2", "size").actual, "81585");
    assert_eq!(cell_row("h1", 5, "L03-3", "blocks").expected, "160");
    assert_eq!(cell_row("h1", 5, "L03-3", "blocks").actual, "160");
    assert_eq!(cell_row("h1", 5, "L03-6", "mode").actual, "33188");
    assert_eq!(cell_row("h1", 5, "L03-5", "times").actual, "1788000060/1788000060/1788000060");
    assert!(!cell_row("h1", 5, "L03-5", "times").discriminable);
    let record_two_kept_intent_seconds = history_named("h3").configuration.get("intent_write_time_seconds").expect("H3 带意图的调用方时间").clone();
    assert_eq!(cell_row("h3", 6, "L03-5", "times").expected, three_equal_times_text(&record_two_kept_intent_seconds));
    assert!(cell_row("h3", 6, "L03-5", "times").discriminable);
}

#[test]
fn batch_sizes_are_remaining_units_times_32634_and_blocks_round_up() {
    return_not_run_unless_exports_present!();
    assert_eq!(cell_row("h1", 6, "L03-2", "size").expected, "65268");
    assert_eq!(cell_row("h1", 6, "L03-3", "blocks").expected, "128");
    assert_eq!(cell_row("h1", 7, "L03-2", "size").expected, "32634");
    assert_eq!(cell_row("h1", 7, "L03-3", "blocks").expected, "64");
}

#[test]
fn change_counts_equal_the_publish_txg_in_every_deletion_version() {
    return_not_run_unless_exports_present!();
    for (history, txg) in [("h1", 5), ("h1", 6), ("h1", 7), ("h2", 5), ("h2", 11), ("h2", 12)] {
        let row = cell_row(history, txg, "L03-4", "change_count");
        assert_eq!(row.expected, txg.to_string());
        assert_eq!(row.actual, txg.to_string());
    }
    // H3 只钉期望一栏（新加的单测不把新导出的读数写成断言，跑前登记 e172-r2 第五节 5.8）。
    for txg in [6u64, 7, 8] {
        assert_eq!(cell_row("h3", txg, "L03-4", "change_count").expected, txg.to_string());
    }
}

#[test]
fn intent_tombstone_record_and_queue_root_match_the_registered_values() {
    return_not_run_unless_exports_present!();
    assert_eq!(cell_row("h1", 5, "L03-9", "range_end").expected, u64::MAX.to_string());
    assert!(cell_row("h1", 5, "L03-9", "range_end").is_equal());
    assert_eq!(cell_row("h1", 5, "L03-9", "death_txg").expected, "5");
    assert!(cell_row("h1", 5, "L03-9", "death_txg").is_equal());
    assert!(cell_row("h1", 5, "L03-8", "deleted_inodes_root").expected.contains("tree65544"));
    assert!(cell_row("h1", 5, "L03-8", "deleted_inodes_root").is_equal());
    assert!(cell_row("h1", 5, "L03-10", "tombstone_container").is_equal());
    assert!(cell_row("h1", 5, "L03-11", "tombstone_root").is_equal());
}

#[test]
fn final_batch_removes_the_record_and_leaves_an_empty_inode_root() {
    return_not_run_unless_exports_present!();
    for field in ["deleted_record_in_written_containers", "inode_root_entry_count", "inode_root_key_range", "inode_leaf_containers_written"] {
        let row = cell_row("h1", 8, "L03-7", field);
        assert!(row.is_equal(), "{row:?}");
    }
    assert_eq!(cell_row("h1", 8, "L03-7", "inode_root_key_range").expected, "0/0");
}

/// L03-13 每一版的期望一栏按带退役项的式子（第 2 次断言过的 98304 / 98386 读数是退役树加进之前的，期望改了跟着改；不把 r3 的读数写成断言）。
#[test]
fn committed_reservation_rows_expect_the_formula_with_retirement_in_every_deletion_version() {
    return_not_run_unless_exports_present!();
    for (history, txg, expected) in [
        ("h1", 5u64, "196678"),
        ("h1", 6, "196678"),
        ("h1", 7, "196678"),
        ("h1", 8, "196608"),
        ("h3", 6, "196830"),
        ("h3", 7, "196830"),
        ("h3", 8, "196830"),
        ("h3", 9, "196760"),
    ] {
        let row = judgement_named(history).cell_rows.iter().find(|row| row.txg == txg && row.cell == "L03-13").expect("每一版一行");
        assert_eq!(row.expected, expected, "{row:?}");
    }
}

#[test]
fn harness_history_intent_segment_string_kind_string_and_roles_are_decoded_independently() {
    return_not_run_unless_exports_present!();
    let row = judgement_named("h1").segment_rows.iter().find(|row| row.publish_kind == "unlink_intent").expect("意图那一行");
    assert_eq!(row.actual_count_text, "26+2+1+2");
    assert_eq!(
        row.actual_kind_text,
        "[unit_write×26,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]"
    );
    assert_eq!(row.actual_roles, "13:t3,t4,t5,t5@0.0.61,t5@0.1.61,t5@1.0.0,t5@1.1.0,t6,t7,t8,td,tt,tt@0");
    assert!(row.roles_equal());
    assert_eq!(row.actual_root_location, "0:7344128");
}

/// R5.a 是附带的（跑前登记 e172-r2 第六节 6.3，不进任何判定）；这一条照第 1 次留作回归。
#[test]
fn final_batch_drops_exactly_the_inode_leaf_container_against_the_two_to_one_batch() {
    return_not_run_unless_exports_present!();
    assert!(line_starting_with("name=r5a ").contains(" only_in_two_to_one=t3 "));
}

/// R5.a 的登记差由 `registered_roles` 现算：2 → 1 独有 t3，最后一批独有 tr、tr@0（第 2 次那一种「最后一批独有为空」不再算登记的）。
#[test]
fn registered_final_batch_difference_is_the_inode_leaf_against_the_retirement_container_and_root() {
    let names = |roles: &[&str]| roles.iter().map(|role| (*role).to_string()).collect::<Vec<String>>();
    assert!(matches_the_registered_final_batch_difference(&names(&["t3"]), &names(&["tr", "tr@0"])));
    assert!(!matches_the_registered_final_batch_difference(&names(&["t3"]), &names(&[])));
}

#[test]
fn queue_and_tombstone_roots_record_an_entry_key_range_reading() {
    return_not_run_unless_exports_present!();
    // 第 3 次起最后一批另有一行 `role=tr`（L03-17 只记值），这里只看意图那一次的 td 与 tt。
    let readings: Vec<&String> =
        fixture().lines.iter().filter(|line| line.starts_with("name=r7 history=h1") && (line.contains(" role=td ") || line.contains(" role=tt "))).collect();
    assert_eq!(readings.len(), 2);
    assert!(readings[0].contains("role=td level=0 key_range=1..1 entry_keys=1..1 reading=entry_key_range"), "{}", readings[0]);
    assert!(readings[1].contains("role=tt level=1 key_range=1..1 entry_keys=1..1 reading=entry_key_range"), "{}", readings[1]);
}

#[test]
fn mount_writes_are_attributed_to_the_same_segments_the_export_cuts() {
    return_not_run_unless_exports_present!();
    let mount_continuation_judgement = judgement_named("h2");
    assert!(!mount_continuation_judgement.attribution_rows.is_empty());
    assert_eq!(mount_continuation_judgement.misattributed_writes(), 0, "{:?}", mount_continuation_judgement.attribution_rows.iter().filter(|row| !row.is_match).collect::<Vec<_>>());
    let acquisition_writes = mount_continuation_judgement.attribution_rows.iter().filter(|row| row.decoded == WriteAttribution::InstanceAcquisition).count();
    assert_eq!(acquisition_writes, 2);
}

/// 每一条对照在它适用的每一段历史上都先绿后红（跑前登记 e172-r3 第五节 5.4：适用的对共 25 个）；
/// 不适用的只出现在表里写了不适用的那几对上（PC-J、PC-L 在 H3 上不适用：H3 不写退役单元）。
#[test]
fn every_positive_control_turns_its_target_red_on_every_history_it_applies_to() {
    return_not_run_unless_exports_present!();
    let mut held = 0usize;
    let mut not_applicable = Vec::new();
    for (history, judgement) in fixture().histories.iter().zip(&fixture().judgements) {
        for control in PositiveControl::EVERY_CONTROL {
            let (line, holds) = positive_control_line(history, judgement, control);
            match holds {
                Some(holds) => {
                    assert!(holds, "{line}");
                    held += 1;
                }
                None => not_applicable.push(format!("{}@{}", control.name(), history.name)),
            }
        }
    }
    assert_eq!(held, 25);
    assert_eq!(held, PositiveControl::APPLICABLE_PAIRS_EXPECTED);
    assert_eq!(not_applicable, ["PC-F@h1", "PC-G@h1", "PC-I@h1", "PC-I@h2", "PC-F@h3", "PC-G@h3", "PC-J@h3", "PC-L@h3"]);
}

/// PC-C 只重封了墓碑容器自己的载荷校验和：父指针（墓碑树根条目、映射、journal 点名项）里那 4 字节整单元校验和照样对不上，S2 那一格要红。
#[test]
fn resealed_container_still_fails_the_parent_location_checksum() {
    return_not_run_unless_exports_present!();
    let history = history_named("h1");
    let corrupted = corrupted_copy(history, PositiveControl::BumpTombstoneDeathTxg).expect("改得到墓碑容器");
    let judgement = judge_history(&corrupted);
    let parent_failures = judgement.structure_failures.iter().filter(|failure| failure.contains("父指针位置条目里的整单元校验和")).count();
    assert!(parent_failures >= 2, "{:?}", judgement.structure_failures);
    assert!(judgement.structure_failures.iter().all(|failure| !failure.contains("码 3 载荷校验和不对")));
}

#[test]
fn harness_history_window_string_is_the_four_registered_strings_end_to_end() {
    return_not_run_unless_exports_present!();
    // 跑前登记 e172-r3 第七节 A-roles-final：H1 最后一批 12 个单元（多 tr@0、tr），整窗四串 26、24、22、24；只钉期望一栏。
    assert!(
        line_starting_with("name=r3d history=h1 ").starts_with("name=r3d history=h1 expected=26+2+1+2+24+2+1+2+22+2+1+2+24+2+1+2 actual="),
        "{}",
        line_starting_with("name=r3d history=h1 ")
    );
    // H3 那一行只钉期望一栏：13、12、11、11 四串首尾相接。
    assert!(
        line_starting_with("name=r3d history=h3 ").starts_with("name=r3d history=h3 expected=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 actual="),
        "{}",
        line_starting_with("name=r3d history=h3 ")
    );
}

// ---- 跑前登记 e172-r2 第五节 5.8 新加的 T1–T13 ----

/// T1：合成系统配置槽，偏移 433 写 5000、437 起写 T_dirty（2 GiB），解出 5000。
#[test]
fn time_threshold_is_read_from_offset_433_of_a_synthetic_system_configuration_slot() {
    let mut slot = vec![0u8; 4096];
    slot[0..4].copy_from_slice(b"SFSB");
    slot[433..437].copy_from_slice(&5000u32.to_le_bytes());
    slot[437..445].copy_from_slice(&(2u64 << 30).to_le_bytes());
    reseal_system_configuration_slot(&mut slot);
    let mut coverage = CoverageMap::new(slot.len());
    let mut failures = Vec::new();
    let configuration = decode_system_configuration(&slot, &mut coverage, &mut failures);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(configuration.time_threshold_milliseconds, 5000);
}

/// T2：每段历史 R1.t 的行数 = 窗口里系统配置槽写次数（从导出的写区域现数），期望一栏都是 5000。
#[test]
fn time_threshold_rows_cover_every_system_configuration_slot_write_and_expect_5000() {
    return_not_run_unless_exports_present!();
    for history in &fixture().histories {
        let slot_writes = history
            .operations
            .iter()
            .filter(|operation| matches!(operation, WindowOperation::Write(write) if (write.offset == 0 || write.offset == 4096) && write.bytes.len() == 4096))
            .count();
        let prefix = format!("name=r1t history={} ", history.name);
        let rows: Vec<&String> = fixture().lines.iter().filter(|line| line.starts_with(&prefix)).collect();
        assert!(slot_writes > 0, "{}", history.name);
        assert_eq!(rows.len(), slot_writes, "{}", history.name);
        assert!(rows.iter().all(|line| line.contains(" expected=5000 ")), "{rows:?}");
    }
}

/// T3：H3 选到 L03-7 第二支、只有 H3 选到；第二支的期望一栏（写出的叶 1 片、根条目数 1）。
#[test]
fn only_the_record_two_kept_history_takes_the_second_layout03_item7_branch_and_expects_one_leaf_and_one_root_entry() {
    return_not_run_unless_exports_present!();
    assert!(line_starting_with("name=l03_7_branch_check ").contains("second_branch_histories=h3 exactly_h3=true"));
    assert_eq!(cell_row("h3", 9, "L03-7", "inode_root_entry_count").expected, "1");
    assert_eq!(cell_row("h3", 9, "L03-7", "inode_leaf_containers_written").expected, "1");
    assert_eq!(cell_row("h3", 9, "L03-7", "deleted_record_in_written_containers").expected, "0");
    assert_eq!(cell_row("h1", 8, "L03-7", "inode_root_entry_count").expected, "0");
    assert_eq!(cell_row("h1", 8, "L03-7", "inode_leaf_containers_written").expected, "0");
}

/// T4：H3 意图 L03-5 的期望 = 导出的 `intent_write_time_seconds=`（与顺序写的时间不同），可分辨。
#[test]
fn record_two_kept_history_intent_times_expect_the_callers_time_and_are_discriminable() {
    return_not_run_unless_exports_present!();
    let configuration = &history_named("h3").configuration;
    let row = cell_row("h3", 6, "L03-5", "times");
    assert_eq!(row.expected, three_equal_times_text(&configuration["intent_write_time_seconds"]));
    assert_ne!(row.expected, three_equal_times_text(&configuration["sequential_write_time_seconds"]));
    assert!(row.discriminable);
}

/// T5：合成输入——批的 L03-5 期望取上一版那条记录的三格，上一版三格与 `sequential_write_time_seconds` 不同。
#[test]
fn batch_times_expect_the_previous_records_three_values_not_the_sequential_write_time() {
    let mut record = vec![0u8; INODE_RECORD_BYTES];
    for offset in [64usize, 72, 80] {
        record[offset..offset + 8].copy_from_slice(&1_788_000_120u64.to_le_bytes());
    }
    let configuration = BTreeMap::from([("sequential_write_time_seconds".to_string(), "1788000060".to_string())]);
    assert_eq!(expected_times_of(false, &configuration, Some(&record)), ("1788000120/1788000120/1788000120".to_string(), true));
    assert_eq!(expected_times_of(true, &configuration, Some(&record)), ("1788000060/1788000060/1788000060".to_string(), false));
}

/// T6：R5.b 行在；判别力自证行里两套期望（H1 的、H3 的）不同时判绿。
#[test]
fn row_five_record_two_kept_history_line_is_reported_and_its_two_expectations_are_never_both_green() {
    return_not_run_unless_exports_present!();
    assert!(line_starting_with("name=r5b history=h3 ").contains(" final_writes_t3="));
    let discrimination = line_starting_with("name=r5b_discrimination history=h3 ");
    assert!(!discrimination.contains("h1_expectation_green=true h3_expectation_green=true"), "{discrimination}");
    assert!(line_starting_with("name=r3_geometry history=h3 ").contains(" expectations_differ=true "));
}

/// T7（跑前登记 e172-r3 改成按版本）：A-tag6 是按（历史，删除类发布）分的字面常量（第七节 7.1）；H3 那四行的期望一栏照它。
#[test]
fn committed_reservation_anchor_constants_follow_the_registered_value_of_each_publish() {
    for history in ["h1", "h2"] {
        for publish in [DeletionPublish::Intent, DeletionPublish::BatchLeavingTwoUnits, DeletionPublish::BatchLeavingOneUnit] {
            assert_eq!(registered_committed_reservation_of_publish(history, publish), Some(196_678), "{history} {publish:?}");
        }
        assert_eq!(registered_committed_reservation_of_publish(history, DeletionPublish::FinalBatch), Some(196_608), "{history}");
    }
    for publish in [DeletionPublish::Intent, DeletionPublish::BatchLeavingTwoUnits, DeletionPublish::BatchLeavingOneUnit] {
        assert_eq!(registered_committed_reservation_of_publish("h3", publish), Some(196_830), "{publish:?}");
    }
    assert_eq!(registered_committed_reservation_of_publish("h3", DeletionPublish::FinalBatch), Some(196_760));
    assert_eq!(registered_committed_reservation_of_publish("h4", DeletionPublish::Intent), None);
    return_not_run_unless_exports_present!();
    for (txg, expected) in [(6u64, 196_830u64), (7, 196_830), (8, 196_830), (9, 196_760)] {
        assert!(line_starting_with(&format!("name=anchor anchor=A-tag6 history=h3 txg={txg} ")).contains(&format!(" expected={expected} ")));
    }
}

/// T8：PC-G 的副本恰有一次写换了段，换的是挂载写行之前那一截的最后一次写。
#[test]
fn positive_control_moves_exactly_the_last_write_of_the_mount_segment_into_the_next_segment() {
    return_not_run_unless_exports_present!();
    let history = history_named("h2");
    let corrupted = corrupted_copy(history, PositiveControl::ShiftMountSegmentEndToItsLastWrite).expect("H2 有挂载写行之前那一段");
    let label_at = |segments: &[ExportSegment], step: usize| segments.iter().find(|segment| step >= segment.start_step && step < segment.end_step).map(|segment| segment.label.clone());
    let write_steps: Vec<usize> = history
        .operations
        .iter()
        .filter_map(|operation| match operation {
            WindowOperation::Write(write) => Some(write.step),
            WindowOperation::Barrier { .. } => None,
        })
        .collect();
    let moved: Vec<usize> = write_steps.iter().copied().filter(|step| label_at(&history.segments, *step) != label_at(&corrupted.segments, *step)).collect();
    let mount = history.segments.iter().find(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish").expect("有那一段");
    let last_write_in_mount = write_steps.iter().copied().filter(|step| *step >= mount.start_step && *step < mount.end_step).max().expect("那一段有写");
    assert_eq!(moved, [last_write_in_mount]);
}

/// T9：PC-H 的副本恰改了一次系统配置槽写，那次解出 5，整槽校验和过。
#[test]
fn positive_control_rewrites_exactly_one_system_configuration_write_to_5_and_reseals_its_slot() {
    return_not_run_unless_exports_present!();
    for history in &fixture().histories {
        let corrupted = corrupted_copy(history, PositiveControl::RewriteIntentTimeThresholdAsSeconds).expect("每段历史的意图都有系统配置槽写");
        let changed: Vec<&RegionWrite> = history
            .operations
            .iter()
            .zip(&corrupted.operations)
            .filter_map(|(original, copy)| match (original, copy) {
                (WindowOperation::Write(original_write), WindowOperation::Write(copy_write)) if original_write.bytes != copy_write.bytes => Some(copy_write),
                (WindowOperation::Write(_) | WindowOperation::Barrier { .. }, WindowOperation::Write(_) | WindowOperation::Barrier { .. }) => None,
            })
            .collect();
        assert_eq!(changed.len(), 1, "{}", history.name);
        assert_eq!(region_kind_of(changed[0]), RegionKind::SystemConfigurationSlot);
        assert_eq!(read_u32(&changed[0].bytes, 433), 5);
        assert!(whole_slot_checksum_holds(&changed[0].bytes, 155), "{}", history.name);
    }
}

/// T10：PC-I 的副本里 H3 意图 1 号记录三格 = 顺序写的时间，码 3 载荷校验和过。
#[test]
fn positive_control_copies_the_sequential_write_time_into_the_record_two_kept_intent_record_and_reseals_the_container() {
    return_not_run_unless_exports_present!();
    let history = history_named("h3");
    let sequential = field_number(&history.configuration, "sequential_write_time_seconds").expect("H3 带顺序写的时间");
    let corrupted = corrupted_copy(history, PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord).expect("H3 意图写了 t3");
    let judgement = judge_history(&corrupted);
    let intent = judgement.analysis.segments.iter().find(|segment| segment.label_kind() == "unlink_intent").expect("有意图那一段");
    let record = record_of_inode(&records_written_in(intent, INODE_TREE, 2), DELETED_INODE).expect("意图写了 1 号记录");
    for offset in [64usize, 72, 80] {
        assert_eq!(read_u64(&record, offset), sequential);
    }
    assert!(intent.structure_failures.iter().all(|failure| !failure.contains("码 3 载荷校验和不对")), "{:?}", intent.structure_failures);
}

fn synthetic_configuration(instance: u64, journal_tail: u64) -> SystemConfiguration {
    SystemConfiguration { generation: 0, journal_tail, instance, time_threshold_milliseconds: 5000 }
}

/// T11：同实例、tail 等于别的实例某次发布的计数器：轮换要实例代号相同，R8.c 归「别的挂载步」。
#[test]
fn rotation_needs_the_same_instance_so_a_tail_matching_another_instances_publish_is_another_mount_step() {
    let attribution = attribute_system_configuration(10, &synthetic_configuration(2, 7), Some(&synthetic_configuration(2, 6)), &[(7, 7, 1, 3)]);
    assert_eq!(attribution, WriteAttribution::OtherMountStep);
}

/// T12：实例换了、tail 没动，tail 恰等于新实例某次在前的发布计数器：先判取号，R8.c 归「取号」。
#[test]
fn acquisition_is_judged_before_rotation_even_when_the_tail_matches_an_earlier_publish_of_the_new_instance() {
    let attribution = attribute_system_configuration(10, &synthetic_configuration(2, 7), Some(&synthetic_configuration(1, 7)), &[(9, 7, 2, 3)]);
    assert_eq!(attribution, WriteAttribution::InstanceAcquisition);
}

/// T13：H3 的 A-txg、A-根槽、A-journal、A-系统配置期望一栏等于跑前登记 e172-r2 第七节 7.1 的 H3 值。
#[test]
fn record_two_kept_history_txg_root_slot_journal_and_system_configuration_anchors_expect_the_registered_values() {
    return_not_run_unless_exports_present!();
    assert!(line_starting_with("name=anchor anchor=A-txg history=h3 deletion_publishes ").contains(" expected=6,7,8,9 "));
    for (txg, root, journal, generation, slot) in [
        (6u64, "0:1056768", 16_797_696u64, 9u64, 4096u64),
        (7, "1:4202496", 16_801_792, 10, 0),
        (8, "0:7348224", 16_805_888, 11, 4096),
        (9, "0:1060864", 16_809_984, 12, 0),
    ] {
        assert!(line_starting_with(&format!("name=anchor anchor=A-root-slot history=h3 txg={txg} ")).contains(&format!(" expected={root} ")));
        assert!(line_starting_with(&format!("name=anchor anchor=A-journal history=h3 txg={txg} ")).contains(&format!(" expected=0:{journal},1:{journal} ")));
        assert!(
            line_starting_with(&format!("name=anchor anchor=A-system-configuration history=h3 txg={txg} "))
                .contains(&format!(" expected=0:{slot}:generation{generation},1:{slot}:generation{generation} "))
        );
    }
}

// ---- 跑前登记 e172-r3 第五节 5.8 新加的 T14–T24 ----

/// T14：H_退役 = 1、n_live = 0、n_记录 = 1 ⇒ 229446（第七节 7.2 A-arith；退役三槽那一项的取样点）。
#[test]
fn committed_reservation_with_one_retirement_level_and_one_dead_record_is_229446() {
    assert_eq!(expected_committed_reservation(0, 1, 0, 1, RetirementShare::Included), 229_446);
}

/// T15：H_墓碑 = 1、n_live = n_记录 = 1 ⇒ 229528。
#[test]
fn committed_reservation_with_one_tombstone_level_and_one_live_record_is_229528() {
    assert_eq!(expected_committed_reservation(1, 0, 1, 1, RetirementShare::Included), 229_528);
}

/// T16：n_live = 0、n_记录 = 1 ⇒ 196678；n_live = 1、n_记录 = 0 ⇒ 196690（41 与 35 两个系数互换就分得开）。
#[test]
fn live_and_record_coefficients_are_told_apart_by_196678_and_196690() {
    assert_eq!(expected_committed_reservation(0, 0, 0, 1, RetirementShare::Included), 196_678);
    assert_eq!(expected_committed_reservation(0, 0, 1, 0, RetirementShare::Included), 196_690);
}

fn synthetic_region(body: DecodedBody) -> DecodedRegion {
    DecodedRegion {
        step: 0,
        device: 0,
        offset: 0,
        bytes: Vec::new(),
        region_kind: RegionKind::Unit,
        body,
        structure_failures: Vec::new(),
        coverage: CoverageMap::new(0),
        locations_inside: Vec::new(),
    }
}

fn synthetic_container(birth_tree: u64, packed_record_type: u64, record_width: usize, record_count: usize) -> PackedContainer {
    PackedContainer {
        birth_tree,
        packed_record_type,
        container_number: 1,
        container_birth: 0,
        record_count,
        record_width,
        birth_txg: 8,
        records: vec![vec![0; record_width]; record_count],
    }
}

/// T17：合成单元 (码 3, 退役树) 与 (码 2, 退役树) 认成 `tr@0` 与 `tr`；墓碑树的照旧是 `tt@0` 与 `tt`。
#[test]
fn retirement_container_and_root_units_are_named_tr_at_0_and_tr() {
    let container = synthetic_region(DecodedBody::PackedContainer(synthetic_container(RETIREMENT_TREE, RETIREMENT_PACKED_RECORD_TYPE, RETIREMENT_RECORD_BYTES, 1)));
    let root_node = |tree_identifier: u64| IndexNode {
        tree_identifier,
        level: 1,
        key_width: 8,
        key_range_low: vec![0; 8],
        key_range_high: vec![0; 8],
        birth_txg: 8,
        entry_count: 0,
        entry_width: INODE_INTERNAL_ENTRY_BYTES,
        entries: Vec::new(),
    };
    let root = synthetic_region(DecodedBody::IndexNode(root_node(RETIREMENT_TREE)));
    let name_of = |region: &DecodedRegion| role_name_of(&role_identity_of(region, true), region, &[], &[1], &[1]);
    assert_eq!(name_of(&container), "tr@0");
    assert_eq!(name_of(&root), "tr");
    let tombstone_container = synthetic_region(DecodedBody::PackedContainer(synthetic_container(TOMBSTONE_TREE, 1, TOMBSTONE_RECORD_BYTES, 1)));
    let tombstone_root = synthetic_region(DecodedBody::IndexNode(root_node(TOMBSTONE_TREE)));
    assert_eq!(name_of(&tombstone_container), "tt@0");
    assert_eq!(name_of(&tombstone_root), "tt");
}

/// T18：最后一批两支——那片叶只装被删那一条：含 tr@0、tr，不含 t3，12 个，次序照 layout/03 第 60 行；另有记录：含 t3，不含 tr，11 个。
#[test]
fn final_batch_roles_take_the_retirement_units_when_the_leaf_empties_and_the_inode_leaf_when_other_records_remain() {
    let emptied = registered_roles("unlink_batch", 0, DeletedRecordLeaf::OnlyTheDeletedRecord).expect("最后一批有登记");
    assert_eq!(emptied, ["t2", "t4", "t5@0.0.61", "t5@0.1.61", "t5@1.0.0", "t5@1.1.0", "t5", "t6", "tr@0", "tr", "t7", "t8"]);
    let beside = registered_roles("unlink_batch", 0, DeletedRecordLeaf::BesideOtherRecords).expect("最后一批有登记");
    assert!(beside.contains(&"t3"));
    assert!(!beside.contains(&"tr") && !beside.contains(&"tr@0"));
    assert_eq!(beside.len(), 11);
    assert_eq!(expected_count_text(emptied.len()), "24+2+1+2");
}

/// T19：R-ret 的两份期望由同一个式子算（第七节 7.1 四个取样点），两份恒不同；导出上三段历史共 12 行，期望两栏照取样点。
#[test]
fn retirement_arm_rows_report_both_expectations_for_every_deletion_version() {
    assert_eq!(retirement_arm_expectations(0, 0, 0, 1), (196_678, 98_304));
    assert_eq!(retirement_arm_expectations(0, 0, 0, 0), (196_608, 98_304));
    assert_eq!(retirement_arm_expectations(0, 0, 1, 2), (196_830, 98_386));
    assert_eq!(retirement_arm_expectations(0, 0, 1, 1), (196_760, 98_386));
    return_not_run_unless_exports_present!();
    let rows: Vec<&String> = fixture().lines.iter().filter(|line| line.starts_with("name=r_ret ")).collect();
    assert_eq!(rows.len(), 12, "{rows:?}");
    for (history, publish, with_retirement, without_retirement) in [
        ("h1", "intent", 196_678u64, 98_304u64),
        ("h1", "final", 196_608, 98_304),
        ("h2", "final", 196_608, 98_304),
        ("h3", "two_to_one", 196_830, 98_386),
        ("h3", "final", 196_760, 98_386),
    ] {
        let row = rows
            .iter()
            .find(|line| line.starts_with(&format!("name=r_ret history={history} ")) && line.contains(&format!(" publish={publish} ")))
            .unwrap_or_else(|| panic!("{history} {publish} 没有 R-ret 行"));
        assert!(row.contains(&format!(" with_retirement={with_retirement} without_retirement={without_retirement} ")), "{row}");
    }
}

/// T20：钉值层的对法——意图、3 → 2、2 → 1 对第 2–4 格（196678），最后一批对第 5 格（196608），第 1 格 196760 在窗口外；
/// 导出上 H1、H2 各 4 行、`pinned` 一栏照它，各有一行 `in_window=false`。
#[test]
fn harness_pin_rows_compare_the_session_and_continuation_histories_with_the_pinned_value_of_each_publish() {
    let pinned_of = |publish: DeletionPublish| harness_pinned_value(harness_index_of(publish));
    assert_eq!(pinned_of(DeletionPublish::Intent), Some(196_678));
    assert_eq!(pinned_of(DeletionPublish::BatchLeavingTwoUnits), Some(196_678));
    assert_eq!(pinned_of(DeletionPublish::BatchLeavingOneUnit), Some(196_678));
    assert_eq!(pinned_of(DeletionPublish::FinalBatch), Some(196_608));
    assert_eq!(harness_index_of(DeletionPublish::Intent), 2);
    assert_eq!(harness_pinned_value(1), Some(196_760));
    assert_eq!(harness_pinned_value(0), None);
    assert_eq!(harness_pinned_value(6), None);
    assert!(history_is_compared_with_the_harness_pins("h1") && history_is_compared_with_the_harness_pins("h2"));
    assert!(!history_is_compared_with_the_harness_pins("h3"));
    return_not_run_unless_exports_present!();
    for history in ["h1", "h2"] {
        let rows: Vec<&String> = fixture().lines.iter().filter(|line| line.starts_with(&format!("name=harness_pin history={history} "))).collect();
        assert_eq!(rows.len(), 4, "{rows:?}");
        let pinned_column: Vec<bool> = rows.iter().map(|line| line.contains(" pinned=196678 ")).collect();
        assert_eq!(pinned_column, [true, true, true, false], "{rows:?}");
        assert!(rows[3].contains(" publish=final harness_index=5 pinned=196608 "), "{}", rows[3]);
        assert!(line_starting_with(&format!("name=harness_pin_out_of_window history={history} ")).ends_with(" harness_index=1 pinned=196760 in_window=false"));
    }
    assert!(!fixture().lines.iter().any(|line| line.starts_with("name=harness_pin history=h3 ")));
    assert!(line_starting_with("name=harness_pin_same_value history=h3 ").contains(" same_value_as_harness_first="));
}

/// T21：死亡 txg 的期望取最后一批（身份消失那一次）的 txg；PC-J 的副本恰改了每盘一份退役记录的死亡 txg（加 1），码 3 载荷校验和过。
#[test]
fn retirement_death_txg_expects_the_final_batch_txg_and_the_death_txg_control_bumps_one_record_per_copy() {
    let publishes = [
        (DeletionPublish::Intent, 5u64),
        (DeletionPublish::BatchLeavingTwoUnits, 6),
        (DeletionPublish::BatchLeavingOneUnit, 7),
        (DeletionPublish::FinalBatch, 8),
    ];
    assert_eq!(expected_retirement_death_txg(&publishes), Some(8));
    assert_eq!(expected_retirement_death_txg(&publishes[..3]), None);
    return_not_run_unless_exports_present!();
    assert_eq!(cell_row("h1", 8, "L03-16", "death_txg").expected, "8");
    let history = history_named("h1");
    let corrupted = corrupted_copy(history, PositiveControl::BumpRetirementDeathTxg).expect("H1 写了退役容器");
    let changed: Vec<(&RegionWrite, &RegionWrite)> = history
        .operations
        .iter()
        .zip(&corrupted.operations)
        .filter_map(|(original, copy)| match (original, copy) {
            (WindowOperation::Write(original_write), WindowOperation::Write(copy_write)) if original_write.bytes != copy_write.bytes => Some((original_write, copy_write)),
            (WindowOperation::Write(_) | WindowOperation::Barrier { .. }, WindowOperation::Write(_) | WindowOperation::Barrier { .. }) => None,
        })
        .collect();
    assert_eq!(changed.len(), 2);
    let death_offset = PACKED_UNIT_RECORDS_OFFSET + 26;
    for (original, copy) in changed {
        assert_eq!(read_u64(&copy.bytes, death_offset), read_u64(&original.bytes, death_offset) + 1);
        assert!(crc_matches(&copy.bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..], read_u32(&copy.bytes, 89)));
    }
}

/// T22：PC-K 的副本里意图那次 t6 的标签 6 恰少 70、PC-L 的副本里退役树根首条分隔 key 为 2，两份码 2 载荷 CRC 都过。
#[test]
fn tag_six_control_reduces_the_intent_value_by_70_and_separator_control_bumps_the_retirement_separator_to_2_with_resealed_checksums() {
    return_not_run_unless_exports_present!();
    let history = history_named("h1");
    let accounting_entries = |bytes: &[u8]| -> Vec<(u64, u64)> {
        let key_width = usize::from(bytes[51]);
        let entries_offset = index_node_header_bytes(key_width) + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
        let entry_count = usize_of(read_u16(bytes, 82 + 2 * key_width));
        (0..entry_count).map(|entry_index| entries_offset + entry_index * ACCOUNTING_ENTRY_BYTES).map(|start| (read_u16(bytes, start), read_u64(bytes, start + 22))).collect()
    };
    let payload_crc_holds = |bytes: &[u8]| {
        let key_width = usize::from(bytes[51]);
        crc_matches(&bytes[index_node_header_bytes(key_width)..], read_u32(bytes, 76 + 2 * key_width))
    };
    for (control, tree) in [(PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare, ACCOUNTING_TREE), (PositiveControl::BumpRetirementRootSeparator, RETIREMENT_TREE)] {
        let corrupted = corrupted_copy(history, control).expect("H1 有目标单元");
        let mut changed = 0usize;
        for (original, copy) in history.operations.iter().zip(&corrupted.operations) {
            let (WindowOperation::Write(original_write), WindowOperation::Write(copy_write)) = (original, copy) else { continue };
            if original_write.bytes == copy_write.bytes {
                continue;
            }
            changed += 1;
            assert_eq!(read_u64(&copy_write.bytes, 42), tree);
            assert!(payload_crc_holds(&copy_write.bytes), "{}", control.name());
            if tree == ACCOUNTING_TREE {
                let before: Vec<(u64, u64)> = accounting_entries(&original_write.bytes);
                let after: Vec<(u64, u64)> = accounting_entries(&copy_write.bytes);
                let tag_six = |entries: &[(u64, u64)]| entries.iter().find(|(tag, _)| *tag == 6).map(|(_, value)| *value).expect("有标签 6");
                assert_eq!(tag_six(&after) + POSITIVE_CONTROL_TAG_SIX_REDUCTION_BYTES, tag_six(&before));
            } else {
                let entries_offset = index_node_header_bytes(usize::from(copy_write.bytes[51])) + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
                assert_eq!(read_u64(&copy_write.bytes, entries_offset), 2);
            }
        }
        assert_eq!(changed, 2, "{}", control.name());
    }
}

/// T23：树表 12 条 × 200 = 2400、退役容器装 959 条、水位 = 退役树的树 ID + 1（第七节 A-arith、A-tree-table）；
/// 导出上 L03-12 条目数期望一栏 12、水位期望一栏 65549，退役树那一条期望 `root_zero=false` 只出现在 H1、H2 的最后一批。
#[test]
fn tree_table_expects_twelve_entries_watermark_65549_and_a_retirement_root_only_after_the_emptying_batch() {
    assert_eq!(TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE * TREE_TABLE_ENTRY_BYTES, 2400);
    assert_eq!(retirement_container_capacity(), 959);
    assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE, RETIREMENT_TREE + 1);
    return_not_run_unless_exports_present!();
    let mut nonzero_root_versions = Vec::new();
    for judgement in &fixture().judgements {
        for row in judgement.cell_rows.iter().filter(|row| row.cell == "L03-12" && row.field == "entry_count") {
            assert_eq!(row.expected, "12", "{row:?}");
        }
        for row in judgement.cell_rows.iter().filter(|row| row.cell == "L03-14") {
            assert_eq!(row.expected, "65549", "{row:?}");
        }
        for row in judgement.cell_rows.iter().filter(|row| row.cell == "L03-12" && row.field == "tree65548") {
            if row.expected.ends_with("/root_zerofalse") {
                nonzero_root_versions.push(format!("{}@{}", row.history, row.publish_kind));
            }
        }
    }
    assert_eq!(nonzero_root_versions, ["h1@unlink_batch", "h2@mount_continuation_unlink_batch"]);
}

/// T24：合成码 3 单元（打包记录类型 3、记录宽 34、两条记录）经 `claim_container_records` 之后，那 68 字节全归 layout/03 格。
#[test]
fn retirement_records_in_a_synthetic_code_three_unit_are_claimed_as_layout_three_cells() {
    let container = synthetic_container(RETIREMENT_TREE, RETIREMENT_PACKED_RECORD_TYPE, RETIREMENT_RECORD_BYTES, 2);
    let mut coverage = CoverageMap::new(DATA_UNIT_BYTES);
    let mut locations = Vec::new();
    claim_container_records(&container, &mut coverage, &mut locations);
    let records_end = PACKED_UNIT_RECORDS_OFFSET + 2 * RETIREMENT_RECORD_BYTES;
    assert_eq!(records_end, 204);
    assert!(coverage.classes[PACKED_UNIT_RECORDS_OFFSET..records_end].iter().all(|class| *class == ByteClass::LayoutThreeCell));
    assert!(coverage.classes[records_end..].iter().all(|class| *class == ByteClass::Unclaimed));
    assert!(locations.is_empty());
}

/// 修订 2：合成退役树根（码 2、条目宽 120、一条条目）的条目整条归 layout/03 格，偏移 34 的子指针进父指针核对。
#[test]
fn retirement_root_entries_are_claimed_as_layout_three_cells_with_their_child_pointer() {
    let mut entry = vec![0u8; INODE_INTERNAL_ENTRY_BYTES];
    entry[34] = 1;
    let node = IndexNode {
        tree_identifier: RETIREMENT_TREE,
        level: 1,
        key_width: 8,
        key_range_low: vec![0; 8],
        key_range_high: vec![0; 8],
        birth_txg: 8,
        entry_count: 1,
        entry_width: INODE_INTERNAL_ENTRY_BYTES,
        entries: vec![entry],
    };
    let entries_offset = index_node_header_bytes(8) + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
    let mut coverage = CoverageMap::new(NODE_BYTES);
    let mut locations = Vec::new();
    claim_index_node_entries(&node, entries_offset, &mut coverage, &mut locations);
    let entries_end = entries_offset + INODE_INTERNAL_ENTRY_BYTES;
    assert!(coverage.classes[entries_offset..entries_end].iter().all(|class| *class == ByteClass::LayoutThreeCell));
    assert_eq!(locations.len(), 2);
}

/// L03-15、L03-16、L03-17 的期望一栏只在那片叶删空的最后一批写退役单元；第 3 次新加的几种行都在产物里（不钉读数）。
#[test]
fn retirement_cells_expect_a_container_record_and_root_only_in_the_batch_that_empties_the_leaf() {
    return_not_run_unless_exports_present!();
    assert_eq!(
        cell_row("h1", 8, "L03-15", "retirement_container").expected,
        "code3/type3/width34/birth_tree65548/records_at136/container1/count1"
    );
    assert_eq!(cell_row("h1", 8, "L03-16", "retirement_records_written").expected, "1");
    assert!(cell_row("h1", 8, "L03-17", "retirement_root").expected.starts_with("tag2/tree65548/key_width8/width120/count1/separator1/birth_tree65548/type3/container1/children"));
    for (history, txg) in [("h1", 5u64), ("h1", 7), ("h3", 9)] {
        assert_eq!(cell_row(history, txg, "L03-16", "retirement_records_written").expected, "0");
        assert_eq!(cell_row(history, txg, "L03-15", "retirement_container_written").expected, "false");
        assert_eq!(cell_row(history, txg, "L03-17", "retirement_root_written").expected, "false");
    }
    for prefix in [
        "name=retirement_unit_histories ",
        "name=a_tag6_geometry history=h1 ",
        "name=a_tag6_geometry history=h3 ",
        "name=r3_geometry_roles history=h1 ",
        "name=r3_geometry_roles history=h3 ",
        "name=harness_pin_geometry publish=final ",
        "name=verdict row=c775 ",
    ] {
        let _ = line_starting_with(prefix);
    }
    let trajectory = line_starting_with("name=r_traj history=h1 ");
    for column in [" inode_records=", " retirement_records=", " retirement_levels="] {
        assert!(trajectory.contains(column), "{trajectory}");
    }
}
