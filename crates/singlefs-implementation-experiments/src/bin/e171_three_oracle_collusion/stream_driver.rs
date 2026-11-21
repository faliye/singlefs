//! 一条流的驱动：逐次调用实现、跟着推理想模型（O1-对拍）。

use super::*;

/// 一条流驱动时手里的东西：录制流、调用表、理想模型与它提交过的版本目录。每次调用实现的入口都经 [`StreamDriver::call`]：
/// 记返回点与结局，入口 panic 记「实现自崩」，错误或 panic 之后这条流不再往下驱动。
pub(super) struct StreamDriver {
    pub(super) name: StreamName,
    pub(super) stream: SharedStream,
    pub(super) mkfs_operation_count: usize,
    pub(super) switches: RunSwitches,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) model: IdealModel,
    /// 每一步之后把模型根环里的每一版并进来（崩溃注入的做法）：O1-崩溃判官拿它问「恢复到的这一版是不是模型提交过的」。
    pub(super) committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    /// 模型不再往前走的那次调用：第一次对不上的、没有对应答的 `unmount`、出了错或 panic 的调用（读法 6）。
    pub(super) model_stopped_at_call: Option<usize>,
    pub(super) implementation_crash: Option<String>,
    pub(super) halted: bool,
    comparison_control_applied: bool,
}

impl StreamDriver {
    pub(super) fn new(
        name: StreamName,
        stream: SharedStream,
        mkfs_operation_count: usize,
        switches: RunSwitches,
    ) -> Self {
        let parameters = experiment_pool::parameters();
        let model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
            ModelPoolGeometry {
                devices: (0..LOCAL_DEVICE_COUNT)
                    .map(|device_number| {
                        ModelDeviceIdentity(u32::try_from(device_number).expect("设备号"))
                    })
                    .collect(),
                device_size_in_bytes: LOCAL_DEVICE_BYTES,
            },
            parameters.geometry.journal_ring_bytes,
        );
        let mut driver = Self {
            name,
            stream,
            mkfs_operation_count,
            switches,
            calls: Vec::new(),
            model,
            committed_versions: BTreeMap::new(),
            model_stopped_at_call: None,
            implementation_crash: None,
            halted: false,
            comparison_control_applied: false,
        };
        driver.merge_committed_versions();
        driver
    }

    /// O1 开着：关一条（Q3-核）关掉 O1 时模型一步都不推、对拍与崩溃判官都不算。
    pub(super) fn model_is_enabled(&self) -> bool {
        self.switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge)
    }

    fn operations_so_far(&self) -> usize {
        self.stream.operation_count() - self.mkfs_operation_count
    }

    fn merge_committed_versions(&mut self) {
        for (key, file) in self.model.committed_versions() {
            self.committed_versions.insert(key, file);
        }
    }

    fn stop_the_model_at(&mut self, call_index: usize) {
        self.model_stopped_at_call.get_or_insert(call_index);
    }

    /// 调一次实现的入口并记一行：调之前与返回时各读一次流长。交回 `Err(Some(错误))` 是实现交回了错误（调用方照错误造拒绝的观测），
    /// `Err(None)` 是 panic。
    pub(super) fn call<Output, Failure: std::fmt::Debug>(
        &mut self,
        label: &'static str,
        run: impl FnOnce() -> Result<Output, Failure>,
    ) -> Result<Output, Option<Failure>> {
        let first_operation = self.operations_so_far();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));
        let return_point = self.operations_so_far();
        let call_index = self.calls.len();
        let (outcome, returned) = match result {
            Ok(Ok(output)) => (CallOutcome::Succeeded, Ok(output)),
            Ok(Err(failure)) => (
                CallOutcome::ReturnedError(error_member_of(&failure)),
                Err(Some(failure)),
            ),
            Err(payload) => {
                let text = panic_text(&*payload);
                self.implementation_crash = Some(format!("call_{call_index}_{label}:{text}"));
                (CallOutcome::Panicked(text), Err(None))
            }
        };
        if outcome != CallOutcome::Succeeded {
            self.halted = true;
            self.stop_the_model_at(call_index);
        }
        self.calls.push(RecordedCall {
            label: label.to_string(),
            first_operation,
            return_point,
            outcome,
            model: ModelVerdictOfCall::NotModelled,
        });
        returned
    }

    /// 刚记的这次调用模型没有对应的答（卸载）：模型停在这里。
    pub(super) fn note_that_the_model_has_no_answer_for_the_last_call(&mut self) {
        let call_index = self.calls.len() - 1;
        self.stop_the_model_at(call_index);
    }

    /// 取号、暖机两次都返回之后，模型推一步（`acquire_and_warm_up_in_the_make_filesystem_process`）；两次不比。
    pub(super) fn advance_the_model_past_the_acquire_and_the_warm_up(&mut self) {
        if self.model_is_enabled() && self.model_stopped_at_call.is_none() {
            self.model
                .acquire_and_warm_up_in_the_make_filesystem_process();
            self.merge_committed_versions();
        }
    }

    /// 取号或暖机交回了错误（刚记的这次调用）：这两条流不带故障，理想模型对这两次调用只有一种走法（往前走，`model.rs` 里没有 `answer_*`
    /// 可问拒绝），所以实现拒了就是「模型说该成、实现拒了」，记成 O1-对拍的红（登记第一节读法 5 末句：一次调用返回错误而模型说该成功，
    /// 不算自崩，是 O1-对拍的红）。不记的话这次调用的 O1-对拍一栏留在 `not_modelled`、之后的态记不适用，一行这样的坏实现三列都不红。
    /// `failure` 是 `None`（panic，实现自崩）不记；O1 关掉（Q3-核）时不记，那一列写 `off`。
    pub(super) fn record_a_refusal_the_model_only_goes_forward_on<Failure: std::fmt::Debug>(
        &mut self,
        failure: Option<Failure>,
    ) {
        let Some(failure) = failure else {
            return;
        };
        if !self.model_is_enabled() {
            return;
        }
        let call_index = self.calls.len() - 1;
        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {
            aspect:
                singlefs_verification::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    .name()
                    .to_string(),
            model_answer: "往前走".to_string(),
            implementation_answer: format!("拒了:{}", error_member_of(&failure)),
        };
        self.stop_the_model_at(call_index);
    }

    /// 拿刚记的这次调用的结局与模型的答案比（与 `history.rs` 的 `judge_by_model` 同一种比法：`answer` 出答案、`judge_and_advance` 比并推）。
    /// 对得上，模型往前走并把它根环里的版本并进目录；对不上，记第一格并让模型停在这次调用。同一次调用可以交几次观测
    /// （可写挂载之后准入推的抬 F）：前面一处对不上，后面不再比。
    pub(super) fn judge(
        &mut self,
        answer: impl FnOnce(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
        mut observed: ObservedOutcome,
    ) {
        if !self.model_is_enabled() {
            return;
        }
        let call_index = self.calls.len() - 1;
        if self
            .model_stopped_at_call
            .is_some_and(|stopped| stopped < call_index)
        {
            self.calls[call_index].model = ModelVerdictOfCall::SkippedAfterTheModelStopped;
            return;
        }
        if matches!(
            self.calls[call_index].model,
            ModelVerdictOfCall::Disagrees { .. }
        ) {
            return;
        }
        if self.switches.positive_control == Some(PositiveControl::ComparisonOfTheCalls)
            && self.name == StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
            && self.calls[call_index].label == "publish"
            && !self.comparison_control_applied
        {
            if let ObservedOutcome::Succeeded(ObservedEffect::Publishes { roots, .. }) =
                &mut observed
            {
                if let Some(ObservedFile::Decoded(content)) =
                    roots.first_mut().map(|root| &mut root.file)
                {
                    if let Some(last_byte) = content.last_mut() {
                        *last_byte ^= 1;
                        self.comparison_control_applied = true;
                    }
                }
            }
        }
        let verdict =
            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));
        match verdict {
            Ok(_) => {
                self.calls[call_index].model = ModelVerdictOfCall::Agrees;
                self.merge_committed_versions();
            }
            Err(disagreement) => {
                self.calls[call_index].model = ModelVerdictOfCall::Disagrees {
                    aspect: disagreement.aspect.name().to_string(),
                    model_answer: disagreement.model_answer,
                    implementation_answer: disagreement.implementation_answer,
                };
                self.stop_the_model_at(call_index);
            }
        }
    }

    /// 实现交回错误时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
    pub(super) fn refusal(
        &self,
        error: &dyn std::fmt::Debug,
        reason: singlefs_verification::model::ObservedRefusalReason,
        publishes_completed: usize,
        stream_length_before: usize,
        reported_ceiling: Option<singlefs_verification::model::ModelCheckpointTxg>,
        slot_still_bad: Option<singlefs_verification::model::ModelRingPosition>,
    ) -> ObservedOutcome {
        ObservedOutcome::Refused {
            member: error_member_of(error),
            reason,
            publishes_completed,
            wrote_anything: self.stream.operation_count() != stream_length_before,
            reported_ceiling,
            reported_root_ring_slot_still_bad_after_one_reread: slot_still_bad,
        }
    }
}

// ───────────────────────────── 驱动两条流（登记 5.3：不 `.expect`、跟着推模型） ─────────────────────────────

pub(super) fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

#[cfg(test)]
mod tests;
