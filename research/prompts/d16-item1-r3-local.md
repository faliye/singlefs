You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against the two proposals below. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Every publication increments checkpoint_txg by exactly one. The root ring has
    R = 3 regions and S slots per region, ring depth N = 3 * S. Each publication
    writes one root slot, region = txg mod 3. Regions 0 and 2 live on disk 0,
    region 1 lives on disk 1. Publish order: new units, barrier, journal record,
    barrier, root slot with FUA.
F2. A block freed in publication f may be reallocated only when a reuse bound
    allows it. If a block is reused while some candidate root still references it,
    that root becomes a fake rollback candidate: it verifies, but its tree points at
    overwritten content. If crash recovery picks such a root, it is unrecoverable.
F3. Recovery verifies every root slot, picks the newest valid root, reloads
    allocator state from it, takes a new instance id and writes an instance table
    row that invalidates later roots of the crashed instance.
F4. A root slot write can fail; then the slot keeps its old root, the txg advances
    by one and the root is retried in the next slot.
F5. A non-empty user window deletes one 8-block object, frees the previous
    publication's 5 metadata blocks, creates one 8-block object and writes 5 new
    metadata blocks. An empty publication frees the previous metadata and writes
    c_empty blocks. In the simulation c_empty was a constant 1 or 5; in a real
    filesystem it grows with tree height (allocation-record tree, accounting tree,
    tree table).
F6. User requirement 1: if df reports at least s free bytes, a write of s bytes must
    succeed, possibly after a bounded amount of internal work. User requirement 2:
    after deleting an object of s bytes, a write of the same size must succeed within
    a computable number of steps.
F7. A deterministic simulation (capacity 4000 blocks, 24 seeds, S in {1, 4, 16},
    c_empty in {1, 5}) found for proposal A: zero fake candidates and zero
    unrecoverable roots in steady churn, torn newest slot, loss of either disk, slot
    write failure, administrator rollback, and 16 slot-failure patterns times 8 crash
    points; zero violations of requirement 1 and 2; at most N publications per
    admission. It did not model in-mount instance switches, the journal replay window,
    or c_empty growing over time.
F8. Near full with S = 16, proposal A needed about 36 to 39 forced publications per
    user write and hid 108 blocks (c_empty = 1) or 480 blocks (c_empty = 5) from df.
    Among its 48 rollback candidates, only about 2.2 on average were non-empty
    publications; the rest were empty publications produced by draining.

THE PROPOSALS

Proposal A (full ring plus drain). Reuse bound = txg of the oldest valid root
actually persisted in the ring (a failed slot counts with its old content).
Candidates = all valid ring roots. When admission of a write fails: publish the
open window first (it already contains the delete), then publish up to N - 1 empty
publications, stopping as soon as the write fits; otherwise report ENOSPC.
Reserve (only publications may use it) = 10 + (N - 1) * c_empty blocks.
df reports: unused + freed blocks - reserve - (5 + (N - 2) * c_empty).

Proposal G-prime (per-disk rollback floor, always drain fully). Every root record
carries a floor F. Reuse bound = max(F_active, oldest persisted valid root txg).
Candidates = valid roots with txg >= F_recovered, where F_recovered is the maximum
F over every self-verifying root still on the surviving disks, including roots of an
abandoned timeline. F may be raised at most to min(newest persisted valid root txg
on each disk, txg of the 4th newest persisted valid root). F becomes active only when
every surviving disk holds at least one persisted valid root carrying it. After a
crash, F_active = minimum over surviving disks of the maximum F carried on that disk.
When admission fails: publish the open window (txg t); keep publishing empty
publications; once the cap reaches t raise F to exactly t; always continue until F = t
is active (at most 7 publications in total), even if the write would already fit.
Reserve = 10 + 6 * c_empty. df reports: unused + freed - reserve - (5 + 5 * c_empty).

WHAT TO PRODUCE

1. For proposals A and G-prime: a concrete sequence involving an in-mount instance
   switch or a journal replay after a crash, after which a candidate root references a
   reused block, or recovery picks an unrecoverable root.
2. For proposals A and G-prime: a sequence in which c_empty grows (for example the
   allocation-record tree gains a level) so that df reports at least 8 free blocks but
   an 8-block write fails after the bounded drain, or a delete of 8 blocks is not
   followed by a successful 8-block write.
3. For proposal G-prime: a sequence with a crash inside the interval between raising F
   and F becoming active, without losing a disk; and a sequence where a slot write
   failure makes the txg jump before F is raised. In either, show a fake candidate or
   a violation of the rule "at least 4 persisted roots and each disk's newest valid
   root stay candidates".
4. For both: a geometry or slot landing in which the number of publications needed
   exceeds the stated bound (N for A, 7 for G-prime).
If you find nothing for an item, say so plainly and list what you tried.
