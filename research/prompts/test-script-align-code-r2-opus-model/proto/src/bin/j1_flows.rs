//! J1：几条流的根槽写（txg 严不严格递增，即 R1 退不退成整段写表那一支）、每个点的块键三样与判器输入，
//! 以及几条流里身份相同的点计划哈希相不相同、不同在判器输入的哪几行。只算计划，不判状态。
use r2proto::flows::{content_of, rollback_flow, two_flow, unmount_flow, RecordedFlow};
use singlefs_checker_tier::crash::layer0_plan_state_count;
use singlefs_checker_tier::crash_amplification::{judge_input_texts_of_each_point, plan_crash_points};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::root_identity_of_write;
use singlefs_harness::segments::StepKind;
use singlefs_harness::memory_pool::WrittenContents;

fn short(bytes: &[u8]) -> String {
    hexadecimal_text(bytes)[..12].to_string()
}

fn describe(flow: &RecordedFlow) {
    let roots: Vec<(usize, u32, u64, u32)> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, write)| {
            let (instance, txg) = root_identity_of_write(write);
            (index, instance.0, txg.0, write.device.0)
        })
        .collect();
    let strict = roots.windows(2).all(|pair| pair[0].2 < pair[1].2);
    let states = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, flow.expansion());
    println!(
        "J1FLOW name={} writes={} segments={} points={} root_writes={} txgs_strictly_increase={strict} expand_up_to={} states={states} judged_root_index={}",
        flow.name, flow.writes.len(), flow.segments.len(), flow.crash_points.len(), roots.len(), flow.expand_up_to, flow.judged_root_index
    );
    println!(
        "J1ROOTS name={} {}",
        flow.name,
        roots.iter().map(|(index, instance, txg, device)| format!("w{index}:i{instance}t{txg}d{device}")).collect::<Vec<_>>().join(" ")
    );
}

fn compare(left: &RecordedFlow, right: &RecordedFlow) {
    let plan_left = plan_crash_points(&left.flow());
    let plan_right = plan_crash_points(&right.flow());
    let texts_left = judge_input_texts_of_each_point(&left.flow());
    let texts_right = judge_input_texts_of_each_point(&right.flow());
    let (mut shared, mut same_block_key, mut differing) = (0, 0, 0);
    for (index_left, point_left) in plan_left.points.iter().enumerate() {
        let Some(index_right) = plan_right.points.iter().position(|point| point.identity == point_left.identity) else { continue };
        let point_right = &plan_right.points[index_right];
        shared += 1;
        let states_left = point_left.ordinals.end - point_left.ordinals.start;
        let states_right = point_right.ordinals.end - point_right.ordinals.start;
        if point_left.plan_hash == point_right.plan_hash {
            same_block_key += 1;
            continue;
        }
        differing += 1;
        let lines_left: Vec<&str> = texts_left[index_left].2.lines().collect();
        let lines_right: Vec<&str> = texts_right[index_right].2.lines().collect();
        let only_left: Vec<&str> = lines_left.iter().filter(|line| !lines_right.contains(line)).copied().collect();
        let only_right: Vec<&str> = lines_right.iter().filter(|line| !lines_left.contains(line)).copied().collect();
        println!(
            "J1SHARED_PLAN_DIFFERS left={} right={} point={}:{} states_left={states_left} states_right={states_right} judge_inputs_equal={} only_left={only_left:?} only_right={only_right:?}",
            left.name, right.name, index_left, point_left.name, point_left.judge_inputs == point_right.judge_inputs
        );
    }
    println!(
        "J1PAIR left={} right={} shared_identities={shared} same_plan_hash={same_block_key} differing_plan_hash={differing}",
        left.name, right.name
    );
}

fn main() {
    let expand_up_to: usize = std::env::var("R2_EXPAND_UP_TO").ok().map_or(6, |text| text.parse().expect("数"));
    let two_a = two_flow("j1-two-a", &content_of(3, 4100), true, expand_up_to).renamed("two_flows_a");
    let two_b = two_flow("j1-two-b", &content_of(11, 4100), false, expand_up_to).renamed("two_flows_b");
    let rollback = rollback_flow("j1-rollback", expand_up_to).renamed("rollback");
    let unmount = unmount_flow("j1-unmount", expand_up_to).renamed("unmount");
    let mut twin = two_flow("j1-twin", &content_of(11, 4100), true, expand_up_to).renamed("two_flows_root_magic_twin");
    let root_index = twin.judged_root_index;
    match &mut twin.writes[root_index].contents {
        WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,
        WrittenContents::Zeros { .. } => panic!("根槽写带字节"),
    }
    for flow in [&two_a, &two_b, &twin, &rollback, &unmount] {
        describe(flow);
    }
    for flow in [&rollback, &unmount] {
        let plan = plan_crash_points(&flow.flow());
        let texts = judge_input_texts_of_each_point(&flow.flow());
        for (index, point) in plan.points.iter().enumerate() {
            let judged = texts[index].2.lines().find(|line| line.starts_with("judged_root_slot_write=")).unwrap_or("").to_string();
            let unfinished = texts[index].2.lines().find(|line| line.starts_with("unfinished_publish_root=")).map(|line| line.chars().take(48).collect::<String>()).unwrap_or_default();
            let reach = texts[index].2.lines().find(|line| line.starts_with("versions_up_to")).unwrap_or("").to_string();
            println!(
                "J1POINT flow={} index={index} name={} writes={}..{} states={} reuse_key={} plan_hash={} {judged} {reach} {unfinished}",
                flow.name, point.name, texts[index].0, texts[index].1, point.ordinals.end - point.ordinals.start,
                short(point.reuse_key.as_bytes()), short(&point.plan_hash)
            );
        }
    }
    compare(&two_a, &two_b);
    compare(&two_a, &rollback);
    compare(&two_a, &unmount);
    compare(&rollback, &unmount);
    compare(&two_b, &twin);
}
