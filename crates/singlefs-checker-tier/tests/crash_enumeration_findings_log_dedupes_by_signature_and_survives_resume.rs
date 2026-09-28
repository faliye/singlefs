//! checker 档模块：crash、layer0_progress
//! 层 0 放量的发现日志（用户 2026-09-27「崩溃放量日志 只写出错日志或者需要全量和错误双份日志 不然你读不过来了」；里程碑「覆盖写、释放、回退与复用」
//! 收尾批「层 0 发现日志」）：在第一条流上（mkfs → 取号 → 暖机 → A，按甲二展开，原地覆写取三态，46 个状态；C577 之前 54 个）把版本表故意写错——
//! 暖机第二次发布（实例 1、txg 2）下面说有文件、A（txg 3）的内容说成别的——落在 txg 2 的状态报「有文件却报没有」，读出 A 的状态报
//! 「读回的内容不对」，两遍恢复（看 journal、不看 journal）各红几段。今天的计数只留一条 `first_violation`；发现表按签名去重之后
//! 是几个签名、各几个状态、最先是哪几个状态，拿一个与枚举器分开写的观察者逐状态重判来核，再核发现日志的每一行。
//! 名字里不带 layer0：它只跑甲二那 46 个状态，平时跑得起。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    check_records, classified_oracle_violation_for_versions,
    enumerate_layer0_in_state_slices_or_one_shard_with_findings_log, publish_of_each_segment,
    quick_tier_expansion, Layer0EnumerationOutcome, Layer0FindingSignature, Layer0Findings,
    Layer0ObserverCounts, Layer0OracleViolationKind, Layer0Parallelism, Layer0PublishOfState,
    Layer0RedPass, Layer0SegmentOfState, Layer0SliceLength, Layer0StateObserver, Layer0Tally,
    Layer0WorkerThreadsSource, TearableInPlaceOverwrites,
};
use singlefs_checker_tier::layer0_progress::{
    unescaped_finding_text, Layer0FindingsLog, Layer0ProgressFileAfterCompletion,
    Layer0ProgressFileNamePart, Layer0ProgressFileSettings, Layer0Resume, Layer0ResumeStart,
    Layer0ShardMerge, Layer0ShardOfShards, Layer0ShardRun, Layer0ToolchainIdentity,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 第一条流按甲二展开的状态数（`crash_enumeration_sharded_across_processes.rs` 的 `QUICK_TIER_STATES`，那里写了怎么数出来的）。
const QUICK_TIER_STATES: u64 = 46;
/// 进度文件、账本与发现日志 begin 行里的流名。
const STREAM_NAME: &str = "findings_new_pool_file_creation_stream";

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

/// 故意写错的版本表：暖机第二次发布（实例 1、txg 2）下面说有文件（它其实没写文件），A（实例 1、txg 3）的内容说成别的。
fn versions_claiming_a_warm_up_file_and_the_wrong_content() -> Vec<PublishedVersion> {
    assert_ne!(file_content(), b"not the content A published".to_vec());
    vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
            content: b"a file the warm-up never wrote".to_vec(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: b"not the content A published".to_vec(),
        },
    ]
}

/// 每个用例一个自己的目录（先清空）。
fn fresh_directory(test_name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "singlefs-crash-enumeration-findings-{}-{test_name}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("清空上一次留下的目录");
    }
    std::fs::create_dir_all(&directory).expect("建目录");
    directory
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

fn name_part(text: &str) -> Layer0ProgressFileNamePart {
    Layer0ProgressFileNamePart::new(text).expect("合法的文件名片段")
}

fn progress_settings(
    directory: &Path,
    after_completion: Layer0ProgressFileAfterCompletion,
) -> Layer0ProgressFileSettings {
    Layer0ProgressFileSettings {
        directory: directory.to_path_buf(),
        input_fingerprint: name_part("fingerprint0"),
        stream_name: name_part(STREAM_NAME),
        start: Layer0ResumeStart::ResumeFromTheProgressFile,
        after_completion,
    }
}

fn toolchain_of_this_test() -> Layer0ToolchainIdentity {
    Layer0ToolchainIdentity {
        rustc_version_lines: "rustc 1.0.0 (findings test)\nbinary: rustc\ncommit-hash: findings"
            .to_string(),
        cargo_version: "cargo 1.0.0 (findings test)".to_string(),
        target_triple: "x86_64-unknown-linux-gnu".to_string(),
    }
}

/// 用例的观察者：只要崩溃镜像与看 journal 那一遍恢复的报告，不记观察者计数。
type ObserverOfEachState<'observer> = &'observer mut dyn FnMut(&CrashImage<'_>, &RecoveryReport);

/// 跑一趟甲二，发现日志写到 `findings_log_path`；观察者（有的话）逐状态交给调用方。
fn enumerate_the_quick_tier_writing_the_findings_log(
    stream: &FirstStream,
    parallelism: Layer0Parallelism,
    observe_state: Option<ObserverOfEachState<'_>>,
    resume: &Layer0Resume,
    findings_log_path: &Path,
) -> Layer0EnumerationOutcome {
    let versions = versions_claiming_a_warm_up_file_and_the_wrong_content();
    let mut observe_without_counting = observe_state.map(|observe| {
        move |image: &CrashImage<'_>,
              report: &RecoveryReport,
              _counts: &mut Layer0ObserverCounts| {
            observe(image, report);
        }
    });
    enumerate_layer0_in_state_slices_or_one_shard_with_findings_log(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.root_index,
        &versions,
        &quick_tier_expansion,
        parallelism,
        observe_without_counting
            .as_mut()
            .map(|observe| observe as Layer0StateObserver<'_>),
        resume,
        &Layer0FindingsLog::AppendedTo(findings_log_path.to_path_buf()),
    )
}

fn whole_stream(outcome: Layer0EnumerationOutcome) -> Layer0Tally {
    match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => {
            panic!("要整条流的计数，拿到的是第 {:?} 片的账本", written.shard)
        }
    }
}

/// 与枚举器分开算的每个状态的段与发布：段按甲二逐段数状态数、首尾相接（最后多一个每一段都整段持久的状态），发布照
/// `publish_of_each_segment`。
fn segment_and_publish_by_ordinal(
    stream: &FirstStream,
) -> Vec<(Layer0SegmentOfState, Layer0PublishOfState)> {
    let tearable = TearableInPlaceOverwrites::of(&stream.base, &stream.writes);
    let publish_of_segment = publish_of_each_segment(&stream.writes, &stream.segments);
    let mut by_ordinal = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        let states_of_the_segment = quick_tier_expansion(segment_index, segment).state_count(
            &stream.writes,
            segment,
            &tearable,
        );
        for _ in 0..states_of_the_segment {
            by_ordinal.push((
                Layer0SegmentOfState::Segment(segment_index),
                publish_of_segment[segment_index],
            ));
        }
    }
    by_ordinal.push((
        Layer0SegmentOfState::AllPersisted,
        Layer0PublishOfState::EveryWritePersisted,
    ));
    by_ordinal
}

/// 观察者逐状态重判出来的：每个签名下判红的状态（序号从小到大）与各自那一遍给的原因；至少一遍判红的状态数。
#[derive(Default)]
struct RejudgedFindings {
    red_states_by_signature: BTreeMap<Layer0FindingSignature, Vec<(u64, String)>>,
    red_states: u64,
}

impl RejudgedFindings {
    /// 签名按最先那个状态的序号排，同一个状态上的几个签名按签名的大小次序（发现日志里的号就是这个次序）。
    fn signatures_in_first_state_order(
        &self,
    ) -> Vec<(&Layer0FindingSignature, &Vec<(u64, String)>)> {
        let mut ordered: Vec<_> = self.red_states_by_signature.iter().collect();
        ordered.sort_by_key(|(signature, red_states)| (red_states[0].0, (*signature).clone()));
        ordered
    }
}

/// 逐状态重判：两遍恢复各过一次 oracle、池级 checker、记录核对器，按（遍、违了哪几条、段、发布）分组。
fn rejudge_state(
    ordinal: u64,
    image: &CrashImage<'_>,
    consulted_report: &RecoveryReport,
    segment_and_publish: (Layer0SegmentOfState, Layer0PublishOfState),
    rejudged: &mut RejudgedFindings,
) {
    let versions = versions_claiming_a_warm_up_file_and_the_wrong_content();
    let newest = newest_persisted_root(image.writes, &image.persisted);
    let mut red_passes: Vec<(Layer0RedPass, String)> = Vec::new();
    if let Some(violation) = classified_oracle_violation_for_versions(
        &consulted_report.outcome,
        consulted_report.effective_root,
        newest,
        &versions,
    ) {
        red_passes.push((
            Layer0RedPass::JournalConsultedOracle(violation.kind),
            violation.reason,
        ));
    }
    let ignored_report = recover(image, JournalPolicy::Ignore);
    if let Some(violation) = classified_oracle_violation_for_versions(
        &ignored_report.outcome,
        ignored_report.effective_root,
        newest,
        &versions,
    ) {
        red_passes.push((
            Layer0RedPass::JournalIgnoredOracle(violation.kind),
            violation.reason,
        ));
    }
    let violated_invariants: Vec<&'static str> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_detail) => Some(invariant),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    if !violated_invariants.is_empty() {
        red_passes.push((
            Layer0RedPass::PoolChecker(violated_invariants),
            String::new(),
        ));
    }
    let record_check = check_records(image, consulted_report.effective_root);
    if record_check.root_without_record || record_check.claimed_state_missing_unit {
        red_passes.push((Layer0RedPass::RecordChecker(record_check), String::new()));
    }
    if !red_passes.is_empty() {
        rejudged.red_states += 1;
    }
    let (segment, publish) = segment_and_publish;
    for (red_pass, reason) in red_passes {
        rejudged
            .red_states_by_signature
            .entry(Layer0FindingSignature {
                red_pass,
                segment,
                publish,
            })
            .or_default()
            .push((ordinal, reason));
    }
}

/// 发现日志里一行拆成种类与 `key=value` 表（按制表符切、按第一个 `=` 切）。
fn parsed_line(line: &str) -> (String, BTreeMap<String, String>) {
    let mut fields = line.split('\t');
    let kind = fields.next().expect("行至少有种类").to_string();
    let values = fields
        .map(|field| {
            let (key, value) = field
                .split_once('=')
                .unwrap_or_else(|| panic!("字段要是 key=value：{field:?}（{line:?}）"));
            (key.to_string(), value.to_string())
        })
        .collect();
    (kind, values)
}

/// 发现日志里签名那四样的期望写法，照报告第一节的接口现拼（不借实现里的名字函数）。
fn expected_signature_fields(signature: &Layer0FindingSignature) -> [(String, String); 4] {
    let oracle_kind_name = |kind: Layer0OracleViolationKind| match kind {
        Layer0OracleViolationKind::NoRootChosen => "no_root_chosen",
        Layer0OracleViolationKind::RecoveredToAnOlderRoot => "recovered_to_an_older_root",
        Layer0OracleViolationKind::ReadFailed => "read_failed",
        Layer0OracleViolationKind::NoFileWhereAnOlderRootHasOne => {
            "no_file_where_an_older_root_has_one"
        }
        Layer0OracleViolationKind::NoFileWhereThisRootHasOne => "no_file_where_this_root_has_one",
        Layer0OracleViolationKind::FileReadWhereThisRootHasNone => {
            "file_read_where_this_root_has_none"
        }
        Layer0OracleViolationKind::WrongContent => "wrong_content",
    };
    let (pass, violated) = match &signature.red_pass {
        Layer0RedPass::JournalConsultedOracle(kind) => (
            "journal_consulted_oracle",
            oracle_kind_name(*kind).to_string(),
        ),
        Layer0RedPass::JournalIgnoredOracle(kind) => (
            "journal_ignored_oracle",
            oracle_kind_name(*kind).to_string(),
        ),
        Layer0RedPass::PoolChecker(invariants) => ("pool_checker", invariants.join(",")),
        Layer0RedPass::RecordChecker(record_check) => (
            "record_checker",
            [
                (record_check.root_without_record, "root_without_record"),
                (
                    record_check.claimed_state_missing_unit,
                    "claimed_state_missing_unit",
                ),
            ]
            .iter()
            .filter(|(holds, _name)| *holds)
            .map(|(_holds, name)| *name)
            .collect::<Vec<&str>>()
            .join(","),
        ),
    };
    let segment = match signature.segment {
        Layer0SegmentOfState::Segment(segment_index) => segment_index.to_string(),
        Layer0SegmentOfState::AllPersisted => "all_persisted".to_string(),
    };
    let publish = match signature.publish {
        Layer0PublishOfState::UpToTheRootOf {
            root_write_index,
            instance,
            checkpoint_txg,
        } => format!(
            "instance{}_txg{}_root_write{root_write_index}",
            instance.0, checkpoint_txg.0
        ),
        Layer0PublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
        Layer0PublishOfState::EveryWritePersisted => "every_write_persisted".to_string(),
    };
    [
        ("pass".to_string(), pass.to_string()),
        ("violated".to_string(), violated),
        ("segment".to_string(), segment),
        ("publish".to_string(), publish),
    ]
}

/// 发现表与重判逐项对得上：签名集合、每个签名的状态数、最先 3 个状态的序号，样本原文以那一遍给的原因打头；判红的状态数。
fn assert_findings_match_the_rejudged(findings: &Layer0Findings, rejudged: &RejudgedFindings) {
    let findings_view: BTreeMap<&Layer0FindingSignature, (u64, Vec<u64>)> = findings
        .by_signature
        .iter()
        .map(|(signature, finding)| {
            (
                signature,
                (
                    finding.states,
                    finding
                        .earliest_samples
                        .iter()
                        .map(|sample| sample.state_ordinal)
                        .collect(),
                ),
            )
        })
        .collect();
    let rejudged_view: BTreeMap<&Layer0FindingSignature, (u64, Vec<u64>)> = rejudged
        .red_states_by_signature
        .iter()
        .map(|(signature, red_states)| {
            (
                signature,
                (
                    u64::try_from(red_states.len()).expect("状态数"),
                    red_states
                        .iter()
                        .take(3)
                        .map(|(ordinal, _reason)| *ordinal)
                        .collect(),
                ),
            )
        })
        .collect();
    assert_eq!(
        findings_view, rejudged_view,
        "每个签名的状态数与最先 3 个状态，与逐状态重判的逐项相同"
    );
    assert_eq!(
        findings.red_states, rejudged.red_states,
        "至少一遍判红的状态数"
    );
    for (signature, finding) in &findings.by_signature {
        for (sample, (_ordinal, reason)) in finding
            .earliest_samples
            .iter()
            .zip(&rejudged.red_states_by_signature[signature])
        {
            assert!(
                sample.violation.starts_with(reason.as_str()),
                "样本原文以那一遍给的原因打头：{:?} / {reason:?}",
                sample.violation
            );
        }
    }
}

/// 发现日志只有这一趟那一节、且是定稿：begin 行、每个签名一行（号、签名四样、状态数、最先几个状态与原文）、汇总行，逐行对重判。
fn assert_final_section_matches_the_rejudged(
    log_text: &str,
    rejudged: &RejudgedFindings,
    expected_begin_line: &str,
) {
    let lines: Vec<&str> = log_text.lines().collect();
    let ordered = rejudged.signatures_in_first_state_order();
    assert_eq!(
        lines.len(),
        ordered.len() + 2,
        "begin 行 + 每个签名一行 + 汇总行：\n{log_text}"
    );
    assert!(log_text.ends_with('\n'), "每行带换行");
    assert_eq!(lines[0], expected_begin_line);
    for (position, (signature, red_states)) in ordered.iter().enumerate() {
        let (kind, fields) = parsed_line(lines[position + 1]);
        assert_eq!(kind, "layer0_finding", "{}", lines[position + 1]);
        let mut expected: BTreeMap<String, String> =
            expected_signature_fields(signature).into_iter().collect();
        expected.insert("finding".to_string(), (position + 1).to_string());
        expected.insert("states".to_string(), red_states.len().to_string());
        expected.insert(
            "sample_states".to_string(),
            red_states
                .iter()
                .take(3)
                .map(|(ordinal, _reason)| ordinal.to_string())
                .collect::<Vec<String>>()
                .join(","),
        );
        for (sample_position, (_ordinal, reason)) in red_states.iter().take(3).enumerate() {
            let key = format!("sample_violation_{}", sample_position + 1);
            let written = unescaped_finding_text(
                fields
                    .get(&key)
                    .unwrap_or_else(|| panic!("缺 {key}：{}", lines[position + 1])),
            )
            .expect("原文的转义解得开");
            assert!(
                written.starts_with(reason.as_str()),
                "{key}：{written:?} / {reason:?}"
            );
            expected.insert(key.clone(), fields[&key].clone());
        }
        assert_eq!(fields, expected, "第 {} 个签名那一行", position + 1);
    }
    let (summary_kind, summary) = parsed_line(lines[lines.len() - 1]);
    assert_eq!(summary_kind, "layer0_findings_summary");
    let states_by_finding: Vec<String> = ordered
        .iter()
        .enumerate()
        .map(|(position, (_signature, red_states))| {
            format!("{}:{}", position + 1, red_states.len())
        })
        .collect();
    assert_eq!(
        summary,
        BTreeMap::from([
            ("signatures".to_string(), ordered.len().to_string()),
            ("red_states".to_string(), rejudged.red_states.to_string()),
            ("states".to_string(), QUICK_TIER_STATES.to_string()),
            ("states_by_finding".to_string(), states_by_finding.join(",")),
        ]),
        "汇总行"
    );
}

fn begin_line(stream: &str) -> String {
    format!("layer0_findings_begin\tformat=1\tstream={stream}\tstates={QUICK_TIER_STATES}")
}

/// 先红后改的那一条：两类违例、两遍恢复、好几段里都红，今天的计数只留一条 `first_violation`；发现表按签名去重，每个签名的状态数、
/// 最先 3 个状态与逐状态重判的相同，发现日志跑完是定稿（一个签名一行、汇总行的数对得上）；跑的过程中每并进一片，这一片带来的
/// 新签名、跨过 10 的签名已经落在文件里（观察者在下一片并进来之前读得到），同一签名只报一次「新」。
#[test]
fn red_states_are_deduplicated_by_signature_and_the_findings_log_is_readable_while_running() {
    let stream = first_stream("findings-dedupe");
    let directory = fresh_directory("dedupe");
    let findings_log_path = directory.join("findings.tsv");
    let segment_and_publish = segment_and_publish_by_ordinal(&stream);
    assert_eq!(
        u64::try_from(segment_and_publish.len()).expect("状态数"),
        QUICK_TIER_STATES
    );
    let mut rejudged = RejudgedFindings::default();
    let mut log_before_each_state: Vec<String> = Vec::new();
    let mut next_ordinal = 0u64;
    let tally = whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        states_per_slice_on(1, 4),
        Some(&mut |image: &CrashImage<'_>, report: &RecoveryReport| {
            log_before_each_state.push(
                std::fs::read_to_string(&findings_log_path).expect("跑的过程中读得到发现日志"),
            );
            let ordinal = next_ordinal;
            next_ordinal += 1;
            rejudge_state(
                ordinal,
                image,
                report,
                segment_and_publish[usize::try_from(ordinal).expect("序号")],
                &mut rejudged,
            );
        }),
        &Layer0Resume::NoProgressFile,
        &findings_log_path,
    ));
    assert_eq!(tally.states, QUICK_TIER_STATES);
    assert!(
        tally.violations > 1 && tally.first_violation.is_some(),
        "今天的计数：违例 {} 个，只留一条 first_violation",
        tally.violations
    );
    let kinds_and_passes: Vec<(&str, Vec<&str>)> = rejudged
        .red_states_by_signature
        .keys()
        .map(|signature| {
            (
                signature.red_pass.pass_name(),
                signature.red_pass.violated_names(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(
        kinds_and_passes,
        vec![
            (
                "journal_consulted_oracle",
                vec!["no_file_where_this_root_has_one"]
            ),
            ("journal_consulted_oracle", vec!["wrong_content"]),
            (
                "journal_ignored_oracle",
                vec!["no_file_where_this_root_has_one"]
            ),
            ("journal_ignored_oracle", vec!["wrong_content"]),
        ],
        "这份版本表让两遍恢复各红两类，池级 checker 与记录核对器不红"
    );
    let largest_signature_states = rejudged
        .red_states_by_signature
        .values()
        .map(Vec::len)
        .max()
        .expect("有签名");
    assert!(
        rejudged.red_states_by_signature.len() >= 4 && largest_signature_states >= 10,
        "至少四个签名、至少一个签名跨过 10 个状态：{} 个签名，最多的 {largest_signature_states} 个状态",
        rejudged.red_states_by_signature.len()
    );
    assert_findings_match_the_rejudged(&tally.findings, &rejudged);

    let log_text = std::fs::read_to_string(&findings_log_path).expect("读发现日志");
    assert_final_section_matches_the_rejudged(&log_text, &rejudged, &begin_line("unnamed"));

    // 跑的过程中：观察者在第 k 个状态那一片并进来之前读的文件，恰好有序号小于 k 的状态带来的新签名与跨台阶。
    for (position, (signature, red_states)) in rejudged
        .signatures_in_first_state_order()
        .iter()
        .enumerate()
    {
        let fields = expected_signature_fields(signature)
            .iter()
            .map(|(key, value)| format!("\t{key}={value}"))
            .collect::<String>();
        let new_line_prefix = format!(
            "layer0_finding_new\tfinding={}{fields}\tfirst_state={}\t",
            position + 1,
            red_states[0].0
        );
        let first_state = usize::try_from(red_states[0].0).expect("序号");
        let lines_starting_with = |snapshot: &str, prefix: &str| {
            snapshot
                .lines()
                .filter(|line| line.starts_with(prefix))
                .count()
        };
        assert_eq!(
            lines_starting_with(&log_before_each_state[first_state], &new_line_prefix),
            0,
            "最先那个状态并进来之前还没报：{new_line_prefix}"
        );
        if let Some(after) = log_before_each_state.get(first_state + 1) {
            assert_eq!(
                lines_starting_with(after, &new_line_prefix),
                1,
                "最先那个状态并进来之后报了一次：{new_line_prefix}\n{after}"
            );
        }
        if let Some((tenth_ordinal, _reason)) = red_states.get(9) {
            let threshold_line = format!(
                "layer0_finding_threshold\tfinding={}{fields}\tstates_at_least=10",
                position + 1
            );
            let tenth = usize::try_from(*tenth_ordinal).expect("序号");
            assert_eq!(
                lines_starting_with(&log_before_each_state[tenth], &threshold_line),
                0,
                "第 10 个状态并进来之前还没跨台阶"
            );
            let after = &log_before_each_state[tenth + 1];
            assert_eq!(
                lines_starting_with(after, &threshold_line),
                1,
                "第 10 个状态并进来之后报了一次跨过 10：{threshold_line}\n{after}"
            );
        }
    }
    std::fs::remove_dir_all(&directory).ok();
}

/// 同一批状态换线程数（1 与 4 与 2、3）与切法（每片 1 个、7 个、整条流 1 片、按线程数定），发现日志逐字节相同，计数逐项相同；
/// 两趟写进同一个文件是两节，前一节原样留着。
#[test]
fn the_findings_log_is_byte_identical_across_worker_threads_and_slicings() {
    let stream = first_stream("findings-slicings");
    let directory = fresh_directory("slicings");
    let parallelisms = [
        states_per_slice_on(1, 1),
        states_per_slice_on(1, 4),
        states_per_slice_on(7, 3),
        states_per_slice_on(QUICK_TIER_STATES, 2),
        Layer0Parallelism {
            worker_threads: NonZeroUsize::new(4).expect("不是 0"),
            worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
        },
    ];
    let mut logs_and_tallies: Vec<(Vec<u8>, Layer0Tally)> = Vec::new();
    for (run_index, parallelism) in parallelisms.into_iter().enumerate() {
        let findings_log_path = directory.join(format!("findings-{run_index}.tsv"));
        let tally = whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
            &stream,
            parallelism,
            None,
            &Layer0Resume::NoProgressFile,
            &findings_log_path,
        ));
        logs_and_tallies.push((
            std::fs::read(&findings_log_path).expect("读发现日志"),
            tally,
        ));
    }
    let (first_log, first_tally) = &logs_and_tallies[0];
    assert!(
        first_tally.findings.by_signature.len() >= 4,
        "这份版本表下至少四个签名"
    );
    for (run_index, (log, tally)) in logs_and_tallies.iter().enumerate() {
        assert_eq!(
            String::from_utf8_lossy(log),
            String::from_utf8_lossy(first_log),
            "第 {run_index} 趟的发现日志与 1 个线程、每片 1 个状态那一趟逐字节相同"
        );
        assert_eq!(tally, first_tally, "第 {run_index} 趟的计数逐项相同");
    }

    let shared_path = directory.join("two-sections.tsv");
    for parallelism in [states_per_slice_on(1, 4), states_per_slice_on(7, 2)] {
        whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
            &stream,
            parallelism,
            None,
            &Layer0Resume::NoProgressFile,
            &shared_path,
        ));
    }
    let mut twice = first_log.clone();
    twice.extend_from_slice(first_log);
    assert_eq!(
        String::from_utf8_lossy(&std::fs::read(&shared_path).expect("读发现日志")),
        String::from_utf8_lossy(&twice),
        "同一个文件跑两趟：两节各是定稿，前一节原样留着"
    );
    std::fs::remove_dir_all(&directory).ok();
}

/// 把进度文件截成「文件头 + 前 `kept_slice_lines` 行片行 + 下一行的前一半」：跑到一半被杀的样子。
fn cut_short(path: &Path, kept_slice_lines: usize) {
    let content = std::fs::read_to_string(path).expect("读进度文件");
    let lines: Vec<&str> = content.lines().collect();
    let mut cut = lines[..=kept_slice_lines].join("\n");
    cut.push('\n');
    let next_line = lines[kept_slice_lines + 1];
    cut.push_str(&next_line[..next_line.len() / 2]);
    std::fs::write(path, cut).expect("写截断的进度文件");
}

fn only_file_in(directory: &Path) -> PathBuf {
    let files: Vec<PathBuf> = std::fs::read_dir(directory)
        .expect("读目录")
        .map(|entry| entry.expect("目录项").path())
        .collect();
    let [file] = files.try_into().expect("恰好一个文件");
    file
}

/// 续跑：跑到一半被杀之后，读回的片带着它们的发现表并进来（照样报新签名），续跑那一趟的发现日志与一口气跑完的逐字节相同、计数逐项相同。
/// 双机分片：两片各写自己那一片的一节（begin 行带 shard=），merge 那一趟写的定稿与单机跑同一条流的逐字节相同。
#[test]
fn findings_survive_a_resume_from_the_progress_file_and_a_merge_of_shard_ledgers() {
    let stream = first_stream("findings-resume");
    let directory = fresh_directory("resume");
    let progress_directory = directory.join("progress");
    let uninterrupted_log = directory.join("uninterrupted.tsv");
    let uninterrupted = whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        states_per_slice_on(1, 4),
        None,
        &Layer0Resume::KeepProgressFile(progress_settings(
            &progress_directory,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        )),
        &uninterrupted_log,
    ));
    let uninterrupted_text = std::fs::read_to_string(&uninterrupted_log).expect("读发现日志");
    assert!(
        uninterrupted_text.starts_with(&format!("{}\n", begin_line(STREAM_NAME))),
        "{uninterrupted_text}"
    );
    let kept_slice_lines = 30;
    let signatures_first_seen_in_the_kept_slices = uninterrupted
        .findings
        .by_signature
        .values()
        .filter(|finding| {
            finding.first_sample().state_ordinal < u64::try_from(kept_slice_lines).expect("片数")
        })
        .count();
    assert!(
        signatures_first_seen_in_the_kept_slices > 0,
        "读回的那 {kept_slice_lines} 片（每片一个状态）里就有签名第一次出现：续跑要从进度文件把它们读回来"
    );
    cut_short(&only_file_in(&progress_directory), kept_slice_lines);
    let resumed_log = directory.join("resumed.tsv");
    let resumed = whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        states_per_slice_on(1, 3),
        None,
        &Layer0Resume::KeepProgressFile(progress_settings(
            &progress_directory,
            Layer0ProgressFileAfterCompletion::Deleted,
        )),
        &resumed_log,
    ));
    assert_eq!(
        resumed, uninterrupted,
        "续跑之后的计数（连发现表）与一口气跑完的逐项相同"
    );
    assert_eq!(
        std::fs::read_to_string(&resumed_log).expect("读发现日志"),
        uninterrupted_text,
        "续跑那一趟的发现日志与一口气跑完的逐字节相同"
    );

    let machines: Vec<PathBuf> = (0..2)
        .map(|machine| directory.join(format!("machine-{machine}")))
        .collect();
    let mut shard_logs: Vec<String> = Vec::new();
    for (shard_index, machine_directory) in machines.iter().enumerate() {
        let shard = Layer0ShardOfShards::new(
            u32::try_from(shard_index).expect("片号"),
            NonZeroU32::new(2).expect("不是 0"),
        )
        .expect("第几片小于片数");
        let shard_log = machine_directory.join("findings.tsv");
        let outcome = enumerate_the_quick_tier_writing_the_findings_log(
            &stream,
            states_per_slice_on(1, 2),
            None,
            &Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
                progress: progress_settings(
                    machine_directory,
                    Layer0ProgressFileAfterCompletion::Deleted,
                ),
                shard,
                toolchain: toolchain_of_this_test(),
            }),
            &shard_log,
        );
        assert!(
            matches!(
                outcome,
                Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_)
            ),
            "分片跑一片写账本"
        );
        shard_logs.push(std::fs::read_to_string(&shard_log).expect("读这一片的发现日志"));
    }
    for (shard_index, shard_log) in shard_logs.iter().enumerate() {
        let first_line = shard_log.lines().next().expect("有 begin 行");
        assert!(
            first_line.starts_with(&format!(
                "{}\tshard={shard_index}/2\tshard_states=",
                begin_line(STREAM_NAME)
            )),
            "分片跑一片的 begin 行带 shard=：{first_line}"
        );
        assert!(
            shard_log
                .lines()
                .last()
                .expect("有汇总行")
                .starts_with("layer0_findings_summary\t"),
            "这一片跑完是定稿：{shard_log}"
        );
    }
    let merge_directory = directory.join("merge");
    std::fs::create_dir_all(&merge_directory).expect("建 merge 目录");
    for machine_directory in &machines {
        for entry in std::fs::read_dir(machine_directory).expect("读这一台的目录") {
            let path = entry.expect("目录项").path();
            if path
                .extension()
                .is_some_and(|extension| extension == "tally")
            {
                std::fs::copy(
                    &path,
                    merge_directory.join(path.file_name().expect("文件名")),
                )
                .expect("把账本拷到 merge 那一台");
            }
        }
    }
    let merged_log = merge_directory.join("findings.tsv");
    let merged = whole_stream(enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        states_per_slice_on(1, 1),
        None,
        &Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
            directory: merge_directory.clone(),
            input_fingerprint: name_part("fingerprint0"),
            stream_name: name_part(STREAM_NAME),
            shard_count: NonZeroU32::new(2).expect("不是 0"),
            toolchain: toolchain_of_this_test(),
        }),
        &merged_log,
    ));
    assert_eq!(
        merged.findings, uninterrupted.findings,
        "merge 并出来的发现表与单机的逐项相同"
    );
    assert_eq!(
        std::fs::read_to_string(&merged_log).expect("读 merge 的发现日志"),
        uninterrupted_text,
        "merge 那一趟写的定稿与单机跑同一条流的逐字节相同"
    );
    std::fs::remove_dir_all(&directory).ok();
}
