//! 卡池上的借：每一笔现读显存，照这个进程借着的与别的进程占的算放不放得下，按状态数借草稿区、指名一张卡借，
//! 读不出或读到的卡比池里多时不借。

use super::{
    an_rtx_5090_in_the_report, booking_kept_on_the_books, ledger_reading,
    no_device_outside_the_books, nvidia_smi_report, reading_of_an_rtx_5090, replace_the_reading,
    replaceable_readings, ANOTHER_PROCESS, AN_EMPTY_RTX_5090_HAS, BUS_OF_THE_RTX_5090, DRIVER,
    THIS_PROCESS, TOTAL_OF_AN_RTX_5090,
};
use crate::gpu_memory_ledger::{
    card_memory_reader_of_one_report_per_reading, most_states_whose_need_fits_the_capacity,
    pci_bus_identifier_without_the_domain, place_states_on_the_cards_that_book_now,
    scratch_booking_refusal_outcome, BookingRefusal, CardMemoryReading,
    DeviceWaitBeforeReturningTheLease, GpuMemoryLedger, NeedLargerThanEveryCard,
    OccupiedByThisProcess, ScratchAttemptOnACard, ScratchBookingRefusalOutcome,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// （G1 二）：这个进程借了 1 000 之后，一个不经这份账的 GPU 用户占了 20 000（卡上空闲从 30 000 掉到 9 000）。
/// 下一笔照这一刻现读的借：9 000 减这个进程借了还没占的 1 000、减还没起设备的驱动那一份，正好 5 440，多 1 MiB 就借不到。
#[test]
fn booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(
        booking_kept_on_the_books(&ledger, 1_000),
        Ok(1_000),
        "空着的卡借得到"
    );
    replace_the_reading(&readings, 0, reading_of_an_rtx_5090(9_000));
    assert_eq!(
        booking_kept_on_the_books(&ledger, 5_441),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 5_441,
            card_rooms_mebibytes: vec![9_000 - 1_000 - DRIVER],
        }),
        "外人占了 20 000：这一笔照现读的 9 000 算，借不到（不判红、不等）"
    );
    assert_eq!(
        booking_kept_on_the_books(&ledger, 5_440),
        Ok(5_440),
        "现读的空闲减借了还没占的、减驱动那一份，正好借得到"
    );
}

/// （G1 二）：外人占着卡（空 5 000）时借 10 000 借不到，不判红（卡空着时借得出 29 550）；
/// 外人走了（空 30 000），下一笔照现读的借到。
#[test]
fn booking_after_an_outsider_left_gets_the_memory_it_returned() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(
        booking_kept_on_the_books(&ledger, 10_000),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 10_000,
            card_rooms_mebibytes: vec![5_000 - DRIVER],
        }),
        "外人占着：借不到，不当成比卡还大"
    );
    replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
    assert_eq!(
        booking_kept_on_the_books(&ledger, 10_000),
        Ok(10_000),
        "外人走了：下一笔照现读的借到"
    );
}

/// （G1 二）：这个进程在同一张卡上先后两笔，卡上读数一直是空 30 000（借的都还没建）：第一笔 20 000，
/// 第二笔最多 7 440，两笔加驱动那一份正好是现读的空闲，多 1 MiB 就借不到。
#[test]
fn two_bookings_of_this_process_on_one_card_stay_within_the_free_memory_read_now() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(booking_kept_on_the_books(&ledger, 20_000), Ok(20_000));
    assert!(
        matches!(
            booking_kept_on_the_books(&ledger, 7_441),
            Err(BookingRefusal::NoCardHasRoomNow { .. })
        ),
        "两笔加驱动那一份超过现读的空闲：借不到"
    );
    assert_eq!(booking_kept_on_the_books(&ledger, 7_440), Ok(7_440));
    assert_eq!(
        20_000 + 7_440 + DRIVER,
        30_000,
        "两笔加驱动那一份正好是现读的空闲"
    );
}

/// （正推探针 G 那一形）：卡上只空 1 000，不到一份驱动开销：借 1 MiB 也借不到，这个进程不登记（调用方不起设备）。
#[test]
fn card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(1_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(
        booking_kept_on_the_books(&ledger, 1),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 1,
            card_rooms_mebibytes: vec![0],
        })
    );
    assert!(
        !ledger.is_this_process_registered_on_the_card(0),
        "借不到就不登记"
    );
}

/// K4-1 用的核账：一笔借据丢掉之前一直在账上，丢掉就不在了。
#[test]
fn lease_is_on_the_books_until_it_is_dropped() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
    let lease_identifier = booking.lease_identifier();
    assert!(ledger.is_the_lease_on_the_books(0, lease_identifier));
    drop(booking);
    assert!(
        !ledger.is_the_lease_on_the_books(0, lease_identifier),
        "丢掉就销账"
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]),
        Ok(vec![30_000 - DRIVER]),
        "销了账，这一笔不再从空闲里扣"
    );
}

/// （正推探针 C、攻方 S3 那一形）：卡上空 20 000，这个进程已经借着 12 000（没建）。按状态数借 30 个、
/// 每个 1 000 MiB：照此刻能借的 5 440 压成 5 个，当场借到，不按卡的整份容量压成 17 个再判放不下。
#[test]
fn scratch_booking_compresses_its_states_to_the_room_now_and_books_them() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(20_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(booking_kept_on_the_books(&ledger, 12_000), Ok(12_000));
    let bookings = ledger
        .book_states_on_the_cards_with_room(&[0], 30, &[30], &|states| states * 1_000)
        .expect("此刻放得下 5 个状态");
    let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
    assert_eq!(
        booked.states, 5,
        "照此刻能借的 20 000 − 12 000 − 驱动那一份压"
    );
    assert_eq!(booked.booking.mebibytes(), 5_000);
}

/// （b、正推探针 D 那一形）：两张卡，第一张这个进程借着 12 000（空 20 000），第二张空 16 000。
/// 30 个状态、每个 1 000 MiB、每张带得了 30 个：第一张照此刻能借的装 5 个，剩下的 25 个里第二张装得下 13 个，两张都用上。
#[test]
fn the_states_a_card_cannot_take_now_go_to_the_next_card_with_room() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(20_000),
        reading_of_an_rtx_5090(16_000),
    ]);
    let ledger = ledger_reading(&readings);
    std::mem::forget(ledger.book_on_the_card(0, 12_000).expect("第一张借得到"));
    let bookings = ledger
        .book_states_on_the_cards_with_room(&[0, 1], 30, &[30, 30], &|states| states * 1_000)
        .expect("两张都借得到");
    let states_booked: Vec<Option<u64>> = bookings
        .iter()
        .map(|booked| booked.as_ref().map(|booked| booked.states))
        .collect();
    assert_eq!(states_booked, vec![Some(5), Some(13)]);
}

/// 要的比卡空着时能借的（空卡上能用的减驱动那一份，29 550）还大，判成比每张卡都大（调用方判红）；不超过它、只是此刻
/// 放不下的是借不到（调用方报本次未跑）。按状态数借时一个状态都放不下同理。
#[test]
fn need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
    let ledger = ledger_reading(&readings);
    let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
    assert_eq!(
        booking_kept_on_the_books(&ledger, ceiling + 1),
        Err(BookingRefusal::NeedLargerThanEveryCard(
            NeedLargerThanEveryCard {
                need_mebibytes: ceiling + 1,
                largest_ceiling_mebibytes: ceiling,
            }
        ))
    );
    assert!(matches!(
        booking_kept_on_the_books(&ledger, ceiling),
        Err(BookingRefusal::NoCardHasRoomNow { .. })
    ));
    assert!(matches!(
        ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states * (ceiling + 1)),
        Err(BookingRefusal::NeedLargerThanEveryCard(_))
    ));
    assert!(matches!(
        ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states * 6_000),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 6_000,
            ..
        })
    ));
    assert!(
        !ledger.is_this_process_registered_on_the_card(0),
        "一笔都没借到：不登记"
    );
}

/// 读不出卡上的显存、读到的卡数与池对不上：判红（[`BookingRefusal::CardMemoryUnreadable`]），什么都不记。
#[test]
fn an_unreadable_card_memory_is_refused_and_nothing_is_booked() {
    let unreadable = GpuMemoryLedger::new(
        1,
        Box::new(|| Err("nvidia-smi 起不来".to_string())),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
        DRIVER,
    );
    assert_eq!(
        booking_kept_on_the_books(&unreadable, 1),
        Err(BookingRefusal::CardMemoryUnreadable {
            reason: "nvidia-smi 起不来".to_string()
        })
    );
    let too_few = GpuMemoryLedger::new(
        2,
        Box::new(|| Ok(vec![reading_of_an_rtx_5090(30_000)])),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
        DRIVER,
    );
    assert!(matches!(
        booking_kept_on_the_books(&too_few, 1),
        Err(BookingRefusal::CardMemoryUnreadable { .. })
    ));
    assert!(!unreadable.is_this_process_registered_on_the_card(0));
    assert!(!too_few.is_this_process_registered_on_the_card(0));
}

/// 「估计在压与借之间变小」那一形换了形态：调用方照空 30 000 时能借的压出 274 个每个 100 MiB 的状态；
/// 借之前外人占了卡（空 10 000），借的时候在锁里照现读的重压成 74 个（7 400 ≤ 10 000 − 驱动那一份），借到。
#[test]
fn states_compressed_before_an_outsider_took_memory_are_compressed_again_in_the_booking() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    let need_of = |states: u64| states * 100;
    let room_when_compressed = ledger.card_rooms_mebibytes_now(&[0]).expect("读得到")[0];
    let states_compressed =
        most_states_whose_need_fits_the_capacity(1_000, room_when_compressed, &|states| {
            need_of(u64::try_from(states).expect("装得进 u64"))
        });
    assert_eq!(states_compressed, 274, "照 30 000 − 驱动那一份压出 274 个");
    replace_the_reading(&readings, 0, reading_of_an_rtx_5090(10_000));
    let bookings = ledger
        .book_states_on_the_cards_with_room(&[0], 274, &[274], &need_of)
        .expect("照现读的重压一次，借得到");
    let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
    assert_eq!(booked.states, 74, "照 10 000 − 驱动那一份重压成 74 个");
    assert_eq!(booked.booking.mebibytes(), 7_400);
}

/// 点名一张卡借，只记在那张卡上：第一张空着也不挪过去；要的比点名的那张空着时能借的大、比第一张小的，
/// 判成比卡大（判红），不记到第一张上。
#[test]
fn booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room() {
    let ledger = GpuMemoryLedger::new(
        2,
        Box::new(|| {
            Ok(vec![
                reading_of_an_rtx_5090(30_000),
                reading_of_an_rtx_5090(30_000),
            ])
        }),
        Box::new(|position, may_hold_mebibytes| match position {
            0 => may_hold_mebibytes.min(29_000),
            _ => may_hold_mebibytes.min(12_000),
        }),
        DRIVER,
    );
    let booking = ledger
        .book_on_the_card(1, 4_000)
        .expect("第二张卡空着，借得到");
    assert_eq!(booking.card_position(), 1, "点名第二张卡就记在第二张卡上");
    drop(booking);
    let refusal = ledger
        .book_on_the_card(1, 20_000)
        .map(|too_large_booking| too_large_booking.card_position())
        .expect_err("要的比第二张卡空着时能借的大");
    assert!(
        matches!(
            scratch_booking_refusal_outcome("crash_amplification::run", 1, &refusal),
            ScratchBookingRefusalOutcome::Red { .. }
        ),
        "比点名的那张大要判红，不挪到第一张卡上：{refusal:?}"
    );
}

/// （两腿甲形）：每借一次只取一份报告，空闲与这个进程占着的出自那一份。假的报告按次序给：
/// 第一份卡空着（这一笔借 3 000）；第二份是第一笔建上卡之后同一刻的空闲与占着的；第三份是外人又占了 10 000。
/// 两次借各取一份、借的那一刻能借的照各自那一份算；取两份拼一份（空闲取前一份、占着的取后一份，或整份取后一份）都对不上。
#[test]
fn each_booking_reads_the_free_memory_and_this_process_occupancy_from_one_report() {
    let reports = [
        nvidia_smi_report(&[an_rtx_5090_in_the_report(30_000, &[])]),
        nvidia_smi_report(&[an_rtx_5090_in_the_report(
            30_000 - DRIVER - 3_000,
            &[(THIS_PROCESS, DRIVER + 3_000)],
        )]),
        nvidia_smi_report(&[an_rtx_5090_in_the_report(
            30_000 - DRIVER - 3_000 - 10_000,
            &[(THIS_PROCESS, DRIVER + 3_000), (ANOTHER_PROCESS, 10_000)],
        )]),
    ];
    let reports_taken = Arc::new(AtomicUsize::new(0));
    let taken_by_the_reader = Arc::clone(&reports_taken);
    let ledger = GpuMemoryLedger::new(
        1,
        card_memory_reader_of_one_report_per_reading(
            Box::new(move || {
                let taken = taken_by_the_reader.fetch_add(1, Ordering::SeqCst);
                reports
                    .get(taken)
                    .cloned()
                    .ok_or_else(|| format!("第 {} 份报告用例里没有", taken + 1))
            }),
            THIS_PROCESS,
            vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            Box::new(no_device_outside_the_books),
        ),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
        DRIVER,
    );
    let first = ledger.book_on_the_card(0, 3_000).expect("空着的卡借得到");
    assert_eq!(
        first.card_room_mebibytes_when_booked(),
        30_000 - DRIVER,
        "第一笔照第一份报告：空 30 000，减还没起设备的驱动那一份"
    );
    let _device = ledger.record_the_device_started_on_the_card(0);
    let second = ledger
        .book_on_the_card(0, 1_000)
        .expect("第二份报告里还空着 24 440");
    assert_eq!(
        second.card_room_mebibytes_when_booked(),
        30_000 - DRIVER - 3_000,
        "第二笔照第二份报告：第一笔建上卡的 3 000 只从空闲里扣一次，不既在空闲里又在占着的里"
    );
    assert_eq!(
        reports_taken.load(Ordering::SeqCst),
        2,
        "借两次各取一份报告：空闲与占着的不分两次读"
    );
    drop(second);
    drop(first);
}

/// （G2 乙「改用乙」）：第一笔 4 000 建上卡，第二笔 3 000 借着还没建；第一笔丢掉之前等完设备
/// （缓冲交回驱动，读数里这个进程只剩驱动那一份）再销账：之后能借的是卡上空着的减第二笔还要建的、减驱动那一份，
/// 销了账的 4 000 不再从空闲里扣（甲在这里多扣驱动那一份 2 560）。
#[test]
fn lease_returned_after_the_device_finished_is_no_longer_taken_off_the_free_memory() {
    const EMPTY: u64 = 30_000;
    const FIRST_LEASE: u64 = 4_000;
    const SECOND_LEASE: u64 = 3_000;
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
    let ledger = ledger_reading(&readings);
    let mut first = ledger
        .book_on_the_card(0, FIRST_LEASE)
        .expect("空着的卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    replace_the_reading(
        &readings,
        0,
        CardMemoryReading {
            free_mebibytes: EMPTY - DRIVER - FIRST_LEASE,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                mebibytes: DRIVER + FIRST_LEASE,
            },
        },
    );
    let second = ledger
        .book_on_the_card(0, SECOND_LEASE)
        .expect("卡上空着 23 440，借得到");
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    replace_the_reading(
        &readings,
        0,
        CardMemoryReading {
            free_mebibytes: EMPTY - DRIVER,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                mebibytes: DRIVER,
            },
        },
    );
    let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
        room_now + SECOND_LEASE + DRIVER,
        EMPTY,
        "能借的 {room_now} 加第二笔还要建的、加驱动那一份，是卡上空着的：销了账的第一笔不再扣"
    );
    assert!(
        matches!(
            ledger.book_on_the_card(0, room_now + 1),
            Err(BookingRefusal::NoCardHasRoomNow { .. })
        ),
        "多 1 MiB 就借不到"
    );
    drop(second);
}

/// （攻方丙形）：报告里一张空着的 5090 空 32 110（总量 32 607 减保留 497）。要的比空卡上能用的减驱动那一份
/// （29 550）大 1 MiB：判成比每张卡都大（判红），不是永远借不到的本次未跑。
#[test]
fn need_beyond_what_an_empty_card_has_never_fits_even_below_the_total_memory() {
    let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(5_000, &[])]);
    let ledger = GpuMemoryLedger::new(
        1,
        card_memory_reader_of_one_report_per_reading(
            Box::new(move || Ok(report.clone())),
            THIS_PROCESS,
            vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            Box::new(no_device_outside_the_books),
        ),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
        DRIVER,
    );
    let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
    assert!(ceiling + 1 < TOTAL_OF_AN_RTX_5090 - DRIVER);
    assert_eq!(
        booking_kept_on_the_books(&ledger, ceiling + 1),
        Err(BookingRefusal::NeedLargerThanEveryCard(
            NeedLargerThanEveryCard {
                need_mebibytes: ceiling + 1,
                largest_ceiling_mebibytes: ceiling,
            }
        )),
        "空卡上能用的装不下：判红，不照总量判成此刻借不到"
    );
}

/// 调用方照此刻能借的算出这张卡一个状态都带不了（带得了 0 个），而空着时放得下：
/// 判成此刻借不到（调用方报本次未跑），不当成前面的卡装完了、一张都没借到就判红。
#[test]
fn card_the_caller_found_holding_no_state_now_is_no_room_now_not_nothing_left_to_place() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(DRIVER + 1_000)]);
    let ledger = ledger_reading(&readings);
    assert_eq!(
        ledger
            .book_states_on_the_cards_with_room(&[0], 10, &[0], &|states| states * 2_000)
            .map(|bookings| bookings.len()),
        Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 2_000,
            card_rooms_mebibytes: vec![1_000],
        }),
        "此刻带不了、空着时放得下：此刻借不到"
    );
    assert_eq!(
        place_states_on_the_cards_that_book_now(
            10,
            &[4, 0],
            &mut |_card_slot, states_to_place| ScratchAttemptOnACard::Booked {
                states: states_to_place
            },
            &mut |_card_slot| ScratchAttemptOnACard::NoRoomNow {
                need_mebibytes: 2_000
            },
        ),
        vec![
            ScratchAttemptOnACard::Booked { states: 4 },
            ScratchAttemptOnACard::NoRoomNow {
                need_mebibytes: 2_000
            },
        ],
        "还剩 6 个要装、第二张此刻带不了：是此刻借不到，不是没有要装的"
    );
}

/// 按状态数借时同一张卡给了两次要断言（两笔记在一张卡上、各按那张卡此刻能借的压，会借超）。
#[test]
fn the_same_card_given_twice_to_a_scratch_booking_is_refused_by_an_assertion() {
    let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
    let ledger = ledger_reading(&readings);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ledger
                .book_states_on_the_cards_with_room(&[0, 0], 4, &[2, 2], &|states| states * 100)
                .map(|bookings| bookings.len())
        }))
        .is_err(),
        "同一张卡给了两次：断言"
    );
    assert!(
        !ledger.is_this_process_registered_on_the_card(0),
        "断言在借之前：什么都没记"
    );
}

/// 读到的卡比池里多（一张卡的池读到两张）也判读不出，什么都不记。
#[test]
fn reading_of_more_cards_than_the_pool_is_refused_and_nothing_is_booked() {
    let too_many = GpuMemoryLedger::new(
        1,
        Box::new(|| {
            Ok(vec![
                reading_of_an_rtx_5090(30_000),
                reading_of_an_rtx_5090(30_000),
            ])
        }),
        Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
        DRIVER,
    );
    assert!(matches!(
        booking_kept_on_the_books(&too_many, 1),
        Err(BookingRefusal::CardMemoryUnreadable { .. })
    ));
    assert!(!too_many.is_this_process_registered_on_the_card(0));
}
