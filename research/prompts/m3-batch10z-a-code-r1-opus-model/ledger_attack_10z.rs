//! 第 10z 批代码轮第一轮攻方模型（m3-batch10z-a-code-r1，攻方腿）。
//! 照冻结副本的显存账原样（`singlefs_harness::gpu_memory_ledger`，一行不改）接一张假卡：卡上这个进程实际占着的由模型按
//! 「借据建了多少、销了账而卡上还没还的、驱动实占、丢了的旧设备」记，每次读数照它出（一份读数里空闲与占着的同一刻）。
//! 「真能再借的」= 空闲 − 每笔活着的借据还没建的那一截 − 驱动还要长的那一截（设备起着是 D − 实占，没起是 D）。
//! 跑法见同目录 run.sh：拷进草稿副本的 `crates/singlefs-harness/tests/`，`cargo test -p singlefs-harness --test ledger_attack_10z`。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, BookingRefusal, CardMemoryReading, GpuMemoryBooking,
    GpuMemoryLedger, OccupiedByThisProcess,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// 驱动的固定开销（卡池用的那个数）与一张空 5090 上能用的（总量减保留），同冻结副本库单测的常数。
const DRIVER: u64 = 2560;
const EMPTY: u64 = 32_110;
/// 驱动实占（第 10y 批攻方模型与第 10z 批库单测都取 600）。
const DRIVER_HOLDS_NOW: u64 = 600;

#[derive(Clone, Default)]
struct Card {
    others: u64,
    driver_now: u64,
    old_device: u64,
    booked: BTreeMap<u64, u64>,
    built: BTreeMap<u64, u64>,
    returned_still_on_the_card: u64,
    listed: bool,
    also_outside: bool,
    device_started: bool,
}

impl Card {
    fn this_process(&self) -> u64 {
        self.driver_now
            + self.old_device
            + self.built.values().sum::<u64>()
            + self.returned_still_on_the_card
    }
    fn free(&self) -> u64 {
        EMPTY - self.others - self.this_process()
    }
    fn reading(&self) -> CardMemoryReading {
        let mine = self.this_process();
        let occupied_by_this_process = if !self.listed || mine == 0 {
            OccupiedByThisProcess::NotListed
        } else if self.also_outside {
            OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: mine }
        } else {
            OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: mine }
        };
        CardMemoryReading {
            free_mebibytes: self.free(),
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process,
        }
    }
    fn truly_lendable(&self) -> u64 {
        let unbuilt: u64 = self
            .booked
            .iter()
            .map(|(id, m)| m - self.built.get(id).copied().unwrap_or(0).min(*m))
            .sum();
        let driver_growth = if self.device_started {
            DRIVER - self.driver_now
        } else {
            DRIVER
        };
        self.free().saturating_sub(unbuilt + driver_growth)
    }
}

struct Bench {
    card: Arc<Mutex<Card>>,
    ledger: GpuMemoryLedger,
}

impl Bench {
    fn new(card: Card) -> Self {
        let card = Arc::new(Mutex::new(card));
        let shared = Arc::clone(&card);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(vec![shared.lock().unwrap().reading()])),
            Box::new(|_position, may_hold| {
                card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)
            }),
            DRIVER,
        );
        Self { card, ledger }
    }
    fn book(&self, need: u64) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
        let booking = self.ledger.book_on_the_card(0, need)?;
        self.card
            .lock()
            .unwrap()
            .booked
            .insert(booking.lease_identifier(), need);
        Ok(booking)
    }
    fn start_the_device(&self) -> Result<(), BookingRefusal> {
        self.ledger
            .prepare_the_driver_share_before_starting_the_device(0)?;
        {
            let mut card = self.card.lock().unwrap();
            card.driver_now = DRIVER_HOLDS_NOW;
            card.device_started = true;
        }
        self.ledger.record_the_device_started_on_the_card(0);
        Ok(())
    }
    fn build(&self, booking: &GpuMemoryBooking<'_>, mebibytes: u64) {
        self.card
            .lock()
            .unwrap()
            .built
            .insert(booking.lease_identifier(), mebibytes);
    }
    /// 借据丢掉（销账）；`freed` 为假时它建的缓冲卡上还没还。
    fn give_back(&self, booking: GpuMemoryBooking<'_>, freed: bool) {
        let id = booking.lease_identifier();
        drop(booking);
        let mut card = self.card.lock().unwrap();
        card.booked.remove(&id);
        let built = card.built.remove(&id).unwrap_or(0);
        if !freed {
            card.returned_still_on_the_card += built;
        }
    }
    /// 这个进程别的线程借之前那一次读数（卡池每次借都在锁里现读一次），也是看账给多少的那一次。
    fn read_by_another_thread(&self) -> u64 {
        self.ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0]
    }
    fn truth(&self) -> u64 {
        self.card.lock().unwrap().truly_lendable()
    }
}

fn line(scenario: &str, step: &str, bench: &Bench) -> (u64, u64) {
    let ledger = bench.read_by_another_thread();
    let truth = bench.truth();
    println!("ATTACK scenario={scenario} step={step} ledger_room={ledger} truly_lendable={truth} difference={}", i128::from(ledger) - i128::from(truth));
    (ledger, truth)
}

/// 长借据 10 000 建满、驱动实占 600 的起点（第 10y 批 P1 那组数：真能再借 19 550）。
fn long_lease_built(bench: &Bench) -> GpuMemoryBooking<'_> {
    let long = bench.book(10_000).expect("空卡借得到");
    bench.start_the_device().expect("头一回起");
    bench.build(&long, 10_000);
    long
}

/// 甲：三条线程交错。每一轮 A、C 各借 1 000，A 建满之后第三条线程借之前读一次（这一次读数见到 A 建上的、比记着的多，
/// 换成见到的、连同那一刻活着的借据：A 与还没建的 C 都在里面），然后 C 建满，A、C 各丢、卡上马上还给驱动（最好的情形）。
#[test]
fn p1_a_lease_alive_and_unbuilt_when_another_thread_reads_costs_its_whole_amount_after_it_is_returned() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    let (start, truth) = line("P1-three-threads", "round=0", &bench);
    assert_eq!((start, truth), (19_550, 19_550));
    let mut last = start;
    for round in 1..=10 {
        let a = bench.book(1_000).unwrap();
        let c = bench.book(1_000).unwrap();
        bench.build(&a, 1_000);
        bench.read_by_another_thread();
        bench.build(&c, 1_000);
        bench.give_back(a, true);
        bench.give_back(c, true);
        let (ledger, truth) = line("P1-three-threads", &format!("round={round}"), &bench);
        assert_eq!(truth, 19_550);
        last = ledger;
    }
    assert_eq!(last, 9_550, "与第 10y 批 P1 第 10 笔之后的 9 550 相同");
    let refused = bench.book(18_550);
    println!("ATTACK scenario=P1-three-threads step=book-18550 outcome={:?}", refused.as_ref().map(|b| b.mebibytes()).map_err(|e| format!("{e:?}")));
    assert!(matches!(refused, Err(BookingRefusal::NoCardHasRoomNow { .. })));
    drop(long);
}

/// 乙：两条线程。A 借 1 000，建到一半（create_buffer_init 一个一个建，五个缓冲之间）时另一条线程借之前读一次，A 建满、丢、马上还。
#[test]
fn p1_a_lease_half_built_when_another_thread_reads_costs_the_unbuilt_half_after_it_is_returned() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    let mut last = 0;
    for round in 1..=10 {
        let a = bench.book(1_000).unwrap();
        bench.build(&a, 500);
        bench.read_by_another_thread();
        bench.build(&a, 1_000);
        bench.give_back(a, true);
        let (ledger, truth) = line("P1-half-built", &format!("round={round}"), &bench);
        assert_eq!(truth, 19_550);
        last = ledger;
    }
    assert_eq!(last, 19_550 - 10 * 500);
    drop(long);
}

/// 丙：一条线程顺次借、建、丢，只是借的量按 MiB 向上取整，建上的少 1 MiB（`mebibytes_crc32c_of_ranges_needs` 用 div_ceil），
/// 建满之后有一次读数（别的线程借之前）。一千轮。
#[test]
fn p1_a_lease_rounded_up_by_one_mebibyte_costs_one_mebibyte_each_time_it_is_returned() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    for _ in 1..=1_000 {
        let a = bench.book(2).unwrap();
        bench.build(&a, 1);
        bench.read_by_another_thread();
        bench.give_back(a, true);
    }
    let (ledger, truth) = line("P1-rounded", "rounds=1000", &bench);
    assert_eq!((ledger, truth), (19_550 - 1_000, 19_550));
    drop(long);
}

/// 对照：同一段历史里不让别的线程在借据活着时读（第 10z 批库单测那一形），每轮不多扣。
#[test]
fn p1_control_without_a_reading_while_the_short_lease_lives_costs_nothing() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    for _ in 1..=10 {
        let a = bench.book(1_000).unwrap();
        bench.build(&a, 1_000);
        bench.give_back(a, true);
    }
    let (ledger, truth) = line("P1-control", "rounds=10", &bench);
    assert_eq!((ledger, truth), (19_550, 19_550));
    drop(long);
}

/// 丁（第二节第 2 条假设被破的一形）：活着的借据 V 每趟派活建、派完丢（第 ② 段核对那一笔），另一笔 C 销了账、卡上还没还。
/// V 建着时有一次读数（见到的那一份含 V），C 销账，V 这一趟派完丢了缓冲，再读：销了账的那一份被压过头，账多给 V 缩掉的那一截。
#[test]
fn seen_shrinks_when_a_live_lease_drops_its_pass_buffers_and_the_returned_share_is_pressed_too_far() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    let scratch = bench.book(8_000).unwrap();
    bench.build(&scratch, 8_000);
    let verify = bench.book(3_000).unwrap();
    bench.build(&verify, 3_000);
    bench.read_by_another_thread();
    bench.give_back(scratch, false);
    bench.build(&verify, 0);
    let (ledger, truth) = line("P1-assumption-broken", "after-the-pass", &bench);
    assert_eq!((ledger, truth), (11_550, 8_550));
    let new = bench.book(ledger).expect("账给 11 550");
    bench.build(&new, ledger);
    bench.build(&verify, 3_000);
    let mut card = bench.card.lock().unwrap();
    card.driver_now = DRIVER;
    let on_the_card = card.this_process() + card.others;
    println!("ATTACK scenario=P1-assumption-broken step=all-grown on_the_card={on_the_card} empty={EMPTY} over={}", i128::from(on_the_card) - i128::from(EMPTY));
    assert_eq!(on_the_card, EMPTY + 3_000);
    drop(card);
    drop((new, verify, long));
}

/// 戊（第二节第 3 条）：设备丢了，旧设备的上下文（实占 600）与旧借据 a 建的 10 000 还在卡上（a 的持有者还拿着旧设备的克隆）。
/// 新的一笔 n=4 000 在丢了的设备上照「驱动那一份已在卡上」借到，起设备之前重新登记。外人占的从 0 扫到 20 000（步长 10），
/// 每一格分四类：借不到、借到而重起被拒且真放得下（多报本次未跑）、借到而重起被拒且真放不下、重起放行而真放不下（多给）。
#[test]
fn a_lost_device_started_again_is_refused_over_a_band_where_it_fits_and_never_overgiven() {
    let mut classes: BTreeMap<&str, (u64, u64, u64)> = BTreeMap::new();
    for others in (0..=20_000).step_by(10) {
        let bench = Bench::new(Card { listed: true, ..Card::default() });
        let a = bench.book(10_000).unwrap();
        bench.start_the_device().unwrap();
        bench.build(&a, 10_000);
        {
            let mut card = bench.card.lock().unwrap();
            card.others = others;
            card.driver_now = 0;
            card.old_device = DRIVER_HOLDS_NOW;
            card.device_started = false;
        }
        let class = match bench.book(4_000) {
            Err(_) => "booking-refused",
            Ok(n) => {
                let prepared = bench.ledger.prepare_the_driver_share_before_starting_the_device(0);
                let fits = bench.card.lock().unwrap().free() >= DRIVER + 4_000;
                let class = match (prepared.is_ok(), fits) {
                    (false, true) => "restart-refused-but-fits",
                    (false, false) => "restart-refused-and-does-not-fit",
                    (true, false) => "restart-allowed-but-does-not-fit",
                    (true, true) => "restart-allowed-and-fits",
                };
                drop(n);
                class
            }
        };
        drop(a);
        let entry = classes.entry(class).or_insert((0, u64::MAX, 0));
        entry.0 += 1;
        entry.1 = entry.1.min(others);
        entry.2 = entry.2.max(others);
    }
    for (class, (cells, lowest, highest)) in &classes {
        println!("ATTACK scenario=restart-sweep class={class} cells={cells} others_from={lowest} others_to={highest}");
    }
    assert!(!classes.contains_key("restart-allowed-but-does-not-fit"));
    assert_eq!(classes["restart-refused-but-fits"], (196, 13_000, 14_950));
}

/// 己（第二节第 3 条之后）：重起放行、新设备起好、n 建满；a 的持有者丢掉 a，旧设备连同 a 的缓冲还给驱动。
/// `reading_between` 为真时 n 建满之后、a 丢之前别的线程读过一次。之后能借的比真能再借的少多少，再读十次还少不少。
fn after_a_restart(reading_between: bool) -> (u64, u64, u64) {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let a = bench.book(10_000).unwrap();
    bench.start_the_device().unwrap();
    bench.build(&a, 10_000);
    {
        let mut card = bench.card.lock().unwrap();
        card.driver_now = 0;
        card.old_device = DRIVER_HOLDS_NOW;
        card.device_started = false;
    }
    let n = bench.book(4_000).unwrap();
    bench.start_the_device().expect("重起放得下");
    bench.build(&n, 4_000);
    if reading_between {
        bench.read_by_another_thread();
    }
    bench.give_back(a, true);
    bench.card.lock().unwrap().old_device = 0;
    let (ledger, truth) = line("restart-after", &format!("reading_between={reading_between}"), &bench);
    for _ in 0..10 {
        bench.read_by_another_thread();
    }
    let (again, _) = line("restart-after", &format!("reading_between={reading_between}-ten-more-readings"), &bench);
    drop(n);
    (ledger, truth, again)
}

#[test]
fn a_restart_with_the_old_device_still_on_the_card_leaves_a_shortfall_that_never_goes_away() {
    assert_eq!(after_a_restart(true), (23_590, 25_550, 23_590));
    assert_eq!(after_a_restart(false), (20_950, 25_550, 20_950));
}

/// 庚（第二节第 1 条乙附的代价）：这个进程在这张卡上另有不走账的设备（总线号记着），长借据 12 000 建满，再要 12 000。
#[test]
fn with_a_device_outside_the_books_on_the_bus_a_fitting_booking_is_refused() {
    for (listed, also_outside, name) in [(true, true, "also-outside"), (false, false, "not-listed")] {
        let bench = Bench::new(Card { listed, also_outside, ..Card::default() });
        let long = bench.book(12_000).unwrap();
        bench.start_the_device().unwrap();
        bench.build(&long, 12_000);
        let (ledger, truth) = line(&format!("yi-fu-{name}"), "long-12000-built", &bench);
        assert_eq!((ledger, truth), (4_950, 17_550));
        assert!(bench.book(12_000).is_err());
        drop(long);
    }
}

/// 辛：放开用户动作的随机历史（只固定起点：长借据 10 000 建满、驱动实占 600）。每步从下面几样里挑一样：借一笔（1..=4 000）、
/// 一笔活着的借据往上建一截、读一次（别的线程借之前那一次）、丢一笔（卡上马上还或还没还）、卡上把还没还的还掉；
/// `shrink` 为真时还许活着的借据往下缩（一趟派完丢了缓冲）。每次读与每次借都比账给的与真能再借的。
struct Sweep { histories: u64, overgive_at_reading_max: i128, overgive_bookings: u64, false_refusals: u64, shortfall_at_reading_max: i128 }

fn random_histories(seed: u64, histories: u64, steps: u64, shrink: bool, returns_left_on_the_card: bool, reads_only_when_built: bool) -> Sweep {
    let mut x = seed;
    let mut next = move |bound: u64| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % bound
    };
    let mut sweep = Sweep { histories, overgive_at_reading_max: 0, overgive_bookings: 0, false_refusals: 0, shortfall_at_reading_max: 0 };
    for _ in 0..histories {
        let bench = Bench::new(Card { listed: true, ..Card::default() });
        let long = long_lease_built(&bench);
        let mut live: Vec<(GpuMemoryBooking<'_>, u64, u64)> = Vec::new();
        for _ in 0..steps {
            let every_live_lease_built = live.iter().all(|(_, booked, built)| booked == built);
            let step = next(6);
            if reads_only_when_built && (step == 0 || step == 2) && !every_live_lease_built {
                continue;
            }
            match step {
                0 => {
                    let need = 1 + next(4_000);
                    let truth = bench.truth();
                    match bench.book(need) {
                        Ok(booking) => {
                            if need > truth {
                                sweep.overgive_bookings += 1;
                            }
                            live.push((booking, need, 0));
                        }
                        Err(_) => {
                            if need <= truth {
                                sweep.false_refusals += 1;
                            }
                        }
                    }
                }
                1 if !live.is_empty() => {
                    let slot = usize::try_from(next(live.len() as u64)).unwrap();
                    let (booking, booked, built) = &mut live[slot];
                    let target = if shrink && next(2) == 0 {
                        next(*booked + 1)
                    } else if reads_only_when_built {
                        *booked
                    } else {
                        *built + next(*booked - *built + 1)
                    };
                    *built = target;
                    bench.build(booking, target);
                }
                2 => {
                    let ledger = bench.read_by_another_thread();
                    let difference = i128::from(ledger) - i128::from(bench.truth());
                    sweep.overgive_at_reading_max = sweep.overgive_at_reading_max.max(difference);
                    sweep.shortfall_at_reading_max = sweep.shortfall_at_reading_max.max(-difference);
                }
                3 if !live.is_empty() => {
                    let slot = usize::try_from(next(live.len() as u64)).unwrap();
                    let (booking, _, _) = live.swap_remove(slot);
                    bench.give_back(booking, !returns_left_on_the_card || next(2) == 0);
                }
                4 if returns_left_on_the_card => {
                    bench.card.lock().unwrap().returned_still_on_the_card = 0;
                }
                _ => {}
            }
        }
        drop(live);
        drop(long);
    }
    sweep
}

#[test]
fn random_histories_with_the_user_steps_set_free() {
    for (name, shrink, left, only_built) in [
        ("control-reads-and-bookings-only-when-every-live-lease-is-built-returns-freed", false, false, true),
        ("grow-only-returns-freed", false, false, false),
        ("grow-only-returns-left-on-the-card", false, true, false),
        ("shrinking-live-leases-returns-left-on-the-card", true, true, false),
    ] {
        let s = random_histories(0x9E37_79B9_7F4A_7C15, 3_000, 80, shrink, left, only_built);
        println!("ATTACK scenario=random mode={name} histories={} steps_each=80 overgive_at_reading_max={} overgive_bookings={} false_refusals={} shortfall_at_reading_max={}", s.histories, s.overgive_at_reading_max, s.overgive_bookings, s.false_refusals, s.shortfall_at_reading_max);
    }
}

/// 壬：随机历史里对照那一档（读与借都只在每笔活着的借据建满时）照样读到少给，这一形是它：一条线程里两笔借据前后重叠。
/// a 建满、读过一次（见到的那一份含 a）；b 借、建满（b 不在那一刻活着的借据里）；a 丢、卡上马上还。再读：压销了账那一份时
/// 「实际占着的减见到的」正好是 b 建上的，b 被当成 a 还没还的，账少给 b 那么多，直到 b 也丢掉。
#[test]
fn a_lease_booked_after_the_reading_is_taken_for_the_returned_one_until_it_is_returned_itself() {
    let bench = Bench::new(Card { listed: true, ..Card::default() });
    let long = long_lease_built(&bench);
    let a = bench.book(4_000).unwrap();
    bench.build(&a, 4_000);
    bench.read_by_another_thread();
    let b = bench.book(3_000).unwrap();
    bench.build(&b, 3_000);
    bench.give_back(a, true);
    let (ledger, truth) = line("P1-overlap", "b-alive", &bench);
    assert_eq!((ledger, truth), (16_550 - 3_000, 16_550));
    let need = truth - 1_000;
    let refused = bench.book(need);
    println!("ATTACK scenario=P1-overlap step=book-{need} outcome={:?}", refused.as_ref().map(|b| b.mebibytes()).map_err(|e| format!("{e:?}")));
    assert!(refused.is_err());
    bench.give_back(b, true);
    let (ledger, truth) = line("P1-overlap", "b-returned", &bench);
    assert_eq!((ledger, truth), (19_550, 19_550));
    drop(long);
}
