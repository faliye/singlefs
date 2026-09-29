//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification、verdict_store
//! 崩溃放量流水线在新池新建文件那条流（取号与暖机、发布两个崩溃点）上的端到端用例：录入 → 核对 → 第二趟复用 → 两方分工与对账 →
//! 两个库并成一个 → ignore 的点不出状态 → 版本表写错要红。要 `verdict-store` 特性（RocksDB），不带特性时这份文件是空的。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::path::PathBuf;

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    layer0_plan_state_count, Layer0SegmentExpansion, LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
};
use singlefs_checker_tier::crash_amplification::{
    check_blocks_where_until, check_recorded_blocks, compare_judge_and_verifier, plan_crash_points,
    read_violations, record_crash_points, unjudged_blocks, unjudged_blocks_of_the_flow,
    verify_blocks_from_facts, CrashFlow, CrashPlan, CrashPointSpan, FactsVerifierOn,
    PipelineJudgingCode, UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs";
/// 只展开不超过 6 次写的段：新池新建文件那条流是 2+2+1+2+2+1+2+24+2+1+2，展开的十段共几十个状态，秒级。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根在 crate 往上两层")
}

fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
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

fn judging_code() -> PipelineJudgingCode {
    PipelineJudgingCode::of_the_judges(
        &repository_root(),
        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法代码摘要算得出")
}

/// 判定行上的判法版本：两个判器的闭包摘要。
fn judge_version() -> JudgeVersion {
    judging_code().version()
}

/// 库放在临时目录，用例结束（成功、失败、panic）都删掉。
struct TemporaryStore {
    directory: PathBuf,
}

impl TemporaryStore {
    fn new(tag: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-{tag}-{}",
            std::process::id()
        ));
        Self { directory }
    }
    fn create(&self) -> VerdictStore {
        VerdictStore::create_empty(&self.directory).expect("建得了判定库")
    }
}

impl Drop for TemporaryStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
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
    assert_eq!(plan.points.len(), 2);
    assert_eq!(
        plan.points[0].ordinals.end, plan.points[1].ordinals.start,
        "两个点的状态序号首尾相接"
    );
    let state_count = layer0_plan_state_count(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    assert_eq!(
        plan.points[1].ordinals.end, state_count,
        "最后一个点收到整条流的末尾"
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
            && plan.points[1].path_number.matches('/').count() == 1,
        "路径编号一点一段"
    );
    let temporary = TemporaryStore::new("reuse");
    let mut store = temporary.create();
    let mut report = CoverageReport::default();
    let recording =
        record_crash_points(&mut store, &plan, &judge_version(), &mut report).expect("录得进");
    assert_eq!(recording.blocks_recorded, total_blocks(&plan));
    assert_eq!(recording.states_recorded, state_count);
    assert_eq!(recording.blocks_already_judged, 0);
    assert_eq!(
        unjudged_blocks(&store, &judge_version())
            .expect("对得了账")
            .len(),
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
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert_eq!(checking.blocks_checked, total_blocks(&plan));
    assert_eq!(checking.states_checked, state_count);
    assert_eq!(
        checking.red_states, 0,
        "健康的流一个红状态都不该有：{:?}",
        checking.red_texts
    );
    assert_eq!(checking.gpu_cpu_disagreements, 0);
    assert!(
        unjudged_blocks(&store, &judge_version())
            .expect("对得了账")
            .is_empty(),
        "核完对账为空"
    );
    assert!(read_violations(&store, &plan, &judge_version())
        .expect("读得了违例")
        .is_empty());
    let mut second_report = CoverageReport::default();
    let second = record_crash_points(&mut store, &plan, &judge_version(), &mut second_report)
        .expect("录得进");
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
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
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
    let temporary = TemporaryStore::new("own-blocks");
    let mut store = temporary.create();
    record_crash_points(
        &mut store,
        &plan,
        &judge_version(),
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    assert_eq!(
        unjudged_blocks_of_the_flow(&store, &plan, &judge_version())
            .expect("对得了账")
            .len(),
        2,
        "录了还没判：两块都在这条流自己的账上"
    );
    // 别的流录进同一份库的一块（拿这条流第一块的键换一个起点序号造出来），一直没判
    let mut block_of_another_flow = unjudged_blocks(&store, &judge_version())
        .expect("对得了账")
        .first()
        .expect("录了两块")
        .0
        .clone();
    block_of_another_flow.block_start.0 += 1 << 40;
    store
        .record_block(&block_of_another_flow, 7)
        .expect("录得进");
    check_recorded_blocks(
        &mut store,
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert!(
        unjudged_blocks_of_the_flow(&store, &plan, &judge_version())
            .expect("对得了账")
            .is_empty(),
        "这条流的块都判完了，它自己的账是空的"
    );
    assert_eq!(
        unjudged_blocks(&store, &judge_version()).expect("对得了账"),
        vec![(block_of_another_flow, 7)],
        "整份库的账照旧报出别的流那一块"
    );
}

/// 核到一半被叫停：不再领新的块，已经判完的那一块在库里；下一趟只判剩下的那一块，对账为空。
#[test]
fn a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest() {
    let recorded = record_new_pool_file_creation("crash-amplification-stop", false, file_content());
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    assert_eq!(total_blocks(&plan), 2, "两个点各一块");
    let temporary = TemporaryStore::new("stop");
    let mut store = temporary.create();
    record_crash_points(
        &mut store,
        &plan,
        &judge_version(),
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    // 领第一块之前问一次（不停），领第二块之前再问一次（停）
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let stop_before_the_second_block =
        || asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 1;
    let stopped = check_blocks_where_until(
        &mut store,
        &flow,
        &plan,
        &|_block| true,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
        &stop_before_the_second_block,
    )
    .expect("核得完");
    assert_eq!(stopped.blocks_checked, 1, "被叫停之前只判了第一块");
    assert_eq!(
        unjudged_blocks(&store, &judge_version())
            .expect("对得了账")
            .len(),
        1,
        "第二块还没判"
    );
    let resumed = check_blocks_where_until(
        &mut store,
        &flow,
        &plan,
        &|_block| true,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
        &|| false,
    )
    .expect("核得完");
    assert_eq!(resumed.blocks_checked, 1, "接着判的那一趟只判剩下的那一块");
    assert_eq!(
        stopped.states_checked + resumed.states_checked,
        plan.points.last().expect("有点").ordinals.end,
        "两趟合起来每个状态恰好判一次"
    );
    assert!(unjudged_blocks(&store, &judge_version())
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
    assert_eq!(total_blocks(&plan), 2, "两个点各一块");
    let shares = WeightedShares::new(vec![1, 1]);
    let first_temporary = TemporaryStore::new("share-a");
    let second_temporary = TemporaryStore::new("share-b");
    let mut first = first_temporary.create();
    let mut second = second_temporary.create();
    for store in [&mut first, &mut second] {
        record_crash_points(
            store,
            &plan,
            &judge_version(),
            &mut CoverageReport::default(),
        )
        .expect("录得进");
    }
    let first_tally = check_recorded_blocks(
        &mut first,
        &flow,
        &plan,
        &shares,
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert_eq!(first_tally.blocks_checked, 1, "第一方只核分到它的那一块");
    assert_eq!(
        unjudged_blocks(&first, &judge_version())
            .expect("对得了账")
            .len(),
        1,
        "另一方那一块在第一方的库里对账为没核完"
    );
    let second_tally = check_recorded_blocks(
        &mut second,
        &flow,
        &plan,
        &shares,
        1,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert_eq!(second_tally.blocks_checked, 1);
    let imported = first.import_from(&second).expect("导得进");
    assert_eq!(imported, 1, "第二方那一块导进第一方的库");
    assert!(
        unjudged_blocks(&first, &judge_version())
            .expect("对得了账")
            .is_empty(),
        "并成一个库之后对账为空"
    );
    assert!(read_violations(&first, &plan, &judge_version())
        .expect("读得了违例")
        .is_empty());
}

#[test]
fn an_ignored_crash_point_keeps_its_place_in_the_tree_but_contributes_only_the_final_state() {
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
        1,
        "ignore 的点只剩全持久那一个状态"
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
    // 判法版本改了：计划不看判法，树与复用键都不动；库里旧版本的判定不复用、旧行留着
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
    let temporary = TemporaryStore::new("ignored");
    let mut store = temporary.create();
    let mut report = CoverageReport::default();
    record_crash_points(&mut store, &plan, &judging.version(), &mut report).expect("录得进");
    assert!(
        report.lines()[1].ends_with(" ignored"),
        "{:?}",
        report.lines()
    );
    assert!(report.lines()[0].ends_with(" checked"));
    let checking = check_recorded_blocks(
        &mut store,
        &recorded.flow(),
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judging.version(),
    )
    .expect("核得完");
    assert!(checking.blocks_checked > 0);
    assert!(unjudged_blocks(&store, &judging.version())
        .expect("对得了账")
        .is_empty());
    let mut other_report = CoverageReport::default();
    let other_recording = record_crash_points(
        &mut store,
        &plan,
        &other_judging.version(),
        &mut other_report,
    )
    .expect("录得进");
    assert_eq!(
        other_recording.blocks_already_judged, 0,
        "判法版本换了，旧版本的判定一块都不复用"
    );
    assert_eq!(
        other_recording.blocks_recorded, checking.blocks_checked,
        "同样的块在新版本下全部待判"
    );
    assert_eq!(
        unjudged_blocks(&store, &other_judging.version())
            .expect("对得了账")
            .len(),
        usize::try_from(checking.blocks_checked).expect("块数"),
        "新版本下对账列出全部块"
    );
    assert!(
        unjudged_blocks(&store, &judging.version())
            .expect("对得了账")
            .is_empty(),
        "旧版本的行还在"
    );
    assert_eq!(
        store.judge_versions().expect("列版本").len(),
        0,
        "直接调录入不登版本描述；描述由 run_from_environment 登"
    );
}

#[test]
fn a_wrong_version_table_makes_the_pipeline_store_red_states_the_gate_can_read() {
    let mut wrong_content = file_content();
    wrong_content[0] ^= 0x01;
    let recorded = record_new_pool_file_creation("crash-amplification-red", false, wrong_content);
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let temporary = TemporaryStore::new("red");
    let mut store = temporary.create();
    record_crash_points(
        &mut store,
        &plan,
        &judge_version(),
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
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
    let violations = read_violations(&store, &plan, &judge_version()).expect("读得了违例");
    assert_eq!(
        u64::try_from(violations.len()).expect("装得进"),
        checking.red_states,
        "门禁读到的违例数等于核出的红状态数"
    );
    for red in &violations {
        assert!(
            red.items.iter().all(
                |item| item.starts_with(LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE)
                    || item.starts_with(LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE)
            ),
            "只有 oracle 那一道该红：{:?}",
            red.items
        );
        assert!(
            red.identity.ends_with("::publish_first_file::")
                || red.identity.contains("::publish_first_file::"),
            "红在发布那个点：{}",
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
    let temporary = TemporaryStore::new("verifier");
    let mut store = temporary.create();
    record_crash_points(
        &mut store,
        &plan,
        &judge_version(),
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert_eq!(checking.red_states, 0);
    let on = FactsVerifierOn::Cpu;
    let tally = verify_blocks_from_facts(&mut store, &flow, &plan, &|_| true, &on).expect("核得完");
    assert_eq!(
        tally.states_verified, checking.states_checked,
        "第 ② 段核了每个状态"
    );
    assert_eq!(tally.verifier_red_states, 0, "健康流第 ② 段也全绿");
    assert_eq!(tally.blocks_already_verified, 0);
    let comparison =
        compare_judge_and_verifier(&store, &plan, &on.verifier_digest(), &judge_version())
            .expect("比得完");
    assert_eq!(comparison.states_compared, checking.states_checked);
    assert_eq!(comparison.both_green, checking.states_checked);
    assert_eq!(comparison.disagreements, 0);
    assert_eq!(comparison.blocks_without_verifier, 0);
    assert_eq!(comparison.blocks_without_judge, 0);
    let again = verify_blocks_from_facts(&mut store, &flow, &plan, &|_| true, &on).expect("核得完");
    assert_eq!(again.blocks_verified, 0, "同一版核对代码第二趟全部复用");
    assert_eq!(again.blocks_already_verified, tally.blocks_verified);
    // 换一版「核对代码」（摘要不同）：旧行留着，新版从头核
    let mut other_digest = on.verifier_digest();
    other_digest[0] ^= 0x01;
    let elsewhere =
        compare_judge_and_verifier(&store, &plan, &other_digest, &judge_version()).expect("比得完");
    assert_eq!(elsewhere.states_compared, 0, "别的摘要下一块都没核过");
    assert_eq!(elsewhere.blocks_without_verifier, tally.blocks_verified);
}

#[test]
fn a_wrong_version_table_is_judge_red_only_for_the_facts_verifier() {
    let mut wrong_content = file_content();
    wrong_content[0] ^= 0x01;
    let recorded =
        record_new_pool_file_creation("crash-amplification-verifier-red", false, wrong_content);
    let flow = recorded.flow();
    let plan = plan_crash_points(&flow);
    let temporary = TemporaryStore::new("verifier-red");
    let mut store = temporary.create();
    record_crash_points(
        &mut store,
        &plan,
        &judge_version(),
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    let checking = check_recorded_blocks(
        &mut store,
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &judge_version(),
    )
    .expect("核得完");
    assert!(checking.red_states > 0, "版本表写错，判器红");
    let on = FactsVerifierOn::Cpu;
    let tally = verify_blocks_from_facts(&mut store, &flow, &plan, &|_| true, &on).expect("核得完");
    assert_eq!(
        tally.verifier_red_states, 0,
        "版本表是判器的输入，不是盘上的事实：第 ② 段看不见它"
    );
    let comparison =
        compare_judge_and_verifier(&store, &plan, &on.verifier_digest(), &judge_version())
            .expect("比得完");
    assert_eq!(
        comparison.judge_red_only, checking.red_states,
        "判器红而核对绿的正是那些状态"
    );
    assert_eq!(comparison.disagreements, 0);
}
