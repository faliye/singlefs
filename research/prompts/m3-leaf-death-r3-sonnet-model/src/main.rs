//! 复跑：`m3-leaf-death-r3-sonnet-model sweep <起>:<止>`（每段 = 一组配置 × 种子）、`configs`、`determinism`、`trace <下标>`。
//! 线程数读 R3_THREADS（默认 6），只影响挂钟、不影响输出（每段历史各自一个种子，输出按段的序号排）。
mod model;
mod clone;
use model::*;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct Shape {
    name: &'static str, cap: usize, ret_cap: usize, tomb_cap: usize, initial: u64, create_max: u64, actions: u32,
    p_create: u64, p_write: u64, p_append: u64, data: DataMode, order: DelOrder, seeds: u64, faults: bool,
}

const SHAPES: [Shape; 4] = [
    // 第二轮攻方腿的两种形状（用户动作全部放开，70% 的删落在每 150 步换一片的「盯着的」叶上）
    Shape { name: "cap233", cap: 233, ret_cap: 959, tomb_cap: 815, initial: 700, create_max: 20, actions: 1500, p_create: 60, p_write: 80, p_append: 40, data: DataMode::AllInodes, order: DelOrder::Focus, seeds: 6, faults: true },
    Shape { name: "cap6", cap: 6, ret_cap: 3, tomb_cap: 3, initial: 40, create_max: 4, actions: 600, p_create: 150, p_write: 150, p_append: 100, data: DataMode::AllInodes, order: DelOrder::Focus, seeds: 20, faults: true },
    // 第一轮 E177 / 第二轮辩方那种前提：只有 1 号有数据、从小到大删
    Shape { name: "ord233", cap: 233, ret_cap: 959, tomb_cap: 815, initial: 1400, create_max: 20, actions: 2000, p_create: 20, p_write: 110, p_append: 0, data: DataMode::OnlyInodeOne, order: DelOrder::Ascending, seeds: 6, faults: true },
    // 多片叶上随机删（合并、借、删空都多）
    Shape { name: "rnd233", cap: 233, ret_cap: 959, tomb_cap: 815, initial: 2400, create_max: 20, actions: 2400, p_create: 20, p_write: 80, p_append: 40, data: DataMode::AllInodes, order: DelOrder::Random, seeds: 4, faults: true },
];

const CANDS: [Cand; 7] = [Cand::Today, Cand::RetA, Cand::RetB, Cand::Persist, Cand::PersistX, Cand::Tsz, Cand::Fsz];
const BORROWS: [(&str, BorrowArm); 3] = [("no_borrow", BorrowArm::NoBorrow), ("n1new", BorrowArm::N1New), ("n2", BorrowArm::N2)];
const GEOMETRIES: [(&str, Geometry); 3] = [("wide", Geometry::Wide), ("ring_lowest", Geometry::RingLowest), ("ring_random", Geometry::RingRandom)];

fn configs() -> Vec<Config> {
    let mut v = Vec::new();
    for shape in SHAPES {
        for (gname, g) in GEOMETRIES {
            for cand in CANDS {
                for (bname, b) in BORROWS {
                    for seed in 1..=shape.seeds {
                        v.push(Config {
                            name: format!("shape={} geometry={gname} cand={} borrow={bname} seed={seed}", shape.name, cand.name()),
                            seed: seed * 1_000_003 + shape.cap as u64 + shape.initial,
                            geometry: g, ring: 24, leaf_cap: shape.cap, ret_cap: shape.ret_cap, tomb_cap: shape.tomb_cap, cand, borrow: b,
                            data_mode: shape.data, del_order: shape.order, initial_inodes: shape.initial, create_max: shape.create_max, actions: shape.actions,
                            p_create: shape.p_create, p_write: shape.p_write, p_append: shape.p_append, p_rollback: if shape.faults { 25 } else { 0 }, p_crash: if shape.faults { 30 } else { 0 }, p_lost_root: if shape.faults { 10 } else { 0 }, crash_k_max: 8,
                            ablate: seed <= 2 && !matches!(cand, Cand::Today), sweep: 0,
                        });
                    }
                }
            }
        }
    }
    // 前提族：同一个形状（叶容量 233，起始 1400 个空 inode，2000 个动作）上一次只换一个前提——谁有数据、怎么挑删、有没有故障——
    // 只配 ring_lowest 几何、四个候选（Today、RetA、Persist、Tsz）、两种借法（不借、N1New）。给「前两轮的数能不能并排」那一格用。
    for (dname, data) in [("data_all", DataMode::AllInodes), ("data_one", DataMode::OnlyInodeOne)] {
        for (oname, order) in [("del_focus", DelOrder::Focus), ("del_random", DelOrder::Random), ("del_ascending", DelOrder::Ascending)] {
            for (fname, faults) in [("faults_on", true), ("faults_off", false)] {
                for cand in [Cand::Today, Cand::RetA, Cand::Persist, Cand::Tsz] {
                    for (bname, b) in [BORROWS[0], BORROWS[1]] {
                        for seed in 1..=6u64 {
                            v.push(Config {
                                name: format!("shape=prem_{dname}_{oname}_{fname} geometry=ring_lowest cand={} borrow={bname} seed={seed}", cand.name()),
                                seed: seed * 1_000_003 + 233 + 1400,
                                geometry: Geometry::RingLowest, ring: 24, leaf_cap: 233, ret_cap: 959, tomb_cap: 815, cand, borrow: b,
                                data_mode: data, del_order: order, initial_inodes: 1400, create_max: 20, actions: 2000,
                                p_create: 20, p_write: if matches!(data, DataMode::OnlyInodeOne) { 110 } else { 80 }, p_append: if matches!(data, DataMode::OnlyInodeOne) { 0 } else { 40 },
                                p_rollback: if faults { 25 } else { 0 }, p_crash: if faults { 30 } else { 0 }, p_lost_root: if faults { 10 } else { 0 }, crash_k_max: 8,
                                ablate: false, sweep: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    // 清扫族：cap6 形状，Today / RetA / Persist / Tsz 四个候选，清扫（抹掉未发布的单元再删行）与漏抹证据单元两种。
    for (sname, sweep) in [("sweep_perfect", 1u8), ("sweep_misses_evidence", 2)] {
        for (gname, g) in [GEOMETRIES[0], GEOMETRIES[1]] {
            for cand in [Cand::Today, Cand::RetA, Cand::Persist, Cand::Tsz] {
                for seed in 1..=12u64 {
                    let shape = SHAPES[1];
                    v.push(Config {
                        name: format!("shape={sname}_cap6 geometry={gname} cand={} borrow=n1new seed={seed}", cand.name()),
                        seed: seed * 1_000_003 + shape.cap as u64 + shape.initial,
                        geometry: g, ring: 24, leaf_cap: shape.cap, ret_cap: shape.ret_cap, tomb_cap: shape.tomb_cap, cand, borrow: BorrowArm::N1New,
                        data_mode: shape.data, del_order: shape.order, initial_inodes: shape.initial, create_max: shape.create_max, actions: shape.actions,
                        p_create: shape.p_create, p_write: shape.p_write, p_append: shape.p_append, p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8,
                        ablate: false, sweep,
                    });
                }
            }
        }
    }
    // N1 旧形（第二轮攻方腿的 N1：借时把右片分隔 key 抬到借后最小 key，之后不再跟着变）：为了和它的 8.9% 对数，只配 Today、RetA，cap233 与 cap6 两个形状。
    for shape in [SHAPES[0], SHAPES[1]] {
        for (gname, g) in GEOMETRIES {
            for cand in [Cand::Today, Cand::RetA] {
                for seed in 1..=shape.seeds {
                    v.push(Config {
                        name: format!("shape={} geometry={gname} cand={} borrow=n1raise seed={seed}", shape.name, cand.name()),
                        seed: seed * 1_000_003 + shape.cap as u64 + shape.initial,
                        geometry: g, ring: 24, leaf_cap: shape.cap, ret_cap: shape.ret_cap, tomb_cap: shape.tomb_cap, cand, borrow: BorrowArm::N1Raise,
                        data_mode: shape.data, del_order: shape.order, initial_inodes: shape.initial, create_max: shape.create_max, actions: shape.actions,
                        p_create: shape.p_create, p_write: shape.p_write, p_append: shape.p_append, p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8,
                        ablate: false, sweep: 0,
                    });
                }
            }
        }
    }
    // 历史长度族：E177 式前提（只有 1 号有数据、起始 1400 个）下一次只换一个前提——动作数 100 / 300 / 1000，两种删序，故障开；
    // 给「每段历史多长」这个前提量开（第一轮 E177 每段平均 392 次发布，本腿主表每段约 1 万次）。只配 ring_lowest、Today / RetA / Persist、N1New。
    for (oname, order) in [("del_ascending", DelOrder::Ascending), ("del_random", DelOrder::Random)] {
        for actions in [100u32, 300, 1000] {
            for cand in [Cand::Today, Cand::RetA, Cand::Persist] {
                for seed in 1..=6u64 {
                    v.push(Config {
                        name: format!("shape=len_{oname}_{actions} geometry=ring_lowest cand={} borrow=n1new seed={seed}", cand.name()),
                        seed: seed * 1_000_003 + 233 + 1400,
                        geometry: Geometry::RingLowest, ring: 24, leaf_cap: 233, ret_cap: 959, tomb_cap: 815, cand, borrow: BorrowArm::N1New,
                        data_mode: DataMode::OnlyInodeOne, del_order: order, initial_inodes: 1400, create_max: 20, actions,
                        p_create: 20, p_write: 110, p_append: 0, p_rollback: 25, p_crash: 30, p_lost_root: 10, crash_k_max: 8,
                        ablate: false, sweep: 0,
                    });
                }
            }
        }
    }
    v
}

pub fn run_history(cfg: Config) -> World { run_history_with(cfg, false) }
fn run_history_with(cfg: Config, traced: bool) -> World {
    let mut w = World::new(cfg.clone());
    w.record_history = traced;
    // 新池新建文件：1 号带一个数据单元；再一次建 initial 个空 inode。
    w.inject = false;
    w.begin_action();
    w.create(1);
    w.begin_action();
    w.overwrite(1, 1, false);
    w.begin_action();
    w.create(cfg.initial_inodes);
    w.inject = true;
    let mut focus_start = 0usize;
    for step in 0..cfg.actions {
        w.begin_action();
        let live: Vec<u64> = {
            let mut v: Vec<u64> = w.cur.leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect();
            v.sort_unstable();
            v
        };
        if step % 150 == 0 { focus_start = w.arng.below(live.len().max(1) as u64) as usize; }
        if w.arng.chance(cfg.p_rollback) { w.rollback(); continue; }
        let roll = w.arng.below(1000);
        let data_target = |w: &mut World| -> Option<u64> {
            match cfg.data_mode {
                DataMode::OnlyInodeOne => live.contains(&1).then_some(1),
                DataMode::AllInodes => if live.is_empty() { None } else { Some(live[w.arng.below(live.len() as u64) as usize]) },
            }
        };
        if live.is_empty() || roll < cfg.p_create {
            let n = 1 + w.arng.below(cfg.create_max);
            w.create(n);
        } else if roll < cfg.p_create + cfg.p_write {
            let u = 1 + w.arng.below(8);
            let same = w.arng.chance(300);
            if let Some(x) = data_target(&mut w) { w.overwrite(x, u, same); }
        } else if roll < cfg.p_create + cfg.p_write + cfg.p_append {
            let m = 1 + w.arng.below(4);
            if let Some(x) = data_target(&mut w) { w.append(x, m); }
        } else {
            let pick: Option<u64> = match cfg.del_order {
                DelOrder::Random => Some(live[w.arng.below(live.len() as u64) as usize]),
                DelOrder::Ascending => live.iter().copied().find(|&x| x >= 2),
                DelOrder::Descending => live.last().copied().filter(|&x| x >= 2),
                DelOrder::Focus => {
                    let a = focus_start.min(live.len() - 1);
                    let window = &live[a..(a + cfg.leaf_cap).min(live.len())];
                    if w.arng.chance(700) { Some(window[w.arng.below(window.len() as u64) as usize]) } else { Some(live[w.arng.below(live.len() as u64) as usize]) }
                }
            };
            if let Some(x) = pick { w.unlink(x); }
        }
    }
    w
}

fn lines_of(w: &World) -> Vec<String> {
    let mut out = Vec::new();
    for ((reader, tree), c) in &w.stats.cells {
        out.push(format!(
            "R3 name=cell {} reader={reader} tree={tree} scans={} r_peak={} m_peak={} r_scans={} m_scans={} m_scans_in_window={} m_scans_outside_window={} first={}",
            w.cfg.name, c.scans, c.r_peak, c.m_peak, c.r_scans, c.m_scans, c.m_scans_in_window, c.m_scans_outside_window,
            c.first.clone().unwrap_or_else(|| "none".into()).replace(' ', "_")
        ));
    }
    let counts: Vec<String> = w.stats.counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
    out.push(format!("R3 name=counts {} final_txg={} instances={} disk_slots={} logical_hash={:016x} {}", w.cfg.name, w.cur.txg, w.max_inst, w.disk.len(), w.logical_trace, counts.join(" ")));
    out
}

fn run_jobs(jobs: Vec<Config>) -> Vec<Vec<String>> {
    let threads: usize = std::env::var("R3_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(6).max(1);
    let n = jobs.len();
    let jobs = Arc::new(Mutex::new(jobs.into_iter().enumerate().rev().collect::<Vec<_>>()));
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
            println!("R3 name=done histories={} emitted={emitted}", b - a);
        }
        "clone" => clone::run(),
        "configs" => println!("{}", configs().len()),
        "names" => { for (i, c) in configs().iter().enumerate() { println!("{i} {}", c.name); } }
        "trace" => {
            let i: usize = args.get(2).and_then(|x| x.parse().ok()).expect("trace <下标>");
            let cfg = configs()[i].clone();
            println!("R3 name=trace {}", cfg.name);
            let w = run_history_with(cfg, true);
            for l in &w.history { println!("{l}"); }
        }
        "determinism" => {
            let all = configs();
            let picks: Vec<Config> = all.iter().enumerate().filter(|(i, c)| c.leaf_cap == 6 && i % 5 == 0 || i % 97 == 0).map(|(_, c)| c.clone()).collect();
            let once = run_jobs(picks.clone());
            let twice = run_jobs(picks.clone());
            let same = once == twice;
            println!("R3 name=determinism histories={} two_runs_identical={same}", picks.len());
            if !same { std::process::exit(1); }
        }
        _ => { eprintln!("用法：sweep <起>:<止> | configs | names | determinism | trace <下标>"); std::process::exit(2); }
    }
}
