Deadlist entry format in a copy-on-write filesystem design with snapshots and clones.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. Every block carries a birth txg: the checkpoint number in which it was published. When a writable head drops a block: if birth > the head's previous_snapshot_txg (the txg of its latest snapshot), the block is freed immediately; otherwise it is appended to the head's own deadlist. Creating a snapshot hands the head's whole deadlist to the new snapshot and gives the head an empty one. Destroying snapshot S merges S's deadlist into the next newer side and frees the entries with birth > prev(S).txg, where prev(S) is the snapshot before S. prev is inherited by the side that absorbs S; if it were not inherited, entries with small birth would never be freed. No full-disk scan.
F2. Block b is referenced by snapshot S if and only if birth(b) <= S.txg < death(b). Moving a block during background defragmentation does not change its birth.
F3. Only the destroy cascade rewrites a published snapshot's deadlist.
F4. The deadlist is one tree for the whole pool (one keyspace), registered in the tree table from day one, following the livelist and sparse side table precedent.
F5. Entries record the version identity of a block (its central mapping key), not its location and no location hint, and use the same key as the sparse side table. Birth must be available inline so the destroy filter stays O(1) per entry.
F6. Central mapping key for data units: class tag 1 byte + birth tree ID 8 + birth txg 8 + write order 10 = 27 bytes. For index nodes and packed containers: class tag 1 + birth tree 8 + birth txg 8 + instance code 4 + birth sequence 4 = 25 bytes, zero-padded to 27.
F7. Livelist precedent: one shared tree; entry key = head tree ID 8 + central mapping key 27 + event type 1 = 36 bytes; empty value; a head's entries are removed by a prefix range delete.
F8. A tree table entry has 24 reserved bytes. When the reserve was sized, two known claimants were named: a deadlist location of 8 bytes, and snapshot lineage nested-interval labels of 16 bytes (two numbers per snapshot, maintained incrementally as the lineage changes). 8 + 16 = 24 exactly. Neither was made a field.
F9. previous_snapshot_txg is one per head and lives in the tree table entry (8 bytes). A new clone's previous_snapshot_txg starts at its origin snapshot's txg. Each writable head holds one previous_snapshot_txg and one deadlist.
F10. Freeing always goes through the central mapping, never through location hints.
F11. Blocks can be shared across heads through clones, so a block's birth tree can differ from the head that holds or drops it.

Candidates for who owns a deadlist entry:
A. Owner = the tree ID of the owning head or snapshot. Key = owner tree ID 8 + mapping key 27 = 35 bytes, empty value. Snapshot creation re-keys all of the head's entries to the new snapshot's tree ID.
B. Owner = a deadlist number stored in 8 of the 24 reserved tree table bytes, one per head and per snapshot. Key = deadlist number 8 + mapping key 27 = 35 bytes. Snapshot creation: the new snapshot takes the head's number and the head gets a fresh number, which is O(1), like swapping objects in ZFS. The destroy cascade re-keys entries to the next side's number.
C. Owner = (head tree ID, the head's previous_snapshot_txg at the moment the block was dropped). Key = head tree ID 8 + epoch txg 8 + mapping key 27 = 43 bytes. Snapshot creation changes the head's previous_snapshot_txg, so later drops land in a new epoch automatically; no entry is rewritten and no tree table bytes are used. Snapshot S's deadlist is the range (head, prev(S).txg). The destroy cascade merges that range into the next side's epoch and frees entries with birth > prev(S).txg.

Bucketing options: (i) no extra bucket, because birth is already inside the mapping key; (ii) put an extra 8-byte birth before the mapping key so entries sort by birth.
Value options: empty (size and location come from the mapping), or carry a size.

Questions:
1. Correctness: for each candidate, try to build a history (snapshot creations, destroying a middle snapshot, consecutive destroys, clones whose blocks are shared across heads) in which the destroy cascade frees a block still referenced by some snapshot, or never frees one that should be freed. Pay special attention to C with clones: is (head tree ID, epoch txg) unique, and does the inherited prev still work when the absorbing side's entries are keyed by a different epoch?
2. Cost: bytes per entry, tree table bytes, how many entries snapshot creation rewrites, how many the destroy cascade rewrites. Give formulas.
3. Alignment: does each candidate keep the same key as the sparse side table (F5) and the same shape as the livelist (F7)? Does B leave room for the 16-byte lineage labels? Another open question wants 8 of the same reserved bytes for an inode counter; does that change your answer?
4. Bucketing and value: which do you choose and why?
5. Which candidate would you pick, and what single observation would change your pick?
