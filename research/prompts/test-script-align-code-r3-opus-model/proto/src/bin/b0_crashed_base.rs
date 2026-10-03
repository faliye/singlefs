//! L1：B0（够得着的根取基镜像里的根、点之前写过的根槽写、没做完那次发布的根槽写三者最大）在基镜像是一个崩溃状态时：
//! 基镜像里 journal 记录落了、它那一代根没落，恢复照记录重建出那一代（比基镜像里任何一条根都新）；
//! 重开之后第一批点之前没有根槽写、又没有单元写与记录写，B0 按基镜像里的根截版本表，把那一代截掉。
//! 一对只差那一代内容的流（R1 的造法）：逐点比块键三样（身份、复用键、计划哈希），块键相同的点逐态现判比 red_items。
//! 环境变量 R3_MUT_1731 设了时仓副本撤掉 B0（变异表第 1731 行：基镜像的根不进上界），当对照。
use r3proto::flows::crashed_base_then_mount;
use singlefs_checker_tier::crash::{judge_layer0_state_range, layer0_plan_state_count};
use singlefs_checker_tier::crash_amplification::{judge_input_texts_of_each_point, plan_crash_points};
use singlefs_harness::memory_pool::closed_form_state_count;

fn main() {
    let expand_up_to: usize = std::env::var("R3_EXPAND_UP_TO").ok().map_or(6, |text| text.parse().expect("数"));
    println!("B0CB b0_removed_by_mutation_1731={} expand_up_to={expand_up_to}", std::env::var_os("R3_MUT_1731").is_some());
    let healthy = crashed_base_then_mount("b0cb-healthy", expand_up_to, false);
    let twin = crashed_base_then_mount("b0cb-twin", expand_up_to, true);
    println!("B0CB base_roots={:?} cut_root={:?}", healthy.base_roots, healthy.cut_root);
    let (h, t) = (&healthy.flow, &twin.flow);
    assert!(h.writes == t.writes && h.base == t.base, "两遍录的基镜像与写表相同");
    let states = layer0_plan_state_count(&h.base, &h.writes, &h.segments, h.expansion());
    println!(
        "B0CB writes={} segments={} points={} states_this_run={states} closed_form_state_count_fully_expanded={} judged_root_index={} versions={:?}",
        h.writes.len(), h.segments.len(), h.crash_points.len(), closed_form_state_count(&h.segments), h.judged_root_index,
        h.versions.iter().map(|v| (v.instance.0, v.checkpoint_txg.0, v.content.len())).collect::<Vec<_>>()
    );
    assert!(states <= 1_000_000, "这一次的状态数超过 10^6");
    let plan_h = plan_crash_points(&h.flow());
    let plan_t = plan_crash_points(&t.flow());
    let texts_h = judge_input_texts_of_each_point(&h.flow());
    let judge = |flow: &r3proto::flows::RecordedFlow, ordinals: std::ops::Range<u64>| {
        let mut items = Vec::new();
        let _ = judge_layer0_state_range(&flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions, flow.expansion(), ordinals, &mut |_o, _i, judgement| items.push(judgement.red_items()));
        items
    };
    let (mut same_key_points, mut same_key_states, mut same_key_differing) = (0u64, 0u64, 0u64);
    for (index, (ph, pt)) in plan_h.points.iter().zip(&plan_t.points).enumerate() {
        let kinds: Vec<&str> = h.crash_points[index].writes.clone().map(|w| h.writes[w].kind.name()).collect();
        let same_key = ph.identity == pt.identity && ph.reuse_key == pt.reuse_key && ph.plan_hash == pt.plan_hash;
        let (jh, jt) = (judge(h, ph.ordinals.clone()), judge(t, pt.ordinals.clone()));
        let differing: Vec<usize> = (0..jh.len()).filter(|o| jh[*o] != jt[*o]).collect();
        let bound = texts_h[index].2.lines().find(|line| line.starts_with("versions_up_to")).unwrap_or("?").to_string();
        let unfinished = texts_h[index].2.lines().find(|line| line.starts_with("unfinished_publish_root")).unwrap_or("?").chars().take(60).collect::<String>();
        println!(
            "B0CBPOINT index={index} name={} write_kinds={kinds:?} states={} same_identity={} same_reuse_key={} same_plan_hash={} {bound} {unfinished} states_judged_differently={} healthy_red_states={}",
            ph.name, jh.len(), ph.identity == pt.identity, ph.reuse_key == pt.reuse_key, ph.plan_hash == pt.plan_hash,
            differing.len(), jh.iter().filter(|items| !items.is_empty()).count()
        );
        for offset in differing.iter().take(2) {
            println!("B0CBSTATE index={index} offset={offset} healthy={:?} twin={:?}", jh[*offset], jt[*offset]);
        }
        if same_key {
            same_key_points += 1;
            same_key_states += jh.len() as u64;
            same_key_differing += differing.len() as u64;
        }
    }
    println!("B0CB_TOTAL same_block_key_points={same_key_points} states_compared={same_key_states} judge_states_differing_under_the_same_block_key={same_key_differing}");
}
