Whether a journal tail pointer stored in the superblock still carries any weight in a copy-on-write file system's crash recovery.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given; F11 is an observation, not a rule):

F1. A decided rule (2026-08-29) chose the superblock form: the authoritative copy of the journal tail lives at a fixed location (the superblock), not inline in each record header.
F2. The cost table behind that choice: the superblock (jbd2-like) form costs 21 extra steady-state writes, equal to the number of checkpoints; the inline (XFS-like) form costs 0 extra steady-state writes but must replay 500 blocks after a crash during an idle period. The table assumed 20000 fsyncs with one checkpoint every 1000 fsyncs.
F3. Every publish increments checkpoint_txg by one, and a publish triggered by fsync is also a publish.
F4. After the root slot is durable, the superblock slot is updated once per checkpoint; this is the last step of the publish sequence and is part of the enumerated crash states.
F5. Recovery must not trust the tail and read forward from it. It must first scan the whole ring, validate every record, and then choose the longest valid prefix.
F6. Recovery applies only records whose (instance id, checkpoint_txg) is strictly greater than the chosen root's. A stale tail is only an optimization of where to start scanning the ring; it no longer decides the set of replayed records.
F7. The tail field is 8 bytes and stores the 48-bit jsn counter. Its redundancy comes from the superblock's own two-slot rotation; there is no separate tail slot. Record n lives at ring offset ((counter - 1) mod slot_count) * 4096. The in-flight record limit, ring slot count divided by F, is also stored in the superblock.
F8. With a fixed-size ring, a wrong tail only causes extra work, so the tail may be derived state. With a chained journal it would have to be authoritative. The design uses a fixed-size ring.
F9. A stale tail combined with legitimate block reuse made a recovery that aborts on mismatch abort itself on healthy images. The fix was to use the chosen root's jsn watermark as the replay lower bound.
F10. The jsn counter continues pool-wide: a new instance writes from prefix end + 1 and never resets. The chain continues after the last record covered by the chosen root.
F11. Observation: in the byte table, the superblock fields that change on each superblock write are: format time generation 1, instance 0; instance allocation generation 2, instance 1; two warm-up publishes generation 3 tail 1, then generation 4 tail 2; first transaction generation 5 tail 3. The slot generation increments whenever a slot is written; the instance id changes only at allocation.
F12. Both empty publishes and normal publishes end with a superblock slot write on each of the 2 disks.
F13. Invariant I-8.5 says: among all tail slot positions, the set of records whose self-checksum passes is non-empty, and besides the highest-generation one there is at least one earlier one. Note that F7 says there is no separate tail slot.
F14. Exhaustive crash replay of the first transaction found zero violations, and the dry-run output has no line about the tail.

Candidates:
A. Keep today's design: the tail lives in the superblock, and every publish writes one superblock slot on each of the 2 disks.
B. XFS-like: the tail is written inline in each journal record header; the superblock is written only at format time and at instance allocation.
C. No tail at all: the scan start and the next jsn are both derived from the whole-ring scan plus the last record covered by the chosen root; the superblock is written only at format time and at instance allocation.

Questions:
1. Under A, set the stored tail to 0, to a stale value (smaller by k), or to a value ahead by k, and also combine the stale value with block reuse. Does any crash state then recover a different chosen root, a different set of applied records, different file contents, or a different next jsn? Give the shortest such sequence, or list the shapes you tried.
2. Can B or C recover the prefix end and the next jsn without a stored tail after ring wrap-around, after an instance switch, after a rollback, and when one record in the middle fails its checksum? Does any of these give a different result from A?
3. Which decided rules does each candidate change: the superblock update step in the publish sequence, the tail field encoding in F7, the superblock field table, invariant I-8.5, the byte table, and the number of enumerated crash states?
4. For each candidate, give one check that would fail if the design were wrong, and say how to prove that the check can fail.
5. Which candidate would you pick, and what single observation would change your pick? Do not decide by which candidate saves more writes; report write counts only as a cost.
