You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against the proposals below. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Every publication increments checkpoint_txg by exactly one; fsync-triggered
    publications count. The root ring has R = 3 regions and S slots per region,
    ring depth N = 3 * S. Each publication writes one root slot (region = txg mod 3).
    Publish order: new units, barrier, journal record, barrier, root slot with FUA.
F2. A block freed in publication f may be reallocated only when a reuse bound
    allows it. If a block is reused while some candidate root still references it,
    that root becomes a fake rollback candidate: it verifies, but its tree points at
    overwritten content. That is detectable by checksum but unrecoverable.
F3. Recovery verifies every root slot, picks the newest valid root (ties by the
    higher instance id), reloads allocator state from that root, takes a new
    instance id and writes an instance table row that invalidates later roots of the
    crashed instance. On two disks, losing disk 0 can force a fallback of 2
    generations, so the minimum protected depth K_min is 3.
F4. Administrator rollback (a user decision): pick any valid root in the ring;
    reload allocator and accounting from it; new instance id; a rollback row marks
    the abandoned timeline invalid; the first new root gets txg = max ring txg + 1.
    The rollback and its first new root are one publication. (Known separate hole:
    if that publication crashes before the root slot lands, the abandoned roots are
    still valid and may reference blocks the rollback publication reused. This hole
    exists for every proposal below; do not report it again.)
F5. A root slot write can fail; then the slot keeps its old root and the txg
    advances by one and is retried in the next slot.
F6. Admission control reserves a pool of 13 blocks that only publications may use.
    Every non-empty publication frees 13 blocks and allocates 13 blocks. An empty
    publication rewrites c_empty blocks (1 or 5).
F7. A deterministic simulation (24 seeds, S in {1, 4, 16}) found zero fake
    candidates for proposals A and D in: steady churn, torn newest root slot, loss
    of disk 0, root slot write failure, and administrator rollback. It did not model
    in-mount instance switches or the journal replay window.
F8. Near full (97 percent), with S = 16 and c_empty = 1, proposal A needed 8.79
    forced empty publications per user write on average and kept all 48 rollback
    candidates; proposal D needed 0.80 and kept on average 8 candidates, minimum 4.
    With c_empty = 5 and S = 16, proposal D stalled (could not publish) in all 24
    seeds while filling; proposal A did not stall but could only fill to about
    93.7 percent and needed 47 forced publications per write.

THE PROPOSALS

Proposal A (full ring plus drain). Reuse bound = txg of the oldest root actually
persisted in the ring (a failed slot counts with its old content). Candidates =
all valid ring roots. When admission fails, publish empty publications until the
frees in the window come back; only then report ENOSPC.

Proposal C (narrow the promise). Reuse bound = current txg - K_min + 1. Candidates
are narrowed to the newest K_min roots. Nothing special when admission fails.

Proposal D (dynamic rollback floor). Every root record carries a floor F. Reuse
bound = max(F_active, oldest persisted ring root txg). Candidates = valid roots
with txg >= F_recovered, where F_recovered is the maximum F over every
self-verifying root still on disk, including roots of an abandoned timeline. When
admission fails, raise F to current - K_min + 1; F only increases; F becomes active
only after the first root carrying it is durable and K_min further publications
are durable.

Proposal D-prime. Proposal D plus a mount-time admission rule: the publication-only
reserve must be at least (K_min + 1) * c_empty.

WHAT TO PRODUCE

1. For proposals A, D and D-prime: a concrete sequence involving an in-mount
   instance switch (the current checkpoint is abandoned, a new instance id is
   taken, in-flight checkpoints are re-published, and transactions numbered at most
   W keep their units) or a journal replay after a crash, after which a candidate
   root references a reused block.
2. For proposal D-prime: a sequence of frees, publications and floor raises that
   still stalls (a publication cannot allocate even its own blocks) despite the
   reserve rule.
3. For proposal A: a near-full sequence in which deleting a file still does not
   let a same-size write succeed, even after the drain.
4. For proposal D: a sequence in which an administrator cannot tell how far back a
   rollback can go, and a rollback silently lands on a root the administrator did
   not intend.
If you find nothing for an item, say so plainly and list what you tried.
