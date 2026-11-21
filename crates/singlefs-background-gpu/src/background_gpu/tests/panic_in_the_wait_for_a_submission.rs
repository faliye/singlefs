//! 等提交时 panic 当这张卡丢了收下，之后不再给它活，切换再点它也不给。

use super::*;

/// 等卡回话那一下 panic（wgpu 30.0.1 在等待报设备丢了时在 `Device::poll` 里 panic）：接住，这一批报这张卡的 GPU 出错、原因是卡当场报出错
/// （带 panic 的原话），判它用不了那一刻读 1 次 PCI 状态，panic 不穿出去；这一段的提交照没等到它做完记在账上（关卡时整张泄漏）。
/// 下一批交活之前那一等当场照记下的报、不再调 wgpu，这张卡一段都不再交（交成的次数停在 1），这一次又判它用不了、再读 1 次。
#[test]
fn a_panic_in_the_wait_for_a_submission_is_caught_as_the_card_lost_and_the_card_gets_no_more_work()
{
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submission_waits_panicking = true;
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
        reason: format!(
            "等提交时 wgpu 的 Device::poll panic 了：{PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD}"
        ),
    };
    let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
    let (first_batch, _devices_given_work) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert_eq!(
        first_batch,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost.clone(),
            pci_state_when_judged_unusable: pci_state_read.clone(),
        })
    );
    assert_eq!(background_gpu.pci_state_readings_taken(), 1);
    assert!(
        background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .has_a_submission_running_past_its_wait(),
        "panic 了的那次提交照没等到它做完记着：关卡时这张卡整张泄漏"
    );
    let (second_batch, _devices_given_work_by_the_second_batch) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert_eq!(
        second_batch,
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost,
            pci_state_when_judged_unusable: pci_state_read,
        })
    );
    assert_eq!(
        background_gpu.cards_on_this_host[0].submissions_made, 1,
        "接住之后这张卡一段都不再交"
    );
    assert_eq!(background_gpu.pci_state_readings_taken(), 2);
}

/// 本机两张替身卡「0000:01:00.0」「0000:41:00.0」都参与，「0000:01:00.0」等提交时 panic（只供测试的钩子）；交一批，交回接住之后的那一份外壳。
fn background_gpu_after_the_wait_on_the_card_at_one_panicked(
) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
    let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card_at_one.submission_waits_panicking = true;
    let card_at_forty_one = CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card_at_one, card_at_forty_one],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let (outcome, _devices_given_work) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(
        matches!(
            &outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                ..
            })
        ),
        "前缀：等的那一下 panic 接住、报卡当场报出错：{outcome:?}"
    );
    background_gpu
}

/// 接住 panic 之后切换先因显存上限晾下「0000:01:00.0」、再请它回来：请回来要先等账上那次提交，等的那一步照记下的报卡当场报出错——
/// 照换设置时量代价出错办，它不参与，下一批报成它的 GPU 出错（带那一刻读的 PCI 状态），它一段都不再交。
#[test]
fn a_switch_bringing_back_the_card_whose_wait_panicked_leaves_it_out_and_gives_it_no_work() {
    let mut background_gpu = background_gpu_after_the_wait_on_the_card_at_one_panicked();
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
    let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
        .budget
        .memory_account()
        .bytes_held()
        .0;
    let leaving_it_out = settings_admitted_on_the_cards(
        &background_gpu,
        None,
        vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
    );
    background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
    let bringing_it_back =
        settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
    assert!(
        matches!(
            background_gpu.batch_not_split_over_the_gpus(),
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device,
                cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                ..
            }) if device == card_identifier("0000:01:00.0")
        ),
        "{:?}",
        background_gpu.batch_not_split_over_the_gpus()
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
    let (outcome, _devices_given_work) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(
        matches!(
            &outcome,
            Err(ChecksumBatchFailed::GpuFailed { device, .. })
                if *device == card_identifier("0000:01:00.0")
        ),
        "{outcome:?}"
    );
    assert_eq!(
        background_gpu.cards_on_this_host[0].submissions_made, 1,
        "接住之后这张卡一段都不再交"
    );
}

/// 接住 panic 之后切换照原样再点「0000:01:00.0」（上限装得下，不重量代价）：下一批交活之前那一等照记下的报卡当场报出错，这一批报成它的 GPU 出错，
/// 它一段都不再交。
#[test]
fn a_switch_naming_again_the_card_whose_wait_panicked_gives_it_no_work() {
    let mut background_gpu = background_gpu_after_the_wait_on_the_card_at_one_panicked();
    let both_cards = settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&both_cards);
    let (outcome, _devices_given_work) =
        checksum_batch_and_the_devices_given_work(&mut background_gpu);
    assert!(
        matches!(
            &outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device,
                cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                ..
            }) if *device == card_identifier("0000:01:00.0")
        ),
        "{outcome:?}"
    );
    assert_eq!(
        background_gpu.cards_on_this_host[0].submissions_made, 1,
        "接住之后这张卡一段都不再交"
    );
}
