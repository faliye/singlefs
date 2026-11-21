//! 树高、一条路径的字节、字节占比读数、树高的增长阈值与环的在途上限。

use super::*;

// ============================================================
// 八、第 5 行：甲每次持久化写字节随对象数怎么涨（第四段，用户直接问的那一题）
// ============================================================

/// 六棵会长高的 / 恒定的树各自的树高（Q5.2 的 `T` 取值：extent、inode、分配记录、记账、映射、树表）。
pub(super) fn tree_heights(outcome: &JiaPublishOutcome) -> [usize; 6] {
    [
        outcome.extent_layers.len(),
        outcome.inode_layers.len(),
        outcome.allocation_layers.len(),
        1, // 记账恒 1 层（K5：15 行远小于 477 的叶容量）。
        outcome.mapping_layers.len(),
        1, // 树表主几何恒 1 层（T = 7 ≤ 81）。
    ]
}

/// 一棵树高 `height` 的树，从叶到根走一条路径要写多少字节：inode 叶是容器（32768），其余每层 16384。
fn one_path_bytes(height: usize, leaf_is_inode_container: bool) -> u64 {
    assert!(height >= 1, "树高至少 1（根即叶）");
    if leaf_is_inode_container {
        UNIT_BYTES + NODE_BYTES * (height as u64 - 1)
    } else {
        NODE_BYTES * height as u64
    }
}

/// Q5.1：甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和，不含数据单元 / 记录 / 根槽 / 系统配置）。
pub(super) fn total_bytes_and_tree_bytes(shape: PoolShape) -> (u64, u64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = outcome.extent_bytes
        + outcome.inode_container_bytes
        + outcome.inode_root_bytes
        + outcome.allocation_bytes
        + outcome.accounting_bytes
        + outcome.mapping_bytes
        + outcome.tree_table_bytes;
    (outcome.total_bytes(), tree_bytes)
}

/// Q5.2a/b/c：三种「树高 × 棵数」界的读法，见登记 6.Q5.2a/b/c。
pub(super) fn rho_readings(shape: PoolShape) -> (f64, f64, f64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = (outcome.extent_bytes
        + outcome.inode_container_bytes
        + outcome.inode_root_bytes
        + outcome.allocation_bytes
        + outcome.accounting_bytes
        + outcome.mapping_bytes
        + outcome.tree_table_bytes) as f64;
    let heights = tree_heights(&outcome);
    let one_path_sum: u64 = one_path_bytes(heights[0], false)
        + one_path_bytes(heights[1], true)
        + one_path_bytes(heights[2], false)
        + one_path_bytes(heights[3], false)
        + one_path_bytes(heights[4], false)
        + one_path_bytes(heights[5], false);
    let maximum_tree_height = *heights.iter().max().expect("六个高度都有值") as u64;
    let path_ratio_by_actual_height = tree_bytes / (DEVICE_COUNT as f64 * one_path_sum as f64);
    let path_ratio_by_maximum_height_node_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * NODE_BYTES as f64 * DEVICE_COUNT as f64);
    let path_ratio_by_maximum_height_unit_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * UNIT_BYTES as f64 * DEVICE_COUNT as f64);
    (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width)
}

/// 在 `[low, high]` 上二分，找「`height_at` 第一次超过 `base_height`」的最小 P；`height_at` 必须单调不减。
/// 找不到（`height_at(high)` 仍 `<= base_height`）时返回 `None`。
fn bisect_first_threshold(low: u64, high: u64, base_height: usize, height_at: impl Fn(u64) -> usize) -> Option<u64> {
    if height_at(high) <= base_height {
        return None;
    }
    let mut low = low;
    let mut high = high;
    while low < high {
        let mid = low + (high - low) / 2;
        if height_at(mid) > base_height {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    Some(low)
}

/// Q5.3 要的「每棵树每次长高一层的门槛」：从 P = 1 的高度开始，逐次二分找下一个跨越点，直到搜索上限。
pub(super) fn height_growth_thresholds(search_upper_bound: u64, height_at: impl Fn(u64) -> usize) -> Vec<u64> {
    let mut thresholds = Vec::new();
    let mut base_height = height_at(1);
    let mut floor = 1u64;
    while let Some(threshold) = bisect_first_threshold(floor, search_upper_bound, base_height, &height_at) {
        thresholds.push(threshold);
        base_height = height_at(threshold);
        floor = threshold;
        if thresholds.len() > 16 {
            break; // 防御性上限：正常情况下 P ≤ search_upper_bound 不会长这么多层。
        }
    }
    thresholds
}

/// 默认环的在飞记录上限（B9：768 MiB ÷ 4096 ÷ 3）。
pub(super) fn default_ring_in_flight_limit() -> u64 {
    RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN
}

#[cfg(test)]
#[path = "tree_heights_and_growth_thresholds_tests.rs"]
mod tests;
