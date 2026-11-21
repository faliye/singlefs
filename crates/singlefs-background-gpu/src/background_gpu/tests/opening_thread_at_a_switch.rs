//! 切换时在另一条线程上开卡：起不了线程、开卡里 panic、重开的间隔、被更新的一组顶替与叫停。

use super::*;

/// 开卡那一步：每被调一次先往 `opening_begun` 送一声，等用例放开（至多 20 秒）之后交回开不成。
fn opening_that_does_not_open_once_released(
    opening_begun: std::sync::mpsc::Sender<()>,
    opening_released: std::sync::mpsc::Receiver<()>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
        // 收的那一头在用例里一直拿着。
        let _signal_sent = opening_begun.send(());
        let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
        Err(reason_the_card_does_not_open_at_the_switch())
    }
}

/// 第 2 条「从 gpu 切到 cpu……丢掉还在开的那一组」：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时告知一张卡都不参与——
/// 开卡线程开完这一次就不再开（重开规矩本来还许 3 次），开卡那一步一共只被调一次；开着的卡为零。
#[test]
fn no_gpu_taking_part_while_a_card_opens_calls_off_the_reopenings() {
    let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_does_not_open_once_released(opening_begun, opening_released),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    every_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开始头一次开卡");
    background_gpu.use_no_gpu_from_the_next_batch();
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(release_the_opening);
    // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
    drop(
        background_gpu
            .opening_of_a_gpu_at_a_switch
            .lock()
            .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
    );
    assert_eq!(
        openings_made_so_far(&cards_handed_to_the_opening),
        1,
        "叫停之后不再开"
    );
    assert!(background_gpu.gpus_opened().is_empty(), "开着的卡为零");
}

/// 卸载时叫停切换开卡线程：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时丢掉后台 GPU——开卡线程开完这一次就不再开，开卡那一步一共只被调一次。
#[test]
fn the_opening_at_a_switch_under_way_at_the_drop_opens_no_more() {
    let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_does_not_open_once_released(opening_begun, opening_released),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    every_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开始头一次开卡");
    let opening_step_kept_by_the_test = Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
    drop(background_gpu);
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(release_the_opening);
    // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
    drop(
        opening_step_kept_by_the_test
            .lock()
            .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
    );
    assert_eq!(
        openings_made_so_far(&cards_handed_to_the_opening),
        1,
        "卸载之后不再开"
    );
}

/// 起不了开卡线程也照重开的规矩（间隔之后再起，至多重开 3 次）：头一次起不了，两批之间那一问到了间隔再起、开成——照常换上，参与的只有「0000:41:00.0」。
#[test]
fn the_opening_thread_not_started_once_is_started_again_and_the_switch_takes_effect() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_always_opens,
            None,
            false,
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "再起一次就开成了"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
}

/// 起不了开卡线程次次都起不了：头一次与 3 次再起都不成，那一问答开不成，带头一张要开的卡与 4 个原因（1 + 重开次数，每次一个），开卡那一步一次都没被调。
#[test]
fn the_opening_thread_never_started_is_reported_with_one_cause_for_each_start() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_always_opens,
            None,
            false,
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(u32::MAX);
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.device == card_identifier("0000:41:00.0")
                    && not_opened.causes_of_each_opening.len() == 4
        ),
        "1 + 重开次数个原因：{answered:?}"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
}

/// 开卡里 panic 了算那一次没开成、照重开：切换点到的「0000:41:00.0」头一次开卡 panic、第二次开成——照常换上，开卡那一步一共被调 2 次。
#[test]
fn an_opening_panicking_once_is_reopened_and_the_switch_takes_effect() {
    let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let openings_counted = Arc::clone(&openings_made);
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                let opening =
                    openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                assert!(opening > 1, "替身：头一次开卡 panic");
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
}

/// 开卡次次 panic：重开到上限之后那一问答开不成，带 4 个原因，每个都带着 panic 的原话；两批之间那一问不再 panic。
#[test]
fn an_opening_panicking_every_time_is_reported_after_the_reopenings_with_every_cause() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                panic!("替身：开卡次次 panic")
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert_eq!(
        answered,
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            ),
            causes_of_each_opening: vec![
                GpuFailureCause::OpeningPanicked {
                    said_by_the_panic: "替身：开卡次次 panic".to_string(),
                };
                4
            ],
        })
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
}

/// 同一张卡前一次开卡没回来就不再起下一次（算这一次又没开成、照间隔往下数）：切换点到的「0000:41:00.0」开卡那一步是产品那一份
/// （`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒），起这张卡那一件挂着直到用例放开——头一次到点算没开成，之后 3 次重开都因前一次没回来不起，
/// 起卡那一件一共只被起一次（同一张卡上同时在开的线程至多一条），那一问答开不成、4 个原因。
#[test]
fn an_earlier_opening_of_the_card_not_returned_keeps_the_reopenings_from_starting_another() {
    const OPENING_LIMIT_OF_THE_TEST: Duration = Duration::from_millis(50);
    let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
    let start_released = Arc::new(Mutex::new(start_released));
    let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let starts_counted = Arc::clone(&starts_made);
    let opening_at_the_switch = opening_of_a_card_within_the_limit(
        OPENING_LIMIT_OF_THE_TEST,
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
            move || {
                starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _released_or_not = start_released_seen_by_the_start
                    .lock()
                    .expect("用例没在别处 panic 过")
                    .recv_timeout(Duration::from_secs(20));
                Ok(card)
            }
        },
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        OpeningsNotReturnedInThisProcess::with_none_not_returned(),
    );
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_at_the_switch,
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    let starts_while_the_first_hangs = starts_made.load(std::sync::atomic::Ordering::SeqCst);
    release_the_start
        .send(())
        .expect("起卡那一件还在等放开：收的那一头还在");
    assert_eq!(
        starts_while_the_first_hangs, 1,
        "前一次没回来：同一张卡上不再起"
    );
    assert_eq!(
        answered,
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            ),
            causes_of_each_opening: vec![
                GpuFailureCause::WaitForTheGpuExceededTheLimit {
                    wait_limit: OPENING_LIMIT_OF_THE_TEST,
                },
                opening_not_started_while_an_earlier_one_has_not_returned(),
                opening_not_started_while_an_earlier_one_has_not_returned(),
                opening_not_started_while_an_earlier_one_has_not_returned(),
            ],
        })
    );
}

/// 运行库收的那一份读得到切换关卡泄漏了哪几张：产品开卡器开成之后换成运行库收的样子（`opened_background_gpu_for_the_mount_runtime`），
/// 「0000:41:00.0」上记着一次还在跑的提交（600 字节），核心层挂载那一组只点「0000:01:00.0」——那一刻关掉「0000:41:00.0」、整张泄漏，
/// 运行库那一份读出它与 600 字节；卸载（丢掉把手）之后读得到丢掉那一刻泄漏的（一张都没有）。
#[test]
fn the_leaks_at_a_switch_are_read_through_what_the_product_opener_hands_to_the_mount_runtime() {
    let (background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
        background_gpu_with_both_cards_opened_at_the_mount(
            opening_that_always_opens,
            |card| card.with_a_submission_running_past_its_wait(600),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let only_the_card_at_one = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_identifier("0000:01:00.0")]),
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    let opened = crate::gpu_opener_of_this_process::opened_background_gpu_for_the_mount_runtime(
        crate::background_gpu_compute::background_gpu_compute_of_the_opened_gpu(background_gpu),
    );
    let gpus_leaked = opened
        .gpus_leaked
        .clone()
        .expect("产品开卡器恒给读泄漏的那一份");
    let background_compute =
            singlefs_core::background_compute::BackgroundComputeOfThisMount::of_a_mount(
                singlefs_core::background_compute::admit_background_compute_settings_with_the_gpu_compute(
                    only_the_card_at_one,
                    Some(opened.gpu_compute),
                )
                .expect("卡清单列着「0000:01:00.0」"),
            );
    assert_eq!(
        gpus_leaked.gpus_leaked_when_closed_at_a_switch(),
        vec![singlefs_mount_runtime::GpuLeakedByTheBackgroundGpu {
            device: card_identifier("0000:41:00.0"),
            memory_held_on_the_device: GpuMemoryInBytes(600),
        }]
    );
    assert_eq!(gpus_leaked.gpus_leaked_when_dropped(), None, "还没丢");
    drop(background_compute);
    assert_eq!(
        gpus_leaked.gpus_leaked_when_dropped(),
        Some(Vec::new()),
        "丢掉之后读得到：开着的那张没有还在跑的提交，一张都没泄漏"
    );
}

/// 起不了开卡线程、到了间隔而之后又交来了别的一组：照最近那一组起，不为被盖掉的那一组起——开卡那一步只被交「0000:81:00.0」，换上的是它。
#[test]
fn the_opening_thread_not_started_for_a_superseded_switch_is_started_for_the_latest_settings() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_always_opens,
            None,
            false,
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(
        *cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过"),
        vec!["0000:81:00.0".to_string()],
        "只为最近那一组开"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:81:00.0")]
    );
}

/// 收下为被盖掉的切换开成的卡、最近那一组还要开卡而没点它：收下那一刻就关掉，不开着等最近那一组开完——切换点「0000:41:00.0」（等用例放开），
/// 又来一次只点「0000:81:00.0」；放开之后「0000:41:00.0」开成、收下，为「0000:81:00.0」开卡那一刻「0000:41:00.0」已析构一次。
#[test]
fn the_card_opened_for_a_superseded_switch_is_closed_as_it_is_taken_in_while_the_latest_switch_still_opens(
) {
    let (release_the_card_at_forty_one, card_at_forty_one_released) =
        std::sync::mpsc::channel::<()>();
    let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
    let drops_seen_when_the_latest_card_opens = Arc::new(Mutex::new(None));
    let drops_seen_by_the_opening = Arc::clone(&drops_seen_when_the_latest_card_opens);
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                    card_at_forty_one_released
                        .recv_timeout(Duration::from_secs(20))
                        .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
                    let mut card = opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )?;
                    card.drops_counted = Arc::clone(&drops_noted_by_the_opening);
                    return Ok(card);
                }
                *drops_seen_by_the_opening
                    .lock()
                    .expect("用例没在别处 panic 过") =
                    Some(drops_noted_by_the_opening.load(std::sync::atomic::Ordering::SeqCst));
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
    release_the_card_at_forty_one
        .send(())
        .expect("开「0000:41:00.0」那一步还在等放开");
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(
        *drops_seen_when_the_latest_card_opens
            .lock()
            .expect("用例没在别处 panic 过"),
        Some(1),
        "为「0000:81:00.0」开卡那一刻，为被盖掉的那一组开成的卡已经关掉"
    );
}

/// 最近那一组点了两张没开的卡，前一张开成、后一张开不成：这一组没生效，为它开成的那一张关掉，开着的只剩此刻生效那一组的「0000:01:00.0」。
#[test]
fn the_card_opened_for_the_latest_switch_is_closed_when_another_card_of_it_does_not_open() {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                    return opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    );
                }
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            None,
            false,
        );
    let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
        &background_gpu,
        &["0000:41:00.0", "0000:81:00.0"],
        None,
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.device == card_identifier("0000:81:00.0")
        ),
        "{answered:?}"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")],
        "为没生效的那一组开成的「0000:41:00.0」关掉"
    );
}

/// 重开的间隔照产品是 1 分钟（用例不真等它：只用来看「该等的没等、不该等的等了」）。
const REOPENINGS_ONE_MINUTE_APART: ReopeningsOfAGpuNotOpened = ReopeningsOfAGpuNotOpened {
    interval: Duration::from_secs(60),
    reopenings_at_most: 3,
};

/// 起不了开卡线程那一回记在头一张要开的卡上：头一回起不了，之后这张卡次次开不成——连同那一回至多 4 次，开卡那一步只被调 3 次，
/// 那一问答开不成，原因是起不了线程那一个在前、3 次开不成在后。
#[test]
fn the_opening_thread_not_started_once_counts_toward_the_openings_of_the_first_card() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.causes_of_each_opening.len() == 4
                    && matches!(
                        not_opened.causes_of_each_opening[0],
                        GpuFailureCause::OpeningThreadNotStarted { .. }
                    )
        ),
        "{answered:?}"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 3);
}

/// 叫停之后不再睡重开的间隔：间隔是 1 分钟，头一次开卡在开（等用例放开）时告知一张卡都不参与——放开之后开卡线程不睡那 1 分钟，20 秒之内就开完、放开那一步。
#[test]
fn no_gpu_taking_part_while_a_card_opens_does_not_sleep_out_the_interval_before_a_reopening() {
    let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_that_does_not_open_once_released(opening_begun, opening_released),
            REOPENINGS_ONE_MINUTE_APART,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    every_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开始头一次开卡");
    background_gpu.use_no_gpu_from_the_next_batch();
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(release_the_opening);
    let opening_step = Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
    let opening_thread_done_within_twenty_seconds = (0..2000).any(|_poll| {
        if opening_step.try_lock().is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
        false
    });
    assert!(
        opening_thread_done_within_twenty_seconds,
        "叫停之后不睡 1 分钟的间隔"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
}

/// 起不了开卡线程之后到了间隔才再起：间隔是 1 分钟，头一回起不了，两批之间那一问当场问——不再起，照旧是那一次起不了，开卡那一步一次都没被调。
#[test]
fn the_opening_thread_not_started_is_not_started_again_before_the_interval() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_that_always_opens,
            REOPENINGS_ONE_MINUTE_APART,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll),
        LatestSettingsHandedIn::StillOpeningItsCards
    );
    assert!(
        matches!(
            background_gpu.opening_at_a_switch,
            Some(OpeningAtASwitch::ThreadNotStarted { .. })
        ),
        "没到间隔：照旧是那一次起不了"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
}

/// 告知一张卡都不参与时生效那一组点的卡清掉：之后为被盖掉的一组开成的卡（「0000:01:00.0」），在最近那一组（只点「0000:81:00.0」）开不成时关掉——
/// 开着的卡为零。
#[test]
fn no_gpu_taking_part_forgets_the_cards_of_the_settings_in_effect_so_a_card_opened_later_is_closed()
{
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_the_card_at_one_once_released_and_no_other(opening_released),
            None,
            false,
        );
    background_gpu.use_no_gpu_from_the_next_batch();
    let to_the_card_at_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_one);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(&answered, LatestSettingsHandedIn::NotOpened(_)),
        "{answered:?}"
    );
    assert!(
        background_gpu.gpus_opened().is_empty(),
        "开着的卡为零：{:?}",
        background_gpu.gpus_opened()
    );
}

/// 告知一张卡都不参与时因显存上限不用的卡一并清掉：挂载那一刻「0000:01:00.0」因显存上限不参与，切到 `cpu` 之后不再报它。
#[test]
fn no_gpu_taking_part_forgets_the_cards_left_out_under_their_memory_limit() {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_always_opens,
            Some(64),
            true,
        );
    assert!(!background_gpu
        .gpus_left_out_under_their_memory_limit()
        .is_empty());
    background_gpu.use_no_gpu_from_the_next_batch();
    assert!(background_gpu
        .gpus_left_out_under_their_memory_limit()
        .is_empty());
}

/// 连着试满的卡在交设置那一刻就答开不成，不为它起开卡线程：「0000:41:00.0」开了 4 次都没开成之后，再交只点它的一组（这一次起线程会被拒）——
/// 那一问当场答连着试满、没有在开的那一次，开卡那一步照旧只被调 4 次。
#[test]
fn the_card_whose_openings_are_used_up_is_answered_not_opened_at_the_handing_in_without_starting_a_thread(
) {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    let _not_opened_after_four_openings = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert!(
        background_gpu.opening_at_a_switch.is_none(),
        "不为连着试满的卡起开卡"
    );
    assert_eq!(
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll),
        not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
}

/// 叫停之后没开的那几次不算进连着开不成的次数：「0000:41:00.0」头一次开卡在开（等用例放开）时告知一张卡都不参与，放开之后头一次开不成、
/// 余下 3 次叫停没开（这 3 次照样交记账那一步，算进去就连同头一次满 4 次）；再交只点它的一组，它照开（这一次开成）——不当试满。
#[test]
fn openings_called_off_do_not_count_toward_the_openings_of_the_card() {
    let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let mut is_the_first_opening = true;
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if is_the_first_opening {
                    is_the_first_opening = false;
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = opening_begun.send(());
                    let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
                    return Err(reason_the_card_does_not_open_at_the_switch());
                }
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    every_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开始头一次开卡");
    background_gpu.use_no_gpu_from_the_next_batch();
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(
        background_gpu
            .opening_of_a_gpu_at_a_switch
            .lock()
            .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
    );
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "叫停的那几次不算：照开、开成"
    );
    assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
}

/// 开卡的结局按交设置的次序认：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次只点「0000:81:00.0」——
/// 「0000:41:00.0」重开到上限都没开成的结局不报给最近那一组，为最近那一组开「0000:81:00.0」、开成、换上。
#[test]
fn the_failure_of_an_opening_for_a_superseded_switch_is_not_reported_to_the_latest_switch() {
    let (release_the_card_at_forty_one, card_at_forty_one_released) =
        std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                    let _released_or_not =
                        card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
                    return Err(reason_the_card_does_not_open_at_the_switch());
                }
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
    release_the_card_at_forty_one
        .send(())
        .expect("开「0000:41:00.0」那一步还在等放开");
    drop(release_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "为被盖掉的那一组开不成的不报给最近那一组"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:81:00.0")]
    );
    assert_eq!(
        background_gpu.pci_state_readings_taken(),
        1,
        "为被盖掉的那一组重开确实用完了（没叫停）：判它开不成那一刻照读一次"
    );
}

/// 最近那一组点的卡被被盖掉的那一次开卡连着试满：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次点它与「0000:81:00.0」的一组——
/// 为被盖掉的那一组开了 4 次之后，为最近那一组不再开「0000:41:00.0」，当场答连着试满；开卡那一步对它一共只被调 4 次，「0000:81:00.0」一次都没开。
#[test]
fn the_latest_switch_naming_a_card_used_up_by_a_superseded_switch_is_answered_not_opened() {
    let (release_the_card_at_forty_one, card_at_forty_one_released) =
        std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                    let _released_or_not =
                        card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
                    return Err(reason_the_card_does_not_open_at_the_switch());
                }
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
        &background_gpu,
        &["0000:41:00.0", "0000:81:00.0"],
        None,
    );
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
    release_the_card_at_forty_one
        .send(())
        .expect("开「0000:41:00.0」那一步还在等放开");
    drop(release_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
    );
    assert_eq!(
        *cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过"),
        vec!["0000:41:00.0".to_string(); 4]
    );
}
