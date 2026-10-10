//! 复跑：`m3-leaf-death-r3-opus-s2-model sweep <段起>:<段止>`（每段 = 一组配置 × 一个种子），或 `demo <名>`、`trace <下标>`、`configs`、`determinism`。
//! 线程数读 R2_THREADS（默认 6），只影响挂钟、不影响输出（每段历史各自一个种子，输出按段的序号排）。
mod model;
use model::*;
use std::sync::{Arc, Mutex};

fn configs() -> Vec<Config> {
    let mut v = Vec::new();
    let geometries = [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest), ("ring_random", Geometry::RingRandom)];
    // 形状：(名, 叶容量, 退役 / 墓碑容器容量, 起始空 inode 数, 一次至多建几个, 用户动作数, 种子数 k1, 种子数 z)
    let shapes = [("cap6", 6usize, 3usize, 40u64, 4u64, 600u32, 8u64, 8u64), ("cap233", 233, 959, 700, 20, 1500, 4, 4)];
    // 族 k1：三个候选并排（候选一 C1*、TSZ 在摘叶的写路径上，候选二在零条目一直留的写路径上），每次挂载末尾千分之 500 跑清扫。
    // 借法臂 (名, 借法, 零条目一直留)
    let arms = [("no_borrow", BorrowArm::NoBorrow, false), ("n1", BorrowArm::N1, false), ("n2", BorrowArm::N2, false),
                ("zero_no_borrow", BorrowArm::NoBorrow, true), ("zero_n1", BorrowArm::N1, true)];
    for (sname, cap, rcap, initial, cmax, actions, seeds_k1, _) in shapes {
        let (pc, pw, pa, pt, pt0) = if cap == 6 { (150, 150, 100, 40, 20) } else { (60, 80, 40, 20, 10) };
        for (gname, g) in geometries {
            for (aname, a, zero) in arms {
                let retirement_arms: &[(&str, bool)] = if zero { &[("none", false)] } else { &[("carried", false), ("r_old", true)] };
                for &(rname, r_old) in retirement_arms {
                    for seed in 1..=seeds_k1 {
                        v.push(Config {
                            name: format!("family=k1 shape={sname} geometry={gname} borrow={aname} retirement_at_rollback={rname} seed={seed}"),
                            seed: seed * 1_000_003 + cap as u64 + 17, geometry: g, ring: 24, leaf_cap: cap, ret_cap: rcap, borrow: a,
                            initial_inodes: initial, create_max: cmax, actions, p_create: pc, p_write: pw, p_append: pa,
                            p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8, ret_from_r_old: r_old,
                            zero_persist: zero, tomb_cap: rcap, p_truncate: pt, p_truncate_zero: pt0, p_hole: 0,
                            p_focus_moved: 300, p_focus_regrow: 500, p_sweep: 500,
                        });
                    }
                }
            }
        }
    }
    // 族 z：K2 的三组新历史（记录被借走 / 被吸收之后对象再变短、多对象交错变短与删除、截短到 0 再长回来）。
    for (sname, cap, rcap, initial, cmax, actions, _, seeds_z) in shapes {
        let (pc, pw, pa, pt, pt0) = if cap == 6 { (120, 80, 150, 200, 100) } else { (60, 60, 150, 200, 100) };
        for (gname, g) in geometries {
            for (aname, a) in [("no_borrow", BorrowArm::NoBorrow), ("n1", BorrowArm::N1)] {
                for seed in 1..=seeds_z {
                    v.push(Config {
                        name: format!("family=z shape={sname} geometry={gname} borrow={aname} retirement_at_rollback=carried seed={seed}"),
                        seed: seed * 2_000_003 + cap as u64 + 29, geometry: g, ring: 24, leaf_cap: cap, ret_cap: rcap, borrow: a,
                        initial_inodes: initial, create_max: cmax, actions, p_create: pc, p_write: pw, p_append: pa,
                        p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8, ret_from_r_old: false,
                        zero_persist: false, tomb_cap: rcap, p_truncate: pt, p_truncate_zero: pt0, p_hole: 0,
                        p_focus_moved: 600, p_focus_regrow: 700, p_sweep: 300,
                    });
                }
            }
        }
    }
    // 族 holes（阳性对照）：打洞让「文件没有洞」那条前提为假，Z 必须中；装置看不看得见 Z 出错。
    for (gname, g) in [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest)] {
        for seed in 1..=4u64 {
            v.push(Config {
                name: format!("family=holes shape=cap6 geometry={gname} borrow=no_borrow retirement_at_rollback=carried seed={seed}"),
                seed: seed * 3_000_017 + 6, geometry: g, ring: 24, leaf_cap: 6, ret_cap: 3, borrow: BorrowArm::NoBorrow,
                initial_inodes: 40, create_max: 4, actions: 600, p_create: 120, p_write: 80, p_append: 100,
                p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8, ret_from_r_old: false,
                zero_persist: false, tomb_cap: 3, p_truncate: 150, p_truncate_zero: 50, p_hole: 80,
                p_focus_moved: 300, p_focus_regrow: 500, p_sweep: 300,
            });
        }
    }
    v
}

pub fn run_history(cfg: Config) -> World { run_history_with(cfg, false) }
pub fn run_history_traced(cfg: Config) -> World { run_history_with(cfg, true) }
fn run_history_with(cfg: Config, traced: bool) -> World {
    let mut w = World::new(cfg.clone());
    w.record_history = traced;
    // 新池新建文件：1 号带一个数据单元；再一次建 initial 个空 inode。
    w.inject = false;
    w.create(1);
    w.overwrite(1, 1, false);
    w.create(cfg.initial_inodes);
    w.inject = true;
    let mut focus_leaf = 0usize;
    for step in 0..cfg.actions {
        if step % 150 == 0 { focus_leaf = w.rng.below(w.cur.leaves.len().max(1) as u64) as usize; }
        let live: Vec<u64> = w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect();
        if w.rng.chance(cfg.p_rollback) { w.rollback(); continue; }
        let roll = w.rng.below(1000);
        let with_data: Vec<u64> = live.iter().copied().filter(|x| w.cur.data.get(x).is_some_and(|a| !a.is_empty())).collect();
        let c1 = cfg.p_create;
        let c2 = c1 + cfg.p_write;
        let c3 = c2 + cfg.p_append;
        let c4 = c3 + cfg.p_truncate;
        let c5 = c4 + cfg.p_truncate_zero;
        let c6 = c5 + cfg.p_hole;
        if live.is_empty() || roll < c1 {
            let n = 1 + w.rng.below(cfg.create_max);
            w.create(n);
        } else if roll < c2 {
            let x = live[w.rng.below(live.len() as u64) as usize];
            let u = 1 + w.rng.below(8);
            let same = w.rng.chance(300);
            w.overwrite(x, u, same);
        } else if roll < c3 {
            let target = w.regrow_target.filter(|t| live.contains(t));
            let x = match target {
                Some(t) if w.rng.chance(cfg.p_focus_regrow) => { w.regrow_target = None; w.stats.add("regrows_of_an_object_truncated_to_zero", 1); t }
                _ => live[w.rng.below(live.len() as u64) as usize],
            };
            let m = 1 + w.rng.below(4);
            w.append(x, m);
        } else if roll < c5 {
            if with_data.is_empty() { continue; }
            let moved: Vec<u64> = w.recently_moved.iter().copied().filter(|x| with_data.contains(x)).collect();
            let x = if !moved.is_empty() && w.rng.chance(cfg.p_focus_moved) {
                w.stats.add("shrinks_of_an_object_whose_record_was_moved_to_another_leaf", 1);
                moved[w.rng.below(moved.len() as u64) as usize]
            } else { with_data[w.rng.below(with_data.len() as u64) as usize] };
            if roll < c4 {
                let len = w.cur.data[&x].len() as u64;
                if len < 2 { continue; }
                let keep = 1 + w.rng.below(len - 1);
                w.truncate(x, keep);
            } else {
                w.truncate(x, 0);
                w.regrow_target = Some(x);
            }
        } else if roll < c6 {
            let x = live[w.rng.below(live.len() as u64) as usize];
            let more = 1 + w.rng.below(3);
            w.extend_with_hole(x, more);
        } else {
            let focused: Vec<u64> = w.cur.leaves.get(focus_leaf.min(w.cur.leaves.len().saturating_sub(1)))
                .map(|l| l.recs.iter().filter(|r| r.nlink > 0).map(|r| r.ino).collect()).unwrap_or_default();
            let x = if !focused.is_empty() && w.rng.chance(700) { focused[w.rng.below(focused.len() as u64) as usize] }
                    else { live[w.rng.below(live.len() as u64) as usize] };
            w.unlink(x);
        }
    }
    w
}

fn lines_of(w: &World) -> Vec<String> {
    let mut out = Vec::new();
    for ((reader, tree), c) in &w.stats.cells {
        out.push(format!(
            "R2 name=cell {} reader={reader} tree={tree} scans={} r_peak={} m_peak={} r_scans={} m_scans={} first={}",
            w.cfg.name, c.scans, c.r_peak, c.m_peak, c.r_scans, c.m_scans,
            c.first.clone().unwrap_or_else(|| "none".into()).replace(' ', "_")
        ));
    }
    let counts: Vec<String> = w.stats.counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
    out.push(format!("R2 name=counts {} final_txg={} instances={} disk_slots={} {}", w.cfg.name, w.cur.txg, w.max_inst, w.disk.len(), counts.join(" ")));
    out
}

fn run_jobs(jobs: Vec<Config>) -> Vec<Vec<String>> {
    let threads: usize = std::env::var("R2_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(6).max(1);
    let n = jobs.len();
    let jobs = Arc::new(Mutex::new(jobs.into_iter().enumerate().collect::<Vec<_>>()));
    let results: Arc<Mutex<Vec<Option<Vec<String>>>>> = Arc::new(Mutex::new(vec![None; n]));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let jobs = Arc::clone(&jobs);
        let results = Arc::clone(&results);
        handles.push(std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some((i, cfg)) = job else { break };
            let w = run_history(cfg);
            results.lock().unwrap()[i] = Some(lines_of(&w));
        }));
    }
    for h in handles { h.join().expect("工作线程 panic"); }
    let r = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    r.into_iter().map(|x| x.expect("每段都跑完")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("selftest");
    match mode {
        "sweep" => {
            let all = configs();
            let (a, b) = args.get(2).and_then(|r| r.split_once(':'))
                .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap().min(all.len()))).unwrap_or((0, all.len()));
            let t0 = std::time::Instant::now();
            let out = run_jobs(all[a..b].to_vec());
            let mut emitted = 0;
            for lines in out { for l in lines { println!("{l}"); emitted += 1; } }
            eprintln!("histories={} elapsed_seconds={:.1}", b - a, t0.elapsed().as_secs_f64());
            println!("R2 name=done histories={} emitted={emitted}", b - a);
        }
        "configs" => println!("{}", configs().len()),
        "trace" => {
            let i: usize = args.get(2).and_then(|x| x.parse().ok()).expect("trace <下标>");
            let cfg = configs()[i].clone();
            println!("R2 name=trace {}", cfg.name);
            let w = run_history_traced(cfg);
            let cut = w.history.iter().position(|l| l.starts_with("FIRST")).map_or(w.history.len(), |p| w.history.len().min(p + 400));
            for l in &w.history[..cut] { println!("{l}"); }
        }
        "determinism" => {
            let all = configs();
            // 每 8 段取一段。
            let picks: Vec<Config> = all.iter().enumerate().filter(|(i, _)| i % 8 == 0).map(|(_, c)| c.clone()).collect();
            let once = run_jobs(picks.clone());
            let twice = run_jobs(picks.clone());
            let same = once == twice;
            println!("R2 name=determinism histories={} two_runs_identical={same}", picks.len());
            if !same { std::process::exit(1); }
        }
        "demo" => demos::run(args.get(2).map(String::as_str).unwrap_or("all")),
        _ => { eprintln!("用法：sweep <起>:<止> | configs | determinism | demo <名>"); std::process::exit(2); }
    }
}

mod demos;
