//! m3-batch33r-code-r1 攻方腿（Opus）原型。用法：`opus-probe-b33r <模式> [参数…]`，模式：
//!   roots                         三段删除历史（H1/H2/H3，两盘；mkfs 只认两盘）的段长、闭式状态数与层 0 枚举域数，不枚举；
//!                                 每个根上把候选结局（没有文件、读出表里每一版、读出别的）逐个喂给新判法与真值，打接受集不同的格
//!   oracle <h1|h2|h3> <1|2>       整条流按层 0 枚举域逐状态判（full_expansion，每段都展开）：新判法（R 的版本表）、旧判法（任一更旧版有文件）、
//!                                 由各次发布的 TransactionOutput 现算的真值，三方逐状态比；再对每个落到的根把候选结局逐个喂给新判法与真值比
//!   threads <h1|h2|h3> 2          R2（流截在第一个文件那次发布之前：取号与两次暖机，全量枚举放得进预算）：
//!                                 观察者在工作线程（1/4/6 线程）与调用线程上各跑一趟，计数逐项比；续跑（截断进度文件）、
//!                                 观察者中途 panic 之后再续、两片分片再并，各数观察者真被调了几次
//! 每次枚举之前先打印闭式状态数，超过 1_000_000 不跑（攻方腿定义 3b）。
use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_checker_tier::crash::{
    classified_oracle_violation_for_the_version_table, classified_oracle_violation_for_versions,
    enumerate_layer0_against_the_version_table_or_one_shard, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, Layer0EnumerationOutcome,
    Layer0ObserverCounts, Layer0Parallelism, Layer0SliceLength, Layer0StateObserving, Layer0Tally,
    Layer0VersionTable, Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::{
    Layer0ProgressFileAfterCompletion, Layer0ProgressFileNamePart, Layer0ProgressFileSettings,
    Layer0Resume, Layer0ResumeStart, Layer0ShardMerge, Layer0ShardOfShards, Layer0ShardRun,
    Layer0ToolchainIdentity,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, publishes_in,
    writes_and_segments_with_stream_indexes_and_entries,
    CrashImage, MemoryPool, PublishedVersion, PublishedVersionWithoutAFile,
    RecordStreamContinuity, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::scenario::{
    e142_parameters, first_file_content, run_deletion_history, three_data_unit_file_content,
    DeletionHistory, DeletionHistoryPublishKind, DeletionHistorySegment,
    E142_FILESYSTEM_IDENTIFIER,
};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordedEntrySpan, RecordingBlockDevice, SharedStream};

/// 一次原型跑的全量状态数上限（攻方腿定义 3b：约 10^6）。
const STATES_PER_RUN_AT_MOST: u64 = 1_000_000;

/// 一个根下面按真值该读出什么。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Truth {
    NoFile,
    File(Vec<u8>),
}

/// 录好的一条删除历史：层 0 要的几样，加每个根的真值与名字、R 的版本表与「每个没有文件的根都进表」的版本表。
struct Stream {
    name: String,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    truth: BTreeMap<(u32, u64), Truth>,
    label: BTreeMap<(u32, u64), String>,
    with_a_file: Vec<PublishedVersion>,
    /// R 的写法：只有意图那一版进「没有文件」组，之后的各批（与卸载、挂载的发布）不进表。
    without_a_file_like_r: Vec<PublishedVersionWithoutAFile>,
}

fn one_device_parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn history_named(name: &str) -> DeletionHistory {
    DeletionHistory::EVERY_HISTORY
        .into_iter()
        .find(|history| history.name() == name)
        .unwrap_or_else(|| panic!("没有这段历史：{name}"))
}

fn key(instance: InstanceGeneration, txg: CheckpointTxg) -> (u32, u64) {
    (instance.0, txg.0)
}

fn record(history: DeletionHistory, device_count: u32) -> Stream {
    let parameters = match device_count {
        1 => one_device_parameters(),
        2 => e142_parameters(512, 512),
        other => panic!("盘数只取 1 或 2：{other}"),
    };
    let identities: Vec<DeviceIdentity> = (0..device_count).map(DeviceIdentity).collect();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = identities
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let run = run_deletion_history(history, &parameters, &mut devices, &stream)
        .unwrap_or_else(|failure| panic!("{} 录不完：{failure:?}", history.name()));
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&identities, TEST_IMAGE_DEFAULT_BYTES);
    base.apply(&operations[..run.mkfs_operation_count]);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    // H2 有卸载入口：入口段的下标按录制流从 0 数，切掉 mkfs 那一截之后平移（C557 的核照做）。
    let entry_spans: Vec<RecordedEntrySpan> = stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - run.mkfs_operation_count
                ..span.operations.end - run.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[run.mkfs_operation_count..],
        &entry_spans,
        &geometry,
    );
    let every_write = vec![true; writes.len()];
    let publishes = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
    let judged_root_index = publishes.iter().map(|publish| publish.root).max().expect("有发布");
    let three_units = three_data_unit_file_content();
    let mut truth = BTreeMap::new();
    let mut label = BTreeMap::new();
    let mut intent_keys = Vec::new();
    for segment in &run.segments {
        if let DeletionHistorySegment::Publish { kind, version, .. } = segment {
            let root = key(version.root.instance, version.root.checkpoint_txg);
            let has_file = version.inode_record.is_some_and(|record| record.nlink != 0);
            truth.insert(root, if has_file { Truth::File(three_units.clone()) } else { Truth::NoFile });
            label.insert(root, kind.name().to_string());
            if *kind == DeletionHistoryPublishKind::UnlinkIntent {
                intent_keys.push(root);
            }
        }
    }
    let first_intent = *intent_keys.first().expect("有意图");
    let mut before: Vec<(u32, u64)> = publishes
        .iter()
        .map(|publish| (publish.instance, publish.checkpoint_txg))
        .filter(|root| (root.1, root.0) < (first_intent.1, first_intent.0))
        .collect();
    before.sort_by_key(|root| (root.1, root.0));
    before.dedup();
    let names: Vec<&str> = match history {
        DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
            vec!["warm_up_1", "warm_up_2", "first_file", "size_zero_inode", "sequential_write"]
        }
        _ => vec!["warm_up_1", "warm_up_2", "first_file", "sequential_write"],
    };
    assert_eq!(before.len(), names.len(), "意图之前的发布：{before:?}");
    let first_file = first_file_content();
    for (root, name) in before.iter().zip(&names) {
        let content = match *name {
            "warm_up_1" | "warm_up_2" => Truth::NoFile,
            "first_file" | "size_zero_inode" => Truth::File(first_file.clone()),
            _ => Truth::File(three_units.clone()),
        };
        truth.insert(*root, content);
        label.insert(*root, (*name).to_string());
    }
    for publish in &publishes {
        assert!(
            truth.contains_key(&(publish.instance, publish.checkpoint_txg)),
            "写表里的根 {:?} 没有真值",
            (publish.instance, publish.checkpoint_txg)
        );
    }
    let with_a_file = truth
        .iter()
        .filter_map(|(root, content)| match content {
            Truth::File(content) => Some(PublishedVersion {
                instance: InstanceGeneration(root.0),
                checkpoint_txg: CheckpointTxg(root.1),
                content: content.clone(),
            }),
            Truth::NoFile => None,
        })
        .collect();
    let without_a_file_like_r = intent_keys
        .iter()
        .map(|root| PublishedVersionWithoutAFile {
            instance: InstanceGeneration(root.0),
            checkpoint_txg: CheckpointTxg(root.1),
        })
        .collect();
    Stream {
        name: format!("{}-{}dev", history.name(), device_count),
        base,
        writes,
        segments,
        judged_root_index,
        truth,
        label,
        with_a_file,
        without_a_file_like_r,
    }
}

impl Stream {
    fn table(&self) -> Layer0VersionTable<'_> {
        Layer0VersionTable::new(&self.with_a_file, &self.without_a_file_like_r)
    }

    fn state_count(&self) -> u64 {
        layer0_state_count_with_torn_in_place_overwrites(
            &self.base,
            &self.writes,
            &self.segments,
            &full_expansion,
        )
    }

    fn describe(&self) {
        let sizes: Vec<usize> = self.segments.iter().map(Vec::len).collect();
        println!(
            "name=stream stream={} writes={} segments={} sizes={:?} closed_form_two_states={} layer0_states={}",
            self.name,
            self.writes.len(),
            self.segments.len(),
            sizes,
            closed_form_state_count(&self.segments),
            self.state_count()
        );
        for (root, content) in &self.truth {
            println!(
                "name=root stream={} instance={} txg={} label={} truth={} in_table_like_r={}",
                self.name,
                root.0,
                root.1,
                self.label[root],
                match content {
                    Truth::NoFile => "no_file".to_string(),
                    Truth::File(content) => format!("file_{}_bytes", content.len()),
                },
                if self.with_a_file.iter().any(|v| key(v.instance, v.checkpoint_txg) == *root) {
                    "with_a_file"
                } else if self.without_a_file_like_r.iter().any(|v| key(v.instance, v.checkpoint_txg) == *root) {
                    "without_a_file"
                } else {
                    "not_in_table"
                }
            );
        }
    }

    fn refuse_if_too_many_states(&self) -> u64 {
        let states = self.state_count();
        println!("name=budget stream={} states={states} limit={STATES_PER_RUN_AT_MOST}", self.name);
        assert!(states <= STATES_PER_RUN_AT_MOST, "{} 有 {states} 个状态，超过一次跑的上限", self.name);
        states
    }
}

/// 真值判：没择到根、落到比盘上最新根槽旧的根、走读失败照 oracle；其余拿结局与这个根的真值比。
fn truth_violation(
    stream: &Stream,
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted: Option<(CheckpointTxg, InstanceGeneration)>,
) -> Option<&'static str> {
    let Some((instance, txg)) = effective_root else {
        return Some("no_root_chosen");
    };
    if newest_persisted.is_some_and(|newest| (txg, instance) < newest) {
        return Some("recovered_to_an_older_root");
    }
    if let RecoveryOutcome::Failed { .. } = outcome {
        return Some("read_failed");
    }
    match (stream.truth.get(&key(instance, txg)), outcome) {
        (None, _) => Some("root_without_truth"),
        (Some(Truth::NoFile), RecoveryOutcome::NoFile { .. }) => None,
        (Some(Truth::NoFile), _) => Some("file_read_where_truth_has_none"),
        (Some(Truth::File(expected)), RecoveryOutcome::FileRead { content, .. }) => {
            (content != expected).then_some("wrong_content")
        }
        (Some(Truth::File(_)), _) => Some("no_file_where_truth_has_one"),
    }
}

fn parallelism(worker_threads: usize) -> Layer0Parallelism {
    Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("线程数不是 0"),
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: std::env::var("OPUS_PROBE_STATES_PER_SLICE")
            .ok()
            .and_then(|text| text.parse::<u64>().ok())
            .and_then(std::num::NonZeroU64::new)
            .map_or(Layer0SliceLength::ScaledToWorkerThreads, Layer0SliceLength::StatesPerSlice),
    }
}

fn worker_threads_from_the_environment() -> usize {
    std::env::var("OPUS_PROBE_THREADS")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(4)
}

fn whole_stream(outcome: Layer0EnumerationOutcome) -> Layer0Tally {
    match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => {
            panic!("要整条流，实际写了第 {} 片的账本", written.shard.text())
        }
    }
}

fn kind_name(violation: Option<singlefs_checker_tier::crash::Layer0OracleViolation>) -> String {
    violation.map_or_else(|| "none".to_string(), |violation| violation.kind.name().to_string())
}

/// 一个状态上两遍恢复各判三种：R 的新判法、旧判法、真值。计数名 `<遍>_new_<类>_old_<类>_truth_<类>`；新判法与真值分开的头 20 个另记原样。
fn judge_three_ways(
    stream: &Stream,
    image: &CrashImage<'_>,
    consulted: &RecoveryReport,
    counts: &mut Layer0ObserverCounts,
    disagreements: &Mutex<Vec<String>>,
    landed_roots: &Mutex<BTreeSet<(u32, u64)>>,
) {
    let newest = newest_persisted_root(image.writes, &image.persisted);
    // 不看 journal 那一遍只由枚举自己的计数按 R 的版本表判（tally.ignored_violations）：这里再恢复一遍要多三成挂钟，估时超过 40 分钟，砍掉（报告「限度」一节）。
    for (pass, report) in [("consulted", consulted)] {
        let new = kind_name(classified_oracle_violation_for_the_version_table(
            &report.outcome,
            report.effective_root,
            newest,
            stream.table(),
        ));
        let old = kind_name(classified_oracle_violation_for_versions(
            &report.outcome,
            report.effective_root,
            newest,
            &stream.with_a_file,
        ));
        let truth = truth_violation(stream, &report.outcome, report.effective_root, newest)
            .unwrap_or("none");
        counts.add(
            &format!("{pass}_new_{}_old_{}_truth_{truth}", new.to_lowercase(), old.to_lowercase()),
            1,
        );
        if let Some((instance, txg)) = report.effective_root {
            landed_roots.lock().expect("锁").insert(key(instance, txg));
            counts.add(&format!("{pass}_landed_{}", label_of(stream, key(instance, txg))), 1);
        }
        if (new == "none") != (truth == "none") {
            let mut disagreements = disagreements.lock().expect("锁");
            if disagreements.len() < 20 {
                disagreements.push(format!(
                    "pass={pass} effective_root={:?} newest_persisted={newest:?} outcome={} new={new} old={old} truth={truth}",
                    report.effective_root,
                    outcome_text(&report.outcome)
                ));
            }
        }
    }
}

fn outcome_text(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { content, .. } => format!("file_read_{}_bytes", content.len()),
        RecoveryOutcome::Failed { failure, .. } => format!("failed_{failure:?}").replace(' ', "_"),
    }
}

fn print_counts(prefix: &str, stream: &Stream, tally: &Layer0Tally) {
    println!(
        "name={prefix}_tally stream={} states={} observed_states={} violations={} ignored_violations={} failed_states={} no_file_states={} file_read_states={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violated={}",
        stream.name,
        tally.states,
        tally.observed_states,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_violated_states.values().sum::<u64>()
    );
    for (name, amount) in tally.observer_counts.iter() {
        println!("name={prefix}_count stream={} counter={name} states={amount}", stream.name);
    }
}

/// 每个根上把候选结局逐个喂给新判法（R 的版本表、盘上最新根槽取这个根本身）与真值，接受集不同的一格打一行 `name=alternative_differs`。
fn alternative_outcomes(stream: &Stream, roots: &BTreeSet<(u32, u64)>) {
    let mut candidates: Vec<(String, Option<Vec<u8>>)> = vec![("no_file".to_string(), None)];
    let mut contents: Vec<Vec<u8>> = stream.with_a_file.iter().map(|v| v.content.clone()).collect();
    contents.dedup();
    for content in contents {
        candidates.push((format!("file_read_{}_bytes", content.len()), Some(content)));
    }
    candidates.push(("file_read_garbage".to_string(), Some(b"garbage".to_vec())));
    let mut judged = 0u64;
    let mut differing = 0u64;
    let mut old_differing = 0u64;
    for root in roots {
        let effective = (InstanceGeneration(root.0), CheckpointTxg(root.1));
        for (name, content) in &candidates {
            let outcome = match content {
                None => RecoveryOutcome::NoFile { root: effective },
                Some(content) => RecoveryOutcome::FileRead { root: effective, content: content.clone() },
            };
            let newest = Some((effective.1, effective.0));
            let new_accepts =
                classified_oracle_violation_for_the_version_table(&outcome, Some(effective), newest, stream.table())
                    .is_none();
            let truth_accepts = truth_violation(stream, &outcome, Some(effective), newest).is_none();
            let old_accepts =
                classified_oracle_violation_for_versions(&outcome, Some(effective), newest, &stream.with_a_file)
                    .is_none();
            if old_accepts != truth_accepts {
                old_differing += 1;
                println!(
                    "name=alternative_old_differs stream={} root={root:?} label={} outcome={name} old_accepts={old_accepts} truth_accepts={truth_accepts}",
                    stream.name,
                    label_of(stream, *root)
                );
            }
            judged += 1;
            if new_accepts != truth_accepts {
                differing += 1;
                println!(
                    "name=alternative_differs stream={} root={root:?} label={} outcome={name} new_accepts={new_accepts} truth_accepts={truth_accepts}",
                    stream.name,
                    stream.label.get(root).map_or("unknown", String::as_str)
                );
            }
        }
    }
    println!(
        "name=alternative_summary stream={} roots={} judged_pairs={judged} differing_pairs={differing} old_differing_pairs={old_differing}",
        stream.name,
        roots.len()
    );
}

fn run_oracle(stream: &Stream) {
    stream.describe();
    stream.refuse_if_too_many_states();
    let disagreements = Mutex::new(Vec::new());
    let landed_roots = Mutex::new(BTreeSet::new());
    let observe: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts) + Sync) =
        &|image, report, counts| {
            judge_three_ways(stream, image, report, counts, &disagreements, &landed_roots);
        };
    let started = std::time::Instant::now();
    let tally = this_run_or_this_shard(enumerate_layer0_against_the_version_table_or_one_shard(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.judged_root_index,
        stream.table(),
        &full_expansion,
        parallelism(worker_threads_from_the_environment()),
        Layer0StateObserving::OnTheWorkerThreads(observe),
        &resume_from_the_environment(),
    ));
    print_counts("oracle", stream, &tally);
    for line in disagreements.lock().expect("锁").iter() {
        println!("name=disagreement stream={} {line}", stream.name);
    }
    let mut roots = landed_roots.into_inner().expect("锁");
    roots.extend(stream.truth.keys().copied());
    alternative_outcomes(stream, &roots);
    println!(
        "name=oracle_wall_clock stream={} seconds={:.1} first_violation={:?}",
        stream.name,
        started.elapsed().as_secs_f64(),
        tally.first_violation
    );
}

/// 一个状态的持久集合压成一个数（FNV-1a），按状态求和：漏一个、重一个，和就变（与次序无关）。
fn state_digest(persisted: &[bool]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for bit in persisted {
        hash ^= u64::from(*bit);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash % 1_000_003
}

fn count_the_state(stream: &Stream, image: &CrashImage<'_>, report: &RecoveryReport, counts: &mut Layer0ObserverCounts) {
    counts.add("observed", 1);
    counts.add("digest_sum", state_digest(&image.persisted));
    let landed = report
        .effective_root
        .map_or("none".to_string(), |(instance, txg)| label_of(stream, key(instance, txg)));
    counts.add(&format!("landed_{landed}"), 1);
}

fn progress_settings(directory: &Path, start: Layer0ResumeStart) -> Layer0ProgressFileSettings {
    Layer0ProgressFileSettings {
        directory: directory.to_path_buf(),
        input_fingerprint: Layer0ProgressFileNamePart::new("opus-probe").expect("合法"),
        stream_name: Layer0ProgressFileNamePart::new("opus_probe_stream").expect("合法"),
        start,
        after_completion: Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
    }
}

fn toolchain() -> Layer0ToolchainIdentity {
    Layer0ToolchainIdentity {
        rustc_version_lines: "rustc probe\nbinary: rustc\nhost: probe".to_string(),
        cargo_version: "cargo probe".to_string(),
        target_triple: "probe".to_string(),
    }
}

/// 观察者放哪、几个线程、怎么续跑：跑一趟，交回计数与这一趟里观察者真被调了几次。
fn enumerate_once(
    stream: &Stream,
    on_worker_threads: bool,
    worker_threads: usize,
    resume: &Layer0Resume,
    panic_at_invocation: Option<u64>,
) -> (Layer0EnumerationOutcome, u64) {
    let invocations = AtomicU64::new(0);
    let observe_worker: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts) + Sync) =
        &|image, report, counts| {
            let invocation = invocations.fetch_add(1, Ordering::SeqCst) + 1;
            if panic_at_invocation == Some(invocation) {
                panic!("原型故意在第 {invocation} 次观察时 panic");
            }
            count_the_state(stream, image, report, counts);
        };
    let mut observe_calling = |image: &CrashImage<'_>, report: &RecoveryReport, counts: &mut Layer0ObserverCounts| {
        let invocation = invocations.fetch_add(1, Ordering::SeqCst) + 1;
        if panic_at_invocation == Some(invocation) {
            panic!("原型故意在第 {invocation} 次观察时 panic");
        }
        count_the_state(stream, image, report, counts);
    };
    let observing = if on_worker_threads {
        Layer0StateObserving::OnTheWorkerThreads(observe_worker)
    } else {
        Layer0StateObserving::OnTheCallingThread(&mut observe_calling)
    };
    let outcome = enumerate_layer0_against_the_version_table_or_one_shard(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.judged_root_index,
        stream.table(),
        &full_expansion,
        parallelism(worker_threads),
        observing,
        resume,
    );
    (outcome, invocations.load(Ordering::SeqCst))
}

fn the_only_file_in(directory: &Path) -> PathBuf {
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .expect("读进度目录")
        .map(|entry| entry.expect("目录项").path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    assert_eq!(files.len(), 1, "进度目录里该恰好一份文件：{files:?}");
    files.remove(0)
}

/// 留下文件头与前一半片行（造「跑到一半被杀」）：交回留下的片行数与原有的片行数。
fn truncate_to_half_of_the_slice_lines(path: &Path) -> (usize, usize) {
    let text = std::fs::read_to_string(path).expect("读进度文件");
    let lines: Vec<&str> = text.lines().collect();
    let slice_lines = lines.len() - 1;
    let kept = slice_lines / 2;
    let mut truncated = lines[..=kept].join("\n");
    truncated.push('\n');
    std::fs::write(path, truncated).expect("写回截断的进度文件");
    (kept, slice_lines)
}

fn fresh_directory(root: &Path, name: &str) -> PathBuf {
    let directory = root.join(name);
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("清掉上一趟的进度目录");
    }
    std::fs::create_dir_all(&directory).expect("建进度目录");
    directory
}

fn line(stream: &Stream, case: &str, tally: &Layer0Tally, invocations: u64, reference: &Layer0Tally) {
    println!(
        "name=threads_case stream={} case={case} states={} observed_states={} invocations_in_this_run={invocations} observed={} digest_sum={} equals_reference={}",
        stream.name,
        tally.states,
        tally.observed_states,
        tally.observer_counts.count_named("observed"),
        tally.observer_counts.count_named("digest_sum"),
        tally == reference
    );
}

fn run_threads(stream: &Stream, progress_root: &Path) {
    stream.describe();
    let states = stream.refuse_if_too_many_states();
    let none = Layer0Resume::NoProgressFile;
    let (reference, invocations) = enumerate_once(stream, false, 1, &none, None);
    let reference = whole_stream(reference);
    line(stream, "calling_thread_1", &reference, invocations, &reference);
    for (case, on_workers, threads) in [
        ("calling_thread_4", false, 4),
        ("worker_threads_1", true, 1),
        ("worker_threads_4", true, 4),
        ("worker_threads_6", true, 6),
    ] {
        let (outcome, invocations) = enumerate_once(stream, on_workers, threads, &none, None);
        line(stream, case, &whole_stream(outcome), invocations, &reference);
    }
    // 续跑：第一趟留着进度文件，截掉后一半片行，换线程数、换观察者所在的线程再续。
    for (case, first_on_workers, resumed_on_workers) in [
        ("resume_worker4_then_worker6", true, true),
        ("resume_calling4_then_worker6", false, true),
        ("resume_worker4_then_calling6", true, false),
    ] {
        let directory = fresh_directory(progress_root, case);
        let start_over = Layer0Resume::KeepProgressFile(progress_settings(&directory, Layer0ResumeStart::StartOverDiscardingTheProgressFile));
        let (first, first_invocations) = enumerate_once(stream, first_on_workers, 4, &start_over, None);
        line(stream, &format!("{case}_first"), &whole_stream(first), first_invocations, &reference);
        let (kept, all) = truncate_to_half_of_the_slice_lines(&the_only_file_in(&directory));
        let resume = Layer0Resume::KeepProgressFile(progress_settings(&directory, Layer0ResumeStart::ResumeFromTheProgressFile));
        let (resumed, invocations) = enumerate_once(stream, resumed_on_workers, 6, &resume, None);
        println!("name=threads_resume stream={} case={case} kept_slice_lines={kept} all_slice_lines={all}", stream.name);
        line(stream, &format!("{case}_resumed"), &whole_stream(resumed), invocations, &reference);
    }
    // 观察者在工作线程上 panic：进度文件还在不在、留下几行；再续一趟。
    let directory = fresh_directory(progress_root, "panic_then_resume");
    let start_over = Layer0Resume::KeepProgressFile(progress_settings(&directory, Layer0ResumeStart::StartOverDiscardingTheProgressFile));
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enumerate_once(stream, true, 4, &start_over, Some(states / 2))
    }));
    let left: Vec<PathBuf> = std::fs::read_dir(&directory).expect("读").map(|e| e.expect("项").path()).collect();
    let left_lines: Vec<usize> = left.iter().map(|p| std::fs::read_to_string(p).map_or(0, |t| t.lines().count())).collect();
    println!(
        "name=threads_panic stream={} panicked={} files_left={} lines_left={left_lines:?}",
        stream.name,
        panicked.is_err(),
        left.len()
    );
    let resume = Layer0Resume::KeepProgressFile(progress_settings(&directory, Layer0ResumeStart::ResumeFromTheProgressFile));
    let (resumed, invocations) = enumerate_once(stream, true, 4, &resume, None);
    line(stream, "panic_then_resume", &whole_stream(resumed), invocations, &reference);
    // 两片分片、再并。
    let directory = fresh_directory(progress_root, "two_shards");
    let shard_count = NonZeroU32::new(2).expect("2");
    let mut shard_invocations = 0;
    for shard_index in 0..2 {
        let run = Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
            progress: progress_settings(&directory, Layer0ResumeStart::StartOverDiscardingTheProgressFile),
            shard: Layer0ShardOfShards::new(shard_index, shard_count).expect("片号"),
            toolchain: toolchain(),
        });
        let (outcome, invocations) = enumerate_once(stream, shard_index == 0, 4, &run, None);
        shard_invocations += invocations;
        assert!(matches!(outcome, Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_)));
    }
    let merge = Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
        directory: directory.clone(),
        input_fingerprint: Layer0ProgressFileNamePart::new("opus-probe").expect("合法"),
        stream_name: Layer0ProgressFileNamePart::new("opus_probe_stream").expect("合法"),
        shard_count,
        toolchain: toolchain(),
    });
    let (merged, invocations) = enumerate_once(stream, true, 4, &merge, None);
    line(stream, "two_shards_merged", &whole_stream(merged), shard_invocations + invocations, &reference);
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let argument = |index: usize| arguments.get(index).map(String::as_str);
    match argument(0) {
        Some("roots") => {
            // 不枚举：每段历史的段长与状态数照打（看得出为什么全量跑不动），每个根上逐个候选结局比新判法与真值。
            for history in DeletionHistory::EVERY_HISTORY {
                let stream = record(history, 2);
                stream.describe();
                let roots: BTreeSet<(u32, u64)> = stream.truth.keys().copied().collect();
                alternative_outcomes(&stream, &roots);
            }
        }
        Some("oracle") => {
            let stream = record(
                history_named(argument(1).expect("历史名")),
                argument(2).expect("盘数").parse().expect("盘数是数"),
            );
            match argument(3) {
                None => run_oracle(&stream),
                Some("after-the-intent-up-to-the-continuation") => {
                    run_oracle(&after_the_intent_up_to_the_continuation(stream, "unlink_intent"));
                }
                Some("mount-up-to-the-continuation") => {
                    run_oracle(&after_the_intent_up_to_the_continuation(stream, "unmount_floor_raise"));
                }
                Some("mount-warm-ups") => {
                    // 写行那一段估时也超过 40 分钟（量的见报告），再缩到挂载的两次暖机（实例 2 的根，比它旧的最新一项是实例 1 的意图）。
                    run_oracle(&after_the_intent_up_to_the_continuation(stream, "mount_instance_row"));
                }
                Some("mount-instance-row") => {
                    // 估时超过 40 分钟之后再缩：只留挂载写行那次发布（实例从 1 换到 2 的那一次）与它之前的写、之后的轮换。
                    run_oracle(&up_to_the_first_unit_of(
                        after_the_intent_up_to_the_continuation(stream, "unmount_floor_raise"),
                        "mount_warm_up",
                    ));
                }
                Some(other) => panic!("没有这种截法：{other}"),
            }
        }
        Some("verdicts") => {
            // 删除那几版上池级 checker 的删除类不变量判没判（Holds / Violated 算判了，NotApplicable / PartiallyJudged 不红也不算判）：
            // 在整条流的状态序号上均匀取 `n` 个、再加最后那个全持久状态，逐个判，按落到的根与不变量数判定的种类。
            let stream = record(history_named(argument(1).expect("历史名")), 2);
            let states = stream.state_count();
            let samples: u64 = argument(2).expect("取样数").parse().expect("是数");
            let mut kinds: BTreeMap<String, u64> = BTreeMap::new();
            let mut ordinals: Vec<u64> = (0..samples).map(|sample| sample * states / samples).collect();
            ordinals.push(states - 1);
            for ordinal in ordinals {
                let _ = singlefs_checker_tier::crash::judge_layer0_state_range(
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    stream.judged_root_index,
                    &stream.with_a_file,
                    &full_expansion,
                    ordinal..ordinal + 1,
                    &mut |_ordinal, _image, judgement| {
                        let landed = judgement
                            .consulted
                            .effective_root
                            .map_or("none".to_string(), |(i, t)| label_of(&stream, key(i, t)));
                        for (invariant, verdict) in &judgement.pool_checker {
                            if !["I-9.5", "I-4.6", "I-4.7", "I-9.19"].contains(invariant) {
                                continue;
                            }
                            let kind = match verdict {
                                singlefs_checker::image::InvariantVerdict::Holds => "holds",
                                singlefs_checker::image::InvariantVerdict::Violated(_) => "violated",
                                singlefs_checker::image::InvariantVerdict::NotApplicable(_) => "not_applicable",
                                singlefs_checker::image::InvariantVerdict::PartiallyJudged(_) => "partially_judged",
                            };
                            *kinds.entry(format!("landed={landed} invariant={invariant} verdict={kind}")).or_insert(0) += 1;
                        }
                    },
                );
            }
            for (cell, count) in kinds {
                println!("name=verdict stream={} {cell} states={count}", stream.name);
            }
        }
        Some("estimate") => {
            // 正式跑之前的估时：在整段状态序号上均匀取 `n` 个，单线程逐个判（judge_layer0_state_range，与枚举同一个判），量每个的平均耗时。
            let stream = after_the_intent_up_to_the_continuation(record(history_named("h2"), 2), argument(2).unwrap_or("unlink_intent"));
            let states = stream.state_count();
            let samples: u64 = argument(1).expect("取样数").parse().expect("是数");
            let started = std::time::Instant::now();
            for sample in 0..samples {
                let ordinal = sample * states / samples;
                let _ = singlefs_checker_tier::crash::judge_layer0_state_range(
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    stream.judged_root_index,
                    &stream.with_a_file,
                    &full_expansion,
                    ordinal..ordinal + 1,
                    &mut |_ordinal, _image, _judgement| {},
                );
            }
            let per_state = started.elapsed().as_secs_f64() / samples as f64;
            println!(
                "name=estimate stream={} states={states} samples={samples} seconds_per_state_one_thread={per_state:.4} estimated_seconds_on_six_threads={:.0}",
                stream.name,
                per_state * states as f64 / 6.0
            );
        }
        Some("threads") => {
            let progress_root = PathBuf::from(std::env::var("OPUS_PROBE_PROGRESS_DIR").expect("设 OPUS_PROBE_PROGRESS_DIR"));
            run_threads(
                &up_to_the_first_file(record(
                    history_named(argument(1).expect("历史名")),
                    argument(2).expect("盘数").parse().expect("盘数是数"),
                )),
                &progress_root,
            );
        }
        other => panic!("模式只有 roots、oracle、threads：{other:?}"),
    }
}

/// 截到第一个文件那次发布的单元写之前（取号与两次暖机，与它们之后的系统配置槽轮换）：R2 的机制对拍用，全量枚举放得进预算。
fn up_to_the_first_file(stream: Stream) -> Stream {
    let every_write = vec![true; stream.writes.len()];
    let publishes = publishes_in(&stream.writes, &every_write, RecordStreamContinuity::OneRecording);
    let first_file = publishes
        .iter()
        .find(|publish| stream.label[&(publish.instance, publish.checkpoint_txg)] == "first_file")
        .expect("有第一个文件那次发布");
    let cut = *first_file.units.iter().min().expect("第一个文件有单元写");
    let segments: Vec<Vec<usize>> = stream
        .segments
        .iter()
        .take_while(|segment| segment.iter().all(|write| *write < cut))
        .cloned()
        .collect();
    let write_count = segments.iter().flatten().max().map_or(0, |last| last + 1);
    let writes = stream.writes[..write_count].to_vec();
    let every_write = vec![true; writes.len()];
    let kept = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
    let judged_root_index = kept.iter().map(|publish| publish.root).max().expect("截下来有根");
    let kept_roots: BTreeSet<(u32, u64)> =
        kept.iter().map(|publish| (publish.instance, publish.checkpoint_txg)).collect();
    Stream {
        name: format!("{}-up-to-the-first-file", stream.name),
        base: stream.base,
        writes,
        segments,
        judged_root_index,
        truth: stream.truth.into_iter().filter(|(root, _)| kept_roots.contains(root)).collect(),
        label: stream.label,
        with_a_file: Vec::new(),
        without_a_file_like_r: Vec::new(),
    }
}

/// 根的名字；写表之外的根（mkfs 的第 0 代根）叫 `mkfs_or_outside_the_recording`。
fn label_of(stream: &Stream, root: (u32, u64)) -> String {
    stream
        .label
        .get(&root)
        .cloned()
        .unwrap_or_else(|| "mkfs_or_outside_the_recording".to_string())
}

/// H2 截一段：意图那次发布连同它的系统配置槽轮换整段并进基镜像（全持久），从卸载那一截起、到挂载续做第一批的单元写之前为止
/// （卸载的两次抬 F 发布、挂载写行、两次暖机，实例从 1 换到 2）。这一段里每一段都展开；版本表、真值与整条流的相同。
/// `from_after` 是窗口前最后一次并进基镜像的发布的名字（`unlink_intent` 从卸载起；`unmount_floor_raise` 取最后一次抬 F，从挂载起）。
fn after_the_intent_up_to_the_continuation(stream: Stream, from_after: &str) -> Stream {
    let every_write = vec![true; stream.writes.len()];
    let publishes = publishes_in(&stream.writes, &every_write, RecordStreamContinuity::OneRecording);
    let label_of_publish = |publish: &singlefs_harness::memory_pool::PublishWrites| {
        label_of(&stream, (publish.instance, publish.checkpoint_txg))
    };
    let intent = publishes.iter().filter(|p| label_of_publish(p) == from_after).last().expect("有窗口前那次发布");
    let first_batch = publishes
        .iter()
        .find(|p| label_of_publish(p) == "mount_continuation_unlink_batch")
        .expect("有挂载续做的一批（H2）");
    let intent_root_segment = stream.segments.iter().position(|s| s.contains(&intent.root)).expect("意图的根槽写在某一段");
    // 意图的根槽写那一段之后是它的系统配置槽轮换那一段；窗口从再下一段起。
    let first_segment = intent_root_segment + 2;
    let cut = *first_batch.units.iter().min().expect("续做的一批有单元写");
    let start = *stream.segments[first_segment].iter().min().expect("段非空");
    let kept: Vec<Vec<usize>> = stream.segments[first_segment..]
        .iter()
        .take_while(|segment| segment.iter().all(|write| *write < cut))
        .map(|segment| segment.iter().map(|write| write - start).collect())
        .collect();
    let end = start + kept.iter().flatten().max().map_or(0, |last| last + 1);
    let mut base = stream.base.clone();
    base.apply_writes(&stream.writes[..start]);
    let writes = stream.writes[start..end].to_vec();
    let every_write = vec![true; writes.len()];
    let in_window = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
    let judged_root_index = in_window.iter().map(|publish| publish.root).max().expect("窗口里有根");
    Stream {
        name: format!("{}-after-the-{from_after}-up-to-the-continuation", stream.name),
        base,
        writes,
        segments: kept,
        judged_root_index,
        truth: stream.truth,
        label: stream.label,
        with_a_file: stream.with_a_file,
        without_a_file_like_r: stream.without_a_file_like_r,
    }
}

/// oracle 那一趟分批跑（子 agent 前台一条命令四分钟以内）：`OPUS_PROBE_SHARD=i/n` 只跑第 i 片、写账本；`OPUS_PROBE_MERGE=n` 并 n 份账本；
/// 都没设就整条流一趟、不留进度文件。进度与账本放 `OPUS_PROBE_PROGRESS_DIR`。
fn resume_from_the_environment() -> Layer0Resume {
    let directory = || PathBuf::from(std::env::var("OPUS_PROBE_PROGRESS_DIR").expect("分片要设 OPUS_PROBE_PROGRESS_DIR"));
    if let Ok(shard) = std::env::var("OPUS_PROBE_SHARD") {
        let (index, count) = shard.split_once('/').expect("i/n");
        return Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
            progress: progress_settings(&directory(), Layer0ResumeStart::ResumeFromTheProgressFile),
            shard: Layer0ShardOfShards::new(
                index.parse().expect("片号"),
                NonZeroU32::new(count.parse().expect("片数")).expect("片数不是 0"),
            )
            .expect("片号小于片数"),
            toolchain: toolchain(),
        });
    }
    if let Ok(count) = std::env::var("OPUS_PROBE_MERGE") {
        return Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
            directory: directory(),
            input_fingerprint: Layer0ProgressFileNamePart::new("opus-probe").expect("合法"),
            stream_name: Layer0ProgressFileNamePart::new("opus_probe_stream").expect("合法"),
            shard_count: NonZeroU32::new(count.parse().expect("片数")).expect("片数不是 0"),
            toolchain: toolchain(),
        });
    }
    Layer0Resume::NoProgressFile
}

fn this_run_or_this_shard(outcome: Layer0EnumerationOutcome) -> Layer0Tally {
    match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => {
            println!("name=shard_written shard={} ledger={}", written.shard.text(), written.ledger_path.display());
            written.tally_of_this_shard
        }
    }
}

/// 再截：到名为 `label` 的第一次发布的第一个单元写之前为止（估时超过 40 分钟时用）。
fn up_to_the_first_unit_of(stream: Stream, label: &str) -> Stream {
    let every_write = vec![true; stream.writes.len()];
    let publishes = publishes_in(&stream.writes, &every_write, RecordStreamContinuity::OneRecording);
    let first = publishes
        .iter()
        .find(|publish| label_of(&stream, (publish.instance, publish.checkpoint_txg)) == label)
        .expect("窗口里有这次发布");
    let cut = *first.units.iter().min().expect("有单元写");
    let segments: Vec<Vec<usize>> = stream
        .segments
        .iter()
        .take_while(|segment| segment.iter().all(|write| *write < cut))
        .cloned()
        .collect();
    let write_count = segments.iter().flatten().max().map_or(0, |last| last + 1);
    let writes = stream.writes[..write_count].to_vec();
    let every_write = vec![true; writes.len()];
    let kept = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
    Stream {
        name: format!("{}-up-to-the-first-{label}", stream.name),
        judged_root_index: kept.iter().map(|publish| publish.root).max().expect("截下来有根"),
        writes,
        segments,
        ..stream
    }
}
