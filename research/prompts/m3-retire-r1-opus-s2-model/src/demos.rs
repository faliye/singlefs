//! 手造的两组：`double`（K2 / K3 / K5：删空 → 回退 → 再退役同一身份 → 回退到被前一次回退抛弃的那一段，用户动作放开扫）与
//! `cascade`（K4：M1 写出来的树换 M3 之后，一次 unlink 最后一批写几条退役）。不注入崩溃与丢根，结果只由动作序列定。
use crate::model::*;
use crate::par_map;
use std::collections::BTreeMap;

fn base(geometry: Geometry, name: &str, cap: usize) -> Config {
    Config {
        name: name.to_string(), seed: 7, geometry, ring: 24, leaf_cap: cap, ret_cap: 3, initial_inodes: 0, create_max: 1, actions: 0,
        p_create: 0, p_rollback: 0, p_crash: 0, p_lost_root: 0, crash_k_max: 16, p_rollback_off_chain: 0, m1_actions: 0,
    }
}

/// 前缀（固定）：建 1 个、再建 11 个（叶容量 4：叶 1 装 1..4、叶 5 装 5..8、叶 9 装 9..12）；unlink 12、11、10、9（M3 下不并不借，
/// 第四次删空叶 9、摘掉、写一条退役）；再空发布两次。交回这个世界（统计清零）与被退役的身份 C、它的死亡 txg。
pub fn prefix(geometry: Geometry) -> (World, Ident, Txg) {
    let mut w = World::new(base(geometry, "double", 4));
    w.inject = false;
    w.create(1);
    w.create(11);
    let c = w.cur.leaves[2].id;
    assert_eq!((c.num, w.cur.leaves.len()), (9, 3), "前缀：三片叶，第三片容器号 9");
    for x in [12, 11, 10, 9] { w.unlink(x); }
    assert!(w.cur.leaves.iter().all(|l| l.id != c), "前缀：叶 9 删空摘掉");
    let death = w.cur.ret_records().iter().find(|r| r.who == c).expect("叶 9 有退役记录").death;
    w.idle(2);
    w.stats = Stats::default();
    (w, c, death)
}

#[derive(Clone, Copy, Debug)]
pub enum Op { Unlink(u64), Create, Idle }

fn options(w: &World) -> Vec<Op> {
    let mut v: Vec<Op> = w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| Op::Unlink(r.ino)).collect();
    v.push(Op::Create);
    v.push(Op::Idle);
    v
}

fn apply(w: &mut World, op: Op) { match op { Op::Unlink(x) => { w.unlink(x); } Op::Create => { w.create(1); } Op::Idle => { w.idle(1); } } }

#[derive(Default)]
pub struct Tally { pub runs: u64, pub c_retired_twice: u64, pub cells: BTreeMap<String, (u64, u64, Option<String>)> }

impl Tally {
    fn record(&mut self, w: &World, c: Ident, desc: &str) {
        self.runs += 1;
        if w.cur.ret_records().iter().filter(|r| r.who == c).count() >= 2 { self.c_retired_twice += 1; }
        for ((reader, tree), cell) in &w.stats.cells {
            let e = self.cells.entry(format!("reader={reader} tree={tree}")).or_default();
            if cell.r_scans > 0 { e.0 += 1; }
            if cell.m_scans > 0 { e.1 += 1; }
            if (cell.r_scans > 0 || cell.m_scans > 0) && e.2.is_none() { e.2 = Some(format!("{desc}@{}", cell.first.clone().unwrap_or_default())); }
        }
    }
    fn merge(&mut self, o: Tally) {
        self.runs += o.runs;
        self.c_retired_twice += o.c_retired_twice;
        for (k, (r, m, f)) in o.cells { let e = self.cells.entry(k).or_default(); e.0 += r; e.1 += m; if e.2.is_none() { e.2 = f; } }
    }
}

pub const SUFFIX_DEPTH: usize = 3;

/// 一件：第一次回退取前缀的第 `i` 个候选；之后用户动作逐个放开（unlink 任一活号、建 1 个、空发布，至多 SUFFIX_DEPTH 步），
/// 每个后缀末尾既判「不再回退」，也逐个判第二次回退到那时的每一个候选（含被第一次回退抛弃的那一段里的根）再空发布一次。
fn double_job((geometry, i): (Geometry, usize)) -> (Tally, Tally) {
    let (p, c, _) = prefix(geometry);
    let (mut single, mut t) = (Tally::default(), Tally::default());
    let mut w1 = p.clone();
    let rb1_txg = w1.line[i].txg;
    w1.rollback_to(i);
    let mut stack: Vec<(World, Vec<Op>)> = vec![(w1, Vec::new())];
    while let Some((s, ops)) = stack.pop() {
        let desc = format!("rb1->txg{rb1_txg}_ops={ops:?}").replace(' ', "");
        single.record(&s, c, &format!("{desc}_rb2=none"));
        for j in s.rollback_candidates() {
            let mut u = s.clone();
            let rb2_txg = u.line[j].txg;
            u.rollback_to(j);
            u.idle(1);
            t.record(&u, c, &format!("{desc}_rb2->txg{rb2_txg}"));
        }
        if ops.len() < SUFFIX_DEPTH {
            for op in options(&s) {
                let mut child = s.clone();
                apply(&mut child, op);
                let mut o = ops.clone();
                o.push(op);
                stack.push((child, o));
            }
        }
    }
    (single, t)
}

pub fn double(geometry: Geometry, gname: &str) -> Vec<String> {
    let (p, c, death) = prefix(geometry);
    let jobs: Vec<(Geometry, usize)> = p.rollback_candidates().into_iter().map(|i| (geometry, i)).collect();
    let rb1_candidates = jobs.len();
    let (mut single, mut total) = (Tally::default(), Tally::default());
    for (s, t) in par_map(jobs, double_job) { single.merge(s); total.merge(t); }
    let mut out = Vec::new();
    for (kind, tally) in [("single", single), ("double", total)] {
        out.push(format!("S2 name={kind}_prefix geometry={gname} retired_identity={}/{} death_txg={death} prefix_txg={} rb1_candidates={rb1_candidates} suffix_depth={SUFFIX_DEPTH} runs={} runs_with_c_retired_twice={}",
            c.num, c.birth, p.cur.txg, tally.runs, tally.c_retired_twice));
        for (k, (r, m, f)) in tally.cells {
            out.push(format!("S2 name={kind} geometry={gname} {k} runs={} r_runs={r} m_runs={m} first={}", tally.runs, f.unwrap_or_else(|| "none".into())));
        }
    }
    out
}

/// 重放一条 (第一次回退的候选下标, 后缀, 第二次回退的候选下标) 并开着历史记录；交回世界。
fn replay(geometry: Geometry, i: usize, ops: &[Op], j: Option<usize>) -> World {
    let (mut w, _, _) = prefix(geometry);
    w.record_history = true;
    w.rollback_to(i);
    for op in ops { apply(&mut w, *op); }
    if let Some(j) = j { w.rollback_to(j); w.idle(1); }
    w
}

/// 找某个读法第一次打中（复活或误判死亡）的最短一条（第一次回退按候选次序、后缀按长度、第二次回退按候选次序），重放并列出：
/// 历史、那一版的谱系父链、退役树里点名 C 的每条记录（在不在链上）、C 的叶在盘上的每一版（过不过 P2、在不在链上）。
pub fn trace(geometry: Geometry, reader: &str, allow_rb2: bool) -> Vec<String> {
    let (p, c, _) = prefix(geometry);
    let hit = |w: &World| w.stats.cells.get(&(reader.to_string(), "inode".to_string())).is_some_and(|x| x.r_scans + x.m_scans > 0);
    let mut found: Option<(usize, Vec<Op>, Option<usize>)> = None;
    // 先按后缀长度、再按第一次回退的候选次序找：交回的是最短的后缀。
    let mut levels: Vec<(usize, Vec<(World, Vec<Op>)>)> = p.rollback_candidates().into_iter()
        .map(|i| { let mut w1 = p.clone(); w1.rollback_to(i); (i, vec![(w1, Vec::new())]) }).collect();
    'outer: for _ in 0..=SUFFIX_DEPTH {
      for (i, level) in levels.iter_mut() {
        let i = *i;
        {
            let mut next_level = Vec::new();
            for (s, ops) in level.iter() {
                if hit(s) { found = Some((i, ops.clone(), None)); break 'outer; }
                for j in if allow_rb2 { s.rollback_candidates() } else { Vec::new() } {
                    let mut u = s.clone();
                    u.rollback_to(j);
                    u.idle(1);
                    if hit(&u) { found = Some((i, ops.clone(), Some(j))); break 'outer; }
                }
                for op in options(s) { let mut ch = s.clone(); apply(&mut ch, op); let mut o = ops.clone(); o.push(op); next_level.push((ch, o)); }
            }
            *level = next_level;
        }
      }
    }
    let Some((i, ops, j)) = found else { return vec![format!("S2 name=trace reader={reader} allow_second_rollback={allow_rb2} found=none")]; };
    let w = replay(geometry, i, &ops, j);
    let mut out = vec![format!("S2 name=trace reader={reader} allow_second_rollback={allow_rb2} rb1_index={i} ops={ops:?} rb2_index={j:?}").replace(", ", ",")];
    out.extend(w.history.iter().map(|l| format!("  {l}")));
    let chain = w.cur.chain();
    let mut ch: Vec<Txg> = chain.iter().copied().filter(|t| *t > 0).collect();
    ch.sort_unstable();
    out.push(format!("  current txg={} lineage={:?} chain={:?}", w.cur.txg, w.cur.lineage, ch));
    out.push(format!("  live leaves now: {:?}", w.cur.leaves.iter().map(|l| (l.id.num, l.id.birth, l.recs.iter().map(|r| r.ino).collect::<Vec<_>>())).collect::<Vec<_>>()));
    for r in w.cur.ret_records() {
        out.push(format!("  retirement record who={}/{} death={} on_chain={} last_write={} last_write_on_chain={}", r.who.num, r.who.birth, r.death, chain.contains(&r.death), r.last_write, chain.contains(&r.last_write)));
    }
    for (slot, u) in w.disk.iter().enumerate() {
        let Some(u) = u.as_deref() else { continue };
        if let Payload::Leaf { id, recs } = &u.payload {
            if *id == c { out.push(format!("  leaf C version slot={slot} birth={} inst={} published={} on_chain={} records={:?}", u.birth, u.inst, w.published(u), chain.contains(&u.birth), recs.iter().map(|r| (r.ino, r.nlink)).collect::<Vec<_>>())); }
        }
    }
    let cell = &w.stats.cells[&(reader.to_string(), "inode".to_string())];
    out.push(format!("  reader={reader} first_hit={} r_peak={} m_peak={}", cell.first.clone().unwrap_or_default(), cell.r_peak, cell.m_peak));
    out
}

/// K4：叶容量 6，先照 M1 写（只删空即摘）：建 60 个（10 片满叶），每片 unlink 到只剩 `keep` 条；换 M3，再 unlink 一条
/// （keep = 1：删空第二片；keep = 2：从第一片删一条）。交回那一次最后一批发布写了几条退役、写了几个退役容器。
pub fn cascade(keep: usize) -> Vec<String> {
    let mut cfg = base(Geometry::Wide, "cascade", 6);
    cfg.m1_actions = 1;
    let mut w = World::new(cfg);
    w.inject = false;
    w.create(1);
    w.create(59);
    let doomed: Vec<u64> = w.cur.leaves.iter().flat_map(|l| l.recs[keep..].iter().map(|r| r.ino).collect::<Vec<_>>()).collect();
    for x in doomed { w.unlink(x); }
    let sizes_before: Vec<usize> = w.cur.leaves.iter().map(|l| l.recs.len()).collect();
    let records_before: usize = w.cur.rets.iter().map(|c| c.recs.len()).sum();
    w.m3 = true;
    w.stats = Stats::default();
    let victim = if keep == 1 { w.cur.leaves[1].recs[0].ino } else { w.cur.leaves[0].recs[keep - 1].ino };
    w.unlink(victim);
    let sizes_after: Vec<usize> = w.cur.leaves.iter().map(|l| l.recs.len()).collect();
    vec![format!("S2 name=cascade leaf_cap=6 retirement_container_cap=3 keep={keep} unlinked={victim} sizes_before={sizes_before:?} retirement_records_before={records_before} sizes_after={sizes_after:?} retirement_records_in_the_last_batch_publish={} retirement_containers_written_in_that_publish={} absorbs={} drops={}",
        w.stats.counts.get("retirement_records_in_one_publish_peak").copied().unwrap_or(0),
        w.stats.counts.get("retirement_containers_written_in_one_publish_peak").copied().unwrap_or(0),
        w.stats.counts.get("absorbs").copied().unwrap_or(0), w.stats.counts.get("drops").copied().unwrap_or(0)).replace(", ", ",")]
}

pub fn run(name: &str) {
    let geos = [(Geometry::Wide, "wide"), (Geometry::RingLowest, "ring_lowest")];
    if name == "double" || name == "all" {
        for (g, gn) in geos { for l in double(g, gn) { println!("{l}"); } }
    }
    if name == "trace" || name == "all" {
        for allow_rb2 in [false, true] {
            for reader in ["first", "over", "lastw", "nofilter", "both"] { for l in trace(Geometry::RingLowest, reader, allow_rb2) { println!("{l}"); } }
        }
    }
    if name == "cascade" || name == "all" {
        for keep in [1, 2] { for l in cascade(keep) { println!("{l}"); } }
    }
}
