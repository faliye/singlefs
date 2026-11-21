//! 调用方在停的标记：置上之后等一批的结果当场放弃，那次提交留在账上，丢的时候那张卡泄漏。

use super::*;

/// 一张「0000:01:00.0」替身卡交成一段之后那一次提交一直不做完（`submissions_never_completing`），装进外壳；交回外壳与这张卡析构了几次。
fn background_gpu_whose_card_never_completes_a_submission(
    memory_leaked: &GpuMemoryLeakedInThisProcess,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<std::sync::atomic::AtomicU32>,
) {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submissions_never_completing = true;
    let drops_of_the_card = Arc::clone(&card.drops_counted);
    let (background_gpu, _present) =
        background_gpu_over_cards_on_the_cpu(vec![card], memory_leaked);
    (background_gpu, drops_of_the_card)
}

/// 一条 1000 字节的单元那一段校验和在卡上合占的字节（替身照产品同一个口径记账）。
fn bytes_held_by_the_checksum_chunk_of_one_unit_of_one_thousand_bytes() -> GpuMemoryInBytes {
    CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
        InputLengthOnTheHostInBytes::of_this_many_bytes(1000),
        EntryCountOfAChunk::of_this_many_entries(1),
    )
}

/// 第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」：外壳收下了调用方在停的标记、标记已置，
/// 一批交给这张卡之后那一次提交一直不做完——等它的那一处醒来就放弃：这一批交回放弃（不重交、不判这张卡用不了、不读 PCI 状态），
/// 那一次提交留在这张卡的账上；丢后台 GPU 时这张卡整张泄漏、不析构，占着的字节记进进程的泄漏账。
#[test]
fn the_batch_whose_wait_is_given_up_for_the_caller_stopping_leaves_the_submission_on_the_card_leaked_when_dropped(
) {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let (mut background_gpu, drops_of_the_card) =
        background_gpu_whose_card_never_completes_a_submission(&memory_leaked);
    let (caller_is_stopping, handle_setting_the_flag) =
        caller_stopping_flag_and_the_handle_setting_it();
    background_gpu.give_up_waiting_for_the_cards_once_the_caller_is_stopping(caller_is_stopping);
    handle_setting_the_flag.tell_the_implementation_the_caller_is_stopping();
    let unit_bytes = unit_bytes_of(1, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::GaveUpWaitingForTheCallerIsStopping,
            pci_state_when_judged_unusable:
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
        }),
        "调用方在停：放弃那一等，不重交到报这张卡用不了"
    );
    assert_eq!(
        background_gpu.pci_state_readings_taken(),
        0,
        "放弃不是判卡用不了的时刻：一次都不读"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 0,
            pieces_submitted: 0,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        },
        "交了一次、没做完、不重交"
    );
    assert!(
        background_gpu.cards_on_this_host[0]
            .budget()
            .memory_account()
            .has_a_submission_running_past_its_wait(),
        "那一次提交留在这张卡的账上"
    );
    let gpus_leaked = background_gpu.gpus_leaked_when_dropped();
    drop(background_gpu);
    assert_eq!(
        gpus_leaked.gpus_leaked(),
        Some(vec![GpuLeakedWhenDropped {
            device: card_identifier("0000:01:00.0"),
            bytes_held_on_the_device:
                bytes_held_by_the_checksum_chunk_of_one_unit_of_one_thousand_bytes(),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
        }]),
        "有提交还在跑的那张卡整张泄漏"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
        bytes_held_by_the_checksum_chunk_of_one_unit_of_one_thousand_bytes(),
        "泄漏的字节记进进程的泄漏账"
    );
    assert_eq!(
        drops_of_the_card.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "泄漏的卡不析构"
    );
}

/// 同一条的对照：外壳收下了标记而标记没置，那一次提交一直不做完——照旧每次等满上限、重交 3 次都超时才报这张卡用不了（等的上限与超时的数法不变）。
#[test]
fn the_batch_whose_submission_never_completes_without_the_caller_stopping_reports_the_card_unusable_after_its_resubmissions(
) {
    let (mut background_gpu, _drops_of_the_card) =
        background_gpu_whose_card_never_completes_a_submission(
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let (caller_is_stopping, _handle_never_setting_the_flag) =
        caller_stopping_flag_and_the_handle_setting_it();
    background_gpu.give_up_waiting_for_the_cards_once_the_caller_is_stopping(caller_is_stopping);
    let unit_bytes = unit_bytes_of(1, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
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
        }),
        "没在停：照旧第 4 次超时报警"
    );
}

/// 核心层的把手把调用方在停的标记交到产品那一份实现、再交到每张开着的卡：经把手说调用方在停之后，
/// 一批里那一次提交一直不做完的校验和那一段报给核心层是放弃那一种；丢掉实现之后这张卡整张泄漏。
#[test]
fn the_caller_stopping_told_through_the_core_handle_reaches_the_cards_and_the_batch_is_reported_as_given_up(
) {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let (background_gpu, _drops_of_the_card) =
        background_gpu_whose_card_never_completes_a_submission(&memory_leaked);
    let gpus_leaked = background_gpu.gpus_leaked_when_dropped();
    let shared_implementation = Arc::new(Mutex::new(
        BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(background_gpu),
    ));
    let shared_as_the_core_takes_it: Arc<Mutex<dyn BackgroundGpuCompute + Send>> =
        shared_implementation.clone();
    let handle =
        BackgroundGpuComputeHandle::of_a_shared_implementation(shared_as_the_core_takes_it);
    handle.tell_the_implementation_the_caller_is_stopping();
    let unit_bytes = vec![0_u8; 1000];
    let units = [UnitChecksumToVerifyOnTheGpu {
        entry_key: RegionRecordEntryKey {
            device: DeviceIdentity(0),
            slot: SlotNumber(0),
            span_slots: 1,
        },
        unit_bytes: &unit_bytes,
        expected_crc32c: singlefs_core::checksum::crc32_castagnoli(&unit_bytes),
    }];
    let failure = shared_implementation
        .lock()
        .expect("用例没在别处借着实现 panic 过")
        .verify_unit_checksums(&units)
        .expect_err("调用方在停、提交一直不做完：这一批交不成");
    assert_eq!(
        failure,
        BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping,
        "经把手说调用方在停：卡上那一等放弃，报给核心层的是放弃那一种"
    );
    drop(handle);
    drop(shared_implementation);
    assert_eq!(
        gpus_leaked.gpus_leaked(),
        Some(vec![GpuLeakedWhenDropped {
            device: card_identifier("0000:01:00.0"),
            bytes_held_on_the_device:
                bytes_held_by_the_checksum_chunk_of_one_unit_of_one_thousand_bytes(),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
        }]),
        "丢掉实现：有提交还在跑的那张卡整张泄漏"
    );
}

/// 外壳收下调用方在停的标记之前，开着的卡等一批的结果不读它（建把手之前没有标记）；收下之后开着的卡都读它，
/// 之后切换开成、收进来的卡收进来那一刻起也读它。
#[test]
fn the_cards_opened_and_the_card_taken_in_at_a_switch_read_the_caller_stopping_once_it_is_handed_in(
) {
    let (mut background_gpu, _cards_handed_to_the_opening) =
        background_gpu_with_one_card_not_opened_at_the_mount(
            opening_that_always_opens,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let cards_reading_the_caller_stopping =
        |background_gpu_read: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>| {
            background_gpu_read
                .cards_on_this_host
                .iter()
                .map(|card| {
                    matches!(
                        card.budget()
                            .caller_stopping_seen_by_the_waits_of_the_batches(),
                        CallerStoppingSeenByTheWait::ReadBetweenShortWaits(_)
                    )
                })
                .collect::<Vec<bool>>()
        };
    assert_eq!(
        cards_reading_the_caller_stopping(&background_gpu),
        vec![false],
        "还没收下标记：挂载那一刻开着的卡不读"
    );
    let (caller_is_stopping, _handle_setting_the_flag) =
        caller_stopping_flag_and_the_handle_setting_it();
    background_gpu.give_up_waiting_for_the_cards_once_the_caller_is_stopping(caller_is_stopping);
    assert_eq!(
        cards_reading_the_caller_stopping(&background_gpu),
        vec![true],
        "收下标记：开着的卡都读"
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
    assert_eq!(
        cards_reading_the_caller_stopping(&background_gpu),
        vec![true, true],
        "切换开成、收进来的卡也读"
    );
}
