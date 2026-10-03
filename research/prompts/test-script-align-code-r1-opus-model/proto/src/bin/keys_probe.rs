//! K3 第 3 行的解释：对照里 A、B 两条流（内容 3 与 11）逐点的名字、复用键、计划哈希、块数，看尾巴上哪几点的块键相同。
use k2proto::flows::{content_of, record_flow, with_the_overwritten_version_miswritten};
use singlefs_checker_tier::crash_amplification::plan_crash_points;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().take(6).map(|byte| format!("{byte:02x}")).collect()
}

fn main() {
    let flow_a = with_the_overwritten_version_miswritten(record_flow("k2-keys-a", &content_of(3, 4100), true));
    let flow_b = record_flow("k2-keys-b", &content_of(11, 4100), true);
    let plan_a = plan_crash_points(&flow_a.flow());
    let plan_b = plan_crash_points(&flow_b.flow());
    for (index, (a, b)) in plan_a.points.iter().zip(&plan_b.points).enumerate() {
        println!(
            "K3KEYS point={index} name={} same_identity={} same_reuse_key={} same_plan_hash={} states_a={} states_b={} key_a={} key_b={}",
            a.name, a.identity == b.identity, a.reuse_key == b.reuse_key, a.plan_hash == b.plan_hash,
            a.ordinals.end - a.ordinals.start, b.ordinals.end - b.ordinals.start,
            hex(a.reuse_key.as_bytes()), hex(b.reuse_key.as_bytes())
        );
    }
}
