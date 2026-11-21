use super::*;

fn ledger_with_three_confirmations_and_a_rollback() -> RandomLedger {
    let mut ledger = RandomLedger::after_make_filesystem((0, 1));
    for (order, content) in [
        (1usize, b"one".to_vec()),
        (2, b"two".to_vec()),
        (3, b"three".to_vec()),
    ] {
        let number = ledger.new_content_number(Some(content));
        ledger.confirmed_numbers.insert(number);
        let root = (1, 3 + u64::try_from(order).expect("小"));
        ledger.carry(root, number);
        ledger.confirmations.push(RandomConfirmation {
            order,
            content_number: number,
            root,
            unit_bytes: BTreeMap::new(),
        });
    }
    ledger.carry((1, 8), 1);
    ledger.rollback_confirmations.push((5, 1, (1, 8)));
    ledger
}

#[test]
fn rollback_aware_reading_takes_the_later_of_the_two_kinds_of_confirmation() {
    let ledger = ledger_with_three_confirmations_and_a_rollback();
    assert_eq!(ledger.last_confirmed_number_before(6), 3);
    assert_eq!(ledger.last_confirmed_number_before_rollback_aware(6), 1);
    assert_eq!(ledger.last_confirmed_number_before_rollback_aware(5), 3);
    assert_eq!(ledger.last_confirmed_number_before(2), 1);
}

fn report_landing_on(root: TimelineRoot, content: Option<&[u8]>) -> RecoveryReport {
    let key = (InstanceGeneration(root.0), CheckpointTxg(root.1));
    RecoveryReport {
        outcome: match content {
            Some(bytes) => RecoveryOutcome::FileRead {
                root: key,
                content: bytes.to_vec(),
            },
            None => RecoveryOutcome::NoFile { root: key },
        },
        effective_root: Some(key),
        journal: singlefs_core::recovery::JournalScanReport::default(),
        mapping_fallbacks: 0,
        devices_without_a_mountable_system_configuration_slot: Vec::new(),
    }
}

#[test]
fn read_back_is_classified_by_the_content_number_of_the_landing_root() {
    let ledger = ledger_with_three_confirmations_and_a_rollback();
    let newest =
        classify_random_read_back(&report_landing_on((1, 6), Some(b"three")), &ledger, 3, 1);
    assert_eq!(newest.literal, "last_confirmed");
    assert_eq!(newest.rollback_aware, "older:3");
    let rolled_back =
        classify_random_read_back(&report_landing_on((1, 8), Some(b"one")), &ledger, 3, 1);
    assert_eq!(rolled_back.literal, "older:1");
    assert_eq!(rolled_back.rollback_aware, "last_confirmed");
    assert!(rolled_back.literal_lost());
    assert!(!rolled_back.rollback_aware_lost());
    let wrong_bytes =
        classify_random_read_back(&report_landing_on((1, 6), Some(b"two")), &ledger, 3, 3);
    assert_eq!(
        wrong_bytes.literal,
        "failed:content_differs_from_the_ledger"
    );
    let genesis = classify_random_read_back(&report_landing_on((0, 1), None), &ledger, 3, 3);
    assert_eq!(genesis.literal, "older:0");
}
