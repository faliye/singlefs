Round: m2-closeout-code-r2, local attack leg (arithmetic only). You are given a set of hard facts copied verbatim (in English translation) from source files in this repository, each with its file path and line number. Your job is to fill in every cell of the tables below with a computed number, show the arithmetic step that produced it, and state whether it matches the pinned value given for that row. Do not answer yes or no anywhere; every cell needs an actual number and the arithmetic that produced it.

Rule for your answer: do not write any file:line citation and do not write any code line number in your answer. When you need to point at something, name the function, the constant, or the table row label instead (for example: write "the row for the 159 MiB ring", not a line number).

For every fact set below, also write one sentence: what observation would show this item's derivation is wrong (a mismatch between the arithmetic you show and the pinned value counts as "wrong").

===================================================================
FACT SET A. First stream (the first transaction), segment sequence and closed form, before and after a barrier change.

A1. The new segment-length sequence for this stream, quoted from research/prompts/m2-impl-c577-barrier-implementer-report.md:
"First stream: 2+2+1+2+2+1+26+2+1+2 (67108885) becomes 2+2+1+2+2+1+2+24+2+1+2. Closed form = 1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 - 1) = 16777240; after the change, the in_place_overwrite test prints exactly 16777240."
This gives you two sequences of 10 segments each: the old sequence [2,2,1,2,2,1,26,2,1,2] pinned to 67108885, and the new sequence [2,2,1,2,2,1,2,24,2,1,2] pinned to 16777240.

A2. The closed-form formula, quoted from the same report:
"closed form = 1 + sum over each segment of (2^(segment length) - 1)"

TASK for fact set A: build a table with one row per segment (10 rows, numbered 1 through 10 in the order given), one column for the old sequence's segment length and one for the new sequence's segment length. For each row and each column, compute 2^n - 1 where n is that column's segment length for that row. Sum all 10 rows' old-column values and add 1; separately sum all 10 rows' new-column values and add 1. State whether your two sums match the pinned 67108885 (old) and 16777240 (new). If either does not match, say by how much and which row you think accounts for the gap.

===================================================================
FACT SET B. Step counts for two shorter paths, before and after the same barrier change.

B1. Quoted from research/prompts/m2-impl-c577-barrier-implementer-report.md:
"Every publish adds one pool barrier after the rotation; the recording stream logs it per device: two more steps overall (one Barrier per disk, two disks)."

B2. Quoted from the same report:
"Isolated-view path: the first transaction's 24+2+1+2 segment sequence is unchanged, the last segment's write kinds gain barrier x2, step count 33 becomes 35. The warm-up path's step count 18 becomes 20; its segment sequence 2+1+2+2+1+2 is unchanged."

TASK for fact set B: using B1's rule (two more steps per publish, one Barrier per disk, two disks), compute old_count + 2 for the first-transaction path (old count 33) and for the warm-up path (old count 18). State whether your two results match the pinned new counts 35 and 20 given in B2. If either does not match, say by how much.

===================================================================
FACT SET C. The sigma segment's closed form, before and after the same barrier change.

C1. Quoted from research/prompts/m2-impl-c577-barrier-implementer-report.md:
"sigma (record_checker): (18, 16) becomes (16, 16); the sigma segment's closed form 2^18 becomes 2^16. The crash-case row c561-sigma-full's registered note 'closed form 2^18' must change along with it."

TASK for fact set C: compute 2 raised to the 18th power and 2 raised to the 16th power. State each result as a plain integer. State which of the two values is the pinned old value and which is the pinned new value, according to C1.

===================================================================
FACT SET D. Unit area start as a function of the journal ring size.

D1. Quoted from research/prompts/m2-rev-a3b-implementer-report.md:
"Under the default 768 MiB ring, the start is still 50176."

D2. Quoted from the same report:
"I take a 6 MiB ring: start = 1024 + 384 = 1408 = 22 x 64, on the segment boundary."

D3. Quoted from research/prompts/m2-rev-a3b-fallout-1-implementer-report.md:
"Change the ring to 159 MiB: start = 1024 + 159 x 64 = 11200 = 50176 - 3 x 12992. 12992 = lcm(812, 64), so the start's position within its leaf and within its segment is the same as 50176's: 168 slots before leaf 13's last slot 11367, same as the default ring's 168 slots before leaf 61's last slot 50343. 159 MiB <= 1 GiB / 4."

D4. The general form you should use, read off from D2 and D3's own arithmetic: start = 1024 + (ring size in MiB) x 64.

TASK for fact set D: for each of the three ring sizes (768 MiB, 6 MiB, 159 MiB), compute start using D4's formula and state whether it matches the pinned value (50176, 1408, 11200 respectively). For each of the three results, also compute result mod 64 and state whether it is 0 (this is the "on a 64-slot segment boundary" requirement — a nonzero remainder means the row fails that requirement). Separately: compute lcm(812, 64) from its prime factorization and state whether it equals 12992; then compute 50176 - 3 x 12992 and state whether it equals 11200.

===================================================================
FACT SET E. In-flight record limit for a 1 MiB journal ring.

E1. Quoted from crates/singlefs-format/src/lib.rs:
"pub const JOURNAL_SAFETY_FACTOR: u64 = 3;"

E2. Quoted from research/prompts/m2-rev-a3b-fallout-1-implementer-report.md:
"The per-publish record-count-limit test is now measured against a 1 MiB ring's in-flight limit of 85: 85 records are admitted, 86 are rejected; the write-row cell is 85 instance-table shards plus the allocation-record tree's nodes."

E3. The formula, read off from E2's own label "in-flight limit": in_flight_limit = floor(ring_bytes / 4096 / JOURNAL_SAFETY_FACTOR), where a 1 MiB ring is 1048576 bytes and JOURNAL_SAFETY_FACTOR is E1's constant.

TASK for fact set E: compute 1048576 / 4096 first, and state whether that division has a remainder. Then divide that quotient by 3 (E1's constant) using floor (integer) division, and state the quotient and the remainder of that second division separately. State whether the final floored quotient matches the pinned value 85. Separately, state whether "85 admitted, 86 rejected" (E2) is consistent with your computed floor value being the maximum admitted count.

===================================================================
FACT SET F. Bounds on system-configuration fields read at mount (invariant I-7.13).

F1. Quoted from .claude/kb/invariants.md:
"Fixed structure slot spacing >= 4096 bytes (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES) and slot 1's whole slot falls before the root ring base address; physical_block_size in [457 (ROOT_RECORD_BYTES), slot spacing]; journal ring bytes / 4096 / F (JOURNAL_SAFETY_FACTOR) >= 1 and the ring's end does not cross the unit-area start slot number."

F2. Quoted from .claude/kb/decisions/22-单元原子性怎么合成.md, which is the definition F1 points at for the numeric form of the slot-1 upper bound:
"When the reader selects a system configuration, it judges the root slot width and the fixed structure slot spacing: fixed structure slot spacing in [4096, 1 MiB - 4096] (the upper bound is that slot 1's whole slot falls before the root ring base address, root_ring::region_start(0) minus the 4096-byte system-configuration slot width); root slot width (physical_block_size) in [457-byte root record width (item 7), slot spacing]; out-of-range values reject the whole pool at mount."

F3. E1's constant applies here too: JOURNAL_SAFETY_FACTOR = 3.

TASK for fact set F: compute the numeric upper bound for the fixed structure slot spacing, 1 MiB minus 4096 bytes (1 MiB = 1048576 bytes), and state the result as a plain integer. Separately, using E3's formula from fact set E (floor(ring_bytes / 4096 / 3) >= 1), find the smallest whole number of bytes ring_bytes can be for this inequality to hold: show your arithmetic for ring_bytes = 12288 (does floor(12288/4096/3) equal 1 or more?) and for ring_bytes = 12287 (does floor(12287/4096/3) equal 1 or more?), and state which of the two is the smallest passing value based on your two computations.

===================================================================
General reminder: every task above asks for actual numbers and arithmetic, not a yes/no verdict. If any fact set gives you two formulas that could be applied in more than one order (for example fact set E's two divisions), show both intermediate results explicitly rather than only the final number, so a mismatch can be traced to a specific step.
