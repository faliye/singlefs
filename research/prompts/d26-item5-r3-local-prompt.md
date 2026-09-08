# Round three: break the third draft of the compaction and snapshot accounting clauses

Answer in English. Do not use any markdown emphasis of any kind in your answer.

This is a from-scratch copy-on-write filesystem in the format design phase. No code exists yet.
Two earlier rounds already attacked earlier drafts. Attack the third draft below, not the earlier
ones.

## Settled facts you must take as given

1. Snapshot accounting uses the ZFS model. A unit carries a birth number, the checkpoint in which
   it was published. Unit b is referenced by snapshot S if and only if birth(b) <= S.txg <
   death(b). When a unit dies, if its birth exceeds the live head's prev_snap_txg it is freed into
   a defer queue, otherwise it is appended to the live head deadlist. Taking a snapshot hands the
   head deadlist to the new snapshot. Destroying a snapshot merges its deadlist into the next
   newer side and frees the entries whose birth exceeds the destroyed snapshot's own previous txg;
   that previous txg is inherited by the next side, so it is transitive across cascaded destroys.

2. Location authority is a hybrid, settled. The 14-byte location entry inside a tree pointer is
   only a hint and can be stale. A central map is the single entry point for dereference and for
   free decisions. Freeing always goes through the map, never through the hint. Compaction changes
   only the map entry.

3. A data unit header is 105 bytes. It carries a 33-byte logical identity tuple (unit class tag 1,
   tree id 8, object id 8, object birth generation 8, anchor offset 8), an 8-byte birth generation,
   a 10-byte write sequence, and a 4-byte payload checksum. The tuple names a logical location,
   not a version: overwriting the same file offset yields a new unit with the same tuple.

4. An invariant orders versions after grouping: readable published units are grouped by the whole
   class identity segment including the write sequence, and the groups for one tuple are pairwise
   ordered by the write sequence. The stated scope of that grouping rule is two members per group,
   because a unit is written in two mirror copies.

5. One planned verification tool is a generational incremental scrub: it keeps a watermark and
   only walks units whose birth generation exceeds it, on the reasoning that an untouched subtree
   still has an intact checksum chain.

6. The allocation record is a btree keyed by (device identity, offset within device); the value
   holds an allocation generation, a freed bit and a release generation.

7. Background work (destroying a snapshot, compaction, scrub) is exempt from the accounting
   constraints that bind the runtime decision paths; it may traverse and it may be slow.

8. There is no upper bound on the number of snapshots anywhere in the design.

9. Cross-head sharing (clones, reflink) does not exist yet; any feature introducing a second live
   reference to one unit requires reopening the snapshot accounting decision first.

## The third draft under attack

Clause 1. Compaction does not change the birth number, which travels with the unit. Compaction
does assign a fresh write sequence to the relocated copy. Two independent reasons: with the write
sequence preserved, the whole class identity segment of the old and the new copy would be byte
identical, so one tuple would group four members while the stated scope is two; and the
generational scrub would otherwise never walk a physically fresh copy, since its watermark has
long passed that old birth number.

Clause 2. A deadlist entry is the 33-byte tuple plus the 8-byte birth, 41 bytes, and carries no
location hint, because freeing always goes through the map so a hint buys nothing and offers an
implementation a path to a double free. Compaction touches the deadlist zero times. Destroying one
snapshot costs one map point lookup per released entry, about 480 per destruction in the measured
model, which the background exemption allows.

Clause 2a. The three writes of one relocation (the new unit bytes, the map entry, and the
allocation record entries marking the source released and the destination allocated) must belong
to one transaction, named by one journal record, verified and applied together.

Clause 3. The central map must resolve a location from the pair (tuple, birth). The encoding is
deliberately not decided here: either the birth goes into the btree key so the key becomes 41
bytes and entries stay fixed width, or the key stays 33 bytes and the value holds a chain of
versions whose length is the number of live snapshots plus one and therefore has no upper bound.

Clause 4. Compacting N locations rewrites, on the source side, ceil(N / fanout) leaves plus one
interior node per level. On the destination side, when the destinations are scattered, the count
is min(N, total leaves) instead. The draft records the cost and its precondition and refuses to
legislate a destination allocation policy.

## Your task

Your stance is: construct counterexamples against the third draft. Do not summarise, do not agree,
do not give a balanced assessment. For each clause build a concrete numbered timeline with
concrete numbers under which the clause yields a wrong result, an unreachable state, a leak, a
double free, an unbounded quantity, or a value that cannot be computed when it is needed. Say
exactly which clause breaks and what the observable wrong outcome is. If you cannot build one for
a clause, say so in one line and move on; do not pad.

Five places deserve particular attention, but do not limit yourself to them:

a. Clause 1 assigns a fresh write sequence. The old copy is not reclaimable immediately because
   of the defer window, so for a while two readable published copies of the same version exist
   with different write sequences. Show what a scanner rebuilding state from unit headers alone
   concludes about those two, and whether it can be wrong.

b. Clause 1 relies on the scrub watermark walking the relocated copy. Show a schedule of writes,
   relocations and scrub passes under which some physical copy is still never walked.

c. Clause 2 removes the hint. Show a reader or a repair path that had no other way to reach the
   location and is now stuck, or show that removing it costs nothing.

d. Clause 2a demands one transaction. Estimate how large that transaction becomes for a batch and
   show whether a single relocation can even be split, then show what breaks if a batch must be
   one transaction.

e. Clause 3 leaves the encoding open while clause 2 already fixes the deadlist entry at 41 bytes.
   Show whether those two are consistent under the second encoding.
