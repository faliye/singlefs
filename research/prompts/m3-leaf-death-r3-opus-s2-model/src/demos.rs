//! 手造的几段历史：固定前缀与故障，之后的用户动作放开扫（菜单里每一样都可以是下一步，长度 0–depth 的全部序列）。
//! K2（族 Z*）：Z 在「记录被借走 / 被吸收之后对象再变短」「多对象交错变短与删除」「截短到 0 再长回来」上；
//! K1（族 S*）：崩溃留下被抛弃的单元、根环转过去之后清扫删行，再放开扫，三个候选并排。
use crate::model::*;

fn base(borrow: BorrowArm, zero: bool) -> World {
    let cfg = Config {
        name: String::new(), seed: 7, geometry: Geometry::Wide, ring: 24, leaf_cap: 6, ret_cap: 3, borrow,
        initial_inodes: 17, create_max: 3, actions: 0, p_create: 0, p_write: 0, p_append: 0, p_rollback: 0,
        p_crash: 0, p_lost_root: 0, crash_k_max: 10, ret_from_r_old: false, zero_persist: zero, tomb_cap: 3,
        p_truncate: 0, p_truncate_zero: 0, p_hole: 0, p_focus_moved: 0, p_focus_regrow: 0, p_sweep: 0,
    };
    let mut w = World::new(cfg);
    w.inject = false;
    w.create(1);
    w.overwrite(1, 1, false);
    w.create(17); // 叶 [1..6] [7..12] [13..18]
    for (x, u) in [(7, 2), (8, 3), (9, 1), (10, 2), (12, 1), (14, 2), (3, 2), (4, 3)] { w.overwrite(x, u, false); }
    w
}

fn live(w: &World) -> Vec<u64> {
    w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect()
}

fn has_data(w: &World, x: u64) -> bool { w.cur.data.get(&x).is_some_and(|a| !a.is_empty()) }

/// 最近被挪过叶、还活着的那个对象（没有就取活着的带数据的最小号）。
fn moved(w: &World) -> Option<u64> {
    let l = live(w);
    w.recently_moved.iter().rev().copied().find(|x| l.contains(x)).or_else(|| l.iter().copied().find(|x| has_data(w, *x)))
}

/// 前缀：从第一片叶删起（不删带数据的 3、4 号与 1 号），删到 `counter` 那一项第一次加一。交回它前一版在 line 里的下标。
fn until(w: &mut World, counter: &str, leaf: usize) -> Option<usize> {
    let before = w.stats.counts.get(counter).copied().unwrap_or(0);
    for _ in 0..40 {
        let l = w.cur.leaves.get(leaf)?;
        let x = l.recs.iter().find(|r| r.nlink > 0 && !has_data(w, r.ino))
            .or_else(|| l.recs.iter().find(|r| r.nlink > 0))?.ino;
        w.unlink(x);
        if w.stats.counts.get(counter).copied().unwrap_or(0) > before { return Some(w.line.len() - 2); }
    }
    None
}

fn act(w: &mut World, a: &str) {
    let l = live(w);
    let m = moved(w);
    match a {
        "create3" => { w.create(3); }
        "unlink_min" => { if let Some(&x) = l.first() { w.unlink(x); } }
        "unlink_max" => { if let Some(&x) = l.last() { w.unlink(x); } }
        "unlink_leaf1_min" => { if let Some(x) = w.cur.leaves.get(1).and_then(|lf| lf.recs.iter().find(|r| r.nlink > 0)).map(|r| r.ino) { w.unlink(x); } }
        "overwrite_min3" => { if let Some(&x) = l.first() { w.overwrite(x, 3, false); } }
        "rollback_oldest" => { if let Some(&i) = w.rollback_candidates().first() { w.rollback_to(i); } }
        "crash_next" => { w.force_crash = true; w.create(1); }
        "lost_root" => { if w.line.len() > 2 { w.recover(Recovery::Lost { back: 1, jump: false }); } }
        "trunc_moved_1" => { if let Some(x) = m { if w.cur.data.get(&x).is_some_and(|a| a.len() >= 2) { w.truncate(x, 1); } } }
        "trunc_moved_0" => { if let Some(x) = m { if has_data(w, x) { w.truncate(x, 0); } } }
        "append_moved_2" => { if let Some(x) = m { w.append(x, 2); } }
        "overwrite_moved_1" => { if let Some(x) = m { w.overwrite(x, 1, false); } }
        "unlink_moved" => { if let Some(x) = m { w.unlink(x); } }
        "trunc_8_1" => { if l.contains(&8) && w.cur.data.get(&8).is_some_and(|a| a.len() >= 2) { w.truncate(8, 1); } }
        "trunc_8_0" => { if l.contains(&8) && has_data(w, 8) { w.truncate(8, 0); } }
        "append_8_1" => { if l.contains(&8) { w.append(8, 1); } }
        "append_8_4" => { if l.contains(&8) { w.append(8, 4); } }
        "overwrite_8_2" => { if l.contains(&8) { w.overwrite(8, 2, false); } }
        "unlink_8" => { if l.contains(&8) { w.unlink(8); } }
        "trunc_7_1" => { if l.contains(&7) && w.cur.data.get(&7).is_some_and(|a| a.len() >= 2) { w.truncate(7, 1); } }
        "unlink_7" => { if l.contains(&7) { w.unlink(7); } }
        "unlink_10" => { if l.contains(&10) { w.unlink(10); } }
        "trunc_10_0" => { if l.contains(&10) && has_data(w, 10) { w.truncate(10, 0); } }
        "append_10_3" => { if l.contains(&10) { w.append(10, 3); } }
        other => panic!("菜单里没有 {other}"),
    }
}

fn suffixes(menu: &[&'static str], depth: usize) -> Vec<Vec<&'static str>> {
    let mut all: Vec<Vec<&'static str>> = vec![vec![]];
    let mut layer: Vec<Vec<&'static str>> = vec![vec![]];
    for _ in 0..depth {
        let mut next = Vec::new();
        for s in &layer { for a in menu { let mut t = s.clone(); t.push(*a); next.push(t); } }
        all.extend(next.iter().cloned());
        layer = next;
    }
    all
}

const SHOWN: [(&str, &str); 16] = [
    ("C1t", "inode"), ("C1t", "data_Z"), ("C1t", "data_RG"), ("C1t_uniform", "inode"), ("C1n", "inode"),
    ("TSZc", "inode_out"), ("TSZc", "data_Z_out"), ("TSZu", "inode_out"), ("TSZc", "inode"),
    ("C2Hold", "inode"), ("C2Hold", "data_Z"), ("C1t_noP2", "inode"), ("TSZc_noP2", "inode_out"), ("C2Hold_noP2", "inode"),
    ("C1t_noLin", "inode"), ("C2Hold_noLin", "inode"),
];

fn report(scenario: &str, suffix: &[&str], w: &World) {
    let s = if suffix.is_empty() { "none".to_string() } else { suffix.join("+") };
    for (reader, tree) in SHOWN {
        let Some(c) = w.stats.cells.get(&(reader.to_string(), tree.to_string())) else { continue };
        println!("R3 name=demo scenario={scenario} suffix={s} reader={reader} tree={tree} scans={} r_peak={} m_peak={} first={}",
            c.scans, c.r_peak, c.m_peak, c.first.clone().unwrap_or_else(|| "none".into()).replace(' ', "_"));
    }
    let keys = ["crash_points", "crash_states_enumerated", "sweeps", "sweep_rows_deleted", "sweep_units_wiped", "truncations", "truncations_to_zero", "borrows", "absorbs", "drops", "zero_versions", "rollbacks"];
    let counts: Vec<String> = keys.iter().map(|k| format!("{k}={}", w.stats.counts.get(*k).copied().unwrap_or(0))).collect();
    println!("R3 name=demo_counts scenario={scenario} suffix={s} {}", counts.join(" "));
}

type Prefix = fn(&mut World) -> bool;

fn z_borrow(w: &mut World) -> bool { until(w, "borrows", 0).is_some() }
/// 前缀：删 2、5、6、3（第一片剩 1、4），再删 11、12（第二片剩 7..10，四条都带数据）：删 12 那次第二片被第一片吸收，
/// 7..10 带着数据挪到第一片。
fn z_absorb(w: &mut World) -> bool {
    for x in [2, 5, 6, 3, 11, 12] { w.unlink(x); }
    w.stats.counts.get("absorbs").copied().unwrap_or(0) >= 1
}
fn z_interleave(w: &mut World) -> bool { w.truncate(8, 2) }
fn z_zero(w: &mut World) -> bool { w.truncate(8, 0) }
/// 删空那次的根丢了，挂载续做重做删空时崩（在飞的单元写逐子集枚举），根环转过去、清扫删行。
fn s_drop(w: &mut World) -> bool {
    if until(w, "drops", 1).is_none() { return false; }
    w.force_crash = true;
    w.recover(Recovery::Lost { back: 1, jump: false });
    w.idle(26) && { w.sweep(); true }
}
fn s_zero(w: &mut World) -> bool {
    if until(w, "zero_versions", 1).is_none() { return false; }
    w.force_crash = true;
    w.recover(Recovery::Lost { back: 1, jump: false });
    w.idle(26) && { w.sweep(); true }
}
/// unlink 意图那次崩（墓碑容器的那一版被抛弃、txg 重发），根环转过去、清扫删行。
fn s_intent(w: &mut World) -> bool {
    w.force_crash = true;
    w.unlink(9);
    w.idle(26) && { w.sweep(); true }
}
/// 回退跨过删空，回退之后下一次发布崩（重发 txg），根环转过去、清扫删行。
fn s_rollback(w: &mut World) -> bool {
    let Some(before) = until(w, "drops", 1) else { return false };
    if !w.rollback_to(before) { return false; }
    w.force_crash = true;
    w.create(1);
    w.idle(26) && { w.sweep(); true }
}

pub fn run(which: &str) {
    let menu_z: &[&'static str] = &["trunc_moved_1", "trunc_moved_0", "append_moved_2", "overwrite_moved_1", "unlink_moved", "unlink_max", "rollback_oldest", "crash_next"];
    let menu_i: &[&'static str] = &["trunc_7_1", "trunc_8_0", "append_8_1", "unlink_7", "unlink_10", "trunc_10_0", "append_10_3", "rollback_oldest", "crash_next"];
    let menu_0: &[&'static str] = &["append_8_1", "append_8_4", "trunc_8_0", "overwrite_8_2", "unlink_8", "rollback_oldest", "crash_next", "lost_root"];
    let menu_s: &[&'static str] = &["create3", "unlink_min", "unlink_max", "unlink_leaf1_min", "overwrite_min3", "rollback_oldest", "crash_next"];
    let scenarios: Vec<(&str, BorrowArm, bool, Prefix, &[&'static str], usize)> = vec![
        ("z_borrow_then_shrink_n1", BorrowArm::N1, false, z_borrow, menu_z, 3),
        ("z_absorb_then_shrink", BorrowArm::NoBorrow, false, z_absorb, menu_z, 3),
        ("z_interleaved_shrink_and_unlink", BorrowArm::NoBorrow, false, z_interleave, menu_i, 3),
        ("z_truncate_to_zero_then_regrow", BorrowArm::NoBorrow, false, z_zero, menu_0, 3),
        ("s_crash_in_the_drop_then_sweep", BorrowArm::NoBorrow, false, s_drop, menu_s, 2),
        ("s_crash_in_the_drop_then_sweep_n2", BorrowArm::N2, false, s_drop, menu_s, 2),
        ("s_crash_in_the_zero_version_then_sweep", BorrowArm::NoBorrow, true, s_zero, menu_s, 2),
        ("s_crash_in_the_unlink_intent_then_sweep", BorrowArm::NoBorrow, false, s_intent, menu_s, 2),
        ("s_crash_in_the_unlink_intent_then_sweep_zero", BorrowArm::NoBorrow, true, s_intent, menu_s, 2),
        ("s_rollback_across_drop_crash_then_sweep", BorrowArm::NoBorrow, false, s_rollback, menu_s, 2),
    ];
    for (name, arm, zero, prefix, menu, depth) in scenarios {
        if which != "all" && which != name { continue; }
        for suffix in suffixes(menu, depth) {
            let mut w = base(arm, zero);
            if !prefix(&mut w) { println!("R3 name=demo scenario={name} prefix_failed=true"); break; }
            w.check();
            for a in &suffix { act(&mut w, a); }
            report(name, &suffix, &w);
        }
    }
}
