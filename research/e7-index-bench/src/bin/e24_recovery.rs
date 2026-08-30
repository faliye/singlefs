//! E24：恢复算法「先信 tail 再往前走」会不会丢记录 —— [checks-owed.md](checks-owed.md) C29。
//!
//! **这条不是性能实验，是安全性实验。** D23 新增的硬要求
//! 「恢复必须先全环扫描全验、再择最长合法前缀，不许先信 tail 再往前走」
//! 是从机制推出来的，没有任何东西验过它。本实验把那个机制做成可执行的，
//! 在**每一个**崩溃点上比对两种恢复算法恢复出的记录集合。
//!
//! ## 要害：陈旧 tail 指向的位置已经被新一圈写掉了
//!
//! 环写满一圈后，位置 `jsn % R` 被后来的记录覆盖。若持久 tail 停在 `T_stale`
//! 而真实 tail 已到 `T_true`，那么 `[T_stale, T_true)` 这段槽位装的是**新记录**。
//! ⇒ 「先信 tail 再往前走」在第一条就 `jsn != expected`，断号即止，
//! **连 `T_true` 之后仍然需要的记录也一条都不重放**。
//!
//! ## 判红条件在实验注册时就写死（rules/test-discipline.md：失败条款不许让结论不可证伪）
//!
//! - 若 `TrustTail` **从不**漏 ⇒ 那个窗口不存在，D23 那条硬要求的推理是错的，整条作废。
//! - 若 `ScanAll` **也漏** ⇒ 「陈旧 tail 只是多做功」整条作废，tail 只能按权威态办。
//!
//! ⚠️ **两条臂互比不够**：还要一条把绝对值钉死的断言——漏掉的记录数**恰等于**
//! 窗口内写入的记录数，该数由环几何独立算出，不从被测代码取。

use e7_index_bench::Emitter;

/// 环里的一个槽。`None` = 从没写过。
type Slot = Option<Record>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Record {
    jsn: u64,
    /// 该记录属于哪个 checkpoint。恢复时 `< 已完成的 checkpoint` 的记录不需要重放。
    ckpt: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Recovery {
    /// jbd2 若按「从持久 tail 起往前走」实现：读到第一个 `jsn != expected` 就停。
    TrustTail,
    /// D22 对根槽定过的次序：先逐个验证全部候选，再择最长合法前缀。
    ScanAll,
}

struct Journal {
    ring: Vec<Slot>,
    next_jsn: u64,
    /// 已持久化的 tail：小于它的 jsn 已经 checkpoint 完、不必重放。
    tail_persisted: u64,
    /// 真实的 tail：checkpoint 实际推进到哪。**可以领先于持久值**——
    /// 那个差就是本实验要测的窗口。
    tail_true: u64,
}

impl Journal {
    fn new(ring_blocks: usize) -> Self {
        Self { ring: vec![None; ring_blocks], next_jsn: 1, tail_persisted: 1, tail_true: 1 }
    }
    fn write(&mut self, ckpt: u64) {
        let j = self.next_jsn;
        let n = self.ring.len();
        self.ring[(j as usize) % n] = Some(Record { jsn: j, ckpt });
        self.next_jsn += 1;
    }
    /// checkpoint 完成：真实 tail 推到最新，但**持久 tail 只有在真的写了才动**。
    fn checkpoint(&mut self, persist_tail: bool) {
        self.tail_true = self.next_jsn;
        if persist_tail {
            self.tail_persisted = self.tail_true;
        }
    }

    /// 恢复：返回重放出来的 jsn 集合（升序）。
    fn recover(&self, how: Recovery) -> Vec<u64> {
        let n = self.ring.len();
        match how {
            Recovery::TrustTail => {
                let mut out = Vec::new();
                let mut expect = self.tail_persisted;
                loop {
                    match self.ring[(expect as usize) % n] {
                        Some(r) if r.jsn == expect => { out.push(r.jsn); expect += 1; }
                        // 断号即止 —— D23 已定的前缀规则
                        _ => break,
                    }
                }
                out
            }
            Recovery::ScanAll => {
                // 先逐个验证全部槽，再择**最长合法前缀**：
                // 取环里最大的 jsn，向下找连续段，段的下界不低于持久 tail。
                let max = self.ring.iter().flatten().map(|r| r.jsn).max();
                let Some(max) = max else { return Vec::new() };
                // ⚠️ 环里最大的 jsn 也可能低于持久 tail（刚 checkpoint 完、还没写新记录）。
                // 那时一条都不该重放。漏掉这个判断会多重放一条 —— 由健康场景的测试抓到。
                if max < self.tail_persisted { return Vec::new(); }
                let mut lo = max;
                while lo > self.tail_persisted {
                    match self.ring[((lo - 1) as usize) % n] {
                        Some(r) if r.jsn == lo - 1 => lo -= 1,
                        _ => break,
                    }
                }
                (lo..=max).collect()
            }
        }
    }
}

/// 本轮**必须**被重放的记录：jsn ≥ 真实 tail 的那些。
/// **独立算出，不从被测代码取**——它是判据，不是观测。
fn must_replay(tail_true: u64, next_jsn: u64) -> Vec<u64> {
    (tail_true..next_jsn).collect()
}

/// 一个崩溃点的结果。
///
/// ⚠️ **`spurious` 这一维是变异测试逼出来的**：只数「漏掉的」时，
/// 把 `ScanAll` 的下界（不低于持久 tail）整个删掉，8 个测试一个都不红——
/// 因为环绕回本身就会截住向下的搜索。而下界真正防的是**多重放**：
/// 低于真实 tail 的记录已经 checkpoint 过，重放它们不是「多做功」那么简单，
/// 它要求每条记录都幂等，而那是一条本工程从没定过的额外要求。
#[derive(Debug, Clone, Copy, PartialEq)]
struct Outcome { missed: usize, replayed: usize, needed: usize, spurious: usize }

/// 在「checkpoint 完成 → tail 写出去」这个窗口内的第 `gap` 个位置崩溃。
///
/// `gap` = checkpoint 之后又写了几条记录才崩。`gap == 0` 表示 tail 写和
/// checkpoint 同时生效（无窗口）。
fn run_one(ring_blocks: usize, warmup: u64, gap: u64, how: Recovery) -> Outcome {
    run_with(ring_blocks, warmup, gap, how, false)
}

/// `persist_tail = true` 是**健康场景**：checkpoint 之后 tail 真的写出去了。
/// ⚠️ **它必须被测**——只测陈旧场景时 `tail_persisted` 恒为 1，
/// `ScanAll` 的下界永远不起作用，把下界删掉一个测试都不红（变异测试实测）。
fn run_with(ring_blocks: usize, warmup: u64, gap: u64, how: Recovery, persist_tail: bool) -> Outcome {
    let mut j = Journal::new(ring_blocks);
    // 先写满一圈以上，保证陈旧 tail 指向的槽真的被覆盖过
    for _ in 0..warmup { j.write(0); }
    j.checkpoint(persist_tail);
    // 窗口内继续写记录
    for _ in 0..gap { j.write(1); }
    let got = j.recover(how);
    let need = must_replay(j.tail_true, j.next_jsn);
    let missed = need.iter().filter(|x| !got.contains(x)).count();
    // 多重放：恢复出的记录里，jsn 低于真实 tail 的那些 —— 它们已经 checkpoint 过了
    let spurious = got.iter().filter(|&&x| x < j.tail_true).count();
    Outcome { missed, replayed: got.len(), needed: need.len(), spurious }
}

fn main() {
    let mut em = Emitter::new();
    let ring = 64usize;
    println!("{}", em.emit_raw(&format!("name=config ring_blocks={ring}")));

    for warmup in [200u64, 1000] {
        for gap in [0u64, 1, 5, 20, 63, 100] {
            for how in [Recovery::TrustTail, Recovery::ScanAll] {
                let o = run_one(ring, warmup, gap, how);
                println!("{}", em.emit_raw(&format!(
                    "name=cell warmup={warmup} gap={gap} algo={how:?} \
                     missed={} replayed={} needed={} spurious={}",
                    o.missed, o.replayed, o.needed, o.spurious)));
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判红条件一（注册时写死）**：若 `TrustTail` 从不漏，本实验推理错、整条作废。
    /// 这条测试就是那个条件的可执行形式——它必须**证明窗口存在**。
    #[test]
    fn trust_tail_really_loses_records_inside_the_window() {
        let o = run_one(64, 200, 5, Recovery::TrustTail);
        assert!(o.needed > 0, "窗口里没有需要重放的记录，这个场景没建对");
        assert_eq!(o.missed, o.needed,
            "先信 tail 的算法本该一条都恢复不出来（断号即止），实测漏 {} / 需 {}",
            o.missed, o.needed);
    }

    /// **判红条件二**：窗口**不超过环**时，若 `ScanAll` 也漏，
    /// 「陈旧 tail 只是多做功」整条作废。
    #[test]
    fn scan_all_loses_nothing_while_the_window_fits_in_the_ring() {
        let ring = 64usize;
        for warmup in [200u64, 1000] {
            for gap in [0u64, 1, 5, 20, 63] {
                assert!(gap < ring as u64, "本测试只覆盖窗口装得下的情形");
                let o = run_one(ring, warmup, gap, Recovery::ScanAll);
                assert_eq!(o.missed, 0,
                    "全环扫描在 warmup={warmup} gap={gap} 处漏了 {} 条", o.missed);
            }
        }
    }

    /// **窗口撑爆环时两条臂一起丢，而且这不是恢复算法的错。**
    ///
    /// 这条是首次跑本实验时被测试抓出来的：`gap=100` 而环只有 64 块，
    /// 窗口内的记录互相覆盖，连全环扫描也漏 36 条。
    /// ⇒ **恢复算法救不了几何**：环大小必须 ≥ 窗口，那是
    /// [checks-owed.md](checks-owed.md) C28（环几何不变量）的职责，不是恢复算法的。
    /// 把这一格写成会红的测试，是为了不让它以后被当成「恢复算法的缺陷」去改错地方。
    #[test]
    fn when_the_window_overflows_the_ring_both_algorithms_lose() {
        let ring = 64usize;
        let gap = 100u64;
        assert!(gap > ring as u64, "本测试要的正是窗口装不下的情形");
        for how in [Recovery::TrustTail, Recovery::ScanAll] {
            let o = run_one(ring, 200, gap, how);
            assert!(o.missed > 0, "{how:?} 在窗口撑爆环时本该丢记录");
        }
        // 绝对值：全环扫描最多只能恢复出环装得下的那些
        let s = run_one(ring, 200, gap, Recovery::ScanAll);
        assert_eq!(s.replayed, ring, "全环扫描恢复出的条数上界就是环的槽数");
        assert_eq!(s.missed as u64, gap - ring as u64, "漏的恰是溢出的那部分");
    }

    /// **绝对值断言，不是臂间互比**（rules/test-discipline.md：
    /// 「只让多条臂互相比，测不出所有臂一起错」）。
    /// 漏掉的记录数必须**恰等于**窗口内写入的记录数，而后者由参数直接给出。
    #[test]
    fn missed_count_equals_records_written_inside_the_window() {
        for gap in [1u64, 5, 20, 63] {
            let o = run_one(64, 200, gap, Recovery::TrustTail);
            assert_eq!(o.missed as u64, gap,
                "gap={gap}：漏掉的条数应恰等于窗口内写入的条数");
            assert_eq!(o.needed as u64, gap);
        }
    }

    /// **阳性对照**：窗口为零（tail 与 checkpoint 同时生效）时两条臂都不许漏。
    /// 少了这条，「TrustTail 会漏」分不清是窗口造成的还是算法根本就不工作。
    #[test]
    fn with_no_window_both_algorithms_lose_nothing() {
        for how in [Recovery::TrustTail, Recovery::ScanAll] {
            let o = run_one(64, 200, 0, how);
            assert_eq!(o.missed, 0, "{how:?} 在无窗口时不该漏");
            assert_eq!(o.needed, 0, "无窗口时本来就没有要重放的记录");
        }
    }

    /// **环必须真的绕回来**，否则陈旧 tail 指向的槽没被覆盖，整个场景不成立。
    #[test]
    fn the_ring_actually_wraps_so_stale_slots_are_overwritten() {
        let ring = 64usize;
        let mut j = Journal::new(ring);
        for _ in 0..200 { j.write(0); }
        // 持久 tail 停在 1；1 号槽此刻装的应当是后来某条记录，不是 jsn=1
        assert_eq!(j.tail_persisted, 1);
        let slot = j.ring[1 % ring].expect("槽应当被写过");
        assert_ne!(slot.jsn, 1, "环没绕回来，陈旧 tail 的槽还是原记录——场景不成立");
        assert!(slot.jsn > ring as u64, "槽里应当是绕回之后写的记录");
    }

    /// **全环扫描不丢数据，但它会把陈旧 tail 以来的记录全部重放一遍。**
    ///
    /// 这条是加 `spurious` 维度时当场测出来的，不是预期内的：
    /// `ScanAll` 只知道**陈旧的**持久 tail，于是把环里 tail 之上能找到的全重放。
    /// gap=0（窗口里一条新记录都没有）时它仍重放 64 条已 checkpoint 的记录。
    ///
    /// ⇒ **「陈旧 tail 只是多做功」这句话有一个前提：重放必须幂等。**
    /// 没有幂等，陈旧 tail 就从「多做功」变成「重复施加已生效的记录」。
    /// 那是本工程从没定过的额外要求，已记进 D23。
    ///
    /// 绝对值钉死：多重放的条数 = 环里 jsn 高于持久 tail、低于真实 tail 的那些，
    /// 环满时就是环的槽数减去窗口内的新记录数。
    #[test]
    fn scan_all_replays_everything_back_to_the_stale_tail() {
        let ring = 64usize;
        for gap in [0u64, 1, 5, 20, 63] {
            let o = run_one(ring, 200, gap, Recovery::ScanAll);
            assert_eq!(o.missed, 0, "全环扫描不该丢");
            assert_eq!(o.spurious, ring - gap as usize,
                "gap={gap}：多重放的条数应是环槽数减去窗口内新记录数");
        }
    }

    /// **先信 tail 的算法一条都不多重放**——因为它一条都恢复不出来。
    /// 这条与上一条合起来说明：两条臂各自付不同的代价，不是一优一劣。
    #[test]
    fn trust_tail_never_over_replays_because_it_recovers_nothing() {
        for gap in [1u64, 5, 20, 63] {
            let o = run_one(64, 200, gap, Recovery::TrustTail);
            assert_eq!(o.spurious, 0);
            assert_eq!(o.replayed, 0, "断号即止 ⇒ 一条都恢复不出来");
        }
    }

    /// **健康场景：tail 正常持久化时，全环扫描恢复出的必须恰好是需要的那些。**
    /// 这条钉住 `ScanAll` 的下界——删掉下界之后它会一路回溯到环的头，
    /// 把已 checkpoint 的记录全部多重放（变异测试已证本测试会红）。
    #[test]
    fn with_a_fresh_tail_scan_all_replays_exactly_what_is_needed() {
        let ring = 64usize;
        for gap in [0u64, 1, 5, 20, 63] {
            let o = run_with(ring, 200, gap, Recovery::ScanAll, true);
            assert_eq!(o.missed, 0, "gap={gap}：健康场景不该丢");
            assert_eq!(o.spurious, 0, "gap={gap}：健康场景不该多重放");
            assert_eq!(o.replayed as u64, gap, "gap={gap}：恢复出的条数应恰等于窗口内的新记录数");
        }
    }

    /// 健康场景下两条臂**必须一致**——不一致说明其中一条在正常路径上就错了。
    #[test]
    fn with_a_fresh_tail_both_algorithms_agree() {
        for gap in [0u64, 1, 5, 20, 63] {
            let t = run_with(64, 200, gap, Recovery::TrustTail, true);
            let s = run_with(64, 200, gap, Recovery::ScanAll, true);
            assert_eq!(t.missed, 0, "gap={gap}：健康场景下先信 tail 也不该丢");
            assert_eq!(t.replayed, s.replayed, "gap={gap}：健康场景下两条臂该一致");
            assert_eq!(t.spurious, 0);
        }
    }

    /// `must_replay` 是判据，必须独立于被测的恢复算法。
    #[test]
    fn must_replay_is_plain_arithmetic() {
        assert_eq!(must_replay(10, 13), vec![10, 11, 12]);
        assert_eq!(must_replay(5, 5), Vec::<u64>::new());
    }

    /// 两条臂在窗口内必须**分开**——这是本实验存在的理由。
    #[test]
    fn the_two_algorithms_diverge_inside_the_window() {
        let t = run_one(64, 200, 5, Recovery::TrustTail);
        let s = run_one(64, 200, 5, Recovery::ScanAll);
        assert!(t.missed > s.missed, "两条臂没分开，实验归零");
        assert_eq!(s.missed, 0);
        assert_eq!(t.missed, 5);
    }
}
