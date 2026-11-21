//! 判一张卡用不了那一刻读一次它的 PCI 状态，带进这一批与之后各批的出错；没判用不了的不读。

use super::*;

/// 外壳装上一张「0000:01:00.0」替身卡（照 `card_as_the_test_wants` 改过），交一批一条 1000 字节的单元，交回这一批的结局与这一次挂载一共读了几次 PCI 状态。
fn checksum_batch_on_the_card_at_one_and_the_pci_state_readings(
    card_as_the_test_wants: impl FnOnce(CardComputedOnTheCpuForTests) -> CardComputedOnTheCpuForTests,
) -> (BothChecksumKernelOutcomesOfTheBatch, u64) {
    let card = card_as_the_test_wants(CardComputedOnTheCpuForTests::of_the_address(
        "0000:01:00.0",
        None,
        0,
    ));
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(1, 1000);
    let outcome = background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
    (outcome, background_gpu.pci_state_readings_taken())
}

/// 第 4 条「挂着时一批在同一张卡上等超时、到判那张卡用不了」那一刻查一次这张卡的硬件与链路：一批 1 片在这张卡上 4 次都等超过上限——
/// 报这张卡用不了，带判它用不了那一刻照它的地址读的 PCI 状态；3 次超时重交之前都不读，一共只读 1 次。
#[test]
fn the_card_unusable_after_its_resubmissions_is_read_once_and_the_batch_failure_carries_the_reading(
) {
    let (outcome, pci_state_readings) =
        checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
            card.submissions_timing_out = VecDeque::from([true, true, true, true]);
            card
        });
    assert_eq!(
        outcome,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 3,
                last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                }),
            },
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:01:00.0"
            ),
        })
    );
    assert_eq!(pci_state_readings, 1);
}

/// 同一条「平常的批、每次超时重交之前不查」：一批 1 片超时 3 次、第 4 次交做完——照常交回，一次都不读。
#[test]
fn the_batch_completing_after_three_timeouts_reads_no_pci_state() {
    let (outcome, pci_state_readings) =
        checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
            card.submissions_timing_out = VecDeque::from([true, true, true]);
            card
        });
    assert!(outcome.is_ok(), "第 4 次交做完：照常交回（{outcome:?}）");
    assert_eq!(pci_state_readings, 0);
}

/// 第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」那一刻查一次：这一批交到卡上时设备丢了——报这张卡的 GPU 出错，
/// 带那一刻读的 PCI 状态，一共只读 1 次。
#[test]
fn the_card_lost_in_a_batch_is_read_once_and_the_batch_failure_carries_the_reading() {
    let (outcome, pci_state_readings) =
        checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
            card.next_submission_failing_with =
                Some(GpuFailureCause::UncapturedErrorOrDeviceLost {
                    reason: "替身：设备丢了".to_string(),
                });
            card
        });
    assert_eq!(
        outcome,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::UncapturedErrorOrDeviceLost {
                reason: "替身：设备丢了".to_string(),
            },
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:01:00.0"
            ),
        })
    );
    assert_eq!(pci_state_readings, 1);
}

/// 同一条：一批里等提交没等成（不是设备丢了、不是超时用完）不是判卡用不了的那三种时刻——照报这张卡的 GPU 出错，读数那一格是没读，一次都不读。
#[test]
fn a_batch_failing_without_judging_the_card_unusable_reads_no_pci_state() {
    let (outcome, pci_state_readings) =
        checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
            card.next_submission_failing_with = Some(GpuFailureCause::WaitForTheSubmissionFailed {
                reason: "替身：没等成".to_string(),
            });
            card
        });
    assert_eq!(
        outcome,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::WaitForTheSubmissionFailed {
                reason: "替身：没等成".to_string(),
            },
            pci_state_when_judged_unusable:
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
        })
    );
    assert_eq!(pci_state_readings, 0);
}

/// 第 4 条「等超时用完到判那张卡用不了」也算交活之前那一等：卡上记着前一次超时、还在跑的提交，这一批交活之前 4 次都等满上限没等到——
/// 报这张卡用不了（一次都没重交到卡上），带那一刻读的 PCI 状态，一共只读 1 次。
#[test]
fn the_card_unusable_while_waiting_before_the_batch_is_read_once_and_the_failure_carries_the_reading(
) {
    let (outcome, pci_state_readings) =
        checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|card| {
            let mut card = card.with_a_submission_running_past_its_wait(100);
            card.earlier_submission_waits_timing_out = VecDeque::from([true, true, true, true]);
            card
        });
    let Err(ChecksumBatchFailed::GpuFailed {
        cause: GpuFailureCause::UnusableAfterResubmissions { .. },
        pci_state_when_judged_unusable,
        ..
    }) = &outcome
    else {
        panic!("交活之前 4 次都没等到：报这张卡用不了，结局是 {outcome:?}");
    };
    assert_eq!(
        pci_state_when_judged_unusable,
        &pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0")
    );
    assert_eq!(pci_state_readings, 1);
}

/// 第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」用于换设置时量代价：「0000:01:00.0」开卡时没量过代价，换设置让它参与时
/// 量代价那一次设备丢了——那一刻读 1 次 PCI 状态，读数跟着「换设置时量代价出错」存下；之后两批之间连问 3 次、连交 2 批，带的都是那一份，不再读。
#[test]
fn the_card_lost_while_its_cost_is_measured_at_a_switch_is_read_once_and_every_later_batch_carries_that_reading(
) {
    let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
        reason: "替身：量代价时设备丢了".to_string(),
    };
    let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card_at_one.measured_cost = None;
    card_at_one.cost_measurement_failing_with = Some(card_lost.clone());
    let card_at_forty_one = CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card_at_one, card_at_forty_one],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let both_cards = settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&both_cards);
    let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
    let not_split = BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
        device: card_identifier("0000:01:00.0"),
        cause: card_lost.clone(),
        pci_state_when_judged_unusable: pci_state_read.clone(),
    };
    for _question in 0..3 {
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: Some(not_split.clone())
            }
        );
    }
    for _batch in 0..2 {
        let (outcome, _devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert_eq!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: card_lost.clone(),
                pci_state_when_judged_unusable: pci_state_read.clone(),
            })
        );
    }
    assert_eq!(background_gpu.pci_state_readings_taken(), 1);
}

/// 卸载或切到 `cpu` 叫停了开卡线程之后不读：「0000:41:00.0」头一次开卡在开（等用例放开）时告知一张卡都不参与，放开之后头一次开不成、余下 3 次叫停没开——
/// 线程交回开不成之前看见叫停，不读，读次数是 0。
#[test]
fn the_opening_called_off_reads_no_pci_state() {
    let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                // 收的那一头在用例里一直拿着。
                let _signal_sent = opening_begun.send(());
                let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
                Err(reason_the_card_does_not_open_at_the_switch())
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
    assert_eq!(background_gpu.pci_state_readings_taken(), 0);
}

/// 第 4 条「开卡重开用完都开不成」用于起不了开卡线程：头一次与 3 次再起都起不了，次数用完——判头一张要开的卡开不成那一刻读 1 次，那一问答的开不成带着那一份。
#[test]
fn the_opening_thread_never_started_reads_the_pci_state_once_its_openings_are_used_up() {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_that_always_opens,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    background_gpu.refuse_to_start_the_opening_thread_for_tests(u32::MAX);
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    let answered = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
    let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0");
    assert!(
        matches!(
            &answered,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.pci_state_when_judged_unusable == pci_state_read
                    && not_opened.causes_of_each_opening.len() == 4
        ),
        "{answered:?}"
    );
    assert_eq!(background_gpu.pci_state_readings_taken(), 1);
}

/// 开卡线程上开到「0000:41:00.0」这一刻它已试满（连着开不成 4 次）：不开，判它开不成那一刻读 1 次，开不成的那一份带着它；
/// 同一刻已叫停的，交回开不成之前看见叫停、不读，读数是没读。
#[test]
fn the_card_used_up_when_the_opening_thread_reaches_it_is_read_once_unless_called_off() {
    let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
    let outcome_and_readings = |is_called_off: bool| {
        let failed_openings_in_this_mount = FailedOpeningsOfEachGpuInThisMount::default();
        for _failed_opening in 0..REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS.openings_at_most() {
            failed_openings_in_this_mount.note_a_failed_opening(&discrete_gpu.identifier);
        }
        let pci_state_reader = pci_state_reader_under_a_root_not_present_for_tests();
        let outcome = gpus_opened_at_a_switch(
            &mut no_opening_at_a_switch(),
            vec![(discrete_gpu.clone(), None)],
            WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            Vec::new(),
            &AtomicBool::new(is_called_off),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            &failed_openings_in_this_mount,
            &pci_state_reader,
        );
        (
            outcome
                .not_opened
                .map(|not_opened| not_opened.pci_state_when_judged_unusable),
            pci_state_reader.readings_taken(),
        )
    };
    assert_eq!(
        outcome_and_readings(false),
        (
            Some(pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0"
            )),
            1
        )
    );
    assert_eq!(
        outcome_and_readings(true),
        (
            Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError),
            0
        )
    );
}

/// 区域扫描与选段打分两段一批出错时，判卡用不了那一刻读的那一份照带进各段的出错（`region_record_batch_failed_of`、`live_slot_batch_failed_of`）。
#[test]
fn the_pci_state_read_is_carried_into_the_region_record_and_live_slot_batch_failures() {
    let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
        reason: "替身：设备丢了".to_string(),
    };
    let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
    let not_computed = || OutcomesOfTheBatchNotComputed::GpuFailed {
        device: card_identifier("0000:01:00.0"),
        cause: card_lost.clone(),
        pci_state_when_judged_unusable: pci_state_read.clone(),
    };
    assert_eq!(
        region_record_batch_failed_of(not_computed()),
        RegionRecordBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost.clone(),
            pci_state_when_judged_unusable: pci_state_read.clone(),
        }
    );
    assert_eq!(
        live_slot_batch_failed_of(not_computed()),
        LiveSlotBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost.clone(),
            pci_state_when_judged_unusable: pci_state_read.clone(),
        }
    );
}

/// 第 4 条「这张卡再一次被判用不了时才再读一次」：切换点「0000:41:00.0」，4 次都开不成——读 1 次（那一刻它的设备目录不在）；
/// 之后连问 3 次、连交 2 批都带那一份，读次数不变。之后它的设备目录出现、使能位关着，再交同一组（它已试满，当场再判一次开不成）——
/// 这一次读出新的那一份（目录在、使能位关），读次数变成 2，之后的问与批带的都是新的那一份，读次数照旧是 2。
#[test]
fn the_card_judged_unusable_again_is_read_again_and_no_new_judgement_reads_nothing_more() {
    let directories =
        crate::pci_state_of_the_gpu::PciDeviceDirectoriesMadeByTheTest::named_after_the_test(
            "judged-again",
        );
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount_read_by(
            |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                Err(reason_the_card_does_not_open_at_the_switch())
            },
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            directories.reader(),
        );
    let reading_of_the_card_now =
        |directories_now: &crate::pci_state_of_the_gpu::PciDeviceDirectoriesMadeByTheTest| {
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(
                crate::pci_state_of_the_gpu::pci_state_under_the_device_directory(
                    "0000:41:00.0",
                    directories_now.device_directory_of("0000:41:00.0"),
                ),
            ))
        };
    let answers_and_batches_carry =
        |background_gpu_asked: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
         pci_state_read: &PciStateOfTheGpuReadWhenJudgedUnusable| {
            (0..3).all(|_question| {
                matches!(
                    background_gpu_asked.put_the_gpus_opened_since_the_switch_in_effect(
                        WaitForTheOpeningAtASwitch::UntilItEnds
                    ),
                    LatestSettingsHandedIn::NotOpened(not_opened)
                        if not_opened.pci_state_when_judged_unusable == *pci_state_read
                )
            }) && (0..2).all(|_batch| {
                matches!(
                    checksum_batch_and_the_devices_given_work(background_gpu_asked).0,
                    Err(ChecksumBatchFailed::GpuFailed {
                        pci_state_when_judged_unusable,
                        ..
                    }) if pci_state_when_judged_unusable == *pci_state_read
                )
            })
        };
    let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    let read_with_the_directory_not_present = reading_of_the_card_now(&directories);
    assert!(answers_and_batches_carry(
        &mut background_gpu,
        &read_with_the_directory_not_present
    ));
    assert_eq!(background_gpu.pci_state_readings_taken(), 1);
    let directories = directories.with_the_file("0000:41:00.0", "enable", "0\n");
    let read_with_the_device_disabled = reading_of_the_card_now(&directories);
    assert_ne!(
        read_with_the_device_disabled, read_with_the_directory_not_present,
        "前提：两次读到的不同"
    );
    background_gpu.use_the_settings_from_the_next_batch(&admitted);
    assert!(answers_and_batches_carry(
        &mut background_gpu,
        &read_with_the_device_disabled
    ));
    assert_eq!(background_gpu.pci_state_readings_taken(), 2);
}
