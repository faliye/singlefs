This is a self-contained arithmetic exercise about a copy-on-write filesystem's crash-recovery ring, its rollback-floor bookkeeping, and its B-tree geometry. Every fact you need is given below in English translation, quoted from a set of frozen design decisions. You do not have access to any external files, source code, or the original Chinese text. Answer only from the text given here. If a number you would need is not given here, say exactly that, and do not guess it or invent a plausible-looking value.

PART 0. Orientation glossary (my own paraphrase, given only so the vocabulary below makes sense; this part is not a quoted clause and carries no numeric authority)

A publish is one atomic batch of change to the filesystem's persistent state; each publish is stamped with a strictly increasing counter called checkpoint_txg (txg for short). A generation is the same thing as a txg: generation N means "as of checkpoint_txg = N". A root is a small on-disk record written once per publish; it is the anchor from which the entire tree of that generation can be reached. Roots live in a root ring: a fixed number of on-disk regions, each holding a fixed number of slots, so the ring has a fixed total capacity in root-record slots; a new publish always writes its root into the next slot in ring order, so once the ring wraps around, the oldest root record gets overwritten (its slot's bytes are physically replaced, whatever generation it held is gone). An instance table records, out of band, which older generations have been thrown away by an administrator rollback and are therefore no longer legitimate targets to recover onto, even if their root-record bytes still happen to be intact somewhere in the ring. A root is valid if its bytes still self-certify (checksum passes) and the instance table does not say its generation has been thrown away. A root is non-empty if the user-visible trees it points to actually differ from the immediately preceding valid root (as opposed to being an identical restatement produced by, e.g., an administrative housekeeping publish that changed nothing user-visible). An empty publish is a publish that writes a new root and updates internal bookkeeping trees but does not change any user-visible tree. The rollback floor, called F in the material below, is a value stored inside the root record; blocks freed at a generation at or below F (or below the ring's oldest valid root, whichever bound is looser) are allowed to be handed back out to new writes; raising F is therefore how old, otherwise-unreachable generations' space gets reclaimed. A unit, in this material, means one distinct on-disk object that a publish COW-rewrites (a tree node, a tree-table object, a root record, and so on); "how many units" means a count of such distinct rewritten objects, not a byte count. A B-tree node has a fan-out (how many children an internal node can have) and the tree has a height (how many nodes you pass through walking from the root down to a leaf, root and leaf both counted). A birth generation is a number stamped on a block or a tree node recording which generation created (most recently rewrote) it.

PART 1. The redesign under evaluation, stated so the three questions below make sense

The administrator rollback operation is being redefined. Today, choosing to roll back to an old root R_old means literally discarding everything published after R_old. Under the redesign, rolling back to R_old instead becomes one more forward publish: the new root points at the same tree that R_old pointed to, txg is incremented as usual, and the generations that used to be "newer than R_old" simply become, on the resulting timeline, generations that are older than the new current generation. Separately, a new operation, "unmount", is being added (it does not exist in the code today); under the semantics called B, a normal (clean) unmount raises the rollback floor F all the way up to the newest root, and the space belonging to now-unreachable older generations is released immediately at that point. Snapshots are excluded from this and are guaranteed to survive permanently regardless (snapshots are out of scope for the current milestone and are not discussed further in this material).

PART 2. Frozen facts, quoted from the decisions

Each fact below is labeled so you can refer back to it by label (for example, "D16-1 ceiling row") instead of re-quoting it. Treat every fact as literal and authoritative. Do not import anything from Part 0 into a fact's meaning beyond what Part 0 itself says; Part 0 is orientation only.

D16-1-a (rollback candidate window, decision D16 "publish semantics", established item 1):
"The rollback candidate set retains the most recent 4 distinct reachable states -- only roots that changed user-visible state are counted; roots produced by drain-empty-publishes or by raising the floor do not count. The window's unit is publishes, not seconds: the defer window protects whether a root in the ring can still be used, and none of the surveyed consumers price this in seconds."

D16-1-b (ring's oldest valid root, same item):
"Ring's oldest valid root: the minimum checkpoint_txg among roots that are self-certifying and valid across all root slots on disk, and are still judged valid by the instance table; a slot whose write failed is counted by its old content; an in-flight, not-yet-durable publish does not count."

D16-1-c (rollback floor F, same item):
"Rollback floor F: the root record carries 8 bytes for it. Under normal circumstances it does not move, and the window is the entire root ring; it is only raised when admission is insufficient."

D16-1-d (ceiling for raising F, same item):
"Ceiling for raising F: min(each disk's newest persistent valid root, the 4th-newest non-empty persistent valid root); when there are fewer than 4 non-empty valid roots, the oldest valid root is taken instead. The target of one round of handling = min(the release-generation being freed this time, the 4th-newest non-empty root)."

D16-1-e (taking effect, same item):
"Taking effect: it takes effect only once every surviving disk carries a persistent root with the new F; after recovery, the effective value = the minimum, across surviving disks, of the maximum F each disk carries."

D16-1-f (rollback candidate set, same item):
"Rollback candidate set: judged still valid by the instance table AND txg is greater than or equal to F_effective."

D16-1-g (the raise-F string, same item):
"The raise-F string: before touching disk, the whole string of empty publishes is rehearsed on a copy of the allocator; if any single step cannot find a placement, zero publishes are issued at all, and the allocator copy is swapped back to the one from before raising F (error name: RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite); when the real publishing runs and a disk write goes wrong partway, the caller is told how many publishes had already landed (error name: RaiseFloorSequencePublishFailed); there is no governing clause for how slots held back in that case get released."

D16-1-h (admission, same item):
"Admission: allocatable = min(reallocatable + live metadata - reserve pool, df); when admission is insufficient, empty publishes are pushed first to raise F (this push does not happen before the row-writing publish, because that is the new instance's first publish and its metadata goes through the switch reserve instead; user-data redo within the same publish still goes through admission and returns ENOSPC if insufficient); one admission attempt issues at most B_adm = 4 + 2 x k_tol publishes (k_tol = 2, so B_adm = 8); only after doing the full B_adm and still being short is ENOSPC reported."

D16-1-i (ring-depth lower bound, same item):
"Ring-depth lower bound: slots per region S is greater than or equal to k_tol + 2 = 4."

D16-1-j (how "non-empty" is recognized on disk, same item):
"How non-empty is recognized on disk: a valid root counts as non-empty if and only if the root pointers of the user-visible trees (the inode tree and the extent tree) in its tree table differ from those in the previous valid root's tree table (ordered by txg; valid = judged still valid by the instance table and txg greater than or equal to the current F). What is compared is these two trees' entries within the tree table, not where the tree-table unit itself lands on disk -- every publish, including an empty one, rewrites the tree-table unit."

D16-9 (an empty publish still writes units, decision D16 established item 9):
"As long as the accounting tree exists, an empty publish still writes it -- an empty publish is still a publish, so it rewrites the accounting rows, and along with them the accounting-tree nodes, the allocation records, the mapping entries, and the tree-table unit. During the warm-up of the first writable mount, the tree table has 0 entries, so this is zero units; later empty publishes cost c_max blocks, computed at the time from the current structure (see D28-4 below). On the version where the tree table has 0 entries, the publish that writes the row writes exactly two units: the instance table and one allocation-record node (the allocation-record tree's root pointer lives in the root record itself, not in the tree table)."

D23-14-a (candidate set restated, decision D23 "journal role and format", established item 14):
"Candidate set = roots in the root ring that are judged still valid by the instance table and whose txg is greater than or equal to F_effective."

D23-14-b (rollback depth, same item):
"Rollback depth is set by the candidate set (txg greater than or equal to F_effective): under normal circumstances it is the entire root ring; when disk space is tight it can shrink to the most recent 4 rollback-reachable states, and every disk's newest persistent valid root is always included in it."

D23-14-c (no clean-shutdown marker, same item):
"The first version has no clean-shutdown marker: every mount goes through recovery (full ring scan, per-record verification, root selection) regardless of whether the previous session ended cleanly; admission, warm-up, the rollback candidate set, and whether F takes effect all read no such marker."

D22-2-a (region count, decision D22 "how unit atomicity is composed", established item 2):
"Region count R: 3 (R = F_fault + 1, F_fault = 2)." Note added for you, not part of the quoted clause: this F_fault is a fault-tolerance count sizing how many ring regions exist; it is a different quantity from the rollback floor F defined in D16-1-c above, even though the source material's own notation happens to call both of them "F" in their respective places. Do not conflate the two in your answer; if you need to refer to the fault-tolerance count, call it F_fault.

D22-2-b (slots per region, same item):
"Slots per region S: lives in the system configuration. Lower bound k_tol + 2 = 4 (per D16-1-i, S must be large enough to hold 4 states plus one round of handling that can include up to k_tol root-slot write failures; this bound is sized directly on S, not on R x S); the upper bound is computed at mount time from however many blocks are pinned down by a separate invariant; picking any value between 4 and 16 touches no edge case. Exactly what value to use within that range is not a format decision -- the format only commits to 'S is a system-configuration field, checked against a range at mount time'."

D22-2-c (reserved slots, same item):
"Reserved slots: none are reserved."

D8-14-a (allocation-record tree geometry, decision D8 "core index structure", established item 14):
"Allocation-record tree: leaf width W = 812 (this is exactly the leaf entry capacity, (16384 - 135) / 20); internal entries are 96 bytes each (a 10-byte key plus an 86-byte child pointer), giving fan-out 169. A node at level L covers W x 169^L slots. The root level is the smallest R_level >= 1 such that the sum, over disks, of ceil(slot count on that disk / (W x 169^(R_level - 1))), is less than or equal to 169. Worked example given in the material: with two 4 GiB disks, the root sits at level 2, so the tree height is 3."

D8-14-b (extent tree geometry, same item):
"Extent tree, upper segment: a leaf covers 143 inodes; internal entries are 110 bytes (a 24-byte key plus an 86-byte child pointer), giving fan-out 147. Extent tree, lower segment: a leaf covers 144 data units; the quoted material does not give a fixed internal fan-out number for the lower segment."

D8-11 (tree height rule, decision D8, established item 11):
"Tree height equals the level number stored in the root node's self-describing header, plus 1."

D5-2 (what a generation is, decision D5 "snapshot and space-accounting mechanism", established item 2):
"Generation is the checkpoint number (checkpoint_txg, incremented by 1 on every publish; so a generation advances with publishes, not with a wall-clock window). Each root's accounting tree holds only that root's own generation's rows -- it does not keep multiple generations' rows at once."

D5-13 (birth generation and the deadlist model, decision D5, established item 13):
"Each block carries its own birth generation (birth txg). On deletion, the block's birth generation is compared against the live head's recorded previous_snapshot_txg (the previous snapshot's generation): if birth is greater than previous_snapshot_txg it is released directly; otherwise it is appended to the live head's own deadlist."

D5-14 (the reference-interval condition, decision D5, established item 14):
"Block b is referenced by snapshot S if and only if birth(b) is less than or equal to S.txg, and S.txg is less than death(b) (left-closed, right-open).
birth(b): the checkpoint number at which the block was published -- this is not the open txg in effect when the write request was issued (that number can be resent after a crash).
death(b): the checkpoint number at which the last live reference to the block was removed and that removal was published; if the block is still reachable in a live tree, death(b) = infinity. This is not the moment a delete call returns, and not the moment the space becomes reallocatable.
S.txg: the last published txg that snapshot S captured -- not the txg the create operation happened to be issued in."

D18-18 (node header field, decision D18 "what information a block carries", established item 18, one row of its field table):
"A code-2 (B-tree) node's self-describing header carries, among other fields at fixed offsets: tree ID, level, key width, key range, and then a birth-generation field, followed by fsid, write-sequence, birth sequence number, payload CRC, and reserved bits." The quoted material lists this birth-generation field as part of every such node's header, but the text quoted here does not spell out whether this per-node field is refreshed to the current generation every time the node is rewritten by a later publish, or whether it is left holding an older value under some other rule. Treat that as not given: do not assume either behavior when you use this fact.

D22-7 (root record, two of its fields, decision D22, established item 7):
"Rollback floor F: 8 bytes, stored in the root record. Checkpoint_txg: 8 bytes, stored in the root record; publish count, rotation key, accounting generation, and replay watermark are all the same number -- there is no separate 'publish number' field." The same established item also states the root record is written in full on every publish (it is the fixed-size anchor record for that publish), and that it holds, in addition to F and checkpoint_txg, pointers to the tree-table unit, the instance-table unit, the central-mapping-tree root, and the allocation-record-tree root (the last of these lives directly in the root record rather than being indexed through the tree table).

D28-4 (checkpoint reserve-pool cost formula, decision D28 "mount-time commitments", established item 4):
"The checkpoint reserve pool has no constant upper bound; it is computed at the time from the current structure, whatever size that structure implies. Its shape: ckpt_cost = (sum, over the allocation-record tree and the central-mapping tree, of that tree's current height) + (the accounting tree's nodes rewritten per publish) + 1 (for the tree-table unit); recomputed on every publish using the tree height at that time. Each code-2 tree's current height = the level read at publish time from its root node's self-describing header, plus 1 (this is the same rule as D8-11); the quoted material explicitly states that how to read the current height of the allocation-record tree and the extent tree -- which, per D8-14-a and D8-14-b, are not ordinary code-2 trees but separately structured, position-addressed trees -- is left to a different, not-yet-settled decision, so do not assume the code-2 height rule automatically carries over to them without saying so. The instance-table chain does not enter ckpt_cost; its cost is charged separately, to the instance-switch reserve." The quoted material states this ckpt_cost is measured in units of 16 KiB metadata blocks, and that it is a runtime-computed quantity with no field on disk recording it. The quoted material does not give a fixed number for "the accounting tree's nodes rewritten per publish", nor a worked example for the central-mapping tree's height; treat both as not given.

TODAY (observed facts about the code as it stands right now, before the redesign; refer to these by the function names given, not by any file or line number):

TODAY-a: Administrator rollback mounting is implemented by functions named mount_rollback and mount_rollback_with_space_admission. Rollback is treated as one recovery: take a new instance number, write a row, rotate the system configuration (a witness of the rollback rides along with that rotation), then warm up.

TODAY-b: Floor-related logic lives in functions named reclaim_floor, rollback_floor_ceiling, raise_rollback_floor, and rehearse_the_publishes_raising_the_floor.

TODAY-c: There is no unmount, close, or shutdown operation anywhere in the codebase today (a search for function names matching those words returns zero hits, aside from two unrelated test function names). Per D23-14-c above, every mount already goes through full recovery regardless of prior shutdown state, because there is no clean-shutdown marker.

PART 3. Three arithmetic questions

Answer these three questions using only Parts 0, 1, and 2 above. For each one, fill in the table exactly as shaped, one row at a time; do not skip a cell. If a cell cannot be computed from the given facts, write "not computable from the given facts" in that cell and name which missing quantity you would need, rather than guessing a number, and rather than answering only "yes" or "no". After each table, write one short paragraph giving your overall answer to the question, and end that paragraph with one sentence starting with "This would be falsified by:" describing a concrete observation that would overturn your answer.

Question 1. Comparing today's guarantee against semantics B at power-off.

Fill this table (three rows, columns: Scenario / Floor-raising target used / Formula or fact you used / Number of valid, non-empty, in-the-candidate-set roots left in the ring / Number of retreat steps a crash-time root-selection fallback still has available):

Row 1: "Today, steady state, admission has never been tight enough to raise F" (use D16-1-c and D23-14-b).
Row 2: "Today, worst case, F has been raised as far as D16-1-d's ceiling formula allows" (use D16-1-d, D16-1-f, D23-14-a, D23-14-b).
Row 3: "Under semantics B, immediately after a completed clean unmount, at the moment the machine is then powered off" (use Part 1's description of semantics B together with D16-1-c, D16-1-d, D16-1-f; state explicitly whether semantics B's description, "raises F all the way up to the newest root", is or is not consistent with the ceiling formula in D16-1-d, and say what number of valid non-empty candidate-set roots results under your reading).

Question 2. Node-read upper bound for a genesis-generation-pruned walk of the newest tree, for a rollback spanning k non-empty states, k = 1, 2, 3, 4.

Fill this table (four rows, one per value of k; columns: k / Which tree you are walking and why / Tree height and fan-out numbers you are using, with their source label / Your upper bound formula in terms of k, height, and fan-out / The resulting number for the two-4-GiB-disk worked example in D8-14-a):

For all four rows, first state, in one sentence, what "genesis-generation-pruned walk" means to you given only D5-2, D5-13, D5-14, and D18-18 above (D18-18 explicitly does not tell you whether a node's own header birth-generation is refreshed on every rewrite; say how you are handling that gap). Then give an upper bound on the number of tree nodes read, as a function of k and the tree's height and fan-out from D8-14-a or D8-14-b; state which of the two trees you chose and why the choice follows from D5-13's release rule (a block is released directly, or appended to a deadlist, depending on birth generation versus previous_snapshot_txg) rather than from a guess. If your bound needs the lower segment's fan-out from D8-14-b and that number is not given, say so in the corresponding cell instead of inventing one.

Question 3. Upper bound on the number of publishes and the number of units in the uninstall's raise-F string.

Fill this table (rows: "number of publishes" and "number of units per publish" and "number of units total"; columns: Upper bound expression / Which facts it is built from / Numeric value, or "not computable from the given facts" and what is missing):

For "number of publishes", derive an upper bound using D16-1-d (the ceiling formula), D16-1-h (the B_adm = 8 cap on one admission attempt, and note explicitly whether that cap is defined for admission-triggered raises or for the kind of raise semantics B describes, and say which one you are assuming and why), D22-2-a and D22-2-b (ring geometry, R = 3, S between 4 and 16), and D23-14-b (that under normal circumstances rollback depth is the entire ring). For "number of units per publish", derive an upper bound using D16-9 (what an empty publish writes) and D28-4's ckpt_cost formula; decide for yourself, given D28-4's caveat about the allocation-record tree's height-reading rule being left to a different, unsettled decision, whether you are licensed to use D8-14-a's worked-example height (3) there anyway, and say explicitly which way you decided and why; mark as "not computable from the given facts" any term in D28-4's formula that this material does not give a number for (the central-mapping tree's height, and the accounting tree's nodes rewritten per publish). For "number of units total", multiply your two upper bounds, carrying forward any "not computable" markers rather than silently dropping them.

PART 4. Formatting rules for your whole answer

Number your three answers 1, 2, 3, matching Question 1, Question 2, Question 3 above; do not merge them and do not add a fourth. Write in English only. Do not use markdown emphasis anywhere (no asterisks for bold or italic, no backticks, no headers marked with hash signs). Do not cite any file name or line number anywhere in your answer; when you need to point at where a fact came from, use the labels already given in Part 2 (for example D16-1-d, D8-14-a, D28-4) or the row numbers of the tables you fill in Part 3. Do not invent facts that are not stated in Part 0 or Part 2; where a needed number is missing, say so explicitly in the relevant table cell and explain what is missing, rather than guessing or leaving the cell blank. Do not answer any question with only "yes" or "no" anywhere in your response. Every table row needs the concrete number or formula, or the concrete missing element, spelled out in words. Each of the three answers must end with a sentence starting with "This would be falsified by:" as instructed above.
