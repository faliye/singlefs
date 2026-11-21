//! 两批之间换设置：交来新的一组、开切换点到的卡、换上与关掉不再点名的卡、切到 `cpu`。

use super::*;

/// 卡清单每次答之前现查本机在的独显——挂载之后那张卡掉了，不点名的切换照「GPU 不在」拒、点名它照「点名的卡不存在」拒。
#[test]
fn the_switch_after_the_card_dropped_off_the_host_is_refused_as_gpu_not_present() {
    let (background_gpu, present) = background_gpu_over_cards_on_the_cpu(
        vec![CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            None,
            0,
        )],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let settings_naming = |named: Option<Vec<GpuDeviceIdentifier>>| BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: named,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    assert!(
        admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu).is_ok(),
        "卡还在：照收"
    );
    present.lock().expect("用例没在别处 panic 过").clear();
    assert_eq!(
        admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu),
        Err(BackgroundComputeSettingsRefused::GpuNotPresent),
        "卡掉了、不点名：GPU 不在"
    );
    assert_eq!(
        admit_gpu_settings_against_the_inventory(
            &settings_naming(Some(vec![card_identifier("0000:01:00.0")])),
            &background_gpu
        ),
        Err(
            BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                device: card_identifier("0000:01:00.0")
            }
        ),
        "卡掉了、点名它：点名的卡不存在"
    );
}

/// 第 2 条「挂着时切换点到还没开的卡，那一刻再开那张卡」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换、校验那一刻不开卡；
/// 交设置之后开「0000:41:00.0」（只开这一张、只开一次），开完了在两批之间换上，两张都参与、按点名的次序；再交同一组不重开。
#[test]
fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect() {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_that_always_opens,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let openings_so_far = || {
        cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过")
            .clone()
    };
    let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
    assert!(openings_so_far().is_empty(), "校验那一刻不开卡");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "开成了：换上，这一组用得上"
    );
    assert_eq!(
        openings_so_far(),
        vec!["0000:41:00.0".to_string()],
        "只开挂载时没开的那一张"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0")
        ]
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0")
        ],
        "两张都参与、按点名的次序"
    );
    assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(openings_so_far().len(), 1, "开着的卡不重开");
}

/// 第 4 条「切换请求照收，在会话锁外开新卡；重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」：切换只点「0000:41:00.0」，
/// 开它那一步等用例放开才交回——交设置照常返回、不等开卡（核心层在会话锁里交设置），两批之间不等它的那一问交回 `None`，
/// 其间那一批照切换之前那一组只分给「0000:01:00.0」；放开之后换上，下一批只分给「0000:41:00.0」，不再点名的那张关掉。
#[test]
fn the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch(
) {
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                // 等放开有个上限：交设置要是在这一刻等开卡（会话锁里等），开卡到点交回开不成，用例照断言红，不一直挂着。
                opening_released
                    .recv_timeout(Duration::from_secs(20))
                    .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
                opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                )
            },
            ReopeningsOfAGpuNotOpened {
                interval: Duration::ZERO,
                reopenings_at_most: 0,
            },
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll),
        LatestSettingsHandedIn::StillOpeningItsCards,
        "还在开：不等它"
    );
    let (outcome_while_opening, devices_given_work_while_opening) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(
        outcome_while_opening.is_ok(),
        "开着的这段时间照常算（{outcome_while_opening:?}）"
    );
    assert_eq!(
        devices_given_work_while_opening,
        vec![card_identifier("0000:01:00.0")],
        "开着的这段时间照旧照切换之前那一组跑"
    );
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "开成了：换上，这一组用得上"
    );
    let (outcome_after_the_opening, devices_given_work_after_the_opening) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(
        outcome_after_the_opening.is_ok(),
        "换上之后照常算（{outcome_after_the_opening:?}）"
    );
    assert_eq!(
        devices_given_work_after_the_opening,
        vec![card_identifier("0000:41:00.0")],
        "开成了：下一批起换成新设置"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:41:00.0")],
        "不再点名的那张关掉"
    );
}

/// 第 4 条「间隔 1 分钟重开，最多重开 3 次」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——照常换上、两张都参与，
/// 开卡那一步一共只开它 3 次。
#[test]
fn the_card_not_opened_twice_at_the_switch_opens_the_third_time_and_the_switch_takes_effect() {
    let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let openings_counted = Arc::clone(&openings_made);
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                let opening =
                    openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if opening < 3 {
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
    let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "第三次开成：换上"
    );
    assert_eq!(
        *cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过"),
        vec!["0000:41:00.0".to_string(); 3],
        "开不成两次之后重开、第三次开成就不再开"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0")
        ],
        "换上了：两张都参与"
    );
    assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
}

/// 第 4 条「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
/// 两批之间那一问答开不成、带这张卡与 4 次的原因（再问照答，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
/// 照旧只开着切换之前那一张。下一次换设置（切回「0000:01:00.0」）解除它，那一批照常只分给它。
#[test]
fn the_switch_to_only_a_card_that_does_not_open_is_reported_as_its_gpu_failure_and_gives_it_no_work(
) {
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let not_opened = GpuNotOpenedAfterTheReopenings {
        device: card_identifier("0000:41:00.0"),
        pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
            "0000:41:00.0",
        ),
        causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 4],
    };
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::NotOpened(not_opened.clone()),
        "头一次与 3 次重开都开不成：报出这张卡与每一次的原因"
    );
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::NotOpened(not_opened.clone()),
        "再问照答开不成、不再开"
    );
    assert_eq!(
        *cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过"),
        vec!["0000:41:00.0".to_string(); 4],
        "开 4 次之后不自动再试"
    );
    assert_eq!(
        background_gpu.batch_not_split_over_the_gpus(),
        Some(
            BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(not_opened.clone())
        ),
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")],
        "开不成：照旧只开着切换之前那一张"
    );
    let (outcome, devices_given_work) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert_eq!(
        outcome,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:41:00.0"),
            cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                causes_of_each_opening: not_opened.causes_of_each_opening,
            },
            pci_state_when_judged_unusable: not_opened.pci_state_when_judged_unusable,
        }),
        "之后那一批报成这张卡的 GPU 出错"
    );
    assert!(
        devices_given_work.is_empty(),
        "一条活都没分出去（{devices_given_work:?}）"
    );
    let back_to_the_card_opened_at_the_mount =
        switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_opened_at_the_mount);
    assert_eq!(
        background_gpu.batch_not_split_over_the_gpus(),
        None,
        "下一次换设置解除它"
    );
    let (outcome_after_switching_back, devices_given_work_after_switching_back) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(outcome_after_switching_back.is_ok());
    assert_eq!(
        devices_given_work_after_switching_back,
        vec![card_identifier("0000:01:00.0")]
    );
}

/// 第 4 条「开卡重开用完都开不成」那一刻查一次、「这张卡再一次被判用不了时才再读一次」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
/// 判它开不成那一刻（会话锁外的开卡线程上）读 1 次；之后两批之间连问 3 次、连交 2 批，报的都是那一份，不再读。
#[test]
fn the_switch_not_opened_is_read_once_and_every_later_answer_and_batch_carries_that_reading() {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0");
    let answers: Vec<LatestSettingsHandedIn> = (0..3)
        .map(|_question| {
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            )
        })
        .collect();
    assert!(
        answers.iter().all(|answer| matches!(
            answer,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.pci_state_when_judged_unusable == pci_state_read
        )),
        "每一问都答开不成、带判它开不成那一刻读的那一份：{answers:?}"
    );
    let batch_outcomes: Vec<BothChecksumKernelOutcomesOfTheBatch> = (0..2)
        .map(|_batch| checksum_batch_and_the_devices_given_work(&mut background_gpu).0)
        .collect();
    assert!(
        batch_outcomes.iter().all(|outcome| matches!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                pci_state_when_judged_unusable,
                ..
            }) if *pci_state_when_judged_unusable == pci_state_read
        )),
        "之后每一批都报成这张卡的 GPU 出错、带同一份读数：{batch_outcomes:?}"
    );
    assert_eq!(background_gpu.pci_state_readings_taken(), 1);
}

/// 同一条「开卡每一次没开成而还要重开时都不查」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——换上了，一次都不读。
#[test]
fn the_card_opened_at_its_third_try_at_a_switch_reads_no_pci_state() {
    let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let openings_counted = Arc::clone(&openings_made);
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                let opening =
                    openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if opening < 3 {
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
    let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(background_gpu.pci_state_readings_taken(), 0);
}

/// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
/// 换上那一刻「0000:41:00.0」关掉（替身卡析构一次、不再开着），卡清单照列它；再切到两张都点，它重开（开卡那一步走一次），两张都参与。
#[test]
fn the_card_no_longer_named_by_a_switch_is_closed_and_reopened_when_named_again() {
    let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
        background_gpu_with_both_cards_opened_at_the_mount(
            opening_that_always_opens,
            |card| card,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let only_the_card_at_one =
        switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
    assert_eq!(
        drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "不再点名的那张关掉：设备丢掉"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")]
    );
    assert_eq!(
        background_gpu
            .devices_with_total_memory()
            .into_iter()
            .map(|(device, _total_memory)| device)
            .collect::<Vec<_>>(),
        vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0")
        ],
        "卡清单照列关掉的那张：切换可以再点它"
    );
    let both_cards = switch_naming_both_cards_admitted_against(&background_gpu);
    background_gpu.use_the_settings_from_the_next_batch(&both_cards);
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
        vec!["0000:41:00.0".to_string()],
        "再点它就重开"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0")
        ]
    );
}

/// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换时关掉的卡：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
/// 切换不再点它——它不析构，进程里的泄漏账在它上面加这一次的 600 字节；它照样不再开着。
#[test]
fn the_card_closed_at_a_switch_with_a_submission_still_running_is_leaked_and_noted_in_the_process()
{
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let (mut background_gpu, _cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
        background_gpu_with_both_cards_opened_at_the_mount(
            opening_that_always_opens,
            |card| card.with_a_submission_running_past_its_wait(600),
            &memory_leaked,
        );
    let only_the_card_at_one =
        switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
    assert_eq!(
        drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "提交还在跑：关掉的这张泄漏、不析构"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
        GpuMemoryInBytes(600),
        "泄漏账加的恰好是这一次占着的 600 字节"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")]
    );
}

/// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换开的卡：切换点挂载时没开的「0000:41:00.0」，开卡那一步是产品那一份
/// （`opening_of_a_card_within_the_limit`，起设备那一件由用例交、造一张替身卡），替身卡上记着一次等超过上限、还在跑的提交（占 600 字节），量代价次次出错——
/// 头一次与 3 次重开造的 4 张卡都不析构（替身的析构一次都没走），进程里的泄漏账在它上面每一次加 600 字节；切换按这张卡重开到上限都没开成报（每一次的原因
/// 都是量代价那一次的错，原样），它没算开成。
#[test]
fn the_card_opened_at_the_switch_whose_cost_measurement_fails_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
) {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let drops_counted = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let drops_counted_by_the_card = Arc::clone(&drops_counted);
    let opening_at_the_switch = opening_of_a_card_within_the_limit(
        WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit,
              _wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let mut card = CardComputedOnTheCpuForTests::of_the_address(
                &discrete_gpu.pci_bus_address,
                memory_limit.map(|memory_limit| memory_limit.0),
                bytes_leaked_by_earlier_mounts.0,
            )
            .with_a_submission_running_past_its_wait(600);
            card.measured_cost = None;
            card.cost_measurement_failing_with =
                Some(failure_of_the_cost_measurement_in_the_test());
            card.drops_counted = Arc::clone(&drops_counted_by_the_card);
            move || Ok(card)
        },
        memory_leaked.clone(),
        OpeningsNotReturnedInThisProcess::with_none_not_returned(),
    );
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_at_the_switch,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &memory_leaked,
        );
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            ),
            causes_of_each_opening: vec![failure_of_the_cost_measurement_in_the_test(); 4],
        }),
        "切换按这张卡重开到上限都没开成报，每一次的原因是量代价那一次的错"
    );
    assert_eq!(
        drops_counted.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "提交还在跑：切换开的这几张卡泄漏、不析构"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
        GpuMemoryInBytes(4 * 600),
        "泄漏账每一次加的恰好是那一次占着的 600 字节"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")],
        "量代价出错的那张没算开成"
    );
}

/// 开卡那一步：等用例放开（至多 20 秒，没放开就交回开不成，用例照断言红、不一直挂着）再开成交来的那张卡。
fn opening_that_opens_once_released(
    opening_released: std::sync::mpsc::Receiver<()>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
        opening_released
            .recv_timeout(Duration::from_secs(20))
            .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
        opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
    }
}

/// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻「0000:01:00.0」因显存上限不参与、
/// 参与的卡一张都不剩；切换只点没开的「0000:41:00.0」（不给上限）——开卡的这段时间那一问答还在开卡（不答切换之前那一组的「一张都不剩」），
/// 开完换上之后答换上了、这一组用得上，参与的只有「0000:41:00.0」。
#[test]
fn the_settings_whose_card_is_still_opening_are_answered_as_still_opening_and_as_usable_once_put_in_effect(
) {
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_opens_once_released(opening_released),
            Some(64),
            true,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    assert_eq!(
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll),
        LatestSettingsHandedIn::StillOpeningItsCards,
        "还在开卡：这一组还没生效，不答切换之前那一组用不用得上"
    );
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "开完换上：这一组用得上"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
}

/// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员」用于点没开的卡：切换只点没开的「0000:41:00.0」、给它 64 字节的上限
/// （连最小的一片都装不下）——开完换上那一问答换上了，连同这一组下一批分不到卡上（参与的卡因显存上限一张都不剩，晾下的是「0000:41:00.0」）。
#[test]
fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_is_answered_with_every_card_left_out(
) {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_always_opens,
            None,
            false,
        );
    let to_the_card_at_forty_one_under_a_tiny_limit = switch_naming_the_cards_admitted_against(
        &background_gpu,
        &["0000:41:00.0"],
        Some(vec![64]),
    );
    background_gpu
        .use_the_settings_from_the_next_batch(&to_the_card_at_forty_one_under_a_tiny_limit);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out)),
            } if left_out.iter().map(|gpu| gpu.device.clone()).collect::<Vec<_>>()
                == vec![card_identifier("0000:41:00.0")]
        ),
        "换上了，连同这一组一张卡都用不上：{answered:?}"
    );
}

/// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：切换点「0000:41:00.0」，它在开（等用例放开）；又来一次切换只点「0000:81:00.0」。放开之后
/// 「0000:41:00.0」开成，接着为最近那一组开「0000:81:00.0」，它头一次与 3 次重开都开不成——此刻生效那一组（挂载那一组）之外开着的卡全部关掉：
/// 为被盖掉的那一组开成的「0000:41:00.0」关掉（替身卡析构一次），开着的只剩「0000:01:00.0」。
#[test]
fn the_card_opened_for_a_superseded_switch_is_closed_when_the_latest_switch_does_not_open() {
    let (release_the_card_at_forty_one, card_at_forty_one_released) =
        std::sync::mpsc::channel::<()>();
    let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
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
                Err(reason_the_card_does_not_open_at_the_switch())
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
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.device == card_identifier("0000:81:00.0")
        ),
        "最近那一组点的「0000:81:00.0」开不成：{answered:?}"
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")],
        "此刻生效那一组之外开着的卡全部关掉"
    );
    assert_eq!(
        drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "为被盖掉的那一组开成的卡关掉了"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:01:00.0")]
    );
}

/// 第 4 条「一张卡至多开 4 次」在一次挂载里连着数（开成一次清零）：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、两批之间那一问还没问到时，
/// 同一组设置又交了一次——不再开它（开卡那一步一共只被调 4 次），那一问答开不成，原因只一个：这一次挂载里连着试满了 4 次。
#[test]
fn the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more() {
    let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
    let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let openings_counted = Arc::clone(&openings_made);
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                let opening =
                    openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if opening == 4 {
                    // 收的那一头还在：用例在等这一声。
                    let _signal_sent = opened_four_times.send(());
                }
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    every_opening_made
        .recv_timeout(Duration::from_secs(20))
        .expect("开卡线程 20 秒内开完 4 次");
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    assert_eq!(
        openings_made_so_far(&cards_handed_to_the_opening),
        4,
        "连着试满了：又交的那一次不再开"
    );
    assert_eq!(
        answered,
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            ),
            causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
                failed_openings: 4
            }],
        })
    );
}

/// 交来的一组点的卡都开着就当场换上，在开的那一次为更早一次交来的设置开成的卡收下时关掉：切换点没开的「0000:41:00.0」，它在开（等用例放开）；
/// 又交来只点开着的「0000:01:00.0」的一组——那一问当场答换上了；放开之后「0000:41:00.0」开成、收下，此刻生效那一组没点它，关掉。
#[test]
fn the_settings_handed_in_while_an_earlier_opening_runs_with_their_cards_open_take_effect_at_once_and_the_card_opened_later_is_closed(
) {
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_opens_once_released(opening_released),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    let back_to_the_card_at_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
    assert_eq!(
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        },
        "点的卡都开着：当场换上"
    );
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(release_the_opening);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(
        background_gpu.gpus_opened(),
        vec![card_identifier("0000:01:00.0")],
        "为更早一次交来的设置开成的卡收下时关掉"
    );
}

/// 上一次切换的卡还在开时又交来一组，等那一次开完再看，不为它另起一条：切换点没开的「0000:41:00.0」，它在开（等用例放开）；又交来点
/// 「0000:41:00.0」「0000:81:00.0」的一组——放开之后收下「0000:41:00.0」，再为最近那一组只开还没开的「0000:81:00.0」，开卡那一步按次序只被交这两张、各一次。
#[test]
fn the_settings_handed_in_while_an_earlier_opening_runs_wait_for_it_and_open_only_the_cards_still_not_opened(
) {
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let mut open_once_released = opening_that_opens_once_released(opening_released);
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                    return open_once_released(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    );
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
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    drop(release_the_opening);
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
        vec!["0000:41:00.0".to_string(), "0000:81:00.0".to_string()],
        "每张卡只开一次"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![
            card_identifier("0000:41:00.0"),
            card_identifier("0000:81:00.0")
        ]
    );
}

/// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载开着「0000:01:00.0」「0000:41:00.0」，告知一张卡都不参与——开着的卡为零（「0000:41:00.0」那张替身卡析构一次），
/// 卡清单照列两张；再交来只点「0000:41:00.0」的一组，它当没开的卡开（开卡那一步走一次），开完换上。
#[test]
fn no_gpu_taking_part_closes_every_open_card_and_the_next_settings_open_their_cards_again() {
    let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
        background_gpu_with_both_cards_opened_at_the_mount(
            opening_that_always_opens,
            |card| card,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    background_gpu.use_no_gpu_from_the_next_batch();
    assert!(
        background_gpu.gpus_opened().is_empty(),
        "开着的卡为零：{:?}",
        background_gpu.gpus_opened()
    );
    assert_eq!(
        drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert_eq!(
        background_gpu.devices_with_total_memory().len(),
        2,
        "卡清单照列关掉的卡：切回 gpu 可以再点"
    );
    let only_the_card_at_forty_one =
        switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_forty_one);
    assert_eq!(
        background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds
        ),
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None
        }
    );
    assert_eq!(
        openings_made_so_far(&cards_handed_to_the_opening),
        1,
        "切回之后当没开的卡开"
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
}

/// 产品那一份实现（两份核那一层是外壳）在两批之间那一问里不等开卡：切换点到的「0000:41:00.0」在开（等用例放开），核心层那一问照常交回、答还在开卡。
#[test]
fn the_product_implementation_asked_between_two_batches_does_not_wait_for_the_opening() {
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_that_opens_once_released(opening_released),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    let mut implementation =
            crate::background_gpu_compute::BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(
                background_gpu,
            );
    let answered = singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut implementation,
        );
    release_the_opening
        .send(())
        .expect("开卡那一步还在等放开：收的那一头还在");
    assert_eq!(
        answered,
        singlefs_core::background_compute::LatestGpuSettingsHandedIn::StillOpeningTheirCards
    );
}
