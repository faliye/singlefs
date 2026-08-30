//! E27：D5 四条风险路径里能建模的三条 —— [experiments.md](experiments.md) E13 的路径 1 / 2 / 4。
//!
//! **路径 3（崩溃重放）不在本实验里**：它要块层写记录 + checker + 崩溃点重放 harness，
//! 那三样都还不存在。本实验只做纯逻辑与纯计数那三条，**不假装覆盖了第 3 条**。
//!
//! ## 路径 1 的定义句，原样贴在这里
//!
//! （`rules/verify-before-claiming.md`：引用一条决策去做推导之前，把定义句原样贴进笔记。
//!  E26 那一轮正是没贴就建，算出与已证性质矛盾的结果。）
//!
//!   「块 b 被快照 S 引用 ⟺ `birth(b) ≤ S.txg < death(b)`，左闭右开」
//!
//! | 边界 | 判定 | 取错的后果 |
//! |---|---|---|
//! | `birth == S.txg` | S **引用**它 | 写成 `birth < S.txg` → 判为比快照新 → 立即释放 → **数据丢失** |
//! | `death == S.txg` | S **不引用** | 写成 `S.txg ≤ death` → 挂在不需要它的快照上 → **空间泄漏 + 归属错** |
//! | `birth == death` | 任何快照都不引用，可立即释放 | 记账上对，但物理空间不许在本 checkpoint 内复用 |
//!
//! **本实验要证明的正是「取错真的会产生那个后果」**——不是「正确规则跑得通」。
//! 只验正确规则等于什么都没验：任何规则在非边界点上都一样。

use e7_index_bench::Emitter;

// ───────────────────────── 路径 1：三个边界 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rule {
    /// D5 正文：`birth ≤ S.txg < death`
    Correct,
    /// 左边界写严：`birth < S.txg < death`
    StrictBirth,
    /// 右边界写闭：`birth ≤ S.txg ≤ death`
    ClosedDeath,
}

impl Rule {
    fn label(self) -> &'static str {
        match self { Rule::Correct => "correct", Rule::StrictBirth => "strict_birth", Rule::ClosedDeath => "closed_death" }
    }
    /// 快照 S 引不引用块 b。
    fn refs(self, birth: u64, death: u64, s_txg: u64) -> bool {
        match self {
            Rule::Correct     => birth <= s_txg && s_txg < death,
            Rule::StrictBirth => birth <  s_txg && s_txg < death,
            Rule::ClosedDeath => birth <= s_txg && s_txg <= death,
        }
    }
}

/// 三个边界各构造一个块。`death = u64::MAX` 表示仍在活树里（∞）。
fn boundary_cases(s_txg: u64) -> Vec<(&'static str, u64, u64, bool)> {
    vec![
        // (名字, birth, death, D5 正文规定的判定)
        ("birth==S.txg", s_txg,     s_txg + 5, true),   // S 引用它
        ("death==S.txg", s_txg - 5, s_txg,     false),  // S 不引用
        ("birth==death", s_txg,     s_txg,     false),  // 任何快照都不引用
    ]
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct P1 { data_loss: u64, leak: u64, agree: u64 }

fn path1(rule: Rule, s_txg: u64) -> P1 {
    let mut o = P1::default();
    for (_, birth, death, expect) in boundary_cases(s_txg) {
        let got = rule.refs(birth, death, s_txg);
        match (expect, got) {
            (true, false) => o.data_loss += 1,  // 该引用却判不引用 ⇒ 立即释放 ⇒ 数据丢失
            (false, true) => o.leak += 1,       // 不该引用却判引用 ⇒ 挂在不需要它的快照上 ⇒ 泄漏
            _ => o.agree += 1,
        }
    }
    o
}

// ─────────────────── 路径 2：销毁快照的级联合并代价 ───────────────────

/// 销毁一个快照时，它的 deadlist 要并进「下一个更老的快照」。
/// **按 birth 分桶**之后，整桶的 birth 都比目标快照老时可以整桶跳过。
/// 量的是**检查过的条目数**，不是时间。
fn path2_merge_cost(entries: &[u64], target_txg: u64, bucketed: bool, bucket_span: u64) -> u64 {
    if !bucketed {
        return entries.len() as u64;                    // 逐条看
    }
    let mut examined = 0u64;
    let mut i = 0usize;
    while i < entries.len() {
        let bucket_hi = entries[i] / bucket_span * bucket_span + bucket_span - 1;
        // 整桶都比 target 老 ⇒ 整桶跳过，只付一次桶头检查
        if bucket_hi < target_txg {
            examined += 1;
            while i < entries.len() && entries[i] <= bucket_hi { i += 1; }
        } else {
            while i < entries.len() && entries[i] <= bucket_hi { examined += 1; i += 1; }
        }
    }
    examined
}

// ─────────────── 路径 4：defer 队列的持久化代价 ───────────────

/// D16 新规则 2：checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合。
/// ⇒ 本该「立即释放」的那一半也要先进 defer 队列，那是额外的持久化写。
/// 量的是**额外写入的条目数与 I/O 次数**（每 `per_block` 条打包成一次写）。
fn path4_defer_cost(immediate_frees: u64, per_block: u64) -> (u64, u64) {
    let entries = immediate_frees;
    let ios = entries.div_ceil(per_block.max(1));
    (entries, ios)
}

fn main() {
    let mut em = Emitter::new();
    let s_txg = 100u64;
    println!("{}", em.emit_raw(&format!("name=config s_txg={s_txg}")));

    // 路径 1
    for rule in [Rule::Correct, Rule::StrictBirth, Rule::ClosedDeath] {
        let o = path1(rule, s_txg);
        println!("{}", em.emit_raw(&format!(
            "name=p1 rule={} data_loss={} leak={} agree={}",
            rule.label(), o.data_loss, o.leak, o.agree)));
    }

    // 路径 2：deadlist 条目按 birth 均匀分布，目标快照在中位
    for n in [64u64, 256, 1024, 4096] {
        let entries: Vec<u64> = (0..n).collect();
        let target = n / 2;
        let plain = path2_merge_cost(&entries, target, false, 64);
        for span in [16u64, 64, 256] {
            let bucketed = path2_merge_cost(&entries, target, true, span);
            println!("{}", em.emit_raw(&format!(
                "name=p2 entries={n} bucket_span={span} plain={plain} bucketed={bucketed}")));
        }
    }

    // 路径 4
    for frees in [64u64, 1024, 16384] {
        for per_block in [128u64, 512] {
            let (e, io) = path4_defer_cost(frees, per_block);
            println!("{}", em.emit_raw(&format!(
                "name=p4 immediate_frees={frees} per_block={per_block} defer_entries={e} defer_ios={io}")));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const S: u64 = 100;

    /// **正确规则在三个边界上必须逐条给出 D5 正文规定的答案。**
    #[test]
    fn the_correct_rule_matches_the_documented_verdict_on_every_boundary() {
        let o = path1(Rule::Correct, S);
        assert_eq!(o.agree, 3, "正确规则该在三个边界上全对");
        assert_eq!(o.data_loss, 0);
        assert_eq!(o.leak, 0);
    }

    /// **左边界写严 ⇒ 恰好一次数据丢失，零泄漏。** 方向必须对得上 D5 正文那张表。
    /// ⚠️ 这条是本实验的理由：只验正确规则等于什么都没验。
    #[test]
    fn writing_the_birth_bound_strict_causes_exactly_one_data_loss() {
        let o = path1(Rule::StrictBirth, S);
        assert_eq!(o.data_loss, 1, "birth==S.txg 那一格该判成数据丢失");
        assert_eq!(o.leak, 0, "写严左边界不该造成泄漏");
    }

    /// **右边界写闭 ⇒ 两次泄漏，零数据丢失。**
    ///
    /// ⚠️ **「两次」不是「一次」，这是本实验测出来的**，第一版按 D5 正文那张表
    /// 猜成一次就红了。原因：`birth ≤ S ≤ death` 在 `death==S.txg` 与
    /// `birth==death` 两格上都判成「引用」。
    /// ⇒ **D5 正文把三个边界列成三行，读起来像三处独立的笔误；
    /// 实际上同一个笔误同时命中两行。** 这一条已回写 D5。
    #[test]
    fn writing_the_death_bound_closed_leaks_on_two_boundaries_not_one() {
        let o = path1(Rule::ClosedDeath, S);
        assert_eq!(o.leak, 2, "写闭右边界该在 death==S.txg 与 birth==death 两格上都泄漏");
        assert_eq!(o.data_loss, 0, "写闭右边界不该造成数据丢失");
        assert_eq!(o.agree, 1, "只剩 birth==S.txg 那一格仍然对");
    }

    /// **两种取错的后果必须不同**——D5 正文说它们不对称，模型里也必须不对称。
    #[test]
    fn the_two_wrong_variants_fail_in_opposite_directions() {
        let sb = path1(Rule::StrictBirth, S);
        let cd = path1(Rule::ClosedDeath, S);
        assert!(sb.data_loss > 0 && sb.leak == 0);
        assert!(cd.leak > 0 && cd.data_loss == 0);
    }

    /// **`birth == death` 那一格三条规则里只有写闭右边界会判错**——
    /// 它是「任何快照都不引用」，而 `birth ≤ S ≤ death` 在 birth==death==S 时为真。
    #[test]
    fn the_birth_equals_death_case_is_only_broken_by_the_closed_death_rule() {
        assert!(!Rule::Correct.refs(S, S, S), "birth==death 时正确规则该判不引用");
        assert!(!Rule::StrictBirth.refs(S, S, S));
        assert!(Rule::ClosedDeath.refs(S, S, S), "写闭右边界在 birth==death 时会误判为引用");
    }

    /// **分桶必须真的省事**，否则路径 2 这一维是摆设。
    #[test]
    fn bucketing_examines_strictly_fewer_entries() {
        let entries: Vec<u64> = (0..1024).collect();
        let plain = path2_merge_cost(&entries, 512, false, 64);
        let bucketed = path2_merge_cost(&entries, 512, true, 64);
        assert_eq!(plain, 1024, "不分桶就是逐条看，绝对值钉死");
        assert!(bucketed < plain, "分桶该更省（{bucketed} vs {plain}）");
    }

    /// **桶越大越省，但省的是可跳过的那一半**——绝对值算得出来：
    /// 1024 条、桶宽 64、目标在 512 ⇒ 前 8 个桶整桶跳（各付 1），其余逐条。
    #[test]
    fn bucketed_cost_is_exactly_the_arithmetic() {
        let entries: Vec<u64> = (0..1024).collect();
        // 桶 [0,63],[64,127],...；bucket_hi < 512 的桶有 8 个（到 [448,511]）
        let expect = 8 + (1024 - 512);
        assert_eq!(path2_merge_cost(&entries, 512, true, 64), expect);
    }

    /// **defer 的 I/O 次数是条目数按打包宽度向上取整**，绝对值钉死。
    #[test]
    fn defer_io_count_is_the_ceiling_of_entries_over_pack_width() {
        assert_eq!(path4_defer_cost(1024, 128), (1024, 8));
        assert_eq!(path4_defer_cost(1025, 128), (1025, 9));
        assert_eq!(path4_defer_cost(0, 128), (0, 0));
    }

    /// **打包越宽 I/O 越少，但条目数不变**——写入量与 I/O 次数是两个指标，不许混。
    #[test]
    fn wider_packing_cuts_ios_but_not_entries() {
        let (e1, io1) = path4_defer_cost(16384, 128);
        let (e2, io2) = path4_defer_cost(16384, 512);
        assert_eq!(e1, e2, "条目数与打包宽度无关");
        assert!(io2 < io1, "打包越宽 I/O 越少");
    }
}
