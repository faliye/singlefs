// 正推腿（第 10z 批第三块代码轮）自造的物理卡模型：在 sonnet_probe_common 之上加「设备丢了、重起、重起被拒」三步。
// 只用 GpuMemoryLedger 的公开函数造真账。量：账给的量 与 此刻真能再借的（保证值 = 空闲 − 活着借据没建的 − 驱动还要长的）。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, GpuMemoryBooking, GpuMemoryLedger,
    OccupiedByThisProcess,
};
use std::sync::{Arc, Mutex};

const D: u64 = 2560;
const EMPTY: u64 = 32_110;

#[derive(Clone, Default, Debug)]
struct Phys {
    started: bool,
    xd: u64,
    outsider: u64,
    live_a: Vec<(usize, u64)>,
    left: Vec<u64>,
    // 丢了的设备留在卡上、驱动还没收回的
    dead: Vec<u64>,
    lost_unnoticed: bool,
}
impl Phys {
    fn x(&self) -> u64 {
        (if self.started { self.xd } else { 0 })
            + self.live_a.iter().map(|(_, a)| a).sum::<u64>()
            + self.left.iter().sum::<u64>()
            + self.dead.iter().sum::<u64>()
    }
    fn reading(&self) -> CardMemoryReading {
        CardMemoryReading {
            free_mebibytes: EMPTY - self.outsider - self.x(),
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: if self.x() > 0 {
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: self.x() }
            } else {
                OccupiedByThisProcess::NotListed
            },
        }
    }
}

struct Rig {
    phys: Arc<Mutex<Phys>>,
    ledger: GpuMemoryLedger,
}
fn rig() -> Rig {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let shared = Arc::clone(&phys);
    let ledger = GpuMemoryLedger::new(
        1,
        Box::new(move || Ok(vec![shared.lock().unwrap().reading()])),
        Box::new(|_p, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, D)),
        D,
    );
    Rig { phys, ledger }
}

fn truth(phys: &Phys, booked: &[(usize, u64)]) -> i64 {
    let f = i64::try_from(EMPTY - phys.outsider - phys.x()).unwrap();
    let unbuilt: i64 = booked
        .iter()
        .map(|(i, b)| {
            let a = phys.live_a.iter().find(|(j, _)| j == i).map_or(0, |(_, a)| *a);
            i64::try_from(*b).unwrap() - i64::try_from(a).unwrap()
        })
        .sum();
    let driver_to_come = i64::try_from(D - if phys.started { phys.xd } else { 0 }).unwrap();
    f - unbuilt - driver_to_come
}
fn room(rig: &Rig) -> i64 {
    i64::try_from(rig.ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0]).unwrap()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Default, Debug)]
struct Stats {
    samples: u64,
    over_noticed: u64,       // 账已经知道设备丢了（或没丢）时的多给
    over_noticed_max: i64,
    over_unnoticed: u64,     // 设备丢了、账还没被问过（第一次 prepare 之前）时的多给
    over_unnoticed_max: i64,
    under: u64,
    refused_restarts: u64,
    ok_restarts: u64,
    restart_ok_but_unfit: u64,
    restart_ok_unfit_without_a_lease: u64, // 重起放行，而新设备起来之后真不够（物理上放不下新设备的驱动）
    shortest_trace: Option<Vec<String>>,
    unfit_trace: Option<Vec<String>>,
}

fn sample(stats: &mut Stats, given: i64, t: i64, unnoticed: bool, trace: &[String], what: &str) {
    stats.samples += 1;
    let t = t.max(0);
    if given > t {
        let over = given - t;
        if unnoticed {
            stats.over_unnoticed += 1;
            stats.over_unnoticed_max = stats.over_unnoticed_max.max(over);
        } else {
            stats.over_noticed += 1;
            stats.over_noticed_max = stats.over_noticed_max.max(over);
            if stats.shortest_trace.as_ref().map_or(true, |s| s.len() > trace.len() + 1) {
                let mut tr = trace.to_vec();
                tr.push(format!("{what}: given {given} truth {t}"));
                stats.shortest_trace = Some(tr);
            }
        }
    } else if given < t {
        stats.under += 1;
    }
}

fn fuzz(seeds: u64, steps: u64, xd_actual: u64, collect_percent: u64, lose_percent: u64) -> Stats {
    let mut stats = Stats::default();
    for seed in 1..=seeds {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let rig = rig();
        rig.phys.lock().unwrap().outsider = rng.below(8) * 1000;
        let mut bookings: Vec<(usize, u64, Option<GpuMemoryBooking<'_>>)> = Vec::new();
        let mut next_id = 0usize;
        let mut trace: Vec<String> = Vec::new();
        let mut lost_not_restarted = false; // 物理上丢了，新设备还没起
        for _ in 0..steps {
            let op = rng.below(14);
            let booked_now: Vec<(usize, u64)> =
                bookings.iter().filter(|b| b.2.is_some()).map(|b| (b.0, b.1)).collect();
            match op {
                0..=2 => {
                    let need = 500 + rng.below(16) * 500;
                    let (t, un) = {
                        let p = rig.phys.lock().unwrap();
                        (truth(&p, &booked_now), p.lost_unnoticed)
                    };
                    if let Ok(b) = rig.ledger.book_on_the_card(0, need) {
                        let given = i64::try_from(b.card_room_mebibytes_when_booked()).unwrap();
                        sample(&mut stats, given, t, un, &trace, &format!("book{need}"));
                        bookings.push((next_id, need, Some(b)));
                        trace.push(format!("book {need} -> id{next_id}"));
                        next_id += 1;
                    }
                }
                3 => {
                    // 头一回起设备（没丢过）
                    let any = !bookings.iter().all(|b| b.2.is_none());
                    let mut p = rig.phys.lock().unwrap();
                    if any && !p.started && !lost_not_restarted && p.dead.is_empty() && p.xd == 0 {
                        p.started = true;
                        p.xd = xd_actual;
                        drop(p);
                        rig.ledger.record_the_device_started_on_the_card(0);
                        trace.push("start".into());
                    }
                }
                4 | 5 => {
                    let live: Vec<usize> =
                        bookings.iter().filter(|b| b.2.is_some()).map(|b| b.0).collect();
                    let mut p = rig.phys.lock().unwrap();
                    if (p.started || lost_not_restarted) && !live.is_empty() {
                        let i = live[usize::try_from(rng.below(live.len() as u64)).unwrap()];
                        let b = bookings.iter().find(|b| b.0 == i).unwrap().1;
                        let cur = p.live_a.iter().find(|(j, _)| *j == i).map_or(0, |(_, a)| *a);
                        let a = rng.below(b + 1);
                        let a = a.max(cur);
                        let others: u64 = p.x() - cur;
                        if others + a + p.outsider <= EMPTY {
                            p.live_a.retain(|(j, _)| *j != i);
                            p.live_a.push((i, a));
                            trace.push(format!("build id{i} a={a}"));
                        }
                    }
                }
                6 | 7 => {
                    let live: Vec<usize> =
                        bookings.iter().filter(|b| b.2.is_some()).map(|b| b.0).collect();
                    if !live.is_empty() {
                        let i = live[usize::try_from(rng.below(live.len() as u64)).unwrap()];
                        let slot = bookings.iter_mut().find(|b| b.0 == i).unwrap();
                        let a = {
                            let mut p = rig.phys.lock().unwrap();
                            let a = p.live_a.iter().find(|(j, _)| *j == i).map_or(0, |(_, a)| *a);
                            p.live_a.retain(|(j, _)| *j != i);
                            if a > 0 {
                                p.left.push(a);
                            }
                            a
                        };
                        drop(slot.2.take());
                        trace.push(format!("return id{i} (leftover {a})"));
                    }
                }
                8 => {
                    let mut p = rig.phys.lock().unwrap();
                    if rng.below(100) < collect_percent {
                        if !p.left.is_empty() {
                            let k = usize::try_from(rng.below(p.left.len() as u64)).unwrap();
                            let v = p.left.remove(k);
                            trace.push(format!("driver frees leftover {v}"));
                        } else if !p.dead.is_empty() {
                            let k = usize::try_from(rng.below(p.dead.len() as u64)).unwrap();
                            let v = p.dead.remove(k);
                            trace.push(format!("driver frees dead device {v}"));
                        }
                    }
                }
                9 => {
                    // 设备丢了：旧设备的驱动那一份留在卡上等收回
                    let mut p = rig.phys.lock().unwrap();
                    if p.started && rng.below(100) < lose_percent {
                        let xd = p.xd;
                        p.started = false;
                        p.xd = 0;
                        if xd > 0 {
                            p.dead.push(xd);
                        }
                        p.lost_unnoticed = true;
                        lost_not_restarted = true;
                        trace.push(format!("device lost (dead {xd} stays on card)"));
                    }
                }
                10 | 11 => {
                    // 重起（卡池照 is_lost 重起，先 prepare）
                    if lost_not_restarted {
                        let r = rig.ledger.prepare_the_driver_share_before_starting_the_device(0);
                        {
                            rig.phys.lock().unwrap().lost_unnoticed = false;
                        }
                        match r {
                            Ok(()) => {
                                stats.ok_restarts += 1;
                                let mut p = rig.phys.lock().unwrap();
                                // 物理上新设备起来要 xd_actual；放不下就是 OOM
                                let has_a_lease = !booked_now.is_empty();
                                if p.x() + p.outsider + xd_actual <= EMPTY {
                                    p.started = true;
                                    p.xd = xd_actual;
                                    drop(p);
                                    rig.ledger.record_the_device_started_on_the_card(0);
                                    lost_not_restarted = false;
                                    trace.push("restart ok".into());
                                } else {
                                    if has_a_lease { stats.restart_ok_but_unfit += 1; } else { stats.restart_ok_unfit_without_a_lease += 1; }
                                    let snap = format!("phys x={} outsider={} dead={:?} left={:?} live_a={:?} leases={:?}", p.x(), p.outsider, p.dead, p.left, p.live_a, booked_now);
                                    trace.push(format!("restart allowed but physically unfit [{snap}]"));
                                    if stats.unfit_trace.as_ref().map_or(true, |s| s.len() > trace.len()) {
                                        stats.unfit_trace = Some(trace.clone());
                                    }
                                }
                            }
                            Err(_) => {
                                stats.refused_restarts += 1;
                                trace.push("restart refused".into());
                            }
                        }
                    }
                }
                _ => {
                    let (t, un) = {
                        let p = rig.phys.lock().unwrap();
                        (truth(&p, &booked_now), p.lost_unnoticed)
                    };
                    let r = room(&rig);
                    sample(&mut stats, r, t, un, &trace, "query");
                }
            }
        }
    }
    stats
}

#[test]
fn fuzz_with_device_loss_and_restart() {
    for (xd, collect, lose) in [(2560u64, 50u64, 20u64), (600, 50, 20), (2560, 90, 40), (1500, 10, 40), (2560, 0, 30)] {
        let s = fuzz(4000, 120, xd, collect, lose);
        println!(
            "LOSTFUZZ xd={xd} collect%={collect} lose%={lose}: samples={} over_noticed={} (max {}) over_before_first_prepare_after_loss={} (max {}) under={} refused_restarts={} ok_restarts={} restart_ok_but_unfit(with a live lease)={} restart_ok_unfit_without_a_lease={}",
            s.samples, s.over_noticed, s.over_noticed_max, s.over_unnoticed, s.over_unnoticed_max, s.under, s.refused_restarts, s.ok_restarts, s.restart_ok_but_unfit, s.restart_ok_unfit_without_a_lease
        );
        if let Some(t) = &s.unfit_trace {
            println!("LOSTFUZZ shortest unfit trace: {t:?}");
        }
        if let Some(t) = &s.shortest_trace {
            println!("LOSTFUZZ shortest over_noticed trace: {t:?}");
        }
    }
}

// ---- 定向：丢了之后账还没被问过的那一段；重起 k 次之后账是否回到正常 ----
fn show(rig: &Rig, booked: &[(usize, u64)], what: &str) {
    let t = truth(&rig.phys.lock().unwrap(), booked);
    println!("DIRECTED {what}: ledger_room={} truth={}", room(rig), t);
}

#[test]
fn directed_loss_window_and_k_restarts() {
    // 一块空卡：借 10000、起设备（驱动实占 D）、建满 10000
    let rig = rig();
    let b = rig.ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    rig.phys.lock().unwrap().live_a.push((0, 10_000));
    let booked = [(0usize, 10_000u64)];
    show(&rig, &booked, "healthy: lease 10000 built, device started");
    // 设备丢了，账还没被问过
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = false;
        p.xd = 0;
        p.dead.push(D);
    }
    show(&rig, &booked, "device lost, ledger not asked yet (window)");
    // 第一次重起判（账被问到了）
    let r = rig.ledger.prepare_the_driver_share_before_starting_the_device(0);
    println!("DIRECTED prepare after loss -> {r:?}");
    show(&rig, &booked, "after the first prepare (new device not started)");
    // 旧设备那一份被驱动收回；新设备起好
    {
        let mut p = rig.phys.lock().unwrap();
        p.dead.clear();
        p.started = true;
        p.xd = D;
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    show(&rig, &booked, "restarted, dead share freed, new device started (k=1)");
    for k in 2..=5 {
        {
            let mut p = rig.phys.lock().unwrap();
            p.started = false;
            p.xd = 0;
            p.dead.push(D);
        }
        let r = rig.ledger.prepare_the_driver_share_before_starting_the_device(0);
        {
            let mut p = rig.phys.lock().unwrap();
            p.dead.clear();
            p.started = true;
            p.xd = D;
        }
        rig.ledger.record_the_device_started_on_the_card(0);
        println!("DIRECTED k={k} prepare -> {r:?}");
        show(&rig, &booked, &format!("restarted again (k={k})"));
    }
    drop(b);
}
