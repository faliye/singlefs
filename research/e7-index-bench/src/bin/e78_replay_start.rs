//! E78：重放的起点 —— 陈旧 tail 叠上块重用窗口之后，「全环扫描 + 断号即止 + 幂等」还够不够。
//!
//! ## 为什么要有这个实验
//!
//! D23（journal 的角色与格式）已定项 3 定了「恢复不许先信 tail，必须先全环扫描、逐条验证，
//! 再择最长合法前缀」，并由 E24（恢复算法：先信 tail 会不会丢记录）逼出「重放必须幂等」——
//! 那时的结论是「陈旧 tail 只是多做功」。**E24 没建模块重用**：
//! I-7.4（近 K 代块未被复用）只扣最近 K 代根引用的块（K ≤ 根环深度，几十个量级），
//! 而 journal 环装得下几千到几万条记录（E51（反向链的碰撞机会有多少次）：10 MiB 环 = 20480 条）
//! ⇒ 陈旧 tail 与最新根之间的记录，其点名块可以已经被**合法地**释放并复用。
//! 重放这些记录时逐项验证（E77（发布的持久顺序）刚证明验证是承重步骤）会撞上校验失配——
//! **在一个完全健康的镜像上**。「多做功」在这个叠加下变成什么，仓里没人算过。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D23 已定项 3：「必须先全环扫描、逐条验证，再择最长合法前缀」；
//!   「tail 只在 checkpoint 持久之后前移」。
//! - I-8.4（重放幂等）：「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」——
//!   ⚠️ 它管「同一条记录重复施加」，**不管「旧记录施加在新状态上」**，两者不是一回事。
//! - I-7.4（近 K 代块未被复用）：「最近 K 代根所引用的块，其物理范围均未被重新分配」——
//!   K ≤ 根环深度。**环里的记录数远大于 K**，这是本实验的成立条件。
//! - E77（发布的持久顺序）：重放施加前必须验证点名单元，否则 b_rs 臂 63 处静默损坏。
//!
//! ## 模型
//!
//! L 个逻辑对象轮转重写（COW）：发布 p 重写对象 `p mod L`，给它分配新块、把旧块送进
//! defer 队列；defer 满 PIN 个发布后进空闲池（FIFO），分配器优先取池里的块（对陈旧重放最坏）。
//! 每次发布写一条记录（jsn = p，点名 (对象, 新块, 内容标签)）并发布根（映射快照 + 水位 jsn = p）。
//! tail 每 TAIL_EVERY 个发布持久一次。跑到 M 后崩溃：
//! healthy 场景一切齐整；torn 场景多一条 M+1 的记录（其单元没落盘）。
//!
//! **稳态下的闭式**（FIFO 池，一进一出）：记录 p 点名的块在发布 p+L 被释放、p+L+PIN 被复用
//! ⇒ 重放窗口 (tail, M] 里失配的记录数 = max(0, M − L − PIN − tail)。
//!
//! 四条恢复算法（前三条都从 tail 起走严格连续前缀）：
//!
//! | 算法 | 形态 | 施加时验证 | 失配时 |
//! |---|---|---|---|
//! | A1 | E24 形态 + E77 的验证 | 是 | **中止恢复**（把失配当损坏） |
//! | A2 | 同上，跳过式 | 是 | **跳过继续** |
//! | B  | 根带 jsn 水位，只放 > 水位的 | 是 | 落在在飞窗口内 ⇒ 判撕裂、丢弃 |
//! | C  | 盲放全前缀 + 收尾走读 + 尾删重试 | 否（收尾验证） | 走读失败 ⇒ 删掉最尾一条事务重来 |
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **失配数必须钉住闭式**：healthy 下 A 系观察到的失配数 == max(0, M−L−PIN−tail)，
//!    并与独立的重分配追踪器（不共享重放代码）逐格相等；手算锚点两个：
//!    (M=13, L=4, PIN=1, tail=8) ⇒ 0；(M=23, L=4, PIN=1, tail=16) ⇒ 2。对不上整轮作废。
//! 2. **阳性对照**：闭式 > 0 的格上 A1 必须中止（健康镜像判损坏）。不中止说明模型没有判别力，作废。
//! 3. **撕裂判别力**：torn 场景下 B 与 C 必须旗标 torn=1 且终态 = M；A2 的 torn 旗标必须为 0
//!    （静默吞掉）——这是它的实测缺陷，如实记，不是模型故障。
//! 4. **终态审计**：凡是宣称完成的恢复，终态映射与真值逐格比对；不等 ⇒ 该算法出局。
//!    审计不共享任何恢复代码。
//! 5. 代价（重放条数、重试轮数、新格式字段）如实报，选哪条是决策不是实验。
//!
//! ## 失败条款
//!
//! - 判据 1 或 2 不中 ⇒ 整轮作废（模型坏了，不是发现）。
//! - 「freed ⇒ 该对象在窗口内必有更晚记录」这条构造不变量若被违反（断言），
//!   C 的尾删会误删好记录 ⇒ 模型与结论一起作废重建。
//!
//! ## 它答不了的
//!
//! 纯计数模型，文件操作 0 处。它不回答：水位字段选 jsn 还是 checkpoint_txg
//! （**若每次发布都推进 checkpoint_txg，现有的记录头字段就够当水位；若一个 txg 罩多次发布，
//! 记录与根的 txg 相等时有歧义**——那是「发布计数器与 checkpoint_txg 是否恒等」的决策，
//! 本实验只证明「必须有一个水位」）；也不回答真实负载下 tail 能陈旧到什么程度。

use e7_index_bench::Emitter;
use std::collections::VecDeque;

/// 逻辑对象数。
const L: u64 = 4;
/// journal 环槽数。窗口必须 ≤ RING，否则残留是 E32（上一条时间线的残留）的射程，不是本实验的。
const RING: u64 = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Record {
    jsn: u64,
    object: u64,
    block: u64,
    /// 内容标签 = 写出它的发布号。盘上内容与它不等 ⇒ 校验失配。
    tag: u64,
}

/// 盘 + 环 + 根的整体状态（发布到 M 之后）。
struct World {
    /// content[block] = 最后写进该块的内容标签。
    content: Vec<u64>,
    ring: Vec<Option<Record>>,
    /// 最新根：对象 → 块 的映射快照 + 水位。
    root_map: Vec<u64>,
    root_watermark: u64,
    tail: u64,
    m: u64,
    /// 独立的重分配追踪：block → 它被重新分配出去时的发布号列表。
    /// **只给审计与闭式对照用，不给任何恢复算法用。**
    realloc_at: Vec<Vec<u64>>,
    /// torn 场景：环里多一条 M+1 的记录，其单元没落盘。
    torn_record: Option<Record>,
}

fn build_world(m: u64, pin: u64, tail_every: u64, torn: bool) -> World {
    assert!(m - (m / tail_every) * tail_every < RING, "窗口必须落在环内");
    let mut content: Vec<u64> = vec![0; 4096];
    let mut ring: Vec<Option<Record>> = vec![None; RING as usize];
    let mut map: Vec<u64> = (0..L).collect(); // 对象 i 初始占块 i，内容标签 0
    let mut next_fresh = L;
    let mut defer: VecDeque<(u64, u64)> = VecDeque::new(); // (freed_at, block)
    let mut pool: VecDeque<u64> = VecDeque::new();
    let mut realloc_at: Vec<Vec<u64>> = vec![Vec::new(); 4096];

    for p in 1..=m {
        // defer 到期进池（I-7.4 的 PIN 窗口）
        while let Some(&(f, b)) = defer.front() {
            if f + pin <= p {
                pool.push_back(b);
                defer.pop_front();
            } else {
                break;
            }
        }
        let o = (p % L) as usize;
        let nb = if let Some(b) = pool.pop_front() {
            realloc_at[b as usize].push(p);
            b
        } else {
            next_fresh += 1;
            next_fresh - 1
        };
        content[nb as usize] = p;
        defer.push_back((p, map[o]));
        map[o] = nb;
        ring[(p % RING) as usize] = Some(Record { jsn: p, object: o as u64, block: nb, tag: p });
    }
    let torn_record = if torn {
        let p = m + 1;
        let o = (p % L) as usize;
        // 记录落了环，单元没落盘：点一个池外新块，内容保持旧垃圾（0xdead 标签用 u64::MAX 代替）。
        let nb = next_fresh;
        let r = Record { jsn: p, object: o as u64, block: nb, tag: p };
        ring[(p % RING) as usize] = Some(r);
        Some(r)
    } else {
        None
    };
    World {
        content,
        ring,
        root_map: map,
        root_watermark: m,
        tail: (m / tail_every) * tail_every,
        m,
        realloc_at,
        torn_record,
    }
}

/// 恢复的读数。全部如实报，判据在测试里。
#[derive(Default, Debug, PartialEq, Eq)]
struct Run {
    completed: bool,
    aborted: bool,
    replayed: u64,
    mismatches: u64,
    skipped: u64,
    retries: u64,
    torn_flagged: bool,
    /// 终态映射（完成时）。
    final_map: Vec<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Algo {
    A1AbortOnMismatch,
    A2SkipOnMismatch,
    BWatermark,
    CBlindThenWalk,
}

impl Algo {
    fn tag(self) -> &'static str {
        match self {
            Algo::A1AbortOnMismatch => "a1_abort",
            Algo::A2SkipOnMismatch => "a2_skip",
            Algo::BWatermark => "b_watermark",
            Algo::CBlindThenWalk => "c_blind_walk",
        }
    }
}

/// 取从 `start`（不含）起的 jsn 严格连续前缀（断号即止，D23 已定的前缀判定）。
fn continuous_prefix(w: &World, start: u64) -> Vec<Record> {
    let mut out = Vec::new();
    let mut expect = start + 1;
    loop {
        match w.ring[(expect % RING) as usize] {
            Some(r) if r.jsn == expect => {
                out.push(r);
                expect += 1;
            }
            _ => break,
        }
    }
    out
}

fn recover(w: &World, algo: Algo) -> Run {
    let mut run = Run { final_map: w.root_map.clone(), ..Default::default() };
    let start = match algo {
        Algo::BWatermark => w.root_watermark,
        Algo::A1AbortOnMismatch | Algo::A2SkipOnMismatch | Algo::CBlindThenWalk => w.tail,
    };
    let prefix = continuous_prefix(w, start);
    match algo {
        Algo::A1AbortOnMismatch | Algo::A2SkipOnMismatch | Algo::BWatermark => {
            for r in &prefix {
                let valid = w.content[r.block as usize] == r.tag;
                if valid {
                    run.final_map[r.object as usize] = r.block;
                    run.replayed += 1;
                } else {
                    run.mismatches += 1;
                    match algo {
                        Algo::A1AbortOnMismatch => {
                            // 把失配当损坏，恢复中止。
                            run.aborted = true;
                            return run;
                        }
                        Algo::A2SkipOnMismatch => {
                            run.skipped += 1;
                        }
                        Algo::BWatermark => {
                            // 只可能是 > 水位的记录 ⇒ 在飞窗口内 ⇒ 判撕裂，丢弃到此为止。
                            run.torn_flagged = true;
                            break;
                        }
                        Algo::CBlindThenWalk => unreachable!(),
                    }
                }
            }
            run.completed = true;
        }
        Algo::CBlindThenWalk => {
            // 盲放全前缀，收尾走读；失败就删掉最尾一条重来（尾删有界：最多前缀长度轮）。
            let mut keep = prefix.len();
            loop {
                let mut map = w.root_map.clone();
                for r in &prefix[..keep] {
                    map[r.object as usize] = r.block;
                }
                // 收尾走读：逐对象验证映射指向的块内容（不复用施加代码的判定）。
                let walk_ok = (0..L as usize).all(|o| {
                    let b = map[o];
                    // 找出该块应有的标签：真值是「最后写它的发布」，走读只有校验和 ⇒
                    // 模型里等价于「内容标签 == 施加进映射的那条记录的 tag 或根快照」。
                    // 用记录集合推期望值，不查 content 之外的真值。
                    let expect = prefix[..keep]
                        .iter()
                        .rev()
                        .find(|r| r.object as usize == o)
                        .map(|r| r.tag);
                    match expect {
                        Some(t) => w.content[b as usize] == t,
                        None => true, // 根快照自带一致性（上次发布已验过）
                    }
                });
                if walk_ok {
                    run.final_map = map;
                    run.replayed = keep as u64;
                    run.completed = true;
                    if keep < prefix.len() {
                        run.torn_flagged = true;
                    }
                    break;
                }
                if keep == 0 {
                    run.aborted = true;
                    break;
                }
                keep -= 1;
                run.retries += 1;
            }
        }
    }
    run
}

/// 独立审计：终态映射必须与真值逐格相等。真值 = torn 丢弃后的 M 状态。
fn audit_final(w: &World, run: &Run) -> bool {
    if !run.completed {
        return false;
    }
    run.final_map == w.root_map
}

/// 闭式：healthy 下重放窗口里失配的记录数（稳态 FIFO 池）。
fn closed_form_mismatches(m: u64, pin: u64, tail: u64) -> u64 {
    (m.saturating_sub(L + pin)).saturating_sub(tail)
}

/// 第二条路径：用重分配追踪器数「窗口内点名块在 M 前被重新分配过」的记录数。
/// 不共享 recover 的任何代码。
fn tracker_mismatches(w: &World) -> u64 {
    let mut n = 0;
    for p in (w.tail + 1)..=w.m {
        let r = w.ring[(p % RING) as usize].expect("窗口在环内");
        assert_eq!(r.jsn, p);
        if w.realloc_at[r.block as usize].iter().any(|&q| q > p && q <= w.m) {
            n += 1;
        }
    }
    n
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!("name=config objects={L} ring={RING} model=counting file_ops=0"))
    );

    // 主格 + 边界格：tail 陈旧程度 × PIN。
    for (m, pin, tail_every) in [(23u64, 1u64, 16u64), (13, 1, 8), (23, 2, 16), (37, 1, 32)] {
        for torn in [false, true] {
            let w = build_world(m, pin, tail_every, torn);
            let cf = closed_form_mismatches(m, pin, w.tail);
            let tr = tracker_mismatches(&w);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=world m={m} pin={pin} tail={} torn={} closed_form={cf} tracker={tr}",
                    w.tail,
                    u8::from(torn)
                ))
            );
            for algo in [Algo::A1AbortOnMismatch, Algo::A2SkipOnMismatch, Algo::BWatermark, Algo::CBlindThenWalk] {
                let r = recover(&w, algo);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=recover m={m} pin={pin} tail={} torn={} algo={} completed={} aborted={} replayed={} mismatches={} skipped={} retries={} torn_flagged={} final_ok={}",
                        w.tail,
                        u8::from(torn),
                        algo.tag(),
                        u8::from(r.completed),
                        u8::from(r.aborted),
                        r.replayed,
                        r.mismatches,
                        r.skipped,
                        r.retries,
                        u8::from(r.torn_flagged),
                        u8::from(audit_final(&w, &r))
                    ))
                );
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 绝对值锚点一**：(M=13, L=4, PIN=1, TAIL_EVERY=8) ⇒ tail=8，失配 0。
    /// 手算：r9 点名的块在 p13 才被释放、p14 才会复用，窗口里没有一条被复用。
    #[test]
    fn absolute_anchor_no_mismatch() {
        let w = build_world(13, 1, 8, false);
        assert_eq!(w.tail, 8);
        assert_eq!(closed_form_mismatches(13, 1, 8), 0);
        assert_eq!(tracker_mismatches(&w), 0);
        let r = recover(&w, Algo::A1AbortOnMismatch);
        assert!(r.completed && !r.aborted);
    }

    /// **判据 1 绝对值锚点二**：(M=23, L=4, PIN=1, TAIL_EVERY=16) ⇒ tail=16，失配恰 2
    /// （p=17、18 两条：块在 p+4 释放、p+5 复用，都 ≤ 23）。
    #[test]
    fn absolute_anchor_two_mismatches() {
        let w = build_world(23, 1, 16, false);
        assert_eq!(w.tail, 16);
        assert_eq!(closed_form_mismatches(23, 1, 16), 2);
        assert_eq!(tracker_mismatches(&w), 2);
    }

    /// **判据 1**：闭式与追踪器在全部主格上逐格相等（两条不共享代码的路径）。
    #[test]
    fn closed_form_equals_tracker_everywhere() {
        for (m, pin, te) in [(23u64, 1u64, 16u64), (13, 1, 8), (23, 2, 16), (37, 1, 32)] {
            let w = build_world(m, pin, te, false);
            assert_eq!(
                closed_form_mismatches(m, pin, w.tail),
                tracker_mismatches(&w),
                "m={m} pin={pin} te={te}"
            );
        }
    }

    /// **判据 2 阳性对照**：失配 > 0 的格上，A1 在健康镜像上必须中止。
    /// 不中止 ⇒ 模型没有判别力 ⇒ 整轮作废。
    #[test]
    fn positive_control_a1_aborts_on_healthy() {
        let w = build_world(23, 1, 16, false);
        let r = recover(&w, Algo::A1AbortOnMismatch);
        assert!(r.aborted, "健康镜像 + 陈旧 tail ⇒ A1 必须把失配当损坏中止");
        assert!(!audit_final(&w, &r));
    }

    /// A2 在健康镜像上悄悄跳过恰好闭式那么多条，终态仍对。
    #[test]
    fn a2_skips_exactly_closed_form() {
        let w = build_world(23, 1, 16, false);
        let r = recover(&w, Algo::A2SkipOnMismatch);
        assert!(r.completed);
        assert_eq!(r.skipped, 2);
        assert!(audit_final(&w, &r), "跳过式的终态必须与真值逐格相等");
    }

    /// **判据 3**：torn 场景下 A2 的旗标必须为 0——它分不出「陈旧失配」与「真撕裂」。
    #[test]
    fn a2_cannot_flag_torn() {
        let w = build_world(23, 1, 16, true);
        let r = recover(&w, Algo::A2SkipOnMismatch);
        assert!(r.completed);
        assert!(!r.torn_flagged, "A2 把撕裂当成又一条陈旧失配吞掉");
        assert_eq!(r.skipped, 3, "2 条陈旧 + 1 条撕裂，同一个计数器");
        assert!(audit_final(&w, &r));
    }

    /// **判据 3**：B 在健康镜像上一条都不重放；torn 场景旗标 1、终态 M。
    #[test]
    fn b_watermark_replays_nothing_healthy_flags_torn() {
        let healthy = build_world(23, 1, 16, false);
        let r = recover(&healthy, Algo::BWatermark);
        assert!(r.completed);
        assert_eq!(r.replayed, 0);
        assert_eq!(r.mismatches, 0);
        assert!(audit_final(&healthy, &r));

        let torn = build_world(23, 1, 16, true);
        let r = recover(&torn, Algo::BWatermark);
        assert!(r.completed);
        assert!(r.torn_flagged);
        assert!(audit_final(&torn, &r));
    }

    /// **判据 3**：C 在健康镜像上零重试；torn 场景恰一轮尾删、旗标 1、终态 M。
    #[test]
    fn c_blind_walk_healthy_and_torn() {
        let healthy = build_world(23, 1, 16, false);
        let r = recover(&healthy, Algo::CBlindThenWalk);
        assert!(r.completed);
        assert_eq!(r.retries, 0, "盲放 + 收尾走读在健康镜像上一轮过");
        assert!(audit_final(&healthy, &r));

        let torn = build_world(23, 1, 16, true);
        let r = recover(&torn, Algo::CBlindThenWalk);
        assert!(r.completed);
        assert_eq!(r.retries, 1, "撕裂 ⇒ 恰一轮尾删");
        assert_eq!(r.replayed, 7, "尾删只许删掉撕裂那一条，窗口里其余 7 条要保住");
        assert!(r.torn_flagged);
        assert!(audit_final(&torn, &r));
    }

    /// **失败条款那条构造不变量**：窗口内被复用块的记录，其对象必有更晚的记录在窗口内。
    /// 违反它 C 的尾删会误删好记录，模型作废。
    #[test]
    fn reused_record_object_has_later_record() {
        let w = build_world(23, 1, 16, false);
        for p in (w.tail + 1)..=w.m {
            let r = w.ring[(p % RING) as usize].unwrap();
            let reused = w.realloc_at[r.block as usize].iter().any(|&q| q > p && q <= w.m);
            if reused {
                let later = ((p + 1)..=w.m).any(|q| {
                    let r2 = w.ring[(q % RING) as usize].unwrap();
                    r2.object == r.object
                });
                assert!(later, "p={p} 的块被复用而对象没有更晚记录");
            }
        }
    }

    /// 边界格：窗口 < L + PIN 时（tail 刚推进过）四条算法全部干净完成。
    /// 这正是「bug 只在 tail 足够陈旧时咬人」的形状——测试常绿不代表没事。
    #[test]
    fn fresh_tail_hides_the_problem() {
        let w = build_world(13, 1, 8, false);
        for algo in [Algo::A1AbortOnMismatch, Algo::A2SkipOnMismatch, Algo::BWatermark, Algo::CBlindThenWalk] {
            let r = recover(&w, algo);
            assert!(r.completed && !r.aborted, "{algo:?}");
            assert!(audit_final(&w, &r), "{algo:?}");
        }
    }

    /// PIN 加大把失配数压小：闭式对 PIN 单调。I-7.4 的窗口每多扣一代，陈旧重放少撞一条。
    #[test]
    fn bigger_pin_fewer_mismatches() {
        let w1 = build_world(23, 1, 16, false);
        let w2 = build_world(23, 2, 16, false);
        assert_eq!(tracker_mismatches(&w1), 2);
        assert_eq!(tracker_mismatches(&w2), 1);
    }

    /// tail 更陈旧失配更多：M=37 / TAIL_EVERY=32 ⇒ tail=32，窗口 5，闭式 0；
    /// 同参数把 tail 压回 16 应给 16 条——直接用闭式验证单调性，不再建世界。
    #[test]
    fn staler_tail_more_mismatches() {
        assert_eq!(closed_form_mismatches(37, 1, 32), 0);
        assert_eq!(closed_form_mismatches(37, 1, 16), 16);
        assert!(closed_form_mismatches(23, 1, 8) > closed_form_mismatches(23, 1, 16));
    }

    /// 撕裂记录的构造自检：环里有 M+1、其单元内容不等于标签。
    #[test]
    fn torn_construction_is_really_torn() {
        let w = build_world(23, 1, 16, true);
        let r = w.torn_record.expect("torn 场景必须有那条记录");
        assert_eq!(r.jsn, 24);
        assert_ne!(w.content[r.block as usize], r.tag, "单元真的没落盘");
    }

    /// 环槽映射自检：窗口内每条 jsn 都能在 slot = jsn % RING 找到自己。
    #[test]
    fn ring_holds_the_window() {
        let w = build_world(37, 1, 32, false);
        for p in (w.tail + 1)..=w.m {
            assert_eq!(w.ring[(p % RING) as usize].unwrap().jsn, p);
        }
    }

    /// **断号即止在本模型里也要被行使**：往窗口中间的槽塞一条错号记录
    /// （上一条时间线的残留，E32（上一条时间线的残留）的形状），前缀必须停在它前面。
    /// E78 的世界天然连续，不造这个场景这条纪律就没被任何测试行使过
    /// （2026-09-02 变异测试实测：M8 一个测试都没红，正是这个洞）。
    #[test]
    fn prefix_stops_at_wrong_jsn() {
        let mut w = build_world(23, 1, 16, false);
        let slot = (20 % RING) as usize;
        w.ring[slot] = Some(Record { jsn: 999, object: 0, block: 0, tag: 0 });
        let prefix = continuous_prefix(&w, w.tail);
        assert_eq!(prefix.len(), 3, "17、18、19 之后必须断号即止");
        assert_eq!(prefix.last().unwrap().jsn, 19);
    }

    /// 审计的判别力：把终态映射改动一格，审计必须翻红。
    #[test]
    fn audit_has_teeth() {
        let w = build_world(13, 1, 8, false);
        let mut r = recover(&w, Algo::BWatermark);
        assert!(audit_final(&w, &r));
        r.final_map[0] = 9999;
        assert!(!audit_final(&w, &r));
    }
}
