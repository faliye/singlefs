Self-describing data unit headers and the locality_id key prefix in a copy-on-write filesystem design, round 2.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. Every data unit carries a plaintext self-describing header. Its identity fields are exactly the set that encryption binds as associated data: (unit type tag, tree ID, object ID, object birth generation, anchor offset) = 1 + 8 + 8 + 8 + 8 = 33 bytes. The design prefers writing more bytes to cutting fields for space.
F2. A proposal, open since 2026-09-03: add a sixth field, locality_id. The extent key starts with it, and on disk it has only one witness. If the inode record is lost, a scan finds the data units but cannot put them back: a unit header gives the object ID and anchor offset but not the key prefix. The field set is also the associated-data set, which is a permanent day-1 contract, so changing it is a permanent contract change.
F3. Extent key = (locality_id, inode, offset). locality_id is inherited from the parent directory at creation and deliberately not updated on rename. It is a hint, not part of correctness: it may be wrong and may be stale. The design says this allows not trusting its meaning, not dropping its bits: as the key prefix its value is exact and defining. The inode record must store a copy of locality_id.
F4. Benefit: keying by inode is 1.46 times slower than a locality_id prefix only with interleaved creates and a leaf cache of 8; when the cache holds the whole tree the effect disappears; renames decay the benefit toward 1.0.
F5. locality_id must never be part of the associated data, because that would make a field declared "may be wrong" load-bearing.
F6. In the first version locality_id is always 0. Today it has no plaintext expression on disk (the header's five fields do not include it). A rejected allocator-hint candidate that clustered data by locality was faulted, among other things, for adding the first leak of a "user object semantic category" when encryption is on; the design notes that clustering by locality moves the same signal into physical placement without adding the field.
F7. Freeze layer 3 holds the key encodings of every keyspace, including locality_id.
F8. Invariant I-9.9: the record's locality_id equals the prefix of every extent key of that object. Before any extent tree rebuild the check reports red or green; after a rebuild both sides come from the same source and it must report "undecidable", never "pass". Invariant I-9.10: the object birth generation matches in the record, in every data unit header, and in the tombstone if any.
F9. First transaction: the inode record's locality_id (offset 16) is 0; the extent leaf record key is (0, 1, 0).
F10. A project rule: "prefer writing more" governs whether a field already shown useful should be cut for space; it does not decide whether to add a field, which needs its own reason.
F11. When encryption is on, only the fsid and the payload CRC stay plaintext in a data unit's identity section; the five fields, the birth code and the write order become ciphertext in place. The user decided on 2026-09-07 that the leak is not accepted. Rebuilding with encryption on needs the key.
F12. locality_id is never updated by rename. Background re-clustering must be incremental, interruptible and resumable, and may be stopped at any time. Degradation is repaired by the resident background reorganization the design already committed to. What value to use when there is no parent directory must also be decided.
F13. Clone heads allocate inode numbers independently after the clone, so numbers overlap; uniqueness is by (tree ID, inode). Inode tree rebuild covers a single-head image only.
F14. Rebuild has three levels by witness; the level with only checksums or MACs consistent and no allocation records mounts read-only.
F15. Objects of 4 KiB or less go into packing containers; each slot carries its own five fields; the key fields in a slot are copied byte for byte from the unit header as it was before packing; packing into a container re-encrypts anyway.
F16. Background moves go through the central mapping and do not change referrers.

Round 1 found: C holds if rebuild identifies an object by (tree ID, inode) and prefers surviving extent keys over the inode record (the record can be an older version after a container rolls back, while a re-clustering has already changed the keys). A must rewrite every unit of a re-clustered object with a new nonce if re-clustering changes keys; whether re-clustering changes keys is not defined anywhere. B as written (plaintext when encryption is on) conflicts with F11; B with the field encrypted and a per-object rule stands.

Candidates:
A. Put locality_id into the five fields, which also puts it into the associated data: header identity 33 -> 41 bytes; rebuild reads the key prefix from the unit header. Changes F1 and removes the exclusion in F5; permanent contract change; changes first-transaction bytes.
B-prime. Put locality_id into the header outside the five fields and outside the associated data (+8 bytes); when encryption is on, it becomes ciphertext together with the five fields. Rebuild picks one value per object in this order: surviving extent key prefix, then inode record, then header hint, then 0. Changes first-transaction bytes.
C-prime. Do not add it. On rebuild, identify an object by (tree ID, inode) and look only at that head's extent tree; give the record and all of the object's extent keys the same prefix, in this order: surviving extent key prefix, then inode record, then 0. A wrong value loses only locality. After a rebuild I-9.9 reports "undecidable". Zero bytes, no contract change.
Optional S5 for A and B-prime: judge I-9.9 per side, so that a rebuilt extent tree whose prefixes came from unit headers can still be checked against an unrebuilt record.

Questions:
1. Rebuild correctness under shapes round 1 did not try: batched re-clustering where each batch is published (intermediate states versus I-9.9); packing container slots (F15); rebuild with encryption on (F11, F14); re-clustering defined as re-keying versus as moving physical placement only; clones and multiple heads (F13). For each candidate, give a rebuild history in which a reader cannot find some data, one object ends up under two different key prefixes, or a reader sees data that was not there before the damage; or list the shapes you tried.
2. Contract: does A conflict with F5 today, and which sentence of that exclusion carries the weight? Does B-prime conflict with F11?
3. Cost: bytes per unit and per packing slot, first-transaction bytes, and what rename, background moves, re-clustering and rebuild must do under each candidate.
4. What I-9.9 can decide under each candidate, and how you would prove that the check itself can turn red.
5. Which one do you pick, and what single observation would change your pick?
