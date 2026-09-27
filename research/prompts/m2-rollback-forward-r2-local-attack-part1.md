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

Part 4. Closing questions

After completing the table, answer the following two questions. Number your answers 1, 2. Each answer must point to the specific row in the table above that it is based on, must not be a bare yes or no, and must end with one sentence stating what entry in the table above would falsify that answer.

Question 1. Across all 16 rows of task one, list every row where the current policy and the candidate change give different accept or reject verdicts for the txg = 20 root, item 5 versus item 6 of that row.

Question 2. Across all 16 rows of task one, list every row where disk 1 is in state O or state OU, meaning disk 1 has never held a root carrying F_new, yet the candidate change's F_effective, item 4, still comes out equal to F_new, 30.

Part 5. Formatting rules for your whole answer

Write in English only. Do not use markdown emphasis anywhere, no asterisks, no underscores for emphasis, no backticks. Do not cite any file name or line number anywhere in your answer; when you need to point at a source, use the clause numbers, clause 1 through clause 4, the row labels, Row (disk0=X, disk1=Y), or the plain names disk 0 and disk 1. Do not invent any fact that is not given in Part 1, Part 2, or Part 3; if you find that a genuinely necessary fact is missing, say so explicitly and mark that item UNDEFINED rather than guessing a number. Do not answer any item, row, or question with only yes or no; always give the concrete number, verdict, or missing element in words.
