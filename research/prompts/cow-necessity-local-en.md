You are reviewing one architectural claim for a from-scratch copy-on-write filesystem.
Your assigned stance: find counterexamples. Look for conditions under which the
challenger scheme wins, and for benefits the claim overlooks.
Do not use any markdown emphasis in your answer. Plain sentences only. Answer in English.

Verified facts about the design (quoted from its own decision records, all confirmed today):

D4: the checksum of a unit is stored inline in the pointer that points to it. The parent
node holds the child's checksum. This forms a Merkle tree. A unit is exactly 32768 bytes
on disk, header included. When encryption is on, that same inline field holds the AEAD MAC.

D20: measured on this machine, the device side atomic write width is 512 bytes and cannot
be relied on. The kernel only exposes a Power Fail variant, and "device declares 1 block"
is indistinguishable from "device declared nothing". So unit atomicity must be synthesized
by the filesystem, not assumed from hardware.

D22 dividing line: a unit that has a parent pointer carrying its checksum is published only
after the unit itself is durable. For that class, copy-on-write plus D4 is sufficient and
torn writes are harmless, because any mix of old and new bytes fails the parent's checksum.
For structures with no parent pointer, in-place overwrite is allowed but requires two things
at once: a whole-unit checksum, and a generation number that is actually checked.

D22 also records this: an in-place overwritten record that was never written at all reads
back as a complete and self-consistent old record from a previous cycle. Its checksum passes.
Only a generation number distinguishes stale from current. Checksum answers "is this unit
complete", generation answers "is this unit from the current transaction". Two questions,
two fields.

D5: snapshots use the ZFS model, birth transaction group plus deadlist. The invariant is
that block b is referenced by snapshot S if and only if birth(b) <= S.txg < death(b).
Deletion compares the block's birth against the live head's prev_snap_txg. D5 also states
that any feature introducing a second live reference to the same block requires reopening D5.

Structures that already use in-place overwrite in this design: the superblock (one copy per
disk, updated by rotating among slots), the root ring (3 regions, rotating), and the journal
ring (fixed length circular log). Everything with a parent pointer uses copy-on-write.

The challenger proposal from the project owner:

Do we need copy-on-write at all? Since our metadata carries a large amount of self
describing information, could we instead, on every write, first copy the old metadata
aside as a backup, then modify the metadata in place? The historical backup copy would
point at copies of the old data, and those copies carry generation information themselves.

The claim I want you to attack:

The proposal cannot reduce write amplification, because D4 makes the checksum live in the
parent pointer. Modifying a unit in place changes its checksum, which forces an update to
the parent pointer, which changes the parent's checksum, and so on up to the root. So the
whole spine must be rewritten either way. The proposal therefore pays the same spine cost
as copy-on-write, plus the extra cost of the undo copy, plus one extra durability barrier
per overwritten unit because the undo copy must be durable before the in-place overwrite
may begin. It is strictly more expensive, not less.

Questions for you:

1. Is there any way the in-place plus undo copy scheme avoids rewriting the Merkle spine?
   Consider whether any weakening of D4 short of abandoning it could allow this.
2. Name a concrete workload, device class, or metadata structure where in-place plus undo
   copy beats redirect-on-write under the constraints above. Be specific about the numbers.
3. What benefit of the proposal does the claim above fail to consider at all? Consider
   read locality, address stability, fragmentation, reverse index cost, and free space
   accounting.
4. Is the claim's barrier argument correct? Count the barriers each scheme needs for one
   transaction that modifies k units, and say whether the undo scheme can batch its barriers.
