//! 真可写单元数的搜索：journal 记录上限的模型、空间条件的上界、下界的收紧与删失。

use super::*;

/// 默认环的在飞记录上限把「空间很空」的状态的探测截住：记录数 = 单元数 + 固定的几条，固定那几条从拒绝里读（`records_of_the_publish − 试的单元数`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JournalModel {
    /// 不带 journal 记录上限就不会被拒的最大单元数；还没从拒绝里学到时 `None`。
    largest_units_below_the_record_limit: Option<u64>,
    records_beyond_one_per_unit: Option<u64>,
    /// 拒绝里报的在飞记录上限（条）：一次顺序写至少每单元一条记录，放行的单元数不会超过它，按需收紧拿它当下界的上界。
    pub(super) record_limit: Option<u64>,
}

impl JournalModel {
    pub(super) const NOT_LEARNED_YET: JournalModel = JournalModel {
        largest_units_below_the_record_limit: None,
        records_beyond_one_per_unit: None,
        record_limit: None,
    };

    fn learn_from(&mut self, tried_units: u64, records_of_the_publish: u64, record_limit: u64) {
        // 记录数 = 单元数 + 最后一个事务点名的非数据角色摊成的几条；后者随单元数与状态缓慢地变，所以这里学到的只是起点，
        // 探测到上限处被拒时由 [`probe_at_the_journal_cap`] 按超出的条数往下收。
        let extra_records = records_of_the_publish
            .checked_sub(tried_units)
            .unwrap_or_else(|| {
                stop_with(
                    "journal_records_fewer_than_units",
                    &format!("units={tried_units} records={records_of_the_publish}"),
                )
            });
        self.records_beyond_one_per_unit = Some(extra_records);
        self.largest_units_below_the_record_limit =
            Some(record_limit.saturating_sub(extra_records));
        self.record_limit = Some(record_limit);
    }
}

/// 一个状态上 T 的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WritableUnits {
    /// 二分到 T 放行、T + 1 因空间被拒。
    Exact {
        units: u64,
        non_monotone_probes_seen: bool,
    },
    /// 记录上限处放行（空间比记录上限还宽）：T 至少这么多个单元。登记字面按「被别的原因拒」记取不出。
    /// `lower_bound_was_tightened`：下界是不是往上收过（[`LowerBoundTightening`]）；没收的停在记录上限处放行的那个点，下界照样成立。
    AtLeastBecauseOfTheJournalCap {
        lower_bound_units: u64,
        lower_bound_was_tightened: bool,
    },
    Unreadable(String),
}

/// 一个状态上空间的两个必要条件给出的 T 的上界（单位单元）：装不下的必拒，所以 T 不超过它们；只用来定探测的起点，不进判定。
pub(super) fn upper_bound_units_by_the_space_conditions(
    pool: &Pool,
    reading: &StateReading,
) -> u64 {
    let formula_bound = reading.minimum_available_slots() / 2;
    let landing_bound = pool
        .session
        .allocator
        .devices
        .iter()
        .map(|map| {
            map.unblocked_slot_pairs_outside_the_cluster_segments(
                pool.session.allocator.cluster_segments(),
            )
            .0
        })
        .min()
        .expect("至少一块盘");
    formula_bound.min(landing_bound)
}

fn interior_points(low_exclusive: u64, high_exclusive: u64, count: usize) -> Vec<u64> {
    let gap = high_exclusive - low_exclusive;
    let mut points: Vec<u64> = (1..=count)
        .map(|index| {
            low_exclusive
                + gap * u64::try_from(index).expect("下标")
                    / (u64::try_from(count).expect("个数") + 1)
        })
        .filter(|point| *point > low_exclusive && *point < high_exclusive)
        .collect();
    points.dedup();
    points
}

/// 单元数的空间上界高过这个数时，先在这个状态上学一次 journal 记录上限（默认环的上限 65536 条在它之上）。
const UNITS_ABOVE_WHICH_THE_JOURNAL_CAP_IS_LEARNED: u64 = 60_000;

fn unreadable_because(verdict: &ProbeVerdict, units: u64) -> WritableUnits {
    WritableUnits::Unreadable(format!("units_{units}_{}", short_debug(verdict)))
}

/// 在 journal 记录上限附近探测：被记录上限拒时，按超出的条数把单元数往下收再试（最多 16 次），收到的单元数记回模型。交回最后试的单元数与它的结局。
fn probe_at_the_journal_cap<Prober: UnitsProbe>(
    prober: &Prober,
    model: &mut JournalModel,
    starting_units: u64,
) -> (u64, ProbeVerdict) {
    let mut units = starting_units;
    for _ in 0..16 {
        match prober.probe_one(units) {
            ProbeVerdict::JournalLimit {
                records_of_the_publish,
                record_limit,
            } => {
                let overshoot = records_of_the_publish.saturating_sub(record_limit).max(1);
                units = units.saturating_sub(overshoot);
                model.largest_units_below_the_record_limit = Some(units);
                model.record_limit = Some(record_limit);
            }
            other @ (ProbeVerdict::Admitted
            | ProbeVerdict::Completed
            | ProbeVerdict::RefusedForSpace
            | ProbeVerdict::Other(_)) => return (units, other),
        }
    }
    (
        units,
        ProbeVerdict::Other("journal_cap_did_not_settle".to_string()),
    )
}

/// 往上收下界时，一轮里步长翻倍的点最多取几个（最远 2^12 个单元之外；再远的点会撞上落点那一道，把收紧提前停掉）。
const TIGHTENING_DOUBLINGS_AT_MOST: usize = 12;

/// 往上收下界时最多跑几轮。
const TIGHTENING_ROUNDS_AT_MOST: usize = 64;

/// 记录上限处放行之后，把下界往上收到「再多一个单元就被记录数拒」。
/// 模型里的上限是在更远的单元数处学来的（[`JournalModel::learn_from`]），固定的那几条记录随单元数缓慢地变，所以它偏低，放行的点不一定贴着上限。
/// 先用 1、2、4、… 倍的步长往上找一个被记录数拒的点，再在放行点与被拒点之间 K 路收缩；
/// 同一轮里排在最小的被记录数拒的点之后的结局不看（探测次序是落点、记录数、式子，被拒点之上的结局被它盖住）；
/// 碰到放行与记录数拒之外的结局就停在当时的下界：放行的点已经证明 T 至少这么多，只是没收紧。
/// 收到的下界不写回模型：下一个状态的起探点只取学到的记录上限，与这一个状态收没收紧无关，按需收紧与一律收紧才在每个状态上给同样的起探点。
fn tighten_the_lower_bound_to_the_record_limit<Prober: UnitsProbe>(
    prober: &Prober,
    record_limit: Option<u64>,
    admitted_units: u64,
) -> u64 {
    let parallel = prober.parallel_probe_count().max(1);
    let doublings = parallel.min(TIGHTENING_DOUBLINGS_AT_MOST);
    let mut lowest_admitted = admitted_units;
    let mut lowest_over_the_record_limit: Option<u64> = None;
    let mut step: u64 = 1;
    for _ in 0..TIGHTENING_ROUNDS_AT_MOST {
        let points: Vec<u64> = match lowest_over_the_record_limit {
            Some(over) => {
                if over - lowest_admitted <= 1 {
                    break;
                }
                interior_points(lowest_admitted, over, parallel)
            }
            None => (0..doublings)
                .map(|index| lowest_admitted.saturating_add(step.saturating_mul(1u64 << index)))
                .collect(),
        };
        let verdicts = prober.probe_in_parallel(&points);
        let mut admitted_points = Vec::new();
        let mut smallest_over_the_record_limit = None;
        let mut stopped_by_another_verdict = false;
        for (point, verdict) in points.iter().zip(&verdicts) {
            match verdict {
                ProbeVerdict::Admitted => admitted_points.push(*point),
                ProbeVerdict::JournalLimit { .. } => {
                    smallest_over_the_record_limit = Some(*point);
                    break;
                }
                ProbeVerdict::Completed
                | ProbeVerdict::RefusedForSpace
                | ProbeVerdict::Other(_) => {
                    stopped_by_another_verdict = true;
                    break;
                }
            }
        }
        if let Some(point) = admitted_points.iter().copied().max() {
            lowest_admitted = lowest_admitted.max(point);
        }
        if let Some(point) = smallest_over_the_record_limit.filter(|point| *point > lowest_admitted)
        {
            lowest_over_the_record_limit =
                Some(lowest_over_the_record_limit.map_or(point, |over| over.min(point)));
        }
        if stopped_by_another_verdict {
            break;
        }
        if lowest_over_the_record_limit.is_none() {
            step = step.saturating_mul(1u64 << doublings);
        }
    }
    if let Some(limit) = record_limit {
        if lowest_admitted > limit {
            stop_with(
                "admitted_above_the_record_limit",
                &format!("admitted_units={lowest_admitted} record_limit={limit}"),
            );
        }
    }
    lowest_admitted
}

/// 截尾状态上收不收紧下界（登记第十二节修订 4）。判定只看「臂要的单元数 ≤ 下界」：收紧只把下界从当前值抬到不超过记录上限的某个值，
/// 所以只有要的单元数落在（当前下界, 记录上限] 里的臂，判定才可能从「未定」变成「一定不多报」；别的臂收不收紧判定都一样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum LowerBoundTightening {
    /// 每个截尾状态都收紧（单测拿它当对照）。
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "一律收紧只在单测里当对照，产物那一趟只走按需收紧")
    )]
    AtEveryCensoredState,
    /// 只在某条臂要的单元数落在（当前下界, 记录上限] 里时收紧；`units_asked_for_by_the_arms` 是这个状态上每条臂的 ⌈X ÷ 32634⌉。
    OnlyWhereAnArmJudgementCanChange {
        units_asked_for_by_the_arms: Vec<u64>,
    },
}

/// 收紧能不能改某条臂的判定：要的单元数落在（当前下界, 记录上限] 里。记录上限还没学到时按「能」算（照收）。
fn tightening_can_change_an_arm_judgement(
    current_lower_bound_units: u64,
    record_limit: Option<u64>,
    units_asked_for_by_the_arms: &[u64],
) -> bool {
    units_asked_for_by_the_arms.iter().any(|asked| {
        *asked > current_lower_bound_units && record_limit.is_none_or(|limit| *asked <= limit)
    })
}

/// 记录上限处放行之后的截尾结果：按 `tightening` 收不收紧下界。
fn censored_at_the_journal_cap<Prober: UnitsProbe>(
    prober: &Prober,
    model: &JournalModel,
    admitted_units: u64,
    tightening: &LowerBoundTightening,
) -> WritableUnits {
    let tightens = match tightening {
        LowerBoundTightening::AtEveryCensoredState => true,
        LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
            units_asked_for_by_the_arms,
        } => tightening_can_change_an_arm_judgement(
            admitted_units,
            model.record_limit,
            units_asked_for_by_the_arms,
        ),
    };
    let lower_bound_units = if tightens {
        tighten_the_lower_bound_to_the_record_limit(prober, model.record_limit, admitted_units)
    } else {
        admitted_units
    };
    WritableUnits::AtLeastBecauseOfTheJournalCap {
        lower_bound_units,
        lower_bound_was_tightened: tightens,
    }
}

/// 找这个状态上写得进的最大单元数 T（探测的搜索：K 路并行的区间收缩，K = 探测线程数）。截尾的状态上下界收不收紧照 `tightening`。
pub(super) fn find_writable_units<Prober: UnitsProbe>(
    prober: &Prober,
    space_bound: u64,
    model: &mut JournalModel,
    tightening: &LowerBoundTightening,
) -> WritableUnits {
    if model.largest_units_below_the_record_limit.is_none()
        && space_bound > UNITS_ABOVE_WHICH_THE_JOURNAL_CAP_IS_LEARNED
    {
        let learning_units = space_bound.min(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS);
        if let ProbeVerdict::JournalLimit {
            records_of_the_publish,
            record_limit,
        } = prober.probe_one(learning_units)
        {
            model.learn_from(learning_units, records_of_the_publish, record_limit);
        }
    }
    let journal_cap_before_the_first_probe = model.largest_units_below_the_record_limit;
    let seed = space_bound + 1;
    let first_units = match journal_cap_before_the_first_probe {
        Some(cap) if seed > cap => cap,
        Some(_) | None => seed,
    };
    let (first_units, first_verdict) = probe_at_the_journal_cap(prober, model, first_units);
    let journal_cap_after_the_first_probe = model.largest_units_below_the_record_limit;
    let (mut lowest_admitted, mut lowest_refused) = match first_verdict {
        ProbeVerdict::RefusedForSpace => (0u64, first_units),
        ProbeVerdict::Admitted => match journal_cap_after_the_first_probe {
            Some(cap) if first_units >= cap => {
                return censored_at_the_journal_cap(prober, model, first_units, tightening);
            }
            Some(cap) => {
                let (cap_units, cap_verdict) = probe_at_the_journal_cap(prober, model, cap);
                match cap_verdict {
                    ProbeVerdict::Admitted => {
                        return censored_at_the_journal_cap(prober, model, cap_units, tightening);
                    }
                    ProbeVerdict::RefusedForSpace => (first_units, cap_units),
                    other @ (ProbeVerdict::Completed
                    | ProbeVerdict::JournalLimit { .. }
                    | ProbeVerdict::Other(_)) => return unreadable_because(&other, cap_units),
                }
            }
            None => return unreadable_because(&ProbeVerdict::Admitted, first_units),
        },
        other @ (ProbeVerdict::Completed
        | ProbeVerdict::JournalLimit { .. }
        | ProbeVerdict::Other(_)) => return unreadable_because(&other, first_units),
    };
    let mut non_monotone_probes_seen = false;
    while lowest_refused - lowest_admitted > 1 {
        let points = interior_points(
            lowest_admitted,
            lowest_refused,
            prober.parallel_probe_count(),
        );
        assert!(
            !points.is_empty(),
            "区间 ({lowest_admitted}, {lowest_refused}) 里取不出内部点"
        );
        let verdicts = prober.probe_in_parallel(&points);
        let mut admitted_points = Vec::new();
        let mut refused_points = Vec::new();
        for (point, verdict) in points.iter().zip(&verdicts) {
            match verdict {
                ProbeVerdict::Admitted => admitted_points.push(*point),
                ProbeVerdict::RefusedForSpace => refused_points.push(*point),
                other @ (ProbeVerdict::Completed
                | ProbeVerdict::JournalLimit { .. }
                | ProbeVerdict::Other(_)) => return unreadable_because(other, *point),
            }
        }
        if let Some(smallest_refused) = refused_points.iter().copied().min() {
            lowest_refused = lowest_refused.min(smallest_refused);
        }
        let admitted_below_the_refused = admitted_points
            .iter()
            .copied()
            .filter(|point| *point < lowest_refused)
            .max();
        if admitted_points.iter().any(|point| *point > lowest_refused) {
            non_monotone_probes_seen = true;
        }
        if let Some(point) = admitted_below_the_refused {
            lowest_admitted = lowest_admitted.max(point);
        }
    }
    WritableUnits::Exact {
        units: lowest_admitted,
        non_monotone_probes_seen,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod search_tests;
