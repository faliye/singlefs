//! checker 档模块：crash、crash_state_enumeration、crash_point_replay_progress、crash_library、crash_amplification
//! 崩溃点重放放量的发现日志（里程碑「覆盖写、释放、回退与复用」收尾批「崩溃点重放发现日志」）：在新池新建文件流上（mkfs → 取号 → 暖机 → A，按甲二展开，
//! 原地覆写取三态，46 个状态；C577 之前 54 个）把版本表故意写错——暖机第二次发布（实例 1、txg 2）下面说有文件、A（txg 3）的内容说成别的——
//! 落在 txg 2 的状态报「有文件却报没有」，读出 A 的状态报「读回的内容不对」，两遍恢复（看 journal、不看 journal）各红几段。
//! 判段从崩溃判定库里读回的结论重建发现表：按签名去重之后是几个签名、各几个状态、最先是哪几个状态，拿一份与判器分开写的 CPU 重判来核
//! （重判也走两段：同一条流经 CPU 取样那一路写进崩溃点重放枚举那一类的库、读回来逐状态重判，不在录入段的内存里判），再核发现日志那一节定稿的
//! 每一行；同一条流再跑一遍全照库里复用，发现日志逐字节相同，观察者的计数从库里读回。
//! 它不是崩溃点重放全量那一类用例：只跑甲二那 46 个状态，平时跑得起。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    check_records, classified_oracle_violation_for_versions, publish_of_each_segment,
    quick_tier_expansion, CrashPointReplayFindingSignature, CrashPointReplayFindings,
    CrashPointReplayObserverCounts, CrashPointReplayOracleViolationKind,
    CrashPointReplayPublishOfState, CrashPointReplayRedPass, CrashPointReplaySegmentOfState,
    CrashPointReplayTally, CrashPointReplayVersionTable, TearableInPlaceOverwrites,
};
use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
use singlefs_checker_tier::crash_library::{
    DisposableLibraryDirectory, CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
};
use singlefs_checker_tier::crash_point_replay_progress::{
    unescaped_finding_text, CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion,
    sample_crash_states_on_the_cpu_against_the_version_table, CrashPointReplayItem,
    CrashPointReplayObserver,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs";
/// 新池新建文件流按甲二展开的状态数（`crash_enumeration_new_pool_file_creation_stream.rs` 的 `QUICK_TIER_STATES`，那里写了怎么数出来的）。
const QUICK_TIER_STATES: u64 = 46;
/// 发现日志 begin 行里的流名：就是这一项的项名。
const ITEM_NAME: &str = "findings_new_pool_file_creation_stream";
/// 逐状态重判那一趟（CPU 取样那一路）的项名：与枚举那一项分开登清单、分开存取样的结论。
const REJUDGED_ITEM_NAME: &str = "findings_new_pool_file_creation_stream_rejudged_on_the_cpu";

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

/// 每个用例一个自己的一次性库目录（`DisposableLibraryDirectory`：起跑是空的，丢掉时连同里面的库与发现日志只删这一份）。这几条用例要从空库起跑
/// （用户 2026-10-15 定：空库起跑的单测只在一次性副本里建库，不碰长期库）：再跑那一条钉头一趟现判、第二趟照库里复用，观察者换版本那一条要
/// 第二趟的观察者在库里一块都没有；几条用例同一条流、同一个项名，流的录入键只由流定，用一份库就会读到别的用例先录下的流，所以各用各的库。
fn disposable_library_directory(test_name: &str) -> DisposableLibraryDirectory {
    DisposableLibraryDirectory::new(&format!("crash-enumeration-findings-{test_name}"))
        .expect("建得了一次性库目录")
}

/// 按甲二跑这一条流一遍（版本表故意写错），发现日志写到 `findings_log_path`（判段写一节定稿），库是 `library` 那一份一次性库，
/// 挂上 `observers`（没有就空）。同一个二进制里改进程环境的用例一条接一条跑（环境守卫里设发现日志与库目录两个变量）。
/// 只录、或配置开了 GPU 而没编进 gpu 特性（判段不跑）的那一趟交 `None`。
fn enumerate_the_quick_tier_writing_the_findings_log(
    stream: &FirstStream,
    findings_log_path: &std::path::Path,
    library: &DisposableLibraryDirectory,
    observers: &[CrashPointReplayObserver<'_>],
) -> Option<CrashPointReplayTally> {
    let versions = versions_claiming_a_warm_up_file_and_the_wrong_content();
    let library_directory = library.library();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE,
            findings_log_path.as_os_str(),
        ),
        (
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            library_directory.as_os_str(),
        ),
    ]);
    let tally = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: ITEM_NAME,
            setting_file: THIS_FILE,
        },
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.root_index,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        &quick_tier_expansion,
        observers,
    )
    .judged();
    drop(environment);
    tally
}

/// 与判器分开写的逐状态重判，也走两段：这一条流（同一份故意写错的版本表、同样按甲二展开）经 CPU 取样那一路写进崩溃点重放枚举那一类的库、
/// 判段从库里读回，每个状态的镜像都是读回的那一份，交 [`rejudge_state`] 重判（不在录入段的内存里判，乙报告验收 3 末尾）。库是 `library`
/// 那一份一次性库（环境守卫里设库目录）。只录那一趟交 `None`。
fn rejudged_findings_of_the_states_read_back_from_the_library(
    stream: &FirstStream,
    library: &DisposableLibraryDirectory,
) -> Option<RejudgedFindings> {
    let versions = versions_claiming_a_warm_up_file_and_the_wrong_content();
    let segment_and_publish = segment_and_publish_by_ordinal(stream);
    assert_eq!(
        u64::try_from(segment_and_publish.len()).expect("状态数"),
        QUICK_TIER_STATES
    );
    let mut rejudged = RejudgedFindings::default();
    let mut states_rejudged = 0u64;
    let library_directory = library.library();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
        library_directory.as_os_str(),
    )]);
    let states_of_the_flow = sample_crash_states_on_the_cpu_against_the_version_table(
        CrashPointReplayItem {
            name: REJUDGED_ITEM_NAME,
            setting_file: THIS_FILE,
        },
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.root_index,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        &quick_tier_expansion,
        0..QUICK_TIER_STATES,
        &mut |ordinal, image, _cpu_judgement| {
            states_rejudged += 1;
            rejudge_state(
                ordinal,
                image,
                segment_and_publish[usize::try_from(ordinal).expect("序号")],
                &mut rejudged,
            );
        },
    )
    .judged();
    drop(environment);
    let states_of_the_flow = states_of_the_flow?;
    assert_eq!(states_of_the_flow, QUICK_TIER_STATES);
    assert_eq!(
        states_rejudged, QUICK_TIER_STATES,
        "每个状态都从库里读回重判一次"
    );
    for red_states in rejudged.red_states_by_signature.values_mut() {
        red_states.sort_by_key(|(ordinal, _reason)| *ordinal);
    }
    Some(rejudged)
}

/// 与枚举器分开算的每个状态的段与发布：段按甲二逐段数状态数、首尾相接（最后多一个每一段都整段持久的状态），发布照
/// `publish_of_each_segment`。
fn segment_and_publish_by_ordinal(
    stream: &FirstStream,
) -> Vec<(
    CrashPointReplaySegmentOfState,
    CrashPointReplayPublishOfState,
)> {
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
                CrashPointReplaySegmentOfState::Segment(segment_index),
                publish_of_segment[segment_index],
            ));
        }
    }
    by_ordinal.push((
        CrashPointReplaySegmentOfState::AllPersisted,
        CrashPointReplayPublishOfState::EveryWritePersisted,
    ));
    by_ordinal
}

/// 观察者逐状态重判出来的：每个签名下判红的状态（序号从小到大）与各自那一遍给的原因；至少一遍判红的状态数。
#[derive(Default)]
struct RejudgedFindings {
    red_states_by_signature: BTreeMap<CrashPointReplayFindingSignature, Vec<(u64, String)>>,
    red_states: u64,
}

impl RejudgedFindings {
    /// 签名按最先那个状态的序号排，同一个状态上的几个签名按签名的大小次序（发现日志里的号就是这个次序）。
    fn signatures_in_first_state_order(
        &self,
    ) -> Vec<(&CrashPointReplayFindingSignature, &Vec<(u64, String)>)> {
        let mut ordered: Vec<_> = self.red_states_by_signature.iter().collect();
        ordered.sort_by_key(|(signature, red_states)| (red_states[0].0, (*signature).clone()));
        ordered
    }
}

/// 逐状态重判：两遍恢复各过一次 oracle、池级 checker、记录核对器，按（遍、违了哪几条、段、发布）分组。镜像是从库里读回的那一份
/// （[`rejudged_findings_of_the_states_read_back_from_the_library`]）；判段配了 GPU 时这就是与 GPU 判器的发现表对的 CPU 参照。
// cpu-reference-for-gpu-comparison
fn rejudge_state(
    ordinal: u64,
    image: &CrashImage<'_>,
    segment_and_publish: (
        CrashPointReplaySegmentOfState,
        CrashPointReplayPublishOfState,
    ),
    rejudged: &mut RejudgedFindings,
) {
    let versions = versions_claiming_a_warm_up_file_and_the_wrong_content();
    let consulted_report = recover(image, JournalPolicy::Consult);
    let newest = newest_persisted_root(image.writes, &image.persisted);
    let mut red_passes: Vec<(CrashPointReplayRedPass, String)> = Vec::new();
    if let Some(violation) = classified_oracle_violation_for_versions(
        &consulted_report.outcome,
        consulted_report.effective_root,
        newest,
        &versions,
    ) {
        red_passes.push((
            CrashPointReplayRedPass::JournalConsultedOracle(violation.kind),
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
            CrashPointReplayRedPass::JournalIgnoredOracle(violation.kind),
            violation.reason,
        ));
    }
    let violated_invariants: Vec<&'static str> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_detail) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    if !violated_invariants.is_empty() {
        red_passes.push((
            CrashPointReplayRedPass::PoolChecker(violated_invariants),
            String::new(),
        ));
    }
    let record_check = check_records(image, consulted_report.effective_root);
    if record_check.root_without_record || record_check.claimed_state_missing_unit {
        red_passes.push((
            CrashPointReplayRedPass::RecordChecker(record_check),
            String::new(),
        ));
    }
    if !red_passes.is_empty() {
        rejudged.red_states += 1;
    }
    let (segment, publish) = segment_and_publish;
    for (red_pass, reason) in red_passes {
        rejudged
            .red_states_by_signature
            .entry(CrashPointReplayFindingSignature {
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
fn expected_signature_fields(
    signature: &CrashPointReplayFindingSignature,
) -> [(String, String); 4] {
    let oracle_kind_name = |kind: CrashPointReplayOracleViolationKind| match kind {
        CrashPointReplayOracleViolationKind::NoRootChosen => "no_root_chosen",
        CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot => "recovered_to_an_older_root",
        CrashPointReplayOracleViolationKind::ReadFailed => "read_failed",
        CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne => {
            "no_file_where_an_older_root_has_one"
        }
        CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne => {
            "no_file_where_this_root_has_one"
        }
        CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone => {
            "file_read_where_this_root_has_none"
        }
        CrashPointReplayOracleViolationKind::WrongContent => "wrong_content",
    };
    let (pass, violated) = match &signature.red_pass {
        CrashPointReplayRedPass::JournalConsultedOracle(kind) => (
            "journal_consulted_oracle",
            oracle_kind_name(*kind).to_string(),
        ),
        CrashPointReplayRedPass::JournalIgnoredOracle(kind) => (
            "journal_ignored_oracle",
            oracle_kind_name(*kind).to_string(),
        ),
        CrashPointReplayRedPass::PoolChecker(invariants) => ("pool_checker", invariants.join(",")),
        CrashPointReplayRedPass::RecordChecker(record_check) => (
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
        CrashPointReplaySegmentOfState::Segment(segment_index) => segment_index.to_string(),
        CrashPointReplaySegmentOfState::AllPersisted => "all_persisted".to_string(),
    };
    let publish = match signature.publish {
        CrashPointReplayPublishOfState::UpToTheRootOf {
            root_write_index,
            instance,
            checkpoint_txg,
        } => format!(
            "instance{}_txg{}_root_write{root_write_index}",
            instance.0, checkpoint_txg.0
        ),
        CrashPointReplayPublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
        CrashPointReplayPublishOfState::EveryWritePersisted => "every_write_persisted".to_string(),
    };
    [
        ("pass".to_string(), pass.to_string()),
        ("violated".to_string(), violated),
        ("segment".to_string(), segment),
        ("publish".to_string(), publish),
    ]
}

/// 发现表与重判逐项对得上：签名集合、每个签名的状态数、最先 3 个状态的序号，样本原文以那一遍给的原因打头；判红的状态数。
fn assert_findings_match_the_rejudged(
    findings: &CrashPointReplayFindings,
    rejudged: &RejudgedFindings,
) {
    let findings_view: BTreeMap<&CrashPointReplayFindingSignature, (u64, Vec<u64>)> = findings
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
    let rejudged_view: BTreeMap<&CrashPointReplayFindingSignature, (u64, Vec<u64>)> = rejudged
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
        let expected_sample = red_item_text_of(signature);
        for sample in &finding.earliest_samples {
            assert_eq!(
                sample.violation, expected_sample,
                "样本原文是库里存的红项（`<pass>:<违了哪几条>`）"
            );
        }
    }
}

/// 库里存的红项的写法：`<pass>:<违了哪几条，逗号接>`（发现表的样本原文与发现日志的 `sample_violation_*` 都是它）。
fn red_item_text_of(signature: &CrashPointReplayFindingSignature) -> String {
    let fields = expected_signature_fields(signature);
    format!("{}:{}", fields[0].1, fields[1].1)
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
        assert_eq!(
            kind,
            "crash_point_replay_finding",
            "{}",
            lines[position + 1]
        );
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
        for sample_position in 0..red_states.len().min(3) {
            let key = format!("sample_violation_{}", sample_position + 1);
            let written = unescaped_finding_text(
                fields
                    .get(&key)
                    .unwrap_or_else(|| panic!("缺 {key}：{}", lines[position + 1])),
            )
            .expect("原文的转义解得开");
            assert_eq!(written, red_item_text_of(signature), "{key}");
            expected.insert(key.clone(), fields[&key].clone());
        }
        assert_eq!(fields, expected, "第 {} 个签名那一行", position + 1);
    }
    let (summary_kind, summary) = parsed_line(lines[lines.len() - 1]);
    assert_eq!(summary_kind, "crash_point_replay_findings_summary");
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
    format!(
        "crash_point_replay_findings_begin\tformat=1\tstream={stream}\tstates={QUICK_TIER_STATES}"
    )
}

/// 先红后改的那一条：两类违例、两遍恢复、好几段里都红，今天的计数只留一条 `first_violation`；判段从库里读回的结论重建的发现表按签名去重，
/// 每个签名的状态数、最先 3 个状态与逐状态重判的相同，发现日志那一节是定稿（begin 行、一个签名一行、汇总行的数对得上）。
/// 重判每一趟都从库里读回、逐状态现判（[`rejudged_findings_of_the_states_read_back_from_the_library`]），判段的结论复用了也照样对得上。
#[test]
fn red_states_are_deduplicated_by_signature_and_the_findings_log_section_is_final() {
    let stream = first_stream("findings-dedupe");
    let directory = disposable_library_directory("dedupe");
    let findings_log_path = directory.join("findings.tsv");
    let Some(tally) = enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        &findings_log_path,
        &directory,
        &[],
    ) else {
        return;
    };
    let Some(rejudged) =
        rejudged_findings_of_the_states_read_back_from_the_library(&stream, &directory)
    else {
        return;
    };
    assert_eq!(tally.states, QUICK_TIER_STATES);
    assert!(
        tally.violations > 1 && tally.first_violation.is_some(),
        "今天的计数：违例 {} 个，只留一条 first_violation",
        tally.violations
    );
    let kinds_and_passes: Vec<(&str, Vec<&str>)> = tally
        .findings
        .by_signature
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
    let largest_signature_states = tally
        .findings
        .by_signature
        .values()
        .map(|finding| finding.states)
        .max()
        .expect("有签名");
    // 钉值出处：2026-10-04 主工作区 checker 档日志 `CRASH_POINT_REPLAY_FINDINGS signatures=15 red_states=25 states=46`
    assert!(
        tally.findings.by_signature.len() >= 4 && largest_signature_states >= 8,
        "至少四个签名、至少一个签名跨过 8 个状态：{} 个签名，最多的 {largest_signature_states} 个状态",
        tally.findings.by_signature.len()
    );
    assert_findings_match_the_rejudged(&tally.findings, &rejudged);
    let log_text = std::fs::read_to_string(&findings_log_path).expect("读发现日志");
    assert_final_section_matches_the_rejudged(&log_text, &rejudged, &begin_line(ITEM_NAME));
    drop(directory);
}

/// 观察者「每个状态数一次」（`count_each_state_once`）：只给每个状态记一次 `seen`。观察者的身份是这个 fn 的闭包键（入口登记在
/// `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
fn count_each_state_once(
    _ordinal: u64,
    _image: &CrashImage<'_>,
    _report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    counts.add("seen", 1);
}

/// 同一条流再跑一遍：块与观察者的结论全照库里复用，计数逐项相同、发现日志那一节逐字节相同；两趟写进同一个文件是两节，前一节原样留着。
/// 观察者两段（用户 2026-10-13 定）：录入段逐状态看、计数存进库，判段从库里读回。从空库起跑（一次性库）：头一趟现判、现录，第二趟起照库里复用；
/// 同名的观察者在别的用例里另有一份库，不在用例之间复用。
#[test]
fn the_findings_log_is_byte_identical_when_rerun_from_the_library() {
    let stream = first_stream("findings-rerun");
    let directory = disposable_library_directory("rerun");
    let observe = count_each_state_once;
    let observers = [CrashPointReplayObserver {
        name: "count_each_state_once",
        // 这个观察者不另读流程输入：只给每个状态记一次（另读的输入摘要照一句说明取，不留全零）
        extra_inputs_digest: sha256_digest(b"no extra inputs: the observer counts each state once"),
        observe: &observe,
    }];
    let first_path = directory.join("first.tsv");
    let Some(first) = enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        &first_path,
        &directory,
        &observers,
    ) else {
        return;
    };
    let second_path = directory.join("second.tsv");
    let Some(second) = enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        &second_path,
        &directory,
        &observers,
    ) else {
        return;
    };
    assert_eq!(second, first, "第二遍全照库里复用，计数逐项相同");
    assert_eq!(
        second.observer_counts.count_named("seen"),
        QUICK_TIER_STATES,
        "观察者的计数从库里读回：每个状态一次"
    );
    let first_text = std::fs::read_to_string(&first_path).expect("读第一趟的发现日志");
    let second_text = std::fs::read_to_string(&second_path).expect("读第二趟的发现日志");
    assert_eq!(second_text, first_text, "发现日志那一节逐字节相同");
    assert!(
        first_text.starts_with(&begin_line(ITEM_NAME)),
        "{first_text}"
    );
    // 两趟写进同一个文件是两节
    let both_path = directory.join("both.tsv");
    let _ = enumerate_the_quick_tier_writing_the_findings_log(
        &stream, &both_path, &directory, &observers,
    );
    let _ = enumerate_the_quick_tier_writing_the_findings_log(
        &stream, &both_path, &directory, &observers,
    );
    let both_text = std::fs::read_to_string(&both_path).expect("读两节的发现日志");
    assert_eq!(
        both_text,
        format!("{first_text}{first_text}"),
        "两节各是一份定稿，前一节原样留着"
    );
    drop(directory);
}

/// 录制复用时补录观察者：同一条流先挂观察者跑一趟入库，再把同一个观察者换一份另读的输入摘要（观察者的版本、它录入段的块的键都变了；
/// 流没变，流的录入键只由流定、照样命中）再跑一趟。第二趟命中流的录制、录入那一段整段不跑，录入段在复用之前核这个观察者录入段的块都在库里，
/// 缺的照录，判段读得回它的计数（每个状态一次），不报「录入段那一块不在库里」。从空库起跑（一次性库）：第二趟的观察者在库里一块都没有。
/// 判别力（`crates/mutations.tsv`）：录制复用时不核观察者的块（补录那一段退回什么都不做），第二趟判段读不到这个观察者的块、整趟交错 panic，这条红。
#[test]
fn an_observer_whose_version_differs_from_the_one_recorded_with_the_flow_is_recorded_on_the_reused_recording_and_read_back_by_the_judge_stage(
) {
    let stream = first_stream("findings-observer-of-another-version");
    let directory = disposable_library_directory("observer-of-another-version");
    let observe = count_each_state_once;
    let observers_recorded_with_the_flow = [CrashPointReplayObserver {
        name: "count_each_state_once",
        // 只给每个状态记一次，另读的输入摘要照一句说明取：两趟各取一句，观察者的版本因此不同
        extra_inputs_digest: sha256_digest(b"the observer recorded together with the flow"),
        observe: &observe,
    }];
    let observers_of_another_version = [CrashPointReplayObserver {
        name: "count_each_state_once",
        extra_inputs_digest: sha256_digest(b"the same observer under other extra inputs"),
        observe: &observe,
    }];
    let Some(recorded_with_the_flow) = enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        &directory.join("recorded-with-the-flow.tsv"),
        &directory,
        &observers_recorded_with_the_flow,
    ) else {
        return;
    };
    assert_eq!(
        recorded_with_the_flow.observer_counts.count_named("seen"),
        QUICK_TIER_STATES,
        "前提：头一趟录流时一起录了观察者，判段读回每个状态一次"
    );
    let Some(on_the_reused_recording) = enumerate_the_quick_tier_writing_the_findings_log(
        &stream,
        &directory.join("on-the-reused-recording.tsv"),
        &directory,
        &observers_of_another_version,
    ) else {
        return;
    };
    assert_eq!(
        on_the_reused_recording.observer_counts.count_named("seen"),
        QUICK_TIER_STATES,
        "换了版本的观察者在复用的录制上照录，判段从库里读回它的计数：每个状态一次"
    );
    assert_eq!(
        on_the_reused_recording.observed_states, QUICK_TIER_STATES,
        "换了版本的观察者看过整条流的每个状态"
    );
    assert_eq!(
        on_the_reused_recording.states, recorded_with_the_flow.states,
        "两趟是同一条流"
    );
    drop(directory);
}
