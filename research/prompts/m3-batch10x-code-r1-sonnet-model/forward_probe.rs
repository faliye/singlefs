//! 正推腿模型（m3-batch10x-code-r1）：只用 `singlefs_harness::gpu_memory_ledger` 的公开接口，喂假的显存读数，不要显卡。
//! 放进 crates/singlefs-harness/tests/ 下编（在草稿目录的仓副本里跑，不改仓里的文件）。
//! 探针：
//!   P1 读数一致时，账给的「能借的量」对一张格点上的每一种状态与独立算的「定义式」「安全上界」逐格比。
//!   P2 先读空闲、隔一会儿再读这个进程实际占的（卡池 `memory_of_the_cards_on_these_buses` 的次序）：读数不一致时给出的量超出多少。
//!   P3 多线程在同一本账上借、建、还（假的卡把借的量真占上），数卡被占超过总量的次数；读数一致、先读空闲再读占的、先读占的再读空闲各一遍。
//!   P4 每张卡此刻都借不出一个状态时，按状态数的容量全是 0（产品路径第 ① 段的预检拿它判「没有一张卡装得下」），账自己却报借不到。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, card_room_for_this_process,
    most_states_whose_need_fits_the_capacity, place_states_on_the_cards_that_book_now,
    BookedByThisProcessOnACard, BookingRefusal, CardMemoryReading, DriverShareOnTheCard,
    GpuMemoryLedger, ScratchAttemptOnACard,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

const DRIVER: u64 = 2560;
const TOTAL: u64 = 16_311;
/// 实测：本机一个起了设备、还没建缓冲的 wgpu 进程在 nvidia-smi 里占 211 MiB（本轮 host-resources 读到的 pid 3870820）。
const DRIVER_ACTUAL: u64 = 211;

/// 卡池 `open_the_card_pool` 里那一段容量算法（`memory_budget_mebibytes_of` 在 checker 档、这里照它的两行重写）。
fn pool_capacity(env: Option<u64>, configured: Option<u64>, may_hold: u64) -> u64 {
    let budget = match configured {
        Some(quota) => quota.min(may_hold),
        None => may_hold / 4 * 3,
    };
    card_ledger_capacity_mebibytes(env, configured, budget, may_hold, DRIVER)
}

/// 独立按文件头「口径」写的能借的量（i128 算，不用 saturating）：额度变量 − B；配置额度 − D − B（没配时 M 的 3/4 − D − B）；
/// F − 借着的连驱动那一份里还没实际占的（没登记的这个进程还要给驱动留一份 D）。
fn definition_room(
    env: Option<u64>,
    configured: Option<u64>,
    free: u64,
    booked: u64,
    occupied: Option<u64>,
    share: DriverShareOnTheCard,
) -> u64 {
    let registered = share != DriverShareOnTheCard::NotRegistered;
    let promised = booked + if registered { DRIVER } else { 0 };
    let occupied_of_promised = match (occupied, share) {
        (Some(occupied), _) => occupied.min(promised),
        (None, DriverShareOnTheCard::DeviceStarted) => DRIVER.min(promised),
        (None, _) => 0,
    };
    let not_yet = (promised - occupied_of_promised) as i128;
    let driver_to_come = if registered { 0 } else { DRIVER as i128 };
    let may_hold = free + occupied_of_promised;
    let mut best = free as i128 - not_yet - driver_to_come;
    let share_cap = match configured {
        Some(quota) => quota as i128 - DRIVER as i128 - booked as i128,
        None => (may_hold / 4 * 3) as i128 - DRIVER as i128 - booked as i128,
    };
    best = best.min(share_cap);
    if let Some(env) = env {
        best = best.min(env as i128 - booked as i128);
    }
    best.max(0) as u64
}

/// 假的卡：别的进程占着 `other`，这个进程的驱动实际占 `driver_actual`（起了设备之后），每笔借据建好的缓冲 `allocated`。
struct World {
    other: u64,
    device_started: bool,
    allocated: BTreeMap<u64, u64>,
    overflow_events: u64,
    worst_overshoot: u64,
}

impl World {
    fn used(&self) -> u64 {
        self.other
            + if self.device_started { DRIVER_ACTUAL } else { 0 }
            + self.allocated.values().sum::<u64>()
    }
    fn free(&self) -> u64 {
        TOTAL.saturating_sub(self.used())
    }
    fn occupied_by_this_process(&self) -> Option<u64> {
        self.device_started
            .then(|| DRIVER_ACTUAL + self.allocated.values().sum::<u64>())
    }
    fn check_overflow(&mut self) {
        if self.used() > TOTAL {
            self.overflow_events += 1;
            self.worst_overshoot = self.worst_overshoot.max(self.used() - TOTAL);
        }
    }
}

fn room_of(
    env: Option<u64>,
    configured: Option<u64>,
    free: u64,
    booked: u64,
    occupied: Option<u64>,
    share: DriverShareOnTheCard,
) -> u64 {
    card_room_for_this_process(
        CardMemoryReading {
            free_mebibytes: free,
            total_mebibytes: TOTAL,
            occupied_by_this_process_mebibytes: occupied,
        },
        BookedByThisProcessOnACard {
            leases_mebibytes: booked,
            driver_share: share,
        },
        DRIVER,
        &|may_hold| pool_capacity(env, configured, may_hold),
    )
    .room_now_mebibytes
}

/// P1a：任意读数（含读数自相矛盾的），账给的量与独立写的定义式逐格相等。
#[test]
fn p1a_room_equals_the_definition_on_a_grid() {
    let mut cells = 0u64;
    let mut mismatches = Vec::new();
    for free in [0u64, 1000, 2560, 3000, 5000, 9000, 16000] {
        for booked in [0u64, 500, 3000, 8000] {
            for share in [
                DriverShareOnTheCard::NotRegistered,
                DriverShareOnTheCard::RegisteredDeviceNotStarted,
                DriverShareOnTheCard::DeviceStarted,
            ] {
                if share == DriverShareOnTheCard::NotRegistered && booked != 0 {
                    continue;
                }
                let occupied_choices: Vec<Option<u64>> = vec![
                    None,
                    Some(0),
                    Some(DRIVER_ACTUAL),
                    Some(DRIVER_ACTUAL + booked / 2),
                    Some(DRIVER_ACTUAL + booked),
                    Some(DRIVER_ACTUAL + booked + 700),
                ];
                for occupied in occupied_choices {
                    for env in [None, Some(4000u64), Some(20000)] {
                        for configured in [None, Some(8000u64), Some(TOTAL)] {
                            cells += 1;
                            let got = room_of(env, configured, free, booked, occupied, share);
                            let want = definition_room(env, configured, free, booked, occupied, share);
                            if got != want {
                                mismatches.push(format!(
                                    "free={free} booked={booked} {share:?} occ={occupied:?} env={env:?} cfg={configured:?}: code={got} definition={want}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    println!("P1a cells={cells} mismatches={}", mismatches.len());
    for line in mismatches.iter().take(20) {
        println!("P1a MISMATCH {line}");
    }
    assert!(mismatches.is_empty());
}

/// P1b：读数一致（空闲、实际占的取自同一个假世界）时，没有额度约束的那一格（配置额度取卡的总量、没设额度变量）：
/// 账给的量恰等于「这张卡总量 − 别人占的 − 一份驱动 − 借着的」（独立的安全上界），不多也不少；有额度约束的格不超过它。
#[test]
fn p1b_room_equals_the_safe_bound_when_the_reading_is_coherent() {
    let mut cells = 0u64;
    let mut over = Vec::new();
    let mut under = Vec::new();
    for other in (0..=15000u64).step_by(500) {
        for started in [false, true] {
            for booked in [0u64, 500, 2000, 6000] {
                for realized_tenths in [0u64, 5, 10] {
                    let realized = booked * realized_tenths / 10;
                    if !started && realized != 0 {
                        continue;
                    }
                    let share = if started {
                        DriverShareOnTheCard::DeviceStarted
                    } else if booked > 0 {
                        DriverShareOnTheCard::RegisteredDeviceNotStarted
                    } else {
                        DriverShareOnTheCard::NotRegistered
                    };
                    let used = other + if started { DRIVER_ACTUAL } else { 0 } + realized;
                    if used > TOTAL {
                        continue;
                    }
                    let free = TOTAL - used;
                    let occupied = started.then_some(DRIVER_ACTUAL + realized);
                    let safe = (TOTAL as i128 - other as i128 - DRIVER as i128 - booked as i128).max(0) as u64;
                    for (env, configured) in [(None, Some(TOTAL)), (Some(4000), Some(8000)), (None, None)] {
                        cells += 1;
                        let got = room_of(env, configured, free, booked, occupied, share);
                        if got > safe {
                            over.push(format!("other={other} started={started} booked={booked} realized={realized} env={env:?} cfg={configured:?}: room={got} safe={safe}"));
                        }
                        if (env, configured) == (None, Some(TOTAL)) && got != safe {
                            under.push(format!("other={other} started={started} booked={booked} realized={realized}: room={got} safe={safe}"));
                        }
                    }
                }
            }
        }
    }
    println!("P1b cells={cells} over_the_safe_bound={} not_equal_when_unquoted={}", over.len(), under.len());
    for line in over.iter().chain(under.iter()).take(20) {
        println!("P1b {line}");
    }
    assert!(over.is_empty() && under.is_empty());
}

/// P2：卡池 `memory_of_the_cards_on_these_buses` 先读空闲（一次 nvidia-smi）、再读这个进程占的（又一次 nvidia-smi）；
/// 两次之间这个进程里已经借了的那一笔把缓冲建起来，空闲读的是建之前的、占的读的是建之后的，同一笔被算了两遍。
#[test]
fn p2_a_free_read_before_the_buffers_and_an_occupied_read_after_them_lend_the_same_memory_twice() {
    println!("P2 other booked realized_between | order | room | safe_at_the_second_read | overshoot");
    let mut worst = 0u64;
    for (other, booked, realized_between) in [(0u64, 6000u64, 6000u64), (3000, 6000, 3000), (3000, 2000, 2000), (0, 6000, 0)] {
        let before = other + DRIVER_ACTUAL;
        let after = before + realized_between;
        let free_before = TOTAL - before;
        let free_after = TOTAL - after;
        let occupied_before = DRIVER_ACTUAL;
        let occupied_after = DRIVER_ACTUAL + realized_between;
        let safe_after = (TOTAL as i128 - other as i128 - DRIVER as i128 - booked as i128).max(0) as u64;
        let share = DriverShareOnTheCard::DeviceStarted;
        for (order, free, occupied) in [
            ("free-then-occupied (code order)", free_before, occupied_after),
            ("occupied-then-free", free_after, occupied_before),
            ("coherent-after", free_after, occupied_after),
        ] {
            let room = room_of(None, Some(TOTAL), free, booked, Some(occupied), share);
            let overshoot = room.saturating_sub(safe_after);
            println!("P2 {other} {booked} {realized_between} | {order} | {room} | {safe_after} | {overshoot}");
            if order.starts_with("free-then") {
                worst = worst.max(overshoot);
            } else {
                assert_eq!(overshoot, 0, "{order}");
            }
        }
    }
    println!("P2 worst_overshoot_of_the_code_order={worst}");
    assert!(worst > 0, "没有量出超出：这条探针没有分辨力");
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ReadOrder {
    Coherent,
    FreeThenOccupied,
    OccupiedThenFree,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ReturnOrder {
    FreeThenDrop,
    DropThenFree,
}

struct Summary {
    bookings: u64,
    refusals: u64,
    overflow_events: u64,
    worst_overshoot: u64,
    promise_overflow_events: u64,
    worst_promise_overshoot: u64,
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn run_threads(read_order: ReadOrder, return_order: ReturnOrder, other: u64, seed: u64) -> Summary {
    let world = Arc::new(Mutex::new(World {
        other,
        device_started: false,
        allocated: BTreeMap::new(),
        overflow_events: 0,
        worst_overshoot: 0,
    }));
    let reader_world = Arc::clone(&world);
    let ledger = GpuMemoryLedger::new(
        1,
        Box::new(move || {
            // 两次 nvidia-smi 之间隔着的时间（真机上是几十毫秒）：这里隔 300 微秒
            let gap = || std::thread::sleep(std::time::Duration::from_micros(300));
            let (free, occupied) = match read_order {
                ReadOrder::Coherent => {
                    let w = reader_world.lock().unwrap();
                    (w.free(), w.occupied_by_this_process())
                }
                ReadOrder::FreeThenOccupied => {
                    let free = reader_world.lock().unwrap().free();
                    gap();
                    (free, reader_world.lock().unwrap().occupied_by_this_process())
                }
                ReadOrder::OccupiedThenFree => {
                    let occupied = reader_world.lock().unwrap().occupied_by_this_process();
                    gap();
                    (reader_world.lock().unwrap().free(), occupied)
                }
            };
            Ok(vec![CardMemoryReading {
                free_mebibytes: free,
                total_mebibytes: TOTAL,
                occupied_by_this_process_mebibytes: occupied,
            }])
        }),
        Box::new(|_position, may_hold| pool_capacity(None, Some(TOTAL), may_hold)),
        DRIVER,
    );
    let started = std::sync::Once::new();
    let live = Mutex::new((0u64, 0u64, 0u64));
    let bookings = std::sync::atomic::AtomicU64::new(0);
    let refusals = std::sync::atomic::AtomicU64::new(0);
    std::thread::scope(|scope| {
        for thread_number in 0..8u64 {
            let (world, ledger, started, bookings, refusals, live) = (&world, &ledger, &started, &bookings, &refusals, &live);
            scope.spawn(move || {
                let mut random = seed.wrapping_mul(2654435761).wrapping_add(thread_number * 977 + 1) | 1;
                for _ in 0..150 {
                    let need = 200 + xorshift(&mut random) % 4800;
                    match ledger.book_on_the_card(0, need) {
                        Err(BookingRefusal::NoCardHasRoomNow { .. }) => {
                            refusals.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            std::thread::yield_now();
                        }
                        Err(other) => panic!("{other:?}"),
                        Ok(booking) => {
                            bookings.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            let id = booking.lease_identifier();
                            {
                                let mut l = live.lock().unwrap();
                                l.0 += need;
                                if other + DRIVER + l.0 > TOTAL {
                                    l.1 += 1;
                                    l.2 = l.2.max(other + DRIVER + l.0 - TOTAL);
                                }
                            }
                            started.call_once(|| {
                                world.lock().unwrap().device_started = true;
                                ledger.record_the_device_started_on_the_card(0);
                            });
                            // 借到之后到缓冲真建起来之间的间隔（先登记、后建设备、再建缓冲）
                            std::thread::sleep(std::time::Duration::from_micros(xorshift(&mut random) % 400));
                            {
                                let mut w = world.lock().unwrap();
                                w.allocated.insert(id, need);
                                w.check_overflow();
                            }
                            std::thread::sleep(std::time::Duration::from_micros(xorshift(&mut random) % 1500));
                            live.lock().unwrap().0 -= need;
                            match return_order {
                                ReturnOrder::FreeThenDrop => {
                                    world.lock().unwrap().allocated.remove(&id);
                                    drop(booking);
                                }
                                ReturnOrder::DropThenFree => {
                                    drop(booking);
                                    std::thread::sleep(std::time::Duration::from_micros(xorshift(&mut random) % 400));
                                    world.lock().unwrap().allocated.remove(&id);
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    let w = world.lock().unwrap();
    let (promise_events, promise_worst) = {
        let l = live.lock().unwrap();
        (l.1, l.2)
    };
    Summary {
        bookings: bookings.load(std::sync::atomic::Ordering::Relaxed),
        refusals: refusals.load(std::sync::atomic::Ordering::Relaxed),
        overflow_events: w.overflow_events,
        worst_overshoot: w.worst_overshoot,
        promise_overflow_events: promise_events,
        worst_promise_overshoot: promise_worst,
    }
}

/// P3：8 条线程在同一本账上借、建、还。数两样：卡真被占超过总量（card_overflow）、账上借着的合计加驱动那一份加别人占的超过总量
/// （promise_overflow：账自己的承诺被破）。只断言两件事：读数一致且先还内存再销账的那一支一次都不该有；先读空闲再读占的（卡池的次序）
/// 要量得出来（否则这条探针没有分辨力）。别的格只打印。
#[test]
fn p3_many_threads_on_one_ledger_by_read_order() {
    println!("P3 other read_order return_order seed | bookings refusals | card_overflow worst_MiB | promise_overflow worst_MiB");
    let mut coherent_clean_violations = 0u64;
    let mut code_order_promise_overflows = 0u64;
    for other in [0u64, 3000] {
        for read_order in [ReadOrder::Coherent, ReadOrder::OccupiedThenFree, ReadOrder::FreeThenOccupied] {
            for return_order in [ReturnOrder::FreeThenDrop, ReturnOrder::DropThenFree] {
                for seed in 1..=3u64 {
                    let s = run_threads(read_order, return_order, other, seed);
                    println!(
                        "P3 {other} {read_order:?} {return_order:?} {seed} | {} {} | {} {} | {} {}",
                        s.bookings, s.refusals, s.overflow_events, s.worst_overshoot,
                        s.promise_overflow_events, s.worst_promise_overshoot
                    );
                    if read_order == ReadOrder::FreeThenOccupied {
                        code_order_promise_overflows += s.promise_overflow_events;
                    }
                    if read_order == ReadOrder::Coherent && return_order == ReturnOrder::FreeThenDrop {
                        coherent_clean_violations += s.overflow_events + s.promise_overflow_events;
                    }
                }
            }
        }
    }
    println!("P3 coherent_and_freed_before_refund violations={coherent_clean_violations}; code_order(free-then-occupied) promise_overflow_events_total={code_order_promise_overflows}");
    assert_eq!(coherent_clean_violations, 0);
    assert!(code_order_promise_overflows > 0, "没量出来：探针没有分辨力");
}

/// P5：销账之后卡上还占着（wgpu 丢掉的缓冲等设备下一次收拾才还给驱动，`GpuCardLease::drop` 只 Poll 一次不等），
/// 同时这个进程另有一笔借了、缓冲还没建：占着的那一份（已丢的）被当成另一笔借据已经建好的那部分，另一笔还没建的那份没扣。
#[test]
fn p5_a_dropped_lease_still_occupying_the_card_is_counted_as_the_realization_of_a_live_lease_that_has_not_built_yet() {
    let other = 0u64;
    let live_booked = 4000u64; // 这个进程还借着、缓冲还没建的一笔
    let stale = 4000u64; // 已销账、卡上还没还的缓冲
    let free = TOTAL - other - DRIVER_ACTUAL - stale;
    let occupied = DRIVER_ACTUAL + stale;
    let room = room_of(None, Some(TOTAL), free, live_booked, Some(occupied), DriverShareOnTheCard::DeviceStarted);
    // 之后真会占的：别人 + 这个进程的驱动那一份（按账的 D 留）+ 借着的这笔 + 卡上还没还的
    let safe = (TOTAL as i128 - other as i128 - DRIVER as i128 - live_booked as i128 - stale as i128).max(0) as u64;
    println!("P5 room={room} safe_while_the_stale_buffers_are_still_on_the_card={safe} overshoot={}", room as i128 - safe as i128);
    assert!(room > safe);
}
/// P4：每张卡此刻都借不出一个状态（别的进程占着）：账自己的按状态数借报借不到（本次未跑那一路）；
/// 而产品路径第 ① 段在借之前先算每张卡「按此刻能借的」装几个状态（`cards_of_the_pool_with_what_each_lends_now`：
/// `most_states_whose_need_fits_the_capacity(…, room_mebibytes, …)`），全是 0 时（`judge_my_blocks_on_gpu_cards_booking…` 的预检）不借、直接判红。
#[test]
fn p4_every_card_without_room_now_is_capacity_zero_before_the_booking_is_ever_tried() {
    let need_of = |states: usize| 3000 + 40 * states as u64; // 表 3000 MiB + 每个状态 40 MiB，随状态数单调
    let readings = Arc::new(Mutex::new(vec![
        CardMemoryReading { free_mebibytes: DRIVER + 2000, total_mebibytes: TOTAL, occupied_by_this_process_mebibytes: None },
        CardMemoryReading { free_mebibytes: DRIVER + 1000, total_mebibytes: TOTAL, occupied_by_this_process_mebibytes: None },
    ]));
    let shared = Arc::clone(&readings);
    let ledger = GpuMemoryLedger::new(
        2,
        Box::new(move || Ok(shared.lock().unwrap().clone())),
        Box::new(|_p, may_hold| pool_capacity(None, Some(TOTAL), may_hold)),
        DRIVER,
    );
    let rooms = ledger.card_rooms_mebibytes_now(&[0, 1]).unwrap();
    let capacities: Vec<u64> = rooms
        .iter()
        .map(|room| most_states_whose_need_fits_the_capacity(45, *room, &|states| need_of(states)) as u64)
        .collect();
    println!("P4 rooms_now={rooms:?} states_capacity_by_room_now={capacities:?} (空着时每张卡能借 {} MiB)", pool_capacity(None, Some(TOTAL), TOTAL));
    assert_eq!(capacities, vec![0, 0]);
    // 预检（crash_amplification.rs 的 `capacities.iter().all(|capacity| *capacity == 0)`）此刻会走到 `Err("没有一张卡装得下……")`；
    // 账自己：真去借，结局是借不到，不是判红
    let outcome = ledger.book_states_on_the_cards_with_room(&[0, 1], 45, &[45, 45], &|states| need_of(states as usize));
    match outcome {
        Err(BookingRefusal::NoCardHasRoomNow { need_mebibytes, .. }) => println!("P4 ledger_outcome=NoCardHasRoomNow need={need_mebibytes}"),
        Err(other) => panic!("账给的是 {other:?}"),
        Ok(_) => panic!("借到了"),
    }
    // 摆上这一刻读数的同一件事：place_states_on_the_cards_that_book_now 若真去试，两张卡各得 NoRoomNow
    let attempts = place_states_on_the_cards_that_book_now(45, &[45, 45], &mut |_slot, _states| ScratchAttemptOnACard::NoRoomNow { need_mebibytes: need_of(1) });
    println!("P4 attempts_if_tried={attempts:?}");
}

/// P6：借不到的卡重排之后，状态是压到一张卡上、还是照各卡此刻借得出的分开：两张卡，每张最多装 10 个，一共 10 个状态，一个状态 100 MiB 加表 1000 MiB。
#[test]
fn p6_states_a_card_cannot_book_now_move_on_instead_of_piling_on_one_card() {
    let need = |states: u64| 1000 + 100 * states;
    let cases: [(&str, [u64; 2], [Option<u64>; 2]); 4] = [
        ("both rooms ample", [DRIVER + 20_000, DRIVER + 20_000], [Some(10), None]),
        ("card0 room fits 4 states, card1 ample", [DRIVER + 1000 + 400, DRIVER + 20_000], [Some(4), Some(6)]),
        ("card0 no room, card1 ample", [DRIVER + 500, DRIVER + 20_000], [None, Some(10)]),
        ("both cards fit 4 states each", [DRIVER + 1000 + 400, DRIVER + 1000 + 400], [Some(4), Some(4)]),
    ];
    for (name, free, expected) in cases {
        let readings = Arc::new(Mutex::new(vec![
            CardMemoryReading { free_mebibytes: free[0], total_mebibytes: TOTAL, occupied_by_this_process_mebibytes: None },
            CardMemoryReading { free_mebibytes: free[1], total_mebibytes: TOTAL, occupied_by_this_process_mebibytes: None },
        ]));
        let shared = Arc::clone(&readings);
        let ledger = GpuMemoryLedger::new(
            2,
            Box::new(move || Ok(shared.lock().unwrap().clone())),
            Box::new(|_p, may_hold| pool_capacity(None, Some(TOTAL), may_hold)),
            DRIVER,
        );
        let booked = ledger.book_states_on_the_cards_with_room(&[0, 1], 10, &[10, 10], &need).unwrap();
        let got: Vec<Option<u64>> = booked.iter().map(|b| b.as_ref().map(|b| b.states)).collect();
        println!("P6 {name}: states_booked_per_card={got:?} expected={expected:?}");
        assert_eq!(got, expected.to_vec(), "{name}");
    }
}
