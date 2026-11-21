//! 择根合判一族的理想恢复与真相、数的量、被判的一方、一个状态的判定（KJ0 至 KJ3），与一个 (域, M, U) 格的枚举和按工作单元的计数（含落点对抗根环）。

use super::*;

// ── 理想恢复与真相（登记 5.5 KJ3；判据一侧不调 core）

/// 理想恢复（D13 已定项 1 M4「恰是 j*」，所选根换成给定的那条根）：从它起沿它的实例逐次往后，每次发布的每条记录至少一份在盘上、
/// 条数累计不超在飞上限、第一次的 txg 是它的 txg + 1（D23 已定项 14 注 1 链首），走得最远的那一版。
pub(super) struct RootChoiceIdealRecovery {
    pub(super) landed_version: RootVersion,
    pub(super) landed_fields: VersionFields,
    applied_publishes: usize,
}

pub(super) fn root_choice_ideal_recovery_from(
    start: &RootChoicePublishMaterial,
    records: &[RootChoiceLandedRecordRef<'_>],
    in_flight_limit: usize,
) -> RootChoiceIdealRecovery {
    let mut recovery = RootChoiceIdealRecovery {
        landed_version: start.version,
        landed_fields: start.fields,
        applied_publishes: 0,
    };
    let mut records_applied = 0usize;
    let mut next_checkpoint_txg = start.version.checkpoint_txg;
    while let Some(following_txg) = next_checkpoint_txg.checked_add(1) {
        let mut publish: Option<&RootChoicePublishMaterial> = None;
        let mut landed_records_of_the_publish = 0usize;
        for record in records {
            if record.material.version.instance == start.version.instance
                && record.material.version.checkpoint_txg == following_txg
            {
                publish = Some(record.material);
                landed_records_of_the_publish += 1;
            }
        }
        let Some(publish) = publish else {
            break;
        };
        if landed_records_of_the_publish != publish.records.len()
            || records_applied + landed_records_of_the_publish > in_flight_limit
        {
            break;
        }
        records_applied += landed_records_of_the_publish;
        recovery.landed_version = publish.version;
        recovery.landed_fields = publish.fields;
        recovery.applied_publishes += 1;
        next_checkpoint_txg = following_txg;
    }
    recovery
}

/// 真相里根环每个槽现存的根（读根环次序），自证过的才有。
fn root_choice_versions_in_ring_order(landing: &RootChoiceLanding<'_>) -> Vec<Option<RootVersion>> {
    landing
        .roots_in_ring_order
        .iter()
        .map(|root| root.map(|material| material.version))
        .collect()
}

/// 版本最大的那一条根（真相里；同版本的几条取读根环次序里最前的，版本相同的根字节相同、哪条都一样）。
pub(super) fn root_choice_newest_root<'world>(
    landing: &RootChoiceLanding<'world>,
) -> Option<&'world RootChoicePublishMaterial> {
    let mut newest: Option<&RootChoicePublishMaterial> = None;
    for root in landing.roots_in_ring_order.iter().copied().flatten() {
        if newest.is_none_or(|current| root.version > current.version) {
            newest = Some(root);
        }
    }
    newest
}

fn root_choice_oldest_root<'world>(
    landing: &RootChoiceLanding<'world>,
) -> Option<&'world RootChoicePublishMaterial> {
    let mut oldest: Option<&RootChoicePublishMaterial> = None;
    for root in landing.roots_in_ring_order.iter().copied().flatten() {
        if oldest.is_none_or(|current| root.version < current.version) {
            oldest = Some(root);
        }
    }
    oldest
}

#[derive(Clone, Debug)]
pub(super) struct RootChoiceTally {
    pub(super) counts: Vec<u64>,
    pub(super) witnesses: Vec<Option<String>>,
    /// 真相里版本最大的根之上、同实例的记录条数的最大值（登记第八节）。
    pub(super) above_newest_root_maximum: usize,
    /// 实现所选的根之上、同实例的记录条数的最大值（登记第八节：变体择到更旧的根时在飞上限可能咬到）。
    pub(super) above_chosen_root_maximum: usize,
    /// 已确认的用户发布条数的最大值（登记第八节）。
    pub(super) confirmed_publishes_maximum: usize,
    /// 盘上读得出的最大计数器。
    pub(super) largest_counter: u64,
}

impl RootChoiceTally {
    pub(super) fn new() -> Self {
        Self {
            counts: vec![0; RootChoiceCounter::ALL.len()],
            witnesses: vec![None; RootChoiceCounter::ALL.len()],
            above_newest_root_maximum: 0,
            above_chosen_root_maximum: 0,
            confirmed_publishes_maximum: 0,
            largest_counter: 0,
        }
    }

    pub(super) fn add(&mut self, counter: RootChoiceCounter) {
        self.counts[counter.index()] += 1;
    }

    /// 违例加一；这个量的第一条违例记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    pub(super) fn add_violation(
        &mut self,
        counter: RootChoiceCounter,
        describe_witness: &dyn Fn() -> String,
    ) {
        let index = counter.index();
        self.counts[index] += 1;
        if self.witnesses[index].is_none() {
            self.witnesses[index] = Some(describe_witness());
        }
    }

    pub(super) fn count_of(&self, counter: RootChoiceCounter) -> u64 {
        self.counts[counter.index()]
    }

    pub(super) fn witness(&self, counter: RootChoiceCounter) -> &str {
        self.witnesses[counter.index()].as_deref().unwrap_or("none")
    }

    pub(super) fn merge(&mut self, later: &Self) {
        for (count, later_count) in self.counts.iter_mut().zip(&later.counts) {
            *count += later_count;
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            if witness.is_none() {
                witness.clone_from(later_witness);
            }
        }
        self.above_newest_root_maximum = self
            .above_newest_root_maximum
            .max(later.above_newest_root_maximum);
        self.above_chosen_root_maximum = self
            .above_chosen_root_maximum
            .max(later.above_chosen_root_maximum);
        self.confirmed_publishes_maximum = self
            .confirmed_publishes_maximum
            .max(later.confirmed_publishes_maximum);
        self.largest_counter = self.largest_counter.max(later.largest_counter);
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`tally_of_work_units`]，计数的种类不同）。
fn root_choice_tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> RootChoiceTally + Sync),
) -> RootChoiceTally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, RootChoiceTally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = RootChoiceTally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

// ── 被判的一方：今天的三步，或阳性对照的假结局（登记 5.7）

/// KL 上本地写的几种择根序（登记 5.7 第四行、9.1 四行替换文的比较式各写一份；落点 = 择到的根，journal 环空）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LocalRootChoiceRule {
    /// 今天的序：txg 大者赢，txg 相等实例代号大者赢，键完全相同取读根环次序里前面的。
    NewestWins,
    /// 变异 1825：txg 平局时实例代号低者赢。
    TieTakesTheLowerInstance,
    /// 变异 1826：实例代号为主、txg 只破平局。
    InstanceFirstThenTxg,
    /// 变异 1823：比较不看实例代号。
    IgnoresTheInstance,
    /// 变异 1824：取第一个自证过的槽、不再换。
    FirstSelfVerifiedWins,
}

impl LocalRootChoiceRule {
    pub(super) const ALL: [Self; 5] = [
        Self::NewestWins,
        Self::TieTakesTheLowerInstance,
        Self::InstanceFirstThenTxg,
        Self::IgnoresTheInstance,
        Self::FirstSelfVerifiedWins,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::NewestWins => "newest_wins",
            Self::TieTakesTheLowerInstance => "tie_takes_the_lower_instance",
            Self::InstanceFirstThenTxg => "instance_first_then_txg",
            Self::IgnoresTheInstance => "ignores_the_instance",
            Self::FirstSelfVerifiedWins => "first_self_verified_wins",
        }
    }

    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
    pub(super) fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
        match self {
            Self::NewestWins => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance > chosen.instance)
            }
            Self::TieTakesTheLowerInstance => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance < chosen.instance)
            }
            Self::InstanceFirstThenTxg => {
                candidate.instance > chosen.instance
                    || (candidate.instance == chosen.instance
                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
            }
            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
            Self::FirstSelfVerifiedWins => false,
        }
    }

    pub(super) fn chooses_among(
        self,
        versions_in_ring_order: &[Option<RootVersion>],
    ) -> Option<RootVersion> {
        let mut chosen: Option<RootVersion> = None;
        for candidate in versions_in_ring_order.iter().copied().flatten() {
            if chosen.is_none_or(|already_chosen| self.replaces(candidate, already_chosen)) {
                chosen = Some(candidate);
            }
        }
        chosen
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootChoiceImplementation {
    /// 被测对象：今天的 `choose_root`、`scan_journal`、`replay_journal(…, true)`，不改一个字。
    TodayThreeSteps,
    /// KJ0 的对照：在指定的一态上 panic，别的态照真结局。
    PanicsOnTheDesignatedState,
    /// KF1：所选根取真相里版本最小的根，落点照理想恢复从它接（不调实现）。
    ChoosesTheOldestRoot,
    /// KF2：有已确认发布的态，落点取最后一次已确认那一版的前一版（同实例、txg 减一）；没有已确认的态照理想恢复。
    LandsOneVersionBeforeTheLastConfirmed,
    /// KL 上本地写的一种择根序，不调实现。
    LocalChoice(LocalRootChoiceRule),
}

#[derive(Debug)]
pub(super) enum RootChoiceOutcome {
    Panicked {
        step: &'static str,
    },
    ChoseNothing,
    Landed {
        chosen: RootVersion,
        landed_version: RootVersion,
        landed_fields: VersionFields,
        /// `replay_journal` 交回 `Err`：落到的版本取所选根的（登记第一节读法）。
        replay_failed: bool,
    },
}

// ── 判一个状态（登记 5.5：KJ0、KJ1、KJ2、KJ3 各报各的）

/// 一个状态判之前先算好的真相：根环里每个槽的版本、版本最大的根、最后一次已确认的版本、理想恢复。
struct RootChoiceStateView<'view, 'world> {
    landing: &'view RootChoiceLanding<'world>,
    confirmed_user_publishes: &'view [RootVersion],
    versions_in_ring_order: Vec<Option<RootVersion>>,
    newest_root: Option<&'world RootChoicePublishMaterial>,
    last_confirmed: Option<RootVersion>,
    ideal_recovery_from_the_newest_root: Option<RootChoiceIdealRecovery>,
}

pub(super) fn counter_of_the_landed_record(record: &RootChoiceLandedRecordRef<'_>) -> u64 {
    record.material.records[record.record_index]
        .parsed
        .as_ref()
        .expect("合法历史的每条记录都解得出（生成时断言过）")
        .counter
}

fn describe_the_roots_in_ring_order(
    versions_in_ring_order: &[Option<RootVersion>],
    slots_per_region: u64,
) -> String {
    let mut parts = Vec::new();
    for (slot_index, version) in versions_in_ring_order.iter().enumerate() {
        if let Some(version) = version {
            let slot_number = u64::try_from(slot_index).expect("根槽数不超过 24");
            parts.push(format!(
                "region{}slot{}={}",
                slot_number / slots_per_region,
                slot_number % slots_per_region,
                version.describe()
            ));
        }
    }
    parts.join("+")
}

struct RootChoiceJudgement {
    domain: RootChoiceDomain,
    implementation: RootChoiceImplementation,
    system_configuration: SystemConfiguration,
    in_flight_limit: usize,
}

impl RootChoiceJudgement {
    pub(super) fn new(domain: RootChoiceDomain, implementation: RootChoiceImplementation) -> Self {
        Self {
            domain,
            implementation,
            system_configuration: system_configuration_for_the_root_choice(domain),
            in_flight_limit: domain.ring().local_in_flight_limit(),
        }
    }

    pub(super) fn judge(
        &self,
        landing: &RootChoiceLanding<'_>,
        confirmed_user_publishes: &[RootVersion],
        is_the_designated_state: bool,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        tally.add(RootChoiceCounter::States);
        let newest_root = root_choice_newest_root(landing);
        let view = RootChoiceStateView {
            landing,
            confirmed_user_publishes,
            versions_in_ring_order: root_choice_versions_in_ring_order(landing),
            newest_root,
            last_confirmed: confirmed_user_publishes.iter().copied().max(),
            ideal_recovery_from_the_newest_root: newest_root.map(|newest| {
                root_choice_ideal_recovery_from(newest, &landing.records, self.in_flight_limit)
            }),
        };
        self.count_the_shape_of_the_state(&view, tally);
        let outcome = self.outcome_of(&view, is_the_designated_state, tally, describe_state);
        self.judge_the_outcome(&view, &outcome, tally, describe_state);
    }

    fn count_the_shape_of_the_state(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
    ) {
        let present_versions: Vec<RootVersion> = view
            .versions_in_ring_order
            .iter()
            .copied()
            .flatten()
            .collect();
        if let (Some(first_self_verified), Some(newest)) =
            (present_versions.first(), view.newest_root)
        {
            if *first_self_verified != newest.version {
                tally.add(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest);
            }
        }
        let mut has_same_txg_different_instance = false;
        let mut has_higher_instance_lower_txg = false;
        for earlier in &present_versions {
            for later in &present_versions {
                has_same_txg_different_instance |= earlier.checkpoint_txg == later.checkpoint_txg
                    && earlier.instance != later.instance;
                has_higher_instance_lower_txg |= earlier.instance > later.instance
                    && earlier.checkpoint_txg < later.checkpoint_txg;
            }
        }
        if has_same_txg_different_instance {
            tally.add(RootChoiceCounter::SameTxgDifferentInstanceRoots);
        }
        if has_higher_instance_lower_txg {
            tally.add(RootChoiceCounter::HigherInstanceLowerTxgRoots);
        }
        let mut instances: Vec<u32> = present_versions
            .iter()
            .map(|version| version.instance)
            .collect();
        instances.sort_unstable();
        instances.dedup();
        if instances.len() >= 2 {
            tally.add(RootChoiceCounter::TwoOrMoreInstancesInTheRing);
        }
        if view.newest_root.is_none() {
            tally.add(RootChoiceCounter::StatesWithoutASelfVerifiedRoot);
        }
        if view.last_confirmed.is_some() {
            tally.add(RootChoiceCounter::ConfirmedStates);
        }
        tally.confirmed_publishes_maximum = tally
            .confirmed_publishes_maximum
            .max(view.confirmed_user_publishes.len());
        if view
            .ideal_recovery_from_the_newest_root
            .as_ref()
            .is_some_and(|ideal| ideal.applied_publishes > 0)
        {
            tally.add(RootChoiceCounter::IdealRecoveryApplied);
        }
        for record in &view.landing.records {
            tally.largest_counter = tally
                .largest_counter
                .max(counter_of_the_landed_record(record));
        }
        if let Some(newest) = view.newest_root {
            let above_newest_root = view
                .landing
                .records
                .iter()
                .filter(|record| {
                    record.material.version.instance == newest.version.instance
                        && record.material.version.checkpoint_txg > newest.version.checkpoint_txg
                })
                .count();
            tally.above_newest_root_maximum =
                tally.above_newest_root_maximum.max(above_newest_root);
            if above_newest_root > self.in_flight_limit {
                tally.add(RootChoiceCounter::AboveNewestRootOverLimit);
            } else if above_newest_root == self.in_flight_limit {
                tally.add(RootChoiceCounter::AboveNewestRootAtLimit);
            }
        }
    }

    fn outcome_of(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        is_the_designated_state: bool,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) -> RootChoiceOutcome {
        match self.implementation {
            RootChoiceImplementation::TodayThreeSteps => {
                self.outcome_of_the_three_steps(view, tally, describe_state)
            }
            RootChoiceImplementation::PanicsOnTheDesignatedState => {
                if is_the_designated_state {
                    RootChoiceOutcome::Panicked {
                        step: "designated_state",
                    }
                } else {
                    self.outcome_of_the_three_steps(view, tally, describe_state)
                }
            }
            RootChoiceImplementation::ChoosesTheOldestRoot => {
                let Some(oldest) = root_choice_oldest_root(view.landing) else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                let ideal = root_choice_ideal_recovery_from(
                    oldest,
                    &view.landing.records,
                    self.in_flight_limit,
                );
                RootChoiceOutcome::Landed {
                    chosen: oldest.version,
                    landed_version: ideal.landed_version,
                    landed_fields: ideal.landed_fields,
                    replay_failed: false,
                }
            }
            RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed => {
                let Some(newest) = view.newest_root else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                if let Some(last_confirmed) = view.confirmed_user_publishes.iter().copied().max() {
                    let one_version_before = RootVersion {
                        checkpoint_txg: last_confirmed.checkpoint_txg - 1,
                        instance: last_confirmed.instance,
                    };
                    let landed_fields = view
                        .landing
                        .records
                        .iter()
                        .find(|record| record.material.version == one_version_before)
                        .map(|record| record.material.fields)
                        .expect("最后一次已确认的用户发布前面一次发布的记录整次落过（生成时按这个次序写）");
                    RootChoiceOutcome::Landed {
                        chosen: newest.version,
                        landed_version: one_version_before,
                        landed_fields,
                        replay_failed: false,
                    }
                } else {
                    let ideal = root_choice_ideal_recovery_from(
                        newest,
                        &view.landing.records,
                        self.in_flight_limit,
                    );
                    RootChoiceOutcome::Landed {
                        chosen: newest.version,
                        landed_version: ideal.landed_version,
                        landed_fields: ideal.landed_fields,
                        replay_failed: false,
                    }
                }
            }
            RootChoiceImplementation::LocalChoice(rule) => {
                let Some(chosen) = rule.chooses_among(&view.versions_in_ring_order) else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                let landed_fields = view
                    .landing
                    .roots_in_ring_order
                    .iter()
                    .copied()
                    .flatten()
                    .find(|material| material.version == chosen)
                    .map(|material| material.fields)
                    .expect("择到的版本来自根环里的某个槽");
                RootChoiceOutcome::Landed {
                    chosen,
                    landed_version: chosen,
                    landed_fields,
                    replay_failed: false,
                }
            }
        }
    }
}

impl RootChoiceJudgement {
    /// 被测对象：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`），各包 `catch_unwind`；
    /// 另逐槽调 `read_root_ring_slot` 与真相对拍（登记 5.5 末段、停机条款 TH4）。
    fn outcome_of_the_three_steps(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) -> RootChoiceOutcome {
        let disk = RootChoiceDisk::of_the_landing(self.domain, view.landing);
        self.check_the_slot_readings_against_the_truth(&disk, view, tally, describe_state);
        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
        let outcome = self.outcome_of_the_three_steps_with_the_panic_hook_quiet(&disk);
        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
        outcome
    }

    pub(super) fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
        &self,
        disk: &RootChoiceDisk<'_>,
    ) -> RootChoiceOutcome {
        let chosen_root = match panic::catch_unwind(AssertUnwindSafe(|| {
            choose_root(disk, &self.system_configuration)
        })) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "choose_root",
                }
            }
            Ok(None) => return RootChoiceOutcome::ChoseNothing,
            Ok(Some(root)) => root,
        };
        let records = match panic::catch_unwind(AssertUnwindSafe(|| {
            scan_journal(disk, &self.system_configuration)
        })) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "scan_journal",
                }
            }
            Ok(records) => records,
        };
        let replay = panic::catch_unwind(AssertUnwindSafe(|| {
            replay_journal(
                disk,
                &chosen_root,
                self.system_configuration.immutable.sizes.journal_ring_bytes,
                &records,
                NamedUnitVerification::Verify,
            )
        }));
        match replay {
            Err(_panic_payload) => RootChoiceOutcome::Panicked {
                step: "replay_journal",
            },
            Ok(Err(_failure)) => RootChoiceOutcome::Landed {
                chosen: RootVersion::of_root(&chosen_root),
                landed_version: RootVersion::of_root(&chosen_root),
                landed_fields: VersionFields::of_root(&chosen_root),
                replay_failed: true,
            },
            Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
                chosen: RootVersion::of_root(&chosen_root),
                landed_version: RootVersion::of_root(&returned_root),
                landed_fields: VersionFields::of_root(&returned_root),
                replay_failed: false,
            },
        }
    }

    /// 读槽对拍：逐槽调 `read_root_ring_slot`，自证过的槽集合与每个槽交回的根必须等于生成时的真相（没写过的槽读成全 0、自证不过）。
    /// 不一致不当 KJ 的违例记，单独数、记见证，收尾走停机条款 TH4。
    fn check_the_slot_readings_against_the_truth(
        &self,
        disk: &RootChoiceDisk<'_>,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        tally.add(RootChoiceCounter::SlotReadingsChecked);
        let immutable = &self.system_configuration.immutable;
        for ring_slot in every_root_ring_slot(&immutable.sizes) {
            let reading = read_root_ring_slot(
                disk,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            );
            let slot_index = root_choice_slot_index_of(
                ring_slot.region,
                ring_slot.slot,
                self.domain.slots_per_region(),
            );
            let truth = view.landing.roots_in_ring_order[slot_index];
            if !root_slot_reading_agrees_with_the_truth(reading, truth) {
                tally.add_violation(
                    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
                    &|| {
                        format!(
                            "region{}_slot{}_read_as={reading:?}_truth={};{}",
                            ring_slot.region,
                            ring_slot.slot,
                            truth.map_or_else(
                                || "empty".to_string(),
                                |material| material.version.describe()
                            ),
                            describe_state()
                        )
                    },
                );
            }
        }
    }

    /// 一个状态的结局落进四条判据（各报各的，不合成一个真假）。
    fn judge_the_outcome(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        outcome: &RootChoiceOutcome,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        match outcome {
            RootChoiceOutcome::Panicked { step } => {
                tally.add(RootChoiceCounter::PanickedStates);
                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
                    format!("panicked_in={step};{}", describe_state())
                });
            }
            RootChoiceOutcome::ChoseNothing => {
                tally.add(RootChoiceCounter::ChoseNothingStates);
                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
                    format!("choose_root_returned_none;{}", describe_state())
                });
            }
            RootChoiceOutcome::Landed {
                chosen,
                landed_version,
                landed_fields,
                replay_failed,
            } => {
                if *replay_failed {
                    tally.add(RootChoiceCounter::ReplayFailedStates);
                }
                let above_chosen_root = view
                    .landing
                    .records
                    .iter()
                    .filter(|record| {
                        record.material.version.instance == chosen.instance
                            && record.material.version.checkpoint_txg > chosen.checkpoint_txg
                    })
                    .count();
                tally.above_chosen_root_maximum =
                    tally.above_chosen_root_maximum.max(above_chosen_root);
                if above_chosen_root > self.in_flight_limit {
                    tally.add(RootChoiceCounter::AboveChosenRootOverLimit);
                } else if above_chosen_root == self.in_flight_limit {
                    tally.add(RootChoiceCounter::AboveChosenRootAtLimit);
                }
                if landed_version != chosen {
                    tally.add(RootChoiceCounter::ReplayAppliedStates);
                }
                let facts = || {
                    format!(
                        "chosen={};landed={};newest_root={};last_confirmed={};roots_in_ring_order={};{}",
                        chosen.describe(),
                        landed_version.describe(),
                        view.newest_root
                            .map_or_else(|| "none".to_string(), |newest| newest.version.describe()),
                        view.last_confirmed
                            .map_or_else(|| "none".to_string(), RootVersion::describe),
                        describe_the_roots_in_ring_order(
                            &view.versions_in_ring_order,
                            self.domain.slots_per_region()
                        ),
                        describe_state()
                    )
                };
                if self.domain.is_a_legal_history() {
                    if let Some(last_confirmed) = view.last_confirmed {
                        tally.add(RootChoiceCounter::Kj1Judged);
                        if *landed_version < last_confirmed {
                            tally.add_violation(RootChoiceCounter::Kj1Violations, &|| {
                                format!("KJ1;{}", facts())
                            });
                        }
                    }
                }
                if let Some(newest) = view.newest_root {
                    tally.add(RootChoiceCounter::Kj2Judged);
                    if *landed_version < newest.version {
                        tally.add_violation(RootChoiceCounter::Kj2Violations, &|| {
                            format!("KJ2;{}", facts())
                        });
                    }
                }
                if self.domain.is_a_legal_history() {
                    if let Some(ideal) = &view.ideal_recovery_from_the_newest_root {
                        tally.add(RootChoiceCounter::Kj3Judged);
                        if *landed_fields != ideal.landed_fields {
                            tally.add_violation(RootChoiceCounter::Kj3Violations, &|| {
                                format!(
                                    "KJ3;ideal_landed={};{}",
                                    ideal.landed_version.describe(),
                                    facts()
                                )
                            });
                        }
                    }
                }
            }
        }
    }
}

/// 一个根环槽的读数对真相：读成自证过的，它的 (txg, 实例代号) 必须等于真相里这个槽的根；读成自证不过的（全 0 的槽），真相里这个槽必须是空的；
/// 读不出一律不合（内存盘对根环的槽只回 512 字节或全 0）。
pub(super) fn root_slot_reading_agrees_with_the_truth(
    reading: RootRingSlotReading,
    truth: Option<&RootChoicePublishMaterial>,
) -> bool {
    match reading {
        RootRingSlotReading::SelfVerified(root) => truth.is_some_and(|material| {
            root.checkpoint_txg.0 == material.version.checkpoint_txg
                && root.instance.0 == material.version.instance
        }),
        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => truth.is_none(),
        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => false,
    }
}

// ── 枚举一个 (域, M, U) 格里的全部状态

/// 一次挂载里写到哪了：下一条记录的计数器、下一个事务号、本实例上一条记录的链值、下一次发布的 txg。
#[derive(Clone, Copy, Debug)]
pub(super) struct RootChoiceMountCursor {
    pub(super) instance: u32,
    pub(super) next_counter: u64,
    pub(super) next_transaction: u64,
    pub(super) previous_chain_value: Option<u32>,
    pub(super) next_checkpoint_txg: u64,
}

impl RootChoiceMountCursor {
    pub(super) fn after_the_publish_of(self, material: &RootChoicePublishMaterial) -> Self {
        let last_slot = material.records.last().expect("每次发布至少一条记录");
        let commit_records = material
            .records
            .iter()
            .filter(|slot| {
                slot.parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出（生成时断言过）")
                    .is_commit
            })
            .count();
        Self {
            instance: self.instance,
            next_counter: self.next_counter
                + u64::try_from(material.records.len()).expect("一次发布的记录条数不超过 2"),
            next_transaction: if material.is_user_publish {
                self.next_transaction
                    + u64::try_from(commit_records).expect("一次发布的记录条数不超过 2")
            } else {
                self.next_transaction
            },
            previous_chain_value: Some(last_slot.chain_value_of_header),
            next_checkpoint_txg: material.version.checkpoint_txg + 1,
        }
    }

    pub(super) fn plan_of_the_next_publish(
        self,
        domain: RootChoiceDomain,
        publishes_started: u64,
        record_count: usize,
        commit_marker_bits: u32,
        is_user_publish: bool,
    ) -> RootChoicePublishPlan {
        RootChoicePublishPlan {
            domain,
            instance: self.instance,
            checkpoint_txg: self.next_checkpoint_txg,
            publish_ordinal: publishes_started,
            first_counter: self.next_counter,
            new_pool_file_creation: self.next_transaction,
            record_count,
            commit_marker_bits,
            previous_chain_value: self.previous_chain_value,
            is_user_publish,
        }
    }
}

/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
pub(super) trait RootChoiceVisitor {
    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
    );

    /// 最后一次挂载的崩溃点：`world` 是崩溃在途那次发布之前的盘，`crashed` 是它的崩溃点，`trace` 是这次挂载到这里为止的写法。
    fn visit_a_final_state(
        &mut self,
        world: &RootChoiceWorld,
        crashed: &RootChoiceCrashedPublish<'_>,
        trace: &str,
    );
}

struct RootChoiceEnumeration {
    domain: RootChoiceDomain,
    mounts: usize,
    stop_before_mount: Option<usize>,
}

impl RootChoiceEnumeration {
    /// 第 `mount_number` 次挂载（实例代号 = 它）：写行那次发布、暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的（登记 5.3）。
    pub(super) fn enumerate_a_mount(
        &self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        if self.stop_before_mount == Some(mount_number) {
            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
            return;
        }
        let is_the_last_mount = mount_number == self.mounts;
        let overhead_txgs = root_choice_warm_up_plan(world.highest_checkpoint_txg_on_disk() + 1);
        let first_user_txg = overhead_txgs.last().expect("写行那次发布至少有一次") + 1;
        let user_publish_counts: Vec<usize> = if is_the_last_mount {
            vec![user_publishes_left]
        } else {
            (0..=user_publishes_left).collect()
        };
        for user_publish_count in user_publish_counts {
            let user_publishes_after_this_mount = user_publishes_left - user_publish_count;
            let mut running_world = world.clone();
            let mut cursor = RootChoiceMountCursor {
                instance: u32::try_from(mount_number).expect("挂载次数不超过 3"),
                next_counter: world.highest_landed_counter() + 1,
                next_transaction: 1,
                previous_chain_value: None,
                next_checkpoint_txg: overhead_txgs[0],
            };
            let mut trace = format!(
                "mount{mount_number}:overhead_txgs={}",
                overhead_txgs
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<String>>()
                    .join(",")
            );
            for overhead_txg in &overhead_txgs {
                assert_eq!(
                    cursor.next_checkpoint_txg, *overhead_txg,
                    "开销发布的 txg 逐个加一"
                );
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain,
                        running_world.publishes_started,
                        1,
                        0,
                        false,
                    ),
                ));
                if user_publish_count == 0 {
                    self.enumerate_the_crash_points(
                        &running_world,
                        &material,
                        &trace,
                        (mount_number, user_publishes_after_this_mount),
                        visitor,
                    );
                }
                running_world = running_world.with_publish_applied(&material, false);
                cursor = cursor.after_the_publish_of(&material);
            }
            if user_publish_count > 0 {
                assert_eq!(cursor.next_checkpoint_txg, first_user_txg);
                trace.push_str(";user_publishes");
                self.enumerate_user_publishes(
                    &running_world,
                    cursor,
                    (
                        mount_number,
                        user_publish_count,
                        user_publishes_after_this_mount,
                    ),
                    &trace,
                    visitor,
                );
            }
        }
    }

    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
    pub(super) fn enumerate_user_publishes(
        &self,
        running_world: &RootChoiceWorld,
        cursor: RootChoiceMountCursor,
        (mount_number, user_publishes_in_the_mount, user_publishes_after_this_mount): (
            usize,
            usize,
            usize,
        ),
        trace: &str,
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        for record_count in self.domain.user_publish_record_counts() {
            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain,
                        running_world.publishes_started,
                        *record_count,
                        commit_marker_bits,
                        true,
                    ),
                ));
                let trace_with_this_publish = format!(
                    "{trace}|txg{}k{record_count}b{commit_marker_bits}",
                    material.version.checkpoint_txg
                );
                if user_publishes_in_the_mount == 1 {
                    self.enumerate_the_crash_points(
                        running_world,
                        &material,
                        &trace_with_this_publish,
                        (mount_number, user_publishes_after_this_mount),
                        visitor,
                    );
                } else {
                    let applied_world = running_world.with_publish_applied(&material, true);
                    self.enumerate_user_publishes(
                        &applied_world,
                        cursor.after_the_publish_of(&material),
                        (
                            mount_number,
                            user_publishes_in_the_mount - 1,
                            user_publishes_after_this_mount,
                        ),
                        &trace_with_this_publish,
                        visitor,
                    );
                }
            }
        }
    }

    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
    pub(super) fn enumerate_the_crash_points(
        &self,
        running_world: &RootChoiceWorld,
        material: &Arc<RootChoicePublishMaterial>,
        trace: &str,
        (mount_number, user_publishes_after_this_mount): (usize, usize),
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        for crash_point in root_choice_crash_points_of(material.records.len()) {
            let crashed = RootChoiceCrashedPublish {
                material,
                crash_point,
            };
            if mount_number == self.mounts {
                visitor.visit_a_final_state(running_world, &crashed, trace);
            } else {
                let crashed_world = running_world.with_crash_applied(&crashed, trace);
                self.enumerate_a_mount(
                    &crashed_world,
                    mount_number + 1,
                    user_publishes_after_this_mount,
                    visitor,
                );
            }
        }
    }
}

// ── 一个格的计数：枚举、判、按工作单元分给几条线程

struct RootChoiceUnit {
    world: RootChoiceWorld,
    mount_number: usize,
    user_publishes_left: usize,
}

/// 工作单元：前面几次挂载（最多两次）的全部走法在崩溃之后得到的盘，每个单元接着枚举剩下的挂载。
struct RootChoiceUnitCollector {
    units: Vec<RootChoiceUnit>,
}

impl RootChoiceVisitor for RootChoiceUnitCollector {
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
    ) {
        self.units.push(RootChoiceUnit {
            world: world.clone(),
            mount_number,
            user_publishes_left,
        });
    }

    fn visit_a_final_state(
        &mut self,
        _world: &RootChoiceWorld,
        _crashed: &RootChoiceCrashedPublish<'_>,
        _trace: &str,
    ) {
        panic!("收工作单元的那一遍停在第 min(M, 3) 次挂载开始之前，走不到最后一次挂载的崩溃点");
    }
}

struct RootChoiceStateJudge<'run> {
    judgement: &'run RootChoiceJudgement,
    tally: RootChoiceTally,
    states_seen_in_this_unit: u64,
    is_the_first_unit: bool,
}

impl RootChoiceVisitor for RootChoiceStateJudge<'_> {
    fn reach_the_start_of_a_mount(
        &mut self,
        _world: &RootChoiceWorld,
        _mount_number: usize,
        _user_publishes_left: usize,
    ) {
        panic!("判状态的那一遍不设 stop_before_mount");
    }

    fn visit_a_final_state(
        &mut self,
        world: &RootChoiceWorld,
        crashed: &RootChoiceCrashedPublish<'_>,
        trace: &str,
    ) {
        let landing = root_choice_landing_of(world, Some(crashed));
        let describe_state =
            || format!("{}|{trace};{}", world.history.join("/"), crashed.describe());
        let is_the_designated_state = self.is_the_first_unit && self.states_seen_in_this_unit == 0;
        self.states_seen_in_this_unit += 1;
        self.judgement.judge(
            &landing,
            &world.confirmed_user_publishes,
            is_the_designated_state,
            &mut self.tally,
            &describe_state,
        );
    }
}

/// KL（第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 1.3）：txg 100、101、102 各自只在它的落点槽里摆根（D22 已定项 16 的落点公式：
/// txg 102 → 区域 0 槽 2，txg 100 → 区域 1 槽 1，txg 101 → 区域 2 槽 1），每个落点槽取空、或那个 txg 的实例 1、实例 2 两条根之一，3³ = 27 态；
/// 其余根槽全 0（不摆第 0 代根），journal 环全 0。同 txg 两条根不同在，C743 的落点核不挡任何一条。
const ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_TXGS: &[u64] = &[100, 101, 102];
const ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_INSTANCES: [u32; 2] = [1, 2];

/// KL 的一态：根槽数组（区域号主序，与 `RootChoiceWorld::root_slots` 同一个次序）与见证串。
pub(super) struct LandingSlotAdversarialRootRing {
    pub(super) root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
    pub(super) witness: String,
}

/// KL 的全部状态。r3 档（三步与本地择根序）与 r4 档（本地择根序）都从这里取，两档的 KL 出自同一段代码（登记 5.6 H2）。
/// 落点槽按读根环次序排；状态号是三位三进制，读根环次序最前的落点槽是最高位，每位 0 空、1 实例 1、2 实例 2。
/// 每摆一条根都过 TH2 回比（本地 (区域, 槽) 与 `root_ring::target_for_publish` 相等，不符当场停）。
pub(super) fn landing_slot_adversarial_root_rings() -> Vec<LandingSlotAdversarialRootRing> {
    let domain = RootChoiceDomain::AdversarialRootRing;
    let mut txgs_in_ring_order: Vec<u64> = ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_TXGS.to_vec();
    txgs_in_ring_order
        .sort_by_key(|checkpoint_txg| root_choice_slot_index_of_publish(domain, *checkpoint_txg));
    let landing_slot_indices: Vec<usize> = txgs_in_ring_order
        .iter()
        .map(|checkpoint_txg| root_choice_slot_index_of_publish(domain, *checkpoint_txg))
        .collect();
    // 每个落点槽的候选根是那个 txg 的两个实例；新根段六条各不相同（同 r3 KA），按 txg 升序、实例升序编号。
    let mut candidates_per_landing_slot: Vec<Vec<Arc<RootChoicePublishMaterial>>> =
        vec![Vec::new(); txgs_in_ring_order.len()];
    let mut segment_number: u64 = 0;
    for checkpoint_txg in ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_TXGS {
        let landing_position = txgs_in_ring_order
            .iter()
            .position(|ordered_txg| ordered_txg == checkpoint_txg)
            .expect("每个 txg 都排进了读根环次序");
        for instance in ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_INSTANCES {
            segment_number += 1;
            candidates_per_landing_slot[landing_position].push(Arc::new(
                root_material_without_records(
                    instance,
                    *checkpoint_txg,
                    NewRootSegment::off_timeline(segment_number, *checkpoint_txg),
                ),
            ));
        }
    }
    let choices_per_landing_slot = ROOT_CHOICE_LANDING_SLOT_ADVERSARIAL_INSTANCES.len() + 1;
    let state_count = choices_per_landing_slot
        .pow(u32::try_from(txgs_in_ring_order.len()).expect("落点槽不超过三个"));
    let root_slot_count = usize::try_from(ROOT_CHOICE_REGION_COUNT * domain.slots_per_region())
        .expect("根槽数不超过 24");
    let mut rings = Vec::with_capacity(state_count);
    for state_number in 0..state_count {
        let mut root_slots = vec![None; root_slot_count];
        let mut choice_texts = vec![String::new(); txgs_in_ring_order.len()];
        let mut remaining_choices = state_number;
        for landing_position in (0..txgs_in_ring_order.len()).rev() {
            let choice = remaining_choices % choices_per_landing_slot;
            remaining_choices /= choices_per_landing_slot;
            choice_texts[landing_position] = if choice == 0 {
                "empty".to_string()
            } else {
                let material = &candidates_per_landing_slot[landing_position][choice - 1];
                assert_the_local_root_slot_arithmetic_against_the_implementation(
                    domain,
                    material.version.checkpoint_txg,
                );
                root_slots[landing_slot_indices[landing_position]] = Some(Arc::clone(material));
                format!(
                    "({},{})",
                    material.version.checkpoint_txg, material.version.instance
                )
            };
        }
        rings.push(LandingSlotAdversarialRootRing {
            root_slots,
            witness: format!("kl_landing_slots=[{}]", choice_texts.join(",")),
        });
    }
    rings
}

pub(super) fn root_choice_tally_of_the_adversarial_root_rings(
    implementation: RootChoiceImplementation,
) -> RootChoiceTally {
    let domain = RootChoiceDomain::AdversarialRootRing;
    let judgement = RootChoiceJudgement::new(domain, implementation);
    let mut tally = RootChoiceTally::new();
    for ring in landing_slot_adversarial_root_rings() {
        let world = RootChoiceWorld {
            domain,
            root_slots: ring.root_slots,
            landed_records: Vec::new(),
            confirmed_user_publishes: Vec::new(),
            publishes_started: 0,
            history: Vec::new(),
        };
        let landing = root_choice_landing_of(&world, None);
        let describe_state = || ring.witness.clone();
        judgement.judge(&landing, &[], false, &mut tally, &describe_state);
    }
    tally
}

/// 一个 (域, M, U) 格上、一种实现下全部状态的计数。KL 单线程走 27 个状态；合法历史的三个域按工作单元分给几条线程。
pub(super) fn root_choice_tally_of_a_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    implementation: RootChoiceImplementation,
    thread_count: usize,
) -> RootChoiceTally {
    if !domain.is_a_legal_history() {
        return root_choice_tally_of_the_adversarial_root_rings(implementation);
    }
    let collecting = RootChoiceEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: Some(cell.mounts.min(3)),
    };
    let mut collector = RootChoiceUnitCollector { units: Vec::new() };
    collecting.enumerate_a_mount(
        &RootChoiceWorld::right_after_mkfs(domain),
        1,
        cell.user_publishes,
        &mut collector,
    );
    let judgement = RootChoiceJudgement::new(domain, implementation);
    let continuing = RootChoiceEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: None,
    };
    root_choice_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
        let mut judge = RootChoiceStateJudge {
            judgement: &judgement,
            tally: RootChoiceTally::new(),
            states_seen_in_this_unit: 0,
            is_the_first_unit: unit_index == 0,
        };
        continuing.enumerate_a_mount(
            &unit.world,
            unit.mount_number,
            unit.user_publishes_left,
            &mut judge,
        );
        judge.tally
    })
}

#[cfg(test)]
mod tests;
