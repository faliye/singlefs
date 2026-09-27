Round: m2-closeout-code-r1, local attack leg (arithmetic only). You are given a set of hard facts copied verbatim from source files in a git snapshot, each with its file path and line number. Your job is to fill in every cell of the tables below with a computed number, show the arithmetic step that produced it, and state whether it matches the pinned value given for that row. Do not answer yes or no anywhere; every cell needs an actual number and the arithmetic that produced it.

Rule for your answer: do not write any file:line citation and do not write any code line number in your answer. When you need to point at something, name the function, the constant, or the table row label instead (for example: write "the EveryProperSubset branch" or "row two-4GiB", not a line number).

For every numbered item below, also write one sentence: what observation would show this item's derivation is wrong (a mismatch between the arithmetic you show and the pinned value counts as "wrong").

===================================================================
FACT SET A. First stream (the first transaction), closed-form state counts.

A1. The segment-length sequence for this stream, copied verbatim from crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs line 457 (the vector both the full-enumeration test and the quick-tier test assert against):
[2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
This is a sequence of 10 segments; the numbers are how many writes are in each segment, in order.

A2. Two-state closed form (every write treated as landed-or-not, no torn state), from the function closed_form_state_count in crates/singlefs-harness/src/crash.rs line 620:
closed_form_state_count(segments) = 1 + sum over each segment of (2^(segment length) - 1)
Pinned value for this stream under this formula: 67_108_885 (constant FULL_STATES_WITH_TWO_STATES_PER_WRITE).

A3. Full torn-state enumeration formula, from the enum variant EveryProperSubset's doc comment in crates/singlefs-harness/src/crash.rs (lines 1720-1723), quoted in English:
"For a segment of n writes of which m are torn in-place overwrites, the state count is 3^m * 2^(n-m) - 1 (taking every combination of landing states and subtracting the one case where the whole segment is fully persisted). When m = 0 this reduces to 2^n - 1 (every proper subset)."

A4. The comment pinning which segments in fact set A1 are torn (system-configuration-slot writes), from crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs lines 437-438, quoted in English:
"Of this stream's writes, only 8 system-configuration-slot writes take the third (torn) state. Four segments consisting purely of system-configuration-slot writes each contribute 3^2 - 1; the segment named A (the 26-write segment) contributes 3^2 * 2^24 - 1; every other segment is as in the two-state case; then add 1 for the all-persisted state."
Pinned total under this rule: 150_994_980 (constant FULL_STATES).

A5. The quick-tier torn-state formula, from the enum variant InPlaceSubsetsWithCopyOnWriteNoneOrAll's doc comment in crates/singlefs-harness/src/crash.rs (lines 1725-1728), quoted in English:
"Within a segment, each in-place write (any write other than a unit write) takes any combination of its own landing states, multiplied by the unit (copy-on-write) writes taken only as either none-landed or all-landed, minus the one case where the whole segment is fully persisted. With k in-place writes (of which m are torn overwrites) and c unit writes, the count is 3^m * 2^(k-m) * (2 if c > 0 else 1) - 1; when m = 0 this is 2^(k+1) - 1 if c > 0, or 2^k - 1 if c = 0."

A6. The comment pinning the quick-tier count for this stream, from crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs lines 440-441, quoted in English:
"For segment A: 2 in-place writes (system-configuration-slot writes, each three-state) times unit writes taken as none-landed-or-all-landed gives 3^2 * 2 - 1 = 17. The four system-configuration-slot-only segments each contribute 3^2 - 1 = 8. Every other segment is as in the full enumeration (a 2-write segment contributes 3, a 1-write segment contributes 1). Then add 1 for the all-persisted state."
Pinned total under this rule: 54 (constant QUICK_TIER_STATES).

TASK for fact set A: build a table with one row per segment in A1 (10 rows, numbered 1 through 10 in the order given). For each row, state: the segment's write count n; whether you classify it as one of the "four system-configuration-slot segments" (m = n = 2, fully torn), as segment A (the 26-write segment, m = 2 of its 24 writes... state your own count for m and for the copy-on-write writes), or as an ordinary segment (m = 0); the full-enumeration state count for that row using the A3 formula; the quick-tier state count for that row using the A5 formula. Then sum all 10 rows' full-enumeration counts and add 1, and separately sum all 10 rows' quick-tier counts and add 1. State whether your two sums match A4's pinned 150_994_980 and A6's pinned 54. If they do not match, say by how much and show which row you think accounts for the gap.

===================================================================
FACT SET B. Second stream (the second transaction through the unmount), closed-form state counts.

B1. The segment-length sequence for this stream, copied verbatim from crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs lines 420-422 (script ReuseAfterRaisingFloorThenNormalUnmount):
[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 2]
This is a sequence of 64 segments, in order.

B2. Two-state closed form for this stream, same formula as A2, from crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs line 88:
Pinned value: 6_649_413_719 (constant FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT).

B3. The comment pinning the torn-state rule for this stream, from crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs lines 89-90, quoted in English:
"On this stream, only 46 system-configuration-slot writes take the third (torn) state. Each segment contributes 3^m * 2^(n-m) - 1, where m is the number of system-configuration-slot writes in that segment and n is the segment's total write count; then add 1 for the all-persisted state. The total is about 2.25 times the two-state count. A 30-write segment contributes 3^2 * 2^28 - 1."
Pinned total under this rule: 14_960_689_284 (constant FULL_STATES_THROUGH_THE_UNMOUNT).

B4. The comment pinning the quick-tier count for this stream, from crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs line 99, quoted in English:
"The quick-tier state count for this stream, computed by the same InPlaceSubsetsWithCopyOnWriteNoneOrAll rule given in A5 above; if every write were treated as two-state only (no torn state), this stream's quick-tier count would be 205."
Pinned total under the quick-tier torn-state rule: 390 (constant QUICK_TIER_STATES_THROUGH_THE_UNMOUNT).

TASK for fact set B: using B1's 64-length sequence, B3's rule (46 total torn writes, each occurring in pairs of 2 per occurrence, same as fact set A's pattern of 2-write system-configuration segments and the larger segments that mix 2 torn writes with the rest as unit writes) and A5's quick-tier rule, build a table with one row per segment in B1 (64 rows, numbered 1 through 64 in the order given). For each row, state: the write count n; your classification of m (0 or 2, and for the segments longer than 2, how many of the remaining n-2 writes you treat as copy-on-write unit writes for the purpose of the quick-tier formula); the full-enumeration state count for that row; the quick-tier state count for that row. Then sum all 64 rows' full-enumeration counts and add 1, and separately sum all 64 rows' quick-tier counts and add 1. State whether your two sums match B3's pinned 14_960_689_284 and B4's pinned 390. If they do not match, say by how much and list every row where your classification of m could plausibly be different (i.e., where the segment's write kind is genuinely ambiguous from the facts given here), because that is where the gap most likely lives.

===================================================================
FACT SET C. Admission ckpt_cost for three A4b measurement-table cells.

C1. The general ckpt_cost formula, quoted in English from crates/singlefs-core/src/admission.rs lines 476-481:
"ckpt_cost for the checkpoint reserve pool: the number of fixed-point units an empty publish could write at most, computed against the version this publish would build on, always taken as the worst case. Allocation record tree: two leaf-to-just-below-root paths per device plus the shared root, so devices * 2 * (height - 1) + 1. Central mapping tree: computed level by level from how many paths this publish could change and how many nodes it could split off at most. Accounting tree: the whole tree is rewritten every publish, so its node count for this version. Tree table: 1 (one unit rewritten every publish). The instance-table chain does not enter ckpt_cost."

C2. The allocation-record-tree formula and its worked constant, quoted in English from crates/singlefs-core/src/admission.rs line 574 and lines 579-580:
"ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST = 2. Formula: devices * 2 * (tree_height - 1) + 1. Worked example given in the source: two 4 GiB devices at height 3 gives 9."

C3. Three rows copied verbatim from the measurement table in research/prompts/m2-rev-a4b-implementer-report.md section 3 (each row is one line of test output; the fields you need are h_alloc (allocation-record-tree height), the alloc= field (nodes rewritten per device plus shared root), map= (central-mapping-tree nodes rewritten), acc= (accounting-tree nodes), tt= (tree-table units, always 1), and ckpt= (the pinned ckpt_cost for that row)):

Row "two-4GiB", first row for that cell (label "overwrite@txg4"):
h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0

Row "two-1GiB", first row for that cell (label "overwrite@txg4"):
h_alloc=2 h_map=1->1 acc_before=1 ckpt_by_height=5 ckpt_a4=6 ckpt=8 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=-2 diff_a4=0

Row "two-4GiB-mapping-4-8", first row for that cell (label "overwrite@txg4"):
h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=24 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=-13 diff_a4=2

TASK for fact set C: for each of the three rows in C3, using C1's formula and C2's allocation-record-tree formula, compute: the allocation-record-tree term (devices * 2 * (h_alloc - 1) + 1, with devices = 2 for all three rows) and check it against the alloc= field's node count in that row (sum the nodes named in alloc=, e.g. "d0:2,d1:2+root1" means 2 + 2 + 1 = 5 nodes); the sum allocation-record-tree term + map= + acc= + tt=; and whether that sum equals the row's ckpt= value. Show your arithmetic for each of the three rows separately. If any row's sum does not equal its ckpt=, state the row name and the exact numeric gap.

===================================================================
FACT SET D. Index-node header width formula.

D1. The formula and its parts, quoted in English from crates/singlefs-format/src/lib.rs lines 47, 52, and 30:
"INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86. The header width including the key range and reserved bits is 86 + 2 * (key width) + 29, where 29 is NONCE_MAC_ALGORITHM_RESERVED_BYTES."

D2. Two pinned constants that use this formula, quoted in English from crates/singlefs-format/src/lib.rs lines 58-59 and 64-65:
"INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES: key width 8, pinned value 131."
"EXTENT_TREE_INDEX_NODE_HEADER_BYTES: key width 24, pinned value 163."

TASK for fact set D: compute 86 + 2*8 + 29 and 86 + 2*24 + 29. State whether each matches the corresponding pinned constant in D2.

===================================================================
FACT SET E. Named entries per journal record.

E1. Three constants and one derived constant, quoted in English from crates/singlefs-format/src/lib.rs lines 191, 200, 203-204, and 207-208:
"JOURNAL_RECORD_BYTES = 4096."
"JOURNAL_HEADER_BYTES = 311."
"JOURNAL_NAMED_ENTRY_BYTES is defined as a formula (position entries * 2, plus a type tag 1, plus birth tree 8, plus birth txg 8, plus key tail 10, plus flags 1); the source's own test (line 404) pins its value at 56."
"JOURNAL_NAMED_ENTRIES_PER_RECORD is defined as (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES, using integer (floor) division; the source's own test (line 405) pins its value at 67."

TASK for fact set E: compute (4096 - 311) / 56 using floor division and state the quotient and the remainder separately. State whether the quotient matches the pinned value 67.

===================================================================
General reminder: every task above asks for actual numbers and arithmetic, not a yes/no verdict. For fact sets A and B, if you cannot uniquely determine every row's classification from the facts given, say so explicitly for each ambiguous row rather than silently picking one option — an ambiguous row that you resolve without flagging it counts as an unflagged assumption, which is a different kind of error from a wrong sum.
