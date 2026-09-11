You are the counterexample leg of a three-way adversarial review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against one verdict. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A central map is the only entry point for dereference and for the free decision.
Pointer position entries are hints only. The map is derived state: it must be
rebuildable by scanning unit headers. Relocation copies a unit byte for byte;
only the map entry changes.

Unit classes: code 1 data units, code 2 index nodes (B-tree nodes), code 3 packed
record units (for example the inode tree leaves). Index nodes and code 3 units are
written only at a checkpoint's fixed point.

Facts you can rely on:
H1. A unit header never changes after birth. Every unit header carries its birth
    tree id and its birth checkpoint number. Code 1 and code 3 headers carry a
    10 byte write order: a 4 byte instance id plus a 6 byte transaction number.
    A code 2 header carries only the 4 byte instance id in that place.
H2. Transaction numbers count from 1 within an instance. Writes are split into
    transactions per unit: one transaction writes at most one data unit's user data
    (this sentence is still pending user review).
H3. Every writable mount takes a fresh instance id = max(ids seen) + 1 before it
    touches any unit; read-only mounts take none. So an instance id is never reused.
H4. Instance switch (inside a mount, after a transient write failure): take a fresh
    instance id, write a row (old instance, last published checkpoint, last applied
    transaction number W), re-issue the in-flight checkpoint; transactions numbered
    at most W are kept, those above W are redone with new write orders or reported
    as errors; all fixed-point units are rewritten under the new instance.
H5. A published-predicate judges each unit from its header, using rows (instance i,
    T_pub, W) written at each recovery or switch. An instance older than the mounted
    root's instance with no row: published. With a row: a code 1 unit is published
    if its birth checkpoint is at most T_pub or its transaction number is at most W;
    a code 2 or code 3 unit only if its birth checkpoint is at most T_pub. The mounted
    root's own instance: published if its birth checkpoint is at most the root's
    checkpoint. A newer instance: corrupt.
H6. When a root slot write fails, the checkpoint number is advanced by one and the
    root is written again (the root ring region is checkpoint number mod R).
H7. The fixed point may iterate: the same container may be written more than once in
    one checkpoint, and every round takes the same transaction number (the largest
    one already allocated in that checkpoint), so two rounds have byte-identical write
    orders. An open disposition would forbid this: one fixed point per instance per
    checkpoint, each container written at most once. It is not adopted.
H8. The first copy-on-write of a node by a clone head gives the new version the
    writing tree's id as its birth tree.
H9. A settled rule rejects any header sequence number whose purpose is to tell stale
    copies from current ones. Scan rebuild merges code 1 and code 3 units with
    identical identity segments (write order included) into one group; for code 3 the
    order key is (birth checkpoint, instance id, transaction number). Code 2 is not in
    that rule.
H10. A new tree's id comes from a watermark = max(value in all root records, highest
    tree id issued in this checkpoint + 1). The watermark is not per instance. The two
    decision files checked do not say whether replaying an adopted transaction that
    created a tree pushes the watermark; this has not been checked across the whole
    repository.

THE VERDICT UNDER ATTACK

Candidate K (class-reuse write-order key, tightened twice):
- code 1 key = class tag 1 + birth tree 8 + birth checkpoint 8 + write order 10
- code 2 and code 3 key = class tag 1 + birth tree 8 + birth checkpoint 8 + instance id 4
  + birth sequence 4
- birth checkpoint means the unit's own birth checkpoint field; the class tag is the
  class of the named object
- the birth sequence counter scope is (tree, checkpoint number, instance id); it is
  issued in memory from 0, one counter shared by code 2 and code 3; the birth
  sequence is only a key segment and is never used to choose the newest version
- code 2 and code 3 headers get 4 more bytes (birth sequence)
- a pointer to a code 1 unit carries 26 bytes of this key (birth tree, birth checkpoint,
  write order); a pointer to a code 2 or code 3 unit carries 24 bytes (birth tree,
  birth checkpoint, instance id, birth sequence); the class tag comes from the lookup
  path, not from the pointer

Candidate L (logical identity plus write order), used only in claim R5 part 1:
- code 1 key = five-tuple 33 + birth checkpoint 8 + write order 10
- code 3 key = class tag 1 + (birth tree, record type, container number, container
  birth generation) 26 + birth checkpoint 8 + write order 10
- code 2 key = (tree id, level, lower bound of key range, birth checkpoint)
- a pointer to a code 1 unit carries the whole L key except the class tag (50 bytes);
  a pointer to a code 2 or code 3 unit carries birth tree and birth checkpoint (16 bytes);
  the level and the lower key bound of a code 2 key come from the lookup path;
  unit headers get no new bytes

Claims made by the verdict:
R1. No two different code 1 units that are both published or both alive can have the
    same K key.
R2. In the scope (tree, checkpoint number, instance id) the counter is never reset or
    moved backwards while the scope stays the same. Try: root slot write failure with
    the checkpoint number advanced by one, instance switch, crash and remount, clean
    unmount and remount, first copy-on-write by a clone head, node split and merge,
    small-object containers, a tree id issued twice (H10).
R3. After an instance switch or a crash remount, an adopted old unit and a redone or
    newly written unit never share a K key while both are judged published and differ
    in content.
R4. Every path that enters the map (dereference, the snapshot free decision, deadlist
    and livelist processing, relocation, checker, scan rebuild) has the full K key in
    hand without an extra unit read or an extra tree lookup.
R5. Part 1: if the open disposition in H7 is adopted, candidate L is also unique at
    the fixed point, including code 2 nodes, splits, merges and first copy-on-write by
    a clone head. Part 2: if it is not adopted, K gives two fixed-point versions
    different keys; at scan rebuild someone still decides which one is current,
    without using the birth sequence, and no map entry is left pointing at a dead
    version whose space can then never be freed.

TASK

For each of R1 to R5, try to construct a concrete counterexample. Give the unit class,
the exact sequence of writes, fixed-point rounds, failures, instance switches, mounts,
clones or splits, and the key bytes of each unit involved. Say which fact each step
relies on.

If you cannot break a claim, say exactly what you tried and why it failed. Do not stop
after the first attempt on any claim.
