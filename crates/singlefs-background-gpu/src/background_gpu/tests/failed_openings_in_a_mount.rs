//! 一张卡在一次挂载里连着开不成的次数：跨切换接着数、中间开成一次清零、用完之后不再开。

use super::*;

/// 一段历史在切换点的卡开不成之后的读数：开不成之后开着的卡、参与的卡，与再交同一组那一刻的答。
struct ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
    answered_once_not_opened: LatestSettingsHandedIn,
    gpus_opened_once_not_opened: Vec<GpuDeviceIdentifier>,
    participating_once_not_opened: Vec<GpuDeviceIdentifier>,
    answered_when_handed_in_again: LatestSettingsHandedIn,
    openings: usize,
}

/// 挂载开着「0000:01:00.0」，核心层交来点它的那一组；`switch_to_the_cpu_first` 时先告知一张卡都不参与；切换只点没开的「0000:41:00.0」，
/// 头一次与 3 次重开都开不成；再交同一组。
fn readings_of_the_switch_not_opened_and_handed_in_again(
    switch_to_the_cpu_first: bool,
) -> ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            None,
            false,
        );
    let at_the_mount =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&at_the_mount);
    if switch_to_the_cpu_first {
        background_gpu.use_no_gpu_from_the_next_batch();
    }
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered_once_not_opened = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    let gpus_opened_once_not_opened = background_gpu.gpus_opened();
    let participating_once_not_opened = participating_devices_of(&background_gpu);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered_when_handed_in_again = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
    ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
        answered_once_not_opened,
        gpus_opened_once_not_opened,
        participating_once_not_opened,
        answered_when_handed_in_again,
        openings: openings_made_so_far(&cards_handed_to_the_opening),
    }
}

/// 先切到 `cpu` 的那一臂：开不成之后一张卡都没开、没参与（核心层生效的照旧是 `cpu`，不交活）；再交同一组，这张卡已连着试满，当场答开不成、不再开。
#[test]
fn a_switch_not_opened_after_the_cpu_leaves_no_card_open_and_the_same_settings_again_are_answered_not_opened_at_once(
) {
    let readings = readings_of_the_switch_not_opened_and_handed_in_again(true);
    assert!(matches!(
        readings.answered_once_not_opened,
        LatestSettingsHandedIn::NotOpened(_)
    ));
    assert!(readings.gpus_opened_once_not_opened.is_empty());
    assert!(readings.participating_once_not_opened.is_empty());
    assert_eq!(
        readings.answered_when_handed_in_again,
        not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
    );
    assert_eq!(readings.openings, 4);
}

/// 不先切到 `cpu` 的那一臂：开不成之后参与的照旧是切换之前那张「0000:01:00.0」，与核心层照旧生效的那一组是同一张；再交同一组当场答开不成、不再开。
#[test]
fn a_switch_not_opened_on_the_gpu_keeps_the_card_before_the_switch_taking_part() {
    let readings = readings_of_the_switch_not_opened_and_handed_in_again(false);
    assert!(matches!(
        readings.answered_once_not_opened,
        LatestSettingsHandedIn::NotOpened(_)
    ));
    assert_eq!(
        readings.gpus_opened_once_not_opened,
        vec![card_identifier("0000:01:00.0")]
    );
    assert_eq!(
        readings.participating_once_not_opened,
        vec![card_identifier("0000:01:00.0")],
        "参与的照旧是切换之前那一组"
    );
    assert_eq!(
        readings.answered_when_handed_in_again,
        not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
    );
    assert_eq!(readings.openings, 4);
}

/// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 20 秒），起卡那一件头一次 panic、之后开成替身卡；
/// 起卡那一件每被起一次往 `starts_made` 加一。
fn opening_panicking_at_its_first_start(
    starts_made: &Arc<std::sync::atomic::AtomicU32>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let starts_counted = Arc::clone(starts_made);
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
            let starts_counted_by_the_start = Arc::clone(&starts_counted);
            move || {
                let starts_before =
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                assert!(starts_before > 0, "替身：起卡那一件头一次 panic");
                Ok(card)
            }
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        OpeningsNotReturnedInThisProcess::with_none_not_returned(),
    )
}

/// 第 4 条「开卡线程 panic 都算这张卡这一次没开成，照重开」用于挂载那一刻：开一张卡那一步（产品那一份）头一次 panic——挂载照重开、第二次开成，
/// 不拒挂载，起卡那一件一共被起 2 次。
#[test]
fn an_opening_panicking_at_the_mount_is_reopened_and_the_mount_is_not_refused() {
    let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
    let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mut opening_at_the_mount = opening_panicking_at_its_first_start(&starts_made);
    let mount = gpus_opened_at_the_mount(
        vec![discrete_gpu.clone()],
        &[ParticipatingGpu {
            discrete_gpu,
            memory_limit: None,
        }],
        bytes_of_the_smallest_piece_every_participating_gpu_holds(),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        ReopeningsAndPciStateReadingOfAGpuNotOpened {
            reopenings: REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            pci_state_reader: &pci_state_reader_under_a_root_not_present_for_tests(),
        },
        |_interval: Duration| {},
        |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
            opening_at_the_mount(
                gpu_to_open,
                memory_limit_at_the_opening,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                cost_at_the_opening,
            )
        },
    );
    assert!(
        mount.is_ok(),
        "头一次 panic 照重开、第二次开成：{:?}",
        mount.err()
    );
    assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
}

/// 开卡那一步：「0000:41:00.0」第 3 次与第 6 次开成、别的次次开不成；别的卡每一次都开成。各次交来的卡记在交回的那一份里（另由外壳那一层记）。
fn opening_the_card_at_forty_one_only_the_third_and_sixth_times() -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let mut openings_of_the_card_at_forty_one: u32 = 0;
    move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
        if discrete_gpu.pci_bus_address == "0000:41:00.0" {
            openings_of_the_card_at_forty_one += 1;
            if openings_of_the_card_at_forty_one != 3 && openings_of_the_card_at_forty_one != 6 {
                return Err(reason_the_card_does_not_open_at_the_switch());
            }
        }
        opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
    }
}

/// 开卡那一步：「0000:41:00.0」次次开不成，对它的第 2 次先往 `second_opening_begun` 送一声、等用例放开（至多 20 秒）；别的卡每一次都开成。
fn opening_the_card_at_forty_one_never_and_holding_its_second_opening(
    second_opening_begun: std::sync::mpsc::Sender<()>,
    second_opening_released: std::sync::mpsc::Receiver<()>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let mut openings_of_the_card_at_forty_one: u32 = 0;
    move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
        if discrete_gpu.pci_bus_address == "0000:41:00.0" {
            openings_of_the_card_at_forty_one += 1;
            if openings_of_the_card_at_forty_one == 2 {
                // 收的那一头在用例里一直拿着。
                let _signal_sent = second_opening_begun.send(());
                let _released_or_not =
                    second_opening_released.recv_timeout(Duration::from_secs(20));
            }
            return Err(reason_the_card_does_not_open_at_the_switch());
        }
        opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
    }
}

/// 第 4 条「一张卡至多开 4 次」数的是一次挂载里这张卡连着开不成的次数，开成一次就清零：「0000:41:00.0」头一次切换开不成 2 次、第 3 次开成；
/// 切走关掉之后再点它，又开不成 2 次、这一次的第 3 次（对它是第 6 次）开成——前后两段的开不成不并成 4 次，不当试满，换上。
#[test]
fn a_card_opened_between_its_failed_openings_counts_its_failed_openings_afresh() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_the_card_at_forty_one_only_the_third_and_sixth_times(),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let back_to_the_card_at_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "第 3 次开成"
    );
    background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "切回「0000:01:00.0」：「0000:41:00.0」关掉"
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "开成过一次就清零：又开不成 2 次不算试满，第 3 次开成"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
    assert_eq!(
        openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
        6
    );
}

/// 连着开不成的次数按卡数、跨切换接着数：「0000:41:00.0」头一次切换开不成 2 次（第 2 次在开时告知一张卡都不参与，余下的叫停没开）；
/// 之后「0000:81:00.0」开成，不给「0000:41:00.0」清零；再点「0000:41:00.0」只剩 2 次可开、都开不成——答开不成、报这 2 次的原因；
/// 再点它不再开，答试满了 4 次，开卡那一步对它一共 4 次。
#[test]
fn the_failed_openings_of_a_card_in_a_row_add_up_across_the_switches_and_another_card_opening_does_not_clear_them(
) {
    let (second_opening_begun, every_second_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_second_opening, second_opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_the_card_at_forty_one_never_and_holding_its_second_opening(
                second_opening_begun,
                second_opening_released,
            ),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    every_second_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开始对「0000:41:00.0」的第 2 次开卡");
    background_gpu.use_no_gpu_from_the_next_batch();
    release_the_second_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(
        background_gpu
            .opening_of_a_gpu_at_a_switch
            .lock()
            .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "「0000:81:00.0」开成"
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            ),
            causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 2],
        }),
        "连着开不成 2 次之后只剩 2 次可开"
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
    );
    assert_eq!(
        openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
        4,
        "试满之后不再开"
    );
}
