# Round 2: does the accounting tree contain a quantity that traversal cannot recompute?

From-scratch COW filesystem, format design phase, no code. Answer in English.
Do not use any markdown emphasis in your answer.

## What round 1 settled, and what it did not

Round 1 asked whether the 6-byte transaction number should be cut from class-code-2
(index node) unit headers. The main agent ruled: no decision, material withdrawn.
Two findings from round 1 stand, both verified verbatim against the repo:

1. The criterion the cutting arm relied on ("zero consumers today, never separately
   argued useful, so the burden of proof sits on keeping it") fails the project's own
   rule that a newly stated criterion must be swept back over existing entries. Swept
   back, it would also cut three settled items: the reserved nonce and MAC field slots
   in the same code 2 field table, the fixed MAC/nonce placeholders in D19 settled item 3,
   and the 32-byte reserve in D8 settled item 8 that a user ruled on 2026-09-06 with the
   verbatim reason "insufficient reserve is a format-level dead end" and "reserve wide".

2. The cost model in the round 1 material was wrong and all three legs inherited the error.

## The new target: a chain the adversarial leg opened, verified by the main agent today

Verbatim clauses, each checked against the repo today:

a. C113 ruling revision 9, rule P3: code 2 is excluded from the current-version selection
   rule because "it can be recomputed offline by traversal; rebuild rewrites it as
   derived state". An earlier revision said "code 2 is derived state" and that reason was
   struck out, because accounting-tree code 2 nodes are authoritative state.

b. D5 settled item 4, statistic number 12: the inode number watermark (the next usable
   inode number) lives in the accounting tree. It is the only statistic carrying a
   tree dimension, maintained only for writable heads.

c. D23 settled item 14: "all accounting statistics have their current values reloaded
   from the accounting tree of R_old" after a rollback.

d. Owed check C143, still unpaid: because of (b) and (c), after a rollback the inode
   number watermark goes backwards and already-issued inode numbers get reissued, which
   collides with D8 settled item 6 "inode numbers are monotonic and never reused".
   C143 records that fixing this needs a user ruling, with two candidates, both
   format-level: put a per-head field in the tree table entry, or exempt monotonic
   accounting quantities from the rollback reload (which changes D23 settled item 14).

e. D8 settled item 6: inode numbers are monotonic and never reused. Reusing one inside
   the deleted_inodes reclaim window would make the extent keys of the old and the new
   object identical in three segments.

## The inference chain to attack or defend

The main agent has NOT verified the following three steps; they are inferences, which is
why they are being put to three independent legs:

Step 1. Traversal cannot recompute the inode number watermark. Traversal yields the
        largest number among objects that still exist; numbers that were issued and then
        deleted are not among them, so a recomputed watermark would be lower than the
        true one and would reissue numbers.

Step 2. Therefore the code 2 node holding that accounting row cannot be recomputed offline
        by traversal, so the P3 replacement reason in (a) is false for that node.

Step 3. Therefore scan-based rebuild must select among multiple on-disk versions of that
        node. Same tree id, same level, same key range, same birth txg, different
        contents, all readable. Within one instance and one checkpoint the instance id
        does not separate them, because the checkpoint fixed point iterates and the same
        node can be written more than once per checkpoint. That is verbatim the reason
        invariant I-1.8 gives for keeping the transaction number in the code 3 total-order
        key: "the transaction number stays in the last position as the tie-breaker within
        one checkpoint (the fixed point iterates, a container is not guaranteed to be
        written only once per checkpoint)".

If all three steps hold, the 6-byte transaction number has a consumer and must not be cut.
If any step fails, say which and why.

## A counter-consideration the main agent found, also unverified

Scan-rebuilt state is permanently read-only today (C113 ruling: promotion needs level 2
witnesses). A read-only filesystem issues no inode numbers, so it may not need the
watermark at all, and the gap would only open on promotion to writable.

Separately: on 2026-09-06 a user ruled that the tree id watermark moves out of accounting
into the root record, taking the max over all root records in the root ring. C143 records
that this fix does not transfer to the inode number watermark, because that statistic
carries a tree dimension, so its row count grows with the number of writable heads, while
root records and superblocks are fixed-width slots.

## The five arms now on the table

A. Cut code 2 to 4 bytes (instance id only).
B. Keep 10 bytes, keep writing the real transaction number (status quo).
C. Keep 10 bytes of width, 6 bytes reserved-must-be-zero, non-zero rejected.
D. Keep 10 bytes of width, rename the 6 bytes to "payload CRC 4 + reserved 2". This pays
   off two registered debts at zero byte cost: C104's code-2 half (verbatim "still owed;
   code 2 has no payload integrity evidence"), which needs exactly 4 bytes, and C102
   (the index node header has no reserved area, so adding a field later is incompat, not
   compat; its verbatim discharge form is "adding a reserved field to the index node
   header must turn it green").
E. Keep 10 bytes and change the value rule of the code 2 transaction number to a fixed-
   point iteration counter, then bring code 2 into I-1.8.

Note on D: on the scan path, does a payload CRC buy code 2 anything at all? D18 settled
item 5 grades rebuild claims by witness: level 2 needs "allocation record plus checksum
or MAC double evidence", level 1 needs "checksum or MAC self-consistent", level 0 is
"cannot even verify the checksum". C104 says that for code 1 and code 2 this
self-consistency "only reaches as far as the header".

## Your assignment: hunt for counterexamples in the settled clauses

Do not pick an arm. Find concrete scenarios built out of the settled clauses in which one
of the three inference steps, or one of the five arms, produces a wrong outcome on disk.

Work through these surfaces and say for each whether you found a counterexample:

1. Step 1. Is there any way traversal recovers the inode number watermark exactly?
   Consider that inode records stay in the inode tree with nlink = 0 until reclaim
   finishes, and that deleted_inodes stores only the numbers. If deleted numbers are
   still reachable on disk, traversal might recover the true watermark after all.

2. Step 3. Within one checkpoint, does the accounting tree leaf holding the inode number
   watermark actually get written more than once? The fixed point iterates because
   allocation changes accounting and accounting changes allocation. If it is written only
   once per checkpoint, birth txg alone separates the versions and no transaction number
   is needed.

3. Arm D. A payload CRC on code 2: name one concrete path that reads a code 2 payload
   without having a parent pointer. If no such path exists, arm D buys nothing and the
   4 bytes are as unused as the transaction number they replace.

4. Arm E. If the code 2 transaction number becomes a fixed-point iteration counter rather
   than the checkpoint's maximum transaction number, what breaks? Note that code 1 and
   code 3 keep the old meaning, so one field would carry two different meanings depending
   on class.

For every counterexample give the exact event sequence and name the violated clause.
If you cannot construct one, say so plainly instead of inventing one.
