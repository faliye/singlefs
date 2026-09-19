1. For L1:
1. Clauses describing current state: "all four already exist: crates/singlefs-core, gate 54" and "layer 0's workload only has the first transaction, without release or reuse".
2. Both clauses carry a date. For "all four already exist: crates/singlefs-core, gate 54", what follows the date describes the state of affairs as of 2026-09-14. For "layer 0's workload only has the first transaction, without release or reuse", what follows the date describes the state of affairs as of 2026-09-14.
3. Verdict: irrelevant. The clauses reference a specific past date (2026-09-14), while the new facts (e.g., Fact 1 as of 2026-09-17) describe a different point in time. The passage is with the topic of layer 0's workload but addresses a distinct fact (state as of 2026-09-14 vs. 2026-09-17), so it is irrelevant to the new facts.
4. Observation that would overturn: If a new fact stated that as of 2026-09-14, layer 0's workload included release and reuse, this would make the clause false and require a needs-change verdict.

2. For L2:
1. Clauses describing current state: "the pool-level checker judges 23 items" and "the layer-1-harness half is still owed".
2. Both clauses carry a date. For "the pool-level checker judges 23 items", what follows the date describes the state of affairs as of 2026-09-14. For "the layer-1-harness half is still owed", what follows the date describes the state of affairs as of 2026-09-14.
3. Verdict: irrelevant. The "pool-level checker judges 23 items" clause refers to a past state (2026-09-14), while Fact 2 describes the current state as of 2026-09-18 (29 items). The layer-1-harness clause is unrelated to any new fact. Both clauses address distinct facts from the new facts, so the row is irrelevant.
4. Observation that would overturn: If a new fact stated that as of 2026-09-18, the pool-level checker judges 23 items, this would contradict Fact 2 and require a needs-change verdict for that clause.

3. For L3:
1. Clauses describing current state: "the change of checkpoint_txg from 1 to 3 for the first transaction has already landed (2026-09-13)" and "the crash-point-replay harness check item is still owed".
2. For "the change of checkpoint_txg from 1 to 3 for the first transaction has already landed (2026-09-13)", the date is present and what follows describes an event that happened on 2026-09-13. For "the crash-point-replay harness check item is still owed", there is no date in the clause itself (the 2026-09-13 reference is part of the prerequisite label, not the clause), so it is a current-state clause without a date.
3. Verdict: needs-change. The clause "the crash-point-replay harness check item is still owed" is false due to Fact 1, which states that layer 0's crash-point-replay workload now includes release and reuse (part of the fixed script), implying the check item is resolved. Revised sentence: "Warning: progress on the prerequisite (2026-09-13, E150): [...] the change of checkpoint_txg from 1 to 3 for the first transaction has already landed (2026-09-13) [...]; the crash-point-replay harness check item is now complete."
4. Observation that would overturn: If a new fact explicitly stated that the crash-point-replay harness check item is still owed as of 2026-09-17, this would confirm the clause and invalidate the needs-change verdict.

4. For L4:
1. No clauses describing current state. The quoted text ("The implementation of row reclamation; the multi-mount recording stream.") lists items without any status indicators (e.g., "already has", "still owed"), so it does not contain any clauses that describe a current state of affairs.
2. Not applicable.
3. Verdict: irrelevant. There are no current-state clauses to evaluate against the new facts, so the row does not relate to the judgment criteria.
4. Observation that would overturn: If the row included a current-state clause (e.g., "the multi-mount recording stream is still owed"), this would require reevaluation against Fact 1.

5. For L5:
1. Clauses describing current state: "Each entry is 18 bytes (length 2 plus 16 reserved)" and "using the 2026-09-06 entry widths: inode tree 175 to 171 [...]".
2. Both clauses carry a date. For "Each entry is 18 bytes (length 2 plus 16 reserved)", what follows the date describes the state of affairs as of 2026-09-06. For "using the 2026-09-06 entry widths: inode tree 175 to 171 [...]", what follows the date describes the state of affairs as of 2026-09-06.
3. Verdict: needs-change. Fact 3 states that as of 2026-09-16, a tree-table entry is 200 bytes with 76 bytes reserved, which contradicts the entry size and tree count in the clause. Revised sentence: "Each entry is 200 bytes (76 bytes reserved) [...] (using the 2026-09-16 entry widths: 81 trees per layer [...])."
4. Observation that would overturn: If a new fact stated that as of 2026-09-16, the entry size is 18 bytes and 112 trees per layer, this would confirm the clause and invalidate the needs-change verdict.

6. For L6:
1. Clauses describing current state: "the overwrite, release, multi-mount, and rollback that the remaining seven items need are not yet in layer 0's workload".
2. The clause carries a date. For "the overwrite, release, multi-mount, and rollback that the remaining seven items need are not yet in layer 0's workload", what follows the date describes the state of affairs as of 2026-09-14.
3. Verdict: irrelevant. The clause references a past state (2026-09-14), while Fact 1 states that as of 2026-09-17, layer 0's workload includes these elements. The passage addresses a distinct fact (state as of 2026-09-14 vs. 2026-09-17), so it is irrelevant to the new facts.
4. Observation that would overturn: If a new fact stated that as of 2026-09-14, the overwrite, release, multi-mount, and rollback were already in layer 0's workload, this would make the clause false and require a needs-change verdict.

7. For L7:
1. Clauses describing current state: "each state gets two recovery passes, plus a checker for 26 invariants, plus a record cross-checker".
2. The clause carries a date. For "each state gets two recovery passes, plus a checker for 26 invariants, plus a record cross-checker", what follows the date describes the state of affairs as of 2026-09-17.
3. Verdict: irrelevant. The new facts do not mention recovery passes, invariants, or record cross-checkers, so the passage does not overlap with any new fact's topic.
4. Observation that would overturn: If a new fact explicitly referenced recovery passes, invariants, or record cross-checkers, this would require reevaluation.

8. For L8:
1. Clauses describing current state: "Not yet done: the multi-mount recording stream" and "the checks for C329, C330, and five newly opened items are all waiting on it".
2. Neither clause carries a date. The section heading includes a date (2026-09-14), but the quoted sentence itself does not reference any date.
3. Verdict: needs-change. Fact 1 states that layer 0's crash-point-replay workload includes "reopening for writes (the second writable instance), warm-up, rollback, raising F, and reuse", which implies the multi-mount recording stream is now complete. Revised sentence: "Done: the multi-mount recording stream (the checks for C329, C330, and five newly opened items are now resolved)."
4. Observation that would overturn: If a new fact explicitly stated that the multi-mount recording stream is still not done as of 2026-09-17, this would confirm the clause and invalidate the needs-change verdict.

9. For L9:
1. No clauses describing current state. The context states "Warning: this is that round's record, not the current state (annotated 2026-09-13)", and the quoted text is part of an older record. The clauses reference past decisions (e.g., "D5 has not decided this"), but the context explicitly states this is not current state, so no current-state clauses are present.
2. Not applicable.
3. Verdict: irrelevant. The row is explicitly labeled as a past record, not current state, so it does not relate to the new facts.
4. Observation that would overturn: If the context did not state that this is a past record and the clauses described current state, this would require reevaluation.

10. For L10:
1. Clauses describing current state: "This half of the kb has been closed out as of 2026-09-13" and "D22 (how unit atomicity is composed) settled item 7 has been unified to compute 112 trees per layer using a 145-byte entry".
2. Both clauses carry a date. For "This half of the kb has been closed out as of 2026-09-13", what follows the date describes the state of affairs as of 2026-09-13. For "D22 (how unit atomicity is composed) settled item 7 has been unified to compute 112 trees per layer using a 145-byte entry", what follows the date describes the state of affairs as of 2026-09-13.
3. Verdict: needs-change. Fact 3 states that as of 2026-09-16, a tree-table entry is 200 bytes with 81 trees per layer and 76 bytes reserved, which contradicts the tree count and entry size in the clause. Revised sentence: "This half of the kb has been closed out as of 2026-09-16: D22 (how unit atomicity is composed) settled item 7 has been unified to compute 81 trees per layer using a 200-byte entry with 76 bytes reserved."
4. Observation that would overturn: If a new fact stated that as of 2026-09-16, the entry size is 145 bytes and 112 trees per layer, this would confirm the clause and invalidate the needs-change verdict.
