//! 攻方腿 m3-batch10t-code-r1 的显存账模型（harness 档，只用 singlefs_harness::gpu_memory_ledger 的公开接口与一份打了两个改法的拷贝）。
//! 不上卡：一张「假卡」记总量、外人占的、每个记账进程实际占的；账本的空闲读法读假卡。账本放在临时目录，不碰真账本。
//! 今天的臂 = singlefs_harness::gpu_memory_ledger；改法甲（每次借在锁里现读空闲重夹估计）、改法乙（按状态数借照「容量 − 已记」压）
//! 在 tests/opus_model/fixed_ledger.rs（今天那份 1–1489 行的拷贝加补丁，补丁见模型目录 fixed_ledger.patch）。
//! 每格打一行 `MODEL scenario=… arm=… …`，复跑比这几行。

#[path = "opus_model/fixed_ledger.rs"]
mod fixed;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use singlefs_harness::gpu_memory_ledger as today;

/// 驱动那一份，照卡池（crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER）。
const DRIVER: u64 = 2560;
/// 一张 5090 的总显存（host-resources.py 读到的）。
const RTX_5090_TOTAL: u64 = 32_607;
/// 一张 5060 Ti 的总显存。
const RTX_5060_TI_TOTAL: u64 = 16_311;
/// 模型里一个状态的草稿区（MiB，约等于 SCRATCH_WORDS × 4 = 1 261 504 字节，取整到 1 MiB 不改结论）与表。
const MEBIBYTES_PER_STATE: u64 = 1;
const TABLES_MEBIBYTES: u64 = 64;
/// 等满的上限按比例缩到 300 毫秒（今天是 30 分钟）：等满的结局与等多久无关，只看这段时间里账变不变。
const SHORT_LONGEST_WAIT: Duration = Duration::from_millis(300);

/// 假卡：总量、外人占的、每个记账进程实际占的（进程号 → MiB）。
struct FakeCard {
    total: u64,
    outsider: Mutex<u64>,
    physical: Mutex<Vec<(u32, u64)>>,
}

impl FakeCard {
    fn new(total: u64) -> Arc<Self> {
        Arc::new(Self {
            total,
            outsider: Mutex::new(0),
            physical: Mutex::new(Vec::new()),
        })
    }
    fn set_outsider(&self, mebibytes: u64) {
        *self.outsider.lock().unwrap() = mebibytes;
    }
    fn allocate(&self, process: u32, mebibytes: u64) {
        self.physical.lock().unwrap().push((process, mebibytes));
    }
    fn release_all_of(&self, process: u32) {
        self.physical.lock().unwrap().retain(|(owner, _)| *owner != process);
    }
    fn demand(&self) -> u64 {
        *self.outsider.lock().unwrap() + self.physical.lock().unwrap().iter().map(|(_, m)| m).sum::<u64>()
    }
    fn free(&self) -> u64 {
        self.total.saturating_sub(self.demand())
    }
    /// 卡上要的比卡大多少（> 0 就是 OOM）。
    fn over(&self) -> u64 {
        self.demand().saturating_sub(self.total)
    }
}

/// 卡池给崩溃放量的额度，照 gpu_unit_checks.rs 的 memory_budget_mebibytes_of（配了额度取额度与空闲的小者，没配取空闲的 3/4）。
fn budget_of(configured: Option<u64>, free: u64) -> u64 {
    match configured {
        Some(quota) => quota.min(free),
        None => free / 4 * 3,
    }
}

/// 卡池开账时给的容量闭包，照 gpu_unit_checks.rs open_the_card_pool（没设额度变量）。
fn pool_capacity(configured: Vec<Option<u64>>) -> Box<dyn Fn(usize, u64) -> u64 + Send + Sync> {
    Box::new(move |position, estimate| {
        today::card_ledger_capacity_mebibytes(
            None,
            configured[position],
            budget_of(configured[position], estimate),
            estimate,
            DRIVER,
        )
    })
}

fn mebibytes_for_states(states: u64) -> u64 {
    TABLES_MEBIBYTES + states * MEBIBYTES_PER_STATE
}

struct Directory(PathBuf);
impl Directory {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("opus-ledger-model-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn process(pid: u32) -> today::RecordedProcess {
    today::RecordedProcess { process_identifier: pid, start_time_in_clock_ticks: 1 }
}
fn fixed_process(pid: u32) -> fixed::RecordedProcess {
    fixed::RecordedProcess { process_identifier: pid, start_time_in_clock_ticks: 1 }
}

/// 今天的臂：假进程 `pid` 开一本或几本卡的账，空闲读假卡。
fn open_today(directory: &Path, pid: u32, cards: &[Arc<FakeCard>], configured: Vec<Option<u64>>) -> today::GpuMemoryLedger {
    let cards = cards.to_vec();
    today::GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
        today::LedgerSettings {
            directory: directory.to_path_buf(),
            this_process: process(pid),
            is_alive: Box::new(|_| true),
            mebibytes_each_process_keeps_on_a_card: DRIVER,
            poll_interval_for_memory_returned_by_other_processes: Duration::from_millis(20),
            longest_wait: SHORT_LONGEST_WAIT,
        },
        (0..cards.len()).map(|i| format!("card-{i}")).collect(),
        &|position| Ok(cards[position].free()),
        pool_capacity(configured),
    )
    .expect("临时目录下的账本读写得了")
}

/// 改法的臂：同上，挂改法甲（`re_read`）、改法乙（`to_room`）。
fn open_fixed(directory: &Path, pid: u32, cards: &[Arc<FakeCard>], configured: Vec<Option<u64>>, re_read: bool, to_room: bool) -> fixed::GpuMemoryLedger {
    let cards = cards.to_vec();
    fixed::GpuMemoryLedger::open_with_the_attack_model_fixes(
        fixed::LedgerSettings {
            directory: directory.to_path_buf(),
            this_process: fixed_process(pid),
            is_alive: Box::new(|_| true),
            mebibytes_each_process_keeps_on_a_card: DRIVER,
            poll_interval_for_memory_returned_by_other_processes: Duration::from_millis(20),
            longest_wait: SHORT_LONGEST_WAIT,
        },
        (0..cards.len()).map(|i| format!("card-{i}")).collect(),
        Box::new(move |position| Ok(cards[position].free())),
        pool_capacity(configured),
        re_read,
        to_room,
    )
    .expect("临时目录下的账本读写得了")
}

/// 借一笔、借到就算「建了缓冲」（假卡上记实际占的；这个进程在这张卡上第一次借时连驱动那一份）。交回借到的量或拒的结局的名字。
fn book_and_allocate_today(ledger: &today::GpuMemoryLedger, card: &FakeCard, pid: u32, first: bool, need: u64) -> String {
    match ledger.book_waiting_for_room(need) {
        Ok(booking) => {
            card.allocate(pid, need + if first { DRIVER } else { 0 });
            let booked = booking.mebibytes();
            std::mem::forget(booking);
            format!("booked={booked}")
        }
        Err(today::BookingRefusal::NeedLargerThanEveryCard(r)) => format!("red_need_larger_than_every_card(capacity={})", r.largest_capacity_mebibytes),
        Err(today::BookingRefusal::WaitedPastTheLongestWait { .. }) => "waited_past_the_longest_wait".to_string(),
        Err(today::BookingRefusal::LedgerFileUnusable(e)) => format!("ledger_unusable({})", e.reason),
    }
}
fn book_and_allocate_fixed(ledger: &fixed::GpuMemoryLedger, card: &FakeCard, pid: u32, first: bool, need: u64) -> String {
    match ledger.book_waiting_for_room(need) {
        Ok(booking) => {
            card.allocate(pid, need + if first { DRIVER } else { 0 });
            let booked = booking.mebibytes();
            std::mem::forget(booking);
            format!("booked={booked}")
        }
        Err(fixed::BookingRefusal::NeedLargerThanEveryCard(r)) => format!("red_need_larger_than_every_card(capacity={})", r.largest_capacity_mebibytes),
        Err(fixed::BookingRefusal::WaitedPastTheLongestWait { .. }) => "waited_past_the_longest_wait".to_string(),
        Err(fixed::BookingRefusal::LedgerFileUnusable(e)) => format!("ledger_unusable({})", e.reason),
    }
}

/// S1（K1-1）：外人在这个进程唯一一次开账之后才来。一个进程、一张 5090、配了整卡额度（配 None 的另跑一遍）。
/// 开账读到整卡空着；外人随后占 20 000；这个进程照账把它的容量借满。
#[test]
fn s1_an_outsider_arriving_after_the_only_opening_is_never_seen() {
    for configured in [Some(RTX_5090_TOTAL), None] {
        for arm in ["today", "fix_a_re_read_at_every_booking"] {
            let directory = Directory::new(&format!("s1-{arm}-{configured:?}"));
            let card = FakeCard::new(RTX_5090_TOTAL);
            let (capacity_at_open, outcome) = if arm == "today" {
                let ledger = open_today(directory.path(), 100, &[card.clone()], vec![configured]);
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                card.set_outsider(20_000);
                (capacity, book_and_allocate_today(&ledger, &card, 100, true, capacity))
            } else {
                let ledger = open_fixed(directory.path(), 100, &[card.clone()], vec![configured], true, false);
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                card.set_outsider(20_000);
                let first = book_and_allocate_fixed(&ledger, &card, 100, true, capacity);
                let after = ledger.card_capacities_mebibytes().unwrap()[0];
                let second = book_and_allocate_fixed(&ledger, &card, 100, true, after);
                (capacity, format!("{first};then_capacity={after};{second}"))
            };
            println!(
                "MODEL scenario=s1 configured={configured:?} arm={arm} capacity_at_open={capacity_at_open} outsider=20000 outcome={outcome} card_demand={} card_total={} over_mib={}",
                card.demand(), card.total, card.over()
            );
        }
    }
}

/// S1b（对照：主 agent 定的零轮设计说的那一形）：进程 100 开账、借 20 000 而还没建缓冲（只起了设备）；外人占 10 000；
/// 进程 200 开账、借满它看到的。偏大的量照文件头说的不超过 min(外人, 借了没建的)。
#[test]
fn s1b_the_documented_form_stays_within_the_smaller_of_the_two() {
    let directory = Directory::new("s1b");
    let card = FakeCard::new(RTX_5090_TOTAL);
    let first = open_today(directory.path(), 100, &[card.clone()], vec![Some(RTX_5090_TOTAL)]);
    let lease = first.book_waiting_for_room(20_000).expect("空卡放得下");
    card.allocate(100, DRIVER);
    card.set_outsider(10_000);
    let second = open_today(directory.path(), 200, &[card.clone()], vec![Some(RTX_5090_TOTAL)]);
    let capacity = second.card_capacities_mebibytes().unwrap()[0];
    let room = capacity - (20_000 + DRIVER);
    let outcome = book_and_allocate_today(&second, &card, 200, true, room);
    card.allocate(100, 20_000);
    println!(
        "MODEL scenario=s1b arm=today second_capacity={capacity} second_booked_room={room} outcome={outcome} card_demand={} card_total={} over_mib={} documented_bound=min(10000,20000)=10000",
        card.demand(), card.total, card.over()
    );
    drop(lease);
}

/// S2（K2-1 判红那一向）：进程开账时外人占着 25 000；外人走了；GPU 用例借 10 000、产品路径借草稿区。
#[test]
fn s2_an_outsider_present_at_the_opening_and_gone_later_turns_bookings_red() {
    for configured in [Some(24 * 1024), None] {
        for arm in ["today", "fix_a_re_read_at_every_booking"] {
            let directory = Directory::new(&format!("s2-{arm}-{configured:?}"));
            let card = FakeCard::new(RTX_5090_TOTAL);
            card.set_outsider(25_000);
            let (lease_outcome, scratch_outcome, capacity) = if arm == "today" {
                let ledger = open_today(directory.path(), 100, &[card.clone()], vec![configured]);
                card.set_outsider(0);
                let lease = book_and_allocate_today(&ledger, &card, 100, true, 10_000);
                let scratch = match ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], 10_000, &[10_000], &mebibytes_for_states) {
                    Ok(b) => format!("scratch_booked_states={:?}", b.iter().map(|x| x.as_ref().map(|s| s.states)).collect::<Vec<_>>()),
                    Err(refusal) => format!("{:?}", today::scratch_booking_refusal_outcome("crash_amplification::run_from_environment", 0, &refusal)).chars().take(90).collect(),
                };
                (lease, scratch, ledger.card_capacities_mebibytes().unwrap()[0])
            } else {
                let ledger = open_fixed(directory.path(), 100, &[card.clone()], vec![configured], true, false);
                card.set_outsider(0);
                let lease = book_and_allocate_fixed(&ledger, &card, 100, true, 10_000);
                let scratch = match ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], 10_000, &[10_000], &mebibytes_for_states) {
                    Ok(b) => format!("scratch_booked_states={:?}", b.iter().map(|x| x.as_ref().map(|s| s.states)).collect::<Vec<_>>()),
                    Err(refusal) => format!("{:?}", fixed::scratch_booking_refusal_outcome("crash_amplification::run_from_environment", 0, &refusal)).chars().take(90).collect(),
                };
                (lease, scratch, ledger.card_capacities_mebibytes().unwrap()[0])
            };
            println!(
                "MODEL scenario=s2 configured={configured:?} arm={arm} free_now={} capacity_now={capacity} gpu_test_lease_10000={lease_outcome} product_path_scratch={scratch_outcome}",
                card.free()
            );
        }
    }
}

/// 产品路径借草稿区的结局写成一行：每张卡借到几个状态，或拒的名字。
fn scratch_line_today(result: Result<Vec<Option<today::StatesBookedOnACard<'_>>>, today::BookingRefusal>) -> String {
    match result {
        Ok(bookings) => {
            let states: Vec<Option<u64>> = bookings.iter().map(|b| b.as_ref().map(|s| s.states)).collect();
            std::mem::forget(bookings);
            format!("booked_states_per_card={states:?}")
        }
        Err(today::BookingRefusal::WaitedPastTheLongestWait { need_mebibytes, .. }) => format!("not_run_waited_past_the_longest_wait(need={need_mebibytes})"),
        Err(other) => format!("red({other:?})"),
    }
}
fn scratch_line_fixed(result: Result<Vec<Option<fixed::StatesBookedOnACard<'_>>>, fixed::BookingRefusal>) -> String {
    match result {
        Ok(bookings) => {
            let states: Vec<Option<u64>> = bookings.iter().map(|b| b.as_ref().map(|s| s.states)).collect();
            std::mem::forget(bookings);
            format!("booked_states_per_card={states:?}")
        }
        Err(fixed::BookingRefusal::WaitedPastTheLongestWait { need_mebibytes, .. }) => format!("not_run_waited_past_the_longest_wait(need={need_mebibytes})"),
        Err(other) => format!("red({other:?})"),
    }
}

/// 一张卡按它此刻的容量（不看别人借了多少）带得了几个状态：照 crash_amplification.rs states_each_card_of_the_pool_lends
/// （most_states_whose_need_fits_the_capacity 对 card_capacities_mebibytes_of 的容量）。
fn states_the_capacity_holds(capacity: u64, most: usize) -> u64 {
    today::most_states_whose_need_fits_the_capacity(most, capacity, &|s| mebibytes_for_states(s as u64)) as u64
}

/// S3（K2-1 白等那一向）：一张 5090、配整卡额度。进程 100（一条 GPU 用例或另一趟产品路径）借着 `held` 一直不还；
/// 进程 200 的大流（状态多于一张卡装得下的）走产品路径借草稿区。今天照整份容量压，借不到、等满报本次未跑；改法乙照剩下的压。
#[test]
fn s3_a_partial_booking_by_another_process_makes_a_large_flow_wait_out_the_longest_wait() {
    for held in [1u64, 4_000, 15_000] {
        for arm in ["today", "fix_b_compress_to_room"] {
            let directory = Directory::new(&format!("s3-{arm}-{held}"));
            let card = FakeCard::new(RTX_5090_TOTAL);
            let other = open_today(directory.path(), 100, &[card.clone()], vec![Some(RTX_5090_TOTAL)]);
            let kept = other.book_waiting_for_room(held).expect("空卡放得下");
            card.allocate(100, held + DRIVER);
            let pending = 1_000_000u64;
            let (holds, room, outcome) = if arm == "today" {
                let ledger = open_today(directory.path(), 200, &[card.clone()], vec![Some(RTX_5090_TOTAL)]);
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                let holds = states_the_capacity_holds(capacity, pending as usize);
                let room = capacity - held - DRIVER;
                (holds, room, scratch_line_today(ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], pending, &[holds], &mebibytes_for_states)))
            } else {
                let ledger = open_fixed(directory.path(), 200, &[card.clone()], vec![Some(RTX_5090_TOTAL)], false, true);
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                let holds = states_the_capacity_holds(capacity, pending as usize);
                let room = capacity - held - DRIVER;
                (holds, room, scratch_line_fixed(ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], pending, &[holds], &mebibytes_for_states)))
            };
            println!(
                "MODEL scenario=s3 held_by_other={held} arm={arm} states_each_card_holds={holds} room_mib={room} states_that_fit_the_room={} outcome={outcome}",
                states_the_capacity_holds(room, pending as usize)
            );
            drop(kept);
        }
    }
}

/// S4（K2-1 / K2-2）：两张卡（5090、5060 Ti）。取卡登记那一刻进程 300 把 5090 借满，进程 100 在 5060 Ti 上借着 4 000 一直不还。
/// 进程 200 登记（每张当场试一次）：只登记上 5060 Ti。紧接着进程 300 还了 5090。产品路径借草稿区时只看登记上的卡
/// （crash_amplification.rs：第 ① 段的卡就是取卡时登记上的那几张），在 5060 Ti 上照整份容量压，等满报本次未跑；5090 一直空着。
#[test]
fn s4_a_card_full_for_an_instant_at_registration_is_left_out_of_the_whole_run() {
    for arm in ["today", "fix_b_compress_to_room"] {
        let directory = Directory::new(&format!("s4-{arm}"));
        let big = FakeCard::new(RTX_5090_TOTAL);
        let small = FakeCard::new(RTX_5060_TI_TOTAL);
        let cards = [big.clone(), small.clone()];
        let configured = vec![None, None];
        let p300 = open_today(directory.path(), 300, &cards, configured.clone());
        let whole_big = p300.book_waiting_for_room(p300.card_capacities_mebibytes().unwrap()[0]).expect("空卡放得下整份");
        let p100 = open_today(directory.path(), 100, &cards, configured.clone());
        let kept_small = loop {
            let b = p100.book_waiting_for_room(4_000).expect("放得下");
            if b.card_position() == 1 { break b; }
            std::mem::forget(b);
        };
        let pending = 1_000_000u64;
        let (registered, after_return, outcome) = if arm == "today" {
            let p200 = open_today(directory.path(), 200, &cards, configured.clone());
            let registered = p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1]).unwrap();
            drop(whole_big);
            let capacities = p200.card_capacities_mebibytes().unwrap();
            let used: Vec<usize> = (0..2).filter(|i| registered[*i]).collect();
            let holds: Vec<u64> = used.iter().map(|i| states_the_capacity_holds(capacities[*i], pending as usize)).collect();
            (registered, capacities, scratch_line_today(p200.book_states_on_the_cards_waiting_only_when_none_has_room(&used, pending, &holds, &mebibytes_for_states)))
        } else {
            let p200 = open_fixed(directory.path(), 200, &cards, configured.clone(), false, true);
            let registered = p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1]).unwrap();
            drop(whole_big);
            let capacities = p200.card_capacities_mebibytes().unwrap();
            let used: Vec<usize> = (0..2).filter(|i| registered[*i]).collect();
            let holds: Vec<u64> = used.iter().map(|i| states_the_capacity_holds(capacities[*i], pending as usize)).collect();
            (registered, capacities, scratch_line_fixed(p200.book_states_on_the_cards_waiting_only_when_none_has_room(&used, pending, &holds, &mebibytes_for_states)))
        };
        println!(
            "MODEL scenario=s4 arm={arm} registered={registered:?} capacities_after_the_5090_was_returned={after_return:?} outcome_on_the_registered_cards={outcome}"
        );
        drop(kept_small);
    }
}

/// S4b（K2-2）：两张卡都登记上了；产品路径借草稿区那一刻 5090 上有一笔 1 MiB 的短借据（一条 GPU 用例），借完它马上还。
/// 今天：5090 那张照整份容量压、放不下、不装状态，全部状态排在 5060 Ti 上；这一趟的分派只做这一次
/// （crash_amplification.rs 第 ① 段每次调用借一次草稿区，之后按块领），5090 在这一趟里一个状态都不判。
#[test]
fn s4b_a_one_mebibyte_lease_at_the_booking_instant_keeps_the_first_card_idle_for_the_run() {
    for arm in ["today", "fix_b_compress_to_room"] {
        let directory = Directory::new(&format!("s4b-{arm}"));
        let big = FakeCard::new(RTX_5090_TOTAL);
        let small = FakeCard::new(RTX_5060_TI_TOTAL);
        let cards = [big.clone(), small.clone()];
        let configured = vec![None, None];
        let p100 = open_today(directory.path(), 100, &cards, configured.clone());
        let pending = 1_000_000u64;
        let outcome = if arm == "today" {
            let p200 = open_today(directory.path(), 200, &cards, configured.clone());
            let registered = p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1]).unwrap();
            let short = p100.book_waiting_for_room(1).expect("放得下");
            let capacities = p200.card_capacities_mebibytes().unwrap();
            let holds: Vec<u64> = capacities.iter().map(|c| states_the_capacity_holds(*c, pending as usize)).collect();
            let line = scratch_line_today(p200.book_states_on_the_cards_waiting_only_when_none_has_room(&[0, 1], pending, &holds, &mebibytes_for_states));
            drop(short);
            format!("registered={registered:?} holds={holds:?} {line}")
        } else {
            let p200 = open_fixed(directory.path(), 200, &cards, configured.clone(), false, true);
            let registered = p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1]).unwrap();
            let short = p100.book_waiting_for_room(1).expect("放得下");
            let capacities = p200.card_capacities_mebibytes().unwrap();
            let holds: Vec<u64> = capacities.iter().map(|c| states_the_capacity_holds(*c, pending as usize)).collect();
            let line = scratch_line_fixed(p200.book_states_on_the_cards_waiting_only_when_none_has_room(&[0, 1], pending, &holds, &mebibytes_for_states));
            drop(short);
            format!("registered={registered:?} holds={holds:?} {line}")
        };
        println!("MODEL scenario=s4b arm={arm} {outcome}");
    }
}

/// 每张卡装几个状态，照 crash_amplification.rs states_each_card_of_the_pool_lends：按登记时克隆的卡的额度
/// （开账那一刻读的空闲，states_in_flight_of_each_card）与按账此刻的容量（most_states_whose_need_fits_the_capacity）取小。
fn states_each_card_holds_like_the_product_path(configured: Option<u64>, free_at_open: u64, capacity: u64, most: usize) -> u64 {
    let budget = budget_of(configured, free_at_open);
    let by_budget = if budget < TABLES_MEBIBYTES + DRIVER { 0 } else { (budget - TABLES_MEBIBYTES - DRIVER) / MEBIBYTES_PER_STATE };
    by_budget.min(states_the_capacity_holds(capacity, most))
}

/// S6（K2-1 / K2-3）：同一台机器上两个进程各跑一趟大流的产品路径，一张 5090、没配额度。两个进程先后开池、各在卡上登记
/// （取卡登记只记驱动那一份、起设备），再各借草稿区。今天：另一个进程只是登记着（一笔借据都没有），这张卡照整份容量压就放不下。
#[test]
fn s6_two_product_path_processes_registered_on_the_same_card_block_each_other() {
    for arm in ["today", "fix_b_compress_to_room"] {
        let directory = Directory::new(&format!("s6-{arm}"));
        let card = FakeCard::new(RTX_5090_TOTAL);
        let pending = 1_000_000u64;
        let mut lines = Vec::new();
        if arm == "today" {
            let p100 = open_today(directory.path(), 100, &[card.clone()], vec![None]);
            let free_at_open_100 = card.free();
            p100.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0]).unwrap();
            card.allocate(100, DRIVER);
            let p200 = open_today(directory.path(), 200, &[card.clone()], vec![None]);
            let free_at_open_200 = card.free();
            p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0]).unwrap();
            card.allocate(200, DRIVER);
            for (pid, ledger, free_at_open) in [(100, &p100, free_at_open_100), (200, &p200, free_at_open_200)] {
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                let holds = states_each_card_holds_like_the_product_path(None, free_at_open, capacity, pending as usize);
                let line = scratch_line_today(ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], pending, &[holds], &mebibytes_for_states));
                lines.push(format!("p{pid}:free_at_open={free_at_open},capacity={capacity},holds={holds},{line}"));
            }
        } else {
            let p100 = open_fixed(directory.path(), 100, &[card.clone()], vec![None], false, true);
            let free_at_open_100 = card.free();
            p100.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0]).unwrap();
            card.allocate(100, DRIVER);
            let p200 = open_fixed(directory.path(), 200, &[card.clone()], vec![None], false, true);
            let free_at_open_200 = card.free();
            p200.register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0]).unwrap();
            card.allocate(200, DRIVER);
            for (pid, ledger, free_at_open) in [(100, &p100, free_at_open_100), (200, &p200, free_at_open_200)] {
                let capacity = ledger.card_capacities_mebibytes().unwrap()[0];
                let holds = states_each_card_holds_like_the_product_path(None, free_at_open, capacity, pending as usize);
                let line = scratch_line_fixed(ledger.book_states_on_the_cards_waiting_only_when_none_has_room(&[0], pending, &[holds], &mebibytes_for_states));
                lines.push(format!("p{pid}:free_at_open={free_at_open},capacity={capacity},holds={holds},{line}"));
            }
        }
        println!("MODEL scenario=s6 arm={arm} {}", lines.join(" "));
    }
}
