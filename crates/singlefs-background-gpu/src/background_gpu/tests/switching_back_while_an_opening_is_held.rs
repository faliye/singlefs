//! 一次开卡还压着时切回这张卡或别的卡：最后一次允许的开卡压着或已回来时各怎么答。

use super::*;

// ── 切换开卡线程开一张卡之前再判一次试满：交设置那一刻还剩次数，前一条线程叫停时正在开的那一次开完之后试满了 ──

/// 切换点到「0000:41:00.0」之后挂着的是它的哪一次开卡：一次挂载里至多开的最后一次的前一次，或最后一次。
#[derive(Clone, Copy, Debug)]
enum OpeningHeldAtTheSwitch {
    TheOneBeforeTheLastAllowed,
    TheLastAllowed,
}

impl OpeningHeldAtTheSwitch {
    /// 挂着的那一次是这张卡的第几次开卡（从 1 数）。
    fn opening_number_of_the_card(self) -> usize {
        let openings_at_most =
            usize::try_from(REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS.openings_at_most())
                .expect("一张卡至多开 4 次：放得进 usize");
        match self {
            Self::TheOneBeforeTheLastAllowed => openings_at_most - 1,
            Self::TheLastAllowed => openings_at_most,
        }
    }
}

/// 用户切到 `cpu` 之后切回的那一刻：挂着的那一次开卡开完之前，还是开完（那条线程放开开卡那一步的锁）之后。
#[derive(Clone, Copy, Debug)]
enum MomentOfTheSwitchBack {
    WhileTheOpeningIsHeld,
    OnceTheHeldOpeningReturned,
}

/// 开卡那一步：「0000:41:00.0」每一次都开不成，对它的第 `held_opening_number` 次先往 `held_opening_begun` 送一声、等用例放开（至多 20 秒）；
/// 别的卡当场开成。
fn opening_the_card_at_forty_one_never_and_holding_one_opening(
    held_opening_number: usize,
    held_opening_begun: std::sync::mpsc::Sender<()>,
    held_opening_released: std::sync::mpsc::Receiver<()>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    let mut openings_of_the_card_at_forty_one: usize = 0;
    move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
        if discrete_gpu.pci_bus_address != "0000:41:00.0" {
            return opening_that_always_opens(
                discrete_gpu,
                memory_limit,
                wait_limit,
                cost_at_the_opening,
            );
        }
        openings_of_the_card_at_forty_one += 1;
        if openings_of_the_card_at_forty_one == held_opening_number {
            // 收的那一头在用例里一直拿着。
            let _signal_sent = held_opening_begun.send(());
            let _released_or_not = held_opening_released.recv_timeout(Duration::from_secs(20));
        }
        Err(reason_the_card_does_not_open_at_the_switch())
    }
}

/// panic 带的原话（`&str` 或 `String`；别的交回一句说明）。
fn text_said_by_the_panic(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|said| (*said).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "（panic 带的不是字符串）".to_string())
}

/// 「切到 cpu 再切回」那一段历史的读数：切回之后当场那一问的答案、等开卡开完那一问的答案（panic 了是它的原话）、开卡那一步对「0000:41:00.0」被调了几次、
/// 之后再切一次只点「0000:81:00.0」等到开完的答案（panic 了是它的原话）。
struct ReadingsOfASwitchBack {
    answered_right_after_the_switch_back: LatestSettingsHandedIn,
    answered_once_ended: Result<LatestSettingsHandedIn, String>,
    openings_of_the_card_at_forty_one: usize,
    answered_at_a_later_switch: Result<LatestSettingsHandedIn, String>,
}

/// 挂载开着「0000:01:00.0」；切换只点没开的「0000:41:00.0」，它每一次都开不成，`held_opening` 那一次挂着；挂着的时候用户切到 `cpu`
/// （叫停那条线程，正在开的那一次照旧开完），再切回点 `switch_back_to`（`moment` 定在挂着的那一次开完之前还是之后）；之后再切一次只点「0000:81:00.0」。
fn readings_of_switching_back_while_an_opening_of_the_card_at_forty_one_is_held(
    held_opening: OpeningHeldAtTheSwitch,
    switch_back_to: &[&str],
    moment: MomentOfTheSwitchBack,
) -> ReadingsOfASwitchBack {
    let (held_opening_begun, every_held_opening_begun) = std::sync::mpsc::channel::<()>();
    let (release_the_held_opening, held_opening_released) = std::sync::mpsc::channel::<()>();
    let (mut background_gpu, cards_handed_to_the_opening) =
        background_gpu_with_the_card_at_one_opened_and_two_not_opened(
            opening_the_card_at_forty_one_never_and_holding_one_opening(
                held_opening.opening_number_of_the_card(),
                held_opening_begun,
                held_opening_released,
            ),
            None,
            false,
        );
    let to_the_card_at_forty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
    background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
    every_held_opening_begun
        .recv_timeout(Duration::from_secs(20))
        .expect("挂着的那一次开卡 20 秒内起了");
    background_gpu.use_no_gpu_from_the_next_batch();
    let switching_back =
        switch_naming_the_cards_admitted_against(&background_gpu, switch_back_to, None);
    match moment {
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld => {}
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => {
            release_the_held_opening
                .send(())
                .expect("挂着的那一次还在等放开");
            // 切换开卡线程从头一次开卡之前到开完都锁着开卡那一步：拿到锁就是它开完了。
            drop(
                background_gpu
                    .opening_of_a_gpu_at_a_switch
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
    }
    background_gpu.use_the_settings_from_the_next_batch(&switching_back);
    let answered_right_after_the_switch_back = background_gpu
        .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
    match moment {
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld => {
            release_the_held_opening
                .send(())
                .expect("挂着的那一次还在等放开");
        }
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => {}
    }
    let answered_once_ended = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds)
    }))
    .map_err(|payload| text_said_by_the_panic(&*payload));
    let openings_of_the_card_at_forty_one =
        openings_of_the_card_at_forty_one(&cards_handed_to_the_opening);
    let to_the_card_at_eighty_one =
        switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
    let answered_at_a_later_switch = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds)
    }))
    .map_err(|payload| text_said_by_the_panic(&*payload));
    ReadingsOfASwitchBack {
        answered_right_after_the_switch_back,
        answered_once_ended,
        openings_of_the_card_at_forty_one,
        answered_at_a_later_switch,
    }
}

/// 「0000:41:00.0」在这一次挂载里连着开不成 4 次、试满了。
fn card_at_forty_one_used_up() -> LatestSettingsHandedIn {
    card_at_forty_one_not_opened_with(vec![GpuFailureCause::OpeningsInThisMountUsedUp {
        failed_openings: 4,
    }])
}

/// 那一段历史照条款的结局：两问的答案、「0000:41:00.0」一共开了几次（一次挂载里连着开不成至多 4 次）；之后再切一次只点「0000:81:00.0」换上了；
/// 哪一问都不 panic。
fn assert_the_switch_back_ends_as(
    held_opening: OpeningHeldAtTheSwitch,
    switch_back_to: &[&str],
    moment: MomentOfTheSwitchBack,
    answered_right_after_the_switch_back: LatestSettingsHandedIn,
    answered_once_ended: LatestSettingsHandedIn,
    openings_of_the_card_at_forty_one: usize,
) {
    let readings = readings_of_switching_back_while_an_opening_of_the_card_at_forty_one_is_held(
        held_opening,
        switch_back_to,
        moment,
    );
    match moment {
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld => assert_eq!(
                readings.answered_right_after_the_switch_back,
                answered_right_after_the_switch_back
            ),
            // 挂着的那一次已开完、为切回起的新线程不用等锁：它开得快时，切回之后当场那一问可能已收到结局。还在开、或答的就是开完时那个结局，两样都对。
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => assert!(
                readings.answered_right_after_the_switch_back == answered_right_after_the_switch_back
                    || readings.answered_right_after_the_switch_back == answered_once_ended,
                "切回之后当场答 {:?}：不是 {answered_right_after_the_switch_back:?}，也不是开完时的 {answered_once_ended:?}",
                readings.answered_right_after_the_switch_back
            ),
        }
    assert_eq!(readings.answered_once_ended, Ok(answered_once_ended));
    assert_eq!(
        readings.openings_of_the_card_at_forty_one,
        openings_of_the_card_at_forty_one
    );
    assert_eq!(
        readings.answered_at_a_later_switch,
        Ok(put_in_effect_on_every_card())
    );
}

/// 挂着的是最后一次，挂着的时候切回同一张卡：交设置那一刻它才开不成 3 次、为它起了新线程；新线程开它之前前一次开完、试满了——
/// 不再开、当场答试满，它一共开 4 次，不 panic。
#[test]
fn switching_back_to_the_card_while_its_last_allowed_opening_is_held_answers_it_used_up_without_opening_it_again(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:41:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_used_up(),
        4,
    );
}

/// 同上，切回点「0000:81:00.0」再点它：先开成「0000:81:00.0」，轮到它时试满了、当场答试满，它一共开 4 次，不 panic。
#[test]
fn switching_back_to_the_other_card_then_the_card_while_its_last_allowed_opening_is_held_answers_it_used_up(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:81:00.0", "0000:41:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_used_up(),
        4,
    );
}

/// 挂着的是最后一次，它开完之后才切回同一张卡：交设置那一刻就试满，当场答试满。
#[test]
fn switching_back_to_the_card_once_its_last_allowed_opening_returned_answers_it_used_up_at_the_handing_in(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:41:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        card_at_forty_one_used_up(),
        card_at_forty_one_used_up(),
        4,
    );
}

/// 同上，切回点「0000:81:00.0」再点它：交设置那一刻就试满，当场答试满。
#[test]
fn switching_back_to_the_other_card_then_the_card_once_its_last_allowed_opening_returned_answers_it_used_up(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:81:00.0", "0000:41:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        card_at_forty_one_used_up(),
        card_at_forty_one_used_up(),
        4,
    );
}

/// 挂着的是最后一次，挂着的时候切回只点「0000:81:00.0」：换上它。
#[test]
fn switching_back_to_the_other_card_while_the_last_allowed_opening_is_held_puts_it_in_effect() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:81:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        put_in_effect_on_every_card(),
        4,
    );
}

/// 同上，最后一次开完之后才切回：换上它。
#[test]
fn switching_back_to_the_other_card_once_the_last_allowed_opening_returned_puts_it_in_effect() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheLastAllowed,
        &["0000:81:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        LatestSettingsHandedIn::StillOpeningItsCards,
        put_in_effect_on_every_card(),
        4,
    );
}

/// 挂着的是最后一次的前一次，挂着的时候切回同一张卡：新线程开它之前它开不成 3 次，还剩一次，开一次又开不成，答开不成、一个原因，一共 4 次。
#[test]
fn switching_back_to_the_card_while_the_opening_before_its_last_is_held_opens_it_once_more() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:41:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
        4,
    );
}

/// 同上，那一次开完之后才切回：开一次又开不成，一共 4 次。
#[test]
fn switching_back_to_the_card_once_the_opening_before_its_last_returned_opens_it_once_more() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:41:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
        4,
    );
}

/// 挂着的是最后一次的前一次，挂着的时候切回点「0000:81:00.0」再点它：先开成「0000:81:00.0」，它再开一次又开不成，一共 4 次。
#[test]
fn switching_back_to_the_other_card_then_the_card_while_the_opening_before_its_last_is_held_opens_it_once_more(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:81:00.0", "0000:41:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
        4,
    );
}

/// 同上，那一次开完之后才切回：它再开一次又开不成，一共 4 次。
#[test]
fn switching_back_to_the_other_card_then_the_card_once_the_opening_before_its_last_returned_opens_it_once_more(
) {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:81:00.0", "0000:41:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        LatestSettingsHandedIn::StillOpeningItsCards,
        card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
        4,
    );
}

/// 挂着的是最后一次的前一次，挂着的时候切回只点「0000:81:00.0」：换上它，「0000:41:00.0」一共 3 次。
#[test]
fn switching_back_to_the_other_card_while_the_opening_before_the_last_is_held_puts_it_in_effect() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:81:00.0"],
        MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
        LatestSettingsHandedIn::StillOpeningItsCards,
        put_in_effect_on_every_card(),
        3,
    );
}

/// 同上，那一次开完之后才切回：换上它，「0000:41:00.0」一共 3 次。
#[test]
fn switching_back_to_the_other_card_once_the_opening_before_the_last_returned_puts_it_in_effect() {
    assert_the_switch_back_ends_as(
        OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
        &["0000:81:00.0"],
        MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
        LatestSettingsHandedIn::StillOpeningItsCards,
        put_in_effect_on_every_card(),
        3,
    );
}
