use singlefs_replay_prefix::gpu_gc_proofs::*;

const EDGES: [u64; 11] = [
    0,
    1,
    3,
    4,
    5,
    u32::MAX as u64,
    1u64 << 32,
    u64::MAX - 3,
    u64::MAX - 2,
    u64::MAX - 1,
    u64::MAX,
];

#[test]
fn gpu_budget_arithmetic_matches_a_u128_reference_at_full_width() {
    for input in EDGES {
        let aligned = u128::from(input).max(4).div_ceil(4) * 4;
        assert_eq!(checked_gpu_input_bytes(input), u64::try_from(aligned).ok());
        for entries in EDGES {
            for per_entry in EDGES {
                for fixed in EDGES {
                    let expected = (u128::from(entries) * u128::from(per_entry))
                        .checked_add(aligned)
                        .and_then(|bytes| bytes.checked_add(u128::from(fixed)))
                        .and_then(|bytes| u64::try_from(bytes).ok());
                    assert_eq!(
                        checked_gpu_chunk_bytes(input, entries, per_entry, fixed),
                        expected
                    );
                }
            }
        }
    }
    for limit in EDGES {
        for held in EDGES {
            assert_eq!(
                remaining_gpu_memory(limit, held),
                limit.saturating_sub(held)
            );
        }
    }
}

#[test]
fn gpu_slices_scans_and_independent_totals_match_wide_arithmetic() {
    for total in EDGES {
        for start in EDGES.into_iter().filter(|start| *start <= total) {
            for capacity in EDGES.into_iter().filter(|capacity| *capacity > 0) {
                let expected = u128::from(total).min(u128::from(start) + u128::from(capacity));
                assert_eq!(
                    u128::from(next_gpu_record_slice_end(start, total, capacity)),
                    expected
                );
            }
        }
    }
    for slot in EDGES {
        for span in EDGES {
            for start in EDGES {
                for end in EDGES {
                    for same_device in [false, true] {
                        for released in [false, true] {
                            let expected = same_device
                                && !released
                                && (start..end).contains(&slot)
                                && u128::from(slot) + u128::from(span) <= u128::from(end);
                            assert_eq!(
                                gpu_record_is_in_scan_region(
                                    same_device,
                                    released,
                                    slot,
                                    span,
                                    start,
                                    end
                                ),
                                expected
                            );
                        }
                    }
                }
            }
        }
    }
    for walking in EDGES {
        for searching in EDGES {
            for next_walking in EDGES {
                for next_searching in EDGES {
                    let left = u64::try_from(u128::from(walking) + u128::from(next_walking)).ok();
                    let right =
                        u64::try_from(u128::from(searching) + u128::from(next_searching)).ok();
                    assert_eq!(
                        checked_live_slot_totals(walking, searching, next_walking, next_searching),
                        left.zip(right)
                    );
                }
            }
        }
    }
}

#[test]
fn gpu_readback_rejects_unwritten_words_and_preserves_both_halves() {
    for value in EDGES {
        let bytes = value.to_le_bytes();
        let low = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let high = u32::from_le_bytes(bytes[4..].try_into().unwrap());
        for verdict in [0, 1, 2, u32::MAX] {
            assert_eq!(
                decoded_gpu_live_slots(low, high, verdict),
                (verdict == 1).then_some(value)
            );
        }
    }
}

#[test]
fn gpu_reconciliation_matches_the_two_round_history_rule() {
    for history in 0u32..(1 << 12) {
        let mut previous_disagreement = false;
        let mut counter = 0;
        for round in 0..12 {
            let agree = history & (1 << round) != 0;
            let expected_cpu = !agree && previous_disagreement;
            let step = reconcile_gpu_entry(counter, agree);
            assert_eq!(step.confirmed, agree);
            assert_eq!(step.ask_cpu, expected_cpu);
            previous_disagreement = !agree && !expected_cpu;
            assert_eq!(step.uncertain_rounds, u32::from(previous_disagreement));
            counter = step.uncertain_rounds;
        }
    }
    let invalid_old_count = reconcile_gpu_entry(u32::MAX, false);
    assert!(invalid_old_count.ask_cpu);
    assert_eq!(invalid_old_count.uncertain_rounds, 0);
}
