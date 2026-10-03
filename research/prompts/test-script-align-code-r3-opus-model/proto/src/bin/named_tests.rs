//! L1 / L4：两流用例文件里三条被变异行点名的用例，照它们的造法与断言（逐条、按用例里的次序）在原型里重算，打出每条断言 PASS / FAIL，
//! 与「第一条 FAIL 的断言」（cargo test 里用例就红在那一条上）。checker 档的测试本身不跑。
//! - M3 `a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts`：变异表第 1587 行（R3_MUT_1587）与第 1732 行（R3_MUT_1732）点名它；
//! - `a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone`：第 1731 行（R3_MUT_1731）；
//! - `root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs`：第 1733 行（R3_MUT_1733）。
//! 跑哪条变异由仓副本里的环境变量开关定（patch_mutations.py），这里只打出开了哪几个。展开上限照用例取 6。
use std::path::PathBuf;

use r3proto::flows::{content_of, two_flow, RecordedFlow};
use singlefs_checker_tier::crash::{judge_layer0_state_range, layer0_plan_state_count};
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, plan_crash_points, read_verdict_items_of_every_state, record_crash_points, CrashFlow,
    CrashPlan, PipelineJudgingCode, UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::CheckpointTxg;
use singlefs_harness::memory_pool::{root_identity_of_write, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;

const EXPAND: usize = 6;

struct Test {
    name: &'static str,
    first_fail: Option<String>,
}
impl Test {
    fn new(name: &'static str) -> Self {
        Self { name, first_fail: None }
    }
    fn check(&mut self, what: &str, ok: bool, detail: String) {
        println!("NT test={} assertion={what} {} {detail}", self.name, if ok { "PASS" } else { "FAIL" });
        if !ok && self.first_fail.is_none() {
            self.first_fail = Some(what.to_string());
        }
    }
    fn end(self) {
        println!("NTEND test={} first_failing_assertion={}", self.name, self.first_fail.as_deref().unwrap_or("none(green)"));
    }
}

fn version() -> JudgeVersion {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tree").canonicalize().expect("副本仓根");
    PipelineJudgingCode::of_the_judges(&root, &Layer0ToolchainIdentity::of_the_cargo_running_this_test()).expect("判法").version()
}

fn fresh(flow: &RecordedFlow) -> Vec<Option<Vec<String>>> {
    let count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, flow.expansion());
    let mut judged = Vec::new();
    let _ = judge_layer0_state_range(&flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions, flow.expansion(), 0..count, &mut |_o, _i, j| judged.push(Some(j.red_items())));
    judged
}

fn prefix_length(plan: &CrashPlan) -> usize {
    plan.points.iter().take_while(|point| !point.name.starts_with("publish_overwrite.")).count()
}

fn identities(plan: &CrashPlan) -> Vec<String> {
    plan.points.iter().map(|point| point.identity.clone()).collect()
}

fn later_read_back(first: &RecordedFlow, later: &RecordedFlow, name: &str, version: &JudgeVersion) -> Vec<Option<Vec<String>>> {
    let directory = PathBuf::from(std::env::var("R3_SCRATCH").expect("R3_SCRATCH")).join(format!("lib-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let mut store = VerdictStore::create_empty(&directory).expect("建得了库");
    for flow in [first, later] {
        let plan = plan_crash_points(&flow.flow());
        record_crash_points(&mut store, &plan, version, &mut CoverageReport::default()).expect("录得进");
        check_recorded_blocks(&mut store, &flow.flow(), &plan, &WeightedShares::new(vec![1]), 0, &UnitChecksumsOn::CpuOnly, version).expect("核得完");
    }
    let read_back = read_verdict_items_of_every_state(&store, &plan_crash_points(&later.flow()), version).expect("读得了");
    drop(store);
    let _ = std::fs::remove_dir_all(directory);
    read_back
}

fn miswritten(mut flow: RecordedFlow) -> RecordedFlow {
    flow.versions.iter_mut().find(|v| v.checkpoint_txg == CheckpointTxg(4)).expect("txg 4").content[0] ^= 0x01;
    flow
}

fn last_write_flipped(writes: &[RetainedWrite]) -> Vec<RetainedWrite> {
    let mut flipped = writes.to_vec();
    match &mut flipped.last_mut().expect("写表不空").contents {
        WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,
        WrittenContents::Zeros { .. } => panic!("最后一次写带字节"),
    }
    flipped
}

fn main() {
    let switches: Vec<&str> = ["R3_MUT_1587", "R3_MUT_1731", "R3_MUT_1732", "R3_MUT_1733"].into_iter().filter(|name| std::env::var_os(name).is_some()).collect();
    println!("NT mutation_switches_on={switches:?}");
    let version = version();
    // M3（用例第 870 行起），断言次序照用例
    {
        let mut test = Test::new("a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts");
        let healthy = two_flow("nt-m3-healthy", &content_of(11, 4100), true, EXPAND);
        let mut twin = two_flow("nt-m3-twin", &content_of(11, 4100), true, EXPAND);
        let root_index = twin.judged_root_index;
        if let WrittenContents::Bytes(bytes) = &mut twin.writes[root_index].contents { bytes[0] ^= 0x01; }
        let (ph, pt) = (plan_crash_points(&healthy.flow()), plan_crash_points(&twin.flow()));
        let owner = pt.points.iter().position(|p| p.segments.iter().any(|s| twin.segments[*s].contains(&root_index))).expect("归某个点");
        test.check("owner_has_states", !pt.points[owner].blocks().is_empty(), format!("owner={owner} name={}", pt.points[owner].name));
        test.check("judge_inputs_equal", pt.points[owner].judge_inputs == ph.points[owner].judge_inputs, String::new());
        test.check("plan_hash_equal", pt.points[owner].plan_hash == ph.points[owner].plan_hash, String::new());
        test.check("reuse_key_differs", pt.points[owner].reuse_key != ph.points[owner].reuse_key, String::new());
        let read_back = later_read_back(&healthy, &twin, "m3", &version);
        let fresh_twin = fresh(&twin);
        let differing: Vec<usize> = (0..fresh_twin.len()).filter(|i| read_back.get(*i) != Some(&fresh_twin[*i])).collect();
        test.check("read_back==fresh_twin", read_back == fresh_twin, format!("states={} differing_ordinals={differing:?}", fresh_twin.len()));
        let after = pt.points[owner].ordinals.end as usize;
        let fresh_healthy = fresh(&healthy);
        test.check("through_the_owner_judged_alike", fresh_healthy[..after] == fresh_twin[..after], format!("after_the_owner_ordinal={after}"));
        let after_differ = (after..fresh_twin.len()).filter(|i| fresh_healthy[*i] != fresh_twin[*i]).count();
        test.check("after_the_owner_judged_differently", fresh_healthy[after..] != fresh_twin[after..], format!("states_after={} differing={after_differ}", fresh_twin.len() - after));
        test.end();
    }
    // 第 806 行
    {
        let mut test = Test::new("a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone");
        let healthy = two_flow("nt-reach-healthy", &content_of(11, 4100), true, EXPAND);
        let bad = miswritten(two_flow("nt-reach-miswritten", &content_of(11, 4100), true, EXPAND));
        let (ph, pm) = (plan_crash_points(&healthy.flow()), plan_crash_points(&bad.flow()));
        let prefix = prefix_length(&ph);
        test.check("has_a_tail", prefix < ph.points.len(), format!("prefix={prefix} points={}", ph.points.len()));
        for index in 0..prefix {
            test.check(&format!("prefix_point_{index}_judge_inputs_equal"), ph.points[index].judge_inputs == pm.points[index].judge_inputs, ph.points[index].name.clone());
        }
        for index in prefix..ph.points.len() {
            test.check(&format!("tail_point_{index}_judge_inputs_differ"), ph.points[index].judge_inputs != pm.points[index].judge_inputs, ph.points[index].name.clone());
        }
        test.end();
    }
    // 第 943 行
    {
        let mut test = Test::new("root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs");
        let recorded = two_flow("nt-txg", &content_of(11, 4100), true, EXPAND);
        let roots: Vec<usize> = recorded.writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
        let (previous_root, last_root) = (roots[roots.len() - 2], roots[roots.len() - 1]);
        let last_write = recorded.writes.len() - 1;
        test.check("last_write_after_the_last_root", last_root < last_write, format!("roots={roots:?} last_write={last_write}"));
        let previous_txg = root_identity_of_write(&recorded.writes[previous_root]).1;
        let mut not_increasing = recorded.writes.clone();
        if let WrittenContents::Bytes(bytes) = &mut not_increasing[last_root].contents { bytes[28..36].copy_from_slice(&previous_txg.0.to_le_bytes()); }
        test.check("txg_equal_now", root_identity_of_write(&not_increasing[last_root]).1 == previous_txg, String::new());
        let plan_over = |writes: &[RetainedWrite]| plan_crash_points(&CrashFlow {
            base: &recorded.base, writes, segments: &recorded.segments, judged_root_index: recorded.judged_root_index,
            versions: &recorded.versions, expansion: recorded.expansion(), crash_points: recorded.crash_points.clone(),
        });
        let increasing = plan_over(&recorded.writes);
        let increasing_flipped = plan_over(&last_write_flipped(&recorded.writes));
        let not_inc = plan_over(&not_increasing);
        let not_inc_flipped = plan_over(&last_write_flipped(&not_increasing));
        let before_last = increasing.points.len() - 1;
        test.check("at_least_two_points_before_the_last", before_last >= 2, format!("{before_last}"));
        for index in 0..before_last {
            test.check(&format!("point_{index}_identity_unchanged"), identities(&not_inc)[index] == identities(&not_inc_flipped)[index], String::new());
            test.check(&format!("point_{index}_not_increasing_judge_inputs_differ"), not_inc.points[index].judge_inputs != not_inc_flipped.points[index].judge_inputs, not_inc.points[index].name.clone());
            test.check(&format!("point_{index}_increasing_judge_inputs_equal"), increasing.points[index].judge_inputs == increasing_flipped.points[index].judge_inputs, increasing.points[index].name.clone());
        }
        test.end();
    }
}
