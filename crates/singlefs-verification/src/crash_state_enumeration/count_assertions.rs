//! 按全量域钉的计数断言：闭式状态数、全量域的钉值、大于零与大于下限，缩过域时不判、打那一行。

use super::*;

/// 缩过域那一趟的闭式态数断言不判时打的那一行的头（[`ClosedFormStateCountJudgement::line`]）。
pub const CLOSED_FORM_NOT_JUDGED_LINE_PREFIX: &str = "CRASH_CASE_CLOSED_FORM_NOT_JUDGED";

/// 崩溃枚举用例「枚举到的态数等于闭式」那一条断言这一趟判了没有。闭式按用例自己的展开算（全量域；态数、按发布分的态数都算），
/// 缩过域的那一趟数出来的本来就比它少。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosedFormStateCountJudgement {
    /// 没缩域：判了，枚举到的等于闭式。
    Judged,
    /// 缩过域（[`crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 设了）：闭式不判；每个崩溃块照判（在枚举那一趟里判，与这一条无关）。
    NotJudgedBecauseTheDomainWasNarrowed {
        /// 这一趟的展开上限（变量的值）。
        expand_up_to: usize,
        /// 这一趟实际展开、枚举到的（`Debug` 的写法）。
        counted: String,
        /// 全量域的闭式（`Debug` 的写法；这一趟不判）。
        closed_form: String,
    },
}

impl ClosedFormStateCountJudgement {
    /// 缩过域的那一趟打的结果行：`CRASH_CASE_CLOSED_FORM_NOT_JUDGED expand_up_to=<n> counted=<这一趟实际展开、枚举到的> closed_form=<全量域的闭式> what=<哪一条>`；
    /// 判了的那一趟不打（`None`）。
    #[must_use]
    pub fn line(&self, what: &str) -> Option<String> {
        match self {
            Self::Judged => None,
            Self::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to,
                counted,
                closed_form,
            } => Some(format!(
                "{CLOSED_FORM_NOT_JUDGED_LINE_PREFIX} expand_up_to={expand_up_to} counted={counted} closed_form={closed_form} what={what}"
            )),
        }
    }
}

/// 崩溃枚举用例「枚举到的态数等于闭式」那一条断言（态数，或按发布分的态数那一串），各用例都经它判：照这个进程的环境
/// （[`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]），没缩域照判，缩过域不判、往 stdout 打结果行
/// （写明缩过、这一趟的展开上限与实际展开、枚举到的）。
///
/// # Panics
/// 没缩域而 `counted` 不等于 `closed_form`；缩域变量写坏了（同 [`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]）。
#[track_caller]
pub fn assert_the_counted_states_equal_the_closed_form_unless_narrowed<Counted, ClosedForm>(
    counted: Counted,
    closed_form: ClosedForm,
    what: &str,
) -> ClosedFormStateCountJudgement
where
    Counted: PartialEq<ClosedForm> + std::fmt::Debug,
    ClosedForm: std::fmt::Debug,
{
    let judgement = closed_form_state_count_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        closed_form,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_counted_states_equal_the_closed_form_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不等于 `closed_form`。
#[track_caller]
pub fn closed_form_state_count_judged_under<Counted, ClosedForm>(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: Counted,
    closed_form: ClosedForm,
    what: &str,
) -> ClosedFormStateCountJudgement
where
    Counted: PartialEq<ClosedForm> + std::fmt::Debug,
    ClosedForm: std::fmt::Debug,
{
    match narrowing {
        None => {
            assert_eq!(counted, closed_form, "{what}：枚举到的要等于闭式");
            ClosedFormStateCountJudgement::Judged
        }
        Some(limit) => ClosedFormStateCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: format!("{counted:?}"),
            closed_form: format!("{closed_form:?}"),
        },
    }
}

/// 缩过域那一趟按全量域钉的计数断言不判时打的那一行的头（[`FullDomainCountJudgement::line`]）。
pub const FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX: &str =
    "CRASH_CASE_FULL_DOMAIN_COUNT_NOT_JUDGED";

/// 崩溃枚举用例里按全量域钉的别的计数——逐条不变量的评估数（`CrashPointReplayTally::checker_evaluated_states`）、观察者计数
/// （`CrashPointReplayTally::observer_counts`）、oracle 与记录核对器那几格（根已持久、读出文件、两遍恢复不同……）、按用例自己的展开现算的数——
/// 那一条断言这一趟判了没有。钉值按用例自己的展开算（全量域）：等于钉的数，或「全量域上多于某个数」（至少一个、多于一个、多于另一格）；
/// 缩过域的那一趟数出来的本来就比它少，同 [`ClosedFormStateCountJudgement`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FullDomainCountJudgement {
    /// 没缩域：判了。
    Judged,
    /// 缩过域：这一条不判；每个崩溃块照判（在枚举那一趟里判，与这一条无关）。
    NotJudgedBecauseTheDomainWasNarrowed {
        /// 这一趟的展开上限（变量的值）。
        expand_up_to: usize,
        /// 这一趟实际展开、数出来的（`Debug` 的写法）。
        counted: String,
        /// 全量域上要的：钉的数（`Debug` 的写法）、[`FULL_DOMAIN_EXPECTS_ABOVE_ZERO`]，或 [`FULL_DOMAIN_EXPECTS_ABOVE_PREFIX`] 接下界。
        full_domain_expects: String,
    },
}

/// 「全量域上至少一个」那一种断言在结果行 `full_domain_expects=` 里的写法。
pub const FULL_DOMAIN_EXPECTS_ABOVE_ZERO: &str = "above_zero";

/// 「全量域上多于下界」那一种断言在结果行 `full_domain_expects=` 里的写法：这个头接下界的十进制（`above_1`）。
pub const FULL_DOMAIN_EXPECTS_ABOVE_PREFIX: &str = "above_";

impl FullDomainCountJudgement {
    /// 缩过域的那一趟打的结果行：`CRASH_CASE_FULL_DOMAIN_COUNT_NOT_JUDGED expand_up_to=<n> counted=<这一趟数出来的> full_domain_expects=<全量域上要的> what=<哪一条>`；
    /// 判了的那一趟不打（`None`）。
    #[must_use]
    pub fn line(&self, what: &str) -> Option<String> {
        match self {
            Self::Judged => None,
            Self::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to,
                counted,
                full_domain_expects,
            } => Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to={expand_up_to} counted={counted} full_domain_expects={full_domain_expects} what={what}"
            )),
        }
    }
}

/// 崩溃枚举用例里按全量域钉了数的计数（评估数、观察者计数、oracle 与记录核对器那几格、按全量的计划现算的数；一个数，或几个数拼成的一组）
/// 那一条断言，各用例都经它判：照这个进程的环境
/// （[`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]），没缩域照判，缩过域不判、往 stdout 打结果行。
/// 和同一趟数出来的别的数比的（`== tally.states`、几格之和等于态数）缩域时照样成立，不经它。
///
/// # Panics
/// 没缩域而 `counted` 不等于 `pinned`；缩域变量写坏了（同 [`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]）。
#[track_caller]
pub fn assert_the_count_equals_its_full_domain_pin_unless_narrowed<Counted, Pinned>(
    counted: Counted,
    pinned: Pinned,
    what: &str,
) -> FullDomainCountJudgement
where
    Counted: PartialEq<Pinned> + std::fmt::Debug,
    Pinned: std::fmt::Debug,
{
    let judgement = count_equal_to_its_full_domain_pin_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        pinned,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_equals_its_full_domain_pin_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不等于 `pinned`。
#[track_caller]
pub fn count_equal_to_its_full_domain_pin_judged_under<Counted, Pinned>(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: Counted,
    pinned: Pinned,
    what: &str,
) -> FullDomainCountJudgement
where
    Counted: PartialEq<Pinned> + std::fmt::Debug,
    Pinned: std::fmt::Debug,
{
    match narrowing {
        None => {
            assert_eq!(counted, pinned, "{what}：数出来的要等于按全量域钉的数");
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: format!("{counted:?}"),
            full_domain_expects: format!("{pinned:?}"),
        },
    }
}

/// 崩溃枚举用例里「全量域上至少一个状态评估过这条不变量 / 跑到过这一格观察者计数」那一条断言，各用例都经它判：
/// 照这个进程的环境，没缩域照判，缩过域不判（缩掉的段里的那几格可以一个都不剩）、往 stdout 打结果行。
///
/// # Panics
/// 没缩域而 `counted` 是 0；缩域变量写坏了。
#[track_caller]
pub fn assert_the_count_is_above_zero_unless_narrowed(
    counted: u64,
    what: &str,
) -> FullDomainCountJudgement {
    let judgement = count_above_zero_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_is_above_zero_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 是 0。
#[track_caller]
pub fn count_above_zero_judged_under(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: u64,
    what: &str,
) -> FullDomainCountJudgement {
    match narrowing {
        None => {
            assert!(counted > 0, "{what}：全量域上至少一个，数出来的是 0");
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: counted.to_string(),
            full_domain_expects: FULL_DOMAIN_EXPECTS_ABOVE_ZERO.to_string(),
        },
    }
}

/// 崩溃枚举用例里「全量域上数出来的多于下界」那一条断言（多于一个状态、态数多于读得出新号的那一格这类：下界是 1，或同一趟数出来的另一格），
/// 各用例都经它判：照这个进程的环境，没缩域照判，缩过域不判（缩掉的段里可以一个都不剩，或只剩那一格的状态）、往 stdout 打结果行。
///
/// # Panics
/// 没缩域而 `counted` 不多于 `floor`；缩域变量写坏了。
#[track_caller]
pub fn assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(
    counted: u64,
    floor: u64,
    what: &str,
) -> FullDomainCountJudgement {
    let judgement = count_exceeding_its_full_domain_floor_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        floor,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_exceeds_its_full_domain_floor_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不多于 `floor`。
#[track_caller]
pub fn count_exceeding_its_full_domain_floor_judged_under(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: u64,
    floor: u64,
    what: &str,
) -> FullDomainCountJudgement {
    match narrowing {
        None => {
            assert!(
                counted > floor,
                "{what}：全量域上要多于 {floor}，数出来的是 {counted}"
            );
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: counted.to_string(),
            full_domain_expects: format!("{FULL_DOMAIN_EXPECTS_ABOVE_PREFIX}{floor}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_amplification::CrashAmplificationEnvironment;

    /// 不缩域（守卫清掉缩域变量）：各用例经它判的闭式态数断言照判——等于闭式交回判了、不等当场 panic。
    /// 判别力（`crates/mutations.tsv`）：没设变量也当缩过域、闭式不判，这条红在不等那一条闭式断言上。
    #[test]
    fn the_closed_form_assertion_is_judged_when_the_domain_is_not_narrowed() {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        assert_eq!(
            assert_the_counted_states_equal_the_closed_form_unless_narrowed(4u64, 4u64, "等于闭式"),
            ClosedFormStateCountJudgement::Judged,
            "没缩：判了"
        );
        assert!(
            std::panic::catch_unwind(|| {
                assert_the_counted_states_equal_the_closed_form_unless_narrowed(
                    3u64,
                    4u64,
                    "比闭式少一个",
                )
            })
            .is_err(),
            "没缩而枚举到的比闭式少：闭式断言当场 panic"
        );
        assert_eq!(
            ClosedFormStateCountJudgement::Judged.line("等于闭式"),
            None,
            "判了的那一趟不打结果行"
        );
    }

    /// 缩过域（上限由调用方给）：闭式不判，交回上限、实际展开的与全量域的闭式；按发布分的那一串同样经它（`Debug` 的写法）。
    /// 判别力（`crates/mutations.tsv`）：缩过域也照判闭式，这条红（集成测试那一条同红）。
    #[test]
    fn the_closed_form_assertion_is_left_unjudged_on_a_narrowed_domain() {
        assert_eq!(
            closed_form_state_count_judged_under(
                Some(ExpandUpToWritesPerSegment(4)),
                3u64,
                4u64,
                "比闭式少一个"
            ),
            ClosedFormStateCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "3".to_string(),
                closed_form: "4".to_string(),
            }
        );
        let by_publish = closed_form_state_count_judged_under(
            Some(ExpandUpToWritesPerSegment(4)),
            "instance1_txg1=20 every_write_persisted=1".to_string(),
            "instance1_txg1=26 every_write_persisted=1",
            "按发布分",
        );
        assert_eq!(
            by_publish.line("按发布分"),
            Some(format!(
                "{CLOSED_FORM_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=\"instance1_txg1=20 every_write_persisted=1\" closed_form=\"instance1_txg1=26 every_write_persisted=1\" what=按发布分"
            ))
        );
    }

    /// 按全量域钉的评估数、观察者计数两种断言：没缩域（守卫清掉缩域变量）照判——等于钉的数、至少一个都交回判了，比钉的少、是 0 当场 panic；
    /// 缩过域（守卫设缩域变量 4）两种都不判，交回上限、这一趟数出来的与全量域上要的，结果行逐字。
    /// 判别力（`crates/mutations.tsv`）：钉数那一种不读缩域变量（缩过域也照判）、至少一个那一种不读缩域变量，各让这条红。
    #[test]
    fn the_full_domain_count_assertions_are_judged_unless_the_environment_narrows_the_domain() {
        {
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
            assert_eq!(
                assert_the_count_equals_its_full_domain_pin_unless_narrowed(9u64, 9u64, "钉 9"),
                FullDomainCountJudgement::Judged,
                "没缩：钉数那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| {
                    assert_the_count_equals_its_full_domain_pin_unless_narrowed(
                        8u64,
                        9u64,
                        "比钉的少一个",
                    )
                })
                .is_err(),
                "没缩而数出来的比钉的少：当场 panic"
            );
            assert_eq!(
                assert_the_count_is_above_zero_unless_narrowed(1, "一个"),
                FullDomainCountJudgement::Judged,
                "没缩：至少一个那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| assert_the_count_is_above_zero_unless_narrowed(
                    0,
                    "一个都没有"
                ))
                .is_err(),
                "没缩而数出来的是 0：当场 panic"
            );
        }
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("4"),
        )]);
        let pinned = assert_the_count_equals_its_full_domain_pin_unless_narrowed(
            [3u64, 12],
            [3u64, 13],
            "两格",
        );
        assert_eq!(
            pinned,
            FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "[3, 12]".to_string(),
                full_domain_expects: "[3, 13]".to_string(),
            },
            "缩过域：钉数那一种不判"
        );
        assert_eq!(
            pinned.line("两格"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=[3, 12] full_domain_expects=[3, 13] what=两格"
            ))
        );
        let above_zero = assert_the_count_is_above_zero_unless_narrowed(0, "缩掉的那一格");
        assert_eq!(
            above_zero.line("缩掉的那一格"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=0 full_domain_expects={FULL_DOMAIN_EXPECTS_ABOVE_ZERO} what=缩掉的那一格"
            )),
            "缩过域：至少一个那一种不判"
        );
        assert_eq!(FullDomainCountJudgement::Judged.line("钉 9"), None);
    }

    /// 「全量域上多于下界」那一种断言：没缩域（守卫清掉缩域变量）照判——多于下界交回判了，等于下界当场 panic；
    /// 缩过域（守卫设缩域变量 4）不判，交回上限、这一趟数出来的与 `above_<下界>`，结果行逐字。
    /// 判别力（`crates/mutations.tsv`「崩溃枚举「全量域上多于下界」的计数断言不读缩域变量（缩过域也照判）」）：不读缩域变量，缩过域那一半当场 panic，这条红。
    #[test]
    fn the_count_exceeding_its_full_domain_floor_is_judged_unless_the_environment_narrows_the_domain(
    ) {
        {
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
            assert_eq!(
                assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(2, 1, "多于一个"),
                FullDomainCountJudgement::Judged,
                "没缩：多于下界那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| {
                    assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(1, 1, "只有一个")
                })
                .is_err(),
                "没缩而数出来的等于下界：当场 panic"
            );
        }
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("4"),
        )]);
        let exceeding =
            assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(1, 1, "缩掉的那几段");
        assert_eq!(
            exceeding,
            FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "1".to_string(),
                full_domain_expects: "above_1".to_string(),
            },
            "缩过域：多于下界那一种不判"
        );
        assert_eq!(
            exceeding.line("缩掉的那几段"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=1 full_domain_expects=above_1 what=缩掉的那几段"
            ))
        );
    }
}
