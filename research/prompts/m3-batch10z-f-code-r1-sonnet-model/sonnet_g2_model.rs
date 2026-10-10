//! 正推腿自己写的模型（不是入库装置）：真 GpuMemoryLedger + 一张假的物理卡，
//! 查「账给的量大于此刻真能再借的」（K3-1）与「账一直少给」（K1-2、K1-3）。只用公开接口。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, DeviceWaitBeforeReturningTheLease,
    GpuMemoryBooking, GpuMemoryLedger, OccupiedByThisProcess,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

const T: u64 = 30_000;
const D: u64 = 2_560;

#[derive(Default)]
struct Phys {
    others: u64,
    /// (代号, 上下文占的 MiB)：还没释放的设备
    contexts: Vec<(u32, u64)>,
    /// 借据号 -> (设备代号, 缓冲占的 MiB)
    buffers: BTreeMap<u64, (u32, u64)>,
}

impl Phys {
    fn occupied(&self) -> u64 {
        self.contexts.iter().map(|c| c.1).sum::<u64>() + self.buffers.values().map(|b| b.1).sum::<u64>()
    }
    fn free(&self) -> i64 {
        T as i64 - self.others as i64 - self.occupied() as i64
    }
}

struct Slot<'l> {
    booking: Option<GpuMemoryBooking<'l>>,
    id: u64,
    size: u64,
    generation: u32,
    /// 等失败又丢掉了（不会再建了）
    kept: bool,
    dropped: bool,
}

struct World<'l> {
    ledger: &'l GpuMemoryLedger,
    phys: Arc<Mutex<Phys>>,
    slots: Vec<Slot<'l>>,
    current_generation: Option<u32>,
    current_lost: bool,
    next_generation: u32,
    /// 失败等之后缓冲留到下一次别人等成功（poll）才还
    freed_by_next_poll: Vec<u64>,
    log: Vec<String>,
}

fn new_ledger(phys: &Arc<Mutex<Phys>>) -> GpuMemoryLedger {
    let p = phys.clone();
    GpuMemoryLedger::new(
        1,
        Box::new(move || {
            let p = p.lock().unwrap();
            let occ = p.occupied();
            let free = p.free().max(0) as u64;
            Ok(vec![CardMemoryReading {
                free_mebibytes: free,
                total_less_reserved_mebibytes: T,
                occupied_by_this_process: if occ == 0 {
                    OccupiedByThisProcess::NotListed
                } else {
                    OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: occ }
                },
            }])
        }),
        Box::new(|_pos, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, D)),
        D,
    )
}

impl<'l> World<'l> {
    fn room(&self) -> u64 {
        self.ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0]
    }
    /// 物理上还要建的：没丢设备的、还没丢掉的借据 size-built，加驱动那一份
    fn needed_physically(&self) -> u64 {
        let p = self.phys.lock().unwrap();
        let mut need = 0u64;
        for s in &self.slots {
            if s.dropped || s.kept { continue; }
            if Some(s.generation) == self.current_generation && !self.current_lost {
                need += s.size - p.buffers.get(&s.id).map_or(0, |b| b.1);
            }
        }
        let driver = match self.current_generation {
            Some(g) if !self.current_lost => D - p.contexts.iter().find(|c| c.0 == g).map_or(0, |c| c.1).min(D),
            _ => D,
        };
        need + driver
    }
    fn safe_room(&self) -> i64 {
        let free = self.phys.lock().unwrap().free();
        free - self.needed_physically() as i64
    }
    fn release_unreferenced_generations(&mut self) {
        let mut p = self.phys.lock().unwrap();
        let held: Vec<u32> = self
            .slots
            .iter()
            .filter(|s| !s.dropped)
            .map(|s| s.generation)
            .chain(self.current_generation)
            .collect();
        let gone: Vec<u32> = p.contexts.iter().map(|c| c.0).filter(|g| !held.contains(g)).collect();
        for g in gone {
            p.contexts.retain(|c| c.0 != g);
            p.buffers.retain(|_, b| b.0 != g);
        }
    }
    fn book(&mut self, size: u64, dr: u64) -> Result<(), String> {
        let booking = match self.ledger.book_on_the_card(0, size) {
            Ok(b) => b,
            Err(_) => return Err("refused".into()),
        };
        let id = booking.lease_identifier();
        let need_start = self.current_generation.is_none() || self.current_lost;
        if need_start {
            if self.ledger.prepare_the_driver_share_before_starting_the_device(0).is_err() {
                drop(booking); // card_lent_for_the_booking 的 Err 路径：不等，直接销账
                return Err("restart refused".into());
            }
            let mut p = self.phys.lock().unwrap();
            if p.free() < dr as i64 {
                drop(p);
                drop(booking);
                return Err("start failed: physical OOM".into());
            }
            let g = self.next_generation;
            self.next_generation += 1;
            p.contexts.push((g, dr));
            drop(p);
            self.ledger.record_the_device_started_on_the_card(0);
            self.current_generation = Some(g);
            self.current_lost = false;
            self.release_unreferenced_generations();
        }
        let generation = self.current_generation.unwrap();
        self.slots.push(Slot { booking: Some(booking), id, size, generation, kept: false, dropped: false });
        let free_true = self.phys.lock().unwrap().free();
        let needed = self.needed_physically();
        if free_true < needed as i64 {
            return Err(format!("OVERGIVE_AT_BOOKING size={size} free_true={free_true} needed_physically={needed}"));
        }
        Ok(())
    }
    fn build(&mut self, i: usize, amount: u64) -> bool {
        let s = &self.slots[i];
        if s.dropped || s.kept || Some(s.generation) != self.current_generation || self.current_lost {
            return true;
        }
        let mut p = self.phys.lock().unwrap();
        let have = p.buffers.get(&s.id).map_or(0, |b| b.1);
        let x = amount.min(s.size - have);
        if x == 0 { return true; }
        if p.free() < x as i64 {
            return false; // 物理上 Out of Memory：账多给了
        }
        let e = p.buffers.entry(s.id).or_insert((s.generation, 0));
        e.1 += x;
        true
    }
    fn shrink(&mut self, i: usize, amount: u64) {
        let s = &self.slots[i];
        if s.dropped { return; }
        let mut p = self.phys.lock().unwrap();
        if let Some(e) = p.buffers.get_mut(&s.id) { e.1 = e.1.saturating_sub(amount); }
    }
    fn lose_device(&mut self) {
        if self.current_generation.is_some() { self.current_lost = true; }
    }
    fn give_back(&mut self, i: usize, wait_ok: bool, freed_by_next_poll: bool) {
        if self.slots[i].dropped { return; }
        let id = self.slots[i].id;
        let generation = self.slots[i].generation;
        let mut booking = self.slots[i].booking.take().unwrap();
        if wait_ok {
            let mut p = self.phys.lock().unwrap();
            p.buffers.remove(&id);
            // 这一次等也是一次 poll：别的等失败的借据留下的，标了下一次 poll 还的，这时还
            let pending: Vec<u64> = std::mem::take(&mut self.freed_by_next_poll);
            for other in pending { p.buffers.remove(&other); }
            drop(p);
            booking.take_in_the_wait_before_returning(DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished);
        } else {
            booking.take_in_the_wait_before_returning(DeviceWaitBeforeReturningTheLease::TheWaitFailed);
            self.slots[i].kept = true;
            if freed_by_next_poll { self.freed_by_next_poll.push(id); }
        }
        drop(booking);
        self.slots[i].dropped = true;
        let _ = generation;
        self.release_unreferenced_generations();
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
}

fn fresh<'l>(ledger: &'l GpuMemoryLedger, phys: &Arc<Mutex<Phys>>) -> World<'l> {
    World { ledger, phys: phys.clone(), slots: vec![], current_generation: None, current_lost: false, next_generation: 1, freed_by_next_poll: vec![], log: vec![] }
}

/// K1-3：设备丢了那一刻活着的借据 A，丢掉时等失败；B 触发重起并正常还；之后账一直少给
#[test]
fn k1_3_a_lease_alive_at_the_loss_that_fails_its_wait_keeps_the_card_in_the_unsplittable_state() {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let ledger = new_ledger(&phys);
    let mut w = fresh(&ledger, &phys);
    w.book(10_000, 600).unwrap(); // A
    assert!(w.build(0, 10_000));
    w.lose_device();
    w.book(5_000, 600).unwrap(); // B 触发重起（新设备）
    assert!(w.build(1, 5_000));
    w.give_back(0, false, true); // A：旧设备上的等失败（比如旧设备真丢了，poll 报错）
    w.give_back(1, true, false); // B 正常还
    // 此刻物理：旧设备已无人持有（A 丢掉了），新设备只剩上下文
    w.release_unreferenced_generations();
    let free = w.phys.lock().unwrap().free();
    let room = w.room();
    println!("K1-3 after: free_true={free} room_by_ledger={room} (safe_room={}) lease A on books={}", w.safe_room(), ledger.is_the_lease_on_the_books(0, w.slots[0].id));
    for _ in 0..3 { w.book(1_000, 600).map(|()| { let n = w.slots.len() - 1; w.give_back(n, true, false); }).ok(); }
    let room_later = w.room();
    println!("K1-3 later: room_by_ledger={room_later} (free_true={})", w.phys.lock().unwrap().free());
    assert!(room < w.safe_room() as u64, "账少给：{} < {}", room, w.safe_room());
    assert_eq!(room, room_later, "再借再还若干笔之后仍然一样少：出不来");
}

/// K1-2：等失败的借据照旧记着，没有设备丢失时永远不销：一笔笔累加，直到整张卡借不出
#[test]
fn k1_2_failed_waits_accumulate_on_the_books_until_nothing_can_be_lent() {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let ledger = new_ledger(&phys);
    let mut w = fresh(&ledger, &phys);
    let mut line = vec![];
    for round in 0..8 {
        match w.book(4_000, 600) {
            Ok(()) => {
                let n = w.slots.len() - 1;
                w.build(n, 4_000);
                // 等超时，之后活做完，缓冲在下一次 poll 还给驱动（物理上卡已经空了）
                w.give_back(n, false, true);
                w.phys.lock().unwrap().buffers.clear();
            }
            Err(e) => { line.push(format!("round {round}: {e}")); }
        }
        line.push(format!("round {round}: room_by_ledger={} free_true={}", w.room(), w.phys.lock().unwrap().free()));
    }
    for l in &line { println!("K1-2 {l}"); }
    // 5 笔 4000 之后卡上其实一个字节的借据缓冲都没有，账却不再给
    assert!(line.iter().any(|l| l.contains("refused")), "最终借不出");
}

fn run_random(seed: u64, steps: usize) -> Result<(), String> {
    let mut rng = Rng(seed);
    let phys = Arc::new(Mutex::new(Phys::default()));
    let ledger = new_ledger(&phys);
    let mut w = fresh(&ledger, &phys);
    for step in 0..steps {
        let alive: Vec<usize> = w.slots.iter().enumerate().filter(|(_, s)| !s.dropped).map(|(i, _)| i).collect();
        let op = rng.below(10);
        let desc;
        match op {
            0..=2 => {
                let size = 500 + rng.below(9) * 1_000;
                let dr = 100 + rng.below(D - 100);
                desc = format!("book {size} dr={dr}");
                if let Err(e) = w.book(size, dr) {
                    if e.starts_with("OVERGIVE") {
                        let tail: Vec<_> = w.log.iter().rev().take(12).rev().cloned().collect();
                        return Err(format!("seed {seed} step {step}: {e} last_ops={tail:?}"));
                    }
                }
            }
            3 | 4 if !alive.is_empty() => {
                let i = alive[rng.below(alive.len() as u64) as usize];
                let amount = 200 + rng.below(8) * 500;
                desc = format!("build #{} {amount}", w.slots[i].id);
                if !w.build(i, amount) {
                    return Err(format!("seed {seed} step {step}: 物理 OOM（账多给）{desc}; log={:?}", w.log));
                }
            }
            5 if !alive.is_empty() => {
                let i = alive[rng.below(alive.len() as u64) as usize];
                let amount = 200 + rng.below(8) * 500;
                desc = format!("shrink #{} {amount}", w.slots[i].id);
                w.shrink(i, amount);
            }
            6 | 7 if !alive.is_empty() => {
                let i = alive[rng.below(alive.len() as u64) as usize];
                let ok = rng.below(4) != 0;
                let later = rng.below(2) == 0;
                desc = format!("give_back #{} ok={ok} later={later}", w.slots[i].id);
                w.give_back(i, ok, later);
            }
            8 => { desc = "lose_device".into(); w.lose_device(); }
            _ => { desc = "noop".into(); }
        }
        w.log.push(desc.clone());
        let room = w.room() as i64;
        let safe = w.safe_room();
        // 设备丢了而账还不知道（要等下一笔借据触发重起才知道）的这一段，room 只是"上一次的口径"，重起时 prepare 会再判一次：不在这一段查
        if room > safe && !w.current_lost {
            let tail: Vec<_> = w.log.iter().rev().take(12).rev().cloned().collect();
            let free_true = w.phys.lock().unwrap().free();
            let needed = w.needed_physically();
            return Err(format!("seed {seed} step {step}: room_by_ledger={room} > safe_room={safe} (free_true={free_true}, needed={needed}) last_ops={tail:?}"));
        }
    }
    Ok(())
}

#[test]
fn k3_1_random_ops_never_let_the_ledger_give_more_than_is_physically_lendable() {
    let seeds: u64 = std::env::var("SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    let mut failures = vec![];
    for seed in 0..seeds {
        if let Err(e) = run_random(0x5eed_0000 + seed, 60) { failures.push(e); }
    }
    println!("K3-1 random: seeds={seeds} violations={}", failures.len());
    for f in failures.iter().take(3) { println!("K3-1 VIOLATION {f}"); }
    assert!(failures.is_empty(), "{} 个种子有过账多给", failures.len());
}

/// K3-2 / K1-3：设备丢了、重起好，旧设备那一代的借据 A 还活着（建满）时，账少给多少、A 还了之后回不回来
#[test]
fn k3_2_while_a_lease_of_the_lost_device_lives_the_ledger_lends_less_and_returns_to_normal_after_it_is_returned() {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let ledger = new_ledger(&phys);
    let mut w = fresh(&ledger, &phys);
    w.book(10_000, 600).unwrap(); // A
    assert!(w.build(0, 10_000));
    w.lose_device();
    w.book(5_000, 600).unwrap(); // B 触发重起
    assert!(w.build(1, 5_000));
    let free = w.phys.lock().unwrap().free();
    let (room_with_a_alive, safe_with_a_alive) = (w.room(), w.safe_room());
    println!("K3-2 A alive (Remain): free_true={free} room_by_ledger={room_with_a_alive} safe_room={safe_with_a_alive} short_by={}", safe_with_a_alive - room_with_a_alive as i64);
    w.give_back(0, true, false); // A 等成功、还了
    let (room_after_a, safe_after_a) = (w.room(), w.safe_room());
    println!("K3-2 A returned ok  : free_true={} room_by_ledger={room_after_a} safe_room={safe_after_a} short_by={}", w.phys.lock().unwrap().free(), safe_after_a - room_after_a as i64);
    assert!(room_with_a_alive as i64 <= safe_with_a_alive);
    // B（触发重起的那一笔，它拿的是新设备）也算"设备丢了那一刻活着的"：A 还了、B 还活着，账仍然少给
    assert!((room_after_a as i64) < safe_after_a);
    w.give_back(1, true, false);
    let (room_after_b, safe_after_b) = (w.room(), w.safe_room());
    println!("K3-2 B returned ok  : free_true={} room_by_ledger={room_after_b} safe_room={safe_after_b}", w.phys.lock().unwrap().free());
    assert_eq!(room_after_b as i64, safe_after_b, "A、B 都还了之后回到照读数算：不多不少");
}

/// K1-3 / K4：卡在 Remain 里的那一笔（旧设备那一代、等失败的 A），第二次丢设备重起时会被清掉；清掉之后全部还了就回到照读数算
#[test]
fn k1_3b_a_second_device_loss_clears_the_lease_stuck_from_the_first_one() {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let ledger = new_ledger(&phys);
    let mut w = fresh(&ledger, &phys);
    w.book(10_000, 600).unwrap(); // A
    assert!(w.build(0, 10_000));
    w.lose_device();
    w.book(5_000, 600).unwrap(); // B，触发第一次重起
    w.give_back(0, false, true); // A 等失败
    w.give_back(1, true, false); // B 正常还
    let stuck = (w.room() as i64, w.safe_room());
    w.lose_device();
    w.book(1_000, 600).unwrap(); // C，触发第二次重起：A 在这里被清掉
    w.give_back(2, true, false);
    let after = (w.room() as i64, w.safe_room());
    println!("K1-3b stuck(before 2nd loss): room={} safe={} | after 2nd restart and C returned: room={} safe={}", stuck.0, stuck.1, after.0, after.1);
    assert!(stuck.0 < stuck.1);
    assert_eq!(after.0, after.1, "第二次重起把 A 清掉，之后全还了回到照读数算");
}

/// K2-2：一张卡报 Out of Memory 把手里那一块交回，别的卡把它判了、也判完了其余的，之后因为叫停收尾：
/// 没有任何一块因为 Out of Memory 丢掉，照叫停收尾那一支仍然有头一句 Out of Memory 那一行可打
#[test]
fn k2_2_an_out_of_memory_that_lost_no_block_still_gets_the_not_run_line_when_the_run_stops_on_request() {
    use singlefs_harness::gpu_memory_ledger::{
        AfterARoundOfTheCards, BlocksForTheCards, RoundsOfTheCards, StopOfTheCards, StopOfTheRun,
    };
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(5);
    let blocks: BlocksForTheCards = rounds.take_the_blocks_of_the_round();
    let first = blocks.take_a_block(StopOfTheCards::NotRequested).expect("队里有块");
    blocks.give_back(first); // 卡 0 报 Out of Memory，把手里这一块交回
    for _ in 0..3 {
        let block = blocks.take_a_block(StopOfTheCards::NotRequested).expect("队里有块");
        rounds.stored_in_order().arrive(block, block, &mut |_block| Ok(()));
    }
    assert_eq!(blocks.take_a_block(StopOfTheCards::RequestedByTheCaller), None, "叫停了：没有判完在等的块，不再领");
    let after = rounds.after_the_round(&blocks, &[0, 1], [1u32], &[(0u32, "卡 0 报了 Out of Memory".to_string())], StopOfTheRun::Requested);
    let line = rounds.not_run_line_of_a_stop_after_an_out_of_memory("crash_amplification::run_from_environment");
    println!("K2-2 after={after:?} blocks_stored={} blocks_waiting={} line={line:?}", rounds.blocks_stored(), rounds.blocks_waiting());
    assert_eq!(after, AfterARoundOfTheCards::StoppedOnRequest);
    assert!(line.is_some());
    assert_eq!(rounds.blocks_stored(), 3);
}
