//! K2-R1：版本表不进复用键，也不进判法版本。同一段写表、两张版本表（一张对、一张把 txg 4 那一版改坏一个字节），
//! 在同一个库、同一判法版本下先后录入核对：后来的那一条读到的是前一条的判定。与不经库逐态现判的结果逐态比。
//! 另照 crash_amplification_two_flows_share_their_prefix.rs 的复用对照原样造一遍（A 尾巴红、旧判法版本下的 B 红、再判 B），看它红不红。
use k2proto::flows::{
    content_of, expansion, fresh_store, record_and_check, record_flow, repository_root,
    with_the_overwritten_version_miswritten, RecordedFlow,
};
use singlefs_checker_tier::crash::{judge_layer0_state_range, layer0_plan_state_count};
use singlefs_checker_tier::crash_amplification::{
    plan_crash_points, read_verdict_items_of_every_state, PipelineJudgingCode,
};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_harness::memory_pool::closed_form_state_count;

const STATE_LIMIT: u64 = 1_000_000;

fn fresh_items(flow: &RecordedFlow, state_count: u64) -> Vec<Option<Vec<String>>> {
    let mut items = Vec::new();
    let judged = judge_layer0_state_range(
        &flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions,
        &expansion, 0..state_count,
        &mut |_ordinal, _image, judgement| items.push(Some(judgement.red_items())),
    );
    assert_eq!(judged, state_count);
    items
}

fn red_count(items: &[Option<Vec<String>>]) -> usize {
    items.iter().filter(|item| item.as_ref().is_some_and(|red| !red.is_empty())).count()
}

fn differing(stored: &[Option<Vec<String>>], fresh: &[Option<Vec<String>>]) -> (usize, Option<usize>) {
    assert_eq!(stored.len(), fresh.len());
    let positions: Vec<usize> = (0..fresh.len()).filter(|index| stored[*index] != fresh[*index]).collect();
    (positions.len(), positions.first().copied())
}

fn read_back(store: &VerdictStore, flow: &RecordedFlow, version: &JudgeVersion) -> Vec<Option<Vec<String>>> {
    read_verdict_items_of_every_state(store, &plan_crash_points(&flow.flow()), version).expect("读得了判定")
}

fn main() {
    let root = repository_root();
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    let judging = PipelineJudgingCode::of_the_judges(&root, &toolchain).expect("判法");
    let version = judging.version();

    let flow_b = record_flow("k2-reuse-b", &content_of(11, 4100), true);
    let flow_b_miswritten = with_the_overwritten_version_miswritten(flow_b.clone());
    let state_count = layer0_plan_state_count(&flow_b.base, &flow_b.writes, &flow_b.segments, &expansion);
    println!(
        "K2R1 name=sizes states_this_run={state_count} closed_form_state_count_fully_expanded={} segments={} writes={}",
        closed_form_state_count(&flow_b.segments), flow_b.segments.len(), flow_b.writes.len()
    );
    assert!(state_count <= STATE_LIMIT, "这一次的状态数超过 10^6");
    assert!(flow_b.writes == flow_b_miswritten.writes, "两条流写表逐字节相同");
    let plan_b = plan_crash_points(&flow_b.flow());
    let plan_m = plan_crash_points(&flow_b_miswritten.flow());
    let same_keys = plan_b.points.iter().zip(&plan_m.points).all(|(left, right)| {
        left.reuse_key == right.reuse_key && left.identity == right.identity && left.plan_hash == right.plan_hash
    });
    println!(
        "K2R1 name=keys points={} every_point_same_reuse_key_identity_plan_hash={same_keys} versions_differ={}",
        plan_b.points.len(), flow_b.versions != flow_b_miswritten.versions
    );

    let fresh_b = fresh_items(&flow_b, state_count);
    let fresh_m = fresh_items(&flow_b_miswritten, state_count);
    println!(
        "K2R1 name=fresh healthy_red_states={} miswritten_red_states={}",
        red_count(&fresh_b), red_count(&fresh_m)
    );

    // 次序一：版本表写错的那一条先进库，健康的那一条后进
    let (directory_one, mut store_one) = fresh_store("order-one");
    let (_, first) = record_and_check(&mut store_one, &flow_b_miswritten, &plan_m, &version);
    let (reused_blocks, second) = record_and_check(&mut store_one, &flow_b, &plan_b, &version);
    let stored = read_back(&store_one, &flow_b, &version);
    let (count, first_position) = differing(&stored, &fresh_b);
    println!(
        "K2R1 name=order_miswritten_then_healthy first_red_states={} healthy_blocks_reused={reused_blocks} healthy_states_checked={} healthy_stored_red_states={} healthy_fresh_red_states={} states_differing_from_fresh={count} first_differing={first_position:?}",
        first.red_states, second.states_checked, red_count(&stored), red_count(&fresh_b)
    );
    drop(store_one);
    let _ = std::fs::remove_dir_all(&directory_one);

    // 次序二：健康的先进库，版本表写错的后进
    let (directory_two, mut store_two) = fresh_store("order-two");
    let (_, first) = record_and_check(&mut store_two, &flow_b, &plan_b, &version);
    let (reused_blocks, second) = record_and_check(&mut store_two, &flow_b_miswritten, &plan_m, &version);
    let stored = read_back(&store_two, &flow_b_miswritten, &version);
    let (count, first_position) = differing(&stored, &fresh_m);
    println!(
        "K2R1 name=order_healthy_then_miswritten first_red_states={} miswritten_blocks_reused={reused_blocks} miswritten_states_checked={} miswritten_stored_red_states={} miswritten_fresh_red_states={} states_differing_from_fresh={count} first_differing={first_position:?}",
        first.red_states, second.states_checked, red_count(&stored), red_count(&fresh_m)
    );
    drop(store_two);
    let _ = std::fs::remove_dir_all(&directory_two);

    // 合入的复用对照原样：A（内容 3、版本表写错）→ 旧判法版本下的 B（版本表写错）→ B
    let mut older_judging = judging.clone();
    let mut flipped = *judging.digest();
    flipped[0] ^= 0x01;
    older_judging.replace_digest(flipped);
    let older_version = older_judging.version();
    let flow_a = with_the_overwritten_version_miswritten(record_flow("k2-control-a", &content_of(3, 4100), true));
    let flow_b_older = with_the_overwritten_version_miswritten(record_flow("k2-control-b-older", &content_of(11, 4100), true));
    let (directory_three, mut store_three) = fresh_store("control-as-merged");
    let (_, checking_a) = record_and_check(&mut store_three, &flow_a, &plan_crash_points(&flow_a.flow()), &version);
    let (_, checking_older) = record_and_check(&mut store_three, &flow_b_older, &plan_crash_points(&flow_b_older.flow()), &older_version);
    let (reused_blocks, _) = record_and_check(&mut store_three, &flow_b, &plan_b, &version);
    let stored = read_back(&store_three, &flow_b, &version);
    let (count, _) = differing(&stored, &fresh_b);
    println!(
        "K2R1 name=control_as_merged a_red_states={} older_b_red_states={} b_blocks_reused={reused_blocks} states_differing_from_fresh={count}",
        checking_a.red_states, checking_older.red_states
    );
    drop(store_three);
    let _ = std::fs::remove_dir_all(&directory_three);

    // 同一个对照，只把「旧判法版本下的 B」换成「同一判法版本下的 B（版本表写错）」：输入只差版本表
    let (directory_four, mut store_four) = fresh_store("control-same-version");
    let _ = record_and_check(&mut store_four, &flow_a, &plan_crash_points(&flow_a.flow()), &version);
    let (_, checking_same) = record_and_check(&mut store_four, &flow_b_older, &plan_crash_points(&flow_b_older.flow()), &version);
    let (reused_blocks, _) = record_and_check(&mut store_four, &flow_b, &plan_b, &version);
    let stored = read_back(&store_four, &flow_b, &version);
    let (count, first_position) = differing(&stored, &fresh_b);
    println!(
        "K2R1 name=control_with_the_miswritten_b_under_the_same_version miswritten_b_red_states={} b_blocks_reused={reused_blocks} states_differing_from_fresh={count} first_differing={first_position:?}",
        checking_same.red_states
    );
    drop(store_four);
    let _ = std::fs::remove_dir_all(&directory_four);
}
