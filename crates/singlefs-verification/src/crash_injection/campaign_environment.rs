//! 崩溃注入的环境变量：重放行、断言组与从环境读出这一批怎么跑。

use super::*;

/// 崩溃注入那个二进制的环境变量前缀；取样参数的后缀与随机历史共用（`singlefs_verification::history::HistorySamplingVariable`）。
pub const CRASH_INJECTION_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_CRASH_INJECTION_");

/// 重放行与重放入口里崩溃注入自己的几个变量（接在 [`CRASH_INJECTION_VARIABLE_PREFIX`] 后面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionVariable {
    /// 种子区间的起点。
    FirstSeed,
    /// 种子数。
    Seeds,
    /// 每段摆几个崩溃状态（`CrashPointDraw::Sampled`）；整段枚举短段那一种是长段上各抽几个。
    Points,
    /// 写数不超过几的段整段枚举（`CrashPointDraw::EveryProperSubsetOfShortSegments`）；只抽样的那一种写 `none`。
    ShortSegmentWrites,
    /// 照哪一组断言判（[`CrashInjectionAssertions`]）。
    Assertions,
}

impl CrashInjectionVariable {
    /// 重放行按这个次序写出崩溃注入自己的几个变量（取样参数接在后面）。
    pub const ALL: [CrashInjectionVariable; 5] = [
        CrashInjectionVariable::FirstSeed,
        CrashInjectionVariable::Seeds,
        CrashInjectionVariable::Points,
        CrashInjectionVariable::ShortSegmentWrites,
        CrashInjectionVariable::Assertions,
    ];

    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            CrashInjectionVariable::FirstSeed => "FIRST_SEED",
            CrashInjectionVariable::Seeds => "SEEDS",
            CrashInjectionVariable::Points => "POINTS",
            CrashInjectionVariable::ShortSegmentWrites => "SHORT_SEGMENT_WRITES",
            CrashInjectionVariable::Assertions => "ASSERTIONS",
        }
    }

    #[must_use]
    pub fn variable_name(self) -> String {
        CRASH_INJECTION_VARIABLE_PREFIX.variable_named(self.suffix())
    }
}

crate::enum_listing_every_member! {
/// 照重放行重跑的那一趟按哪一组断言判：默认批、回退取样那几条用例除了清单外的失败，各有自己的结构与覆盖断言
/// （每个崩溃状态与每个二次崩溃都跑过什么、各条路径跑到没有），「复现这一段」那一行只判清单外的失败。重放行带上它，崩溃注入那条入口照它挑断言，
/// 红在结构或覆盖断言上的照抄重放行也红在同一条上（重放行一律交只判新发现的入口的话，
/// 那一类红照抄复现不出）。各组断言写在 `tests/crash_injection_campaign.rs` 的 `judge_crash_injection_report`，这里只定名字与取值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionAssertions {
    /// 只判清单外的失败：新发现之后「复现这一段」那一行照它判（一段的新发现与整批的结构计数无关）；设变量跑别的一批时也可以点名它。
    NewFindingsOnly,
    /// 默认批那一组（`crash_injection_recovers_only_into_versions_the_model_committed` 不设变量时跑的那一批）：
    /// 三截每一截在每个崩溃状态上都跑过、二次崩溃摆出来过且每个都判过，各条路径都跑到了。
    DefaultBatch,
    /// 偏向抬 F 之后回退的取样点那一组（`crash_states_land_inside_rollbacks_and_floor_raises`）。
    RollbackSampling,
}
}

impl CrashInjectionAssertions {
    /// 重放行与崩溃注入入口的环境变量里这一组的取值（[`CrashInjectionVariable::Assertions`]）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            CrashInjectionAssertions::NewFindingsOnly => "new-findings",
            CrashInjectionAssertions::DefaultBatch => "default-batch",
            CrashInjectionAssertions::RollbackSampling => "rollback-sampling",
        }
    }
}

/// 只抽样（`CrashPointDraw::Sampled`）时 [`CrashInjectionVariable::ShortSegmentWrites`] 的取值。
const SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE: &str = "none";

/// 重放行里这一种摆法的两个变量的取值：(每段几个 / 长段上各抽几个, 短段的写数上限或 `none`)。
fn draw_environment_values(draw: CrashPointDraw) -> (String, String) {
    match draw {
        CrashPointDraw::Sampled {
            crash_points_per_history,
        } => (
            crash_points_per_history.to_string(),
            SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE.to_string(),
        ),
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes,
            sampled_in_longer_segments,
        } => (
            sampled_in_longer_segments.to_string(),
            segment_writes.to_string(),
        ),
        CrashPointDraw::ExactlyOneCrashState { .. } => {
            panic!("{TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}")
        }
    }
}

/// 崩溃注入判红时打的重放命令：种子区间、崩溃状态怎么摆、照哪一组断言判、取样参数（每段步数、比重、checker、盘宽、空间准入），
/// 交崩溃注入那条入口（`crash_injection_campaign` 的 `crash_injection_recovers_only_into_versions_the_model_committed`）照抄就是原来那一批、照原来那一组断言判。
/// 线程数不在里面：线程数不改结论（`one_thread_and_four_threads_render_the_same_report`）。
#[must_use]
pub fn crash_injection_replay_line(
    first_seed: u64,
    seed_count: u64,
    sampling: &HistorySampling,
    draw: CrashPointDraw,
    assertions: CrashInjectionAssertions,
) -> String {
    let (points, short_segment_writes) = draw_environment_values(draw);
    let mut assignments: Vec<(String, String)> = CrashInjectionVariable::ALL
        .iter()
        .map(|variable| {
            let value = match variable {
                CrashInjectionVariable::FirstSeed => first_seed.to_string(),
                CrashInjectionVariable::Seeds => seed_count.to_string(),
                CrashInjectionVariable::Points => points.clone(),
                CrashInjectionVariable::ShortSegmentWrites => short_segment_writes.clone(),
                CrashInjectionVariable::Assertions => assertions.environment_value().to_string(),
            };
            (variable.variable_name(), value)
        })
        .collect();
    assignments.extend(sampling.environment_assignments(CRASH_INJECTION_VARIABLE_PREFIX));
    format!(
        "{} cargo test --release -p singlefs-verification --test crash_injection_campaign -- --exact crash_injection_recovers_only_into_versions_the_model_committed",
        render_environment_assignments(&assignments)
    )
}

/// 崩溃注入那条入口读环境：读的变量与 [`crash_injection_replay_line`] 写出的逐个相同（照哪一组断言判也在里面），没设的取 `defaults` 的；
/// 线程数与镜像留不留不在重放行里，照 `defaults` 的。
/// 这是读环境变量的测试入口，哪个变量写错都停下（不交回错误让调用方各自处置）：种子起点、种子数、崩溃点数、短段写数
/// 经数值环境变量共用的那一份解析；断言组与取样参数（每段步数、比重、checker、盘宽、空间准入）经 `history` 的复现入口解析
/// （随机历史、坏盘输入、故障注入的入口也读这几个），认不得时照它报的那一句 panic。消息都带变量名与原值。
///
/// # Panics
/// 某个变量的取值认不得：数值变量不是装得进的非负整数（短段写数也不是 `none`），或取值是一组名字的变量不在认得的那几个里；
/// 「跑几个」的计数写 0（种子数、每段步数、只抽样时每段几个崩溃状态、短段写数上限）也算认不得。
#[must_use]
pub fn crash_injection_campaign_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &CrashInjectionCampaign,
) -> CrashInjectionCampaign {
    let first_seed = u64_from_reproduction_environment(
        lookup,
        &CrashInjectionVariable::FirstSeed.variable_name(),
    )
    .unwrap_or(defaults.first_seed);
    let seed_count = count_from_reproduction_environment_panicking_when_unreadable(
        lookup,
        &CrashInjectionVariable::Seeds.variable_name(),
    )
    .unwrap_or(defaults.seed_count);
    let (default_points, default_short_segment_writes) = draw_environment_values(defaults.draw);
    let points_variable = CrashInjectionVariable::Points.variable_name();
    let points_text = lookup(&points_variable).unwrap_or(default_points);
    let points = usize_from_the_variable_text(&points_variable, &points_text);
    let short_segment_writes_variable = CrashInjectionVariable::ShortSegmentWrites.variable_name();
    let short_segment_writes =
        lookup(&short_segment_writes_variable).unwrap_or(default_short_segment_writes);
    // 只抽样时崩溃点数是每段摆几个，写 0 就一个崩溃状态都不摆；短段整段枚举时它是长段上另抽几个，0 是「只枚举短段」。
    // 短段写数是写数上限，写 0 没有一段够得上，与 none 同义却让重放行多一种写法，按认不出停下。
    let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {
        CrashPointDraw::Sampled {
            crash_points_per_history: std::num::NonZeroUsize::new(points)
                .unwrap_or_else(|| {
                    panic!(
                        "{points_variable}={points_text:?} 认不出：只抽样（短段写数是 {SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE}）时每段摆几个崩溃状态要写正整数，\
                         0 个就是这一批一个崩溃状态都不摆"
                    )
                })
                .get(),
        }
    } else {
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: {
                let segment_writes = positive_count_from_the_variable_text(
                    &short_segment_writes_variable,
                    &short_segment_writes,
                );
                // 整段枚举按 u64 掩码摆子集（`1u64 << 写数`），写数到 64 位移就溢出：上限与抽样那一支的掩码宽同一个数
                assert!(
                    segment_writes <= WRITES_A_SUBSET_MASK_HOLDS,
                    "{short_segment_writes_variable}={short_segment_writes:?} 认不出：短段整段枚举按 u64 掩码摆子集，写数上限最多 {WRITES_A_SUBSET_MASK_HOLDS}"
                );
                segment_writes
            },
            sampled_in_longer_segments: points,
        }
    };
    let assertions = choice_from_reproduction_environment(
        lookup,
        &CrashInjectionVariable::Assertions.variable_name(),
        &CrashInjectionAssertions::ALL,
        CrashInjectionAssertions::environment_value,
        defaults.assertions,
    )
    .unwrap_or_else(|unrecognized| panic!("{}", unrecognized.render()));
    let sampling = HistorySampling::from_environment(
        CRASH_INJECTION_VARIABLE_PREFIX,
        lookup,
        &HistorySampling {
            operations_per_history: defaults.operations_per_history,
            weights: defaults.weights,
            execution: defaults.execution,
        },
    )
    .unwrap_or_else(|unrecognized| panic!("{}", unrecognized.render()));
    CrashInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history: sampling.operations_per_history,
        draw,
        weights: sampling.weights,
        execution: sampling.execution,
        worker_threads: defaults.worker_threads,
        image_retention: defaults.image_retention,
        assertions,
    }
}

/// 读一个 u64 的非负整数变量（种子起点）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份。
///
/// # Panics
/// 设了而不是装得进 u64 的非负整数：消息带变量名与原值。
fn u64_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable::<u64>(
        variable,
        lookup(variable).as_deref(),
    )
}

/// 读一个「跑几个」的计数变量（种子数）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份，取 `NonZeroU64`：
/// 写 0 就是这一批一段都不跑，照样跑完、按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」），与写错一样停下。
///
/// # Panics
/// 设了而不是装得进 u64 的正整数：消息带变量名与原值。
fn count_from_reproduction_environment_panicking_when_unreadable(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(
        variable,
        lookup(variable).as_deref(),
    )
    .map(std::num::NonZeroU64::get)
}

/// 一个装得进 usize 的非负整数变量的原文换成数：解析经数值环境变量共用的那一份，写错就 panic、消息带变量名与原值。
///
/// # Panics
/// `text` 不是装得进 usize 的非负整数。
fn usize_from_the_variable_text(variable: &str, text: &str) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<usize>(variable, Some(text)).expect(
        "交进去的值是 Some（没设时调用方已取了默认原文），共用解析只在值是 None 时交回 None",
    )
}

/// 一个计数变量（短段的写数上限）的原文换成数：同 [`usize_from_the_variable_text`]，取 `NonZeroUsize`，写 0 同样停下。
///
/// # Panics
/// `text` 不是装得进 usize 的正整数。
fn positive_count_from_the_variable_text(variable: &str, text: &str) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
        variable,
        Some(text),
    )
    .expect("交进去的值是 Some（没设时调用方已取了默认原文），共用解析只在值是 None 时交回 None")
    .get()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 重放行写出的两种摆法（只抽样、短段整段枚举）都经重放入口的解析原样读回，连同种子区间与取样参数；
    /// 默认值取的是另一组，读回的不是写出的那一组就红。
    #[test]
    fn every_crash_point_draw_reads_back_from_its_own_replay_line() {
        let defaults = CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 9,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        };
        for draw in [
            CrashPointDraw::Sampled {
                crash_points_per_history: 4,
            },
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 5,
                sampled_in_longer_segments: 7,
            },
        ] {
            let sampling = HistorySampling {
                operations_per_history: 24,
                weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
                execution: HistoryExecution {
                    per_step_checker: crate::history::PerStepChecker::Skipped,
                    device_width: crate::history::HistoryDeviceWidth::UnitAreaOf256Slots,
                    space_admission:
                        singlefs_core::admission::SpaceAdmission::SkippedByTheTestOnlySwitch,
                },
            };
            let line = crash_injection_replay_line(
                11,
                12,
                &sampling,
                draw,
                CrashInjectionAssertions::RollbackSampling,
            );
            let assignments = crate::history::environment_assignments_in_line(&line);
            let read_back = crash_injection_campaign_from_environment(
                &|variable: &str| assignments.get(variable).cloned(),
                &defaults,
            );
            assert_eq!(
                read_back,
                CrashInjectionCampaign {
                    first_seed: 11,
                    seed_count: 12,
                    operations_per_history: 24,
                    draw,
                    weights: sampling.weights,
                    execution: sampling.execution,
                    worker_threads: defaults.worker_threads,
                    image_retention: defaults.image_retention,
                    assertions: CrashInjectionAssertions::RollbackSampling,
                },
                "重放行：{line}"
            );
        }
    }

    /// 重放行写出的每一组断言（只判新发现、默认批、回退取样）都经重放入口的解析原样读回；默认值取的是另一组，读回的不是写出的那一组就红。
    #[test]
    fn every_assertion_set_reads_back_from_its_own_replay_line() {
        let defaults_judged_by = |assertions: CrashInjectionAssertions| CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: FOUR_CRASH_POINTS,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions,
        };
        let sampling = HistorySampling {
            operations_per_history: 3,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        for (position, assertions) in CrashInjectionAssertions::ALL.into_iter().enumerate() {
            let another =
                CrashInjectionAssertions::ALL[(position + 1) % CrashInjectionAssertions::ALL.len()];
            let line = crash_injection_replay_line(1, 2, &sampling, FOUR_CRASH_POINTS, assertions);
            let assignments = crate::history::environment_assignments_in_line(&line);
            assert_eq!(
                crash_injection_campaign_from_environment(
                    &|variable: &str| assignments.get(variable).cloned(),
                    &defaults_judged_by(another),
                ),
                defaults_judged_by(assertions),
                "重放行：{line}"
            );
        }
    }

    /// 重放入口只设了 `variable` 这一个变量（取 `value`）时读回的那一批；其余取一组写死的默认。
    fn campaign_read_back_with_only_this_variable_set(
        variable: &'static str,
        value: &'static str,
    ) -> CrashInjectionCampaign {
        campaign_read_back_with_these_variables_set(&[(variable, value)])
    }

    /// 重放入口只设了 `assignments` 里这几个变量（变量名、取值）时读回的那一批；其余取一组写死的默认
    /// （种子起点 1、2 段、每段 3 步、只抽样每段 4 个）。
    fn campaign_read_back_with_these_variables_set(
        assignments: &[(&str, &str)],
    ) -> CrashInjectionCampaign {
        crash_injection_campaign_from_environment(
            &|asked: &str| {
                assignments
                    .iter()
                    .find(|(variable, _value)| *variable == asked)
                    .map(|(_variable, value)| (*value).to_string())
            },
            &CrashInjectionCampaign {
                first_seed: 1,
                seed_count: 2,
                operations_per_history: 3,
                draw: FOUR_CRASH_POINTS,
                weights: GenerationWeights::BROAD,
                execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
                image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                assertions: CrashInjectionAssertions::NewFindingsOnly,
            },
        )
    }

    /// 断言组写成认不得的名字：重放入口停下，消息带变量名、原值与认得的那几个，不悄悄退回默认的那一组
    /// （代码三方 test-script-align-code-r1 判决 S2；重放入口写错一律停下）。
    #[test]
    #[should_panic(
        expected = "SINGLEFS_CRASH_INJECTION_ASSERTIONS=structure：只认 new-findings、default-batch、rollback-sampling"
    )]
    fn an_unknown_crash_injection_assertion_set_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_ASSERTIONS",
            "structure",
        );
    }

    /// 取样参数里的盘宽写成认不得的名字：重放入口停下，消息带变量名与原值，不悄悄退回默认的盘宽。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_DEVICES=small：只认")]
    fn an_unknown_crash_injection_device_width_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_DEVICES",
            "small",
        );
    }

    /// 种子起点写错：经数值环境变量共用的那一份解析停下，消息带变量名与原值，不悄悄退回默认的种子起点。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_FIRST_SEED=\"eleven\" 认不出")]
    fn an_unreadable_crash_injection_first_seed_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_FIRST_SEED",
            "eleven",
        );
    }

    /// 短段写数既不是 `none` 也不是数：经数值环境变量共用的那一份解析停下，消息带变量名与原值，不悄悄当成只抽样。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"five\" 认不出")]
    fn an_unreadable_crash_injection_short_segment_writes_value_panics_naming_the_variable_and_the_value(
    ) {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "five",
        );
    }

    /// 短段写数超过 u64 掩码的宽（63）：整段枚举 `1u64 << 写数` 会溢出，解析时就停下，消息带变量名、原值与上限。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"64\" 认不出")]
    fn crash_injection_short_segment_writes_value_past_the_mask_width_panics_naming_the_limit() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "64",
        );
    }

    /// 重放行里崩溃点数写错：经数值环境变量共用的那一份解析 panic，消息带变量名与原值。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_POINTS=\"four\" 认不出")]
    fn an_unreadable_crash_injection_points_value_panics_naming_the_variable_and_the_value() {
        let defaults = CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: FOUR_CRASH_POINTS,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        };
        let _campaign = crash_injection_campaign_from_environment(
            &|variable: &str| {
                (variable == "SINGLEFS_CRASH_INJECTION_POINTS").then(|| "four".to_string())
            },
            &defaults,
        );
    }

    /// 同一个重放入口里种子起点（经数值环境变量共用的那一份解析）与每段步数（经 `history` 的复现入口解析）前后都带空白：
    /// 两个结局相同，都去掉空白读回（不是一个照收、一个停下）。
    #[test]
    fn blanks_around_the_first_seed_and_the_step_count_of_the_replay_entry_are_dropped_alike() {
        let read_back = campaign_read_back_with_these_variables_set(&[
            ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", " 11"),
            ("SINGLEFS_CRASH_INJECTION_OPERATIONS", " 24"),
        ]);
        assert_eq!(
            (read_back.first_seed, read_back.operations_per_history),
            (11, 24),
            "种子起点与每段步数前后的空白都去掉读回"
        );
    }

    /// 种子数写 0：重放入口停下，消息带变量名与原值，不照收（一段都不跑的一批照只判清单外失败的那一组判成绿）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SEEDS=\"0\" 认不出")]
    fn crash_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign =
            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_SEEDS", "0");
    }

    /// 每段步数写 0：重放入口照 `history` 的复现入口报的那一句停下，消息带变量名与原值。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_OPERATIONS=0：只认 正整数")]
    fn crash_injection_step_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_OPERATIONS",
            "0",
        );
    }

    /// 只抽样时每段崩溃点数写 0：重放入口停下，消息带变量名与原值（每段一个崩溃状态都不摆）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_POINTS=\"0\" 认不出")]
    fn zero_crash_points_with_sampling_only_panics_naming_the_variable_and_the_value() {
        let _campaign =
            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_POINTS", "0");
    }

    /// 短段整段枚举时长段上另抽 0 个：照收，是「只枚举短段」那一种摆法（0 只在只抽样时停下）。
    #[test]
    fn zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only() {
        assert_eq!(
            campaign_read_back_with_these_variables_set(&[
                ("SINGLEFS_CRASH_INJECTION_POINTS", "0"),
                ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", "3"),
            ])
            .draw,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 3,
                sampled_in_longer_segments: 0,
            },
            "长段上另抽 0 个、写数不超过 3 的段整段枚举"
        );
    }

    /// 短段写数上限写 0：重放入口停下，消息带变量名与原值（没有一段够得上，与 `none` 同义却多一种写法）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"0\" 认不出")]
    fn short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "0",
        );
    }

    /// 线程数从环境变量取，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值，不看本机核数）；
    /// 设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn crash_injection_worker_thread_count_comes_from_the_environment_variable_or_the_heavy_thread_default(
    ) {
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Some("3"),
                Some("5"),
            ),
            CrashInjectionWorkerThreads::FromTheEnvironmentVariable(3),
            "自己的变量先读"
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
                Some("5")
            ),
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(5),
            "没设自己的变量读 SINGLEFS_THREAD_CAP"
        );
        let registered = crate::heavy_threads::registered_heavy_threads().get();
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
                None
            ),
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(registered),
            "都没设取 .claude/heavy-thread-default 的登记值"
        );
        assert_eq!(
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(4).count(),
            4
        );
    }

    /// 线程数设成 0：经数值环境变量共用的那一份解析停下（线程数取 `NonZeroUsize`，0 认不出），消息带变量名与原值，
    /// 不悄悄退回单线程或默认；口径与仓里别的线程数变量相同。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_THREADS=\"0\" 认不出")]
    fn crash_injection_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _threads = CrashInjectionWorkerThreads::from_the_environment_value(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
            Some("0"),
            None,
        );
    }
}
