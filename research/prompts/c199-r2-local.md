You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italics. Number your answers 1 to 6 and keep each under 220 words. Your stance: find concrete sequences that break candidate B-prime (the writer's preference) and candidate A-prime. If you cannot construct a sequence, say so plainly and list what you tried.

Settled facts (use them; do not dispute them):

F1. Every journal record carries jsn = instance number (32 bits) followed by a counter (48 bits). A new instance number is taken at every writable mount (a clean remount too; a read-only mount takes none), at every crash recovery, at every in-mount instance switch and at every administrator rollback. The new number is max(numbers seen in superblocks and in all root records) + 1, so numbers can skip.

F2. Publication order: data units and tree nodes, barrier, journal record, barrier, root slot written with FUA. No operation may publish a root without writing a journal record first. An fsync returns only after its publication's root slot is durable. Every publication increments checkpoint txg by one.

F3. Root ring in the first version: 3 regions with 8 slots each on 2 disks; regions 0 and 2 are on disk 0, region 1 is on disk 1. Publication number n writes one slot in region n mod 3. Root slots are not mirrored. The journal ring is mirrored on both disks. A settled text says losing one disk loses nothing from the replay window.

F4. Recovery chooses the newest readable, self-verified root (txg first, instance number breaks ties). It replays only records whose (instance number, checkpoint txg) is greater than the chosen root's; how the pair is compared (instance first or txg first) is not settled. Only transactions with a complete commit marker are applied; every unit a record names is verified before applying.

F5. The instance table holds rows (instance i, T_pub, W). Rows are written only by the recovery after an unclean end, by a rollback and by a switch, in the same publication as that operation's first new root; one row for each instance from the chosen root's instance up to, not including, the new one. W is the largest transaction number of instance i applied by that replay. A data unit of instance i counts as published iff it has no row and i is older than the mounted instance, or its birth txg <= T_pub, or (for plain data units) its transaction number <= W. Consequence: the table version a root points to never contains a row for that root's own instance.

F6. An administrator rollback picks an older root R_old, applies no later record, takes a new instance number and writes a rollback row that later recoveries may not overwrite. Today's text says the rollback has no durable effect until its first new root is durable. Roots of the abandoned timeline stay in the ring; their exclusive units may be scrubbed.

F7. A switch re-issues the in-flight checkpoint under a new instance; transactions <= W keep their effect, transactions > W are redone under the new instance or fail back to callers that have not returned. At most 3 switches per mount.

F8. Each record carries previous_hash, a 32-bit CRC over the whole previous record header including its jsn.

Candidates:
A = strict: the chain starts right after the last record the chosen root covers; jsn must be exactly the previous jsn + 1; stop at an instance boundary.
A-prime = A plus a warm-up rule: a new instance lets no fsync return and confirms no rollback until its own roots are durable on two different disks; it issues empty publications until its regions cover both disks (at most 3).
B-prime = the chain starts right after the last record the chosen root covers (for the mkfs root, at the first record ever written); within an instance counter + 1; across instances the next record must have a larger instance number, counter = previous record's counter + 1, and a matching previous_hash; a rollback takes effect once its journal record with commit marker is durable; after crossing instances, recovery writes a row (j, chosen root txg, W_j) for every crossed instance j, where W_j counts only transactions still in effect, and such rows may not be overwritten later; fixed-point units published after the chosen root and still referenced are all rewritten.

Criteria: 1 correctness, fault count irrelevant: applying a record outside the current timeline, applying a publication on the wrong base, reading a scrubbed or reused unit, or the published predicate reviving a lost write, puts the candidate out. 2 durability: losing or undoing an acknowledged publication (fsync returned, or rollback confirmed) while a readable root and intact records could keep it puts the candidate out if at most one fault is involved (one unreadable root slot or one lost disk; crashes are free).

Answer these six items:
1. Attack B-prime under criterion 1: construct a sequence (instance numbers, counters, txgs, which roots are readable, which records exist) where B-prime applies a wrong record or the rows it writes let the published predicate revive a lost write.
2. Attack B-prime around rollback: abandoned timeline roots, rollback rows, an intermediate instance crossed during replay.
3. Attack A-prime under criterion 2 with one fault: find a single-fault sequence that still loses an acknowledged publication, including a crash during warm-up or a failed warm-up write.
4. Should the 48-bit counter restart at every instance or continue across instances? Give one concrete reason for each choice.
5. Should the watermark compare instance number first or txg first? Construct the sequence where the two orders differ.
6. Your recommendation among A, A-prime, B-prime, one sentence of reason each.
