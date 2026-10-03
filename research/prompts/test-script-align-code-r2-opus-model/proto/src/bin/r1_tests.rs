//! 两流用例文件（crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs）里与 R1、M3 有关的七条用例，
//! 照它们的造法与断言在原型里重算一遍（checker 档的测试本身不跑）：每条断言打一行 PASS / FAIL 与两边的值。
//! 展开上限照用例取 6。R2_FIX_BOUND_BY_THE_BASE_ROOT 设了时，仓副本里的判器输入按攻方腿的改法算（patch_tree_fix.py）。
use std::path::PathBuf;

use r2proto::flows::{content_of, two_flow, RecordedFlow};
use singlefs_checker_tier::crash::{judge_layer0_state_range, layer0_plan_state_count};
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, plan_crash_points, read_verdict_items_of_every_state, record_crash_points, CheckingTally,
    CrashPlan, PipelineJudgingCode, RecordingTally, UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::CheckpointTxg;
use singlefs_harness::memory_pool::WrittenContents;
use singlefs_harness::segments::StepKind;

const EXPAND: usize = 6;

fn check(test: &str, what: &str, ok: bool, detail: String) {
    println!("R1TEST test={test} assertion={what} {} {detail}", if ok { "PASS" } else { "FAIL" });
}

fn store(name: &str) -> (PathBuf, VerdictStore) {
    let directory = PathBuf::from(std::env::var("R2_SCRATCH").expect("R2_SCRATCH")).join(format!("lib-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let store = VerdictStore::create_empty(&directory).expect("建得了库");
    (directory, store)
}

fn version() -> JudgeVersion {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tree").canonicalize().expect("副本仓根");
    PipelineJudgingCode::of_the_judges(&root, &Layer0ToolchainIdentity::of_the_cargo_running_this_test()).expect("判法").version()
}

fn prefix_length(plan: &CrashPlan) -> usize {
    plan.points.iter().take_while(|point| !point.name.starts_with("publish_overwrite.")).count()
}
fn prefix_blocks(plan: &CrashPlan) -> u64 {
    plan.points[..prefix_length(plan)].iter().map(|point| point.blocks().len() as u64).sum()
}

fn record_and_check(store: &mut VerdictStore, flow: &RecordedFlow, plan: &CrashPlan, version: &JudgeVersion, report: &mut CoverageReport) -> (RecordingTally, CheckingTally) {
    let recording = record_crash_points(store, plan, version, report).expect("录得进");
    let checking = check_recorded_blocks(store, &flow.flow(), plan, &WeightedShares::new(vec![1]), 0, &UnitChecksumsOn::CpuOnly, version).expect("核得完");
    (recording, checking)
}

fn fresh(flow: &RecordedFlow) -> Vec<Option<Vec<String>>> {
    let count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, flow.expansion());
    let mut judged = Vec::new();
    let _ = judge_layer0_state_range(&flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions, flow.expansion(), 0..count, &mut |_o, _i, j| judged.push(Some(j.red_items())));
    judged
}

fn miswritten(mut flow: RecordedFlow) -> RecordedFlow {
    flow.versions.iter_mut().find(|v| v.checkpoint_txg == CheckpointTxg(4)).expect("txg 4").content[0] ^= 0x01;
    flow
}

fn later_read_back(first: &RecordedFlow, later: &RecordedFlow, name: &str, version: &JudgeVersion) -> (RecordingTally, Vec<Option<Vec<String>>>) {
    let (directory, mut store) = store(name);
    let plan_first = plan_crash_points(&first.flow());
    record_and_check(&mut store, first, &plan_first, version, &mut CoverageReport::default());
    let plan_later = plan_crash_points(&later.flow());
    let (recording, _) = record_and_check(&mut store, later, &plan_later, version, &mut CoverageReport::default());
    let read_back = read_verdict_items_of_every_state(&store, &plan_later, version).expect("读得了");
    drop(store);
    let _ = std::fs::remove_dir_all(directory);
    (recording, read_back)
}

fn main() {
    println!("R1TEST fix_bound_by_the_base_root={}", std::env::var_os("R2_FIX_BOUND_BY_THE_BASE_ROOT").is_some());
    let version = version();
    // 一、two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail（第四段：B 录入时前缀全部复用）
    {
        let a = two_flow("t0-a", &content_of(3, 4100), true, EXPAND);
        let b = two_flow("t0-b", &content_of(11, 4100), false, EXPAND);
        let (plan_a, plan_b) = (plan_crash_points(&a.flow()), plan_crash_points(&b.flow()));
        let (directory, mut st) = store("t0");
        record_and_check(&mut st, &a, &plan_a, &version, &mut CoverageReport::default());
        let mut report_b = CoverageReport::default();
        let recording_b = record_crash_points(&mut st, &plan_b, &version, &mut report_b).expect("录得进");
        let prefix = prefix_length(&plan_a);
        check("two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail", "recording_b.blocks_already_judged==prefix_blocks", recording_b.blocks_already_judged == prefix_blocks(&plan_a), format!("left={} right={}", recording_b.blocks_already_judged, prefix_blocks(&plan_a)));
        let bad: Vec<&String> = report_b.lines()[..prefix].iter().filter(|line| !(line.ends_with(" reused") || line.ends_with(" not_expanded"))).collect();
        check("two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail", "prefix_lines_end_with_reused_or_not_expanded", bad.is_empty(), format!("offending={bad:?}"));
        drop(st);
        let _ = std::fs::remove_dir_all(directory);
    }
    // 二、三：只差版本表的一对，两个次序
    for (test, first_is_miswritten) in [
        ("a_healthy_flow_judged_after_its_twin_with_a_miswritten_version_table_reads_back_its_own_verdicts", true),
        ("a_flow_with_a_miswritten_version_table_judged_after_its_healthy_twin_reads_back_its_own_verdicts", false),
    ] {
        let healthy = two_flow("t2-healthy", &content_of(11, 4100), true, EXPAND);
        let bad = miswritten(two_flow("t2-miswritten", &content_of(11, 4100), true, EXPAND));
        let (first, later) = if first_is_miswritten { (&bad, &healthy) } else { (&healthy, &bad) };
        let (recording, read_back) = later_read_back(first, later, if first_is_miswritten { "t2" } else { "t3" }, &version);
        let fresh_later = fresh(later);
        check(test, "read_back==fresh", read_back == fresh_later, format!("states={}", fresh_later.len()));
        check(test, "fresh_differ", fresh(first) != fresh_later, String::new());
        let plan_later = plan_crash_points(&later.flow());
        check(test, "recording.blocks_already_judged==prefix_blocks", recording.blocks_already_judged == prefix_blocks(&plan_later), format!("left={} right={}", recording.blocks_already_judged, prefix_blocks(&plan_later)));
    }
    // 四：a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone
    {
        let healthy = two_flow("t1-healthy", &content_of(11, 4100), true, EXPAND);
        let bad = miswritten(two_flow("t1-miswritten", &content_of(11, 4100), true, EXPAND));
        let (ph, pm) = (plan_crash_points(&healthy.flow()), plan_crash_points(&bad.flow()));
        let prefix = prefix_length(&ph);
        let unequal_prefix: Vec<usize> = (0..prefix).filter(|i| ph.points[*i].judge_inputs != pm.points[*i].judge_inputs).collect();
        let equal_tail: Vec<usize> = (prefix..ph.points.len()).filter(|i| ph.points[*i].judge_inputs == pm.points[*i].judge_inputs).collect();
        check("a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone", "prefix_judge_inputs_equal", unequal_prefix.is_empty(), format!("prefix_points_with_unequal_inputs={unequal_prefix:?} first_name={:?}", unequal_prefix.first().map(|i| ph.points[*i].name.clone())));
        check("a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone", "tail_judge_inputs_differ", equal_tail.is_empty(), format!("tail_points_with_equal_inputs={equal_tail:?}"));
    }
    // 五：reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split
    {
        let mut older = PipelineJudgingCode::of_the_judges(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tree").canonicalize().expect("根"), &Layer0ToolchainIdentity::of_the_cargo_running_this_test()).expect("判法");
        let mut flipped = *older.digest();
        flipped[0] ^= 0x01;
        older.replace_digest(flipped);
        let older_version = older.version();
        let a = miswritten(two_flow("t4-a", &content_of(3, 4100), true, EXPAND));
        let b_older = miswritten(two_flow("t4-b-older", &content_of(11, 4100), true, EXPAND));
        let b = two_flow("t4-b", &content_of(11, 4100), true, EXPAND);
        let plan_b = plan_crash_points(&b.flow());
        let (directory, mut st) = store("t4");
        record_and_check(&mut st, &a, &plan_crash_points(&a.flow()), &version, &mut CoverageReport::default());
        record_and_check(&mut st, &b_older, &plan_crash_points(&b_older.flow()), &older_version, &mut CoverageReport::default());
        let (recording_b, _) = record_and_check(&mut st, &b, &plan_b, &version, &mut CoverageReport::default());
        let with_reuse = read_verdict_items_of_every_state(&st, &plan_b, &version).expect("读");
        let test = "reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split";
        check(test, "with_reuse==fresh", with_reuse == fresh(&b), String::new());
        check(test, "recording_b.blocks_already_judged==prefix_blocks", recording_b.blocks_already_judged == prefix_blocks(&plan_b), format!("left={} right={}", recording_b.blocks_already_judged, prefix_blocks(&plan_b)));
        drop(st);
        let _ = std::fs::remove_dir_all(directory);
    }
    // 六：moving_the_judged_root_slot_write_changes_the_judge_inputs_but_not_the_tree
    {
        let flow = two_flow("t6", &content_of(11, 4100), true, EXPAND);
        let plan = plan_crash_points(&flow.flow());
        let first_root = flow.writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根");
        let mut moved = flow.flow();
        moved.judged_root_index = first_root;
        let moved_plan = plan_crash_points(&moved);
        check("moving_the_judged_root_slot_write_changes_the_judge_inputs_but_not_the_tree", "some_judge_input_changes", moved_plan.points.iter().zip(&plan.points).any(|(m, o)| m.judge_inputs != o.judge_inputs), String::new());
    }
    // 七：a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts（M3）
    {
        let healthy = two_flow("t5-healthy", &content_of(11, 4100), true, EXPAND);
        let mut twin = two_flow("t5-twin", &content_of(11, 4100), true, EXPAND);
        let root_index = twin.judged_root_index;
        if let WrittenContents::Bytes(bytes) = &mut twin.writes[root_index].contents { bytes[0] ^= 0x01; }
        let (ph, pt) = (plan_crash_points(&healthy.flow()), plan_crash_points(&twin.flow()));
        let owner = pt.points.iter().position(|p| p.segments.iter().any(|s| twin.segments[*s].contains(&root_index))).expect("归某个点");
        let test = "a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts";
        check(test, "owner_has_states", !pt.points[owner].blocks().is_empty(), format!("owner={} name={}", owner, pt.points[owner].name));
        check(test, "judge_inputs_equal", pt.points[owner].judge_inputs == ph.points[owner].judge_inputs, String::new());
        check(test, "plan_hash_equal", pt.points[owner].plan_hash == ph.points[owner].plan_hash, String::new());
        check(test, "reuse_key_differs", pt.points[owner].reuse_key != ph.points[owner].reuse_key, String::new());
        let (_, read_back) = later_read_back(&healthy, &twin, "t5", &version);
        let fresh_twin = fresh(&twin);
        check(test, "read_back==fresh", read_back == fresh_twin, String::new());
        let ordinals = pt.points[owner].ordinals.start as usize..pt.points[owner].ordinals.end as usize;
        let fresh_healthy = fresh(&healthy);
        check(test, "owner_point_judged_differently", fresh_healthy[ordinals.clone()] != fresh_twin[ordinals.clone()], format!("healthy={:?} twin={:?}", &fresh_healthy[ordinals.clone()], &fresh_twin[ordinals]));
    }
}
