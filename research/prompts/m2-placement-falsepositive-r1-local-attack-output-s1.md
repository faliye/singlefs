Z1.1 yes, placements_to_release_via_mapping
this would be falsified by observing that placements_to_release_via_mapping is not called during release operations under fault-free conditions
Z1.2 no, no specific test
this would be falsified by finding a test that fails when the checksum check is removed or added, indicating the test pins the current behavior
Z1.3 1, add checksum check in placements_to_release_via_mapping
this would be falsified by finding more than one distinct candidate fix described in the facts
Z1.4 no
this would be falsified by the fix for Z1 modifying a file, function, structure, or invariant that is also modified by the fix for another item
Z2.1 yes, allocation in allocator.rs
this would be falsified by observing that the allocator does not return refusal variants when devices disagree
Z2.2 yes, filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
this would be falsified by a test that passes even when the refusal behavior changes
Z2.3 2, one candidate is restoring the comparison step for policy_mismatches counter, another is modifying the placement structure to support per-device slot numbers
this would be falsified by finding only one or more than two candidate fixes described
Z2.4 yes, shared file allocator.rs with Z3
this would be falsified by the fix for Z2 modifying only files not shared with any other item's fix
Z3.1 yes, check_pool_image
this would be falsified by check_pool_image not being called during invariant checks
Z3.2 yes, residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
this would be falsified by changing the behavior and the test still passing
Z3.3 2, adjust walk to include deferred-release units or adjust allocator's count
this would be falsified by finding more than two candidates
Z3.4 yes, shared file allocator.rs with Z2
this would be falsified by Z3's fix modifying only files not shared with Z2
Z4.1 yes, check_records_against
this would be falsified by the function not being called during record checking
Z4.2 yes, stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
this would be falsified by the test passing when the criterion's behavior changes
Z4.3 1, modify the written-over-later check to verify the later write is persisted in the crash state
this would be falsified by finding more than one candidate fix
Z4.4 no
this would be falsified by the fix for Z4 modifying a file, function, structure, or invariant shared with another item
