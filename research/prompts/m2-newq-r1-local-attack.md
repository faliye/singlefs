This is a self-contained fact-filling exercise about a crash-recovery / journal design. Every fact you need is quoted below in English translation. You do not have access to any external files. Answer only from the text given here.

Part 1. Background terms (my own paraphrase, given only for orientation, not a quoted clause)

A filesystem instance keeps a journal (a ring of records) and a small set of on-disk "roots" (checkpoints of the whole tree). At any moment a running instance holds one root value "in memory" (the root it is currently using). Recovery, and a thing called "instance switch" (triggered when a write to a fixed system-config slot fails after the corresponding root has already reached disk), both involve picking a root by re-reading the on-disk state and re-running the normal root-selection procedure (primarily comparing txg, breaking ties by instance number). A "checkpoint" is the batch of not-yet-fully-published work belonging to the current instance; it can be "in flight" (open) or not. A transaction record inside the journal is "committed" if it is self-certifying: it has a nonzero transaction number and its checksum/MAC verifies on at least one of its two mirror copies. An "instance-table row" records, for a past instance i, a triple (i, T_pub, W): T_pub is a txg boundary and W is the highest transaction number of instance i's work that was carried over into a later instance. A "rollback" is an administrator-initiated recovery that discards everything after a chosen old root R_old and starts a new instance from there. The "published predicate" decides, for any stored unit, whether it currently counts as durably published. The "root ring" is a fixed-size on-disk ring holding past root records, which recovery, switch and rollback all choose from. A root's timeline is "abandoned" once a rollback has discarded it: it is not an ancestor of, and not reachable from, the currently valid line of roots. F_effective is a txg floor (defined elsewhere) below which a root is never offered as a rollback target; treat it here as an opaque given threshold, not something you need to resolve.

Part 2. The six clauses

These are English translations of six sentences/passages from two decision records, called D23 decided item 14 (about journal replay, rollback, failure handling and instance switch) and D18 decided item 11 (about the instance-table row format). Treat each clause as literal, authoritative text. Do not import anything from Part 1 into a clause's meaning beyond what the clause itself says.

Clause A (D23 decided item 14, the instance-switch sentence):
"Instance switch = performing one recovery within the mount: take a new instance number, write the row(s), resend the in-flight checkpoint. The selected root is chosen by re-reading the disk: against the still-open device handles, select using the same rule as recovery's root selection (the choose_root function), not the root recorded in the caller's memory (when the root has already been persisted to disk but the system-config-slot step of that publish failed, the root recorded in memory is older than the current root on disk). The row(s) fall only within the half-open range [this root's instance, new instance). For each instance k whose write order is being carried over unchanged, write (k, k's last published txg (0 if it never published a root), the maximum transaction number belonging to k's write order that is being carried over unchanged); the rest follow D18 decided item 11 (when the old instance published a root and the carried-over transactions are all its own, this is exactly the row (old instance, last published txg, the maximum transaction number in the checkpoint being resent)). Transactions numbered less than or equal to W are carried over unchanged (every completed transaction in the in-flight checkpoint is greater than W); transactions numbered greater than W are either redone under the new write order or reported as an error to callers that have not yet returned; fixed-point units are all rewritten under the new instance. It does not fall back to read-only, and it does not wait for the next mount. Cost = one row plus rewriting the entire instance-table chain plus rewriting data units numbered greater than W plus one fixed-point redo; frequency not measured."

Clause B (D23 decided item 14, note 4, about W):
"On switch, W is taken as the maximum transaction number in the checkpoint being resent: every completed transaction in the open checkpoint is greater than W, and is redone under the new write order. Those records are not after the selected root. If W were taken literally as 'the last applied transaction number', their units would be judged already-published by the predicate and nobody would apply them. When the checkpoint being resent contains no record with a nonzero transaction number (the resend is then an empty publish), W = 0 (transaction number 0 never enters the max used for W). W is written on the row of each instance whose write order is being carried over unchanged. The selected root for the switch is taken per the instance-switch sentence (Clause A): re-read from disk, not from memory."

Clause C (D18 decided item 11, "the row a switch writes"):
"Instance switch writes rows the same way a crash does, except the selected root is taken per the instance-switch sentence of D23 decided item 14: chosen by re-reading the disk, not the root recorded in the caller's memory. A switch does not replay the journal. Rows fall only within [this root's instance, new instance), one row per instance, taking the first applicable rule of the following four, in order: (1) the row(s) that the publish being resent itself already wrote are resent as-is (this covers: the recovery row written by an earlier row-writing pass, or the rollback row and intermediate-instance rows written by an earlier rollback); (2) for the instance k whose write order is being carried over unchanged, write (k, k's last-published root's txg (0 if it never published a root), the maximum transaction number belonging to k's write order that is being carried over unchanged); (3) the instance of the selected root writes (i, this root's txg, 0); (4) every other instance writes (i, 0, 0). Rule (2) does not apply at all when the checkpoint being resent contains no record with a nonzero transaction number."

Clause D (D18 decided item 11, "the published predicate, globally"):
"Let i_now be the instance of the mounted root. For a stored unit with write-order (i, n) and birth generation b: if i is less than i_now and there is no instance-table row for i, the unit is published. If there is a row (i, T_pub, W) for i: for a data unit, it is published if and only if b is less than or equal to T_pub, or n is less than or equal to W (meaning either it lies within the selected root, or it was applied by replay); for an index-node or packed-record unit, it is published if and only if b is less than or equal to T_pub (fixed-point units that come after the root that replay ends on are all unpublished, and the recovering instance rewrites them; the fixed-point units belonging to the publishes that replay did apply are referenced by the new-root segment carried in the journal record, and are judged published using T_pub equal to the txg of the root that replay ends on; for a packed-record unit the 6-byte transaction number does not enter the published predicate at all -- it only breaks ties within the same checkpoint; an index-node's write-order has already been narrowed down to instance number only). If i equals i_now, the unit is published if and only if b is less than or equal to the mounted root's txg. If i is greater than i_now, the unit is judged corrupt. The rule 'no row implies published' holds only within the same timeline."

Clause E (D23 decided item 14, "the explicit exception: administrator rollback", the part about choosing R_old):
"Rollback is one recovery: out of band, the administrator picks an old root R_old from the rollback candidate set. The candidate set consists of roots in the root ring that the instance table judges still valid, and whose txg is greater than or equal to F_effective. A root (i, T) is selectable if and only if the instance table has no row for instance i, or has a row (i, Ti, Wi) for instance i with T less than or equal to Ti (roots belonging to an abandoned timeline are selectable under neither condition). No record after R_old is applied. A new instance number is taken. On the version of the instance table that R_old points to, the rollback row (r_old, T_old, 0) is written, together with the intermediate instances' rows (i, 0, 0)."

Clause F (D23 decided item 14, the note listing three undecided things caused by switch re-reading disk):
"Three things brought on by making instance switch re-read the disk to choose its root have no governing clause, and are broken out as a separate open question: (i) when the root chosen by re-reading disk is newer than the root recorded in memory, is the in-flight checkpoint that would otherwise be resent, still resent or not; (ii) which root -- the one chosen by re-reading disk, or the one recorded in memory -- does the row and the W value get written against; (iii) in the publish that constitutes a rollback, how does the root chosen by re-reading disk correspond to the R_old that the administrator chose."

Part 3. The fact table you must fill

There are two independent axes.

Axis 1, the relation between the root chosen by re-reading disk and the root recorded in memory, has four values:
R1 = the two are equal (same instance and same txg).
R2 = the root chosen by re-reading disk is newer than the root recorded in memory (using the normal root ordering: compare txg first, break ties by instance number).
R3 = the root chosen by re-reading disk is older than the root recorded in memory (same ordering).
R4 = there is no root recorded in memory at all.

Axis 2, the state of the in-flight checkpoint at the moment of the switch, has three values:
K1 = there is no in-flight checkpoint.
K2 = there is an in-flight checkpoint, and it contains no committed transaction record yet (no journal record with a nonzero transaction number that is self-certifying).
K3 = there is an in-flight checkpoint, and it contains at least one committed transaction record (at least one journal record with a nonzero transaction number that is self-certifying).

Crossing the two axes gives twelve rows:
Row 1 = R1,K1. Row 2 = R1,K2. Row 3 = R1,K3.
Row 4 = R2,K1. Row 5 = R2,K2. Row 6 = R2,K3.
Row 7 = R3,K1. Row 8 = R3,K2. Row 9 = R3,K3.
Row 10 = R4,K1. Row 11 = R4,K2. Row 12 = R4,K3.

The columns are the six clauses: Clause A, Clause B, Clause C, Clause D, Clause E, Clause F.

Part 4. What to produce for the table

For every one of the 72 cells (12 rows times 6 clauses), decide one of exactly two outcomes:

ACTION -- the clause's own wording settles what happens in that row. If you answer ACTION, quote or closely paraphrase the exact action or value the clause gives for that specific row (which root is used, what tuple is written, what W is, whether something is published, whether something is resent, and so on). A generic answer such as "it follows the clause" is not acceptable; state the concrete action. If the clause's text treats several rows the same way (for example, if its wording does not change across checkpoint states), say so explicitly and still give the concrete action, do not just say "not dependent".

UNDEFINED -- no sentence in Clause A through Clause F settles that row. If you answer UNDEFINED, name exactly which clause (or clauses) you looked at and say precisely what information is missing that would be needed to settle it. Do not answer UNDEFINED merely because settling it takes some reasoning; only answer UNDEFINED when the six clauses' text genuinely does not decide it.

Do not answer with only "yes" or "no" anywhere in the table. Every cell needs the concrete action or the concrete missing element, spelled out in words.

Produce the table row by row. For each of the 12 rows, write a line "Row n (R_, K_):" followed by six sub-lines, one per clause, in the form "Clause X: ACTION -- <text>" or "Clause X: UNDEFINED -- <text>". Cover all 12 rows and all 6 clauses per row; do not skip any cell and do not merge cells together.

Part 5. Three questions to answer after the table

After completing the full table, answer these three questions. Number your answers 1, 2, 3. Each answer must be derived only from what you wrote in the table (point to the specific row(s) and clause(s) you are using), must not merely say yes or no, and must end with one sentence stating what observation in the table above would falsify that answer.

Question 1. After instance switch is changed to choose its root by re-reading the disk: when the root chosen by re-reading disk is newer than the root recorded in memory, is the in-flight checkpoint that would otherwise be resent, still resent or not?

Question 2. In that same situation, does the instance-table row and the W value that the switch writes get written against the root chosen by re-reading disk, or against the root recorded in memory?

Question 3. In the publish that constitutes a rollback, how does the root chosen by re-reading disk correspond to the R_old that the administrator chose?

Part 6. Formatting rules for your whole answer

Write in English only. Do not use markdown emphasis anywhere (no asterisks for bold or italic, no backticks). Do not cite any file name or line number in your answer; when you need to point at a source, use the clause labels (Clause A through Clause F), the row labels (Row 1 through Row 12, R1 through R4, K1 through K3), or the names already used in this prompt (D23 decided item 14, D18 decided item 11). Do not invent new facts that are not stated in Part 1 or Part 2; if you need a fact that is not given here, say so and mark the cell UNDEFINED rather than guessing.
