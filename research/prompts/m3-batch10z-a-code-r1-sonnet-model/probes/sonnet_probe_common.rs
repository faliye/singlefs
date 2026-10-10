// 正推腿的探针：只用公开函数造真账，对一个「物理卡」模型量账给的量与此刻真能用的。两份树（改后、改前）各拷一份跑。
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
}
impl Phys {
    fn x(&self) -> u64 {
        if !self.started {
            return 0;
        }
        self.xd + self.live_a.iter().map(|(_, a)| a).sum::<u64>() + self.left.iter().sum::<u64>()
    }
    fn reading(&self) -> CardMemoryReading {
        CardMemoryReading {
            free_mebibytes: EMPTY - self.outsider - self.x(),
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: if self.started {
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

/// 此刻真能再借的（保证值）：空闲减每笔活着的借据还没建的、减驱动还要长的；没登记时驱动那一份也要放得下。
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
    over_count: u64,
    over_max: i64,
    under_count: u64,
    under_max: i64,
    quiet_samples: u64,
    quiet_under_count: u64,
    quiet_under_max: i64,
    shortest_over_trace: Option<Vec<String>>,
}

fn fuzz(seeds: u64, steps: u64, fluct: bool, xd_actual: u64, collect_percent: u64) -> Stats {
    fuzz2(seeds, steps, fluct, xd_actual, collect_percent, false, 0)
}

fn fuzz2(seeds: u64, steps: u64, fluct: bool, xd_actual: u64, collect_percent: u64, full: bool, free_on_return: u64) -> Stats {
    fuzz3(seeds, steps, fluct, xd_actual, collect_percent, full, free_on_return, false)
}

#[allow(clippy::too_many_arguments)]
fn fuzz3(seeds: u64, steps: u64, fluct: bool, xd_actual: u64, collect_percent: u64, full: bool, free_on_return: u64, only_built_returns: bool) -> Stats {
    let mut stats = Stats::default();
    for seed in 1..=seeds {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let rig = rig();
        rig.phys.lock().unwrap().outsider = rng.below(8) * 1000;
        let mut bookings: Vec<(usize, u64, Option<GpuMemoryBooking<'_>>)> = Vec::new();
        let mut next_id = 0usize;
        let mut trace: Vec<String> = Vec::new();
        for _ in 0..steps {
            let op = rng.below(10);
            let booked_now: Vec<(usize, u64)> = bookings.iter().filter(|b| b.2.is_some()).map(|b| (b.0, b.1)).collect();
            match op {
                0..=2 => {
                    let need = 500 + rng.below(16) * 500;
                    let t = truth(&rig.phys.lock().unwrap(), &booked_now);
                    if let Ok(b) = rig.ledger.book_on_the_card(0, need) {
                        let given = i64::try_from(b.card_room_mebibytes_when_booked()).unwrap();
                        sample(&mut stats, given, t, &rig, &mut trace, &format!("book{need}"), false);
                        bookings.push((next_id, need, Some(b)));
                        trace.push(format!("book {need} -> id{next_id}"));
                        next_id += 1;
                    }
                }
                3 => {
                    let any = !bookings.iter().all(|b| b.2.is_none());
                    let mut p = rig.phys.lock().unwrap();
                    if any && !p.started {
                        p.started = true;
                        p.xd = xd_actual;
                        drop(p);
                        rig.ledger.record_the_device_started_on_the_card(0);
                        trace.push("start".into());
                    }
                }
                4 | 5 => {
                    let live: Vec<usize> = bookings.iter().filter(|b| b.2.is_some()).map(|b| b.0).collect();
                    let mut p = rig.phys.lock().unwrap();
                    if p.started && !live.is_empty() {
                        let i = live[usize::try_from(rng.below(live.len() as u64)).unwrap()];
                        let b = bookings.iter().find(|b| b.0 == i).unwrap().1;
                        let cur = p.live_a.iter().find(|(j, _)| *j == i).map_or(0, |(_, a)| *a);
                        let lo = if fluct { 0 } else { cur };
                        let a = if full { b } else { lo + rng.below(b - lo + 1) };
                        p.xd = D.min(p.xd.max(xd_actual));
                        let others: u64 = p.x() - cur;
                        if others + a + p.outsider <= EMPTY {
                            p.live_a.retain(|(j, _)| *j != i);
                            p.live_a.push((i, a));
                            trace.push(format!("build id{i} a={a}"));
                        }
                    }
                }
                6 | 7 => {
                    let live: Vec<usize> = {
                        let p = rig.phys.lock().unwrap();
                        bookings.iter().filter(|b| b.2.is_some()).filter(|b| !only_built_returns || p.live_a.iter().any(|(j, a)| *j == b.0 && *a > 0)).map(|b| b.0).collect()
                    };
                    if !live.is_empty() {
                        let i = live[usize::try_from(rng.below(live.len() as u64)).unwrap()];
                        let slot = bookings.iter_mut().find(|b| b.0 == i).unwrap();
                        let a = {
                            let mut p = rig.phys.lock().unwrap();
                            let a = p.live_a.iter().find(|(j, _)| *j == i).map_or(0, |(_, a)| *a);
                            p.live_a.retain(|(j, _)| *j != i);
                            if a > 0 && rng.below(100) >= free_on_return {
                                p.left.push(a);
                            }
                            a
                        };
                        drop(slot.2.take());
                        trace.push(format!("return id{i} (leftover {a} still on card)"));
                    }
                }
                8 => {
                    let mut p = rig.phys.lock().unwrap();
                    if !p.left.is_empty() && rng.below(100) < collect_percent {
                        let k = usize::try_from(rng.below(p.left.len() as u64)).unwrap();
                        let v = p.left.remove(k);
                        trace.push(format!("driver frees leftover {v}"));
                    }
                }
                _ => {
                    let t = truth(&rig.phys.lock().unwrap(), &booked_now);
                    let r = room(&rig);
                    let quiet = rig.phys.lock().unwrap().left.is_empty();
                    sample(&mut stats, r, t, &rig, &mut trace, "query", quiet);
                }
            }
        }
    }
    stats
}

fn sample(stats: &mut Stats, given: i64, truth: i64, _rig: &Rig, trace: &mut Vec<String>, label: &str, quiet: bool) {
    stats.samples += 1;
    let diff = given - truth;
    if diff > 0 {
        stats.over_count += 1;
        stats.over_max = stats.over_max.max(diff);
        let mut t = trace.clone();
        t.push(format!("{label}: ledger {given} truth {truth} over {diff}"));
        if stats.shortest_over_trace.as_ref().is_none_or(|s| t.len() < s.len()) {
            stats.shortest_over_trace = Some(t);
        }
    }
    if diff < 0 {
        stats.under_count += 1;
        stats.under_max = stats.under_max.max(-diff);
    }
    if quiet {
        stats.quiet_samples += 1;
        if diff < 0 {
            stats.quiet_under_count += 1;
            stats.quiet_under_max = stats.quiet_under_max.max(-diff);
        }
    }
}

#[test]
fn fuzz_monotone_live_buffers() {
    for (xd, collect) in [(D, 50), (D, 90), (600, 50)] {
        let s = fuzz(400, 120, false, xd, collect);
        println!("FUZZ monotone xd={xd} collect%={collect}: {s:?}");
    }
}

#[test]
fn fuzz_live_buffers_may_shrink() {
    for (xd, collect) in [(D, 50), (D, 90)] {
        let s = fuzz(400, 120, true, xd, collect);
        println!("FUZZ shrink xd={xd} collect%={collect}: {s:?}");
    }
}


/// S1：头一笔借据建了、丢了，卡上在第一次读数之前就还给驱动；之后再借再建。
#[test]
fn s1_first_returned_lease_never_seen_on_the_card() {
    for b1 in [1_000u64, 2_560, 10_000] {
        let rig = rig();
        let l1 = rig.ledger.book_on_the_card(0, b1).unwrap();
        {
            let mut p = rig.phys.lock().unwrap();
            p.started = true;
            p.xd = D;
            p.live_a.push((1, b1));
        }
        rig.ledger.record_the_device_started_on_the_card(0);
        drop(l1);
        rig.phys.lock().unwrap().live_a.clear(); // 建的缓冲在第一次读数之前就还给驱动了
        let p = rig.phys.lock().unwrap().clone();
        println!("S1 b1={b1}: after drop, no leftover: ledger {} truth {}", room(&rig), truth(&p, &[]));
        let l2 = rig.ledger.book_on_the_card(0, 10_000).unwrap();
        rig.phys.lock().unwrap().live_a.push((2, 10_000));
        let p = rig.phys.lock().unwrap().clone();
        println!("S1 b1={b1}: L2 built 10000: ledger {} truth {}", room(&rig), truth(&p, &[(2, 10_000)]));
        drop(l2);
    }
}

/// S1b：对照：同一段，但第一次读数时那一笔还在卡上（过一会才还）。
#[test]
fn s1b_returned_lease_seen_on_the_card_then_freed() {
    let b1 = 10_000;
    let rig = rig();
    let l1 = rig.ledger.book_on_the_card(0, b1).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.live_a.push((1, b1));
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    drop(l1);
    {
        let mut p = rig.phys.lock().unwrap();
        p.live_a.clear();
        p.left.push(b1);
    }
    let p = rig.phys.lock().unwrap().clone();
    println!("S1b: leftover still on card: ledger {} truth {}", room(&rig), truth(&p, &[]));
    rig.phys.lock().unwrap().left.clear();
    let p = rig.phys.lock().unwrap().clone();
    println!("S1b: leftover freed: ledger {} truth {}", room(&rig), truth(&p, &[]));
}

/// S3：长借据建满、读过一次，之后它的缓冲还给驱动（借据还活着），同时一笔短借据销了账、卡上还没还。
#[test]
fn s3_live_lease_shrinks_while_a_returned_short_lease_is_still_on_the_card() {
    let rig = rig();
    let l1 = rig.ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    rig.phys.lock().unwrap().live_a.push((0, 10_000));
    let p = rig.phys.lock().unwrap().clone();
    println!("S3: L1 built: ledger {} truth {}", room(&rig), truth(&p, &[(0, 10_000)]));
    let s = rig.ledger.book_on_the_card(0, 1_000).unwrap();
    rig.phys.lock().unwrap().live_a.push((1, 1_000));
    let p = rig.phys.lock().unwrap().clone();
    println!("S3: S built: ledger {} truth {}", room(&rig), truth(&p, &[(0, 10_000), (1, 1_000)]));
    drop(s);
    {
        let mut p = rig.phys.lock().unwrap();
        p.live_a.retain(|(i, _)| *i != 1);
        p.left.push(1_000); // 短借据还在卡上
        p.live_a.retain(|(i, _)| *i != 0); // 长借据活着，但这一刻它的每趟缓冲已还给驱动
    }
    let p = rig.phys.lock().unwrap().clone();
    println!("S3: L1 idle (live, a=0), S leftover 1000 on card: ledger {} truth {} (> truth is an overlend)", room(&rig), truth(&p, &[(0, 10_000)]));
    drop(l1);
}

/// S5：一笔借据在「见到的那一份」更新时已借出、没建，后来没建就丢了。
#[test]
fn s5_an_unbuilt_lease_in_the_seen_set_is_returned_without_ever_being_built() {
    let rig = rig();
    let l1 = rig.ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    rig.phys.lock().unwrap().live_a.push((0, 10_000));
    let u = rig.ledger.book_on_the_card(0, 5_000).unwrap(); // 借了，没建
    let l3 = rig.ledger.book_on_the_card(0, 3_000).unwrap();
    rig.phys.lock().unwrap().live_a.push((2, 3_000));
    let booked = [(0, 10_000), (1, 5_000), (2, 3_000)];
    let p = rig.phys.lock().unwrap().clone();
    println!("S5: L1 built, U booked unbuilt, L3 built: ledger {} truth {}", room(&rig), truth(&p, &booked));
    drop(u); // 没建就丢了：卡上什么都没留
    let booked = [(0, 10_000), (2, 3_000)];
    let p = rig.phys.lock().unwrap().clone();
    println!("S5: U returned unbuilt: ledger {} truth {}", room(&rig), truth(&p, &booked));
    println!("S5: again: ledger {} truth {}", room(&rig), truth(&p, &booked));
    drop(l1);
    drop(l3);
}

#[test]
fn fuzz_well_behaved_leases_built_in_full() {
    for (xd, free_on_return) in [(D, 0u64), (D, 50), (D, 100), (600, 100)] {
        let s = fuzz2(400, 120, false, xd, 50, true, free_on_return);
        println!("FUZZ full-build xd={xd} free_on_return%={free_on_return}: {s:?}");
    }
}

#[test]
fn fuzz_only_built_leases_are_returned() {
    for (xd, free_on_return) in [(D, 0u64), (D, 50), (D, 100)] {
        let s = fuzz3(400, 120, false, xd, 50, true, free_on_return, true);
        println!("FUZZ built-only-returns xd={xd} free_on_return%={free_on_return}: {s:?}");
    }
}

/// S7：设备起好了而 nvidia-smi 不按进程报（这个进程没列）：账给的与真能用的。
#[test]
fn s7_started_device_but_the_process_is_not_listed() {
    for built in [0u64, 10_000] {
        let free = EMPTY - D - built;
        let phys = Arc::new(Mutex::new(CardMemoryReading {
            free_mebibytes: EMPTY,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }));
        let shared = Arc::clone(&phys);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(vec![*shared.lock().unwrap()])),
            Box::new(|_p, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, D)),
            D,
        );
        let _l1 = ledger.book_on_the_card(0, 10_000).unwrap();
        ledger.record_the_device_started_on_the_card(0);
        phys.lock().unwrap().free_mebibytes = free;
        let given = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        // 真能再借的：空闲减借着的里还没建的（10000 - built），驱动 D 已经在卡上
        let truth = free - (10_000 - built);
        println!("S7 built={built}: ledger {given} truth {truth} under {}", i128::from(truth) - i128::from(given));
    }
}

/// S8：S1 之后，真空闲 19 550 放得下的一笔 17 000 借不借得到。
#[test]
fn s8_a_need_that_truly_fits_after_a_returned_lease_was_never_seen() {
    let rig = rig();
    let l1 = rig.ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.live_a.push((1, 10_000));
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    drop(l1);
    rig.phys.lock().unwrap().live_a.clear();
    let truth_now = truth(&rig.phys.lock().unwrap().clone(), &[]);
    let result = rig.ledger.book_on_the_card(0, 27_000);
    println!("S8: truth {truth_now}; book 27000 -> {}", match &result { Ok(_) => "Ok".to_string(), Err(e) => format!("{e:?}") });
}

/// S10：实现员那组 P1 的数（空卡 32 110、长借据 10 000、十笔 1 000 短借据各借、建、丢、卡上马上还给驱动）：每还一笔账给的与真能借的。
#[test]
fn s10_the_p1_numbers() {
    let rig = rig();
    let long = rig.ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = rig.phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.live_a.push((0, 10_000));
    }
    rig.ledger.record_the_device_started_on_the_card(0);
    let mut line = format!("S10 long built: ledger {} truth {};", room(&rig), truth(&rig.phys.lock().unwrap().clone(), &[(0, 10_000)]));
    for n in 1..=10usize {
        let short = rig.ledger.book_on_the_card(0, 1_000).unwrap();
        rig.phys.lock().unwrap().live_a.push((n, 1_000));
        drop(short);
        rig.phys.lock().unwrap().live_a.retain(|(i, _)| *i == 0);
        let t = truth(&rig.phys.lock().unwrap().clone(), &[(0, 10_000)]);
        line += &format!(" #{n} ledger {} truth {t};", room(&rig));
    }
    println!("{line}");
    drop(long);
}
