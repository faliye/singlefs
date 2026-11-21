//! 一段历史怎么跑（每步的 checker、盘宽、空间准入）与复现：环境变量、取样点、复现行。

use super::*;

enum_listing_every_member! {
/// 每一步之后 harness 自己跑不跑池级 checker：一律不跑，每一步的镜像经钩子交出去（[`execute_history_handing_out_every_step_image`]），
/// 池级 checker 那一遍归给定状态引擎两段（镜像作为给定状态写进判定库、判段从库读、判在 GPU；
/// `crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`），
/// harness 随时跑的只有与理想模型、RefFS 比对那一截；就地判、判红就停的那一种不再有：
/// 要判红就停那一档口径的调用方（E158 的故障注入）经故障注入的合并入口收给定状态引擎判回的结论
/// （`crate::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerStepChecker {
    /// 不跑：只由理想模型、执行器自己的判定与 panic 让历史停下，每一步的镜像经钩子交出去（harness 用例、崩溃注入、坏盘输入、故障注入都用它）。
    Skipped,
}
}

impl PerStepChecker {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PerStepChecker::Skipped => "不跑池级 checker（只由模型、执行器的判定与 panic 判）",
        }
    }

    /// 复现行与复现入口的环境变量里这一种的取值（`HistorySamplingVariable::Checker`）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            PerStepChecker::Skipped => "skip",
        }
    }
}

/// 一段历史怎么跑：每一步之后的 checker、两块盘多宽（取样点在种子、步数、比重之外的两个参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryExecution {
    pub per_step_checker: PerStepChecker,
    pub device_width: HistoryDeviceWidth,
    /// 判不判空间准入（只供测试的开关，`singlefs_core::admission::SpaceAdmission`）。取样点里要测「准入放行而落点取不到」那条兜底拒绝的
    /// （小盘上式子先拒，走不到落点那一道）装 `SkippedByTheTestOnlySwitch`；别的一律判。
    pub space_admission: SpaceAdmission,
}

impl HistoryExecution {
    /// harness 用例的执行器：harness 不跑每一步的池级 checker，每一步的镜像经钩子交出去（池级 checker 那一遍归给定状态引擎两段），
    /// 两块 4 GiB 的盘、判空间准入。
    pub const HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution =
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };

    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> String {
        match self.space_admission {
            SpaceAdmission::JudgedByTheFormula => format!(
                "{}；{}",
                self.per_step_checker.name(),
                self.device_width.name()
            ),
            SpaceAdmission::SkippedByTheTestOnlySwitch => format!(
                "{}；{}；空间准入关掉（只供测试的开关，{}）",
                self.per_step_checker.name(),
                self.device_width.name(),
                self.space_admission.branch_name()
            ),
        }
    }
}

/// 复现行与复现入口的环境变量里空间准入那一臂的取值（`HistorySamplingVariable::SpaceAdmission`）。复现入口认的空间准入全集是核心层生成的
/// `SpaceAdmission::ALL`（不在这里另抄一张）：核心层加一个成员，这里的穷举 `match` 不补取值就编不过，补了复现入口就认得它。
#[must_use]
pub const fn space_admission_environment_value(space_admission: SpaceAdmission) -> &'static str {
    match space_admission {
        SpaceAdmission::JudgedByTheFormula => "judged",
        SpaceAdmission::SkippedByTheTestOnlySwitch => "skipped",
    }
}

/// 复现行与复现入口里环境变量名的前缀：每个装置一个（随机历史 [`RANDOM_HISTORY_VARIABLE_PREFIX`]，崩溃注入与坏盘输入各起一个，
/// 故障注入在它的用例文件 `tests/fault_injection_campaign.rs` 里起一个），
/// 后缀由 [`HistorySamplingVariable::suffix`] 与各装置自己的几个变量定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReproductionVariablePrefix(pub &'static str);

impl ReproductionVariablePrefix {
    /// 前缀接上后缀的整个变量名。
    #[must_use]
    pub fn variable_named(self, suffix: &str) -> String {
        format!("{}{suffix}", self.0)
    }
}

/// 随机历史那个二进制的环境变量前缀。
pub const RANDOM_HISTORY_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_RANDOM_HISTORY_");

/// 「收缩一个种子」那条入口读的种子变量（接在 [`RANDOM_HISTORY_VARIABLE_PREFIX`] 后面）。
pub const RANDOM_HISTORY_SHRINK_SEED_SUFFIX: &str = "SHRINK_SEED";

/// 随机历史的工作线程数变量：`random_histories` 的默认批与设了变量的那一批、各取样点与「收缩一个种子」那条入口
/// （example `shrink_one_failing_random_history_seed`）同读这一个，没设取每台默认重活线程数（`crate::heavy_threads`）。
pub const RANDOM_HISTORY_WORKER_THREADS_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_THREADS";

/// 随机历史默认批的取样参数（每段 30 步、broad 比重、每一步的镜像交出去、两块 4 GiB 的盘、判空间准入）：
/// `random_histories` 不设变量时照它跑，「收缩一个种子」那条入口读环境时没设的变量取它的（[`random_history_to_reproduce_from_environment`]）。
pub const RANDOM_HISTORY_DEFAULT_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: 30,
    weights: GenerationWeights::BROAD,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// [`HistorySampling`] 的每一项在环境变量里的那一个（后缀）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistorySamplingVariable {
    Operations,
    Weights,
    Checker,
    Devices,
    SpaceAdmission,
}

impl HistorySamplingVariable {
    /// 复现行按这个次序写出 [`HistorySampling`] 的每一项。
    pub const ALL: [HistorySamplingVariable; 5] = [
        HistorySamplingVariable::Operations,
        HistorySamplingVariable::Weights,
        HistorySamplingVariable::Checker,
        HistorySamplingVariable::Devices,
        HistorySamplingVariable::SpaceAdmission,
    ];

    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            HistorySamplingVariable::Operations => "OPERATIONS",
            HistorySamplingVariable::Weights => "WEIGHTS",
            HistorySamplingVariable::Checker => "CHECKER",
            HistorySamplingVariable::Devices => "DEVICES",
            HistorySamplingVariable::SpaceAdmission => "SPACE_ADMISSION",
        }
    }
}

/// 复现入口读环境变量时认不得的输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReproductionEnvironmentError {
    /// 这个变量必给（种子这一类），没设。
    Missing { variable: String },
    /// 设了，取值不在认得的那几个里（数值变量认得的是「非负整数」）。
    Unrecognized {
        variable: String,
        value: String,
        recognized: Vec<String>,
    },
}

impl ReproductionEnvironmentError {
    /// 给人看：哪个变量、给了什么、认得什么。
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            ReproductionEnvironmentError::Missing { variable } => format!("要给 {variable}"),
            ReproductionEnvironmentError::Unrecognized {
                variable,
                value,
                recognized,
            } => format!("{variable}={value}：只认 {}", recognized.join("、")),
        }
    }
}

/// 读一个非负整数变量：没设取 `default`。前后空白先去掉，与数值环境变量共用的那一份解析
/// （`numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable`）同一口径：
/// 同一个复现入口里有的数经它、有的数经共用解析，`" 11"` 不能这边照收、那边停下。
///
/// # Errors
/// 设了但不是非负整数。
pub fn number_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    default: u64,
) -> Result<u64, ReproductionEnvironmentError> {
    Ok(
        number_given_in_reproduction_environment::<u64>(lookup, variable, "非负整数")?
            .unwrap_or(default),
    )
}

/// 读一个「跑几个」的计数变量（种子数、每段步数、每段注入次数这一类）：没设取 `default`；写 0 与写错一样认不得。
/// 0 个就是这一批一项都没跑，照样跑完、照样退 0，按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
/// 前后空白照 [`number_from_reproduction_environment`] 先去掉。
///
/// # Errors
/// 设了但不是正整数。
pub fn count_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    default: u64,
) -> Result<u64, ReproductionEnvironmentError> {
    Ok(
        number_given_in_reproduction_environment::<std::num::NonZeroU64>(
            lookup,
            variable,
            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,
        )?
        .map_or(default, std::num::NonZeroU64::get),
    )
}

/// [`count_from_reproduction_environment`] 认不得时报的「认得的取值」。
pub const COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT: &str = "正整数（0 个就是一项都没跑）";

/// 复现入口里一个数值变量的取值换成 `Number`：没设是 `None`；前后空白先去掉；认不出交回带原值的
/// [`ReproductionEnvironmentError::Unrecognized`]，认得的取值写成 `recognized`。
fn number_given_in_reproduction_environment<Number: std::str::FromStr>(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    recognized: &str,
) -> Result<Option<Number>, ReproductionEnvironmentError> {
    let Some(text) = lookup(variable) else {
        return Ok(None);
    };
    match text.trim().parse::<Number>() {
        Ok(number) => Ok(Some(number)),
        Err(_not_a_number) => Err(ReproductionEnvironmentError::Unrecognized {
            variable: variable.to_string(),
            value: text,
            recognized: vec![recognized.to_string()],
        }),
    }
}

/// 读一个必给的非负整数变量（种子这一类）。
///
/// # Errors
/// 没设，或不是非负整数。
pub fn required_number_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Result<u64, ReproductionEnvironmentError> {
    match lookup(variable) {
        None => Err(ReproductionEnvironmentError::Missing {
            variable: variable.to_string(),
        }),
        Some(_given) => number_from_reproduction_environment(lookup, variable, 0),
    }
}

/// 读一个取值在 `choices` 里的变量：按 `environment_value_of` 认，没设取 `default`。
///
/// # Errors
/// 设了，取值不是 `choices` 里任一个的 `environment_value_of`。
pub fn choice_from_reproduction_environment<Choice: Copy>(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    choices: &[Choice],
    environment_value_of: fn(Choice) -> &'static str,
    default: Choice,
) -> Result<Choice, ReproductionEnvironmentError> {
    let Some(value) = lookup(variable) else {
        return Ok(default);
    };
    choices
        .iter()
        .copied()
        .find(|choice| environment_value_of(*choice) == value)
        .ok_or_else(|| ReproductionEnvironmentError::Unrecognized {
            variable: variable.to_string(),
            value,
            recognized: choices
                .iter()
                .map(|choice| environment_value_of(*choice).to_string())
                .collect(),
        })
}

/// 种子之外，复现一段随机历史要的全部参数：每段步数、比重、怎么跑（每一步之后的 checker、盘宽、空间准入）。
/// 判红时打的复现行（[`HistorySampling::environment_assignments`]）与复现入口的解析（[`HistorySampling::from_environment`]）
/// 都经它、经同一张变量表（[`HistorySamplingVariable`]），格式只有这一份定义。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistorySampling {
    pub operations_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
}

impl HistorySampling {
    /// 这一项在环境变量里的取值。
    #[must_use]
    pub fn environment_value_of(&self, variable: HistorySamplingVariable) -> String {
        match variable {
            HistorySamplingVariable::Operations => self.operations_per_history.to_string(),
            HistorySamplingVariable::Weights => self.weights.environment_value().to_string(),
            HistorySamplingVariable::Checker => self
                .execution
                .per_step_checker
                .environment_value()
                .to_string(),
            HistorySamplingVariable::Devices => {
                self.execution.device_width.environment_value().to_string()
            }
            HistorySamplingVariable::SpaceAdmission => {
                space_admission_environment_value(self.execution.space_admission).to_string()
            }
        }
    }

    /// 复现行里这几项的「变量名、取值」，按 [`HistorySamplingVariable::ALL`] 的次序。
    #[must_use]
    pub fn environment_assignments(
        &self,
        prefix: ReproductionVariablePrefix,
    ) -> Vec<(String, String)> {
        HistorySamplingVariable::ALL
            .iter()
            .map(|variable| {
                (
                    prefix.variable_named(variable.suffix()),
                    self.environment_value_of(*variable),
                )
            })
            .collect()
    }

    /// 从环境读回（`lookup` 交回变量的取值，没设是 None）：没设的那一项取 `defaults` 的。
    /// 每段步数是计数（[`count_from_reproduction_environment`]）：每段 0 步的一批只跑了起点，写 0 与写错一样认不得。
    ///
    /// # Errors
    /// 某个变量的取值认不得（每段步数写 0 也算）。
    pub fn from_environment(
        prefix: ReproductionVariablePrefix,
        lookup: &dyn Fn(&str) -> Option<String>,
        defaults: &HistorySampling,
    ) -> Result<HistorySampling, ReproductionEnvironmentError> {
        let operations_variable =
            prefix.variable_named(HistorySamplingVariable::Operations.suffix());
        let operations_per_history_given = count_from_reproduction_environment(
            lookup,
            &operations_variable,
            u64::try_from(defaults.operations_per_history).expect("步数装得进 u64"),
        )?;
        let operations_per_history =
            usize::try_from(operations_per_history_given).map_err(|_too_many| {
                ReproductionEnvironmentError::Unrecognized {
                    variable: operations_variable.clone(),
                    value: operations_per_history_given.to_string(),
                    recognized: vec!["装得进 usize 的步数".to_string()],
                }
            })?;
        let weights = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Weights.suffix()),
            &GenerationWeights::NAMED,
            |weights| weights.environment_value(),
            defaults.weights,
        )?;
        let per_step_checker = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Checker.suffix()),
            &PerStepChecker::ALL,
            PerStepChecker::environment_value,
            defaults.execution.per_step_checker,
        )?;
        let device_width = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Devices.suffix()),
            &HistoryDeviceWidth::ALL,
            HistoryDeviceWidth::environment_value,
            defaults.execution.device_width,
        )?;
        let space_admission = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::SpaceAdmission.suffix()),
            &SpaceAdmission::ALL,
            space_admission_environment_value,
            defaults.execution.space_admission,
        )?;
        Ok(HistorySampling {
            operations_per_history,
            weights,
            execution: HistoryExecution {
                per_step_checker,
                device_width,
                space_admission,
            },
        })
    }

    /// 按种子生成这一种取样下的那段历史。
    #[must_use]
    pub fn generate(&self, seed: HistorySeed) -> GeneratedHistory {
        generate_history_with_weights(seed, self.operations_per_history, &self.weights)
    }
}

/// 「变量名、取值」拼成一行 `名=值 名=值 …`：复现行的赋值那一截只由它拼（[`environment_assignments_in_line`] 是它的反向）。
#[must_use]
pub fn render_environment_assignments(assignments: &[(String, String)]) -> String {
    assignments
        .iter()
        .map(|(variable, value)| format!("{variable}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 从一行复现命令里取出「变量名=取值」（空白隔开，变量名 `SINGLEFS_` 起头；报告里紧贴在「复现：」这类前缀后面的那一个也认）：
/// 自证拿它把判红时打的那一行原样喂回复现入口。
#[must_use]
pub fn environment_assignments_in_line(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|token| token.find("SINGLEFS_").map(|start| &token[start..]))
        .filter_map(|assignment| assignment.split_once('='))
        .map(|(variable, value)| (variable.to_string(), value.to_string()))
        .collect()
}

/// 随机历史判红时打的那一行复现命令：种子与这一段的全部取样参数，交「收缩一个种子」那条入口
/// （harness 的 example `shrink_one_failing_random_history_seed`，`cargo run --example` 起；它是收缩工具，不在测试目标里）
/// 照抄就是原来那一段历史、原来那种跑法。
#[must_use]
pub fn random_history_reproduction_line(seed: HistorySeed, sampling: &HistorySampling) -> String {
    let mut assignments = vec![(
        RANDOM_HISTORY_VARIABLE_PREFIX.variable_named(RANDOM_HISTORY_SHRINK_SEED_SUFFIX),
        seed.0.to_string(),
    )];
    assignments.extend(sampling.environment_assignments(RANDOM_HISTORY_VARIABLE_PREFIX));
    format!(
        "{} cargo run --release -p singlefs-verification --example shrink_one_failing_random_history_seed",
        render_environment_assignments(&assignments)
    )
}

/// 「收缩一个种子」那条入口读环境：种子必给，取样参数没设的取 `defaults` 的。
/// 读的变量与 [`random_history_reproduction_line`] 写出的逐个相同。
///
/// # Errors
/// 种子没给，或某个变量的取值认不得。
pub fn random_history_to_reproduce_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &HistorySampling,
) -> Result<(HistorySeed, HistorySampling), ReproductionEnvironmentError> {
    let seed = required_number_from_reproduction_environment(
        lookup,
        &RANDOM_HISTORY_VARIABLE_PREFIX.variable_named(RANDOM_HISTORY_SHRINK_SEED_SUFFIX),
    )?;
    let sampling =
        HistorySampling::from_environment(RANDOM_HISTORY_VARIABLE_PREFIX, lookup, defaults)?;
    Ok((HistorySeed(seed), sampling))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 复现行的赋值那一截写出的每一种取样参数（每一组比重、每一种 checker（只剩不跑那一种）、四档盘宽、空间准入两臂，各配一个步数）都能经复现入口的解析原样读回：
    /// 一项的取值写了读不回（名字对不上、两项同名），或解析漏读一项，这里就红。
    #[test]
    fn every_sampling_choice_reads_back_from_its_own_reproduction_assignments() {
        let defaults = HistorySampling {
            operations_per_history: 1,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let mut combinations = 0usize;
        for weights in GenerationWeights::NAMED {
            for per_step_checker in PerStepChecker::ALL {
                for device_width in HistoryDeviceWidth::ALL {
                    for space_admission in SpaceAdmission::ALL {
                        let sampling = HistorySampling {
                            operations_per_history: 150 + combinations,
                            weights,
                            execution: HistoryExecution {
                                per_step_checker,
                                device_width,
                                space_admission,
                            },
                        };
                        let line = render_environment_assignments(
                            &sampling.environment_assignments(RANDOM_HISTORY_VARIABLE_PREFIX),
                        );
                        let assignments = environment_assignments_in_line(&line);
                        let read_back = HistorySampling::from_environment(
                            RANDOM_HISTORY_VARIABLE_PREFIX,
                            &|variable: &str| assignments.get(variable).cloned(),
                            &defaults,
                        );
                        assert_eq!(read_back, Ok(sampling), "复现行：{line}");
                        combinations += 1;
                    }
                }
            }
        }
        assert_eq!(
            combinations,
            GenerationWeights::NAMED.len()
                * PerStepChecker::ALL.len()
                * HistoryDeviceWidth::ALL.len()
                * SpaceAdmission::ALL.len(),
            "每一种组合都走过一遍"
        );
    }

    /// 照 [`run_history_campaign`] 汇总的一条新发现（造一个假发现，不必真红）打出的复现行，原样喂回复现入口
    /// （[`random_history_to_reproduce_from_environment`]，「收缩一个种子」那条入口读的就是它），读回的种子与取样参数就是这一批的，
    /// 重新生成的历史与那一段逐步相同。取样参数的每一项都取与入口默认（默认批那一种）不同的值（checker 只剩一种取值，那一项取不出不同的）：哪一项在汇总时装错了，读回就不同
    /// （C607：这一处装配没有用例经过时，装错了没人发现）。
    #[test]
    fn the_reproduction_line_of_a_campaign_finding_regenerates_the_history_the_campaign_ran() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let sampling = HistorySampling {
            operations_per_history: 17,
            weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
            execution: HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
                space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
            },
        };
        let first_seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let finding_history = sampling.generate(HistorySeed(first_seed + 1));
        let observation = FailureObservation {
            position: StepPosition::Operation(3),
            operation_kind: Some(finding_history.operations[3].kind()),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let run_ending = |ending: HistoryEnding| HistoryRun {
            ending,
            outcomes: Vec::new(),
            tally: HistoryTally::default(),
            operations_written_by_make_filesystem: 0,
            committed_versions_at_the_end: Vec::new(),
            earliest_stream_index_after_each_power_loss: Vec::new(),
            sizes_requested_by_the_truncations: BTreeMap::new(),
            inodes_named_by_the_truncations: BTreeMap::new(),
            partial_writes_requested: BTreeMap::new(),
            copies_quarantined_by_the_implementation: Vec::new(),
        };
        let finished = vec![
            (
                finding_history.clone(),
                run_ending(HistoryEnding::NewFinding {
                    signature: FailureSignature::HarnessJudgement {
                        judgement: "自证用的假发现",
                    },
                    observation,
                }),
            ),
            (
                sampling.generate(HistorySeed(first_seed)),
                run_ending(HistoryEnding::Completed),
            ),
        ];
        let report = campaign_report_of_finished_histories(
            first_seed,
            2,
            &sampling,
            finished,
            1,
            FindingShrinking::ReportSeedsOnly,
        );
        assert_eq!(report.sampling, sampling, "报告带的取样参数不是这一批的");
        assert_eq!(report.new_findings.len(), 1, "假发现归成一类");
        let rendered = report.new_findings[0].render();
        let reproduction_lines: Vec<&str> = rendered
            .lines()
            .filter(|line| line.contains("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED="))
            .collect();
        assert_eq!(
            reproduction_lines.len(),
            1,
            "新发现里恰好一行复现命令：\n{rendered}"
        );
        let assignments = environment_assignments_in_line(reproduction_lines[0]);
        let (seed, reproduced_sampling) = random_history_to_reproduce_from_environment(
            &|variable: &str| assignments.get(variable).cloned(),
            &defaults,
        )
        .unwrap_or_else(|error| panic!("{}", error.render()));
        assert_eq!(
            (seed, reproduced_sampling),
            (finding_history.seed, sampling),
            "照复现行读回的种子与取样参数（步数、比重、checker、盘宽、空间准入）不是判红那一段的：{}",
            reproduction_lines[0]
        );
        assert_eq!(
            reproduced_sampling.generate(seed),
            finding_history,
            "照复现行重新生成的历史与判红那一段不同"
        );
    }

    /// 复现行起的是这个包里真有的那个收缩 example（`cargo run … --example <名>`，`examples/<名>.rs` 在）：收缩一个种子是工具、不在测试目标里，
    /// 复现行改回起测试目标、或点名一个不在的 example，照抄那一行都起不到收缩入口。
    #[test]
    fn the_reproduction_line_starts_the_shrink_example_of_this_package() {
        let line = random_history_reproduction_line(
            HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 5),
            &RANDOM_HISTORY_DEFAULT_SAMPLING,
        );
        let (command, example_name) = line
            .split_once(" --example ")
            .unwrap_or_else(|| panic!("复现行点名一个 example：{line}"));
        assert!(
            command.ends_with(" cargo run --release -p singlefs-verification"),
            "复现行在 harness 包里 cargo run 那个 example：{line}"
        );
        // 源文件路径写成一串字面量，变异判法的单元键才认得出读的是哪份文件；复现行点名的名字先与它比。
        assert_eq!(
            example_name, "shrink_one_failing_random_history_seed",
            "复现行点名的 example 就是这个包里那个收缩 example：{line}"
        );
        let example_source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/shrink_one_failing_random_history_seed.rs");
        assert!(
            example_source.is_file(),
            "复现行点名的 example 在这个包里有源文件 {}：{line}",
            example_source.display()
        );
    }

    /// 复现入口认不得的取值报出变量名、给的值与认得的那几个，不悄悄退回默认。
    #[test]
    fn an_unrecognized_reproduction_value_is_refused_with_the_recognized_values() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let refused = HistorySampling::from_environment(
            RANDOM_HISTORY_VARIABLE_PREFIX,
            &|variable: &str| {
                (variable == "SINGLEFS_RANDOM_HISTORY_DEVICES").then(|| "small".to_string())
            },
            &defaults,
        );
        assert_eq!(
            refused,
            Err(ReproductionEnvironmentError::Unrecognized {
                variable: "SINGLEFS_RANDOM_HISTORY_DEVICES".to_string(),
                value: "small".to_string(),
                recognized: vec![
                    "4gib".to_string(),
                    "unit-area-384".to_string(),
                    "unit-area-256".to_string(),
                    "unit-area-240".to_string(),
                ],
            })
        );
    }

    /// 「收缩一个种子」那条入口里种子与每段步数前后都带空白：两个都去掉空白读回，与数值环境变量共用的那一份解析同一口径
    /// （同一个入口里一个数照收、另一个数停下，复现行手抄时多一个空格就分不清是哪个变量写错了）。
    #[test]
    fn blanks_around_the_seed_and_the_step_count_of_one_reproduction_entry_are_dropped_alike() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let assignments = BTreeMap::from([
            (
                "SINGLEFS_RANDOM_HISTORY_SHRINK_SEED".to_string(),
                " 11".to_string(),
            ),
            (
                "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
                " 24".to_string(),
            ),
        ]);
        assert_eq!(
            random_history_to_reproduce_from_environment(
                &|variable: &str| assignments.get(variable).cloned(),
                &defaults,
            ),
            Ok((
                HistorySeed(11),
                HistorySampling {
                    operations_per_history: 24,
                    weights: GenerationWeights::BROAD,
                    execution:
                        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                }
            )),
            "种子与每段步数前后的空白都去掉读回"
        );
    }

    /// 每段步数写 0：复现入口报出变量名、给的值与认得的取值，不照收（每段 0 步的一批只跑了起点，照样判成绿）。
    #[test]
    fn reproduction_step_count_of_zero_is_refused_with_the_recognized_values() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        assert_eq!(
            HistorySampling::from_environment(
                RANDOM_HISTORY_VARIABLE_PREFIX,
                &|variable: &str| {
                    (variable == "SINGLEFS_RANDOM_HISTORY_OPERATIONS").then(|| "0".to_string())
                },
                &defaults,
            ),
            Err(ReproductionEnvironmentError::Unrecognized {
                variable: "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
                value: "0".to_string(),
                recognized: vec![COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT.to_string()],
            })
        );
    }
}
