//! checker 档模块：crash、crash_state_enumeration、crash_point_replay_progress、crash_given_states、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks
//! 崩溃放量流水线在新池新建文件那条流（取号与暖机、发布两个崩溃点）上的端到端用例：录入 → 核对 → 第二趟复用 → 两方分工与对账 →
//! 两个库并成一个 → ignore 的点不出状态 → 版本表写错要红；经公共模块按环境真跑两趟，驱动行带复用数、全域与否、展开上限；
//! 被叫停、导入别的库、只排派活计划的那一趟照实报复用数（L1）；两台分片之后按计划点名导入，流水线那一侧与给定状态表那一侧
//! 各与一台判完、与旧的整库导入逐项相同。经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。
//! 要 `verdict-store` 特性（RocksDB），不带特性时这份文件是空的。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::MutexGuard;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
};
use singlefs_checker_tier::crash_amplification::{
    check_blocks_where_until, check_recorded_blocks, compare_judge_and_verifier,
    facts_verifier_row_key_of_the_plan, plan_crash_points, read_violations, record_crash_points,
    record_the_flow_and_read_it_back, unjudged_blocks, unjudged_blocks_of_the_flow,
    verify_blocks_from_facts, ConclusionsStoredUnder, CrashFlow, CrashPlan, CrashPointSpan,
    EnvironmentRun, ExpandUpToWritesPerSegment, FactsVerifierOn, GivenStatesEngine,
    GivenStatesItem, GivenStatesRun, NodeOfAnObserver, ObserverCode, ObserverOnTheNodes,
    PipelineJudgingCode, WeightedShares, END_OF_FLOW_NODE_NAME,
    ENVIRONMENT_VARIABLES_READ_BY_A_RUN,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_identity::{
    judging_units_key, ConclusionKind, ConclusionVersion, CoverageReport, JudgingUnits,
    JudgingUnitsKey, ObserverName,
};
use singlefs_checker_tier::crash_library::{
    DisposableLibraryDirectory, LibraryOpenInThisProcess, LibraryRole,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
#[cfg(feature = "gpu")]
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_checker_tier::verdict_store::{
    BlockCountVector, EncodedVerdictVector, JudgingDevice, JudgingUnitsFingerprint,
    RecoveryWordsOfState, StateOffsetInBlock, VerdictBlockKey, VerdictStore, VerdictVectorNumber,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs";
/// 只展开不超过 6 次写的段：新池新建文件那条流是 2+2+1+2+2+1+2+24+2+1+2，展开的十段共几十个状态，秒级。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

fn expansion(_segment_index: usize, segment: &[usize]) -> CrashPointReplaySegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 录好的一条流：崩溃枚举要的五样加两个崩溃点。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    crash_points: Vec<CrashPointSpan>,
}

impl RecordedFlow {
    fn flow(&self) -> CrashFlow<'_> {
        CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        }
    }
}

/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 harness 档 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切。
fn record_new_pool_file_creation(
    tag: &str,
    ignore_the_publish: bool,
    expected_content: Vec<u8>,
) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let publish_boundary = pool.warm_up_operation_count - pool.mkfs_operation_count;
    let first_publish_write = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= publish_boundary)
        .expect("发布至少一次写");
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let versions = vec![PublishedVersion {
        instance: pool.output.root.instance,
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: expected_content,
    }];
    let crash_points = vec![
        CrashPointSpan {
            name: "acquire_instance_and_warm_up".to_string(),
            code_file_in_repository: THIS_FILE.to_string(),
            writes: 0..first_publish_write,
            ignored: false,
        },
        CrashPointSpan {
            name: "publish_first_file".to_string(),
            code_file_in_repository: THIS_FILE.to_string(),
            writes: first_publish_write..writes.len(),
            ignored: ignore_the_publish,
        },
    ];
    RecordedFlow {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions,
        crash_points,
    }
}

/// CPU 参照核对那一组判法单元的键（核对键里判器那一半，经 `crash_identity::judging_units_key` 取）。
fn cpu_facts_verifier_units() -> JudgingUnitsKey {
    judging_units_key(
        &JudgingUnits::CpuFactsVerifier,
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|refusal| panic!("{}", refusal.sentence()))
}

fn judging_code() -> PipelineJudgingCode {
    PipelineJudgingCode::of_the_judges(
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法代码摘要算得出")
}

/// 这条用例的库：一次性目录里的一份空库（`crash_library::DisposableLibraryDirectory`，用户 2026-10-15 定：流水线单测起跑不清长期库，
/// 空库起跑的单测在一次性目录里建库、用完删），长期库一行不写；这一趟的判定写在这一趟自己的判法单元指纹下（[`fresh_judging_units_fingerprint`]）。
struct UnitTestLibrary {
    // 先关库、后删目录：字段按声明的次序丢
    handle: LibraryOpenInThisProcess,
    _directory: DisposableLibraryDirectory,
}

impl UnitTestLibrary {
    /// 同 [`Self::emptied_at_the_start`]：一次性的库每趟都是空的。
    fn new(tag: &str) -> Self {
        Self::emptied_at_the_start(tag)
    }

    /// 一次性目录底下的 `library`（库只在不存在的目录上建）：整份库的账（哪些块没判、存了几份结论）从空库数起。
    fn emptied_at_the_start(tag: &str) -> Self {
        let directory = DisposableLibraryDirectory::new(tag).expect("建得了一次性库目录");
        let library = directory.library();
        let handle =
            LibraryOpenInThisProcess::open_or_share(&library, LibraryRole::ThisHostsLibrary)
                .unwrap_or_else(|refusal| {
                    panic!("开得了用例的库 {}：{refusal}", library.display())
                });
        Self {
            handle,
            _directory: directory,
        }
    }
    fn store(&self) -> MutexGuard<'_, VerdictStore> {
        self.handle.lock()
    }
}

/// 这一趟自己的判法单元指纹（进程号、纳秒与一个流水号的摘要）：长期库里这一版下一块都没判过，用例钉的「第一趟全判、第二趟全复用」
/// 那一套账每趟都成立；别的趟判过的块都在别的版本下，流、路径与块键照摘要复用。
fn fresh_version_digest() -> [u8; DIGEST_BYTES] {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("时钟在 1970 之后")
        .as_nanos();
    sha256_digest(
        format!(
            "fresh-version process={} nanos={nanos} number={}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        )
        .as_bytes(),
    )
}

fn fresh_judging_units_fingerprint() -> JudgingUnitsFingerprint {
    JudgingUnitsFingerprint(fresh_version_digest())
}

fn fresh_conclusion_version() -> ConclusionVersion {
    ConclusionVersion(fresh_version_digest())
}

fn total_blocks(plan: &CrashPlan) -> u64 {
    plan.points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum()
}

#[test]
fn records_checks_and_then_reuses_every_block_of_the_new_pool_file_creation_flow() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-pipeline", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    assert_eq!(plan.points.len(), 3, "两个点，加这条流自己的末节点");
    assert_eq!(
        plan.points[0].ordinals.end, plan.points[1].ordinals.start,
        "两个点的状态序号首尾相接"
    );
    let state_count = crash_point_replay_plan_state_count(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    assert_eq!(
        (
            plan.points[2].name.as_str(),
            plan.points[2].ordinals.clone()
        ),
        (END_OF_FLOW_NODE_NAME, state_count - 1..state_count),
        "整条流最后全部持久那一个状态归末节点，不归最后一个点"
    );
    let mut cursor = 0u64;
    for point in &plan.points {
        assert_eq!(
            point.ordinals.start, cursor,
            "各点的序号区间严丝合缝地划分整条流"
        );
        cursor = point.ordinals.end;
    }
    assert_eq!(cursor, state_count);
    assert!(
        plan.points[0].path_number.matches('/').count() == 0
            && plan.points[1].path_number.matches('/').count() == 1
            && plan.points[2].path_number.matches('/').count() == 2,
        "路径编号一点一段（末节点接在最后一个点下面）"
    );
    let library = UnitTestLibrary::new("reuse");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    let mut report = CoverageReport::default();
    let recording = record_crash_points(&mut store, &plan, &version, &mut report).expect("录得进");
    assert_eq!(recording.blocks_recorded, total_blocks(&plan));
    assert_eq!(recording.states_recorded, state_count);
    assert_eq!(recording.blocks_already_judged, 0);
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        usize::try_from(total_blocks(&plan)).expect("装得进")
    );
    assert!(
        report.lines().iter().all(|line| line.ends_with(" checked")),
        "第一趟每个点都是 checked：{:?}",
        report.lines()
    );
    assert!(
        report.lines()[0]
            .split(' ')
            .nth(1)
            .is_some_and(|identity| identity
                .starts_with(&format!("{THIS_FILE}::acquire_instance_and_warm_up::"))),
        "报告行 = 路径 身份 去向，身份三段：{}",
        report.lines()[0]
    );
    let checking = check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert_eq!(checking.blocks_checked, total_blocks(&plan));
    assert_eq!(checking.states_checked, state_count);
    assert_eq!(
        checking.red_states, 0,
        "健康的流一个红状态都不该有：{:?}",
        checking.red_texts
    );
    assert!(
        unjudged_blocks(&store, &version)
            .expect("对得了账")
            .is_empty(),
        "核完对账为空"
    );
    assert!(read_violations(&store, &plan, &version)
        .expect("读得了违例")
        .is_empty());
    let mut second_report = CoverageReport::default();
    let second =
        record_crash_points(&mut store, &plan, &version, &mut second_report).expect("录得进");
    assert_eq!(
        second.blocks_already_judged,
        total_blocks(&plan),
        "第二趟全部复用"
    );
    assert_eq!(second.blocks_recorded, 0);
    assert!(
        second_report
            .lines()
            .iter()
            .all(|line| line.ends_with(" reused")),
        "{:?}",
        second_report.lines()
    );
    let nothing_left = check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert_eq!(nothing_left.blocks_checked, 0, "复用的块不再核");
}

/// 一份库装几条流：这条流的块都判完了，它自己的账就是空的，哪怕库里还有别的流录了没判的块；整份库的账照旧报出那一块。
#[test]
fn a_flow_reconciles_its_own_blocks_and_not_the_blocks_another_flow_recorded() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-own-blocks", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let library = UnitTestLibrary::new("own-blocks");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    assert_eq!(
        unjudged_blocks_of_the_flow(
            &store,
            &plan,
            &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version),
        )
        .expect("对得了账")
        .len(),
        3,
        "录了还没判：两个点与末节点各一块，都在这条流自己的账上"
    );
    // 别的流录进同一份库的一块（拿这条流第一块的键换一个起点序号造出来），一直没判
    let mut block_of_another_flow = unjudged_blocks(&store, &version)
        .expect("对得了账")
        .first()
        .expect("录了三块")
        .0
        .clone();
    block_of_another_flow.block_start.0 += 1 << 40;
    store
        .record_block(&block_of_another_flow, 7)
        .expect("录得进");
    check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert!(
        unjudged_blocks_of_the_flow(
            &store,
            &plan,
            &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version),
        )
        .expect("对得了账")
        .is_empty(),
        "这条流的块都判完了，它自己的账是空的"
    );
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账"),
        vec![(block_of_another_flow, 7)],
        "整份库的账照旧报出别的流那一块"
    );
}

/// 失败点码表挪出 `gpu` 特性（崩溃点重放合并第三轮判决 J8，M4 第 8 条）：CPU 判器按结论种类存的结论每一块都带恢复字、一个状态八个字，
/// 不编 `gpu` 特性的二进制也一样；按发布分的态数随块存（J7），合起来等于这一块的状态数。这一份用例在只开 `verdict-store` 的那一趟里跑到。
#[test]
fn cpu_conclusions_store_the_recovery_words_of_every_state_without_the_gpu_feature() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-recovery-words", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    // 数的是整份库里存了几份结论：从空库起
    let library = UnitTestLibrary::emptied_at_the_start("recovery-words");
    let mut store = library.store();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    let version_of_each_point = vec![fresh_conclusion_version(); plan.points.len()];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &version_of_each_point,
    };
    let checking = check_blocks_where_until(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_block| true,
        &conclusions,
        &|| false,
    )
    .expect("核得完");
    assert_eq!(checking.blocks_checked, total_blocks(&plan), "每一块都判了");
    let stored_keys = store.stored_conclusion_keys().expect("列得了结论");
    assert_eq!(
        u64::try_from(stored_keys.len()).expect("块数装得进 u64"),
        total_blocks(&plan),
        "每一块存了一份 CPU 结论"
    );
    for (key, kind, version) in &stored_keys {
        let stored = store
            .read_conclusion_block(key, kind, version)
            .expect("读得了")
            .expect("列出来的结论读得回来");
        let states = stored.block.verdict_vector_number_of_each_state().len();
        assert_eq!(
            stored.recovery_words.map(|words| words.len()),
            Some(states),
            "块 {key:?}：每个状态一份恢复字（不编 gpu 特性也有）"
        );
        assert_eq!(
            stored.count_vector.states_by_publish.values().sum::<u64>(),
            u64::try_from(states).expect("状态数装得进 u64"),
            "块 {key:?}：按发布分的态数合起来是这一块的状态数"
        );
    }
}

/// 计数向量每一趟都判（崩溃点重放合并第三轮判决 J7）：CPU 判器存的结论读回来，计数向量与计划对得上（按发布分的态数照段的区间现算、
/// 清单里每条不变量一行、判到加不适用是状态数）；同一批块在另一把版本下存一份空的计数向量，那一把读回来块块都对不上。
#[test]
fn the_count_vectors_read_back_are_checked_against_the_plan_every_run() {
    use singlefs_checker_tier::crash_amplification::count_vector_disagreements_of_the_flow;
    use singlefs_checker_tier::verdict_store::{
        BlockCountVector, ConclusionBlockRecord, EncodedVerdictVector, JudgingDevice, VerdictBlock,
    };
    let recorded =
        record_new_pool_file_creation("crash-amplification-count-vectors", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let library = UnitTestLibrary::new("count-vectors");
    let mut store = library.store();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    let judged_versions = vec![fresh_conclusion_version(); plan.points.len()];
    let judged = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &judged_versions,
    };
    check_blocks_where_until(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_block| true,
        &judged,
        &|| false,
    )
    .expect("核得完");
    assert_eq!(
        count_vector_disagreements_of_the_flow(
            &store,
            &flow_read_back_from_the_library,
            &plan,
            &judged,
            &|_block| true
        )
        .expect("读得了"),
        Vec::<String>::new(),
        "判器存的计数向量与计划对得上"
    );
    let empty_vector = store
        .register_verdict_vector(&EncodedVerdictVector(Vec::new()))
        .expect("登记得了");
    let blank_versions = vec![fresh_conclusion_version(); plan.points.len()];
    // 只改这条流自己的块：长期库里还有别的趟、别的流的结论
    for key in plan.points.iter().flat_map(|point| {
        point
            .blocks()
            .into_iter()
            .map(move |local| point.block_key(&local))
    }) {
        let stored = store
            .read_conclusion_block(&key, &ConclusionKind::ImageJudgement, &judged_versions[0])
            .expect("读得了")
            .expect("列出来的读得回来");
        let states = stored.block.verdict_vector_number_of_each_state().len();
        store
            .store_conclusion_block(
                &key,
                &ConclusionKind::ImageJudgement,
                &blank_versions[0],
                &ConclusionBlockRecord {
                    block: VerdictBlock::new(vec![empty_vector; states]).expect("至少一个状态"),
                    violating_states: Vec::new(),
                    count_vector: BlockCountVector::default(),
                    judged_on: JudgingDevice::Cpu,
                    recovery_words: None,
                },
            )
            .expect("存得进");
    }
    let blank = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &blank_versions,
    };
    let disagreements = count_vector_disagreements_of_the_flow(
        &store,
        &flow_read_back_from_the_library,
        &plan,
        &blank,
        &|_block| true,
    )
    .expect("读得了");
    assert!(
        disagreements
            .iter()
            .filter(|disagreement| disagreement.contains("按发布分的态数"))
            .count()
            == usize::try_from(total_blocks(&plan)).expect("块数装得进 usize"),
        "空的计数向量块块都对不上按发布分的态数：{disagreements:?}"
    );
    assert!(
        disagreements
            .iter()
            .any(|disagreement| disagreement.contains("逐条不变量计数的行")),
        "空的计数向量也对不上不变量清单：{disagreements:?}"
    );
}

/// 核到一半被叫停：不再领新的块，已经判完的那一块在库里；下一趟只判剩下的那一块，对账为空。
#[test]
fn a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest() {
    let recorded = record_new_pool_file_creation("crash-amplification-stop", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    assert_eq!(total_blocks(&plan), 3, "两个点与末节点各一块");
    let library = UnitTestLibrary::new("stop");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    // 领第一块之前问一次（不停），领第二块之前再问一次（停）
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let stop_before_the_second_block =
        || asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 1;
    let stopped = check_blocks_where_until(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_block| true,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version),
        &stop_before_the_second_block,
    )
    .expect("核得完");
    assert_eq!(stopped.blocks_checked, 1, "被叫停之前只判了第一块");
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        2,
        "第二块与末节点那一块还没判"
    );
    let resumed = check_blocks_where_until(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_block| true,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version),
        &|| false,
    )
    .expect("核得完");
    assert_eq!(resumed.blocks_checked, 2, "接着判的那一趟只判剩下的那两块");
    assert_eq!(
        stopped.states_checked + resumed.states_checked,
        plan.points.last().expect("有点").ordinals.end,
        "两趟合起来每个状态恰好判一次"
    );
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
}

#[test]
fn two_shares_leave_the_other_side_unjudged_until_that_side_checks_and_two_libraries_merge_by_import(
) {
    let recorded =
        record_new_pool_file_creation("crash-amplification-shares", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    assert_eq!(total_blocks(&plan), 3, "两个点与末节点各一块");
    let shares = WeightedShares::new(vec![1, 1]);
    let first_library = UnitTestLibrary::new("share-a");
    let second_library = UnitTestLibrary::new("share-b");
    let mut first = first_library.store();
    let mut second = second_library.store();
    let version = fresh_judging_units_fingerprint();
    // 两方各从自己的库里读回流来判：判法只吃库里读回的那一份。
    let first_read_back =
        record_the_flow_and_read_it_back(&mut first, &flow).expect("流写得进第一方的库、读得回");
    let second_read_back =
        record_the_flow_and_read_it_back(&mut second, &flow).expect("流写得进第二方的库、读得回");
    for store in [&mut first, &mut second] {
        record_crash_points(store, &plan, &version, &mut CoverageReport::default())
            .expect("录得进");
    }
    let first_tally = check_recorded_blocks(
        &mut first,
        &first_read_back.flow(),
        &plan,
        &shares,
        0,
        &version,
    )
    .expect("核得完");
    assert_eq!(
        first_tally.blocks_checked, 2,
        "第一方只核分到它的那两块（平滑加权轮转：第 0、2 块）"
    );
    assert_eq!(
        unjudged_blocks(&first, &version).expect("对得了账").len(),
        1,
        "另一方那一块在第一方的库里对账为没核完"
    );
    let second_tally = check_recorded_blocks(
        &mut second,
        &second_read_back.flow(),
        &plan,
        &shares,
        1,
        &version,
    )
    .expect("核得完");
    assert_eq!(second_tally.blocks_checked, 1);
    let imported = first.import_from(&second).expect("导得进");
    assert!(
        imported >= 1,
        "第二方那一块导进第一方的库（整库导入连长期库里上几趟的块也算已导入）：{imported}"
    );
    assert!(
        unjudged_blocks(&first, &version)
            .expect("对得了账")
            .is_empty(),
        "并成一个库之后对账为空"
    );
    assert!(read_violations(&first, &plan, &version)
        .expect("读得了违例")
        .is_empty());
}

#[test]
fn an_ignored_crash_point_keeps_its_place_in_the_tree_and_contributes_no_state_before_the_end_of_flow(
) {
    let recorded =
        record_new_pool_file_creation("crash-amplification-ignored", true, file_content());
    let full =
        record_new_pool_file_creation("crash-amplification-not-ignored", false, file_content());
    let judging = judging_code();
    let plan = plan_crash_points(&recorded.flow());
    let full_plan = plan_crash_points(&full.flow());
    assert!(plan.points[1].ignored);
    assert_eq!(
        plan.points[1].ordinals.end - plan.points[1].ordinals.start,
        0,
        "ignore 的点的段不展开，一个状态都不剩（整条流最后全部持久那一个归末节点）"
    );
    assert_eq!(
        (
            plan.points[2].name.as_str(),
            plan.points[2].ordinals.end - plan.points[2].ordinals.start
        ),
        (END_OF_FLOW_NODE_NAME, 1),
        "末节点管最后全部持久那一个状态"
    );
    assert_eq!(
        plan.points[0].ordinals, full_plan.points[0].ordinals,
        "上游的点不受影响"
    );
    assert_eq!(
        plan.points[1].path_number, full_plan.points[1].path_number,
        "ignore 不动树的编号"
    );
    assert_eq!(
        plan.points[1].reuse_key, full_plan.points[1].reuse_key,
        "ignore 不动复用键：以后开 ignore 只补它自己的块"
    );
    // 判法单元指纹改了：计划不看判法，树与复用键都不动；库里旧版本的判定不复用、旧行留着
    let mut other_judging = judging.clone();
    let mut flipped = *judging.digest();
    flipped[0] ^= 0x01;
    other_judging.replace_digest(flipped);
    assert_ne!(other_judging.version(), judging.version());
    let other_plan = plan_crash_points(&full.flow());
    assert_eq!(
        other_plan.points[1].path_number, full_plan.points[1].path_number,
        "判法改了树不动"
    );
    assert_eq!(
        other_plan.points[1].reuse_key, full_plan.points[1].reuse_key,
        "判法改了复用键也不动：块键只由流程定"
    );
    let library = UnitTestLibrary::new("ignored");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    let other_version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &recorded.flow()).expect("流写得进库、读得回");
    let mut report = CoverageReport::default();
    record_crash_points(&mut store, &plan, &version, &mut report).expect("录得进");
    assert!(
        report.lines()[1].ends_with(" ignored"),
        "{:?}",
        report.lines()
    );
    assert!(report.lines()[0].ends_with(" checked"));
    let checking = check_recorded_blocks(
        &mut store,
        &read_back.flow(),
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert!(checking.blocks_checked > 0);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
    let mut other_report = CoverageReport::default();
    let other_recording =
        record_crash_points(&mut store, &plan, &other_version, &mut other_report).expect("录得进");
    assert_eq!(
        other_recording.blocks_already_judged, 0,
        "判法单元指纹换了，旧版本的判定一块都不复用"
    );
    assert_eq!(
        other_recording.blocks_recorded, checking.blocks_checked,
        "同样的块在新版本下全部待判"
    );
    assert_eq!(
        unjudged_blocks(&store, &other_version)
            .expect("对得了账")
            .len(),
        usize::try_from(checking.blocks_checked).expect("块数"),
        "新版本下对账列出全部块"
    );
    assert!(
        unjudged_blocks(&store, &version)
            .expect("对得了账")
            .is_empty(),
        "旧版本的行还在"
    );
    assert_eq!(
        store
            .judging_units_fingerprints()
            .expect("列判法单元指纹")
            .len(),
        0,
        "直接调录入不登组成清单；组成清单由 run_from_environment 登"
    );
}

#[test]
fn a_wrong_version_table_makes_the_pipeline_store_red_states_the_gate_can_read() {
    let mut wrong_content = file_content();
    wrong_content[0] ^= 0x01;
    let recorded = record_new_pool_file_creation("crash-amplification-red", false, wrong_content);
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let library = UnitTestLibrary::new("red");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert!(
        checking.red_states > 0,
        "版本表的内容写错，根落了的状态读回的内容对不上，必须红"
    );
    assert!(
        !checking.red_texts.is_empty()
            && checking
                .red_texts
                .iter()
                .all(|(_, _, text)| !text.is_empty()),
        "每个红状态有自己现算的正文"
    );
    let violations = read_violations(&store, &plan, &version).expect("读得了违例");
    assert_eq!(
        u64::try_from(violations.len()).expect("装得进"),
        checking.red_states,
        "门禁读到的违例数等于核出的红状态数"
    );
    for red in &violations {
        assert!(
            red.items.iter().all(|item| item
                .starts_with(CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE)
                || item.starts_with(CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE)),
            "只有 oracle 那一道该红：{:?}",
            red.items
        );
        // 读回内容不对的状态是根落了的状态：发布那个点里根槽写之后的那几个，与整条流最后全部持久的那一个——
        // 后者自从每条流最后全部持久那一态另立成末节点（END_OF_FLOW_NODE_NAME，崩溃点重放合并第三轮正文 H3「共享到 C」）就归末节点
        assert!(
            red.identity.contains("::publish_first_file::")
                || red
                    .identity
                    .contains(&format!("::{END_OF_FLOW_NODE_NAME}::")),
            "红在发布那个点或末节点（全部持久那一态）：{}",
            red.identity
        );
    }
}

#[test]
fn the_cpu_facts_verifier_agrees_with_the_judge_on_every_state_and_reuses_its_blocks() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-verifier", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let library = UnitTestLibrary::new("verifier");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert_eq!(checking.red_states, 0);
    let on = FactsVerifierOn::Cpu;
    let verifier_units = cpu_facts_verifier_units();
    let tally = verify_blocks_from_facts(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_| true,
        &on,
        &verifier_units,
    )
    .expect("核得完");
    let verifier_row_key = facts_verifier_row_key_of_the_plan(
        &flow_read_back_from_the_library,
        &plan,
        &verifier_units,
    );
    assert_eq!(
        tally.blocks_verified + tally.blocks_already_verified,
        total_blocks(&plan),
        "第 ② 段每一块都核了或早核过（长期库：上一趟这一版核对代码核过的块照摘要复用）"
    );
    assert_eq!(tally.verifier_red_states, 0, "健康流第 ② 段也全绿");
    let comparison =
        compare_judge_and_verifier(&store, &plan, &verifier_row_key, &version).expect("比得完");
    assert_eq!(comparison.states_compared, checking.states_checked);
    assert_eq!(comparison.both_green, checking.states_checked);
    assert_eq!(comparison.disagreements, 0);
    assert_eq!(comparison.blocks_without_verifier, 0);
    assert_eq!(comparison.blocks_without_judge, 0);
    let again = verify_blocks_from_facts(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_| true,
        &on,
        &verifier_units,
    )
    .expect("核得完");
    assert_eq!(again.blocks_verified, 0, "同一版核对代码第二趟全部复用");
    assert_eq!(again.blocks_already_verified, total_blocks(&plan));
    // 换一版「核对代码」（摘要不同）：旧行留着，新版从头核
    let mut other_digest = verifier_row_key;
    other_digest[0] ^= 0x01;
    let elsewhere =
        compare_judge_and_verifier(&store, &plan, &other_digest, &version).expect("比得完");
    assert_eq!(elsewhere.states_compared, 0, "别的摘要下一块都没核过");
    assert_eq!(elsewhere.blocks_without_verifier, total_blocks(&plan));
}

#[test]
fn a_wrong_version_table_is_judge_red_only_for_the_facts_verifier() {
    let mut wrong_content = file_content();
    wrong_content[0] ^= 0x01;
    let recorded =
        record_new_pool_file_creation("crash-amplification-verifier-red", false, wrong_content);
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let library = UnitTestLibrary::new("verifier-red");
    let mut store = library.store();
    let version = fresh_judging_units_fingerprint();
    // 判法只吃库里读回的那一份：流先写进库再读回，下面判、核、对账都用读回的。
    let read_back =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let flow_read_back_from_the_library = read_back.flow();
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &version,
    )
    .expect("核得完");
    assert!(checking.red_states > 0, "版本表写错，判器红");
    let on = FactsVerifierOn::Cpu;
    let verifier_units = cpu_facts_verifier_units();
    let tally = verify_blocks_from_facts(
        &mut store,
        &flow_read_back_from_the_library,
        &plan,
        &|_| true,
        &on,
        &verifier_units,
    )
    .expect("核得完");
    assert_eq!(
        tally.verifier_red_states, 0,
        "版本表是判器的输入，不是盘上的事实：第 ② 段看不见它"
    );
    let comparison = compare_judge_and_verifier(
        &store,
        &plan,
        &facts_verifier_row_key_of_the_plan(
            &flow_read_back_from_the_library,
            &plan,
            &verifier_units,
        ),
        &version,
    )
    .expect("比得完");
    assert_eq!(
        comparison.judge_red_only, checking.red_states,
        "判器红而核对绿的正是那些状态"
    );
    assert_eq!(comparison.disagreements, 0);
}

/// 一行 `CRASH_AMPLIFICATION` 的 `key=value` 字段。
fn fields_of_the_line(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .skip(1)
        .filter_map(|field| field.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// 一条录好的流经公共模块按环境跑一趟（展开上限默认 6），交回这一趟的结果与它打的那一行。环境由调用方的守卫定。
fn amplify(recorded: &RecordedFlow, flow_name: &str) -> (EnvironmentRun, Option<String>) {
    common_crash_points::amplify_prepared_flow_and_hand_back_its_line(
        flow_name,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        recorded.crash_points.clone(),
        LARGEST_EXPANDED_SEGMENT_WRITES,
    )
}

/// 按环境跑的用例的目录：一次性目录（`crash_library::DisposableLibraryDirectory`，起跑是空的、用完删，长期库一行不写），里面一份空的多机配置
/// （单机 CPU）与一份库 `library`（两台分片的用例再放几份）；环境守卫只设这两个、别的全清。这几条用例钉的是空库起跑的账
/// （第一趟全判、第二趟全复用、两台各判各的再导入）。
struct EnvironmentRunDirectory {
    directory: DisposableLibraryDirectory,
    configuration: PathBuf,
    library: PathBuf,
}

impl EnvironmentRunDirectory {
    fn new(tag: &str) -> Self {
        let directory =
            DisposableLibraryDirectory::new(tag).expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host.env");
        std::fs::write(&configuration, "").expect("写得了空配置");
        let library = directory.join("library");
        Self {
            directory,
            configuration,
            library,
        }
    }

    fn environment(&self) -> common_crash_points::CrashAmplificationEnvironment {
        common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
            ("SINGLEFS_MULTI_HOST_CONFIG", self.configuration.as_os_str()),
            (
                "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
                self.library.as_os_str(),
            ),
        ])
    }
}

/// 驱动行的复用数、全域与否、展开上限（入口 `scripts/crash/crash-amplification.sh` 按这几个字段汇总）：在新池新建文件那条流上
/// 经公共模块真跑两趟（同一份库），两趟打出来的行里三个字段都在；第一趟库是空的、复用 0；第二趟 `states_reused` 等于库里这条流
/// 在这一判法单元指纹下已判的态数（第一趟全判完了，就是整条流的态数），也等于录入时数到的已判态数。
/// 环境由守卫清掉再设（多机配置换成一份空的、单机 CPU，库落在测试自己的临时目录），跑完或 panic 都还原。要 `verdict-store` 特性：快档靠 C604 才编进来。
#[test]
fn a_driver_line_reports_reuse_exhaustiveness_and_depth_and_its_reuse_equals_the_states_judged_in_the_library(
) {
    let recorded =
        record_new_pool_file_creation("crash-amplification-line-fields", false, file_content());
    let directory = EnvironmentRunDirectory::new("line-fields");
    let environment = directory.environment();
    let (first_run, first_line) = amplify(&recorded, "line-fields");
    let (second_run, second_line) = amplify(&recorded, "line-fields");
    drop(environment);
    let first_line = first_line.expect("判了的那一趟打一行");
    let second_line = second_line.expect("判了的那一趟打一行");
    for line in [&first_line, &second_line] {
        let fields = fields_of_the_line(line);
        for key in ["states_reused", "exhaustive", "expand_up_to"] {
            assert!(fields.contains_key(key), "行里没有 {key}=：{line}");
        }
        assert_eq!(
            fields.get("expand_up_to").map(String::as_str),
            Some(second_run.expand_up_to.0.to_string().as_str()),
            "{line}"
        );
        assert_eq!(
            fields.get("exhaustive").map(String::as_str),
            Some(second_run.exhaustive.to_string().as_str()),
            "{line}"
        );
    }
    assert_eq!(
        fields_of_the_line(&first_line)
            .get("states_reused")
            .map(String::as_str),
        Some("0"),
        "第一趟库是空的，一态都没复用：{first_line}"
    );
    assert!(first_run.checking.states_checked > 0, "第一趟判了状态");

    let limit: ExpandUpToWritesPerSegment = second_run.expand_up_to;
    let expansion_of_the_run =
        move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion_of_the_run,
        crash_points: recorded.crash_points.clone(),
    };
    let plan = plan_crash_points(&flow);
    let state_count = plan.points.last().expect("有点").ordinals.end;
    let (store, _) = VerdictStore::open_existing(&directory.library).expect("开得了这两趟的库");
    let unjudged_states: u64 =
        unjudged_blocks_of_the_flow(&store, &plan, &second_run.judged_conclusions())
            .expect("对得了账")
            .iter()
            .map(|(_, states)| *states)
            .sum();
    let judged_in_the_library = state_count - unjudged_states;
    assert_eq!(judged_in_the_library, state_count, "第一趟把这条流全判完了");
    assert_eq!(
        fields_of_the_line(&second_line)
            .get("states_reused")
            .map(String::as_str),
        Some(judged_in_the_library.to_string().as_str()),
        "第二趟行里的复用数等于库里这条流已判的态数：{second_line}"
    );
    assert_eq!(
        second_run.states_reused, second_run.recording.states_already_judged,
        "一台跑、没导入：复用数就是录入时数到的已判态数"
    );
    assert_eq!(second_run.checking.states_checked, 0, "第二趟一态都不再判");
    drop(store);
    drop(directory);
}

/// L1：被叫停的那一趟照实报它录入时库里已判的态数，不写 0。同一份库上第一趟把整条流判完；第二趟开跑前叫停文件已经放着，
/// 它全部复用、一态不判、被叫停收尾，行上的 `states_reused` 等于整条流的态数。
#[test]
fn a_run_stopped_with_every_block_already_judged_reports_them_as_reused() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-stopped-reuse", false, file_content());
    let directory = EnvironmentRunDirectory::new("stopped-reuse");
    let mut environment = directory.environment();
    let (first_run, _) = amplify(&recorded, "stopped-reuse");
    let stop_file = directory.directory.join("stop");
    std::fs::write(&stop_file, "").expect("放得了叫停文件");
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
        stop_file.as_os_str(),
    );
    let (stopped_run, stopped_line) = amplify(&recorded, "stopped-reuse");
    drop(environment);
    let state_count = first_run.recording.states_recorded;
    assert!(state_count > 0, "第一趟录了状态");
    assert!(!first_run.stopped_early, "第一趟没叫停");
    assert!(stopped_run.stopped_early, "第二趟开跑前叫停文件已经在");
    assert_eq!(
        stopped_run.checking.states_checked, 0,
        "库里全判过，叫停的那一趟一态都没判"
    );
    assert_eq!(
        stopped_run.states_reused, state_count,
        "叫停的那一趟录入时库里整条流都判过"
    );
    let stopped_line = stopped_line.expect("叫停的那一趟也打一行");
    assert_eq!(
        fields_of_the_line(&stopped_line)
            .get("states_reused")
            .map(String::as_str),
        Some(state_count.to_string().as_str()),
        "{stopped_line}"
    );
    drop(directory);
}

/// L1：导入别的库的那一趟照实报导入之前库里已判的态数，不写 0。同一份库上第一趟把整条流判完；第二趟导入一份空库，
/// 导入之前这份库里整条流都判过，行上的 `states_reused` 等于整条流的态数，导进来的块是 0。
#[test]
fn a_run_importing_another_library_reports_the_states_judged_before_the_import_as_reused() {
    let recorded =
        record_new_pool_file_creation("crash-amplification-import-reuse", false, file_content());
    let directory = EnvironmentRunDirectory::new("import-reuse");
    let mut environment = directory.environment();
    let (first_run, _) = amplify(&recorded, "import-reuse");
    let other_library = directory.directory.join("other-library");
    drop(VerdictStore::create_empty(&other_library).expect("建得了要导入的那一份空库"));
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        other_library.as_os_str(),
    );
    let (importing_run, importing_line) = amplify(&recorded, "import-reuse");
    drop(environment);
    let state_count = first_run.recording.states_recorded;
    assert!(state_count > 0, "第一趟录了状态");
    assert_eq!(importing_run.imported_blocks, 0, "导入的是一份空库");
    assert_eq!(
        importing_run.states_reused, state_count,
        "导入之前库里整条流都判过"
    );
    let importing_line = importing_line.expect("判了的那一趟打一行");
    assert_eq!(
        fields_of_the_line(&importing_line)
            .get("states_reused")
            .map(String::as_str),
        Some(state_count.to_string().as_str()),
        "{importing_line}"
    );
    drop(directory);
}

/// 一块一种结论按内容读回（两份库各有各的判定向量编号，按编号比没有意义）：逐状态的判定向量、违例（块内序号与判定向量）、计数向量、
/// 块属性、恢复字。
#[derive(Debug, PartialEq, Eq)]
struct ConclusionReadBackByContent {
    verdict_vector_of_each_state: Vec<EncodedVerdictVector>,
    violations: Vec<(StateOffsetInBlock, EncodedVerdictVector)>,
    count_vector: BlockCountVector,
    judged_on: JudgingDevice,
    recovery_words: Option<Vec<RecoveryWordsOfState>>,
}

/// `store` 里这一块这一种结论在这一把版本下按内容读回；没存过是 `None`。
fn conclusion_read_back_by_content(
    store: &VerdictStore,
    block_key: &VerdictBlockKey,
    kind: &ConclusionKind,
    conclusion_version: &ConclusionVersion,
) -> Option<ConclusionReadBackByContent> {
    let stored = store
        .read_conclusion_block(block_key, kind, conclusion_version)
        .expect("读得了结论块")?;
    let vector_of_number = |number: VerdictVectorNumber| {
        store
            .verdict_vector_of_number(number)
            .expect("结论块里的编号在它自己的库里登记过")
            .clone()
    };
    Some(ConclusionReadBackByContent {
        verdict_vector_of_each_state: stored
            .block
            .verdict_vector_number_of_each_state()
            .iter()
            .map(|number| vector_of_number(*number))
            .collect(),
        violations: stored
            .violations
            .iter()
            .map(|violation| {
                (
                    violation.offset_in_block,
                    vector_of_number(violation.verdict_vector_number),
                )
            })
            .collect(),
        count_vector: stored.count_vector,
        judged_on: stored.judged_on,
        recovery_words: stored.recovery_words,
    })
}

/// 导入用例里挂在发布那个点上的观察者：只数它看过几个状态，不判红。
const STATES_SEEN_BY_THE_IMPORT_PROBE_COUNTER: &str = "states_seen_by_the_import_probe";

/// 一条录好的流带着挂在节点上的观察者经公共模块按环境跑一趟（展开上限默认 6），交回这一趟的结果。环境由调用方的守卫定。
fn amplify_with_observers(
    recorded: &RecordedFlow,
    flow_name: &str,
    observers: &[ObserverOnTheNodes<'_>],
) -> EnvironmentRun {
    common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
        flow_name,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &[],
        recorded.crash_points.clone(),
        LARGEST_EXPANDED_SEGMENT_WRITES,
        observers,
    )
    .0
}

/// 导入那一条用例挂的观察者：每看一个状态加一次计数。观察者的身份是这个 fn 的闭包键
/// （入口登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
fn count_each_state_for_the_import_probe(
    _ordinal: u64,
    _image: &CrashImage<'_>,
    _report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    counts.add(STATES_SEEN_BY_THE_IMPORT_PROBE_COUNTER, 1);
}

/// 判法单元拒算（4-2a 第 7 条，连同后半句「拒算的那几条照判不入库、别的照键入库同一份库」）：挂一个没在入口登记表
/// （`singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）里登记的观察者，取它那一把键拒算（编译时的判器键表里没有这一组）。
/// 流水线不交错、不整趟退 78：开跑前就点名拒算的那一组与形态，照常在这条用例的长期库里判完、核完、比完（不另建一次性库）；
/// 库里 CPU 判器的结论照键入库，那个观察者的结论一块都没进库（它的版本是这个进程的占位，照判不入库）。
/// 判别力（`crates/mutations.tsv`）：观察者那几把占位版本不登记成照判不入库，它的结论写进库，这条红。
#[test]
fn an_observer_without_a_registered_entry_is_judged_without_storing_its_conclusions_and_named_as_refused(
) {
    let recorded = record_new_pool_file_creation(
        "crash-amplification-refused-observer",
        false,
        file_content(),
    );
    let observe_by_counting = count_each_state_for_the_import_probe;
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("unregistered_probe").expect("观察者的名字非空、不长"),
        observer_code_file: THIS_FILE.to_string(),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: vec![NodeOfAnObserver {
            setting_file: THIS_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::CountsOnly(&observe_by_counting),
    }];
    let directory = EnvironmentRunDirectory::new("refused-observer");
    let long_term_library = directory.library.clone();
    let environment = directory.environment();
    let run = amplify_with_observers(&recorded, "refused-observer", &observers);
    drop(environment);
    assert_eq!(
        run.refused_judging_units.len(),
        1,
        "只有没登记的观察者那一把拒算：{:?}",
        run.refused_judging_units
    );
    assert!(
        run.refused_judging_units[0]
            .contains(&format!("observer_code:{THIS_FILE}:unregistered_probe"))
            && run.refused_judging_units[0].contains("not_computed_at_the_build"),
        "红句点名拒算的那一组与形态：{}",
        run.refused_judging_units[0]
    );
    assert_eq!(
        run.library_directory, long_term_library,
        "拒算的那一趟照常在这条用例的长期库里判，不另建一次性库"
    );
    assert_eq!(run.unjudged_blocks, 0, "这条流照常判完");
    let comparison = run
        .comparison
        .as_ref()
        .expect("没开 GPU：第 ② 段由 CPU 参照核、第 ③ 段比了");
    assert_eq!(comparison.disagreements, 0, "判器与核对照样逐状态比");
    let (store, _) = VerdictStore::open_existing(&long_term_library).expect("开得了库");
    let conclusions_of = |wanted: ConclusionKind| {
        store
            .stored_conclusion_keys()
            .expect("列得出库里的结论")
            .into_iter()
            .filter(|(_key, kind, _version)| *kind == wanted)
            .count()
    };
    assert!(
        conclusions_of(ConclusionKind::ImageJudgement) > 0,
        "取得出键的 CPU 判器照键入库"
    );
    assert_eq!(
        conclusions_of(ConclusionKind::Observer),
        0,
        "拒算的观察者照判不入库：它的结论一块都没进库"
    );
}

/// 两台分片之后的导入（`scripts/crash/crash-amplification-two-hosts.sh` 那几步：本机 0/2、第二台 1/2 各写一份库，本机再带
/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<第二台的库>` 跑一趟 0/2）按这一趟的计划只点不归本机的块。新池新建文件那条流 3 块，平滑加权轮转
/// 把第 0、2 块分给 0/2、第 1 块（发布那个点）分给 1/2；发布那个点挂一个只数状态的观察者。导入之后这条流一块没判的都没有；
/// 本机库里每块每种结论（两种判器、观察者）按内容读回，与一台把整条流判完的那份库逐项相同；导入写的结论块数等于第二台那一份库里的
/// 结论块数，钉成绝对值 2（CPU 判器 1 块、观察者 1 块；没配 GPU，GPU 判器那一种两边都没有）。
/// 四趟都没开 GPU（第 ② 段由 CPU 参照核），第二台那一块只在第二台的库里核过（攻方 P4-1 的 h02 形状）：
/// 导入那一趟第 ③ 段比对没有一块缺核对行，本机库里每块在 CPU 核对代码下的核对行与一台判完的逐项相同。
/// 再带同一份导入跑一趟（P5-2）：要导的全是本库已有且相同的，`imported_blocks` 照数，与第一次导入相同。
/// 判别力（`crates/mutations.tsv`）：排键时选块条件反过来（点的是本机那一份），第二台那一块导不进来、对账剩 1 块；
/// 排键漏掉观察者那一种，导入只写 1 块、本机库里发布那一块没有观察者的结论；排键一行核对行都不点，导入那一趟比对缺 1 块核对行；
/// 导入块数只数写下的、不数已有且相同的，再导一次报 0。
#[test]
fn importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow()
{
    let recorded =
        record_new_pool_file_creation("crash-amplification-import-planned", false, file_content());
    let observe_by_counting = count_each_state_for_the_import_probe;
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("import_probe").expect("观察者的名字非空、不长"),
        observer_code_file: THIS_FILE.to_string(),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: vec![NodeOfAnObserver {
            setting_file: THIS_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::CountsOnly(&observe_by_counting),
    }];
    let plan = plan_crash_points(&recorded.flow());
    assert_eq!(total_blocks(&plan), 3, "前提：两个点与末节点各一块");
    let shares = WeightedShares::new(vec![1, 1]);
    let blocks_of_the_other_share = (0..total_blocks(&plan))
        .filter(|block| shares.owner_of_block(*block) == 1)
        .count();
    assert_eq!(
        blocks_of_the_other_share, 1,
        "前提：1/2 那一份只有第 1 块（发布那个点）"
    );
    let directory = EnvironmentRunDirectory::new("import-planned");
    let one_host_library = directory.directory.join("one-host-library");
    let other_share_library = directory.directory.join("other-share-library");
    let this_share_library = directory.library.clone();
    let mut environment = directory.environment();
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        one_host_library.as_os_str(),
    );
    let one_host_run = amplify_with_observers(&recorded, "import-planned", &observers);
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        other_share_library.as_os_str(),
    );
    environment.set_variable("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2"));
    let other_share_run = amplify_with_observers(&recorded, "import-planned", &observers);
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        this_share_library.as_os_str(),
    );
    environment.set_variable("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
    let this_share_run = amplify_with_observers(&recorded, "import-planned", &observers);
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        other_share_library.as_os_str(),
    );
    let importing_run = amplify_with_observers(&recorded, "import-planned", &observers);
    let importing_again_run = amplify_with_observers(&recorded, "import-planned", &observers);
    drop(environment);
    assert_eq!(one_host_run.unjudged_blocks, 0, "一台判完整条流");
    let comparison_of_the_import = importing_run
        .comparison
        .as_ref()
        .expect("没开 GPU：第 ② 段由 CPU 参照核，导入那一趟核了、比了");
    assert_eq!(
        (
            comparison_of_the_import.blocks_without_verifier,
            comparison_of_the_import.blocks_without_judge,
            comparison_of_the_import.disagreements
        ),
        (0, 0, 0),
        "导入那一趟第 ③ 段：第二台那一块的核对行随结论一起导进来，没有一块缺核对行"
    );
    assert_eq!(
        (
            importing_again_run.imported_blocks,
            importing_again_run.checking.blocks_checked,
            importing_again_run.unjudged_blocks
        ),
        (importing_run.imported_blocks, 0, 0),
        "再导一次：要导的全是本库已有且相同的，导入块数照数、与第一次相同，一块不判"
    );
    assert_eq!(
        (
            other_share_run.checking.blocks_checked,
            this_share_run.checking.blocks_checked
        ),
        (1, 2),
        "1/2 判第 1 块，0/2 判第 0、2 块"
    );
    assert_eq!(
        this_share_run.unjudged_blocks, 1,
        "导入之前 0/2 的库里第二台那一块没判"
    );
    assert_eq!(
        importing_run.unjudged_blocks, 0,
        "导入之后这条流一块没判的都没有"
    );
    assert_eq!(
        importing_run.checking.blocks_checked, 0,
        "导入那一趟本机那一份全复用、一块不判"
    );
    let (one_host_store, _) =
        VerdictStore::open_existing(&one_host_library).expect("开得了一台判完的库");
    let (other_share_store, _) =
        VerdictStore::open_existing(&other_share_library).expect("开得了第二台那一份库");
    let (this_share_store, _) =
        VerdictStore::open_existing(&this_share_library).expect("开得了导入之后的本机库");
    let other_share_conclusions = other_share_store
        .stored_conclusion_keys()
        .expect("列得出第二台那一份的结论");
    assert_eq!(
        other_share_conclusions.len(),
        2,
        "第二台那一份：发布那一块的 CPU 判器结论与观察者结论"
    );
    assert_eq!(
        importing_run.imported_blocks, 2,
        "导入写的结论块数等于第二台那一份的结论块数"
    );
    let one_host_conclusions = one_host_store
        .stored_conclusion_keys()
        .expect("列得出一台判完的结论");
    let this_share_conclusions = this_share_store
        .stored_conclusion_keys()
        .expect("列得出导入之后的结论");
    assert_eq!(
        one_host_conclusions.len(),
        4,
        "一台判完：3 块 CPU 判器结论、发布那一块的观察者结论"
    );
    assert_eq!(
        this_share_conclusions, one_host_conclusions,
        "导入之后本机库里的（块键，种类，版本）与一台判完的逐项相同"
    );
    for (block_key, kind, conclusion_version) in &one_host_conclusions {
        let read_back_from_one_host =
            conclusion_read_back_by_content(&one_host_store, block_key, kind, conclusion_version);
        assert!(read_back_from_one_host.is_some(), "刚列出来的结论读得到");
        assert_eq!(
            conclusion_read_back_by_content(&this_share_store, block_key, kind, conclusion_version),
            read_back_from_one_host,
            "块 {} 的 {kind:?} 结论：导入之后的本机库与一台判完的按内容读回不同",
            block_key.short_text()
        );
    }
    let mut block_keys_of_the_flow: Vec<&VerdictBlockKey> = Vec::new();
    for (block_key, _, _) in &one_host_conclusions {
        if !block_keys_of_the_flow.contains(&block_key) {
            block_keys_of_the_flow.push(block_key);
        }
    }
    assert_eq!(block_keys_of_the_flow.len(), 3, "前提：这条流 3 块");
    for block_key in block_keys_of_the_flow {
        let verifier_keys = one_host_store
            .verifier_row_keys_of_block(block_key)
            .expect("列得出一台判完的核对键");
        assert_eq!(
            verifier_keys.len(),
            1,
            "一台判完、核了：块 {} 有一把核对键（CPU 核对单元与事实表摘要）",
            block_key.short_text()
        );
        let cpu_verifier_digest = verifier_keys[0];
        let verifier_row_of_one_host = one_host_store
            .read_verifier_block(block_key, &cpu_verifier_digest)
            .expect("读得了一台判完的核对行")
            .unwrap_or_else(|| {
                panic!(
                    "一台判完、核了：块 {} 有 CPU 核对代码的核对行",
                    block_key.short_text()
                )
            });
        assert_eq!(
            this_share_store
                .read_verifier_block(block_key, &cpu_verifier_digest)
                .expect("读得了导入之后的核对行")
                .as_ref(),
            Some(&verifier_row_of_one_host),
            "块 {} 的核对行：导入之后的本机库与一台判完的读回不同",
            block_key.short_text()
        );
        let facts_of_one_host = one_host_store
            .read_flow_facts(&verifier_row_of_one_host.facts_digest)
            .expect("读得了一台判完的事实行");
        assert!(
            facts_of_one_host.is_some(),
            "一台判完：核对行点名的事实行在库里"
        );
    }
    drop((one_host_store, other_share_store, this_share_store));
    drop(directory);
}

/// 给定状态表导入用例的两项：要导入的这一项，与第二台库里另判的另一项（按名导入不该带过来的）。
const GIVEN_STATES_IMPORTED_ITEM: &str = "given_states_import_probe";
const GIVEN_STATES_OTHER_ITEM: &str = "given_states_import_probe_other_item";
const GIVEN_STATES_TABLES_OF_THE_IMPORTED_ITEM: usize = 4;
const GIVEN_STATES_TABLES_OF_THE_OTHER_ITEM: usize = 2;

/// 新池新建文件那条流上的几张给定状态表：第 k 张两个状态，持久了前（k·写数/张数 + `prefix_shift`）次写的与全部持久的
/// （一条录制流的前缀都是合法的崩溃状态），表名带 k。块键只由内容定：两项的表内容相同就是同几块，`prefix_shift` 让两项的前缀错开。
fn given_states_tables_of_the_flow(
    recorded: &RecordedFlow,
    table_count: usize,
    prefix_shift: usize,
) -> Vec<GivenStatesOfATable<'_>> {
    let write_count = recorded.writes.len();
    (0..table_count)
        .map(|table_index| {
            let persisted_prefix_writes = table_index * write_count / table_count + prefix_shift;
            GivenStatesOfATable {
                table_name: format!("import-probe-table-{table_index}"),
                base: &recorded.base,
                writes: &recorded.writes,
                segments: &recorded.segments,
                seams: &[],
                persisted_sets: vec![
                    (0..write_count)
                        .map(|write_index| write_index < persisted_prefix_writes)
                        .collect(),
                    vec![true; write_count],
                ],
            }
        })
        .collect()
}

/// 照此刻的环境起一个给定状态表的引擎，判完 `tables` 每一张、收尾，交回每张的结论与这一趟。
fn judge_every_given_states_table(
    item_name: &str,
    tables: &[GivenStatesOfATable<'_>],
) -> (Vec<GivenStatesConclusionsOfATable>, GivenStatesRun) {
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{}::new_pool_file_creation import_probe", item_name),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("CPU 配置的给定状态表引擎起得来");
    let conclusions = tables
        .iter()
        .map(|table| {
            engine
                .judge_the_given_states(table)
                .unwrap_or_else(|error| {
                    panic!("项 {item_name} 判得了表 {}：{error}", table.table_name)
                })
        })
        .collect();
    (conclusions, engine.finish())
}

/// 给定状态表那一路的导入按这一趟问到的表点名导：权重 0,1 让每张表都归 1/2 那一台；那一台判这一项的 4 张表与另一项的 2 张表、写一份库 B。
/// 本机这一份（0/2，一张表都不归它）带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=B` 判这一项的 4 张表：每张都从 B 点名导入、照库里的复用，
/// 一张都不现判。与旧的整库导入对拍（`VerdictStore::import_from` 把 B 整个导进一份空库，再在那份库上起本机这一份、不设导入，判同一批）：
/// 两边交回的结论逐张相同；按名导入的库里这一项的（块键，种类，版本）与整库导入的那份里这一项的逐项相同、按内容读回逐项相同，
/// 另一项的一行都没有。账钉成绝对值：导入 4 块（每张表一块 CPU 判器结论；没配 GPU，GPU 判器那一种 B 里没有）、复用 4 张、现判 0 张。
/// 再带同一份导入判一遍这 4 张表（P5-2）：要导的全是本库已有且相同的，导入块数照数，账与第一次相同。
/// 判别力（`crates/mutations.tsv`）：归别的机器的表不导（导入条件里的归属反过来），引擎判第一张表就报「导入之后库里还是没有」；
/// 点名漏掉 CPU 判器那一种，同样报错；导入块数只数写下的、不数已有且相同的，再导一遍报 0。
#[test]
fn given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import(
) {
    let recorded = record_new_pool_file_creation(
        "crash-amplification-given-states-import",
        false,
        file_content(),
    );
    let tables =
        given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES_OF_THE_IMPORTED_ITEM, 0);
    let other_item_tables =
        given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES_OF_THE_OTHER_ITEM, 1);
    let directory = EnvironmentRunDirectory::new("given-states-import");
    let other_host_library = directory.directory.join("other-host-library");
    let whole_import_library = directory.directory.join("whole-import-library");
    let named_import_library = directory.library.clone();
    let mut environment = directory.environment();
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
        OsStr::new("0,1"),
    );
    environment.set_variable("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2"));
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        other_host_library.as_os_str(),
    );
    let (_, other_host_run) = judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
    let (_, other_host_other_item_run) =
        judge_every_given_states_table(GIVEN_STATES_OTHER_ITEM, &other_item_tables);
    {
        let (other_host_store, _) =
            VerdictStore::open_existing(&other_host_library).expect("开得了第二台的库");
        let mut whole_import_store =
            VerdictStore::create_empty(&whole_import_library).expect("建得了整库导入那一份");
        whole_import_store
            .import_from(&other_host_store)
            .expect("整库导得进");
    }
    environment.set_variable("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        whole_import_library.as_os_str(),
    );
    let (after_the_whole_import, whole_import_run) =
        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        named_import_library.as_os_str(),
    );
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        other_host_library.as_os_str(),
    );
    let (after_the_named_import, named_import_run) =
        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
    let (after_importing_again, importing_again_run) =
        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
    drop(environment);
    assert_eq!(
        (
            other_host_run.tally.tables_judged_here,
            other_host_other_item_run.tally.tables_judged_here
        ),
        (4, 2),
        "前提：权重 0,1 下每张表都归 1/2，那一台现判了两项的每一张"
    );
    let tally_counts = |run: &GivenStatesRun| {
        (
            run.tally.imported_blocks,
            run.tally.tables_reused,
            run.tally.tables_judged_here,
            run.tally.tables_left_to_the_other_hosts,
        )
    };
    assert_eq!(
        tally_counts(&whole_import_run),
        (0, 4, 0, 0),
        "整库导入那一份：不设导入，4 张表全照库里的复用"
    );
    assert_eq!(
        tally_counts(&named_import_run),
        (8, 4, 0, 0),
        "按名导入：每张表按段两块（持久前缀那一块、全持久那一块），导入 8 块（写下的加已有且相同的）、复用 4 张、现判 0 张、留给别的机器 0 张"
    );
    assert_eq!(
        tally_counts(&importing_again_run),
        tally_counts(&named_import_run),
        "再导一遍：要导的 4 块全是本库已有且相同的，导入块数照数，账与第一次相同"
    );
    assert_eq!(
        after_importing_again, after_the_named_import,
        "再导一遍之后引擎交回的结论与第一次逐张相同"
    );
    assert_eq!(
        after_the_named_import, after_the_whole_import,
        "两种导法之后引擎交回的结论逐张相同"
    );
    let (other_host_store, _) =
        VerdictStore::open_existing(&other_host_library).expect("开得了第二台的库");
    let (whole_import_store, _) =
        VerdictStore::open_existing(&whole_import_library).expect("开得了整库导入那一份");
    let (named_import_store, _) =
        VerdictStore::open_existing(&named_import_library).expect("开得了按名导入那一份");
    let whole_import_conclusions = whole_import_store
        .stored_conclusion_keys()
        .expect("列得出整库导入那一份的结论");
    assert_eq!(
        whole_import_conclusions.len(),
        7,
        "前提：整库导入把另一项也带过来了：这一项 4 块持久前缀、另一项 2 块持久前缀、两项共用的全持久一块（块键只由内容定）"
    );
    let named_import_conclusions = named_import_store
        .stored_conclusion_keys()
        .expect("列得出按名导入那一份的结论");
    assert_eq!(
        named_import_conclusions.len(),
        5,
        "这一项 4 张表：4 块持久前缀、全持久一块，各一份 CPU 判器结论"
    );
    assert!(
        named_import_conclusions
            .iter()
            .all(|conclusion| whole_import_conclusions.contains(conclusion)),
        "按名导入的库里只有这一项的结论，都在整库导入那一份里"
    );
    for (block_key, kind, conclusion_version) in &named_import_conclusions {
        let read_back_from_the_whole_import = conclusion_read_back_by_content(
            &whole_import_store,
            block_key,
            kind,
            conclusion_version,
        );
        assert!(
            read_back_from_the_whole_import.is_some(),
            "刚列出来的结论读得到"
        );
        assert_eq!(
            conclusion_read_back_by_content(
                &named_import_store,
                block_key,
                kind,
                conclusion_version
            ),
            read_back_from_the_whole_import,
            "表 {} 的 {kind:?} 结论：两种导法按内容读回不同",
            block_key.short_text()
        );
        assert_eq!(
            conclusion_read_back_by_content(&other_host_store, block_key, kind, conclusion_version),
            read_back_from_the_whole_import,
            "表 {} 的 {kind:?} 结论：导过来的与第二台库里的按内容读回不同",
            block_key.short_text()
        );
    }
    drop((other_host_store, whole_import_store, named_import_store));
    drop(directory);
}

/// L1：只排派活计划的那一趟照实报录入时库里已判的态数，不写 0。同一份库上第一趟（单机 CPU）把整条流判完；第二趟配置开 GPU、
/// 登记 `crash_judge_gpu`、只排计划：一块都不判，复用数等于整条流的态数。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」
/// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而点名的卡找不到判红。
/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存；那一趟的卡照这个进程的卡池用（第 10t 批 K4-4：每张卡这个进程只起一个设备；
/// 第 10x 批：借到哪张卡才在哪张上起，驱动那一份记在这个进程的显存账上），不经产品路径另起一套不记账的设备。要 `verdict-store,gpu` 两个特性。
#[cfg(feature = "gpu")]
#[test]
fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {
    use singlefs_checker_tier::crash_amplification::GpuCardsSource;
    if gpu_card_pool_or_report_not_run(
        "a_placement_only_run_reports_the_states_judged_at_recording_as_reused",
    )
    .is_none()
    {
        return;
    }
    let recorded =
        record_new_pool_file_creation("crash-amplification-placement-reuse", false, file_content());
    // 清场之下要清场之后那一份显卡先后：照抄调用方给这个进程的那一份写进配置（同 crash_injection_campaign.rs；
    // 不读此刻的环境，别的用例拿着改环境的锁换着临时配置时也一样）
    let cards = singlefs_checker_tier::crash_amplification::card_priority_lines_of_the_caller(
        &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
    )
    .expect("本机的多机配置读得了");
    let directory = EnvironmentRunDirectory::new("placement-reuse");
    let mut environment = directory.environment();
    // 两趟都开 GPU：结论按（块键，结论种类，版本）存（崩溃点重放合并第三轮判决 J8），CPU 判过的块不是 GPU 那一种的复用
    let gpu_configuration = directory.directory.join("multi-host-with-gpu.env");
    std::fs::write(&gpu_configuration, format!("ENABLE_GPU=1\n{cards}"))
        .expect("写得了开 GPU 的配置");
    environment.set_variable("SINGLEFS_MULTI_HOST_CONFIG", gpu_configuration.as_os_str());
    let (first_run, _) = amplify(&recorded, "placement-reuse");
    environment.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
        OsStr::new("1"),
    );
    let (placement_run, placement_line) = amplify(&recorded, "placement-reuse");
    drop(environment);
    let state_count = first_run.recording.states_recorded;
    assert!(state_count > 0, "第一趟录了状态");
    assert!(placement_run.placement_only, "第二趟只排了派活计划");
    assert!(placement_line.is_none(), "只排计划的那一趟不打汇总行");
    assert_eq!(
        placement_run.states_reused, state_count,
        "只排计划的那一趟录入时库里整条流都判过"
    );
    assert_eq!(
        placement_run.gpu_cards_taken_from,
        GpuCardsSource::TheCardPoolOfThisProcess,
        "这个进程起过卡池：只排计划的那一趟照池里的卡用，不另起不记账的设备"
    );
    drop(directory);
}

/// K5：环境守卫里清掉、还原的那一半（`EnvironmentRestoredOnDrop`）：一趟读的变量在守卫里全是没设的，用例设的那一个照它的；
/// 守卫丢掉之后，调用方原来设了的写回原值、原来没设的仍然没设。调用方的值由外层守卫设（外层守卫拿着改环境的锁，别的用例这时不改环境）。
#[test]
fn the_environment_guard_clears_what_a_run_reads_and_restores_the_values_left_by_the_caller() {
    let mut caller = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
        OsStr::new("1/2"),
    )]);
    caller.set_variable(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        OsStr::new("/caller/library"),
    );
    {
        let mut inner = common_crash_points::EnvironmentRestoredOnDrop::saved_then_cleared(
            ENVIRONMENT_VARIABLES_READ_BY_A_RUN,
        );
        for name in ENVIRONMENT_VARIABLES_READ_BY_A_RUN {
            assert_eq!(std::env::var_os(name), None, "守卫里 {name} 是清掉的");
        }
        inner.set_variable("SINGLEFS_CRASH_AMPLIFICATION_STAGE", OsStr::new("judge"));
        assert_eq!(
            std::env::var("SINGLEFS_CRASH_AMPLIFICATION_STAGE").as_deref(),
            Ok("judge"),
            "用例设的那一个照它的"
        );
    }
    assert_eq!(
        std::env::var("SINGLEFS_CRASH_AMPLIFICATION_SHARE").as_deref(),
        Ok("1/2"),
        "调用方设了的写回原值"
    );
    assert_eq!(
        std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT").as_deref(),
        Ok("/caller/library"),
        "调用方设了的写回原值"
    );
    assert_eq!(
        std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STAGE"),
        None,
        "调用方没设的仍然没设"
    );
    drop(caller);
}

/// 用户 2026-10-13 定「配 GPU 必用 GPU」：配置开了 GPU 而这份二进制没编进 gpu 特性（提交时快档不带特性那一趟），这一趟判段不跑、报本次未跑
/// （不判红、不退回 CPU 判：一块都不判），录入段照录（库里有这条流的清单）；带特性那一趟这条用例不编，照常上 GPU 判
/// （`crash_amplification_planned_import_names_the_gpu_conclusions.rs`）。判别力（`crates/mutations.tsv`）：不带特性时照 CPU 判下去，这条红。
#[cfg(not(feature = "gpu"))]
#[test]
fn a_binary_without_the_gpu_feature_reports_the_judge_not_run_when_the_configuration_switches_the_gpu_on(
) {
    let recorded = record_new_pool_file_creation(
        "crash-amplification-gpu-without-the-feature",
        false,
        file_content(),
    );
    let directory = EnvironmentRunDirectory::new("gpu-without-the-feature");
    std::fs::write(&directory.configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let environment = directory.environment();
    let (run, line) = amplify(&recorded, "gpu-without-the-feature");
    drop(environment);
    assert!(
        run.not_run_without_the_gpu_feature
            .as_ref()
            .is_some_and(|reason| reason.contains("ENABLE_GPU=1")),
        "判段报本次未跑：{:?}",
        run.not_run_without_the_gpu_feature
    );
    assert!(run.recorded_only, "录入段照录、判段没跑");
    assert_eq!(
        (run.checking.blocks_checked, run.checking.states_checked),
        (0, 0),
        "不退回 CPU 判：一块都不判"
    );
    assert_eq!(line, None, "判段没跑的那一趟不打判过的那一行");
    let (store, _) = VerdictStore::open_existing(&directory.library).expect("开得了库");
    assert_eq!(
        store.recorded_run_manifest_count().expect("读得了清单"),
        1,
        "录入段照录：库里有这条流的清单（按录入键登）"
    );
}

/// 用户 2026-10-13 定「配 GPU 必用 GPU、做不到报错不降级」：崩溃枚举用例那一路（`crash_state_enumeration::enumerate_crash_states_with_expansion`）
/// 遇配置开了 GPU 而这份二进制没编进 gpu 特性，照崩溃放量那一处的写法：录入段照录（库里有这一项的清单），判段不判、报本次未跑
/// （`crash_state_enumeration::crash_point_replay_judge_not_run_line`）、交只录（`.judged()` 是 `None`，用例照它收工），库里一块判器结论都没有
/// （不退回 CPU 判）。新池新建文件那条流当一个崩溃点，只展开写数不超过 6 的段。带特性那一趟这条用例不编，照常上 GPU 判。
/// 判别力（`crates/mutations.tsv`）：选路把这一种退回 CPU 判，这条红；判段闭包不先交只录，这条红。
// crash-case-check:not-a-crash-case 新池新建文件那条流只展开写数不超过 6 的段，几十个状态，判段又不判，单跑几秒
#[cfg(not(feature = "gpu"))]
#[test]
fn a_crash_enumeration_case_without_the_gpu_feature_records_and_reports_the_judge_not_run_when_the_configuration_switches_the_gpu_on(
) {
    use singlefs_checker_tier::crash_state_enumeration::{
        enumerate_crash_states_with_expansion, CrashPointReplayItem,
    };
    let recorded = record_new_pool_file_creation(
        "crash-enumeration-gpu-without-the-feature",
        false,
        file_content(),
    );
    let directory = EnvironmentRunDirectory::new("crash-enumeration-gpu-without-the-feature");
    std::fs::write(&directory.configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let environment = directory.environment();
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let item_name = "crash_enumeration_gpu_without_the_feature";
    let outcome = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        singlefs_checker_tier::crash::CrashPointReplayVersionTable::with_a_file_only(
            &recorded.versions,
        ),
        &expansion,
        &[],
    );
    drop(environment);
    assert!(outcome.judged().is_none(), "判段不判、交只录：用例照它收工");
    let (store, _) = VerdictStore::open_existing(&directory.library).expect("开得了库");
    assert_eq!(
        store.recorded_run_manifest_count().expect("读得了清单"),
        1,
        "录入段照录：库里有这一项的清单（按录入键登）"
    );
    let judge_conclusions = store
        .stored_conclusion_keys()
        .expect("列得出库里的结论")
        .into_iter()
        .filter(|(_key, kind, _version)| matches!(kind, ConclusionKind::ImageJudgement))
        .count();
    assert_eq!(
        judge_conclusions, 0,
        "不退回 CPU 判：库里一块判器结论都没有"
    );
}

/// 用户 2026-10-13 定「能走两段的都走两段」：CPU 取样（`crash_state_enumeration::sample_crash_states_on_the_cpu`，实验装置逐状态取样那一路）
/// 从库里读回流、逐状态在 CPU 上判，取样序号段整块罩住的块照 CPU 判器那一处（`JudgedBy::CpuJudge`）存进崩溃点重放枚举那一类的库。
/// 新池新建文件那条流按步切成前缀树的节点（L6，`crash_state_enumeration` 的 `crash_points_of_the_steps`，连同末节点）：先取一个空序号段
/// 得整条流的状态数（一块都不存），再取全部状态，库里 CPU 判器那一种结论的块数等于录入段录下的块数（这条用例的库是空库起跑的，
/// 录入表里只有这条流计划的块）。
/// 判别力（`crates/mutations.tsv`）：取样一块都不认（结论不入库），这条红。
// crash-case-check:not-a-crash-case 新池新建文件那条流只展开写数不超过 6 的段，几十个状态，单跑几秒
#[test]
fn sampling_every_crash_state_on_the_cpu_stores_the_cpu_judge_conclusion_of_every_block() {
    use singlefs_checker_tier::crash::CrashStateJudgement;
    use singlefs_checker_tier::crash_state_enumeration::{
        sample_crash_states_on_the_cpu, CrashPointReplayItem,
    };
    let recorded = record_new_pool_file_creation(
        "crash-amplification-sampled-on-the-cpu",
        false,
        file_content(),
    );
    let directory = EnvironmentRunDirectory::new("sampled-on-the-cpu");
    let environment = directory.environment();
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let item = CrashPointReplayItem {
        name: "sampled_new_pool_file_creation",
        setting_file: THIS_FILE,
    };
    let states = sample_crash_states_on_the_cpu(
        item,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &expansion,
        0..0,
        &mut |_ordinal: u64, _image: &CrashImage<'_>, _judgement: &CrashStateJudgement| {},
    )
    .judged()
    .expect("两段先后跑：判段交回整条流的状态数");
    let (store_before_sampling, _) =
        VerdictStore::open_existing(&directory.library).expect("开得了库");
    let cpu_judge_blocks_of = |store: &VerdictStore| {
        store
            .stored_conclusion_keys()
            .expect("列得出库里的结论")
            .into_iter()
            .filter(|(_key, kind, _version)| *kind == ConclusionKind::ImageJudgement)
            .count()
    };
    assert_eq!(
        cpu_judge_blocks_of(&store_before_sampling),
        0,
        "空序号段一块都不存"
    );
    drop(store_before_sampling);
    let mut sampled = 0u64;
    sample_crash_states_on_the_cpu(
        item,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &expansion,
        0..states,
        &mut |_ordinal: u64, _image: &CrashImage<'_>, _judgement: &CrashStateJudgement| {
            sampled += 1;
        },
    )
    .judged()
    .expect("两段先后跑：判段判完");
    drop(environment);
    assert_eq!(sampled, states, "取样把每个状态都交回装置");
    let (store_after_sampling, _) =
        VerdictStore::open_existing(&directory.library).expect("开得了库");
    let recorded_blocks = store_after_sampling
        .recorded_blocks()
        .expect("读得了录入表")
        .len();
    assert!(
        recorded_blocks > 2,
        "新池新建文件那条流按步切成几个节点，块不止一个崩溃点加末节点那两块：录入表里 {recorded_blocks} 块"
    );
    assert_eq!(
        cpu_judge_blocks_of(&store_after_sampling),
        recorded_blocks,
        "整条流取样完：录入段录下的每一块（按步切成的每个节点的块连同末节点那一块）的 CPU 判器结论都在库里"
    );
}
