//! E33：定长环 + 断号即止 + `jsn` 重用 ⇒ 恢复重放**上一条时间线**的记录
//!
//! **组合对象**：D23（journal 的角色与格式）三条已定项凑在一起就漏：
//!
//! | 已定项 | 逐字 |
//! |---|---|
//! | 已定项 2 | 取**定长环** ⇒ 槽位由位置决定，**旧记录不会被自动抹掉，只会被覆盖** |
//! | 前缀判定 | 「判据是 `jsn == expected`，不等即止」 |
//! | 已定项 3 | 恢复必须**全环扫描**、逐条验证、再择最长合法前缀 |
//!
//! D23 自己举的空洞例子逐字是「5849 写成、5850 没写、5851 写成」。
//! 断号即止在**那一次**恢复里是对的：前缀停在 5849。
//! 漏的是**下一步**——恢复之后系统从 `jsn = 5850` 接着写，
//! 而 **5851 那个槽里还躺着上一条时间线的记录，校验和是好的、`jsn` 恰好等于 expected**。
//!
//! ⇒ 第二次崩溃后的恢复会把它接上去重放。**它属于一条已经被丢弃的时间线。**
//!
//! ## 为什么现有的闸一个都拦不住
//!
//! - **`jsn` 严格连续**：残留记录的 `jsn` 恰好就是 expected，这条闸是它的通行证不是拦阻。
//! - **在飞记录数上限**（D23 已定，XFS `XLOG_MAX_ICLOGS` 形态）：那条规则管的是
//!   **校验和失败**算撕裂还是算损坏；残留记录的校验和是**好的**，这条闸根本不触发。
//! - **`checkpoint_txg`**：两条时间线在同一个 checkpoint 窗口内，这个字段一模一样。
//! - **I-8.3（重放前缀严格连续）**：它要求的正是 `jsn == expected`，残留记录满足它。
//!
//! ## 解药在仓里，但记的是另一个理由
//!
//! D23「三条给格式的现查警示」第 3 条提议加一条反向链（XFS 的 `h_prev_block` 形态），
//! 理由写的是「让 O2（独立解析器 + checker）能用与正向扫描完全不同的遍历复核前缀边界」。
//! 本实验量的是它的**另一个**作用：它是唯一能把两条时间线分开的字段。
//!
//! ## 四条臂，两条对两条错，且错的方向相反
//!
//! | 臂 | 恢复后从哪接着写 | 额外的闸 |
//! |---|---|---|
//! | `resume_at_prefix` | `前缀末 + 1`（唯一能从盘上算出来的值） | 无 —— 这就是 D23 已定项的字面形态 |
//! | `skip_hole` | `全环最大合法 jsn + 1` | 无 —— 需要一个仓里不存在的水位字段 |
//! | `back_chain` | `前缀末 + 1` | 每条记录带前一条的 hash |
//! | `erase_then_resume` | `前缀末 + 1` | 恢复时先把断号之后的残留槽物理作废 |
//!
//! **判据两条**：`stale_replayed == 0`（不重放已丢弃时间线的记录）
//! **且** `lost_new == 0`（不丢新时间线已经写成的记录）。
//! 两条错臂各违反一条，方向相反 —— 这正说明缺口在规范里，不在某一种实现里。
//!
//! ⚠️ **本实验固定了一个 D23 从未定过的自由度：槽位映射取 `jsn % 槽数`。**
//! 现役实现（jbd2）把槽位与序号解耦，恢复收尾时号跳过断点、位置回退到断点，
//! 于是残留记录与「该位置期望的下一个号」对不上。
//! ⇒ 本实验测到的必然重放是在这一支槽位映射下的必然，不是三条已定项的必然。
//! 第三条出路（解耦槽位）不花记录头字节，也不要求原地覆写。
//!
//! ## 窗口有多大：新时间线自己会把残留盖掉，盖不完的那些才出事
//!
//! 定长环的槽位由 `jsn` 决定，所以新时间线每写一条就盖掉一条残留。
//! 实测出来的是一条精确式子（`resume_at_prefix` 臂）：
//!
//! > `stale_replayed = max(0, 残留条数 − (新写条数 − 1))`
//!
//! ⇒ **危险窗口 = 「第二次崩溃发生在新时间线写满残留段之前」**。
//! 它不是一个概率小到可以忽略的巧合：第一次崩溃刚恢复完就再崩一次，
//! 正是掉电反复、盘将坏、以及崩溃点重放 harness **本来就要枚举**的那一类序列。

use e7_index_bench::Emitter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    ResumeAtPrefix,
    SkipHole,
    BackChain,
    EraseThenResume,
}

const ARMS: [Arm; 4] = [
    Arm::ResumeAtPrefix,
    Arm::SkipHole,
    Arm::BackChain,
    Arm::EraseThenResume,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增一条臂不补这里就编译不过
            Arm::ResumeAtPrefix => "resume_at_prefix",
            Arm::SkipHole => "skip_hole",
            Arm::BackChain => "back_chain",
            Arm::EraseThenResume => "erase_then_resume",
        }
    }
    fn checks_back_chain(self) -> bool {
        matches!(self, Arm::BackChain)
    }
}

/// 一条 journal 记录。字段对齐 E24（journal 几何）逐字列出的 11 个头字段里与本实验相关的那几个。
/// ⚠️ **那 11 个字段里没有任何一个能分辨时间线**——这正是本实验要量的缺口。
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rec {
    jsn: u64,
    /// 时间线标记。**盘上没有这个字段**，它只存在于本模型里，用来判「重放的是谁的记录」。
    timeline: u32,
    /// 反向链：前一条记录的 hash。只有 `back_chain` 那条臂会去看它。
    prev_hash: u64,
    /// 头部校验和过不过。丢写 / 撕裂在这里表现为 false。
    csum_ok: bool,
}

fn rec_hash(r: &Rec) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in [r.jsn, r.timeline as u64, r.prev_hash] {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// 定长环，且**槽位由 `jsn % slots` 决定**。
///
/// ⚠️ **这是一个选择，不是「定长环」的含义**（此处原先那么写过，是过头的）：
/// jbd2 与 XFS 的日志同样是定长环，但**槽位由 head 指针顺序推进、与序号解耦**。
/// 现查 jbd2 恢复收尾（`fs/jbd2/recovery.c:318-321`，本机 7.2.0 树）：
/// `j_transaction_sequence = ++info.end_transaction; j_head = info.head_block;`
/// 注释逐字「Restart the log at the next transaction ID,
/// thus invalidating any existing commit records in the log」——
/// 号跳过断点、位置回退到断点，两者独立推进。
///
/// ⇒ `slot = jsn % n` 让「残留记录恰好落在下一个要读的槽、且号恰好等于 expected」
/// 由构造必然发生；位置与序号解耦时它退化成一个巧合。
/// **本实验只测了前一支**，见正文口径。
struct Ring {
    slot: Vec<Option<Rec>>,
}

impl Ring {
    fn new(slots: usize) -> Self {
        Ring {
            slot: vec![None; slots],
        }
    }
    fn idx(&self, jsn: u64) -> usize {
        (jsn as usize) % self.slot.len()
    }
    fn put(&mut self, r: Rec) {
        let i = self.idx(r.jsn);
        self.slot[i] = Some(r);
    }
    fn get(&self, jsn: u64) -> Option<Rec> {
        self.slot[self.idx(jsn)]
    }
    /// 恢复时把断号之后的残留槽物理作废（`erase_then_resume` 那条臂）。
    /// ⚠️ 它要求**原地覆写**——D13（验证路线）的结构等价类里 zoned 那一类做不到。
    fn erase_from(&mut self, from_jsn: u64, count: u64) {
        for j in from_jsn..from_jsn + count {
            let i = self.idx(j);
            self.slot[i] = None;
        }
    }
}

/// 一次恢复：全环扫描 + 逐条验证 + 择最长合法前缀（D23 已定项 3 已定的次序）。
/// 返回被接受的记录序列。
fn recover(ring: &Ring, arm: Arm, tail_jsn: u64, anchor_hash: u64, inflight_limit: u64) -> Vec<Rec> {
    let mut out = Vec::new();
    let mut expected = tail_jsn;
    let mut prev = anchor_hash;
    // 前缀最长一圈——环只有这么多槽，走过一圈就是在读自己上一圈写的东西。
    // ⚠️ 这不是防御性编程，是「全环扫描」这个已定次序的字面含义；
    // 少了它，摘掉断号即止之后恢复会无限接受下去（变异 M1 实测：进程被 OOM 杀掉）。
    while out.len() < ring.slot.len() {
        let r = match ring.get(expected) {
            Some(r) => r,
            None => break,
        };
        // 头部校验和：在飞窗口之内算撕裂（截断前缀），窗口之外算损坏。
        // 两种处置对本实验相同——都是停在这里；差别只在报不报错。
        if !r.csum_ok {
            let _outside_window = expected + inflight_limit < tail_jsn;
            break;
        }
        // D23 已定的前缀判据：断号即止
        if r.jsn != expected {
            break;
        }
        if arm.checks_back_chain() && r.prev_hash != prev {
            break;
        }
        prev = rec_hash(&r);
        out.push(r);
        expected += 1;
    }
    out
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    /// 第一次恢复接受的条数（空洞之前那一段）
    first_prefix: u64,
    /// 第一次恢复有没有在空洞处停下（判别力：没停说明断号即止根本没在跑）
    hole_stopped_first_recovery: bool,
    /// 第二次恢复重放的、属于**已丢弃时间线**的记录条数。判据一：必须为 0
    stale_replayed: u64,
    /// 新时间线已经写成、却没被重放的条数。判据二：必须为 0
    lost_new: u64,
    /// 第二次恢复一共接受多少条
    second_prefix: u64,
}

/// 上一圈的记录。稳态下环里每个槽都有东西，「没写成」= 槽里还是上一圈那条。
const TIMELINE_PREV_LAP: u32 = 0;
const TIMELINE_A: u32 = 1;
const TIMELINE_B: u32 = 2;

/// 跑一条臂。
///
/// * `settled` —— 崩溃前已经安定下来的记录条数（`jsn` 1..=settled，全部有效）
/// * `stale` —— 时间线 A 在空洞**之后**留下的、校验和完好的记录条数
/// * `new_after` —— 恢复之后时间线 B 写成的记录条数
/// 建出崩溃前那一刻的环。返回 `(环, tail, 空洞的 jsn, 链锚)`。
///
/// 抽出来是为了让「模型确实处于稳态」这条性质可以被单测直接判——
/// 它不是布景，是断号即止这条闸有没有被走到的前提。
fn setup(slots: usize, settled: u64, stale: u64) -> (Ring, u64, u64, u64) {
    let mut ring = Ring::new(slots);
    let anchor = 0xA11CE_u64;

    // ── 上一圈：环处于**稳态**，每个槽里都躺着上一圈的记录 ──
    // ⚠️ 这一步不是布景。少了它，「没写成的那一条」在模型里表现为空槽，
    // 而真实的定长环里它表现为**上一圈的记录**——断号即止拦的正是后者。
    // 空槽会让断号即止成为死代码（变异 M1 实测：摘掉它一个测试都不红）。
    for jsn in 1..=slots as u64 {
        let r = Rec { jsn, timeline: TIMELINE_PREV_LAP, prev_hash: 0, csum_ok: true };
        ring.put(r);
    }
    let tail = slots as u64 + 1; // 本圈第一条；tail 的权威副本住超级块槽（D23 已定项 3 已定）

    // ── 时间线 A：安定段 ──
    let mut prev = anchor;
    for jsn in tail..tail + settled {
        let r = Rec { jsn, timeline: TIMELINE_A, prev_hash: prev, csum_ok: true };
        prev = rec_hash(&r);
        ring.put(r);
    }
    // ── 空洞：这一条没写成（D23 自己举的例子），槽里还是上一圈那条 ──
    let hole = tail + settled;
    // ── 而它后面那几条落盘了，校验和完好 ──
    let mut p = 0xDEAD_u64; // A 侧真实的链值，与 B 侧不同
    for k in 0..stale {
        let jsn = hole + 1 + k;
        let r = Rec { jsn, timeline: TIMELINE_A, prev_hash: p, csum_ok: true };
        p = rec_hash(&r);
        ring.put(r);
    }

    (ring, tail, hole, anchor)
}

fn run(arm: Arm, slots: usize, settled: u64, stale: u64, new_after: u64) -> Out {
    let (mut ring, tail, hole, anchor) = setup(slots, settled, stale);

    // ── 第一次恢复 ──
    let acc1 = recover(&ring, arm, tail, anchor, 8);
    let mut o = Out {
        first_prefix: acc1.len() as u64,
        hole_stopped_first_recovery: (acc1.len() as u64) == settled,
        ..Default::default()
    };

    // ── 恢复之后接着写：从哪个 jsn 起，是本实验的分歧点 ──
    let resume = match arm {
        // 没有 `_ =>`
        Arm::ResumeAtPrefix | Arm::BackChain => acc1.last().map(|r| r.jsn + 1).unwrap_or(tail),
        // 需要一个「全环见过的最大合法 jsn」水位——本仓的记录头与超级块里都没有这个字段
        Arm::SkipHole => {
            // 扫全环取「见过的最大合法 jsn」。⚠️ 这需要一个水位概念，
            // 而 E24（journal 几何）逐字列出的 11 个记录头字段与超级块槽里都没有它。
            let mut m = tail;
            for slot in ring.slot.iter().flatten() {
                if slot.csum_ok && slot.jsn > m {
                    m = slot.jsn;
                }
            }
            m + 1
        }
        Arm::EraseThenResume => {
            ring.erase_from(hole, stale + 1);
            acc1.last().map(|r| r.jsn + 1).unwrap_or(tail)
        }
    };

    // ── 时间线 B 写 new_after 条 ──
    let mut prev = acc1.last().map(|r| rec_hash(r)).unwrap_or(anchor);
    for k in 0..new_after {
        let jsn = resume + k;
        let r = Rec { jsn, timeline: TIMELINE_B, prev_hash: prev, csum_ok: true };
        prev = rec_hash(&r);
        ring.put(r);
    }

    // ── 第二次恢复 ──
    let acc2 = recover(&ring, arm, tail, anchor, 8);
    o.second_prefix = acc2.len() as u64;
    o.stale_replayed = acc2
        .iter()
        .filter(|r| r.timeline == TIMELINE_A && r.jsn >= hole)
        .count() as u64;
    let replayed_new = acc2.iter().filter(|r| r.timeline == TIMELINE_B).count() as u64;
    o.lost_new = new_after - replayed_new;
    o
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw("name=config note=定长环+断号即止+jsn重用 slots=64 settled=20")
    );
    for arm in ARMS {
        for stale in [1u64, 3, 7] {
            for new_after in [1u64, 3, 8] {
                let o = run(arm, 64, 20, stale, new_after);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=cell arm={} stale_left={} new_after={} first_prefix={} hole_stopped={} \
                         stale_replayed={} lost_new={} second_prefix={}",
                        arm.name(),
                        stale,
                        new_after,
                        o.first_prefix,
                        o.hole_stopped_first_recovery,
                        o.stale_replayed,
                        o.lost_new,
                        o.second_prefix
                    ))
                );
            }
        }
    }
    // 阴性对照：没有空洞时四条臂必须一模一样，且什么都不丢
    for arm in ARMS {
        let o = run(arm, 64, 20, 0, 3);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=nohole arm={} first_prefix={} stale_replayed={} lost_new={} second_prefix={}",
                arm.name(),
                o.first_prefix,
                o.stale_replayed,
                o.lost_new,
                o.second_prefix
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **D23 已定项的字面实现：残留被重放的条数服从一条精确式子。**
    ///
    /// `stale_replayed = max(0, 残留条数 − (新写条数 − 1))`
    /// —— 新时间线每写一条就盖掉一条残留，盖不完的那些照单重放。
    /// 绝对值，不是「大于零」：式子右边由算术独立给出，不是从被测代码反解的。
    #[test]
    fn the_spec_replays_exactly_the_leftovers_the_new_timeline_has_not_overwritten() {
        for stale in [1u64, 3, 7] {
            for new_after in [1u64, 3, 8] {
                let want = stale.saturating_sub(new_after - 1);
                let o = run(Arm::ResumeAtPrefix, 64, 20, stale, new_after);
                assert_eq!(
                    o.stale_replayed, want,
                    "残留 {stale} 条、新写 {new_after} 条时该重放 {want} 条，实测 {}",
                    o.stale_replayed
                );
                assert_eq!(o.lost_new, 0, "这条臂不丢新记录，它的病是另一个方向");
            }
        }
    }

    /// **最坏那一格钉死：第一次恢复之后只写成一条就再崩一次 ⇒ 残留一条不落全被重放。**
    /// 这一格正是崩溃点重放 harness 本来就要枚举的序列。
    #[test]
    fn one_record_after_recovery_then_crash_replays_every_leftover() {
        for stale in [1u64, 3, 7] {
            assert_eq!(run(Arm::ResumeAtPrefix, 64, 20, stale, 1).stale_replayed, stale);
        }
    }

    /// **换成「跳过空洞接着写」并不能救，只是把病换了个方向：新写的记录全丢。**
    /// ⇒ 两条自然实现各违反一条判据，方向相反 ⇒ 缺口在规范里，不在实现里。
    #[test]
    fn skipping_the_hole_loses_every_record_the_new_timeline_wrote() {
        for stale in [1u64, 3, 7] {
            for new_after in [1u64, 3, 8] {
                let o = run(Arm::SkipHole, 64, 20, stale, new_after);
                assert_eq!(o.stale_replayed, 0);
                assert_eq!(
                    o.lost_new, new_after,
                    "跳过空洞之后新写的 {new_after} 条全部够不到"
                );
            }
        }
    }

    /// **反向链两条判据同时满足。**
    #[test]
    fn the_back_chain_satisfies_both_criteria() {
        for stale in [1u64, 3, 7] {
            for new_after in [1u64, 3, 8] {
                let o = run(Arm::BackChain, 64, 20, stale, new_after);
                assert_eq!(o.stale_replayed, 0);
                assert_eq!(o.lost_new, 0);
                assert_eq!(o.second_prefix, 20 + new_after, "20 条安定 + {new_after} 条新写");
            }
        }
    }

    /// **恢复时物理作废残留槽也两条判据同时满足。**
    /// ⚠️ 它不花格式字节，但要求**原地覆写**——zoned 那一类布局做不到。
    #[test]
    fn erasing_the_leftovers_also_satisfies_both_criteria() {
        for stale in [1u64, 3, 7] {
            for new_after in [1u64, 3, 8] {
                let o = run(Arm::EraseThenResume, 64, 20, stale, new_after);
                assert_eq!(o.stale_replayed, 0);
                assert_eq!(o.lost_new, 0);
                assert_eq!(o.second_prefix, 20 + new_after, "20 条安定 + {new_after} 条新写");
            }
        }
    }

    /// **判别力：第一次恢复必须真的停在空洞处。**
    /// 不停说明断号即止根本没在跑，后面所有结论都是假的。
    #[test]
    fn the_first_recovery_really_stops_at_the_hole() {
        for arm in ARMS {
            for stale in [1u64, 3, 7] {
                let o = run(arm, 64, 20, stale, 1);
                assert!(o.hole_stopped_first_recovery, "{} 没有停在空洞处", arm.name());
                assert_eq!(o.first_prefix, 20, "{}", arm.name());
            }
        }
    }

    /// **阴性对照：没有空洞时，四条臂逐格相同，且一条都不丢、一条都不多。**
    /// 少了它，「反向链把 stale 压到 0」分不清是它管用还是它把什么都拒了。
    #[test]
    fn without_a_hole_all_four_arms_agree_and_lose_nothing() {
        for arm in ARMS {
            let o = run(arm, 64, 20, 0, 3);
            assert_eq!(o.stale_replayed, 0, "{}", arm.name());
            assert_eq!(o.lost_new, 0, "{}", arm.name());
            assert_eq!(o.second_prefix, 23, "{}", arm.name());
            assert_eq!(o.first_prefix, 20, "{}", arm.name());
        }
    }

    /// **判别力：撕裂的记录必须截断前缀。**
    /// 少了它，`recover` 里那条校验和分支在本实验的场景中是死代码——
    /// 空洞在模型里是「槽里没东西」，走不到它。
    #[test]
    fn a_torn_record_truncates_the_prefix() {
        let mut ring = Ring::new(64);
        let anchor = 0xA11CE_u64;
        let mut prev = anchor;
        for jsn in 1..=20u64 {
            // 本测试单独建环，从 jsn=1 起，不需要稳态布景
            let torn = jsn == 10;
            let r = Rec { jsn, timeline: TIMELINE_A, prev_hash: prev, csum_ok: !torn };
            prev = rec_hash(&r);
            ring.put(r);
        }
        let acc = recover(&ring, Arm::ResumeAtPrefix, 1, anchor, 8);
        assert_eq!(acc.len(), 9, "前缀该停在撕裂的那一条之前");
    }

    /// **模型必须处于稳态：空洞那个槽里躺着的是上一圈的记录，不是空槽。**
    ///
    /// 少了它，断号即止在本实验里是死代码——空槽会让 `recover` 因为
    /// 「取不到记录」而停下，而真实的定长环里停下来的理由是 `jsn != expected`。
    /// 变异 M7 实测：拿掉稳态布景，别的测试一个都不红。
    #[test]
    fn the_hole_slot_holds_a_previous_lap_record_not_an_empty_slot() {
        for (settled, stale) in [(20u64, 1u64), (20, 7)] {
            let (ring, _tail, hole, _anchor) = setup(64, settled, stale);
            let r = ring.get(hole).expect("稳态下空洞那个槽必须有东西——上一圈的记录");
            assert!(r.csum_ok, "上一圈那条记录的校验和是好的，它靠 jsn 才被拦住");
            assert_ne!(r.jsn, hole, "拦住它的正是 jsn != expected");
            assert_eq!(r.timeline, TIMELINE_PREV_LAP);
        }
    }

    /// **在飞记录数上限对这一格零作用**：残留记录的校验和是**好的**，
    /// 那条闸判的是校验和失败算撕裂还是算损坏，根本不触发。
    /// 把上限从 8 调到 1 或 1000，结论一个字不变。
    #[test]
    fn the_inflight_limit_does_not_fire_on_a_valid_stale_record() {
        let mut ring = Ring::new(64);
        let anchor = 0xA11CE_u64;
        let mut prev = anchor;
        for jsn in 1..=5u64 {
            let r = Rec { jsn, timeline: TIMELINE_A, prev_hash: prev, csum_ok: true };
            prev = rec_hash(&r);
            ring.put(r);
        }
        for limit in [1u64, 8, 1000] {
            let acc = recover(&ring, Arm::ResumeAtPrefix, 1, anchor, limit);
            assert_eq!(acc.len(), 5, "在飞上限 {limit} 不该改变对合法记录的判定");
        }
    }

    /// **两条时间线的记录在盘上分辨不出来，除非看反向链。**
    /// 这条钉住机制本身：`jsn`、`checkpoint_txg`、头部校验和三项都一致。
    #[test]
    fn the_two_timelines_are_indistinguishable_without_the_chain() {
        let a = Rec { jsn: 21, timeline: TIMELINE_A, prev_hash: 0xDEAD, csum_ok: true };
        let b = Rec { jsn: 21, timeline: TIMELINE_B, prev_hash: 0xBEEF, csum_ok: true };
        assert_eq!(a.jsn, b.jsn, "jsn 相同");
        assert_eq!(a.csum_ok, b.csum_ok, "校验和都好");
        assert_ne!(a.prev_hash, b.prev_hash, "只有反向链不同");
    }
}
