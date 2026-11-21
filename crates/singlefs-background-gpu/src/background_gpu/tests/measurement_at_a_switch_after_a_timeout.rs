//! 量代价或一批超时之后，切换时再量这张卡：在上限之内等那次提交做完，等不成、装不下就不用它。

use super::*;

// ── 切换时量代价先等账上还在跑的那次提交（替身卡的量代价照产品那一份走账）──

/// 两张替身卡：「0000:01:00.0」量代价照产品走账（`measured_at_the_mount` 为假时开卡时没量过代价）、「0000:41:00.0」照旧。
fn two_cards_measuring_the_cost_through_the_account(
    measured_at_the_mount: bool,
) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
    let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card_at_one.cost_measured_through_the_checksum_chunk = true;
    if !measured_at_the_mount {
        card_at_one.measured_cost = None;
    }
    let card_at_forty_one = CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
    let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card_at_one, card_at_forty_one],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    background_gpu
}

/// 历史甲：「0000:01:00.0」开卡时没量过代价、被晾下；切换请回来，量代价那一次提交等超过上限（账上记成还在跑）。
fn card_at_one_whose_measurement_at_a_switch_timed_out(
) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
    let mut background_gpu = two_cards_measuring_the_cost_through_the_account(false);
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
    let leaving_it_out = settings_admitted_on_the_cards(
        &background_gpu,
        None,
        vec![smallest_piece_bytes - 1, 1 << 30],
    );
    background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
    background_gpu.cards_on_this_host[0]
        .submissions_timing_out
        .push_back(true);
    let bringing_it_back =
        settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
    assert!(
        background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .has_a_submission_running_past_its_wait(),
        "前缀：量代价那一次等超过上限，账上记着还在跑"
    );
    background_gpu
}

/// 历史乙：「0000:01:00.0」量过代价、参与；一批校验和头一次交等超过上限、之后三回等前一次都没等到（报这张卡用不了，账上照旧记着还在跑），
/// 切换给它装不下最小一片的上限（晾下）。
fn card_at_one_left_out_while_its_timed_out_submission_runs(
) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
    let mut background_gpu = two_cards_measuring_the_cost_through_the_account(true);
    {
        let card_at_one = &mut background_gpu.cards_on_this_host[0];
        card_at_one.submissions_timing_out = VecDeque::from([true]);
        card_at_one.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
    }
    let unit_bytes = unit_bytes_of(1, 1000);
    let batch = background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
    assert!(batch.is_err(), "前缀：这一批报这张卡用不了");
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
    let leaving_it_out = settings_admitted_on_the_cards(
        &background_gpu,
        None,
        vec![smallest_piece_bytes - 1, 1 << 30],
    );
    background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
    assert!(
        background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .has_a_submission_running_past_its_wait(),
        "前缀：晾下之后账上照旧记着还在跑"
    );
    background_gpu
}

/// 之后用户的那一次切换。
#[derive(Clone, Copy, Debug)]
enum SwitchAfterTheHistory {
    BothNamedUnderALargeLimit,
    BothNamedUnderALimitJustHoldingTheSmallestPiece,
    BothNamedUnderALimitOneByteShort,
    OnlyTheOtherCardNamed,
    ToTheCpu,
}

/// 在前缀之后交 `switch`，接住 panic：交回 panic 带的原话（没 panic 是 `None`）与「0000:01:00.0」此刻开没开着、账上还记不记着在跑。
fn outcome_of_the_switch_after_the_history(
    mut background_gpu: BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    switch: SwitchAfterTheHistory,
) -> (Option<String>, bool, bool) {
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
    let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
        .budget
        .memory_account()
        .bytes_held()
        .0;
    let settings =
        match switch {
            SwitchAfterTheHistory::BothNamedUnderALargeLimit => Some(
                settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]),
            ),
            SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece => {
                Some(settings_admitted_on_the_cards(
                    &background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes, 1 << 30],
                ))
            }
            SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort => {
                Some(settings_admitted_on_the_cards(
                    &background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
                ))
            }
            SwitchAfterTheHistory::OnlyTheOtherCardNamed => Some(settings_admitted_on_the_cards(
                &background_gpu,
                Some(vec![card_identifier("0000:41:00.0")]),
                vec![1 << 30],
            )),
            SwitchAfterTheHistory::ToTheCpu => None,
        };
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &settings {
        Some(settings) => background_gpu.use_the_settings_from_the_next_batch(settings),
        None => background_gpu.use_no_gpu_from_the_next_batch(),
    }));
    let said_by_the_panic = caught.err().map(|payload| {
        payload
            .downcast_ref::<&str>()
            .map(|said| (*said).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default()
    });
    let card_at_one = background_gpu
        .cards_on_this_host
        .iter()
        .find(|card| card.discrete_gpu.identifier == card_identifier("0000:01:00.0"));
    (
        said_by_the_panic,
        card_at_one.is_some(),
        card_at_one.is_some_and(|card| {
            card.budget
                .memory_account()
                .has_a_submission_running_past_its_wait()
        }),
    )
}

/// 切换让账上还记着在跑的那张卡量代价：先在等的上限之内等它做完、放账，再量——不 panic，量完账上不再记着在跑。
#[test]
fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_large_limit() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_whose_measurement_at_a_switch_timed_out(),
            SwitchAfterTheHistory::BothNamedUnderALargeLimit
        ),
        (None, true, false)
    );
}

#[test]
fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
) {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_whose_measurement_at_a_switch_timed_out(),
            SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
        ),
        (None, true, false)
    );
}

/// 对照：上限少一个字节，这张卡照旧晾下、不量代价——账上照旧记着在跑。
#[test]
fn a_switch_after_the_measurement_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_whose_measurement_at_a_switch_timed_out(),
            SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
        ),
        (None, true, true)
    );
}

#[test]
fn a_switch_after_the_measurement_timed_out_naming_only_the_other_card_closes_it() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_whose_measurement_at_a_switch_timed_out(),
            SwitchAfterTheHistory::OnlyTheOtherCardNamed
        ),
        (None, false, false)
    );
}

#[test]
fn a_switch_to_the_cpu_after_the_measurement_timed_out_closes_every_card() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_whose_measurement_at_a_switch_timed_out(),
            SwitchAfterTheHistory::ToTheCpu
        ),
        (None, false, false)
    );
}

#[test]
fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_large_limit() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_left_out_while_its_timed_out_submission_runs(),
            SwitchAfterTheHistory::BothNamedUnderALargeLimit
        ),
        (None, true, false)
    );
}

#[test]
fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
) {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_left_out_while_its_timed_out_submission_runs(),
            SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
        ),
        (None, true, false)
    );
}

#[test]
fn a_switch_after_a_batch_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_left_out_while_its_timed_out_submission_runs(),
            SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
        ),
        (None, true, true)
    );
}

#[test]
fn a_switch_after_a_batch_timed_out_naming_only_the_other_card_closes_it() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_left_out_while_its_timed_out_submission_runs(),
            SwitchAfterTheHistory::OnlyTheOtherCardNamed
        ),
        (None, false, false)
    );
}

#[test]
fn a_switch_to_the_cpu_after_a_batch_timed_out_closes_every_card() {
    assert_eq!(
        outcome_of_the_switch_after_the_history(
            card_at_one_left_out_while_its_timed_out_submission_runs(),
            SwitchAfterTheHistory::ToTheCpu
        ),
        (None, false, false)
    );
}

/// 等不到账上还在跑的那次提交：量代价照出错交回，这张卡不参与，下一批报成换设置时量代价出错（等前一次没等到的那一种）。
#[test]
fn a_measurement_at_a_switch_whose_wait_for_the_running_submission_times_out_leaves_the_card_out() {
    let mut background_gpu = card_at_one_whose_measurement_at_a_switch_timed_out();
    background_gpu.cards_on_this_host[0]
        .earlier_submission_waits_timing_out
        .push_back(true);
    let bringing_it_back =
        settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
    assert_eq!(
        background_gpu.batch_not_split_over_the_gpus(),
        Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
            },
            pci_state_when_judged_unusable:
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
        })
    );
    assert_eq!(
        participating_devices_of(&background_gpu),
        vec![card_identifier("0000:41:00.0")]
    );
    assert_eq!(
        background_gpu.pci_state_readings_taken(),
        0,
        "等前一次没等到不是判卡用不了的那一种：量代价出错也不读"
    );
}
