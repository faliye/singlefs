Rollback and inode number reuse in a copy-on-write filesystem design, round 3.

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
F18. A writable head that still has its own live snapshots may not be destroyed; a head with no snapshots may be destroyed.
F19. An empty publish writes a journal record with transaction number 0. The journal ring length is a mkfs parameter, 768 MiB by default (196608 record slots), and must not exceed a quarter of the device.
F20. Every publish increments checkpoint_txg; the accounting "generation" follows publishes. Accounting keeps only the most recent K generations; dropping one is a point delete of generation (current - K).

The problem: after rollback to R_old, head H's inode counter returns to R_old's value. The abandoned timeline had already published objects with larger numbers. New objects created after the rollback reuse those published numbers, which violates F2.

Rounds 1 and 2 found: A and B break with one fault (the newest root slot unreadable at rollback; a root slot write failure that keeps R_old alive while newer abandoned roots leave the ring). The tightening AJ (also read the counter from journal records newer than R_old) broke with one or two faults: a head destroyed in the abandoned timeline and revived by the rollback; a rollback whose publish crashed after its record and was redone, because the retry's first record overwrote the only record that carried the counter; and the journal ring wrapping around during the rollback window. C was never broken by a history that spared A. CJ (first new root txg also covers every readable record's checkpoint_txg) needed 5 slot faults where C needed 2, but in about 1158 scripts CJ broke where C did not, because pushing the first new root later moves the birth generation of the first new object, sometimes onto an abandoned object's birth generation.

Candidates:
C. Narrow the rule to: a published pair (inode number, object birth generation) is never reused. Keep the counter in accounting and keep rollback unchanged. Companion rules: the externally visible identity is (inode number, birth generation); any structure keyed by inode number either rolls back with the root or has the birth generation in its key.
C+CJ. C, and the first new root after rollback gets txg = max(txg of readable roots, checkpoint_txg of every self-verifying record in the journal ring) + 1. C+CJ2 does the same for the first publish of every new instance (mount, switch, rollback).
A/B+AJ. A or B, and on rollback also take into the maximum the counter value in the tree version that each readable journal record with txg > R_old points to, when the units it points to still exist.
G. Root records and journal record headers each carry a pool-wide inode counter (the maximum over all writable heads, never decreasing, 8 more bytes each, which changes first-transaction bytes). Rollback and every mount raise each head's counter to the maximum found in readable roots and records. Uniqueness is by number.

Questions:
1. For each of C, C+CJ, C+CJ2 and G, build a reachable history in which a published identity is issued again after a rollback, using shapes the earlier rounds did not use: CJ or CJ2 moving the first new object's birth generation onto an abandoned object's; heads destroyed in the abandoned timeline, clone heads and rollbacks combined; the two readings of where the rollback's first journal record goes; several rollbacks in a row and a rollback retried after a crash. List each publish, which root slots and journal slots are readable, and which units were reused. Count the faults. If you cannot build one, say which shapes you tried.
2. Is there today any consumer that relies on the inode number alone and does not roll back with the root? Name where it lives in the design.
3. Cost: bytes, first-transaction bytes, and what rollback and mount must read under each candidate.
4. For each candidate, one check that turns red when it is broken, and how you would prove that the check itself can turn red.
5. Which one do you pick, and what single observation would change your pick?
