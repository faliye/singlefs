//! 手造的几段历史：固定前缀与故障，故障之后的用户动作放开扫（长度 0–2 的全部序列，菜单 7 样）。
use crate::model::*;

fn base(borrow: BorrowArm) -> World {
    let cfg = Config {
        name: String::new(), seed: 7, geometry: Geometry::Wide, ring: 24, leaf_cap: 6, ret_cap: 3, borrow,
        initial_inodes: 17, create_max: 3, actions: 0, p_create: 0, p_write: 0, p_append: 0, p_rollback: 0,
        p_crash: 0, p_lost_root: 0, crash_k_max: 8, ret_from_r_old: false,
    };
    let mut w = World::new(cfg);
    w.inject = false;
    w.create(1);
    w.overwrite(1, 1, false);
    w.create(17); // 叶 [1..6] [7..12] [13..18]
    for (x, u) in [(7, 2), (8, 3), (9, 1), (10, 2), (12, 1), (14, 2)] { w.overwrite(x, u, false); }
    w
}

fn live(w: &World) -> Vec<u64> {
    w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect()
}

/// 前缀：从第二片叶删起，删到 `counter` 那一项第一次加一（这次发布就是要跨过的那一次），交回它前一版在 line 里的下标。
fn until_structural(w: &mut World, counter: &str) -> Option<usize> {
    let before = w.stats.counts.get(counter).copied().unwrap_or(0);
    for _ in 0..40 {
        // 合并要两片都变小：删第二、三片里记录多的那一片；别的（删空、借）只删第二片。
        let pick = if counter == "absorbs" && w.cur.leaves.len() > 2 && w.cur.leaves[2].recs.len() > w.cur.leaves[1].recs.len() { 2 } else { 1 };
        let leaf = w.cur.leaves.get(pick)?;
        let x = leaf.recs.iter().find(|r| r.nlink > 0)?.ino;
        w.unlink(x);
        if w.stats.counts.get(counter).copied().unwrap_or(0) > before {
            return Some(w.line.len() - 2);
        }
    }
    None
}

const MENU: [&str; 7] = ["create3", "unlink_min", "unlink_max", "unlink_leaf1_min", "overwrite_min3", "append_max2", "rollback_oldest"];

fn act(w: &mut World, a: &str) {
    let l = live(w);
    match a {
        "create3" => { w.create(3); }
        "unlink_min" => { if let Some(&x) = l.first() { w.unlink(x); } }
        "unlink_max" => { if let Some(&x) = l.last() { w.unlink(x); } }
        "unlink_leaf1_min" => {
            if let Some(x) = w.cur.leaves.get(1).and_then(|lf| lf.recs.iter().find(|r| r.nlink > 0)).map(|r| r.ino) { w.unlink(x); }
        }
        "overwrite_min3" => { if let Some(&x) = l.first() { w.overwrite(x, 3, false); } }
        "append_max2" => { if let Some(&x) = l.last() { w.append(x, 2); } }
        "rollback_oldest" => { if let Some(&i) = w.rollback_candidates().first() { w.rollback_to(i); } }
        _ => unreachable!(),
    }
}

fn suffixes() -> Vec<Vec<&'static str>> {
    let mut v: Vec<Vec<&'static str>> = vec![vec![]];
    for a in MENU { v.push(vec![a]); }
    for a in MENU { for b in MENU { v.push(vec![a, b]); } }
    v
}

const SHOWN: [(&str, &str); 9] = [("S", "inode"), ("C1n", "inode"), ("C1n_uniform", "inode"), ("C1t_uniform", "inode"), ("C1t", "inode"), ("C1t", "data_Z"), ("C1t", "data_RG"), ("C1t_noP2", "inode"), ("C1t_noLin", "inode")];

fn report(scenario: &str, suffix: &[&str], w: &World) {
    let s = if suffix.is_empty() { "none".to_string() } else { suffix.join("+") };
    for (reader, tree) in SHOWN {
        let c = w.stats.cells.get(&(reader.to_string(), tree.to_string())).cloned().unwrap_or_default();
        println!("R2 name=demo scenario={scenario} suffix={s} reader={reader} tree={tree} scans={} r_peak={} m_peak={} first={}",
            c.scans, c.r_peak, c.m_peak, c.first.unwrap_or_else(|| "none".into()).replace(' ', "_"));
    }
}

pub fn run(which: &str) {
    let scenarios: [(&str, BorrowArm, &str, &str); 7] = [
        ("rollback_across_drop", BorrowArm::NoBorrow, "drops", "rollback"),
        ("rollback_across_absorb", BorrowArm::NoBorrow, "absorbs", "rollback"),
        ("rollback_across_n1_borrow", BorrowArm::N1, "borrows", "rollback"),
        ("rollback_across_n2_borrow", BorrowArm::N2, "borrows", "rollback"),
        ("crash_in_the_drop_publish", BorrowArm::NoBorrow, "drops", "crash"),
        ("lost_root_of_the_n2_borrow_readable_but_unusable", BorrowArm::N2, "borrows", "lost_jump"),
        ("lost_three_roots_ending_at_the_absorb", BorrowArm::NoBorrow, "absorbs", "lost_three"),
    ];
    for (name, arm, counter, fault) in scenarios {
        if which != "all" && which != name { continue; }
        for suffix in suffixes() {
            let mut w = base(arm);
            let Some(before) = until_structural(&mut w, counter) else { println!("R2 name=demo scenario={name} prefix_failed=true"); break; };
            match fault {
                "rollback" => { w.stats = Stats::default(); w.rollback_to(before); }
                "crash" => {
                    // 那次结构性发布的根丢了（回到它前一版），挂载续做重做它时强制崩溃（在飞的单元写逐子集枚举），再恢复、再续做。
                    w.stats = Stats::default();
                    w.force_crash = true;
                    w.recover(Recovery::Lost { back: 1, jump: false });
                }
                "lost_jump" => { w.stats = Stats::default(); w.recover(Recovery::Lost { back: 1, jump: true }); }
                _ => { w.stats = Stats::default(); w.recover(Recovery::Lost { back: 3, jump: false }); }
            }
            for a in &suffix { act(&mut w, a); }
            report(name, &suffix, &w);
        }
    }
}
