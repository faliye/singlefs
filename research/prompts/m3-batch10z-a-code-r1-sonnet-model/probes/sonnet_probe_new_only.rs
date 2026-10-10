// 只在改后的树上跑：重起（prepare_the_driver_share_before_starting_the_device）这个接口改前没有。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, GpuMemoryLedger, OccupiedByThisProcess,
};
use std::sync::{Arc, Mutex};

const D: u64 = 2560;
const EMPTY: u64 = 32_110;

fn reading(x: Option<u64>) -> CardMemoryReading {
    let x_value = x.unwrap_or(0);
    CardMemoryReading {
        free_mebibytes: EMPTY - x_value,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: x.map_or(OccupiedByThisProcess::NotListed, |mebibytes| {
            OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }
        }),
    }
}

fn ledger(shared: Arc<Mutex<CardMemoryReading>>) -> GpuMemoryLedger {
    GpuMemoryLedger::new(
        1,
        Box::new(move || Ok(vec![*shared.lock().unwrap()])),
        Box::new(|_p, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, D)),
        D,
    )
}

/// 设备丢了重起：旧设备那一份在新设备起来之前就还了 / 还在卡上到新设备起来才还。
#[test]
fn s4_lost_device_restart_variants() {
    for old_device_still_on_the_card_when_prepared in [false, true] {
        let r = Arc::new(Mutex::new(reading(None)));
        let ledger = ledger(Arc::clone(&r));
        let l1 = ledger.book_on_the_card(0, 10_000).unwrap();
        assert_eq!(ledger.prepare_the_driver_share_before_starting_the_device(0), Ok(()));
        ledger.record_the_device_started_on_the_card(0);
        *r.lock().unwrap() = reading(Some(D + 10_000));
        let before = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        // 设备丢了
        *r.lock().unwrap() = if old_device_still_on_the_card_when_prepared {
            reading(Some(D + 10_000))
        } else {
            reading(None)
        };
        let prepared = ledger.prepare_the_driver_share_before_starting_the_device(0);
        // 新设备起来，借据重新建上 10000，旧设备的那一份已经还了
        ledger.record_the_device_started_on_the_card(0);
        *r.lock().unwrap() = reading(Some(D + 10_000));
        let after_1 = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        let after_2 = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
        let truth = EMPTY - D - 10_000;
        println!(
            "S4 old_device_still_on_the_card_when_prepared={old_device_still_on_the_card_when_prepared}: before_loss {before} prepared {prepared:?} after_restart reads {after_1} {after_2} truth {truth} (under {})",
            i128::from(truth) - i128::from(after_2)
        );
        drop(l1);
    }
}

/// 丢了的设备重起被拒之后，下一次再来重起：放得下的判断还在不在。
#[test]
fn s6_a_refused_restart_is_asked_again() {
    let r = Arc::new(Mutex::new(reading(None)));
    let ledger = ledger(Arc::clone(&r));
    let _l1 = ledger.book_on_the_card(0, 10_000).unwrap();
    assert_eq!(ledger.prepare_the_driver_share_before_starting_the_device(0), Ok(()));
    ledger.record_the_device_started_on_the_card(0);
    *r.lock().unwrap() = reading(Some(D + 10_000));
    let room = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
    let _l2 = ledger.book_on_the_card(0, room).unwrap(); // 把剩下的全借走，还没建
    let first = ledger.prepare_the_driver_share_before_starting_the_device(0);
    let second = ledger.prepare_the_driver_share_before_starting_the_device(0);
    let third = ledger.prepare_the_driver_share_before_starting_the_device(0);
    let room_after = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0];
    println!("S6: room before second lease {room}; restart attempts: first {first:?}; second {second:?}; third {third:?}; room now {room_after}");
}

/// S9：设备丢了重起之后（旧设备的已还、新设备起来、借据重新建上 10 000），真空闲 19 550 放得下的 17 000 借不借得到。
#[test]
fn s9_a_need_that_truly_fits_after_the_restart() {
    let r = Arc::new(Mutex::new(reading(None)));
    let ledger = ledger(Arc::clone(&r));
    let _l1 = ledger.book_on_the_card(0, 10_000).unwrap();
    assert_eq!(ledger.prepare_the_driver_share_before_starting_the_device(0), Ok(()));
    ledger.record_the_device_started_on_the_card(0);
    *r.lock().unwrap() = reading(Some(D + 10_000));
    let _ = ledger.card_rooms_mebibytes_now(&[0]).unwrap();
    *r.lock().unwrap() = reading(None);
    assert_eq!(ledger.prepare_the_driver_share_before_starting_the_device(0), Ok(()));
    ledger.record_the_device_started_on_the_card(0);
    *r.lock().unwrap() = reading(Some(D + 10_000));
    let result = ledger.book_on_the_card(0, 17_000);
    println!("S9: truth 19550; book 17000 -> {}", match &result { Ok(_) => "Ok".to_string(), Err(e) => format!("{e:?}") });
}
