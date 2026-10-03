//! J3 第 1587 行（M3）：两流用例的 `a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts` 照它的造法造一对
//! （同一次覆盖写录两遍，第二遍把覆盖写那一次的根槽写第一个字节改坏），逐点比块键三样，逐态现判比判定：
//! 用例推的「管根槽写的那个点上判得不同」成不成立；另看管它前一个点（journal 记录那一段）判定同不同（那个点只靠
//! 「没做完的那次发布的根槽写」那一行分开，判定相同的话，那一行拿掉也没有用例会红）。
use r2proto::flows::{content_of, two_flow};
use singlefs_checker_tier::crash::judge_layer0_state_range;
use singlefs_checker_tier::crash_amplification::plan_crash_points;
use singlefs_harness::memory_pool::{closed_form_state_count, WrittenContents};

fn main() {
    let expand_up_to: usize = std::env::var("R2_EXPAND_UP_TO").ok().map_or(6, |text| text.parse().expect("数"));
    let healthy = two_flow("m3-healthy", &content_of(11, 4100), true, expand_up_to);
    let mut twin = two_flow("m3-twin", &content_of(11, 4100), true, expand_up_to);
    let root_index = twin.judged_root_index;
    match &mut twin.writes[root_index].contents {
        WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,
        WrittenContents::Zeros { .. } => panic!("根槽写带字节"),
    }
    println!("M3 closed_form_state_count_fully_expanded={} expand_up_to={expand_up_to} root_index={root_index}", closed_form_state_count(&twin.segments));
    let plan_healthy = plan_crash_points(&healthy.flow());
    let plan_twin = plan_crash_points(&twin.flow());
    let fresh = |flow: &r2proto::flows::RecordedFlow, ordinals: std::ops::Range<u64>| {
        let mut items = Vec::new();
        let _ = judge_layer0_state_range(&flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions, flow.expansion(), ordinals, &mut |_o, _i, judgement| items.push(judgement.red_items()));
        items
    };
    for (index, (h, t)) in plan_healthy.points.iter().zip(&plan_twin.points).enumerate() {
        let owns_root = t.segments.iter().any(|segment| twin.segments[*segment].contains(&root_index));
        let judged_h = fresh(&healthy, h.ordinals.clone());
        let judged_t = fresh(&twin, t.ordinals.clone());
        let differing: Vec<usize> = (0..judged_h.len()).filter(|offset| judged_h[*offset] != judged_t[*offset]).collect();
        println!(
            "M3POINT index={index} name={} owns_the_flipped_root_write={owns_root} states={} same_identity={} same_reuse_key={} same_judge_inputs={} same_plan_hash={} states_judged_differently={}",
            t.name, judged_t.len(), h.identity == t.identity, h.reuse_key == t.reuse_key, h.judge_inputs == t.judge_inputs, h.plan_hash == t.plan_hash, differing.len()
        );
        for offset in differing.iter().take(4) {
            println!("M3STATE index={index} offset={offset} healthy={:?} twin={:?}", judged_h[*offset], judged_t[*offset]);
        }
    }
}
