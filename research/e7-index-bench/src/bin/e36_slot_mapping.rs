//! E36：解耦槽位到底是不是一条出路，还是只把「必然」变成了「偶然」
//!
//! 设计与依据见 `.claude/kb/experiments/36-槽位映射那一维.md`。
//!
//! **它接的是 E32（上一条时间线的残留）欠的那一维。** E32 的失败条款要求
//! 「两种槽位映射 × 两种续写策略」四个组合，实跑只覆盖了 2 / 4——
//! 欠的正是现役实现（jbd2）用的那一种：**位置与序号解耦**。
//!
//! 三方论证的反推腿据此判「jbd2 因而免疫」。**本实验要验的就是这句话。**
//!
//! ## 推下来它不成立，所以才要测
//!
//! 解耦之后位置仍然是**顺序**推进的：恢复走到第 p 个位置，读到的记录若序号恰好
//! 等于 expected 就接受。新时间线从断点位置接着写 ⇒ 它后面那个位置**仍然**是
//! 残留序列的下一条。⇒ **仅仅解耦不改变对齐。**
//!
//! 真正能打散对齐的是**记录长度**：jbd2 的一个事务占「描述块 + 数据块 + 提交块」，
//! 条数随事务内容变。新时间线写的记录若与被丢弃那条**占的位置数不同**，
//! 后续残留就整体错位，恢复走到那里时序号对不上。
//!
//! ⇒ **本实验把「记录长度变不变」做成显式的一轴**，问：
//! 解耦买到的是结构性免疫，还是一个依赖长度恰好不同的巧合。
//!
//! ## 三轴
//!
//! | 轴 | 取值 |
//! |---|---|
//! | 槽位映射 | `by_jsn`（位置 = 序号 % 槽数）/ `decoupled`（位置由游标顺序推进）|
//! | 续写策略 | `resume_at_prefix`（接前缀末 + 1）/ `skip_hole`（号跳过断点）|
//! | 新记录长度 | 与被丢弃那条**相同** / **不同** |
//!
//! ## 判据（两条，必须同时满足）
//!
//! 1. `stale_replayed == 0`——不重放已丢弃时间线的记录。
//! 2. `lost_new == 0`——不丢新时间线已经写成的记录。
//!
//! ## 阳性对照
//!
//! `by_jsn` + `resume_at_prefix` + 等长这一格**必须**重放残留——
//! 它是 E32（上一条时间线的残留）已经测出的那一格，两个实验必须给同一个答案。
//! 对不上说明本实验的模型跑偏了。

use e7_index_bench::Emitter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotMap {
    /// 位置 = 序号 % 槽数。E32（上一条时间线的残留）测的那一支。
    ByJsn,
    /// 位置由写游标顺序推进，与序号解耦。jbd2 形态。
    Decoupled,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Resume {
    /// 从前缀末 + 1 接着写
    AtPrefix,
    /// 号跳过断点（jbd2 恢复收尾 `j_transaction_sequence = ++info.end_transaction`）
    SkipHole,
}

impl SlotMap {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>`
            SlotMap::ByJsn => "by_jsn",
            SlotMap::Decoupled => "decoupled",
        }
    }
}

impl Resume {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>`
            Resume::AtPrefix => "at_prefix",
            Resume::SkipHole => "skip_hole",
        }
    }
}

/// 一条记录。`len` = 它占几个位置（jbd2 的一个事务占描述块 + 数据块 + 提交块）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rec {
    jsn: u64,
    timeline: u32,
    len: usize,
    csum_ok: bool,
}

const TIMELINE_PREV_LAP: u32 = 0;
const TIMELINE_A: u32 = 1;
const TIMELINE_B: u32 = 2;

/// 环：每个位置放一条记录的**起始标记**（`Some`）或续块（`None` 表示被前一条占着）。
/// 为了让「起始位置」可判，续块用 `Filler` 标出来。
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Empty,
    Start(Rec),
    /// 被前一条记录占用的续位
    Filler,
}

struct Ring {
    cell: Vec<Cell>,
}

impl Ring {
    fn new(slots: usize) -> Self {
        Ring { cell: vec![Cell::Empty; slots] }
    }
    fn n(&self) -> usize {
        self.cell.len()
    }
    /// 在位置 `pos` 起写一条占 `len` 位的记录，返回下一个可写位置。
    fn write_at(&mut self, pos: usize, r: Rec) -> usize {
        let n = self.n();
        self.cell[pos % n] = Cell::Start(r);
        for k in 1..r.len {
            self.cell[(pos + k) % n] = Cell::Filler;
        }
        (pos + r.len) % n
    }
    fn start_at(&self, pos: usize) -> Option<Rec> {
        match self.cell[pos % self.n()] {
            // 没有 `_ =>`
            Cell::Start(r) => Some(r),
            Cell::Empty | Cell::Filler => None,
        }
    }
}

/// 恢复：从 `tail_pos` 起，按位置顺序走，逐条验证，择最长合法前缀。
///
/// **两种槽位映射共用这一段**——差别只在写侧位置怎么定，不在读侧怎么走。
/// 这正是「解耦不改变读侧对齐」那句推理的可执行形式。
fn recover(ring: &Ring, tail_pos: usize, first_expected: u64) -> Vec<Rec> {
    let mut out = Vec::new();
    let mut pos = tail_pos;
    let mut expected = first_expected;
    let mut steps = 0;
    while steps < ring.n() {
        let r = match ring.start_at(pos) {
            Some(r) => r,
            None => break,
        };
        if !r.csum_ok || r.jsn != expected {
            break;
        }
        out.push(r);
        pos = (pos + r.len) % ring.n();
        expected += 1;
        steps += r.len;
    }
    out
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    first_prefix: u64,
    stale_replayed: u64,
    lost_new: u64,
    /// 新时间线的写**盖掉了几条残留**——它是「字节落在哪」的可观测形式。
    /// ⚠️ 少了它，两种槽位映射在 `at_prefix` 下恰好重合（那正是主结论），
    /// 于是「解耦」这一维在度量上完全不可见（变异 M1 实测：改掉它一个测试都不红）。
    stale_overwritten: u64,
}

/// * `settled` —— 崩溃前安定下来的记录条数
/// * `stale` —— 断点之后时间线 A 留下的、校验和完好的记录条数
/// * `new_after` —— 恢复之后时间线 B 写成的记录条数
/// * `old_len` / `new_len` —— 旧 / 新记录各占几个位置
fn run(
    map: SlotMap,
    resume: Resume,
    slots: usize,
    settled: u64,
    stale: u64,
    new_after: u64,
    old_len: usize,
    new_len: usize,
) -> Out {
    let mut ring = Ring::new(slots);

    // 稳态布景：上一圈的记录铺满环（等长，占位与旧记录相同）
    let mut p = 0usize;
    let mut lap_jsn = 1u64;
    while p + old_len <= slots {
        let r = Rec { jsn: lap_jsn, timeline: TIMELINE_PREV_LAP, len: old_len, csum_ok: true };
        p = ring.write_at(p, r);
        lap_jsn += 1;
        if p == 0 {
            break;
        }
    }

    let tail_jsn = 1000u64; // 本圈第一条的序号，远离上一圈
    let tail_pos = 0usize;

    // 时间线 A：安定段
    let mut pos = tail_pos;
    for i in 0..settled {
        let r = Rec { jsn: tail_jsn + i, timeline: TIMELINE_A, len: old_len, csum_ok: true };
        pos = ring.write_at(pos, r);
    }
    let hole_jsn = tail_jsn + settled;
    let hole_pos = pos;
    // 空洞那条没写成 ⇒ 它的位置上还是上一圈的东西；后面几条落盘了
    let mut spos = (hole_pos + old_len) % slots;
    for k in 0..stale {
        let r = Rec { jsn: hole_jsn + 1 + k, timeline: TIMELINE_A, len: old_len, csum_ok: true };
        spos = ring.write_at(spos, r);
    }

    // 第一次恢复
    let acc1 = recover(&ring, tail_pos, tail_jsn);
    let mut o = Out { first_prefix: acc1.len() as u64, ..Default::default() };

    // 恢复之后接着写：位置与序号各自怎么定
    let resume_jsn = match resume {
        // 没有 `_ =>`
        Resume::AtPrefix => acc1.last().map(|r| r.jsn + 1).unwrap_or(tail_jsn),
        Resume::SkipHole => {
            // 号跳过断点：跳到「见过的最大合法序号 + 1」
            let mut m = acc1.last().map(|r| r.jsn).unwrap_or(tail_jsn);
            for i in 0..slots {
                if let Some(r) = ring.start_at(i) {
                    if r.csum_ok && r.jsn > m && r.jsn >= tail_jsn {
                        m = r.jsn;
                    }
                }
            }
            m + 1
        }
    };
    let resume_pos = match map {
        // 没有 `_ =>`
        // 位置是序号的函数 ⇒ 由 resume_jsn 决定
        SlotMap::ByJsn => ((resume_jsn - tail_jsn) as usize * old_len) % slots,
        // 位置由游标顺序推进 ⇒ 回退到断点位置（jbd2 的 `j_head = info.head_block`）
        SlotMap::Decoupled => hole_pos,
    };

    // 时间线 B 写 new_after 条
    let mut bpos = resume_pos;
    for k in 0..new_after {
        let r = Rec { jsn: resume_jsn + k, timeline: TIMELINE_B, len: new_len, csum_ok: true };
        bpos = ring.write_at(bpos, r);
    }

    // 新时间线盖掉了几条残留
    let survived = (0..slots)
        .filter_map(|i| ring.start_at(i))
        .filter(|r| r.timeline == TIMELINE_A && r.jsn >= hole_jsn)
        .count() as u64;
    o.stale_overwritten = stale - survived.min(stale);

    // 第二次恢复
    let acc2 = recover(&ring, tail_pos, tail_jsn);
    o.stale_replayed = acc2
        .iter()
        .filter(|r| r.timeline == TIMELINE_A && r.jsn >= hole_jsn)
        .count() as u64;
    let replayed_new = acc2.iter().filter(|r| r.timeline == TIMELINE_B).count() as u64;
    o.lost_new = new_after - replayed_new;
    o
}

const SLOTS: usize = 256;
const SETTLED: u64 = 20;
const OLD_LEN: usize = 2;

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config slots={SLOTS} settled={SETTLED} old_len={OLD_LEN} note=槽位映射那一维"
        ))
    );
    for map in [SlotMap::ByJsn, SlotMap::Decoupled] {
        for resume in [Resume::AtPrefix, Resume::SkipHole] {
            for new_len in [OLD_LEN, OLD_LEN + 1] {
                for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
                    let o = run(map, resume, SLOTS, SETTLED, stale, new_after, OLD_LEN, new_len);
                    let pass = o.stale_replayed == 0 && o.lost_new == 0;
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=cell map={} resume={} new_len={} stale={} new_after={} \
                             first_prefix={} stale_replayed={} lost_new={} \
                             stale_overwritten={} pass={}",
                            map.name(),
                            resume.name(),
                            new_len,
                            stale,
                            new_after,
                            o.first_prefix,
                            o.stale_replayed,
                            o.lost_new,
                            o.stale_overwritten,
                            pass
                        ))
                    );
                }
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **阳性对照 / 与 E32（上一条时间线的残留）对拍**：
    /// `by_jsn` + `at_prefix` + 等长这一格必须重放残留，
    /// 且条数服从 E32（上一条时间线的残留）测出的那条式子。
    /// 对不上说明本实验的模型跑偏了。
    #[test]
    fn the_by_jsn_arm_reproduces_what_e33_measured() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let want = stale.saturating_sub(new_after - 1);
            let o = run(SlotMap::ByJsn, Resume::AtPrefix, SLOTS, SETTLED, stale, new_after, OLD_LEN, OLD_LEN);
            assert_eq!(
                o.stale_replayed, want,
                "by_jsn 该复现 E32 的式子：残留 {stale}、新写 {new_after} ⇒ 重放 {want}"
            );
        }
    }

    /// **本实验的主结论：仅仅解耦槽位救不了。**
    /// 等长时 `decoupled` 与 `by_jsn` 逐格相同——位置解耦了，但读侧仍然顺序对齐。
    #[test]
    fn decoupling_alone_does_not_help_when_record_lengths_match() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let a = run(SlotMap::ByJsn, Resume::AtPrefix, SLOTS, SETTLED, stale, new_after, OLD_LEN, OLD_LEN);
            let b = run(SlotMap::Decoupled, Resume::AtPrefix, SLOTS, SETTLED, stale, new_after, OLD_LEN, OLD_LEN);
            assert_eq!(
                a.stale_replayed, b.stale_replayed,
                "等长时两种槽位映射该给同一个数（残留 {stale}、新写 {new_after}）"
            );
            assert!(b.stale_replayed > 0, "解耦这一支等长时照样重放");
        }
    }

    /// **真正打散对齐的是记录长度不同。**
    /// 新记录多占一位 ⇒ 后续残留整体错位 ⇒ 恢复走到那里序号对不上。
    #[test]
    fn a_different_record_length_is_what_breaks_the_alignment() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let o = run(SlotMap::Decoupled, Resume::AtPrefix, SLOTS, SETTLED, stale, new_after, OLD_LEN, OLD_LEN + 1);
            assert_eq!(
                o.stale_replayed, 0,
                "长度不同时残留该被错位挡住（残留 {stale}、新写 {new_after}）"
            );
        }
    }

    /// **而那是巧合不是保证**：长度相同的那一格立刻退化回去。
    /// ⇒ 「解耦槽位」买到的是概率，不是结构性免疫。
    #[test]
    fn the_protection_from_length_is_a_coincidence_not_a_guarantee() {
        let protected = run(SlotMap::Decoupled, Resume::AtPrefix, SLOTS, SETTLED, 7, 1, OLD_LEN, OLD_LEN + 1);
        let exposed = run(SlotMap::Decoupled, Resume::AtPrefix, SLOTS, SETTLED, 7, 1, OLD_LEN, OLD_LEN);
        assert_eq!(protected.stale_replayed, 0);
        assert_eq!(exposed.stale_replayed, 7, "同一条臂，只把新记录长度改回等长就全数重放");
    }

    /// **判别力：第一次恢复必须真的停在空洞处。**
    #[test]
    fn the_first_recovery_stops_at_the_hole_in_every_configuration() {
        for map in [SlotMap::ByJsn, SlotMap::Decoupled] {
            for resume in [Resume::AtPrefix, Resume::SkipHole] {
                let o = run(map, resume, SLOTS, SETTLED, 3, 1, OLD_LEN, OLD_LEN);
                assert_eq!(o.first_prefix, SETTLED, "{} / {} 没停在空洞处", map.name(), resume.name());
            }
        }
    }

    /// **号跳过断点那一支照样有病，只是方向相反**——新写的记录接不上前缀。
    #[test]
    fn skipping_the_hole_loses_the_new_records_under_both_mappings() {
        for map in [SlotMap::ByJsn, SlotMap::Decoupled] {
            let o = run(map, Resume::SkipHole, SLOTS, SETTLED, 3, 2, OLD_LEN, OLD_LEN);
            assert_eq!(o.stale_replayed, 0, "{}", map.name());
            assert_eq!(o.lost_new, 2, "{} 跳过断点后新写的两条都够不到", map.name());
        }
    }

    /// **两种槽位映射在「字节落在哪」上确实不同**——`skip_hole` 下才看得出来。
    ///
    /// 解耦那一支把新记录写回**断点位置**（jbd2 的 `j_head = info.head_block`），
    /// 于是它盖掉残留；按序号那一支把新记录写到**残留之后**，一条也盖不掉。
    /// ⚠️ 没有这条，「解耦」这一维在度量上不可见（变异 M1）。
    #[test]
    fn the_two_mappings_put_the_bytes_in_different_places() {
        let dec = run(SlotMap::Decoupled, Resume::SkipHole, SLOTS, SETTLED, 3, 2, OLD_LEN, OLD_LEN);
        let by = run(SlotMap::ByJsn, Resume::SkipHole, SLOTS, SETTLED, 3, 2, OLD_LEN, OLD_LEN);
        // 第一条新记录填的是**空洞那个位置**本身，所以盖掉的是「新写条数 − 1」条残留
        // ——与 E32（上一条时间线的残留）那条式子同源。
        assert_eq!(dec.stale_overwritten, 1, "解耦：新写 2 条落在断点位置，该盖掉 2−1 = 1 条残留");
        assert_eq!(by.stale_overwritten, 0, "按序号：新写的落在残留之后，一条也盖不掉");
    }

    /// **绝对值：环几何与布景由构造给出。**
    /// 少了它，四条轴可以一起错而互比仍然「成立」。
    #[test]
    fn the_ring_geometry_is_pinned_by_construction() {
        let ring = Ring::new(SLOTS);
        assert_eq!(ring.n(), 256);
        let mut r = Ring::new(8);
        let next = r.write_at(0, Rec { jsn: 1, timeline: TIMELINE_A, len: 2, csum_ok: true });
        assert_eq!(next, 2, "占 2 位的记录写完，下一个可写位置该是 2");
        assert!(matches!(r.cell[0], Cell::Start(_)));
        assert_eq!(r.cell[1], Cell::Filler, "续位该被标成 Filler");
        assert_eq!(r.start_at(1), None, "续位上取不到记录起始");
    }
}
