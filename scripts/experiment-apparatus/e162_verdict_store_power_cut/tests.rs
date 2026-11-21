// ============================== 单测 ==============================

use super::*;

/// 单测自己的临时目录：`${TMPDIR:-/tmp}/e162-s4-test-<pid>-<名>`，Drop 时删掉。
struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(name: &str) -> TestDirectory {
        let path = std::env::temp_dir().join(format!("e162-s4-test-{}-{name}", std::process::id()));
        let _stale = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("建得了测试目录");
        TestDirectory { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _cleanup = fs::remove_dir_all(&self.path);
    }
}

const MAIN_CANDIDATES: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

fn quiet_output() -> ProductLines {
    ProductLines::discarding()
}

/// 在宿主目录上建一个库、写前 written 块（seed_S1、2¹⁶ 状态）。
fn host_library(directory: &Path, candidate: Candidate, written: u64) -> PathBuf {
    let library = directory.join(format!("library-{}", candidate.label()));
    let mut store = open_store_for_writing(candidate, WriteSettings::registered_for(candidate), &library).expect("宿主目录上打得开新库");
    let fingerprint = input_fingerprint();
    for block_index in 0..written {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        store.put_block(&key, &value).expect("宿主目录上写得进");
    }
    library
}

fn full_plan(block_count: u64) -> VerificationPlan {
    VerificationPlan { states_per_block: PRIMARY_STATES_PER_BLOCK, block_count, confirmed_end: block_count, started_end: block_count }
}

#[test]
fn every_registered_anchor_matches_the_independent_script() {
    let mut output = quiet_output();
    assert_eq!(run_anchors(&mut output), 0, "登记 7.2 与补 7 的锚点应当全对（补 13 锚点脚本的原样输出）");
}

#[test]
fn sha256_matches_the_standard_test_vectors() {
    assert_eq!(sha256_hexadecimal(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(sha256_hexadecimal(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    let fifty_six_bytes = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    assert_eq!(sha256_hexadecimal(fifty_six_bytes), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
}

#[test]
fn marker_round_trips_and_rejects_a_flipped_byte() {
    let marker = Marker { kind: MarkerKind::Begin, sequence: 12_001, block_index: Some(23_999), arm_index: 17 };
    let sector = encode_marker(&marker);
    assert_eq!(decode_marker(&sector), MarkerDecoding::Valid(marker));
    let start = Marker { kind: MarkerKind::Start, sequence: 0, block_index: None, arm_index: 0 };
    assert_eq!(decode_marker(&encode_marker(&start)), MarkerDecoding::Valid(start));
    let mut flipped = sector;
    flipped[9] ^= 1;
    assert_eq!(decode_marker(&flipped), MarkerDecoding::ChecksumMismatch);
    assert_eq!(decode_marker(&[0u8; SECTOR_BYTES]), MarkerDecoding::NotAMarker);
}

#[test]
fn parser_stops_at_the_declared_entry_count_when_a_stale_entry_follows() {
    let mut entries = synthetic_anchor_log_entries();
    entries.push((20, 1, 0, vec![0x66; 512]));
    let log = LogBytes::InMemory(synthetic_log_bytes(&entries, 6));
    let declared = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
    assert_eq!(declared.entries.len(), 6, "系统配置声明 6 个条目，第 7 个不许读进来（B8b）");
    assert_eq!(declared.end_offset_in_log, 8_192, "截到第 6 个条目的末尾");
    let until_zero = parse_device_log(&log, DeclaredCountHandling::ReadUntilZeroHeader).expect("解析得了");
    assert_eq!(until_zero.entries.len(), 7);
}

#[test]
fn parser_rejects_flush_with_data_and_unknown_flags() {
    let with_data = LogBytes::InMemory(synthetic_log_bytes(&[(0, 1, LOG_FLUSH_FLAG, vec![0x11; 512])], 1));
    assert!(matches!(parse_device_log(&with_data, DeclaredCountHandling::StopAtDeclared), Err(DeviceLogError::UnsupportedFlagCombination { index: 0, .. })));
    let marked = LogBytes::InMemory(synthetic_log_bytes(&[(0, 0, LOG_MARK_FLAG, Vec::new())], 1));
    assert!(matches!(parse_device_log(&marked, DeclaredCountHandling::StopAtDeclared), Err(DeviceLogError::UnknownFlags { index: 0, .. })));
}

#[test]
fn replay_images_match_the_registered_synthetic_log_digests() {
    let log = synthetic_log_bytes(&synthetic_anchor_log_entries(), 6);
    assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::DropUnflushed).expect("重放得了"), "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0");
    assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::KeepUnflushed).expect("重放得了"), "6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510");
    assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::RandomSubset).expect("重放得了"), "df9020850109e36d3b6d34da043be5a0fb016ffd4004372c52dac115009c1d65");
    assert_eq!(
        synthetic_anchor_image_digest(log, 6, ReplayVariant::RandomSubsetWithTear).expect("重放得了"),
        "6467f85ca3d8eefcb7eac7ea3cd4978d4cbb92d6b66b842b68a19adb6d46a763",
        "U-tear 要把留下的那个多扇区写截短（补 9 M19）"
    );
}

#[test]
fn subset_generator_matches_the_registered_keep_patterns() {
    let first_subset = unflushed_subset(0, 0, &[8, 1, 16, 8, 1, 1, 128, 8, 2, 1]);
    assert_eq!(bit_string(&first_subset.kept), "1000000100", "补 7 B3");
    assert_eq!(first_subset.torn, Some(TornWrite { position_in_unflushed: 7, sectors_kept: 1 }));
    let write_cache_model_subset = unflushed_subset(9, 0, &[1u64; 100]);
    assert_eq!(write_cache_model_subset.kept.iter().filter(|&&is_kept| !is_kept).count(), 47, "PC4-C 的 U-rand 丢恰好 47 个（补 7 B4）");
    assert_eq!(write_cache_model_subset.torn, None, "单扇区写不撕");
}

#[test]
fn write_cache_model_on_the_synthetic_log_gives_the_registered_counts() {
    let log = LogBytes::InMemory(synthetic_write_cache_model_log());
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
    let judgement = judge_write_cache_model(&log, &device_log).expect("判得了");
    assert_eq!(judgement.point, 201, "k = 第 200 个写条目下标 + 1（中间夹一个 FLUSH）");
    assert_eq!((judgement.entries_between_batches, judgement.flushes_between_batches), (1, 1));
    assert_eq!(
        judgement.lost_by_variant,
        vec![
            (ReplayVariant::DropUnflushed, 0, 100),
            (ReplayVariant::KeepUnflushed, 0, 0),
            (ReplayVariant::RandomSubset, 0, 47),
            (ReplayVariant::RandomSubsetWithTear, 0, 47),
        ],
        "前 100 个四种取法都不丢；后 100 个 U-drop 丢 100（M16）、U-keep 丢 0、U-rand 丢 47；FLUSH 之前的写不丢（M17）"
    );
    assert!(judgement.registered_counts_hold);
}

#[test]
fn forced_unit_access_write_after_the_last_flush_survives_drop_unflushed() {
    let entries = vec![
        LogEntry::Write { first_sector: 0, sector_count: 1, is_forced_unit_access: false, data_offset_in_log: 1_024 },
        LogEntry::Flush,
        LogEntry::Write { first_sector: 1, sector_count: 1, is_forced_unit_access: true, data_offset_in_log: 2_048 },
        LogEntry::Write { first_sector: 2, sector_count: 1, is_forced_unit_access: false, data_offset_in_log: 3_072 },
    ];
    let selection = select_replayed_writes(&entries, 4, ReplayVariant::DropUnflushed, 0, 0);
    let sectors: Vec<u64> = selection.writes.iter().map(|write| write.first_sector).collect();
    assert_eq!(sectors, vec![0, 1], "FUA 写收到即持久，只丢不带 FUA 的那一个");
    assert_eq!(selection.unflushed_entries, vec![3]);
    assert_eq!(unflushed_write_count(&entries, 4), 1);
    assert_eq!(unflushed_write_count(&entries, 2), 0);
}

/// 合成一份写盘开机的设备侧日志：S、B0、C0、B1、C1、B2、（断电点落在这里）、C2、E，全部落在 vda2 的第 c 个扇区。
fn synthetic_marker_log(arm_index: u32, block_count: u64) -> Vec<u8> {
    let mut entries = Vec::new();
    let mut sequence = 0u64;
    let mut push = |entries: &mut Vec<(u64, u64, u64, Vec<u8>)>, kind: MarkerKind, block_index: Option<u64>| {
        let marker = Marker { kind, sequence, block_index, arm_index };
        entries.push((marker_partition_start_sector() + sequence, 1, 0, encode_marker(&marker).to_vec()));
        sequence += 1;
    };
    push(&mut entries, MarkerKind::Start, None);
    for block_index in 0..block_count {
        push(&mut entries, MarkerKind::Begin, Some(block_index));
        entries.push((FIRST_PARTITION_START_SECTOR + 8 * block_index, 8, 0, vec![0xAB; 8 * SECTOR_BYTES]));
        entries.push((0, 0, LOG_FLUSH_FLAG, Vec::new()));
        push(&mut entries, MarkerKind::Confirm, Some(block_index));
    }
    push(&mut entries, MarkerKind::End, None);
    let declared = u64::try_from(entries.len()).expect("装得进 u64");
    synthetic_log_bytes(&entries, declared)
}

#[test]
fn in_flight_block_between_its_markers_is_not_counted_as_confirmed() {
    let log = LogBytes::InMemory(synthetic_marker_log(3, 3));
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
    let scan = scan_markers(&log, &device_log).expect("扫得了");
    let audit = audit_markers(&scan, 3, 3);
    assert!(audit.problems.is_empty(), "合成的标记应当过 V22：{:?}", audit.problems);
    let windows = audit.windows.expect("有窗口");
    assert_eq!(windows.begin_entries, vec![1, 5, 9]);
    assert_eq!(windows.confirm_entries, vec![4, 8, 12]);
    let point = windows.begin_entries[2] + 1;
    assert_eq!(block_classes_at(&windows, point), (2, 3), "第 2 块在 (B_2, C_2] 里：已确认 2 块、已开始 3 块（M18）");
    let directory = TestDirectory::new("in-flight");
    let library = host_library(&directory.path, Candidate::HardenedFile, 2);
    let (confirmed_end, started_end) = block_classes_at(&windows, point);
    let plan = VerificationPlan { states_per_block: PRIMARY_STATES_PER_BLOCK, block_count: 3, confirmed_end, started_end };
    let report = verify_library(Candidate::HardenedFile, &library, &plan);
    assert_eq!(report.lost_confirmed.len(), 0, "在途而读不到不算丢块");
    assert!(report.phantom_keys.is_empty() && report.silently_corrupted.is_empty());
}

#[test]
fn marker_audit_reports_a_missing_begin_and_a_count_mismatch() {
    let log_bytes = synthetic_marker_log(0, 2);
    let log = LogBytes::InMemory(log_bytes);
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
    let mut scan = scan_markers(&log, &device_log).expect("扫得了");
    let wrong_count = audit_markers(&scan, 0, 5);
    assert!(wrong_count.windows.is_none());
    assert!(wrong_count.problems.iter().any(|problem| problem.starts_with("confirm_markers=2")));
    scan.markers.remove(1);
    let renumbered: Vec<(usize, Marker)> = scan
        .markers
        .iter()
        .enumerate()
        .map(|(position, (entry, marker))| (*entry, Marker { sequence: u64::try_from(position).expect("小"), ..*marker }))
        .collect();
    scan.markers = renumbered;
    let missing_begin = audit_markers(&scan, 0, 2);
    assert!(missing_begin.windows.is_none(), "C_0 前面没有 B_0 要判 V22");
}

#[test]
fn write_loop_emits_the_confirm_marker_only_after_the_write_call_returns() {
    let directory = TestDirectory::new("event-order");
    let library = directory.path.join("library");
    let mut store = open_store_for_writing(Candidate::HardenedFile, WriteSettings::registered_for(Candidate::HardenedFile), &library).expect("打得开");
    let mut sink = MarkerSink::recording(3);
    let confirmed = write_blocks_with_markers(&mut store, &mut sink, 3, SMALL_STATES_PER_BLOCK).expect("写得进");
    assert_eq!(confirmed, 3);
    let MarkerSink::Recording { events, .. } = sink else { panic!("是记录用的发射器") };
    let described: Vec<String> = events
        .iter()
        .map(|event| match event {
            WriteLoopEvent::Marker(marker) => format!("{:?}{}", marker.kind, marker.block_index.map_or_else(String::new, |block| block.to_string())),
            WriteLoopEvent::WriteCallStarted { block_index } => format!("start{block_index}"),
            WriteLoopEvent::WriteCallReturned { block_index } => format!("return{block_index}"),
        })
        .collect();
    assert_eq!(
        described.join(" "),
        "Start Begin0 start0 return0 Confirm0 Begin1 start1 return1 Confirm1 Begin2 start2 return2 Confirm2 End",
        "C 标记在写入调用返回之后（补 9 M21）"
    );
    let sequences: Vec<u64> = events
        .iter()
        .filter_map(|event| match event {
            WriteLoopEvent::Marker(marker) => Some(marker.sequence),
            WriteLoopEvent::WriteCallStarted { .. } | WriteLoopEvent::WriteCallReturned { .. } => None,
        })
        .collect();
    assert_eq!(sequences, (0..8).collect::<Vec<u64>>(), "c 从 0 起逐个加 1");
}

#[test]
fn phantom_key_positive_control_finds_exactly_one_phantom_on_every_candidate() {
    let directory = TestDirectory::new("phantom");
    for candidate in MAIN_CANDIDATES {
        let library = host_library(&directory.path, candidate, 16);
        apply_library_damage(candidate, &library, LibraryDamage::PhantomKey, PRIMARY_STATES_PER_BLOCK).expect("加得进计划外的键");
        let report = verify_library(candidate, &library, &full_plan(16));
        assert_eq!(report.phantom_keys.len(), 1, "{} 上 PC4-P 的 Q4e 恰好 1（补 9 M22）", candidate.label());
        assert_eq!(report.lost_confirmed.len() + report.silently_corrupted.len() + report.confirmed_read_errors.len(), 0);
        assert!(!report.open_failed);
    }
}

#[test]
fn silent_corruption_positive_control_finds_block_seven_on_every_candidate() {
    let directory = TestDirectory::new("silent");
    for candidate in MAIN_CANDIDATES {
        let library = host_library(&directory.path, candidate, 16);
        apply_library_damage(candidate, &library, LibraryDamage::SilentCorruption, PRIMARY_STATES_PER_BLOCK).expect("改得了第 7 块");
        let report = verify_library(candidate, &library, &full_plan(16));
        assert_eq!(report.silently_corrupted, vec![7], "{} 上 PC4-S 的 Q4b 恰好 1、在第 7 块（补 9 M23）", candidate.label());
        assert_eq!(report.lost_confirmed.len() + report.phantom_keys.len() + report.confirmed_read_errors.len(), 0);
    }
}

#[test]
fn open_breaking_positive_control_fails_to_open_on_every_candidate() {
    let directory = TestDirectory::new("open-breaking");
    for candidate in MAIN_CANDIDATES {
        let library = host_library(&directory.path, candidate, 16);
        apply_library_damage(candidate, &library, LibraryDamage::OpenBreaking, PRIMARY_STATES_PER_BLOCK).expect("毁得了打开要用的那一处");
        let report = verify_library(candidate, &library, &full_plan(16));
        assert!(report.open_failed, "{} 上 PC4-O 的 Q4f 恰好 1（补 9 M24）", candidate.label());
        let repair_expectation = if candidate == Candidate::RocksDatabase { report.opened_after_repair.is_some() } else { report.opened_after_repair.is_none() };
        assert!(repair_expectation, "只有 K 另报修复后能不能开");
    }
}

#[test]
fn untouched_library_verifies_clean_on_every_candidate() {
    let directory = TestDirectory::new("clean");
    for candidate in MAIN_CANDIDATES {
        let library = host_library(&directory.path, candidate, 16);
        let report = verify_library(candidate, &library, &full_plan(16));
        assert_eq!(
            (report.lost_confirmed.len(), report.silently_corrupted.len(), report.phantom_keys.len(), report.open_failed, report.enumerated_key_count),
            (0, 0, 0, false, 16),
            "{} 没动过的库逐块核全对、枚举出恰好 16 个键",
            candidate.label()
        );
    }
}

#[test]
fn every_control_form_differs_from_its_correct_form_in_exactly_the_registered_setting() {
    for arm in ALL_POWER_CUT_ARMS {
        let Some(settings) = arm.write_settings() else {
            assert_eq!(arm, PowerCutArm::WriteCacheModel, "只有 PC4-C 没有候选");
            continue;
        };
        let correct = arm.correct_form().write_settings().expect("正确形态有写入设定");
        let differing = settings.differing_setting_names(correct);
        match arm.registered_control_difference() {
            Some(registered) => assert_eq!(differing, vec![registered], "{} 与 {} 应当恰好差「{registered}」（补 9 M26）", arm.label(), arm.correct_form().label()),
            None => assert!(differing.is_empty(), "{} 不是对照形态，写入设定应当与正确形态相同", arm.label()),
        }
    }
    assert_eq!(PowerCutArm::RocksDatabaseWithoutSync.write_settings().map(|settings| settings.rocksdb_write_sync), Some(RocksDatabaseWriteSync::NoSync));
}

#[test]
fn bucket_pools_split_windows_and_gaps_on_a_synthetic_marker_layout() {
    let windows = CommitWindows { start_entry: 10, end_entry: 100, begin_entries: vec![12, 20, 60], confirm_entries: vec![15, 23, 90] };
    let pools = bucket_pools(&windows);
    assert_eq!(pools.median_window_entries, 3);
    assert_eq!(pools.long_blocks, vec![2], "30 > 4 × 3 算长窗");
    assert_eq!(pools.normal_commit_points, vec![13, 14, 15, 21, 22, 23]);
    assert_eq!(pools.long_commit_points, (61..=90).collect::<Vec<usize>>());
    let expected_between: Vec<usize> = (11..=12).chain(16..=20).chain(24..=60).chain(91..=100).collect();
    assert_eq!(pools.between_commit_points, expected_between);
    let all_points = pools.normal_commit_points.len() + pools.long_commit_points.len() + pools.between_commit_points.len();
    assert_eq!(all_points, 100 - 10, "三个桶合起来恰好是 (S, E] 的全部边界");
}

#[test]
fn commit_window_boundaries_keep_both_ends_and_cap_at_sixty_four() {
    assert_eq!(commit_window_boundaries(5, 9), vec![6, 7, 8, 9]);
    assert_eq!(commit_window_boundaries(0, 64).len(), 64);
    let sixty_five = commit_window_boundaries(0, 65);
    assert_eq!((sixty_five.len(), sixty_five[0], sixty_five[63]), (64, 1, 65));
    let thousand = commit_window_boundaries(100, 1_100);
    assert_eq!((thousand.len(), thousand[0], thousand[63]), (64, 101, 1_100));
    assert!(thousand.windows(2).all(|pair| pair[0] < pair[1]), "等距点严格递增、不重复");
    let largest_subset_index = COMMIT_SUBSET_INDEX_BASE + COMMIT_SUBSET_INDEX_STRIDE * 7 + 63;
    assert_eq!(largest_subset_index, 10_763, "C-commit 的生成器下标装得进 (a << 32) | i 的低 32 位");
}

#[test]
fn main_arm_plan_has_the_registered_item_counts() {
    let begin_entries: Vec<usize> = (0..6_000).map(|block| 100 + 10 * block).collect();
    let confirm_entries: Vec<usize> = (0..6_000).map(|block| 100 + 10 * block + if block % 100 == 7 { 60 } else { 4 }).collect();
    let windows = CommitWindows { start_entry: 90, end_entry: 60_200, begin_entries, confirm_entries };
    let pools = bucket_pools(&windows);
    assert_eq!(pools.long_blocks.len(), 60);
    let full_replay_point = 60_250;
    let items = verification_items(PowerCutArm::RedbQuickRepair, &windows, &pools, full_replay_point);
    let stratified = items.iter().filter(|item| item.group.labels().0 == "C-strat").count();
    let commit_sweep = items.iter().filter(|item| item.group.labels().0 == "C-commit").count();
    let damage: Vec<(usize, u64, u64)> = items
        .iter()
        .filter(|item| item.damage != LibraryDamage::Untouched)
        .map(|item| (item.point, item.confirmed_end, item.started_end))
        .collect();
    assert_eq!(stratified, 200 * 4, "C-strat 200 点 × 四种取法");
    assert_eq!(commit_sweep, (6 * 4 + 2 * 60) * 2, "6 个普通窗各 4 个边界、2 个长窗各 60 个边界，各核 U-drop 与 U-rand");
    assert_eq!(damage, vec![(60_250, 6_000, 6_000); 3], "PC4-S/P/O 在全部条目重放出的盘面上做，块全都已确认");
    let small_items = verification_items(PowerCutArm::RedbQuickRepairSmallBlocks, &windows, &pools, full_replay_point);
    assert_eq!(small_items.len(), 100 * 2, "S4-bs10：100 点 × U-drop、U-rand");
    let control_items = verification_items(PowerCutArm::RocksDatabaseWithoutSync, &windows, &pools, full_replay_point);
    assert_eq!(control_items.len(), 20);
    assert!(control_items.iter().all(|item| item.variant == ReplayVariant::KeepUnflushed), "PC4-L 核 U-keep");
    let barrier_items = verification_items(PowerCutArm::HardenedFileWithoutBarrier, &windows, &pools, full_replay_point);
    assert!(barrier_items.iter().all(|item| item.variant == ReplayVariant::DropUnflushed), "PC4-NB 核 U-drop");
    let parsed: Vec<VerifyItem> = items.iter().map(|item| parse_plan_item(&item.plan_line()).expect("计划行读得回来")).collect();
    assert_eq!(parsed, items, "计划行写出去再读回来逐项相同");
}

#[test]
fn planned_boots_follow_the_registered_order() {
    let boots = planned_write_boots(&[PowerCutArm::RedbQuickRepair, PowerCutArm::HardenedFile]);
    let labels: Vec<&str> = boots.iter().map(|boot| boot.arm.label()).collect();
    assert_eq!(labels, vec!["PC4-C", "R1-None", "R1-NB", "F-nofsync", "F-NB", "F-nodirsync", "R1", "F", "R1-bs10", "F-bs10"]);
    let blocks: Vec<u64> = boots.iter().map(|boot| boot.block_count).collect();
    assert_eq!(blocks, vec![200, 1_000, 1_000, 1_000, 1_000, 1_000, 6_000, 6_000, 6_000, 6_000]);
}

#[test]
fn role_arguments_skip_the_device_path_that_vm_bench_prepends() {
    let arguments: Vec<String> = ["/dev/vda", "--role", "s4-write-guest", "--arm", "12", "--dry-run"].iter().map(|text| text.to_string()).collect();
    let parsed = RoleArguments::parse(&arguments);
    assert_eq!(parsed.text("role"), Some("s4-write-guest"));
    assert_eq!(parsed.number("arm"), Some(12));
    assert!(parsed.flag("dry-run"));
    assert_eq!(tagged_guest_line("3-K-write0", "name=write_guest arm=K confirmed=6000"), "name=write_guest boot=3-K-write0 arm=K confirmed=6000");
}

/// `--arms` 不给就是全部四臂；给了取写的子集并标缩过，四臂都写上不标；写错或空的拒。
#[test]
fn drive_without_arms_selects_all_four_main_arms_and_a_subset_is_marked_reduced() {
    let every_arm = selected_main_arms(None).expect("不给 --arms 就是全部四臂");
    assert_eq!(every_arm.iter().map(|arm| arm.label()).collect::<Vec<_>>(), vec!["R1", "R0", "K", "F"]);
    assert_eq!(reduced_scale_note(&every_arm), "");
    let subset = selected_main_arms(Some("R1,K")).expect("R1,K 是合法子集");
    assert_eq!(subset.len(), 2);
    assert_eq!(reduced_scale_note(&subset), " reduced=yes registered_arms=R1,R0,K,F not_official_product=yes");
    assert_eq!(selected_main_arms(Some("")).map(|arms| arms.len()), None);
    assert_eq!(selected_main_arms(Some("R1,X9")).map(|arms| arms.len()), None);
}
