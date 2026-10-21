//! m3-close-c693-r3 辩方腿草稿（仓副本里编、跑，不进主仓 crates）：拿 `crates/` 的记账树规划器
//! （`plan_the_tree_after_this_publish`，记账树叶 477、内部 150）量四条记账臂每次发布重写几个记账树节点。
//! 在第二轮攻方的同类装置（`m3-close-c693-r2-opus-model/accounting_planner_carry_cost_with_long_step_histories.rs`）上加三样：
//!   ① 台阶行条数扫密（1、3、30、100、200、300、400、500、2000），找「Carry 比 NoCarry 多重写节点」从哪一档起；
//!   ② 第四条臂「按统计量」（PerStatistic）：有台阶行例外的统计量（inode 号水位）不带这一代那一行，其余统计量照 Carry 带；
//!   ③ `r_tree_dim` = 2：不认识的头多一个带树维的统计量（今天不存在，取样点是假设；用码 7 与码 13 各放一次，看它的 key 排在水位前还是后有没有影响），
//!      这个统计量只有「这一代的行」、没有台阶行（E186 第一节第 9 条的造法）。
//! 行形照 `accounting_entries_of_this_publish`：池级三行、每盘六行、照抄台阶行；认识的头每个统计量一行「这一代的行」随 txg 走；
//! 不认识的头每个 `unknown_steps` 级台阶（代段早于 T0）加各统计量一行「这一代的行」（代段 T0，lpc = 0）。旧写者 G 次发布不建 inode。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, TreeIdentifier};
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
use std::collections::BTreeSet;

const KNOWN_HEAD: u64 = 65537;
const T0: u64 = 100_000;

#[derive(Clone, Copy, Debug)]
enum Arm {
    Carry,
    NoCarry,
    KeepWithRoot,
    PerStatistic,
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

/// 不认识的头在这一版里「这一代的行」的代段：None = 不写这一行。`has_step_rows` = 这个统计量有台阶行例外。
fn this_generation_row_of_an_unknown_head(arm: Option<Arm>, txg: u64, has_step_rows: bool) -> Option<u64> {
    match arm {
        None => Some(T0),
        Some(Arm::Carry) => Some(txg),
        Some(Arm::NoCarry) => None,
        Some(Arm::KeepWithRoot) => Some(T0),
        Some(Arm::PerStatistic) => if has_step_rows { None } else { Some(txg) },
    }
}

#[allow(clippy::too_many_arguments)]
fn keys_of_version(
    txg: u64, devices: u32, known_steps: u64, unknown_heads: u64, unknown_steps: u64,
    second_statistic: Option<u16>, arm: Option<Arm>,
) -> BTreeSet<CodeTwoTreeKey> {
    let mut rows = vec![
        row(STATISTIC_PENDING_DELETE_BYTES, 0, STATISTIC_NO_DEVICE_DIMENSION, txg),
        row(STATISTIC_COMMITTED_RESERVATION_BYTES, 0, STATISTIC_NO_DEVICE_DIMENSION, txg),
        row(STATISTIC_INODE_WATERMARK, KNOWN_HEAD, STATISTIC_NO_DEVICE_DIMENSION, txg),
    ];
    if let Some(second) = second_statistic {
        rows.push(row(second, KNOWN_HEAD, STATISTIC_NO_DEVICE_DIMENSION, txg));
    }
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
        if let Some(generation) = this_generation_row_of_an_unknown_head(arm, txg, true) {
            rows.push(row(STATISTIC_INODE_WATERMARK, tree, STATISTIC_NO_DEVICE_DIMENSION, generation));
        }
        if let Some(second) = second_statistic {
            if let Some(generation) = this_generation_row_of_an_unknown_head(arm, txg, false) {
                rows.push(row(second, tree, STATISTIC_NO_DEVICE_DIMENSION, generation));
            }
        }
    }
    rows.iter()
        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
        .collect()
}

#[test]
fn rewritten_accounting_nodes_per_publish_for_four_arms_against_step_history_length_and_a_second_statistic() {
    let capacity = CodeTwoTreeNodeCapacity::of_the_node_format(22, 34);
    eprintln!("R3PLANNER capacity leaf={} internal={}", capacity.leaf_entries, capacity.internal_entries);
    assert_eq!((capacity.leaf_entries, capacity.internal_entries), (477, 150), "记账树叶 477、内部 150（transaction.rs 那一行）");
    const PUBLISHES: u64 = 10;
    // (r_tree_dim, 第二个统计量的码)：r_tree_dim = 1 不放第二个统计量；= 2 时码 13 排在水位（12）之后、码 7 排在水位之前
    let second_choices: [(u8, Option<u16>); 3] = [(1, None), (2, Some(13)), (2, Some(7))];
    for devices in [2u32, 64] {
        for known_steps in [1u64, 1000] {
            for unknown_heads in [2u64, 4] {
                for unknown_steps in [1u64, 3, 30, 100, 200, 300, 400, 500, 2000] {
                    for (r_tree_dim, second) in second_choices {
                        let initial = keys_of_version(T0, devices, known_steps, unknown_heads, unknown_steps, second, None);
                        let initial_shape = plan_the_tree_after_this_publish(&CodeTwoTreeShape::default(), &initial, capacity)
                            .expect("R 那一版规划得出")
                            .shape;
                        let mut line = format!(
                            "R3PLANNER devices={devices} known_steps={known_steps} unknown_heads={unknown_heads} unknown_steps={unknown_steps} r_tree_dim={r_tree_dim} second={} rows_in_r={} height_in_r={}",
                            second.map_or(0, u32::from),
                            initial.len(),
                            initial_shape.height()
                        );
                        for arm in [Arm::Carry, Arm::NoCarry, Arm::KeepWithRoot, Arm::PerStatistic] {
                            let mut shape = initial_shape.clone();
                            let mut rewritten_per_publish = Vec::new();
                            let mut rows_steady = 0usize;
                            for g in 1..=PUBLISHES {
                                let keys = keys_of_version(T0 + g, devices, known_steps, unknown_heads, unknown_steps, second, Some(arm));
                                rows_steady = keys.len();
                                let plan = plan_the_tree_after_this_publish(&shape, &keys, capacity).expect("规划得出");
                                rewritten_per_publish.push(plan.rewritten_positions().len());
                                shape = plan.shape;
                            }
                            let steady = rewritten_per_publish[1..].iter().max().copied().unwrap_or(0);
                            let first = rewritten_per_publish[0];
                            line.push_str(&format!(" {arm:?}:first={first},steady_max={steady},rows={rows_steady}"));
                        }
                        eprintln!("{line}");
                    }
                }
            }
        }
    }
}
