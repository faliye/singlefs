//! 失败的签名：panic 捕获、池级 checker 细节里的分配统计、已知红的形态与归类。

use super::*;

/// panic 钩子记下的那一次：位置（文件:行）与消息。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapturedPanic {
    pub location: String,
    pub message: String,
}

thread_local! {
    static IS_CAPTURING_PANICS: Cell<bool> = const { Cell::new(false) };
    static CAPTURED_PANIC: RefCell<Option<CapturedPanic>> = const { RefCell::new(None) };
}

static INSTALL_PANIC_CAPTURE_HOOK: Once = Once::new();

fn payload_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "（panic 的载荷不是字符串）".to_string())
}

/// 装一次全进程的 panic 钩子：本线程在捕获（`with_panic_capture` 里）就只记位置与消息、不打印；别的线程与捕获之外照旧交给原来的钩子。
fn install_panic_capture_hook() {
    INSTALL_PANIC_CAPTURE_HOOK.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_details| {
            if IS_CAPTURING_PANICS.with(Cell::get) {
                let location = panic_details.location().map_or_else(
                    || "（没有位置）".to_string(),
                    |location| format!("{}:{}", location.file(), location.line()),
                );
                let message = payload_text(panic_details.payload());
                CAPTURED_PANIC.with(|slot| {
                    *slot.borrow_mut() = Some(CapturedPanic { location, message });
                });
            } else {
                previous_hook(panic_details);
            }
        }));
    });
}

/// 跑 `body`，panic 就接住、交回钩子记下的位置与消息。
/// 坏盘输入（增补 3 第 5 件）拿它兜着恢复、可写挂载与池级 checker：panic 是被测代码的缺口，要接住记下来，不是让用例崩。
pub fn with_panic_capture<Output>(body: impl FnOnce() -> Output) -> Result<Output, CapturedPanic> {
    install_panic_capture_hook();
    let was_capturing = IS_CAPTURING_PANICS.with(|flag| flag.replace(true));
    let result = catch_unwind(AssertUnwindSafe(body));
    IS_CAPTURING_PANICS.with(|flag| flag.set(was_capturing));
    result.map_err(|payload| {
        CAPTURED_PANIC
            .with(|slot| slot.borrow_mut().take())
            .unwrap_or_else(|| CapturedPanic {
                location: "（钩子没记到位置）".to_string(),
                message: payload_text(&*payload),
            })
    })
}

/// 一次失败（违例或 panic）连同它发生时的盘面事实，拿去对「已知红」清单。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureObservation {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    /// checker 判红的不变量与它报的第一处。
    pub violations: Vec<(&'static str, String)>,
    pub panic: Option<CapturedPanic>,
    /// 按 checker 的读法从盘上读：根环里最新那条根的 txg；读不到时 None。
    pub newest_ring_root_txg: Option<u64>,
    /// 根环一圈的槽数 R × S（按 checker 从系统配置读出的几何）；读不到时 None。
    pub root_ring_slot_count: Option<u64>,
    /// 执行器自己判出的失败（分配代、冷启动读回）。
    pub harness_judgement: Option<HarnessJudgement>,
    /// 理想模型与实现对不上的那一格（第 2 件）。
    pub model_disagreement: Option<ModelDisagreement>,
    /// 抬 F 那一步（入口返回 Ok）之后判红时：抬之前的镜像上，新 F 那个 txg 上的根是不是全属于被抛弃的实例
    /// （`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。
    /// 崩溃状态上算的是同一个谓词、读的是这个崩溃镜像自己：F 取镜像里最新那条根带的回退下界
    /// （`crash_injection::raised_floor_of_the_newest_root_lands_only_on_abandoned_roots`）。
    pub raised_floor_lands_only_on_abandoned_roots: Option<bool>,
    /// 记录核对器（`crash::check_records_against`）在这个盘面上判出的两类：活盘面那一路不跑它，恒是默认值（两项都 false）。
    pub record_check: RecordCheck,
}

impl FailureObservation {
    /// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
    #[must_use]
    pub fn root_ring_has_turned(&self) -> bool {
        root_ring_has_turned(self.newest_ring_root_txg, self.root_ring_slot_count)
    }
}

/// I-3.1 的违例文字 `盘 N：记账的已分配 Some(A)，遍历全部有效根得到 B；机理：…`（`singlefs-checker` 的 `walk.rs`）里的 A 与 B；
/// 读不出就 None。B 后面跟着机理标识那一段，所以只取紧跟着的那串数字，不把整段尾巴拿去 parse。
#[must_use]
pub fn allocated_and_walked_bytes(detail: &str) -> Option<(u64, u64)> {
    let after_allocated = detail.split_once("记账的已分配 Some(")?.1;
    let (allocated_text, rest) = after_allocated.split_once(')')?;
    Some((
        allocated_text.parse().ok()?,
        leading_number_after(rest, "遍历全部有效根得到 ")?,
    ))
}

/// `label` 之后紧跟着的那串十进制数字；`label` 不在、或它后面不是数字，都是 None。
fn leading_number_after(text: &str, label: &str) -> Option<u64> {
    text.split_once(label)?
        .1
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// I-3.1 判红时 checker 在说明文字里带的机理标识（`singlefs-checker` 的 `walk.rs` 写的那一段）：遍历覆盖了哪些根槽、
/// 剩下的按什么理由没覆盖。「已知红」清单按它分辨机理，不按签名（只有 I-3.1 红、记账多于遍历）认人。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationStatisticMechanism {
    /// 根环一圈的槽数 R × S。
    pub root_ring_slot_count: u64,
    /// 环里最新那条根的 txg。
    pub newest_root_txg: u64,
    /// 环里自证过的根槽个数。
    pub readable_root_slot_count: u64,
    /// 环里最老的那条自证过的根的 txg。
    pub oldest_readable_root_txg: u64,
    /// 遍历真走过的候选根槽个数。
    pub walked_root_slot_count: u64,
    /// 被最新根的实例表判成「被抛弃的时间线」而没走的根槽个数。
    pub root_slots_dropped_as_abandoned: u64,
    /// 最新根带的回退下界 F。
    pub rollback_floor: u64,
    /// 低于 F 而没走的根槽个数。
    pub root_slots_dropped_below_floor: u64,
}

/// 从 I-3.1 的说明文字里读机理标识；少一项就 None（说明这条违例不是那一段格式写出来的）。
#[must_use]
pub fn allocation_statistic_mechanism(detail: &str) -> Option<AllocationStatisticMechanism> {
    Some(AllocationStatisticMechanism {
        root_ring_slot_count: leading_number_after(detail, "根环槽数 ")?,
        newest_root_txg: leading_number_after(detail, "最新根 txg ")?,
        readable_root_slot_count: leading_number_after(detail, "环里自证过的根槽 ")?,
        oldest_readable_root_txg: leading_number_after(detail, "最老的自证过的根 txg ")?,
        walked_root_slot_count: leading_number_after(detail, "遍历的候选根槽 ")?,
        root_slots_dropped_as_abandoned: leading_number_after(detail, "被实例表判抛弃的根槽 ")?,
        rollback_floor: leading_number_after(detail, "回退下界 F ")?,
        root_slots_dropped_below_floor: leading_number_after(detail, "低于 F 的根槽 ")?,
    })
}

/// 「已知红」清单的一条：形态与它在增补 2 收口表里的那一行。
pub struct KnownRedForm {
    /// 增补 2 收口表（`.claude/kb/milestone/02-second-txn.md`「增补 2」）里的哪一行。
    pub closeout_table_row: &'static str,
    /// 形态：哪条不变量或哪种 panic、在什么操作之后。
    pub shape: &'static str,
    pub matches: fn(&FailureObservation) -> bool,
}

/// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/random_histories.rs` 里钉着）要转绿。
/// 增补 2 收口表第 43 行（F 抬进被抛弃实例留下的空档、抬 F 之后只有 I-3.1 红、记账多于遍历）取丁-defer 修好
/// 它的最小复现（`tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）
/// 与随机历史那边的复现都跑绿，清单里没有已知红的形态（代码轮收口表第 43 行那一项）；清单此刻是空的。
pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];

/// 新发现的「同一个」：panic 按位置、违例按判红的不变量集合。收缩只留签名不变的删法。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureSignature {
    Panic {
        location: String,
    },
    HarnessJudgement {
        judgement: &'static str,
    },
    ModelDisagreement {
        aspect: &'static str,
    },
    /// 记录核对器判红的那几类（`crash::RecordCheck`，只有崩溃状态上跑得到）。
    RecordCheck {
        aspects: Vec<&'static str>,
    },
    CheckerViolations {
        invariants: Vec<&'static str>,
    },
}

/// 记录核对器判红的那几类的名字，按 `RecordCheck` 的字段序；两项都 false 时是空的。
#[must_use]
pub fn record_check_aspects(check: &RecordCheck) -> Vec<&'static str> {
    let RecordCheck {
        root_without_record,
        claimed_state_missing_unit,
    } = check;
    let mut aspects = Vec::new();
    if *root_without_record {
        aspects.push("root_without_record");
    }
    if *claimed_state_missing_unit {
        aspects.push("claimed_state_missing_unit");
    }
    aspects
}

impl FailureSignature {
    /// panic 先于执行器判的失败，执行器判的失败先于模型对不上，模型对不上先于记录核对器，记录核对器先于 checker 的违例
    /// （同一步里都有时签名取前一种，别的照样在观察里）。
    pub(super) fn of(observation: &FailureObservation) -> Self {
        match (
            &observation.panic,
            &observation.harness_judgement,
            &observation.model_disagreement,
        ) {
            (Some(panic), Some(_) | None, Some(_) | None) => FailureSignature::Panic {
                location: panic.location.clone(),
            },
            (None, Some(judgement), Some(_) | None) => FailureSignature::HarnessJudgement {
                judgement: judgement.name(),
            },
            (None, None, Some(disagreement)) => FailureSignature::ModelDisagreement {
                aspect: disagreement.aspect.name(),
            },
            (None, None, None) => {
                let aspects = record_check_aspects(&observation.record_check);
                if aspects.is_empty() {
                    FailureSignature::CheckerViolations {
                        invariants: observation
                            .violations
                            .iter()
                            .map(|(invariant, _)| *invariant)
                            .collect(),
                    }
                } else {
                    FailureSignature::RecordCheck { aspects }
                }
            }
        }
    }
}

/// 一次失败对「已知红」清单：按清单次序第一条对得上的就是它，都对不上是新发现。
#[must_use]
pub fn classify_failure(observation: FailureObservation) -> HistoryEnding {
    match KNOWN_RED_FORMS
        .iter()
        .position(|form| (form.matches)(&observation))
    {
        Some(form) => HistoryEnding::KnownRed { form, observation },
        None => HistoryEnding::NewFinding {
            signature: FailureSignature::of(&observation),
            observation,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// I-3.1 的违例文字读得出记账与遍历两个数；读不出（None、别的措辞）就不认。
    #[test]
    fn allocated_and_walked_bytes_are_read_from_the_checker_detail() {
        assert_eq!(
            allocated_and_walked_bytes(
                "盘 0：记账的已分配 Some(3031040)，遍历全部有效根得到 2850816"
            ),
            Some((3_031_040, 2_850_816))
        );
        assert_eq!(
            allocated_and_walked_bytes("盘 1：记账的已分配 None，遍历全部有效根得到 0"),
            None
        );
        assert_eq!(allocated_and_walked_bytes("别的措辞"), None);
    }

    /// checker 在 I-3.1 的说明文字里带的那一段机理标识（`singlefs-checker` 的 `walk.rs` 写的格式）。
    fn allocation_statistic_detail(
        newest_root_txg: u64,
        oldest_readable_root_txg: u64,
        rollback_floor: u64,
        root_slots_dropped_below_floor: u64,
    ) -> String {
        format!(
            "盘 0：记账的已分配 Some(3817472)，遍历全部有效根得到 3801088；机理：根环槽数 24、最新根 txg {newest_root_txg}、环里自证过的根槽 24 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 20 个、被实例表判抛弃的根槽 0 个、回退下界 F {rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    }

    /// 收口表第 43 行那一形的失败观察：抬 F 那一步之后只有 I-3.1 红、记账多算，F 落在回退留下的空档里、环没转圈，
    /// checker 报的机理是「F 把环里读得到的根挡在了遍历之外」（`KNOWN_RED_FORMS` 那一条要的全都有）。
    fn raised_into_the_gap_observation() -> FailureObservation {
        FailureObservation {
            position: StepPosition::Operation(10),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: vec![("I-3.1", allocation_statistic_detail(9, 0, 5, 4))],
            panic: None,
            newest_ring_root_txg: Some(9),
            root_ring_slot_count: Some(24),
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: Some(true),
            record_check: RecordCheck::default(),
        }
    }

    /// 执行器判出的失败不进「已知红」：哪怕同一步 checker 判的正是清单那一形，签名取执行器判的那一种。
    #[test]
    fn harness_judgement_is_never_classified_as_a_known_red_form() {
        let mut observation = raised_into_the_gap_observation();
        observation.harness_judgement =
            Some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                records: vec![record(50178, 1, 5, false)],
                first_publish_txg: CheckpointTxg(10),
                last_publish_txg: CheckpointTxg(10),
            });
        let HistoryEnding::NewFinding { signature, .. } = classify_failure(observation.clone())
        else {
            panic!("要是新发现");
        };
        assert_eq!(
            signature,
            FailureSignature::HarnessJudgement {
                judgement: "改写或新增的分配记录的代不是这次发布的 txg"
            }
        );
        let mut without_the_judgement = observation;
        without_the_judgement.harness_judgement = None;
        assert!(
            matches!(
                classify_failure(without_the_judgement),
                HistoryEnding::NewFinding {
                    signature: FailureSignature::CheckerViolations { .. },
                    ..
                }
            ),
            "去掉执行器判的那一条，签名是 checker 判红的那几条；收口表第 43 行那一形已从清单里删了，照新发现报"
        );
    }

    /// 清单删空之后（收口表第 ② 行、第 43 行两形都修好、都删了），只有 I-3.1 红、记账多于遍历的观察不论 checker 报的机理是哪一种、
    /// F 落没落在被抛弃实例留下的空档里，一律照新发现报。
    #[test]
    fn with_the_known_red_list_empty_an_allocated_statistic_above_walked_is_a_new_finding_whatever_its_mechanism(
    ) {
        assert!(KNOWN_RED_FORMS.is_empty(), "清单是空的");
        let raised_into_the_gap = raised_into_the_gap_observation();
        let mut the_ring_also_turned = raised_into_the_gap.clone();
        the_ring_also_turned.violations = vec![("I-3.1", allocation_statistic_detail(27, 4, 8, 4))];
        the_ring_also_turned.newest_ring_root_txg = Some(27);
        let mut later_step = raised_into_the_gap.clone();
        later_step.operation_kind = Some(HistoryOperationKind::PublishOverwrite);
        for (observation, what) in [
            (
                raised_into_the_gap,
                "F 落在空档里、环没转圈、机理是 F 挡掉了读得到的根",
            ),
            (the_ring_also_turned, "环也转过一圈、缺口出在 F 上"),
            (later_step, "抬 F 之后的别的步上同一机理"),
        ] {
            assert!(
                matches!(
                    classify_failure(observation),
                    HistoryEnding::NewFinding {
                        signature: FailureSignature::CheckerViolations { .. },
                        ..
                    }
                ),
                "{what}：照新发现报"
            );
        }
    }

    /// 记录核对器判红的不接进「已知红」：签名是 `RecordCheck`，照新发现报。
    #[test]
    fn record_check_violations_are_never_absorbed_into_known_red_forms() {
        let mut observation = raised_into_the_gap_observation();
        observation.record_check = RecordCheck {
            root_without_record: true,
            claimed_state_missing_unit: false,
        };
        let HistoryEnding::NewFinding { signature, .. } = classify_failure(observation.clone())
        else {
            panic!("记录核对器判红了，要是新发现");
        };
        assert_eq!(
            signature,
            FailureSignature::RecordCheck {
                aspects: vec!["root_without_record"]
            }
        );
        observation.record_check = RecordCheck::default();
        assert!(
            matches!(
                classify_failure(observation),
                HistoryEnding::NewFinding {
                    signature: FailureSignature::CheckerViolations { .. },
                    ..
                }
            ),
            "去掉记录核对器那一条，签名是 checker 判红的那几条（收口表第 43 行那一形已从清单里删了）"
        );
    }
}
