Task for a local arithmetic reviewer (attacker role, arithmetic only).

You are given a fixed set of numbered facts (clauses and constants taken from a filesystem's
design decisions and from its source code). You are asked four numbered items. For each item,
fill in every row of the worksheet given for that item with a number, or with an explicit
statement "not determined by the given facts, my estimate is ___, because ___". Do not answer
any item with only yes or no. Show your arithmetic. Reference facts only by their number
(such as "Fact 6") or by the exact constant or function name given in that fact; do not invent
a source file name or a line number, and do not use any line number even if you happen to know
one. End every item with one sentence starting with "This would be refuted by:" describing a
concrete alternative computation, number, or missing term that would overturn your answer to
that item.

Facts.

Fact 1. A filesystem's admission formula defines, for every device d in a pool, available(d) as

available(d) = capacity(d) - allocated(d) - unreclaimable(d) - deferred_pending_release(d)
  - mount_time_commitment(d) - abandoned_root_exclusive(d)
  - pending_delete_occupancy / replicas - committed_reservation / replicas
  - checkpoint_reserve_pool / replicas

A write (or a mount) is admitted only if available(d) is at least demand(d) on every device d
in the pool separately (a per-device conjunction): a surplus on one device never offsets a
shortfall on another device.

The first six terms (capacity, allocated, unreclaimable, deferred_pending_release,
mount_time_commitment, abandoned_root_exclusive) are each that device's own value.

The last three terms (pending_delete_occupancy, committed_reservation, checkpoint_reserve_pool)
are pool-wide quantities recorded as physical bytes summed over every replica of the pool; each
device's share of each of these three terms is that pool-wide sum divided by the replica count.
In every case discussed in this task, that division has no remainder.

Fact 2. mount_time_commitment(d), the fifth term in Fact 1, equals instance_switch_reserve(d)
only. It does not include checkpoint_reserve_pool. checkpoint_reserve_pool is accounted for
separately, as the last term of the formula in Fact 1.

Fact 3. For a pool that has never deleted anything and has never made a reservation such as a
tombstone: pending_delete_occupancy = 0 and committed_reservation = 0 (these two are the fixed
values used for what the source code calls "the first version" of a pool). For a pool that does
not run on zoned storage: unreclaimable(d) = 0 on every device.

Fact 4. abandoned_root_exclusive(d), the sixth term in Fact 1, equals the number of slots on
device d that are referenced only by abandoned roots, that is: (slots referenced by abandoned
roots) minus (slots referenced by roots in the rollback candidate set), floored at 0. In a pool
history that never performs an administrator rollback, no root is ever abandoned, so this term
is 0 on every device throughout that history.

Fact 5. checkpoint_reserve_pool (the last term in Fact 1) is derived from ckpt_cost, a count of
16 KiB metadata blocks: checkpoint_reserve_pool, summed over every replica of the pool, equals
ckpt_cost times 16384 times the replica count. Therefore each device's share of it (that
pool-wide sum divided by the replica count) is exactly ckpt_cost times 16384 bytes, that is,
ckpt_cost slots, regardless of how many replicas the pool has.

Fact 6. ckpt_cost is defined as: the sum, over the allocation-record tree and the
central-mapping tree, of each tree's current height, plus the accounting tree's node count for
this publish, plus 1 for the tree table. The instance-table chain is not part of ckpt_cost (its
own cost is instance_switch_reserve, Fact 2 and Fact 7). Each tree's height is read from that
tree's own root node's header, as level + 1. For a publish that already has a file, all of these
numbers are read directly from the version being built on. The same ckpt_cost number is also
used as c_max, the worst-case cost of one warm-up empty publish.

Fact 7. instance_switch_reserve(d) is computed as follows. Let rows0 be the number of rows in
the instance table right after this mount's row-publish. Let N_switch = 3 and R = 3. Let
pages_of_chain = ceiling((rows0 + N_switch) / 369), with a minimum of 1 (369 = 370 - 1: one
instance-table page holds 370 records, of which the last is always a chain-pointer record, so
369 rows of real data fit on one page). Let chain_rewrite = pages_of_chain times 2 slots (one
instance-table page spans 2 slots). Let warm_up = c_max (Fact 6) times R, in slots. Let
one_switch = chain_rewrite + warm_up. Then instance_switch_reserve(d) = one_switch times
(N_switch + 1), the same number on every device.

Fact 8. Every unit written by a publish occupies a fixed number of 16 KiB slots, and belongs to
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

Fact 9. One physical slot, and also one 16 KiB metadata node, is 16384 bytes (this constant is
named SLOT_BYTES in the source).

Fact 10. The allocation-record tree is addressed by absolute slot position, not by content: its
shape depends only on the total number of absolute slots on each device, never on how many
records are actually written. Slot 50176 is where a device's unit area starts (this constant is
named UNIT_AREA_START_SLOT); a device configured with N unit-area slots therefore has
50176 + N absolute slots in total. One leaf of the allocation-record tree covers 812 consecutive
absolute slots (this constant is named ALLOCATION_RECORD_TREE_LEAF_SLOTS); one internal node has
fan-out 169 (this constant is named ALLOCATION_RECORD_TREE_INTERNAL_FANOUT). The tree's root
level is the smallest R greater than or equal to 1 such that, summed over every device in the
pool, ceiling(total_absolute_slots_of_that_device divided by (812 times 169 to the power of
(R-1))) is at most 169. The tree's height equals root level plus 1.

Fact 11. The central-mapping tree's height and the accounting tree's height (when a height is
wanted for either of them) are read the same way as the allocation-record tree's height: from
that tree's own root node's header, as level + 1. The accounting tree's contribution to
ckpt_cost (Fact 6) is its node count for this publish, not a height.

Fact 12. The central-mapping tree's node header is 169 bytes wide and each of its entries is 55
bytes, so one leaf holds at most floor((16384 - 169) / 55) = 294 entries. The accounting tree's
node header is 159 bytes wide and each of its entries is 34 bytes, so one leaf holds at most
floor((16384 - 159) / 34) = 477 entries.

Fact 13. Directly observed shape of a pool made of two 4 GiB devices, right after its very first
transaction (mkfs and the first file published in the same process, one data unit, no other
publish yet): the allocation-record tree's root is at level 2 (so its height is 3, by Fact 10's
rule). The central-mapping tree's root is also that tree's only node (level 0, so its height is
1, by Fact 11's rule) and holds exactly 10 entries. The accounting tree's root is also that
tree's only node (level 0) and holds 15 rows, so its node count for this publish is 1 (this row
count constant is named FIRST_TRANSACTION_ACCOUNTING_ROWS).

Fact 14. Besides the 4 GiB device used in Fact 13, this codebase's tests also use three
small-disk unit-area sizes: 384 slots, 256 slots and 240 slots.

Fact 15. A code comment attached to the 384-slot small-disk configuration states, in the
author's own words and marked as approximate: after the first file, each device holds about 13
slots; one more overwrite of the same shape (content that still fits inside one data unit)
occupies roughly 10 more slots per device, and this growth continues for dozens of publishes
before reclamation catches up.

Fact 16. A code comment attached to the 256-slot small-disk configuration states, in the
author's own words and marked as approximate: once the root ring has turned over once,
steady-state occupancy on a small disk is roughly the accounts retained by the 24 generations
kept in the root ring, about 10 slots per device per generation; at that steady-state occupancy
the 384-slot disk's raw allocator (not the admission formula discussed in Fact 1) never runs out
of positions, while the 256-slot disk's raw allocator does run out of positions for user data.

Fact 17. In both small-disk histories described in Fact 18 and Fact 19, the pool has exactly 2
devices and therefore exactly 2 replicas (one copy of every unit on each device). From mkfs
onward, each of these two histories consists only of two kinds of step: mounting the pool
writable, and publishing an overwrite of the existing file's content. Neither history ever
performs an administrator rollback.

Fact 18. Setup for a 256-slot device, taken directly from a harness test. Starting from a pool
that has just published its first file (mkfs and the first file publish happen in the same
process), the pool is closed and mounted writable once, then 4 overwrites of the file are
published one after another; every one of those 4 overwrites uses content that still fits inside
a single data unit, and every one of those 4 overwrites is admitted with a test-only switch that
skips the admission formula entirely (so the formula's verdict on any of those 4 overwrites is
not observed). After those 4 overwrites, the image is reopened in a brand-new writable mount,
and this new mount is admitted by the formula (it asks for 0 ordinary allocation, being a plain
mount). In that new mount, one further overwrite of the same shape (content that still fits
inside a single data unit) is attempted, and this time the admission formula is consulted for
that one overwrite before anything is read from disk or written.

Fact 19. Setup for a 384-slot device, taken directly from a harness test. Starting from a pool
that has just published its first file, the pool is closed and mounted writable once, then 10
overwrites of the file are published one after another; every one of those 10 overwrites uses
content that still fits inside a single data unit, and every one of those 10 overwrites is
admitted with the same test-only switch that skips the admission formula entirely. After those
10 overwrites, the image is reopened for a brand-new writable mount, and this time the admission
formula is consulted for the mount itself, before acquisition, with demand(d) = 0 on every
device (a plain writable mount, with no write following it yet, does not add to demand(d)).

Item 1. The 256-slot device and the one additional overwrite described in Fact 18.

Fill in every row below for one device d (by symmetry the two devices in this pool are
identical in every one of these rows, so one worksheet covers both).

Row 1. capacity(d), in slots.
Row 2. allocated(d), in slots, at the moment this additional overwrite is about to be checked
  (right after the new mount of Fact 18, right before this overwrite).
Row 3. unreclaimable(d), in slots.
Row 4. deferred_pending_release(d), in slots, at that same moment.
Row 5. instance_switch_reserve(d), in slots, for this mount (this is the whole of
  mount_time_commitment(d), by Fact 2). Show rows0, pages_of_chain, chain_rewrite, warm_up and
  one_switch from Fact 7 as you compute this.
Row 6. abandoned_root_exclusive(d), in slots.
Row 7. pending_delete_occupancy divided by replicas, in slots.
Row 8. committed_reservation divided by replicas, in slots.
Row 9. ckpt_cost, in 16 KiB blocks, for the version this overwrite would build on. Show the four
  components you summed, per Fact 6.
Row 10. checkpoint_reserve_pool divided by replicas, in slots (by Fact 5 this should equal
  row 9 exactly).
Row 11. available(d), in slots: row 1 minus every one of rows 2 through 6, minus rows 7, 8 and
  10 (do not subtract row 9 separately; row 9 only feeds into row 10).
Row 12. demand(d), in slots, for this one additional overwrite: list every unit from Fact 8's
  Demand category that an overwrite of an existing, unchanged-length file rewrites, with each
  unit's span in slots, and sum them.
Row 13. Is available(d) (row 11) at least demand(d) (row 12)? State yes or no, and by how many
  slots.

This would be refuted by: [one sentence].

Item 2. The 384-slot device at the moment of the fresh mount described in Fact 19.

Fill in every row below for one device d (by symmetry the two devices in this pool are
identical in every one of these rows).

Row 1. capacity(d), in slots.
Row 2. allocated(d), in slots, at the moment of this fresh mount (right after the 10 overwrites
  of Fact 19, right before this mount is checked).
Row 3. unreclaimable(d), in slots.
Row 4. deferred_pending_release(d), in slots, at that same moment.
Row 5. instance_switch_reserve(d), in slots, for this mount. Show rows0, pages_of_chain,
  chain_rewrite, warm_up and one_switch from Fact 7 as you compute this.
Row 6. abandoned_root_exclusive(d), in slots.
Row 7. pending_delete_occupancy divided by replicas, in slots.
Row 8. committed_reservation divided by replicas, in slots.
Row 9. ckpt_cost, in 16 KiB blocks, for the version this mount would build on. Show the four
  components you summed, per Fact 6.
Row 10. checkpoint_reserve_pool divided by replicas, in slots.
Row 11. available(d), in slots: row 1 minus every one of rows 2 through 6, minus rows 7, 8 and
  10.
Row 12. demand(d), in slots, for this mount by itself (Fact 19 already tells you what this is;
  state it and say why).
Row 13. Is available(d) (row 11) at least demand(d) (row 12)? State yes or no, and by how many
  slots.

This would be refuted by: [one sentence].

Item 3. ckpt_cost for a publish that would build on the very first transaction described in
Fact 13 (two 4 GiB devices).

Row 1. Allocation-record tree's height right after the first transaction (Fact 13).
Row 2. Central-mapping tree's height right after the first transaction (Fact 13).
Row 3. Accounting tree's node count for this publish, right after the first transaction
  (Fact 13).
Row 4. Tree table's contribution (Fact 6).
Row 5. ckpt_cost, in 16 KiB blocks: sum of rows 1 through 4.
Row 6. ckpt_cost, in bytes: row 5 times 16384.

State explicitly, one sentence each, which of the Sigma-list members named in Fact 6
(allocation-record tree, central-mapping tree, accounting tree, tree table, instance-table
chain) you included in row 5 and which one you excluded, and why.

This would be refuted by: [one sentence].

Item 4. For the 384-slot device, starting from which number of completed overwrites does a
subsequent writable mount first get refused by the admission formula?

Let N be the number of completed overwrites of the small-disk kind described in Fact 19,
counting from N = 0 (the moment right after the first file is published, before any such
overwrite has ever been published on this device). Using Fact 15's growth note, give your best
estimate of allocated(d) as a function of N, for N from 0 up to at least 12. For every other row
of item 2's worksheet (rows 3 through 10), state whether you believe that row changes with N or
stays fixed as N grows from 0 to 12, and say which fact supports your belief. Then, combining
these, give your best estimate of available(d) as a function of N, and state the smallest value
of N for which you estimate that a subsequent writable mount (demand(d) = 0, as in item 2) would
be refused. State explicitly that this uses Fact 15's approximate growth note rather than an
exact count, so your estimate might be off by a few.

This would be refuted by: [one sentence].

End of task. Answer items 1 through 4 in order, each with its row-by-row worksheet filled in and
its closing "This would be refuted by:" sentence. Do not use any markdown emphasis (no bold, no
italics, no headings marked with number signs, no backtick code spans) anywhere in your answer;
plain numbered lines are fine. Do not write any file name or any line number anywhere in your
answer; refer only to fact numbers, row numbers, and the exact constant or function names given
above.
