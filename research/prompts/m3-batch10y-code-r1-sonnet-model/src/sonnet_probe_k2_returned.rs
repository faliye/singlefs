//! 正推腿探针（草稿副本里的，不进仓）：销了账、卡上其实已经还了的借据，在有一笔长借据建着时会不会让别的借据少借（K2-2）。
//! 假的卡：free 与「本进程占的」按下面每一步由用例设；容量照卡池真用的 `card_ledger_capacity_mebibytes`（配置额度大于卡，取 M − 驱动）。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, BookingRefusal, CardMemoryReading, CardMemoryReader, GpuMemoryLedger,
    OccupiedByThisProcess,
};
use std::sync::{Arc, Mutex};

const DRIVER: u64 = 2560;
const EMPTY: u64 = 32_110; // 5090 空卡：总量 32607 − 保留 497

fn ledger(reading: Arc<Mutex<CardMemoryReading>>) -> GpuMemoryLedger {
    let reader: CardMemoryReader = Box::new(move || Ok(vec![*reading.lock().unwrap()]));
    GpuMemoryLedger::new(
        1,
        reader,
        Box::new(|_position, may_hold| {
            card_ledger_capacity_mebibytes(None, Some(40_000), may_hold.min(40_000), may_hold, DRIVER)
        }),
        DRIVER,
    )
}

fn set(reading: &Arc<Mutex<CardMemoryReading>>, free: u64, occupied: u64) {
    *reading.lock().unwrap() = CardMemoryReading {
        free_mebibytes: free,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: occupied },
    };
}

#[test]
fn a_returned_lease_whose_memory_is_already_free_still_costs_the_card_while_another_lease_is_alive() {
    let reading = Arc::new(Mutex::new(CardMemoryReading {
        free_mebibytes: EMPTY,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::NotListed,
    }));
    let ledger = ledger(Arc::clone(&reading));
    // 一笔长借据 A 借 10000，设备起好，A 建满：本进程占 D + 10000，空闲 EMPTY − D − 10000
    let long_lease = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
    ledger.record_the_device_started_on_the_card(0);
    let alive_free = EMPTY - DRIVER - 10_000;
    set(&reading, alive_free, DRIVER + 10_000);
    println!("A 建满之后此刻能借的 = {:?}（真空闲 {alive_free}）", ledger.card_rooms_mebibytes_now(&[0]));
    // 十笔短借据各 1000：借、建、丢，丢了之后卡上马上还了（占的回到 D + 10000）
    for round in 1..=10 {
        let short = ledger.book_on_the_card(0, 1_000).expect("借得到");
        set(&reading, alive_free - 1_000, DRIVER + 10_000 + 1_000);
        drop(short);
        set(&reading, alive_free, DRIVER + 10_000);
        let room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        println!("第 {round} 笔短借据还了、卡上已经还给驱动：此刻能借的 = {room}（真空闲 {alive_free}，少借 {})", alive_free - room);
    }
    let room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
    let need = alive_free - 1_000;
    println!("要 {need} MiB（真空闲 {alive_free}，放得下）：");
    let outcome = ledger.book_on_the_card(0, need);
    match &outcome {
        Ok(_) => println!("  借到了"),
        Err(BookingRefusal::NoCardHasRoomNow { need_mebibytes, card_rooms_mebibytes }) => {
            println!("  借不到（NoCardHasRoomNow，调用方报本次未跑）：要 {need_mebibytes}，此刻能借 {card_rooms_mebibytes:?}")
        }
        Err(other) => println!("  {other:?}"),
    }
    drop(long_lease);
    assert!(room < alive_free, "探针：少借");
}
