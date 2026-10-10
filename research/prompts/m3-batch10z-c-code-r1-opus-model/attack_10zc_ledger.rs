//! 第 10z 批第三块代码轮第一轮攻方模型：K2-1「甲之下还有没有一条路让账给的量大于此刻真能用的」，攻重起与 S6 的新状态
//! `DeviceLostNotStartedAgain`。冻结副本的显存账原样（一行不改）接一张假卡（照第 10z 批第一块攻方模型 `ledger_attack_10z.rs` 的卡，加丢设备）：
//! - 设备丢了：旧设备的驱动实占、建在旧设备上的缓冲、销了账卡上还没还的，一起留在卡上（旧设备那一堆），直到旧设备的借据都还了、旧设备收走；
//!   丢之前借的借据（「死借据」）再也建不上（设备不在了），不再长。
//! - 借：照卡池 `lease_card` 的次序（gpu_unit_checks.rs 第 1323 行起）先记账，设备没起着（头一次或丢了）就 `prepare`，放行才起新设备、记起好了，
//!   被拒就把这一笔丢掉（起不来的借据还掉）。新设备的驱动实占每次随机取 200–2560。
//! - 「真能再借的」= 空闲 − 活借据（不算死借据）还没建的 − 驱动还要长的（设备起着是 D − 实占，没起是 D）。
//! 每读一次比账给的与真能再借的；每次账放行（借到、重起放行）之后看许出去的有没有超过空闲（放行之前没超、之后超了记一次）。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, GpuMemoryBooking, GpuMemoryLedger,
    OccupiedByThisProcess,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

const DRIVER: u64 = 2560;
const EMPTY: u64 = 32_110;

#[derive(Clone, Default)]
struct Card {
    others: u64,
    driver_now: u64,
    device_started: bool,
    old_pile: u64,
    booked: BTreeMap<u64, u64>,
    built: BTreeMap<u64, u64>,
    dead: BTreeSet<u64>,
    returned_still_on_the_card: u64,
}

impl Card {
    fn this_process(&self) -> u64 {
        self.driver_now + self.old_pile + self.built.values().sum::<u64>() + self.returned_still_on_the_card
    }
    fn free(&self) -> u64 {
        EMPTY.saturating_sub(self.others + self.this_process())
    }
    fn reading(&self) -> CardMemoryReading {
        let mine = self.this_process();
        CardMemoryReading {
            free_mebibytes: self.free(),
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: if mine == 0 {
                OccupiedByThisProcess::NotListed
            } else {
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: mine }
            },
        }
    }
    fn promised_growth(&self) -> u64 {
        let unbuilt: u64 = self
            .booked
            .iter()
            .filter(|(id, _)| !self.dead.contains(id))
            .map(|(id, m)| m - self.built.get(id).copied().unwrap_or(0).min(*m))
            .sum();
        unbuilt + if self.device_started { DRIVER - self.driver_now } else { DRIVER }
    }
    fn truly_lendable(&self) -> i128 {
        i128::from(self.free()) - i128::from(self.promised_growth())
    }
}

#[derive(Default, Debug)]
struct Sweep {
    readings: u64,
    overgive_at_reading_max: i128,
    overgive_readings: u64,
    new_overcommit_after_a_booking: u64,
    new_overcommit_after_a_restart: u64,
    restarts_allowed: u64,
    restarts_refused: u64,
    restarts_refused_but_fit: u64,
    losses: u64,
    first_overgive: Option<String>,
}

fn histories(seed: u64, count: u64, steps: u64) -> Sweep {
    let mut x = seed;
    let mut next = move |bound: u64| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % bound
    };
    let mut sweep = Sweep::default();
    for _ in 0..count {
        let card = Arc::new(Mutex::new(Card { others: next(16_000), ..Card::default() }));
        let shared = Arc::clone(&card);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(vec![shared.lock().unwrap().reading()])),
            Box::new(|_position, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)),
            DRIVER,
        );
        let mut live: Vec<GpuMemoryBooking<'_>> = Vec::new();
        for _ in 0..steps {
            match next(9) {
                0 | 1 => {
                    let need = 1 + next(6_000);
                    let truth_before = card.lock().unwrap().truly_lendable();
                    let Ok(booking) = ledger.book_on_the_card(0, need) else { continue };
                    let id = booking.lease_identifier();
                    card.lock().unwrap().booked.insert(id, need);
                    // 账放行了一笔真放不下的（借之前真能再借的就比它少）
                    if truth_before < i128::from(need) {
                        sweep.new_overcommit_after_a_booking += 1;
                    }
                    if !card.lock().unwrap().device_started {
                        // 新设备的驱动那一份此刻真放不放得下（起之前、按真能再借的看）
                        let fits_before = card.lock().unwrap().truly_lendable() >= 0;
                        let others_before = card.lock().unwrap().others;
                        match ledger.prepare_the_driver_share_before_starting_the_device(0) {
                            Ok(()) => {
                                sweep.restarts_allowed += 1;
                                let mut c = card.lock().unwrap();
                                c.driver_now = 200 + next(DRIVER - 199);
                                c.device_started = true;
                                drop(c);
                                ledger.record_the_device_started_on_the_card(0);
                                // 重起放行了而新设备的驱动那一份连借着的真放不下
                                if !fits_before {
                                    sweep.new_overcommit_after_a_restart += 1;
                                }
                                assert_eq!(card.lock().unwrap().others, others_before);
                            }
                            Err(_) => {
                                sweep.restarts_refused += 1;
                                if fits_before {
                                    sweep.restarts_refused_but_fit += 1;
                                }
                                card.lock().unwrap().booked.remove(&id);
                                drop(booking);
                                continue;
                            }
                        }
                    }
                    live.push(booking);
                }
                2 if !live.is_empty() => {
                    let slot = usize::try_from(next(live.len() as u64)).unwrap();
                    let id = live[slot].lease_identifier();
                    let mut c = card.lock().unwrap();
                    if c.dead.contains(&id) || !c.device_started {
                        continue;
                    }
                    let booked = c.booked[&id];
                    let built = c.built.get(&id).copied().unwrap_or(0);
                    c.built.insert(id, built + next(booked - built + 1));
                }
                3 => {
                    let room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
                    let truth = card.lock().unwrap().truly_lendable().max(0);
                    let difference = i128::from(room) - truth;
                    sweep.readings += 1;
                    sweep.overgive_at_reading_max = sweep.overgive_at_reading_max.max(difference);
                    if difference > 0 {
                        sweep.overgive_readings += 1;
                        if sweep.first_overgive.is_none() {
                            let c = card.lock().unwrap();
                            sweep.first_overgive = Some(format!(
                                "room={room} truth={truth} free={} others={} driver_now={} started={} old_pile={} booked={:?} built={:?} dead={:?} returned_still={}",
                                c.free(), c.others, c.driver_now, c.device_started, c.old_pile, c.booked, c.built, c.dead, c.returned_still_on_the_card
                            ));
                        }
                    }
                }
                4 if !live.is_empty() => {
                    let slot = usize::try_from(next(live.len() as u64)).unwrap();
                    let booking = live.swap_remove(slot);
                    let id = booking.lease_identifier();
                    drop(booking);
                    let mut c = card.lock().unwrap();
                    c.booked.remove(&id);
                    if !c.dead.remove(&id) {
                        let built = c.built.remove(&id).unwrap_or(0);
                        if next(2) == 0 {
                            c.returned_still_on_the_card += built;
                        }
                    }
                }
                5 => card.lock().unwrap().returned_still_on_the_card = 0,
                6 if card.lock().unwrap().device_started && next(3) == 0 => {
                    sweep.losses += 1;
                    let mut c = card.lock().unwrap();
                    let pile = c.driver_now + c.built.values().sum::<u64>() + c.returned_still_on_the_card;
                    c.old_pile += pile;
                    c.built.clear();
                    c.returned_still_on_the_card = 0;
                    c.driver_now = 0;
                    c.device_started = false;
                    let ids: Vec<u64> = c.booked.keys().copied().collect();
                    c.dead.extend(ids);
                }
                7 => {
                    let mut c = card.lock().unwrap();
                    if c.dead.is_empty() {
                        c.old_pile = 0;
                    }
                }
                _ => {
                    let mut c = card.lock().unwrap();
                    c.others = c.others.saturating_sub(500).max(next(16_000).min(c.others + 500));
                }
            }
        }
        drop(live);
    }
    sweep
}

#[test]
fn k2_1_random_histories_with_lost_devices_and_restarts() {
    for (seed, count, steps) in [(0x9E37_79B9_7F4A_7C15_u64, 4_000, 120), (0x2545_F491_4F6C_DD1D, 4_000, 120)] {
        let s = histories(seed, count, steps);
        println!("ATTACK k2_1=random-restarts seed={seed:#x} histories={count} steps_each={steps} {s:?}");
    }
}
