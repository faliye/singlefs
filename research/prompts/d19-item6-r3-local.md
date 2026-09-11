You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A central map is the only entry point for dereference and for the free decision.
Pointer position entries are hints only. The map must be rebuildable by scanning
unit headers. Relocation copies a unit byte for byte; only the map entry changes.

Unit classes: code 1 data units, code 2 index nodes, code 3 packed record units
(for example the inode tree leaves). Small objects may also be packed into
shared containers whose slots carry the object's own identity.

Facts you can rely on:
F1. Everything in a unit header is the value it had at birth; relocation never
    changes it.
F2. Today a pointer is 59 bytes and carries no birth checkpoint number and no
    birth tree.
F3. Every unit header has a birth tree id and a birth checkpoint number. A code 1
    header also has a write order field made of a 4 byte instance id and a 6 byte
    transaction number. A code 2 header has only the 4 byte instance id in that
    field.
F4. The snapshot free decision reads the birth checkpoint number from the pointer
    with zero extra I/O. Shared (reflinked or cloned) blocks do not use that path.
F5. Checkpoint numbers: a published number is never issued again. An in-flight
    number that was never published is issued again after a crash or an in-mount
    instance switch. In an instance switch, transactions numbered at most W keep
    their units (those units are adopted into the reissued checkpoint and count as
    published); transactions above W are redone with new write orders.
F6. Each transaction writes at most one code 1 unit. Transaction numbers count
    from 1 within an instance. Every abandoned checkpoint forces a new instance id.
    A single-copy write failure retries at another location with the same write
    order.
F7. Code 3 containers are written at the checkpoint fixed point. Several fixed
    point iterations in one checkpoint can write the same container more than once
    with an identical write order.
F8. When a cloned head first copies-on-write a shared inode leaf, the new leaf's
    birth tree becomes the writing tree.
F9. Index nodes of derived trees are rebuilt from leaves during a scan rebuild;
    only the accounting tree's index nodes are authoritative and must be recovered
    as they were.

Seven candidate map keys:

A. Logical identity plus write order. Code 1 key is the 33 byte logical five-tuple
   plus birth checkpoint 8 plus write order 10, 51 bytes, and the pointer to a data
   unit carries all of it. Code 3 key is its container identity 26 plus birth
   checkpoint 8 plus write order 10. Code 2 key is tree id, level, lower key bound,
   birth checkpoint; level and lower bound come from the lookup path.
B. Birth identity plus instance. All classes: birth tree 8, birth checkpoint 8,
   instance id 4, birth sequence 4, 24 bytes. The birth sequence counts from 0 per
   (tree, checkpoint number, instance), issued in memory. The pointer carries all
   24 bytes. Every header adds the 4 byte sequence.
C. Class-wise reuse of write order. Code 1 key is birth tree 8, birth checkpoint 8,
   write order 10. Code 2 and code 3 key is birth tree 8, birth checkpoint 8,
   instance id 4, birth sequence 4. Only code 2 and code 3 headers add 4 bytes.
D. Birth location. Birth device 4, birth slot 6, birth checkpoint 8. The pointer's
   position entry is frozen at the birth location; slots returned by a failed write
   are not reusable before the checkpoint publishes.
E. Per-instance counter. Instance id 4 plus a counter 8 that starts at 1 in each
   instance. Every header adds the counter.
F. Random 16 byte identifier in every header and pointer.
G. Birth location plus a 2 byte per-slot version number kept in a persistent table.

YOUR TASK

For each item, give a concrete counterexample with a specific operation sequence
and specific values, or say in one sentence that you could not. Try hardest
against C, then B.

Q1. Find two live units that end up with the same key under a candidate. Consider
    fixed point rewrites, reflink followed by overwrite in one checkpoint, instance
    switches with adoption, crash and remount, relocation, and clones.
Q2. Find a unit that a scan over headers cannot give the exact key the pointer
    holds, for a unit class that must be recovered as it was.
Q3. Find a path that must enter the map but does not hold the full key without
    an extra unit read or tree lookup.
Q4. For C specifically: find a case where two different code 1 units have the same
    (birth tree, birth checkpoint, write order).
Q5. For G: say where the per-slot version table must live and construct a crash or
    wraparound case that makes two live units share a key.
Q6. Construct an eighth key family that is not worse than any of A to G on Q1 to
    Q3 and strictly better than at least one of them on at least one question.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong.
