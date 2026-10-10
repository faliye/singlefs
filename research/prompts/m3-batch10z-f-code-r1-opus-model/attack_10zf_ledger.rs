//! 第 10z 批第六块代码轮攻方（opus）模型：显存账 G2 乙的「等失败那一支」（K1-2）与「丢了的设备两种状态」（K1-3）。
//! 入库的 `GpuMemoryLedger` 原样调用；读数由下面这张「真卡」给：`occupied` 是这个进程此刻真占在卡上的（MiB），
//! `growth` 是之后还要长上去的（活着的借据还没建的、设备驱动那一份还要长到 D 的）。真能再借的 = 空卡 − occupied − growth。
//! 放进 crates/singlefs-harness/tests/ 跑：`cargo test -p singlefs-harness --test attack_10zf_ledger -- --nocapture --test-threads 1`。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, DeviceWaitBeforeReturningTheLease,
    GpuMemoryBooking, GpuMemoryLedger, OccupiedByThisProcess,
};
use std::sync::{Arc, Mutex};

const EMPTY: u64 = 32_110; // 5090 空卡上能用的（文件头）
const QUOTA: u64 = 32_000; // 配给卡的额度（同入库用例）
const D: u64 = 2_560; // MEBIBYTES_LEFT_FOR_THE_DRIVER
const CTX: u64 = 600; // 驱动实占（同入库用例）
const FINISHED: DeviceWaitBeforeReturningTheLease =
    DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished;
const FAILED: DeviceWaitBeforeReturningTheLease = DeviceWaitBeforeReturningTheLease::TheWaitFailed;

#[derive(Default)]
struct Truth {
    occupied: u64,
    growth: u64,
}

type Card = Arc<Mutex<Truth>>;

fn ledger_on(card: &Card) -> GpuMemoryLedger {
    let shared = Arc::clone(card);
    GpuMemoryLedger::new(
        1,
        Box::new(move || {
            let truth = shared.lock().unwrap();
            Ok(vec![CardMemoryReading {
                free_mebibytes: EMPTY - truth.occupied,
                total_less_reserved_mebibytes: EMPTY,
                occupied_by_this_process: if truth.occupied == 0 {
                    OccupiedByThisProcess::NotListed
                } else {
                    OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: truth.occupied }
                },
            }])
        }),
        Box::new(|_position, may_hold| {
            card_ledger_capacity_mebibytes(None, Some(QUOTA), may_hold.min(QUOTA), may_hold, D)
        }),
        D,
    )
}

fn change(card: &Card, occupied: i64, growth: i64) {
    let mut truth = card.lock().unwrap();
    truth.occupied = (truth.occupied as i64 + occupied) as u64;
    truth.growth = (truth.growth as i64 + growth) as u64;
}

/// 打一行：账此刻给的、真能再借的（真能再借的不超过额度那一项：QUOTA − D − 活着的借据合计由调用方传进来）。
fn report(arm: &str, step: &str, ledger: &GpuMemoryLedger, card: &Card, live_leases: u64) -> (u64, u64) {
    let ledger_room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
    let truth = card.lock().unwrap();
    let truth_room = (EMPTY - truth.occupied)
        .saturating_sub(truth.growth)
        .min((QUOTA - D).saturating_sub(live_leases));
    println!(
        "ATTACK arm={arm} step={step} ledger_room={ledger_room} truth_room={truth_room} under={} over={}",
        truth_room.saturating_sub(ledger_room),
        ledger_room.saturating_sub(truth_room)
    );
    (ledger_room, truth_room)
}

/// 借一笔、在这张卡上起好设备（头一回起：驱动长到 CTX）、建满。
fn book_and_build<'l>(ledger: &'l GpuMemoryLedger, card: &Card, mebibytes: u64) -> GpuMemoryBooking<'l> {
    let booking = ledger.book_on_the_card(0, mebibytes).expect("借得到");
    change(card, 0, mebibytes as i64);
    change(card, mebibytes as i64, -(mebibytes as i64));
    booking
}

/// 一笔借据正常用完：缓冲先丢、等完设备（缓冲交回驱动）、销账。
fn return_after_a_finished_wait(mut booking: GpuMemoryBooking<'_>, card: &Card) {
    let mebibytes = booking.mebibytes();
    change(card, -(mebibytes as i64), 0);
    booking.take_in_the_wait_before_returning(FINISHED);
    drop(booking);
}

fn start_the_first_device(ledger: &GpuMemoryLedger, card: &Card) {
    ledger.prepare_the_driver_share_before_starting_the_device(0).unwrap();
    ledger.record_the_device_started_on_the_card(0);
    // 借的那一笔里已经记了驱动那一份要长到 D：起好之后实占 CTX，还要长 D − CTX
    change(card, CTX as i64, -(CTX as i64));
}

/// K1-2：等超时的那一笔照旧记在账上，卡上它的缓冲之后还了，账到设备丢了之前一直少给。
#[test]
fn k1_2_a_lease_whose_wait_timed_out_stays_booked_after_the_card_gave_its_memory_back() {
    let arm = "timeout-kept";
    let card: Card = Arc::new(Mutex::new(Truth::default()));
    let ledger = ledger_on(&card);
    let mut kept = ledger.book_on_the_card(0, 20_000).unwrap();
    change(&card, 0, 20_000 + D as i64);
    start_the_first_device(&ledger, &card);
    change(&card, 20_000, -20_000);
    report(arm, "k_built", &ledger, &card, 20_000);
    // 这一笔丢掉时设备上还排着比 30 秒长的活：Wait 超时
    kept.take_in_the_wait_before_returning(FAILED);
    drop(kept);
    report(arm, "k_wait_timed_out_buffers_still_in_flight", &ledger, &card, 0);
    // 活做完、下一次有人 poll：缓冲交回驱动
    change(&card, -20_000, 0);
    report(arm, "k_work_done_memory_back", &ledger, &card, 0);
    let wanted = 15_000;
    println!(
        "ATTACK arm={arm} step=book_{wanted} result={}",
        match ledger.book_on_the_card(0, wanted) {
            Ok(_) => "booked".to_string(),
            Err(refusal) => format!("refused:{refusal:?}").replace(' ', ""),
        }
    );
    for _ in 0..50 {
        let short = book_and_build(&ledger, &card, 1_000);
        return_after_a_finished_wait(short, &card);
    }
    report(arm, "after_50_leases_waited_and_returned_normally", &ledger, &card, 0);
    // 最坏：每次借满此刻能借的、都等超时
    let mut timeouts = 1;
    loop {
        let room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        if room == 0 {
            break;
        }
        let mut lease = book_and_build(&ledger, &card, room);
        change(&card, -(room as i64), 0);
        lease.take_in_the_wait_before_returning(FAILED);
        drop(lease);
        timeouts += 1;
    }
    let (_, truth_room) = report(arm, &format!("after_{timeouts}_timeouts"), &ledger, &card, 0);
    let one = ledger.book_on_the_card(0, 1).map(|_| ()).map_err(|refusal| format!("{refusal:?}").replace(' ', ""));
    println!("ATTACK arm={arm} step=book_1_after_{timeouts}_timeouts result={one:?} truth_room={truth_room}");
    // 出路只有一条：这张卡的设备丢了、要重起（真卡：旧设备没了，新设备驱动那一份还没长）
    let _ = ledger.prepare_the_driver_share_before_starting_the_device(0);
    change(&card, -(CTX as i64), CTX as i64);
    report(arm, "after_a_device_loss_and_restart_judged", &ledger, &card, 0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// 另一条用例借的 L 起卡时见设备丢了：照重起记、起新设备（L 拿新设备、建满 5 000）。
    Restart,
    /// 旧设备上的借据 A 丢掉（等的是旧设备）。
    DropA,
    /// L 用完、正常还。
    ReturnL,
}

/// K1-3：旧设备 gen1 上借着 A（10 000，建满）时设备丢了；之后的步骤照 `order`，A 的等照 `a_wait`。
/// 真卡：旧设备连 A 的缓冲在旧设备的最后一个句柄丢掉时还给驱动（池里那一格换成新设备、或 A 的卡丢掉，后发生的那一下）。
/// 交（最后一步之后账少给的、借一笔 10 000 时账给不给、真放不放得下）。
fn k1_3_history(order: &[Step], a_wait: DeviceWaitBeforeReturningTheLease) -> (u64, u64, bool, bool) {
    let label = format!("{order:?}-{}", if a_wait == FAILED { "a_wait_failed" } else { "a_wait_finished" })
        .replace([' ', '[', ']'], "")
        .replace(',', ">");
    let arm = format!("lost-device:{label}");
    let card: Card = Arc::new(Mutex::new(Truth::default()));
    let ledger = ledger_on(&card);
    let lease_a = ledger.book_on_the_card(0, 10_000).unwrap();
    change(&card, 0, 10_000 + D as i64);
    start_the_first_device(&ledger, &card);
    change(&card, 10_000, -10_000);
    // gen1 丢了（驱动重置一类：卡上的东西要等旧设备没了才还）
    let mut lease_l = Some(ledger.book_on_the_card(0, 5_000).unwrap());
    change(&card, 0, 5_000);
    let mut lease_a = Some(lease_a);
    let mut slot_holds_the_old_device = true;
    let mut a_holds_the_old_device = true;
    let mut old_device_freed = false;
    let mut over_max = 0;
    for step in order {
        match step {
            Step::Restart => {
                ledger.prepare_the_driver_share_before_starting_the_device(0).expect("重起放得下");
                ledger.record_the_device_started_on_the_card(0);
                change(&card, CTX as i64, 0);
                change(&card, 5_000, -5_000);
                slot_holds_the_old_device = false;
            }
            Step::DropA => {
                let mut booking = lease_a.take().expect("A 只丢一次");
                booking.take_in_the_wait_before_returning(a_wait);
                // A 的卡（旧设备的一个句柄）先丢、借据后丢（GpuCardLease 的字段次序）
                a_holds_the_old_device = false;
                if !slot_holds_the_old_device && !old_device_freed {
                    change(&card, -((CTX + 10_000) as i64), 0);
                    old_device_freed = true;
                }
                drop(booking);
            }
            Step::ReturnL => {
                let booking = lease_l.take().expect("L 只还一次");
                return_after_a_finished_wait(booking, &card);
            }
        }
        if !slot_holds_the_old_device && !a_holds_the_old_device && !old_device_freed {
            change(&card, -((CTX + 10_000) as i64), 0);
            old_device_freed = true;
        }
        let live = 10_000 * u64::from(lease_a.is_some()) + 5_000 * u64::from(lease_l.is_some());
        let (ledger_room, truth_room) = report(&arm, &format!("{step:?}"), &ledger, &card, live);
        over_max = over_max.max(ledger_room.saturating_sub(truth_room));
    }
    assert!(lease_a.is_none() && lease_l.is_none() && old_device_freed);
    // 之后一条长借据 Q 15 000 建满，再来一条要 10 000 的用例
    let long = book_and_build(&ledger, &card, 15_000);
    let (ledger_room, truth_room) = report(&arm, "q_15000_built", &ledger, &card, 15_000);
    let booked = ledger.book_on_the_card(0, 10_000).is_ok();
    let mut refused_short = 0;
    for _ in 0..100 {
        if ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0] < 1_000 {
            refused_short += 1;
            continue;
        }
        let short = book_and_build(&ledger, &card, 1_000);
        return_after_a_finished_wait(short, &card);
    }
    println!("ATTACK arm={arm} step=100_leases_of_1000 refused={refused_short}");
    let (after_cycles, _) = report(&arm, "after_100_normal_leases", &ledger, &card, 15_000);
    println!(
        "ATTACK arm={arm} summary under_after_q={} book_10000_after_q={} truth_fits_10000={} over_max={over_max} still_short_after_100={}",
        truth_room.saturating_sub(ledger_room),
        if booked { "booked" } else { "refused" },
        truth_room >= 10_000,
        after_cycles < truth_room
    );
    drop(long);
    (truth_room.saturating_sub(ledger_room), over_max, booked, truth_room >= 10_000)
}

/// 把用户（别的用例）决定的先后放开扫：重起、A 丢掉、L 还的三种次序（L 要先起卡才建得了），A 的等成或不成。
#[test]
fn k1_3_a_lease_of_the_lost_device_whose_wait_fails_after_the_restart_keeps_the_card_short() {
    let orders: [&[Step]; 3] = [
        &[Step::Restart, Step::DropA, Step::ReturnL],
        &[Step::Restart, Step::ReturnL, Step::DropA],
        &[Step::DropA, Step::Restart, Step::ReturnL],
    ];
    let mut stuck = 0;
    for order in orders {
        for a_wait in [FINISHED, FAILED] {
            let (under, over, booked, fits) = k1_3_history(order, a_wait);
            if !booked && fits {
                stuck += 1;
            }
            println!("ATTACK table order={order:?} a_wait={a_wait:?} under={under} over_max={over} book_10000={booked} truth_fits={fits}");
        }
    }
    println!("ATTACK k1_3 histories_refusing_a_lease_that_fits={stuck} of 6");
}
