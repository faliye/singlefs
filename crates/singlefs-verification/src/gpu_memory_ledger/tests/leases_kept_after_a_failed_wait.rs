//! 等设备没等成的借据：照旧记在账上，到 poll 那台设备队列空了、同一设备上另一笔等成、设备记成丢了或放掉才销。

use super::{
    ledger_on_a_card_under_the_ledger, ledger_reading, reading_of_an_rtx_5090, replace_the_reading,
    replaceable_readings, room_now, room_truly_left, CardUnderTheLedger, AN_EMPTY_RTX_5090_HAS,
    DRIVER,
};
use crate::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, CardMemoryReading, DeviceStartedOnTheBooks,
    DeviceWaitBeforeReturningTheLease, GpuMemoryBooking, GpuMemoryLedger,
    LeasesKeptAfterAFailedWaitWhenListed, OccupiedByThisProcess,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// （G2 乙，等失败那一支；数照 第 2 条攻方模型「丁」：5090 空卡 32 110，驱动实占 600）：长借据 10 000、
/// C 8 000、V 3 000 都建满、读过一次；C 丢掉之前等设备没等成（超时或报错），卡上还没还，V 这一趟派完、缓冲缩回驱动（V 的借据还活着）。
/// C 照旧记在账上：能借的是配给卡的额度 32 000 减驱动那一份、减三笔借据的 8 440，不超过卡上此刻真能再给的 8 550（空 13 510 减驱动还要长的
/// 1 960、减 V 还要长回的 3 000）；C 照常销了账是 13 510，照账借满卡上就超了。这张卡的设备丢了、账照重起记，C 才销账。
#[test]
fn lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more() {
    const EMPTY: u64 = 32_110;
    const DRIVER_HOLDS_NOW: u64 = 600;
    const LONG_LEASE: u64 = 10_000;
    const RETURNED_LEASE: u64 = 8_000;
    const SHRINKING_LEASE: u64 = 3_000;
    const CONFIGURED_CARD_QUOTA: u64 = 32_000;
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
                Some(CONFIGURED_CARD_QUOTA),
                may_hold_mebibytes.min(CONFIGURED_CARD_QUOTA),
                may_hold_mebibytes,
                DRIVER,
            )
        }),
        DRIVER,
    );
    let occupied = |mebibytes: u64| CardMemoryReading {
        free_mebibytes: EMPTY - mebibytes,
        total_less_reserved_mebibytes: EMPTY,
        occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
    };
    let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    let mut returned_lease = ledger
        .book_on_the_card(0, RETURNED_LEASE)
        .expect("卡上空着 19 440");
    let returned_lease_identifier = returned_lease.lease_identifier();
    let shrinking_lease = ledger
        .book_on_the_card(0, SHRINKING_LEASE)
        .expect("卡上空着 11 440");
    replace_the_reading(
        &readings,
        0,
        occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE),
    );
    assert!(
        ledger.card_rooms_mebibytes_now(&[0]).is_ok(),
        "三笔都建满时读一次"
    );
    returned_lease.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    drop(returned_lease);
    replace_the_reading(
        &readings,
        0,
        occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE),
    );
    let free_now = EMPTY - DRIVER_HOLDS_NOW - LONG_LEASE - RETURNED_LEASE;
    let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
    assert_eq!(
        room_now + DRIVER + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE,
        CONFIGURED_CARD_QUOTA,
        "C 没等成、照旧记着：能借的 {room_now} 加驱动那一份与三笔借据，是配给卡的额度"
    );
    assert!(
            room_now + (DRIVER - DRIVER_HOLDS_NOW) + SHRINKING_LEASE <= free_now,
            "C 卡上还没还，V 缩回驱动：能借的 {room_now} 加驱动还要长的、V 还要长回的，不超过卡上空着的 13 510（不多给）"
        );
    assert!(
        ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
        "没等成的那一笔丢掉之后照旧记在账上"
    );
    let _restart_judged = ledger.prepare_the_driver_share_before_starting_the_device(0);
    assert!(
        !ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
        "设备丢了、账照重起记：没等成的那一笔这时销账"
    );
    drop(shrinking_lease);
    drop(long_lease);
}

/// 借一笔建满 `mebibytes`、等设备超时（设备上还排着别的活）、丢掉：交回它的借据号；缓冲还在卡上，由调用方后来还给驱动。
fn lease_built_and_dropped_after_a_timed_out_wait(
    card: &Mutex<CardUnderTheLedger>,
    ledger: &GpuMemoryLedger,
    device: &DeviceStartedOnTheBooks<'_>,
    mebibytes: u64,
) -> u64 {
    let mut lease = ledger.book_on_the_card(0, mebibytes).expect("借得到");
    let lease_identifier = lease.lease_identifier();
    card.lock().expect("假卡不 panic").built_mebibytes += mebibytes;
    lease.take_in_the_wait_before_returning(
        device,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    drop(lease);
    lease_identifier
}

/// （两腿都量到等超时的那几笔永久少借，正推每笔 4 000、第 7 笔起借不到）：设备起好（驱动实占 600），
/// 一笔笔借 4 000、建满、等设备超时、丢掉；活后来做完、缓冲还给驱动，卡池借下一笔之前对这张卡 poll 一次交回队列空了：没等成的那一笔销账，
/// 能借的回到卡上真能再给的（空卡减驱动那一份）。八笔笔笔借得到，中间没有别的等成。
#[test]
fn lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    for timed_out_lease_number in 1..=8 {
        let timed_out_lease =
            lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
        assert!(
            ledger.is_the_lease_on_the_books(0, timed_out_lease),
            "第 {timed_out_lease_number} 笔没等成：缓冲可能还在卡上，照旧记着"
        );
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "第 {timed_out_lease_number} 笔的缓冲还在卡上时不多给"
        );
        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
        assert_eq!(
            listed
                .iter()
                .map(|listed_on_a_card| (
                    listed_on_a_card.card_position(),
                    listed_on_a_card.lease_identifiers()
                ))
                .collect::<Vec<(usize, Vec<u64>)>>(),
            vec![(0, vec![timed_out_lease])],
            "第 {timed_out_lease_number} 笔：这张卡上有没等成而照旧记着的，卡池借之前要 poll 它"
        );
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
        assert!(
            !ledger.is_the_lease_on_the_books(0, timed_out_lease),
            "第 {timed_out_lease_number} 笔：poll 交回队列空了，那一笔销账"
        );
        assert_eq!(
            room_now(&ledger),
            room_truly_left(&card, 0),
            "第 {timed_out_lease_number} 笔销了账：能借的是卡上真能再给的"
        );
    }
    assert!(ledger
        .leases_kept_after_a_failed_wait_on_each_card()
        .is_empty());
}

/// （攻方 `opus_k11_poll_race.rs` 的 take-in-between：卡池列卡、poll 交回队列空了，
/// 销账之前另一条线程借了一笔、建满、提交了活、等设备没等成；销账时它被一起销了，它的缓冲还在卡上，账多给）：X 10 000 建满、没等成、照旧记着；
/// 卡池列卡（列出 X）、poll 交回队列空了（X 的缓冲还给驱动）；A 借 8 000、建满、没等成；卡池照列出来的销：X 销账，A 照旧记着，不多给。
/// 下一次列卡列出 A，A 的缓冲还给驱动之后再 poll 交回队列空了，A 才销账。
#[test]
fn lease_kept_after_the_listing_stays_on_the_books_when_the_poll_after_the_listing_finds_the_device_idle(
) {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    let lease_listed_after_its_failed_wait =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 10_000);
    let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
    assert_eq!(
        listed
            .iter()
            .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
            .collect::<Vec<Vec<u64>>>(),
        vec![vec![lease_listed_after_its_failed_wait]],
        "列卡那一刻照旧记着的只有 X"
    );
    // poll 交回队列空了：X 的活做完、缓冲还给驱动
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    let lease_booked_after_the_listing =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 8_000);
    device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
    assert!(
        !ledger.is_the_lease_on_the_books(0, lease_listed_after_its_failed_wait),
        "X 列出来了、poll 那一刻之前提交的活都做完了：销账"
    );
    assert!(
        ledger.is_the_lease_on_the_books(0, lease_booked_after_the_listing),
        "A 是列卡之后才记成没等成的：它的活可以是 poll 之后才提交的，照旧记着"
    );
    assert!(
        room_now(&ledger) <= room_truly_left(&card, 0),
        "A 的缓冲还在卡上：不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, 0)
    );
    let listed_again = ledger.leases_kept_after_a_failed_wait_on_each_card();
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    device.take_in_that_the_work_of_the_listed_leases_finished(&listed_again[0]);
    assert!(
        !ledger.is_the_lease_on_the_books(0, lease_booked_after_the_listing),
        "下一次列卡列出 A、poll 交回队列空了：A 销账"
    );
    assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
}

/// （攻方 `scripts/mut.py` 的 M1、M2 照绿：放掉的是在用的那一台没有用例）：在用的那一台上 X 没等成、
/// 照旧记着，这台设备的克隆都丢了（卡池整个丢掉那一类；今天卡池活到进程退出，产品路径走不到，账的接口上走得到）：X 销账、这张卡上不再有在用的那一台，
/// 下一台起得来，能借的是卡上真能再给的。
#[test]
fn releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    let kept_lease = lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
    {
        // 放掉：wgpu 收掉设备，它上面的缓冲与驱动那一份交回驱动
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.built_mebibytes = 0;
    }
    drop(device);
    assert!(
        !ledger.is_the_lease_on_the_books(0, kept_lease),
        "在用的那一台放掉了：它上面没等成的那一笔销账"
    );
    let next_lease = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
    let next_device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    assert_eq!(
        room_now(&ledger),
        room_truly_left(&card, 1_000),
        "下一台起好了：能借的是卡上真能再给的"
    );
    drop(next_lease);
    drop(next_device);
}

/// 正推账层探针那一形的起头：设备起好（驱动实占 600）、头一笔等完设备还了；再借此刻能借的减 1 000、建满、等设备超时、丢掉（照旧记着）。
/// 交回账上的那一台、照旧记着的那一笔的借据号与它借的量（MiB）。
fn started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait<'ledger>(
    card: &Mutex<CardUnderTheLedger>,
    ledger: &'ledger GpuMemoryLedger,
) -> (DeviceStartedOnTheBooks<'ledger>, u64, u64) {
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    let kept_mebibytes = room_now(ledger) - 1_000;
    let kept_lease =
        lease_built_and_dropped_after_a_timed_out_wait(card, ledger, &device, kept_mebibytes);
    (device, kept_lease, kept_mebibytes)
}

/// （正推账层探针 `probe_kept_leases_on_a_lost_device.rs`：丢了的设备上没等成的那一笔
/// 记在账上，账到重起才知道设备丢了，卡池只在借到之后才走到重起；能借的 1 000、借 4 000 被拒，这张卡在这个进程里再也借不出）：设备丢了，
/// 它上面的缓冲与驱动那一份交回了驱动（卡上空着）；卡池借之前列卡、看到留着的设备丢了：不 poll 它、告诉账，那一笔销账，借 4 000 借得到，
/// 能借的不超过卡上真能再给的；之后的重起照常（驱动那一份照此刻判、新的一台记得上）。
#[test]
fn lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (device, kept_lease, _kept_mebibytes) =
        started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.driver_occupied_mebibytes = 0;
        card.built_mebibytes = 0;
    }
    assert_eq!(
        room_now(&ledger),
        1_000,
        "账还不知道设备丢了：照旧记着的那一笔占着，能借的 1 000（正推探针 room_now_after_lost）"
    );
    let polls_of_the_lost_device = AtomicUsize::new(0);
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| true,
        |_kept| {
            polls_of_the_lost_device.fetch_add(1, Ordering::SeqCst);
            true
        },
        |kept| *kept,
    );
    assert_eq!(
        polls_of_the_lost_device.load(Ordering::SeqCst),
        0,
        "丢了的设备不 poll（wgpu 在丢了的设备上 poll 会 panic）"
    );
    assert!(
        !ledger.is_the_lease_on_the_books(0, kept_lease),
        "借之前账就知道设备丢了：它上面没等成的那一笔照重起记、销账"
    );
    assert!(
        room_now(&ledger) <= room_truly_left(&card, 0),
        "不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, 0)
    );
    let booking = ledger
        .book_on_the_card(0, 4_000)
        .expect("卡上空着：借 4 000 借得到（改之前 NoCardHasRoomNow，能借的 1 000）");
    ledger
        .prepare_the_driver_share_before_starting_the_device(0)
        .expect("重起：驱动那一份照此刻放得下");
    let replacement = ledger.record_the_device_started_on_the_card(0);
    drop(booking);
    drop(replacement);
    drop(device);
}

/// （K4-2 那一改不多给）：同上，设备丢了而它上面那一笔的缓冲与驱动那一份还占着卡（旧设备还没收拾）；卡池借之前告诉账之后，
/// 那一笔出了账、这张卡照分不出算：能借的不超过卡上真能再给的，借 4 000 照旧借不到。
#[test]
fn device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (device, kept_lease, kept_mebibytes) =
        started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.driver_occupied_mebibytes = 0;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = 600 + kept_mebibytes;
    }
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| true,
        |_kept| panic!("丢了的设备不 poll"),
        |kept| *kept,
    );
    assert!(
        !ledger.is_the_lease_on_the_books(0, kept_lease),
        "告诉账之后照重起记：那一笔出了账（缓冲在丢了的那一台上，放掉之前这张卡照分不出算）"
    );
    let room = room_now(&ledger);
    let room_truly_left_on_the_card = room_truly_left(&card, 0);
    assert!(
            room <= room_truly_left_on_the_card,
            "旧设备连那一笔的缓冲还占着卡：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}（不多给）"
        );
    assert!(
        ledger.book_on_the_card(0, 4_000).is_err(),
        "卡上真没有 4 000：照旧借不到"
    );
    drop(device);
}

/// （K4-2 那一改，卡池看到的是重起之前留着的那一份）：丢了的那一台重起过了（重起那一步已经把它记成丢了），
/// 新的一台上有一笔没等成而照旧记着；卡池借之前取到的还是丢了的那一台、告诉账它丢了：账不动，新的一台没丢，它上面那一笔照旧记着。
#[test]
fn device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let lost_device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &lost_device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    ledger
        .prepare_the_driver_share_before_starting_the_device(0)
        .expect("重起：驱动那一份放得下");
    let replacement = ledger.record_the_device_started_on_the_card(0);
    let kept_on_the_replacement =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
    let room_before = room_now(&ledger);
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&lost_device),
        |_kept| true,
        |_kept| panic!("丢了的设备不 poll"),
        |kept| *kept,
    );
    assert!(
        ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
        "新的一台没丢：它上面没等成的那一笔照旧记着"
    );
    assert_eq!(room_now(&ledger), room_before, "账不动");
    assert_eq!(
        ledger
            .leases_kept_after_a_failed_wait_on_each_card()
            .iter()
            .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
            .collect::<Vec<Vec<u64>>>(),
        vec![vec![kept_on_the_replacement]],
        "下一次列卡照旧列出新的一台上那一笔"
    );
    drop(replacement);
    drop(lost_device);
}

/// （卡池借之前 poll 的结局改成恒真照绿）：留着的设备没丢，poll 交回还有在飞的提交：
/// poll 了一次，那一笔照旧记着（它的缓冲可能还在卡上），账不动。
#[test]
fn device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (device, kept_lease, _kept_mebibytes) =
        started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
    let room_before = room_now(&ledger);
    let polls = AtomicUsize::new(0);
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| false,
        |_kept| {
            polls.fetch_add(1, Ordering::SeqCst);
            false
        },
        |kept| *kept,
    );
    assert_eq!(polls.load(Ordering::SeqCst), 1, "没丢：poll 一次");
    assert!(
        ledger.is_the_lease_on_the_books(0, kept_lease),
        "还有在飞的提交：那一笔照旧记着"
    );
    assert_eq!(room_now(&ledger), room_before, "账不动");
    assert!(room_now(&ledger) <= room_truly_left(&card, 0), "不多给");
    drop(device);
}

/// （K5，正推 Y2、Y3：poll 的结局改成恒假、列卡的循环只看第一张卡照绿）：两张卡上各有一笔没等成而照旧记着，
/// 两台设备都没丢、poll 都交回队列空了：每张卡 poll 一次，两笔都销账。
#[test]
fn every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
    ]);
    let ledger = ledger_reading(&readings);
    let mut devices = Vec::new();
    let mut kept_leases = Vec::new();
    for card_position in 0..2 {
        let mut lease = ledger
            .book_on_the_card(card_position, 1_000)
            .expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(card_position);
        lease.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        kept_leases.push(lease.lease_identifier());
        drop(lease);
        devices.push(device);
    }
    let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| devices.get(card_position),
        |_kept| false,
        |kept| {
            polls_on_each_card[kept.card_position].fetch_add(1, Ordering::SeqCst);
            true
        },
        |kept| *kept,
    );
    assert_eq!(
        polls_on_each_card
            .iter()
            .map(|polls| polls.load(Ordering::SeqCst))
            .collect::<Vec<usize>>(),
        vec![1, 1],
        "列出来的每张卡 poll 一次"
    );
    for (card_position, kept_lease) in kept_leases.iter().enumerate() {
        assert!(
            !ledger.is_the_lease_on_the_books(card_position, *kept_lease),
            "第 {card_position} 张卡的设备队列空了：那一笔销账"
        );
    }
}

/// （K5，正推 G1：去掉销列出来的借据时「等的是 poll 的那一台」那一条照绿）：丢了的那一台重起过、还没放掉，新的一台上
/// 有一笔没等成而照旧记着；拿丢了的那一台交「列出来的那几笔的活做完了」：列出来的那一笔等的是新的一台，不销。
#[test]
fn finished_poll_of_a_lost_device_left_alive_does_not_return_the_lease_kept_on_its_replacement() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let lost_device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &lost_device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    ledger
        .prepare_the_driver_share_before_starting_the_device(0)
        .expect("重起：驱动那一份放得下");
    let replacement = ledger.record_the_device_started_on_the_card(0);
    let kept_on_the_replacement =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
    let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
    lost_device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
    assert!(
        ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
        "poll 的是丢了的那一台：等新的一台的那一笔不销"
    );
    drop(replacement);
    drop(lost_device);
}

/// （K5，正推 G4：去掉「列的是这台设备那张卡」那一条断言照绿）：列的是第 1 张卡，交给第 0 张卡上的设备销：断言拦下，
/// 第 1 张卡上那一笔照旧记着。
#[test]
fn the_listed_leases_of_another_card_given_to_a_device_are_refused_by_an_assertion() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
    ]);
    let ledger = ledger_reading(&readings);
    let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
    let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
    let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
    let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
    lease_on_the_second_card.take_in_the_wait_before_returning(
        &device_on_the_second_card,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
    drop(lease_on_the_second_card);
    let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        device_on_the_first_card.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
    }));
    assert!(
        refused.is_err(),
        "列的是第 1 张卡、交给第 0 张卡上的设备：断言拦下"
    );
    assert!(ledger.is_the_lease_on_the_books(1, kept_on_the_second_card));
    drop(booking_on_the_first_card);
    drop(device_on_the_first_card);
    drop(device_on_the_second_card);
}

/// （K4-2 那一改的接线）：卡池那一格交的设备在账上记在别的卡上（列的是第 1 张卡，取到的是第 0 张卡上的设备）：
/// 断言拦下。
#[test]
fn kept_device_on_the_books_of_another_card_is_refused_by_an_assertion() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
    ]);
    let ledger = ledger_reading(&readings);
    let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
    let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
    let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
    let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
    lease_on_the_second_card.take_in_the_wait_before_returning(
        &device_on_the_second_card,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
    drop(lease_on_the_second_card);
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ledger.take_in_the_devices_kept_before_booking(
            |_card_position| Some(&device_on_the_first_card),
            |_kept| true,
            |_kept| panic!("丢了的设备不 poll"),
            |kept| *kept,
        );
    }));
    assert!(refused.is_err(), "取到的设备不在列出来的那张卡上：断言拦下");
    assert!(
        ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
        "第 1 张卡上那一笔照旧记着"
    );
    drop(booking_on_the_first_card);
    drop(device_on_the_first_card);
    drop(device_on_the_second_card);
}

/// （拦 Z8：借之前没丢的那一支 poll 之后销这张卡此刻全部没等成的、
/// 不只列出来的，照绿）：poll 的当口（不在账的锁里）又有一笔借据记成没等成；poll 交回队列空了：列出来的那一笔销账，
/// 列卡之后才记成没等成的那一笔照旧记着（它的活可以是 poll 之后才提交的）。
#[test]
fn lease_kept_after_the_listing_survives_the_poll_before_booking() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (device, listed_lease, _kept_mebibytes) =
        started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
    let lease_kept_during_the_poll = std::cell::Cell::new(None);
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| false,
        |kept| {
            let mut lease = ledger
                .book_on_the_card(0, 500)
                .expect("还剩 1 000：借得到 500");
            lease.take_in_the_wait_before_returning(
                kept,
                DeviceWaitBeforeReturningTheLease::TheWaitFailed,
            );
            lease_kept_during_the_poll.set(Some(lease.lease_identifier()));
            drop(lease);
            true
        },
        |kept| *kept,
    );
    assert!(
        !ledger.is_the_lease_on_the_books(0, listed_lease),
        "列出来的那一笔：poll 交回队列空了，销账"
    );
    assert!(
        ledger.is_the_lease_on_the_books(
            0,
            lease_kept_during_the_poll
                .get()
                .expect("poll 的当口记了一笔没等成的")
        ),
        "列卡之后才记成没等成的那一笔照旧记着"
    );
    drop(device);
}

/// （K4，正推 T2 拦 Z6：告诉账丢了时在用的那一台不进丢了还没放掉的那一堆，照绿）：借之前告诉账设备丢了、
/// 之后重起好了，丢了的那一台连它的缓冲与驱动那一份还占着卡（还没放掉）：能借的不超过卡上真能再给的。
#[test]
fn the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (device, _kept_lease, kept_mebibytes) =
        started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.driver_occupied_mebibytes = 0;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = 600 + kept_mebibytes;
    }
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| true,
        |_kept| panic!("丢了的设备不 poll"),
        |kept| *kept,
    );
    ledger
        .prepare_the_driver_share_before_starting_the_device(0)
        .expect("重起：驱动那一份放得下");
    let replacement = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    let room = room_now(&ledger);
    let room_truly_left_on_the_card = room_truly_left(&card, 0);
    assert!(
        room <= room_truly_left_on_the_card,
        "丢了的那一台还在卡上：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}"
    );
    drop(replacement);
    drop(device);
}

/// （K4，正推 T3 拦 Z1：取留着的设备缺了的那张卡就停、不看后面的卡，照绿）：第 0 张卡那一格没有留着的设备
/// （正在起），第 1 张卡的设备队列空了：第 1 张卡上那一笔照样销账，第 0 张卡上那一笔照旧记着。
#[test]
fn card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
    ]);
    let ledger = ledger_reading(&readings);
    let mut devices = Vec::new();
    let mut kept_leases = Vec::new();
    for card_position in 0..2 {
        let mut lease = ledger
            .book_on_the_card(card_position, 1_000)
            .expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(card_position);
        lease.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        kept_leases.push(lease.lease_identifier());
        drop(lease);
        devices.push(device);
    }
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 1).then(|| &devices[1]),
        |_kept| false,
        |_kept| true,
        |kept| *kept,
    );
    assert!(
        ledger.is_the_lease_on_the_books(0, kept_leases[0]),
        "第 0 张卡那一格没有留着的设备可 poll：那一笔照旧记着"
    );
    assert!(
        !ledger.is_the_lease_on_the_books(1, kept_leases[1]),
        "第 1 张卡的设备队列空了：那一笔销账"
    );
}

/// 设备起好（驱动实占 600）、头一笔等完设备还了，之后设备丢了，
/// 它的驱动那一份还占着卡，账上没有没等成的借据（列卡列不出这张卡）。账还不知道丢了时照读数把那 600 当成这个进程许出去的：
/// 能借的 29 550、卡上真能再给的 28 950，借走之后重起被拒。借之前那一步看池里每张卡：丢了的照重起记，能借的就是卡上真能再给的，
/// 借走它之后重起放得下。
#[test]
fn device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart(
) {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
    }
    first.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(first);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.driver_occupied_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = 600;
    }
    assert_eq!(
        room_now(&ledger),
        29_550,
        "账还不知道设备丢了：把丢了的设备还占着的驱动那一份 600 当成许出去的（正推 T4 room_now）"
    );
    assert_eq!(
        room_truly_left(&card, 0),
        28_950,
        "卡上真能再给的（正推 T4 truly_left）"
    );
    let losses_asked = AtomicUsize::new(0);
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| (card_position == 0).then_some(&device),
        |_kept| {
            losses_asked.fetch_add(1, Ordering::SeqCst);
            true
        },
        |_kept| panic!("丢了的设备不 poll"),
        |kept| *kept,
    );
    assert_eq!(
        losses_asked.load(Ordering::SeqCst),
        1,
        "没有没等成借据的卡也问一次丢没丢"
    );
    let room = room_now(&ledger);
    assert_eq!(
        room,
        room_truly_left(&card, 0),
        "借之前账就知道丢了：能借的就是卡上真能再给的（不多给，也不少给）"
    );
    let booking = ledger
        .book_on_the_card(0, room)
        .expect("能借的那么多借得到");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "借走能借的全部之后重起：驱动那一份照此刻放得下"
    );
    let replacement = ledger.record_the_device_started_on_the_card(0);
    drop(booking);
    drop(replacement);
    drop(device);
}

/// （K3-3 那一改：借之前看池里每张卡，只 poll 列出来的）：第 0 张卡的设备起着、没丢、没有没等成的借据，
/// 第 1 张卡上一笔没等成：只 poll 第 1 张卡，那一笔销账，第 0 张卡的账不动。
#[test]
fn card_without_a_kept_lease_is_not_polled_before_booking() {
    let readings = replaceable_readings(vec![
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
    ]);
    let ledger = ledger_reading(&readings);
    let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
    let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
    let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
    let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
    lease_on_the_second_card.take_in_the_wait_before_returning(
        &device_on_the_second_card,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
    drop(lease_on_the_second_card);
    let devices = [&device_on_the_first_card, &device_on_the_second_card];
    let rooms_before = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出");
    let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
    ledger.take_in_the_devices_kept_before_booking(
        |card_position| devices.get(card_position).copied(),
        |_kept| false,
        |kept| {
            polls_on_each_card[kept.card_position].fetch_add(1, Ordering::SeqCst);
            true
        },
        |kept| *kept,
    );
    assert_eq!(
        polls_on_each_card
            .iter()
            .map(|polls| polls.load(Ordering::SeqCst))
            .collect::<Vec<usize>>(),
        vec![0, 1],
        "只 poll 列出来的那张卡"
    );
    assert!(
        !ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
        "第 1 张卡的设备队列空了：那一笔销账"
    );
    assert_eq!(
        ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出"),
        rooms_before,
        "第 0 张卡的账不动"
    );
    drop(booking_on_the_first_card);
    drop(device_on_the_first_card);
    drop(device_on_the_second_card);
}

/// （攻方改法 F-K1）：没等成的那一笔记着，同一台设备上后来另一笔借据等成了（等的是到那一刻为止的全部提交）：
/// 没等成的那一笔一起销账。
#[test]
fn lease_whose_wait_timed_out_leaves_the_books_when_another_lease_on_its_device_finishes_its_wait()
{
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut finishing_later = ledger.book_on_the_card(0, 2_000).expect("空卡借得到");
    let device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
        card.built_mebibytes = 2_000;
    }
    let timed_out_lease =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 6_000);
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    finishing_later.take_in_the_wait_before_returning(
        &device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    assert!(
        !ledger.is_the_lease_on_the_books(0, timed_out_lease),
        "同一台上另一笔等成了：没等成的那一笔的活也做完了，销账"
    );
    drop(finishing_later);
    assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
}

/// （正推：触发重起的那一笔拿的是新设备）：A 建满在旧设备上，设备丢了；B 借到、照重起记、
/// 新设备起好、B 建满，B 等新设备超时、丢掉（缓冲还在卡上）。A 在旧设备上等成了：不销 B（B 等的是新设备）。旧设备放掉之后照读数分，
/// B 照旧记着，不多给；新设备后来 poll 交队列空了、B 的缓冲还给驱动，B 才销账。
#[test]
fn finished_wait_on_a_lost_device_leaves_a_lease_kept_on_its_replacement_on_the_books() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (mut lease_on_the_lost_device, lost_device) =
        ten_thousand_built_with_the_device_held(&card, &ledger);
    let mut lease_starting_the_replacement =
        ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
    let starting_the_replacement = lease_starting_the_replacement.lease_identifier();
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = DRIVER + 10_000;
    }
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    let replacement = ledger.record_the_device_started_on_the_card(0);
    {
        // 新设备的驱动还没长满（实占 600）：B 的缓冲要是被当成驱动还要长的那一份，就看得出多给
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = 600;
        card.built_mebibytes = 5_000;
    }
    lease_starting_the_replacement.take_in_the_wait_before_returning(
        &replacement,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    drop(lease_starting_the_replacement);
    lease_on_the_lost_device.take_in_the_wait_before_returning(
        &lost_device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    assert!(
        ledger.is_the_lease_on_the_books(0, starting_the_replacement),
        "旧设备上等成了，说明不了新设备上 B 的活做完了：B 照旧记着"
    );
    card.lock()
        .expect("假卡不 panic")
        .left_by_a_lost_device_mebibytes = 0;
    drop(lost_device);
    drop(lease_on_the_lost_device);
    assert!(
        room_now(&ledger) <= room_truly_left(&card, 0),
        "旧设备放掉、照读数分：B 的缓冲还在卡上，不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, 0)
    );
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
    assert_eq!(listed.len(), 1, "B 照旧记在这张卡上：列得出");
    replacement.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
    assert!(!ledger.is_the_lease_on_the_books(0, starting_the_replacement));
    assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
}

/// 借一笔 10 000、起设备、建满（驱动实占 [`DRIVER`]），交回借据与账上的这一台（`Arc`：借据的卡拿着一份；卡池那一格的那一份
/// 由调用方丢的时候另克隆，这里不留）。
fn ten_thousand_built_with_the_device_held<'ledger>(
    card: &Mutex<CardUnderTheLedger>,
    ledger: &'ledger GpuMemoryLedger,
) -> (
    GpuMemoryBooking<'ledger>,
    Arc<DeviceStartedOnTheBooks<'ledger>>,
) {
    let lease = ledger
        .book_on_the_card(0, 10_000)
        .expect("空卡借得到 10 000");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    let device = Arc::new(ledger.record_the_device_started_on_the_card(0));
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER;
        card.built_mebibytes = 10_000;
    }
    (lease, device)
}

/// （与改法 F-K3 自报的风险）：旧设备上借着 A 10 000（建满，驱动实占 600），
/// 设备丢了；L 5 000 借到、照重起记、新设备起好、L 建满，卡池那一格换成新设备。A 等旧设备没等成、丢掉：照常销账（这时照分不出算）。
/// 调用方另留着旧设备的一个克隆（产品路径的 `Arc<GpuCard>` 一类）：旧设备连 A 的缓冲还占着卡，L 也还了，这张卡照旧照分不出算，不多给
/// （照「那一刻活着的借据都还了」判，这时回到照读数分，旧设备的那一份被当成新借据建上的）。那个克隆也丢了、旧设备放掉，照读数分：
/// 长借据 Q 15 000 建满之后再借 10 000，借得到，能借的就是卡上真能再给的（攻方 `k1_3` 那六段的形状）。
#[test]
fn lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released() {
    const DRIVER_HOLDS_NOW: u64 = 600;
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut lease_on_the_lost_device = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
    let lease_on_the_lost_device_identifier = lease_on_the_lost_device.lease_identifier();
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    let held_by_the_pool = Arc::new(ledger.record_the_device_started_on_the_card(0));
    let held_by_the_lease_on_the_lost_device = Arc::clone(&held_by_the_pool);
    let held_by_the_caller = Arc::clone(&held_by_the_pool);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
        card.built_mebibytes = 10_000;
    }
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 10_000;
    }
    let mut lease_on_the_restarted_device =
        ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "重起放得下"
    );
    let replacement = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
        card.built_mebibytes = 5_000;
    }
    drop(held_by_the_pool);
    lease_on_the_lost_device.take_in_the_wait_before_returning(
        &held_by_the_lease_on_the_lost_device,
        DeviceWaitBeforeReturningTheLease::TheWaitFailed,
    );
    drop(held_by_the_lease_on_the_lost_device);
    drop(lease_on_the_lost_device);
    assert!(
        !ledger.is_the_lease_on_the_books(0, lease_on_the_lost_device_identifier),
        "A 等的是丢了还没放掉的那一台：没等成也照常销账，不卡在账上"
    );
    assert!(
        room_now(&ledger) <= room_truly_left(&card, 0),
        "旧设备还被调用方拿着、连 A 的缓冲占着卡：不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, 0)
    );
    card.lock().expect("假卡不 panic").built_mebibytes = 0;
    lease_on_the_restarted_device.take_in_the_wait_before_returning(
        &replacement,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(lease_on_the_restarted_device);
    assert!(
        room_now(&ledger) <= room_truly_left(&card, 0),
        "L 也还了、旧设备还在：照旧照分不出算，不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, 0)
    );
    card.lock()
        .expect("假卡不 panic")
        .left_by_a_lost_device_mebibytes = 0;
    drop(held_by_the_caller);
    assert_eq!(
        room_now(&ledger),
        room_truly_left(&card, 0),
        "旧设备放掉了：照读数分，能借的是卡上真能再给的"
    );
    let _long_lease = ledger.book_on_the_card(0, 15_000).expect("借得到 15 000");
    card.lock().expect("假卡不 panic").built_mebibytes = 15_000;
    assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
    assert!(
        ledger.book_on_the_card(0, 10_000).is_ok(),
        "Q 建满之后卡上放得下 10 000：借得到"
    );
}

/// （正推探针 P5、攻方 H5 的 X7：重起好了、旧设备还在时又丢一次，不重新记账）：第一台丢了、调用方还拿着它，
/// 第二台起好、L 在它上面建满之后也丢了，M 借到、照重起记、第三台起好。第一台放掉了，第二台还被 L 的卡与调用方拿着、连 L 的缓冲占着卡：
/// 照旧照分不出算，不多给；L 还了、第二台也放掉，照读数分。
#[test]
fn second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released(
) {
    const DRIVER_HOLDS_NOW: u64 = 600;
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let mut lease_on_the_first = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    let first_device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
    }
    lease_on_the_first.take_in_the_wait_before_returning(
        &first_device,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(lease_on_the_first);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
    }
    let mut lease_on_the_restarted_device =
        ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "第一次重起"
    );
    let second_device = Arc::new(ledger.record_the_device_started_on_the_card(0));
    let restarted_device_held_by_its_lease = Arc::clone(&second_device);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
        card.built_mebibytes = 5_000;
    }
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + DRIVER_HOLDS_NOW + 5_000;
    }
    let _lease_after_the_second_loss = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(()),
        "第二次重起：第一台还没放掉时又丢一次"
    );
    let _third_device = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
        card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 5_000;
    }
    drop(first_device);
    let unbuilt_mebibytes_of_the_lease_after_the_second_loss = 1_000;
    assert!(
        room_now(&ledger)
            <= room_truly_left(&card, unbuilt_mebibytes_of_the_lease_after_the_second_loss),
        "第一台放掉了、第二台连 L 的缓冲还在卡上：照分不出算，不多给（{} 对 {}）",
        room_now(&ledger),
        room_truly_left(&card, unbuilt_mebibytes_of_the_lease_after_the_second_loss)
    );
    card.lock()
        .expect("假卡不 panic")
        .left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
    lease_on_the_restarted_device.take_in_the_wait_before_returning(
        &restarted_device_held_by_its_lease,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
    drop(restarted_device_held_by_its_lease);
    drop(lease_on_the_restarted_device);
    assert!(
        room_now(&ledger)
            <= room_truly_left(&card, unbuilt_mebibytes_of_the_lease_after_the_second_loss),
        "L 还了、第二台还被调用方拿着：不多给"
    );
    card.lock()
        .expect("假卡不 panic")
        .left_by_a_lost_device_mebibytes = 0;
    drop(second_device);
    assert_eq!(
        room_now(&ledger),
        room_truly_left(&card, unbuilt_mebibytes_of_the_lease_after_the_second_loss),
        "丢了的两台都放掉了：照读数分，能借的是卡上真能再给的"
    );
}

/// （正推探针 P3、攻方 H5 的 X1 那一形：重起时没等成的借据要一起出账，不然这张卡到下一次丢设备都少给）：
/// C 在设备上没等成、照旧记着（缓冲还在卡上）；设备丢了、重起：C 这时销账；旧设备连 C 的缓冲放掉之后，能借的就是卡上真能再给的。
#[test]
fn lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short() {
    let (card, ledger) = ledger_on_a_card_under_the_ledger();
    let (lease, lost_device) = ten_thousand_built_with_the_device_held(&card, &ledger);
    let kept_lease =
        lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &lost_device, 4_000);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.built_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = DRIVER + 10_000 + 4_000;
    }
    let mut lease_starting_the_replacement =
        ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
    assert_eq!(
        ledger.prepare_the_driver_share_before_starting_the_device(0),
        Ok(())
    );
    assert!(
        !ledger.is_the_lease_on_the_books(0, kept_lease),
        "设备丢了、账照重起记：没等成的那一笔这时销账"
    );
    let replacement = ledger.record_the_device_started_on_the_card(0);
    {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = true;
        card.driver_occupied_mebibytes = DRIVER;
        card.left_by_a_lost_device_mebibytes = 0;
    }
    drop(lost_device);
    drop(lease);
    assert_eq!(
        room_now(&ledger),
        room_truly_left(&card, 1_000),
        "旧设备放掉了：能借的是卡上真能再给的，没有一笔卡在账上"
    );
    lease_starting_the_replacement.take_in_the_wait_before_returning(
        &replacement,
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
    );
}
