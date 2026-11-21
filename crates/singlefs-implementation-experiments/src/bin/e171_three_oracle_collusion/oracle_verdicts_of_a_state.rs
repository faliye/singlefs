//! 一个取样态上的判：实现读回的那一版、崩溃判官（O1-崩溃判官）、池级 checker（O2）与两段的给定状态表。

use super::*;

/// 一个状态上实现的判定（登记第一节读法 6：「实现都放行」= 池级 checker 一条 Violated 都没有，且记录核对器两项都为假）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImplementationVerdict {
    pub(super) consulted_root: String,
    pub(super) consulted_outcome: String,
    pub(super) ignored_root: String,
    pub(super) ignored_outcome: String,
    pub(super) red_items: Vec<String>,
    pub(super) pool_checker_red: Vec<String>,
    pub(super) root_without_record: bool,
    pub(super) claimed_state_missing_unit: bool,
}

impl ImplementationVerdict {
    pub(super) fn of(judgement: &CrashStateJudgement) -> Self {
        let report_root = |report: &RecoveryReport| root_text(report.effective_root);
        Self {
            consulted_root: report_root(&judgement.consulted),
            consulted_outcome: outcome_text(&judgement.consulted.outcome),
            ignored_root: report_root(&judgement.ignored),
            ignored_outcome: outcome_text(&judgement.ignored.outcome),
            red_items: judgement.red_items(),
            pool_checker_red: judgement
                .pool_checker
                .iter()
                .filter(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => true,
                    InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_) => false,
                })
                .map(|(invariant, _)| (*invariant).to_string())
                .collect(),
            root_without_record: judgement.records.root_without_record,
            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit,
        }
    }
}

// ───────────────────────────── 每个取样状态上的三条 oracle（O1-崩溃判官、O2；O3 在 Python 一侧） ─────────────────────────────

/// O1-崩溃判官在一个状态上的结论（登记第一节读法 1、6）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CrashJudgeVerdict {
    /// 这一态有写落在模型没有答的调用里，或落在对拍判出不同（或出错）之后的调用里（读法 6）。
    NotApplicable,
    Green,
    /// 红：模型那一侧说的话（`ModelDisagreement::implementation_answer`，空白换成下划线）。
    Red {
        what: String,
    },
    /// 关一条（Q3-核）关掉了 O1：这一列不算。
    Off,
}

impl CrashJudgeVerdict {
    pub(super) fn word(&self) -> &'static str {
        match self {
            Self::NotApplicable => "na",
            Self::Green => "green",
            Self::Red { .. } => "red",
            Self::Off => "off",
        }
    }
}

/// 一遍恢复报告的摘要（触达判法 ③，登记第一节读法 4）：择到的根、结局种类、读回内容的 sha256、施加的记录条数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecoverySummary {
    pub(super) root: String,
    pub(super) outcome: String,
    pub(super) read_back_sha256: String,
    pub(super) records_applied: usize,
}

impl RecoverySummary {
    pub(super) fn of(report: &RecoveryReport) -> Self {
        Self {
            root: root_text(report.effective_root),
            outcome: outcome_text(&report.outcome),
            read_back_sha256: match &report.outcome {
                RecoveryOutcome::FileRead { content, .. } => sha256_hexadecimal(content),
                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => "-".to_string(),
            },
            records_applied: report.journal.prefix_applied,
        }
    }
    pub(super) fn text(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.root, self.outcome, self.read_back_sha256, self.records_applied
        )
    }
}

// ───────────────────────────── O2 的盘两段：录入段只录、判段只判录下的 ─────────────────────────────

/// 录入段录下的一张要判的盘：表名与这张表录下的那一份的摘要（判段凭摘要读回）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecordedTableReference {
    pub(super) table_name: String,
    pub(super) digest: [u8; DIGEST_BYTES],
}

/// O2 要判的盘交给谁：录入段只录（只在 CPU 上，不碰给定状态引擎与卡），判段只判录下的。
/// 产品是 [`GivenStatesTablesInTheLibrary`]；单测换成内存里的替身，数录与判各几次。
pub(super) trait PoolCheckerTablesOfTheTwoStages: Sync {
    /// 录入段：这张盘作一张给定状态表录下，交回判段读回它要的引用。
    fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference;

    /// 判段：读回录下的那张盘判，交回池级 checker 判红的不变量名（按 `IMPLEMENTED_INVARIANTS` 的次序，与 `check_pool_image` 交回的次序相同）。
    ///
    /// # Errors
    /// 交接与录下的对不上：照交接里的摘要读不回那张盘（没录过、读回的字节摘要不是它、解不出），一句话点名那张表与出路（重跑录入段）。
    ///
    /// # Panics
    /// 引擎判不了这张盘（起不了、判的时候失败、库读写不了；不退回别处判）。
    fn judge_a_recorded_table(&self, table: &RecordedTableReference)
        -> Result<Vec<String>, String>;
}

/// O2 两段的产品那一种。录入段：这张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合），照给定状态引擎录入段写进库的那一种字节
/// （`RecordedGivenStatesTableWithItsColumns`，两栏不填）按摘要写进这个进程的判定库（prune-server 里那一份；库里已有逐字节相同的那一份就不再写），
/// 不起引擎。判段：照摘要从库里读回，交 [`given_states_engine_of_this_experiment`] 判（引擎在判段第一次判时才起：配了 GPU 且编进了 `gpu` 特性在卡上判，
/// 否则由 CPU 判器判；同一份字节引擎照录入键命中、不再写，判过的复用）。
pub(super) struct GivenStatesTablesInTheLibrary;

impl GivenStatesTablesInTheLibrary {
    /// 这个进程的判定库（第一次用时开；与引擎开的是同一份，`LibraryOpenInThisProcess::open_or_share`）。
    ///
    /// # Panics
    /// 库开不了（连不上 prune-server 时 `open_or_share` 照它的退出码退出进程）。
    fn library() -> &'static LibraryOpenInThisProcess {
        static LIBRARY: std::sync::OnceLock<LibraryOpenInThisProcess> = std::sync::OnceLock::new();
        LIBRARY.get_or_init(|| {
            LibraryOpenInThisProcess::open_or_share()
                .unwrap_or_else(|refusal| panic!("开不了判定库：{refusal}"))
        })
    }
}

impl PoolCheckerTablesOfTheTwoStages for GivenStatesTablesInTheLibrary {
    fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference {
        use singlefs_verification::crash_given_states::GivenStatesOfATable;
        let table = GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: pool,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        };
        let recorded_bytes = RecordedGivenStatesTableWithItsColumns::of((&table).into()).to_bytes();
        let digest = sha256_digest(&recorded_bytes);
        let stored = Self::library()
            .lock()
            .read_given_states_table(&digest)
            .unwrap_or_else(|error| panic!("给定状态表 {table_name} 读不了库：{error:?}"));
        if stored.as_deref() != Some(recorded_bytes.as_slice()) {
            Self::library()
                .lock()
                .store_given_states_table(&digest, &recorded_bytes)
                .unwrap_or_else(|error| panic!("给定状态表 {table_name} 写不进库：{error:?}"));
        }
        RecordedTableReference {
            table_name: table_name.to_string(),
            digest,
        }
    }

    fn judge_a_recorded_table(
        &self,
        table: &RecordedTableReference,
    ) -> Result<Vec<String>, String> {
        use singlefs_verification::crash_given_states::{
            GivenStatesConclusionsSource, GivenStatesJudge,
        };
        let table_name = &table.table_name;
        let recorded_bytes = Self::library()
            .lock()
            .read_given_states_table(&table.digest)
            .unwrap_or_else(|error| panic!("给定状态表 {table_name} 读不了库：{error:?}"))
            .ok_or_else(|| {
                format!(
                    "交接里的给定状态表 {table_name} 不在库里：先跑录入段（{STAGE_ARGUMENT} record，或不给 {STAGE_ARGUMENT}、两段先后跑）"
                )
            })?;
        if sha256_digest(&recorded_bytes) != table.digest {
            return Err(format!(
                "交接里的给定状态表 {table_name} 从库里读回的字节摘要不是交接里录的那一个：重跑录入段（{STAGE_ARGUMENT} record）"
            ));
        }
        let recorded = RecordedGivenStatesTableWithItsColumns::from_bytes(&recorded_bytes)
            .ok_or_else(|| {
                format!(
                    "交接里的给定状态表 {table_name} 从库里读回的字节解不出：重跑录入段（{STAGE_ARGUMENT} record）"
                )
            })?;
        let judged = recorded
            .with_the_table(|table_read_back| {
                given_states_engine_of_this_experiment()
                    .judge_the_given_states_with_its_columns(table_read_back)
            })
            .unwrap_or_else(|reason| {
                panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
            });
        match judged.source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => panic!(
                "装置的判段不分片、引擎不分段：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name}）"
            ),
        }
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{table_name}：一个状态一份结论，交回 {} 份",
                judged.states.len()
            );
        };
        Ok(conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant.to_string())
            .collect())
    }
}

/// 这份装置判段判盘的给定状态引擎（进程里一份，判段第一次判时起；录入段不起它）。产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment(
) -> &'static singlefs_verification::crash_amplification::GivenStatesEngine {
    static ENGINE: std::sync::OnceLock<
        singlefs_verification::crash_amplification::GivenStatesEngine,
    > = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        singlefs_verification::crash_amplification::GivenStatesEngine::start_from_environment(
            singlefs_verification::crash_amplification::GivenStatesItem {
                item_name: "e171_three_oracle_collusion::fixed".to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 只有 `Violated` 算判红的那几条的名字。O2 的名单走两段（[`PoolCheckerTablesOfTheTwoStages`]），
/// 这个函数只给单测用：登记第九节 X6 那条（`only_a_violated_invariant_counts_as_a_red_pool_checker_verdict`）与不上卡的判法（`pool_checker_red_names_of_a_given_image_on_the_cpu`）。
#[cfg(test)]
pub(super) fn violated_invariant_names(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(_) => true,
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => false,
        })
        .map(|(invariant, _)| (*invariant).to_string())
        .collect()
}

/// 一个取样状态上看到的：批号、崩溃状态序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
/// 三条 oracle 的结论与各自耗时、触达的最后一次调用。
pub(super) struct StateObservation {
    pub(super) batch: u8,
    pub(super) ordinal: u64,
    pub(super) segment_index: usize,
    pub(super) landing_vector: String,
    pub(super) image_digest: String,
    pub(super) verdict: ImplementationVerdict,
    pub(super) judge_nanoseconds: u128,
    pub(super) mask_matches_the_facts: bool,
    /// 这一态里持久或撕裂的写属于的最后一次调用（一个写都没落的态是 None）。
    pub(super) touched_call: Option<usize>,
    pub(super) crash_judge: CrashJudgeVerdict,
    /// O2：关掉、录入段录下了这一态要判的盘、判段判完。
    pub(super) pool_checker: PoolCheckerOfAState,
    pub(super) consulted: RecoverySummary,
    pub(super) ignored: RecoverySummary,
    pub(super) crash_judge_nanoseconds: u128,
    pub(super) pool_checker_nanoseconds: u128,
    pub(super) recovery_nanoseconds: u128,
}

/// 一个取样态上 O2 走到哪一步。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PoolCheckerOfAState {
    /// 关一条（Q3-核）关掉了 O2：这一列写 `off`，S3 不比。
    Off,
    /// 录入段把这一态要判的盘录下了，判段还没判。
    RecordedForTheJudgeStage(RecordedTableReference),
    /// 判段判完：判红的不变量名，与 S3（判出的名单与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同；O2 阳性对照那一趟不比，记相同）。
    Judged {
        red: Vec<String>,
        matches_the_judgement: bool,
    },
}

impl StateObservation {
    /// O2 判红的不变量（关掉 O2 时 None）。
    ///
    /// # Panics
    /// 这一态录下的盘判段还没判（`oracle-verdicts.tsv` 与结果行都在判段判完之后才写）。
    pub(super) fn pool_checker_red(&self) -> Option<&[String]> {
        match &self.pool_checker {
            PoolCheckerOfAState::Off => None,
            PoolCheckerOfAState::Judged { red, .. } => Some(red.as_slice()),
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => panic!(
                "{} 判段还没判：O2 的名单在判段判完之后才读",
                table.table_name
            ),
        }
    }

    /// S3：O2 判出的名单与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同（关掉 O2 时不比，记相同）。
    ///
    /// # Panics
    /// 同 [`StateObservation::pool_checker_red`]。
    pub(super) fn pool_checker_matches_the_judgement(&self) -> bool {
        match &self.pool_checker {
            PoolCheckerOfAState::Off => true,
            PoolCheckerOfAState::Judged {
                matches_the_judgement,
                ..
            } => *matches_the_judgement,
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => {
                panic!("{} 判段还没判：S3 在判段判完之后才比", table.table_name)
            }
        }
    }
}

/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次。「第一个」要一条线程按序号走才说得清，
/// 这个阳性对照那一趟只开一条线程（[`sampling_threads_given_the_positive_control`]）。
pub(super) struct ObservationControls {
    pub(super) switches: RunSwitches,
    pub(super) crash_judge_control_pending: AtomicBool,
}

/// 把写表每一项落在哪一次调用里算出来：写在录制流里的下标落在 `first_operation` 不超过它的最后一次调用里。
pub(super) fn call_index_of_each_write(stream: &PreparedStream) -> Vec<Option<usize>> {
    stream
        .stream_indexes
        .iter()
        .map(|operation_index| {
            stream
                .calls
                .iter()
                .rposition(|call| call.first_operation <= *operation_index)
        })
        .collect()
}

/// O1-崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 的 `inject_crashes_into_history_with_every_switch` 里问理想模型那一段——「盘上已持久的最新根槽」由 `newest_persisted_root`
/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
pub(super) fn crash_judge_of_the_state(
    stream: &PreparedStream,
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    image: &CrashImage<'_>,
    consulted: &RecoveryReport,
    batch: u8,
    controls: &ObservationControls,
) -> CrashJudgeVerdict {
    let newest_persisted_root_key =
        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let mut read_back = observed_read_back_after_a_crash(consulted);
    if controls.crash_judge_control_pending.load(Ordering::SeqCst) && batch == 1 {
        if let ObservedReadBack::FileRead { content, .. } = &mut read_back {
            if let Some(last_byte) = content.last_mut() {
                *last_byte ^= 1;
                controls
                    .crash_judge_control_pending
                    .store(false, Ordering::SeqCst);
            }
        }
    }
    crash_judge_verdict(committed_versions, newest_persisted_root_key, &read_back)
}

/// 这条线程自己的一份模型版本目录：`crash_recovery_disagreement` 收 `Rc`，`Rc` 不跨线程，照流里存的内容造。
pub(super) fn committed_versions_for_this_thread(
    stream: &PreparedStream,
) -> BTreeMap<ModelRootKey, Option<Rc<[u8]>>> {
    stream
        .committed_version_contents
        .iter()
        .map(|(key, content)| (*key, content.as_deref().map(Rc::<[u8]>::from)))
        .collect()
}

/// 模型的 `crash_recovery_disagreement` 判一态读回的版本：有不同记红（带实现那一侧说的话），没有不同记绿。
fn crash_judge_verdict(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root_key: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> CrashJudgeVerdict {
    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {
        Some(disagreement) => CrashJudgeVerdict::Red {
            what: disagreement
                .implementation_answer
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("_"),
        },
        None => CrashJudgeVerdict::Green,
    }
}

/// 一态上 O1-崩溃判官适用不适用（读法 6）：这一态里有写落在模型停下的那次调用（含）之后的调用里，模型的版本目录里没有那些调用写出的版本，不适用。
pub(super) fn crash_judge_is_applicable(
    touched_call: Option<usize>,
    model_stopped_at_call: Option<usize>,
) -> bool {
    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))
}

#[cfg(test)]
mod tests;
