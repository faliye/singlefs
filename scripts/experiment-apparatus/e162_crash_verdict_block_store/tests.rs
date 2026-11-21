// ============================== 单测 ==============================

use super::*;

/// 单测里起子进程角色：测试二进制带着这个环境变量、只跑 `child_process_role_entry` 一个测试。
pub(super) const CHILD_ROLE_ENVIRONMENT: &str = "E162_TEST_CHILD_ROLE";

#[test]
fn child_process_role_entry() {
    let Ok(joined) = std::env::var(CHILD_ROLE_ENVIRONMENT) else { return };
    let arguments: Vec<String> = joined.split('\u{1f}').map(str::to_string).collect();
    let remaining = &arguments[1..];
    let exit_code = match arguments[0].as_str() {
        "writer" => run_writer_role(remaining),
        "verify" => run_verify_role(remaining),
        "read" => run_read_role(remaining),
        other => panic!("认不出的子进程角色 {other}"),
    };
    std::process::exit(exit_code);
}

/// 每个单测一个自己的临时目录，测完删掉。
struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(name: &str) -> TestDirectory {
        let path = std::env::temp_dir().join(format!("e162-test-{}-{name}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).expect("删得了上一次的临时目录");
        }
        fs::create_dir_all(&path).expect("建得了临时目录");
        TestDirectory { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _cleanup_outcome = fs::remove_dir_all(&self.path);
    }
}

const ALL_CANDIDATES: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

/// 碰盘的单测一个一个跑：并行跑时别的单测的 fsync 挤进计时区间（PC-rate、H2 两条会因此误红）。
static DEVICE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn device_lock() -> std::sync::MutexGuard<'static, ()> {
    DEVICE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn seeds_equal_the_registered_bitwise_or_with_the_prefix() {
    assert_eq!(SEED_CRASH_BLOCKS, 0xE162_0000_0000_0001);
    assert_eq!(SEED_THROUGHPUT_BLOCKS, 0xE162_0000_0000_0002);
    assert_eq!(SEED_KILL, 0xE162_0000_0000_0003);
    assert_eq!(SEED_READ, 0xE162_0000_0000_0005);
    assert_eq!(SEED_ORDER, 0xE162_0000_0000_0006);
    assert_eq!(SEED_FINGERPRINT, 0xE162_0000_0000_0007);
    assert_eq!(SEED_LARGE, 0xE162_0000_0000_0008);
}

#[test]
fn splitmix_and_crc_match_registered_anchor_values() {
    let mut generator = SplitMix64::new(0);
    assert_eq!(generator.next_output(), 0xe220_a839_7b1d_cdaf);
    assert_eq!(generator.next_output(), 0x6e78_9e6a_a1b9_65f4);
    assert_eq!(generator.next_output(), 0x06c4_5d18_8009_454f);
    assert_eq!(crc32c(&[b"123456789"]), 0xe306_9283);
    assert_eq!(crc32c(&[b"1234", b"56789"]), 0xe306_9283, "分段喂与整段喂同值");
}

#[test]
fn random_block_values_match_registered_anchor_values() {
    let first = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
    assert_eq!(first.len(), 65_536);
    assert_eq!(hexadecimal(&first[..16]), "76020f1b59444d587a6607195c0f8fed");
    assert_eq!(first.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 8_351_809);
    assert_eq!(crc32c(&[&first]), 0x08ee_fd5c);
    let later = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 12_345, PRIMARY_STATES_PER_BLOCK);
    assert_eq!(hexadecimal(&later[..16]), "b3c1f0df0c77c9e98c9df5b21d0395bf");
    assert_eq!(later.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 8_321_879);
    assert_eq!(crc32c(&[&later]), 0x565d_4801);
    let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
    assert_eq!(small.len(), 1_024);
    assert_eq!(small.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 131_533);
}

#[test]
fn sparse_block_values_match_registered_anchor_values() {
    let first = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
    assert_eq!(first.iter().filter(|&&byte| byte != 0).count(), 267);
    assert_eq!(first.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 32_629);
    let later = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, 12_345, PRIMARY_STATES_PER_BLOCK);
    assert_eq!(later.iter().filter(|&&byte| byte != 0).count(), 245);
    assert_eq!(later.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 28_761);
}

#[test]
fn fingerprint_and_interleaved_keys_match_registered_anchor_values() {
    let fingerprint = input_fingerprint();
    assert_eq!(hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
    for (block_index, expected) in [
        (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
        (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
        (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
    ] {
        assert_eq!(hexadecimal(&interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint)), expected);
    }
    assert_eq!(
        hexadecimal(&interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint)),
        "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac"
    );
}

#[test]
fn permutation_and_kill_plan_match_registered_anchor_values() {
    let permutation = random_permutation(THROUGHPUT_BLOCK_COUNT, SEED_ORDER);
    assert_eq!(permutation[..6], [40_238, 12_140, 20_790, 5_207, 14_245, 28_683]);
    assert_eq!(permutation[99_999], 91_647);
    let plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
    let rendered: Vec<String> = plan.iter().take(5).map(kill_plan_row_text).collect();
    assert_eq!(
        rendered,
        [
            "0/write/12/0.3236817149915837",
            "1/write/51/0.5669778417801824",
            "2/write/50/0.6058528089881228",
            "3/write/59/0.14049280271075437",
            "4/write/7/0.8853027823689169",
        ]
    );
    assert_eq!(plan.iter().filter(|row| row.phase == KillPhase::DuringOpen).count(), 20);
    assert_eq!(plan.iter().map(|row| confirmations_of(row.phase)).sum::<u64>(), 6_178);
    assert_eq!(plan[9].phase, KillPhase::DuringOpen);
}

#[test]
fn block_classification_follows_the_registered_table() {
    use BlockClass::{Confirmed, InFlight, NotStarted};
    use BlockVerdict::{ConfirmedReadError, Correct, LostConfirmed, Phantom, SilentlyCorrupted, UnconfirmedReadError};
    use ReadKind::{DifferentBytes, MatchingBytes, NotFound, ReadError};
    let expected = [
        (Confirmed, MatchingBytes, Correct),
        (Confirmed, DifferentBytes, SilentlyCorrupted),
        (Confirmed, NotFound, LostConfirmed),
        (Confirmed, ReadError, ConfirmedReadError),
        (InFlight, MatchingBytes, Correct),
        (InFlight, DifferentBytes, SilentlyCorrupted),
        (InFlight, NotFound, Correct),
        (InFlight, ReadError, UnconfirmedReadError),
        (NotStarted, MatchingBytes, Phantom),
        (NotStarted, DifferentBytes, Phantom),
        (NotStarted, NotFound, Correct),
        (NotStarted, ReadError, UnconfirmedReadError),
    ];
    for (class, read, verdict) in expected {
        assert_eq!(classify_block(class, read), verdict, "{class:?} × {read:?}");
    }
}

fn write_blocks(store: &mut BlockStore, seed: u64, blocks: std::ops::Range<u64>) {
    let fingerprint = input_fingerprint();
    for block_index in blocks {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, seed, block_index, PRIMARY_STATES_PER_BLOCK);
        store.put_blocks(&[(key, value)]).expect("写得进");
    }
}

fn plan_for(confirmed_end: u64, started_end: u64) -> VerificationPlan {
    VerificationPlan {
        seed: SEED_CRASH_BLOCKS,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        confirmed_end,
        started_end,
        checked_blocks: CheckedBlocks::Range { start: 0, end: started_end + NOT_STARTED_BLOCKS_READ_AFTER_STARTED },
    }
}

fn assert_clean(report: &VerificationReport, context: &str) {
    assert!(!report.open_failed, "{context}: 打不开 {}", report.open_error);
    assert!(report.lost_confirmed.is_empty(), "{context}: 丢块 {:?}", report.lost_confirmed);
    assert!(report.silently_corrupted.is_empty(), "{context}: 坏块 {:?}", report.silently_corrupted);
    assert!(report.confirmed_read_errors.is_empty(), "{context}: 已确认块读报错 {:?}", report.confirmed_read_errors);
    assert!(report.unconfirmed_read_errors.is_empty(), "{context}: 未确认块读报错 {:?} {}", report.unconfirmed_read_errors, report.first_read_error);
    assert!(report.phantom_keys.is_empty(), "{context}: 幽灵块 {:?}", report.phantom_keys);
}

/// M1：F 的登记写法在任何一步被打断（等同那一步之后被 SIGKILL，页缓存还在），终名下都不会留半截文件。
#[test]
fn file_registered_write_path_leaves_no_partial_file_at_any_abandoned_step() {
    let _device = device_lock();
    for steps_allowed in 0..24u64 {
        let directory = TestDirectory::new(&format!("abandon-{steps_allowed}"));
        let library = directory.path.join("library");
        let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
        write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..4);
        match &mut store.backend {
            StoreBackend::File(file_library) => file_library.remaining_steps = Some(steps_allowed),
            StoreBackend::Redb(_) | StoreBackend::RocksDatabase(_) => unreachable!("F 臂的后端是文件"),
        }
        let fingerprint = input_fingerprint();
        let key = interleaved_block_key(4, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 4, PRIMARY_STATES_PER_BLOCK);
        let confirmed_end = match store.put_blocks(&[(key, value)]) {
            Ok(()) => 5,
            Err(StoreError::AbandonedAtStep) => 4,
            Err(other) => panic!("第 {steps_allowed} 步：{other}"),
        };
        drop(store);
        let report = verify_library(Candidate::HardenedFile, &library, &plan_for(confirmed_end, 5));
        assert_clean(&report, &format!("打断在第 {steps_allowed} 步"));
    }
}

/// M5：半截文件（头加前 4 096 字节块值）在读路径上是读报错，不是读到字节。
#[test]
fn file_read_path_reports_a_half_written_file_as_a_read_error() {
    let _device = device_lock();
    let directory = TestDirectory::new("half-file");
    let library = directory.path.join("library");
    let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
    write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..3);
    drop(store);
    let fingerprint = input_fingerprint();
    let key = interleaved_block_key(3, PRIMARY_STATES_PER_BLOCK, &fingerprint);
    let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 3, PRIMARY_STATES_PER_BLOCK);
    let final_path = library.join(block_file_relative_path(&key).expect("32 字节"));
    fs::create_dir_all(final_path.parent().expect("有父目录")).expect("建得了段目录");
    let mut partial = block_file_header(&key, &value).to_vec();
    partial.extend_from_slice(&value[..DIRECT_WRITE_CHUNK_BYTES]);
    fs::write(&final_path, &partial).expect("写得了半截文件");
    let report = verify_library(Candidate::HardenedFile, &library, &plan_for(3, 4));
    assert_eq!(report.unconfirmed_read_errors, vec![3]);
    assert!(report.silently_corrupted.is_empty(), "半截文件被读成了字节：{:?}", report.silently_corrupted);
    assert!(report.lost_confirmed.is_empty());
}

/// M6：PC-S 在每条臂上恰好查出第 7 块。
#[test]
fn silent_corruption_control_finds_exactly_block_seven_on_every_arm() {
    let _device = device_lock();
    for candidate in ALL_CANDIDATES {
        let directory = TestDirectory::new(&format!("pc-s-{}", candidate.label()));
        let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::SilentCorruption);
        assert_eq!(report.silently_corrupted, vec![SILENT_CORRUPTION_BLOCK], "{}", candidate.label());
        assert!(all_zero_except(&report, "q1b"), "{}: {report:?}", candidate.label());
        assert_eq!(report.enumerated_key_count, 64, "{}", candidate.label());
    }
}

/// M8：PC-P 在每条臂上恰好查出一个计划外的键。
#[test]
fn phantom_key_control_finds_exactly_one_key_on_every_arm() {
    let _device = device_lock();
    for candidate in ALL_CANDIDATES {
        let directory = TestDirectory::new(&format!("pc-p-{}", candidate.label()));
        let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::PhantomKey);
        assert_eq!(report.phantom_keys.len(), 1, "{}", candidate.label());
        assert!(report.phantom_keys.iter().all(|key| key.starts_with("ffffffff00000000")), "{}", candidate.label());
        assert!(all_zero_except(&report, "q1e"), "{}: {report:?}", candidate.label());
    }
}

/// M9：PC-O 在每条臂上恰好记一次打不开。
#[test]
fn open_breaking_control_counts_exactly_one_open_failure_on_every_arm() {
    let _device = device_lock();
    for candidate in ALL_CANDIDATES {
        let directory = TestDirectory::new(&format!("pc-o-{}", candidate.label()));
        let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::OpenBreaking);
        assert!(report.open_failed, "{} 被毁之后还打得开", candidate.label());
        assert!(all_zero_except(&report, "q1f"), "{}: {report:?}", candidate.label());
    }
}

/// M7：在途的块没落下来是对的，不算丢块。
#[test]
fn in_flight_block_that_never_landed_is_not_a_lost_block() {
    let _device = device_lock();
    let directory = TestDirectory::new("in-flight");
    let library = directory.path.join("library");
    let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
    write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..10);
    drop(store);
    let report = verify_library(Candidate::HardenedFile, &library, &plan_for(10, 11));
    assert_clean(&report, "第 10 块在途、没落下来");
    assert_eq!(report.confirmed_checked, 10);
    assert_eq!(report.in_flight_checked, 1);
    assert_eq!(report.not_started_checked, 64);
}

/// 一次杀：写 20 块确认之后立刻杀，重开逐块核。
fn kill_after_twenty_confirmations(candidate: Candidate, write_path: WritePath, name: &str) -> KillCampaign {
    let directory = TestDirectory::new(name);
    let library = directory.path.join("library");
    let mut campaign = KillCampaign::new(name, candidate, write_path, library, SEED_CRASH_BLOCKS, 0);
    let row = KillPlanRow { cycle_index: 0, phase: KillPhase::DuringWrite { confirmations_before_kill: 20 }, delay_fraction: 0.0 };
    let line = campaign.run_cycle(&row, false);
    assert!(line.contains("confirmations_this_cycle=20"), "{name}: {line}");
    campaign
}

/// M2：redb 的登记写法（Durability::Immediate）被 SIGKILL 之后，已确认的块一块不少；R-None 丢块（PC-L 的形态）。
#[test]
fn redb_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
    let _device = device_lock();
    for candidate in [Candidate::RedbQuickRepair, Candidate::RedbDefault] {
        let registered = kill_after_twenty_confirmations(candidate, WritePath::Registered, &format!("kill-{}", candidate.label()));
        assert!(registered.cumulative_lost.is_empty(), "{} 丢了已确认的块 {:?}", candidate.label(), registered.cumulative_lost);
        assert_eq!(registered.open_failures, 0);
        let none = kill_after_twenty_confirmations(candidate, WritePath::RedbDurabilityNone, &format!("kill-none-{}", candidate.label()));
        assert!(!none.cumulative_lost.is_empty(), "{} 的 Durability::None 被杀之后一块都没丢，PC-L 看不见丢块", candidate.label());
    }
}

/// M4：RocksDB 的登记写法（同步写、WAL 开）被 SIGKILL 之后，已确认的块一块不少；K-noWAL 丢块。
#[test]
fn rocksdb_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
    let _device = device_lock();
    let registered = kill_after_twenty_confirmations(Candidate::RocksDatabase, WritePath::Registered, "kill-K");
    assert!(registered.cumulative_lost.is_empty(), "K 丢了已确认的块 {:?}", registered.cumulative_lost);
    assert_eq!(registered.open_failures, 0);
    let no_log = kill_after_twenty_confirmations(Candidate::RocksDatabase, WritePath::RocksDatabaseWithoutWriteAheadLog, "kill-K-noWAL");
    assert!(!no_log.cumulative_lost.is_empty(), "K-noWAL 被杀之后一块都没丢，PC-L 看不见丢块");
}

/// F 的登记写法被 SIGKILL 之后一块不少；F-lazyrename 丢块（PC-L 的形态）。
#[test]
fn file_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
    let _device = device_lock();
    let registered = kill_after_twenty_confirmations(Candidate::HardenedFile, WritePath::Registered, "kill-F");
    assert!(registered.cumulative_lost.is_empty(), "F 丢了已确认的块 {:?}", registered.cumulative_lost);
    let lazy = kill_after_twenty_confirmations(Candidate::HardenedFile, WritePath::FileLazyRename, "kill-F-lazy");
    assert!(
        !lazy.cumulative_lost.is_empty(),
        "F-lazyrename 第 17 次起的确认还没改名就被杀，应当丢块；confirmed_end={}",
        lazy.confirmed_end
    );
}

#[test]
fn verification_report_survives_the_line_encoding() {
    let report = VerificationReport {
        open_failed: true,
        open_error: "redb: bad magic number".to_string(),
        opened_after_repair: Some(false),
        open_microseconds: 1_234,
        confirmed_checked: 10,
        in_flight_checked: 1,
        not_started_checked: 64,
        lost_confirmed: vec![3, 4],
        silently_corrupted: vec![7],
        confirmed_read_errors: Vec::new(),
        unconfirmed_read_errors: vec![10],
        phantom_keys: ["ffff".to_string()].into_iter().collect(),
        enumerated_key_count: 12,
        enumerated_value_bytes: 786_432,
        enumeration_error: String::new(),
        first_read_error: "block=10:CRC-32C 不对".to_string(),
    };
    let decoded = decode_verification_report(&encode_verification_report(&report)).expect("读得回");
    assert_eq!(decoded.open_error, "redb:_bad_magic_number");
    assert_eq!(decoded.lost_confirmed, vec![3, 4]);
    assert_eq!(decoded.silently_corrupted, vec![7]);
    assert_eq!(decoded.unconfirmed_read_errors, vec![10]);
    assert_eq!(decoded.opened_after_repair, Some(false));
    assert_eq!(decoded.enumerated_value_bytes, 786_432);
    assert_eq!(decoded.phantom_keys.len(), 1);
}

#[test]
fn timing_summary_pins_rate_windows_and_backlog() {
    // 2 000 块，第 k 块（从 1 数）在 k × 1 000 µs 返回：1 000 块/秒，每窗 1 000 块、1 秒。
    let offsets: Vec<u64> = (1..=2_000u64).map(|block| block * 1_000).collect();
    let latencies = vec![1_000u64; 2_000];
    let lines = timing_summary_lines(&latencies, &offsets, 1_024);
    assert!(lines[0].contains("blocks=2000 "), "{}", lines[0]);
    assert!(lines[0].contains("elapsed_microseconds=2000000 "), "{}", lines[0]);
    assert!(lines[0].contains("blocks_per_second=1000.000 "), "{}", lines[0]);
    assert!(lines[0].contains("states_per_second=1024000.0 "), "{}", lines[0]);
    assert!(lines[0].contains("commit_median_microseconds=1000 "), "{}", lines[0]);
    // 按实测速率产出：第 k 块返回时已产出 k × 1 024 个状态、已消化 (k − 1) × 1 024，积压恒为 1 024。
    assert!(
        lines.iter().any(|line| line.starts_with("Q backlog_rate_percent_of_measured=100 maximum_backlog_states=1024 ")),
        "{lines:?}"
    );
    // 按实测速率的一半产出：库总比产出快，积压最大就是第一块返回之前攒的 512 个状态。
    assert!(
        lines.iter().any(|line| line.starts_with("Q backlog_rate_percent_of_measured=50 maximum_backlog_states=512 peak_at_block=1")),
        "{lines:?}"
    );
    assert!(lines.iter().any(|line| line == "W window=1 end_microseconds=2000000 states_per_second=1024000.0"), "{lines:?}");
    assert_eq!(lines.iter().filter(|line| line.starts_with("W ")).count(), 2);
}

#[test]
fn block_file_validation_rejects_each_kind_of_damage() {
    let fingerprint = input_fingerprint();
    let key = interleaved_block_key(5, PRIMARY_STATES_PER_BLOCK, &fingerprint);
    let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 5, PRIMARY_STATES_PER_BLOCK);
    let mut good = block_file_header(&key, &value).to_vec();
    good.extend_from_slice(&value);
    assert_eq!(good.len(), 65_584);
    assert_eq!(validate_block_file(&good, &key).expect("好文件读得出"), value);
    let other_key = interleaved_block_key(6, PRIMARY_STATES_PER_BLOCK, &fingerprint);
    assert!(validate_block_file(&good, &other_key).is_err(), "头里的键不对");
    assert!(validate_block_file(&good[..47], &key).is_err(), "短于文件头");
    assert!(validate_block_file(&good[..good.len() - 1], &key).is_err(), "少一个字节");
    for damaged_offset in [0usize, 4, 40, 44, 48, 65_583] {
        let mut damaged = good.clone();
        damaged[damaged_offset] ^= 0x01;
        assert!(validate_block_file(&damaged, &key).is_err(), "第 {damaged_offset} 字节翻了一位");
    }
}

#[test]
fn write_rate_judgement_flips_only_below_the_requirement() {
    assert!(write_rate_flips(99.0, 100.0));
    assert!(!write_rate_flips(100.0, 100.0));
    assert!(!write_rate_flips(101.0, 100.0));
}


/// M3：每条臂的登记写法每块提交中位 ≥ 0.5 × 同一目录里裸「追加 64 KiB + fdatasync」的中位（H2）。
#[test]
// wallclock-assert: 断言两组各自实测的中位数之比（每块提交中位乘 2 不小于裸写追加加 fdatasync 的中位），两组分在不同时刻测，各受负载影响
fn every_registered_arm_commit_waits_for_the_device() {
    let _device = device_lock();
    let directory = TestDirectory::new("h2");
    let raw_median = median_of(&raw_append_fdatasync_probe(&directory.path));
    for candidate in ALL_CANDIDATES {
        let library = directory.path.join(format!("library-{}", candidate.label()));
        let mut store = open_store_for_writing(candidate, WritePath::Registered, &library).expect("建得了库");
        let fingerprint = input_fingerprint();
        let mut latencies = Vec::new();
        for block_index in 0..50u64 {
            let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
            let started = Instant::now();
            store.put_blocks(&[(key, value)]).expect("写得进");
            latencies.push(elapsed_microseconds(started));
        }
        let commit_median = median_of(&latencies);
        assert!(commit_median * 2 >= raw_median, "{}：每块提交中位 {commit_median} µs < 0.5 × L_raw {raw_median} µs", candidate.label());
    }
}

/// M11：计时区间包住提交调用：计时区间里多睡 5 ms，每块提交中位多出 ≥ 4.5 ms。
#[test]
fn commit_timing_includes_the_injected_five_millisecond_sleep() {
    let _device = device_lock();
    let directory = TestDirectory::new("pc-rate");
    let mut plain = calibration_settings(Candidate::HardenedFile, directory.path.join("plain"), SEED_THROUGHPUT_BLOCKS, RunTier::Registered);
    plain.raw_probe = false;
    plain.block_count = 30;
    let mut slept = plain.clone();
    slept.library = directory.path.join("slept");
    slept.sleep_microseconds = POSITIVE_CONTROL_SLEEP_MICROSECONDS;
    let plain_outcome = run_timed_write(&plain);
    let slept_outcome = run_timed_write(&slept);
    assert_eq!(timing_field(&plain_outcome, "blocks"), 30.0, "{}", plain_outcome.timing_line);
    let difference = timing_field(&slept_outcome, "commit_median_microseconds") - timing_field(&plain_outcome, "commit_median_microseconds");
    assert!(
        difference >= 4_500.0,
        "两遍中位数之差 {difference} µs；照常：{}；睡 5 ms：{}",
        plain_outcome.timing_line,
        slept_outcome.timing_line
    );
}

/// 不写档位参数就是登记档、产物判；写 `feasibility` 是缩域，核产物那一格写 `not_judged…`，不当正式产物。
#[test]
fn feasibility_tier_is_a_narrowed_run_whose_verdict_says_not_judged() {
    assert_eq!(RunTier::from_argument(None), Some(RunTier::Registered));
    assert_eq!(RunTier::from_argument(Some(&"feasibility".to_string())), Some(RunTier::Feasibility));
    assert_eq!(RunTier::from_argument(Some(&"unknown-tier".to_string())), None);
    assert_eq!(RunTier::Registered.judgement(), "judged_by_registration");
    assert!(RunTier::Feasibility.judgement().starts_with("not_judged"));
}
