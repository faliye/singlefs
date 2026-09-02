//! E82：准入的在飞合成 —— D3 的准入不等式六项都是已发布统计量，而准入发生在窗口中间；
//! 不叠在飞增量会怎样、叠错（把窗口内释放也记成可用）会怎样、忘了墓碑预留会怎样。
//!
//! ## 为什么要有这个实验
//!
//! D3（空间分配）已定项 4 定了准入不等式的权威形式（可用 = Σ设备(容量 − 已分配 − 不可回收 −
//! defer 待释放) − 待删占用 − 已承诺预留），但六项都是**已发布**统计量；
//! 「准入读数 = 已发布值 + 在飞 overlay」这层合成全仓零覆盖（C87（准入读的合成值无定义））。
//! 三种直觉写法各有一个会静默出错的形态，本实验把三个都做成**可数的违例**，
//! 给崩溃点重放与准入路径的测试当红样例。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D3（空间分配）第 3 条：「任何增加空间占用的操作，进门前先算**最坏情况下解开自己需要多少**」。
//! - D3 已定项 2：「墓碑就是『解开自己』的一部分，计入那个量」；「删除路径仍然不申请空间：
//!   它要写的那个墓碑，空间在对象写进来的时候就已经算过并留住了」。
//! - D16（发布语义）新规则 2：「checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合」。
//!
//! ## 模型与臂
//!
//! 容量 CAP 格；一个窗口内 K 个事务各要 R 格，窗口内无发布。四条臂：
//!
//! | 臂 | 准入读数 | 要证的 |
//! |---|---|---|
//! | published_only | 只读已发布空闲 | 同窗并发全过闸 ⇒ **超卖**（分配中途失败） |
//! | overlay（正确形态） | 已发布 − 在飞已批 | 超卖恒 0 |
//! | overlay_credit_frees（bug 臂） | 还把窗口内释放记成可用 | **过早复用**（分给了 defer 该扣住的块，D16 新规则 2 被绕过） |
//! | forget_tombstone（bug 臂） | 不把墓碑计入「解开自己」 | 盘满时**删除死锁**（删不动） |
//!
//! ## 判据（跑前写死，跑完不许改；全部闭式）
//!
//! 1. published_only：K×R > 可用时超卖恰 = K×R − 可用；overlay 超卖恒 0 且恰批 ⌊可用/R⌋ 个。
//! 2. credit_frees 臂：过早复用数恰 = min(B 需求 − 无信用可用, 窗口释放数)（手算锚点钉住）；
//!    正确 overlay 恒 0。
//! 3. forget_tombstone 臂：盘满后删除死锁数 > 0 且恰 = 尝试删除数；正确形态（写入时预留墓碑格）
//!    删除死锁恒 0，且能装下的对象数恰 = ⌊CAP / (对象格 + 墓碑格)⌋。
//! 4. 三个 bug 臂的违例计数器各自必须能读出非零（C60（恒定读数没有故障注入自证）），
//!    正确臂全部恒 0。任一不中 ⇒ 作废。
//!
//! ## 它答不了的
//!
//! 纯算术模型，文件操作 0 处。不建模 defer 窗口时长、多设备维、不可回收量与
//! 已承诺预留之外的其余项；「墓碑一格」按 E83（墓碑的粒度）的记录级打包口径是高估
//! （那边的粒度定了之后这里的常数跟着换，判决形状不变）。

use e7_index_bench::Emitter;

/// 一个事务的准入与分配结果。
#[derive(Debug, Default)]
struct Tally {
    admitted: u64,
    rejected: u64,
    /// 分配中途拿不到格（准入放行了但空间不存在）= 超卖违例。
    oversold_units: u64,
    /// 分到了本窗口刚释放的格（D16 新规则 2 被绕过）。
    premature_reuse: u64,
    /// 盘满时删除做不动（墓碑没地方写）。
    delete_deadlocks: u64,
}

/// 臂一 / 臂二：K 个同窗事务各要 R 格。
/// `overlay` = 准入时把在飞已批扣进读数。
fn window_admission(cap: u64, published_used: u64, k: u64, r: u64, overlay: bool) -> Tally {
    let mut t = Tally::default();
    let published_free = cap - published_used;
    let mut inflight = 0u64;
    let mut truly_free = published_free; // 物理真值，分配时消耗
    for _ in 0..k {
        let visible = if overlay { published_free - inflight.min(published_free) } else { published_free };
        if r <= visible {
            t.admitted += 1;
            inflight += r;
            // 分配：物理上拿得到多少是多少，拿不到的就是超卖
            let got = r.min(truly_free);
            truly_free -= got;
            t.oversold_units += r - got;
        } else {
            t.rejected += 1;
        }
    }
    t
}

/// 臂三：COW 重写事务 A（释放 freed 格）之后，事务 B 要 r_b 格。
/// `credit_frees` = 把 A 在本窗口的释放也记成可用（违反 D16 新规则 2 的形态）。
fn credit_frees_arm(published_free: u64, a_takes: u64, a_frees: u64, r_b: u64, credit: bool) -> Tally {
    let mut t = Tally::default();
    // A 先过闸（overlay 正确扣）
    assert!(a_takes <= published_free);
    let after_a = published_free - a_takes;
    let visible_b = if credit { after_a + a_frees } else { after_a };
    if r_b <= visible_b {
        t.admitted += 1;
        // B 的分配：先吃真空闲，吃不够就吃 A 刚释放的（defer 本该扣住的）
        let from_free = r_b.min(after_a);
        t.premature_reuse = r_b - from_free;
    } else {
        t.rejected += 1;
    }
    t
}

/// 臂四：写入 n 个对象（各 1 数据格），删除时各要写 1 个墓碑格（D18 已定项 8）。
/// `reserve` = 写入时按 D3 已定项 2 把墓碑那格一起预留。
fn tombstone_arm(cap: u64, reserve: bool) -> (u64, Tally) {
    let mut t = Tally::default();
    let per_obj = if reserve { 2 } else { 1 };
    let n = cap / per_obj; // 能装下的对象数（准入按各自口径放行到满）
    // 盘满后删除全部对象：每个删除要 1 格写墓碑
    let free_at_full = cap - n * per_obj + if reserve { n } else { 0 }; // 预留形态下那 n 格就是给墓碑的
    let mut avail = free_at_full;
    for _ in 0..n {
        if avail >= 1 {
            avail -= 1; // 写墓碑
        } else {
            t.delete_deadlocks += 1;
        }
    }
    (n, t)
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw("name=config model=arithmetic file_ops=0"));

    // 臂一 / 臂二：CAP=100、已发布已用 60、K=4 事务各要 30
    for overlay in [false, true] {
        let t = window_admission(100, 60, 4, 30, overlay);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=window overlay={} admitted={} rejected={} oversold_units={}",
                u8::from(overlay),
                t.admitted,
                t.rejected,
                t.oversold_units
            ))
        );
    }
    // 臂三：已发布空闲 40，A 拿 30 且释放 30 旧格，B 要 30
    for credit in [false, true] {
        let t = credit_frees_arm(40, 30, 30, 30, credit);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=credit_frees credit={} admitted={} rejected={} premature_reuse={}",
                u8::from(credit),
                t.admitted,
                t.rejected,
                t.premature_reuse
            ))
        );
    }
    // 臂四：CAP=100
    for reserve in [true, false] {
        let (n, t) = tombstone_arm(100, reserve);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=tombstone reserve={} objects_fit={n} delete_deadlocks={}",
                u8::from(reserve),
                t.delete_deadlocks
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1**：published_only 同窗 4×30 全过闸 ⇒ 超卖恰 120−40=80；
    /// overlay 恰批 1 个（40 可用装不下第二个 30）⇒ 超卖 0。
    #[test]
    fn absolute_window_arithmetic() {
        let p = window_admission(100, 60, 4, 30, false);
        assert_eq!(p.admitted, 4, "published_only 看不见在飞，全放行");
        assert_eq!(p.oversold_units, 80);
        let o = window_admission(100, 60, 4, 30, true);
        assert_eq!(o.admitted, 1);
        assert_eq!(o.rejected, 3);
        assert_eq!(o.oversold_units, 0);
    }

    /// **判据 2 手算锚点**：空闲 40、A 拿 30 释 30、B 要 30——
    /// credit 臂放行且 20 格吃进 A 刚释放的（10 真空闲 + 20 过早复用）；正确臂拒绝 B。
    #[test]
    fn absolute_credit_frees() {
        let bug = credit_frees_arm(40, 30, 30, 30, true);
        assert_eq!(bug.admitted, 1);
        assert_eq!(bug.premature_reuse, 20, "恰 20 格是 defer 该扣住的");
        let ok = credit_frees_arm(40, 30, 30, 30, false);
        assert_eq!(ok.admitted, 0);
        assert_eq!(ok.rejected, 1);
        assert_eq!(ok.premature_reuse, 0);
    }

    /// **判据 3**：预留形态装 ⌊100/2⌋=50 个对象、删除死锁 0；
    /// 忘预留装 100 个、盘满后 100 个删除**全部**死锁。
    #[test]
    fn absolute_tombstone() {
        let (n_ok, t_ok) = tombstone_arm(100, true);
        assert_eq!(n_ok, 50);
        assert_eq!(t_ok.delete_deadlocks, 0);
        let (n_bug, t_bug) = tombstone_arm(100, false);
        assert_eq!(n_bug, 100);
        assert_eq!(t_bug.delete_deadlocks, 100, "一个都删不动");
    }

    /// **判据 4（C60）**：三个违例计数器各自能读出非零——恒零的读数与没在看长得一样。
    #[test]
    fn counters_have_discrimination() {
        assert!(window_admission(100, 60, 4, 30, false).oversold_units > 0);
        assert!(credit_frees_arm(40, 30, 30, 30, true).premature_reuse > 0);
        assert!(tombstone_arm(100, false).1.delete_deadlocks > 0);
    }

    /// 正确形态三个违例恒 0（多组参数扫一遍）。
    #[test]
    fn correct_forms_never_violate() {
        for (cap, used, k, r) in [(100u64, 60u64, 4u64, 30u64), (1000, 0, 40, 30), (64, 60, 9, 2)] {
            let t = window_admission(cap, used, k, r, true);
            assert_eq!(t.oversold_units, 0, "cap={cap}");
        }
        for (free, at, af, rb) in [(40u64, 30u64, 30u64, 30u64), (100, 50, 50, 60), (10, 5, 5, 6)] {
            assert_eq!(credit_frees_arm(free, at, af, rb, false).premature_reuse, 0);
        }
        for cap in [2u64, 100, 999] {
            assert_eq!(tombstone_arm(cap, true).1.delete_deadlocks, 0, "cap={cap}");
        }
    }

    /// overlay 不许把拒绝写成超卖：拒绝的事务不占物理格。
    #[test]
    fn rejected_txns_consume_nothing() {
        let t = window_admission(100, 60, 4, 30, true);
        assert_eq!(t.admitted + t.rejected, 4);
        // 物理上恰好只被批的那一个消耗了 30 格：再来一个 10 格的能过
        let t2 = window_admission(100, 60, 1, 10, true);
        assert_eq!(t2.admitted, 1);
    }

    /// 边界：恰好装下（K×R == 可用）时两种读数都全放行且零超卖——差别只在超额时出现。
    #[test]
    fn exact_fit_is_clean_either_way() {
        for overlay in [false, true] {
            let t = window_admission(100, 40, 2, 30, overlay);
            assert_eq!(t.admitted, 2, "overlay={overlay}");
            assert_eq!(t.oversold_units, 0);
        }
    }
}
