# Round two: break the revised clauses on compaction versus snapshot accounting

Answer in English. Do not use any markdown emphasis of any kind in your answer.

This is a from-scratch copy-on-write filesystem in the format design phase. No code exists yet.
Round one already attacked an earlier draft of these clauses; the drafts below are the revised
ones. Attack the revised text, not the earlier text.

## Settled facts you must take as given

1. Snapshot accounting uses the ZFS birth-txg plus deadlist model. Each unit carries a birth
   number, the checkpoint number in which it was published. A unit b is referenced by snapshot S
   if and only if birth(b) <= S.txg < death(b), half open on the right. When a unit dies the
   runtime decides in constant time: if birth is greater than prev_snap_txg it is freed
   immediately into a defer queue, otherwise it is appended to the live head deadlist. Creating a
   snapshot hands the live head deadlist to the new snapshot and gives the head an empty one.
   Destroying a snapshot merges its deadlist into the next newer side, and frees the entries whose
   birth exceeds the destroyed snapshot's own previous txg; that previous txg is inherited by the
   next side, so it is transitive across cascaded destroys.

2. Location authority is a hybrid, settled. The location entry inside a tree pointer, 14 bytes, is
   only a hint and can be stale. A central map is the single entry point for dereference and for
   free decisions. Freeing always goes through the map and never through the hint. Compaction
   changes only the map entry.

3. The logical identity of a data unit is a five-field tuple, 33 bytes: unit class tag 1, tree id
   8, object id 8, object birth generation 8, anchor offset 8. It identifies a logical location,
   not a version: overwriting the same file offset yields a new unit with the same five fields.
   The on-disk unit header is 105 bytes and carries the birth generation as a separate 8-byte
   field next to that tuple.

4. The allocation record is a btree keyed by (device identity, offset within device), value is the
   allocation generation plus a freed bit plus a release generation.

5. The number of snapshots has no upper bound anywhere in the design.

6. Background work (destroying a snapshot, compaction, scrub) is explicitly exempt from the
   accounting rules that bind the runtime decision paths; it may traverse and it may be slow.

7. A measurement on a counting model (4096 logical keys, 200 rounds, a snapshot every 8 rounds,
   4 snapshots retained) found: 5517 to 5554 live map entries against only 4096 distinct keys, at
   most 4 versions alive under one key at once, and 474 to 488 deadlist entries released per
   snapshot destruction.

## The revised clauses under attack

Clause 1. Compaction does not change the birth number; it travels with the unit to its new location.
Scope note added: birth is a property of the logical version, so any consumer that reads birth as
a measure of how recent the physical copy is will never see a relocated copy.

Clause 2. A deadlist entry is keyed by version identity, not by location: the 33-byte tuple plus
the 8-byte birth, 41 bytes. Whether it also carries a 14-byte non-authoritative location hint is
left open. Compaction therefore touches the deadlist zero times. The birth filter at destroy time
reads only the entry itself. Resolving the location for the actual free costs one central map
point lookup per entry, roughly 480 lookups per snapshot destruction, which the background exempt
allows.

Clause 3. The central map must be able to resolve a location from the pair (tuple, birth), that
is, by version identity. Two encodings both achieve that and the choice is deliberately left open:
encoding A puts the birth into the btree key, so the key becomes 41 bytes and entries are fixed
width and publishing is a pure insert; encoding B keeps a 33-byte key and stores a chain of
versions in the value, so the entry width has no upper bound because the number of snapshots has
no upper bound, and publishing becomes a read-modify-write.

Clause 4. Compacting N locations rewrites, on each of the two sides, ceil(N / fanout) leaves plus
one interior node per level. The source side is contiguous in the allocation-record key space. The
destination side has nothing guaranteeing contiguity, and the only allocation policy ever modelled
(lowest free slot first) guarantees the destination is scattered exactly in the aged state where
compaction runs. The clause deliberately does not legislate a destination policy; it records the
cost formula and its precondition, and hands the policy question to a separate open item.

## Your task

Your assigned stance is: construct counterexamples against the revised text. Do not summarise, do
not agree, do not give a balanced assessment.

For each clause, build a concrete numbered timeline with concrete numbers under which the clause
yields a wrong result, an unreachable state, a leak, a double free, an unbounded quantity, or a
value that cannot be computed at the moment it is needed. State exactly which clause breaks and
what the observable wrong outcome is. If you cannot build one for a clause, say so in one line and
move on; do not pad.

Four places deserve particular attention, but do not limit yourself to them:

a. Clause 2 leaves open whether the entry carries a hint. Consider both variants and show that one
   of them is strictly worse, or show that the choice cannot be deferred because something else
   already depends on it.

b. Clause 3 leaves the encoding open. Consider whether anything downstream of this design can be
   specified at all while both encodings remain live, and consider whether encoding A really is
   fixed width once you count how many map entries exist for a key overwritten k times with k
   snapshots retained.

c. Clause 1's scope note claims only one known consumer reads birth as a physical age. Try to find
   a second consumer inside the facts given above, in particular anything that decides whether a
   location may be reused or whether an orphan copy may be reclaimed.

d. Clause 4 refuses to legislate a destination policy. Show what breaks downstream if the
   destination policy stays undefined, and whether the cost formula in the clause is even
   well defined without it.
