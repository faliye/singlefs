use super::*;

fn synthetic_report(class: AdmissionClass) -> MountReport {
    MountReport {
        class,
        readings: Vec::new(),
        instance_rows_by_the_allocator: 0,
        instance_rows_read_back_from_the_disk: None,
        mount_tallies: Vec::new(),
        available_minimum_bytes: 0,
    }
}

fn trial_whose_empty_mount_ends_as(device_mebibytes: u64, class: AdmissionClass) -> Trial {
    Trial {
        device_bytes: device_mebibytes * MEBIBYTE,
        make_filesystem: StageOutcome::Succeeded(()),
        empty_mount: StageOutcome::Succeeded(synthetic_report(class)),
        first_file: StageOutcome::NotReached,
        unmount: StageOutcome::NotReached,
        file_mount: StageOutcome::NotReached,
    }
}

#[test]
fn mount_that_stays_short_after_raising_the_floor_is_not_counted_as_admitted() {
    assert!(AdmissionClass::AdmittedBeforeAcquisition.counts_as_admitted());
    assert!(AdmissionClass::AdmittedAfterTheFloorRaises.counts_as_admitted());
    assert!(!AdmissionClass::StillShortAfterTheFloorRaises.counts_as_admitted());
    assert!(!AdmissionClass::NotJudgedByTheTestOnlySwitch.counts_as_admitted());
    let trials = [
        trial_whose_empty_mount_ends_as(20, AdmissionClass::StillShortAfterTheFloorRaises),
        trial_whose_empty_mount_ends_as(21, AdmissionClass::AdmittedAfterTheFloorRaises),
    ];
    let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
    assert_eq!(
        minimum_device_bytes(&flags, Milestone::EmptyMountAdmitted),
        Some(21 * MEBIBYTE)
    );
    assert_eq!(
        minimum_device_bytes(&flags, Milestone::EmptyMountStillShort),
        Some(20 * MEBIBYTE)
    );
}

#[test]
fn the_file_state_needs_every_step_of_the_chain_to_have_succeeded() {
    let mut trial = trial_whose_empty_mount_ends_as(30, AdmissionClass::AdmittedBeforeAcquisition);
    trial.file_mount =
        StageOutcome::Succeeded(synthetic_report(AdmissionClass::AdmittedBeforeAcquisition));
    assert!(
        !trial.flags().file_mount_admitted,
        "首个文件和卸载都没走到时不算成"
    );
    trial.first_file = StageOutcome::Succeeded(());
    trial.unmount = StageOutcome::Succeeded(());
    assert!(trial.flags().file_mount_admitted);
    trial.first_file = StageOutcome::Failed("PlacementRefused".to_string());
    assert!(!trial.flags().file_mount_admitted);
}

#[test]
fn admission_positive_control_needs_the_switch_mount_and_the_admission_refusal_variant() {
    let refused: StageOutcome<MountReport> =
        StageOutcome::Failed("SpaceAdmissionRefusedBeforeAcquisition".to_string());
    let other_failure: StageOutcome<MountReport> = StageOutcome::Failed("Publish".to_string());
    assert!(admission_positive_control_holds(true, &refused));
    assert!(!admission_positive_control_holds(false, &refused));
    assert!(!admission_positive_control_holds(true, &other_failure));
    assert!(!admission_positive_control_holds(
        true,
        &StageOutcome::NotReached
    ));
}

#[test]
fn roomy_disk_reaches_every_stage_and_every_reading_check_holds() {
    let trial = run_trial(256 * MEBIBYTE, MEBIBYTE, 8);
    let flags = trial.flags();
    assert!(flags.make_filesystem_succeeded, "{trial:?}");
    assert!(flags.empty_mount_admitted, "{:?}", trial.empty_mount);
    assert!(flags.file_mount_admitted, "{:?}", trial.file_mount);
    for outcome in [&trial.empty_mount, &trial.file_mount] {
        let StageOutcome::Succeeded(report) = outcome else {
            panic!("挂载没成：{outcome:?}");
        };
        assert_eq!(report.readings.len(), 2);
        for snapshot in &report.readings {
            let checks = checks_of_a_reading(snapshot, 256 * MEBIBYTE, MEBIBYTE, 8);
            assert!(
                checks.mount_time_commitment_by_the_clauses
                    && checks.pending_delete_is_zero
                    && checks.capacity_by_the_local_geometry,
                "{:?} {checks:?}",
                snapshot.entry
            );
        }
    }
}
