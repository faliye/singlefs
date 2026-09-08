# Find a counterexample: compaction versus snapshot accounting in a from-scratch COW filesystem

Answer in English. Do not use any markdown emphasis of any kind in your answer.

This is a from-scratch copy-on-write filesystem in the format design phase. No code exists yet.
Every fact below was read out of the repository today by the main agent.

## Settled facts you must take as given

1. Snapshot accounting uses the ZFS birth-txg plus deadlist model. Each block carries a birth
   number, which is the checkpoint number in which the block was published. A block b is
   referenced by snapshot S if and only if birth(b) <= S.txg < death(b), half open on the right.
   When a block dies, the runtime decides in O(1): if birth > prev_snap_txg it is freed
   immediately, otherwise it is appended to the live head's deadlist. Creating a snapshot hands
   the live head's whole deadlist to the new snapshot and gives the head an empty one.
   Destroying a snapshot merges its deadlist into the next newer side and frees the entries
   whose birth is greater than the previous snapshot's txg.

2. Location authority was settled on 2026-09-06 as a hybrid. The location entry inside a tree
   pointer (14 bytes: device id 4, physical offset 6 in 16 KiB slot numbers, ciphertext checksum 4)
   is demoted to a hint: a reader may go to that location and try, but it can be stale. A central
   map is the single entry point for dereference and for free decisions. Compaction changes only
   one entry in the central map; it never needs to know who references the unit. Three hard rules
   ship with that decision: freeing always goes through the map and never through the hint; the
   unit header or the hint must carry the block birth; the stale-hint extra-hop rate needs a
   runtime observation point.

3. The logical identity of a data unit is a five-field tuple, settled at 33 bytes total:
   unit class tag 1, tree id 8, object id 8, object birth generation 8, anchor offset 8.
   The same five fields are also the AAD field set for encryption, and changing that set is a
   permanent day-one contract change. Note carefully: this tuple identifies a logical location,
   not a version. Overwriting the same file offset produces a new unit with the same five fields.

4. The on-disk header of a data unit is 105 bytes and does contain the birth generation as a
   separate 8-byte field, alongside the 33-byte tuple.

5. The allocation record is a separate btree whose key is (device identity, offset within device)
   and whose value is the allocation generation. So physically adjacent locations are adjacent in
   that key space too.

6. An experiment measured the hybrid architecture over 200 rounds with snapshots every 8 rounds
   and 4 snapshots retained. In that model the central map is keyed by version, where a version is
   the pair (key, birth), and a deadlist entry stores a version plus a hint. With a unit header
   that only holds the five-tuple, snapshot reads silently returned the newer content of the same
   file offset 40 to 128 times per 200 rounds under a uniform load and about 4000 to 5300 times
   under an adversarial load. With a unit header that holds the tuple plus block birth, the count
   was exactly 0 in all 20 runs, because a mismatch falls back to the map.

7. Another experiment measured that compacting a batch of 4096 locations rewrites 13 tree nodes
   when the locations are contiguous in the allocation-record key space, and 4143 to 8208 nodes
   when they are scattered. That is a 370x difference in the amount of copy-on-write amplification.

8. A separate settled clause says that blocks produced by the commit itself (new copy-on-write
   nodes of the allocation record tree and the accounting tree) must be allocated from a
   key-space contiguous clustered segment, and states verbatim that the locations of user data
   blocks are not subject to that constraint.

## The draft clauses under attack

Clause 1. Compaction does not change birth. The birth travels with the unit to its new location.

Clause 2. A deadlist entry is keyed by version identity, not by location. One entry is the
33-byte five-tuple plus the 8-byte birth generation, 41 bytes, optionally plus a 14-byte
non-authoritative location hint. Therefore compaction touches the deadlist zero times. When a
snapshot is destroyed, the birth filter reads only the entry itself, and resolving the actual
location for the free goes through the central map.

Clause 3. The key of the central map must include the birth generation, so 41 bytes rather than
the 33 bytes currently written in the repository. Reason: deadlists and snapshot trees reference
old versions, and the map is the single entry point for dereference and free. If the key were
only the five-tuple, the map could only point at the current version, so an old version could
neither be dereferenced nor freed.

Clause 4. Compacting N locations rewrites ceil(N / fanout) leaves plus one interior node per
level, and does not scale with device capacity. That holds only if the locations are contiguous
in the allocation-record key space. The source side is contiguous because compaction works on one
region at a time. The destination side has nothing guaranteeing it today, so clause 4 adds a
policy sentence: the compaction destination must be one contiguous run.

## Your task

Your assigned stance is: construct counterexamples. Do not summarise, do not agree, do not give
a balanced assessment. For each clause, try to build a concrete sequence of operations
(writes, snapshot creation, compaction batches, snapshot destruction, crashes) under which the
clause produces a wrong result, an unreachable state, a leak, a double free, or a quantity that
cannot be computed at the moment it is needed.

Give each construction as a numbered timeline of concrete steps with concrete numbers, and say
exactly which clause breaks and what the observable wrong outcome is. If you cannot construct
one for a clause, say so in one line and move to the next; do not pad.

Pay particular attention to these four places, but do not limit yourself to them:

a. Clause 2 says the deadlist entry has no location. Consider what happens when the same
   five-tuple has several dead versions alive in different deadlists at the same time.

b. Clause 3 changes the map key. Consider the lifetime of a map entry: who creates it, who
   deletes it, and how many entries exist for one file offset that has been overwritten k times
   while k snapshots are retained.

c. Clause 1 says birth travels with the unit. Consider a crash in the middle of a compaction
   batch, and consider what the allocation record and the map say about the old and the new
   location at every truncation point.

d. Clause 4 asks the allocator for a contiguous destination run. Consider what happens when no
   contiguous run of the needed length exists, and what the compaction is then allowed to do.
