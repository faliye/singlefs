//! 一张卡这一笔能借多少、借不借的判法：一次派活带几个状态、每张卡的额度、这个进程能占的与许出去而还没占上卡的、此刻的余量，
//! 几张卡一起按状态数借草稿区时每张卡放几个、都试过之后怎么判；都是算术，不读卡、不拿锁。

use super::*;

/// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
/// 草稿区按它开：状态数之外开的一个状态都用不上（一条 45 个状态的流按显存开满一张空着的 5090 是约 21 GiB，用得上的约 54 MiB）。
#[must_use]
pub fn states_in_one_dispatch_capped_by_the_state_count(
    states_the_memory_holds: usize,
    requested_states_per_dispatch: usize,
    state_count: u64,
) -> usize {
    let state_count_as_usize = usize::try_from(state_count).unwrap_or(usize::MAX);
    states_the_memory_holds
        .min(requested_states_per_dispatch)
        .min(state_count_as_usize)
        .max(1)
}

/// 一张卡上这个进程一共能占多少（MiB，这个进程自己的驱动那一份已经扣掉；减去它借着的才是这一笔能借的，文件头「口径」）。
/// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）就取三者里最小的：这个额度、
/// 配置给卡的额度减驱动的固定开销、这个进程此刻能在这张卡上占的减驱动的固定开销（额度变量不越过配置给卡的额度）。
/// 配置给卡的额度：`GPU_CARDS_BY_PRIORITY` 配给这张卡的（`configured_card_quota_mebibytes`）；没配显卡优先级（`None`）时由
/// 没配额度时默认的那一份代替，即 `budget_mebibytes`（那份显存的 3/4：主 agent 定没配优先级时 3/4 那一份也并进最小值）。
/// 没设额度变量照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度与这个进程能占的里小的那个，或这个进程能占的 3/4）
/// 减驱动的固定开销（它是每张卡的上限，不是每条用例的上限，不判红）。
#[must_use]
pub fn card_ledger_capacity_mebibytes(
    card_quota_from_the_environment_mebibytes: Option<u64>,
    configured_card_quota_mebibytes: Option<u64>,
    budget_mebibytes: u64,
    memory_this_process_may_hold_mebibytes: u64,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    let default_share_without_a_configured_card_quota_mebibytes =
        match configured_card_quota_mebibytes {
            Some(_) => u64::MAX,
            None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
        };
    match card_quota_from_the_environment_mebibytes {
        Some(quota) => quota.min(
            configured_card_quota_mebibytes
                .map_or(u64::MAX, |configured| {
                    configured.saturating_sub(mebibytes_left_for_the_driver)
                })
                .min(default_share_without_a_configured_card_quota_mebibytes)
                .min(
                    memory_this_process_may_hold_mebibytes
                        .saturating_sub(mebibytes_left_for_the_driver),
                ),
        ),
        None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
    }
}

/// 这个进程在一张卡上记着的：活着的借据合计（MiB）与驱动那一份走到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BookedByThisProcessOnACard {
    pub leases_mebibytes: u64,
    pub driver_share: DriverShareOnTheCard,
}

/// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
/// （[`occupied_part_of_what_this_process_promised_mebibytes`]）。
#[must_use]
pub fn memory_this_process_may_hold_mebibytes(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    reading
        .free_mebibytes
        .saturating_add(occupied_part_of_what_this_process_promised_mebibytes(
            reading,
            booked,
            mebibytes_left_for_the_driver,
        ))
}

/// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是活着的借据，加登记了时驱动那一份）：比许出去的多（分配器按块拿）
/// 只算到许出去的那么多；读不到实际占着的、这个进程在这张卡上另有不走这份账的设备、或这张卡上丢了的设备可能还占着
/// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]），
/// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少、哪一份是旧设备的都分不出）。
/// 销了账的借据不在许出去的里，不再从实际占着的里扣：它的缓冲在销账之前已经交回驱动（等完了设备，或那台设备上提交的活后来做完了，
/// 文件头「口径」，G2 乙），或者它拿着的是丢了还没放掉的设备，那时这里本来就交 0、放掉之前缓冲交回驱动。
#[must_use]
pub fn occupied_part_of_what_this_process_promised_mebibytes(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    let promised_mebibytes = match booked.driver_share {
        DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
        DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
            booked
                .leases_mebibytes
                .saturating_add(mebibytes_left_for_the_driver)
        }
        // 旧设备留在卡上的那一堆读数里与新设备的分不出：当一点都不是许出去的（只会少借）
        DriverShareOnTheCard::DeviceLostNotStartedAgain
        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,
    };
    match reading.occupied_by_this_process {
        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
            mebibytes.min(promised_mebibytes)
        }
        // 驱动那一份当整份在卡上，实占的 x 比 D 小时多给 D − x（攻方乙附）：驱动之后还会长到 D
        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,
    }
}

/// 一张卡此刻对这个进程：这一笔最多能借多少，与这张卡空着时最多能借多少（MiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRoomForThisProcess {
    pub room_now_mebibytes: u64,
    /// 同一个算法拿空卡上能用的（[`CardMemoryReading::total_less_reserved_mebibytes`]）当这个进程能占的、借着的当 0：
    /// 要的比它大，等多久都放不下。
    pub ceiling_mebibytes: u64,
}

/// 一张卡此刻能借给这个进程的这一笔（文件头「口径」）：`capacity_of_memory_this_process_may_hold` 把这个进程能占的
/// （[`memory_this_process_may_hold_mebibytes`]）换成它一共能占的（卡池照 [`card_ledger_capacity_mebibytes`] 算），减去借着的。
#[must_use]
pub fn card_room_for_this_process(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
    capacity_of_memory_this_process_may_hold: &dyn Fn(u64) -> u64,
) -> CardRoomForThisProcess {
    let may_hold_mebibytes =
        memory_this_process_may_hold_mebibytes(reading, booked, mebibytes_left_for_the_driver);
    CardRoomForThisProcess {
        room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)
            .saturating_sub(booked.leases_mebibytes),
        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(
            reading.total_less_reserved_mebibytes,
        ),
    }
}

/// 借一次显存的结局（[`decide_booking_by_priority`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookingDecision {
    /// 记在这张卡上：照卡的先后，第一张此刻放得下的卡（值是它在给的表里的次序）。
    BookOnTheCardAt { card_position: usize },
    /// 此刻每张卡都放不下，而至少一张卡空着时放得下：借不到（不等）。
    NoRoomNow,
    /// 要的比每张卡空着时能借的都大（一张卡都没有也算）：等多久都放不下。
    NeverFits { largest_ceiling_mebibytes: u64 },
}

/// 一次借 `need_mebibytes` 记在哪张卡上：`rooms` 是每张卡此刻对这个进程的，次序就是优先级。
#[must_use]
pub fn decide_booking_by_priority(
    rooms: &[CardRoomForThisProcess],
    need_mebibytes: u64,
) -> BookingDecision {
    let largest_ceiling_mebibytes = rooms
        .iter()
        .map(|room| room.ceiling_mebibytes)
        .max()
        .unwrap_or(0);
    if need_mebibytes > largest_ceiling_mebibytes {
        return BookingDecision::NeverFits {
            largest_ceiling_mebibytes,
        };
    }
    match rooms
        .iter()
        .position(|room| need_mebibytes <= room.room_now_mebibytes)
    {
        Some(card_position) => BookingDecision::BookOnTheCardAt { card_position },
        None => BookingDecision::NoRoomNow,
    }
}

/// 要的比每张卡空着时能借的都大：配了 GPU 就一定用 GPU，调用方据此判红，不悄悄换小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeedLargerThanEveryCard {
    pub need_mebibytes: u64,
    pub largest_ceiling_mebibytes: u64,
}

/// 在 `0..=most_states` 里找最多的状态数，它要的显存（`mebibytes_for_states`，MiB）不超过 `capacity_mebibytes`：
/// 0 个状态不借、恒放得下；`mebibytes_for_states` 要随状态数单调不减（判器按状态数借的那一族都是）。二分，最多问 65 次。
#[must_use]
pub fn most_states_whose_need_fits_the_capacity(
    most_states: usize,
    capacity_mebibytes: u64,
    mebibytes_for_states: &dyn Fn(usize) -> u64,
) -> usize {
    if most_states == 0 || mebibytes_for_states(most_states) <= capacity_mebibytes {
        return most_states;
    }
    // 不变量：`fitting` 放得下（0 恒放得下），`not_fitting` 放不下；两者之间每轮砍半
    let mut fitting = 0usize;
    let mut not_fitting = most_states;
    while not_fitting - fitting > 1 {
        let middle = fitting + (not_fitting - fitting) / 2;
        if mebibytes_for_states(middle) <= capacity_mebibytes {
            fitting = middle;
        } else {
            not_fitting = middle;
        }
    }
    fitting
}

/// 产品路径在一张卡上当场借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchAttemptOnACard {
    /// 前面的卡已经装下了全部要判的状态：没去借。调用方算出这张卡一个状态都带不了的不是这一种，是下面两种之一
    /// （「此刻带不了」不并进「没有要装的」，否则一张都借不到时落进判红那一支）。
    NothingLeftToPlace,
    /// 借到了 `states` 个状态的草稿区（在锁里照此刻能借的压过，不多于要装的）。
    Booked { states: u64 },
    /// 此刻连一个状态都放不下（这个进程别的借据或卡上别的用户占着），空着时放得下：这张卡这一趟不装状态；
    /// `need_mebibytes` 是一个状态要借的量。
    NoRoomNow { need_mebibytes: u64 },
    /// 这张卡空着时连一个状态的草稿区都放不下，等多久都一样。
    CannotHoldOneState { ceiling_mebibytes: u64 },
}

/// 产品路径按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上：每张卡要装的是剩下的与它带得了的（`states_each_card_holds`）
/// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次；借到几个就从剩下的里减几个，
/// 借不到的卡不装，它那一份留给后面的卡。装满了的卡之后剩下的状态排队等前面的卡判完（不算在任何一张卡上），
/// 后面的卡没有要装的，不去借（[`ScratchAttemptOnACard::NothingLeftToPlace`]）。还有要装的而调用方算出这张卡一个状态都带不了，
/// 不去借，交 `classify_a_card_holding_no_state(卡在表里的次序)` 判是此刻带不了还是空着时也带不了。
/// 交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
///
/// # Panics
/// `attempt_now` 交回的借到的状态数是 0、或比要装的多；`classify_a_card_holding_no_state` 交回的不是此刻借不到、空着时也放不下两种之一。
pub fn place_states_on_the_cards_that_book_now(
    pending_states: u64,
    states_each_card_holds: &[u64],
    attempt_now: &mut dyn FnMut(usize, u64) -> ScratchAttemptOnACard,
    classify_a_card_holding_no_state: &mut dyn FnMut(usize) -> ScratchAttemptOnACard,
) -> Vec<ScratchAttemptOnACard> {
    let mut states_left_to_place = pending_states;
    let mut attempts = Vec::with_capacity(states_each_card_holds.len());
    for (card_slot, states_the_card_holds) in states_each_card_holds.iter().enumerate() {
        if states_left_to_place == 0 {
            attempts.push(ScratchAttemptOnACard::NothingLeftToPlace);
            continue;
        }
        if *states_the_card_holds == 0 {
            let classified = classify_a_card_holding_no_state(card_slot);
            match classified {
                ScratchAttemptOnACard::NoRoomNow { .. }
                | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
                ScratchAttemptOnACard::NothingLeftToPlace
                | ScratchAttemptOnACard::Booked { .. } => {
                    panic!(
                        "第 {card_slot} 张卡一个状态都带不了，判出来的是 {classified:?}：只能是此刻借不到或空着时也放不下"
                    )
                }
            }
            attempts.push(classified);
            continue;
        }
        let states_to_place = states_left_to_place.min(*states_the_card_holds);
        let attempt = attempt_now(card_slot, states_to_place);
        match attempt {
            ScratchAttemptOnACard::Booked { states } => {
                assert!(
                    states > 0 && states <= states_to_place,
                    "第 {card_slot} 张卡要装 {states_to_place} 个状态，借到的是 {states} 个：借到的要在 1 到要装的之间"
                );
                states_left_to_place -= states;
            }
            ScratchAttemptOnACard::NothingLeftToPlace
            | ScratchAttemptOnACard::NoRoomNow { .. }
            | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
        }
        attempts.push(attempt);
    }
    attempts
}

/// 一张此刻连一个状态（`need_of_one_state_mebibytes`）都借不出的卡算哪一种：空着时也放不下是 [`ScratchAttemptOnACard::CannotHoldOneState`]，
/// 空着时放得下是此刻借不到（[`ScratchAttemptOnACard::NoRoomNow`]）。
#[must_use]
pub fn attempt_on_a_card_without_room_for_one_state(
    need_of_one_state_mebibytes: u64,
    room: CardRoomForThisProcess,
) -> ScratchAttemptOnACard {
    if need_of_one_state_mebibytes > room.ceiling_mebibytes {
        return ScratchAttemptOnACard::CannotHoldOneState {
            ceiling_mebibytes: room.ceiling_mebibytes,
        };
    }
    ScratchAttemptOnACard::NoRoomNow {
        need_mebibytes: need_of_one_state_mebibytes,
    }
}

/// 每张卡当场试过一次之后怎么办（[`decide_after_attempting_every_card`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchPlacementDecision {
    /// 至少一张卡借到了：就在借到的那几张上判。
    JudgeOnTheCardsBooked,
    /// 一张都没借到，而至少一张卡空着时放得下：借不到（不等），调用方报本次未跑（`need_mebibytes` 是第一张放不下的卡要借的量）。
    NoRoomOnAnyCardNow { need_mebibytes: u64 },
    /// 一张都没借到，每张卡空着时连一个状态都放不下：等多久都一样，调用方判红。
    NoCardHoldsOneState { largest_ceiling_mebibytes: u64 },
}

/// 一张借到了就判；一张都没借到时，有空着时放得下的就是此刻借不到，没有就判成没有一张卡装得下一个状态。
#[must_use]
pub fn decide_after_attempting_every_card(
    attempts: &[ScratchAttemptOnACard],
) -> ScratchPlacementDecision {
    let mut first_need_without_room = None;
    let mut largest_ceiling_mebibytes = 0u64;
    for attempt in attempts {
        match attempt {
            ScratchAttemptOnACard::Booked { .. } => {
                return ScratchPlacementDecision::JudgeOnTheCardsBooked;
            }
            ScratchAttemptOnACard::NoRoomNow { need_mebibytes } => {
                first_need_without_room = first_need_without_room.or(Some(*need_mebibytes));
            }
            ScratchAttemptOnACard::CannotHoldOneState { ceiling_mebibytes } => {
                largest_ceiling_mebibytes = largest_ceiling_mebibytes.max(*ceiling_mebibytes);
            }
            ScratchAttemptOnACard::NothingLeftToPlace => {}
        }
    }
    match first_need_without_room {
        Some(need_mebibytes) => ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes },
        None => ScratchPlacementDecision::NoCardHoldsOneState {
            largest_ceiling_mebibytes,
        },
    }
}

/// 产品路径第 ① 段借草稿区之前怎么走（[`decide_scratch_before_booking`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchBeforeBooking {
    /// 至少一张卡此刻带得了一个状态：去借。
    BookOnTheCardsThatHoldAStateNow,
    /// 每张卡此刻都带不了一个状态，至少一张空着时带得了：此刻借不到，调用方报本次未跑（不等、不判红）。
    NoCardHoldsAStateNow,
    /// 每张卡空着时也带不了一个状态（一张卡都没有也算）：等多久都一样，调用方判红。
    NoCardHoldsAStateEvenWhenEmpty,
}

/// 照每张卡此刻带得了几个状态（`states_each_card_holds_now`）与空着时带得了几个（`states_each_card_holds_when_empty`，逐张对上）
/// 定借草稿区之前怎么走（每张卡此刻都放不下一个状态而空着时放得下，是借不到，不是放不下）。
///
/// # Panics
/// 两张表的卡数不同。
#[must_use]
pub fn decide_scratch_before_booking(
    states_each_card_holds_now: &[u64],
    states_each_card_holds_when_empty: &[u64],
) -> ScratchBeforeBooking {
    assert_eq!(
        states_each_card_holds_now.len(),
        states_each_card_holds_when_empty.len(),
        "此刻与空着时两张表要逐张对上"
    );
    if states_each_card_holds_now.iter().any(|states| *states > 0) {
        return ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow;
    }
    if states_each_card_holds_when_empty
        .iter()
        .any(|states| *states > 0)
    {
        return ScratchBeforeBooking::NoCardHoldsAStateNow;
    }
    ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty
}

/// 一条线程上同时还活着几趟派活的缓冲：wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动，这一趟的缓冲建起来时
/// 上一趟丢掉的可能还占着（推的，没上卡量过）。
pub const PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD: u64 = 2;

/// 一段在卡上建的缓冲合起来占多少显存（MiB，向上取整）：上卡一次、整段都在的 `bytes_uploaded_once`，
/// 加每一趟派活建、派完就丢的 `bytes_of_one_pass` 乘同时还活着的趟数 `passes_alive_at_once`
/// （几条线程同时派活时是线程数乘 [`PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`]）。
#[must_use]
pub fn mebibytes_of_buffers_held_at_once(
    bytes_uploaded_once: u64,
    bytes_of_one_pass: u64,
    passes_alive_at_once: u64,
) -> u64 {
    bytes_uploaded_once
        .saturating_add(bytes_of_one_pass.saturating_mul(passes_alive_at_once))
        .div_ceil(1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::{
        attempt_on_a_card_without_room_for_one_state, card_ledger_capacity_mebibytes,
        card_room_for_this_process, decide_after_attempting_every_card, decide_booking_by_priority,
        decide_scratch_before_booking, mebibytes_of_buffers_held_at_once,
        memory_this_process_may_hold_mebibytes, most_states_whose_need_fits_the_capacity,
        place_states_on_the_cards_that_book_now, states_in_one_dispatch_capped_by_the_state_count,
        BookedByThisProcessOnACard, BookingDecision, CardRoomForThisProcess, ScratchAttemptOnACard,
        ScratchBeforeBooking, ScratchPlacementDecision,
    };
    use crate::gpu_memory_ledger::tests::{reading_of_an_rtx_5090, AN_EMPTY_RTX_5090_HAS, DRIVER};
    use crate::gpu_memory_ledger::{
        CardMemoryReading, DriverShareOnTheCard, OccupiedByThisProcess,
    };

    /// 这个进程能占的是现读的空闲加它实际占着的里属于它许出去的那一部分：没登记、登记了没起设备都只算空闲；
    /// 起了设备而读不到实际占着的，驱动那一份也不当已经在卡上（实占多少分不出）；读得到的只算到许出去的那么多（分配器多拿的块不算）。
    #[test]
    fn the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised() {
        let free = reading_of_an_rtx_5090(20_000);
        let booked = |leases_mebibytes: u64, driver_share: DriverShareOnTheCard| {
            BookedByThisProcessOnACard {
                leases_mebibytes,
                driver_share,
            }
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(0, DriverShareOnTheCard::NotRegistered),
                DRIVER
            ),
            20_000,
            "没登记：只有现读的空闲"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(1_000, DriverShareOnTheCard::RegisteredDeviceNotStarted),
                DRIVER
            ),
            20_000,
            "登记了、设备没起、实际占着的读不到：借的当一点都没建"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000,
            "起了设备、实际占着的读不到：驱动那一份实占多少分不出，不当已经在卡上"
        );
        let occupied = |mebibytes: u64| CardMemoryReading {
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
            ..free
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                occupied(DRIVER + 500),
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000 + DRIVER + 500,
            "读得到实际占着的：驱动那一份与借的里建了的 500"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                occupied(9_000),
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000 + DRIVER + 1_000,
            "实际占着的比许出去的多：只算到许出去的那么多"
        );
    }

    /// K4-3：额度变量 20 000、配置给卡 24 G 都设了，这个进程此刻能占的只有 10 000：容量照能占的那一项压到
    /// 10 000 减驱动那一份，不取两个额度里小的那个。
    #[test]
    fn the_memory_this_process_may_hold_caps_the_capacity_even_when_both_quotas_are_set() {
        const CONFIGURED_CARD_QUOTA: u64 = 24 * 1024;
        const MAY_HOLD: u64 = 10_000;
        assert_eq!(
            card_ledger_capacity_mebibytes(
                Some(20_000),
                Some(CONFIGURED_CARD_QUOTA),
                CONFIGURED_CARD_QUOTA.min(MAY_HOLD),
                MAY_HOLD,
                DRIVER
            ),
            MAY_HOLD - DRIVER,
            "能占的那一项最小：取它"
        );
    }

    /// K4-1：配置给这张卡 8 G、额度变量 20 000：容量不越过配置的额度（减驱动那一份），额度变量更小时取额度变量；
    /// 没设额度变量照旧。数照一张清场后空 30 000 MiB 的 5090。
    #[test]
    fn an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota(
    ) {
        const CONFIGURED_CARD_QUOTA: u64 = 8 * 1024;
        let budget = CONFIGURED_CARD_QUOTA.min(30_000);
        let capacity = card_ledger_capacity_mebibytes(
            Some(20_000),
            Some(CONFIGURED_CARD_QUOTA),
            budget,
            30_000,
            DRIVER,
        );
        assert!(
            capacity <= CONFIGURED_CARD_QUOTA,
            "额度变量 20 000 不越过配置给卡的 8 G：实际 {capacity}"
        );
        assert_eq!(
            capacity,
            CONFIGURED_CARD_QUOTA - DRIVER,
            "取配置额度减驱动那一份"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(
                Some(4_000),
                Some(CONFIGURED_CARD_QUOTA),
                budget,
                30_000,
                DRIVER
            ),
            4_000,
            "额度变量更小：取额度变量"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(
                None,
                Some(CONFIGURED_CARD_QUOTA),
                budget,
                30_000,
                DRIVER
            ),
            CONFIGURED_CARD_QUOTA - DRIVER,
            "没设额度变量：照旧是给崩溃放量的额度减驱动那一份"
        );
    }

    /// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（日志里那一行）：45 个状态的流只开 45 个，
    /// 131 195 个状态的大流照旧按显存开，调用方要得更少就照它的，一点显存都不给、流是空的也带一个。
    #[test]
    fn small_flow_opens_scratch_for_its_own_states_and_a_large_flow_still_follows_the_memory() {
        const STATES_A_WHOLE_RTX_5090_HOLDS: usize = 17_867;
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                usize::MAX,
                45
            ),
            45,
            "45 个状态的流只开 45 个状态的草稿区"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                usize::MAX,
                131_195
            ),
            STATES_A_WHOLE_RTX_5090_HOLDS,
            "比显存装得下的还多的流照显存开"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                256,
                131_195
            ),
            256,
            "调用方要的更少就照它的"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(0, usize::MAX, 0),
            1,
            "至少带一个"
        );
    }

    /// 设了卡的额度变量：容量取额度与「这个进程能占的 − 驱动那一份」里小的那个（额度小取额度，额度大取能占的那一边）；
    /// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份。数照一张 5090（空 30 000 MiB，3/4 是 22 500）。
    #[test]
    fn card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged(
    ) {
        assert_eq!(
            card_ledger_capacity_mebibytes(Some(4096), None, 22_500, 30_000, DRIVER),
            4096,
            "额度比能占的减驱动那一份小：取额度"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(Some(40_000), None, 22_500, 30_000, DRIVER),
            22_500 - DRIVER,
            "额度比 3/4 那一份减驱动那一份大：没配显卡优先级时 3/4 那一份代替配置的额度，取它（第 10v 批）"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(None, None, 22_500, 30_000, DRIVER),
            22_500 - DRIVER,
            "没设额度：照旧是给崩溃放量的额度减驱动那一份"
        );
    }

    /// 没配显卡优先级、设了额度变量 20 000、能占 30 000 MiB：没配额度时默认的 3/4 那一份（22 500）代替配置的额度，
    /// 并进最小值，容量不超过它减驱动那一份。
    #[test]
    fn an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share(
    ) {
        const MAY_HOLD: u64 = 30_000;
        let three_quarter_share = MAY_HOLD / 4 * 3;
        let capacity = card_ledger_capacity_mebibytes(
            Some(20_000),
            None,
            three_quarter_share,
            MAY_HOLD,
            DRIVER,
        );
        assert!(
            capacity <= three_quarter_share - DRIVER,
            "额度变量 20 000 不越过 3/4 那一份 {three_quarter_share} 减驱动那一份：实际 {capacity}"
        );
        assert_eq!(
            capacity,
            three_quarter_share - DRIVER,
            "三者里最小的是 3/4 那一份减驱动那一份"
        );
    }

    /// 按判器那一族的算法（表 1 MiB，每个状态 1 261 616 字节）在 56 MiB 里装得下 45 个状态（45 个要 56、46 个要 57）；
    /// 上限内全放得下就取上限，一个都放不下取 0。
    #[test]
    fn the_most_states_whose_need_fits_is_the_largest_count_whose_need_is_within_the_capacity() {
        let judge_needs = |states: usize| -> u64 {
            1 + (u64::try_from(states).expect("装得进 u64") * 1_261_616).div_ceil(1024 * 1024)
        };
        assert_eq!(judge_needs(45), 56);
        assert_eq!(judge_needs(46), 57);
        assert_eq!(
            most_states_whose_need_fits_the_capacity(100_000, 56, &judge_needs),
            45,
            "56 MiB 装 45 个状态"
        );
        assert_eq!(
            most_states_whose_need_fits_the_capacity(30, 56, &judge_needs),
            30,
            "上限内全放得下取上限"
        );
        assert_eq!(
            most_states_whose_need_fits_the_capacity(100_000, 0, &judge_needs),
            0,
            "一个都放不下取 0"
        );
    }

    /// 这几条用例里每张卡都带得了状态：调用方算出一个都带不了的那一支走不到。
    fn every_card_here_holds_a_state(card_slot: usize) -> ScratchAttemptOnACard {
        panic!("第 {card_slot} 张卡带得了状态，不该问它一个都带不了时算哪一种")
    }

    /// 「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张此刻借不到。
    /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张）。
    #[test]
    fn second_card_without_room_leaves_every_state_to_the_first_card() {
        let mut attempts_made = Vec::new();
        let attempts = place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| {
                attempts_made.push((card_slot, states_to_place));
                match card_slot {
                    0 => ScratchAttemptOnACard::Booked {
                        states: states_to_place,
                    },
                    _ => ScratchAttemptOnACard::NoRoomNow {
                        need_mebibytes: 2_000,
                    },
                }
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            attempts_made,
            vec![(0, 60), (1, 40)],
            "每张卡当场试一次：第一张装 60、第二张装剩下的 40"
        );
        assert_eq!(
            attempts,
            vec![
                ScratchAttemptOnACard::Booked { states: 60 },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 2_000
                },
            ],
            "第二张借不到就不装状态"
        );
        assert_eq!(
            decide_after_attempting_every_card(&attempts),
            ScratchPlacementDecision::JudgeOnTheCardsBooked,
            "第一张借到了：就在第一张上判"
        );
    }

    /// 借不到的卡那一份留给后面的卡。第一张此刻借不到，100 个状态里第二张装满它带得了的 60 个；
    /// 第一张借到的比要装的少（在锁里照此刻能借的压过，60 个里借到 30 个），第二张装的是剩下的 70 个与 60 里小的那个。
    #[test]
    fn the_states_a_card_did_not_book_go_to_the_next_card() {
        let first_card_without_room = place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| match card_slot {
                0 => ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 3_000,
                },
                _ => ScratchAttemptOnACard::Booked {
                    states: states_to_place,
                },
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            first_card_without_room[1],
            ScratchAttemptOnACard::Booked { states: 60 },
            "第一张借不到：第二张装满它带得了的 60 个"
        );
        let mut second_card_asked_for = 0;
        place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| match card_slot {
                0 => ScratchAttemptOnACard::Booked { states: 30 },
                _ => {
                    second_card_asked_for = states_to_place;
                    ScratchAttemptOnACard::Booked {
                        states: states_to_place,
                    }
                }
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            second_card_asked_for, 60,
            "第一张只借到 30 个：剩下的 70 个里第二张装它带得了的 60 个"
        );
    }

    /// 一张都没借到时，有空着时放得下的卡就是此刻借不到（要的是第一张放不下的卡那一笔，调用方报本次未跑），
    /// 每张卡空着时连一个状态都放不下就判红。
    #[test]
    fn with_no_card_booked_a_card_that_fits_when_empty_means_no_room_now_and_none_means_red() {
        assert_eq!(
            decide_after_attempting_every_card(&[
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 40
                },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 3_000
                },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 1_000
                },
            ]),
            ScratchPlacementDecision::NoRoomOnAnyCardNow {
                need_mebibytes: 3_000
            },
            "第二、三张空着时放得下：此刻借不到"
        );
        assert_eq!(
            decide_after_attempting_every_card(&[
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 40
                },
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 90
                },
            ]),
            ScratchPlacementDecision::NoCardHoldsOneState {
                largest_ceiling_mebibytes: 90
            },
            "每张都连一个状态都放不下：判红，带最大的那张空着时能借的"
        );
    }

    /// 第 ② 段核对与单元校验和的缓冲进账的量。事实表 3 MiB 多 1 字节上卡一次，一趟派活建 512 KiB 多 48 字节、
    /// 同时两趟活着：合起来 4 MiB 多 97 字节，向上取整 5 MiB；什么都不建是 0。
    #[test]
    fn buffers_held_at_once_are_the_uploaded_tables_plus_every_pass_still_alive_rounded_up() {
        const MEBIBYTE: u64 = 1024 * 1024;
        assert_eq!(
            mebibytes_of_buffers_held_at_once(3 * MEBIBYTE + 1, MEBIBYTE / 2 + 48, 2),
            5,
            "3 MiB + 1 + 2 × (512 KiB + 48) = 4 MiB + 97 字节，向上取整"
        );
        assert_eq!(
            mebibytes_of_buffers_held_at_once(0, MEBIBYTE, 64),
            64,
            "32 条线程各两趟：64 趟各 1 MiB"
        );
        assert_eq!(mebibytes_of_buffers_held_at_once(0, 0, 0), 0);
    }

    /// 两张卡：前一张此刻放得下就一直记在前一张上，前一张放不下这一笔才记到下一张；每张都放不下而空着时放得下的是此刻借不到，
    /// 比每张空着时都大的等多久都放不下。
    #[test]
    fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
        let room = |room_now_mebibytes: u64, ceiling_mebibytes: u64| CardRoomForThisProcess {
            room_now_mebibytes,
            ceiling_mebibytes,
        };
        assert_eq!(
            decide_booking_by_priority(&[room(29_000, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 0 },
            "两张都空着：记在优先的那张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(600, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 0 },
            "前一张剩的正好放得下：照旧记在前一张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(500, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 1 },
            "前一张放不下这一笔：才记到下一张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(20, 100), room(10, 50)], 30),
            BookingDecision::NoRoomNow,
            "两张此刻都放不下 30，而第一张空着时放得下：此刻借不到"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(100, 100), room(50, 50)], 101),
            BookingDecision::NeverFits {
                largest_ceiling_mebibytes: 100
            },
            "比最大的那张空着时还大：永远放不下"
        );
        assert_eq!(
            decide_booking_by_priority(&[], 1),
            BookingDecision::NeverFits {
                largest_ceiling_mebibytes: 0
            },
            "一张卡都没有：永远放不下"
        );
    }

    /// 一张卡此刻能借给这个进程的照 [`card_room_for_this_process`]：容量拿这个进程能占的算、减借着的；空着时能借的拿空卡上能用的
    /// （总量减保留）算、不减借着的。
    #[test]
    fn the_room_now_subtracts_the_leases_and_the_ceiling_takes_what_an_empty_card_has() {
        let room = card_room_for_this_process(
            CardMemoryReading {
                free_mebibytes: 20_000,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::NotListed,
            },
            BookedByThisProcessOnACard {
                leases_mebibytes: 3_000,
                driver_share: DriverShareOnTheCard::DeviceStarted,
            },
            DRIVER,
            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
        );
        assert_eq!(
            room,
            CardRoomForThisProcess {
                room_now_mebibytes: 20_000 - DRIVER - 3_000,
                ceiling_mebibytes: AN_EMPTY_RTX_5090_HAS - DRIVER,
            },
            "读不到实际占着的：驱动那一份不当已经在卡上，从现读的空闲里扣"
        );
    }

    /// （攻方乙形）：这个进程在卡上占着 9 000，其中有不走这份账的设备（读数记成分不出）。
    /// 起了设备、借着 4 000 还没建：能借的照读不到实际占着的算，不把那 9 000 当成借据已经建上的那一份。
    #[test]
    fn memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built() {
        let booked = BookedByThisProcessOnACard {
            leases_mebibytes: 4_000,
            driver_share: DriverShareOnTheCard::DeviceStarted,
        };
        let reading = |occupied_by_this_process| CardMemoryReading {
            free_mebibytes: 15_000,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process,
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 9_000 }),
                booked,
                DRIVER
            ),
            15_000,
            "分不出：借着的 4 000 当一点都没建，起好了设备的驱动那一份也不当在卡上（第 10z 批：实占多少分不出）"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::NotListed),
                booked,
                DRIVER
            ),
            15_000,
            "与没列这个进程同样算"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 9_000 }),
                booked,
                DRIVER
            ),
            15_000 + DRIVER + 4_000,
            "都走这份账时那 9 000 封顶到许出去的 6 560，才算借据建上了"
        );
    }

    /// （白红那一处）：借草稿区之前，每张卡此刻都带不了一个状态而至少一张空着时带得了，是此刻借不到
    /// （报本次未跑）；空着时也带不了（一张卡都没有也算）才判红；有一张此刻带得了就去借。
    #[test]
    fn with_no_card_holding_a_state_now_a_card_holding_one_when_empty_means_not_run_not_red() {
        assert_eq!(
            decide_scratch_before_booking(&[0, 0], &[0, 3]),
            ScratchBeforeBooking::NoCardHoldsAStateNow,
            "第二张空着时带得了 3 个：此刻借不到"
        );
        assert_eq!(
            decide_scratch_before_booking(&[0, 0], &[0, 0]),
            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
            "空着时也带不了：判红"
        );
        assert_eq!(
            decide_scratch_before_booking(&[], &[]),
            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
            "一张卡都没有：判红"
        );
        assert_eq!(
            decide_scratch_before_booking(&[0, 2], &[5, 5]),
            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
            "第二张此刻带得了 2 个：去借"
        );
    }

    /// （攻方乙附，数照攻方模型那一行：空 21 010，借着 8 000 还没建，驱动实占 600）：
    /// 设备起好了而报告里没列这个进程，驱动那一份不当整份在卡上：能借的是空闲减驱动那一份减借着的 10 450，不超过卡上此刻真能再给的
    /// 11 050（空闲减驱动还要长的 1 960 减借着的）；当整份在卡上是 13 010，多给 D − x。
    #[test]
    fn with_this_process_not_listed_a_started_device_leaves_room_for_its_whole_driver_share() {
        const FREE: u64 = 21_010;
        const LEASES: u64 = 8_000;
        const DRIVER_HOLDS_NOW: u64 = 600;
        let room = card_room_for_this_process(
            reading_of_an_rtx_5090(FREE),
            BookedByThisProcessOnACard {
                leases_mebibytes: LEASES,
                driver_share: DriverShareOnTheCard::DeviceStarted,
            },
            DRIVER,
            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
        );
        assert_eq!(room.room_now_mebibytes, FREE - DRIVER - LEASES);
        assert!(
            room.room_now_mebibytes <= FREE - (DRIVER - DRIVER_HOLDS_NOW) - LEASES,
            "不超过卡上此刻真能再给的：{room:?}"
        );
    }

    /// （正推 X4）：一个状态要的恰好等于空着时能借的，是此刻借不到（报本次未跑），不是放不下；
    /// 多 1 MiB 才是空着时也放不下。
    #[test]
    fn state_needing_exactly_what_an_empty_card_lends_is_no_room_now() {
        let room = CardRoomForThisProcess {
            room_now_mebibytes: 0,
            ceiling_mebibytes: 1_000,
        };
        assert_eq!(
            attempt_on_a_card_without_room_for_one_state(1_000, room),
            ScratchAttemptOnACard::NoRoomNow {
                need_mebibytes: 1_000
            }
        );
        assert_eq!(
            attempt_on_a_card_without_room_for_one_state(1_001, room),
            ScratchAttemptOnACard::CannotHoldOneState {
                ceiling_mebibytes: 1_000
            }
        );
    }
}
