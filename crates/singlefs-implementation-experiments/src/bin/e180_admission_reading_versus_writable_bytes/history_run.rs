//! 一族历史的跑法：逐步改动、观测与判定。

use super::*;

/// 一族历史的运行态：池、产物行缓冲、累计。
pub(super) struct HistoryRun {
    pub(super) pool: Pool,
    pub(super) family: &'static str,
    with_probes: bool,
    /// 只对填满族做的「捷径与整写同判」（登记 5.1 探测捷径）。
    checks_the_shortcut_against_full_writes: bool,
    pub(super) probe_threads: usize,
    pub(super) model: JournalModel,
    pub(super) state_index: u64,
    /// 每隔几个状态打一行 `state`（长挂载族两千个状态只抽着打，累计照样逐状态数）。
    pub(super) emit_state_line_every: u64,
    pub(super) lines: LineBuffer,
    pub(super) tally: FamilyTally,
}

impl HistoryRun {
    pub(super) fn new(
        pool: Pool,
        family: &'static str,
        with_probes: bool,
        probe_threads: usize,
    ) -> Self {
        let device_count = pool.devices.len();
        HistoryRun {
            pool,
            family,
            with_probes,
            checks_the_shortcut_against_full_writes: family == "fill",
            probe_threads,
            model: JournalModel::NOT_LEARNED_YET,
            state_index: 0,
            emit_state_line_every: 1,
            lines: LineBuffer::default(),
            tally: FamilyTally::new(device_count),
        }
    }

    pub(super) fn state_name(&self, index: u64) -> String {
        format!("{}/{}/{index}", self.pool.geometry.name, self.family)
    }

    fn measure_writable_units(&mut self, reading: &StateReading) -> WritableUnits {
        let stripped = strip_bytes_the_probe_does_not_need(&mut self.pool.session);
        let (writable, probes_run) = {
            let prober = StateProber {
                pool: &self.pool,
                probe_threads: self.probe_threads,
                probes_run: AtomicU64::new(0),
            };
            let writable = find_writable_units(
                &prober,
                upper_bound_units_by_the_space_conditions(&self.pool, reading),
                &mut self.model,
                &LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
                    units_asked_for_by_the_arms: reading.units_asked_for_by_every_arm(),
                },
            );
            (writable, prober.probes_run.load(Ordering::Relaxed))
        };
        restore_stripped_bytes(&mut self.pool.session, stripped);
        self.tally.probes_run += probes_run;
        writable
    }

    /// 捷径与整写同判：T 与 T + 1 各整写一遍（不中途报错、带字节的整份拷贝），判法逐项相同才算。
    fn check_the_shortcut_against_full_writes(&mut self, units: u64, state_name: &str) {
        for tried in [units, units + 1] {
            if tried == 0 {
                continue;
            }
            let expected_by_the_shortcut = if tried == units {
                ProbeVerdict::Completed
            } else {
                ProbeVerdict::RefusedForSpace
            };
            let full = run_probe(
                &self.pool.devices,
                clone_session(&self.pool.session),
                tried,
                DataWritePolicy::AllowDataWrites,
            );
            self.tally.shortcut_pairs_checked += 1;
            if full != expected_by_the_shortcut {
                self.tally.shortcut_differences.push(format!(
                    "{state_name}:units={tried}:shortcut_said={}:full_write_said={}",
                    if tried == units {
                        "admitted"
                    } else {
                        "refused_for_space"
                    },
                    short_debug(&full)
                ));
            }
        }
    }
}

impl HistoryRun {
    /// 在当前状态上量一遍：读数、残差、段内关着的空槽、（要探测的族）T 与全部臂的判定；打一行 `state`，累计进族的计数。
    pub(super) fn observe(&mut self, kind: &str, outcome: &str) {
        let index = self.state_index;
        self.state_index += 1;
        let state_name = self.state_name(index);
        let reading = read_state(&self.pool);
        self.tally.states += 1;
        if !reading.records_agree_with_the_disk {
            self.tally.records_disagree_with_the_disk_states += 1;
        }
        if !reading.decomposition_holds {
            self.tally.decomposition_failure_states += 1;
        }
        if !reading.available_is_whole_slots {
            self.tally.not_whole_slot_available_states += 1;
        }
        if reading.lag_unattributed_slots > 0 {
            self.tally.lag_unattributed_states += 1;
        }
        let mut line = ProductLine::new("state")
            .field("geometry", self.pool.geometry.name)
            .field("family", self.family)
            .field("index", index)
            .field("kind", kind)
            .field("outcome", outcome)
            .field("data_units", reading.data_units)
            .field("checkpoint_cost_blocks", reading.checkpoint_cost_blocks)
            .field("lag_slots", reading.lag_slots)
            .field("lag_unattributed_slots", reading.lag_unattributed_slots)
            .field("oldest_valid_root_txg", reading.oldest_valid_root_txg)
            .field(
                "fourth_newest_nonempty_valid_root_txg",
                reading
                    .fourth_newest_nonempty_valid_root_txg
                    .map_or("none".to_string(), |txg| txg.to_string()),
            )
            .field("cluster_segments", reading.cluster_segment_count)
            .field(
                "records_agree_with_the_disk",
                reading.records_agree_with_the_disk,
            )
            .field("decomposition_holds", reading.decomposition_holds)
            .field(
                "abandoned_root_ring_slots",
                self.pool
                    .session
                    .allocator
                    .root_ring_occupancy()
                    .map_or(0, |occupancy| {
                        occupancy
                            .ring_slots_recorded_as_holding_an_abandoned_root()
                            .len()
                    }),
            );
        line = self.record_residuals_and_trajectories(line, &reading, &state_name);
        if self.with_probes {
            let writable = self.measure_writable_units(&reading);
            line = self.record_arms(line, &reading, &writable, &state_name);
        }
        if index.is_multiple_of(self.emit_state_line_every) {
            self.lines.push(line.finish());
        }
    }

    fn record_residuals_and_trajectories(
        &mut self,
        mut line: ProductLine,
        reading: &StateReading,
        state_name: &str,
    ) -> ProductLine {
        for (device_index, device) in reading.devices.iter().enumerate() {
            let available_plus_common = device.available_plus_common_slots();
            let unblocked = i128::from(device.unblocked_slots);
            let overlap = i128::from(device.overlap_slots_allocated_and_isolated);
            // 装置自记的扣住账（登记第二节「扣住位」）：第一段的历史里抬 F 那一串都整串发成、整串放开，没有留着的扣住位，恒 0。
            let held_ledger = 0i128;
            let residuals = residuals_of(available_plus_common, unblocked, overlap, held_ledger);
            let residual_with_both_terms =
                residuals.rewritten_check_residual_with_the_held_and_overlap_terms;
            let residual_without_the_overlap_term =
                residuals.rewritten_check_residual_without_the_overlap_term;
            let original_check_residual = residuals.original_formula_residual;
            let residual_tally = &mut self.tally.residuals[device_index];
            residual_tally.states += 1;
            if residual_with_both_terms != 0 {
                residual_tally.rewritten_check_residual_nonzero_states += 1;
                self.tally.stop_clause_held_ledger_mismatch_states += 1;
                if residual_tally
                    .first_state_with_a_nonzero_rewritten_check_residual
                    .is_empty()
                {
                    residual_tally.first_state_with_a_nonzero_rewritten_check_residual =
                        state_name.to_string();
                }
                residual_tally.maximum_absolute_rewritten_check_residual = residual_tally
                    .maximum_absolute_rewritten_check_residual
                    .max(residual_with_both_terms.abs());
            }
            if original_check_residual != 0 {
                residual_tally.original_formula_residual_nonzero_states += 1;
                if residual_tally
                    .first_state_with_a_nonzero_original_formula_residual
                    .is_empty()
                {
                    residual_tally.first_state_with_a_nonzero_original_formula_residual =
                        state_name.to_string();
                }
                residual_tally.maximum_absolute_original_formula_residual = residual_tally
                    .maximum_absolute_original_formula_residual
                    .max(original_check_residual.abs());
            }
            if overlap > 0 {
                residual_tally.states_with_overlap += 1;
                if residual_without_the_overlap_term == 0 {
                    residual_tally.control_without_overlap_term_stayed_zero += 1;
                }
            }
            let closed = i128::from(device.closed_free_slots_in_cluster_segments);
            let ring_window = i128::from(device.ring_window_slots);
            let all_common_slots = device
                .common_commitments_bytes
                .div_euclid(i128::from(SLOT_BYTES));
            self.tally.closed_minus_ring_window[device_index].record(closed - ring_window);
            self.tally.closed_alone[device_index].record(closed);
            self.tally.closed_minus_all_common_commitments[device_index]
                .record(closed - all_common_slots);
            let key = |name: &str| format!("{name}_{device_index}");
            line = line
                .field(&key("capacity_slots"), device.capacity_slots)
                .field(&key("live_slots"), device.live_slots)
                .field(
                    &key("live_slots_in_memory_account"),
                    device.live_slots_in_memory_account,
                )
                .field(
                    &key("available_slots"),
                    device.available_slots_rounded_down(),
                )
                .field(&key("allocated_slots"), device.allocated_slots)
                .field(&key("deferred_slots"), device.deferred_slots)
                .field(&key("isolated_slots"), device.isolated_slots)
                .field(&key("overlap_allocated_and_isolated_slots"), overlap)
                .field(&key("unblocked_slots"), unblocked)
                .field(&key("closed_free_slots_in_cluster_segments"), closed)
                .field(&key("ring_window_slots"), ring_window)
                .field(&key("common_commitments_slots"), all_common_slots)
                .field(&key("rewritten_check_residual"), residual_with_both_terms)
                .field(&key("original_formula_residual"), original_check_residual)
                .field(
                    &key("rewritten_check_residual_without_the_overlap_term"),
                    residual_without_the_overlap_term,
                );
        }
        line.field("held_ledger_slots", 0)
    }
}

impl HistoryRun {
    fn record_arms(
        &mut self,
        mut line: ProductLine,
        reading: &StateReading,
        writable: &WritableUnits,
        state_name: &str,
    ) -> ProductLine {
        match writable {
            WritableUnits::Exact {
                units,
                non_monotone_probes_seen,
            } => {
                self.tally.exact_states += 1;
                if *non_monotone_probes_seen {
                    self.tally.non_monotone_states += 1;
                }
                line = line
                    .field("writable_status", "exact")
                    .field("writable_units", units)
                    .field("writable_bytes", units * DATA_UNIT_PAYLOAD_BYTES)
                    .field("non_monotone_probes_seen", non_monotone_probes_seen);
            }
            WritableUnits::AtLeastBecauseOfTheJournalCap {
                lower_bound_units,
                lower_bound_was_tightened,
            } => {
                self.tally.censored_states += 1;
                line = line
                    .field("writable_status", "censored_by_the_journal_record_cap")
                    .field("writable_lower_bound_units", lower_bound_units)
                    .field(
                        "writable_lower_bound_bytes",
                        lower_bound_units * DATA_UNIT_PAYLOAD_BYTES,
                    )
                    .field("writable_lower_bound_tightened", lower_bound_was_tightened);
            }
            WritableUnits::Unreadable(reason) => {
                self.tally.unreadable_states += 1;
                line = line
                    .field("writable_status", "unreadable")
                    .field("reason", reason);
            }
        }
        for arm in all_arms() {
            let raw = reading.raw_bytes_of(arm);
            let reported = reported_bytes(raw);
            let judgement = judge(reported, writable);
            let tally = self.tally.arms.entry(arm).or_default();
            tally.count(&judgement, state_name);
            let (over_bytes, under_bytes) = match &judgement {
                Judgement::OverReports { over_bytes } => (*over_bytes, 0),
                Judgement::DoesNotOverReport { under_bytes } => (0, *under_bytes),
                Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
                | Judgement::Undetermined => (0, 0),
            };
            let label = arm.label();
            line = line
                .field(&format!("reported_bytes_{label}"), reported)
                .field(&format!("judgement_{label}"), judgement.label())
                .field(&format!("over_bytes_{label}"), over_bytes)
                .field(&format!("under_bytes_{label}"), under_bytes);
            if let WritableUnits::Exact { units, .. } = writable {
                self.check_the_controls_of_one_arm(arm, reported, *units, state_name);
            }
        }
        if let WritableUnits::Exact { units, .. } = writable {
            if self.checks_the_shortcut_against_full_writes {
                self.check_the_shortcut_against_full_writes(*units, state_name);
            }
        }
        line
    }

    /// 阳性对照（登记 5.1）：每条臂的「这条臂的值再顶到 T 之上一个载荷」必须判多报；恒多报的阳性对照在 T > 0 的状态上判多报且大于 2T（登记第七节 B4）；
    /// 恒报 0 的阴性对照从不判多报。只在 T 取得准的状态上跑。
    fn check_the_controls_of_one_arm(
        &mut self,
        arm: Arm,
        reported: u64,
        writable_units: u64,
        state_name: &str,
    ) {
        let writable = WritableUnits::Exact {
            units: writable_units,
            non_monotone_probes_seen: false,
        };
        let writable_bytes = writable_units * DATA_UNIT_PAYLOAD_BYTES;
        let pushed_above_the_writable_bytes =
            reported.max(writable_bytes) + DATA_UNIT_PAYLOAD_BYTES;
        if !matches!(
            judge(pushed_above_the_writable_bytes, &writable),
            Judgement::OverReports { .. }
        ) {
            self.tally.control_failures.push(format!(
                "{state_name}:arm={}:pushed_above_the_writable_bytes_not_judged_over",
                arm.label()
            ));
        }
        match arm {
            Arm::PositiveControlAlwaysOverReports if writable_units > 0 => {
                if !matches!(judge(reported, &writable), Judgement::OverReports { .. }) {
                    self.tally.control_failures.push(format!(
                        "{state_name}:always_over_reports_control_not_judged_over"
                    ));
                }
                if reported <= 2 * writable_bytes {
                    self.tally.control_failures.push(format!(
                        "{state_name}:always_over_reports_control_not_above_twice_the_writable_bytes:reported={reported}:writable_bytes={writable_bytes}"
                    ));
                }
            }
            Arm::NegativeControlReportsZero => {
                if matches!(judge(0, &writable), Judgement::OverReports { .. }) {
                    self.tally
                        .control_failures
                        .push(format!("{state_name}:zero_control_judged_over"));
                }
            }
            Arm::Candidate(_) | Arm::PositiveControlAlwaysOverReports => {}
        }
    }
}
