//! 一次注入判完之后记下的：注入点落在这段历史的哪一段、抽到的故障、重开走到的那一版、结局与新发现、
//! 注入之后才红与注入那一刻就红的已知红形，与按种子累加的计数 `FaultInjectionTally`。

use super::*;

/// 注入点落在这段历史的哪一段上：起点段，还是起点之后的第几步（那一步是哪一类操作）。
///
/// 起点段是 `HistoryPool::start` 里的 mkfs、取号、暖机、第一个文件，在录制流里排在第一个操作之前。
/// 改法二把它放开：在那之前 `draw_faults` 从第 1 步起算，起点段结构性地一次都摆不到
/// （判定五：49 次 panic 全在 `history.rs` 起点段那几处 `expect` 上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultedSegment {
    TheStartingPoint,
    Operation {
        /// 起点之后的第几步，从 0 数。
        step_index: usize,
        operation_kind: HistoryOperationKind,
    },
}

impl FaultedSegment {
    /// 这一段在执行器里的位置，拿去与 `FailureObservation::position` 比。
    #[must_use]
    pub const fn position(self) -> StepPosition {
        match self {
            FaultedSegment::TheStartingPoint => StepPosition::StartingPoint,
            FaultedSegment::Operation { step_index, .. } => StepPosition::Operation(step_index),
        }
    }

    /// 这一段是哪一类操作；起点段不是历史里的一步，没有这一项。
    #[must_use]
    pub const fn operation_kind(self) -> Option<HistoryOperationKind> {
        match self {
            FaultedSegment::TheStartingPoint => None,
            FaultedSegment::Operation { operation_kind, .. } => Some(operation_kind),
        }
    }

    /// 跨段可比的那一半（不带步号）：注入点按它分类。
    #[must_use]
    pub const fn kind(self) -> FaultedSegmentKind {
        match self {
            FaultedSegment::TheStartingPoint => FaultedSegmentKind::TheStartingPoint,
            FaultedSegment::Operation { operation_kind, .. } => {
                FaultedSegmentKind::Operation(operation_kind)
            }
        }
    }

    #[must_use]
    pub fn render(self) -> String {
        match self {
            FaultedSegment::TheStartingPoint => {
                "起点段（mkfs、取号、暖机、第一个文件）".to_string()
            }
            FaultedSegment::Operation {
                step_index,
                operation_kind,
            } => format!("第 {step_index} 步（{operation_kind:?}）"),
        }
    }
}

/// 注入点分类用的「哪一段」：起点段，或起点之后的某一类操作。不带步号，几段历史之间可比。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultedSegmentKind {
    TheStartingPoint,
    Operation(HistoryOperationKind),
}

impl FaultedSegmentKind {
    /// 报告里从上到下的次序：起点段排在七类操作之前（它在历史里也排在最前面）。
    #[must_use]
    pub fn every_kind_in_report_order() -> Vec<FaultedSegmentKind> {
        let mut kinds = vec![FaultedSegmentKind::TheStartingPoint];
        kinds.extend(
            HistoryOperationKind::ALL
                .into_iter()
                .map(FaultedSegmentKind::Operation),
        );
        kinds
    }

    #[must_use]
    pub fn render(self) -> String {
        match self {
            FaultedSegmentKind::TheStartingPoint => "TheStartingPoint".to_string(),
            FaultedSegmentKind::Operation(operation_kind) => format!("{operation_kind:?}"),
        }
    }
}

/// 一段历史上注入哪一次调用：注入什么，加上那次调用是整池第几次这一种调用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawnFault {
    pub fault: InjectedFault,
    /// 整池第几次这一种调用（1 起）。
    pub call_ordinal: u64,
    /// 这次调用落在哪一段里。
    pub segment: FaultedSegment,
}

impl DrawnFault {
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "{}：整池第 {} 次{}，落在{}",
            self.fault.name(),
            self.call_ordinal,
            self.fault.call_kind().name(),
            self.segment.render()
        )
    }
}

/// 一个注入点：哪一段上的哪一种调用（写再按落点分成哪一类结构）。验收要的「每个注入点的命中次数」按它数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InjectionPoint {
    pub segment: FaultedSegmentKind,
    pub call: FaultCallKind,
    /// 写落在哪一类结构上；读与屏障没有这一项。
    pub written_structure: Option<StepKind>,
}

/// 写落在哪一类结构上，四种（`StepKind::Barrier` 不是写）。
const WRITTEN_STRUCTURES: [StepKind; 4] = [
    StepKind::UnitWrite,
    StepKind::JournalRecord,
    StepKind::RootRecordFua,
    StepKind::SystemConfigurationSlot,
];

impl InjectionPoint {
    /// 全部注入点：（起点段 + 七类操作） × （四类写落点 + 读 + 屏障）。里面有几格在今天的实现上摆不出来
    /// （冷启动只读不写、零单元发布不写单元），报告里按「没命中」列名，不当失败。
    #[must_use]
    pub fn every_injection_point() -> Vec<InjectionPoint> {
        let mut points = Vec::new();
        for segment in FaultedSegmentKind::every_kind_in_report_order() {
            for structure in WRITTEN_STRUCTURES {
                points.push(InjectionPoint {
                    segment,
                    call: FaultCallKind::Write,
                    written_structure: Some(structure),
                });
            }
            for call in [FaultCallKind::Read, FaultCallKind::Barrier] {
                points.push(InjectionPoint {
                    segment,
                    call,
                    written_structure: None,
                });
            }
        }
        points
    }

    #[must_use]
    pub fn render(&self) -> String {
        match self.written_structure {
            Some(structure) => format!("{}/{}", self.segment.render(), structure.name()),
            None => format!("{}/{}", self.segment.render(), self.call.name()),
        }
    }
}

/// 注入之后重开走到的那一版落在哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReopenedVersion {
    /// 走到的是模型认下来的某一版（注入那一段历史跑完时模型根环里的那几条，`HistoryRun::committed_versions_at_the_end`）。
    CommittedByTheModel,
    /// 模型没认这一版，但它正是不注入时失败那一步会写出的那一版（增补 2 收口表第 40 行 / C381 在争的那一格；这里只记不判）。
    TheVersionTheFaultedStepWasWriting,
    /// 两样都不是：`crash_recovery_disagreement` 判出对不上。
    OutsideEverythingTheModelAllows,
    /// 重开走读失败，而这一次注入之后盘上本来就没有一版立得住（`failed_reopen_is_the_right_answer`）：
    /// 走读失败正是要的结果，只记不判。悄悄读回一版模型从没提交过的内容照样判红。
    NothingWasLeftToRecoverAndTheReopenSaidSo,
}

/// 注入之后这一次怎么收尾。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultOutcome {
    /// 注入那一步返回了 `Err`（或冷启动读回报错），历史到此为止：要的就是这一条。
    SurfacedAsAnError,
    /// 注入了，那一步照样做成了（读错被容错、吞掉的写没被用到），整段历史跑完。
    ToleratedAndTheHistoryFinished,
    /// 注入了，历史却停在别的东西上（不是注入那一步返回的错）：这一格逐条分类，有真缺口就在这里。
    StoppedOnSomethingOtherThanTheInjectedError,
    /// 注入点没走到（这一段历史在注入那一次调用之前就停了）。
    NeverFired,
}

impl FaultOutcome {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            FaultOutcome::SurfacedAsAnError => "返回了错误",
            FaultOutcome::ToleratedAndTheHistoryFinished => "被容错、历史跑完",
            FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => "历史停在别的东西上",
            FaultOutcome::NeverFired => "没走到",
        }
    }
}

/// 一次注入上的一条失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub drawn: DrawnFault,
    pub observation: FailureObservation,
    /// 重开走到了哪一版（判红的那一次才带）。
    pub reopen: String,
}

impl FaultFinding {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "种子 {} {}：{:?}\n",
            self.seed.0,
            self.drawn.render(),
            self.signature
        );
        let _ = writeln!(text, "  重开：{}", self.reopen);
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic：{} —— {}", panic.location, panic.message);
        }
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        text
    }
}

/// 只有故障注入撞得到的「已知红」形态：随机历史不注入，撞不到；两边共用的那份清单是 `history::KNOWN_RED_FORMS`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnownRedFormOnlyAfterAFault {
    /// 起点段里 mkfs 自己把错交回来，录制流里一次系统配置槽写都没落盘；注入之后的镜像上池级 checker 只判红
    /// I-8.5（tail 槽有效集非空）一条，重开走读失败（[`ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo`]）。
    /// 盘上全池没有一个自证过的系统配置槽，就是 I-8.5 那一行写的「全池皆废」，红是对的，照判、记在这里；
    /// 它不是要修的缺口。
    MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
}

impl KnownRedFormOnlyAfterAFault {
    /// 报告里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded => {
                "mkfs 在系统配置槽落盘之前报错，全池皆废、只红 I-8.5"
            }
        }
    }
}

/// 一次注入以「已知红」收尾时撞上的是哪一条：两边共用的清单里第几条，或只有故障注入撞得到的那几条之一。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnownRedFormAtAFault {
    /// `history::KNOWN_RED_FORMS` 的第几条（从 0 数）。
    SharedWithRandomHistories(usize),
    OnlyAfterAFault(KnownRedFormOnlyAfterAFault),
}

impl KnownRedFormAtAFault {
    /// 报告里的一段：共用清单写第几条，故障注入自己那几条写名字。
    #[must_use]
    pub fn render(self) -> String {
        match self {
            KnownRedFormAtAFault::SharedWithRandomHistories(form) => format!("第 {form} 条"),
            KnownRedFormAtAFault::OnlyAfterAFault(form) => {
                format!("故障注入那一条（{}）", form.name())
            }
        }
    }
}

/// 判故障注入自己那几条「已知红」要看的事实，都是 [`inject_one_fault_into_the_segment`] 一路判到这一步时本来就拿着的：不另读盘、不另跑实现。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactsAfterTheFault {
    /// 注入那一段历史停在起点段某一步把错交回来上（`HarnessJudgement::TheStartingPointReturnedAnError`）时是哪一步；
    /// 别的结局（跑完了、停在起点之后的某一步、停在 panic 上）是 None。
    pub starting_point_step_that_returned_an_error: Option<StartingPointStep>,
    /// 录制流（只记真正落到盘上的写）里写在系统配置槽上的写有几次（[`StepKind::SystemConfigurationSlot`]；整段清零不算）。
    pub system_configuration_slot_writes_that_landed: usize,
    /// 重开走到了哪一格。
    pub reopened: ReopenedVersion,
}

/// 这条失败是不是只有故障注入撞得到的那几条「已知红」之一；不是就交回 None，照共用清单与新发现往下判。
///
/// mkfs 没做完那一条要五样同时成立，少一样就不认（照报新发现，不放宽）：
/// 1. 停在起点段、把错交回来的是 mkfs 自己（取号、暖机、第一个文件报错时 mkfs 已经写下了系统配置槽）；
/// 2. 录制流里一次系统配置槽写都没落盘（mkfs 写完系统配置之后才报错的，盘上有自证过的槽，I-8.5 红就不是这一形）；
/// 3. 重开走读失败、而盘上本来就没有一版立得住；
/// 4. 这条失败里没有 panic、执行器判出的失败、模型对不上、记录核对器判红；
/// 5. checker（连同 RefFS 借违例那一栏带的那一票）判红的恰好一条，就是 I-8.5。
#[must_use]
pub fn known_red_form_only_after_a_fault(
    observation: &FailureObservation,
    facts: &FactsAfterTheFault,
) -> Option<KnownRedFormOnlyAfterAFault> {
    let make_filesystem_returned_the_error = match facts.starting_point_step_that_returned_an_error
    {
        Some(StartingPointStep::MakeFilesystem) => true,
        Some(
            StartingPointStep::ReadThePoolForTheSession
            | StartingPointStep::AcquireInstance
            | StartingPointStep::WarmUp
            | StartingPointStep::PublishFirstFile,
        )
        | None => false,
    };
    let no_system_configuration_slot_landed =
        facts.system_configuration_slot_writes_that_landed == 0;
    let the_reopen_found_nothing_to_recover = match facts.reopened {
        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo => true,
        ReopenedVersion::CommittedByTheModel
        | ReopenedVersion::TheVersionTheFaultedStepWasWriting
        | ReopenedVersion::OutsideEverythingTheModelAllows => false,
    };
    let only_the_checker_flagged_it = observation.panic.is_none()
        && observation.harness_judgement.is_none()
        && observation.model_disagreement.is_none()
        && crate::history::record_check_aspects(&observation.record_check).is_empty();
    let only_some_device_carries_a_self_verified_system_configuration_is_violated = matches!(
        observation.violations.as_slice(),
        [(invariant, _)] if *invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
    );
    (make_filesystem_returned_the_error
        && no_system_configuration_slot_landed
        && the_reopen_found_nothing_to_recover
        && only_the_checker_flagged_it
        && only_some_device_carries_a_self_verified_system_configuration_is_violated)
        .then_some(
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
        )
}

/// 「已知红」收尾的一次注入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtAFault {
    pub seed: HistorySeed,
    pub form: KnownRedFormAtAFault,
    pub drawn: DrawnFault,
}

/// 跑过的注入的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FaultInjectionTally {
    pub histories: u64,
    /// 测量跑（不注入）就以「已知红」收尾的段数：注入点只摆在测量跑跑完的那几步里。
    pub histories_whose_measurement_run_stopped_early: u64,
    pub histories_without_any_injection_point: u64,
    pub faults: u64,
    pub faults_by_kind: BTreeMap<&'static str, u64>,
    pub faults_by_outcome: BTreeMap<&'static str, u64>,
    pub faults_by_injection_point: BTreeMap<String, u64>,
    /// 注入之后整段历史里 `singlefs-core` panic 的次数：这是被测的性质，一次都不许有。
    pub faults_that_panicked: u64,
    /// 说谎的设备（[`InjectedFault::the_device_lies`]）吞掉写之后留下的失败、按注入点认得出是它留下的次数
    /// （[`left_by_the_lying_device`]）：不算新发现，按签名列进报告。认不出的不进这一格，照报成新发现。
    /// 一次都没有说明说谎那两种注得进去却什么也没碰着，判别力要重新量。
    pub faults_where_a_lying_device_left_the_image_inconsistent: u64,
    /// 上一格逐条的签名：判红的是哪几条不变量 / 哪一处模型对不上。
    pub lying_device_signatures: BTreeMap<String, u64>,
    /// 注入那一步返回的错误成员（`StepOutcome::Refused` 的成员名）。
    pub refusal_members: BTreeMap<String, u64>,
    /// 起点段哪一步把错交回来了（`StartingPointStep::name`）：改法二把 `HistoryPool::start` 那四处 `expect`
    /// 改成可失败之后才有这一格。
    pub starting_point_failures: BTreeMap<&'static str, u64>,
    /// 取号之后挂载报错、新号没回卷的次数，按 [`AcquiredInstanceLeftAfterAFailedMount::name`] 分（C378；认下的已知行为，
    /// 不算新发现，在报告里点名）。
    pub acquired_instances_left_after_a_failed_mount: BTreeMap<&'static str, u64>,
    pub reopens: u64,
    pub reopens_reading_a_file: u64,
    pub reopens_without_a_file: u64,
    pub reopens_that_failed: u64,
    pub reopened_into_a_version_the_model_committed: u64,
    /// 重开走到了失败那一步正在写、模型没提交的那一版（收口表第 40 行那一格；只记不判）。
    pub reopened_into_the_version_the_faulted_step_was_writing: u64,
    pub reopens_outside_everything_the_model_allows: u64,
    /// 重开走读失败，而盘上本来就没有一版立得住（`failed_reopen_is_the_right_answer`）：只记不判。
    pub reopens_that_correctly_found_nothing_to_recover: u64,
    /// 重开上 RefFS 与 O1 判官两票合起来（任一红即红），按哪一边红分开计；RefFS 弃权的按理由另计。
    pub reopens_by_the_reference_filesystem_and_model_verdicts:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_abstentions_at_the_reopen: BTreeMap<&'static str, u64>,
    /// harness 在注入之后重开的盘上就地跑池级 checker 的次数：那一遍归给定状态引擎两段，harness 一次都不跑，这一格恒为 0
    /// （用例照它核 harness 没把就地判偷回来）；下面三格（判绿、不适用、部分没判的条次）同样恒空。
    pub checker_runs_on_the_image_after_the_fault: u64,
    /// 注入之后重开的盘经钩子交出去的次数（[`ImagesAfterAFault`]，每个注入点一次；池级 checker 那一遍归给定状态引擎两段）。
    /// 合并入口（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`]）交给判的那几次不记在这一格。
    pub images_after_the_fault_handed_out: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 判成「部分没判」（`singlefs_checker::image::InvariantVerdict::PartiallyJudged`）的条次：不红，也不算判绿。
    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
    pub faults_ending_known_red: BTreeMap<KnownRedFormAtAFault, u64>,
    pub faults_ending_new_finding: u64,
}

impl FaultInjectionTally {
    /// 把另一份计数并进来（按片的次序相加，结果与线程数无关）。
    pub fn absorb(&mut self, other: &FaultInjectionTally) {
        self.histories += other.histories;
        self.histories_whose_measurement_run_stopped_early +=
            other.histories_whose_measurement_run_stopped_early;
        self.histories_without_any_injection_point += other.histories_without_any_injection_point;
        self.faults += other.faults;
        for (key, count) in &other.faults_by_kind {
            *self.faults_by_kind.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_outcome {
            *self.faults_by_outcome.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_injection_point {
            *self
                .faults_by_injection_point
                .entry(key.clone())
                .or_insert(0) += count;
        }
        self.faults_that_panicked += other.faults_that_panicked;
        self.faults_where_a_lying_device_left_the_image_inconsistent +=
            other.faults_where_a_lying_device_left_the_image_inconsistent;
        for (key, count) in &other.lying_device_signatures {
            *self.lying_device_signatures.entry(key.clone()).or_insert(0) += count;
        }
        for (key, count) in &other.refusal_members {
            *self.refusal_members.entry(key.clone()).or_insert(0) += count;
        }
        for (key, count) in &other.starting_point_failures {
            *self.starting_point_failures.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.acquired_instances_left_after_a_failed_mount {
            *self
                .acquired_instances_left_after_a_failed_mount
                .entry(key)
                .or_insert(0) += count;
        }
        self.reopens += other.reopens;
        self.reopens_reading_a_file += other.reopens_reading_a_file;
        self.reopens_without_a_file += other.reopens_without_a_file;
        self.reopens_that_failed += other.reopens_that_failed;
        self.reopened_into_a_version_the_model_committed +=
            other.reopened_into_a_version_the_model_committed;
        self.reopened_into_the_version_the_faulted_step_was_writing +=
            other.reopened_into_the_version_the_faulted_step_was_writing;
        self.reopens_outside_everything_the_model_allows +=
            other.reopens_outside_everything_the_model_allows;
        self.reopens_that_correctly_found_nothing_to_recover +=
            other.reopens_that_correctly_found_nothing_to_recover;
        for (verdict, count) in &other.reopens_by_the_reference_filesystem_and_model_verdicts {
            *self
                .reopens_by_the_reference_filesystem_and_model_verdicts
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (reason, count) in &other.reference_filesystem_abstentions_at_the_reopen {
            *self
                .reference_filesystem_abstentions_at_the_reopen
                .entry(reason)
                .or_insert(0) += count;
        }
        self.checker_runs_on_the_image_after_the_fault +=
            other.checker_runs_on_the_image_after_the_fault;
        self.images_after_the_fault_handed_out += other.images_after_the_fault_handed_out;
        for (key, count) in &other.invariant_holds {
            *self.invariant_holds.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_partially_judged {
            *self.invariant_partially_judged.entry(key).or_insert(0) += count;
        }
        for (form, count) in &other.faults_ending_known_red {
            *self.faults_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.faults_ending_new_finding += other.faults_ending_new_finding;
    }

    /// 一个注入点都没命中的那几格（验收要的「没命中的逐个列名」）。
    #[must_use]
    pub fn injection_points_never_hit(&self) -> Vec<String> {
        InjectionPoint::every_injection_point()
            .into_iter()
            .map(|point| point.render())
            .filter(|name| !self.faults_by_injection_point.contains_key(name))
            .collect()
    }

    /// 这一段上注入过几次（验收要的「每个发布步骤与挂载步骤上各注入过至少一次」与改法二的「起点段注入点大于 0」按它判）。
    #[must_use]
    pub fn faults_on_segment(&self, segment: FaultedSegmentKind) -> u64 {
        let prefix = format!("{}/", segment.render());
        self.faults_by_injection_point
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, count)| count)
            .sum()
    }

    /// 给人看的一整块：绝对数。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：测量跑就提前停的 {} 段、摆不出注入点的 {} 段；注入 {} 次",
            self.histories,
            self.histories_whose_measurement_run_stopped_early,
            self.histories_without_any_injection_point,
            self.faults
        );
        for (kind, count) in &self.faults_by_kind {
            let _ = writeln!(text, "  注入 {kind}：{count} 次");
        }
        for (outcome, count) in &self.faults_by_outcome {
            let _ = writeln!(text, "  收尾「{outcome}」：{count} 次");
        }
        let _ = writeln!(
            text,
            "注入之后 core panic 的次数：{}（被测的性质：返回错误而不是 panic）",
            self.faults_that_panicked
        );
        let _ = writeln!(
            text,
            "说谎的设备丢掉一份内容之后盘面不一致：{} 次（设备丢的，不算新发现）",
            self.faults_where_a_lying_device_left_the_image_inconsistent
        );
        for (signature, count) in &self.lying_device_signatures {
            let _ = writeln!(text, "    {signature}：{count} 次");
        }
        for segment in FaultedSegmentKind::every_kind_in_report_order() {
            let _ = writeln!(
                text,
                "  {} 上注入 {} 次",
                segment.render(),
                self.faults_on_segment(segment)
            );
        }
        for (point, count) in &self.faults_by_injection_point {
            let _ = writeln!(text, "    注入点 {point}：{count} 次");
        }
        let never_hit = self.injection_points_never_hit();
        let _ = writeln!(
            text,
            "  没命中的注入点 {} 个：{}",
            never_hit.len(),
            never_hit.join("、")
        );
        for (member, count) in &self.refusal_members {
            let _ = writeln!(text, "  返回的错误成员 {member}：{count} 次");
        }
        for (step, count) in &self.starting_point_failures {
            let _ = writeln!(text, "  起点段的{step}把错交回来：{count} 次");
        }
        let _ = writeln!(
            text,
            "取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）：用户 2026-09-23 认下的已知行为，D23（journal 的角色与格式） 已定项 16）：{} 次",
            self.acquired_instances_left_after_a_failed_mount
                .values()
                .sum::<u64>()
        );
        for kind in [
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
        ] {
            let _ = writeln!(
                text,
                "  {}：{} 次",
                kind.name(),
                self.acquired_instances_left_after_a_failed_mount
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "重开 {} 次：读回文件 {}、没有文件 {}、走读失败 {}",
            self.reopens,
            self.reopens_reading_a_file,
            self.reopens_without_a_file,
            self.reopens_that_failed
        );
        let _ = writeln!(
            text,
            "  走到模型认下的某一版 {} 次、走到失败那一步正在写的那一版 {} 次（收口表第 40 行那一格，只记不判）、模型都不允许的 {} 次、本来就没一版立得住而走读失败 {} 次（只记不判）",
            self.reopened_into_a_version_the_model_committed,
            self.reopened_into_the_version_the_faulted_step_was_writing,
            self.reopens_outside_everything_the_model_allows,
            self.reopens_that_correctly_found_nothing_to_recover
        );
        let _ = writeln!(
            text,
            "重开上 RefFS 与 O1 判官两票（任一红即红）：{:?}；RefFS 弃权的理由 {:?}",
            self.reopens_by_the_reference_filesystem_and_model_verdicts
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_abstentions_at_the_reopen
        );
        let _ = writeln!(
            text,
            "注入之后重开的盘交出去 {} 次；harness 就地在上面跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次、部分没判 {} 条次",
            self.images_after_the_fault_handed_out,
            self.checker_runs_on_the_image_after_the_fault,
            self.invariant_holds.values().sum::<u64>(),
            self.invariant_not_applicable.values().sum::<u64>(),
            self.invariant_partially_judged.values().sum::<u64>()
        );
        let _ = writeln!(
            text,
            "以「已知红」收尾 {:?}、新发现 {}",
            self.faults_ending_known_red
                .iter()
                .map(|(form, count)| (form.render(), *count))
                .collect::<BTreeMap<_, _>>(),
            self.faults_ending_new_finding
        );
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::StartingPointStep;
    use crate::model::ModelDisagreement;

    /// 注入之后的镜像上 checker 判红的就是 `violated_invariants` 那几条、模型对不上的是 `model_disagreement` 的一条失败，停在起点段。
    fn failure_after_the_fault_at_the_starting_point(
        violated_invariants: &[&'static str],
        model_disagreement: Option<ModelDisagreement>,
    ) -> FailureObservation {
        FailureObservation {
            position: StepPosition::StartingPoint,
            operation_kind: None,
            violations: violated_invariants
                .iter()
                .map(|invariant| (*invariant, "池里 2 块盘一个都没自证过".to_string()))
                .collect(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        }
    }

    /// 注入之后的镜像上只红 I-8.5 的那条失败（mkfs 没做完那一形的签名）。
    fn failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
    ) -> FailureObservation {
        failure_after_the_fault_at_the_starting_point(
            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
            None,
        )
    }

    /// 判那一形要的三样事实，逐样由调用方给。
    fn facts_after_the_fault(
        starting_point_step_that_returned_an_error: Option<StartingPointStep>,
        system_configuration_slot_writes_that_landed: usize,
        reopened: ReopenedVersion,
    ) -> FactsAfterTheFault {
        FactsAfterTheFault {
            starting_point_step_that_returned_an_error,
            system_configuration_slot_writes_that_landed,
            reopened,
        }
    }

    /// mkfs 报错、重开走读失败；落盘的系统配置槽写有几次由调用方给。
    fn make_filesystem_failed_with_system_configuration_slot_writes_landed(
        system_configuration_slot_writes_that_landed: usize,
    ) -> FactsAfterTheFault {
        facts_after_the_fault(
            Some(StartingPointStep::MakeFilesystem),
            system_configuration_slot_writes_that_landed,
            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
        )
    }

    /// 对照（那一形只认「mkfs 在系统配置槽落盘之前报错」）：同一个签名（只红 I-8.5、重开走读失败、mkfs 报的错），
    /// 而录制流里系统配置槽写已经落了盘——mkfs 写完系统配置之后才报错，盘上本该有自证过的槽，I-8.5 红就不是「mkfs 没做完」能交代的，
    /// 照报新发现。同一条失败换成一次都没落盘时认成那一形，证明这一格不是因为判法什么都不认才绿。
    #[test]
    fn tail_witness_invariant_only_failure_after_make_filesystem_wrote_its_system_configuration_slots_stays_a_new_finding(
    ) {
        let observation =
            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
            );
        let slots_written_by_make_filesystem =
            2 * usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                .expect("每盘槽数装得进 usize");
        assert_eq!(
            known_red_form_only_after_a_fault(
                &observation,
                &make_filesystem_failed_with_system_configuration_slot_writes_landed(0)
            ),
            Some(
                KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded
            ),
            "mkfs 在系统配置槽落盘之前报错、只红 I-8.5、重开走读失败：这是用户定的那一形，要认"
        );
        assert_eq!(
            known_red_form_only_after_a_fault(
                &observation,
                &make_filesystem_failed_with_system_configuration_slot_writes_landed(
                    slots_written_by_make_filesystem
                )
            ),
            None,
            "系统配置槽已经落盘（{slots_written_by_make_filesystem} 次）之后的 I-8.5 红不是 mkfs 没做完那一形"
        );
        match classify_failure(observation) {
            HistoryEnding::NewFinding { signature, .. } => assert_eq!(
                signature,
                FailureSignature::CheckerViolations {
                    invariants: vec![SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
                },
                "共用清单接不走它，签名照旧是只红 I-8.5"
            ),
            ending @ (HistoryEnding::Completed | HistoryEnding::KnownRed { .. }) => {
                panic!("只红 I-8.5 的失败不在共用清单里，该是新发现：{ending:?}")
            }
        }
    }

    /// mkfs 没做完那一形不放宽：别的步报错、多红一条或换一条、重开走到别的格、带着模型对不上的，一律不认。
    #[test]
    fn make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen() {
        let observation =
            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
            );
        let facts = make_filesystem_failed_with_system_configuration_slot_writes_landed(0);
        assert!(
            known_red_form_only_after_a_fault(&observation, &facts).is_some(),
            "基准那一条要认，下面每一格只改一样"
        );
        for step in [
            StartingPointStep::ReadThePoolForTheSession,
            StartingPointStep::AcquireInstance,
            StartingPointStep::WarmUp,
            StartingPointStep::PublishFirstFile,
        ] {
            let facts_of_another_step = facts_after_the_fault(
                Some(step),
                0,
                ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
            );
            assert_eq!(
                known_red_form_only_after_a_fault(&observation, &facts_of_another_step),
                None,
                "{}报错时 mkfs 已经做完，不是这一形",
                step.name()
            );
        }
        let facts_without_a_starting_point_error = facts_after_the_fault(
            None,
            0,
            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&observation, &facts_without_a_starting_point_error),
            None,
            "历史没停在起点段的报错上，不是这一形"
        );
        for reopened in [
            ReopenedVersion::CommittedByTheModel,
            ReopenedVersion::TheVersionTheFaultedStepWasWriting,
            ReopenedVersion::OutsideEverythingTheModelAllows,
        ] {
            let facts_of_another_reopen =
                facts_after_the_fault(Some(StartingPointStep::MakeFilesystem), 0, reopened);
            assert_eq!(
                known_red_form_only_after_a_fault(&observation, &facts_of_another_reopen),
                None,
                "重开走到 {reopened:?}，不是这一形"
            );
        }
        let with_one_more_violation = failure_after_the_fault_at_the_starting_point(
            &[
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                "I-4.1",
            ],
            None,
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&with_one_more_violation, &facts),
            None,
            "除了 I-8.5 还红别的，不是这一形"
        );
        let with_another_violation =
            failure_after_the_fault_at_the_starting_point(&["I-4.1"], None);
        assert_eq!(
            known_red_form_only_after_a_fault(&with_another_violation, &facts),
            None,
            "红的不是 I-8.5，不是这一形"
        );
        let with_a_model_disagreement = failure_after_the_fault_at_the_starting_point(
            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
            Some(ModelDisagreement {
                aspect: ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
                model_answer: "该成".to_string(),
                implementation_answer: "拒了".to_string(),
                implementation_reported_ceiling: None,
                implementation_reported_unreadable_place: None,
                implementation_rows_written: None,
            }),
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&with_a_model_disagreement, &facts),
            None,
            "带着模型对不上，不是这一形"
        );
    }
}
