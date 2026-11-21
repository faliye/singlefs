//! S1 账与每一步的判定：发号事件、重发号与冲突的分类，每段历史的计数。
use super::*;

/// S2 一条不变量在一步上的判定落在哪一栏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerdictClass {
    Holds,
    NotApplicable,
    PartiallyJudged,
    Violated,
    /// checker 交回的列表里没有这一条（读不到 ≠ 读到 0，另数）。
    Missing,
}

impl VerdictClass {
    fn of(verdict: &InvariantVerdict) -> VerdictClass {
        match verdict {
            InvariantVerdict::Holds => VerdictClass::Holds,
            InvariantVerdict::NotApplicable(_) => VerdictClass::NotApplicable,
            InvariantVerdict::PartiallyJudged(_) => VerdictClass::PartiallyJudged,
            InvariantVerdict::Violated(_) => VerdictClass::Violated,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            VerdictClass::Holds => "holds",
            VerdictClass::NotApplicable => "not_applicable",
            VerdictClass::PartiallyJudged => "partially_judged",
            VerdictClass::Violated => "violated",
            VerdictClass::Missing => "missing",
        }
    }
}

impl VerdictColumns {
    pub(super) fn count(&mut self, class: VerdictClass) {
        match class {
            VerdictClass::Holds => self.holds += 1,
            VerdictClass::NotApplicable => self.not_applicable += 1,
            VerdictClass::PartiallyJudged => self.partially_judged += 1,
            VerdictClass::Violated => self.violated += 1,
            VerdictClass::Missing => self.missing += 1,
        }
    }

    pub(super) fn render(&self, prefix: &str) -> String {
        format!(
            "{prefix}_holds={} {prefix}_not_applicable={} {prefix}_partially_judged={} {prefix}_violated_steps={} {prefix}_missing={}",
            self.holds, self.not_applicable, self.partially_judged, self.violated, self.missing
        )
    }
}

/// 一步的样子，留给下一步比（回退、建 inode、挂载那几个量都拿「那一步之前」比）。
#[derive(Clone, Debug)]
pub(super) struct StepView {
    readable: Vec<RootFacts>,
    abandoned: BTreeMap<RootIdentity, bool>,
    current: Option<RootFacts>,
}

/// 一步之后现行那一版的位置、水位与 inode 树。
pub(super) type CurrentVersionAfterAStep = (String, Option<u64>, Vec<(u64, u64)>);

/// 这一步读到的盘面（S1）与 S3 两路的判定。
struct ReadStep {
    readable: Vec<RootFacts>,
    table: Option<InstanceTableRecords>,
}

impl HistoryAccount {
    pub(super) fn new(breakage: Breakage) -> HistoryAccount {
        HistoryAccount {
            breakage,
            root_cache: HashMap::new(),
            births: BTreeMap::new(),
            watermarks_seen: BTreeMap::new(),
            rows_by_steps: BTreeMap::new(),
            previous: None,
            previous_model_newest: None,
            open_windows_after_a_rollback_across_issued_numbers: 0,
            counts: HistoryCounts::default(),
            events: Vec::new(),
            first_violation: None,
            first_abandonment_path_disagreement: None,
            step_verdicts: Vec::new(),
            creation_issues: Vec::new(),
            first_creation_not_issuing_the_previous_watermark: None,
            current_trajectory: Vec::new(),
            rollback_targets: Vec::new(),
        }
    }

    /// S1：列可读根、按根键缓存重建、按 (txg, 实例) 升序排、同一身份只留一条；再读最新那条根的实例表（S3 路 1 用）。
    fn read_step(&mut self, image: &MemoryPool) -> Option<ReadStep> {
        let Ok(system_configuration) = choose_system_configuration(image) else {
            self.counts.system_configuration_unchosen_steps += 1;
            return None;
        };
        let roots = readable_roots(
            image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        );
        let mut journal_records: Option<BTreeMap<(InstanceGeneration, u64), JournalRecord>> = None;
        let mut by_identity: BTreeMap<(u64, u32), (RootFacts, RootRecord)> = BTreeMap::new();
        for root in &roots {
            let key = format!("{root:?}");
            let facts = if let Some(cached) = self.root_cache.get(&key) {
                cached.clone()
            } else {
                let records = journal_records
                    .get_or_insert_with(|| scan_journal(image, &system_configuration));
                let facts = RootFacts {
                    identity: root_identity(root),
                    rollback_floor: root.rollback_floor.0,
                    version: version_facts_of(image, root, records),
                };
                self.counts.version_rebuilds += 1;
                self.root_cache.insert(key, facts.clone());
                facts
            };
            let order = (facts.identity.1, facts.identity.0);
            match by_identity.get(&order) {
                Some((kept, _)) if *kept != facts => {
                    self.counts.same_identity_with_different_facts += 1
                }
                Some(_) => {}
                None => {
                    by_identity.insert(order, (facts, *root));
                }
            }
        }
        let table = by_identity
            .values()
            .last()
            .and_then(|(_, newest)| instance_table_of_root(image, newest));
        if table.is_none() && !by_identity.is_empty() {
            self.counts.instance_table_unreadable_steps += 1;
        }
        let readable: Vec<RootFacts> = by_identity.into_values().map(|(facts, _)| facts).collect();
        self.counts.root_observations += u64::try_from(readable.len()).expect("根环至多几十条");
        self.counts.unreadable_root_observations += u64::try_from(
            readable
                .iter()
                .filter(|facts| matches!(facts.version, VersionFacts::Unreadable(_)))
                .count(),
        )
        .expect("根环至多几十条");
        Some(ReadStep { readable, table })
    }

    /// S3 路 1（现行实例表）。读不出实例表时按不被抛弃算（步数另记）。
    fn abandoned_by_path_one(
        &self,
        identity: RootIdentity,
        table: Option<&InstanceTableRecords>,
    ) -> bool {
        table.is_some_and(|table| abandoned_by_the_table(identity, table))
    }

    fn abandoned_by_path_two(&self, identity: RootIdentity) -> bool {
        abandoned_by_the_step_ledger(identity, &self.rows_by_steps)
    }

    /// 给发号事件分类（Q1 / Q3b、Q9）时持有旧出生代的根按 S3 判；弄坏开关 every-root-abandoned 打开时每条都判被抛弃。
    fn holder_abandoned(
        &self,
        identity: RootIdentity,
        table: Option<&InstanceTableRecords>,
    ) -> bool {
        self.breakage.every_root_abandoned || self.abandoned_by_path_one(identity, table)
    }
}

impl HistoryAccount {
    /// 执行器每一步之后（起点之后也一次）调它。
    pub(super) fn observe(&mut self, observation: &StepObservation<'_>) {
        let position = match observation.position {
            StepPosition::StartingPoint => "start".to_string(),
            StepPosition::Operation(step_index) => format!("op{step_index}"),
        };
        self.counts.steps_observed += 1;
        let applied_effect = match observation.outcome {
            Some(StepOutcome::Applied(effect)) => Some(effect),
            Some(StepOutcome::Refused { .. } | StepOutcome::NotApplicable(_)) | None => None,
        };
        // 路 2 的步账先记：这一步做成的挂载接的是那一步之前模型根环里最新的那条根。
        if let (Some(AppliedEffect::Mounted { .. }), Some(chosen)) =
            (applied_effect, self.previous_model_newest)
        {
            self.rows_by_steps.insert(chosen.0, chosen.1);
        }
        let model_newest = observation
            .model
            .committed_versions()
            .iter()
            .map(|(key, _)| key)
            .max()
            .map(|key| {
                (
                    u32::try_from(key.instance.0).expect("模型的实例代号来自盘上 4 字节"),
                    key.checkpoint_txg.0,
                )
            });
        let Some(read) = self.read_step(observation.image) else {
            self.previous_model_newest = model_newest;
            return;
        };
        let table = read.table.as_ref();
        let mut abandoned: BTreeMap<RootIdentity, bool> = BTreeMap::new();
        for facts in &read.readable {
            let by_path_one = self.abandoned_by_path_one(facts.identity, table);
            if table.is_some() && by_path_one != self.abandoned_by_path_two(facts.identity) {
                self.counts.abandonment_path_disagreements += 1;
                if self.first_abandonment_path_disagreement.is_none() {
                    self.first_abandonment_path_disagreement = Some(format!(
                        "{position}:root{:?}:path1={by_path_one}:rows={:?}",
                        facts.identity, self.rows_by_steps
                    ));
                }
            }
            abandoned.insert(facts.identity, by_path_one);
            self.watermarks_seen
                .insert(facts.identity, facts.watermark());
        }
        let current = read
            .readable
            .iter()
            .rev()
            .find(|facts| !abandoned[&facts.identity])
            .cloned();
        let current_rollback_floor = current.as_ref().map_or(0, |facts| facts.rollback_floor);
        self.current_trajectory.push((
            position.clone(),
            current.as_ref().and_then(RootFacts::watermark),
            current
                .as_ref()
                .map(|facts| facts.inode_births().to_vec())
                .unwrap_or_default(),
        ));
        let readable_identities: BTreeSet<RootIdentity> =
            read.readable.iter().map(|facts| facts.identity).collect();
        let mut events_of_this_step: Vec<ReissueEvent> = Vec::new();
        for facts in &read.readable {
            for &(inode, birth) in facts.inode_births() {
                let birth_key = if self.breakage.ledger_ignores_birth {
                    0
                } else {
                    birth
                };
                let older: Vec<(u64, BTreeSet<RootIdentity>)> = self
                    .births
                    .get(&inode)
                    .map(|births| {
                        births
                            .iter()
                            .filter(|(older_birth, _)| **older_birth != birth_key)
                            .map(|(older_birth, holders)| (*older_birth, holders.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                let is_new = !self
                    .births
                    .get(&inode)
                    .is_some_and(|births| births.contains_key(&birth_key));
                if is_new {
                    self.counts.issuances += 1;
                    if matches!(observation.position, StepPosition::Operation(_)) {
                        self.counts.issuances_at_operations += 1;
                    }
                    if !older.is_empty() {
                        let holders: Vec<RootIdentity> = older
                            .iter()
                            .flat_map(|(_, holders)| holders.iter().copied())
                            .collect();
                        let same_timeline = holders
                            .iter()
                            .any(|holder| !self.holder_abandoned(*holder, table));
                        let views: Vec<HolderAtThisStep> = holders
                            .iter()
                            .map(|holder| HolderAtThisStep {
                                readable_now: readable_identities.contains(holder),
                                abandoned_now: self.holder_abandoned(*holder, table),
                                txg: holder.1,
                            })
                            .collect();
                        events_of_this_step.push(ReissueEvent {
                            position: position.clone(),
                            inode,
                            new_birth: birth,
                            older_births: older
                                .iter()
                                .map(|(older_birth, _)| *older_birth)
                                .collect(),
                            same_timeline,
                            older_birth_candidacy: classify_older_birth_candidacy(
                                &views,
                                current_rollback_floor,
                                self.breakage,
                            ),
                            one_birth_per_number_at_this_step: VerdictClass::Missing,
                        });
                    }
                }
                self.births
                    .entry(inode)
                    .or_default()
                    .entry(birth_key)
                    .or_default()
                    .insert(facts.identity);
            }
        }
        self.judge_with_the_checker(observation.image, &position, &mut events_of_this_step);
        self.count_watermark_regression(current.as_ref(), table);
        self.count_operation_specific(observation, applied_effect, &position, current.as_ref());
        self.events.extend(events_of_this_step);
        self.previous = Some(StepView {
            readable: read.readable,
            abandoned,
            current,
        });
        self.previous_model_newest = model_newest;
    }
}

impl HistoryAccount {
    /// S2：这一步的镜像交给池级 checker；数 I-9.29、I-9.6 的四栏，记第一个 Violated，核第十节 F3 两形。
    fn judge_with_the_checker(
        &mut self,
        image: &MemoryPool,
        position: &str,
        events_of_this_step: &mut [ReissueEvent],
    ) {
        let mut one_birth_per_number = VerdictClass::Missing;
        let mut watermark_above_both_highest_numbers = VerdictClass::Missing;
        for (invariant, verdict) in check_pool_image(image) {
            if let InvariantVerdict::Violated(detail) = &verdict {
                if self.first_violation.is_none() {
                    let first_line = detail.lines().next().unwrap_or("").replace(' ', "_");
                    self.first_violation = Some(format!("{position}:{invariant}:{first_line}"));
                }
            }
            if invariant == ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME {
                one_birth_per_number = VerdictClass::of(&verdict);
            }
            if invariant == WATERMARK_ABOVE_BOTH_HIGHEST_NUMBERS_NAME {
                watermark_above_both_highest_numbers = VerdictClass::of(&verdict);
            }
        }
        self.counts.one_birth_per_number.count(one_birth_per_number);
        self.counts
            .watermark_above_both_highest_numbers
            .count(watermark_above_both_highest_numbers);
        for event in events_of_this_step.iter_mut() {
            event.one_birth_per_number_at_this_step = one_birth_per_number;
            if event.same_timeline
                && event.older_birth_candidacy == OlderBirthStillInTheCandidateSet::Still
            {
                match one_birth_per_number {
                    VerdictClass::Holds => self.counts.still_candidate_but_one_birth_holds += 1,
                    VerdictClass::PartiallyJudged => {
                        self.counts.still_candidate_but_one_birth_partially_judged += 1;
                    }
                    VerdictClass::NotApplicable
                    | VerdictClass::Violated
                    | VerdictClass::Missing => {}
                }
            }
        }
        if one_birth_per_number == VerdictClass::Violated
            && !self.births.values().any(|births| births.len() >= 2)
        {
            self.counts
                .one_birth_violated_without_two_births_in_the_ledger += 1;
        }
        self.step_verdicts.push(StepVerdicts {
            position: position.to_string(),
            one_birth_per_number,
            watermark_above_both_highest_numbers,
        });
    }

    /// Q3：现行那一版的水位 W 与到这一步为止不被抛弃的根的水位最大值 M 比，深度 = M − W（W ≥ M 记 0）。
    fn count_watermark_regression(
        &mut self,
        current: Option<&RootFacts>,
        table: Option<&InstanceTableRecords>,
    ) {
        let Some(watermark) = current.and_then(RootFacts::watermark) else {
            self.counts.steps_whose_current_version_has_no_watermark += 1;
            return;
        };
        let highest_seen = self
            .watermarks_seen
            .iter()
            .filter(|(identity, _)| !self.abandoned_by_path_one(**identity, table))
            .filter_map(|(_, seen)| *seen)
            .max()
            .unwrap_or(watermark);
        let depth = highest_seen.saturating_sub(watermark);
        if depth > 0 {
            self.counts.watermark_regression_steps += 1;
        }
        self.counts.watermark_regression_peak = self.counts.watermark_regression_peak.max(depth);
        self.counts.watermark_regression_final = depth;
        self.counts.watermark_peak = self.counts.watermark_peak.max(watermark);
        self.counts.watermark_final = watermark;
    }

    /// 拿「那一步之前」比的几个量：Q4、Q5、Q6（回退）、Q11（建 inode）、Q7（挂载 / 恢复）。
    fn count_operation_specific(
        &mut self,
        observation: &StepObservation<'_>,
        applied_effect: Option<&AppliedEffect>,
        position: &str,
        current: Option<&RootFacts>,
    ) {
        let Some(operation) = observation.operation else {
            return;
        };
        let previous_current_watermark = self
            .previous
            .as_ref()
            .and_then(|previous| previous.current.as_ref())
            .and_then(RootFacts::watermark);
        match applied_effect {
            Some(AppliedEffect::RolledBack { target, .. }) => {
                self.counts.rollbacks_applied += 1;
                let target_identity = (target.instance.0, target.checkpoint_txg.0);
                let previous_readable = self
                    .previous
                    .as_ref()
                    .map_or(&[][..], |previous| &previous.readable[..]);
                let target_watermark = match previous_readable
                    .iter()
                    .find(|facts| facts.identity == target_identity)
                {
                    Some(facts) => facts.watermark(),
                    None => {
                        self.counts.rollback_targets_not_in_the_previous_ring += 1;
                        None
                    }
                };
                self.rollback_targets
                    .push((target_identity, target_watermark));
                if let (Some(target_watermark), Some(before)) =
                    (target_watermark, previous_current_watermark)
                {
                    if target_watermark < before {
                        self.counts.rollbacks_across_issued_numbers += 1;
                        self.open_windows_after_a_rollback_across_issued_numbers += 1;
                    }
                }
                if let Some(before) = previous_current_watermark {
                    self.counts.rollbacks_judged_against_the_ring_maximum += 1;
                    let ring_highest = previous_readable
                        .iter()
                        .filter_map(RootFacts::watermark)
                        .max();
                    if ring_highest.is_some_and(|ring_highest| ring_highest < before) {
                        self.counts
                            .rollbacks_whose_ring_maximum_is_below_the_current += 1;
                    }
                }
            }
            Some(AppliedEffect::Mounted { .. } | AppliedEffect::Recovered { .. }) => {
                self.counts.mounts_or_recoveries_applied += 1;
                let previous_highest = self.previous.as_ref().and_then(|previous| {
                    previous
                        .readable
                        .iter()
                        .filter(|facts| {
                            !previous
                                .abandoned
                                .get(&facts.identity)
                                .copied()
                                .unwrap_or(false)
                        })
                        .filter_map(RootFacts::watermark)
                        .max()
                });
                if let (Some(after), Some(previous_highest)) =
                    (current.and_then(RootFacts::watermark), previous_highest)
                {
                    if after < previous_highest {
                        self.counts.mounts_landing_on_a_lower_watermark += 1;
                    }
                }
            }
            // 只读快照的建与读不发 inode 号、不动水位账；整理搬单元、不发号。E175 不生成这些步，列在这里只为穷尽。
            // 判不了、交主 agent：CreateInodes 一次发多个号，其结局若落在 Published 上就已在本臂；若某天 E175 生成它，Q11 的 creations_applied 只数种类 CreateInode，要不要数 CreateInodes 要登记定。
            Some(
                AppliedEffect::Published { .. }
                | AppliedEffect::RaisedFloor { .. }
                | AppliedEffect::SnapshotPublished { .. }
                | AppliedEffect::SnapshotRead { .. }
                | AppliedEffect::Compacted { .. },
            )
            | None => {}
        }
        if operation.kind() == HistoryOperationKind::CreateInode && applied_effect.is_some() {
            self.counts.creations_applied += 1;
            let before: BTreeSet<(u64, u64)> = self
                .previous
                .as_ref()
                .and_then(|previous| previous.current.as_ref())
                .map(|facts| facts.inode_births().iter().copied().collect())
                .unwrap_or_default();
            let issued: Vec<(u64, u64)> = current
                .map(|facts| {
                    facts
                        .inode_births()
                        .iter()
                        .copied()
                        .filter(|pair| !before.contains(pair))
                        .collect()
                })
                .unwrap_or_default();
            let issued_numbers: Vec<u64> = issued.iter().map(|(inode, _)| *inode).collect();
            if issued_numbers.len() != 1 || Some(issued_numbers[0]) != previous_current_watermark {
                self.counts.creations_not_issuing_the_previous_watermark += 1;
                if self
                    .first_creation_not_issuing_the_previous_watermark
                    .is_none()
                {
                    self.first_creation_not_issuing_the_previous_watermark = Some(format!(
                        "{position}:issued={issued:?}:previous_watermark={previous_current_watermark:?}"
                    ).replace(' ', ""));
                }
            }
            self.creation_issues.push(CreationIssue {
                position: position.to_string(),
                issued_numbers,
                previous_watermark: previous_current_watermark,
            });
            self.counts.windows_with_a_creation_after_such_a_rollback +=
                self.open_windows_after_a_rollback_across_issued_numbers;
            self.open_windows_after_a_rollback_across_issued_numbers = 0;
        }
        if operation_ends_the_session(operation.kind()) {
            self.open_windows_after_a_rollback_across_issued_numbers = 0;
        }
    }
}
