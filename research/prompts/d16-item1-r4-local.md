You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against the two proposals below and to answer one existence question. Do not argue
which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Every publication increments txg by exactly one. The root ring has R = 3
    regions and S slots per region, ring depth N = 3 * S. Each publication writes
    one root slot, region = txg mod 3, slot within region = floor(txg / 3) mod S.
    Regions 0 and 2 live on disk 0, region 1 lives on disk 1. Publish order: new
    units, barrier, journal record, barrier, root slot with FUA.
F2. A block freed in publication f may be reallocated only when a reuse bound
    allows it. Every root with txg between the block's allocation and f references
    the block. If a block is reused while some candidate root still references it,
    that root becomes a fake rollback candidate: it verifies, but its tree points at
    overwritten content. If crash recovery picks such a root, it is unrecoverable.
F3. Recovery verifies every root slot, picks the newest valid root, reloads
    allocator state from it, takes a new instance id and writes an instance table
    row that invalidates later roots of the crashed instance.
F4. A root slot write can fail; then the slot keeps its old root (which stays a
    valid root), the txg advances by one and the root is retried in the next slot.
    No new blocks are allocated for the retry.
F5. A non-empty user window deletes one 8-block object, frees the previous
    publication's metadata blocks, creates one 8-block object and writes 5 new
    metadata blocks. An empty publication frees the previous metadata and writes
    c blocks. c is c0 at mount and may grow later; the design declares an upper
    bound c_max and sizes everything by c_max. The simulation used (c0, c_max) =
    (1, 5) and (5, 10).
F6. User requirement 1: if df reports at least s free bytes, a write of s bytes must
    succeed, possibly after a bounded amount of internal work. User requirement 2:
    after deleting an object of s bytes, a write of the same size must succeed within
    a computable number of steps.
F7. User rule on rollback history: when the disk is nearly full the candidate set may
    shrink, but at least 4 distinct user-visible states must remain candidates
    (empty publications produced by draining do not count as states), and the newest
    valid root on every disk must remain a candidate.
F8. A deterministic simulation (capacity 4000 blocks, 24 seeds, S in {1, 4, 16})
    found for proposal G2R: zero fake candidates and zero unrecoverable roots in
    steady churn, torn newest slot, loss of either disk, slot write failure,
    administrator rollback, faults inside the interval between raising F and F
    becoming active (with and without disk loss, and after slot failures that made
    the txg jump before F was raised), 1 to 3 slot write failures during a drain, a
    slot that fails forever, and c growing from c0 to c_max. In every one of those
    cells it had zero false ENOSPC, every delete was followed by a successful write,
    and an admission needed at most 11 publications (exactly 11 with 2 failures).
F9. For proposal G2S the same simulation found the same correctness and zero false
    ENOSPC, at most 7 publications per admission, and at least 4 distinct states as
    candidates at all times, but: after the disk first became full, the first 3
    delete-then-write windows failed (the write, not the delete) in every seed, and
    afterwards none failed for 400 windows. With S = 4 and 3 slot failures inside one
    drain (more than the tolerated 2), the 12-slot ring overwrote two of the four
    states that had been counted when F was raised.
F10. The simulation did not model in-mount instance switches, the journal replay
    window, c exceeding c_max, more than 3 failures in one drain, metadata of
    non-empty publications growing with tree height, or rings other than S in
    {1, 4, 16} on two disks.

THE PROPOSALS

Proposal G2R (per-disk rollback floor, counting roots). Every root record carries
a floor F. Reuse bound = max(F_active, txg of the oldest persisted valid root).
Candidates = valid roots with txg >= F_recovered, where F_recovered is the maximum F
over every self-verifying root still on the surviving disks. F may be raised at
most to min(newest persisted valid root txg on each disk, txg of the 4th newest
persisted valid root). F becomes active only when every surviving disk holds at
least one persisted valid root carrying it. After a crash, F_active = minimum over
surviving disks of the maximum F carried on that disk. When admission of a write
fails: publish the open window (txg t, it already contains the delete); then before
each empty publication, if the cap has reached t and F < t raise F to exactly t;
keep publishing empty publications until the write fits or the blocks freed at t
are reusable, at most 10 empty publications (bound 11 publications in total, that
is 7 plus 2 per tolerated slot failure, 2 tolerated). Reserve, only publications may
use it: 10 + 10 * c_max blocks. Live metadata blocks count as reserve usage.
Admission grants min(reusable + live metadata - reserve, df_R) blocks, where
df_R = unused + freed + live metadata - reserve - (5 + 9 * c_max).

Proposal G2S (per-disk rollback floor, counting states). Same as G2R except: the
cap on F is min(newest persisted valid root txg on each disk, txg of the 4th newest
non-empty persisted valid root); when fewer than 4 non-empty valid roots exist the
cap is the oldest valid root. The drain target is min(t, that 4th newest non-empty
root). Bound 8 publications (4 plus 2 per tolerated failure). Reserve 10 + 7 * c_max.
df_S = unused + freed + live metadata - reserve - (5 + 6 * c_max) - lag, where lag
counts freed blocks whose free txg is above max(4th newest non-empty valid root,
oldest valid root) and that were not freed by an empty publication.

WHAT TO PRODUCE

1. For G2R: a concrete sequence, using something the simulation did not model (F10),
   after which a candidate root references a reused block, or df_R reports at least 8
   free blocks and an 8-block write fails after at most 11 publications, or a delete
   of 8 blocks is not followed by a successful 8-block write within 11 publications.
2. For G2S: the same, using the bound 8 and df_S, excluding the first 3 windows after
   the disk first became full (those are already known to fail).
3. The existence question. Stay inside the per-disk rollback floor family: one floor
   F carried in root records, candidates are valid roots with txg >= F, reuse bound
   max(F_active, oldest persisted valid root). Either construct a variant in which at
   every moment at least 4 distinct user-visible states remain candidates and, after
   a delete at txg t on a nearly full disk, the blocks freed at t become reusable
   within a bounded number of publications without any further user-visible state
   change; or give a short argument why no such variant exists in this family. If you
   construct one, show what it does in the simulation's near-full delete-then-write
   loop.
4. The bound arithmetic. Give a slot landing and a pattern of k slot write failures
   inside one drain for which G2R needs more than 7 + 2k publications, or G2S more
   than 4 + 2k. Also give the smallest ring depth N, as a function of the drain
   length, for which G2S can keep 4 states as candidates through a drain that burns
   k extra txgs.
If you find nothing for an item, say so plainly and list what you tried.
