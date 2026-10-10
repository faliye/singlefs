//! 攻方腿 m3-batch10x-code-r1 的模型（只在攻方自己的副本里跑，不入库装置）：只用 `singlefs_harness::gpu_memory_ledger`
//! 的公开函数与假的显存读数，不上卡、不起 wgpu。每一格打一张表（`ROW ...` 行），断言只钉「这一形出现了」。
//! 卡池的容量闭包照冻结副本 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1516-1525 行，
//! 其中 `memory_budget_mebibytes_of` 照同一份文件第 304-315 行抄成 [`budget`]（checker 档的函数，harness 档调不到）。
//! 读数里的两个本机数是 2026-10-07 在本机读的（报告「量的数」一节）：空着的 5090 空闲 32110 MiB、总量 32607 MiB；
//! 起了设备、借据都还了的 GPU 测试进程 `nvidia-smi --query-compute-apps` 报它占 2687 MiB。

use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, decide_after_attempting_every_card, BookingRefusal,
    CardMemoryReading, GpuMemoryLedger, ScratchAttemptOnACard, ScratchPlacementDecision,
};
use std::sync::{Arc, Mutex};

/// 驱动那一份（冻结副本 `crash_judge_dispatch.rs` 的 `MEBIBYTES_LEFT_FOR_THE_DRIVER`，账里的常数）。
const D: u64 = 2560;
const TOTAL_5090: u64 = 32_607;
/// 本机空着的 5090 此刻的空闲（量的，不是总量：总量与空闲差约 495 MiB 是驱动留着的）。
const FREE_EMPTY_5090: u64 = 32_110;
/// 起了设备、编过内核的 GPU 测试进程 `nvidia-smi` 报它占的（量的）。
const BASELINE: u64 = 2_687;

/// 抄 `gpu_unit_checks.rs` 第 304-315 行 `memory_budget_mebibytes_of`。
fn budget(configured: Option<u64>, may_hold: u64) -> u64 {
    match configured {
        Some(quota) => quota.min(may_hold),
        None => may_hold / 4 * 3,
    }
}

/// 一张卡的账，读数由测试随时换（`readings`），容量闭包照卡池（`gpu_unit_checks.rs` 第 1516-1525 行）。
fn pool_ledger(
    readings: &Arc<Mutex<Vec<CardMemoryReading>>>,
    environment_quota: Option<u64>,
    configured: Option<u64>,
) -> GpuMemoryLedger {
    let shared = Arc::clone(readings);
    let card_count = readings.lock().unwrap().len();
    GpuMemoryLedger::new(
        card_count,
        Box::new(move || Ok(shared.lock().unwrap().clone())),
        Box::new(move |_, may_hold| {
            card_ledger_capacity_mebibytes(
                environment_quota,
                configured,
                budget(configured, may_hold),
                may_hold,
                D,
            )
        }),
        D,
    )
}

fn reading(free: u64, occupied: Option<u64>) -> CardMemoryReading {
    CardMemoryReading {
        free_mebibytes: free,
        total_mebibytes: TOTAL_5090,
        occupied_by_this_process_mebibytes: occupied,
    }
}

fn set(readings: &Arc<Mutex<Vec<CardMemoryReading>>>, now: Vec<CardMemoryReading>) {
    *readings.lock().unwrap() = now;
}

fn room_now(ledger: &GpuMemoryLedger) -> u64 {
    ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0]
}

/// 四种配置：清场后那一份（`local:0:32G`）、日常那一份（`local:0:15G`）、没配优先级（3/4）、没配优先级而设了额度变量 30000。
const CONFIGURATIONS: [(&str, Option<u64>, Option<u64>); 4] = [
    ("quiesced-32G", None, Some(32 * 1024)),
    ("daily-15G", None, Some(15 * 1024)),
    ("unconfigured-3/4", None, None),
    ("unconfigured+env30000", Some(30_000), None),
];

/// K2-1 甲形：`memory_of_the_cards_on_these_buses`（`gpu_unit_checks.rs` 第 1482-1483 行）先起一次 `nvidia-smi` 读空闲、
/// 再起一次读这个进程占着的，两次之间这个进程另一条线程把它已经借到的那一笔建上卡。空闲是建之前的、占着的是建之后的，
/// 这一笔被算了两遍：空闲里还当它空着，占着的里又把它算成「借了已经占上的」不再扣。
/// 取样：别的进程占着 0 到 20 GiB（每 4 GiB 一档）× 先借的那一笔 1、4、8、12 GiB × 四种配置。
/// 每一行报三种读法下第二笔能借的：建之前一致的、建之后一致的、混着的；`over` 是混着的减卡上真空着的（正数就是借出去的比卡上有的多）。
#[test]
fn k2_a_free_read_before_and_occupied_read_after_this_process_allocated_counts_the_lease_twice() {
    let mut worst_over: i64 = i64::MIN;
    let mut rows_over = 0usize;
    let mut rows = 0usize;
    for (name, environment_quota, configured) in CONFIGURATIONS {
        for other_gibibytes in [0u64, 4, 8, 12, 16, 20] {
            for lease_gibibytes in [1u64, 4, 8, 12] {
                let other = other_gibibytes * 1024;
                let lease = lease_gibibytes * 1024;
                let readings = Arc::new(Mutex::new(vec![reading(FREE_EMPTY_5090 - other, None)]));
                let ledger = pool_ledger(&readings, environment_quota, configured);
                let Ok(first) = ledger.book_on_the_first_card_with_room(lease) else {
                    continue;
                };
                // 设备起好：驱动与内核占上卡（量的 2687），这一笔还没建
                ledger.record_the_device_started_on_the_card(0);
                let free_before = FREE_EMPTY_5090 - other - BASELINE;
                let free_after = free_before - lease;
                set(&readings, vec![reading(free_before, Some(BASELINE))]);
                let consistent_before = room_now(&ledger);
                set(&readings, vec![reading(free_after, Some(BASELINE + lease))]);
                let consistent_after = room_now(&ledger);
                set(&readings, vec![reading(free_before, Some(BASELINE + lease))]);
                let mixed = room_now(&ledger);
                let over = i64::try_from(mixed).unwrap() - i64::try_from(free_after).unwrap();
                println!(
                    "ROW k2a config={name} other={other} lease={lease} free_after={free_after} room_consistent_before={consistent_before} room_consistent_after={consistent_after} room_mixed={mixed} over={over}"
                );
                rows += 1;
                if over > 0 {
                    rows_over += 1;
                }
                worst_over = worst_over.max(over);
                drop(first);
            }
        }
    }
    println!("SUMMARY k2a rows={rows} rows_over_physical_free={rows_over} worst_over={worst_over}");
    assert!(worst_over > 1024, "混着读的那一形借出去的比卡上真空着的多出超过 1 GiB");
}

/// K2-1 乙形：这个进程在卡上占着一份不在账上的显存 `unbooked`（同一个测试二进制里没走卡池、自己起设备的产品路径那一趟，
/// `crash_amplification.rs` 文件头第 5 条「同一个测试二进制里有取卡池的 GPU 用例、而经 run_from_environment 的用例跑在它们起池之前的」；
/// 或还了的借据留在卡上还没放的缓冲），卡池先借了一笔 `first`、设备还没起（登记了、驱动那一份许出去了、一个字节都没建）。
/// `memory_this_process_may_hold_mebibytes` 把实际占着的里「不超过许出去的」那一部分算成这一笔已经占上，于是第二笔能借的里
/// 没扣 `first` 与驱动那一份。`over` 是第二笔能借的减「第一笔连驱动建上之后卡上真空着的」。
/// 取样：不在账上的 4、8、16、20 GiB × 第一笔 1、2、4 GiB × 四种配置，别的进程不占。
#[test]
fn k2_b_memory_this_process_holds_off_the_books_is_taken_as_the_unbuilt_lease_already_on_the_card() {
    let mut worst_over: i64 = i64::MIN;
    let mut rows = 0usize;
    for (name, environment_quota, configured) in CONFIGURATIONS {
        for unbooked_gibibytes in [4u64, 8, 16, 20] {
            for first_gibibytes in [1u64, 2, 4] {
                let unbooked = unbooked_gibibytes * 1024 + BASELINE;
                let first_need = first_gibibytes * 1024;
                let free = FREE_EMPTY_5090 - unbooked;
                let readings = Arc::new(Mutex::new(vec![reading(free, Some(unbooked))]));
                let ledger = pool_ledger(&readings, environment_quota, configured);
                let Ok(first) = ledger.book_on_the_first_card_with_room(first_need) else {
                    println!("ROW k2b config={name} unbooked={unbooked} first={first_need} first_refused=true");
                    continue;
                };
                let second_room = room_now(&ledger);
                let free_once_the_first_is_built = i64::try_from(free).unwrap()
                    - i64::try_from(first_need + BASELINE).unwrap();
                let over = i64::try_from(second_room).unwrap() - free_once_the_first_is_built;
                println!(
                    "ROW k2b config={name} unbooked={unbooked} first={first_need} free={free} second_room={second_room} free_once_the_first_is_built={free_once_the_first_is_built} over={over}"
                );
                rows += 1;
                worst_over = worst_over.max(over);
                drop(first);
            }
        }
    }
    println!("SUMMARY k2b rows={rows} worst_over={worst_over}");
    assert!(worst_over > 1024, "不在账上的那一份被当成第一笔已经占上");
}

/// K2-2 白红甲形：每张卡此刻都放不下一个状态（这个进程别的用例或卡上别的进程占着），空着时放得下。
/// 产品路径先照此刻能借的算每张卡带得了几个（`crash_amplification.rs` 第 2224-2259 行），全是 0 就不去借、判红
/// （第 2596 行与第 2634-2636 行）；就算去借，账把「带得了 0 个、没去借」记成 `NothingToPlace`，
/// `decide_after_attempting_every_card` 交 `NoCardHoldsOneState`（判红那一支），不是 `NoRoomOnAnyCardNow`（本次未跑）。
#[test]
fn k2_w1_cards_full_right_now_but_fit_when_empty_end_in_the_red_branch_not_in_not_run() {
    // 一张卡：别的占到只剩 3000 MiB 空闲（不够驱动那一份加一个状态），空着时放得下
    let readings = Arc::new(Mutex::new(vec![reading(3_000, None)]));
    let ledger = pool_ledger(&readings, None, Some(32 * 1024));
    let one_state = |states: u64| 600 + states * 2; // 表 600 MiB、一个状态 2 MiB（只要单调、一个状态至少 1 MiB）
    let room = room_now(&ledger);
    let ceiling_room = {
        set(&readings, vec![reading(FREE_EMPTY_5090, None)]);
        let empty = room_now(&ledger);
        set(&readings, vec![reading(3_000, None)]);
        empty
    };
    // 调用方照此刻能借的算带得了几个：0
    let holds_now = if one_state(1) <= room { 1 } else { 0 };
    let outcome = ledger.book_states_on_the_cards_with_room(&[0], 1000, &[holds_now], &one_state);
    let class = match &outcome {
        Ok(_) => "booked".to_string(),
        Err(BookingRefusal::NoCardHasRoomNow { .. }) => "not-run".to_string(),
        Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => format!(
            "red need={} largest_ceiling={}",
            refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
        ),
        Err(BookingRefusal::CardMemoryUnreadable { reason }) => format!("unreadable {reason}"),
    };
    let decision = decide_after_attempting_every_card(&[ScratchAttemptOnACard::NothingToPlace]);
    println!(
        "ROW k2w1 room_now={room} room_when_empty={ceiling_room} one_state={} holds_now={holds_now} booking={class} decision_on_nothing_to_place={decision:?}",
        one_state(1)
    );
    assert!(one_state(1) <= ceiling_room && one_state(1) > room, "空着时放得下、此刻放不下");
    assert!(class.starts_with("red"), "此刻放不下而空着时放得下，落进判红那一支");
    assert_eq!(
        decision,
        ScratchPlacementDecision::NoCardHoldsOneState { largest_ceiling_mebibytes: 0 }
    );
}

/// K1-1 丙形：「空着时能借的」拿卡的总量当这个进程能占的（`gpu_memory_ledger.rs` 第 174 行），而空着的卡读得到的空闲
/// 比总量少驱动留着的那一份（本机 5090 量的 32607 − 32110 = 497 MiB）。要的落在两者之间：账判成「空着时放得下、此刻借不到」，
/// 永远报本次未跑，到不了「等多久都放不下、判红」那一支。取样：四种配置，要的从空卡此刻能借的加 1 起到「空着时能借的」。
#[test]
fn k1_c_the_ceiling_takes_the_total_while_an_empty_card_never_reads_its_total_as_free() {
    let mut band_rows = 0usize;
    for (name, environment_quota, configured) in CONFIGURATIONS {
        let readings = Arc::new(Mutex::new(vec![reading(FREE_EMPTY_5090, None)]));
        let ledger = pool_ledger(&readings, environment_quota, configured);
        let room_on_the_empty_card = room_now(&ledger);
        let need = room_on_the_empty_card + 1;
        let outcome = ledger.book_on_the_first_card_with_room(need);
        let (class, ceiling) = match &outcome {
            Ok(_) => ("booked", 0),
            Err(BookingRefusal::NoCardHasRoomNow { .. }) => ("not-run", 0),
            Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => {
                ("red", refusal.largest_ceiling_mebibytes)
            }
            Err(BookingRefusal::CardMemoryUnreadable { .. }) => ("unreadable", 0),
        };
        // 「空着时能借的」：要大到判红为止，二分找门槛
        let (mut low, mut high) = (need, TOTAL_5090 * 2);
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            match ledger.book_on_the_first_card_with_room(middle) {
                Err(BookingRefusal::NeedLargerThanEveryCard(_)) => high = middle,
                _ => low = middle,
            }
        }
        let band = low - room_on_the_empty_card;
        println!(
            "ROW k1c config={name} room_on_the_empty_card={room_on_the_empty_card} need={need} outcome={class} ceiling_seen_in_red={ceiling} largest_need_still_not_run={low} band_mebibytes={band}"
        );
        if class == "not-run" && band > 0 {
            band_rows += 1;
        }
    }
    println!("SUMMARY k1c configurations_with_a_band={band_rows}");
    assert!(band_rows > 0, "至少一种配置下有一段要的量永远报本次未跑、不判红");
}

/// K2-2 白未跑：同一个进程里另一条用例借着（这个进程自己的、它判完就还），这一条借不到不等，当场报本次未跑；
/// `lease_card_waiting_for_room`（`gpu_unit_checks.rs` 第 1158-1167 行）借不到判红。取样：清场那一份配置、空卡，
/// 先借的那一笔 16、20、24 GiB，后借的 8、12 GiB。
#[test]
fn k2_d_a_sibling_lease_in_this_process_turns_the_next_lease_into_not_run_without_waiting() {
    let mut not_run_rows = 0usize;
    for first_gibibytes in [16u64, 20, 24] {
        for second_gibibytes in [8u64, 12] {
            let readings = Arc::new(Mutex::new(vec![reading(FREE_EMPTY_5090, None)]));
            let ledger = pool_ledger(&readings, None, Some(32 * 1024));
            let first = ledger
                .book_on_the_first_card_with_room(first_gibibytes * 1024)
                .unwrap_or_else(|refusal| panic!("空卡借得出第一笔：{refusal:?}"));
            ledger.record_the_device_started_on_the_card(0);
            let free = FREE_EMPTY_5090 - BASELINE - first_gibibytes * 1024;
            set(&readings, vec![reading(free, Some(BASELINE + first_gibibytes * 1024))]);
            let second = ledger.book_on_the_first_card_with_room(second_gibibytes * 1024);
            let class = match &second {
                Ok(_) => "booked",
                Err(BookingRefusal::NoCardHasRoomNow { .. }) => "not-run",
                Err(BookingRefusal::NeedLargerThanEveryCard(_)) => "red",
                Err(BookingRefusal::CardMemoryUnreadable { .. }) => "unreadable",
            };
            drop(second);
            drop(first);
            // 第一笔还了（同一进程、几秒之后），同一个要的就借得到
            set(&readings, vec![reading(FREE_EMPTY_5090 - BASELINE, Some(BASELINE))]);
            let after_return = ledger.book_on_the_first_card_with_room(second_gibibytes * 1024).is_ok();
            println!(
                "ROW k2d first={} second={} free_while_first_held={free} second_outcome={class} second_books_after_the_first_returned={after_return}",
                first_gibibytes * 1024,
                second_gibibytes * 1024
            );
            if class == "not-run" && after_return {
                not_run_rows += 1;
            }
        }
    }
    println!("SUMMARY k2d not_run_rows={not_run_rows}");
    assert!(not_run_rows > 0);
}
