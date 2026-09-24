TASK

You are given four independent items, named Z1, Z2, Z3, and Z4, from a filesystem project's open-issues list. For each item you are given a set of numbered facts (F1, F2, and so on) taken directly from the project's source code, its test suite, and its own tracked-issue notes. Your job is purely formal: for each item, answer four fixed questions using only the facts given below plus ordinary reasoning about what those facts imply. Do not invent facts that are not given here. Do not evaluate whether the underlying design is a good one; that is a separate task done elsewhere.

ANSWER FORMAT

Produce exactly 16 numbered answers, labeled Z1.1, Z1.2, Z1.3, Z1.4, Z2.1, Z2.2, Z2.3, Z2.4, Z3.1, Z3.2, Z3.3, Z3.4, Z4.1, Z4.2, Z4.3, Z4.4, in that order. For every one of the 16 answers, name which fact numbers you used to support it; if you cannot name a supporting fact number for some part of an answer, write "no supporting fact" for that part instead of leaving it blank or guessing.

Do not cite source-code line numbers or file line numbers anywhere in your answer. Refer to functions, tests, and structures by the names given in the facts below, or by fact number, or by table row (Z1, Z2, Z3, Z4) and column (1, 2, 3, 4).

Do not use markdown bold, italic, or heading markup. Plain numbered lines only.

Do not write prose beyond what is needed to answer the four questions for each item. Do not summarize or repeat the facts back to me.

THE FOUR QUESTIONS, ASKED ONCE PER ITEM

column 1: Under fault-free operation (no injected crash, no corrupted bytes, no partial write), is the code path this item describes reached and run to completion today? Answer yes, no, or unknown, and name the function or functions it passes through.

column 2: Does an existing test in the project's test suite pin down today's behavior for this item, in the sense that today's behavior changing would make an existing test fail? Answer yes or no, and name the specific test function.

column 3: How many genuinely different candidate fixes are described or implied by the facts given for this item? Give the count as a number, and one sentence describing each candidate.

column 4: Does the fix implied for this item touch the same source file, the same function or structure, or the same named invariant as the fix implied for one or more of the other three items? Answer yes or no; if yes, name which file, function, structure, or invariant is shared, and name which other item (Z1, Z2, Z3, or Z4) it is shared with.

For every one of the 16 answers, add one sentence that starts with the words "this would be falsified by" and describes an observation that would force you to change that specific answer.

unknown is an acceptable answer for column 1 only; it is not treated as a failure. If you answer unknown anywhere, add one line at the very end of your reply: "cells answered unknown: N" where N is the count (write 0 if none).

ITEM Z1: release does not check the unit checksum carried by a mapping entry's location field

F1. In the source file crates/singlefs-core/src/transaction.rs, the function placements_to_release_via_mapping decides, for a previous published version, which placements to release. For each unit, it looks up the previous version's mapping to get two location entries, one per device. It then checks three things on each device before releasing: the allocation record for that slot is on file, the record is not already marked released, and the record's span matches the expected span for that kind of unit. It does not read or compare the 32-bit unit-checksum field that each location entry also carries.

F1a. In crates/singlefs-core/src/recovery.rs, the function rebuild_version reads the mapping tree's root node and, for each entry in it, calls a parser that returns a key and a pair of location entries; the code binds the key and explicitly discards the pair of location entries (the discarded value is bound to a name that marks it as intentionally unused). The unit-checksum field described in F3 lives inside those discarded location entries.

F2. In the source file crates/singlefs-core/src/pointer.rs, the function slot_shared_by_both_location_entries takes the two location entries belonging to one pointer and returns the slot number they share, after checking only that their slot fields are equal to each other. It does not read the unit-checksum field of either entry.

F3. In the same source file, the location-entry structure (the type used for both entries in a pointer, and for both entries looked up by placements_to_release_via_mapping) has exactly three fields: a device identity, a slot number, and a 32-bit unit checksum.

F4. A search of the project's test suite (the tests directories under crates/singlefs-core and crates/singlefs-harness) for any test that asserts something about the unit-checksum field specifically at release time found none. The tests that do check the unit-checksum field all do so at write time or at read and recovery time; for example, the function read_unit_via_locations, used during recovery, does compare a location entry's unit checksum against the bytes read from disk before accepting them, but that function is not called by placements_to_release_via_mapping.

F5. The project's own tracked-issues file has a row, identifier C394, opened 2026-09-19, whose one-line description is: the release-judgment path does not check the unit checksum carried by the mapping entry's location field when rebuilding the previous version from disk.

ITEM Z2: placement answers can differ across devices of unequal size

F6. In crates/singlefs-core/src/allocator.rs, the placement structure (the type returned by allocation and consumed by release) has exactly two fields: a single slot number and a span. It does not carry one slot number per device.

F7. In the same file, the placement-refusal enum (the error type returned when the allocator refuses to place something) has four variants. One variant means every device is out of matching free space, an ordinary out-of-space condition. The other three are returned, before any on-disk state is modified, when devices of unequal size disagree about where something should go: one variant covers "the smaller device is full, and there is no defined rule for which devices stay in the write set once that happens"; one variant covers "each device's free-space search picked a different slot number for the same piece of user data, and today's placement structure and location-entry format cannot represent a different slot number per device"; and one variant covers "each device picked a different answer for where to open or continue a commit-generated segment, and there is no defined rule for whether per-device cluster segments must stay aligned."

F8. Three tests in crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs assert this reject-before-mutating-state behavior by name: filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking, devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written, and user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written.

F9. In crates/singlefs-core/src/allocator.rs, the pool-allocator structure has a public field, policy_mismatches, of type u64. It is initialized to 0 in the constructor and is never incremented anywhere else in the file. A test in the same file, first_transaction_placements_follow_the_byte_table, asserts it stays 0 after a sequence of allocations.

F10. The project's verification-build notes describe this counter's intended meaning as "the number of times a piece of user data's placement disagrees with the policy function," and record that the comparison step that would have fed this counter was removed on 2026-09-17, with the note "the placement that gets sent out is already the answer every device agreed on." A separate claim, made elsewhere about this same counter, says the field's name written in the project's verification notes does not match the field's actual name in the code. Reading the verification notes directly today shows the name written there, policy_mismatches, already matches the field's actual name in the code; this prompt does not resolve why the two disagree, it only reports what today's file contains.

ITEM Z3: invariant I-3.1, "allocated accounting matches", is judged violated on states the project's own notes call legal

F11. In crates/singlefs-checker/src/walk.rs, the public function check_pool_image computes a set of candidate roots: the newest root, plus every other root that is neither superseded, per the newest root's own instance table, nor older than the newest root's own rollback-floor value, a generation number carried inside the root record itself.

F12. In the same function, invariant I-3.1 is judged by comparing the allocator's own recorded allocated-byte count, per device, against the sum obtained by walking every unit reachable from the candidate-root set described in F11; a mismatch is reported as an I-3.1 violation. Note on this fact only: this source file was being actively edited by a separate, concurrent process while these facts were being gathered; two readings taken minutes apart showed a different checksum and a roughly fifteen-line shift in where this logic sits in the file, so no line number for it is stable enough to report. The function name and the logic described here were read directly from the file's contents at each of the two readings, and the logic itself (which roots count as candidates, and that I-3.1 compares allocator counts against a walk over those candidates) was identical both times.

F13. A test named residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it, in crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs, seeds a base image with one published version whose journal record was applied but whose root slot was never persisted, then enumerates 31 crash states reachable from that base image. It asserts, through a shared helper named assert_checker_and_record_checker_counts, that invariant I-3.1 is violated on exactly 12 of those 31 states. An inline comment in that test attributes the gap to 65536 bytes per device, tracing it to four fixed-point units belonging to that residual version: a later write-line publish moves those four units into the deferred-release queue, and no root in the candidate set described in F11 still references them, so the walk undercounts them while the allocator still counts them as allocated. The same inline comment adds that which side of that disagreement should change is an open design question, and that this test pins today's status quo rather than endorsing it as correct behavior.

F14. The project's milestone closure table has a row, labeled 54 in that table, that records this same twelve-out-of-thirty-one gap as an open item still requiring further argument, and separately records that an attacker exercise, tagged c381-r1, reproduced an equivalent-shaped state using a plain power-loss injection on a separate copy of the current core code, on 2026-09-19.

ITEM Z4: the record checker's second criterion does not recognize legitimate reuse

F15. In crates/singlefs-harness/src/crash.rs, the function check_records_against implements the record checker's second criterion. For each publish whose checkpoint generation is at or below the effective root's generation, it groups that publish's own unit writes by device and byte offset. It flags claimed_state_missing_unit when, for some offset group, every copy in that group is both: absent from the crash image being checked, meaning its bytes on disk do not match what was written, and not written-over-later, where written-over-later means some other write, later in the retained write list passed into this function, targets an overlapping byte range on the same device. This written-over-later check looks only at the position of writes in the list; it does not separately check whether that later write is itself marked persisted in the specific crash state under examination.

F16. A test named stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state, in crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs, builds a stream with a stale journal tail plus a reused named unit, enumerates exactly 8 crash states, and asserts, through the same shared helper named in F13, with an empty list of expected checker violations, that claimed_state_missing_unit is 0 on all 8 of those states. An inline comment in that test states that this count was pinned at 8, meaning all 8 states flagged as missing, when the test was first written on 2026-09-17, at a time when this question was still undecided; the comment says that on 2026-09-20 the project decided the reading "a unit overwritten later by an already-persisted write at the same location does not count as missing," and that reverting that decision would turn this specific test red.

F17. The project's milestone closure table has a row, labeled 55 in that table, that describes this same criterion as currently misreporting on all 8 states of a stale-tail stream, using the same wording, "a unit overwritten later by an already-persisted write at the same location does not count as missing," as a candidate fix that the row says still needs further review, not as a decision already made.

F17a, a note rather than a fact for you to resolve: F16 and F17 describe the same shape, 8 states, a stale-tail stream, the same named exemption rule, but disagree about whether today's code produces 0 flagged states or 8. This prompt does not tell you which of the two is authoritative. Use whichever of the two is load-bearing for the specific column you are answering, and if the disagreement itself matters for a column, for example column 1 or column 2, name the disagreement explicitly in your answer rather than silently picking one side.

FACTS ABOUT WHICH FILES AND NAMES EACH ITEM'S CODE LIVES IN, FOR COLUMN 4

F18. F1 through F5, item Z1, concern code in crates/singlefs-core/src/transaction.rs, the function placements_to_release_via_mapping, and crates/singlefs-core/src/pointer.rs, the location-entry structure and the function slot_shared_by_both_location_entries, together with two release-facing functions in crates/singlefs-core/src/allocator.rs: a lookup function named record_for and a function named release, both of which operate on the placement structure named in F6.

F19. F6 through F10, item Z2, concern the placement structure and the placement-refusal enum, both in crates/singlefs-core/src/allocator.rs, together with the policy_mismatches field on the pool-allocator structure in the same file. The placement structure named here is the same type used by the release-facing functions named in F18: record_for takes a slot number, and release takes a placement value directly.

F20. F11 through F14, item Z3, concern crates/singlefs-checker/src/walk.rs, the function check_pool_image, and, on the side it compares against, the allocator's per-device allocated-byte counters and its deferred-release bookkeeping in crates/singlefs-core/src/allocator.rs, which is exercised through the same release function named in F18.

F21. F15 through F17, item Z4, concern crates/singlefs-harness/src/crash.rs, the function check_records_against and the two helper closures named in F15. This file lives in the project's test-harness crate, not in the core crate or the checker crate that F18, F19, and F20 concern.

END OF FACTS. Answer Z1.1 through Z4.4 now, in the format given above.
