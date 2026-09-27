//! 层 0 崩溃重放按双机分片（里程碑三第六项，`.claude/kb/milestone/03-third-txn.md` 第六节）：在第一个事务那条小流上
//! （mkfs → 取号 → 暖机 → A，按甲二展开，原地覆写取三态，54 个状态）拿真的账本核分片与 merge 的每一条。两台机器在这里是同一个进程里顺序跑的几趟，
//! 各用自己的进度目录；merge 那一趟的目录里只放拷过来的账本。
//! 名字里不带 layer0：它只跑甲二那几十个状态，平时跑得起。

mod common;

use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};

use common::{build_pool, file_content, geometry};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_harness::crash::{
    enumerate_layer0_in_state_slices, enumerate_layer0_in_state_slices_or_one_shard,
    layer0_state_count_with_torn_in_place_overwrites, quick_tier_expansion, writes_and_segments,
    CrashImage, Layer0EnumerationOutcome, Layer0ObserverCounts, Layer0Parallelism,
    Layer0SliceLength, Layer0Tally, Layer0WorkerThreadsSource, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::layer0_progress::{
    shard_ledger_path, Layer0ProgressFileAfterCompletion, Layer0ProgressFileNamePart,
    Layer0ProgressFileSettings, Layer0Resume, Layer0ResumeStart, Layer0ShardMerge,
    Layer0ShardOfShards, Layer0ShardRun, Layer0ToolchainIdentity,
    LAYER0_SHARD_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::segments::StepKind;

/// 第一条流按甲二展开的状态数（`first_transaction_step_seven_layer0.rs` 的 `QUICK_TIER_STATES`）。十段写数 `2,2,1,2,2,1,26,2,1,2`；
/// 系统配置槽写是原地覆写、各取三态（实审 B3a-2 第 4 条），其余写取两态：只有两次系统配置槽写的三段（取号、暖机第一次发布末尾、
/// A 之后）各 3² − 1 = 8，A 那一段（暖机末尾两次系统配置槽写 + A 的 24 次单元写，单元写只取全不落或全落）3² · 2 − 1 = 17，
/// 三段 journal 记录各 2² − 1 = 3，三段根槽写各 1，再加全部持久那一个：3 · 8 + 17 + 3 · 3 + 3 · 1 + 1 = 54
/// （补第三态之前两态口径是 3 · 3 + 7 + 3 · 3 + 3 · 1 + 1 = 29）。
const QUICK_TIER_STATES: u64 = 54;
/// 用例里账本与进度文件的流名。
const STREAM_NAME: &str = "first_transaction_stream_quick_tier";
/// 驱动脚本 `--selftest` 跑的那条用例的流名。
const SELFTEST_STREAM_NAME: &str = "sharded_selftest_first_transaction_stream_quick_tier";
/// 驱动脚本 `--selftest` 那条用例每片几个状态：54 个状态切成 14 片，分两片时每片 7 片、各起不止一个工作线程。
const SELFTEST_STATES_PER_SLICE: u64 = 4;

struct FirstStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    root_index: usize,
}

fn first_stream(tag: &str) -> FirstStream {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    FirstStream {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        root_index,
    }
}

/// 第一个事务那一版（实例 1、txg 3）带着给定的内容。
fn versions_with_content(content: Vec<u8>) -> Vec<PublishedVersion> {
    vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content,
    }]
}

/// 版本表的内容故意写错：每个读出文件的状态都违例，「第一处违例」才有得选（序号最小的那一处）。
fn versions_with_the_wrong_content() -> Vec<PublishedVersion> {
    versions_with_content(b"not the content the first transaction published".to_vec())
}

/// 每个用例一个自己的目录（先清空）。
fn fresh_directory(test_name: &str, role: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "singlefs-crash-enumeration-sharded-{}-{test_name}-{role}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("清空上一次留下的目录");
    }
    std::fs::create_dir_all(&directory).expect("建目录");
    directory
}

fn files_in(directory: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .expect("读目录")
        .map(|entry| entry.expect("目录项").path())
        .collect();
    files.sort();
    files
}

fn name_part(text: &str) -> Layer0ProgressFileNamePart {
    Layer0ProgressFileNamePart::new(text).expect("合法的文件名片段")
}

fn toolchain_of_this_test() -> Layer0ToolchainIdentity {
    Layer0ToolchainIdentity {
        rustc_version_lines: "rustc 1.0.0 (sharded test)\nbinary: rustc\ncommit-hash: sharded"
            .to_string(),
        cargo_version: "cargo 1.0.0 (sharded test)".to_string(),
        target_triple: "x86_64-unknown-linux-gnu".to_string(),
    }
}

fn shard(shard_index: u32, shard_count: u32) -> Layer0ShardOfShards {
    Layer0ShardOfShards::new(
        shard_index,
        NonZeroU32::new(shard_count).expect("片数不是 0"),
    )
    .expect("第几片小于片数")
}

fn states_per_slice_on(states_per_slice: u64, worker_threads: usize) -> Layer0Parallelism {
    Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("不是 0"),
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: Layer0SliceLength::StatesPerSlice(
            NonZeroU64::new(states_per_slice).expect("不是 0"),
        ),
    }
}

/// 一台机器跑一片的那些设置。
struct ShardRunSettings<'settings> {
    directory: &'settings Path,
    input_fingerprint: &'settings str,
    shard: Layer0ShardOfShards,
    toolchain: Layer0ToolchainIdentity,
    after_completion: Layer0ProgressFileAfterCompletion,
}

impl ShardRunSettings<'_> {
    fn resume(&self) -> Layer0Resume {
        Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
            progress: Layer0ProgressFileSettings {
                directory: self.directory.to_path_buf(),
                input_fingerprint: name_part(self.input_fingerprint),
                stream_name: name_part(STREAM_NAME),
                start: Layer0ResumeStart::ResumeFromTheProgressFile,
                after_completion: self.after_completion,
            },
            shard: self.shard,
            toolchain: self.toolchain.clone(),
        })
    }
}

fn merge_resume(directory: &Path, shard_count: u32) -> Layer0Resume {
    Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
        directory: directory.to_path_buf(),
        input_fingerprint: name_part("fingerprint0"),
        stream_name: name_part(STREAM_NAME),
        shard_count: NonZeroU32::new(shard_count).expect("片数不是 0"),
        toolchain: toolchain_of_this_test(),
    })
}

/// 跑一趟甲二：观察者把「读出文件 / 没有文件 / 走读失败」记进观察者计数（要随账本到 merge 的累计），另数这一趟它自己被调了几次。
fn run(
    stream: &FirstStream,
    versions: &[PublishedVersion],
    parallelism: Layer0Parallelism,
    resume: &Layer0Resume,
) -> (Layer0EnumerationOutcome, u64) {
    let mut observer_calls_in_this_run = 0u64;
    let outcome = enumerate_layer0_in_state_slices_or_one_shard(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.root_index,
        versions,
        &quick_tier_expansion,
        parallelism,
        Some(&mut |_image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut Layer0ObserverCounts| {
            observer_calls_in_this_run += 1;
            match report.outcome {
                RecoveryOutcome::FileRead { .. } => counts.add("file_read_states", 1),
                RecoveryOutcome::NoFile { .. } => counts.add("no_file_states", 1),
                RecoveryOutcome::Failed { .. } => counts.add("failed_states", 1),
            }
        }),
        resume,
    );
    (outcome, observer_calls_in_this_run)
}

fn whole_stream(outcome: Layer0EnumerationOutcome) -> Layer0Tally {
    match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => {
            panic!(
                "要整条流的计数，交回的是第 {} 片的账本",
                written.shard.text()
            )
        }
    }
}

/// 跑一片，交回它的账本路径与这一趟观察者被调的次数；核这一片只跑了归它的切片、账本在、进度文件删了。
fn run_one_shard(
    stream: &FirstStream,
    versions: &[PublishedVersion],
    parallelism: Layer0Parallelism,
    settings: &ShardRunSettings<'_>,
) -> (PathBuf, u64) {
    let (outcome, observer_calls) = run(stream, versions, parallelism, &settings.resume());
    let written = match outcome {
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => written,
        Layer0EnumerationOutcome::WholeStream(_tally) => {
            panic!("分片跑一片要交回这一片的账本，交回的是整条流的计数")
        }
    };
    assert_eq!(written.shard, settings.shard);
    assert!(
        written.ledger_path.is_file(),
        "账本写下了：{}",
        written.ledger_path.display()
    );
    assert_eq!(
        written.tally_of_this_shard.observed_states, written.tally_of_this_shard.states,
        "这一片的观察者（连续跑读回的片记的）看过归它的每一个状态"
    );
    assert!(
        observer_calls <= written.tally_of_this_shard.states,
        "这一趟观察者只看归这一片的状态：调了 {observer_calls} 次，这一片 {} 个状态",
        written.tally_of_this_shard.states
    );
    (written.ledger_path, observer_calls)
}

/// 用例钉死计数时打的那一行（照 `LAYER0B` 的写法把整份计数报全）：merge 之后这一行与不分片的逐字相同，才说明门禁读到的判定一样。
fn count_line(tally: &Layer0Tally) -> String {
    format!(
        "LAYER0_SHARDED states={} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} ignored_violations={} record_root_without_record={} record_claimed_state_missing_unit={} {} states_by_publish=[{}] observed_states={} observer_counts=[{}] first_violation={} first_ignored_violation={}",
        tally.states,
        tally.states == QUICK_TIER_STATES,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.ignored_violations,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_counts_by_invariant(),
        tally.states_by_publish_text(),
        tally.observed_states,
        tally
            .observer_counts
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect::<Vec<String>>()
            .join(","),
        tally.first_violation.as_deref().unwrap_or("none"),
        tally.first_ignored_violation.as_deref().unwrap_or("none")
    )
}

/// 跑 n 片（第 i 片起 i + 1 个工作线程，各用自己的进度目录），把 n 份账本拷进 merge 目录，交回 merge 目录与各片观察者被调的次数。
fn run_every_shard_and_gather_the_ledgers(
    stream: &FirstStream,
    versions: &[PublishedVersion],
    test_name: &str,
    shard_count: u32,
) -> (PathBuf, Vec<u64>) {
    let merge_directory = fresh_directory(test_name, "merge");
    let mut observer_calls_by_shard = Vec::new();
    for shard_index in 0..shard_count {
        let shard_directory = fresh_directory(test_name, &format!("shard-{shard_index}"));
        let (ledger_path, observer_calls) = run_one_shard(
            stream,
            versions,
            states_per_slice_on(1, usize::try_from(shard_index).expect("片号") + 1),
            &ShardRunSettings {
                directory: &shard_directory,
                input_fingerprint: "fingerprint0",
                shard: shard(shard_index, shard_count),
                toolchain: toolchain_of_this_test(),
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            },
        );
        assert_eq!(
            files_in(&shard_directory),
            vec![ledger_path.clone()],
            "这一片跑完只留账本，进度文件删了"
        );
        std::fs::copy(
            &ledger_path,
            merge_directory.join(ledger_path.file_name().expect("账本有文件名")),
        )
        .expect("把账本拷进 merge 目录");
        std::fs::remove_dir_all(&shard_directory).ok();
        observer_calls_by_shard.push(observer_calls);
    }
    (merge_directory, observer_calls_by_shard)
}

/// merge 要 panic：交回 panic 的那一句。
fn panic_message_of_merge(
    stream: &FirstStream,
    versions: &[PublishedVersion],
    parallelism: Layer0Parallelism,
    resume: &Layer0Resume,
) -> String {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(stream, versions, parallelism, resume)
    }));
    let payload = outcome.expect_err("merge 核不齐要 panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .expect("panic 带一句话")
}

/// 不分片跑一次；分 2 片、分 3 片各跑一次再 merge：三样的计数（连观察者计数、「第一处」）与用例会打的计数行逐项相同；
/// 各片的观察者合起来恰好看过每个状态一次；merge 那一趟不再调观察者。绝对值另钉：54 个状态，违例数等于读出文件的状态数且不为 0。
#[test]
fn two_and_three_shards_merge_into_the_same_tally_and_count_line_as_one_unsharded_run() {
    let stream = first_stream("sharded-merge");
    let versions = versions_with_the_wrong_content();
    let (unsharded_outcome, unsharded_calls) = run(
        &stream,
        &versions,
        states_per_slice_on(1, 4),
        &Layer0Resume::NoProgressFile,
    );
    let unsharded = whole_stream(unsharded_outcome);
    assert_eq!(unsharded.states, QUICK_TIER_STATES);
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &quick_tier_expansion
        ),
        QUICK_TIER_STATES,
        "甲二的状态数由展开方式另算一遍（原地覆写取三态，与枚举同一个口径）"
    );
    assert_eq!(unsharded_calls, QUICK_TIER_STATES);
    assert_eq!(unsharded.observed_states, QUICK_TIER_STATES);
    assert!(
        unsharded.violations > 0 && unsharded.violations == unsharded.file_read_states,
        "版本表内容写错：每个读出文件的状态都违例（违例 {}、读出文件 {}）",
        unsharded.violations,
        unsharded.file_read_states
    );
    assert!(unsharded.first_violation.is_some());
    assert_eq!(
        unsharded.observer_counts.get("file_read_states")
            + unsharded.observer_counts.get("no_file_states")
            + unsharded.observer_counts.get("failed_states"),
        QUICK_TIER_STATES,
        "观察者计数把每个状态记进恰好一格"
    );
    for shard_count in [2u32, 3] {
        let test_name = format!("merge-into-{shard_count}");
        let (merge_directory, observer_calls_by_shard) =
            run_every_shard_and_gather_the_ledgers(&stream, &versions, &test_name, shard_count);
        assert_eq!(
            observer_calls_by_shard.iter().sum::<u64>(),
            QUICK_TIER_STATES,
            "{shard_count} 片的观察者合起来恰好看过每个状态一次：{observer_calls_by_shard:?}"
        );
        assert!(
            observer_calls_by_shard.iter().all(|calls| *calls > 0),
            "每一片都分到了状态：{observer_calls_by_shard:?}"
        );
        let (merged_outcome, merge_calls) = run(
            &stream,
            &versions,
            states_per_slice_on(1, 2),
            &merge_resume(&merge_directory, shard_count),
        );
        let merged = whole_stream(merged_outcome);
        assert_eq!(merge_calls, 0, "merge 不枚举、不调观察者");
        assert_eq!(
            merged, unsharded,
            "分 {shard_count} 片再 merge：计数、观察者计数与每一处「第一处」都与不分片逐项相同"
        );
        assert_eq!(
            count_line(&merged),
            count_line(&unsharded),
            "分 {shard_count} 片再 merge：用例打的计数行逐字相同"
        );
        std::fs::remove_dir_all(&merge_directory).ok();
    }
}

/// 少一份账本：merge 停下，说清缺的是第几片、账本该在哪。
#[test]
fn merge_stops_and_names_the_shard_whose_ledger_is_missing() {
    let stream = first_stream("sharded-missing");
    let versions = versions_with_content(file_content());
    let (merge_directory, _) =
        run_every_shard_and_gather_the_ledgers(&stream, &versions, "missing", 3);
    let missing_ledger = shard_ledger_path(&merge_directory, &name_part(STREAM_NAME), shard(1, 3));
    std::fs::remove_file(&missing_ledger).expect("删掉第 1/3 片的账本");
    let message = panic_message_of_merge(
        &stream,
        &versions,
        states_per_slice_on(1, 2),
        &merge_resume(&merge_directory, 3),
    );
    assert!(
        message.contains("缺 1 份账本") && message.contains("第 1/3 片"),
        "{message}"
    );
    assert!(
        message.contains(&missing_ledger.display().to_string()),
        "{message}"
    );
    std::fs::remove_dir_all(&merge_directory).ok();
}

/// 第 1 片在另一份输入上跑（输入指纹不同）：merge 停下，说清是第 1 片的账本、`input_fingerprint` 那一处不同、两边各是什么。
#[test]
fn merge_stops_when_a_ledger_was_run_on_another_input_fingerprint() {
    let stream = first_stream("sharded-fingerprint");
    let versions = versions_with_content(file_content());
    let merge_directory = fresh_directory("fingerprint", "merge");
    for (shard_index, input_fingerprint) in [(0u32, "fingerprint0"), (1, "fingerprint1")] {
        let (ledger_path, _) = run_one_shard(
            &stream,
            &versions,
            states_per_slice_on(1, 2),
            &ShardRunSettings {
                directory: &merge_directory,
                input_fingerprint,
                shard: shard(shard_index, 2),
                toolchain: toolchain_of_this_test(),
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            },
        );
        assert!(ledger_path.starts_with(&merge_directory));
    }
    let message = panic_message_of_merge(
        &stream,
        &versions,
        states_per_slice_on(1, 2),
        &merge_resume(&merge_directory, 2),
    );
    assert!(
        message.contains("第 1/2 片的账本")
            && message
                .contains("input_fingerprint 账本写的 \"fingerprint1\"、这一趟是 \"fingerprint0\""),
        "{message}"
    );
    std::fs::remove_dir_all(&merge_directory).ok();
}

/// 两份账本同一片（第 0 片的账本拷到了第 1 片的名下）：merge 停下，说清第 1 片名下记的是第 0 片。
#[test]
fn merge_stops_when_two_ledgers_are_the_same_shard() {
    let stream = first_stream("sharded-same-shard");
    let versions = versions_with_content(file_content());
    let (merge_directory, _) =
        run_every_shard_and_gather_the_ledgers(&stream, &versions, "same-shard", 2);
    std::fs::copy(
        shard_ledger_path(&merge_directory, &name_part(STREAM_NAME), shard(0, 2)),
        shard_ledger_path(&merge_directory, &name_part(STREAM_NAME), shard(1, 2)),
    )
    .expect("第 0 片的账本盖到第 1 片名下");
    let message = panic_message_of_merge(
        &stream,
        &versions,
        states_per_slice_on(1, 2),
        &merge_resume(&merge_directory, 2),
    );
    assert!(
        message.contains("第 1/2 片的账本") && message.contains("记的是第 0 片"),
        "{message}"
    );
    std::fs::remove_dir_all(&merge_directory).ok();
}

/// 第 1 片按另一种切法跑（每片 2 个状态，这一趟每片 1 个）：merge 停下，说清是切片方案那几处不同。
#[test]
fn merge_stops_when_a_ledger_was_sliced_another_way() {
    let stream = first_stream("sharded-slicing");
    let versions = versions_with_content(file_content());
    let merge_directory = fresh_directory("slicing", "merge");
    for (shard_index, states_per_slice) in [(0u32, 1u64), (1, 2)] {
        run_one_shard(
            &stream,
            &versions,
            states_per_slice_on(states_per_slice, 2),
            &ShardRunSettings {
                directory: &merge_directory,
                input_fingerprint: "fingerprint0",
                shard: shard(shard_index, 2),
                toolchain: toolchain_of_this_test(),
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            },
        );
    }
    let message = panic_message_of_merge(
        &stream,
        &versions,
        states_per_slice_on(1, 2),
        &merge_resume(&merge_directory, 2),
    );
    // 每片 2 个状态切 ⌈54 / 2⌉ = 27 片，每片 1 个切 54 片。
    assert!(
        message.contains("第 1/2 片的账本")
            && message.contains(&format!(
                "slices 账本写的 \"{}\"、这一趟是 \"{QUICK_TIER_STATES}\"",
                QUICK_TIER_STATES.div_ceil(2)
            ))
            && message.contains("states_per_slice 账本写的 \"2\"、这一趟是 \"1\""),
        "{message}"
    );
    std::fs::remove_dir_all(&merge_directory).ok();
}

/// 第 1 片由另一套工具链编出来的测试二进制跑（rustc 版本不同）：merge 停下，说清是 rustc 那一处、两边各是什么。
#[test]
fn merge_stops_when_a_ledger_was_built_by_another_toolchain() {
    let stream = first_stream("sharded-toolchain");
    let versions = versions_with_content(file_content());
    let merge_directory = fresh_directory("toolchain", "merge");
    let mut other_toolchain = toolchain_of_this_test();
    other_toolchain.rustc_version_lines =
        "rustc 1.0.1 (other)\nbinary: rustc\ncommit-hash: other".to_string();
    for (shard_index, toolchain) in [(0u32, toolchain_of_this_test()), (1, other_toolchain)] {
        run_one_shard(
            &stream,
            &versions,
            states_per_slice_on(1, 2),
            &ShardRunSettings {
                directory: &merge_directory,
                input_fingerprint: "fingerprint0",
                shard: shard(shard_index, 2),
                toolchain,
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            },
        );
    }
    let message = panic_message_of_merge(
        &stream,
        &versions,
        states_per_slice_on(1, 2),
        &merge_resume(&merge_directory, 2),
    );
    assert!(
        message.contains("第 1/2 片的账本")
            && message.contains("rustc 账本写的 \"rustc 1.0.1 (other)"),
        "{message}"
    );
    std::fs::remove_dir_all(&merge_directory).ok();
}

/// 分片跑一片被杀之后续跑：进度文件名带 `shard-0-of-2`、计划哈希与单机的不同（两格互不相认）；截掉一半再跑，读回的片不再跑，
/// 写出的账本片行与一口气跑完的逐行相同。
#[test]
fn a_shard_cut_short_resumes_from_its_own_progress_file_to_the_same_ledger() {
    let stream = first_stream("sharded-resume");
    let versions = versions_with_content(file_content());
    let unsharded_directory = fresh_directory("resume", "unsharded");
    let unsharded_resume = Layer0Resume::KeepProgressFile(Layer0ProgressFileSettings {
        directory: unsharded_directory.clone(),
        input_fingerprint: name_part("fingerprint0"),
        stream_name: name_part(STREAM_NAME),
        start: Layer0ResumeStart::ResumeFromTheProgressFile,
        after_completion: Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
    });
    whole_stream(
        run(
            &stream,
            &versions,
            states_per_slice_on(1, 2),
            &unsharded_resume,
        )
        .0,
    );
    let [unsharded_progress_file] = files_in(&unsharded_directory)
        .try_into()
        .expect("单机那一趟恰好一个进度文件");

    let shard_directory = fresh_directory("resume", "shard");
    let settings = ShardRunSettings {
        directory: &shard_directory,
        input_fingerprint: "fingerprint0",
        shard: shard(0, 2),
        toolchain: toolchain_of_this_test(),
        after_completion: Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
    };
    let (ledger_path, uninterrupted_calls) =
        run_one_shard(&stream, &versions, states_per_slice_on(1, 2), &settings);
    let uninterrupted_ledger = std::fs::read_to_string(&ledger_path).expect("读账本");
    let shard_progress_file = files_in(&shard_directory)
        .into_iter()
        .find(|path| *path != ledger_path)
        .expect("留着的进度文件");
    let shard_progress_file_name = shard_progress_file
        .file_name()
        .expect("有文件名")
        .to_string_lossy()
        .to_string();
    assert!(
        shard_progress_file_name.ends_with("-shard-0-of-2.txt"),
        "{shard_progress_file_name}"
    );
    let plan_hash_of = |path: &Path| -> String {
        std::fs::read_to_string(path)
            .expect("读进度文件")
            .lines()
            .next()
            .expect("有文件头")
            .split(' ')
            .find_map(|field| field.strip_prefix("plan="))
            .expect("文件头有 plan=")
            .to_string()
    };
    assert_ne!(
        plan_hash_of(&shard_progress_file),
        plan_hash_of(&unsharded_progress_file),
        "分片的计划哈希带着这一片，与单机的不同"
    );

    let content = std::fs::read_to_string(&shard_progress_file).expect("读进度文件");
    let lines: Vec<&str> = content.lines().collect();
    let kept_slice_lines = 5;
    let mut cut = lines[..=kept_slice_lines].join("\n");
    cut.push('\n');
    let next_line = lines[kept_slice_lines + 1];
    cut.push_str(&next_line[..next_line.len() / 2]);
    std::fs::write(&shard_progress_file, cut).expect("写截断的进度文件");
    std::fs::remove_file(&ledger_path).expect("删掉头一趟的账本");

    let (resumed_ledger_path, resumed_calls) =
        run_one_shard(&stream, &versions, states_per_slice_on(1, 3), &settings);
    assert_eq!(resumed_ledger_path, ledger_path);
    assert_eq!(
        resumed_calls,
        uninterrupted_calls - u64::try_from(kept_slice_lines).expect("行数"),
        "读回的 5 片不再跑；末尾半行那一片重跑"
    );
    let slice_lines_of = |ledger: &str| -> Vec<String> {
        ledger
            .lines()
            .filter(|line| line.starts_with("slice="))
            .map(str::to_string)
            .collect()
    };
    let resumed_ledger = std::fs::read_to_string(&resumed_ledger_path).expect("读账本");
    assert_eq!(
        slice_lines_of(&resumed_ledger),
        slice_lines_of(&uninterrupted_ledger),
        "续跑写出的账本片行与一口气跑完的逐行相同"
    );
    assert_eq!(
        resumed_ledger.lines().next(),
        uninterrupted_ledger.lines().next(),
        "账本文件头逐字相同"
    );
    std::fs::remove_dir_all(&unsharded_directory).ok();
    std::fs::remove_dir_all(&shard_directory).ok();
}

/// 驱动脚本 `research/scripts/layer0-shard-run.sh --selftest` 跑的那条小流：分片开关、进度目录、输入指纹、线程数都从环境变量取
/// （与两条流的全量用例同一个入口）。没设分片开关时照常跑完整条流；分片跑一片时只写账本、不打计数行；merge 那一趟打的计数行
/// 与不分片的逐字相同（驱动脚本比这两行），并照钉死的数判。
#[test]
fn the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts() {
    let stream = first_stream("sharded-selftest");
    let versions = versions_with_content(file_content());
    let from_environment = Layer0Parallelism::from_environment();
    let parallelism = Layer0Parallelism {
        worker_threads: from_environment.worker_threads,
        worker_threads_source: from_environment.worker_threads_source,
        slice_length: Layer0SliceLength::StatesPerSlice(
            NonZeroU64::new(SELFTEST_STATES_PER_SLICE).expect("不是 0"),
        ),
    };
    let (outcome, _observer_calls) = run(
        &stream,
        &versions,
        parallelism,
        &Layer0Resume::from_environment(SELFTEST_STREAM_NAME),
    );
    let tally = match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_written) => return,
    };
    println!("{}", count_line(&tally));
    assert_eq!(tally.states, QUICK_TIER_STATES);
    assert_eq!(tally.observed_states, QUICK_TIER_STATES);
    assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_eq!(
        tally.observer_counts.get("file_read_states"),
        tally.file_read_states,
        "观察者数的读出文件与计数里的相同"
    );
}

/// golden 比对的子进程把进度文件写进这个目录（只供 golden 那条用例起的子进程读）。
const GOLDEN_DIRECTORY_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_SHARDED_TEST_GOLDEN_DIRECTORY";

/// 不分片跑两趟，都只起 1 个工作线程（进度行的次序才确定）：不留进度文件、片长按线程数定；留进度文件、每片 1 个状态。
/// 打印行与留下的进度文件是 golden 比对的对象：这段代码只用分片之前就有的入口，分片之前与之后编出来跑的是同一段。
fn run_the_unsharded_enumerations_for_the_golden_comparison(
    stream: &FirstStream,
    directory: &Path,
) {
    // libtest 在 `--nocapture` 下先打「test <名字> ... 」不换行：先换一行，第一行打印才从行首起。
    println!();
    let versions = versions_with_content(file_content());
    let one_worker_thread = |slice_length: Layer0SliceLength| Layer0Parallelism {
        worker_threads: NonZeroUsize::MIN,
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length,
    };
    let runs = [
        (
            one_worker_thread(Layer0SliceLength::ScaledToWorkerThreads),
            Layer0Resume::NoProgressFile,
        ),
        (
            one_worker_thread(Layer0SliceLength::StatesPerSlice(NonZeroU64::MIN)),
            Layer0Resume::KeepProgressFile(Layer0ProgressFileSettings {
                directory: directory.to_path_buf(),
                input_fingerprint: name_part("fingerprint0"),
                stream_name: name_part("golden_first_transaction_stream"),
                start: Layer0ResumeStart::ResumeFromTheProgressFile,
                after_completion: Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
            }),
        ),
    ];
    for (parallelism, resume) in runs {
        let tally = enumerate_layer0_in_state_slices(
            &stream.base,
            &stream.writes,
            &stream.segments,
            stream.root_index,
            &versions,
            &quick_tier_expansion,
            parallelism,
            Some(&mut |_image: &CrashImage<'_>,
                       report: &RecoveryReport,
                       counts: &mut Layer0ObserverCounts| {
                match report.outcome {
                    RecoveryOutcome::FileRead { .. } => counts.add("file_read_states", 1),
                    RecoveryOutcome::NoFile { .. } => counts.add("no_file_states", 1),
                    RecoveryOutcome::Failed { .. } => counts.add("failed_states", 1),
                }
            }),
            &resume,
        );
        println!("GOLDEN_TALLY {}", count_line(&tally));
    }
}

/// 跑 [`run_the_unsharded_enumerations_for_the_golden_comparison`]：golden 那条用例起的子进程（`--exact`、设了 golden 目录）跑的就是它，
/// 打印行交给父进程比；平时 `cargo test` 里它自己也跑一遍，进度文件放进自己的临时目录。两样都核：留下恰好一个进度文件、文件头加每片一行共 55 行。
#[test]
fn print_the_unsharded_enumerations_for_the_golden_comparison() {
    let (directory, own_directory_to_remove) =
        match std::env::var(GOLDEN_DIRECTORY_ENVIRONMENT_VARIABLE) {
            Ok(golden_directory) => (PathBuf::from(golden_directory), None),
            Err(std::env::VarError::NotPresent) => {
                let own_directory = fresh_directory("golden-helper", "progress");
                (own_directory.clone(), Some(own_directory))
            }
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{GOLDEN_DIRECTORY_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        };
    let stream = first_stream("sharded-golden");
    run_the_unsharded_enumerations_for_the_golden_comparison(&stream, &directory);
    let [progress_file] = files_in(&directory)
        .try_into()
        .expect("留下恰好一个进度文件");
    assert_eq!(
        std::fs::read_to_string(&progress_file)
            .expect("读进度文件")
            .lines()
            .count(),
        usize::try_from(QUICK_TIER_STATES + 1).expect("行数装得进 usize"),
        "文件头 + 每片一个状态的 54 行片行"
    );
    if let Some(own_directory) = own_directory_to_remove {
        std::fs::remove_dir_all(&own_directory).ok();
    }
}

/// [`run_the_unsharded_enumerations_for_the_golden_comparison`] 打出的行：以 `LAYER0_` / `GOLDEN_` 起头的 65 行，
/// 进度目录换成 `<directory>`、`elapsed_seconds=` 的值换成 `<elapsed>`，每行带换行拼起来的 SHA-256。
/// 65 行 = 第一趟（片长按线程数定，每片 16 个状态）`LAYER0_PARALLEL_START` + ⌈54 / 16⌉ = 4 行 `LAYER0_PROGRESS` +
/// `LAYER0_PARALLEL_FINISHED` + `GOLDEN_TALLY` 共 7 行，第二趟（每片 1 个状态）`LAYER0_RESUME` + `LAYER0_PARALLEL_START` +
/// 54 行 `LAYER0_PROGRESS` + `LAYER0_PARALLEL_FINISHED` + `GOLDEN_TALLY` 共 58 行。
///
/// 钉值的来历：头一版取自加分片之前的代码（`crash.rs` 与 `layer0_progress.rs` 加分片之前那一版：38 行、SHA-256 `6deeb48f…`，
/// 进度文件计划哈希 `a1d502a7…`、内容 `798bef3c…`），证的是加分片没改不分片那条路。实审 B3a-2 按设备记屏障、原地覆写补第三态之后
/// 状态数 29 → 54，撕裂镜像接进枚举用的写表（计划哈希跟着变），实审 B3a-3 在那一版上重取；重取之前与同一棵树上退回 B3a-2 之前的
/// 四份源码打的行逐行对过：每种行的词项一个不多一个不少，不同的只有随状态数走的计数与计划哈希。从这一版起，这组钉值守的是
/// 「不分片那条路打的行与留下的进度文件不悄悄变」。
const GOLDEN_PRINTED_LINE_COUNT: usize = 65;
const GOLDEN_PRINTED_LINES_SHA256: &str =
    "e4d8027a6b45e3a5557788d41b49b1a40d249ba62411bc0491bb984ab4a05800";
/// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（55 行：文件头 + 54 行片行）。
const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_first_transaction_stream-fingerprint0-5026593803c5654d562ab1858215ddd4a5be477116564e882947660016a1eb5e.txt";
const GOLDEN_PROGRESS_FILE_SHA256: &str =
    "ff0be4e3b94c2619b8e345dd5de6303be9150278b1bed62cdec9447270b08fac";

/// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
fn masked_printed_line(line: &str, directory: &Path) -> String {
    line.replace(&directory.display().to_string(), "<directory>")
        .split(' ')
        .map(|token| {
            if token.starts_with("elapsed_seconds=") {
                "elapsed_seconds=<elapsed>"
            } else {
                token
            }
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

/// 默认不分片（用户 2026-09-27 定）：没设分片开关时，枚举打的每一行（`LAYER0_PARALLEL_START` / `LAYER0_PROGRESS` /
/// `LAYER0_PARALLEL_FINISHED` / `LAYER0_RESUME`）、交回的计数与留下的进度文件（名字里的计划哈希、每一个字节）与钉值逐字相同
/// （钉值头一版取自加分片之前的代码，补第三态之后重取，来历见 [`GOLDEN_PRINTED_LINE_COUNT`]）。
/// 在子进程里跑（父进程要读它的标准输出），分片开关从子进程的环境里清掉。
#[test]
fn unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding() {
    let directory = fresh_directory("golden", "progress");
    let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
        .args([
            "--exact",
            "print_the_unsharded_enumerations_for_the_golden_comparison",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(GOLDEN_DIRECTORY_ENVIRONMENT_VARIABLE, &directory)
        .env_remove(LAYER0_SHARD_ENVIRONMENT_VARIABLE)
        .output()
        .expect("起子进程");
    let standard_output = String::from_utf8(output.stdout).expect("子进程的输出是 UTF-8");
    assert!(
        output.status.success(),
        "子进程失败：{:?}\n{standard_output}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let masked_lines: Vec<String> = standard_output
        .lines()
        .filter(|line| line.starts_with("LAYER0_") || line.starts_with("GOLDEN_"))
        .map(|line| masked_printed_line(line, &directory))
        .collect();
    let masked_text: String = masked_lines
        .iter()
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(
        (
            masked_lines.len(),
            singlefs_harness::sha256::sha256_hexadecimal(masked_text.as_bytes())
        ),
        (
            GOLDEN_PRINTED_LINE_COUNT,
            GOLDEN_PRINTED_LINES_SHA256.to_string()
        ),
        "不分片时打印的行与钉值（来历见 GOLDEN_PRINTED_LINE_COUNT 的文档）不同：\n{masked_text}"
    );
    let [progress_file] = files_in(&directory)
        .try_into()
        .expect("留下恰好一个进度文件");
    let progress_file_content = std::fs::read(&progress_file).expect("读进度文件");
    assert_eq!(
        (
            progress_file
                .file_name()
                .expect("有文件名")
                .to_string_lossy()
                .to_string(),
            singlefs_harness::sha256::sha256_hexadecimal(&progress_file_content)
        ),
        (
            GOLDEN_PROGRESS_FILE_NAME.to_string(),
            GOLDEN_PROGRESS_FILE_SHA256.to_string()
        ),
        "不分片时的进度文件与钉值不同：\n{}",
        String::from_utf8_lossy(&progress_file_content)
    );
    std::fs::remove_dir_all(&directory).ok();
}
