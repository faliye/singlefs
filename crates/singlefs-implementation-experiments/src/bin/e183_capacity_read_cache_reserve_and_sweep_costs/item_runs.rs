use super::*;

// ============================================================================================
// 各项的运行：constants、positive-control-heap、anchors、h4-small-*、h4-default-*。
// ============================================================================================

pub(super) const DEFAULT_DEVICE_BYTES: u64 = 4 * GIBIBYTE;
pub(super) const SLOTS_PER_REGION_SAMPLING_POINTS: [u64; 3] = [4, 8, 16];
/// 几何敏感性的方向相反的取样点（登记 8.2）：S = 8 上环长 4 MiB。
pub(super) const ALTERNATIVE_JOURNAL_RING_BYTES: u64 = 4 * MEBIBYTE;
pub(super) const SMALL_DISK_JOURNAL_RING_BYTES: u64 = MEBIBYTE;

pub(super) fn run_constants_item(emitter: &mut Emitter) -> u64 {
    let comparisons = constants_against_the_implementation();
    let mut mismatches = 0;
    for comparison in &comparisons {
        emitter.emit(&format!(
            "name=constant_compare constant={} local={} implementation={} equal={}",
            comparison.name,
            comparison.local,
            comparison.implementation,
            comparison.is_equal(),
        ));
        mismatches += u64::from(!comparison.is_equal());
    }
    emitter.emit(&format!(
        "name=verdict item=constants constants_compared={} constants_mismatched={mismatches}",
        comparisons.len()
    ));
    mismatches
}

pub(super) fn run_positive_control_heap_item(emitter: &mut Emitter) -> u64 {
    let control = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
    emitter.emit(&format!(
        "name=positive_control_heap peak_over_baseline_bytes={} lower_bytes={} upper_bytes={} holds={}",
        control.peak_over_baseline_bytes,
        HEAP_POSITIVE_CONTROL_BYTES,
        HEAP_POSITIVE_CONTROL_BYTES + HEAP_POSITIVE_CONTROL_SLACK_BYTES,
        control.holds,
    ));
    emitter.emit(&format!(
        "name=verdict item=positive-control-heap positive_control_heap_holds={}",
        control.holds
    ));
    u64::from(!control.holds)
}

pub(super) fn run_anchors_item(emitter: &mut Emitter) -> u64 {
    let mut mismatches = 0;
    for anchor in &REGISTERED_RING_WINDOW_ANCHORS {
        let local = ring_window_by_the_clauses(
            anchor.device_bytes,
            anchor.journal_ring_bytes,
            anchor.slots_per_region,
            0,
        );
        let reproduced = anchor.is_reproduced_by(&local);
        mismatches += u64::from(!reproduced);
        emitter.emit(&format!(
            "name=anchor_geometry device_bytes={} journal_ring_bytes={} slots_per_region={} unit_area_first_slot={} unit_area_slots={} root_level={} positions_at_each_level={:?} deletion_publish_slots={} ring_window_slots={} ring_window_bytes={} matches_the_registered_script_output={reproduced}",
            anchor.device_bytes,
            anchor.journal_ring_bytes,
            anchor.slots_per_region,
            local.unit_area_first_slot,
            local.unit_area_slots,
            local.root_level,
            local.positions_at_each_level,
            local.deletion_publish_slots,
            local.ring_window_slots,
            local.ring_window_bytes(),
        ));
    }
    for (rows, registered_bytes) in REGISTERED_CHAIN_REWRITE_BYTES {
        let local_bytes = chain_rewrite_bytes_of_the_switch_reserve(rows);
        mismatches += u64::from(local_bytes != registered_bytes);
        emitter.emit(&format!(
            "name=anchor_chain_rewrite instance_rows={rows} local_bytes={local_bytes} registered_bytes={registered_bytes} matches_the_registered_script_output={}",
            local_bytes == registered_bytes,
        ));
    }
    let default_ring_start_slot = JOURNAL_RING_START_SLOT + JOURNAL_RING_DEFAULT_BYTES / SLOT_BYTES;
    emitter.emit(&format!(
        "name=anchor_default_ring_unit_area_start local_slot={default_ring_start_slot} registered_slot={UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING} equal={}",
        default_ring_start_slot == UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING,
    ));
    mismatches += u64::from(default_ring_start_slot != UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING);
    emitter.emit(&format!(
        "name=verdict item=anchors anchor_mismatches={mismatches}"
    ));
    mismatches
}

/// PC-准入（登记 5.3）：在 C_mkfs 上用只供测试的开关挂载得上；在 C_挂空 − 1 MiB 上产品路径报 `SpaceAdmissionRefusedBeforeAcquisition`。
#[derive(Clone, Debug, PartialEq, Eq)]
struct AdmissionPositiveControl {
    mount_with_the_test_only_switch_at_the_make_filesystem_minimum: String,
    one_step_below_the_empty_mount_minimum_mebibytes: Option<u64>,
    empty_mount_one_step_below: String,
    file_mount_one_step_below_the_file_minimum: String,
    pub(super) holds: bool,
}

const REFUSAL_BEFORE_ACQUISITION_VARIANT: &str = "SpaceAdmissionRefusedBeforeAcquisition";

/// 判 PC-准入的纯函数：开关挂载得上，且低一步的那次尝试里空池挂载败在准入拒绝上。
pub(super) fn admission_positive_control_holds(
    switch_mount_succeeded: bool,
    empty_mount_one_step_below: &StageOutcome<MountReport>,
) -> bool {
    switch_mount_succeeded
        && matches!(empty_mount_one_step_below, StageOutcome::Failed(variant)
            if variant == REFUSAL_BEFORE_ACQUISITION_VARIANT)
}

fn admission_positive_control(
    trials: &[Trial],
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> AdmissionPositiveControl {
    let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
    let make_filesystem_minimum = minimum_device_bytes(&flags, Milestone::MakeFilesystemSucceeds);
    let empty_mount_minimum = minimum_device_bytes(&flags, Milestone::EmptyMountAdmitted);
    let file_mount_minimum = minimum_device_bytes(&flags, Milestone::FileMountAdmitted);
    let switch_mount = match make_filesystem_minimum {
        None => (false, "not_run".to_string()),
        Some(device_bytes) => {
            let parameters = make_filesystem_parameters(journal_ring_bytes, slots_per_region);
            let mut pool = counted_pool(device_bytes, journal_ring_bytes, slots_per_region);
            let mounted = guarded(|| {
                make_filesystem(&parameters, &mut pool)
                    .map_err(|error| variant_name_of(&format!("{error:?}")))?;
                mount_writable_with_space_admission(
                    &parameters,
                    &mut pool,
                    SpaceAdmission::SkippedByTheTestOnlySwitch,
                )
                .map(|mounted| AdmissionClass::of_the_mount(&mounted.output.space_admission))
                .map_err(|error| variant_name_of(&format!("{error:?}")))
            });
            match mounted {
                Ok(class) => (true, format!("ok:{}", class.name())),
                Err(reason) => (false, format!("failed:{reason}")),
            }
        }
    };
    let trial_one_step_below = |device_bytes: Option<u64>| {
        device_bytes
            .and_then(|bytes| bytes.checked_sub(SEARCH_STEP_BYTES))
            .and_then(|below| trials.iter().find(|trial| trial.device_bytes == below))
    };
    let below_empty = trial_one_step_below(empty_mount_minimum);
    let empty_mount_one_step_below =
        below_empty.map_or(StageOutcome::NotReached, |trial| trial.empty_mount.clone());
    let file_mount_one_step_below_text = trial_one_step_below(file_mount_minimum)
        .map_or("not_run".to_string(), |trial| mount_text(&trial.file_mount));
    AdmissionPositiveControl {
        mount_with_the_test_only_switch_at_the_make_filesystem_minimum: switch_mount.1,
        one_step_below_the_empty_mount_minimum_mebibytes: below_empty
            .map(|trial| trial.device_bytes / MEBIBYTE),
        empty_mount_one_step_below: mount_text(&empty_mount_one_step_below),
        file_mount_one_step_below_the_file_minimum: file_mount_one_step_below_text,
        holds: admission_positive_control_holds(switch_mount.0, &empty_mount_one_step_below),
    }
}

fn trial_at(trials: &[Trial], device_bytes: Option<u64>) -> Option<&Trial> {
    device_bytes.and_then(|bytes| trials.iter().find(|trial| trial.device_bytes == bytes))
}

fn ratio_text(numerator: Option<u64>, denominator: Option<u64>) -> String {
    match (numerator, denominator) {
        (Some(top), Some(bottom)) if bottom > 0 => {
            format!("{}permille", u128::from(top) * 1000 / u128::from(bottom))
        }
        (_, _) => "none".to_string(),
    }
}

pub(super) fn run_small_disk_item(
    emitter: &mut Emitter,
    item: &str,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> ItemOutcome {
    emitter.emit(&format!(
        "name=h4_config item={item} slots_per_region={slots_per_region} journal_ring_bytes={journal_ring_bytes} start_device_mebibytes={} step_mebibytes={} window_mebibytes={} ceiling_mebibytes={}",
        SEARCH_START_DEVICE_BYTES / MEBIBYTE,
        SEARCH_STEP_BYTES / MEBIBYTE,
        MONOTONICITY_WINDOW_BYTES / MEBIBYTE,
        SEARCH_CEILING_DEVICE_BYTES / MEBIBYTE,
    ));
    let trials = search_small_disks(journal_ring_bytes, slots_per_region);
    for trial in &trials {
        emit_trial(emitter, item, trial);
    }
    let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
    let minimum = |milestone| minimum_device_bytes(&flags, milestone);
    emitter.emit(&format!(
        "name=h4_minimum item={item} make_filesystem_mebibytes={} empty_mount_mebibytes={} file_mount_mebibytes={} empty_mount_still_short_mebibytes={} file_mount_still_short_mebibytes={} file_mount_over_make_filesystem={} trials={} panicked_trials={}",
        optional_mebibytes_text(minimum(Milestone::MakeFilesystemSucceeds)),
        optional_mebibytes_text(minimum(Milestone::EmptyMountAdmitted)),
        optional_mebibytes_text(minimum(Milestone::FileMountAdmitted)),
        optional_mebibytes_text(minimum(Milestone::EmptyMountStillShort)),
        optional_mebibytes_text(minimum(Milestone::FileMountStillShort)),
        ratio_text(
            minimum(Milestone::FileMountAdmitted),
            minimum(Milestone::MakeFilesystemSucceeds)
        ),
        trials.len(),
        panicked_stage_count(&trials),
    ));
    let mut every_milestone_monotone = true;
    for milestone in Milestone::THREE_MINIMUM_DISKS {
        let monotonicity = monotonicity_of(&flags, milestone);
        every_milestone_monotone &= monotonicity.is_monotone();
        emitter.emit(&format!(
            "name=h4_monotonicity item={item} milestone={} minimum_mebibytes={} failures_in_the_window_mebibytes={} never_tried_mebibytes={} monotone={}",
            milestone.name(),
            optional_mebibytes_text(monotonicity.minimum_device_bytes),
            list_of_mebibytes_text(&monotonicity.failures_in_the_window),
            list_of_mebibytes_text(&monotonicity.window_points_never_tried),
            monotonicity.is_monotone(),
        ));
    }
    let control = admission_positive_control(&trials, journal_ring_bytes, slots_per_region);
    emitter.emit(&format!(
        "name=positive_control_admission item={item} mount_with_the_test_only_switch_at_the_make_filesystem_minimum={} one_step_below_the_empty_mount_minimum_mebibytes={} empty_mount_one_step_below={} file_mount_one_step_below_the_file_minimum={} holds={}",
        control.mount_with_the_test_only_switch_at_the_make_filesystem_minimum,
        optional_number_text(control.one_step_below_the_empty_mount_minimum_mebibytes),
        control.empty_mount_one_step_below,
        control.file_mount_one_step_below_the_file_minimum,
        control.holds,
    ));
    let (dominance_points, reading_checks_hold) = emit_states_of_a_trial(
        emitter,
        item,
        journal_ring_bytes,
        slots_per_region,
        [
            StateToRead {
                state: StateKind::Empty,
                trial: trial_at(&trials, minimum(Milestone::EmptyMountAdmitted)),
            },
            StateToRead {
                state: StateKind::File,
                trial: trial_at(&trials, minimum(Milestone::FileMountAdmitted)),
            },
        ],
    );
    emitter.emit(&format!(
        "name=verdict item={item} positive_control_admission_holds={} make_filesystem_minimum_found={} empty_mount_minimum_found={} file_mount_minimum_found={} every_minimum_disk_monotone_over_the_window={every_milestone_monotone} reading_checks_hold={reading_checks_hold} panicked_trials={}",
        control.holds,
        minimum(Milestone::MakeFilesystemSucceeds).is_some(),
        minimum(Milestone::EmptyMountAdmitted).is_some(),
        minimum(Milestone::FileMountAdmitted).is_some(),
        panicked_stage_count(&trials),
    ));
    ItemOutcome {
        dominance_points,
        invalid_device_conditions: 0,
    }
}

/// 尝试里 panic 掉的步数（`guarded` 把 panic 记成 `failed:panicked:…`）。
fn panicked_stage_count(trials: &[Trial]) -> usize {
    let is_panic = |reason: &String| reason.starts_with("panicked:");
    trials
        .iter()
        .map(|trial| {
            let failed_texts = [
                matches!(&trial.make_filesystem, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.empty_mount, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.first_file, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.unmount, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.file_mount, StageOutcome::Failed(reason) if is_panic(reason)),
            ];
            failed_texts.iter().filter(|failed| **failed).count()
        })
        .sum()
}

pub(super) fn run_default_disk_item(
    emitter: &mut Emitter,
    item: &str,
    slots_per_region: u64,
) -> ItemOutcome {
    emitter.emit(&format!(
        "name=h4_config item={item} slots_per_region={slots_per_region} journal_ring_bytes={JOURNAL_RING_DEFAULT_BYTES} device_mebibytes={}",
        DEFAULT_DEVICE_BYTES / MEBIBYTE,
    ));
    let trial = run_trial(
        DEFAULT_DEVICE_BYTES,
        JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
    );
    emit_trial(emitter, item, &trial);
    let states_hold = trial.flags().empty_mount_admitted && trial.flags().file_mount_admitted;
    let (dominance_points, reading_checks_hold) = emit_states_of_a_trial(
        emitter,
        item,
        JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
        [
            StateToRead {
                state: StateKind::Empty,
                trial: Some(&trial),
            },
            StateToRead {
                state: StateKind::File,
                trial: Some(&trial),
            },
        ],
    );
    emitter.emit(&format!(
        "name=verdict item={item} both_states_admitted={states_hold} reading_checks_hold={reading_checks_hold} panicked_trials={}",
        panicked_stage_count(std::slice::from_ref(&trial)),
    ));
    ItemOutcome {
        dominance_points,
        invalid_device_conditions: 0,
    }
}

/// R4c 跨点（登记 8.2）：同一状态同一入口上，各取样点的主导项是不是同一项；不同记「不稳定」。
pub(super) fn emit_dominance_across_points(
    emitter: &mut Emitter,
    group: &str,
    points: &[DominancePoint],
) -> bool {
    let mut every_group_stable = true;
    for state in ["empty", "file"] {
        for entry in ReadingEntry::BOTH {
            let selected: Vec<&DominancePoint> = points
                .iter()
                .filter(|point| point.state == state && point.entry == entry)
                .collect();
            if selected.is_empty() {
                continue;
            }
            let names: BTreeSet<&str> = selected
                .iter()
                .map(|point| match &point.dominance {
                    Dominance::Dominant { item, .. } => *item,
                    Dominance::NoItemAboveHalf { .. } => "no_item_above_half",
                })
                .collect();
            let stable = names.len() == 1;
            every_group_stable &= stable;
            emitter.emit(&format!(
                "name=h4_dominance_across_points group={group} state={state} entry={} points={} outcomes={} stable={stable}",
                entry.name(),
                selected.len(),
                selected
                    .iter()
                    .map(|point| format!("{}={}", point.item, point.dominance.verdict_text()))
                    .collect::<Vec<_>>()
                    .join(";"),
            ));
        }
    }
    every_group_stable
}

/// 登记第八节「判别力自证」③：把 R4c 的门槛挪到真产物每份读数最大一项份额的另一侧，判定必须转；一份没转，这个判定的几何敏感性作废（V36）。
pub(super) fn emit_discrimination_self_proof(
    emitter: &mut Emitter,
    points: &[DominancePoint],
) -> bool {
    let flipped = points
        .iter()
        .filter(|point| dominance_flips_when_the_threshold_is_moved(&point.reserve_items))
        .count();
    let holds = !points.is_empty() && flipped == points.len();
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=r4c_dominance readings={} flipped={flipped} holds={holds}",
        points.len(),
    ));
    holds
}
