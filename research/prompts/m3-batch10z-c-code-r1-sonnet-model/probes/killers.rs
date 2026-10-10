// 正推腿（第 10z 批第三块代码轮 K3）：对存活的变异补的会红用例草稿（只用公开函数；真代码上绿，变异上红）。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, judge_blocks_on_one_card, BlockHandedToTheStore,
    BlockJudgedOnACard, BlocksForTheCards, BookingRefusal, CardLeftTheBlocks, CardMemoryReading,
    GpuMemoryLedger, OccupiedByThisProcess,
};
use std::sync::{Arc, Mutex};

const D: u64 = 2560;
const EMPTY: u64 = 32_110;

#[derive(Clone, Default)]
struct Phys {
    started: bool,
    xd: u64,
    outsider: u64,
    built: u64,   // 活着的借据建上的合计
    left: u64,    // 销了账、卡上还没还的
    dead: u64,    // 丢了的设备留在卡上的
}
impl Phys {
    fn x(&self) -> u64 {
        (if self.started { self.xd } else { 0 }) + self.built + self.left + self.dead
    }
    fn reading(&self) -> CardMemoryReading {
        CardMemoryReading {
            free_mebibytes: EMPTY - self.outsider - self.x(),
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: if self.x() > 0 {
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: self.x() }
            } else {
                OccupiedByThisProcess::NotListed
            },
        }
    }
}
fn rig() -> (Arc<Mutex<Phys>>, GpuMemoryLedger) {
    let phys = Arc::new(Mutex::new(Phys::default()));
    let shared = Arc::clone(&phys);
    let ledger = GpuMemoryLedger::new(
        1,
        Box::new(move || Ok(vec![shared.lock().unwrap().reading()])),
        Box::new(|_p, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, D)),
        D,
    );
    (phys, ledger)
}
fn room(ledger: &GpuMemoryLedger) -> u64 {
    ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0]
}

/// 杀 M01（读数不压）：长借据建满又还了、卡上还给驱动之后，新借一笔建满，能借的要把新借的建上的算进去。
#[test]
fn k_pressing_lets_a_later_lease_count_once_the_card_took_the_returned_one_back() {
    let (phys, ledger) = rig();
    let a = ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.built = 10_000;
    }
    ledger.record_the_device_started_on_the_card(0);
    drop(a);
    {
        let mut p = phys.lock().unwrap();
        p.built = 0;
        p.left = 10_000;
    }
    let _ = room(&ledger); // 读到：卡上还占着，压不掉
    phys.lock().unwrap().left = 0; // 驱动收回了
    let _ = room(&ledger); // 读到：这个进程只占着驱动那一份，销了账的压到它为止
    let _b = ledger.book_on_the_card(0, 5_000).unwrap();
    phys.lock().unwrap().built = 5_000;
    // 真空闲 24 550；甲：销了账的压到了 2 560，B 建上的 5 000 算进占着的；差驱动那一份（甲的代价）
    assert_eq!(room(&ledger), 24_550 - D - 0, "B 建满：能借的 21 990（不压的话只有 16 990）");
}

/// 杀 M05（重起被拒一次记成仍然起着）：被拒几次，卡上读数没变，能借的就不变。
#[test]
fn k_a_refused_restart_asked_again_does_not_take_another_driver_share_off_the_room() {
    let (phys, ledger) = rig();
    let a = ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.built = 10_000;
    }
    ledger.record_the_device_started_on_the_card(0);
    {
        let mut p = phys.lock().unwrap();
        p.started = false;
        p.xd = 0;
        p.dead = D;
        p.outsider = 19_000; // 新设备放不下：26 990 − 19 000 = 7 990 < 借着的 10 000
    }
    let refused = Err(BookingRefusal::NoCardHasRoomNow { need_mebibytes: D, card_rooms_mebibytes: vec![0] });
    assert_eq!(ledger.prepare_the_driver_share_before_starting_the_device(0), refused);
    phys.lock().unwrap().outsider = 0;
    let room_after_one = room(&ledger);
    for _ in 0..3 {
        let _ = ledger.prepare_the_driver_share_before_starting_the_device(0);
    }
    assert_eq!(room(&ledger), room_after_one, "再被问三次，读数没变，能借的不变");
    drop(a);
}

/// 杀 M13（被拒之后又借了一笔，驱动那一份被记回「登记了、没起」）：之后卡上被占，再来起设备仍要拒。
#[test]
fn k_a_lease_booked_after_a_refused_restart_keeps_the_restart_judged() {
    let (phys, ledger) = rig();
    let a = ledger.book_on_the_card(0, 10_000).unwrap();
    {
        let mut p = phys.lock().unwrap();
        p.started = true;
        p.xd = D;
        p.built = 10_000;
    }
    ledger.record_the_device_started_on_the_card(0);
    {
        let mut p = phys.lock().unwrap();
        p.started = false;
        p.xd = 0;
        p.dead = D;
        p.outsider = 19_000;
    }
    assert!(ledger.prepare_the_driver_share_before_starting_the_device(0).is_err());
    phys.lock().unwrap().outsider = 0;
    let _c = ledger.book_on_the_card(0, 5_000).expect("别人走了，借得到");
    phys.lock().unwrap().outsider = 19_000; // 又被占
    assert!(
        ledger.prepare_the_driver_share_before_starting_the_device(0).is_err(),
        "被拒之后借了一笔，再来起设备还是照此刻判：放不下"
    );
    drop(a);
}

/// 杀 M10（差 1 MiB 也放行）：借着的 10 000，新设备的驱动那一份放得下的边界两侧。
#[test]
fn k_the_restart_is_refused_when_one_mebibyte_short_and_allowed_when_exactly_enough() {
    for (outsider, expect_ok) in [(16_990u64, true), (16_991, false)] {
        let (phys, ledger) = rig();
        let _a = ledger.book_on_the_card(0, 10_000).unwrap();
        {
            let mut p = phys.lock().unwrap();
            p.started = true;
            p.xd = D;
            p.built = 10_000;
        }
        ledger.record_the_device_started_on_the_card(0);
        {
            let mut p = phys.lock().unwrap();
            p.started = false;
            p.xd = 0;
            p.dead = D;
            p.outsider = outsider;
        }
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0).is_ok(),
            expect_ok,
            "outsider={outsider}"
        );
    }
}

/// 杀 M18（卡线程那一圈不问叫停）、M30（判不了接着领）、M31（写库那头走了接着领）。
#[test]
fn k_the_card_loop_stops_on_request_on_refusal_and_when_the_store_is_gone() {
    let judged = Mutex::new(Vec::new());
    let blocks = BlocksForTheCards::new(0..5);
    let left = judge_blocks_on_one_card(
        &blocks,
        &|| true,
        &mut |i| {
            judged.lock().unwrap().push(i);
            BlockJudgedOnACard::Judged(i)
        },
        &mut |_, _| BlockHandedToTheStore::Handed,
    );
    assert_eq!(left, CardLeftTheBlocks::NothingLeftOrStopped);
    assert!(judged.lock().unwrap().is_empty(), "叫停了一块都不领");

    let blocks = BlocksForTheCards::new(0..5);
    let mut taken = Vec::new();
    let left = judge_blocks_on_one_card(
        &blocks,
        &|| false,
        &mut |i| {
            taken.push(i);
            if i == 1 {
                BlockJudgedOnACard::<usize>::Refused { reason: "判不了".into() }
            } else {
                BlockJudgedOnACard::Judged(i)
            }
        },
        &mut |_, _| BlockHandedToTheStore::Handed,
    );
    assert_eq!(left, CardLeftTheBlocks::Refused { reason: "判不了".into() });
    assert_eq!(taken, vec![0, 1], "判不了的那一块不交回、这张卡不再领");

    let blocks = BlocksForTheCards::new(0..5);
    let mut handed = 0;
    let left = judge_blocks_on_one_card(
        &blocks,
        &|| false,
        &mut |i| BlockJudgedOnACard::Judged(i),
        &mut |_, _| {
            handed += 1;
            BlockHandedToTheStore::TheStoreIsGone
        },
    );
    assert_eq!(left, CardLeftTheBlocks::TheStoreIsGone);
    assert_eq!(handed, 1, "写库那头走了：不再领");
}
