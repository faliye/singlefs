Question 1
M1: None ; reason: The mutation only affects the non-empty check in rollback_floor_ceiling, but T2 and T9 scenarios do not alter their assertion outcomes because T2's valid roots all have non-zero transactions and T9's error occurs before non-empty check.
M2: T1 ; reason: T1's ceiling calculation would include roots below current_floor due to readable candidates, causing the ceiling to be different and the assertion to fail.
M3: T1 ; reason: T1's non-empty check would now ignore extent tree changes, leading to incorrect non-empty status and ceiling value.
M4: T6 ; reason: T6's prepare function uses the allocator state, and incorrect allocation of mkfs units would alter the recorded writes and segment structure.
M5: T5 ; reason: T5's segment sequence depends on the back_chain value; setting it to 0 changes the journal record structure.
M6: T6 ; reason: The missing barrier would reduce the number of recorded operations, altering the segment structure that T6 depends on.
M7: T4 ; reason: T4's allocator state assertion expects 3 allocated slots, but incorrect span would reduce it to 2, causing the assertion to fail.
M8: T8 ; reason: T8's disk_snapshot comparison would detect the superblock slot write from acquire_instance, which happens before refusal.
M9: None ; reason: The mutation only affects the check for rollback to empty tree table, which is only relevant to T8.
M10: T1, T2 ; reason: Unreadable tree tables are now treated as ABSENT, altering non-empty status in ceiling calculation for T1 and T2's scenarios.

Question 2
T1: M2, M3, M10 ; reason: M2 changes candidate roots, M3 ignores extent tree changes, and M10 treats unreadable trees as ABSENT, all affecting T1's ceiling assertion.
T2: M3, M10 ; reason: M3 ignores extent tree changes and M10 treats unreadable trees as ABSENT, altering T2's non-empty status and ceiling calculation.
T3: None ; reason: T3 is a unit test for user_visible_trees_changed and no other mutation affects this function directly.
T4: M5, M7 ; reason: M5 changes back_chain value affecting journal records and M7 alters allocator span causing slot count mismatch.
T5: M5 ; reason: M5 sets back_chain to 0 which directly changes the journal record structure T5 checks.
T6: M4, M6 ; reason: M4 skips affects allocator allocation and M6 removes a barrier, both altering the recorded operations and segments T6 checks.
T7: None ; reason: T7's disk_snapshot comparison only checks superblock_slots, readable_roots, and recorded_operations, which no other mutation affects in its specific path.
T8: M8 ; reason: M8 swaps acquire_instance and refusal, causing superblock writes before refusal which T8's disk_snapshot would detect.
T9: M2, M3 ; reason: M2 changes candidate roots and M3 ignores extent tree changes, both affecting T9's ceiling calculation if tree table is readable.

Question 3
W1/T7: Covered ; reason: T7 compares superblock_slots which would differ if one device's superblock slot is written.
W1/T8: Covered ; reason: T8 compares superblock_slots which would differ if one device's superblock slot is written.
W1/T9: Not-covered ; reason: T9 does not compare superblock_slots at all.
W2/T7: Covered ; reason: T7 compares readable_roots and recorded_operations which would change due to journal record write.
W2/T8: Covered ; reason: T8 compares readable_roots and recorded_operations which would change due to journal record write.
W2/T9: Covered ; reason: T9 compares recorded_operations which increases with journal record write.
W3/T7: Covered ; reason: T7 compares readable_roots which would include the new self-certified root.
W3/T8: Covered ; reason: T8 compares readable_roots which would include the new self-certified root.
W3/T9: Not-covered ; reason: T9 does not compare readable_roots at all.
W4/T7: Covered ; reason: T7 compares superblock_slots which would differ due to the torn write on one device.
W4/T8: Covered ; reason: T8 compares superblock_slots which would differ due to the torn write on one device.
W4/T9: Not-covered ; reason: T9 does not compare superblock_slots at all.
W5/T7: Covered ; reason: T7 compares recorded_operations which increases with unit write.
W5/T8: Covered ; reason: T8 compares recorded_operations which increases with unit write.
W5/T9: Covered ; reason: T9 compares recorded_operations which increases with unit write.
W6/T7: Not-covered ; reason: Consecutive barriers are collapsed so recorded_operations does not change.
W6/T8: Not-covered ; reason: Consecutive barriers are collapsed so recorded_operations does not change.
W6/T9: Not-covered ; reason: Consecutive barriers are collapsed so recorded_operations does not change.
W7/T7: Not-covered ; reason: T7 does not compare allocator_state.
W7/T8: Not-covered ; reason: T8 does not compare allocator_state.
W7/T9: Covered ; reason: T9 compares allocator_state directly.

Question 4
If the tests interact with code not described in sections 1-6 such as hidden dependencies or unmentioned code paths that the mutations affect.

Question 5
If the tests have unmentioned dependencies on other parts of the system not covered in sections 1-6 that would cause additional mutations to fail them.

Question 6
If PoolReader@@read, readable_roots, or choose_superblock behave differently than described in section 6.1 such as filtering superblock slots or validating readable_roots beyond self-certification.
