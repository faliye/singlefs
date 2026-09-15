Rollback and inode number reuse in a copy-on-write filesystem design.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. Each writable head (a writable branch of the filesystem) has an inode next-free counter: the next free inode number. It is stored as a row in the accounting tree, a copy-on-write B-tree that is versioned with every published root. Monotonic statistics merge by taking the maximum.
F2. Rule: published inode numbers are never reused. A crash that loses an unpublished checkpoint may re-issue numbers that were never published; that is allowed.
F3. Admin rollback: the administrator picks an older root R_old from the root ring (a fixed ring of recent root slots, 24 slots in the first version). Rollback applies no journal records after R_old, takes a new instance code, and reloads the defer queue, the allocator cursor and all accounting statistics from R_old's accounting tree. The first new root after rollback gets checkpoint_txg = (maximum txg over all root records in the ring) + 1. Blocks referenced by abandoned-timeline roots may not be reallocated or wiped until those roots leave the ring.
F4. The tree-ID counter, a pool-wide monotonic counter, lives in the root record. Every publish writes new value = max(that field over all root records in the ring, highest tree ID issued in this checkpoint + 1). It survives rollback because the ring still holds the abandoned roots.
F5. A tree table entry is 148 bytes and has 24 reserved bytes. There is one entry per tree; each writable head has its own inode tree entry.
F6. Each inode record carries an object birth generation = the checkpoint_txg of the publish that created the object. Encryption is not in the first version but the format reserves it: the AEAD associated data includes the object birth generation, and nonces are stored per extent, not derived. The design text calls birth generation defense in depth: if a bug reuses an inode number, birth generation still separates the old and new object at the associated-data and nonce layers.
F7. Extent key = (locality_id, inode, offset). It does not include birth generation.
F8. Deleting an object writes a range record: object ID + object birth generation + full range.
F9. Accounting keeps only the most recent K generations; K is tied to how many generations a root can roll back.
F10. When a root slot is unreadable, recovery falls back to the newest readable self-verifying root.

The problem: after rollback to R_old, head H's inode counter returns to R_old's value. The abandoned timeline had already published objects with larger numbers. New objects created after the rollback reuse those published numbers, which violates F2.

Candidates:
A. Put the inode counter into the tree table entry (8 of the 24 reserved bytes), one per writable head. On rollback take the maximum of that field over the tree tables of all readable roots in the ring, the same shape as F4.
B. Change rollback: reload non-monotonic statistics from R_old, but for monotonic statistics (including the inode counter) take the maximum of R_old's value and the value in every readable root's accounting in the ring.
C. Narrow the rule to: a published pair (inode number, object birth generation) is never reused. Keep the counter in accounting and keep rollback unchanged. Reasoning: after rollback the first new root's txg is ring max + 1, larger than any abandoned txg, so every new object's birth generation is strictly larger than any abandoned object's.

Questions:
1. For each candidate, construct a reachable history (list each publish, which root slots are readable, when rollback and crashes happen) in which a published identity is issued again after rollback. For A and B the identity is the number; for C it is the pair. Consider at least: the newest root being temporarily unreadable at rollback time; two rollbacks in a row; a crash right after rollback; creating a clone head after rollback. If you cannot construct one, say so and say what would be needed.
2. Candidate C: list every consumer of the old rule that might rely on the inode number alone, for example the extent key in F7, background reclaim of deleted inodes whose extents are still being freed, externally visible inode numbers such as NFS-style file handles, and the rule that inserts always go to the rightmost leaf of the inode tree. For each, say whether it breaks under C and give the concrete sequence of operations if it does.
3. Candidate B: accounting keeps only K generations (F9). Can a root that is still in the ring have had its counter row deleted, so that B cannot read it? What happens then?
4. Candidate A: does moving the counter into the tree table change anything else, for example journal records that swap the tree table pointer during replay? Is the accounting row still needed?
5. Which candidate would you pick, and what single observation would change your pick?
