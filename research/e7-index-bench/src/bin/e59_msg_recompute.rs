//! E59：缓冲里那条消息能不能只从单元重算出来 —— D11 已定项 4（跑这个实验时它还未定）。
//!
//! **它答的是一个「态别」问题，不是性能问题。**
//! D11 已定项 1（2026-08-31 用户定案）已经定了「留消息缓冲区」、已定项 2 定了 ε=0.65。
//! 剩下的生死前置逐字是：
//!
//! > 缓冲里那条消息能不能只从单元重算出来。
//! > 能 ⇒ 缓冲仍是纯派生态；不能 ⇒ 它已晋升为权威态，
//! > D21 的「权威态 = 单元 + 记账 + 根」必须扩项。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md：不许照印象建模）
//!
//! - D18 已定项 3（2026-08-29 用户定案）：每个数据单元带一个明文自描述头，字段取
//!   `(单元类型标签, 树 ID, 对象 ID, 对象出生代, 锚点偏移)`。**禁放清单**：物理落点 /
//!   设备 ID、文件名 / 分隔 key。
//!   ⇒ 头里**没有**「这一次写是第几次写」——`对象出生代` 是对象的出生代，不是本次写的序号。
//! - D18 未定项 1：谱系重写序号「要不要做、宽度多少」**未定**。
//! - D20：单元自包含 =「我是谁、我属于谁、我是第几代」。
//! - D21（权威态与派生态的分界）（2026-08-28 用户定案）：权威态 = 单元 + 记账 + 根；索引是派生态。
//! - D8 已定项 1（2026-08-29 用户定案）：条目形态取**幂等完整值**（含 tombstone），不是增量 Δ。
//! - D11 已定项 2：16 KiB 节点、ε=0.65 ⇒ 扇出 119、缓冲 665 条。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 甲臂（只从单元）四类消息的分歧数：
//!
//! | 观测 | 判 |
//! |---|---|
//! | 四类全 0 | **能重算**：缓冲是纯派生态，D11 已定项 4 答「能」 |
//! | 部分类 > 0 | **按类拆**：只有那几类晋升权威态，D21 扩项限于它们 |
//! | 四类全 > 0 | **不能重算**：D21 必须扩项 |
//! | 陈旧副本不可见时全 0、可见时 > 0 | 结论**条件于「陈旧副本的可见性」**，那条前提要单列成欠账 |
//!
//! ## 怎样才算失败（跑前写死）
//!
//! | 条款 | 触发时怎么判 |
//! |---|---|
//! | 扫到的单元数 ≠ 独立算出的应有数 | **整轮作废**：扫描没在读盘（读不到 ≠ 读到 0） |
//! | 丁臂（序号 + 显式墓碑单元）任一类 ≠ 0 | **整轮作废**：全对照都闭不上 ⇒ 模型另有 bug |
//! | 往任一臂的重建结果注入 k 处破坏，报出的 `wrong` ≠ k | **整轮作废**：比较器没有判别力 |
//! | 驻留容量算出来是 0 | **整轮作废**：没测到「消息驻留在缓冲里」这件事 |
//! | 结论方向与 D11 已定项 1 相左（例如答「不能」） | **如实记录。** 已定项 1 自陈是「在两个已知未决之上做的裁决」 |
//!
//! ## 它测的是什么、不是什么
//!
//! 测的是**一条消息的内容能不能由扫单元重新产生**，与它此刻躺在哪个节点无关。
//! 所以模型不重建整棵 Bε 树，只按 D11 已定项 2 的几何算出「全树缓冲能扣住多少条」，
//! 用它划出驻留窗口。**几何只决定窗口大小，不决定某一类消息可不可重算。**

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

// ── D11 已定项 2 的几何。改这三个数要同步改 kb（gate.d/27-format-constants.sh）──
const D11_NODE_BYTES: usize = 16384;
const D11_FANOUT: usize = 119;
const D11_BUF_MSGS: usize = 665;
/// ε 以万分比记，0.65 ⇒ 6500。
const D11_EPS_BP: usize = 6500;
// 只有几何自洽那条断言用得到它们（把 ε 反算回 119 / 665）。
#[cfg(test)] const MSG_BYTES: usize = 16;
#[cfg(test)] const PIVOT_BYTES: usize = 48;
#[cfg(test)] const NODE_HDR_BYTES: usize = 16;

/// 单元的自描述头。字段集逐字取 D18 已定项 3 的那五个。
/// `seq` 是 D18 未定项 1 的谱系重写序号——**未定**，所以它是臂的变量，不是既有事实。
/// `tomb` 是「删除也写一个单元」这条候选，同样未定。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Hdr {
    kind: u8,
    tree: u64,
    obj: u64,
    birth_gen: u64,
    anchor: u64,
    seq: Option<u64>,
    tomb: bool,
}

/// 盘上的一个单元。`pba` 不进头（D18 已定项 3 的禁放清单），
/// 但扫描的人天然知道自己读的是哪个落点——这是观测，不是权威记录。
#[derive(Clone, Copy, Debug)]
struct Unit {
    pba: u64,
    hdr: Option<Hdr>,
    /// 记账（D21 已定的权威态之一）说这个落点还活着吗。
    /// **只有乙臂的重建路径看得到它**——甲臂按 D11 已定项 4 的字面问题，只许看单元。
    live: bool,
}

type Key = (u64, u64, u64); // (tree, obj, anchor)

/// 一次重建要产出的四类消息。全部按 D8 已定项 1 的「幂等完整值」形态。
#[derive(Clone, Default, Debug)]
struct State {
    /// 映射类：逻辑身份 → 物理落点
    map: BTreeMap<Key, u64>,
    /// 反向索引类：物理落点 → 逻辑身份
    backref: BTreeMap<u64, Key>,
    /// 记账类：每棵树的活单元数（幂等完整值，不是增量 Δ）
    acct: BTreeMap<u64, u64>,
    /// 墓碑类：这些 key 必须**不在** map 里
    dead: BTreeSet<Key>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Arm {
    /// 头里带谱系重写序号（D18 未定项 1）
    seq: bool,
    /// 删除也写一个墓碑单元
    tomb_unit: bool,
    /// 重建路径可以问记账「这个落点还活着吗」（= 不再是「只从单元」）
    use_acct: bool,
    /// 被覆写 / 被删的旧副本仍然物理可读
    stale_visible: bool,
    /// 头全被抹掉（阴性对照）
    no_hdr: bool,
    /// 分配器复用了释放出来的低地址（新副本的落点比旧副本**小**）
    /// ⇒ 按物理地址扫盘时，先扫到的反而是新的。
    reuse_alloc: bool,
}

impl Arm {
    /// 甲：只从单元。**这是 D11 已定项 4 的字面问题。**
    fn jia() -> Self { Arm { seq: false, tomb_unit: false, use_acct: false, stale_visible: true, no_hdr: false, reuse_alloc: false } }
    /// 乙：单元 + 记账（D21 已定记账也是权威态，它没跟着索引一起丢）
    fn yi() -> Self { Arm { use_acct: true, ..Arm::jia() } }
    /// 丙：单元 + 谱系重写序号（D18 未定项 1 的候选）
    fn bing() -> Self { Arm { seq: true, ..Arm::jia() } }
    /// 丁：序号 + 显式墓碑单元。**全对照：这一臂必须四类全 0，否则整轮作废。**
    fn ding() -> Self { Arm { seq: true, tomb_unit: true, ..Arm::jia() } }
    /// 戊：陈旧副本不可读（旧副本一释放就物理消失）
    fn wu() -> Self { Arm { stale_visible: false, ..Arm::jia() } }
    /// 己：头全抹掉。**阴性对照：分歧必须等于全部 key。**
    fn ji() -> Self { Arm { no_hdr: true, ..Arm::jia() } }
    /// 庚：甲臂 + 分配器复用低地址。**它拆掉甲臂里那个隐藏的运气**——
    /// 单调向前分配时「按地址扫、取最后一个」恰好等于「取最新的一个」，
    /// 而那是模型的产物，不是任何条款给的保证。
    fn geng() -> Self { Arm { reuse_alloc: true, ..Arm::jia() } }
}

const ARMS: [(&str, fn() -> Arm); 7] = [
    ("jia_units_only", Arm::jia),
    ("yi_units_plus_acct", Arm::yi),
    ("bing_units_plus_seq", Arm::bing),
    ("ding_seq_plus_tombstone", Arm::ding),
    ("wu_stale_invisible", Arm::wu),
    ("ji_headers_erased", Arm::ji),
    ("geng_allocator_reuses_low_addresses", Arm::geng),
];

/// 负载参数。三个数各自独立驱动一类分歧，所以绝对值断言钉得住。
#[derive(Clone, Copy, Debug)]
struct Load {
    n_obj: u64,
    /// 覆写次数（每次覆写造一个新单元，旧的成为陈旧副本）
    n_overwrite: u64,
    /// 删除次数（删掉的是**没被覆写过**的那一段 key，两类互不重叠）
    n_delete: u64,
    n_tree: u64,
}

impl Load {
    /// 独立算出的应有单元数：建 n_obj 个 + 覆写 n_overwrite 个 + （墓碑臂）n_delete 个。
    /// **这是绝对值断言的分母**，不是从被测代码里读回来的。
    fn expected_units(&self, arm: Arm) -> u64 {
        let base = self.n_obj + self.n_overwrite + if arm.tomb_unit { self.n_delete } else { 0 };
        if arm.stale_visible { base } else { base - self.n_overwrite - self.n_delete }
    }
}

/// 造盘：按臂的规则写单元，同时算出真值状态。
fn build(l: Load, arm: Arm) -> (Vec<Unit>, State) {
    assert!(l.n_overwrite + l.n_delete <= l.n_obj, "覆写段与删除段不许重叠、不许超过对象数");
    let mut units: Vec<Unit> = Vec::new();
    let mut truth = State::default();
    // 三段互不相交的落点空间。**覆写落在哪一段是臂的变量**：
    // 单调向前 ⇒ 新副本地址更大；复用释放空间 ⇒ 新副本地址更小。
    let mut pba = 1_000_000u64;      // ① 建对象
    let mut pba_ow = 2_000_000u64;   // ② 覆写（单调向前）
    let mut pba_reuse = 999_999u64;  // ② 覆写（复用低地址）
    let mut pba_tomb = 3_000_000u64; // ③ 墓碑单元
    let mut seq = 0u64;
    let mut cur: BTreeMap<Key, u64> = BTreeMap::new();

    let mk = |l: &Load, i: u64, seq: u64, tomb: bool, arm: Arm| Hdr {
        kind: if tomb { 2 } else { 1 },
        tree: i % l.n_tree,
        obj: i,
        birth_gen: 1, // 对象出生代：覆写不改它，这正是 D18 已定项 3 字段集的形状
        anchor: 0,
        seq: if arm.seq { Some(seq) } else { None },
        tomb,
    };

    // ① 建 n_obj 个对象
    for i in 0..l.n_obj {
        seq += 1;
        let h = mk(&l, i, seq, false, arm);
        units.push(Unit { pba, hdr: Some(h), live: true });
        cur.insert((h.tree, h.obj, h.anchor), pba);
        pba += 1;
    }
    // ② 覆写前 n_overwrite 个：新单元，旧单元按臂决定留不留
    for i in 0..l.n_overwrite {
        seq += 1;
        let h = mk(&l, i, seq, false, arm);
        let k = (h.tree, h.obj, h.anchor);
        let old = cur[&k];
        for u in units.iter_mut() {
            if u.pba == old { u.live = false; }
        }
        let np = if arm.reuse_alloc { pba_reuse -= 1; pba_reuse + 1 } else { pba_ow += 1; pba_ow - 1 };
        units.push(Unit { pba: np, hdr: Some(h), live: true });
        cur.insert(k, np);
    }
    // ③ 删掉最后 n_delete 个（与覆写段不重叠）
    for i in (l.n_obj - l.n_delete)..l.n_obj {
        seq += 1;
        let h = mk(&l, i, seq, true, arm);
        let k = (h.tree, h.obj, h.anchor);
        let old = cur[&k];
        for u in units.iter_mut() {
            if u.pba == old { u.live = false; }
        }
        cur.remove(&k);
        truth.dead.insert(k);
        if arm.tomb_unit {
            units.push(Unit { pba: pba_tomb, hdr: Some(h), live: true });
            pba_tomb += 1;
        }
    }
    // 陈旧副本不可读的臂：旧副本物理消失
    if !arm.stale_visible {
        units.retain(|u| u.live);
    }
    // 阴性对照：头全抹掉
    if arm.no_hdr {
        for u in units.iter_mut() { u.hdr = None; }
    }

    for (k, p) in &cur {
        truth.map.insert(*k, *p);
        truth.backref.insert(*p, *k);
        *truth.acct.entry(k.0).or_insert(0) += 1;
    }
    (units, truth)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    units_scanned: u64,
    /// 同一个 key 有多份可读副本、且这一臂没有任何判据能分出哪份是现行值
    ambiguous: u64,
    /// 真值里有、重建里没有
    map_missing: u64,
    /// 重建里有、但指向别的落点（含「赌扫描顺序赌错」的那些）
    map_wrong: u64,
    /// 已删的 key 在重建结果里活了过来
    resurrected: u64,
    /// 反向索引条目缺 / 错
    backref_bad: u64,
    /// 记账类对不上的树数
    acct_bad: u64,
    /// 判别力自证真的注入了几处（重建结果是空集时注入不进去，见 E57 的「空集不是能重建」）
    injected: u64,
    /// 重建出来的映射条数
    rebuilt: u64,
}

impl Out {
    fn total_divergence(&self) -> u64 {
        self.map_missing + self.map_wrong + self.resurrected + self.backref_bad + self.acct_bad
    }
}

/// 只按这一臂允许看到的输入重建四类消息。
///
/// ⚠️ 没有判据可分辨同一 key 的多份副本时，**记成 ambiguous 并按「赌扫描顺序」落值**——
/// 两个数都报出来：`ambiguous` 是「重建方知道自己不知道」，`map_wrong` 是「赌错了几处」。
fn rebuild(units: &[Unit], arm: Arm) -> (State, u64, u64) {
    let mut cands: BTreeMap<Key, Vec<(u64, Option<u64>, bool)>> = BTreeMap::new(); // key → [(pba, seq, tomb)]
    let mut scanned = 0u64;
    // **按物理地址扫盘**，不是按写入顺序——扫的人手里只有盘，没有写入顺序。
    let mut by_pba: Vec<&Unit> = units.iter().collect();
    by_pba.sort_by_key(|u| u.pba);
    for u in by_pba {
        scanned += 1;
        let Some(h) = u.hdr else { continue };
        if arm.use_acct && !u.live { continue; } // 记账说它已经不是现行落点
        cands.entry((h.tree, h.obj, h.anchor)).or_default().push((u.pba, h.seq, h.tomb));
    }
    let mut st = State::default();
    let mut ambiguous = 0u64;
    for (k, v) in &cands {
        let pick = if v.len() == 1 {
            v[0]
        } else if arm.seq {
            // 序号能分出哪一份是最后写的
            *v.iter().max_by_key(|(_, s, _)| s.expect("seq 臂的每个头都该带序号")).unwrap()
        } else {
            ambiguous += 1;
            *v.last().unwrap() // 赌地址顺序：取扫到的最后一份（= 落点最大的那份）
        };
        if pick.2 {
            continue; // 墓碑：这个 key 已经死了
        }
        st.map.insert(*k, pick.0);
        st.backref.insert(pick.0, *k);
        *st.acct.entry(k.0).or_insert(0) += 1;
    }
    (st, scanned, ambiguous)
}

/// 逐条比对重建结果与真值。`corrupt` = 往重建结果里注入几处破坏（判别力自证）。
fn measure(l: Load, arm: Arm, corrupt: usize) -> Out {
    let (units, truth) = build(l, arm);
    let (mut re, scanned, ambiguous) = rebuild(&units, arm);
    // 判别力自证：把 corrupt 条重建出来的映射改到别处去，比较器必须逐条报出来。
    let rebuilt = re.map.len() as u64;
    // **只往「本来是对的」那些条目上注入**——往已经错的条目上再改一次，
    // 计数不会动，比较器会被冤枉成没有判别力（庚臂实测踩到）。
    let victims: Vec<Key> = re.map.iter()
        .filter(|(k, p)| truth.map.get(*k) == Some(*p))
        .map(|(k, _)| *k).take(corrupt).collect();
    let injected = victims.len() as u64;
    for k in victims {
        let old = re.map[&k];
        re.map.insert(k, old + 500_000);
    }

    let mut o = Out { units_scanned: scanned, ambiguous, injected, rebuilt, ..Default::default() };
    for (k, v) in &truth.map {
        match re.map.get(k) {
            None => o.map_missing += 1,
            Some(p) if p != v => o.map_wrong += 1,
            Some(_) => {}
        }
    }
    for k in &truth.dead {
        if re.map.contains_key(k) { o.resurrected += 1; }
    }
    for (p, k) in &truth.backref {
        match re.backref.get(p) {
            None => o.backref_bad += 1,
            // ⚠️ **「同一个落点指向别的 key」这一支不可达，所以它不在这里。**
            // 一个落点的主人由**那个落点自己的头**说了算：重建方给 key K 选了落点 p，
            // 说明 p 的头写着 K；真值里 p 属于 K2，说明 p 的头写着 K2。两者不能同真。
            // 2026-08-31 变异测试报它是盲区（M12 改掉计数，一个测试都没红）——
            // 查下去它是**死代码**，不是缺检查。现在写成断言：模型一旦自相矛盾就炸。
            Some(k2) => assert_eq!(k2, k, "落点 {p} 在两侧指向不同的 key，模型自相矛盾"),
        }
    }
    for t in 0..l.n_tree {
        let a = truth.acct.get(&t).copied().unwrap_or(0);
        let b = re.acct.get(&t).copied().unwrap_or(0);
        if a != b { o.acct_bad += 1; }
    }
    o
}

/// 全树缓冲能扣住多少条消息：按 D11 已定项 2 的几何算，`nleaf` 个叶。
/// **这个数只用来划驻留窗口**，不参与任何一类消息可不可重算的判定。
fn resident_capacity(nleaf: u64) -> u64 {
    let mut internal = 0u64;
    let mut n = nleaf;
    while n > 1 {
        n = n.div_ceil(D11_FANOUT as u64);
        internal += n;
    }
    internal * D11_BUF_MSGS as u64
}

const LOADS: [Load; 3] = [
    Load { n_obj: 256, n_overwrite: 64, n_delete: 32, n_tree: 4 },
    Load { n_obj: 4096, n_overwrite: 1024, n_delete: 512, n_tree: 8 },
    Load { n_obj: 16384, n_overwrite: 4096, n_delete: 2048, n_tree: 16 },
];

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config note=只扫单元头重算缓冲消息 node_bytes={D11_NODE_BYTES} eps_bp={D11_EPS_BP} \
         fanout={D11_FANOUT} buf_msgs={D11_BUF_MSGS}")));
    for nleaf in [1024u64, 8192, 32768] {
        println!("{}", em.emit_raw(&format!(
            "name=geom nleaf={nleaf} resident_capacity={}", resident_capacity(nleaf))));
    }
    for l in LOADS {
        for (name, mk) in ARMS {
            let arm = mk();
            let o = measure(l, arm, 0);
            println!("{}", em.emit_raw(&format!(
                "name=cell arm={name} n_obj={} n_ow={} n_del={} units={} expected_units={} \
                 ambiguous={} map_missing={} map_wrong={} resurrected={} backref_bad={} \
                 acct_bad={} divergence={}",
                l.n_obj, l.n_overwrite, l.n_delete, o.units_scanned, l.expected_units(arm),
                o.ambiguous, o.map_missing, o.map_wrong, o.resurrected, o.backref_bad,
                o.acct_bad, o.total_divergence())));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const L: Load = LOADS[1];

    /// **作废条款①**：扫到的单元数必须等于独立算出的应有数。
    /// 读不到 ≠ 读到 0——扫描没在读盘时，所有「分歧为 0」都是假的。
    #[test]
    fn scan_reads_exactly_the_units_that_were_written() {
        for l in LOADS {
            for (name, mk) in ARMS {
                let arm = mk();
                let o = measure(l, arm, 0);
                assert_eq!(o.units_scanned, l.expected_units(arm),
                    "{name}：扫到 {} 个单元，独立算出应有 {}", o.units_scanned, l.expected_units(arm));
                assert!(o.units_scanned > 0, "{name}：一个单元都没扫到");
            }
        }
    }

    /// **作废条款②（全对照）**：序号 + 显式墓碑单元这一臂必须四类全 0。
    /// 闭不上说明模型另有 bug，整轮作废。
    #[test]
    fn the_full_control_arm_closes_every_class_to_zero() {
        for l in LOADS {
            let o = measure(l, Arm::ding(), 0);
            assert_eq!(o.map_missing, 0);
            assert_eq!(o.map_wrong, 0);
            assert_eq!(o.resurrected, 0);
            assert_eq!(o.backref_bad, 0);
            assert_eq!(o.acct_bad, 0);
            assert_eq!(o.ambiguous, 0, "有序号就不该有分不出来的副本");
        }
    }

    /// **作废条款③：判别力自证，且对每一条臂都跑。**
    /// 只对一条臂跑等于另外五条从没过闸（test-discipline.md）。
    ///
    /// ⚠️ **注入不进去的那一臂要单独说清楚，不许算它通过**：己臂重建出的是**空集**，
    /// 没有条目可破坏 —— 这正是 E57 记下的「0 是空集造成的，不是『能重建』」那个坑。
    #[test]
    fn the_comparator_reports_injected_damage_on_every_arm() {
        for (name, mk) in ARMS {
            let arm = mk();
            let clean = measure(L, arm, 0);
            for k in [1usize, 5, 17] {
                let dirty = measure(L, arm, k);
                assert_eq!(dirty.map_wrong, clean.map_wrong + dirty.injected,
                    "{name}：注入 {} 处破坏，比较器该逐条报出来", dirty.injected);
                if arm.no_hdr {
                    assert_eq!(dirty.rebuilt, 0, "{name}：头全抹掉时重建结果本该是空集");
                    assert_eq!(dirty.injected, 0, "{name}：空集里注入不进任何破坏");
                } else {
                    assert_eq!(dirty.injected, k as u64,
                        "{name}：本该注入 {k} 处，实际只注入了 {}", dirty.injected);
                }
                assert_eq!(dirty.map_missing, clean.map_missing, "{name}：注入不该改变缺条数");
            }
        }
    }

    /// **作废条款④**：驻留容量不许是 0，否则「消息驻留在缓冲里」这件事根本没被测到。
    #[test]
    fn the_resident_window_is_not_empty() {
        for nleaf in [1024u64, 8192, 32768] {
            assert!(resident_capacity(nleaf) > 0, "nleaf={nleaf} 的驻留容量算出来是 0");
        }
        // 绝对值：1024 叶 ⇒ 内部节点 9+1=10 个（1024→9→1），扣得住 10×665=6650 条。
        assert_eq!(resident_capacity(1024), 6650);
    }

    /// **绝对值①**：D11 已定项 2 的几何自洽——ε=0.65 的 16 KiB 节点确实给出 119 / 665。
    /// 三条臂互比测不出「三条一起错」，所以这里钉死绝对值（test-discipline.md）。
    #[test]
    fn the_geometry_matches_what_d11_settled() {
        let buf_bytes = D11_NODE_BYTES * D11_EPS_BP / 10_000;
        assert_eq!(buf_bytes / MSG_BYTES, D11_BUF_MSGS, "ε=0.65 的缓冲该装 665 条消息");
        let pivot_bytes = D11_NODE_BYTES - NODE_HDR_BYTES - buf_bytes;
        assert_eq!(pivot_bytes / PIVOT_BYTES, D11_FANOUT, "剩下的字节该给出扇出 119");
        assert_eq!(D11_NODE_BYTES, 16384, "D8 已定项 2 钉的是 16 KiB");
    }

    /// **绝对值②**：甲臂分不出来的副本数**恰好等于覆写次数**，不是「有一些」。
    #[test]
    fn ambiguity_equals_exactly_the_overwrite_count() {
        for l in LOADS {
            assert_eq!(measure(l, Arm::jia(), 0).ambiguous, l.n_overwrite,
                "只从单元时，分不出现行值的 key 数该恰好等于覆写次数");
        }
    }

    /// **绝对值③**：甲臂复活的 key 数**恰好等于删除次数**。
    /// ⚠️ 这一条与序号无关——丙臂（有序号）同样全数复活，因为删除在盘上没留任何单元。
    #[test]
    fn every_deleted_key_comes_back_when_no_tombstone_unit_exists() {
        for l in LOADS {
            assert_eq!(measure(l, Arm::jia(), 0).resurrected, l.n_delete);
            assert_eq!(measure(l, Arm::bing(), 0).resurrected, l.n_delete,
                "谱系重写序号解决不了删除——盘上没有那条消息的任何痕迹");
        }
    }

    /// **绝对值④（阴性对照）**：头全抹掉 ⇒ 缺的条数等于真值里的全部 key。
    #[test]
    fn erasing_headers_loses_exactly_every_key() {
        for l in LOADS {
            let live_keys = l.n_obj - l.n_delete;
            let o = measure(l, Arm::ji(), 0);
            assert_eq!(o.map_missing, live_keys, "头没了该一条都重建不出来");
            assert_eq!(o.resurrected, 0, "什么都没重建出来时不存在复活");
        }
    }

    /// **谱系重写序号买到的和买不到的，各自钉一个绝对值。**
    /// 买到：覆写造成的歧义归零。买不到：删除造成的复活一条不减。
    #[test]
    fn the_rewrite_sequence_number_fixes_overwrites_but_not_deletes() {
        for l in LOADS {
            let bing = measure(l, Arm::bing(), 0);
            assert_eq!(bing.ambiguous, 0);
            assert_eq!(bing.map_wrong, 0, "有序号就该逐条指对落点");
            assert_eq!(bing.resurrected, l.n_delete);
        }
    }

    /// **记账那条腿买到什么**：它同时解决覆写与删除——因为「哪个落点还活着」正是它记的东西。
    /// ⚠️ 但它已经不是「只从单元」了，所以它是**答案的边界**，不是答案。
    #[test]
    fn asking_the_accounting_layer_closes_both_gaps_but_changes_the_question() {
        for l in LOADS {
            let o = measure(l, Arm::yi(), 0);
            assert_eq!(o.map_wrong, 0);
            assert_eq!(o.resurrected, 0);
            assert_eq!(o.ambiguous, 0);
            assert_eq!(o.acct_bad, 0);
        }
    }

    /// **陈旧副本不可读时，甲臂的分歧全部消失。**
    /// ⇒ 结论条件于「陈旧副本可不可见」这条前提，它必须被写下来。
    #[test]
    fn the_whole_divergence_is_conditional_on_stale_copies_being_readable() {
        for l in LOADS {
            let visible = measure(l, Arm::jia(), 0);
            let invisible = measure(l, Arm::wu(), 0);
            assert!(visible.total_divergence() > 0, "陈旧副本可读时本该有分歧");
            assert_eq!(invisible.total_divergence(), 0, "陈旧副本不可读时本该一处不差");
            // 绝对值：不可读那一臂扫到的单元数恰好等于活单元数
            assert_eq!(invisible.units_scanned, l.n_obj - l.n_delete);
        }
    }

    /// **记账类消息跟着映射类一起错**：它是从同一份「哪些单元是活的」推出来的。
    /// 绝对值：甲臂对不上的树数等于全部树数（每棵树都被复活的 key 顶高了）。
    #[test]
    fn the_accounting_class_breaks_together_with_the_mapping_class() {
        for l in LOADS {
            let o = measure(l, Arm::jia(), 0);
            assert_eq!(o.acct_bad, l.n_tree, "每棵树的活单元数都该被复活的 key 顶高");
        }
    }

    /// **甲臂那个 0 是运气，不是保证。**
    /// 单调向前分配时「按地址扫、取地址最大的一份」恰好等于「取最新的一份」；
    /// 分配器一旦复用释放出来的低地址（庚臂），同一段代码就**逐条赌错**。
    /// ⚠️ 两条绝对值一起钉：甲臂赌对 0 处、庚臂赌错的处数**恰好等于覆写次数**。
    #[test]
    fn guessing_by_address_order_is_luck_and_the_reuse_arm_shows_it() {
        for l in LOADS {
            let jia = measure(l, Arm::jia(), 0);
            let geng = measure(l, Arm::geng(), 0);
            assert_eq!(jia.map_wrong, 0, "单调向前分配时恰好赌对");
            assert_eq!(jia.ambiguous, l.n_overwrite, "但重建方并不知道自己赌对了");
            assert_eq!(geng.map_wrong, l.n_overwrite, "复用低地址时逐条赌错");
            assert_eq!(geng.ambiguous, l.n_overwrite, "歧义数与分配顺序无关");
        }
    }

    /// **反向索引类：三条臂各钉一个绝对值。**
    ///
    /// ⚠️ 这条的第一版只写了甲臂的 `backref_bad == map_wrong`，而甲臂上**两个数都是 0**——
    /// `0 == 0` 什么也没证明。2026-08-31 变异测试实测：把缺条那一支的计数改成 `+= 0`，
    /// **一个测试都没红**。互比断言旁边必须有钉绝对值的那条（test-discipline.md）。
    #[test]
    fn the_backref_class_breaks_by_exactly_these_counts() {
        for l in LOADS {
            let live_keys = l.n_obj - l.n_delete;
            assert_eq!(measure(l, Arm::jia(), 0).backref_bad, 0, "甲臂赌对了落点，反向索引不缺条");
            assert_eq!(measure(l, Arm::geng(), 0).backref_bad, l.n_overwrite,
                "庚臂赌错几处落点，反向索引就缺几条");
            assert_eq!(measure(l, Arm::ji(), 0).backref_bad, live_keys,
                "头全抹掉时反向索引一条都建不出来");
            assert_eq!(measure(l, Arm::ding(), 0).backref_bad, 0);
        }
    }
}
