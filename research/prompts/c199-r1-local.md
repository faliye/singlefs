You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italics. Number your answers 1 to 6 and keep each under 200 words. Your stance: find concrete sequences that break candidate B, the writer's preference; also try to show that candidate A loses fsync-acknowledged data. If you cannot construct a sequence, say so plainly.

Settled facts (do not dispute them; use them):

F1. Every journal record carries jsn = instance number (32 bits) followed by a counter (48 bits). The instance number increases at every recovery and at every in-mount instance switch.

F2. Replay applies only records whose (instance number, checkpoint txg) is strictly greater than the chosen root's watermark. The applied prefix must satisfy all five rules: jsn strictly consecutive, stop at the first gap; the watermark rule above; only transactions whose commit marker is complete are applied; every unit named by a record is verified before applying; if the chosen root's instance has a rollback row in the instance table, that instance's records are applied only up to the row's W.

F3. An experiment showed: putting the instance number into the jsn high bits blocks stale records left in the ring by an earlier timeline, but once the instance number increases, the new instance's records are not consecutive with the old prefix, so under "jsn == expected, stop at the first gap" all newly written records are lost. The journal decision text says the prefix rule "must accept instance number + 1 with the counter continuing", but that change was never written into any settled rule. The same text says correctness depends on the instance number never rolling back.

F4. The instance table holds rows (instance i, T_pub, W). Rows are written only at recovery, rollback and instance switch, in the same publication as the first new root. A recovery writes one row for every instance from the chosen root's instance up to, not including, the new instance. W is the largest transaction number of instance i applied by this replay. A settled text argues: because replay is strictly consecutive and stops at the first gap, no replay ever crosses an instance boundary, so among the rows written by one recovery only the chosen root's instance can have W > 0; all others have W = 0. A data unit counts as published iff its birth txg <= T_pub or its transaction number <= W.

F5. Recovery chooses the newest readable, self-verified root; it may not be the newest root ever published. If a root slot is bad, recovery steps back one root; records after it that do not connect are not applied and W = 0.

F6. Publication order: units and nodes, barrier, journal records, barrier, root slot written with FUA. Every publication increments the checkpoint txg, including publications forced by fsync. An fsync returns after its publication's root is durable.

F7. An instance switch is a recovery inside a running mount: take a new instance number, write rows, re-issue the in-flight checkpoint; transactions <= W keep their effect, transactions > W are redone under the new instance. At most 3 consecutive switches per mount.

F8. The journal tail lives in the superblock and is written only after a checkpoint is durable. Recovery never trusts the tail; it scans the whole ring and verifies every record. Replay must be idempotent.

F9. The journal record format is a separately frozen component; a new record type needs a new record type code plus an incompat flag.

Candidates:
A = keep the strict rule; replay never crosses an instance boundary.
B = within one instance the counter increments by 1; at the end of an instance (up to its row's W if it has a row, otherwise at the first gap) replay may continue with a record of a larger instance number whose counter equals the last applied record's counter + 1; an instance whose row has W = 0 contributes no records after its T_pub.
C = the new instance's first record is a handover record naming (previous instance number, last applied counter); replay may cross an instance boundary only through a handover record that matches the timeline.

Answer these six items:
1. Under candidate A, construct a sequence (instance numbers, counters, checkpoint txgs, which roots are readable) in which an fsync-acknowledged transaction is lost although a readable root plus intact journal records would suffice to recover it. State how many independent faults must coincide. If impossible, say so.
2. Attack candidate B: construct a sequence where B applies a record that does not belong to the current timeline, for example records of an abandoned instance whose first root never became durable, records discarded by an administrator rollback, or records of an instance whose row says W = 0.
3. Attack candidate C the same way.
4. For each candidate, which settled sentence above (F2, F4, F5) must change.
5. Does any candidate change the bytes written by the first transaction (one mkfs plus one publication)?
6. Your recommendation among A, B, C, with one sentence of reason for each.
