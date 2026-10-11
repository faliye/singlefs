//! E30：范围重建 vs 逐项修改的交叉点 —— [experiments.md](experiments.md) E4 里能建模的那一半。
//!
//! **E4 原文**：现在写在文档里的「机械盘约 1%、SSD 约 10%」是**推理，不是实测**，必须量出来。
//! E4 完整版要 QEMU 量**耗时**；本实验只做**写放大**那一半，纯计数、不碰设备。
//! ⚠️ **耗时那一半仍然欠着**，本实验不假装覆盖了它。
//!
//! ## 两条实现
//!
//! | | 机制 | 代价 |
//! |---|---|---|
//! | 逐项修改 | 改 K 个条目，每个走一次 COW 到根 | K × (1 + 树高)，条目分散时不共享脊柱 |
//! | 范围重建 | 整段读出来、改完整段写回 | 覆盖该范围的全部节点，与 K 无关 |
//!
//! **交叉点**：K 大到「逐项的脊柱总数」超过「整段的节点数」时，范围重建更省。
//! 连续与散布是两条完全不同的曲线——连续时逐项共享脊柱，散布时不共享。

use e7_index_bench::Emitter;

const POINTER_BYTES: u64 = 40;

fn fanout(node_bytes: u64) -> u64 { ((node_bytes.saturating_sub(64)) / POINTER_BYTES).max(2) }

fn height(node_bytes: u64, leaf_count: u64) -> u32 {
    let node_fanout = fanout(node_bytes); let mut tree_height = 0u32; let mut reachable_leaf_count = 1u64;
    while reachable_leaf_count < leaf_count { reachable_leaf_count = reachable_leaf_count.saturating_mul(node_fanout); tree_height += 1; } tree_height
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layout { Contiguous, Scattered }

/// 逐项修改写出的节点数。
///
/// 连续：K 个条目落在 ceil(K/扇出) 个叶子上，脊柱在上层高度共享；
/// 散布：K 个条目落在 min(K, 叶数) 个不同叶子上，脊柱只在根共享。
fn per_item_writes(node_bytes: u64, leaf_count: u64, changed_item_count: u64, layout: Layout) -> u64 {
    let node_fanout = fanout(node_bytes);
    let tree_height = height(node_bytes, leaf_count) as u64;
    let touched_leaf_count = match layout {
        Layout::Contiguous => changed_item_count.div_ceil(node_fanout).max(1),
        Layout::Scattered  => changed_item_count.min(leaf_count),
    };
    // 每一层被触到的互异节点数：上一层节点数 / 扇出，向上取整，至少 1
    let mut written_node_count = touched_leaf_count;
    let mut touched_nodes_at_level = touched_leaf_count;
    for _ in 0..tree_height {
        touched_nodes_at_level = touched_nodes_at_level.div_ceil(node_fanout).max(1);
        written_node_count += touched_nodes_at_level;
    }
    written_node_count
}

/// 范围重建写出的节点数：覆盖该范围的全部叶子 + 其上的脊柱，**与 K 无关**。
fn range_rebuild_writes(node_bytes: u64, leaf_count: u64, range_leaf_count: u64) -> u64 {
    let node_fanout = fanout(node_bytes);
    let tree_height = height(node_bytes, leaf_count) as u64;
    let mut written_node_count = range_leaf_count;
    let mut rebuilt_nodes_at_level = range_leaf_count;
    for _ in 0..tree_height { rebuilt_nodes_at_level = rebuilt_nodes_at_level.div_ceil(node_fanout).max(1); written_node_count += rebuilt_nodes_at_level; }
    written_node_count
}

/// 总条目数 = 叶数 × 扇出。
/// ⚠️ **K 数的是条目不是叶子**——第一版把两者混了，
/// 于是连续布局在 100% 时也只触到 1/扇出 的叶子，交叉点永远不出现（测试当场红）。
fn total_entries(node_bytes: u64, leaf_count: u64) -> u64 { leaf_count.saturating_mul(fanout(node_bytes)) }

/// 交叉点：受影响比例 K/N 达到多少时范围重建开始更省。返回千分比。
fn crossover_permille(node_bytes: u64, leaf_count: u64, layout: Layout) -> Option<u64> {
    let entry_count = total_entries(node_bytes, leaf_count);
    for permille in 1..=1000u64 {
        let changed_item_count = (entry_count * permille / 1000).max(1);
        if per_item_writes(node_bytes, leaf_count, changed_item_count, layout) >= range_rebuild_writes(node_bytes, leaf_count, leaf_count) {
            return Some(permille);
        }
    }
    None
}

fn main() {
    let mut emitter = Emitter::new();
    let leaf_count = 1u64 << 20;
    println!("{}", emitter.emit_raw(&format!("name=config leaves={leaf_count} ptr_bytes={POINTER_BYTES}")));
    for node_bytes in [4096u64, 16384] {
        for layout in [Layout::Contiguous, Layout::Scattered] {
            for permille in [1u64, 10, 50, 100, 300, 1000] {
                let changed_item_count = (total_entries(node_bytes, leaf_count) * permille / 1000).max(1);
                println!("{}", emitter.emit_raw(&format!(
                    "name=cell node_bytes={node_bytes} layout={layout:?} permille={permille} k={changed_item_count} \
                     per_item={} range={}",
                    per_item_writes(node_bytes, leaf_count, changed_item_count, layout),
                    range_rebuild_writes(node_bytes, leaf_count, leaf_count))));
            }
            println!("{}", emitter.emit_raw(&format!(
                "name=crossover node_bytes={node_bytes} layout={layout:?} permille={}",
                crossover_permille(node_bytes, leaf_count, layout).map(|crossover_at_permille| crossover_at_permille.to_string())
                    .unwrap_or_else(|| "none".into()))));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const LEAF_COUNT: u64 = 1 << 20;

    /// **范围重建的代价与 K 无关**——这是它区别于逐项的唯一理由。
    #[test]
    fn range_rebuild_cost_is_independent_of_changed_item_count() {
        let first_rebuild_writes = range_rebuild_writes(4096, LEAF_COUNT, LEAF_COUNT);
        let second_rebuild_writes = range_rebuild_writes(4096, LEAF_COUNT, LEAF_COUNT);
        assert_eq!(first_rebuild_writes, second_rebuild_writes);
        // 且它恰等于「全部叶子 + 各层脊柱」，绝对值可算
        let node_fanout = fanout(4096);
        let mut expected_written_node_count = LEAF_COUNT; let mut expected_nodes_at_level = LEAF_COUNT;
        for _ in 0..height(4096, LEAF_COUNT) { expected_nodes_at_level = expected_nodes_at_level.div_ceil(node_fanout).max(1); expected_written_node_count += expected_nodes_at_level; }
        assert_eq!(first_rebuild_writes, expected_written_node_count);
    }

    /// **K 很小时逐项必须明显更省**，否则交叉点无从谈起。
    #[test]
    fn per_item_is_far_cheaper_when_only_a_few_items_change() {
        for layout in [Layout::Contiguous, Layout::Scattered] {
            let single_item_writes = per_item_writes(4096, LEAF_COUNT, 1, layout);
            let full_range_rebuild_writes = range_rebuild_writes(4096, LEAF_COUNT, LEAF_COUNT);
            assert!(single_item_writes * 100 < full_range_rebuild_writes, "改一项该比整段重建便宜两个数量级（{layout:?}）");
        }
    }

    /// **散布比连续贵**——连续时多个条目共享叶子与脊柱，散布时不共享。
    /// 少了这条，`layout` 这一维就是摆设。
    #[test]
    fn scattered_costs_more_than_contiguous_at_the_same_changed_item_count() {
        for changed_item_count in [100u64, 1000, 10_000] {
            let contiguous_writes = per_item_writes(4096, LEAF_COUNT, changed_item_count, Layout::Contiguous);
            let scattered_writes = per_item_writes(4096, LEAF_COUNT, changed_item_count, Layout::Scattered);
            assert!(scattered_writes > contiguous_writes, "k={changed_item_count} 时散布该比连续贵（{scattered_writes} vs {contiguous_writes}）");
        }
    }

    /// **两种布局的交叉点必须不同**，且散布的交叉点更早（散布更贵 ⇒ 更早值得整段重建）。
    #[test]
    fn the_two_layouts_cross_over_at_different_ratios() {
        let contiguous_crossover_permille = crossover_permille(4096, LEAF_COUNT, Layout::Contiguous).expect("连续该有交叉点");
        let scattered_crossover_permille = crossover_permille(4096, LEAF_COUNT, Layout::Scattered).expect("散布该有交叉点");
        assert!(scattered_crossover_permille < contiguous_crossover_permille, "散布更贵，交叉点该更早（散布 {scattered_crossover_permille}‰ vs 连续 {contiguous_crossover_permille}‰）");
    }

    /// **总条目数是叶数 × 扇出，不是叶数。** 这条钉住第一版混掉的那个口径。
    #[test]
    fn total_entries_counts_entries_not_leaves() {
        assert_eq!(total_entries(4096, LEAF_COUNT), LEAF_COUNT * 100);
        assert_eq!(total_entries(16384, LEAF_COUNT), LEAF_COUNT * 408);
    }

    /// **绝对值：改一项时逐项的写出节点数恰等于 1 + 树高。**
    #[test]
    fn changing_one_item_writes_exactly_one_leaf_plus_the_spine() {
        for node_bytes in [4096u64, 16384] {
            let tree_height = height(node_bytes, LEAF_COUNT) as u64;
            assert_eq!(per_item_writes(node_bytes, LEAF_COUNT, 1, Layout::Scattered), 1 + tree_height);
        }
    }

    /// 散布触到的叶数封顶于叶总数：K 超过叶数后每叶都被触到，代价不再涨
    /// （变异审计补的：此前 k > 叶数只在输出档出现，没有测试走到封顶分支）。
    #[test]
    fn scattered_leaf_count_saturates_at_the_leaf_total() {
        let writes_at_leaf_total = per_item_writes(4096, LEAF_COUNT, LEAF_COUNT, Layout::Scattered);
        let writes_beyond_leaf_total = per_item_writes(4096, LEAF_COUNT, 10 * LEAF_COUNT, Layout::Scattered);
        assert_eq!(writes_at_leaf_total, writes_beyond_leaf_total, "K 超过叶数后散布代价该饱和");
    }

    /// 扇出用独立算术钉死，防止所有臂一起错。
    #[test]
    fn fanout_matches_independent_arithmetic() {
        assert_eq!(fanout(4096), 100);
        assert_eq!(fanout(16384), 408);
    }
}
