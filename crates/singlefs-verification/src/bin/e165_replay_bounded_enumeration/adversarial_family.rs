//! 对抗一族（登记 5.2 的 A、A 小环、A 链）：记录字母表、槽的取法、所选根、逐态枚举与判定、按工作单元与按 (域, N) 的计数。

use super::*;

// ── 对抗一族（登记 5.2 的 A、A 小环、A 链）

/// 对抗域里一个槽上那条记录的自由字段（登记 5.2：txg ∈ {t, t + 1, t + 2} × 序号 ∈ {0, 1, 2} × 末条标志 × 提交标记 × 事务号 ∈ {1, 2}）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AdversarialRecordFields {
    txg_above_the_chosen_root: u64,
    pub(super) ordinal_within_publish: u32,
    pub(super) carries_the_last_record_flag: bool,
    pub(super) is_commit: bool,
    pub(super) transaction: u64,
}

/// 72 种自由字段的取法，次序写死（枚举次序进见证的先后）。
pub(super) fn adversarial_record_alphabet() -> Vec<AdversarialRecordFields> {
    let mut alphabet = Vec::new();
    for txg_above_the_chosen_root in 0..3u64 {
        for ordinal_within_publish in 0..3u32 {
            for carries_the_last_record_flag in [false, true] {
                for is_commit in [false, true] {
                    for transaction in 1..=2u64 {
                        alphabet.push(AdversarialRecordFields {
                            txg_above_the_chosen_root,
                            ordinal_within_publish,
                            carries_the_last_record_flag,
                            is_commit,
                            transaction,
                        });
                    }
                }
            }
        }
    }
    alphabet
}

/// 一个槽的取法：不在，或字母表里第几种记录；A 链另取反向链对 / 错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdversarialSlotChoice {
    HoldsNothing,
    Record {
        alphabet_index: usize,
        back_chain_is_wrong: bool,
    },
}

/// 一个槽的全部取法：A、A 小环 73 种，A 链 145 种（每条在的记录的反向链另取对 / 错）。
fn adversarial_slot_choices(domain: Domain, alphabet_length: usize) -> Vec<AdversarialSlotChoice> {
    let mut choices = vec![AdversarialSlotChoice::HoldsNothing];
    for alphabet_index in 0..alphabet_length {
        choices.push(AdversarialSlotChoice::Record {
            alphabet_index,
            back_chain_is_wrong: false,
        });
        if domain == Domain::AdversarialBackChain {
            choices.push(AdversarialSlotChoice::Record {
                alphabet_index,
                back_chain_is_wrong: true,
            });
        }
    }
    choices
}

/// 对抗一族的所选根：时间线 R_0 那一份（登记 r2 第一节：投影之外的四个字段照档取第 0 版的）。
pub(super) fn adversarial_chosen_root(run_profile: RunProfile) -> RootRecord {
    VersionFields {
        instance: CHOSEN_INSTANCE,
        checkpoint_txg: ORDINARY_FIRST_TXG,
        segment: NewRootSegment::of_timeline_publish(0, ORDINARY_FIRST_TXG),
    }
    .to_root_with(run_profile.fields_beyond_the_projection_of_version(0))
}

/// 对抗一族锚点之后第 `position` 个槽上字母表第 `alphabet_index` 种记录的新根段投影与后三项：区分号 d = 槽位 × 1000 + 字母表下标，
/// 新根段取时间线之外的值，后三项取 h(d)（登记 r5 5.3 第 2 条；r1–r4 档是 r1 常量）。生成器与生成器真相都从这里取。
pub(super) fn adversarial_record_projection_and_fields(
    chosen_root: &RootRecord,
    alphabet_fields: AdversarialRecordFields,
    position: usize,
    alphabet_index: usize,
    run_profile: RunProfile,
) -> (VersionFields, RecordFieldsBeyondTheProjection) {
    let checkpoint_txg = chosen_root.checkpoint_txg.0 + alphabet_fields.txg_above_the_chosen_root;
    let distinguishing_number =
        u64::try_from(position * 1000 + alphabet_index).expect("槽位与字母表下标都很小");
    (
        VersionFields {
            instance: CHOSEN_INSTANCE,
            checkpoint_txg,
            segment: NewRootSegment::off_timeline(distinguishing_number, checkpoint_txg),
        },
        run_profile.record_fields_off_the_timeline(distinguishing_number),
    )
}

/// 对抗一族一个域在 `slot_count` 个槽上能造出的每条记录（锚点与每个槽位上字母表的每一种）的投影对到后三项（登记 r5 5.3 第 3 条）。
pub(super) fn adversarial_record_truth(
    slot_count: usize,
    alphabet: &[AdversarialRecordFields],
    chosen_root: &RootRecord,
    run_profile: RunProfile,
) -> GeneratedRecordTruth {
    let mut truth = GeneratedRecordTruth::new();
    truth.note(
        VersionFields::of_root(chosen_root),
        run_profile.record_fields_of_the_timeline_publish(0),
    );
    for position in 1..=slot_count {
        for (alphabet_index, alphabet_fields) in alphabet.iter().enumerate() {
            let (projection, record_fields) = adversarial_record_projection_and_fields(
                chosen_root,
                *alphabet_fields,
                position,
                alphabet_index,
                run_profile,
            );
            truth.note(projection, record_fields);
        }
    }
    truth
}

/// 对抗域枚举到一半的状态：锚点槽与其后各槽上放着什么、由它们建出来的表。
struct AdversarialEnumeration<'run> {
    domain: Domain,
    slot_count: usize,
    alphabet: &'run [AdversarialRecordFields],
    choices: &'run [AdversarialSlotChoice],
    chosen_root: RootRecord,
    implementation: Implementation,
    run_profile: RunProfile,
    /// 这个域这一档字母表能造出的每条记录（与锚点）的投影对到后三项（登记 r5 5.3 第 3 条），J8a 拿它定期望。
    record_truth: &'run GeneratedRecordTruth,
    /// 下标 0 是锚点槽，之后是第 1..n 个槽；`None` 是不在。
    slots: Vec<Option<GeneratedSlot>>,
    choice_trail: Vec<AdversarialSlotChoice>,
    is_the_designated_state: bool,
}

impl AdversarialEnumeration<'_> {
    pub(super) fn anchor_counter(&self) -> u64 {
        self.domain.ring().first_history_counter()
    }

    /// 锚点：计数器 a，txg = 所选根的 txg、序号 1、带末条标志与提交标记（登记 5.2）。
    fn generate_the_anchor_slot(&self) -> GeneratedSlot {
        let chosen_fields = VersionFields::of_root(&self.chosen_root);
        // 锚点是所选根 R_0 那次发布的记录：r5 档带 R_0 的后三项（登记 r5 5.3 第 1 条），别的档 r1 常量。
        let anchor_record_fields = self.run_profile.record_fields_of_the_timeline_publish(0);
        let record = JournalRecord {
            instance: InstanceGeneration(CHOSEN_INSTANCE),
            counter: self.anchor_counter(),
            checkpoint_txg: CheckpointTxg(chosen_fields.checkpoint_txg),
            transaction: 1,
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
            back_chain: BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: chosen_fields.segment.tree_table,
            new_mapping_root: chosen_fields.segment.mapping_root,
            new_tree_identifier_watermark: chosen_fields.segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(chosen_fields.segment.rollback_floor),
            new_instance_table: anchor_record_fields.instance_table,
            new_allocation_record_tree_root: anchor_record_fields.allocation_record_tree_root,
            new_unmount_marker: anchor_record_fields.unmount_marker,
            named: Vec::new(),
            new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
            new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        };
        generate_slot(
            0,
            self.domain.ring(),
            &record,
            ParseExpectation::MustRoundTrip,
        )
    }

    /// 锚点之后第 `position` 个槽（从 1 数）上的一条记录：字段照字母表，新根段每条各不相同，
    /// 反向链照盘上前一槽的真值（前一槽不在时取固定值），A 链取错时把真值异或 1。
    fn generate_a_record_slot(
        &self,
        position: usize,
        alphabet_index: usize,
        back_chain_is_wrong: bool,
    ) -> GeneratedSlot {
        let fields = self.alphabet[alphabet_index];
        let true_back_chain = match self.slots[position - 1].as_ref() {
            Some(slot_before) => slot_before.chain_value_of_header,
            None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
        };
        let (projection, record_fields) = adversarial_record_projection_and_fields(
            &self.chosen_root,
            fields,
            position,
            alphabet_index,
            self.run_profile,
        );
        let checkpoint_txg = projection.checkpoint_txg;
        let segment = projection.segment;
        let record = JournalRecord {
            instance: InstanceGeneration(CHOSEN_INSTANCE),
            counter: self.anchor_counter() + u64::try_from(position).expect("槽位不超过 4"),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            transaction: fields.transaction,
            is_commit: fields.is_commit,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                fields.ordinal_within_publish,
            ),
            place_in_publish: if fields.carries_the_last_record_flag {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain: if back_chain_is_wrong {
                true_back_chain ^ 1
            } else {
                true_back_chain
            },
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: segment.tree_table,
            new_mapping_root: segment.mapping_root,
            new_tree_identifier_watermark: segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(segment.rollback_floor),
            new_instance_table: record_fields.instance_table,
            new_allocation_record_tree_root: record_fields.allocation_record_tree_root,
            new_unmount_marker: record_fields.unmount_marker,
            named: Vec::new(),
            new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
            new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        };
        generate_slot(
            0,
            self.domain.ring(),
            &record,
            ParseExpectation::OrdinalZeroMustBeRejected,
        )
    }

    fn describe_the_state(&self) -> String {
        let anchor = if self.slots[0].is_some() {
            "anchor=present"
        } else {
            "anchor=absent"
        };
        let slots: Vec<String> = self
            .choice_trail
            .iter()
            .map(|choice| match choice {
                AdversarialSlotChoice::HoldsNothing => "-".to_string(),
                AdversarialSlotChoice::Record {
                    alphabet_index,
                    back_chain_is_wrong,
                } => {
                    let fields = self.alphabet[*alphabet_index];
                    format!(
                        "t+{}:o{}:L{}:C{}:x{}:chain_{}",
                        fields.txg_above_the_chosen_root,
                        fields.ordinal_within_publish,
                        u8::from(fields.carries_the_last_record_flag),
                        u8::from(fields.is_commit),
                        fields.transaction,
                        if *back_chain_is_wrong {
                            "wrong"
                        } else {
                            "true"
                        }
                    )
                }
            })
            .collect();
        format!("{anchor};slots={}", slots.join(","))
    }

    /// 从第 `position` 个槽起把剩下的槽取遍；取满 n 个槽就判这一个状态。
    fn descend(&mut self, position: usize, tally: &mut Tally) {
        if position > self.slot_count {
            self.judge_the_state(tally);
            self.is_the_designated_state = false;
            return;
        }
        for choice_index in 0..self.choices.len() {
            self.put_the_choice_and_descend(position, self.choices[choice_index], tally);
        }
    }

    fn put_the_choice_and_descend(
        &mut self,
        position: usize,
        choice: AdversarialSlotChoice,
        tally: &mut Tally,
    ) {
        let slot = match choice {
            AdversarialSlotChoice::HoldsNothing => None,
            AdversarialSlotChoice::Record {
                alphabet_index,
                back_chain_is_wrong,
            } => Some(self.generate_a_record_slot(position, alphabet_index, back_chain_is_wrong)),
        };
        self.slots.push(slot);
        self.choice_trail.push(choice);
        self.descend(position + 1, tally);
        self.choice_trail.pop();
        self.slots.pop();
    }

    /// 一个对抗状态的写法（逐态行的 `state=` 与每条违例的见证）：锚点与各槽、第一次调用的结局、S 三种读法的期望。
    fn describe_the_state_with_its_first_outcome(
        &self,
        first: &ReplayOutcome,
        expectations: &[SpecificationExpectation; 3],
    ) -> String {
        format!(
            "{};first={};expected={}",
            self.describe_the_state(),
            describe_outcome(first),
            expectations
                .iter()
                .map(|expectation| match expectation {
                    SpecificationExpectation::FailsWithoutApplying => "fails".to_string(),
                    SpecificationExpectation::ReturnsRoot(fields) => format!(
                        "txg{}:watermark{}",
                        fields.checkpoint_txg, fields.segment.tree_identifier_watermark
                    ),
                })
                .collect::<Vec<String>>()
                .join("/")
        )
    }

    /// 判对抗域的一个状态：A0、AS 三条、A-互 三个数、A-结构、A-限、J6。记录表怎么建由档的 `table_source` 定（登记 r2 5.4）：
    /// `Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。对抗域的槽都在 0 号盘，盘次序不改直接建的表，不另判「照盘报的次序」那一栏。
    fn judge_the_state(&self, tally: &mut Tally) {
        let probe_decision = begin_probing_a_state(tally);
        if matches!(probe_decision, ProbeDecision::SkipThisState) {
            return;
        }
        let ring = self.domain.ring();
        let mut direct_table = RecordTable::new();
        let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
        let mut contents: Vec<(u32, u64, StoredBytes<'_>)> = Vec::new();
        let mut holds_a_record_rejected_by_parse = false;
        for slot in self.slots.iter().flatten() {
            contents.push((
                slot.device,
                slot.offset_in_bytes,
                StoredBytes::Readable(&slot.bytes),
            ));
            match slot.parsed.as_ref() {
                Some(parsed) => {
                    direct_table.insert((parsed.instance, parsed.counter), parsed.clone());
                    truths.push(RecordTruthOnDisk {
                        instance: parsed.instance.0,
                        counter: parsed.counter,
                        chain_value_of_header: slot.chain_value_of_header,
                        named_units_all_verified: true,
                    });
                }
                None => holds_a_record_rejected_by_parse = true,
            }
        }
        let disk = MemoryDisk {
            ring,
            reported_disk_order: self.run_profile.reported_disk_order,
            contents,
        };
        let judge_on = |table: &RecordTable,
                        implementation_of_the_pass: Implementation,
                        designated_in_the_pass: bool,
                        tally_of_the_pass: &mut Tally| {
            self.judge_the_state_on_a_table(
                table,
                &disk,
                &truths,
                holds_a_record_rejected_by_parse,
                implementation_of_the_pass,
                designated_in_the_pass,
                tally_of_the_pass,
            )
        };
        let judgement = match self.run_profile.table_source {
            RecordTableSource::Direct => judge_on(
                &direct_table,
                self.implementation,
                self.is_the_designated_state,
                tally,
            ),
            RecordTableSource::Scanned => {
                let scanned_table = scanned_record_table_of(&disk, self.implementation);
                judge_on(
                    &scanned_table,
                    self.implementation,
                    self.is_the_designated_state,
                    tally,
                )
            }
            RecordTableSource::Both => {
                let scanned_table = scanned_record_table_of(&disk, self.implementation);
                let counts_before_the_state = tally.counts.clone();
                let (scan_pass_implementation, scan_pass_is_designated) =
                    implementation_and_designation_of_the_scan_pass(
                        self.implementation,
                        self.is_the_designated_state,
                    );
                let mut scanned_tally = Tally::new();
                let scanned_judgement = judge_on(
                    &scanned_table,
                    scan_pass_implementation,
                    scan_pass_is_designated,
                    &mut scanned_tally,
                );
                let scanned_summary = JudgedRowSummary {
                    first_outcome: scanned_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        &scanned_tally,
                        &Tally::new().counts,
                    ),
                };
                let direct_judgement = judge_on(
                    &direct_table,
                    self.implementation,
                    self.is_the_designated_state,
                    tally,
                );
                let direct_summary = JudgedRowSummary {
                    first_outcome: direct_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        tally,
                        &counts_before_the_state,
                    ),
                };
                add_the_comparison_of_two_judged_tables(
                    tally,
                    (Counter::TablesDiffer, Counter::RowsDiffer),
                    (&direct_table, &direct_summary),
                    (&scanned_table, &scanned_summary),
                    &|| {
                        self.describe_the_state_with_its_first_outcome(
                            &direct_judgement.first_outcome,
                            &direct_judgement.expectations,
                        )
                    },
                );
                if !direct_table.is_empty() {
                    tally.add(Counter::DirectTableHoldsARecord);
                }
                direct_judgement
            }
        };
        finish_probing_a_state(probe_decision, tally, &|| {
            self.describe_the_state_with_its_first_outcome(
                &judgement.first_outcome,
                &judgement.expectations,
            )
        });
    }

    /// [`Self::judge_the_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
    #[allow(
        clippy::too_many_arguments,
        reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
    )]
    fn judge_the_state_on_a_table(
        &self,
        records: &RecordTable,
        disk: &MemoryDisk<'_>,
        truths: &[RecordTruthOnDisk],
        holds_a_record_rejected_by_parse: bool,
        implementation: Implementation,
        is_the_designated_state: bool,
        tally: &mut Tally,
    ) -> AdversarialJudgement {
        let in_flight_limit = self.domain.ring().local_in_flight_limit();
        let view = SpecificationView {
            chosen_root: &self.chosen_root,
            records,
            in_flight_limit,
            verify_named_units: true,
            truths,
        };
        let expectations =
            FollowReading::REGISTERED.map(|reading| specification_expectation(&view, reading));
        let today = || call_replay_journal(disk, &self.chosen_root, records, true);
        let first = match implementation {
            Implementation::TodayReplayJournal => today(),
            Implementation::Control(control) => match control {
                ControlKind::PanicsOnTheDesignatedState => {
                    if is_the_designated_state {
                        ReplayOutcome::Panicked
                    } else {
                        today()
                    }
                }
                ControlKind::SpecificationWithReading(reading) => outcome_of_the_expectation(
                    &self.chosen_root,
                    specification_expectation(&view, reading),
                ),
                ControlKind::RootFromARecordWithoutTheLastRecordFlag => {
                    let record_without_the_flag = records.values().find(|record| {
                        record.instance == self.chosen_root.instance
                            && record.checkpoint_txg > self.chosen_root.checkpoint_txg
                            && record.is_commit
                            && !record_carries_the_last_record_flag(record)
                    });
                    match record_without_the_flag {
                        Some(record) => ReplayOutcome::Returned(ReturnedReplay {
                            root: VersionFields::of_record(record)
                                .to_root_rebuilt_over(&self.chosen_root),
                            prefix_applied: 1,
                            verification_passed: 1,
                        }),
                        None => today(),
                    }
                }
                ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag => {
                    ReplayOutcome::Returned(ReturnedReplay {
                        root: self.chosen_root,
                        prefix_applied: 0,
                        verification_passed: 0,
                    })
                }
                ControlKind::ReportsOneRecordOverTheLimit => match today() {
                    ReplayOutcome::Returned(mut returned) => {
                        returned.prefix_applied = in_flight_limit + 1;
                        ReplayOutcome::Returned(returned)
                    }
                    ReplayOutcome::Panicked | ReplayOutcome::Failed(_) => {
                        ReplayOutcome::Returned(ReturnedReplay {
                            root: self.chosen_root,
                            prefix_applied: in_flight_limit + 1,
                            verification_passed: 0,
                        })
                    }
                },
                ControlKind::OffTimelineRootWheneverApplied
                | ControlKind::LandsOnePublishTooFarOverAnIncompletePublish
                | ControlKind::IgnoresTheInFlightLimit
                | ControlKind::AlwaysReturnsTheChosenRoot
                | ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk
                | ControlKind::IdealWithTheCumulativeAppliedSet
                | ControlKind::SecondCallMovesOn
                | ControlKind::ReportsNoFailureOnTheLargestTxg
                | ControlKind::ReportsOneVerificationWithoutVerifying
                | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
                | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
                | ControlKind::IdealFirstCallThenTodaysSecondCall
                | ControlKind::IdealWithAFieldOfTheLandedVersion(_)
                | ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied
                | ControlKind::IdealWritingNotWrittenWhenNothingIsApplied
                | ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied
                | ControlKind::IdealWritingOneUnmountMarkerWhenApplied
                | ControlKind::IdealCountingOnlyTheFirstDeviceCopies
                | ControlKind::ScanTableLosesItsLargestKeyRecord
                | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
            },
        };
        let describe_witness =
            || self.describe_the_state_with_its_first_outcome(&first, &expectations);

        tally.add(Counter::States);
        // I-8.12 成不成立（登记 r3 5.6）：对抗一族每个状态各数一次，不看实现的结局；只有 r3 档数。
        let table_keeps_the_txg_order = self
            .run_profile
            .splits_j6qn_by_the_txg_order()
            .then(|| record_table_keeps_the_txg_order(records));
        match table_keeps_the_txg_order {
            Some(true) => tally.add(Counter::TablesKeepTheTxgOrder),
            Some(false) => tally.add(Counter::TablesBreakTheTxgOrder),
            None => {}
        }
        if holds_a_record_rejected_by_parse {
            tally.add(Counter::StatesHoldingARecordRejectedByParse);
        }
        if expectations[0] == SpecificationExpectation::FailsWithoutApplying {
            tally.add(Counter::SpecificationExpectsFailure);
        }
        let above_water_record_count = records
            .values()
            .filter(|record| {
                record.instance == self.chosen_root.instance
                    && record.checkpoint_txg > self.chosen_root.checkpoint_txg
            })
            .count();
        tally.above_water_maximum = tally.above_water_maximum.max(above_water_record_count);
        if above_water_record_count > in_flight_limit {
            tally.add(Counter::AboveWaterOverLimit);
        }
        if above_water_record_count == in_flight_limit {
            tally.add(Counter::AboveWaterAtLimit);
        }

        // AS-全、AS-连甲、AS-连乙：实现的结局与 S 在那种读法下的期望不同即一条违例。
        let specification_violation_counters = [
            Counter::AsWholeTableViolations,
            Counter::AsGapContinuesViolations,
            Counter::AsGapStopsViolations,
        ];
        for (expectation, violation_counter) in expectations
            .into_iter()
            .zip(specification_violation_counters)
        {
            if !outcome_matches_the_expectation(&first, expectation) {
                tally.add_violation(violation_counter, &describe_witness);
            }
        }
        // A-互：三种读法两两之间期望不同的态数。
        if expectations[0] != expectations[1] {
            tally.add_violation(Counter::MutualWholeTableAndGapContinues, &describe_witness);
        }
        if expectations[0] != expectations[2] {
            tally.add_violation(Counter::MutualWholeTableAndGapStops, &describe_witness);
        }
        if expectations[1] != expectations[2] {
            tally.add_violation(Counter::MutualGapContinuesAndGapStops, &describe_witness);
        }

        match &first {
            ReplayOutcome::Panicked => tally.add_violation(Counter::Panicked, &describe_witness),
            ReplayOutcome::Failed(_kind) => {
                tally.add(Counter::Failed);
                tally.add(Counter::FixedPointFirstCallFailed);
            }
            ReplayOutcome::Returned(returned) => {
                let returned_fields = VersionFields::of_root(&returned.root);
                let chosen_fields = VersionFields::of_root(&self.chosen_root);
                if returned_fields != chosen_fields {
                    tally.add(Counter::AppliedAtLeastOne);
                }
                // A-结构（M2 字面，不靠 S）：返回的根要么六个字段等于所选根，要么它的新根段四个字段与 txg 等于表里某一条
                // 「同实例、txg 大于所选根、带末条标志、带提交标记」的记录的。
                let comes_from_a_committed_last_record = records.values().any(|record| {
                    record.instance == self.chosen_root.instance
                        && record.checkpoint_txg > self.chosen_root.checkpoint_txg
                        && record.is_commit
                        && record_carries_the_last_record_flag(record)
                        && VersionFields::of_record(record).checkpoint_txg
                            == returned_fields.checkpoint_txg
                        && VersionFields::of_record(record).segment == returned_fields.segment
                });
                if returned_fields != chosen_fields && !comes_from_a_committed_last_record {
                    tally.add_violation(Counter::StructureViolations, &describe_witness);
                }
                // A-限（I-8.3）：报告的 prefix_applied 不超过在飞上限。
                if returned.prefix_applied > in_flight_limit {
                    tally.add_violation(Counter::LimitViolations, &describe_witness);
                }
                judge_the_fixed_point(
                    implementation,
                    self.run_profile,
                    returned,
                    None,
                    above_water_record_count <= in_flight_limit,
                    table_keeps_the_txg_order,
                    records,
                    disk,
                    true,
                    &describe_witness,
                    tally,
                );
                if self.run_profile.judges_copied_fields() {
                    judge_the_fields_beyond_the_projection(
                        &self.chosen_root,
                        &returned.root,
                        &describe_witness,
                        tally,
                    );
                }
                if self.run_profile.judges_new_root_segment_fields() {
                    judge_the_fields_beyond_the_projection_against_the_generated_records(
                        &self.chosen_root,
                        &returned.root,
                        self.record_truth,
                        None,
                        &describe_witness,
                        tally,
                    );
                }
            }
        }
        AdversarialJudgement {
            first_outcome: first,
            expectations,
        }
    }
}

/// 一个对抗状态判完的结果：第一次调用的结局与 S 三种读法的期望（逐态行的写法要）。
struct AdversarialJudgement {
    first_outcome: ReplayOutcome,
    expectations: [SpecificationExpectation; 3],
}

/// 对抗域的一个工作单元：锚点在不在、第一个槽取哪一种，其余的槽在单元里取遍。
#[derive(Clone, Copy, Debug)]
struct AdversarialWorkUnit {
    anchor_is_on_disk: bool,
    first_choice: AdversarialSlotChoice,
}

fn tally_of_an_adversarial_work_unit(
    domain: Domain,
    slot_count: usize,
    unit: &AdversarialWorkUnit,
    unit_index: usize,
    implementation: Implementation,
    run_profile: RunProfile,
) -> Tally {
    let alphabet = adversarial_record_alphabet();
    let choices = adversarial_slot_choices(domain, alphabet.len());
    let chosen_root = adversarial_chosen_root(run_profile);
    let record_truth = adversarial_record_truth(slot_count, &alphabet, &chosen_root, run_profile);
    let mut enumeration = AdversarialEnumeration {
        domain,
        slot_count,
        alphabet: &alphabet,
        choices: &choices,
        chosen_root,
        implementation,
        run_profile,
        record_truth: &record_truth,
        slots: Vec::new(),
        choice_trail: Vec::new(),
        is_the_designated_state: unit_index == 0 && slot_count == 1,
    };
    let anchor_slot = unit
        .anchor_is_on_disk
        .then(|| enumeration.generate_the_anchor_slot());
    enumeration.slots.push(anchor_slot);
    let mut tally = Tally::new();
    enumeration.put_the_choice_and_descend(1, unit.first_choice, &mut tally);
    tally
}

/// 一个域在一个 N（或 Nc、n）上数出来的全部量。
pub(super) fn tally_of_a_domain_size(
    domain: Domain,
    size: usize,
    implementation: Implementation,
    run_profile: RunProfile,
    thread_count: usize,
) -> Tally {
    match domain.family() {
        Family::LegalHistory => {
            let units = legal_work_units(domain, size, run_profile);
            match domain {
                Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => {
                    tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                        tally_of_a_monotone_history(unit, unit_index, implementation)
                    })
                }
                Domain::LegalBigRing
                | Domain::LegalSmallRingSix
                | Domain::LegalSmallRingNine
                | Domain::NamedUnitsVerified
                | Domain::NamedUnitsNotVerified
                | Domain::AdversarialBigRing
                | Domain::AdversarialSmallRingSix
                | Domain::AdversarialBackChain
                | Domain::CrossInstance
                | Domain::TxgOneBelowTheLargest
                | Domain::TxgAtTheLargest
                | Domain::Residue => {
                    tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                        tally_of_a_legal_history(domain, unit, unit_index, implementation)
                    })
                }
                Domain::MirroredBigRing
                | Domain::MirroredSmallRingSix
                | Domain::MirroredMonotoneBigRing => {
                    unreachable!("多盘域走 Family::MultiDisk 那一支")
                }
            }
        }
        Family::MultiDisk => {
            let units = legal_work_units(domain, size, run_profile);
            if domain == Domain::MirroredMonotoneBigRing {
                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                    tally_of_a_mirrored_monotone_history(unit, unit_index, implementation)
                })
            } else {
                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                    tally_of_a_mirrored_history(unit, unit_index, implementation)
                })
            }
        }
        Family::Adversarial => {
            let choices = adversarial_slot_choices(domain, adversarial_record_alphabet().len());
            let mut units = Vec::new();
            for anchor_is_on_disk in [false, true] {
                for first_choice in &choices {
                    units.push(AdversarialWorkUnit {
                        anchor_is_on_disk,
                        first_choice: *first_choice,
                    });
                }
            }
            tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                tally_of_an_adversarial_work_unit(
                    domain,
                    size,
                    unit,
                    unit_index,
                    implementation,
                    run_profile,
                )
            })
        }
    }
}

#[cfg(test)]
mod root_choice_fault_tests;
