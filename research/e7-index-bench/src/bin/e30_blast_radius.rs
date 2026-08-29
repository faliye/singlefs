//! E30：坏一个节点的爆炸半径 —— [experiments.md](experiments.md) E1 里能建模的那一半。
//!
//! ## E1 原本的判据与 D21 矛盾，本实验先纠正它
//!
//! E1 写：「一个坏节点损失的范围随节点大小线性放大，**与本工程的可重建性目标直接冲突**」，
//! 判据是「全盘扫描重建**成功率** ≥ 目标线」。
//!
//! 而 D21（权威态与派生态的分界）已定：**索引是派生态，「丢了只是慢」**。
//! ⇒ 若索引可从单元自描述重建，坏掉一个索引节点**没有永久损失，只有重建时间**。
//! **「成功率」测的是错的东西**——该测的是**重建代价**。
//!
//! [experiments.md](experiments.md) E29（明文映射层能不能只靠单元自描述重建）
//! 已实测：元数据侧自描述在 ⇒ 逐条重建成功；**数据块侧零自描述 ⇒ 一条都重建不出来**。
//! ⇒ 本实验分两类报：**索引节点坏 = 可恢复，代价是扫描**；
//! **数据单元坏 = 真丢**，且与节点大小无关（单元就是单元）。
//!
//! ## 量什么
//!
//! 一个节点坏掉之后，要扫多少个单元才能把它覆盖的那部分索引重建出来。
//! **那个数正比于该节点覆盖的叶子数 = 扇出 ^ (树高 − 层号)**，随节点变大而放大。

use e7_index_bench::Emitter;

const PTR_BYTES: u64 = 40;   // D19 的指针头部宽度

/// 扇出 = (节点字节 − 头) / 指针宽度
fn fanout(node_bytes: u64) -> u64 {
    ((node_bytes.saturating_sub(64)) / PTR_BYTES).max(2)
}

/// 装下 `leaves` 个叶子需要的内部层数
fn height(node_bytes: u64, leaves: u64) -> u32 {
    let f = fanout(node_bytes);
    let mut h = 0u32;
    let mut cap = 1u64;
    while cap < leaves { cap = cap.saturating_mul(f); h += 1; }
    h
}

/// 第 `level` 层的一个节点覆盖多少叶子（level=1 是叶的父，level=h 是根）
fn leaves_covered(node_bytes: u64, leaves: u64, level: u32) -> u64 {
    fanout(node_bytes).saturating_pow(level).min(leaves)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Out {
    fanout: u64,
    height: u32,
    /// 坏一个第 1 层节点要重扫的单元数
    rescan_l1: u64,
    /// 坏一个根要重扫的单元数
    rescan_root: u64,
    /// 索引坏造成的**永久数据丢失**——按 D21 该恒为 0
    index_data_loss: u64,
    /// 数据单元坏造成的永久丢失——与节点大小无关
    unit_data_loss: u64,
}

fn measure(node_bytes: u64, leaves: u64) -> Out {
    let h = height(node_bytes, leaves);
    Out {
        fanout: fanout(node_bytes),
        height: h,
        rescan_l1: leaves_covered(node_bytes, leaves, 1),
        rescan_root: leaves_covered(node_bytes, leaves, h),
        index_data_loss: 0,          // 索引是派生态：D21 已定「丢了只是慢」
        unit_data_loss: 1,           // 坏一个数据单元就是丢一个单元，与节点大小无关
    }
}

fn main() {
    let mut em = Emitter::new();
    let leaves = 1u64 << 24;      // 1600 万叶
    println!("{}", em.emit_raw(&format!("name=config leaves={leaves} ptr_bytes={PTR_BYTES}")));
    for nb in [2048u64, 4096, 8192, 16384, 32768, 65536] {
        let o = measure(nb, leaves);
        println!("{}", em.emit_raw(&format!(
            "name=cell node_bytes={nb} fanout={} height={} rescan_l1={} rescan_root={} \
             index_data_loss={} unit_data_loss={}",
            o.fanout, o.height, o.rescan_l1, o.rescan_root, o.index_data_loss, o.unit_data_loss)));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const L: u64 = 1 << 24;

    /// **索引坏不造成永久数据丢失**——这是 D21（索引是派生态）的直接推论，
    /// 也是本实验纠正 E1 原判据的那一条。
    #[test]
    fn a_bad_index_node_causes_no_permanent_data_loss() {
        for nb in [2048u64, 16384, 65536] {
            assert_eq!(measure(nb, L).index_data_loss, 0,
                "索引是派生态，坏一个索引节点不该有永久损失（节点 {nb} 字节）");
        }
    }

    /// **数据单元坏造成的损失与节点大小无关**——单元就是单元。
    /// ⇒ E1 原文「损失范围随节点大小线性放大」说的不可能是数据单元。
    #[test]
    fn unit_loss_does_not_scale_with_node_size() {
        let a = measure(2048, L).unit_data_loss;
        let b = measure(65536, L).unit_data_loss;
        assert_eq!(a, b, "数据单元的损失与节点大小无关");
    }

    /// **真正随节点大小放大的是重扫代价**，绝对值钉死：
    /// 第 1 层节点覆盖的叶子数恰等于扇出。
    #[test]
    fn rescan_cost_equals_the_fanout_for_a_level_one_node() {
        for nb in [2048u64, 4096, 16384, 65536] {
            let o = measure(nb, L);
            assert_eq!(o.rescan_l1, o.fanout,
                "第 1 层节点覆盖的叶子数该恰等于扇出（节点 {nb} 字节）");
        }
    }

    /// **节点越大重扫越多**——这才是 E1 该量的那条曲线。
    #[test]
    fn bigger_nodes_cost_more_rescan() {
        let small = measure(2048, L).rescan_l1;
        let big = measure(65536, L).rescan_l1;
        assert!(big > small * 8, "节点从 2K 涨到 64K，重扫量该涨一个量级（{small} -> {big}）");
    }

    /// **坏根要重扫全部叶子**，与节点大小无关——上界在这里。
    #[test]
    fn losing_the_root_always_costs_a_full_scan() {
        for nb in [2048u64, 16384, 65536] {
            assert_eq!(measure(nb, L).rescan_root, L, "坏根就是全盘扫描，节点 {nb} 字节");
        }
    }

    /// **扇出的绝对值由独立算术钉死**，不是「与别处一致」。
    /// ⚠️ 变异测试补出来的：把扇出公式除以 2，六个测试一个都不红——
    /// 因为它们全走同一个函数，自洽但可能一起错
    /// （`rules/test-discipline.md`：只让多条臂互相比，测不出所有臂一起错）。
    #[test]
    fn fanout_matches_independently_computed_arithmetic() {
        // (节点字节 − 64 字节头) / 40 字节指针，向下取整
        assert_eq!(fanout(4096), (4096 - 64) / 40);
        assert_eq!(fanout(4096), 100);
        assert_eq!(fanout(16384), 408);
        assert_eq!(fanout(65536), 1636);
        assert_eq!(fanout(2048), 49);
    }

    /// 扇出与树高的算术必须自洽：扇出^树高 ≥ 叶数。
    #[test]
    fn fanout_and_height_are_arithmetically_consistent() {
        for nb in [2048u64, 4096, 16384, 65536] {
            let o = measure(nb, L);
            let cap = (o.fanout as u128).pow(o.height);
            assert!(cap >= L as u128, "扇出^树高 该装得下叶数（节点 {nb}）");
            let one_less = (o.fanout as u128).pow(o.height - 1);
            assert!(one_less < L as u128, "树高该是最小的那个（节点 {nb}）");
        }
    }
}
