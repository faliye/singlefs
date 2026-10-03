//! M3 的改法候选（攻方腿提的，只在我的模型上量过、被攻过零轮）：同一次覆盖写录两遍，第二遍只改坏一次写的一个字节，
//! 看「管这次写的那个点」上能不能同时满足：父镜像与判器输入、计划哈希相同（只剩复用键分得开），且这个点有状态判得不同。
//! 满足的那种改坏，才让「复用键不罩写表」那一行变异在行为上会红（读回的判定与现判的不同），而不只是红在「复用键不同」那条直接比键的断言上。
use r2proto::flows::{content_of, two_flow, RecordedFlow};
use singlefs_checker_tier::crash::judge_layer0_state_range;
use singlefs_checker_tier::crash_amplification::plan_crash_points;
use singlefs_harness::memory_pool::WrittenContents;

fn fresh(flow: &RecordedFlow, ordinals: std::ops::Range<u64>) -> Vec<Vec<String>> {
    let mut items = Vec::new();
    let _ = judge_layer0_state_range(&flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions, flow.expansion(), ordinals, &mut |_o, _i, j| items.push(j.red_items()));
    items
}

fn main() {
    let healthy = two_flow("m3c-healthy", &content_of(11, 4100), true, 6);
    let plan_healthy = plan_crash_points(&healthy.flow());
    let tail_start = plan_healthy.points.iter().position(|p| p.name.starts_with("publish_overwrite.")).expect("尾巴");
    let first_tail_write = healthy.crash_points[tail_start].writes.start;
    let mut tried = 0;
    let mut qualifying = 0;
    for write_index in first_tail_write..healthy.writes.len() {
        let WrittenContents::Bytes(bytes) = &healthy.writes[write_index].contents else { continue };
        let length = bytes.len();
        for offset in [0usize, 8, 40, length / 2, length - 1] {
            let mut twin = healthy.clone();
            if let WrittenContents::Bytes(bytes) = &mut twin.writes[write_index].contents { bytes[offset] ^= 0x5a; }
            let plan_twin = plan_crash_points(&twin.flow());
            let Some(owner) = plan_twin.points.iter().position(|p| p.segments.iter().any(|s| twin.segments[*s].contains(&write_index))) else { continue };
            let (h, t) = (&plan_healthy.points[owner], &plan_twin.points[owner]);
            tried += 1;
            let only_reuse_key_separates = h.plan_hash == t.plan_hash && h.identity != t.identity && h.ordinals == t.ordinals;
            let differing = if t.ordinals.is_empty() { 0 } else {
                let (a, b) = (fresh(&healthy, h.ordinals.clone()), fresh(&twin, t.ordinals.clone()));
                a.iter().zip(&b).filter(|(x, y)| x != y).count()
            };
            if only_reuse_key_separates && differing > 0 { qualifying += 1; }
            println!(
                "M3CAND write={write_index} kind={} offset={offset} owner={owner}:{} states={} only_reuse_key_separates={only_reuse_key_separates} states_judged_differently={differing}",
                healthy.writes[write_index].kind.name(), t.name, t.ordinals.end - t.ordinals.start
            );
        }
    }
    println!("M3CAND_TOTAL tried={tried} qualifying={qualifying}");
}
