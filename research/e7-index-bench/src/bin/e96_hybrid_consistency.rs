//! E96：混合架构的一致性 —— 指针位置条目当提示 + 中央映射为权威，一致性上有没有洞。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/96-混合架构的一致性.md，跑前写死）
//!
//! 持久 COW 树（叶条目 = (版本, 落点提示)），中央映射 版本→落点 为权威，占用表 落点→版本。
//! 读：提示优先——提示槽的占用者「单元头」与要读的版本匹配就返回，否则回映射多一跳。
//! 单元头两臂：hdr_key（五元组 ⇒ 只有 key）/ hdr_birth（五元组 + 块 birth ⇒ (key, birth)）。
//! 整理只改映射与占用，不碰任何叶的提示。defer：本轮放的槽下一轮才可复用。
//!
//! ## 判据（跑前写死）
//!
//! 1. 手算锚点：快照 → 搬 key 3 → 更新 key 3 复用旧槽 → 读快照 key 3：hdr_key 错读 1，hdr_birth 错读 0 多跳 1。
//! 2. 守恒：被引用的每个版本 v 满足 occupant[map[v]] == v；映射与占用互为反函数。
//! 3. 阳性对照：踩坏映射 ⇒ 审计红；对调占用 ⇒ 错读能响。
//! 4. hdr_key 错读数 == 独立扫描的「提示槽被同 key 另一版本占用」数；hdr_birth 错读恒 0。
//! 5. 按提示释放的误放数 > 0（adversarial）；按映射释放恒 0。
//! 6. 搬迁三步各截断一次：错读恒 0、守恒恒成立。
//! 7. 只报数：活根 / 快照根的多跳率随轮数。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

const L: usize = 4096;
const F: usize = 16;
const S: usize = 32768;
const N_BATCH: usize = 64;
const U_PER_ROUND: usize = 64;
const ROUNDS: usize = 200;
const SNAP_EVERY: usize = 8;
const K_SNAP: usize = 4;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if s == 0 {
            s = 0xDEAD_BEEF;
        }
        Rng(s)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Header {
    Key,
    KeyBirth,
}
impl Header {
    fn tag(self) -> &'static str {
        match self {
            Header::Key => "hdr_key",
            Header::KeyBirth => "hdr_birth",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Alloc {
    Lowest, // 最低空槽优先
    Lifo,   // 最近释放的先复用（空闲栈）
}
impl Alloc {
    fn tag(self) -> &'static str {
        match self {
            Alloc::Lowest => "alloc_lowest",
            Alloc::Lifo => "alloc_lifo",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Load {
    Uniform,
    Adversarial,
}
impl Load {
    fn tag(self) -> &'static str {
        match self {
            Load::Uniform => "uniform",
            Load::Adversarial => "adversarial",
        }
    }
}

#[derive(Clone)]
enum Node {
    Inner(Vec<u32>),
    Leaf(Vec<Entry>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    ver: u32,
    hint: u32,
}

#[derive(Clone, Copy)]
struct Version {
    key: u32,
    birth: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DlEntry {
    ver: u32,
    hint: u32,
}

#[derive(Clone)]
struct Snap {
    txg: u64,
    prev_txg: u64,
    root: u32,
    deadlist: Vec<DlEntry>,
}

#[derive(Clone)]
struct World {
    f: usize,
    l: usize,
    depth: usize,
    header: Header,
    alloc: Alloc,
    nodes: Vec<Node>,
    live_root: u32,
    snaps: Vec<Snap>,
    head_deadlist: Vec<DlEntry>,
    prev_snap_txg: u64,
    versions: Vec<Version>,
    free: BTreeSet<u32>,
    /// 释放次序（LIFO 用）：末尾是最近回空的槽。
    free_order: Vec<u32>,
    deferred: Vec<u32>,
    /// 整理的扫描游标：批次取游标起的 N 个占用槽，扫完一圈回头。
    sweep: u32,
    occupant: BTreeMap<u32, u32>,
    map_slot: BTreeMap<u32, u32>,
    txg: u64,
    /// 负臂计数：销毁快照时若按提示解落点会误放几次（只计数，不施加）。
    free_via_hint_misfree: u64,
}

impl World {
    fn new(f: usize, l: usize, s: usize, header: Header, alloc: Alloc) -> World {
        let mut depth = 0;
        let mut span = 1;
        while span < l {
            span *= f;
            depth += 1;
        }
        assert_eq!(span, l);
        let mut nodes = Vec::new();
        let mut level: Vec<u32> = (0..l / f)
            .map(|i| {
                let es: Vec<Entry> =
                    (0..f).map(|j| Entry { ver: (i * f + j) as u32, hint: (i * f + j) as u32 }).collect();
                nodes.push(Node::Leaf(es));
                (nodes.len() - 1) as u32
            })
            .collect();
        while level.len() > 1 {
            level = level
                .chunks(f)
                .map(|c| {
                    nodes.push(Node::Inner(c.to_vec()));
                    (nodes.len() - 1) as u32
                })
                .collect();
        }
        World {
            f,
            l,
            depth,
            header,
            alloc,
            nodes,
            live_root: level[0],
            snaps: Vec::new(),
            head_deadlist: Vec::new(),
            prev_snap_txg: 0,
            versions: (0..l).map(|k| Version { key: k as u32, birth: 1 }).collect(),
            free: (l as u32..s as u32).collect(),
            free_order: Vec::new(),
            deferred: Vec::new(),
            sweep: 0,
            occupant: (0..l as u32).map(|k| (k, k)).collect(),
            map_slot: (0..l as u32).map(|k| (k, k)).collect(),
            txg: 1,
            free_via_hint_misfree: 0,
        }
    }

    fn digits(&self, key: usize) -> Vec<usize> {
        let mut span = self.l;
        let mut ds = Vec::with_capacity(self.depth);
        let mut k = key;
        for _ in 0..self.depth {
            span /= self.f;
            ds.push(k / span);
            k %= span;
        }
        ds
    }

    fn alloc(&mut self) -> u32 {
        let slot = match self.alloc {
            Alloc::Lowest => *self.free.iter().next().expect("槽用尽"),
            Alloc::Lifo => loop {
                match self.free_order.pop() {
                    Some(s) if self.free.contains(&s) => break s,
                    Some(_) => continue,
                    None => break *self.free.iter().next().expect("槽用尽"),
                }
            },
        };
        self.free.remove(&slot);
        slot
    }

    /// 释放：进 defer，轮末才回空槽集。
    fn release(&mut self, slot: u32, ver: u32) {
        assert_eq!(self.occupant.remove(&slot), Some(ver));
        self.map_slot.remove(&ver);
        self.deferred.push(slot);
    }

    fn end_round(&mut self) {
        for s in std::mem::take(&mut self.deferred) {
            self.free.insert(s);
            self.free_order.push(s);
        }
    }

    /// 活头写一个 key：新版本、新槽、叶条目提示 = 新槽；kill 旧版本按 D5 规则。
    fn update(&mut self, key: usize) {
        self.txg += 1;
        let slot = self.alloc();
        let ver = self.versions.len() as u32;
        self.versions.push(Version { key: key as u32, birth: self.txg });
        self.occupant.insert(slot, ver);
        self.map_slot.insert(ver, slot);
        let ds = self.digits(key);
        let mut path = Vec::with_capacity(self.depth);
        let mut cur = self.live_root;
        for &d in &ds {
            path.push((cur, d));
            cur = match &self.nodes[cur as usize] {
                Node::Inner(cs) => cs[d],
                Node::Leaf(_) => break,
            };
        }
        let (leaf_id, idx) = *path.last().unwrap();
        let mut es = match &self.nodes[leaf_id as usize] {
            Node::Leaf(es) => es.clone(),
            _ => unreachable!(),
        };
        let old = es[idx];
        es[idx] = Entry { ver, hint: slot };
        self.nodes.push(Node::Leaf(es));
        let mut child = (self.nodes.len() - 1) as u32;
        for &(node_id, i) in path.iter().rev().skip(1) {
            let mut cs = match &self.nodes[node_id as usize] {
                Node::Inner(cs) => cs.clone(),
                _ => unreachable!(),
            };
            cs[i] = child;
            self.nodes.push(Node::Inner(cs));
            child = (self.nodes.len() - 1) as u32;
        }
        self.live_root = child;
        if self.versions[old.ver as usize].birth > self.prev_snap_txg {
            let s = self.map_slot[&old.ver];
            self.release(s, old.ver);
        } else {
            // deadlist 条目带的是叶里那个提示——正是危险所在
            self.head_deadlist.push(DlEntry { ver: old.ver, hint: old.hint });
        }
    }

    fn snapshot(&mut self) {
        self.snaps.push(Snap {
            txg: self.txg,
            prev_txg: self.prev_snap_txg,
            root: self.live_root,
            deadlist: std::mem::take(&mut self.head_deadlist),
        });
        self.prev_snap_txg = self.txg;
    }

    /// 销毁最旧者（D5 约定 B）。释放**按映射**解落点；同时数一下按提示解会误放几次。
    fn destroy_oldest(&mut self) {
        let s = self.snaps.remove(0);
        let mut merged = if let Some(next) = self.snaps.first_mut() {
            std::mem::take(&mut next.deadlist)
        } else {
            std::mem::take(&mut self.head_deadlist)
        };
        merged.extend(s.deadlist);
        let mut kept = Vec::new();
        for e in merged {
            if self.versions[e.ver as usize].birth > s.prev_txg {
                if self.occupant.get(&e.hint) != Some(&e.ver) {
                    self.free_via_hint_misfree += 1;
                }
                let slot = self.map_slot[&e.ver];
                self.release(slot, e.ver);
            } else {
                kept.push(e);
            }
        }
        if let Some(next) = self.snaps.first_mut() {
            next.deadlist = kept;
            next.prev_txg = s.prev_txg;
        } else {
            self.head_deadlist = kept;
        }
    }

    /// 整理：搬一个版本到新槽——只改映射与占用，叶提示不动。
    fn relocate(&mut self, ver: u32) {
        let old = self.map_slot[&ver];
        let new = self.alloc();
        self.occupant.insert(new, ver);
        self.map_slot.insert(ver, new);
        assert_eq!(self.occupant.remove(&old), Some(ver));
        self.deferred.push(old);
    }

    /// 单元头匹配：hdr_key 只比 key；hdr_birth 比 (key, birth) ⇒ 等价于同一个版本。
    fn header_matches(&self, occupant_ver: u32, wanted: u32) -> bool {
        match self.header {
            Header::Key => self.versions[occupant_ver as usize].key == self.versions[wanted as usize].key,
            Header::KeyBirth => occupant_ver == wanted,
        }
    }

    fn leaf_entry(&self, root: u32, key: usize) -> Entry {
        let ds = self.digits(key);
        let mut cur = root;
        for &d in &ds[..ds.len() - 1] {
            cur = match &self.nodes[cur as usize] {
                Node::Inner(cs) => cs[d],
                _ => unreachable!(),
            };
        }
        match &self.nodes[cur as usize] {
            Node::Leaf(es) => es[*ds.last().unwrap()],
            _ => unreachable!(),
        }
    }

    /// 读：提示优先，失配回映射。返回 (读到的版本, 有没有多跳)。
    fn read(&self, root: u32, key: usize) -> (u32, bool) {
        let e = self.leaf_entry(root, key);
        if let Some(&occ) = self.occupant.get(&e.hint) {
            if self.header_matches(occ, e.ver) {
                return (occ, false);
            }
        }
        let slot = self.map_slot[&e.ver];
        (self.occupant[&slot], true)
    }

    /// 读全部根的全部 key：返回 (错读数, 活根多跳数, 快照根多跳数, 快照读总数)。
    fn read_everything(&self) -> (u64, u64, u64, u64) {
        let (mut wrong, mut hop_live, mut hop_snap, mut snap_reads) = (0u64, 0u64, 0u64, 0u64);
        for k in 0..self.l {
            let e = self.leaf_entry(self.live_root, k);
            let (got, hop) = self.read(self.live_root, k);
            if got != e.ver {
                wrong += 1;
            }
            if hop {
                hop_live += 1;
            }
        }
        for s in &self.snaps {
            for k in 0..self.l {
                let e = self.leaf_entry(s.root, k);
                let (got, hop) = self.read(s.root, k);
                snap_reads += 1;
                if got != e.ver {
                    wrong += 1;
                }
                if hop {
                    hop_snap += 1;
                }
            }
        }
        (wrong, hop_live, hop_snap, snap_reads)
    }

    /// 独立扫描（不走 read）：提示槽被同 key 的**另一个**版本占着的条目数——
    /// hdr_key 臂错读的预测值。
    fn stale_same_key_entries(&self) -> u64 {
        let mut n = 0u64;
        let roots: Vec<u32> = std::iter::once(self.live_root).chain(self.snaps.iter().map(|s| s.root)).collect();
        for root in roots {
            for k in 0..self.l {
                let e = self.leaf_entry(root, k);
                if let Some(&occ) = self.occupant.get(&e.hint) {
                    if occ != e.ver && self.versions[occ as usize].key == self.versions[e.ver as usize].key {
                        n += 1;
                    }
                }
            }
        }
        n
    }

    /// 守恒审计：被引用的每个版本 occupant[map[v]] == v；映射与占用互为反函数。
    fn audit(&self) -> Result<(), String> {
        let mut referenced: BTreeSet<u32> = BTreeSet::new();
        let roots: Vec<u32> = std::iter::once(self.live_root).chain(self.snaps.iter().map(|s| s.root)).collect();
        for root in roots {
            for k in 0..self.l {
                referenced.insert(self.leaf_entry(root, k).ver);
            }
        }
        for s in &self.snaps {
            referenced.extend(s.deadlist.iter().map(|e| e.ver));
        }
        referenced.extend(self.head_deadlist.iter().map(|e| e.ver));
        for &v in &referenced {
            let slot = *self.map_slot.get(&v).ok_or(format!("ver {v} 无映射"))?;
            if self.occupant.get(&slot) != Some(&v) {
                return Err(format!("ver {v} 映射到 {slot} 但占用者不是它"));
            }
        }
        for (&slot, &v) in &self.occupant {
            if self.map_slot.get(&v) != Some(&slot) {
                return Err(format!("槽 {slot} 被 {v} 占着但映射不指回来"));
            }
        }
        if self.occupant.len() != self.map_slot.len() {
            return Err("占用数与映射数不等".into());
        }
        Ok(())
    }

    /// 整理批次：从扫描游标起取 N 个占用槽的版本，扫到末尾回头；游标推进到批次末尾之后。
    fn pick_batch(&mut self, n: usize) -> Vec<u32> {
        let mut out: Vec<(u32, u32)> = self.occupant.range(self.sweep..).take(n).map(|(&s, &v)| (s, v)).collect();
        if out.len() < n {
            out.extend(self.occupant.range(..self.sweep).take(n - out.len()).map(|(&s, &v)| (s, v)));
        }
        self.sweep = out.last().map(|&(s, _)| s + 1).unwrap_or(0);
        out.into_iter().map(|(_, v)| v).collect()
    }
}

struct RoundStats {
    wrong: u64,
    hop_live_pct: f64,
    hop_snap_pct: f64,
}

fn run(header: Header, alloc: Alloc, load: Load, seed: u64, rounds: usize) -> (World, Vec<RoundStats>, u64) {
    let mut w = World::new(F, L, S, header, alloc);
    let mut rng = Rng::new(seed);
    let mut stats = Vec::new();
    let mut last_moved: Vec<u32> = Vec::new();
    let mut wrong_total = 0u64;
    for r in 1..=rounds {
        for i in 0..U_PER_ROUND {
            let key = match load {
                Load::Uniform => rng.below(L),
                // 对抗负载：一半更新打刚被搬走的 key
                Load::Adversarial if i % 2 == 0 && !last_moved.is_empty() => {
                    w.versions[last_moved[i / 2 % last_moved.len()] as usize].key as usize
                }
                Load::Adversarial => rng.below(L),
            };
            w.update(key);
        }
        let batch = w.pick_batch(N_BATCH);
        for &v in &batch {
            w.relocate(v);
        }
        last_moved = batch;
        if r % SNAP_EVERY == 0 {
            w.snapshot();
            if w.snaps.len() > K_SNAP {
                w.destroy_oldest();
            }
        }
        w.end_round();
        w.audit().unwrap_or_else(|e| panic!("轮 {r} 守恒破了：{e}"));
        let (wrong, hop_live, hop_snap, snap_reads) = w.read_everything();
        // 判据 4：错读数与独立扫描逐轮相等（hdr_key）；hdr_birth 恒 0
        match header {
            Header::Key => assert_eq!(wrong, w.stale_same_key_entries(), "轮 {r} 错读数与独立扫描分叉"),
            Header::KeyBirth => assert_eq!(wrong, 0, "轮 {r} hdr_birth 出现错读"),
        }
        wrong_total += wrong;
        stats.push(RoundStats {
            wrong,
            hop_live_pct: 100.0 * hop_live as f64 / L as f64,
            hop_snap_pct: if snap_reads > 0 { 100.0 * hop_snap as f64 / snap_reads as f64 } else { 0.0 },
        });
    }
    (w, stats, wrong_total)
}

/// 判据 6：一次搬迁在三步之后各截断一次，恢复后**不新增**错读（相对截断前的基线）、守恒恒成立。
/// 基线差值而不是绝对 0：hdr_key 臂的世界里本来就可能有提示落到同 key 复用槽的错读，那不是截断造成的。
/// 步骤：① 写新副本（分配 + 占用）② 发布映射 ③ defer 放旧槽。
/// 截断在 ① 之后：未发布的分配回滚（新槽回空、占用去掉）；截断在 ② 之后：旧槽由 defer 放。
fn crash_truncation(base: &World, ver: u32) -> Vec<(usize, i64)> {
    let (baseline, _, _, _) = base.read_everything();
    let mut out = Vec::new();
    for cut in 1..=3usize {
        let mut w = base.clone();
        let old = w.map_slot[&ver];
        let new = w.alloc();
        w.occupant.insert(new, ver); // ① 新副本落盘（占用表暂时两处都有它）
        if cut >= 2 {
            w.map_slot.insert(ver, new); // ② 发布映射
        }
        if cut >= 3 {
            w.occupant.remove(&old); // ③ 旧槽进 defer
            w.deferred.push(old);
        }
        // 恢复
        if cut == 1 {
            // 未发布的分配回滚
            w.occupant.remove(&new);
            w.free.insert(new);
        } else if cut == 2 {
            // 映射已发布：旧副本按 defer 放
            w.occupant.remove(&old);
            w.deferred.push(old);
        }
        w.end_round();
        w.audit().unwrap_or_else(|e| panic!("截断 {cut} 后守恒破了：{e}"));
        let (wrong, _, _, _) = w.read_everything();
        out.push((cut, wrong as i64 - baseline as i64));
    }
    out
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config l={L} f={F} s={S} n_batch={N_BATCH} u_per_round={U_PER_ROUND} rounds={ROUNDS} snap_every={SNAP_EVERY} k_snap={K_SNAP} model=counting file_ops=0 seeds=5"
        ))
    );
    for header in [Header::Key, Header::KeyBirth] {
        for alloc in [Alloc::Lowest, Alloc::Lifo] {
        for load in [Load::Uniform, Load::Adversarial] {
            for seed in 0..5u64 {
                let (mut w, stats, wrong_total) = run(header, alloc, load, seed, ROUNDS);
                let last = stats.last().unwrap();
                let mid = &stats[ROUNDS / 2 - 1];
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=hybrid hdr={} alloc={} load={} seed={seed} wrong_total={wrong_total} wrong_last={} hop_live_mid={:.1} hop_live_last={:.1} hop_snap_mid={:.1} hop_snap_last={:.1} misfree_via_hint={} misfree_via_map=0",
                        header.tag(),
                        alloc.tag(),
                        load.tag(),
                        last.wrong,
                        mid.hop_live_pct,
                        last.hop_live_pct,
                        mid.hop_snap_pct,
                        last.hop_snap_pct,
                        w.free_via_hint_misfree
                    ))
                );
                if seed == 0 {
                    // 判据 6：拿轮末世界里一个被快照引用的版本做三步截断
                    let victim = w.pick_batch(1)[0];
                    let cuts = crash_truncation(&w, victim);
                    let s: Vec<String> = cuts.iter().map(|(c, wr)| format!("{c}:{wr}")).collect();
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=crash hdr={} alloc={} load={} seed=0 wrong_delta_after_cut={}",
                            header.tag(),
                            alloc.tag(),
                            load.tag(),
                            s.join(",")
                        ))
                    );
                    assert!(cuts.iter().all(|(_, wr)| *wr == 0), "截断新增了错读");
                }
            }
        }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(header: Header) -> World {
        World::new(2, 8, 32, header, Alloc::Lowest)
    }

    /// 造判据 1 的场景：快照 → 搬 key 3（槽 3 → 8）→ 轮末放旧槽 → 更新 key 3（新版本复用槽 3）。
    fn hand_scenario(header: Header) -> World {
        let mut w = small(header);
        w.snapshot(); // S1 @ txg 1
        w.relocate(3);
        assert_eq!(w.map_slot[&3], 8);
        w.end_round(); // 槽 3 回空
        w.update(3); // 新版本取最低空槽 = 3
        let live = w.leaf_entry(w.live_root, 3);
        assert_eq!(live.hint, 3, "新版本必须复用了槽 3，场景才成立");
        w
    }

    /// **判据 1（hdr_key）**：快照读 key 3 走提示槽 3，占用者是同 key 的新版本，五元组匹配 ⇒ 静默错读。
    #[test]
    fn hand_key_header_silently_reads_newer_version() {
        let w = hand_scenario(Header::Key);
        let s1 = w.snaps[0].root;
        let want = w.leaf_entry(s1, 3).ver;
        let (got, hop) = w.read(s1, 3);
        assert_eq!(want, 3);
        assert_ne!(got, want, "读到了另一个版本");
        assert!(!hop, "提示「匹配」了所以没多跳——错就错在这里");
        let (wrong, _, _, _) = w.read_everything();
        assert_eq!(wrong, 1);
        assert_eq!(w.stale_same_key_entries(), 1);
    }

    /// **判据 1（hdr_birth）**：同场景，头带 birth ⇒ 提示失配 ⇒ 回映射 ⇒ 读对，多跳 1。
    #[test]
    fn hand_birth_header_falls_back_to_map() {
        let w = hand_scenario(Header::KeyBirth);
        let s1 = w.snaps[0].root;
        let (got, hop) = w.read(s1, 3);
        assert_eq!(got, 3);
        assert!(hop);
        let (wrong, hop_live, hop_snap, _) = w.read_everything();
        assert_eq!((wrong, hop_live, hop_snap), (0, 0, 1));
    }

    /// **判据 5 手算**：deadlist 条目带的是叶提示（槽 3）；S1 销毁时按提示解会放掉新版本正占的槽 3，
    /// 按映射解放的是槽 8。误放计数恰 1，且真正放掉的是 8。
    #[test]
    fn free_via_hint_misfrees_reused_slot() {
        let mut w = hand_scenario(Header::KeyBirth);
        // hand_scenario 里 update(3) 把旧 ver 3（birth 1 ≤ prev_snap 1）推进了活头 deadlist，提示 = 3
        assert_eq!(w.head_deadlist, vec![DlEntry { ver: 3, hint: 3 }]);
        w.snapshot(); // S2 接管
        w.destroy_oldest(); // 毁 S1：ver 3 释放
        assert_eq!(w.free_via_hint_misfree, 1);
        assert!(w.deferred.contains(&8), "按映射解落点放的是 8");
        assert!(w.occupant.contains_key(&3), "槽 3 仍归新版本");
        w.end_round();
        w.audit().unwrap();
    }

    /// **判据 3**：踩坏一条映射 ⇒ 审计必须红；对调两个槽的占用者 ⇒ 错读计数必须响。
    #[test]
    fn audit_and_wrong_counter_have_teeth() {
        let mut w = small(Header::KeyBirth);
        w.map_slot.insert(3, 9);
        assert!(w.audit().is_err());
        let mut w = small(Header::KeyBirth);
        let a = w.occupant.remove(&3).unwrap();
        let b = w.occupant.remove(&4).unwrap();
        w.occupant.insert(3, b);
        w.occupant.insert(4, a);
        let (wrong, _, _, _) = w.read_everything();
        assert_eq!(wrong, 2);
        // 反向核对也要有牙：一个没人引用的版本占着槽 20 而映射说它在 21——
        // 正向只查被引用的版本、长度又相等，只有反向核对能红
        let mut w = small(Header::KeyBirth);
        w.occupant.insert(20, 99);
        w.map_slot.insert(99, 21);
        assert_eq!(w.occupant.len(), w.map_slot.len());
        assert!(w.audit().is_err());
    }

    /// 更新必须刷新自己那条叶条目的提示：活根读刚写的 key 不许多跳，提示 == 映射。
    #[test]
    fn update_refreshes_own_hint() {
        let mut w = small(Header::KeyBirth);
        w.update(3);
        let e = w.leaf_entry(w.live_root, 3);
        assert_eq!(e.hint, w.map_slot[&e.ver]);
        assert_eq!(e.hint, 8);
        let (got, hop) = w.read(w.live_root, 3);
        assert_eq!(got, e.ver);
        assert!(!hop, "刚写的 key 走提示就该命中");
    }

    /// **判据 6 手算**：8 对象、快照后搬 key 3，三步各截断一次：不新增错读、守恒恒成立
    /// （hdr_key 与 hdr_birth 都跑；hdr_key 的基线本来是 0，所以差值 0 也就是绝对 0）。
    #[test]
    fn crash_truncation_is_clean() {
        for h in [Header::Key, Header::KeyBirth] {
            let mut w = small(h);
            w.snapshot();
            let cuts = crash_truncation(&w, 3);
            assert_eq!(cuts, vec![(1, 0), (2, 0), (3, 0)]);
            let (wrong, _, _, _) = w.read_everything();
            assert_eq!(wrong, 0);
        }
    }

    /// **判据 4 全流程**：LIFO 分配 + adversarial 负载下 hdr_key 的错读总数必须 > 0（危险可达），
    /// hdr_birth 恒 0（run 内逐轮断言），两臂快照读的多跳率都 > 0。
    #[test]
    fn adversarial_load_reaches_the_hazard() {
        let (_, stats, wrong_key) = run(Header::Key, Alloc::Lifo, Load::Adversarial, 0, 24);
        assert!(wrong_key > 0, "对抗负载没造出同 key 复用");
        assert!(stats.last().unwrap().hop_snap_pct > 0.0);
        let (_, stats_b, wrong_birth) = run(Header::KeyBirth, Alloc::Lifo, Load::Adversarial, 0, 24);
        assert_eq!(wrong_birth, 0);
        assert!(stats_b.last().unwrap().hop_snap_pct > 0.0);
    }

    /// 扫描游标：连续两批不重叠且推进，扫完一圈回头。
    #[test]
    fn sweep_cursor_advances_and_wraps() {
        let mut w = small(Header::KeyBirth);
        let b1 = w.pick_batch(3);
        let b2 = w.pick_batch(3);
        let b3 = w.pick_batch(3);
        assert_eq!(b1, vec![0, 1, 2]);
        assert_eq!(b2, vec![3, 4, 5]);
        assert_eq!(b3, vec![6, 7, 0], "扫到末尾回头");
    }

    /// LIFO 分配先复用最近回空的槽：同一轮先搬 2（去 8）再搬 5（去 9），轮末回空次序是 2、5
    /// ⇒ 下一次分配拿 5（最低空槽政策会拿 2），再下一次拿 2，栈空了才退回最低空槽 10。
    #[test]
    fn lifo_reuses_most_recently_freed() {
        let mut w = World::new(2, 8, 32, Header::KeyBirth, Alloc::Lifo);
        w.relocate(2);
        w.relocate(5);
        assert_eq!((w.map_slot[&2], w.map_slot[&5]), (8, 9));
        w.end_round();
        assert_eq!(w.alloc(), 5, "最近回空的先复用");
        assert_eq!(w.alloc(), 2);
        assert_eq!(w.alloc(), 10, "栈空后退回最低空槽");
    }

    /// 整理只改映射不碰提示：搬完后叶条目的提示还是旧槽。
    #[test]
    fn relocate_leaves_hints_alone() {
        let mut w = small(Header::KeyBirth);
        w.relocate(5);
        assert_eq!(w.leaf_entry(w.live_root, 5).hint, 5);
        assert_eq!(w.map_slot[&5], 8);
        w.end_round();
        w.audit().unwrap();
    }
}
