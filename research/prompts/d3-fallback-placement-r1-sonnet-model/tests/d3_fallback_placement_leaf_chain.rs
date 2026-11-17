//! d3-fallback-placement-r1 正推腿模型（补丁树上跑）：提交内生块回落时的落点政策对「取落点碰到新叶」那一半固定点轮数的影响。
//! 几何是抽象的（叶 = 槽号 ÷ 812、父 = 叶序号 ÷ 169，每片重写的叶每块盘各一个角色），落点政策是真的：每个角色的落点都经
//! `PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set` 取，空闲图用真的 `DeviceFreeMap`（两块盘同形）。
//! 固定点循环照 `transaction::settle_the_allocation_record_tree` 的收敛判法写（重写集 ∩ 这一版的节点 ∪ 新节点；
//! 取完落点后「装着记录的节点等于这一版的节点、变了的都在重写集里」才定下），不走真发布，所以轮数是模型的轮数，
//! 对拍真实现看 `d3_fallback_placement_forged_sweep.rs` 在未打补丁的树上的读数。
//!
//! 三条臂：
//! - `lowest_free_slot`：今天的政策，回落取该设备内槽号最小的空槽（重写集传空集）。
//! - `rewrite_set_first`：补丁的政策，先取重写集的叶里槽号最小的空槽，那些叶里没有再取该设备内槽号最小的空槽。
//! - `rewrite_set_then_densest_leaf`：上一条，另加「重写集的叶里没有空槽时，取重写集外空槽最多的那片叶（并列取序号小的）里槽号最小的空槽」。
//!   这一条的选叶在测试里做（经「重写集 ∪ 选中的叶」传给同一个分配器入口），产品代码里没有对应实现，要维护每片叶的空槽数索引。
//!
//! 形态（`Layout`）：低处一串「链叶」每片 `chain_free` 个空槽（偶数起点连着，在叶内偏移 400 起）、链叶首片开头 `prefix` 个空槽、
//! 链叶之后每个 64 槽段留段首之外的槽（一个全空段都没有）；另有几片「现行叶」（上一版树的叶，重写集起点）每片 `slack_free` 个空槽（叶内偏移 100 起）。
//! 单元区里别的槽全占着。

use std::collections::BTreeSet;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, RewriteSetLeaves, UnitFootprint};
use singlefs_format::{
    ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS, SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};

const UNIT_AREA_SLOTS: u64 = 1 << 18;
const ROUNDS_LIMIT: usize = 64;
/// 一次覆盖写里不是分配记录树节点的落点占的槽数：extent 树根 1、inode 树叶容器 2、inode 树根 1、记账树根 1、中央映射树根 1、树表单元 1、
/// 数据单元 2，共 9 槽，模型里当 9 个一槽的角色取（容器与数据单元的对齐不建模）。
/// 手算对拍：重写集一片叶、第 1 层一个节点时取落点的槽数 9 + 2 + 2 + 1 = 14，再多一片新叶（两个角色）就是 16——
/// 攻方模型在未打补丁的树上扫稠密前缀，前缀 14、15 撞上界、前缀 16 起 3 轮定下，门槛正是 16。
const SLOTS_OF_THE_PLACEMENTS_OUTSIDE_THE_ALLOCATION_TREE: usize = 9;
const OFFSET_OF_THE_CHAIN_FREE_SLOTS: u64 = 400;
const OFFSET_OF_THE_SLACK_FREE_SLOTS: u64 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    LowestFreeSlot,
    RewriteSetFirst,
    RewriteSetThenDensestLeaf,
}

impl Arm {
    const ALL: [Arm; 3] = [
        Arm::LowestFreeSlot,
        Arm::RewriteSetFirst,
        Arm::RewriteSetThenDensestLeaf,
    ];

    fn name(self) -> &'static str {
        match self {
            Arm::LowestFreeSlot => "lowest_free_slot",
            Arm::RewriteSetFirst => "rewrite_set_first",
            Arm::RewriteSetThenDensestLeaf => "rewrite_set_then_densest_leaf",
        }
    }
}

#[derive(Clone, Debug)]
struct Layout {
    name: &'static str,
    current_leaves: Vec<u64>,
    slack_free: u64,
    chain_first_leaf: u64,
    chain_leaves: u64,
    chain_free: u64,
    prefix: u64,
}

impl Layout {
    fn is_free(&self, slot: u64) -> bool {
        let leaf = slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        let offset = slot % ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        if self.current_leaves.contains(&leaf) {
            (OFFSET_OF_THE_SLACK_FREE_SLOTS..OFFSET_OF_THE_SLACK_FREE_SLOTS + self.slack_free)
                .contains(&offset)
        } else if leaf < self.chain_first_leaf {
            false
        } else if leaf == self.chain_first_leaf {
            offset < self.prefix
        } else if leaf <= self.chain_first_leaf + self.chain_leaves {
            (OFFSET_OF_THE_CHAIN_FREE_SLOTS..OFFSET_OF_THE_CHAIN_FREE_SLOTS + self.chain_free)
                .contains(&offset)
        } else {
            (slot - UNIT_AREA_START_SLOT) % 64 != 0
        }
    }

    /// 单元区里每片叶的空槽数（叶序号 → 个数）。
    fn free_slots_of_each_leaf(&self, unit_area_slots: u64) -> Vec<u64> {
        let last_leaf =
            (UNIT_AREA_START_SLOT + unit_area_slots) / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        let mut free_of_each_leaf = vec![0u64; usize::try_from(last_leaf + 1).expect("片数")];
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + unit_area_slots {
            if self.is_free(slot) {
                free_of_each_leaf
                    [usize::try_from(slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS).expect("叶")] += 1;
            }
        }
        free_of_each_leaf
    }

    fn allocator(&self) -> PoolAllocator {
        let device_bytes = (UNIT_AREA_START_SLOT + UNIT_AREA_SLOTS) * SLOT_BYTES;
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        for device_map in &mut allocator.devices {
            for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
                if !self.is_free(slot) {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                }
            }
        }
        allocator
    }
}

struct Outcome {
    outcome: &'static str,
    /// 模型里取过落点的轮数；真实现的第一轮还要先把释放弄脏的叶并进重写集（不取落点），对拍时加一。
    rounds_taking_placements: usize,
    roles: usize,
    rewritten_leaves: usize,
    leaves_outside_the_current_ones: usize,
}

fn the_densest_leaf_outside(free_of_each_leaf: &[u64], rewritten: &BTreeSet<u64>) -> Option<u64> {
    free_of_each_leaf
        .iter()
        .enumerate()
        .filter(|(leaf, free)| {
            **free > 0 && !rewritten.contains(&u64::try_from(*leaf).expect("叶"))
        })
        .max_by_key(|(leaf, free)| (**free, std::cmp::Reverse(*leaf)))
        .map(|(leaf, _)| u64::try_from(leaf).expect("叶"))
}

/// 一次发布的固定点：每一轮按这一版的节点与重写集算角色数，在分配器的拷贝上按选定的臂取够这么多一槽的落点，
/// 取到的落点所在的叶都已在这一版的节点里、也都在重写集里才定下，否则并进去再走一轮。
fn settle(layout: &Layout, base: &PoolAllocator, arm: Arm) -> Outcome {
    let previous_leaves: BTreeSet<u64> = layout.current_leaves.iter().copied().collect();
    let free_at_the_start = layout.free_slots_of_each_leaf(base.devices[0].unit_area_slots());
    let mut rewritten = previous_leaves.clone();
    let mut nodes_after = previous_leaves.clone();
    for round in 0..ROUNDS_LIMIT {
        let rewritten_now: BTreeSet<u64> = rewritten
            .intersection(&nodes_after)
            .copied()
            .chain(nodes_after.difference(&previous_leaves).copied())
            .collect();
        let parents: BTreeSet<u64> = rewritten_now
            .iter()
            .map(|leaf| leaf / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
            .collect();
        // 每片重写的叶每块盘一个角色，每个重写的第 1 层节点每块盘一个，根一个。
        let roles = SLOTS_OF_THE_PLACEMENTS_OUTSIDE_THE_ALLOCATION_TREE
            + 2 * rewritten_now.len()
            + 2 * parents.len()
            + 1;
        let mut allocator = base.clone();
        let mut free_of_each_leaf = free_at_the_start.clone();
        let mut touched: BTreeSet<u64> = BTreeSet::new();
        for _role in 0..roles {
            let preferred = match arm {
                Arm::LowestFreeSlot => RewriteSetLeaves::none(),
                Arm::RewriteSetFirst | Arm::RewriteSetThenDensestLeaf => {
                    let mut leaves = rewritten_now.clone();
                    let free_in_the_rewrite_set: u64 = leaves
                        .iter()
                        .map(|leaf| free_of_each_leaf[usize::try_from(*leaf).expect("叶")])
                        .sum();
                    if arm == Arm::RewriteSetThenDensestLeaf && free_in_the_rewrite_set == 0 {
                        if let Some(densest) =
                            the_densest_leaf_outside(&free_of_each_leaf, &rewritten_now)
                        {
                            leaves.insert(densest);
                        }
                    }
                    RewriteSetLeaves::present_on_every_device(&[leaves.clone(), leaves])
                }
            };
            let Ok(placement) = allocator.try_allocate_commit_generated_preferring_the_rewrite_set(
                UnitFootprint::OneSlot,
                CheckpointTxg(5),
                &preferred,
            ) else {
                return Outcome {
                    outcome: "refused_no_free_slot",
                    rounds_taking_placements: round + 1,
                    roles,
                    rewritten_leaves: rewritten_now.len(),
                    leaves_outside_the_current_ones: 0,
                };
            };
            let leaf = placement.slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
            free_of_each_leaf[usize::try_from(leaf).expect("叶")] -= 1;
            touched.insert(leaf);
        }
        let holding: BTreeSet<u64> = previous_leaves.union(&touched).copied().collect();
        if holding == nodes_after && touched.is_subset(&rewritten_now) {
            return Outcome {
                outcome: "settled",
                rounds_taking_placements: round + 1,
                roles,
                rewritten_leaves: rewritten_now.len(),
                leaves_outside_the_current_ones: rewritten_now.difference(&previous_leaves).count(),
            };
        }
        rewritten.extend(touched.iter().copied());
        nodes_after = holding;
    }
    Outcome {
        outcome: "did_not_settle_limit_64",
        rounds_taking_placements: ROUNDS_LIMIT,
        roles: 0,
        rewritten_leaves: rewritten.len(),
        leaves_outside_the_current_ones: 0,
    }
}

fn report(layout: &Layout, base: &PoolAllocator) {
    for arm in Arm::ALL {
        let outcome = settle(layout, base, arm);
        println!(
            "d3-fallback-placement: leafchain shape={} arm={} current_leaves={:?} slack_free={} chain_free={} chain_leaves={} prefix={} outcome={} rounds_taking_placements={} roles={} rewritten_leaves={} new_leaves={}",
            layout.name,
            arm.name(),
            layout.current_leaves,
            layout.slack_free,
            layout.chain_free,
            layout.chain_leaves,
            layout.prefix,
            outcome.outcome,
            outcome.rounds_taking_placements,
            outcome.roles,
            outcome.rewritten_leaves,
            outcome.leaves_outside_the_current_ones
        );
    }
}

/// 第一段：对拍。上一版树的叶只有一片、里面没有空槽（slack 0），链叶每片两个空槽，扫链叶首片开头的稠密前缀：
/// 未打补丁的真实现在同一类形态（`d3_fallback_placement_forged_sweep.rs`）上的轮数随前缀怎么走，今天的政策这一臂就该怎么走（轮数差一的口径见 `Outcome`）。
/// 第二段：上一版树的叶在链叶的上面（槽号更高），里面有 slack 个空槽——今天的政策先取更低的链叶，补丁的政策先取现行叶里的。
#[test]
fn leaf_chain_sweep_over_the_three_arms() {
    let mut probes = 0u64;
    for prefix in 0..=20u64 {
        let layout = Layout {
            name: "forged_like_the_attack_model",
            current_leaves: vec![62],
            slack_free: 0,
            chain_first_leaf: 63,
            chain_leaves: 120,
            chain_free: 2,
            prefix,
        };
        let base = layout.allocator();
        let today = settle(&layout, &base, Arm::LowestFreeSlot);
        let preferring = settle(&layout, &base, Arm::RewriteSetFirst);
        println!(
            "d3-fallback-placement: leafchain claim_rewrite_set_first_equals_lowest_free_slot_without_slack prefix={prefix} same={}",
            today.outcome == preferring.outcome
                && today.rounds_taking_placements == preferring.rounds_taking_placements
                && today.roles == preferring.roles
        );
        report(&layout, &base);
        probes += 1;
    }
    for current_leaf_count in [1u64, 3] {
        for chain_free in [2u64, 3] {
            for slack_free in [0u64, 1, 2, 4, 8, 16, 32] {
                let layout = Layout {
                    name: "current_leaves_above_the_chain",
                    current_leaves: (200..200 + current_leaf_count).collect(),
                    slack_free,
                    chain_first_leaf: 62,
                    chain_leaves: 120,
                    chain_free,
                    prefix: 0,
                };
                let base = layout.allocator();
                report(&layout, &base);
                probes += 1;
            }
        }
    }
    println!("d3-fallback-placement: leafchain_summary layouts={probes}");
    assert!(probes > 0);
}
