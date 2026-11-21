//! 一次派活带几个状态的定法：一条草稿缓冲装几个、给的显存一次装得下几个、量宽度的停止判据与拐点、慢了减半快了加倍、
//! 拆不开的下限与停在下限上报错，记账切片的宽度；一张卡给一个判器要多少显存。都是不碰卡的算法，调用方与派活宽度调度照它们定宽度。

use super::*;

/// 量宽度：同一段状态从一条草稿缓冲那么宽起，一档加一条缓冲地派，`samples` 是量过的各档（带几个状态、卡算了几秒）。
/// 下一档还量不量：最新一档已经到这张卡带得了的上限、它最久的那一次提交超过 `seconds_the_longest_submission_may_take` 的一半
/// （`seconds_of_the_longest_submission_of_the_latest`：驱动掐的是卡上一段不可抢占的活，一次派活拆成几趟、几片提交，比的是最久的那一次提交，
/// 不是整次派活的用时；上限照 [`longest_tolerated_submission_seconds`]：量过拆不开的下限是 [`SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT`]，
/// 量宽度停在它的一半，与加宽的门槛 [`widening_keeps_the_longest_submission_tolerable`] 同一条线）、
/// 或每个状态的用时比前面最好那一档多出 [`CALIBRATION_WORSE_THAN_THE_BEST`] 倍，就不量了。
/// 一次派活带的状态多到一定程度，每个状态的用时猛涨（5090 过了 21 700 个、5060 Ti 过了 6 510 个，那时显存远没用满；
/// 拐点随状态轻重挪），所以宽度在卡上现量，不按显存算。
#[must_use]
pub fn calibration_goes_on(
    samples: &[(usize, f64)],
    seconds_of_the_longest_submission_of_the_latest: f64,
    widest_possible: usize,
    seconds_the_longest_submission_may_take: f64,
) -> bool {
    let Some((&(width, seconds), earlier)) = samples.split_last() else {
        return true;
    };
    if width >= widest_possible
        || seconds_of_the_longest_submission_of_the_latest
            > seconds_the_longest_submission_may_take / 2.0
    {
        return false;
    }
    let best_earlier = earlier
        .iter()
        .map(|(earlier_width, earlier_seconds)| earlier_seconds / *earlier_width as f64)
        .fold(f64::INFINITY, f64::min);
    seconds / width as f64 <= best_earlier * CALIBRATION_WORSE_THAN_THE_BEST
}

/// 量过的各档里每个状态用时最短的那一档带几个状态。
///
/// # Panics
/// 一档都没量。
#[must_use]
pub fn calibrated_width(samples: &[(usize, f64)]) -> usize {
    samples
        .iter()
        .min_by(|(left_width, left_seconds), (right_width, right_seconds)| {
            (left_seconds / *left_width as f64).total_cmp(&(right_seconds / *right_width as f64))
        })
        .expect("至少量了一档")
        .0
}

/// 量完一次之后，以后量宽度最宽试到几个状态之前：最宽那一档不是选中的那一档（它每个状态更慢、或单次太久），
/// 以后就不再试到它——过了拐点的那一档一次能慢到几秒（第二台的 5060 Ti 上 8 680 个要 3.6 秒），每次重量都试它既费时又可能撞驱动的超时。
#[must_use]
pub fn knee_after_calibration(samples: &[(usize, f64)], knee: usize) -> usize {
    let widest = samples.last().expect("至少量了一档").0;
    if widest == calibrated_width(samples) {
        knee
    } else {
        knee.min(widest)
    }
}

/// 一次派活最久那一次提交超过许的上限（[`longest_tolerated_submission_seconds`]；驱动会掐掉跑得太久的派活，内核日志 Xid 109）：下一次减半，
/// 不少于 [`NARROWEST_DISPATCH`]、不超过容量。
#[must_use]
pub fn width_after_a_slow_dispatch(width: usize, capacity: usize) -> usize {
    (width / 2).clamp(NARROWEST_DISPATCH.min(capacity).max(1), capacity.max(1))
}

/// 拆不开的下限（宽度到 [`NARROWEST_DISPATCH`] 时那一次派活最久那一次提交几秒）在一次派活（实际带了 `states_carried` 个状态，
/// 最久那一次提交 `seconds_of_the_longest_submission` 秒）之后记成多少：
/// - 只带了 [`NARROWEST_DISPATCH`] 个状态：就是这一次的（状态变重变轻，下限跟着挪）。
/// - 带得更多、前面的派活量过下限：取两者小的那个。少带几个状态算得不会更久，这一次比下限还快说明状态变轻了，下限至多是这一次的。
/// - 带得更多、前面的派活没量过：还是没量过。
#[must_use]
pub fn indivisible_floor_after_a_dispatch(
    seconds_of_the_indivisible_floor: Option<f64>,
    states_carried: usize,
    seconds_of_the_longest_submission: f64,
) -> Option<f64> {
    if states_carried <= NARROWEST_DISPATCH {
        Some(seconds_of_the_longest_submission)
    } else {
        seconds_of_the_indivisible_floor
            .map(|floor_seconds| floor_seconds.min(seconds_of_the_longest_submission))
    }
}

/// 一次派活最久那一次提交许算多久（秒）：量过拆不开的下限（`is_the_indivisible_floor_measured`）的，是驱动会掐的那条线
/// [`SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT`]，不看下限多长、过没过 [`LONGEST_DISPATCH_SECONDS`]（下限本身过了那条线时宽度停在下限，
/// 见 [`dispatches_stuck_at_the_floor_after_a_dispatch`]）；没量过的是 [`LONGEST_DISPATCH_SECONDS`]。
#[must_use]
pub fn longest_tolerated_submission_seconds(is_the_indivisible_floor_measured: bool) -> f64 {
    if is_the_indivisible_floor_measured {
        SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT
    } else {
        LONGEST_DISPATCH_SECONDS
    }
}

/// 最久那一次提交 `seconds_of_the_longest_submission` 秒的一次派活之后加一倍宽，最久那一次提交会不会越过许的上限
/// （[`longest_tolerated_submission_seconds`]）：用时照状态数翻倍也不越过的才加宽。两条满足一条就加宽：
/// - 不到 [`LONGEST_DISPATCH_SECONDS`] 的四分之一：加倍之后还不到它的一半。
/// - 量过拆不开的下限（`is_the_indivisible_floor_measured`），又不超过 [`SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT`] 的一半：加倍之后不过那条线。
///   不拿下限的倍数比：状态在卡上并行时加宽几乎不加时，宽度要一路涨上去；状态互相拖慢、用时随宽度成正比时，
///   宽度停在最久那一次提交过了那条线一半的那一档，加宽那一步不会把它推过那条线，也不在两档之间来回跳。
#[must_use]
pub fn widening_keeps_the_longest_submission_tolerable(
    seconds_of_the_longest_submission: f64,
    is_the_indivisible_floor_measured: bool,
) -> bool {
    seconds_of_the_longest_submission < LONGEST_DISPATCH_SECONDS / 4.0
        || (is_the_indivisible_floor_measured
            && seconds_of_the_longest_submission <= SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT / 2.0)
}

/// 一次派活（此刻的宽度 `width`，实际带了 `states_carried` 个状态，最久那一次提交 `seconds_of_the_longest_submission` 秒，
/// 照这一次记过之后量没量过拆不开的下限是 `is_the_indivisible_floor_measured`）之后的宽度：
/// - 超过 [`longest_tolerated_submission_seconds`]：照实际带的状态数减半（[`width_after_a_slow_dispatch`]）。这一段剩的状态比宽度少时，按宽度减半还是太宽。
/// - 加一倍宽不越过它（[`widening_keeps_the_longest_submission_tolerable`]），这一次又带满了宽度，宽度还没到一条草稿缓冲：加倍，不超过一条草稿缓冲、不超过容量；
///   没带满的那一次说不出带满了多久，不加宽。
/// - 其余不动。到了一条草稿缓冲往上由量宽度定（[`calibration_goes_on`]），这里不加宽。
#[must_use]
pub fn dispatch_width_after_a_dispatch(
    width: usize,
    states_carried: usize,
    seconds_of_the_longest_submission: f64,
    states_per_lane: usize,
    capacity: usize,
    is_the_indivisible_floor_measured: bool,
) -> usize {
    let widening_keeps_the_longest_submission_under_the_tolerated_line =
        widening_keeps_the_longest_submission_tolerable(
            seconds_of_the_longest_submission,
            is_the_indivisible_floor_measured,
        );
    if seconds_of_the_longest_submission
        > longest_tolerated_submission_seconds(is_the_indivisible_floor_measured)
    {
        width_after_a_slow_dispatch(states_carried, capacity)
    } else if widening_keeps_the_longest_submission_under_the_tolerated_line
        && states_carried == width
        && width < states_per_lane
    {
        width
            .saturating_mul(2)
            .min(states_per_lane)
            .min(capacity.max(1))
    } else {
        width
    }
}

/// 宽度停在拆不开的下限上连着几次派活：这一次只带了 [`NARROWEST_DISPATCH`] 个状态、之后的宽度 `next_width` 还是它（加不了宽），
/// 最久那一次提交又超过 [`LONGEST_DISPATCH_SECONDS`]，就在 `stuck_before`（前面连着几次）上加一；别的一律从 0 重数。
/// 到 [`DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING`] 次判器报错（[`StuckAtTheIndivisibleFloor`]），不再一个一个派下去。
/// 只带 1 个状态的那一次就量了下限，之后的宽度照量过下限的判法定（[`dispatch_width_after_a_dispatch`]）：不过驱动那条线的一半就加宽，
/// 过了那条线减半还是 1，过了一半、没过那条线的不动。所以数得上的是两种：单个状态就过了那条线的一半（再加宽，状态互相拖慢时就过线），
/// 与一条草稿缓冲或容量只装得下 1 个状态、加不了宽而单个状态过了 [`LONGEST_DISPATCH_SECONDS`] 的。
#[must_use]
pub fn dispatches_stuck_at_the_floor_after_a_dispatch(
    stuck_before: u64,
    states_carried: usize,
    seconds_of_the_longest_submission: f64,
    next_width: usize,
) -> u64 {
    if states_carried <= NARROWEST_DISPATCH
        && next_width <= NARROWEST_DISPATCH
        && seconds_of_the_longest_submission > LONGEST_DISPATCH_SECONDS
    {
        stuck_before + 1
    } else {
        0
    }
}

/// 判器停在拆不开的下限上：宽度到了 [`NARROWEST_DISPATCH`]、加不了宽，最久那一次提交连着 [`DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING`] 次
/// 超过 [`LONGEST_DISPATCH_SECONDS`]。这一段序号判到 `next_ordinal` 之前为止（之后的没判），这个判器一共判了 `states_judged_by_the_judge` 个状态。
#[derive(Clone, Debug, PartialEq)]
pub struct StuckAtTheIndivisibleFloor {
    pub card_name: String,
    /// 最后那一次（宽度在下限上）最久那一次提交几秒。
    pub seconds_of_the_floor: f64,
    pub dispatches_in_a_row: u64,
    pub next_ordinal: u64,
    pub states_judged_by_the_judge: u64,
    /// 这条流一共几个状态。
    pub state_count: u64,
}

impl StuckAtTheIndivisibleFloor {
    /// 可 grep 的一行：[`STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX`] 起头，`flow=` 只在调用方知道流名时有（判器不知道判的是哪条流；
    /// 崩溃放量判段照 `crate::crash_amplification::gpu_judging_refusal_naming_the_flow` 补上）。
    #[must_use]
    pub fn line(&self, flow_name: Option<&str>) -> String {
        let flow = flow_name.map_or_else(String::new, |name| format!(" flow={name}"));
        format!(
            "{STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX}{flow} card={:?} seconds_of_the_floor={:.3} dispatches_in_a_row={} \
             longest_dispatch_seconds={LONGEST_DISPATCH_SECONDS} driver_line_seconds={SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT} \
             next_ordinal={} states_judged_by_the_judge={} state_count={} \
             说明=宽度到了 1 个状态还拆不开，单个状态的提交就过了驱动会掐的那条线的一半（或这张卡一次只带得了 1 个状态），加宽要冒被驱动掐通道（Xid 109）的险，一个一个派每秒判不到一个状态；\
             这一趟报错退出，不再派下去。出路：从内核这一侧缩短单个状态那一趟（看 GPU_LIFECYCLE 行里哪一趟最久）",
            self.card_name,
            self.seconds_of_the_floor,
            self.dispatches_in_a_row,
            self.next_ordinal,
            self.states_judged_by_the_judge,
            self.state_count
        )
    }
}

/// 记账分片交完、等完一片（卡算了 `seconds` 秒）之后下一片的片宽（几个位置）：多于 [`LONGEST_DISPATCH_SECONDS`] 的四分之一（0.125 秒）减半，
/// 不窄过 [`NARROWEST_ACCOUNTING_SLICE`]；少于它的十六分之一（0.03125 秒）加倍，不宽过 [`ACCOUNTING_SLICE_POSITIONS`]；中间不动。
/// 门槛比每片的上限 0.25 秒（它的一半）低一倍：片宽只在一片算完之后才调，相邻几片每个位置的用时能差到两倍多
/// （落点 10124 那张表上照 0.25 秒那一档调，2048 个位置的一片从 0.184 秒跳到 0.283 秒）。轻的流头一次派活六片之内涨到一轮一片。
#[must_use]
pub fn accounting_slice_width_after_a_slice(width: u32, seconds: f64) -> u32 {
    if seconds > LONGEST_DISPATCH_SECONDS / 4.0 {
        (width / 2).max(NARROWEST_ACCOUNTING_SLICE)
    } else if seconds < LONGEST_DISPATCH_SECONDS / 16.0 {
        width.saturating_mul(2).min(ACCOUNTING_SLICE_POSITIONS)
    } else {
        width
    }
}

/// 一张卡按给它的显存（MiB）一次派活带得了几个状态：表与留给驱动的那一份先扣掉，剩下的装草稿区。
/// 本机的卡传它的 [`GpuCard::memory_budget_mebibytes`]；别的机器上的卡按配置的额度算（绑定上限按本机的卡估）。
#[must_use]
pub fn states_in_flight_within(
    maximum_storage_buffer_binding_size: u64,
    memory_mebibytes: u64,
    tables: &PackedJudgeTables,
) -> usize {
    let scratch_memory_mebibytes = memory_mebibytes
        .saturating_sub(tables.mebibytes())
        .saturating_sub(MEBIBYTES_LEFT_FOR_THE_DRIVER);
    states_per_dispatch_within_the_scratch_memory(
        maximum_storage_buffer_binding_size,
        scratch_memory_mebibytes,
    )
}

/// 一个判器在卡上要占多少显存（MiB，向上取整），不算 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 那一份（那一份每张卡只付一次，归卡池的账）：
/// 表，加一次派活带 `requested_states_per_dispatch` 与这条流状态数里小的那个（至少 1）的草稿区，
/// 加每次派活的结论与回读缓冲（每个状态各 [`VERDICT_WORDS`] 个字）。用例向卡池借卡时按它借
/// （`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_or_report_not_run`，此刻借不到排队等这个进程别的用例还卡，没有可等的才报本次未跑）；
/// 借来的卡额度是它加那一份，判器照 [`states_in_flight_within`] 开出来的草稿区不少于这里算进去的状态数。
#[must_use]
pub fn mebibytes_a_judge_needs(
    tables: &PackedJudgeTables,
    requested_states_per_dispatch: usize,
) -> u64 {
    let states = u64::try_from(states_in_one_dispatch_capped_by_the_state_count(
        usize::MAX,
        requested_states_per_dispatch,
        tables.state_count,
    ))
    .expect("装得进 u64");
    let bytes_per_state =
        u64::try_from(SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4).expect("装得进 u64");
    tables.mebibytes() + (states * bytes_per_state).div_ceil(1024 * 1024)
}

/// 一条草稿缓冲装几个状态：每个状态 `SCRATCH_WORDS × 4` 字节，一条不超过这张卡的存储绑定上限；至少 1。
#[must_use]
pub fn states_per_scratch_lane(maximum_storage_buffer_binding_size: u64) -> usize {
    let per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
    usize::try_from(maximum_storage_buffer_binding_size / per_state)
        .unwrap_or(usize::MAX)
        .max(1)
}

/// 一张卡一次派活带几个状态：草稿区不超过给它的显存（MiB），也不超过 [`MAXIMUM_SCRATCH_LANES`] 条缓冲装得下的；至少 1。
#[must_use]
pub fn states_per_dispatch_within_the_scratch_memory(
    maximum_storage_buffer_binding_size: u64,
    scratch_memory_mebibytes: u64,
) -> usize {
    let per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
    let by_memory =
        usize::try_from(scratch_memory_mebibytes * 1024 * 1024 / per_state).unwrap_or(usize::MAX);
    let by_lanes =
        states_per_scratch_lane(maximum_storage_buffer_binding_size) * MAXIMUM_SCRATCH_LANES;
    by_memory.min(by_lanes).max(1)
}

#[cfg(test)]
mod tests;
