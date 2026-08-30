//! E40：反向链挡不挡得住残留记录 —— D23 已定项 8 的方向定案要验的那一步。
//!
//! ## 方向已定不等于有效性已证
//!
//! E33 实测过那个漏洞成立：残留记录校验和完好、`jsn` 恰好等于 expected，
//! I-8.3、在飞记录数上限、`checkpoint_txg` 三条闸逐条核实都不触发。
//! **本实验测的是补丁，不是漏洞。**
//!
//! ## 三条判据（experiments.md E40）
//!
//! 1. **有效性**：加反向链之后，旧时间线的记录被接受进前缀的次数必须为 **0**。
//! 2. **不误杀**：同一条时间线内的合法前缀必须一条不少地被接受
//!    （否则那个 0 是靠「什么都不接受」拿到的）。
//! 3. **宽度**：扫 hash 截断宽度，找出误接受仍为 0 的最小宽度。
//!
//! ⚠️ **阳性对照**：把反向链整个关掉，误接受必须回到非零——
//! 否则「0」分不清是补丁有效还是模型根本没造出残留。
//!
//! ⚠️ 本实验的模型独立于 E33 / E37 重写，不共用它们的代码。
//! 关链那一臂的数应当与 E33 同向（非零），那是一次免费的交叉核对。

use e7_index_bench::Emitter;

/// 槽位怎么映射。E33 只测了 `Mod` 一种，E37 补了解耦那一种。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slots {
    /// `slot = jsn % 槽数`：位置由序号决定。
    ByJsn,
    /// 位置顺序推进、与序号解耦（jbd2 形态）。
    Decoupled,
}

/// 恢复之后新时间线的 `jsn` 从哪儿接着写。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Resume {
    /// 从「前缀末 + 1」接着写——D23 已定项的字面读法。
    AtPrefix,
    /// 跳过断点、号继续涨（jbd2 恢复收尾的形态）。
    SkipHole,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rec {
    jsn: u64,
    timeline: u32,
    prev_hash: u64,
    csum_ok: bool,
}

/// FNV-1a + 终混，再按 `bits` 截断。截断宽度是本实验的自变量之一。
/// 种子让每一轮试验的 hash 各不相同——**碰撞是概率事件，一轮试验量不出概率**。
///
/// ⚠️ **种子必须放在最前，且必须有终混**，两条都是被实测打出来的：
/// 第一版把种子放在末尾、并直接截低位，于是 **8 位那档 2 万轮一次碰撞都没有**（期望 78 次）。
/// 原因是 FNV-1a 的低字节演化 `low = (low ^ b) * 0xb3 mod 256` 对每个输入字节都是**双射**
/// ⇒ 在两端各自的状态之后追加**同样**的种子字节，低字节相等与否**与种子无关**
/// ⇒ 换种子根本没换掉被比较的那几位。**那不是「8 位够用」，那是没量到东西。**
/// ⚠️ 种子是**参数**不是全局：单测并行跑，全局种子会被别的测试改掉，
/// 于是「8 位该出几百次」实测 3 次——那不是结论，是测试之间在互相踩。
fn hash(r: &Rec, bits: u32, seed: u64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in [seed, r.jsn, r.timeline as u64, r.prev_hash] {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    // 终混（splitmix64 的形态）：把高位的差异搅进低位，否则截低位等于只看一条窄通道
    h ^= h >> 33; h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33; h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    if bits >= 64 { h } else { h & ((1u64 << bits) - 1) }
}

struct Ring { slots: usize, cells: Vec<Option<Rec>>, head: usize }

impl Ring {
    fn new(slots: usize) -> Self { Ring { slots, cells: vec![None; slots], head: 0 } }
    fn put(&mut self, map: Slots, r: Rec) {
        let i = match map {
            Slots::ByJsn => (r.jsn as usize) % self.slots,
            Slots::Decoupled => { let h = self.head; self.head = (h + 1) % self.slots; h }
        };
        self.cells[i] = Some(r);
    }
    /// 恢复读侧：按位置顺序往前走。`ByJsn` 下位置由 expected 推出，
    /// `Decoupled` 下位置从 0 顺序推进——两种映射的读侧对齐方式不同。
    fn at(&self, map: Slots, expected: u64, step: usize) -> Option<Rec> {
        let i = match map {
            Slots::ByJsn => (expected as usize) % self.slots,
            Slots::Decoupled => step % self.slots,
        };
        self.cells[i]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Out {
    /// 被接受进前缀、却属于旧时间线的记录数。判据 1 要它为 0。
    replayed_stale: u64,
    /// 属于新时间线、却没被接受的记录数。判据 2 要它为 0。
    lost_own: u64,
    /// 前缀总长度，用来确认「不是靠什么都不接受拿到的 0」。
    accepted: u64,
}

/// 造一条被丢弃的时间线：`settled` 条落定、第 `settled` 条是空洞（csum 坏）、
/// 其后 `stale` 条写成了（校验和好，`jsn` 连续）—— 这就是 E33 复现出的那个形态。
/// 然后恢复、新时间线续写 `new_after` 条，再崩一次，最后做第二次恢复。
fn run(map: Slots, resume: Resume, chain_on: bool, bits: u32,
       slots: usize, settled: u64, stale: u64, new_after: u64, seed: u64) -> Out {
    let mut ring = Ring::new(slots);

    // ── 旧时间线 ──
    let mut prev = 0u64;
    for j in 0..settled {
        let r = Rec { jsn: j, timeline: 0, prev_hash: prev, csum_ok: true };
        prev = hash(&r, bits, seed);
        ring.put(map, r);
    }
    let hole = Rec { jsn: settled, timeline: 0, prev_hash: prev, csum_ok: false };
    let hole_hash = hash(&hole, bits, seed);
    ring.put(map, hole);
    let mut p = hole_hash;
    for k in 0..stale {
        let r = Rec { jsn: settled + 1 + k, timeline: 0, prev_hash: p, csum_ok: true };
        p = hash(&r, bits, seed);
        ring.put(map, r);
    }

    // ── 第一次恢复：断号即止 ⇒ 前缀停在 settled − 1 ──
    // 新时间线接着写。链锚点是前缀最后一条的 hash。
    let anchor = {
        let mut a = 0u64; let mut pv = 0u64;
        for j in 0..settled {
            let r = Rec { jsn: j, timeline: 0, prev_hash: pv, csum_ok: true };
            a = hash(&r, bits, seed); pv = a;
        }
        a
    };
    let start = match resume { Resume::AtPrefix => settled, Resume::SkipHole => settled + stale + 1 };
    // ⚠️ 解耦映射下，恢复要把写头**退回断点位置**再续写——jbd2 恢复收尾就是这么做的
    // （`j_head = info.head_block`）。不退回的话新记录写在残留后面，
    // 顺序扫描根本走不到它们，量出来的是「新记录全丢」而不是「残留被重放」。
    if map == Slots::Decoupled { ring.head = (settled as usize) % slots; }
    let mut pv = anchor;
    for k in 0..new_after {
        let r = Rec { jsn: start + k, timeline: 1, prev_hash: pv, csum_ok: true };
        pv = hash(&r, bits, seed);
        ring.put(map, r);
    }

    // ── 第二次恢复：全环扫描 + 严格连续 + （可选）反向链 ──
    let mut expected = 0u64;
    let mut prev_hash = 0u64;
    let mut accepted = 0u64;
    let mut stale_hit = 0u64;
    let mut step = 0usize;
    loop {
        let Some(r) = ring.at(map, expected, step) else { break };
        if !r.csum_ok || r.jsn != expected { break }
        if chain_on && r.prev_hash != prev_hash { break }
        accepted += 1;
        if r.timeline == 0 && r.jsn >= settled { stale_hit += 1; }
        prev_hash = hash(&r, bits, seed);
        expected += 1;
        step += 1;
        if accepted as usize > slots { break }        // 防环
    }
    // 新时间线本该被接受的条数：从 start 起、且 jsn 连着前缀的那些
    let own_expected = if start == settled { new_after } else { 0 };
    let own_accepted = if start == settled { accepted.saturating_sub(settled).min(new_after) } else { 0 };
    Out { replayed_stale: stale_hit, lost_own: own_expected - own_accepted, accepted }
}

/// 记录头现在是 84 字节（E24 逐字列出的 11 个字段）。反向链要在它之上加 `bits/8` 字节。
const HDR_BYTES: u64 = 84;
/// D23 已定项 7 定案新增：事务号 8 + 提交标记 1。
const TXN_BYTES: u64 = 9;
/// E24 的点名项宽度。
const ITEM_BYTES: u64 = 56;

/// 一条记录**在盘上真的占几字节**：D23 已定项 4 已定「记录头完整落在一个原子单元内」
/// ⇒ 记录要向上取整到原子单元。
///
/// ⚠️ **「头涨百分之几」不是盘上代价**：头住在一个已经被整单元占住的空间里，
/// 多两个字节多半一个块都不多。本函数是拿来把这件事钉死的。
fn on_disk_bytes(chain_bytes: u64, named_items: u64, unit: u64) -> u64 {
    let sz = HDR_BYTES + TXN_BYTES + chain_bytes + named_items * ITEM_BYTES;
    sz.div_ceil(unit) * unit
}

/// 一个原子单元里装得下几个点名项。
fn items_per_unit(chain_bytes: u64, unit: u64) -> u64 {
    unit.saturating_sub(HDR_BYTES + TXN_BYTES + chain_bytes) / ITEM_BYTES
}

/// **不对齐**（多条记录挤同一个原子单元）时的记录字节数。
///
/// ⚠️ 「链宽免费」这个结论**条件于 D23 已定项 4 已定的「记录头完整落在一个原子单元内」**。
/// E24 已记：不对齐则空间不浪费，**但除第一条外都判不了撕裂**。
/// 若那条决定被推翻，每条记录多的那几个字节就按比例收费——本函数把那一档也算出来。
fn packed_bytes(chain_bytes: u64, named_items: u64) -> u64 {
    HDR_BYTES + TXN_BYTES + chain_bytes + named_items * ITEM_BYTES
}

/// 跑 `trials` 轮独立试验，数有多少轮里残留骗过了反向链。
/// **一轮试验量不出概率**：碰撞在每轮只有一次机会（新时间线最后一条 → 第一条活着的残留），
/// 概率约 2⁻ⁿ，所以要靠轮数把它逼出来。
fn false_accept_rate(map: Slots, resume: Resume, bits: u32,
                     slots: usize, settled: u64, stale: u64, new_after: u64,
                     trials: u64) -> u64 {
    let mut bad = 0u64;
    for t in 0..trials {
        let seed = t.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        if run(map, resume, true, bits, slots, settled, stale, new_after, seed).replayed_stale > 0 {
            bad += 1;
        }
    }
    bad
}

fn main() {
    let mut em = Emitter::new();
    let (slots, settled, stale, new_after) = (64usize, 20u64, 8u64, 3u64);
    let trials = 2_000_000u64;
    println!("{}", em.emit_raw(&format!(
        "name=config slots={slots} settled={settled} stale={stale} new_after={new_after} \
         trials={trials} hdr_bytes={HDR_BYTES}")));

    // ① 功能：四个组合 × 开关链，单轮（种子固定为 0）
    for map in [Slots::ByJsn, Slots::Decoupled] {
        for resume in [Resume::AtPrefix, Resume::SkipHole] {
            for chain_on in [false, true] {
                let o = run(map, resume, chain_on, 64, slots, settled, stale, new_after, 0);
                println!("{}", em.emit_raw(&format!(
                    "name=cell slots_map={map:?} resume={resume:?} chain={chain_on} bits=64 \
                     replayed_stale={} lost_own={} accepted={}",
                    o.replayed_stale, o.lost_own, o.accepted)));
            }
        }
    }

    // ② 宽度：拿 20 万轮独立试验逼出误接受率，并报它要多少字节
    for bits in [8u32, 16, 32, 64] {
        let bad = false_accept_rate(Slots::ByJsn, Resume::AtPrefix, bits,
                                    slots, settled, stale, new_after, trials);
        let add = bits as u64 / 8;
        // 每百万轮的误接受数，整数算
        let ppm = bad.saturating_mul(1_000_000) / trials;
        println!("{}", em.emit_raw(&format!(
            "name=width bits={bits} false_accept={bad} trials={trials} false_accept_ppm={ppm} \
             hdr_bytes_added={add} hdr_bytes_total={} hdr_growth_bp={} \
             on_disk_512_items1={} on_disk_512_items12={} items_per_512_unit={}",
            HDR_BYTES + add, add.saturating_mul(10_000) / HDR_BYTES,
            on_disk_bytes(add, 1, 512), on_disk_bytes(add, 12, 512), items_per_unit(add, 512))));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const S: usize = 64; const SET: u64 = 20; const ST: u64 = 8; const NEW: u64 = 3;

    /// **阳性对照 / 交叉核对**：关掉反向链、`slot = jsn % 槽数` 且从前缀末续写时，
    /// 残留记录**必然**被重放——这是 E33 复现出的那个形态。
    /// 它非零，才说明模型真的造出了残留。
    #[test]
    fn without_the_chain_the_leftovers_get_replayed() {
        let o = run(Slots::ByJsn, Resume::AtPrefix, false, 64, S, SET, ST, NEW, 0);
        assert!(o.replayed_stale > 0, "关链时残留该被重放，实测 {}", o.replayed_stale);
    }

    /// **判据 1**：开链之后，四个组合上残留一条都进不来。
    #[test]
    fn the_chain_blocks_every_leftover_in_all_four_combinations() {
        for map in [Slots::ByJsn, Slots::Decoupled] {
            for resume in [Resume::AtPrefix, Resume::SkipHole] {
                let o = run(map, resume, true, 64, S, SET, ST, NEW, 0);
                assert_eq!(o.replayed_stale, 0, "开链该挡住残留（{map:?} / {resume:?}）");
            }
        }
    }

    /// **与 E37 的交叉核对**：仅仅把槽位与序号解耦**并不免疫**——
    /// 恢复把写头退回断点后，读侧的对齐一模一样，残留照样被重放。
    /// 这个数与 `ByJsn` 那一支相等，是一次独立复现（本实验的模型没用 E37 的代码）。
    #[test]
    fn decoupling_the_slots_alone_does_not_help() {
        let by_jsn = run(Slots::ByJsn, Resume::AtPrefix, false, 64, S, SET, ST, NEW, 0);
        let decoup = run(Slots::Decoupled, Resume::AtPrefix, false, 64, S, SET, ST, NEW, 0);
        assert_eq!(decoup.replayed_stale, by_jsn.replayed_stale,
            "解耦不改变读侧对齐，重放的残留条数该与 jsn 取模那一支相同");
        assert!(decoup.replayed_stale > 0, "而且它非零——解耦本身挡不住");
    }

    /// **判据 2**：开链不许误杀自己那条时间线的记录。
    #[test]
    fn the_chain_does_not_reject_the_new_timelines_own_records() {
        let o = run(Slots::ByJsn, Resume::AtPrefix, true, 64, S, SET, ST, NEW, 0);
        assert_eq!(o.lost_own, 0, "自己的记录一条都不许丢");
    }

    /// **那个 0 不是靠「什么都不接受」拿到的**——前缀长度必须是落定数 + 新写数。
    /// **绝对值断言**：由构造直接算出 20 + 3 = 23。
    #[test]
    fn the_accepted_prefix_is_exactly_settled_plus_new() {
        let o = run(Slots::ByJsn, Resume::AtPrefix, true, 64, S, SET, ST, NEW, 0);
        assert_eq!(o.accepted, SET + NEW, "前缀该恰好是 20 条落定 + 3 条新写");
    }

    /// **绝对值断言**：关链时被重放的残留条数，由构造直接算出。
    ///
    /// 新时间线从 `jsn = settled` 起写 `new_after` 条，`slot = jsn % 槽数` ⇒
    /// 它盖掉的是空洞那一格加其后 `new_after − 1` 条残留
    /// ⇒ **还活着的残留 = stale − (new_after − 1)**。
    /// ⚠️ 这个减项是写这条断言时被实测打出来的：原先按 `ST` 写，实测 6 ≠ 8。
    /// **模型没错，是断言少算了覆盖。**
    #[test]
    fn the_number_of_leftovers_replayed_equals_the_number_constructed() {
        let survive = ST - (NEW - 1);
        assert_eq!(survive, 6, "8 条残留被新时间线盖掉 2 条，还剩 6 条");
        let o = run(Slots::ByJsn, Resume::AtPrefix, false, 64, S, SET, ST, NEW, 0);
        assert_eq!(o.replayed_stale, survive, "活下来的残留该全部被重放");
        assert_eq!(o.accepted, SET + NEW + survive, "前缀 = 20 落定 + 3 新写 + 6 残留");
    }

    /// **覆盖关系本身要能被观测到**：新时间线多写一条，活下来的残留就少一条。
    /// 少了它，上一条断言里那个减项只是个凑出来的常数。
    #[test]
    fn each_extra_new_record_overwrites_exactly_one_leftover() {
        let a = run(Slots::ByJsn, Resume::AtPrefix, false, 64, S, SET, ST, 3, 0).replayed_stale;
        let b = run(Slots::ByJsn, Resume::AtPrefix, false, 64, S, SET, ST, 4, 0).replayed_stale;
        assert_eq!(a - b, 1, "多写一条新记录该恰好盖掉一条残留（{a} -> {b}）");
    }

    /// **判据 3**：宽度扫描。64 / 32 / 16 位都该挡住；本实验规模下 8 位也够，
    /// 但那不构成「8 位够用」——碰撞概率随记录数涨，见正文口径。
    #[test]
    fn narrower_hashes_still_block_at_this_scale() {
        for bits in [8u32, 16, 32, 64] {
            let o = run(Slots::ByJsn, Resume::AtPrefix, true, bits, S, SET, ST, NEW, 0);
            assert_eq!(o.replayed_stale, 0, "{bits} 位在本规模下该挡得住");
        }
    }

    /// **跳过断点续写时残留在号上就对不上**——这一支不靠反向链也安全。
    /// 它把「反向链买到什么」限定在「从前缀末续写」那一支上。
    #[test]
    fn skipping_the_hole_makes_the_leftovers_unreachable_by_number() {
        let o = run(Slots::ByJsn, Resume::SkipHole, false, 64, S, SET, ST, NEW, 0);
        assert_eq!(o.replayed_stale, 0, "号跳过断点后残留的 jsn 不再等于 expected");
    }

    /// **序号连续那条闸必须真的在拦**（I-8.3 重放前缀严格连续）。
    /// 解耦映射 + 跳过断点续写时，断点位置上躺着的是**校验和完好、但序号对不上**的新记录
    /// ⇒ 前缀必须恰好停在 20 条。
    /// ⚠️ 变异测试补出来的：摘掉 `jsn != expected` 那半个条件时，九个测试一个都没红——
    /// 因为别的臂里读到的记录序号本来就等于 expected，那条闸在它们身上是冗余的。
    #[test]
    fn the_sequence_check_stops_a_record_whose_checksum_is_fine() {
        let o = run(Slots::Decoupled, Resume::SkipHole, false, 64, S, SET, ST, NEW, 0);
        assert_eq!(o.accepted, SET, "前缀该恰好停在 {SET} 条：断点位置上的新记录序号对不上");
        assert_eq!(o.replayed_stale, 0);
    }

    /// **绝对值断言：实测误接受率必须贴住理论值 2⁻ⁿ。**
    /// 这一条是本实验唯一能发现「hash 根本没在随种子变」的检查——
    /// 第一版把种子放末尾又直接截低位，8 位那档实测 0 次（理论 1/256），
    /// 而当时**没有任何断言比对过理论值**，那个 0 看起来就像一条结论。
    #[test]
    fn the_measured_false_accept_rate_tracks_two_to_the_minus_n() {
        let trials = 200_000u64;
        let bad = false_accept_rate(Slots::ByJsn, Resume::AtPrefix, 8, S, SET, ST, NEW, trials);
        let expect = trials as f64 / 256.0;          // 2⁻⁸
        let ratio = bad as f64 / expect;
        assert!((0.7..1.4).contains(&ratio),
            "8 位的误接受率该贴住 1/256：实测 {bad}，理论 {expect:.0}，比值 {ratio:.2}");
    }

    /// **宽的那几档在同样轮数下必须显著更少**——否则宽度这个自变量没起作用。
    #[test]
    fn wider_hashes_are_measurably_safer() {
        let trials = 200_000u64;
        let b8 = false_accept_rate(Slots::ByJsn, Resume::AtPrefix, 8, S, SET, ST, NEW, trials);
        let b16 = false_accept_rate(Slots::ByJsn, Resume::AtPrefix, 16, S, SET, ST, NEW, trials);
        assert!(b8 > 100, "8 位在 20 万轮里该出几百次，实测 {b8}");
        assert!(b16 * 50 < b8, "16 位该比 8 位少两个数量级（{b16} vs {b8}）");
    }

    /// **绝对值断言：链取 2 / 4 / 8 字节，盘上占的字节数一个都不差。**
    /// 「头涨 4.76%」那个百分比是头的百分比，**不是盘上代价**——
    /// 记录头要向上取整到原子单元（D23 已定项 4 已定），多两个字节多半一个块都不多。
    #[test]
    fn two_four_and_eight_byte_chains_cost_the_same_on_disk() {
        for unit in [512u64, 4096] {
            for items in [1u64, 7, 8, 12, 16] {
                let a = on_disk_bytes(2, items, unit);
                let b = on_disk_bytes(4, items, unit);
                let c = on_disk_bytes(8, items, unit);
                assert_eq!((a, b), (b, c), "链 2/4/8 字节该占同样多（单元 {unit}，点名 {items} 项）");
            }
        }
        // 绝对值：512 单元、点名 1 项时，2 字节链的记录是 151 字节，占满一个 512 扇区
        assert_eq!(84 + 9 + 2 + 56, 151);
        assert_eq!(on_disk_bytes(2, 1, 512), 512);
        assert_eq!(on_disk_bytes(8, 12, 512), 1024);
    }

    /// **「免费」是条件性的**：只在「记录头独占一个原子单元」这条已定规则下成立。
    /// 若改成多条记录挤一个单元，2 → 4 字节就按比例收费——**最坏也只有 1.3%**。
    #[test]
    fn if_records_were_packed_the_extra_bytes_would_cost_proportionally() {
        // 点名 1 项：151 → 153 字节
        assert_eq!(packed_bytes(2, 1), 151);
        assert_eq!(packed_bytes(4, 1), 153);
        let worst = (packed_bytes(4, 1) - packed_bytes(2, 1)) as f64 / packed_bytes(2, 1) as f64;
        assert!(worst < 0.014, "点名 1 项时 2→4 字节最多涨 1.4%，实测 {:.3}", worst);
        // 点名 12 项（D25 粗粒度那一档）：767 → 769，涨幅更小
        assert_eq!(packed_bytes(2, 12), 767);
        assert_eq!(packed_bytes(4, 12), 769);
        let coarse = (packed_bytes(4, 12) - packed_bytes(2, 12)) as f64 / packed_bytes(2, 12) as f64;
        assert!(coarse < 0.003, "点名 12 项时只涨 0.26%，实测 {:.4}", coarse);
    }

    /// **链要多宽才真的开始收费**：512 单元下 28 字节起才挤掉一个点名项。
    /// ⇒ 2 与 4 的差别在容量上**不存在**，不是「很小」。
    #[test]
    fn a_chain_only_starts_costing_capacity_at_28_bytes() {
        assert_eq!(items_per_unit(0, 512), 7);
        assert_eq!(items_per_unit(4, 512), 7);
        assert_eq!(items_per_unit(27, 512), 7);
        assert_eq!(items_per_unit(28, 512), 6, "28 字节起容量掉到 6");
    }

    /// **hash 真的随宽度收窄**——否则宽度扫描测的是同一个东西四遍。
    #[test]
    fn truncation_actually_narrows_the_hash() {
        let r = Rec { jsn: 7, timeline: 0, prev_hash: 123, csum_ok: true };
        assert!(hash(&r, 8, 0) < 256);
        assert!(hash(&r, 16, 0) < 65536);
        assert_eq!(hash(&r, 64, 0), hash(&r, 64, 0));
        assert_eq!(hash(&r, 8, 0), hash(&r, 64, 0) & 0xff);
    }
}
