//! E77：发布的持久顺序 —— 单元 / journal 记录 / 根槽之间，哪几道屏障是必要的。
//!
//! ## 为什么要有这个实验
//!
//! 一次发布要写三类东西：COW 单元（数据叶 + 祖先节点）、journal 记录（定长环）、根槽（自证单元）。
//! 全仓对它们之间持久顺序的约束只有两句，且都自陈是推导：
//!
//! - D25（目标负载优先级）逐字：「根槽不许比它指向的东西先持久（一道），fsync 要等根槽持久才返回
//!   （第二道）……**这是推，不是实测**」。
//! - D23（journal 的角色与格式）逐字：「journal 记录不是『事务的全部』，是**已经写到盘上的东西的
//!   发布指令**」——「已经写到盘上」是发出还是持久，没写。
//!
//! 记录与根槽之间的顺序、单元与记录之间要不要一道独立屏障，**全仓零覆盖**（2026-09-02 grep 证实）。
//! 而崩溃点重放 harness 要拿这份顺序当规格——规格不存在，harness 无从写起。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D23（journal 的角色与格式）已定项 1：「取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录」。
//! - D23 已定项 7：「可以跨多条，记录头带事务边界字段」；恢复要「丢掉提交标记还没出现的那个事务的全部记录」。
//! - D23 前缀判定：「判据是 `jsn == expected`，不等即止」。
//! - D22（单元原子性怎么合成）已定三 / 根的读取次序：「必须先逐个验证全部候选、再在有效者中按代号择新」。
//! - D4（校验和位置）：校验和内联进指针 ⇒ 记录点名的每一项自带目标单元的校验和
//!   （D23 已定项 1 逐字「fsync 记录（点名的每项自带校验和）」）。
//!
//! ## 模型
//!
//! 一次发布 = 6 个单元（U0..U5，全部写到新位置）+ 3 条记录（R0 点名 U0,U1；R1 点名 U2,U3；
//! R2 点名 U4,U5 并带提交标记；三条同一个事务）+ 1 个根槽写 S（指向全部 6 个单元，代号 +1）。
//! 上一代根 S0 与旧单元恒在盘上（上次发布的产物）。
//!
//! **屏障语义（FLUSH）**：屏障把写分成段；崩溃状态 = 「前若干段全持久 + 当前段任意子集持久 +
//! 之后段全没持久」。段内自由子集就是设备重排；这是块层唯一承诺的模型
//! （D20（承重面：单元的原子性与自包含）：「块层从未承诺一个 bio 不会被撕裂」）。
//!
//! 四条臂 = 四种屏障摆法：
//!
//! | 臂 | 段 | 含义 |
//! |---|---|---|
//! | `b_all`  | [U][R][S] | 两道屏障全上（单元→记录一道、记录→根槽一道） |
//! | `b_ur`   | [U][R,S]  | 只留「单元→其余」那道；记录与根槽自由重排 |
//! | `b_rs`   | [U,R][S]  | 只留「根槽之前」那道；单元与记录自由重排 |
//! | `b_none` | [U,R,S]   | 一道屏障都不上 |
//!
//! **恢复模型**（按 D22 / D23 已定条款）：先逐个验证根槽候选、按代号择新；
//! 择中新根 ⇒ 走读校验全部 6 个单元（读路径必验校验和）；
//! 择中旧根 ⇒ 重放：取 jsn 严格连续前缀，只施加提交标记齐全的事务，
//! **施加前逐项验证点名单元的校验和**（validating 模式）。
//! 另设 naive 模式（阳性对照）：重放不验点名单元，直接嫁接。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **违例的定义**：① 根槽已持久（fsync 可能已返回）而恢复结果 ≠ 新状态或走读失败；
//!    ② 恢复出「部分事务」状态（既非旧态也非新态）；③ 重放把校验失败的单元嫁接进树。
//!    根槽未持久时旧态与新态都合法（fsync 没返回，两个都是允许的结局）。
//! 2. **崩溃状态数必须等于闭式**：b_all 72、b_ur 79、b_rs 513、b_none 1024——
//!    数错说明枚举本身漏了，整轮作废。
//! 3. **阳性对照必须红**：naive 模式在 b_rs 上违例数必须恰为 63（闭式：根槽缺席 × 三条记录齐 ×
//!    至少一个单元缺席 = 2^6 − 1）。不红说明「验证点名单元」这一步在模型里没参与，整轮作废。
//! 4. 各臂的违例数如实报，**哪几道屏障必要由数字说了算**，不预设结论。
//!
//! ## 失败条款
//!
//! - 判据 2 或 3 不中 ⇒ 整轮作废（模型或对照坏了，不是发现）。
//! - 判据 4 的结果与 D25 那句推导不一致 ⇒ 如实并列，不回头改判据。
//!
//! ## 它答不了的
//!
//! 纯枚举模型，文件操作 0 处。它不回答：真实设备的 FLUSH 是否如宣称生效（那是 C6（块层语义假设写错）
//! 的射程）；FUA 与 FLUSH 的代价差；多次发布交错时环回绕的语义（E78 的射程）。
//! 「段内任意子集」是块层承诺的下界模型——真实设备可能更有序，但不能依赖。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

/// 一次发布里的单元数。6 = 目标负载的量级缩样（D25 是 8 叶 + 4 祖先，取 6 让 2^10 可穷举）。
const UNITS: usize = 6;
/// 记录数。3 条同一事务，测 D23 已定项 7 的「跨多条 + 提交标记」。
const RECORDS: usize = 3;
/// 写 id 布局：0..6 单元，6..9 记录，9 根槽。
const W_ROOT: usize = UNITS + RECORDS;
const TOTAL_WRITES: usize = W_ROOT + 1;

/// 每条记录点名哪两个单元。R2（最后一条）带提交标记。
fn record_names(r: usize) -> [usize; 2] {
    [r * 2, r * 2 + 1]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    BAll,
    BUr,
    BRs,
    BNone,
}

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::BAll => "b_all",
            Arm::BUr => "b_ur",
            Arm::BRs => "b_rs",
            Arm::BNone => "b_none",
        }
    }
    /// 屏障把 0..TOTAL_WRITES 分成的段。没有 `_ =>` 通配臂。
    fn segments(self) -> Vec<Vec<usize>> {
        let units: Vec<usize> = (0..UNITS).collect();
        let records: Vec<usize> = (UNITS..UNITS + RECORDS).collect();
        let root = vec![W_ROOT];
        match self {
            Arm::BAll => vec![units, records, root],
            Arm::BUr => vec![units, [records, root].concat()],
            Arm::BRs => vec![[units, records].concat(), root],
            Arm::BNone => vec![[units, records, root].concat()],
        }
    }
}

/// 枚举一条臂的全部崩溃状态（持久集合的位图）。
/// 屏障语义：段 f 里有任何写持久 ⇒ 段 < f 的全部写持久。
fn crash_states(arm: Arm) -> BTreeSet<u16> {
    let segs = arm.segments();
    let mut states = BTreeSet::new();
    for frontier in 0..segs.len() {
        // 段 < frontier 全持久
        let mut base: u16 = 0;
        for seg in segs.iter().take(frontier) {
            for &w in seg {
                base |= 1 << w;
            }
        }
        // 当前段任意子集
        let cur = &segs[frontier];
        for sub in 0u32..(1 << cur.len()) {
            let mut m = base;
            for (i, &w) in cur.iter().enumerate() {
                if sub & (1 << i) != 0 {
                    m |= 1 << w;
                }
            }
            states.insert(m);
        }
    }
    states
}

fn persisted(state: u16, w: usize) -> bool {
    state & (1 << w) != 0
}

/// 恢复的结局。**三态不够**——「部分事务」必须是独立一格，
/// 把它并进旧态或新态都会把违例藏起来。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    /// 回到上一代（发布整体丢弃）。
    StateOld,
    /// 发布完整生效（择中新根走读全过，或重放完整嫁接成功）。
    StateNew,
    /// 走读撞到校验失败（根指向没持久的单元）。
    BrokenRoot,
    /// 恢复出部分事务，或嫁接了校验失败的单元。
    Corrupt,
}

/// 重放要不要验证点名单元。naive 是阳性对照，不是候选。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Replay {
    Validating,
    Naive,
}

/// 对一个崩溃状态跑恢复，返回结局。
fn recover(state: u16, replay: Replay) -> Outcome {
    // 先逐个验证全部候选、再按代号择新（D22 已定的读取次序）。
    // S 持久 ⇒ 自证校验和过 ⇒ 是合法候选且代号最大。
    if persisted(state, W_ROOT) {
        // 走读：读路径必验校验和；没持久的单元读到旧圈垃圾 ⇒ 校验必失配。
        for u in 0..UNITS {
            if !persisted(state, u) {
                return Outcome::BrokenRoot;
            }
        }
        return Outcome::StateNew;
    }
    // 择中旧根 S0：重放。前缀 = jsn 严格连续（R0 起，断号即止）。
    let mut prefix_len = 0;
    for r in 0..RECORDS {
        if persisted(state, UNITS + r) {
            prefix_len = r + 1;
        } else {
            break; // 断号即止
        }
    }
    // 事务过滤：提交标记在最后一条记录上；前缀不含它 ⇒ 事务不完整，全部丢弃。
    let committed = prefix_len == RECORDS;
    if !committed {
        return Outcome::StateOld;
    }
    // 施加：嫁接全部点名单元。
    match replay {
        Replay::Validating => {
            // 施加前逐项验证点名单元；任何一项失配 ⇒ 整个事务丢弃（回旧态）。
            for r in 0..RECORDS {
                for u in record_names(r) {
                    if !persisted(state, u) {
                        return Outcome::StateOld;
                    }
                }
            }
            Outcome::StateNew
        }
        // 不验证，直接嫁接并**自称成功**。它错没错由 audit 判，不由它自己判——
        // 审计与被审计不许用同一段代码（evidence-discipline.md）。
        Replay::Naive => Outcome::StateNew,
    }
}

/// 独立审计：恢复方**自称**的结局对不对，对照物理真值重判。
/// 自称新态而有单元没持久 ⇒ 树里挂着垃圾 ⇒ 改判 Corrupt。
/// 没有这一步，「重放不验证」的静默损坏会被记成成功（2026-09-02 变异测试当场抓到这个形态）。
fn audit(state: u16, claim: Outcome) -> Outcome {
    if claim == Outcome::StateNew && (0..UNITS).any(|u| !persisted(state, u)) {
        return Outcome::Corrupt;
    }
    claim
}

/// 判一个 (状态, 结局) 是不是违例。判据 1 的可执行形式。
fn is_violation(state: u16, outcome: Outcome) -> bool {
    match outcome {
        Outcome::BrokenRoot | Outcome::Corrupt => true,
        Outcome::StateNew => false,
        // 根槽已持久（fsync 可能已返回）时回旧态 = 丢已承诺的数据。
        Outcome::StateOld => persisted(state, W_ROOT),
    }
}

/// 一条臂 × 一种重放模式的统计。
struct Tally {
    states: usize,
    violations: usize,
    state_new: usize,
    state_old: usize,
    /// 根槽持久而记录缺席的状态数（记录流有洞：核对器与反向链的输入不完整）。
    record_holes: usize,
}

fn run(arm: Arm, replay: Replay) -> Tally {
    let states = crash_states(arm);
    let mut t = Tally { states: states.len(), violations: 0, state_new: 0, state_old: 0, record_holes: 0 };
    for &s in &states {
        let o = audit(s, recover(s, replay));
        if is_violation(s, o) {
            t.violations += 1;
        }
        match o {
            Outcome::StateNew => t.state_new += 1,
            Outcome::StateOld => t.state_old += 1,
            Outcome::BrokenRoot | Outcome::Corrupt => {}
        }
        if persisted(s, W_ROOT) && (0..RECORDS).any(|r| !persisted(s, UNITS + r)) {
            t.record_holes += 1;
        }
    }
    t
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config units={UNITS} records={RECORDS} txn=1 commit_on_last=1 model=exhaustive file_ops=0"
        ))
    );
    for arm in [Arm::BAll, Arm::BUr, Arm::BRs, Arm::BNone] {
        for replay in [Replay::Validating, Replay::Naive] {
            let t = run(arm, replay);
            let mode = match replay {
                Replay::Validating => "validating",
                Replay::Naive => "naive",
            };
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=tally arm={} mode={mode} states={} violations={} state_new={} state_old={} record_holes={}",
                    arm.tag(),
                    t.states,
                    t.violations,
                    t.state_new,
                    t.state_old,
                    t.record_holes
                ))
            );
        }
    }
    // 判据 4 的判决行：validating 模式下违例为 0 的臂集合，就是「够用的屏障摆法」集合。
    let safe: Vec<&str> = [Arm::BAll, Arm::BUr, Arm::BRs, Arm::BNone]
        .into_iter()
        .filter(|&a| run(a, Replay::Validating).violations == 0)
        .map(Arm::tag)
        .collect();
    println!(
        "{}",
        em.emit_raw(&format!("name=verdict safe_arms_validating={}", safe.join(","),))
    );
    // 必要屏障的判定：b_none 违例 > 0 且 b_rs 违例 = 0 ⇒ 「根槽之前一道屏障」是必要且充分的
    // 数据完整性条件；b_ur 与 b_all 的差别只在 record_holes（记录流完整性），不在数据。
    let none_v = run(Arm::BNone, Replay::Validating).violations;
    let rs_v = run(Arm::BRs, Replay::Validating).violations;
    let naive_rs = run(Arm::BRs, Replay::Naive).violations;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=necessity barrier_before_root_necessary={} replay_validation_load_bearing={}",
            u8::from(none_v > 0 && rs_v == 0),
            u8::from(naive_rs > 0)
        ))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**（判据 2）：四条臂的崩溃状态数等于闭式。
    /// b_all: 2^6 + (2^3−1) + (2^1−1) = 72；b_ur: 2^6 + (2^4−1) = 79；
    /// b_rs: 2^9 + 1 = 513；b_none: 2^10 = 1024。
    #[test]
    fn absolute_state_counts() {
        assert_eq!(crash_states(Arm::BAll).len(), 72);
        assert_eq!(crash_states(Arm::BUr).len(), 79);
        assert_eq!(crash_states(Arm::BRs).len(), 513);
        assert_eq!(crash_states(Arm::BNone).len(), 1024);
    }

    /// **绝对值断言**：validating 模式下 b_none 的违例恰为 504
    /// （闭式：根槽持久的 2^9 个状态里，6 单元齐全的只有 2^3 个 ⇒ 512 − 8）。
    #[test]
    fn absolute_none_violations() {
        assert_eq!(run(Arm::BNone, Replay::Validating).violations, 504);
    }

    /// **判据 3 阳性对照**：naive 模式在 b_rs 上违例恰为 63（2^6 − 1）。
    /// 不中 ⇒ 「验证点名单元」这一步没参与，整轮作废。
    #[test]
    fn positive_control_naive_rs_is_63() {
        assert_eq!(run(Arm::BRs, Replay::Naive).violations, 63);
        // 同一臂 validating 模式必须是 0——差值就是那一步验证买到的全部。
        assert_eq!(run(Arm::BRs, Replay::Validating).violations, 0);
    }

    /// b_none 的 naive 违例 = 走读抓的 504 + 重放嫁接的 63。
    #[test]
    fn absolute_none_naive_violations() {
        assert_eq!(run(Arm::BNone, Replay::Naive).violations, 567);
    }

    /// 两道屏障全上（b_all）：违例 0 且记录流无洞。
    #[test]
    fn b_all_is_clean() {
        let t = run(Arm::BAll, Replay::Validating);
        assert_eq!(t.violations, 0);
        assert_eq!(t.record_holes, 0);
    }

    /// b_ur（记录与根槽自由重排）：数据违例 0，但记录流有洞恰 7 个
    /// （根槽在而记录缺的组合 2^3 − 1）。洞不丢数据，丢的是核对器与反向链的输入。
    #[test]
    fn b_ur_holes_are_exactly_7() {
        let t = run(Arm::BUr, Replay::Validating);
        assert_eq!(t.violations, 0);
        assert_eq!(t.record_holes, 7);
    }

    /// b_rs：根槽持久 ⇒ 段 1 全持久 ⇒ 走读必过、记录必齐。
    #[test]
    fn b_rs_no_holes_no_violations() {
        let t = run(Arm::BRs, Replay::Validating);
        assert_eq!(t.violations, 0);
        assert_eq!(t.record_holes, 0);
    }

    /// b_none 的记录洞数：根槽持久 512 个状态 − 记录齐全的 2^6 = 448。
    #[test]
    fn absolute_none_holes() {
        assert_eq!(run(Arm::BNone, Replay::Validating).record_holes, 448);
    }

    /// **提交标记纪律**：记录只落了前两条（无提交标记）⇒ 事务整体丢弃，回旧态。
    #[test]
    fn incomplete_txn_is_dropped() {
        let mut s: u16 = 0;
        for u in 0..UNITS {
            s |= 1 << u;
        }
        s |= 1 << UNITS; // R0
        s |= 1 << (UNITS + 1); // R1，R2（带提交标记）没落
        assert_eq!(recover(s, Replay::Validating), Outcome::StateOld);
        assert_eq!(recover(s, Replay::Naive), Outcome::StateOld);
    }

    /// **断号即止**：R0 缺席而 R1、R2 在场 ⇒ 前缀为空 ⇒ 回旧态。
    /// 没有这一条，带提交标记的尾巴会被当成完整事务施加（部分嫁接）。
    #[test]
    fn gap_stops_prefix() {
        let mut s: u16 = 0;
        for u in 0..UNITS {
            s |= 1 << u;
        }
        s |= 1 << (UNITS + 1); // R1
        s |= 1 << (UNITS + 2); // R2（提交标记在场！）
        assert_eq!(recover(s, Replay::Validating), Outcome::StateOld);
    }

    /// **fsync 语义**：根槽持久而单元缺一个 ⇒ BrokenRoot，且它是违例。
    #[test]
    fn root_over_missing_unit_is_violation() {
        let mut s: u16 = 1 << W_ROOT;
        for u in 1..UNITS {
            s |= 1 << u; // U0 缺席
        }
        for r in 0..RECORDS {
            s |= 1 << (UNITS + r);
        }
        let o = recover(s, Replay::Validating);
        assert_eq!(o, Outcome::BrokenRoot);
        assert!(is_violation(s, o));
    }

    /// **走读的完备性**：6 个单元里任缺哪一个，走读自己（不靠审计兜底）都必须报 BrokenRoot。
    /// 审计会把漏网的改判成 Corrupt，统计上看不出差别——所以这条只能逐单元直接测 recover
    /// （2026-09-02 第二版变异测试实测：M4 被审计掩住，一个统计测试都没红）。
    #[test]
    fn walk_checks_every_unit() {
        for missing in 0..UNITS {
            let mut s: u16 = (1 << TOTAL_WRITES) - 1;
            s &= !(1 << missing);
            assert_eq!(
                recover(s, Replay::Validating),
                Outcome::BrokenRoot,
                "缺 U{missing} 时走读必须自己报警"
            );
        }
    }

    /// 根槽未持久时旧态不是违例（fsync 没返回，丢弃合法）。
    #[test]
    fn old_state_without_root_is_legal() {
        let s: u16 = 0b111111; // 只有单元持久
        let o = recover(s, Replay::Validating);
        assert_eq!(o, Outcome::StateOld);
        assert!(!is_violation(s, o));
    }

    /// 全部持久 ⇒ 新态，且四条臂都包含这个状态。
    #[test]
    fn full_state_recovers_new_everywhere() {
        let full: u16 = (1 << TOTAL_WRITES) - 1;
        assert_eq!(recover(full, Replay::Validating), Outcome::StateNew);
        for arm in [Arm::BAll, Arm::BUr, Arm::BRs, Arm::BNone] {
            assert!(crash_states(arm).contains(&full), "{arm:?}");
        }
    }

    /// 判决的两半：屏障必要性与重放验证承重性。
    /// 这是 name=necessity 那一行的可判定形式。
    #[test]
    fn necessity_verdict() {
        assert!(run(Arm::BNone, Replay::Validating).violations > 0, "不上屏障必须出违例");
        assert_eq!(run(Arm::BRs, Replay::Validating).violations, 0, "只留根槽前一道就够（数据侧）");
        assert!(run(Arm::BRs, Replay::Naive).violations > 0, "重放不验证就出违例");
    }

    /// 枚举去重：b_none 的状态集必须恰好是全体子集（无重复、无遗漏）。
    #[test]
    fn none_enumerates_all_subsets() {
        let s = crash_states(Arm::BNone);
        assert_eq!(s.len(), 1 << TOTAL_WRITES);
        assert!(s.contains(&0));
    }

    /// **审计的判别力**：自称新态而单元缺席 ⇒ 必须被改判 Corrupt；齐全 ⇒ 不改判。
    /// 审计与被审计不共享判定——没有这一步，「重放不验证」的静默损坏会被记成成功
    /// （2026-09-02 第一版变异测试实测：M2 一个测试都没红，正是这个形态）。
    #[test]
    fn audit_reclassifies_false_success() {
        let missing_one: u16 = 0b111110; // U0 缺席，其余单元在
        assert_eq!(audit(missing_one, Outcome::StateNew), Outcome::Corrupt);
        let all_units: u16 = 0b111111;
        assert_eq!(audit(all_units, Outcome::StateNew), Outcome::StateNew);
        assert_eq!(audit(missing_one, Outcome::StateOld), Outcome::StateOld, "旧态不经审计改判");
    }

    /// **fsync 承诺检查器自身的判别力**：根槽持久 + 回旧态必须算违例。
    /// 正确的 recover 走不到这一格（根槽持久必走走读），所以必须直接测检查器——
    /// 一条走不到的检查与没有这条检查，在统计里长得一模一样
    /// （2026-09-02 第一版变异测试实测：M6 一个测试都没红）。
    #[test]
    fn fsync_promise_checker_has_teeth() {
        let s: u16 = 1 << W_ROOT;
        assert!(is_violation(s, Outcome::StateOld), "根槽持久时回旧态 = 丢已承诺的数据");
        assert!(!is_violation(0, Outcome::StateOld), "根槽未持久时回旧态合法");
    }
}
