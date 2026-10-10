//! 复跑：`m3-leaf-death-r3-opus-model sweep <起>:<止>`（每段 = 一组配置 × 种子）、`configs`、`determinism <起>:<止>`、`clone`。
//! 线程数读 R3_THREADS（默认 6），只影响挂钟、不影响输出（每段历史各自一个种子，输出按段的序号排）。
mod clone;
mod model;
use model::*;
use std::sync::{Arc, Mutex};

pub fn configs() -> Vec<Config> {
    let mut v = Vec::new();
    let geometries = [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest), ("ring_random", Geometry::RingRandom)];
    let paths = [
        ("ret_carried", WritePath::RetCarried, BorrowArm::NoBorrow), ("ret_carried", WritePath::RetCarried, BorrowArm::N1), ("ret_carried", WritePath::RetCarried, BorrowArm::N2),
        ("ret_r_old", WritePath::RetROld, BorrowArm::NoBorrow), ("ret_r_old", WritePath::RetROld, BorrowArm::N1), ("ret_r_old", WritePath::RetROld, BorrowArm::N2),
        ("stubs", WritePath::Stubs, BorrowArm::NoBorrow), ("stubs", WritePath::Stubs, BorrowArm::N1),
    ];
    // (名, 叶容量, 退役容器容量, 墓碑容器容量, 起始空 inode 数, 一次至多建几个, 用户动作数, 建‰, 写‰, 顺序写‰)
    let shapes = [("cap233", 233usize, 959usize, 815usize, 700u64, 20u64, 1500u32, 60u64, 80u64, 40u64), ("cap6", 6, 3, 4, 40, 4, 600, 150, 150, 100)];
    // (名, 截短‰, 挑最近被挪过的‰, 挑盯着的那片‰, w2, 清扫, 清扫只删行不抹单元（阳性对照）)
    let variants = [("plain", 0u64, 0u64, 0u64, false, false, false), ("zhist", 400, 400, 400, false, false, false), ("w2", 0, 0, 0, true, false, false),
                    ("sweep", 0, 0, 0, false, true, false), ("w2sweep", 0, 0, 0, true, true, false), ("sweep_without_erasing", 0, 0, 0, false, true, true)];
    for (sname, cap, rcap, tcap, initial, cmax, actions, pc, pw, pa) in shapes {
        for (gname, g) in geometries {
            for (pname, path, borrow) in paths {
                for (vname, pt, pm, pf, w2, sweep, noerase) in variants {
                    for seed in 1..=8u64 {
                        v.push(Config {
                            name: format!("shape={sname} geometry={gname} path={pname} borrow={borrow:?} variant={vname} seed={seed}"),
                            seed: seed * 1_000_003 + cap as u64, geometry: g, ring: 24, leaf_cap: cap, ret_cap: rcap, tomb_cap: tcap, borrow, path,
                            initial_inodes: initial, create_max: cmax, actions, p_create: pc, p_write: pw, p_append: pa, p_rollback: 25, p_crash: 30,
                            p_lost_root: 10, crash_k_max: 8, p_truncate: pt, p_target_moved: pm, p_target_focus: pf, w2, p_damage: 60, repair_after: 6, sweep, sweep_without_erasing: noerase,
                        });
                    }
                }
            }
        }
    }
    v
}

fn pick(w: &mut World, from: &[u64]) -> u64 { from[w.rng.below(from.len() as u64) as usize] }

pub fn run_history(cfg: Config) -> World {
    let mut w = World::new(cfg.clone());
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
        let focused: Vec<u64> = w.cur.leaves.get(focus_leaf.min(w.cur.leaves.len().saturating_sub(1)))
            .map(|l| l.recs.iter().filter(|r| r.nlink > 0).map(|r| r.ino).collect()).unwrap_or_default();
        if w.rng.chance(cfg.p_rollback) { w.rollback(); continue; }
        let roll = w.rng.below(1000);
        if live.is_empty() || roll < cfg.p_create {
            let n = 1 + w.rng.below(cfg.create_max);
            w.create(n);
        } else if roll < cfg.p_create + cfg.p_write {
            let moved: Vec<u64> = w.recent_moved.iter().copied().filter(|i| live.contains(i)).collect();
            let x = if !moved.is_empty() && w.rng.chance(cfg.p_target_moved) { pick(&mut w, &moved) }
                    else if !focused.is_empty() && w.rng.chance(cfg.p_target_focus) { pick(&mut w, &focused) }
                    else { pick(&mut w, &live) };
            let have = w.cur.data.get(&x).map_or(0, Vec::len) as u64;
            if have > 0 && w.rng.chance(cfg.p_truncate) {
                let u = if w.rng.chance(300) { 0 } else { w.rng.below(have) };
                w.truncate(x, u);
            } else {
                let u = 1 + w.rng.below(8);
                let same = w.rng.chance(300);
                w.overwrite(x, u, same);
            }
        } else if roll < cfg.p_create + cfg.p_write + cfg.p_append {
            let truncated: Vec<u64> = w.recent_truncated.iter().copied().filter(|i| live.contains(i)).collect();
            let x = if !truncated.is_empty() && w.rng.chance(cfg.p_target_moved) { pick(&mut w, &truncated) } else { pick(&mut w, &live) };
            let m = 1 + w.rng.below(4);
            w.append(x, m);
        } else {
            let x = if !focused.is_empty() && w.rng.chance(700) { pick(&mut w, &focused) } else { pick(&mut w, &live) };
            w.unlink(x);
        }
    }
    w
}

fn lines_of(w: &World) -> Vec<String> {
    let mut out = Vec::new();
    for ((reader, tree), c) in &w.stats.cells {
        out.push(format!(
            "R3 name=cell {} reader={reader} tree={tree} scans={} r_peak={} m_peak={} q_peak={} r_scans={} m_scans={} first={}",
            w.cfg.name, c.scans, c.r_peak, c.m_peak, c.q_peak, c.r_scans, c.m_scans, c.first.clone().unwrap_or_else(|| "none".into()).replace(' ', "_")
        ));
    }
    let counts: Vec<String> = w.stats.counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
    out.push(format!("R3 name=counts {} final_txg={} instances={} rows_left={} disk_slots={} {}", w.cfg.name, w.cur.txg, w.max_inst, w.rows.len(), w.disk[0].len(), counts.join(" ")));
    out
}

fn run_jobs(jobs: Vec<Config>) -> Vec<Vec<String>> {
    let threads: usize = std::env::var("R3_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(6).max(1);
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

fn range_arg(args: &[String], len: usize) -> (usize, usize) {
    args.get(2).and_then(|r| r.split_once(':')).map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap().min(len))).unwrap_or((0, len))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str).unwrap_or("") {
        "sweep" => {
            let all = configs();
            let (a, b) = range_arg(&args, all.len());
            let t0 = std::time::Instant::now();
            let out = run_jobs(all[a..b].to_vec());
            let mut emitted = 0;
            for lines in out { for l in lines { println!("{l}"); emitted += 1; } }
            eprintln!("histories={} elapsed_seconds={:.1}", b - a, t0.elapsed().as_secs_f64());
            println!("R3 name=done histories={} emitted={emitted}", b - a);
        }
        "configs" => println!("{}", configs().len()),
        "determinism" => {
            let all = configs();
            let (a, b) = range_arg(&args, all.len());
            let picks: Vec<Config> = all[a..b].to_vec();
            let once = run_jobs(picks.clone());
            let twice = run_jobs(picks.clone());
            let same = once == twice;
            println!("R3 name=determinism range={a}:{b} histories={} two_runs_identical={same}", picks.len());
            if !same { std::process::exit(1); }
        }
        "clone" => clone::run(),
        _ => { eprintln!("用法：sweep <起>:<止> | configs | determinism <起>:<止> | clone"); std::process::exit(2); }
    }
}
