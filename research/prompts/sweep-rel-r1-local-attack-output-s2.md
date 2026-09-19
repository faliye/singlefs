1. L1
1. Current-state clauses: "as of 2026-09-14 all four already exist: crates/singlefs-core, gate 54" and "layer 0's workload only has the first transaction, without release or reuse".
2. First clause: yes, carries date. What follows the date describes the state of affairs as of that day (all four existing). Second clause: no, does not carry a date in the quoted text.
3. Verdict: irrelevant. The passage's words overlap with the new facts' topic (layer 0's workload and crash-point-replay harness) but the passage is not talking about the same fact. The new facts are about different dates (Fact 1 as of 2026-09-17, Fact 2 as of 2026-09-18, Fact 3 as of 2026-09-16) while the row's clauses are about 2026-09-14, making them distinct facts.
4. Observation that would overturn: a new fact stating that as of 2026-09-14, one of the four components (transaction layer, allocator, root ring, crash-point-replay harness) did not exist or that the layer 0 workload included release or reuse.

2. L2
1. Current-state clauses: "as of 2026-09-14 the first layout's form exists: the pool-level checker judges 23 items, each one paired with a bad-mirror image [...]" and "the layer-1-harness half is still owed".
2. First clause: yes, carries date. What follows the date describes the state of affairs as of that day (pool-level checker judges 23 items). Second clause: no, does not carry a date in the quoted text.
3. Verdict: irrelevant. The passage's words overlap with the new facts' topic (pool-level checker) but the passage is not talking about the same fact. The new fact (Fact 2) is about 2026-09-18 with 29 items, while the row's clause is about 2026-09-14 with 23 items, making them distinct facts. The layer-1-harness clause is not mentioned in new facts.
4. Observation that would overturn: a new fact stating that as of 2026-09-14, the pool-level checker judged 29 items or that the layer-1-harness half is no longer owed.

3. L3
1. Current-state clauses: "the change of checkpoint_txg from 1 to 3 for the first transaction has already landed (2026-09-13)" and "the crash-point-replay harness check item is still owed".
2. First clause: yes, carries date. What follows the date describes something that happened on that specific day (change landed). Second clause: no, does not carry a date in the quoted text.
3. Verdict: event-clause-no-change. The first clause describes an event that happened on 2026-09-13 (change landed), so a later fact cannot make it false. The second clause is a current-state clause that remains true but is not affected by new facts; the verdict is determined by the event clause.
4. Observation that would overturn: a new fact stating that the change of checkpoint_txg from 1 to 3 for the first transaction did not land on 2026-09-13.

4. L4
1. Current-state clauses: none. The text "The implementation of row reclamation; the multi-mount recording stream" describes items without stating current state (e.g., no "already has", "still owed", etc.).
2. No clauses.
3. Verdict: irrelevant. There are no current-state clauses to judge, and the new facts do not mention row reclamation or multi-mount recording stream.
4. Observation that would overturn: if the text included a current-state clause (e.g., "row reclamation implementation is complete") that contradicted new facts.

5. L5
1. Current-state clauses: "Each entry is 18 bytes (length 2 plus 16 reserved)" and "using the 2026-09-06 entry widths: inode tree 175 to 171 [...]".
2. First clause: no, does not carry a date in the quoted text. Second clause: yes, carries date. What follows the date describes the state of affairs as of that day (inode tree sizes).
3. Verdict: needs-change. The clause "Each entry is 18 bytes (length 2 plus 16 reserved)" is false due to Fact 3 (tree-table entry is 200 bytes as of 2026-09-16). Revised sentence: "Each entry is 200 bytes (total, with 76 bytes reserved) [...] (using the 2026-09-16 entry widths: 81 trees per layer, with 76 bytes reserved)."
4. Observation that would overturn: a new fact stating that the tree-table entry size is 18 bytes as of 2026-09-16.

6. L6
1. Current-state clauses: "the overwrite, release, multi-mount, and rollback that the remaining seven items need are not yet in layer 0's workload".
2. Yes, carries date (2026-09-14 status). What follows the date describes the state of affairs as of that day (items not yet in workload).
3. Verdict: needs-change. The clause is false due to Fact 1 (layer 0's workload includes overwrite, release, rollback, etc. as of 2026-09-17). Revised sentence: "2026-09-17 status: [...] the overwrite, release, multi-mount, and rollback that the remaining seven items need are now in layer 0's workload."
4. Observation that would overturn: a new fact stating that as of 2026-09-17, the overwrite, release, multi-mount, and rollback are still not in layer 0's workload.

7. L7
1. Current-state clauses: "each state gets two recovery passes, plus a checker for 26 invariants, plus a record cross-checker".
2. Yes, carries date (partially landed as of 2026-09-17). What follows the date describes the state of affairs as of that day (recovery passes, invariants, etc.).
3. Verdict: irrelevant. The new facts do not mention invariants, recovery passes, or record cross-checkers; the passage's topic does not overlap with new facts.
4. Observation that would overturn: a new fact stating the number of invariants or recovery passes differs from 26.

8. L8
1. Current-state clauses: "Not yet done: the multi-mount recording stream" and "the checks for C329, C330, and five newly opened items are all waiting on it".
2. No, does not carry a date in the quoted text.
3. Verdict: irrelevant. The new facts do not mention the multi-mount recording stream or checks for C329/C330; the passage's topic does not overlap with new facts.
4. Observation that would overturn: a new fact stating the multi-mount recording stream is complete.

9. L9
1. Current-state clauses: "D5 has not decided this" and "the content of a deadlist entry will be decided together with the location authority".
2. No, does not carry a date in the quoted text.
3. Verdict: irrelevant. The context states this is a past record (not current state), and new facts do not mention D5 or deadlist entries.
4. Observation that would overturn: a new fact stating D5 has decided the deadlist entry content.

10. L10
1. Current-state clauses: "D22 (how unit atomicity is composed) settled item 7 has been unified to compute 112 trees per layer using a 145-byte entry".
2. Yes, carries date (as of 2026-09-13). What follows the date describes the state of affairs as of that day (trees per layer and entry size).
3. Verdict: needs-change. The clause is false due to Fact 3 (tree-table entry is 200 bytes and 81 trees per layer as of 2026-09-16). Revised sentence: "This half of the kb has been closed out as of 2026-09-16: D22 (how unit atomicity is composed) settled item 7 has been unified to compute 81 trees per layer using a 200-byte entry."
4. Observation that would overturn: a new fact stating the tree-table entry is 145 bytes with 112 trees per layer as of 2026-09-16.
