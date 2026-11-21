//! `histories_of_the_families` 的单测。
use super::*;

/// A7：F3a P 臂 16 105 段、V 臂 1 464 段、P 臂步数 143 335（登记第七节那一行的命令输出 `16105 1464 143335`）。
#[test]
fn directed_exhaustive_scale_matches_the_registered_counts() {
    assert_eq!(
        Family::DirectedExhaustive.registered_history_count(Arm::Product),
        16_105
    );
    assert_eq!(
        Family::DirectedExhaustive.registered_history_count(Arm::RollbackTakesTheTargetsWatermark),
        1_464
    );
    let product_steps: u64 = (0..16_105_u64)
        .map(|index| u64::try_from(directed_exhaustive_operations(index).len()).expect("几步"))
        .sum();
    assert_eq!(product_steps, 143_335);
    assert_eq!(directed_exhaustive_middle_letters(0), Vec::<u64>::new());
    assert_eq!(directed_exhaustive_middle_letters(1), vec![0]);
    assert_eq!(directed_exhaustive_middle_letters(11), vec![10]);
    assert_eq!(directed_exhaustive_middle_letters(12), vec![0, 0]);
    assert_eq!(
        directed_exhaustive_middle_letters(16_104),
        vec![10, 10, 10, 10]
    );
}

#[test]
fn random_with_rollbacks_weight_tables_each_sum_to_one_hundred_and_generate_sixty_steps() {
    for table in [
        &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITHOUT_FILE[..],
        &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITH_FILE[..],
        &RANDOM_WITH_ROLLBACKS_AFTER_THE_FILE_WAS_DELETED[..],
        &RANDOM_WITH_ROLLBACKS_WITH_SESSION_CLOSED[..],
    ] {
        assert_eq!(table.iter().map(|(_, weight)| weight).sum::<u64>(), 100);
    }
    let item = history_item(Family::RandomWithRollbacks, 0);
    assert_eq!(
        item.history.operations.len(),
        RANDOM_WITH_ROLLBACKS_OPERATIONS_PER_HISTORY
    );
    assert_eq!(
        item.history.starting_point,
        HistoryStartingPoint::AfterFirstFile
    );
}
