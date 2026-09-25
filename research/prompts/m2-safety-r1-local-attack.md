Task for a local arithmetic reviewer (attacker role, arithmetic only, two independent parts).

You are given a fixed set of numbered facts about a filesystem's space-admission formula and
about its rollback-witness table, taken from its design decisions and from its source code. You
are asked four numbered items. For each item, fill in every row of the worksheet given for that
item with a number, or with an explicit statement "not determined by the given facts, my estimate
is ___, because ___". Do not answer any item with only yes or no. Show your arithmetic. Reference
facts only by their number (such as "Fact 6") or by the exact constant or function name given in
that fact; do not invent a source file name or a line number, and do not use any line number even
if you happen to know one. End every item with one sentence starting with "This would be refuted
by:" describing a concrete alternative computation, number, or missing term that would overturn
your answer to that item.

Part 1 is about the admission formula (available and demand). Part 2 is about the rollback-witness
table (how many entries it holds). The two parts do not share any fact; do not carry a number from
one part into the other unless a fact explicitly says so.

Part 1: the admission formula.

Fact 1. A filesystem's admission formula defines, for every device d in a pool, available(d) as

available(d) = capacity(d) - allocated(d) - unreclaimable(d) - deferred_pending_release(d)
  - mount_time_commitment(d) - abandoned_root_exclusive(d)
  - pending_delete_occupancy / replicas - committed_reservation / replicas
  - checkpoint_reserve_pool / replicas

A write (or a mount) is admitted only if available(d) is at least demand(d) on every device d in
the pool separately (a per-device conjunction): a surplus on one device never offsets a shortfall
on another device. The first six terms (capacity, allocated, unreclaimable,
deferred_pending_release, mount_time_commitment, abandoned_root_exclusive) are each that device's
own value. The last three terms (pending_delete_occupancy, committed_reservation,
checkpoint_reserve_pool) are pool-wide quantities recorded as physical bytes summed over every
replica of the pool; each device's share of each of these three terms is that pool-wide sum
divided by the replica count. In every case discussed in this task, that division has no
remainder, and every pool discussed in this task has exactly 2 devices and exactly 2 replicas (one
copy of every unit on each device).

Fact 2. capacity(d), the first term in Fact 1, equals the number of 16 KiB slots in device d's
unit area. The unit area is the region of the device used for ordinary allocation; it starts at a
fixed absolute slot number that is itself not part of the unit area and is not part of capacity(d)
(this starting slot number is named UNIT_AREA_START_SLOT in the source, and the function that
computes a device's unit-area slot count from its total byte size is named
unit_area_slots_of_device; the field that stores this same count on an already-built device map is
read through a method named unit_area_slots; the place that copies this value into capacity when
building an admission reading is a function named of_allocator, whose doc comment states in the
author's own words "容量 = 单元区槽数", that is, "capacity equals the unit area's slot count").
capacity(d) never includes UNIT_AREA_START_SLOT and never includes any other fixed region of the
device (such as the root ring or the journal ring), only the unit-area slot count itself.

Fact 3. mount_time_commitment(d), the fifth term in Fact 1, equals instance_switch_reserve(d)
only. It does not include checkpoint_reserve_pool. checkpoint_reserve_pool is accounted for
separately, as the last term of the formula in Fact 1.

Fact 4. For a pool that has never deleted a file and has never made a reservation such as a
tombstone: pending_delete_occupancy = 0 and committed_reservation = 0. Every history discussed in
this task only ever mounts the pool writable and overwrites the content of one already-existing
file; it never deletes a file and never creates a tombstone, so this fact applies throughout. For
a pool that does not run on zoned storage: unreclaimable(d) = 0 on every device; every pool
discussed in this task is non-zoned.

Fact 5. abandoned_root_exclusive(d), the sixth term in Fact 1, equals the number of slots on
device d that are referenced only by abandoned roots (roots left behind by an administrator
rollback). In a pool history that never performs an administrator rollback, no root is ever
abandoned, so this term is 0 on every device throughout that history. Every history discussed in
Part 1 of this task never performs an administrator rollback.

Fact 6. checkpoint_reserve_pool (the last term in Fact 1) is derived from ckpt_cost, a count of
16 KiB metadata blocks: checkpoint_reserve_pool, summed over every replica of the pool, equals
ckpt_cost times 16384 times the replica count. Therefore each device's share of it (that pool-wide
sum divided by the replica count) is exactly ckpt_cost times 16384 bytes, that is, ckpt_cost slots,
regardless of how many replicas the pool has.

Fact 7. ckpt_cost is defined as: the sum, over the allocation-record tree and the central-mapping
tree, of each tree's current height, plus the accounting tree's node count for this publish, plus
1 for the tree table. The instance-table chain is not part of ckpt_cost (its own cost is
instance_switch_reserve, Fact 3 and Fact 8). Each tree's height is read from that tree's own root
node's header, as level + 1. The same ckpt_cost number is also used as c_max, the worst-case cost
of one warm-up empty publish.

Fact 8. instance_switch_reserve(d) is computed as follows. Let rows0 be the number of rows in the
instance table right after this mount's own row-publish (see Fact 9 for how to get rows0 in the
histories of this task). Let N_switch = 3 and R = 3 (R is the same root-ring-region count used
throughout this task; it is named ROOT_RING_REGIONS in the source). Let pages_of_chain =
ceiling((rows0 + N_switch) / 369), with a minimum of 1 (369 = 370 - 1: one instance-table page
holds 370 records, of which the last is always a chain-pointer record, so 369 rows of real data
fit on one page). Let chain_rewrite = pages_of_chain times 2 slots (one instance-table page spans
2 slots). Let warm_up = c_max (Fact 7) times R, in slots. Let one_switch = chain_rewrite +
warm_up. Then instance_switch_reserve(d) = one_switch times (N_switch + 1), the same number on
every device. one_switch by itself, without the times (N_switch + 1), is the real (not
reserved) cost of one instance-switch-shaped event: for a writable mount with m at least 2 (Fact
9), it is exactly the cost that mount actually spends on its own establishment (one real row,
that is, chain_rewrite, plus its own warm-up), as opposed to the larger reserved headroom that
mount_time_commitment sets aside against up to N_switch further switches. For the very first
writable mount (m = 1), the real row-write cost is 0, not chain_rewrite, because Fact 9 gives it
zero rows to write; only its warm-up cost is real.

Fact 9. Consider a pool right after mkfs and the first file are published in the same process (no
row-publish has happened yet, and the pool has not yet been closed). The pool is then closed and
mounted writable for the first time; this becomes instance 1. Every later close-and-mount-writable
cycle takes the next integer instance number (2, 3, 4, ...). Every writable mount, including a
plain one that performs no administrator rollback, writes into the instance table, as part of its
own establishment, one row for every instance number from (the previous root's own instance
number, or 1, whichever is larger) up to (but not including) its own new instance number; a
mount's own new instance number is never included in the rows it writes. Consequently: the first
writable mount (instance 1) writes rows for the range from max(0, 1) = 1 up to (not including) 1,
which is empty, so it writes zero rows. The second writable mount (instance 2) writes rows for the
range from max(1, 1) = 1 up to (not including) 2, which is exactly {1}, so it writes exactly one
row (for instance 1). In general, the m-th writable mount (m = 1, 2, 3, ...), when it is a plain
mount that follows directly after the (m-1)-th writable mount with no administrator rollback in
between, writes exactly one row (for instance m-1) if m is at least 2, and zero rows if m = 1.
Therefore, right after the m-th writable mount's own row-publish, rows0 for that mount's own
admission check equals (m-1) for m = 1, 2, 3, ... (that is, rows0 = 0 at the first writable mount,
rows0 = 1 at the second, rows0 = 2 at the third, and so on), counting only rows written by mounts
that follow one another with no administrator rollback in between, as in every history in Part 1
of this task.

Fact 10. Every unit written by a publish occupies a fixed number of 16 KiB slots, and belongs to
exactly one of three categories.

Category Demand (counted in demand(d) for an ordinary write; not covered by any reserve): one
user data unit, 2 slots. One extent-tree node below the root, 1 slot. The extent-tree root, 1
slot. One inode leaf container, 2 slots. The inode-tree root, 1 slot.

Category CheckpointReservePool (paid for out of checkpoint_reserve_pool; never counted in
demand): allocation-record-tree nodes below the root and the allocation-record-tree root, 1 slot
each. Accounting-tree nodes below the root and the accounting-tree root, 1 slot each.
Central-mapping-tree nodes below the root and the central-mapping-tree root, 1 slot each. The
tree table, 1 slot.

Category InstanceSwitchReserve (paid for out of instance_switch_reserve; never counted in
demand): the instance table and any later page of the instance-table chain, 2 slots each.

Fact 11. One physical slot, and also one 16 KiB metadata node, is 16384 bytes (this constant is
named SLOT_BYTES in the source).

Fact 12. The allocation-record tree is addressed by absolute slot position, not by content: its
shape depends only on the total number of absolute slots on each device, never on how many records
are actually written. A device configured with N unit-area slots has UNIT_AREA_START_SLOT + N
absolute slots in total (UNIT_AREA_START_SLOT is given in Fact 2; its value is 50176). One leaf of
the allocation-record tree covers 812 consecutive absolute slots (this constant is named
ALLOCATION_RECORD_TREE_LEAF_SLOTS); one internal node has fan-out 169 (this constant is named
ALLOCATION_RECORD_TREE_INTERNAL_FANOUT). The tree's root level is the smallest R greater than or
equal to 1 such that, summed over every device in the pool, ceiling(total_absolute_slots_of_that_
device divided by (812 times 169 to the power of (R-1))) is at most 169 (this R is an unrelated,
purely local variable of this one rule and is not the same R as in Fact 8). The tree's height
equals root level plus 1.

Fact 13. The central-mapping tree's height and the accounting tree's height (when a height is
wanted for either of them) are read the same way as the allocation-record tree's height: from that
tree's own root node's header, as level + 1. The accounting tree's contribution to ckpt_cost
(Fact 7) is its node count for this publish, not a height. The central-mapping tree's node header
is 169 bytes wide and each of its entries is 55 bytes, so one leaf holds at most
floor((16384 - 169) / 55) = 294 entries. The accounting tree's node header is 159 bytes wide and
each of its entries is 34 bytes, so one leaf holds at most floor((16384 - 159) / 34) = 477 entries.

Fact 14. Directly observed shape of a pool made of two 4 GiB devices, right after its very first
transaction (mkfs and the first file published in the same process, one data unit, no other
publish yet): the allocation-record tree's root is at level 2 (so its height is 3, by Fact 12's
rule). The central-mapping tree's root is also that tree's only node (level 0, so its height is 1)
and holds exactly 10 entries. The accounting tree's root is also that tree's only node (level 0)
and holds 15 rows, so its node count for this publish is 1.

Fact 15. Besides the 4 GiB device used in Fact 14, this codebase's tests also use three
small-disk unit-area sizes: 384 slots, 256 slots and 240 slots (each with the same
UNIT_AREA_START_SLOT = 50176 offset before the unit area itself, per Fact 2). A code comment
attached to the 384-slot configuration states, in the author's own words and marked as
approximate: after the first file, each device holds about 13 slots; one more overwrite of the
same shape (content that still fits inside one data unit) occupies roughly 10 more slots per
device, and this growth continues for dozens of publishes before reclamation catches up. A code
comment attached to the 256-slot configuration only describes a different regime (the steady state
after the root ring has turned over once, about 24 generations later), which does not apply to the
first few overwrites of a fresh pool and is not used in this task. For the 256-slot and 240-slot
configurations, in the absence of a directly observed early-growth number, assume the same
approximate rate as the 384-slot comment (about 13 slots after the first file, about 10 more per
same-shape overwrite), since the underlying mechanism (copy-on-write replacement of one data unit,
held for a while by the deferred-release window) does not depend on device width; state explicitly
in your answer that this carries the 384-slot comment's number over to 256 and 240 by analogy, not
by direct observation.

Fact 16. A test exercises three small-disk pools (2 devices each, unit-area widths 240, 256 and
384 slots), with the space-admission formula judged (not skipped) at every step, that is, the real
product-path admission check runs for every overwrite and every mount. Starting right after mkfs
and the first file are published in the same process, the pool is closed and mounted writable once
(this is writable mount 1, rows0 = 0 by Fact 9); then a number of overwrites of the same file are
published one after another, every one of them using content that still fits inside a single data
unit, and every one of them admitted by the real formula (none skipped). The largest number of
such overwrites that the formula admits in this first session, before the formula itself refuses a
further same-shape overwrite, is called k_max: k_max = 5 for the 240-slot pool, k_max = 6 for the
256-slot pool, k_max = 10 for the 384-slot pool. After k = k_max - 1 admitted overwrites (that is,
one less than k_max), the pool is closed and mounted writable a second time (mount 2, rows0 = 1 by
Fact 9): this second mount is admitted. With nothing written in between, the pool is closed and
mounted writable a third time (mount 3, rows0 = 2 by Fact 9): this third mount is refused by the
formula. After k = k_max admitted overwrites (that is, the largest number the session allows), the
pool is closed and mounted writable a second time (mount 2, rows0 = 1 by Fact 9): this second mount
is already refused by the formula.

Fact 17. Publish-time admission and mount-time admission read the same formula (Fact 1) but at
two different call sites, named in the source as prepare_the_version_publish (for a publish, such
as one of the overwrites in Fact 16) and establish_instance (for a writable mount). At the publish
call site, demand(d) is computed from the ordinary allocation this publish is about to make (data
units, extent-tree nodes, inode-tree nodes; the Category Demand list in Fact 10), and is checked
before anything is read from disk or written for this publish. At the mount call site, demand(d) is
fixed at 0 on every device for this check, before the mount takes its new instance number and
before it runs its own pre-write simulation; a publish that has no ordinary allocation (an empty
publish, a row-write, a warm-up, or a raise-the-floor step) is likewise never checked against a
nonzero demand(d). Both call sites read the same available(d) formula (Fact 1); mount_time_
commitment(d) and checkpoint_reserve_pool(d) are recomputed at whichever call site is currently
running, from whatever allocated(d), rows0 and ckpt_cost that call site observes at that moment
(they are not cached from an earlier call).

Item 1. The 256-slot pool of Fact 16, at k = k_max = 6 (the case where the very next mount is
already refused).

Fill in every row below for one device d (by symmetry the two devices in this pool are identical
in every row, so one worksheet covers both). Row group A is the publish-time admission check for
the 6th (last) overwrite, while it is still being checked, before it is applied. Row group B is
the mount-time admission check for writable mount 2 (rows0 = 1 by Fact 9), right after that 6th
overwrite, with nothing else written in between.

Row A1. capacity(d), in slots (Fact 2; use the 256-slot unit area).
Row A2. allocated(d), in slots, at the moment the 6th overwrite is about to be checked (that is,
  after the first file and 5 prior same-shape overwrites have already been applied). Use Fact 15's
  approximate growth note and show your arithmetic; state explicitly that this is an estimate.
Row A3. unreclaimable(d), in slots.
Row A4. deferred_pending_release(d), in slots, at that same moment.
Row A5. instance_switch_reserve(d), in slots, for writable mount 1 (rows0 = 0 by Fact 9), which is
  the mount whose session this 6th overwrite is published under. Show rows0, pages_of_chain,
  chain_rewrite, warm_up and one_switch from Fact 8 as you compute this; you will need ckpt_cost
  from row A9 below.
Row A6. abandoned_root_exclusive(d), in slots.
Row A7. pending_delete_occupancy divided by replicas, in slots.
Row A8. committed_reservation divided by replicas, in slots.
Row A9. ckpt_cost, in 16 KiB blocks, for the version this 6th overwrite would build on. Show the
  four components you summed, per Fact 7; state explicitly, citing Fact 13 and Fact 14's leaf
  capacities, why the central-mapping tree's height and the accounting tree's node count for this
  publish should still be at their Fact 14 baseline (height 1 and node count 1) after only 5 prior
  same-shape overwrites of one file, rather than having grown.
Row A10. checkpoint_reserve_pool divided by replicas, in slots (by Fact 6 this should equal row A9
  exactly).
Row A11. available(d), in slots: row A1 minus every one of rows A2 through A6, minus rows A7, A8
  and A10 (do not subtract row A9 separately; row A9 only feeds into row A10).
Row A12. demand(d), in slots, for this one overwrite: list every unit from Fact 10's Demand
  category that an overwrite of an existing, unchanged-length file rewrites, with each unit's span
  in slots, and sum them.
Row A13. Given that this overwrite is in fact admitted (Fact 16), is row A11 at least row A12?
  State yes or no, and by how many slots of slack (row A11 minus row A12).

Row B1. capacity(d), in slots (same device, same as row A1).
Row B2. allocated(d), in slots, at the moment writable mount 2 is about to be checked: row A2 plus
  whatever the 6th overwrite of row group A actually consumed once admitted (state your assumption
  about how much of row A12's demand becomes real allocated(d) growth, and say why).
Row B3. unreclaimable(d), in slots.
Row B4. deferred_pending_release(d), in slots, at that same moment.
Row B5. instance_switch_reserve(d), in slots, freshly recomputed for writable mount 2 itself
  (rows0 = 1 by Fact 9, per Fact 17's "not cached" rule). Show rows0, pages_of_chain,
  chain_rewrite, warm_up and one_switch again; state whether ckpt_cost (row A9's components) has
  any reason to have changed between mount 1's session and this recomputation, citing Fact 13.
Row B6. abandoned_root_exclusive(d), in slots.
Row B7. pending_delete_occupancy divided by replicas, in slots.
Row B8. committed_reservation divided by replicas, in slots.
Row B9. ckpt_cost, in 16 KiB blocks, freshly recomputed for writable mount 2.
Row B10. checkpoint_reserve_pool divided by replicas, in slots.
Row B11. available(d), in slots, computed the same way as row A11 but from the row B values.
Row B12. demand(d), in slots, for writable mount 2 itself (Fact 17 already tells you what this is;
  state it and say why).
Row B13. Using only rows B2 through B10 as you computed them (that is, without adding any further
  real consumption beyond what row B2 already states), is row B11 at least row B12? State yes or
  no, and by how many slots.

Fact 16 states that writable mount 2 is in fact refused when k = k_max = 6. If your row B13 says
yes (admitted), state explicitly, in one paragraph, the smallest additional real consumption (in
slots, and naming which term among rows B2 through B10 it would add to) that would be needed to
flip row B13 to a refusal, and say which fact in this task (if any) accounts for that additional
consumption, or state that no given fact accounts for it.

This would be refuted by: [one sentence].

Item 2. The 384-slot pool of Fact 16, at k = k_max - 1 = 9 (the case where the next mount is
admitted but the one after that, with nothing written in between, is refused).

Fill in every row below for one device d. Row group A is the publish-time admission check for the
9th (last) overwrite in this session. Row group B is the mount-time admission check for writable
mount 2 (rows0 = 1 by Fact 9), right after that 9th overwrite. Row group C is the mount-time
admission check for writable mount 3 (rows0 = 2 by Fact 9), right after mount 2 is admitted and
established, with nothing written by the user in between mount 2 and mount 3.

Row A1 through A13. Same rows as Item 1's row group A, but for the 384-slot device and k = 9 (that
  is, allocated(d) in row A2 reflects the first file plus 8 prior same-shape overwrites).

Row B1 through B13. Same rows as Item 1's row group B, for writable mount 2 here.

Row C2. allocated(d), in slots, at the moment writable mount 3 is about to be checked. Start from
  your row B2 value and add the real cost that writable mount 2 itself actually spent on its own
  establishment once it was admitted (state this using one_switch as computed from your row B5,
  per Fact 8's definition of one_switch as the real, not reserved, cost of one ordinary mount's own
  establishment for m at least 2); show this addition explicitly as its own line before giving the
  row C2 total.
Row C5. instance_switch_reserve(d), in slots, freshly recomputed for writable mount 3 itself
  (rows0 = 2 by Fact 9). Show rows0, pages_of_chain, chain_rewrite, warm_up and one_switch again.
Row C9. ckpt_cost, in 16 KiB blocks, freshly recomputed for writable mount 3.
Row C10. checkpoint_reserve_pool divided by replicas, in slots.
Row C11. available(d), in slots, computed the same way as row A11 but from the row C values (C2,
  C3 = row B3, C4 = row B4, C5, C6 = row B6, C7 = row B7, C8 = row B8, C10).
Row C12. demand(d), in slots, for writable mount 3 itself.
Row C13. Is row C11 at least row C12? State yes or no, and by how many slots.

Fact 16 states that, for this pool, writable mount 2 is admitted and writable mount 3 is refused.
State explicitly, in one paragraph: does your row C13 agree with this (that is, does your
arithmetic, using only the real one_switch consumption added in row C2, already predict a
refusal), or does it still predict admission? If it still predicts admission, state the smallest
additional real consumption needed to flip it, and whether any fact in this task accounts for it,
exactly as in Item 1's closing paragraph.

This would be refuted by: [one sentence].

Part 2: the rollback-witness table. This part shares no fact with Part 1; R and S below are a
different pair of names for the same root-ring-region count and same root-ring-slots-per-region
count used in Part 1's Fact 8, but Part 2 restates them from scratch so it stands alone.

Fact 18. Every administrator rollback writes exactly one witness entry into a system-wide witness
table. A witness entry is a triple (N, r_old, T_old): N is the new instance number this rollback
takes; r_old is the instance number of the old root R_old that this rollback targets (the root it
rolls back to); T_old is that old root's own checkpoint-generation number (its txg). A root (i, T)
(instance number i, txg T) is judged abandoned by a witness entry (N, r_old, T_old) if and only if
both: (r_old, T_old) is strictly less than (i, T), comparing instance number first and txg second;
and i is strictly less than N. Root selection (choosing which root to mount or recover from) skips
every root judged abandoned by any witness entry currently in the table.

Fact 19. The witness table's capacity is R times S minus 1 entries, where R = 3 is the number of
root-ring regions and S = 8 is the number of slots per region at the geometry used throughout this
task (so capacity = 23 entries); this capacity comes purely from the root-ring geometry, not from
any property of the witness entries themselves. The root ring itself holds at most R times S = 24
distinct root generations at any time: generation g (a root published as the g-th root ever, g = 0
for the mkfs genesis root, g = 1 for the first root published after that, and so on, one new
generation per publish that produces a new root) occupies ring region (g mod R) and, within that
region, ring slot ((g div R) mod S); consequently generation g and generation g + 24 occupy the
exact same physical ring position, and generation g is overwritten (no longer present in the ring,
no longer readable as a distinct root) exactly when generation g + 24 is published. Every history
in this task is free of any hardware fault: every ring slot is always readable and always
self-certifies correctly, at every point in every history.

Fact 20. Deletion rule (i): a witness entry (N, r_old, T_old) is deleted if and only if every slot
of the root ring is readable and self-certifies (always true in this task, by Fact 19), and none of
the roots currently present in the root ring (Fact 19's rotating set of at most 24 generations) has
an instance number falling in the half-open range [r_old, N).

Fact 21. Covered-deletion rule (an additional, independent deletion trigger, added on top of rule
(i) in Fact 20, and not conditioned on ring readability): a witness entry (Na, ra, Ta) is deleted
if there exists some other entry (Nb, rb, Tb) currently in the table such that Nb is greater than
or equal to Na, and (rb, Tb) is less than or equal to (ra, Ta) (comparing instance number first,
txg second). Every root abandoned by the covered entry (Na, ra, Ta) is also abandoned by the
covering entry (Nb, rb, Tb), so deleting the covered one never changes which roots are judged
abandoned by the table as a whole. This rule can fire even when some ring slot would fail rule
(i)'s readability condition; it never requires the ring to be fully readable.

Fact 22. Capacity check: right before a rollback would take its own new instance number, if the
witness table, after backfill (Fact 23) and after removing every entry that rule (i) (Fact 20) or
the covered-deletion rule (Fact 21) deletes, already holds capacity (23, Fact 19) entries, the
rollback is refused before taking any new instance number and before writing anything; nothing on
disk changes. Because nothing changes, every later rollback attempt, with nothing else written in
between, hits exactly the same check and is refused the same way, forever, until something outside
this task's history (not modeled here) changes the table.

Fact 23. Backfill rule (vi): right before running the deletion rules (Fact 20 and Fact 21) at any
writable mount (including a mount that is itself about to perform a rollback), for every row in the
currently-mounted root's own instance table that is flagged as a rollback row (recording some
(r_old, T_old) at the table position for instance r_old, per Fact 25 below), if the witness table
does not currently contain a matching entry for it, one is created: scan the same instance table
forward from the position right after r_old for the first row whose own recorded txg field is
nonzero; if such a row is found at position p, the backfilled entry is (N = p, r_old, T_old); if no
such row is found (every row from r_old + 1 up to, but not including, the currently-mounted root's
own instance number is an unflagged intermediate row recording txg 0, per Fact 25), the backfilled
entry is (N = the currently-mounted root's own instance number, r_old, T_old).

Fact 24. mkfs creates instance 0 with txg 0 (this is generation g = 0 in Fact 19); the first
writable mount takes instance 1. Every root a rollback publishes gets a txg strictly greater than
every txg currently in the root ring and every txg of any self-certifying record currently in the
journal, specifically (current maximum of those) plus 1; in every history in this task, since
nothing else ever publishes in between the steps described, this makes the sequence of txg values
across successive rollbacks strictly increasing, in the same order as the sequence of instance
numbers, so comparing by instance number alone and comparing by (instance number, txg) always
agree throughout this task.

Fact 25. When an administrator rollback targets an old root R_old (old instance number r_old, old
txg T_old) and takes a new instance number N, its new instance table is built by starting from
R_old's own instance table exactly as R_old itself left it (not from whatever table the
most-recently-mounted root happens to have, if that differs from R_old), and then: writing one row,
at the table position for instance r_old, flagged as a rollback row, recording that its own txg
field equals T_old; and writing, for every instance number i with r_old < i < N, an unflagged
intermediate row at the table position for instance i, recording that its own txg field equals 0.
No row is ever written at the table position for instance N itself. A row, once written at a given
table position by one of these rules, is never overwritten later by a different rollback's rule,
except when a later event's own writing range (its own [r_old, N) or the general mount range of
Fact 9) includes that same position again.

Fact 26. History A ("rollback to the same target", 30 rollbacks, numbered t = 1, 2, ..., 30): every
rollback targets the mkfs genesis root itself (instance 0, txg 0, per Fact 24), regardless of how
many rollbacks have already happened. Rollback t takes new instance number t (instance numbers are
1, 2, 3, ..., 30 across the 30 rollbacks, in order, since nothing else ever takes an instance number
in this history). Worked example using Fact 25: rollback 1 targets R_old = instance 0; its new
table is built from instance 0's own table (mkfs's genesis table, which has no rows at all) plus one
rollback row at position 0 (recording txg 0); there are no intermediate rows, since r_old = 0 and
N = 1 leaves no i with 0 < i < 1. Rollback 2 also targets R_old = instance 0 (not instance 1, since
this history always targets the same fixed root); its new table is again built from instance 0's
own table (still just the mkfs genesis table, unrelated to whatever table instance 1 built for
itself), plus one rollback row at position 0. In general, for every t in this history, the
currently-mounted root going into rollback t is the root instance (t - 1) produced by rollback
(t - 1) itself (or the mkfs root, for t = 1); its own instance table, by this same reasoning applied
at step (t - 1), contains exactly one row: a rollback row at position 0 recording txg 0 (and, for
t = 1, contains no rows at all, since no rollback has happened yet).

Fact 27. History B ("rollback to the newest root", 30 rollbacks, numbered t = 1, 2, ..., 30): each
rollback targets whichever root was most recently produced, that is, rollback t targets R_old =
instance (t - 1) (the root produced by rollback (t - 1) itself, or the mkfs root for t = 1), with
its own txg (Fact 24 gives these strictly increasing). Rollback t takes new instance number t.
Worked example using Fact 25: rollback 1 targets instance 0 exactly as in Fact 26, giving a table
with one rollback row, at position 0. Rollback 2 targets R_old = instance 1 (the root rollback 1
itself just produced); its new table is built from instance 1's own table (the one just described,
with its single rollback row at position 0) plus one new rollback row at position 1 (recording
instance 1's own txg); there is no intermediate row, since r_old = 1 and N = 2 leaves no i with
1 < i < 2. Rollback 3 targets R_old = instance 2, building from instance 2's own table (rollback
rows at positions 0 and 1) plus one new rollback row at position 2. In general, for every t in this
history, the currently-mounted root going into rollback t is instance (t - 1); its own instance
table, by induction on this same reasoning, contains exactly (t - 1) rollback rows, at positions
0, 1, ..., t - 2, with the row at position s recording instance s's own txg (and, for t = 1,
contains no rows at all).

Fact 28. At every writable mount in either history (including a mount that performs its own
rollback), the order of events is: first, backfill (Fact 23) adds any currently-missing entries;
second, the deletion rules (Fact 20 and Fact 21) run once over the resulting set of entries and
remove every entry either rule deletes; third, the capacity check (Fact 22) runs against the
resulting count; fourth, if not refused, this mount's own rollback (if it is performing one) takes
its new instance number and adds its own new witness entry (N, r_old, T_old) to the table. Step
four's newly-added entry is not itself re-examined by the deletion rules again within this same
mount; the next mount's own step one and step two are the first point at which it can be deleted
or can cause an earlier entry to be deleted.

Item 3. History A (Fact 26): the witness table's entry count after each of the 30 rollbacks.

For t = 1, 2, 3 and 4, show, step by step: which flagged rollback rows are visible in the
currently-mounted root's instance table at the start of step t (per Fact 26); which witness
entries backfill (Fact 23) adds because they are currently missing (name each one as a triple);
which entries the deletion rules (Fact 20, Fact 21) then remove (name each one and say which rule
removed it); the resulting count right before step t's own rollback is admitted or refused (apply
Fact 22); and, if admitted, the entry step t's own rollback adds and the resulting end-of-step
count. Then give a table with one row per t from 1 to 30, with columns: t, count right before step
t's own rollback (after backfill and deletion), admitted or refused (Fact 22), end-of-step count.
State a general closed-form rule (in terms of t) for both of those count columns, valid for every
t from 5 to 30, and say which of Fact 20, Fact 21 or Fact 23 is doing the work of keeping the count
from growing to 23 in this history, and which of those facts would have to fail for it to grow
that far instead.

This would be refuted by: [one sentence].

Item 4. History B (Fact 27): the witness table's entry count after each of the 30 rollbacks.

For t = 1, 2, 3 and 4, show, step by step, the same five things as Item 3 asked for, but using Fact
27's table contents instead of Fact 26's. Then give a table with one row per t from 1 to 30, same
columns as Item 3. Pay particular attention to t = 23, 24 and 25: show explicitly, for t = 24,
whether Fact 20 could possibly delete any entry at that point (name the specific entry it would
have to delete and the specific root, by generation number in Fact 19's numbering, that would have
to have left the root ring for that deletion to fire, and say whether that root has in fact left
the ring by generation 24, using Fact 19's rotation rule), and state the resulting admitted-or-
refused outcome for t = 24 and for every t from 24 to 30. State a general closed-form rule (in
terms of t) for both count columns, valid for every t from 1 to 30, noting the point (if any) at
which the rule changes shape. State explicitly whether Fact 21 (covered-deletion) ever removes any
entry anywhere in this history, and why or why not.

This would be refuted by: [one sentence].

End of task. Answer items 1 through 4 in order, each with its worksheet or table fully filled in
and its closing "This would be refuted by:" sentence. Do not use any markdown emphasis (no bold,
no italics, no headings marked with number signs, no backtick code spans) anywhere in your answer;
plain numbered lines and plain tables using only hyphens, letters and digits are fine. Do not write
any file name or any line number anywhere in your answer; refer only to fact numbers, row or
column names, and the exact constant or function names given above.
