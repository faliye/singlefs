//! 复跑：`m3-retire-r1-opus-s2-model sweep <段起>:<段止>`（每段 = 一组配置 × 一个种子）、`demo <名>`、`configs`、`determinism`。
//! 线程数读 S2_THREADS（默认 6），只影响挂钟、不影响输出（每段历史各自一个种子，输出按段的序号排）。
mod model;
mod demos;
use model::*;
use std::sync::{Arc, Mutex};

pub fn configs() -> Vec<Config> {
    let mut v = Vec::new();
    let geometries = [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest), ("ring_random", Geometry::RingRandom)];
    // 形状：(名, 叶容量, 退役容器容量, 起始空 inode 数, 用户动作数)
    let shapes = [("cap4", 4usize, 3usize, 30u64, 800u32), ("cap6", 6, 3, 40, 800), ("cap12", 12, 5, 80, 1200)];
    // 族 r：随机历史，M3 写路径；回退的目标两种挑法（全部候选里均匀挑 / 千分之 700 只挑不在现行谱系父链上的）。
    for (sname, cap, rcap, initial, actions) in shapes {
        for (gname, g) in geometries {
            for (oname, off) in [("uniform", 0u64), ("off_chain_700", 700)] {
                for seed in 1..=16u64 {
                    v.push(Config {
                        name: format!("family=r shape={sname} geometry={gname} rollback_pick={oname} seed={seed}"),
                        seed: seed * 1_000_003 + cap as u64 * 7 + off, geometry: g, ring: 24, leaf_cap: cap, ret_cap: rcap,
                        initial_inodes: initial, create_max: 4, actions, p_create: 330, p_rollback: 60, p_crash: 20, p_lost_root: 8,
                        crash_k_max: 16, p_rollback_off_chain: off, m1_actions: 0,
                    });
                }
            }
        }
    }
    // 族 up（K4）：前 400 次用户动作照 M1（`crates/` 今天只删空即摘）写，之后换 M3：升级之后一次发布里写几条退役。
    for (gname, g) in [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest)] {
        for seed in 1..=8u64 {
            v.push(Config {
                name: format!("family=up shape=cap6 geometry={gname} rollback_pick=uniform seed={seed}"),
                seed: seed * 2_000_003 + 6, geometry: g, ring: 24, leaf_cap: 6, ret_cap: 3,
                initial_inodes: 60, create_max: 4, actions: 800, p_create: 250, p_rollback: 30, p_crash: 20, p_lost_root: 8,
                crash_k_max: 16, p_rollback_off_chain: 0, m1_actions: 400,
            });
        }
    }
    v
}

pub fn run_history(cfg: Config) -> World {
    let mut w = World::new(cfg.clone());
    w.inject = false;
    w.create(1);
    w.create(cfg.initial_inodes);
    w.inject = true;
    let mut focus_leaf = 0usize;
    for step in 0..cfg.actions {
        if cfg.m1_actions > 0 && step == cfg.m1_actions { w.m3 = true; w.stats.add("switched_from_m1_to_m3", 1); }
        if step % 100 == 0 { focus_leaf = w.rng.below(w.cur.leaves.len().max(1) as u64) as usize; }
        let live: Vec<u64> = w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect();
        if w.rng.chance(cfg.p_rollback) { w.rollback(); continue; }
        if live.is_empty() || w.rng.below(1000) < cfg.p_create {
            let n = 1 + w.rng.below(cfg.create_max);
            w.create(n);
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

pub fn lines_of(w: &World) -> Vec<String> {
    let mut out = Vec::new();
    for ((reader, tree), c) in &w.stats.cells {
        out.push(format!(
            "S2 name=cell {} reader={reader} tree={tree} scans={} r_peak={} m_peak={} r_scans={} m_scans={} first={}",
            w.cfg.name, c.scans, c.r_peak, c.m_peak, c.r_scans, c.m_scans, c.first.clone().unwrap_or_else(|| "none".into()).replace(' ', "_")
        ));
    }
    let counts: Vec<String> = w.stats.counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
    out.push(format!("S2 name=counts {} final_txg={} instances={} disk_slots={} {}", w.cfg.name, w.cur.txg, w.max_inst, w.disk.len(), counts.join(" ")));
    out
}

pub fn threads() -> usize { std::env::var("S2_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(6).max(1) }

/// 把 `jobs` 分给几个工作线程跑，交回按下标排好的结果（线程数只影响挂钟）。
pub fn par_map<T: Send + 'static, R: Send + 'static>(jobs: Vec<T>, f: fn(T) -> R) -> Vec<R> {
    let n = jobs.len();
    let jobs = Arc::new(Mutex::new(jobs.into_iter().enumerate().collect::<Vec<_>>()));
    let results: Arc<Mutex<Vec<Option<R>>>> = Arc::new(Mutex::new((0..n).map(|_| None).collect()));
    let mut handles = Vec::new();
    for _ in 0..threads() {
        let (jobs, results) = (Arc::clone(&jobs), Arc::clone(&results));
        handles.push(std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some((i, x)) = job else { break };
            let r = f(x);
            results.lock().unwrap()[i] = Some(r);
        }));
    }
    for h in handles { h.join().expect("工作线程 panic"); }
    let r = Arc::try_unwrap(results).ok().expect("线程都收了").into_inner().unwrap();
    r.into_iter().map(|x| x.expect("每件都跑完")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str).unwrap_or("") {
        "sweep" => {
            let all = configs();
            let (a, b) = args.get(2).and_then(|r| r.split_once(':'))
                .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap().min(all.len()))).unwrap_or((0, all.len()));
            let t0 = std::time::Instant::now();
            let out = par_map(all[a..b].to_vec(), |c| lines_of(&run_history(c)));
            let mut emitted = 0;
            for lines in out { for l in lines { println!("{l}"); emitted += 1; } }
            eprintln!("histories={} elapsed_seconds={:.1}", b - a, t0.elapsed().as_secs_f64());
            println!("S2 name=done histories={} emitted={emitted}", b - a);
        }
        "configs" => println!("{}", configs().len()),
        "determinism" => {
            let picks: Vec<Config> = configs().into_iter().enumerate().filter(|(i, _)| i % 9 == 0).map(|(_, c)| c).collect();
            let once = par_map(picks.clone(), |c| lines_of(&run_history(c)));
            let twice = par_map(picks.clone(), |c| lines_of(&run_history(c)));
            let same = once == twice;
            println!("S2 name=determinism histories={} two_runs_identical={same}", picks.len());
            if !same { std::process::exit(1); }
        }
        "demo" => demos::run(args.get(2).map(String::as_str).unwrap_or("all")),
        _ => { eprintln!("用法：sweep <起>:<止> | configs | determinism | demo <名>"); std::process::exit(2); }
    }
}
