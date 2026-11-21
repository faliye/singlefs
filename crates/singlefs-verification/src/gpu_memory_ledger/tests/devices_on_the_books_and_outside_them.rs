//! 账上的设备与不走这份账的设备：起设备之前登记驱动那一份、起好了算占上卡，借据还了之后卡上留多少，
//! 不走账的设备按总线记，设备丢了之后重起照此刻再判。

use super::{
    an_rtx_5090_in_the_report, booking_kept_on_the_books, ledger_on_a_card_under_the_ledger,
    ledger_reading, nvidia_smi_report, reading_of_an_rtx_5090,
    record_the_device_started_and_kept_by_the_pool, replace_the_reading, replaceable_readings,
    room_now, ten_thousand_built_on_a_started_device, the_device_is_lost_with_others_taking,
    ReplaceableReadings, AN_EMPTY_RTX_5090_HAS, BUS_OF_THE_RTX_5090, DRIVER, THIS_PROCESS,
};
use crate::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, card_memory_reader_of_one_report_per_reading,
    pci_bus_identifier_without_the_domain, BookingRefusal, BusesWithDevicesStartedOutsideTheBooks,
    CardMemoryReading, DeviceWaitBeforeReturningTheLease, GpuMemoryLedger, OccupiedByThisProcess,
};
use std::sync::Arc;

/// （先登记、后建设备，这个进程里）：没在一张卡上借过就起设备要断言；借过之后起得了，起好了驱动那一份算在卡上：
/// 读数里这个进程占着的驱动那一份从空闲里少了，不再从能借的里扣第二次；读不到实际占着的时候照旧扣（实占多少分不出）。
#[test]
fn the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card(
) {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ledger.check_registered_before_starting_the_device(0);
        }))
        .is_err(),
        "没借过就起设备：先登记、后建设备的次序被打乱，要断言"
    );
    assert!(!ledger.is_this_process_registered_on_the_card(0));
    let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
    assert!(
        ledger.is_this_process_registered_on_the_card(0),
        "借过就登记了"
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![30_000 - 1_000 - DRIVER]),
        "设备没起：驱动那一份还要从空闲里扣"
    );
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "头一回起：驱动那一份在登记的那一笔里借过"
    );
    let _device = ledger.record_the_device_started_on_the_card(0);
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![30_000 - 1_000 - DRIVER]),
        "设备起好了而读不到实际占着的：驱动那一份照旧扣"
    );
    replace_the_reading(
        &readings,
        0,
        CardMemoryReading {
            free_mebibytes: 30_000 - DRIVER,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                mebibytes: DRIVER,
            },
        },
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![30_000 - DRIVER - 1_000]),
        "设备起好了、读到占着驱动那一份：现读的空闲里少了它，不再扣第二次"
    );
    drop(booking);
}

/// （攻方乙形）：不走账的设备起之前就记下它在哪张卡上：起的那一段里卡池读这张卡已经认得出
/// 这个进程另有不走账的设备；没起过的卡不记。
#[test]
fn device_outside_the_books_is_marked_on_its_bus_before_it_starts() {
    let buses = BusesWithDevicesStartedOutsideTheBooks::new();
    let (marked_while_starting, _marked_on_the_bus) = buses
        .start_a_device_marked_first("01:00.0", || {
            Ok::<bool, String>(buses.has_devices_on_the_bus("01:00.0"))
        })
        .expect("造的设备起得来");
    assert!(marked_while_starting, "起的那一段里已经记着");
    assert!(
        buses.has_devices_on_the_bus("01:00.0"),
        "起好的那一台还在：记着"
    );
    assert!(!buses.has_devices_on_the_bus("10:00.0"), "没起过的卡不记");
}

/// 不走账的设备起不来照样记着（分不出它占没占，只会少借）。
#[test]
fn device_outside_the_books_that_failed_to_start_stays_marked_on_its_bus() {
    let buses = BusesWithDevicesStartedOutsideTheBooks::new();
    assert_eq!(
        buses
            .start_a_device_marked_first("10:00.0", || Err::<(), String>("起不来".to_string()))
            .map(|(device, _marked_on_the_bus)| device),
        Err("起不来".to_string())
    );
    assert!(buses.has_devices_on_the_bus("10:00.0"));
}

/// 一条总线上起了两台不走账的设备：丢一台照样记着（还有一台），两台都丢了才摘掉这条总线，计数回到 0。
/// 判别力（`crates/mutations.tsv`「显存账 不走账的设备丢一台就摘掉整条总线」）：丢一台就摘，「丢了一台、还有一台」那条断言红。
/// 第二记包在 `ManuallyDrop` 里、两条中间断言过了才丢：丢一台就摘的账在断言 panic 展开时若再丢第二记，
/// 第二记的 `Drop` 找不到那条总线、在展开中又 panic，进程中止，测试红不在断言上。
#[test]
fn bus_stays_marked_until_the_last_device_outside_the_books_on_it_is_dropped() {
    let buses = BusesWithDevicesStartedOutsideTheBooks::new();
    let (_first_device, first_marked_on_the_bus) = buses
        .start_a_device_marked_first("20:00.0", || Ok::<(), String>(()))
        .expect("造的设备起得来");
    let (_second_device, second_marked_on_the_bus) = buses
        .start_a_device_marked_first("20:00.0", || Ok::<(), String>(()))
        .expect("造的设备起得来");
    let second_marked_on_the_bus = std::mem::ManuallyDrop::new(second_marked_on_the_bus);
    assert_eq!(buses.devices_marked_on_every_bus(), 2);
    drop(first_marked_on_the_bus);
    assert!(
        buses.has_devices_on_the_bus("20:00.0"),
        "丢了一台、还有一台：照样记着"
    );
    assert_eq!(buses.devices_marked_on_every_bus(), 1);
    drop(std::mem::ManuallyDrop::into_inner(second_marked_on_the_bus));
    assert!(
        !buses.has_devices_on_the_bus("20:00.0"),
        "两台都丢了：摘掉这条总线"
    );
    assert_eq!(buses.devices_marked_on_every_bus(), 0);
}

/// （不走账的设备丢了之后借据不再扣两遍）：这个进程在 5090 上起了一台不走账的设备，又经账借了 3 000、起了账上的设备、
/// 建满（报告里这个进程占着驱动那一份加 3 000）。不走账的那一台还在时分不出，能借的是空闲再减驱动那一份、再减借着的（扣两遍，只会少借）；
/// 那一台丢了之后回到按账拆：能借的就是空闲（借着的与驱动那一份已经不在空闲里，不再扣第二遍）。
/// 判别力（`crates/mutations.tsv`「显存账 不走账的设备丢掉时不减」）：丢掉时不减，丢了之后照样扣两遍，这条红。
#[test]
fn once_the_device_outside_the_books_is_dropped_the_card_room_counts_the_leases_once() {
    const LEASED: u64 = 3_000;
    const FREE: u64 = 30_000 - DRIVER - LEASED;
    let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
        FREE,
        &[(THIS_PROCESS, DRIVER + LEASED)],
    )]);
    let buses = Arc::new(BusesWithDevicesStartedOutsideTheBooks::new());
    let buses_read_by_the_ledger = Arc::clone(&buses);
    let ledger = GpuMemoryLedger::new(
        1,
        card_memory_reader_of_one_report_per_reading(
            Box::new(move || Ok(report.clone())),
            THIS_PROCESS,
            vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            Box::new(move |bus: &str| buses_read_by_the_ledger.has_devices_on_the_bus(bus)),
        ),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
        DRIVER,
    );
    let (_device_outside_the_books, marked_on_the_bus) = buses
        .start_a_device_marked_first(
            &pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090),
            || Ok::<(), String>(()),
        )
        .expect("造的设备起得来");
    let lease = ledger
        .book_on_the_card(0, LEASED)
        .expect("空闲 24 440 放得下 3 000 连驱动那一份");
    let _device_on_the_books = ledger.record_the_device_started_on_the_card(0);
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![FREE - DRIVER - LEASED]),
        "不走账的那一台还在：分不出，借着的与驱动那一份再扣一遍"
    );
    drop(marked_on_the_bus);
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![FREE]),
        "那一台丢了：按账拆，借着的与驱动那一份只从空闲里扣一次"
    );
    drop(lease);
}

/// （G2 乙；数照 攻方丙形那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满，
/// 驱动实占 600）：那 4 000 丢掉之前等完设备、交回驱动再销账，读数里这个进程只剩驱动实占的 600；下一笔能借的是卡上此刻真能再给的
/// 19 550（空 21 510 减驱动还要长的 1 960），不把驱动还没长到的那一截当成能借的（多给 D − x）。
#[test]
fn lease_returned_after_the_device_finished_leaves_room_only_for_what_the_driver_still_grows_to() {
    const EMPTY: u64 = 32_110;
    const OUTSIDER: u64 = 10_000;
    const DRIVER_HOLDS_NOW: u64 = 600;
    const LEASE: u64 = 4_000;
    let readings = replaceable_readings(vec![CardMemoryReading {
        free_mebibytes: EMPTY - OUTSIDER,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::NotListed,
    }]);
    let shared = Arc::clone(&readings);
    let ledger = GpuMemoryLedger::new(
        1,
        Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
        Box::new(|_position, may_hold_mebibytes| {
            card_ledger_capacity_mebibytes(
                None,
                Some(32_000),
                may_hold_mebibytes.min(32_000),
                may_hold_mebibytes,
                DRIVER,
            )
        }),
        DRIVER,
    );
    let mut lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
    let device = ledger.record_the_device_started_on_the_card(0);
    lease.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(lease);
    let free_now = EMPTY - OUTSIDER - DRIVER_HOLDS_NOW;
    replace_the_reading(
        &readings,
        0,
        CardMemoryReading {
            free_mebibytes: free_now,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                mebibytes: DRIVER_HOLDS_NOW,
            },
        },
    );
    let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
        room_now + (DRIVER - DRIVER_HOLDS_NOW),
        free_now,
        "能借的 {room_now} 加驱动还要长的 1 960 是卡上空着的 21 510：不多给"
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![room_now]),
        "再读一次照旧"
    );
}

/// （G2 乙；数照 P1、正推在真账上造出的那一组 S10）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据
/// 各借、建、等完设备再丢（丢了之后读数里这个进程占的回到驱动那一份加 10 000）。销了账的不再记：第 k 笔还了之后能借的照旧是真空闲 19 550，
/// 第 10 笔之后要 18 550 照样借得到（甲在这里每笔多扣 1 000，第 10 笔之后只给 9 550、报本次未跑）。
#[test]
fn returned_short_leases_while_a_long_lease_stays_built_leave_the_room_as_it_was() {
    const EMPTY: u64 = 32_110;
    const LONG_LEASE: u64 = 10_000;
    const SHORT_LEASE: u64 = 1_000;
    let readings = replaceable_readings(vec![CardMemoryReading {
        free_mebibytes: EMPTY,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::NotListed,
    }]);
    let shared = Arc::clone(&readings);
    let ledger = GpuMemoryLedger::new(
        1,
        Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
        Box::new(|_position, may_hold_mebibytes| {
            card_ledger_capacity_mebibytes(
                None,
                Some(40_000),
                may_hold_mebibytes.min(40_000),
                may_hold_mebibytes,
                DRIVER,
            )
        }),
        DRIVER,
    );
    let occupied = |free_mebibytes: u64, mebibytes: u64| CardMemoryReading {
        free_mebibytes,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
    };
    let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    let free_while_the_long_lease_is_built = EMPTY - DRIVER - LONG_LEASE;
    replace_the_reading(
        &readings,
        0,
        occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![free_while_the_long_lease_is_built]),
        "长借据建满：能借的是真空闲"
    );
    for short_lease_number in 1..=10 {
        let mut short_lease = ledger
            .book_on_the_card(0, SHORT_LEASE)
            .expect("卡上空着 19 550");
        replace_the_reading(
            &readings,
            0,
            occupied(
                free_while_the_long_lease_is_built - SHORT_LEASE,
                DRIVER + LONG_LEASE + SHORT_LEASE,
            ),
        );
        short_lease.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(short_lease);
        replace_the_reading(
            &readings,
            0,
            occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
        );
        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
                room_now + DRIVER + LONG_LEASE,
                EMPTY,
                "第 {short_lease_number} 笔短借据等完设备、还了：能借的 {room_now} 加长借据与驱动那一份是空卡，销了账的几笔不再扣"
            );
    }
    let need = free_while_the_long_lease_is_built - SHORT_LEASE;
    assert_eq!(
        booking_kept_on_the_books(&ledger, need),
        Ok(need),
        "要 18 550、真空闲 19 550：借得到"
    );
    drop(long_lease);
}

/// 的两条重起用例共用：一张 5090 上空 30 000，借一笔 10 000、设备起好、建满（卡上空 17 440），再借 `second_lease_mebibytes`
/// 还没建。交账与换读数的那一份（卡上的读数停在建满那一刻：丢了的设备还占着）。
fn ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
    second_lease_mebibytes: u64,
) -> (GpuMemoryLedger, ReplaceableReadings) {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    booking_kept_on_the_books(&ledger, 10_000).expect("空着的卡借得到");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    record_the_device_started_and_kept_by_the_pool(&ledger, 0);
    replace_the_reading(
        &readings,
        0,
        CardMemoryReading {
            free_mebibytes: 30_000 - DRIVER - 10_000,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                mebibytes: DRIVER + 10_000,
            },
        },
    );
    booking_kept_on_the_books(&ledger, second_lease_mebibytes).expect("建满之后卡上还空着 17 440");
    (ledger, readings)
}

/// 起过的设备丢了要重起，驱动那一份重新登记：卡上空 17 440（丢了的设备还占着），
/// 第二笔 17 440 还没建，再起一个设备连它的驱动那一份要 20 000，放不下：交借不到，这个进程照旧登记着（驱动那一份许出去而没占上卡）。
#[test]
fn lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now() {
    let (ledger, _readings) = ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
        30_000 - DRIVER - 10_000,
    );
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: DRIVER,
            card_rooms_mebibytes: vec![0],
        }),
        "新设备的驱动那一份放不下：不起"
    );
    assert!(ledger.is_this_process_registered_on_the_card(0));
}

/// 同上，第二笔只有 2 000：再起的设备连驱动那一份照此刻的空闲放得下，起；之后能借的照新设备的驱动那一份还没占上卡、
/// 这个进程占着的分不出算（乙：17 440 减驱动那一份、减借着的 12 000，建上了的 10 000 也照没建扣），
/// 不照丢了的那一份当已在卡上。
#[test]
fn lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card() {
    let (ledger, _readings) =
        ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(2_000);
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![30_000 - DRIVER - 10_000 - 2_000]),
        "丢之前：驱动那一份在卡上"
    );
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "新设备连驱动那一份放得下"
    );
    let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
        room_now + DRIVER + 10_000 + 2_000,
        30_000 - DRIVER - 10_000,
        "重起之前：能借的 {room_now} 加新设备的驱动那一份、加借着的两笔，是此刻空着的 17 440"
    );
}

/// 丢了的设备重起被拒过一次（卡上空 17 440，第二笔 17 440 还没建），被拒之前已借到的
/// 并发用例再来起设备，照此刻再判、照样拒，不直接放行；旧设备与它上面的缓冲都还给驱动之后（卡上空 30 000、这个进程不在报告里）再来，放得下，起。
#[test]
fn lost_device_refused_a_restart_is_judged_again_on_every_later_attempt() {
    let (ledger, readings) = ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
        30_000 - DRIVER - 10_000,
    );
    let refused = Err(BookingRefusal::NoCardHasRoomNow {
        need_mebibytes: DRIVER,
        card_rooms_mebibytes: vec![0],
    });
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        refused,
        "头一次重起：放不下，不起"
    );
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        refused,
        "被拒之前已借到的用例再来：照此刻再判，照样放不下"
    );
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        refused,
        "再来一次照旧"
    );
    replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "旧设备还给驱动之后：借着的 27 440 连新设备的驱动那一份放得下，起"
    );
    let _replacement = ledger.record_the_device_started_on_the_card(0);
    assert!(ledger.is_this_process_registered_on_the_card(0));
}

/// （正推 N10、攻方第三节第 1310 行 乙改数）：重起时借着的连新设备的驱动那一份照此刻的空闲
/// 恰好放得下（空 17 440 = 驱动那一份 2 560 + 借着的 10 000 + 4 880，建上了的那 10 000 照没建算），起，不拒；多 1 MiB 就拒。
#[test]
fn lost_device_started_again_fits_when_its_driver_share_fits_exactly() {
    let (ledger, _readings) =
        ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_880);
    assert_eq!(DRIVER + 10_000 + 4_880, 30_000 - DRIVER - 10_000);
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "恰好放得下：起"
    );
    let (ledger_one_mebibyte_short, _readings_one_mebibyte_short) =
        ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_881);
    assert!(
        ledger_one_mebibyte_short
            .prepare_the_driver_share_before_starting_the_device(0)
            .is_err(),
        "多 1 MiB：不起"
    );
}

/// （G2 乙：「销了账、卡上可能还没还的」那一份拿掉之后，丢了的设备留在卡上的不再记成一份改按放没放掉判）：
/// 空 30 000 的卡上借 A 10 000、设备起好、建满（驱动实占 600）；设备丢了，B 5 000 借到、照重起记，新设备起好（也实占 600），卡池那一格换成新设备。
/// A 的卡还拿着旧设备：旧设备的 600 与 A 的 10 000 留在卡上，读数分不出哪一份是新设备的，能借的照分不出算（空 18 800 减驱动那一份、减 A、B，
/// 1 240），不超过卡上此刻真能再给的 11 840（空 18 800 减新设备驱动还要长的 1 960、减 B 还要建的 5 000）；把旧设备的 600 当新设备的驱动那一份算是
/// 12 440，多给 600。A 等完设备、它的卡丢掉，旧设备的句柄都没了（wgpu 收掉、卡上还给驱动），这一台放掉：照读数分得出算，能借的加 B 与驱动那一份
/// 是空卡（B 是设备丢了那一刻就借着的，照「那一刻活着的借据都还了」判时要等 B 也还了，少给 600）。之后借 C 3 000、B 也还了，照旧。
#[test]
fn lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement() {
    const EMPTY: u64 = 30_000;
    const DRIVER_HOLDS_NOW: u64 = 600;
    const LEASE_ON_THE_LOST_DEVICE: u64 = 10_000;
    const LEASE_STARTING_THE_REPLACEMENT: u64 = 5_000;
    const LEASE_AFTER_THE_RESTART: u64 = 3_000;
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
    let ledger = ledger_reading(&readings);
    let occupied = |mebibytes: u64| CardMemoryReading {
        free_mebibytes: EMPTY - mebibytes,
        total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
        occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
    };
    let mut lease_on_the_lost_device = ledger
        .book_on_the_card(0, LEASE_ON_THE_LOST_DEVICE)
        .expect("空着的卡借得到");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    let lost_device_held_by_the_pool = Arc::new(ledger.record_the_device_started_on_the_card(0));
    let lost_device_held_by_the_lease = Arc::clone(&lost_device_held_by_the_pool);
    replace_the_reading(
        &readings,
        0,
        occupied(DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE),
    );
    let mut lease_starting_the_replacement = ledger
        .book_on_the_card(0, LEASE_STARTING_THE_REPLACEMENT)
        .expect("卡上空着 19 400");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "设备丢了要重起：空 19 400 放得下驱动那一份与两笔"
    );
    let replacement = ledger.record_the_device_started_on_the_card(0);
    drop(lost_device_held_by_the_pool);
    let occupied_with_the_lost_device_left =
        DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE + DRIVER_HOLDS_NOW;
    replace_the_reading(&readings, 0, occupied(occupied_with_the_lost_device_left));
    let free_now = EMPTY - occupied_with_the_lost_device_left;
    let room_while_the_lost_device_remains =
        ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
            room_while_the_lost_device_remains
                + DRIVER
                + LEASE_ON_THE_LOST_DEVICE
                + LEASE_STARTING_THE_REPLACEMENT,
            free_now,
            "旧设备还在卡上：能借的 {room_while_the_lost_device_remains} 加驱动那一份与两笔，是空着的 18 800"
        );
    assert!(
        room_while_the_lost_device_remains
            + (DRIVER - DRIVER_HOLDS_NOW)
            + LEASE_STARTING_THE_REPLACEMENT
            <= free_now,
        "不超过卡上此刻真能再给的（不多给）"
    );
    lease_on_the_lost_device.take_in_the_wait_before_returning(
        &lost_device_held_by_the_lease,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    replace_the_reading(&readings, 0, occupied(DRIVER_HOLDS_NOW));
    drop(lost_device_held_by_the_lease);
    drop(lease_on_the_lost_device);
    let room_after_the_lost_device_is_released =
        ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
            room_after_the_lost_device_is_released + DRIVER + LEASE_STARTING_THE_REPLACEMENT,
            EMPTY,
            "丢了的那一台放掉了：照读数分得出算，能借的 {room_after_the_lost_device_is_released} 加驱动那一份与 B 是空卡"
        );
    let mut lease_after_the_restart = ledger
        .book_on_the_card(0, LEASE_AFTER_THE_RESTART)
        .expect("卡上空着 29 400");
    lease_starting_the_replacement.take_in_the_wait_before_returning(
        &replacement,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(lease_starting_the_replacement);
    let room_after_the_lease_starting_the_replacement_is_returned =
        ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
            room_after_the_lease_starting_the_replacement_is_returned
                + LEASE_AFTER_THE_RESTART
                + DRIVER,
            EMPTY,
            "B 也还了：能借的 {room_after_the_lease_starting_the_replacement_is_returned} 加重起之后借的与驱动那一份是空卡"
        );
    lease_after_the_restart.take_in_the_wait_before_returning(
        &replacement,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
}

/// （`probes/killers.rs` 乙改数）：长借据建满、等完设备还了
/// （卡上还给驱动），新借一笔建满：能借的照读数算，新借的建上的只扣一次，还了的那一笔不再扣（甲在这里多扣驱动那一份，21 990）。
#[test]
fn lease_booked_after_another_was_returned_counts_its_built_buffers_once() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER;
        card.built_mebibytes = 10_000;
    }
    let device = ledger.record_the_device_started_on_the_card(0);
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    drop(first);
    let _second = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
    card.lock().expect("假卡不 panic").built_mebibytes = 5_000;
    let room_after_the_second_is_built = room_now(&ledger);
    assert_eq!(
        room_after_the_second_is_built + 5_000 + DRIVER,
        AN_EMPTY_RTX_5090_HAS,
        "第二笔建满：能借的 {room_after_the_second_is_built} 加第二笔与驱动那一份是空卡"
    );
}

/// 设备丢了、重起被拒一次之后，再被问三次而卡上读数没变，能借的不变
/// （被拒之后记成「丢了、还没重起」，不记回「起着」而每问一次多扣一份）。
#[test]
fn refused_restart_asked_again_takes_no_further_driver_share_off_the_room() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
    // 新设备放不下：卡上 32 110 − 19 000 − 2 560 − 10 000 = 550，驱动那一份要 2 560
    the_device_is_lost_with_others_taking(&card, 19_000);
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: DRIVER,
            card_rooms_mebibytes: vec![0],
        })
    );
    card.lock().expect("假卡不 panic").others_mebibytes = 0;
    let room_after_one_refusal = room_now(&ledger);
    for _ in 0..3 {
        let _ = ledger.prepare_the_driver_share_before_starting_the_device(0);
    }
    assert_eq!(
        room_now(&ledger),
        room_after_one_refusal,
        "再问三次、读数没变：能借的不变"
    );
}

/// 设备丢了、重起被拒之后又借了一笔，之后卡上又被占：再来起设备照此刻判、放不下就拒
/// （借一笔不把驱动那一份记回「登记了、没起」而直接放行）。
#[test]
fn lease_booked_after_a_refused_restart_keeps_the_restart_judged_again() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
    the_device_is_lost_with_others_taking(&card, 19_000);
    assert!(ledger
        .prepare_the_driver_share_before_starting_the_device(0)
        .is_err());
    card.lock().expect("假卡不 panic").others_mebibytes = 0;
    let _another = ledger
        .book_on_the_card(0, 5_000)
        .expect("别的进程走了，借得到");
    card.lock().expect("假卡不 panic").others_mebibytes = 19_000;
    assert!(
        ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .is_err(),
        "被拒之后借了一笔，再来起设备照此刻判：放不下"
    );
}

/// （乙改数）：设备丢了要重起，借着的 10 000 连新设备的驱动那一份
/// 照此刻的空闲恰好放得下就起，差 1 MiB 就拒（丢了的设备与建上的 10 000 留在卡上、不从读数里分出来：空 32 110 − 6 990 − 12 560 = 12 560）。
#[test]
fn restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short() {
    assert_eq!(
        AN_EMPTY_RTX_5090_HAS - 6_990 - (DRIVER + 10_000),
        DRIVER + 10_000
    );
    for (others_mebibytes, fits) in [(6_990u64, true), (6_991, false)] {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
        the_device_is_lost_with_others_taking(&card, others_mebibytes);
        assert_eq!(
            ledger
                .prepare_the_driver_share_before_starting_the_device(0)
                .is_ok(),
            fits,
            "别的进程占着 {others_mebibytes}"
        );
    }
}
