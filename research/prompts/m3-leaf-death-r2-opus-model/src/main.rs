//! 复跑：`m3-leaf-death-r2-opus-model sweep <批号起>:<批号止>`（每批 = 一组配置 × 种子），或 `demo <名>`、`selftest`。
//! 线程数读 R2_THREADS（默认 6），只影响挂钟、不影响输出（每段历史各自一个种子，输出按段的序号排）。
mod model;
use model::*;
use std::sync::{Arc, Mutex};

fn configs() -> Vec<Config> {
    let mut v = Vec::new();
    let geometries = [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest), ("ring_random", Geometry::RingRandom)];
    let arms = [("no_borrow", BorrowArm::NoBorrow), ("n1", BorrowArm::N1), ("n2", BorrowArm::N2)];
    // (名, 叶容量, 退役容器容量, 起始空 inode 数, 一次至多建几个, 用户动作数, 建‰, 覆盖写‰, 顺序写‰)
    let shapes = [("cap233", 233usize, 959usize, 700u64, 20u64, 1500u32, 60u64, 80u64, 40u64), ("cap6", 6, 3, 40, 4, 600, 150, 150, 100)];
    for (sname, cap, rcap, initial, cmax, actions, pc, pw, pa) in shapes {
        for (gname, g) in geometries {
            for (aname, a) in arms {
                for (rname, r_old) in [("carried", false), ("r_old", true)] {
                for seed in 1..=16u64 {
                    v.push(Config {
                        name: format!("shape={sname} geometry={gname} borrow={aname} retirement_at_rollback={rname} seed={seed}"), seed: seed * 1_000_003 + cap as u64,
                        geometry: g, ring: 24, leaf_cap: cap, ret_cap: rcap, borrow: a, initial_inodes: initial, create_max: cmax,
                        actions, p_create: pc, p_write: pw, p_append: pa, p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8, ret_from_r_old: r_old,
                    });
                }
                }
            }
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
        if live.is_empty() || roll < cfg.p_create {
            let n = 1 + w.rng.below(cfg.create_max);
            w.create(n);
        } else if roll < cfg.p_create + cfg.p_write {
            let x = live[w.rng.below(live.len() as u64) as usize];
            let u = 1 + w.rng.below(8);
            let same = w.rng.chance(300);
            w.overwrite(x, u, same);
        } else if roll < cfg.p_create + cfg.p_write + cfg.p_append {
            let x = live[w.rng.below(live.len() as u64) as usize];
            let m = 1 + w.rng.below(4);
            w.append(x, m);
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
            // 叶容量 6 那 288 段全取，叶容量 233 那一半每 37 段取一段。
            let picks: Vec<Config> = all.iter().enumerate().filter(|(i, c)| c.leaf_cap == 6 || i % 37 == 0).map(|(_, c)| c.clone()).collect();
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
