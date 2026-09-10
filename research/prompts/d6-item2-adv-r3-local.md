You are one leg of a three-way adversarial review for a from-scratch copy-on-write filesystem.
Your assigned stance is adversarial construction. Build concrete comparisons with specific numbers.
Do not use any markdown emphasis in your answer. Answer in English.

This is round three of three. The attack surface for this round is: does the owner's stated reason
actually hold, and are the load-bearing numbers correct when recomputed by hand. Do not attack
on-disk format details or crash recovery: rounds one and two covered those.

The decision under review. To destroy a writable head, the blocks born inside that head must be
released, which needs a dedicated structure. Two carriers are compared.

PER HEAD: each head gets its own livelist tree. Destroying the head drops that whole tree.
SHARED: one livelist tree for the whole pool, with the head tree id as the key prefix. Destroying a
head deletes the key range with that prefix.

The owner chose PER HEAD and gave this reason, verbatim in translation: we have always intended to
trade space for complexity. In other words, spend more space to get a simpler system.

Settled facts. Treat them as given.

1. All indexes are one btree implementation with several independent keyspaces, one tree per
   kind of data.
2. For trees that go through the write buffer front end, a deletion is written as a whiteout.
   A settled clause says whiteouts are cleared only when a node is rewritten and its batches are
   merged, and that no other clearing path is allowed. When nodes get rewritten is not decided.
3. Every tree is registered in a tree table. A tree table unit holds 134 entries per level.
4. Each head has at least two data trees, extent and inode. The pool has three pool-level trees:
   allocation records, accounting, and the central map.
5. A long operation such as destroying a head runs as an intent: committed first, done in batches,
   redone idempotently after a crash, and every intermediate state must be legal.
6. Both carriers need an entry counter per head, and both need the livelist entries to be keyed by
   the central map identity rather than by a physical location hint, because physical locations
   can go stale when blocks are moved.
7. A tree table entry has no field saying which head it belongs to.
8. Newly adopted: a writable head that still has live snapshots of its own cannot be destroyed.
   Its own snapshots must be destroyed first. The origin snapshot it was cloned from does not count
   as its own. There is no promote operation. A snapshot that is a branch point, meaning it has
   more than one child, cannot be destroyed either.

Questions.

Q1. For each carrier, list every mechanism that must be implemented and verified to support
    destroying a head: creation at clone time, recording ALLOC and FREE, snapshot handling,
    destruction in batches, crash resume, counter maintenance, and any cleanup afterwards. For each
    mechanism, count the distinct crash points. Then state whether PER HEAD really has fewer
    mechanisms or fewer crash points than SHARED. If it does not, the owner's reason fails. Show the
    two lists side by side.
Q2. SHARED destroys a head of 65536 live entries by deleting a key range. Using fact 2, count the
    whiteouts left behind, and state what bounds how long they stay. PER HEAD drops the whole tree.
    Construct the difference with numbers.
Q3. Recompute by hand. With 134 entries per level, PER HEAD uses 3H + 3 tree table entries for H
    heads (two data trees and one livelist per head, plus three pool trees), SHARED uses 2H + 4.
    For each, find the smallest H at which a second level is needed and the smallest H at which a
    third level is needed. Also do it for 4H + 3. Show the arithmetic.
Q4. Recompute by hand. A node is 16384 bytes with a 64 byte header. A leaf holds entries of width w.
    An internal node holds entries of width w + 59. For w = 24 and for w = 32, compute the leaf
    fanout, the internal fanout, and how many entries fit in two levels. Show the arithmetic.
Q5. The guard in fact 8, with no promote. Construct a sequence of legal operations after which
    some writable head can never be destroyed. State exactly which blocks stay allocated forever,
    and whether any operation in the facts can undo it. Then say whether this is a real cost or
    only a missing feature.

Rules. Every claim needs numbers. Do not argue from what other filesystems do. If a question cannot
be answered from the facts given, name the missing fact.
