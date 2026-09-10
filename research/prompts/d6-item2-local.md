You are one leg of a three-way design review for a from-scratch copy-on-write filesystem
called singlefs. Your assigned stance is adversarial construction: build concrete
counterexample scenarios. Do not use any markdown emphasis in your answer. Answer in English.

Background facts. Quote them as given; do not assume anything beyond them.

1. The filesystem supports multiple writable heads. A settled decision states that each
   writable head gets its own tree. Heads are created by cloning from an origin snapshot.
2. Every block carries a birth transaction group number, written when the block is first
   allocated. Transaction group numbers are globally ordered.
3. Space accounting uses the ZFS style birth-txg plus deadlist scheme. Each writable head
   holds one prev_snap_txg scalar and one deadlist. When a block is deleted, the runtime
   compares the block birth against the head prev_snap_txg. If birth is greater than
   prev_snap_txg the block is released immediately, otherwise it is appended to the deadlist.
   This decision is O(1) and uses no additional I/O.
4. A written clause says: destroying one writable head requires a second structure, in the
   form of a ZFS livelist, holding ALLOC and FREE pairs grouped by birth txg, incrementally
   maintained and condensable. The stated reason for requiring it is: without it, the promise
   of never traversing after the fact is false on this path, because the blocks to release are
   those with birth greater than the origin txg, and they are scattered across the whole tree.
5. A different written clause in the same project says: destroying a snapshot falls into the
   third bucket of a three-bucket discipline. The third bucket is background, resumable,
   non-decision paths, and it carries no prohibitions, so an O(N) traversal is explicitly
   permitted there by project discipline.
6. A written admission criterion for any snapshot implementation model says: bounded
   destruction, meaning destruction must be batchable, resumable, and each batch must have a
   bounded worst-case space requirement that can be reserved before the batch starts. The same
   criterion carries a note saying the current accounting scheme dodges this criterion by
   forbidding destruction of branch-point snapshots, which is a missing feature rather than
   satisfaction of the criterion.
7. A separate written rule says: releasing space must not itself require allocating space.

The four candidates under review for the second structure:

A. A dedicated per-head livelist as described in fact 4.
B. No structure at all. Destroying a writable head performs a batchable, resumable traversal,
   relying on the third-bucket exemption in fact 5.
C. Reuse a separate sparse side table, currently undefined, whose purpose is reference
   counting of blocks shared across heads.
D. First version omits the feature: writable heads may not be destroyed.

Your task. Construct concrete scenarios, with specific transaction group numbers, specific
block births, and specific crash points, that answer these questions:

Q1. Construct a workload and crash sequence in which candidate B loses data, leaks space, or
    violates fact 7. If you cannot construct one, say so and explain what blocks the
    construction.
Q2. Candidate B must be resumable. Where does the resume progress live across a crash? Show a
    scenario where the resume progress itself must be persisted, and state whether that makes
    B differ from A only in shape rather than in whether a persistent structure exists.
Q3. Candidate C reference counts blocks shared across heads. Candidate A tracks blocks born
    inside the clone itself, that is birth greater than origin txg. Construct a concrete block
    that is covered by exactly one of the two and not the other, in both directions, to show
    whether the two structures can be merged.
Q4. Candidate D forbids destroying a writable head. Construct the space consequence over a
    long-running system: state precisely what becomes unreclaimable and under which sequence.
Q5. Which of the four candidates turns a misuse into a mount refusal rather than a silent
    misread? Answer for each candidate separately.

Rules for your answer. Every claim must come with a concrete scenario containing numbers.
Do not argue from what other filesystems do. If a question cannot be answered from the facts
given, say which fact is missing.
