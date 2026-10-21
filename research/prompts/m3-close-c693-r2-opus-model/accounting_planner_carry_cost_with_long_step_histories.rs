//! m3-close-c693-r2 攻方腿草稿（仓副本，不进主仓）：拿 `crates/` 的记账树规划器（`plan_the_tree_after_this_publish`，记账树叶 477、内部 150）
//! 量 D8 已定项 8 ㊀ 三条记账臂每次发布重写几个记账树节点，扫「每个头的台阶行条数」这一维（D5 已定项 2 射程：台阶行在建 inode 与建克隆的发布写、
//! 随根一直留着）。E186 的取样点里台阶行至多 3 级（s ∈ {0,1,3}），第一轮攻方 W1-丁 的「每格每代最多 47 行、三条臂每次发布都写 1 个记账节点」
//! 只在那一段上量过。行形照 `accounting_entries_of_this_publish`：池级三行（挂起删除、已承诺预留不带树、inode 号水位带认识的头）、每盘六行、
//! 照抄台阶行；不认识的头每个 `steps` 级台阶（代段早于 T0）加一行「这一代的行」（代段 T0，`lpc = 0`）。旧写者 G 次发布不建 inode（认识的头不长台阶）。
//! Carry：每次发布把不认识的头那一行换成这一代的代号；NoCarry：不写；随根留着：原样留在 T0。
use singlefs_core::code_two_tree::{
    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity,
    CodeTwoTreeShape,
};
use singlefs_core::records::{
    AccountingEntry, STATISTIC_ALLOCATED_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES,
    STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS,
    STATISTIC_FREE_BYTES, STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION,
    STATISTIC_PENDING_DELETE_BYTES, STATISTIC_UNRECLAIMABLE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, TreeIdentifier};
use std::collections::BTreeSet;

const KNOWN_HEAD: u64 = 65537;
const T0: u64 = 100_000;

#[derive(Clone, Copy, Debug)]
enum Arm {
    Carry,
    NoCarry,
    KeepWithRoot,
}

fn row(statistic: u16, tree: u64, device: u32, generation: u64) -> AccountingEntry {
    AccountingEntry {
        statistic,
        tree: TreeIdentifier(tree),
        device: DeviceIdentity(device),
        generation: CheckpointTxg(generation),
        value: 1,
        sequence: 0,
    }
}

fn keys_of_version(txg: u64, devices: u32, known_steps: u64, unknown_heads: u64, unknown_steps: u64, arm: Option<Arm>) -> BTreeSet<CodeTwoTreeKey> {
    let mut rows = vec![
        row(STATISTIC_PENDING_DELETE_BYTES, 0, STATISTIC_NO_DEVICE_DIMENSION, txg),
        row(STATISTIC_COMMITTED_RESERVATION_BYTES, 0, STATISTIC_NO_DEVICE_DIMENSION, txg),
        row(STATISTIC_INODE_WATERMARK, KNOWN_HEAD, STATISTIC_NO_DEVICE_DIMENSION, txg),
    ];
    for device in 0..devices {
        for statistic in [
            STATISTIC_ALLOCATED_BYTES, STATISTIC_FREE_BYTES, STATISTIC_UNRECLAIMABLE_BYTES,
            STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_EMPTY_CLUSTER_SEGMENTS,
        ] {
            rows.push(row(statistic, 0, device, txg));
        }
    }
    for step in 0..known_steps {
        rows.push(row(STATISTIC_INODE_WATERMARK, KNOWN_HEAD, STATISTIC_NO_DEVICE_DIMENSION, T0 - 1 - step));
    }
    for head in 0..unknown_heads {
        let tree = KNOWN_HEAD + 1000 + head;
        for step in 0..unknown_steps {
            rows.push(row(STATISTIC_INODE_WATERMARK, tree, STATISTIC_NO_DEVICE_DIMENSION, T0 - 1 - step));
        }
        let this_generation_row = match arm {
            None => Some(T0),
            Some(Arm::Carry) => Some(txg),
            Some(Arm::NoCarry) => None,
            Some(Arm::KeepWithRoot) => Some(T0),
        };
        if let Some(generation) = this_generation_row {
            rows.push(row(STATISTIC_INODE_WATERMARK, tree, STATISTIC_NO_DEVICE_DIMENSION, generation));
        }
    }
    rows.iter()
        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
        .collect()
}

#[test]
fn carry_cost_in_rewritten_accounting_nodes_against_the_length_of_the_step_histories() {
    let capacity = CodeTwoTreeNodeCapacity::of_the_node_format(22, 34);
    eprintln!("R2PLANNER capacity leaf={} internal={}", capacity.leaf_entries, capacity.internal_entries);
    assert_eq!((capacity.leaf_entries, capacity.internal_entries), (477, 150), "记账树叶 477、内部 150（transaction.rs 那一行）");
    const PUBLISHES: u64 = 10;
    for devices in [2u32, 64] {
        for known_steps in [1u64, 1000] {
            for unknown_heads in [2u64, 4] {
                for unknown_steps in [1u64, 3, 500, 2000] {
                    let initial = keys_of_version(T0, devices, known_steps, unknown_heads, unknown_steps, None);
                    let initial_shape = plan_the_tree_after_this_publish(&CodeTwoTreeShape::default(), &initial, capacity)
                        .expect("R 那一版规划得出")
                        .shape;
                    let mut line = format!(
                        "R2PLANNER devices={devices} known_steps={known_steps} unknown_heads={unknown_heads} unknown_steps={unknown_steps} rows_in_r={} height_in_r={}",
                        initial.len(),
                        initial_shape.height()
                    );
                    for arm in [Arm::Carry, Arm::NoCarry, Arm::KeepWithRoot] {
                        let mut shape = initial_shape.clone();
                        let mut rewritten_per_publish = Vec::new();
                        for g in 1..=PUBLISHES {
                            let keys = keys_of_version(T0 + g, devices, known_steps, unknown_heads, unknown_steps, Some(arm));
                            let plan = plan_the_tree_after_this_publish(&shape, &keys, capacity).expect("规划得出");
                            rewritten_per_publish.push(plan.rewritten_positions().len());
                            shape = plan.shape;
                        }
                        let steady = rewritten_per_publish[1..].iter().max().copied().unwrap_or(0);
                        let first = rewritten_per_publish[0];
                        line.push_str(&format!(" {arm:?}:first={first},steady_max={steady}"));
                    }
                    eprintln!("{line}");
                }
            }
        }
    }
}
