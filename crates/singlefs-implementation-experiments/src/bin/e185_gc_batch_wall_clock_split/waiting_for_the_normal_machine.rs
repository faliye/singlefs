use super::*;

// ───────────────────────────── 作废之后等常态（第 2 次重跑登记第十一节 R2-2） ─────────────────────────────

/// 「未入账超过挂钟 10%」作废的那一条的 `invalid_reason`（原登记第十一节「机器被别的活挤」）。
pub(super) const UNACCOUNTED_INVALID_REASON: &str =
    "unaccounted_time_above_ten_percent_of_the_wall";
/// 等常态时隔几秒做一次校准。
const SECONDS_BETWEEN_NORMAL_CHECKS: u64 = 10;
/// 连着几次校准比 ≤ 1.10 才重量。
const NORMAL_CHECKS_IN_A_ROW_FOR_THE_RETRY: usize = 2;
/// 单次至多等几秒（等满照样重量）。
const SECONDS_OF_ONE_WAIT_AT_MOST: u64 = 300;
/// 一段里累计至多等几秒（等满就停、退 5）。
const SECONDS_OF_WAITING_PER_SEGMENT_AT_MOST: u64 = 600;

/// 这一种作废之后要先等机器回到常态再重量（R2-2）；别的原因（挂载失败、钟读不到、扫描读为 0、拷回不成、校准锚点不符）当场重量。
pub(super) fn is_retried_only_after_the_machine_is_back_to_normal(invalid_reason: &str) -> bool {
    invalid_reason == UNACCOUNTED_INVALID_REASON
        || invalid_reason == CALIBRATION_RATIO_INVALID_REASON
}

/// 一次等常态的结局。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NormalWaitOutcome {
    /// 连着两次校准比 ≤ 1.10。
    BackToNormal,
    /// 这一次等满 300 秒，照样重量（重量的那一次照样受 R2-1 判）。
    OneWaitCapReached,
    /// 一段里累计等满 600 秒：打 `name=stopped`、收尾、退 5。
    SegmentWaitCapReached,
}

impl NormalWaitOutcome {
    pub(super) const fn label(self) -> &'static str {
        match self {
            NormalWaitOutcome::BackToNormal => "back_to_normal",
            NormalWaitOutcome::OneWaitCapReached => "one_wait_cap_reached_retrying_anyway",
            NormalWaitOutcome::SegmentWaitCapReached => "segment_wait_cap_reached",
        }
    }

    /// 这个结局要不要停下这一段：要停交回退出码（等满上限仍没等到，同别的性能测量那一条的退出码）。
    pub(super) const fn exit_code_stopping_the_segment(self) -> Option<i32> {
        match self {
            NormalWaitOutcome::BackToNormal | NormalWaitOutcome::OneWaitCapReached => None,
            NormalWaitOutcome::SegmentWaitCapReached => {
                Some(EXIT_CODE_OTHER_MEASUREMENT_STILL_RUNNING)
            }
        }
    }
}

/// 一次等常态的经过。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalWait {
    pub(super) checks: usize,
    pub(super) waited_seconds: u64,
    pub(super) ratio_per_check: Vec<Option<f64>>,
    pub(super) outcome: NormalWaitOutcome,
}

/// 一段里等过的账（600 秒上限与第八节轨迹）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WaitingLedger {
    pub(super) total_seconds: u64,
    pub(super) longest_seconds: u64,
    pub(super) waits: usize,
}

/// R2-2：这一种作废之后先等常态再重量——每 10 秒做一次校准（`calibrate` 交回 CRC 那份的校准比，读不到是 `None`），连着两次 ≤ 1.10 才回去
/// 重量；单次等满 300 秒照样重量；一段里累计等满 600 秒交回 [`NormalWaitOutcome::SegmentWaitCapReached`]。不用等的原因交回 `None`、不睡。
/// 钟与校准源从参数进来（单测换成假的，不真睡）；记账写进 `ledger`。
pub(super) fn wait_before_the_retry(
    invalid_reason: &str,
    ledger: &mut WaitingLedger,
    calibrate: &mut dyn FnMut() -> Option<f64>,
    sleep: &mut dyn FnMut(Duration),
) -> Option<NormalWait> {
    if !is_retried_only_after_the_machine_is_back_to_normal(invalid_reason) {
        return None;
    }
    let mut wait = NormalWait {
        checks: 0,
        waited_seconds: 0,
        ratio_per_check: Vec::new(),
        outcome: NormalWaitOutcome::OneWaitCapReached,
    };
    let mut normal_checks_in_a_row = 0_usize;
    // 迭代上界：每一圈 waited_seconds 加 10，到 300 必出去（30 圈）；一段累计到 600 也出去。跨圈的状态：normal_checks_in_a_row =
    // 到这一圈为止连着 ≤ 1.10 的次数，读不到或超过 1.10 归零。
    loop {
        sleep(Duration::from_secs(SECONDS_BETWEEN_NORMAL_CHECKS));
        wait.waited_seconds += SECONDS_BETWEEN_NORMAL_CHECKS;
        ledger.total_seconds += SECONDS_BETWEEN_NORMAL_CHECKS;
        let ratio = calibrate();
        wait.checks += 1;
        wait.ratio_per_check.push(ratio);
        if ratio.is_some_and(|value| !is_the_calibration_ratio_above_the_edge(value)) {
            normal_checks_in_a_row += 1;
        } else {
            normal_checks_in_a_row = 0;
        }
        if normal_checks_in_a_row >= NORMAL_CHECKS_IN_A_ROW_FOR_THE_RETRY {
            wait.outcome = NormalWaitOutcome::BackToNormal;
            break;
        }
        if ledger.total_seconds >= SECONDS_OF_WAITING_PER_SEGMENT_AT_MOST {
            wait.outcome = NormalWaitOutcome::SegmentWaitCapReached;
            break;
        }
        if wait.waited_seconds >= SECONDS_OF_ONE_WAIT_AT_MOST {
            wait.outcome = NormalWaitOutcome::OneWaitCapReached;
            break;
        }
    }
    ledger.waits += 1;
    ledger.longest_seconds = ledger.longest_seconds.max(wait.waited_seconds);
    Some(wait)
}

#[cfg(test)]
mod tests;
