use super::*;

fn flags_at(device_mebibytes: u64, empty: bool, file: bool) -> TrialFlags {
    TrialFlags {
        device_bytes: device_mebibytes * MEBIBYTE,
        make_filesystem_succeeded: device_mebibytes >= 18,
        empty_mount_admitted: empty,
        file_mount_admitted: file,
        empty_mount_still_short: false,
        file_mount_still_short: false,
    }
}

#[test]
fn minimum_is_the_first_disk_where_the_milestone_holds_even_if_later_ones_fail() {
    let trials = vec![
        flags_at(17, false, false),
        flags_at(18, false, false),
        flags_at(19, true, false),
        flags_at(20, false, false),
        flags_at(21, true, true),
    ];
    assert_eq!(
        minimum_device_bytes(&trials, Milestone::MakeFilesystemSucceeds),
        Some(18 * MEBIBYTE)
    );
    assert_eq!(
        minimum_device_bytes(&trials, Milestone::EmptyMountAdmitted),
        Some(19 * MEBIBYTE)
    );
    assert_eq!(
        minimum_device_bytes(&trials, Milestone::FileMountAdmitted),
        Some(21 * MEBIBYTE)
    );
    assert_eq!(
        minimum_device_bytes(&trials, Milestone::EmptyMountStillShort),
        None
    );
}

#[test]
fn monotonicity_lists_every_failure_in_the_sixteen_mebibyte_window_and_nothing_beyond() {
    let mut trials: Vec<TrialFlags> = (17..=60)
        .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, mebibytes >= 21))
        .collect();
    let clean = monotonicity_of(&trials, Milestone::EmptyMountAdmitted);
    assert!(clean.is_monotone(), "{clean:?}");
    trials[(19 + 16 - 17) as usize].empty_mount_admitted = false;
    trials[(19 + 17 - 17) as usize].empty_mount_admitted = false;
    let broken = monotonicity_of(&trials, Milestone::EmptyMountAdmitted);
    assert_eq!(broken.failures_in_the_window, vec![35 * MEBIBYTE]);
    assert!(!broken.is_monotone());
    let short: Vec<TrialFlags> = (17..=30)
        .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, true))
        .collect();
    let unverified = monotonicity_of(&short, Milestone::EmptyMountAdmitted);
    assert_eq!(
        unverified.window_points_never_tried,
        vec![
            31 * MEBIBYTE,
            32 * MEBIBYTE,
            33 * MEBIBYTE,
            34 * MEBIBYTE,
            35 * MEBIBYTE
        ]
    );
    assert!(!unverified.is_monotone());
}

#[test]
fn search_stops_sixteen_mebibytes_after_the_largest_of_the_three_minima() {
    let up_to = |last_mebibytes: u64| -> Vec<TrialFlags> {
        (4..=last_mebibytes)
            .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, mebibytes >= 25))
            .collect()
    };
    assert!(!search_may_stop(&[]));
    assert!(!search_may_stop(&up_to(40)));
    assert!(search_may_stop(&up_to(41)));
    let never_found: Vec<TrialFlags> = (4..=100)
        .map(|mebibytes| flags_at(mebibytes, false, false))
        .collect();
    assert!(!search_may_stop(&never_found));
    let at_the_ceiling: Vec<TrialFlags> = vec![flags_at(1024, false, false)];
    assert!(search_may_stop(&at_the_ceiling));
}
