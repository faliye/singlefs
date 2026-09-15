Rollback and inode number reuse in a copy-on-write filesystem design, round 2.

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
F11. Warm-up: a new instance (after a mount, a switch or a rollback) does not let any fsync return and does not confirm a rollback to the administrator until the roots written by this instance cover both devices. The method is to publish empty roots in a row: at most R = 3 in the geometry, exactly 2 in the first version, which is a format constant because the three root ring regions sit on devices 0, 1, 0.
F12. Within region (txg mod R), the root slot is (txg div R) mod S. When the design talks about the oldest valid root in the ring, a slot whose write failed counts as still holding its old content, and an in-flight publish that never became durable does not count.
F13. Publish order: units, barrier, journal record, barrier, root slot written with FUA. The journal header fields that must stay readable forever include jsn and checkpoint_txg. Every record header carries a new-root segment: tree table pointer, central mapping root pointer, tree-ID counter, rollback lower bound F. Recovery scans the whole journal ring and verifies every record before choosing the longest valid prefix. A new instance continues writing the journal from the end of the prefix plus 1, never resetting, so records of an abandoned timeline get overwritten over time. The journal ring is mirrored on both devices.
F14. The first version has no mount: it is a user-space library over a block device abstraction.
F15. The tree-ID counter is justified by: roots that a rollback can jump over are still in the ring.
F16. Known open problem: if all roots of an older instance are temporarily unreadable, a new instance's first publish counts txg up from an even older root.
F17. Where the reclaim state of deleted inodes lives is not defined yet.

The problem: after rollback to R_old, head H's inode counter returns to R_old's value. The abandoned timeline had already published objects with larger numbers. New objects created after the rollback reuse those published numbers, which violates F2.

Round 1 found: A and B each break with one fault: the newest root slot unreadable at rollback time; a root slot write failure that keeps R_old alive in the ring while the newer abandoned roots that carried a later destroyed head leave the ring; and a root slot write failure during the rollback publish, if the resulting instance switch reloads the counter from R_old. C needed two faults under the strict warm-up of F11, and every history that broke C also broke A. Whether externally visible inode numbers break C is disputed, because the first version has no mount (F14).

Candidates:
A. Put the inode counter into the tree table entry (8 of the 24 reserved bytes), one per writable head. On rollback take the maximum of that field over the tree tables of all readable roots in the ring, the same shape as F4. The 24 reserved bytes already have other claimants.
B. Change rollback: reload non-monotonic statistics from R_old, but for monotonic statistics (including the inode counter) take the maximum of R_old's value and the value in every readable root's accounting in the ring.
C. Narrow the rule to: a published pair (inode number, object birth generation) is never reused. Keep the counter in accounting and keep rollback unchanged.
AJ (tightening for A and B). On rollback also take into the maximum the counter value in the tree version that each self-verifying journal record with txg > R_old points to through its new-root segment. AJ2 does the same at every mount.
CJ (tightening for C). The first new root after rollback gets txg = max(txg of readable roots, checkpoint_txg of every self-verifying record in the journal ring) + 1. CJ2 does the same for the first publish of every new instance (mount, switch, rollback).

Questions:
1. For each of A, B, C, AJ and CJ, construct a reachable history in which a published identity is issued again after rollback, using shapes round 1 did not use: journal records overwritten, or unreadable on both devices, at rollback time; the journal ring wrapping during the rollback window; mounts and switches after a rollback; any path in the first version that lets a user transaction into the warm-up publishes; more devices or a different region assignment. List each publish, which root slots and journal slots are readable, and when rollbacks and crashes happen. If you cannot construct one, say which shapes you tried.
2. Is there today any consumer that relies on the inode number alone? For each one, name where it lives in the design. Consider externally visible numbers (F14), structures keyed by inode number that do not roll back (F17), and the extent key (F7).
3. Cost: what AJ and CJ must read at rollback and at mount. Give formulas.
4. For each candidate and each tightening, one check that turns red when it is broken, and how you would prove that the check itself can turn red.
5. Which one do you pick, and what single observation would change your pick?
