//! m3-root-ring-code-r1 攻方原型 K2-f：挂载时读不出的槽那段循环（头尾两段各至多 3S 轮）与逐 txg 走的原写法逐输入比。
//! 原写法：对 (N − 3S, first_txg − 1] 里每一个 txg 走 target_for_publish，按槽记最小、最大。取样：S 取 4..=16，N 与 first_txg
//! 取小值全域（N ≤ 60、first_txg ≤ 90）加贴着 u64::MAX 的一批随机值，读不出的槽随机取一到三个、或全部。
use std::collections::BTreeMap;

use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::proto_unreadable_ring_slots_inside_the_window;
use singlefs_core::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};

fn naive(unreadable: &[RootRingSlot], anchor: u64, first: u64, s: RootRingSlotsPerRegion) -> Vec<(RootRingSlot, CheckpointTxg, CheckpointTxg)> {
    let ring = 3 * s.count();
    let lo = anchor.checked_sub(ring).map_or(0, |b| b + 1);
    let mut map: BTreeMap<RootRingSlot, (u64, u64)> = BTreeMap::new();
    let mut txg = lo;
    while txg < first {
        let slot = target_for_publish(CheckpointTxg(txg), s);
        if unreadable.contains(&slot) {
            map.entry(slot).and_modify(|(_, h)| *h = txg).or_insert((txg, txg));
        }
        txg += 1;
    }
    map.into_iter().map(|(k, (a, b))| (k, CheckpointTxg(a), CheckpointTxg(b))).collect()
}

fn main() {
    let mut state: u64 = 0x1234_5678;
    let mut next = || { state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); state >> 11 };
    let (mut cases, mut mismatches) = (0u64, 0u64);
    for s_count in 4..=16u64 {
        let s = RootRingSlotsPerRegion::from_system_configuration_field(s_count).expect("S");
        let every: Vec<RootRingSlot> = (0..3).flat_map(|r| (0..s_count).map(move |j| RootRingSlot { region: r, slot: j })).collect();
        let mut check = |unreadable: &[RootRingSlot], anchor: u64, first: u64| {
            let got = proto_unreadable_ring_slots_inside_the_window(unreadable, &every, CheckpointTxg(anchor), CheckpointTxg(first), s);
            let want = naive(unreadable, anchor, first, s);
            cases += 1;
            if got != want {
                mismatches += 1;
                if mismatches <= 5 { println!("MISMATCH S={s_count} N={anchor} first={first} unreadable={unreadable:?} got={got:?} want={want:?}"); }
            }
        };
        for anchor in 0..=60u64 {
            for first in 0..=90u64 {
                let k = 1 + (next() % 3) as usize;
                let unreadable: Vec<RootRingSlot> = (0..k).map(|_| every[(next() % every.len() as u64) as usize]).collect();
                check(&unreadable, anchor, first);
                check(&every, anchor, first);
            }
        }
        for _ in 0..2000 {
            let anchor = u64::MAX - (next() % 200);
            let first = anchor.saturating_add(1 + next() % 100).min(u64::MAX);
            let first = if next() % 4 == 0 { u64::MAX } else { first };
            let k = 1 + (next() % 3) as usize;
            let unreadable: Vec<RootRingSlot> = (0..k).map(|_| every[(next() % every.len() as u64) as usize]).collect();
            // 原写法逐 txg 走到 first：贴着 u64::MAX 时只走 (N − 3S, first) 那一段，轮数小。
            check(&unreadable, anchor, first);
        }
    }
    println!("WINDOW_LOOP_EQUIV cases={cases} mismatches={mismatches}");
}
