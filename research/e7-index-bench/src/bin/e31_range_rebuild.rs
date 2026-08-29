//! E31：范围重建 vs 逐项修改的交叉点 —— [experiments.md](experiments.md) E4 里能建模的那一半。
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

const PTR_BYTES: u64 = 40;

fn fanout(node_bytes: u64) -> u64 { ((node_bytes.saturating_sub(64)) / PTR_BYTES).max(2) }

fn height(node_bytes: u64, leaves: u64) -> u32 {
    let f = fanout(node_bytes); let mut h = 0u32; let mut cap = 1u64;
    while cap < leaves { cap = cap.saturating_mul(f); h += 1; } h
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layout { Contiguous, Scattered }

/// 逐项修改写出的节点数。
///
/// 连续：K 个条目落在 ceil(K/扇出) 个叶子上，脊柱在上层高度共享；
/// 散布：K 个条目落在 min(K, 叶数) 个不同叶子上，脊柱只在根共享。
fn per_item_writes(node_bytes: u64, leaves: u64, k: u64, layout: Layout) -> u64 {
    let f = fanout(node_bytes);
    let h = height(node_bytes, leaves) as u64;
    let touched_leaves = match layout {
        Layout::Contiguous => k.div_ceil(f).max(1),
        Layout::Scattered  => k.min(leaves),
    };
    // 每一层被触到的互异节点数：上一层节点数 / 扇出，向上取整，至少 1
    let mut total = touched_leaves;
    let mut cur = touched_leaves;
    for _ in 0..h {
        cur = cur.div_ceil(f).max(1);
        total += cur;
    }
    total
}

/// 范围重建写出的节点数：覆盖该范围的全部叶子 + 其上的脊柱，**与 K 无关**。
fn range_rebuild_writes(node_bytes: u64, leaves: u64, range_leaves: u64) -> u64 {
    let f = fanout(node_bytes);
    let h = height(node_bytes, leaves) as u64;
    let mut total = range_leaves;
    let mut cur = range_leaves;
    for _ in 0..h { cur = cur.div_ceil(f).max(1); total += cur; }
    total
}

/// 总条目数 = 叶数 × 扇出。
/// ⚠️ **K 数的是条目不是叶子**——第一版把两者混了，
/// 于是连续布局在 100% 时也只触到 1/扇出 的叶子，交叉点永远不出现（测试当场红）。
fn total_entries(node_bytes: u64, leaves: u64) -> u64 { leaves.saturating_mul(fanout(node_bytes)) }

/// 交叉点：受影响比例 K/N 达到多少时范围重建开始更省。返回千分比。
fn crossover_permille(node_bytes: u64, leaves: u64, layout: Layout) -> Option<u64> {
    let n = total_entries(node_bytes, leaves);
    for pm in 1..=1000u64 {
        let k = (n * pm / 1000).max(1);
        if per_item_writes(node_bytes, leaves, k, layout) >= range_rebuild_writes(node_bytes, leaves, leaves) {
            return Some(pm);
        }
    }
    None
}

fn main() {
    let mut em = Emitter::new();
    let leaves = 1u64 << 20;
    println!("{}", em.emit_raw(&format!("name=config leaves={leaves} ptr_bytes={PTR_BYTES}")));
    for nb in [4096u64, 16384] {
        for layout in [Layout::Contiguous, Layout::Scattered] {
            for pm in [1u64, 10, 50, 100, 300, 1000] {
                let k = (total_entries(nb, leaves) * pm / 1000).max(1);
                println!("{}", em.emit_raw(&format!(
                    "name=cell node_bytes={nb} layout={layout:?} permille={pm} k={k} \
                     per_item={} range={}",
                    per_item_writes(nb, leaves, k, layout),
                    range_rebuild_writes(nb, leaves, leaves))));
            }
            println!("{}", em.emit_raw(&format!(
                "name=crossover node_bytes={nb} layout={layout:?} permille={}",
                crossover_permille(nb, leaves, layout).map(|x| x.to_string())
                    .unwrap_or_else(|| "none".into()))));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const L: u64 = 1 << 20;

    /// **范围重建的代价与 K 无关**——这是它区别于逐项的唯一理由。
    #[test]
    fn range_rebuild_cost_is_independent_of_k() {
        let a = range_rebuild_writes(4096, L, L);
        let b = range_rebuild_writes(4096, L, L);
        assert_eq!(a, b);
        // 且它恰等于「全部叶子 + 各层脊柱」，绝对值可算
        let f = fanout(4096);
        let mut expect = L; let mut cur = L;
        for _ in 0..height(4096, L) { cur = cur.div_ceil(f).max(1); expect += cur; }
        assert_eq!(a, expect);
    }

    /// **K 很小时逐项必须明显更省**，否则交叉点无从谈起。
    #[test]
    fn per_item_is_far_cheaper_when_only_a_few_items_change() {
        for layout in [Layout::Contiguous, Layout::Scattered] {
            let one = per_item_writes(4096, L, 1, layout);
            let full = range_rebuild_writes(4096, L, L);
            assert!(one * 100 < full, "改一项该比整段重建便宜两个数量级（{layout:?}）");
        }
    }

    /// **散布比连续贵**——连续时多个条目共享叶子与脊柱，散布时不共享。
    /// 少了这条，`layout` 这一维就是摆设。
    #[test]
    fn scattered_costs_more_than_contiguous_at_the_same_k() {
        for k in [100u64, 1000, 10_000] {
            let c = per_item_writes(4096, L, k, Layout::Contiguous);
            let s = per_item_writes(4096, L, k, Layout::Scattered);
            assert!(s > c, "k={k} 时散布该比连续贵（{s} vs {c}）");
        }
    }

    /// **两种布局的交叉点必须不同**，且散布的交叉点更早（散布更贵 ⇒ 更早值得整段重建）。
    #[test]
    fn the_two_layouts_cross_over_at_different_ratios() {
        let c = crossover_permille(4096, L, Layout::Contiguous).expect("连续该有交叉点");
        let s = crossover_permille(4096, L, Layout::Scattered).expect("散布该有交叉点");
        assert!(s < c, "散布更贵，交叉点该更早（散布 {s}‰ vs 连续 {c}‰）");
    }

    /// **总条目数是叶数 × 扇出，不是叶数。** 这条钉住第一版混掉的那个口径。
    #[test]
    fn total_entries_counts_entries_not_leaves() {
        assert_eq!(total_entries(4096, L), L * 100);
        assert_eq!(total_entries(16384, L), L * 408);
    }

    /// **绝对值：改一项时逐项的写出节点数恰等于 1 + 树高。**
    #[test]
    fn changing_one_item_writes_exactly_one_leaf_plus_the_spine() {
        for nb in [4096u64, 16384] {
            let h = height(nb, L) as u64;
            assert_eq!(per_item_writes(nb, L, 1, Layout::Scattered), 1 + h);
        }
    }

    /// 扇出用独立算术钉死，防止所有臂一起错。
    #[test]
    fn fanout_matches_independent_arithmetic() {
        assert_eq!(fanout(4096), 100);
        assert_eq!(fanout(16384), 408);
    }
}
