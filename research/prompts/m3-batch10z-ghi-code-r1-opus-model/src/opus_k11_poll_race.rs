//! m3-batch10z-ghi-code-r1 攻方原型 K1-1：借之前 poll 与销账之间不在一次锁里，poll 之后才记成没等成的那一笔被一起销。
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

/// 卡池 `take_in_the_devices_found_idle` 那一串：B 线程先列出有没等成借据的卡、poll 交回队列空了（X 的活做完、缓冲交回驱动），
/// 这之后、B 调 `take_in_that_the_submitted_work_finished` 之前，A 线程借一笔、建好缓冲、提交了活、丢借据时没等成（照旧记着）；
/// B 接着销账：A 那一笔一起销了，它的缓冲还在卡上（活在 poll 之后才提交）。
#[test]
fn k1_1_a_lease_kept_after_the_poll_is_returned_with_the_ones_kept_before_it() {
    for arm in ["take-in-between", "take-in-before-a"] {
        let card: Card = Arc::new(Mutex::new(Truth::default()));
        let ledger = ledger_on(&card);
        let mut x = ledger.book_on_the_card(0, 10_000).unwrap();
        change(&card, 0, 10_000 + D as i64);
        ledger.prepare_the_driver_share_before_starting_the_device(0).unwrap();
        let device = ledger.record_the_device_started_on_the_card(0);
        change(&card, CTX as i64, -(CTX as i64));
        change(&card, 10_000, -10_000);
        x.take_in_the_wait_before_returning(&device, FAILED);
        drop(x);
        report(arm, "x_kept_buffers_in_flight", &ledger, &card, 0);
        // B：列卡、poll 交回队列空了——X 的活做完，缓冲交回驱动
        assert_eq!(ledger.cards_with_leases_kept_after_a_failed_wait(), vec![0]);
        change(&card, -10_000, 0);
        if arm == "take-in-before-a" {
            device.take_in_that_the_submitted_work_finished();
        }
        // A：借一笔、建好、提交了活、丢借据时没等成
        let mut a = ledger.book_on_the_card(0, 8_000).unwrap();
        change(&card, 0, 8_000);
        change(&card, 8_000, -8_000);
        a.take_in_the_wait_before_returning(&device, FAILED);
        drop(a);
        report(arm, "a_kept_buffers_in_flight", &ledger, &card, 0);
        if arm == "take-in-between" {
            device.take_in_that_the_submitted_work_finished();
        }
        report(arm, "after_b_takes_in", &ledger, &card, 0);
        println!("ATTACK arm={arm} a_on_the_books={}", ledger.is_the_lease_on_the_books(0, 1));
    }
}
