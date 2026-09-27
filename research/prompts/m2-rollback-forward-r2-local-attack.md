Part 1. Background (my own paraphrase, for orientation only, not a quoted clause)

A pool has two disks, called disk 0 and disk 1. The filesystem keeps a fixed-size ring of on-disk root records; each root record is a full checkpoint of the tree at some txg, a counter that increases by exactly 1 with every publish, including publishes that write no user-visible change, called empty publishes. The ring has a fixed number of physical regions; each region always belongs to the same physical disk, fixed at format time, and a publish's txg determines which region it lands in, and therefore which disk. Every root record carries, among other fields, its own copy of a value called F, the rollback floor. Once raised (subject to the rule given in clause 2 below), F acts as a threshold: an administrator's rollback may only target a root whose txg is at or above the currently effective value of F, called F_effective. The terms instance and checkpoint are not needed for this exercise and are not used below.

For this exercise, assume no earlier raise-F attempt has ever completed, so F_old = 10 is the value carried by every root written before the raise-F attempt under discussion, and F_new = 30 is the newly raised value that this raise-F attempt is trying to establish. If, in a given row of the table in Part 3, the condition for F to take effect (stated in clause 2) does not hold, assume F_effective simply stays at F_old = 10, since nothing has previously raised it.

Part 2. Quoted clauses (translated; treat each as literal, authoritative text)

Clause 1 (D16 decided item 1, a known-edge-case note; only part of the source sentence is given here, and the rest must not be assumed):
"F only lives in the root record; root slots are not mirrored. At the moment a raise takes effect, it is possible for a given disk to have only one persistent root carrying the newly raised value."

Clause 2 (D16 decided item 1, the takes-effect row, current policy):
"It only takes effect once every surviving disk carries a persistent root with the new F. Once that holds, the effective value after recovery equals the minimum, taken across surviving disks, of each disk's own maximum carried F."

Clause 3 (D16 decided item 1, the rollback candidate set row):
"A root is a rollback candidate if and only if the instance table still judges it valid, and its txg is greater than or equal to F_effective."

Clause 4 (this round's own framing of a candidate change to clause 2; not itself a quote from a decision record):
"The candidate change replaces only the value-computation part of clause 2: instead of the minimum, taken across surviving disks, of each disk's own maximum carried F, the effective value after recovery is the maximum, taken across surviving disks, of each disk's own carried F. Nothing else in clause 2's wording is stated as changed. Whether clause 2's leading precondition, that every surviving disk must carry a persistent root with the new F, still governs the candidate change is exactly one of the things this exercise asks you to work out; do not assume either way before you reach the relevant row. This candidate change does not alter the on-disk format."

Clause 5 (D22 decided item 2, root-ring geometry, region count):
"Number of regions R: 3."

Clause 6 (D22 decided item 2, root-ring geometry, slot order, the part that determines which region a txg lands in):
"Cross-region rotation: region equals txg mod R."

Clause 7 (D16 decided item 8, the warm-up paragraph; the region-to-disk assignment, and separately the general counting rule used for later raises):
"The root ring's three regions have their device ownership fixed at format time as 0, 1, 0, meaning region 0 belongs to disk 0, region 1 belongs to disk 1, and region 2 belongs to disk 0; so txg 1 lands in region 1, disk 1, and txg 2 lands in region 2, disk 0. For later mounts, after a rollback, and when the mount crashes partway through and is retried, the count is worked out on the spot: each subsequent empty publish's txg is one more than the previous one, it lands on whichever disk the root-ring placement gives it, and the sequence continues until this instance's root covers every disk, up to R times."

Clause 8 (this round's candidate B1, quoted from the material given to you):
"B1: unmounting sets a separate ceiling, each disk's newest persistent valid root. At shutdown, the fallback leaves only the newest root, one per disk, two disks."

Part 3. Task one: fill in a 16-row table comparing the current policy and the candidate change

Consider the moment recovery looks at the pool's state, after a raise-F attempt has been made. Each of the two disks, independently, is in one of four states, describing that disk's own most recently written root record and whether it is currently readable. In every one of the 16 rows below, treat both disks as currently surviving, meaning both are present and available to contribute to the gate and the formulas in clause 2, clause 3, and clause 4; the word unreadable, in every state below, refers only to one specific root record being unreadable, never to the whole disk being gone.

State N: the disk's newest root record carries F_new (30) and is currently readable.
State NU: the disk's newest root record was written carrying F_new (30), but that particular root record is currently unreadable. Given premise: the disk still has an earlier root record that is currently readable, and that earlier one carries F_old (10).
State O: the disk's newest root record carries F_old (10) and is currently readable; this disk has not received a root carrying F_new yet.
State OU: the disk's newest root record carries F_old (10) and is currently unreadable. Given premise: the disk still has an earlier root record that is currently readable, and, because nothing on this disk has ever carried anything but F_old, that earlier one also carries F_old (10).

There are 16 rows: disk 0's state (N, NU, O, or OU) crossed with disk 1's state (N, NU, O, or OU).

For every one of the 16 rows, work out and report all of the following:
1. The F value disk 0 currently contributes, 10 or 30, and the F value disk 1 currently contributes, 10 or 30.
2. Whether the precondition in clause 2, that every surviving disk carries a persistent root with the new F, holds in this row. Name which disk, if any, fails it.
3. F_effective under the current policy, clause 2 as printed, as a number, with one sentence showing the arithmetic.
4. F_effective under the candidate change, clause 4, as a number. State explicitly, in one sentence, whether you are treating clause 2's leading precondition as still governing the candidate change or not in this row, and why you read it that way.
5. The rollback candidate set under the current policy, using clause 3's rule with the F_effective number from item 3 (name it as: every instance-table-valid root whose txg is at or above that number). Then state whether a root at txg = 20, itself instance-table-valid, would be accepted or rejected by that set, with the one-line arithmetic comparing 20 to the number from item 3.
6. The same as item 5, but using the candidate change's F_effective number from item 4.

Write the table row by row. Use the heading "Row (disk0=X, disk1=Y):" for each of the 16 rows, X and Y each one of N, NU, O, OU, followed by six numbered sub-lines answering items 1 through 6 above. Cover all 16 rows; do not skip or merge any of them, even when two rows give identical numbers; if two rows are identical, say so explicitly and still show the arithmetic for each.

Part 4. Task two: how many empty publishes does B1's unmount sequence perform

Under candidate B1, clause 8, unmounting performs a chain of consecutive empty publishes to raise every disk's newest persistent valid root to a freshly written one. Using clause 6's rule, region equals txg mod R, with R = 3 from clause 5, and clause 7's region-to-disk assignment, region 0 to disk 0, region 1 to disk 1, region 2 to disk 0, and the same per-publish counting method clause 7 gives for later mounts: let t be the txg of the most recently persisted root at the moment unmounting begins. The first empty publish in this chain uses txg = t + 1, the second uses t + 2, and so on, each one landing on the disk that its own txg's region maps to, continuing until every one of the two disks has received at least one of these new empty publishes.

Because only t mod 3 determines the sequence of regions that follow, there are exactly three cases to work out: t mod 3 = 0, t mod 3 = 1, and t mod 3 = 2.

For each of the three cases, write a line "t mod 3 = k:" followed by, for each empty publish in order starting from txg = t+1: its txg expressed as t+1, t+2, and so on; the region that txg lands in, as a residue mod 3; and the disk that region maps to. Stop listing publishes as soon as both disks have appeared at least once, and then state the total number of empty publishes performed in that case. Show the reasoning for every step; do not just state the final count. As a consistency check, verify that none of your three case counts exceeds R, 3, as given in clause 7; if one does, re-examine your reasoning before reporting it.

Part 5. Closing questions

After completing both tables, answer the following three questions. Number your answers 1, 2, 3. Each answer must point to the specific row or case in the tables above that it is based on, must not be a bare yes or no, and must end with one sentence stating what entry in the tables above would falsify that answer.

Question 1. Across all 16 rows of task one, list every row where the current policy and the candidate change give different accept or reject verdicts for the txg = 20 root, item 5 versus item 6 of that row.

Question 2. Across all 16 rows of task one, list every row where disk 1 is in state O or state OU, meaning disk 1 has never held a root carrying F_new, yet the candidate change's F_effective, item 4, still comes out equal to F_new, 30.

Question 3. Across the three cases in task two, state the smallest and the largest total number of empty publishes found, and which case or cases achieves each.

Part 6. Formatting rules for your whole answer

Write in English only. Do not use markdown emphasis anywhere, no asterisks, no underscores for emphasis, no backticks. Do not cite any file name or line number anywhere in your answer; when you need to point at a source, use the clause numbers, clause 1 through clause 8, the row labels, Row (disk0=X, disk1=Y), the case labels, t mod 3 = k, or the plain names disk 0 and disk 1. Do not invent any fact that is not given in Part 1, Part 2, Part 3, or Part 4; if you find that a genuinely necessary fact is missing, say so explicitly and mark that item UNDEFINED rather than guessing a number. Do not answer any item, row, or question with only yes or no; always give the concrete number, verdict, or missing element in words.
