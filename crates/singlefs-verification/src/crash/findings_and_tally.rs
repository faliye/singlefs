//! 崩溃点重放的发现与计数：违例种类、红在哪一趟、发现签名与样本、计数与观察者计数。

use super::*;

/// 崩溃点重放的一个崩溃状态按发布归到哪一格（里程碑「覆盖写、释放、回退与复用」步 6 验收第 1 条「每次发布各多少」）。
/// 归法按段：状态所在的那一段往后数，第一次根槽 FUA 写所在的那一段写出的根，就是这个状态归的那次发布——
/// 上一次发布的根槽写之后、这一次发布的根槽写为止，崩在中间的状态都归这一次。上一次发布的系统配置槽轮换与这一次的单元写
/// 之间没有屏障、并在同一段时，那一段整段归这一次：段是枚举的最小单位，一个状态落在哪一段是确定的，落在哪一次写上不是。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayPublishOfState {
    /// 这次发布的根槽 FUA 写：写表里的下标（按它排就是录制流里的次序）与它写出的根的实例代号、checkpoint_txg。
    UpToTheRootOf {
        root_write_index: usize,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    },
    /// 写表里最后一次根槽写之后的段（最后那次发布的系统配置槽轮换）：后面没有根槽写可归。
    AfterTheLastRoot,
    /// 每一段都整段持久的那一个状态（闭式 1 + Σ(2^|段| − 1) 里的那个 1）。
    EveryWritePersisted,
}

impl CrashPointReplayPublishOfState {
    /// 计数行里的名字：`instance2_txg5`、`after_the_last_root`、`every_write_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            CrashPointReplayPublishOfState::UpToTheRootOf {
                instance,
                checkpoint_txg,
                ..
            } => format!("instance{}_txg{}", instance.0, checkpoint_txg.0),
            CrashPointReplayPublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
            CrashPointReplayPublishOfState::EveryWritePersisted => {
                "every_write_persisted".to_string()
            }
        }
    }

    /// 发现日志里的名字：`instance2_txg5_root_write14`、`after_the_last_root`、`every_write_persisted`。
    /// 比 [`Self::name`] 多带根槽写的下标：两块盘各写一条同身份的根时，两格只差这一项，发现表的签名不许把它们并成一格。
    #[must_use]
    pub fn name_with_the_root_write_index(self) -> String {
        match self {
            CrashPointReplayPublishOfState::UpToTheRootOf {
                root_write_index, ..
            } => format!("{}_root_write{root_write_index}", self.name()),
            CrashPointReplayPublishOfState::AfterTheLastRoot
            | CrashPointReplayPublishOfState::EveryWritePersisted => self.name(),
        }
    }
}

/// 每一段归哪次发布（[`CrashPointReplayPublishOfState`] 的归法）：从最后一段往前走，记着「后面最近的那次根槽写」。
#[must_use]
pub fn publish_of_each_segment(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<CrashPointReplayPublishOfState> {
    let mut next_root: Option<CrashPointReplayPublishOfState> = None;
    let mut publish_of_segment =
        vec![CrashPointReplayPublishOfState::AfterTheLastRoot; segments.len()];
    for (segment_index, segment) in segments.iter().enumerate().rev() {
        if let Some(root_write_index) = segment.iter().rev().copied().find(|write_index| {
            writes
                .get(*write_index)
                .is_some_and(|write| write.kind == StepKind::RootRecordFua)
        }) {
            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);
            next_root = Some(CrashPointReplayPublishOfState::UpToTheRootOf {
                root_write_index,
                instance,
                checkpoint_txg,
            });
        }
        publish_of_segment[segment_index] =
            next_root.unwrap_or(CrashPointReplayPublishOfState::AfterTheLastRoot);
    }
    publish_of_segment
}

/// 崩溃点重放的 oracle（[`oracle_violation_for_versions`]）判的违例是哪一类：崩溃点重放发现表按它分签名（两遍恢复各判一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayOracleViolationKind {
    /// 恢复没择到根。
    NoRootChosen,
    /// 盘上已持久的最新根槽比恢复实际走的根新。
    RecoveredToAnOlderRoot,
    /// 走读失败。
    ReadFailed,
    /// 走到一个没有版本的根、报没有文件，而更旧的根下面有文件。
    NoFileWhereAnOlderRootHasOne,
    /// 走到的根下面有文件却报没有。
    NoFileWhereThisRootHasOne,
    /// 走到的根下面没有文件却读出了内容。
    FileReadWhereThisRootHasNone,
    /// 读回的内容与走到的根写出的那一版不同。
    WrongContent,
}

impl CrashPointReplayOracleViolationKind {
    /// 每一类各一个：[`Self::from_name`] 在里面找（新加一类要加进来，不然红例行里这一类按名字读不回）。
    pub const EVERY_KIND: [Self; 7] = [
        Self::NoRootChosen,
        Self::RecoveredToAnOlderRoot,
        Self::ReadFailed,
        Self::NoFileWhereAnOlderRootHasOne,
        Self::NoFileWhereThisRootHasOne,
        Self::FileReadWhereThisRootHasNone,
        Self::WrongContent,
    ];

    /// 发现日志与 `CRASH_POINT_REPLAY_FINDING` 行里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NoRootChosen => "no_root_chosen",
            Self::RecoveredToAnOlderRoot => "recovered_to_an_older_root",
            Self::ReadFailed => "read_failed",
            Self::NoFileWhereAnOlderRootHasOne => "no_file_where_an_older_root_has_one",
            Self::NoFileWhereThisRootHasOne => "no_file_where_this_root_has_one",
            Self::FileReadWhereThisRootHasNone => "file_read_where_this_root_has_none",
            Self::WrongContent => "wrong_content",
        }
    }

    /// [`Self::name`] 反过来：不是 [`Self::EVERY_KIND`] 里哪一类的名字交回 `None`。
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::EVERY_KIND
            .into_iter()
            .find(|kind| kind.name() == name)
    }
}

/// oracle 判的一处违例：哪一类、给人看的原因。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayOracleViolation {
    pub kind: CrashPointReplayOracleViolationKind,
    pub reason: String,
}

/// 崩溃点重放发现表里一个签名的一半：判红的是哪一遍、违了哪几条。一个状态几遍都红，就各进各的签名。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayRedPass {
    /// 看 journal 那一遍恢复过 oracle 判违例（计进 [`CrashPointReplayTally::violations`]）。
    JournalConsultedOracle(CrashPointReplayOracleViolationKind),
    /// 不看 journal 那一遍恢复过 oracle 判违例（计进 [`CrashPointReplayTally::ignored_violations`]）。
    JournalIgnoredOracle(CrashPointReplayOracleViolationKind),
    /// 池级 checker 判违例的不变量，至少一条；次序照 `check_pool_image` 报的次序（它按 checker 的清单 `IMPLEMENTED_INVARIANTS` 报）。
    PoolChecker(Vec<&'static str>),
    /// 记录核对器：两条判据里至少一条成立。
    RecordChecker(RecordCheck),
}

/// 记录核对器第一条判据在发现日志里的名字。
pub const RECORD_CHECKER_ROOT_WITHOUT_RECORD: &str = "root_without_record";
/// 记录核对器第二条判据在发现日志里的名字。
pub const RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT: &str = "claimed_state_missing_unit";
/// 发现日志里 `pass=` 的四个值，依次是 [`CrashPointReplayRedPass`] 的四个成员。
pub const CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE: &str = "journal_consulted_oracle";
pub const CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE: &str = "journal_ignored_oracle";
pub const CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER: &str = "pool_checker";
pub const CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER: &str = "record_checker";

impl CrashPointReplayRedPass {
    /// 发现日志里 `pass=` 的值。
    #[must_use]
    pub fn pass_name(&self) -> &'static str {
        match self {
            Self::JournalConsultedOracle(_) => CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
            Self::JournalIgnoredOracle(_) => CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
            Self::PoolChecker(_) => CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
            Self::RecordChecker(_) => CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER,
        }
    }

    /// 发现日志里 `violated=` 的那几项（逗号拼起来就是值）。
    #[must_use]
    pub fn violated_names(&self) -> Vec<&'static str> {
        match self {
            Self::JournalConsultedOracle(kind) | Self::JournalIgnoredOracle(kind) => {
                vec![kind.name()]
            }
            Self::PoolChecker(invariants) => invariants.clone(),
            Self::RecordChecker(record_check) => {
                let mut names = Vec::new();
                if record_check.root_without_record {
                    names.push(RECORD_CHECKER_ROOT_WITHOUT_RECORD);
                }
                if record_check.claimed_state_missing_unit {
                    names.push(RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT);
                }
                names
            }
        }
    }
}

/// 一个状态在哪一段：段表里的下标，或最后那个每一段都整段持久的状态（不在任何一段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplaySegmentOfState {
    Segment(usize),
    AllPersisted,
}

impl CrashPointReplaySegmentOfState {
    /// 发现日志与 `CRASH_POINT_REPLAY_PROGRESS` 行里的写法：段号，或 `all_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Segment(segment_index) => segment_index.to_string(),
            Self::AllPersisted => "all_persisted".to_string(),
        }
    }
}

/// 崩溃点重放发现表的签名：判红的是哪一遍、违了哪几条、状态所在的段、状态归哪次发布。大小次序（先 pass 与违了哪几条，再段、再发布）
/// 只在同一个状态上出了几个签名时用来排号。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrashPointReplayFindingSignature {
    pub red_pass: CrashPointReplayRedPass,
    pub segment: CrashPointReplaySegmentOfState,
    pub publish: CrashPointReplayPublishOfState,
}

/// 发现表里每个签名最多留几个样本（最先的几个状态）。
pub const CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT: usize = 3;

/// 签名下的一个样本：状态序号与那一条违例原文。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayFindingSample {
    pub state_ordinal: u64,
    pub violation: String,
}

/// 一个签名下判红的状态：几个，与最先的至多 [`CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT`] 个（序号从小到大，个数 = min(状态数, 3)）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayFinding {
    pub states: u64,
    pub earliest_samples: Vec<CrashPointReplayFindingSample>,
}

impl CrashPointReplayFinding {
    /// 最先那个状态：签名进表时就记了第一个样本。
    ///
    /// # Panics
    /// 一个样本都没有（不变量被破坏：进表的签名至少一个状态、至少一个样本）。
    #[must_use]
    pub fn first_sample(&self) -> &CrashPointReplayFindingSample {
        self.earliest_samples
            .first()
            .expect("进了发现表的签名至少一个状态，第一个状态进表时就记成了样本")
    }
}

/// 崩溃点重放的发现表：判红的状态按签名去重（[`CrashPointReplayFindingSignature`]），每个签名记状态数与最先几个样本；另记至少一遍判红的状态数
/// （一个状态几遍都红只算一次）。只有按段枚举的那几个入口记这一项（签名要状态所在的段与发布）；直接调
/// [`evaluate_state_for_versions`] 评手摆的状态时它是空的。判红的状态逐个经 [`CrashPointReplayFindings::record_red_state`] 记进来
/// （现判时在评那个状态的那一刻，从判定库读回时按库里存的红例行重建）；发现日志里的号按
/// [`CrashPointReplayFindings::signatures_in_first_state_order`] 排。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayFindings {
    pub red_states: u64,
    pub by_signature: BTreeMap<CrashPointReplayFindingSignature, CrashPointReplayFinding>,
}

impl CrashPointReplayFindings {
    /// 记一个判红的状态在一个签名下；这个签名的样本还不满 [`CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT`] 个时才调 `violation` 取原文。
    /// 同一片里状态按序号从小到大评，样本自然从小到大。
    pub(crate) fn record_red_state(
        &mut self,
        signature: CrashPointReplayFindingSignature,
        state_ordinal: u64,
        violation: impl FnOnce() -> String,
    ) {
        let finding =
            self.by_signature
                .entry(signature)
                .or_insert_with(|| CrashPointReplayFinding {
                    states: 0,
                    earliest_samples: Vec::new(),
                });
        finding.states += 1;
        if finding.earliest_samples.len() < CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT {
            finding
                .earliest_samples
                .push(CrashPointReplayFindingSample {
                    state_ordinal,
                    violation: violation(),
                });
        }
    }

    /// 签名按最先那个状态的序号排（同一个状态上的几个签名按签名的大小次序）：发现日志里的号就是这个次序里的第几个（从 1 起）。
    #[must_use]
    pub fn signatures_in_first_state_order(
        &self,
    ) -> Vec<(&CrashPointReplayFindingSignature, &CrashPointReplayFinding)> {
        let mut ordered: Vec<(&CrashPointReplayFindingSignature, &CrashPointReplayFinding)> =
            self.by_signature.iter().collect();
        ordered.sort_by(
            |(left_signature, left_finding), (right_signature, right_finding)| {
                (left_finding.first_sample().state_ordinal, *left_signature)
                    .cmp(&(right_finding.first_sample().state_ordinal, *right_signature))
            },
        );
        ordered
    }
}

/// 崩溃点重放的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayTally {
    pub states: u64,
    /// 枚举出来的状态按发布分（[`CrashPointReplayPublishOfState`] 的归法）：各格之和 = `states`。只有按段枚举的那几个入口
    /// （`enumerate_crash_states*`）记这一项；直接调 [`evaluate_state_for_versions`] 评手摆的状态时它是空的。
    pub states_by_publish: BTreeMap<CrashPointReplayPublishOfState, u64>,
    pub violations: u64,
    pub root_persisted_states: u64,
    pub no_file_states: u64,
    pub file_read_states: u64,
    pub failed_states: u64,
    /// 两遍恢复落到的根、结局码或失败点不同的状态数（`crash_judge_conclusions::recovery_passes_differ`：现判与从库里的恢复字重建按同一组字比）。
    pub journal_differing_states: u64,
    pub first_violation: Option<String>,
    /// 不看 journal 那一遍恢复（`JournalPolicy::Ignore`）过同一个 oracle 判违例的状态数，另计、不混进 `violations`
    /// （靶向对照里「根槽已持久而单元缺席」那一格两遍都违例）。发布 B 之后两遍恢复会读出两个不同的版本，这一遍专判这一点。
    pub ignored_violations: u64,
    pub first_ignored_violation: Option<String>,
    /// 记录核对器判「根在案而记录缺席」的状态数。
    pub record_root_without_record: u64,
    /// 记录核对器判「恢复自称新态而单元缺席」的状态数。
    pub record_claimed_state_missing_unit: u64,
    /// checker：每条不变量在几个状态上评估过（成立或违例）、在几个状态上判违例、第一处违例。
    pub checker_evaluated_states: BTreeMap<&'static str, u64>,
    pub checker_violated_states: BTreeMap<&'static str, u64>,
    pub checker_first_violation: BTreeMap<&'static str, String>,
    /// checker 报「不适用」（这条不变量判的代码在这个状态上没跑到）的状态数：与评估过的状态数分开报，阴性结果不与「没跑到」混在一起
    /// （里程碑「覆盖写、释放、回退与复用」步 6 验收第 3 条）；每条不变量的评估过 + 不适用 = `states`。
    pub checker_not_applicable_states: BTreeMap<&'static str, u64>,
    /// 观察者看过的状态数（没有观察者时是 0）：从库里复用的块不再给观察者看，判段按块从库里存的观察者结论重建这个数与 `observer_counts`；
    /// 有观察者时整条流跑完它必须等于 `states`（崩溃点重放规模）。
    pub observed_states: u64,
    /// 观察者按名字累计的数（[`CrashPointReplayObserverCounts`]），按块存进库（观察者各自的结论种类）。
    pub observer_counts: CrashPointReplayObserverCounts,
    /// 判红的状态按签名去重的发现表（[`CrashPointReplayFindings`]），从库里存的红例行重建。
    pub findings: CrashPointReplayFindings,
}

/// 观察者在状态上记的数，按名字累计（名字只许小写字母、数字、`_`：它原样进库里的结论种类名与打印的计数行）。从库里复用的块不再给观察者看，
/// 观察者要接着用的累计都得记在这里，记在观察者自己的变量里的复用之后就少了那几块。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayObserverCounts(BTreeMap<String, u64>);

impl CrashPointReplayObserverCounts {
    /// 名字能不能当观察者计数的名字：非空，只有小写字母、数字、`_`。
    #[must_use]
    pub fn is_counter_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    }

    /// 给名字为 `name` 的计数加 `amount`。
    ///
    /// # Panics
    /// 名字不合 [`Self::is_counter_name`]。
    pub fn add(&mut self, name: &str, amount: u64) {
        assert!(
            Self::is_counter_name(name),
            "观察者计数的名字只许小写字母、数字、_：{name:?}"
        );
        *self.0.entry(name.to_string()).or_insert(0) += amount;
    }

    /// 名字为 `name` 的计数，没记过是 0。
    #[must_use]
    pub fn count_named(&self, name: &str) -> u64 {
        self.0.get(name).copied().unwrap_or(0)
    }

    /// 按名字排好的全部计数。
    pub fn counts_in_name_order(&self) -> impl Iterator<Item = (&str, u64)> {
        self.0.iter().map(|(name, count)| (name.as_str(), *count))
    }
}

impl CrashPointReplayTally {
    /// checker 那一半按不变量报成一段：`I-x.y=评估过/判违例/不适用`，次序照 checker 的清单。
    #[must_use]
    pub fn checker_counts_by_invariant(&self) -> String {
        let count_of = |counts: &BTreeMap<&'static str, u64>, invariant: &str| {
            counts.get(invariant).copied().unwrap_or(0)
        };
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                format!(
                    "{invariant}={}/{}/{}",
                    count_of(&self.checker_evaluated_states, invariant),
                    count_of(&self.checker_violated_states, invariant),
                    count_of(&self.checker_not_applicable_states, invariant)
                )
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// 按发布分的状态数报成一段：`instance1_txg1=15 … after_the_last_root=3 every_write_persisted=1`，次序照录制流。
    #[must_use]
    pub fn states_by_publish_text(&self) -> String {
        self.states_by_publish
            .iter()
            .map(|(publish, states)| format!("{}={states}", publish.name()))
            .collect::<Vec<String>>()
            .join(" ")
    }
}
