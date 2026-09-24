Z1.1 yes, placements_to_release_via_mapping (F1, F4)
this would be falsified by observing that placements_to_release_via_mapping does not run to completion under fault-free operation.

Z1.2 no, no specific test function (F4)
this would be falsified by finding an existing test that asserts behavior related to the unit checksum at release time.

Z1.3 1, candidate: modify placements_to_release_via_mapping to check the unit checksum (F1, F5)
this would be falsified by discovering that the unit checksum cannot be meaningfully checked at release time and no other candidate fix is viable.

Z1.4 no (F18, F19, F20, F21)
this would be falsified by the fix for Z1 requiring a change in the same file, function, structure, or invariant as the fix for another item.

Z2.1 no, allocation function in allocator.rs (F7, F8)
this would be falsified by observing that when placement answers differ across devices of unequal size, the code path runs to completion without error.

Z2.2 yes, filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking, devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written, and user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written (F8)
this would be falsified by the test passing when the refusal behavior is not present (e.g., when placement is allowed to proceed despite disagreement).

Z2.3 2, candidates: (1) modify placement structure to support per-device slot numbers, (2) define policy rules for resolving disagreements (e.g., for unit data, use the smaller device's slot; for segments, define alignment rules) (F7)
this would be falsified by discovering that no candidate fix is viable (e.g., changing the structure is impossible due to other constraints, and defining rules is not feasible).

Z2.4 yes, allocator.rs shared with Z3 (F19, F20)
this would be falsified by the fix for Z2 and the fix for Z3 not touching allocator.rs (e.g., if Z3's fix only changes walk.rs and Z2's fix only changes the placement-refusal enum).

Z3.1 yes, check_pool_image (F11, F12)
this would be falsified by observing that check_pool_image does not run to completion under fault-free operation (e.g., it panics or returns an error).

Z3.2 yes, residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it (F13)
this would be falsified by the test passing when the number of I-3.1 violations changes from 12 to something else.

Z3.3 2, candidates: (1) modify the walk to include deferred-release units in the count, (2) change the invariant definition to exclude deferred-release units (F13)
this would be falsified by discovering that no candidate fix is viable (e.g., the deferred-release units must be counted in both, but the current walk doesn't include them and the allocator does, and there's no way to change that without breaking other things).

Z3.4 yes, allocator.rs shared with Z2 (F19, F20)
this would be falsified by the fix for Z3 not changing allocator.rs and the fix for Z2 not changing allocator.rs (e.g., Z2's fix only changes the placement-refusal enum and Z3's fix only changes walk.rs).

Z4.1 yes, check_records_against (F15, F16)
this would be falsified by observing that check_records_against does not run to completion under fault-free operation (e.g., it panics or returns an error).

Z4.2 yes, stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state (F16)
this would be falsified by the test passing when claimed_state_missing_unit is not 0.

Z4.3 1, candidate: modify the written-over-later check to verify that the later write is persisted in the crash state (F15, F16)
this would be falsified by discovering that no candidate fix is viable (e.g., the check cannot be modified without breaking other parts).

Z4.4 no (F21, F18, F19, F20)
this would be falsified by the fix for Z4 requiring a change in the same file, function, structure, or invariant as the fix for another item.