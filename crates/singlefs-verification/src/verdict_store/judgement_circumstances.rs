//! 判定日期与跑法：块属性与对拍结论里记的东京日期与剪枝、逐条两种跑法。

#[cfg(test)]
use super::*;

/// 块属性里的跑法（words 路第八节第 7 项）：剪枝（键相同的复用、对不上的现判）、逐条（点名单跑一项）。两种都照键读库。
/// GPU 还是 CPU 在 [`JudgingDevice`] 那一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunKindOfTheJudgement {
    Pruned,
    ItemByItem,
}

/// 入口脚本交跑法的环境变量：`pruned`、`item_by_item`；没设是剪枝。
pub const CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_LIBRARY_RUN_KIND";

impl RunKindOfTheJudgement {
    /// 块属性里的码。
    pub(super) fn code(self) -> u8 {
        match self {
            Self::Pruned => 2,
            Self::ItemByItem => 3,
        }
    }

    /// 块属性里的码换回跑法；这一版不认的码交回 None（读回的那一行照不合布局报错，不当没判）。
    pub(super) fn of_code(code: u8) -> Option<Self> {
        match code {
            2 => Some(Self::Pruned),
            3 => Some(Self::ItemByItem),
            _ => None,
        }
    }

    /// 行里报的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pruned => "pruned",
            Self::ItemByItem => "item_by_item",
        }
    }

    /// 跑法变量的值（`None` 是没设）换成跑法。
    ///
    /// # Panics
    /// 跑法变量设了而认不得：写明变量名与值，不悄悄当剪枝。
    #[must_use]
    pub fn of_the_variable_value(run_kind: Option<&str>) -> Self {
        match run_kind {
            Some("pruned") => Self::Pruned,
            Some("item_by_item") => Self::ItemByItem,
            Some(other) => panic!(
                "{CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE}={other:?} 认不得：只认 pruned、item_by_item，或不设"
            ),
            None => Self::Pruned,
        }
    }

    /// 这个进程的跑法。
    ///
    /// # Panics
    /// 同 [`Self::of_the_variable_value`]。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::of_the_variable_value(
            std::env::var(CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
    }
}

/// 一块判定是在什么情形下判的（块属性，不进键、不进结论的比对）：判定日期（东京日期 `YYYY-MM-DD`）与跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgementCircumstances {
    pub judged_date: String,
    pub run_kind: RunKindOfTheJudgement,
}

impl JudgementCircumstances {
    /// 此刻在这个进程里判的：今天的东京日期与这个进程的跑法。
    ///
    /// # Panics
    /// 同 [`RunKindOfTheJudgement::of_this_process`]。
    #[must_use]
    pub fn of_this_process_now() -> Self {
        Self {
            judged_date: tokyo_date_text_of(std::time::SystemTime::now()),
            run_kind: RunKindOfTheJudgement::of_this_process(),
        }
    }
}

/// 东京比协调世界时早 9 小时，不用夏令时。
const TOKYO_SECONDS_AHEAD_OF_UTC: u64 = 9 * 3600;
const SECONDS_PER_DAY: u64 = 86_400;

/// 一个时刻在东京的日期 `YYYY-MM-DD`（公历；1970 年以前的时刻算作 1970-01-01；history-kept: 日期格式的说明与例子）。
#[must_use]
pub fn tokyo_date_text_of(moment: std::time::SystemTime) -> String {
    let seconds_since_the_epoch = moment
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let days_since_the_epoch =
        i64::try_from((seconds_since_the_epoch + TOKYO_SECONDS_AHEAD_OF_UTC) / SECONDS_PER_DAY)
            .expect("天数装得进 i64");
    let (year, month, day) = civil_date_of_days_since_the_epoch(days_since_the_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

/// 从 1970-01-01 起的天数换成公历（年，月，日）（history-kept: 日期格式的说明与例子）：按四百年一轮、三月起算的年把天数拆开（闰日落在年末）。
fn civil_date_of_days_since_the_epoch(days_since_the_epoch: i64) -> (i64, i64, i64) {
    const DAYS_FROM_THE_ERA_START_TO_THE_EPOCH: i64 = 719_468;
    const DAYS_PER_ERA: i64 = 146_097;
    let days_since_the_era_start = days_since_the_epoch + DAYS_FROM_THE_ERA_START_TO_THE_EPOCH;
    let era = days_since_the_era_start.div_euclid(DAYS_PER_ERA);
    let day_of_the_era = days_since_the_era_start.rem_euclid(DAYS_PER_ERA);
    let year_of_the_era = (day_of_the_era - day_of_the_era / 1460 + day_of_the_era / 36_524
        - day_of_the_era / 146_096)
        / 365;
    let day_of_the_year_from_march =
        day_of_the_era - (365 * year_of_the_era + year_of_the_era / 4 - year_of_the_era / 100);
    let month_from_march = (5 * day_of_the_year_from_march + 2) / 153;
    let day = day_of_the_year_from_march - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_the_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判定日期是东京日期：协调世界时 15 点是东京第二天零点；跑法变量认得两种写法，没设是剪枝；别的值（拿一个从没用过的名字测）照认不得拒。
    #[test]
    fn the_judged_date_is_the_tokyo_date_and_the_run_kind_comes_from_the_run_kind_variable() {
        let at = |seconds: u64| std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
        assert_eq!(tokyo_date_text_of(at(0)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600 - 1)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600)), "1970-01-02");
        assert_eq!(tokyo_date_text_of(at(951_782_400)), "2000-02-29", "闰日");
        assert_eq!(tokyo_date_text_of(at(1_791_990_000)), "2026-10-15");
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(None),
            RunKindOfTheJudgement::Pruned
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(Some("pruned")),
            RunKindOfTheJudgement::Pruned
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(Some("item_by_item")),
            RunKindOfTheJudgement::ItemByItem
        );
        let refused_value = "run_kind_value_never_used";
        assert!(
            std::panic::catch_unwind(|| {
                RunKindOfTheJudgement::of_the_variable_value(Some(refused_value))
            })
            .is_err(),
            "跑法变量设成 {refused_value:?} 认不得，不当剪枝"
        );
    }

    /// 库里一行的跑法码不是这一版认得的（写成 1；块属性那一行与对拍结论那一行各一份）：问缺哪几种、读块、存块、读对拍结论都照不合布局报错，
    /// 不当没判、不照键现判后换上。判别力（`crates/mutations.tsv`）：问缺哪几种时把认不出的块属性当没判，这条红。
    #[test]
    fn a_stored_row_whose_run_kind_code_is_not_of_this_version_is_refused_as_not_matching_the_layout(
    ) {
        let library = TestLibrary::new("run-kind-code-not-of-this-version");
        let block_key = block_key_named("node", 0);
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let stored_block = conclusion_record(block_of(&[clean, clean]), &[]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &stored_block,
            )
            .expect("存");
        let attribute_key = VerdictStoreKey::ConclusionBlockAttributes(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .to_key_bytes();
        let mut attribute_with_a_run_kind_code_of_one = conclusion_block_attribute_bytes(
            &JudgingDevice::Cpu,
            &JudgementCircumstances {
                judged_date: "2026-10-01".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        );
        *attribute_with_a_run_kind_code_of_one
            .last_mut()
            .expect("块属性末尾是跑法码那一个字节") = 1;
        let pairing_key = PairingConclusionKey {
            inputs_digest: [4; SHA256_DIGEST_BYTES],
        };
        let mut pairing_bytes_with_a_run_kind_code_of_one = StoredPairingConclusion {
            conclusion: b"agree states=3".to_vec(),
            agreement: PairingAgreement::BothSidesAgree,
            circumstances: JudgementCircumstances {
                judged_date: "2026-10-15".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        }
        .to_bytes();
        *pairing_bytes_with_a_run_kind_code_of_one
            .last_mut()
            .expect("对拍结论的值末尾是跑法码那一个字节") = 1;
        library.write_raw_rows(vec![
            (attribute_key.0, attribute_with_a_run_kind_code_of_one),
            (
                VerdictStoreKey::PairingConclusion(&pairing_key)
                    .to_key_bytes()
                    .0,
                pairing_bytes_with_a_run_kind_code_of_one,
            ),
        ]);
        let mut reopened = library.open();
        let required = [(ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION)];
        assert!(
            matches!(
                reopened.conclusions_missing_for_block(&block_key, &required),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "问缺哪几种：认不出的跑法码照块属性不合布局报错，不当没判"
        );
        assert!(
            matches!(
                reopened.read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                ),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "读块：认不出的跑法码照块属性不合布局报错，不当没判"
        );
        assert!(
            matches!(
                reopened.store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                    &stored_block,
                ),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "存块：库里那一份认不出，报错，不整份换上"
        );
        assert!(
            matches!(
                reopened.read_pairing_conclusion(&pairing_key),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::PairingConclusions,
                    ..
                })
            ),
            "读对拍结论：认不出的跑法码照不合布局报错，不当没判"
        );
    }
}
