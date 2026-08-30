//! E25：环与链各自要多大的保留池 —— [checks-owed.md](checks-owed.md) C30。
//!
//! D23 判「取定长环」时用过一条量化：链式下保留池要从 `ckpt_cost` 涨到
//! `ckpt_cost + 一个 checkpoint 窗口内的 journal 块`。**那条是算出来的，没验过。**
//! 本实验把它做成可执行模型并逐档扫描。
//!
//! ## 机制：checkpoint 自己也要写一条 journal 记录
//!
//! 环下 journal 空间在 mkfs 时就划走了，写记录是覆写自有空间，**不向分配器要**。
//! 链下每个 journal 块都要分配 ⇒ **checkpoint 要落地，先得给它自己的那条记录找到块**。
//! 于是 checkpoint 的空间需求从 `ckpt_cost` 变成 `ckpt_cost + 该窗口的 journal 块`。
//!
//! ## 判据：卡死次数是不是恰好在 `reserve ≥ 需求` 那一档归零
//!
//! 不是「链式更差」这种相对判断——**要钉住归零的那一档在哪**
//! （`rules/test-discipline.md`：只让多条臂互相比，测不出所有臂一起错）。

use e7_index_bench::Emitter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape { Ring, Chain }

impl Shape {
    fn label(self) -> &'static str {
        match self { Shape::Ring => "ring", Shape::Chain => "chain" }
    }
}

/// 一次 checkpoint 要落地，需要多少块。
///
/// **这是本实验的被测命题**：环只需 `ckpt_cost`；链还要加上该窗口自己那些 journal 块，
/// 因为 checkpoint 记录本身也要分配。
fn checkpoint_demand(shape: Shape, ckpt_cost: u64, journal_blocks_per_window: u64) -> u64 {
    match shape {
        Shape::Ring => ckpt_cost,
        Shape::Chain => ckpt_cost + journal_blocks_per_window,
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    ckpt_stall: u64,
    ckpts_done: u64,
    normal_denied: u64,
    min_free_seen: u64,
}

/// 跑一轮。`fill` = 起始占用率（0..100）。
///
/// ⚠️ **必须有每窗口的净泄漏**，否则 checkpoint 把消耗的空间全数还回去，
/// free 永远回到起点、从不接近保留池地板，整个实验测不到任何东西（首版就是这样）。
/// 泄漏是真实存在的：D16（发布语义）已定「checkpoint 内的释放在发布前不得进入可分配集合」，
/// 加上 D22（单元原子性怎么合成）的根环 K 代扣住 —— 那部分空间在窗口内拿不回来。
const LEAK_PER_WINDOW: u64 = 8;

fn run(shape: Shape, reserve: u64, capacity: u64, fill_pct: u64,
       ops: u64, ckpt_every: u64, ckpt_cost: u64, blocks_per_op: u64) -> Out {
    let mut o = Out { min_free_seen: capacity, ..Default::default() };
    // 起始：盘被填到 fill_pct，剩下的是自由空间
    let mut free = capacity - capacity * fill_pct / 100;
    let journal_per_window = ckpt_every * blocks_per_op;

    for i in 1..=ops {
        // 普通分配（写数据/元数据）：**不许动保留池**
        let want = blocks_per_op;
        if free.saturating_sub(reserve) >= want {
            free -= want;
        } else {
            o.normal_denied += 1;
        }
        // 链式：journal 块也是普通分配，同样不许动保留池
        if shape == Shape::Chain && free.saturating_sub(reserve) >= blocks_per_op {
            free -= blocks_per_op;
        } else if shape == Shape::Chain {
            o.normal_denied += 1;
        }
        o.min_free_seen = o.min_free_seen.min(free);

        if i % ckpt_every == 0 {
            let need = checkpoint_demand(shape, ckpt_cost, journal_per_window);
            // checkpoint **可以**动保留池——那正是保留池存在的理由
            if free >= need {
                free -= need;
                o.ckpts_done += 1;
                // 回收本窗口搅动的空间，**减去被扣住的那部分**
                let recycled = ckpt_every * blocks_per_op
                      + if shape == Shape::Chain { journal_per_window } else { 0 }
                      + need;
                free += recycled.saturating_sub(LEAK_PER_WINDOW);
            } else {
                o.ckpt_stall += 1;
            }
        }
    }
    o
}

fn main() {
    let mut em = Emitter::new();
    let (capacity, ops, ckpt_every, ckpt_cost, bpo) = (100_000u64, 20_000u64, 100u64, 64u64, 1u64);
    let journal_per_window = ckpt_every * bpo;
    println!("{}", em.emit_raw(&format!(
        "name=config capacity={capacity} ops={ops} ckpt_every={ckpt_every} \
         ckpt_cost={ckpt_cost} blocks_per_op={bpo} journal_per_window={journal_per_window}")));

    for fill in [95u64, 99] {
        for shape in [Shape::Ring, Shape::Chain] {
            for reserve in [0u64, 32, 64, 100, 128, 164, 256, 512] {
                let o = run(shape, reserve, capacity, fill, ops, ckpt_every, ckpt_cost, bpo);
                println!("{}", em.emit_raw(&format!(
                    "name=cell fill={fill} shape={} reserve={reserve} \
                     ckpt_stall={} ckpts_done={} normal_denied={} min_free={}",
                    shape.label(), o.ckpt_stall, o.ckpts_done, o.normal_denied, o.min_free_seen)));
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: u64 = 100_000;
    const OPS: u64 = 20_000;
    const EVERY: u64 = 100;
    const COST: u64 = 64;
    const BPO: u64 = 1;

    /// **被测命题的绝对值，逐条钉死**，不是「链比环要得多」这种相对判断。
    #[test]
    fn checkpoint_demand_is_exactly_the_documented_arithmetic() {
        assert_eq!(checkpoint_demand(Shape::Ring, 64, 100), 64, "环只需 ckpt_cost");
        assert_eq!(checkpoint_demand(Shape::Chain, 64, 100), 164, "链需 ckpt_cost + 窗口 journal 块");
        // 窗口 journal 块为 0 时两者必须相等 —— 否则差异来自别处，不是来自分配
        assert_eq!(checkpoint_demand(Shape::Ring, 64, 0), checkpoint_demand(Shape::Chain, 64, 0));
    }

    /// **归零的那一档必须恰好落在 `reserve == 需求` 上**，两条臂各自钉死。
    /// 这条是 C30 要还的那个量：环 64、链 164。
    #[test]
    fn stall_hits_zero_exactly_at_the_predicted_reserve() {
        let cases = [(Shape::Ring, COST), (Shape::Chain, COST + EVERY * BPO)];
        for (shape, need) in cases {
            let below = run(shape, need - 1, CAP, 99, OPS, EVERY, COST, BPO);
            let at = run(shape, need, CAP, 99, OPS, EVERY, COST, BPO);
            assert!(below.ckpt_stall > 0,
                "{shape:?}：reserve={} 时本该还有卡死", need - 1);
            assert_eq!(at.ckpt_stall, 0,
                "{shape:?}：reserve={need} 时卡死本该归零，实测 {}", at.ckpt_stall);
        }
    }

    /// **链的需求恰好比环多一个窗口的 journal 块**，绝对值。
    #[test]
    fn chain_needs_exactly_one_window_of_journal_blocks_more() {
        let ring_need = COST;
        let chain_need = COST + EVERY * BPO;
        assert_eq!(chain_need - ring_need, EVERY * BPO);
        // 而且在环够用的那一档上，链**必须**还在卡
        let chain_at_ring_need = run(Shape::Chain, ring_need, CAP, 99, OPS, EVERY, COST, BPO);
        assert!(chain_at_ring_need.ckpt_stall > 0,
            "链在环够用的那一档上本该还在卡死");
    }

    /// **阳性对照，对每一条臂都跑**：盘不满时两条臂都不该卡。
    /// 少了这条，「reserve 不够会卡」分不清是保留池的作用还是盘本来就满。
    #[test]
    fn with_a_roomy_disk_neither_shape_stalls_even_at_zero_reserve() {
        for shape in [Shape::Ring, Shape::Chain] {
            let o = run(shape, 0, CAP, 50, OPS, EVERY, COST, BPO);
            assert_eq!(o.ckpt_stall, 0, "{shape:?}：盘只用一半却卡死了");
            assert_eq!(o.ckpts_done, OPS / EVERY, "{shape:?}：checkpoint 次数不对");
        }
    }

    /// **保留池只在「够大、不卡死」的区间里才是纯税。**
    ///
    /// ⚠️ 这条的第一版写错了：拿 `reserve=0` 与 `reserve=512` 比，
    /// 结果是**小保留池拒绝得更多**（9340 对 1204）——因为 checkpoint 卡死之后
    /// 空间根本不回收，拒绝数被卡死支配，与「税」无关。
    /// ⇒ E19（defer 窗口下的假性 ENOSPC）那条「保留池对普通分配是纯税」有一个隐含前提：
    /// **保留池已经大到不卡死**。前提之外它的符号是反的。
    #[test]
    fn within_the_no_stall_range_a_bigger_reserve_denies_more() {
        let at_need = run(Shape::Ring, COST, CAP, 99, OPS, EVERY, COST, BPO);
        let much_bigger = run(Shape::Ring, 512, CAP, 99, OPS, EVERY, COST, BPO);
        assert_eq!(at_need.ckpt_stall, 0, "两侧都必须在不卡死的区间里");
        assert_eq!(much_bigger.ckpt_stall, 0);
        assert!(much_bigger.normal_denied > at_need.normal_denied,
            "不卡死区间内保留池变大却没有多拒普通分配 —— 它就不是税了");
    }

    /// **卡死区间里的符号是反的**，这一格必须显式钉住，否则上一条的前提是隐含的。
    #[test]
    fn inside_the_stalling_range_a_smaller_reserve_denies_far_more() {
        let starved = run(Shape::Ring, 0, CAP, 99, OPS, EVERY, COST, BPO);
        let ok = run(Shape::Ring, COST, CAP, 99, OPS, EVERY, COST, BPO);
        assert!(starved.ckpt_stall > 0, "reserve=0 本该卡死");
        assert_eq!(ok.ckpt_stall, 0);
        assert!(starved.normal_denied > ok.normal_denied * 5,
            "卡死时拒绝数本该被卡死支配，远高于不卡死那档");
    }

    /// checkpoint 次数必须与 ops/ckpt_every 一致（没卡死时），否则计数本身错了。
    #[test]
    fn checkpoints_are_counted_correctly_when_nothing_stalls() {
        let o = run(Shape::Ring, COST, CAP, 99, OPS, EVERY, COST, BPO);
        assert_eq!(o.ckpt_stall, 0);
        assert_eq!(o.ckpts_done, OPS / EVERY);
        assert_eq!(o.ckpts_done, 200);
    }
}
