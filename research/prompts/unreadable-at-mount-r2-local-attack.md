Mount-by-mount trajectory grid after one fault, in a copy-on-write file system (attacking leg)

You are the attacking side in a review of a file system design. Your job is to fill one trajectory grid by reasoning only from the fact table below. You do not run code. Do not assume facts that are not in the table. Where a candidate's definition leaves a cell open, pick the outcome under an assumption and state that assumption in the cell.

Rules for your answer:
- Write in English only.
- Do not use markdown emphasis (no asterisks and no underscores around words). Plain text lines only.
- Do not cite source code line numbers or file line numbers. Refer to facts only by their fact id (F01, F02, ...) and to code only by function name.
- Answer every cell. Do not answer with only yes or no.
- Number your answers exactly as asked in Q1 and Q2.
- For every cell, write one observation that would overturn your answer.

Notation. A version or root is written (instance generation, checkpoint_txg). A system configuration slot is written (device, slot generation, journal tail). jsn is the journal record counter. An acknowledged write is one whose fsync returned. "Unreadable" means the read returns a block device error. A slot "self-verifies" when it can be read, its whole-slot checksum passes and its filesystem id is this pool's.

The mount sequence asked about. After the history of a row, the pool is mounted again and again: mount 1, mount 2, mount 3, and possibly more later. Each mount first tries a writable mount; if the writable mount is refused, a read-only mount is done instead. Each mount ends before the next one starts. The fault named by the row stays in place for all mounts; no other fault is added and nothing lifts the fault. Nothing other than these mounts writes to the disks. The question does not fix whether the user writes files during a writable mount.

Labels per mount: W = the writable mount succeeds; R = the writable mount is refused and the mount is read-only.

FACT TABLE

Part A. The histories (rows)

F01 (torn rows, common history). Instance 1: warm-ups txg 1 and 2, A at txg 3. Reopen as instance 2, with no fault injected: row txg 4, warm-up txg 5, B at txg 6, C at txg 7; all acknowledged. Then instance 2 publishes D = (2, 8): D's root is written with FUA and D's journal records are on disk; then comes the rotation write of the system configuration, one write per device, to slot number (the largest self-verified slot generation on that device before D, plus 1) mod 2; that rotation write is torn or not persisted as each row says; then the process crashes. D was not acknowledged. The system configuration slot write is an in-place overwrite longer than one sector. In the torn rows no read fault is injected anywhere.

F02 (torn slot). A torn slot holds bytes of which the first half of the range that differs between the old and the new content is new and the second half is old, so neither the old nor the new content can be read from it. Reading it returns bytes without a device error; its whole-slot checksum fails, so it does not self-verify.

F03 (row torn_both). The rotation write is torn on both devices. Self-verified slots after the crash: (0, 10, 7), (1, 10, 7).

F04 (row torn_d0). The rotation write is torn on device 0 and not persisted on device 1 (device 1 keeps the old bytes of that slot). Self-verified slots after the crash: (0, 10, 7), (1, 10, 7), (1, 9, 6).

F05 (row torn_d0_d1_persisted). The rotation write is torn on device 0 and persisted on device 1. Self-verified slots after the crash: (0, 10, 7), (1, 10, 7), (1, 11, 8).

F06 (tree table row and allocation record tree row, common history). A and B (instance 1); reopen, acquire instance 2; C = (2, 8); crash recovery abandons C: during that recovery mount, C's root slot and data units were temporarily unreadable and, on each device, the system configuration slot with the largest slot generation was zeroed; recovery lands on (2, 7); instance 3 writes its row at txg 9 and a warm-up at txg 10. C then becomes readable again. C is an abandoned root still in the root ring.

F07 (the zeroed slots of F06). The recovery mount's acquisition write lands back on each zeroed slot (slot generation = the largest readable self-verified generation + 1, slot = generation mod 2).

F08 (row tree table). From mount 1 on, both copies of C's tree table unit are unreadable on every read, in every mount. Nothing else has a read fault.

F09 (row allocation record tree). From mount 1 on, both copies of the root node of C's allocation record tree are unreadable on every read, in every mount. C's tree table stays readable. Nothing else has a read fault.

Part B. Design rules

F10 (rule, publish order). A publish persists in this order: copy-on-write units and nodes, barrier, journal record, barrier, root slot (FUA), system configuration slot, barrier. fsync returns only after the system configuration rotation is durable. The system configuration slot is updated after the root slot is durable, once per checkpoint.

F11 (rule, system configuration slots). Each device has exactly 2 system configuration slots. The slot generation starts at 1 and goes up by 1 per write; the next write goes to slot (generation mod 2). The slot chosen is the one whose checksum passes and whose generation is largest. Counted per device, all three kinds of system configuration write (acquisition, the rotation at the end of a publish, and the rollback write after an all-or-nothing acquisition failure) write the largest self-verified generation on this device + 1.

F12 (rule, torn slot). The content is allowed to cross 512 bytes (the probed physical block size). Once it crosses, one write may land half-done; the whole-slot checksum (covering the whole 4096-byte slot including padding) detects it, and the slot choice rule (checksum passes and generation largest) falls back to the other slot by itself. Detectable and recoverable. Accepted cost: the mount after a tear gets the previous generation of the configuration. Warning in the same rule: today the content is 489 bytes and does not cross 512; the rule says crossing is allowed, not that it has happened.

F13 (rule, a writable mount finds a newer state it cannot read). After the read stage (choose the root, scan the journal, replay), c_witness = the maximum journal tail over all self-verified system configuration slots among the two slots of every device in the pool. If c_witness is 0: not judged. If the selected version's record that carries the last-record flag is readable (either copy): true when c_witness is greater than that record's counter. If that record is unreadable but the record whose counter equals c_witness is readable: true when that record's (instance generation, checkpoint_txg) is greater than the selected version's. If both are unreadable: true. When true, the read stage is redone once, on the read cache that is valid within this mount (the cache only keeps locations that were read successfully and are not all zeros); R = 1, with no waiting between the two reads. If the redone pass is false, the mount continues with it; if it is still true, the writable mount is refused before instance acquisition, with the disks unchanged byte for byte, and the read-only mount proceeds as usual. If the instance table of the newest root, which the allocator rebuild uses to judge abandonment, is unreadable, it is likewise reread once, and if it is still unreadable the writable mount is refused. The acquisition write does not write tail 0: it writes the witness value read at the moment of acquisition by the same method (over the two slots of every device, the maximum journal tail of all self-verified slots whose filesystem id is this pool's; 0 when there is none; read once after the first barrier and before the first acquisition write). The rollback write after a failed acquisition carries the same witness value. If at the moment of acquisition some device has not a single self-verified system configuration slot (the pass that reads the witness value fails while the per-device check pass before it could read): refuse; the rule text says this refusal is being implemented. A newest root that the system configuration did not witness: after the acquisition-write part lands, the next mount sees it in its own c_witness; this form is recorded as an open item.

F14 (rule, abandoned roots). Abandoned timelines are today only created by crash recovery (an administrator rollback is a forward publish while mounted and abandons no root). The blocks that abandoned roots reference likewise must not be reallocated and must not have their headers wiped before those roots leave the root ring.

Part C. What the code does today (function names only)

F15 (newer_publish_witness and verified_system_configuration_slots). A slot that cannot be read and a slot that does not self-verify drop out of c_witness in the same way. If no slot remains, c_witness = 0 (NothingWitnessed, judged false). The comparison named Undecidable is judged true.

F16 (read_stage and read_stage_settled_at_most_on_the_one_reread). On each pass the witness read reads the system configuration slots directly from the devices. First pass false: continue with it. First pass true: call the hook before_rereading, redo the read stage; reread pass true: refuse with NewerStateStillUnreadableAfterOneReread before acquisition, disks unchanged byte for byte; reread pass false: continue with the reread pass.

F17 (order inside the writable mount). Read stage with its one reread; rebuild the previous version; rebuilt_allocator (reads the newest root's instance table and computes the shadow ledger); then establish_instance, which does the acquisition write and the publishes of the new instance. Every step before establish_instance receives the devices by shared reference, and the block device write operation needs a mutable reference. A refusal returned before establish_instance leaves the disks unchanged byte for byte.

F18 (write_system_configuration_slot). The slot generation written on a device = (the largest slot generation among that device's self-verified slots) + 1, or 1 when no slot of that device self-verifies; the physical slot written = generation mod 2.

F19 (self_verified_system_configurations_of_every_device, today's acquisition check). At acquisition the two slots of every device are read again; a device with zero self-verified slots makes the acquisition fail; a device with at least one passes. The acquisition write then writes, on every device, one system configuration slot carrying the maximum journal tail read at that moment.

F20 (what a writable mount publishes). After acquisition the new instance writes its row publish and warm-up publishes, each ending with a system configuration rotation on every device (F10). In the history of F06, instance 3 wrote its row at txg 9 and a warm-up at txg 10. In the torn rows, today's mount 1 made instance 3, and three later overwrites were published at txg 11, 12 and 13.

F21 (mount_read_only). It chooses the system configuration, chooses the root, scans the journal, replays, and opens the pool along the resulting root. It writes no byte. The reader it is given offers only read operations (list devices, device size, read, journal offset hint); it has no write operation.

F22 (choose_root and root_is_abandoned_by_the_instance_table). choose_root picks, among all self-verified roots in the root ring, the one with the largest (checkpoint_txg, instance generation). A root (i, T) is abandoned when the instance table of the newest root has a row (i, Ti, ...) and T > Ti. The acquisition of a new instance writes a row for the instance it continues from: (that instance, the selected root's txg, ...).

F23 (isolate_slots_referenced_only_by_abandoned_roots, the shadow ledger). It runs in every writable mount, over the readable roots in the root ring. For each abandoned root it calls placements_referenced_by_root once (this reads that root's tree table and allocation record tree). If that returns None, it adds 1 to an unreadable counter and continues: nothing of that root is isolated, nothing is reread, no error, no refusal. Isolation is a bit in the allocator of that mount, separate from the allocation bit; the ledger is recomputed at every mount and at every raise of the rollback floor. The isolation of an abandoned root is cleared only by the publish that makes that root leave the root ring.

F24 (target_for_publish). The root ring slot a publish writes is determined by its txg alone. A root leaves the root ring only when a later publish overwrites its slot. In the history of F06, C's root slot is overwritten by the publish at txg 32.

Part D. Measured results of mount 1 only (no second mount was measured)

F25 (Today, torn rows, mount 1). All three torn rows: the writable mount succeeded with no loss; the selected version was D (2, 8), D's last record counter 8; first-pass c_witness 7 in torn_both and torn_d0, 8 in torn_d0_d1_persisted; settled on the first pass.

F26 (Jia, torn rows, mount 1). All three torn rows: the writable mount was refused with NewerStateStillUnreadableAfterOneReread.

F27 (Today, tree table row and allocation record tree row, mount 1). The writable mount succeeded; unreadable counter 1; isolation 0 / 0; the first overwrite after the mount (txg 14) reused C's data unit; the pool checker's I-7.4 was red. The read stage settled on the first pass: the selected version (3, 10) is the one the system configuration witnessed (jsn 10).

F28 ((a), tree table row and allocation record tree row, mount 1). The writable mount was refused; the reason names the account of an abandoned root.

Part E. Candidates (columns)

- Today: the code of Part C.
- Jia: in the witness read, if any system configuration slot is unreadable or fails self-verification, the comparison is treated as Undecidable (true), so the read stage is redone once; if the redone pass still has such a slot, the writable mount is refused. In the version measured, this is "some device has fewer than 2 self-verified slots". Its acquisition extension: at acquisition, a device with fewer than 2 self-verified slots makes the acquisition refuse. Everything else as today.
- Jia-read-error: only a slot whose read returns a device error (EIO) counts as undecided; a slot that is read but does not self-verify (torn, all zeros, bad content) drops out of the maximum as today. Its acquisition extension uses the same definition of undecided. Everything else as today.
- (a): when the shadow ledger cannot read or cannot decode an abandoned root's account (tree table or allocation record tree), reread once; if it is still so, refuse the writable mount; the read-only mount proceeds as usual. In the version measured, the refusal is returned from inside rebuilt_allocator, before establish_instance. Everything else as today.
- (b)-from-tree-table: when an abandoned root's allocation record tree cannot be read and the same root's tree table can be read, walk from the tree table to compute the slots this root references, isolate them only in memory, and write nothing to disk. When the tree table cannot be read, do as (a). Everything else as today.

QUESTIONS

Q1. Trajectory grid. Rows, in this order: torn_both, torn_d0, torn_d0_d1_persisted, tree table, allocation record tree. Columns, in this order: Today, Jia, Jia-read-error, (a), (b)-from-tree-table. That is 25 cells, numbered 1.1 to 1.25: torn_both-Today, torn_both-Jia, torn_both-Jia-read-error, torn_both-(a), torn_both-(b)-from-tree-table, torn_d0-Today, and so on to allocation record tree-(b)-from-tree-table. For each cell write one block of lines:
 1.n Row-Column: mount 1 = W or R; mount 2 = W or R; mount 3 = W or R
 after mount 3: STAYS or CAN CHANGE; if CAN CHANGE, name the event in the fact table that changes it and at which mount
 disk between mounts: what each mount writes to the disks (or "nothing"), whether the torn slot is rewritten, whether C leaves the root ring, and at which mount
 facts: the fact ids you used, for mount 1, mount 2 and mount 3 separately
 assumption: only when the candidate's definition does not settle the cell
 overturned if: one observation that would make this trajectory wrong

Q2. List every cell where you picked an answer under an assumption, and every cell where two trajectories seemed possible. For each, name the missing definition or the missing fact. Number them 2.1, 2.2, and so on.

Do not add any section beyond Q1 and Q2.
