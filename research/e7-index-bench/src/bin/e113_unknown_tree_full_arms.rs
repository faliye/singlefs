//! E113：不认识的树·补齐臂集 —— E112 的重做，臂集与选法都改了。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D18（块里携带什么信息）：「偏移 7 是 8 个 flag 位，第一版恒 0，**读者遇到非 0 位一律拒收**
//!   ……本工程**不给**这 8 位留「忽略陌生位」的通道」⇒ `RejectEntry` 臂的出处。
//! - D8（核心索引结构）：「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**
//!   （对象级，不拒整个容器）」，落成 I-9.7（记录三段恒零）。
//! - D15（格式冻结政策）：「新增记录类型（新 key 类型）| 只读遍历可能误判；整理 / 写可能误删看不懂的记录
//!   | **compat_ro** | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判」⇒ `ReadOnly` 臂的出处。
//! - D5（快照 / 空间记账机制）：「销毁快照时把它的 deadlist **合并到下一个更新的那一侧**，
//!   并按 `birth > prev(S).txg` 过滤出可释放的条目」⇒ `blocks_freed_while_referenced` 的机制推导。
//! - D5（快照 / 空间记账机制）已定项 2：「只保留最近 K 代」；**K 的值至今未定**
//!   （E54 逐字「真正要定的只有 K」）⇒ K 必须扫，不许钉死。
//! - D5（快照 / 空间记账机制）已定项 4 第 12 项：inode 号水位，带树维「**带**（只为可写头维护）」，
//!   **今天只有这一项带树维**。
//!
//! ## 选法（跑前写死，字典序）
//!
//! 1 不许丢数据 → 2 不许违反语义 → 3 不许丢账 → 4 静默最少 → 5 可写代数最多 → 6 写行数最少。
//! **不设 `mount_ok` 筛子**：只读挂载的代价由第 5 关定价，不是把它请出场。

use e7_index_bench::Emitter;

const T_KNOWN: u64 = 12;
const H_KNOWN: u64 = 6;
/// v1 不认识的树（新种类，或已知种类但置了 v1 不认识的 flags 位）。
const T_NEW: u64 = 4;
/// 其中是可写头的 —— 只有这些才有带树维的记账行。
const H_NEW: u64 = 2;
/// 其中那个未知位取「只读」语义的（现役先例：btrfs 的 `BTRFS_ROOT_SUBVOL_RDONLY`）。
const T_NEW_RDONLY: u64 = 1;
/// 其中那个未知位取「半删」语义的（现役先例：`BTRFS_ROOT_SUBVOL_DEAD`）。
const T_NEW_DEAD: u64 = 1;
const R_TREE_DIM: u64 = 1;
const R_POOL: u64 = 11;
const G_GENS: u64 = 10;
/// 每代出生多少块 —— 误释放数的量纲。
const BLOCKS_PER_GEN: u64 = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flags {
    Ignore,
    Skip,
    RejectEntry,
    ReadOnly,
    PerBit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Acct {
    Carry,
    NoCarry,
    Exempt,
}

#[derive(Clone, Copy, Debug)]
struct Params {
    /// 只保留最近几代（D5 已定项 2，值未定 ⇒ 扫）。
    k_keep: u64,
    /// 隐形窗口跨几代（0 = 没有隐形窗口，阴性对照）。
    w_gens: u64,
    /// 不认识的树里有几棵是可写头。
    h_new: u64,
    /// 不认识的位里有几棵树的位是**已登记为 compat** 的（`PerBit` 才用得上）。
    trees_with_registered_bit: u64,
}

impl Params {
    fn base() -> Self {
        Params { k_keep: 4, w_gens: 3, h_new: H_NEW, trees_with_registered_bit: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Outcome {
    writable_gens: u64,
    blocks_freed_while_referenced: u64,
    rdonly_violations: u64,
    dead_revived: u64,
    rows_lost: u64,
    rows_written_per_gen: u64,
    /// 出了错而**不变量清单里没有任何一条**盖得住它。
    silent: u64,
}

/// `PerBit` 是个条件政策：只要还有一棵树的位没登记，整次挂载就退到只读；
/// 全部登记过（登记时已逐项证明写侧安全）才退化成跳过。
fn perbit_reduces_to(p: Params) -> Flags {
    if p.trees_with_registered_bit >= T_NEW {
        Flags::Skip
    } else {
        Flags::ReadOnly
    }
}

/// 误释放的机制推导：一棵对写者隐形的快照使「下一个更新的那一侧」认成更新的那个
/// ⇒ 过滤用的 `prev` 更大 ⇒ 隐形窗口里出生的块全部通过「可直接释放」。
fn blocks_wrongly_freed(w_gens: u64) -> u64 {
    BLOCKS_PER_GEN * w_gens
}

fn acct_rows(f: Flags, a: Acct, p: Params, writable_gens: u64) -> (u64, u64) {
    // 一代都没发布 ⇒ 代号不前进 ⇒ 既不写也不丢。
    if writable_gens == 0 {
        return (0, 0);
    }
    let own = R_POOL + H_KNOWN * R_TREE_DIM;
    let theirs = p.h_new * R_TREE_DIM;
    // 自以为认识 ⇒ 那些行本来就在它的维护范围里，搬运政策没有作用点。
    if f == Flags::Ignore {
        return (0, own + theirs);
    }
    match a {
        Acct::Carry => (0, own + theirs),
        // 豁免点删：不写，也不丢。
        Acct::Exempt => (0, own),
        Acct::NoCarry => {
            let lost = if writable_gens > p.k_keep { theirs } else { 0 };
            (lost, own)
        }
    }
}

fn run(f: Flags, a: Acct, p: Params) -> Outcome {
    let eff = if f == Flags::PerBit { perbit_reduces_to(p) } else { f };
    let writable_gens = if eff == Flags::ReadOnly { 0 } else { G_GENS };

    // 只有「树对写者隐形」才会认错 deadlist 的邻居。
    // RejectEntry 下条目在、只是不可用 ⇒ 写者知道那儿有一棵树 ⇒ 不做跨它的合并。
    let blocks = if eff == Flags::Skip && writable_gens > 0 {
        blocks_wrongly_freed(p.w_gens)
    } else {
        0
    };

    // 只有「自以为认识」才会把只读树当可写、把半删树当活的。
    let (rdonly, dead) = if eff == Flags::Ignore {
        (T_NEW_RDONLY * writable_gens, T_NEW_DEAD)
    } else {
        (0, 0)
    };

    let (rows_lost, rows_written_per_gen) = acct_rows(eff, a, p, writable_gens);

    // 谁盖得住：丢账有 I-9.11（可写克隆头有水位行）；误释放有 I-3.1（已分配统计对得上），
    // 但只在 checker 不跟着跳过时成立（C158）；违反只读 / 复活半删树**一条都没有**。
    let covered = |harm: u64, has_invariant: bool| -> u64 {
        u64::from(harm > 0 && !has_invariant)
    };
    let silent = covered(blocks, true) + covered(rdonly, false) + covered(dead, false)
        + covered(rows_lost, true);

    Outcome {
        writable_gens,
        blocks_freed_while_referenced: blocks,
        rdonly_violations: rdonly,
        dead_revived: dead,
        rows_lost,
        rows_written_per_gen,
        silent,
    }
}

/// 跑前写死的字典序。返回值越小越好，逐位比较。
fn rank_key(o: &Outcome) -> (u64, u64, u64, u64, u64, u64) {
    (
        o.blocks_freed_while_referenced,             // 1 不许丢数据
        o.rdonly_violations + o.dead_revived,        // 2 不许违反语义
        o.rows_lost,                                 // 3 不许丢账
        o.silent,                                    // 4 静默最少
        G_GENS - o.writable_gens,                    // 5 可写代数最多
        o.rows_written_per_gen,                      // 6 写行数最少
    )
}

const FLAGS_ARMS: [Flags; 5] = [
    Flags::Ignore,
    Flags::Skip,
    Flags::RejectEntry,
    Flags::ReadOnly,
    Flags::PerBit,
];
const ACCT_ARMS: [Acct; 3] = [Acct::Carry, Acct::NoCarry, Acct::Exempt];

fn winners(p: Params) -> Vec<String> {
    let mut best: Option<(u64, u64, u64, u64, u64, u64)> = None;
    let mut w: Vec<String> = Vec::new();
    for f in FLAGS_ARMS {
        for a in ACCT_ARMS {
            let k = rank_key(&run(f, a, p));
            match best {
                None => {
                    best = Some(k);
                    w = vec![format!("{f:?}_{a:?}")];
                }
                Some(b) if k < b => {
                    best = Some(k);
                    w = vec![format!("{f:?}_{a:?}")];
                }
                Some(b) if k == b => w.push(format!("{f:?}_{a:?}")),
                _ => {}
            }
        }
    }
    w.sort();
    w
}

fn main() {
    let mut em = Emitter::new();
    let p0 = Params::base();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config t_known={T_KNOWN} h_known={H_KNOWN} t_new={T_NEW} h_new={H_NEW} \
             t_new_rdonly={T_NEW_RDONLY} t_new_dead={T_NEW_DEAD} r_tree_dim={R_TREE_DIM} \
             r_pool={R_POOL} g_gens={G_GENS} blocks_per_gen={BLOCKS_PER_GEN} \
             k_keep={} w_gens={} model=arithmetic file_ops=0",
            p0.k_keep, p0.w_gens
        ))
    );

    for f in FLAGS_ARMS {
        for a in ACCT_ARMS {
            let o = run(f, a, p0);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=cell arm={f:?}_{a:?} writable_gens={} blocks_freed_while_referenced={} \
                     rdonly_violations={} dead_revived={} rows_lost={} rows_written_per_gen={} silent={}",
                    o.writable_gens,
                    o.blocks_freed_while_referenced,
                    o.rdonly_violations,
                    o.dead_revived,
                    o.rows_lost,
                    o.rows_written_per_gen,
                    o.silent
                ))
            );
        }
    }

    println!(
        "{}",
        em.emit_raw(&format!("name=verdict_base winners={}", winners(p0).join("+")))
    );

    // 参数扫：跑前写死要报「排名翻不翻」。
    let base_w = winners(p0);
    let mut flips = 0u64;
    for k in [1u64, 2, 4, 8, 20] {
        for w in [0u64, 1, 3, 8] {
            for h in [0u64, 2, 4] {
                for reg in [0u64, T_NEW] {
                    let p = Params { k_keep: k, w_gens: w, h_new: h, trees_with_registered_bit: reg };
                    let ws = winners(p);
                    let flipped = ws != base_w;
                    if flipped {
                        flips += 1;
                    }
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=sweep k_keep={k} w_gens={w} h_new={h} registered={reg} \
                             winners={} flipped={}",
                            ws.join("+"),
                            u8::from(flipped)
                        ))
                    );
                }
            }
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=sweep_summary points={} flipped={flips}",
            5 * 4 * 3 * 2
        ))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：模型常量逐个钉住。
    #[test]
    fn absolute_constants() {
        assert_eq!(R_TREE_DIM + R_POOL, 12, "在用十二个统计量");
        assert_eq!(FLAGS_ARMS.len(), 5);
        assert_eq!(ACCT_ARMS.len(), 3);
        assert_eq!(BLOCKS_PER_GEN, 64);
        assert_eq!(T_NEW_RDONLY + T_NEW_DEAD + 2, T_NEW, "四棵里两棵带语义位");
    }

    /// **阳性对照·Ignore**：必须违反只读语义并复活半删树。
    #[test]
    fn positive_ignore() {
        let o = run(Flags::Ignore, Acct::Carry, Params::base());
        assert_eq!(o.rdonly_violations, T_NEW_RDONLY * G_GENS);
        assert_eq!(o.rdonly_violations, 10);
        assert_eq!(o.dead_revived, 1);
        assert_eq!(o.blocks_freed_while_referenced, 0, "树可见 ⇒ 邻居认得对");
    }

    /// **阳性对照·Skip**：必须误释放仍被引用的块，且数是闭式算出来的。
    #[test]
    fn positive_skip() {
        let p = Params::base();
        let o = run(Flags::Skip, Acct::Carry, p);
        assert!(o.blocks_freed_while_referenced > 0, "误释放这一维没进模型");
        assert_eq!(o.blocks_freed_while_referenced, BLOCKS_PER_GEN * p.w_gens);
        assert_eq!(o.blocks_freed_while_referenced, 192);
    }

    /// **阳性对照·RejectEntry**：不丢数据、不违反语义，但账照样会丢（NoCarry 下）。
    #[test]
    fn positive_reject_entry() {
        let o = run(Flags::RejectEntry, Acct::NoCarry, Params::base());
        assert_eq!(o.blocks_freed_while_referenced, 0, "条目在 ⇒ 写者知道那儿有树");
        assert_eq!(o.rdonly_violations + o.dead_revived, 0);
        assert!(o.rows_lost > 0, "RejectEntry 这一维的代价没进模型");
        assert_eq!(o.rows_lost, 2);
    }

    /// **阳性对照·ReadOnly**：一代都发不出去，其余伤害全零。
    #[test]
    fn positive_read_only() {
        for a in ACCT_ARMS {
            let o = run(Flags::ReadOnly, a, Params::base());
            assert_eq!(o.writable_gens, 0, "ReadOnly 这一维没进模型");
            assert_eq!(o.blocks_freed_while_referenced, 0);
            assert_eq!(o.rows_lost, 0);
            assert_eq!(o.rows_written_per_gen, 0);
        }
    }

    /// **阳性对照·PerBit**：它是条件政策，两档各退化到一条已有的臂。
    #[test]
    fn positive_per_bit_is_conditional() {
        let unreg = Params { trees_with_registered_bit: 0, ..Params::base() };
        assert_eq!(perbit_reduces_to(unreg), Flags::ReadOnly);
        assert_eq!(run(Flags::PerBit, Acct::Carry, unreg), run(Flags::ReadOnly, Acct::Carry, unreg));

        let reg = Params { trees_with_registered_bit: T_NEW, ..Params::base() };
        assert_eq!(perbit_reduces_to(reg), Flags::Skip);
        assert_eq!(run(Flags::PerBit, Acct::Carry, reg), run(Flags::Skip, Acct::Carry, reg));
    }

    /// **阴性对照**：没有隐形窗口就不该有误释放。
    #[test]
    fn negative_no_window_no_loss() {
        let p = Params { w_gens: 0, ..Params::base() };
        assert_eq!(run(Flags::Skip, Acct::Carry, p).blocks_freed_while_referenced, 0);
    }

    /// **不许用筛子请人出场**：五条 flags 臂都必须出现在排名里。
    #[test]
    fn every_arm_is_ranked() {
        let p = Params::base();
        let mut seen = 0;
        for f in FLAGS_ARMS {
            for a in ACCT_ARMS {
                let _ = rank_key(&run(f, a, p));
                seen += 1;
            }
        }
        assert_eq!(seen, 15, "15 个格子一个都不许被跳过");
    }

    /// **字典序本身**：丢数据必须压过一切，哪怕对方一代都发不出去。
    #[test]
    fn lexicographic_order_puts_data_loss_first() {
        let p = Params::base();
        let skip = rank_key(&run(Flags::Skip, Acct::Carry, p));
        let ro = rank_key(&run(Flags::ReadOnly, Acct::Carry, p));
        assert!(ro < skip, "丢数据要输给「只读但不丢」");
    }

    /// **基准点的赢家**：钉成绝对值，不是「相对更好」。
    /// ⚠️ 跑之前写下的接受条款倾向 `PerBit`，实算赢家是 `RejectEntry_Exempt`
    /// ⇒ 按那条条款改结论。这里连**它凭哪一位赢**一起钉住。
    #[test]
    fn absolute_winner_at_base() {
        let p = Params::base();
        assert_eq!(winners(p), vec!["RejectEntry_Exempt".to_string()]);
        // 凭哪一位赢：六位全零到第五位，第六位 17。
        assert_eq!(rank_key(&run(Flags::RejectEntry, Acct::Exempt, p)), (0, 0, 0, 0, 0, 17));
        // ReadOnly 在第五位（可写代数）输掉 —— 它不丢任何东西，但一代都发不出去。
        assert_eq!(rank_key(&run(Flags::ReadOnly, Acct::Exempt, p)), (0, 0, 0, 0, 10, 0));
        // Skip 在第一位（丢数据）就输了。
        // ⚠️ 它的第四位（静默）是 0 而不是 1：误释放**按规范**有 I-3.1（已分配统计对得上） 盖着。
        // 但那要 checker 不跟着跳过 —— 跟着跳过的话两边一起少一项照样平，落点 C158。
        // ⇒ 这一位在这里不构成 Skip 的辩护，它已经在第一位输掉了。
        assert_eq!(
            rank_key(&run(Flags::Skip, Acct::Carry, p)),
            (192, 0, 0, 0, 0, 19)
        );
        // Ignore 在第二位（违反语义）输，第一位与赢家平。
        assert_eq!(rank_key(&run(Flags::Ignore, Acct::Carry, p)), (0, 11, 0, 2, 0, 19));
    }

    /// **记账三臂的写代价**：各钉一个绝对值。
    #[test]
    fn absolute_rows_written() {
        let p = Params::base();
        assert_eq!(run(Flags::Skip, Acct::Carry, p).rows_written_per_gen, 19);
        assert_eq!(run(Flags::Skip, Acct::NoCarry, p).rows_written_per_gen, 17);
        assert_eq!(run(Flags::Skip, Acct::Exempt, p).rows_written_per_gen, 17);
        // 豁免点删与不搬一样便宜，而且不丢账 —— 它的代价在别处（要改 D5 已定项 2）。
        assert_eq!(run(Flags::Skip, Acct::Exempt, p).rows_lost, 0);
        assert_eq!(run(Flags::Skip, Acct::NoCarry, p).rows_lost, 2);
    }

    /// **K 的边界**：K 大到 ≥ 发布代数就不丢账 —— K 未定，这条边界要钉住。
    #[test]
    fn absolute_k_boundary() {
        let lo = Params { k_keep: G_GENS - 1, ..Params::base() };
        let hi = Params { k_keep: G_GENS, ..Params::base() };
        assert_eq!(run(Flags::Skip, Acct::NoCarry, lo).rows_lost, 2);
        assert_eq!(run(Flags::Skip, Acct::NoCarry, hi).rows_lost, 0);
    }
}
