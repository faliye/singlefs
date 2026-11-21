//! 实现的判定（登记 5.3 `impl-verdicts.tsv`）。

use super::*;

// ───────────────────────────── 实现的判定（登记 5.3 `impl-verdicts.tsv`） ─────────────────────────────

fn root_text(root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
    match root {
        Some((instance, checkpoint_txg)) => format!("{},{}", instance.0, checkpoint_txg.0),
        None => "none".to_string(),
    }
}

/// outcome 的种类；走不下去的那一种带上失败的名字（空白换成下划线，不破坏 tsv 与结果行）。
fn outcome_text(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{failure:?}")
            .chars()
            .map(|character| {
                if character.is_whitespace() {
                    '_'
                } else {
                    character
                }
            })
            .collect(),
    }
}

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

    pub(super) fn pool_checker_and_record_checker_both_pass(&self) -> bool {
        self.pool_checker_red.is_empty()
            && !self.root_without_record
            && !self.claimed_state_missing_unit
    }
}

pub(super) fn list_text(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

/// 一个取样状态上看到的：批号、崩溃状态序号、段号、落地向量、两份镜像摘要之一（直接从 `CrashImage` 算的）、实现判定与耗时、两条自检。
pub(super) struct StateObservation {
    pub(super) batch: u8,
    pub(super) ordinal: u64,
    pub(super) segment_index: usize,
    pub(super) landing_vector: String,
    pub(super) image_digest: String,
    pub(super) verdict: ImplementationVerdict,
    pub(super) judge_nanoseconds: u128,
    pub(super) mask_matches_the_facts: bool,
    pub(super) rejudge_matches: bool,
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_crash_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
/// 再在同一个镜像上单独计时调一次 `judge_crash_image`（单线程耗时；两次判定逐项比，记 `rejudge_matches`）。
/// 回调的持久集合另与 `crash_facts` 按序号现算的掩码比（记 `mask_matches_the_facts`）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn observe_sampled_states(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    sampled: &SampledOrdinals,
) -> Vec<StateObservation> {
    let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
    for (batch_offset, batch) in sampled.batches.iter().enumerate() {
        for ordinal in batch {
            let batch_number = u8::try_from(batch_offset + 1).expect("三批");
            assert!(
                batch_of_ordinal.insert(*ordinal, batch_number).is_none(),
                "三批互不重叠"
            );
        }
    }
    let ordinals: Vec<u64> = batch_of_ordinal.keys().copied().collect();
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in ordinals {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let recorded_write_count = stream.writes.len();
    let mut observations: Vec<StateObservation> = Vec::new();
    for run in runs {
        let state_count_of_the_stream = sample_crash_states_on_the_cpu(
            CrashPointReplayItem {
                name: &sampling_item_name_of_this_call(),
                setting_file: CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
            },
            &stream.base,
            &stream.writes,
            &stream.segments,
            stream.judged_root_index,
            &stream.versions,
            &full_expansion,
            run,
            &mut |ordinal, image, judgement| {
                let started = Instant::now();
                let timed = judge_crash_image(image, stream.judged_root_index, &stream.versions);
                let judge_nanoseconds = started.elapsed().as_nanos();
                let verdict = ImplementationVerdict::of(judgement);
                observations.push(StateObservation {
                    batch: batch_of_ordinal[&ordinal],
                    ordinal,
                    segment_index: tables
                        .state_ranges_by_segment
                        .partition_point(|state_range| state_range.end <= ordinal),
                    landing_vector: landing_vector_of(
                        &image.persisted,
                        recorded_write_count,
                        torn_image_index_by_write,
                    ),
                    image_digest: sha256_text(&crash_image_digest(image)),
                    rejudge_matches: ImplementationVerdict::of(&timed) == verdict,
                    verdict,
                    judge_nanoseconds,
                    mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
                        == image.persisted,
                });
            },
        )
        .judged()
        .expect("实验装置不分只录、只判两趟跑：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE");
        assert_eq!(
            state_count_of_the_stream, tables.state_count,
            "judge_crash_state_range 交回的整条流状态数与枚举表的相同"
        );
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    observations
}

pub(super) fn sha256_text(digest: &[u8; DIGEST_BYTES]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
