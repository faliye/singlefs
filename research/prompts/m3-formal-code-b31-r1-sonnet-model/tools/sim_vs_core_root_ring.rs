//! 腿自己的核对（只放进腿的草稿副本编，不进仓）：overwrite_in_one_instance.rs 里「用例自己模拟根环」那个函数
//! （oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to）与 core 分配器记根时按 `RootRingOccupancy` 实际持有的根环，
//! 是不是每个 txg 上给同一个「环里最旧有效根」。core 一侧不改 core：读 `RootRingOccupancy` 的 `{:?}` 文本里 ValidRoot 的 txg 取最小。
mod common;

use std::collections::BTreeMap;

use common::{build_pool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_format::ROOT_RING_REGIONS;

fn simulated(newest: CheckpointTxg, genesis: CheckpointTxg) -> CheckpointTxg {
    let slots_per_region = common::parameters().geometry.root_ring_slots_per_region;
    let mut table: BTreeMap<RootRingSlot, CheckpointTxg> = (0..ROOT_RING_REGIONS)
        .map(|region| (RootRingSlot { region, slot: 0 }, genesis))
        .collect();
    for txg in genesis.0 + 1..=newest.0 {
        table.insert(
            target_for_publish(CheckpointTxg(txg), slots_per_region),
            CheckpointTxg(txg),
        );
    }
    *table.values().min().unwrap()
}

fn oldest_valid_in_debug_text(text: &str) -> u64 {
    let mut oldest = u64::MAX;
    let mut rest = text;
    while let Some(at) = rest.find("ValidRoot {") {
        rest = &rest[at..];
        let key = "CheckpointTxg(";
        let start = rest.find(key).unwrap() + key.len();
        let end = start + rest[start..].find(')').unwrap();
        oldest = oldest.min(rest[start..end].parse::<u64>().unwrap());
        rest = &rest[end..];
    }
    oldest
}

#[test]
fn the_overwrite_test_simulation_of_the_root_ring_equals_the_ring_the_allocator_holds_at_every_txg() {
    let mut pool = build_pool("zz-sim-vs-core-root-ring");
    let genesis = pool.genesis.root.checkpoint_txg;
    let mut compared = 0;
    let mut index = 0usize;
    while pool.output.root.checkpoint_txg.0 < 90 {
        let previous = pool.output.clone();
        let content: Vec<u8> = (0..3000 + index * 37)
            .map(|i| u8::try_from((i * 11 + index * 5 + 1) % 251).unwrap())
            .collect();
        pool.output = common::publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content,
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .unwrap();
        index += 1;
        let txg = pool.output.root.checkpoint_txg;
        let text = format!("{:?}", pool.allocator.root_ring_occupancy().unwrap());
        assert_eq!(
            oldest_valid_in_debug_text(&text),
            simulated(txg, genesis).0,
            "txg {txg:?}: core ring {text}"
        );
        compared += 1;
    }
    println!("compared_txgs={compared}");
}
