//! 同一张卡上前一次开卡到点还没回来时不起下一次（进程级的那一张表），没到点的排队等它开完。

use super::*;

/// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒）、还没回来的开卡那一张表是 `openings_not_returned`：
/// 起卡那一件头一次挂着直到用例放开（至多 20 秒），之后当场开成；每被起一次往 `starts_made` 加一、往 `start_begun` 送一声。
fn opening_held_at_its_first_start_until_released(
    start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
    starts_made: &Arc<std::sync::atomic::AtomicU32>,
    start_begun: &std::sync::mpsc::Sender<()>,
    openings_not_returned: &OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let start_released = Arc::clone(start_released);
    let starts_counted = Arc::clone(starts_made);
    let start_begun = start_begun.clone();
    opening_of_a_card_within_the_limit(
        Duration::from_millis(50),
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit,
              _wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let card = CardComputedOnTheCpuForTests::of_the_address(
                &discrete_gpu.pci_bus_address,
                memory_limit.map(|memory_limit| memory_limit.0),
                bytes_leaked_by_earlier_mounts.0,
            );
            let start_released_seen_by_the_start = Arc::clone(&start_released);
            let starts_counted_by_the_start = Arc::clone(&starts_counted);
            let start_begun_by_the_start = start_begun.clone();
            move || {
                let starts_before =
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                // 收的那一头在用例里一直拿着。
                let _signal_sent = start_begun_by_the_start.send(());
                if starts_before == 0 {
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .expect("用例没在别处 panic 过")
                        .recv_timeout(Duration::from_secs(20));
                }
                Ok(card)
            }
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        openings_not_returned.clone(),
    )
}

/// 第 4 条「同一张卡前一次开卡没回来就不再起下一次」管到整个进程：前一次挂载在「0000:41:00.0」上的一次开卡到点之后还挂着，
/// 下一次挂载（另起一份开卡那一步，共用进程开卡器那一张表）开它不再起，算没开成；起卡那一件一共只被起 1 次。
#[test]
fn an_opening_hung_in_an_earlier_mount_keeps_a_later_mount_from_starting_another_on_that_card() {
    let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
    let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
    let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
    let start_released = Arc::new(Mutex::new(start_released));
    let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
    let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mut opening_of_the_earlier_mount = opening_held_at_its_first_start_until_released(
        &start_released,
        &starts_made,
        &start_begun,
        &openings_not_returned,
    );
    let mut opening_of_the_later_mount = opening_held_at_its_first_start_until_released(
        &start_released,
        &starts_made,
        &start_begun,
        &openings_not_returned,
    );
    let earlier_mount = opening_of_the_earlier_mount(
        &discrete_gpu,
        None,
        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
    )
    .err();
    every_start_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("前一次挂载的那一次开卡 20 秒内起了");
    let later_mount = opening_of_the_later_mount(
        &discrete_gpu,
        None,
        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
    )
    .err();
    let starts_after_the_later_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
    release_the_start
        .send(())
        .expect("起卡那一件还在等放开：收的那一头还在");
    assert!(matches!(
        earlier_mount,
        Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
    ));
    assert_eq!(
        later_mount,
        Some(GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned),
        "下一次挂载在这张卡上不再起"
    );
    assert_eq!(starts_after_the_later_mount, 1);
}

/// 前一次开卡回来了（那条线程跑完）就从那一张表里划掉：同一张卡下一次照起、开成。
#[test]
fn an_opening_that_returns_after_its_limit_lets_the_next_opening_of_that_card_start() {
    let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
    let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
    let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
    let start_released = Arc::new(Mutex::new(start_released));
    let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
    let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mut opening = opening_held_at_its_first_start_until_released(
        &start_released,
        &starts_made,
        &start_begun,
        &openings_not_returned,
    );
    let hung = opening(
        &discrete_gpu,
        None,
        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
    );
    every_start_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("那一次开卡 20 秒内起了");
    release_the_start
        .send(())
        .expect("起卡那一件还在等放开：收的那一头还在");
    let returned_within_twenty_seconds = (0..2000).any(|_poll| {
        if openings_not_returned
            .devices_with_an_opening_not_returned()
            .is_empty()
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
        false
    });
    let again = opening(
        &discrete_gpu,
        None,
        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
    );
    assert!(matches!(
        hung.err(),
        Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
    ));
    assert!(
        returned_within_twenty_seconds,
        "放开之后那一次开卡 20 秒内回来了"
    );
    assert!(again.is_ok(), "回来之后照起：{:?}", again.err());
    assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
}

// ── 前一次开卡到点没回来不记进连着开不成的次数 ──

/// 「0000:41:00.0」上前一次开卡到点没回来：开卡那一步次次报没回来——切换点它开 4 次都报没回来、答开不成；再交同一组，照常为它起开卡、又开 4 次，
/// 报的仍是没回来、不是试满（没回来不记进这一次挂载里连着开不成的次数）。
#[test]
fn openings_answered_not_returned_four_times_in_a_row_leave_the_card_opened_again_at_the_next_switch(
) {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(opening_not_started_while_an_earlier_one_has_not_returned())
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered_at_the_switch = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered_at_the_next_switch = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    let not_returned_four_times = card_at_forty_one_not_opened_with(vec![
            opening_not_started_while_an_earlier_one_has_not_returned();
            4
        ]);
    assert_eq!(answered_at_the_switch, not_returned_four_times);
    assert_eq!(
        answered_at_the_next_switch, not_returned_four_times,
        "再切到它照常起开卡，报的是没回来、不是试满"
    );
    assert_eq!(
        openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
        8
    );
}

// ── 进程级那一张表：没到点、正在开的开卡，后到的一方排队等它开完 ──

/// 两次挂载拿的那一张表：进程开卡器那一张（产品照这样给），或各拿一张。
#[derive(Clone, Copy, Debug)]
enum TablesOfTheTwoMounts {
    OneTableOfTheProcess,
    OneTableForEachMount,
}

/// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 `opening_limit`，在开的开卡那一张表是 `openings_on_the_devices`）：
/// 起卡那一件先往 `start_begun` 送一声、等用例放开（至多 20 秒）再开成替身卡。
fn opening_held_until_released_within(
    opening_limit: Duration,
    start_released: std::sync::mpsc::Receiver<()>,
    start_begun: std::sync::mpsc::Sender<()>,
    openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let start_released = Arc::new(Mutex::new(start_released));
    opening_of_a_card_within_the_limit(
        opening_limit,
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit,
              _wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let card = CardComputedOnTheCpuForTests::of_the_address(
                &discrete_gpu.pci_bus_address,
                memory_limit.map(|memory_limit| memory_limit.0),
                bytes_leaked_by_earlier_mounts.0,
            );
            let start_released_seen_by_the_start = Arc::clone(&start_released);
            let start_begun_by_the_start = start_begun.clone();
            move || {
                // 收的那一头在用例里一直拿着。
                let _signal_sent = start_begun_by_the_start.send(());
                let _released_or_not = start_released_seen_by_the_start
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .recv_timeout(Duration::from_secs(20));
                Ok(card)
            }
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        openings_on_the_devices.clone(),
    )
}

/// 产品那一份开一张卡那一步（一次至多等 20 秒，在开的开卡那一张表是 `openings_on_the_devices`），起卡那一件当场开成替身卡。
fn opening_that_opens_at_once_within_twenty_seconds(
    openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    opening_of_a_card_within_the_limit(
        Duration::from_secs(20),
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit,
              _wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let card = CardComputedOnTheCpuForTests::of_the_address(
                &discrete_gpu.pci_bus_address,
                memory_limit.map(|memory_limit| memory_limit.0),
                bytes_leaked_by_earlier_mounts.0,
            );
            move || Ok(card)
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        openings_on_the_devices.clone(),
    )
}

/// 等到 `openings_on_the_devices` 里有一次开卡在排队（至多 20 秒）：等到了交回 `true`。
fn an_opening_queued_within_twenty_seconds(
    openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
) -> bool {
    (0..2000).any(|_poll| {
        if openings_on_the_devices.openings_waiting_for_an_earlier_one() > 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
        false
    })
}

/// 「另一次挂载正在开」那一段历史的读数。
struct ReadingsOfASwitchWhileAnotherMountOpens {
    the_other_mount_failed_with: Option<GpuFailureCause>,
    this_mount_queued_behind_the_other: bool,
    answered_while_the_other_mount_opens: LatestSettingsHandedIn,
    answered_once_handed_in_again: LatestSettingsHandedIn,
    openings_in_this_mount: usize,
}

/// 同一进程里另一次挂载正在开「0000:41:00.0」（一次至多等 20 秒，起卡那一件等用例放开，没到点）；这一次挂载（切换开卡那一步用产品那一份）
/// 切换只点它，等开完；之后再交同一组。`OneTableOfTheProcess` 时这一次挂载的开卡排上队之后才放开另一次挂载，不然那一问答了之后才放开。
fn readings_of_a_switch_while_another_mount_opens_the_card(
    tables: TablesOfTheTwoMounts,
) -> ReadingsOfASwitchWhileAnotherMountOpens {
    let table_of_this_mount = OpeningsNotReturnedInThisProcess::with_none_not_returned();
    let table_of_the_other_mount = match tables {
        TablesOfTheTwoMounts::OneTableOfTheProcess => table_of_this_mount.clone(),
        TablesOfTheTwoMounts::OneTableForEachMount => {
            OpeningsNotReturnedInThisProcess::with_none_not_returned()
        }
    };
    let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
    let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
    let mut opening_of_the_other_mount = opening_held_until_released_within(
        Duration::from_secs(20),
        start_released,
        start_begun,
        &table_of_the_other_mount,
    );
    let the_other_mount = std::thread::spawn(move || {
        opening_of_the_other_mount(
            &discrete_gpu_not_opened_at("0000:41:00.0"),
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err()
    });
    every_start_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("另一次挂载的开卡 20 秒内起了");
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_opens_at_once_within_twenty_seconds(&table_of_this_mount),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let this_mount_queued_behind_the_other = match tables {
        TablesOfTheTwoMounts::OneTableOfTheProcess => {
            let queued = an_opening_queued_within_twenty_seconds(&table_of_this_mount);
            release_the_start
                .send(())
                .expect("另一次挂载的起卡那一件还在等放开");
            queued
        }
        TablesOfTheTwoMounts::OneTableForEachMount => false,
    };
    let answered_while_the_other_mount_opens = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    match tables {
        TablesOfTheTwoMounts::OneTableOfTheProcess => {}
        TablesOfTheTwoMounts::OneTableForEachMount => {
            release_the_start
                .send(())
                .expect("另一次挂载的起卡那一件还在等放开");
        }
    }
    let the_other_mount_failed_with = the_other_mount.join().expect("另一次挂载那条线程没 panic");
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered_once_handed_in_again = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    ReadingsOfASwitchWhileAnotherMountOpens {
        the_other_mount_failed_with,
        this_mount_queued_behind_the_other,
        answered_while_the_other_mount_opens,
        answered_once_handed_in_again,
        openings_in_this_mount: openings_made_so_far(&cards_handed_to_the_opening),
    }
}

/// 第 4 条（两个池同时开同一张卡，后到的等先到的开完再开）：两次挂载共用进程那一张表，另一次挂载正在开「0000:41:00.0」、没到点——
/// 这一次挂载的切换开卡排队等它开完再开，开成、换上，不算它「前一次开卡没回来」、不在这一次挂载里试满；另一次挂载照常开成。
#[test]
fn a_switch_while_another_mount_opens_the_card_queues_behind_that_opening_and_puts_the_card_in_effect(
) {
    let readings = readings_of_a_switch_while_another_mount_opens_the_card(
        TablesOfTheTwoMounts::OneTableOfTheProcess,
    );
    assert_eq!(readings.the_other_mount_failed_with, None);
    assert!(
        readings.this_mount_queued_behind_the_other,
        "这一次挂载的开卡排在另一次挂载那一次后面"
    );
    assert_eq!(
        readings.answered_while_the_other_mount_opens,
        put_in_effect_on_every_card()
    );
    assert_eq!(
        readings.answered_once_handed_in_again,
        put_in_effect_on_every_card()
    );
    assert_eq!(readings.openings_in_this_mount, 1);
}

/// 对照：两次挂载各拿一张表，这一次挂载不排队、当场开成换上。
#[test]
fn a_switch_while_another_mount_with_its_own_table_opens_the_card_puts_the_card_in_effect_at_once()
{
    let readings = readings_of_a_switch_while_another_mount_opens_the_card(
        TablesOfTheTwoMounts::OneTableForEachMount,
    );
    assert_eq!(readings.the_other_mount_failed_with, None);
    assert_eq!(
        readings.answered_while_the_other_mount_opens,
        put_in_effect_on_every_card()
    );
    assert_eq!(
        readings.answered_once_handed_in_again,
        put_in_effect_on_every_card()
    );
    assert_eq!(readings.openings_in_this_mount, 1);
}

/// 排队等的时间算进后到那一方自己的期限：另一次挂载正在开「0000:41:00.0」（一次至多等 20 秒，没到点），这一次开它至多等 50 毫秒——
/// 等满 50 毫秒还没轮到，照这一次到点办（报等超过上限、带 50 毫秒），这一次的起卡那一件一次都没起。
#[test]
fn an_opening_queued_behind_another_mount_past_its_own_limit_is_not_opened_within_that_limit() {
    const OPENING_LIMIT_OF_THIS_OPENING: Duration = Duration::from_millis(50);
    let openings_on_the_devices = OpeningsNotReturnedInThisProcess::with_none_not_returned();
    let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
    let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
    let mut opening_of_the_other_mount = opening_held_until_released_within(
        Duration::from_secs(20),
        start_released,
        start_begun,
        &openings_on_the_devices,
    );
    let the_other_mount = std::thread::spawn(move || {
        opening_of_the_other_mount(
            &discrete_gpu_not_opened_at("0000:41:00.0"),
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err()
    });
    every_start_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("另一次挂载的开卡 20 秒内起了");
    let starts_of_this_opening = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let starts_counted = Arc::clone(&starts_of_this_opening);
    let mut opening_of_this_mount = opening_of_a_card_within_the_limit(
        OPENING_LIMIT_OF_THIS_OPENING,
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit,
              _wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let card = CardComputedOnTheCpuForTests::of_the_address(
                &discrete_gpu.pci_bus_address,
                memory_limit.map(|memory_limit| memory_limit.0),
                bytes_leaked_by_earlier_mounts.0,
            );
            let starts_counted_by_the_start = Arc::clone(&starts_counted);
            move || {
                starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(card)
            }
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        openings_on_the_devices.clone(),
    );
    let this_opening = opening_of_this_mount(
        &discrete_gpu_not_opened_at("0000:41:00.0"),
        None,
        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
    )
    .err();
    release_the_start
        .send(())
        .expect("另一次挂载的起卡那一件还在等放开");
    let the_other_mount_failed_with = the_other_mount.join().expect("另一次挂载那条线程没 panic");
    assert_eq!(
        this_opening,
        Some(GpuFailureCause::WaitForTheGpuExceededTheLimit {
            wait_limit: OPENING_LIMIT_OF_THIS_OPENING,
        })
    );
    assert_eq!(
        starts_of_this_opening.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert_eq!(the_other_mount_failed_with, None);
}
