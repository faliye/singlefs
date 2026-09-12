You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. A block's birth is the checkpoint number in which it was published. A block
    referenced by a root R (a snapshot or a live head) must have birth less than or
    equal to R.txg. Treating a block with birth equal to S.txg as newer than snapshot S
    frees it immediately, which loses data.
F2. Free decision when a block dies in a head: if pointer.birth is greater than
    head.previous_snapshot_txg, free now; otherwise append to the head's deadlist.
    A clone's previous_snapshot_txg starts at its origin snapshot's txg.
F3. Snapshots and clones are created in one checkpoint without touching any published
    block, so a new snapshot or clone shares every index node (class 2) of the tree it
    was taken from until the live head rewrites them.
F4. A class 2 node header has: tree ID, level, birth generation, instance id 4,
    birth serial 4, payload CRC. Its central-map key is (class 2, birth tree, birth txg,
    instance id, birth serial). Every pointer to it (parent node entry, tree table root
    pointer, pointers in other trees that share it) carries the same key fields.
    Deadlist and livelist entries also name it by this key.
F5. Published predicate for class 2: a node written by instance i counts as published
    if i has no row in the instance table and i is older than the mounting instance;
    if i has a row (i, T_pub, W), it counts as published only if birth <= T_pub. This
    is what lets an unpublished node from a crashed checkpoint be swept as garbage.
F6. Instance id: a writable mount takes a new id = max + 1; a read-only mount takes
    none. Birth serial is counted per (tree, checkpoint, instance id).
F7. Rebuild levels: with allocation records plus checksum evidence the rebuild may
    become writable; with checksums only it is read-only mount; in that state no
    authoritative structure may be repaired from rebuild results. Today there is no
    witness that allows the writable level after an index rebuild.
F8. Allocation records treat a unit as an orphan only if its write-sequence instance is
    older than the current instance and it is judged unpublished.
F9. How to pick the snapshot-time historical version of an index node from the leaves
    is not specified anywhere.

OPTIONS (both share part 1)

Part 1, for class 2 nodes referenced only by the live head: rebuild by R1 (copy the
four key fields and birth from any readable holder, treat as relocation, change only
the map entry) or, if no holder survives, R2 (write as a normal new node: birth = the
checkpoint that publishes it, instance id = current writable mount, fresh serial).

Part 2, for class 2 nodes referenced by a live snapshot or a clone origin, or named by
a deadlist or livelist entry:
C-prime. Only R1 may be persisted. If R1 is impossible, the pool stays read-only until
         that snapshot is destroyed.
D. If R1 is impossible, still persist: fresh identity with birth = the smallest txg
   among all snapshots and clone origins referencing it, instance id = current writable
   mount, fresh serial.

WHAT TO PRODUCE

1. A node whose part 1 or part 2 membership cannot be decided at rebuild time from the
   tree table, units and accounting alone.
2. A sequence where R1 copies a wrong key or birth (holders disagree or are stale) and a
   reader then gets a wrong answer.
3. For D: a sequence where the fresh identity gives a wrong answer. Try the published
   predicate in F5 with a crash before the rebuild is published, key collisions, and
   F1.
4. For part 1 R2: any wrong answer for a node referenced only by the live head,
   including later snapshots or clones taken after the rebuild.
5. For C-prime: is there any way out of read-only once entered? Destroying the snapshot
   needs writes.
If you find nothing for an item, say so plainly and list what you tried.
