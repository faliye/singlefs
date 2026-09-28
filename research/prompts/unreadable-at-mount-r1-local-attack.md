Outcome grid for two mount-time "cannot read" cases in a copy-on-write file system (attacking leg)

You are the attacking side in a review of a file system design. Your job is to fill two outcome grids by reasoning only from the fact table below. You do not run code. Do not assume facts that are not in the table. Where a candidate's definition leaves a cell open, pick the outcome under an assumption and state that assumption in the cell.

Rules for your answer:
- Write in English only.
- Do not use markdown emphasis (no asterisks and no underscores around words). Plain text lines only.
- Do not cite source code line numbers or file line numbers. Refer to facts only by their fact id (T01, T02, ...) and to code only by function name.
- Answer every cell. Do not answer with only yes or no.
- Number your answers exactly as asked in Q1 to Q4.
- For every cell, write one observation that would overturn your answer.

Notation. A version or root is written (instance generation, checkpoint_txg). jsn is the journal record counter. An acknowledged write is one whose fsync returned. "Unreadable" means the read returns a block device error. A root that cannot be read at a given mount is called hidden at that mount.

Outcome labels. Use exactly one per cell:
- REFUSE: the candidate refuses a writable mount in this history (name which mount); the read-only mount still proceeds.
- WRITABLE-NO-LOSS: every writable mount in the history succeeds, no acknowledged write is lost, and no unit referenced by an abandoned root that is still in the root ring is reallocated.
- WRITABLE-LOSS: a writable mount succeeds and, by the end of the history, an acknowledged write is lost or overridden (name whose write: the new instance's or the older instance's).
- WRITABLE-REUSE: a writable mount succeeds, no acknowledged write is lost, but a later publish reallocates a unit referenced by an abandoned root that is still in the root ring.
If one history has both a refusal and a loss, use WRITABLE-LOSS when the loss happens before the refusal or without it, and REFUSE only when the refusal prevents the loss.

FACT TABLE

Part A. Design rules

T01 (rule: a writable mount finds a newer state it cannot read). After the read stage of a writable mount (choose the root, scan the journal, replay) has finished, take c_witness = the maximum journal tail over all self-verified system configuration slots among the two slots of every device in the pool. If c_witness is 0: not judged. If the record of the selected version's publish that carries the last-record flag is readable (either copy): the criterion is true when c_witness is greater than that record's counter. If that record is unreadable but the record whose counter equals c_witness is readable: the criterion is true when that record's (instance generation, checkpoint_txg) is greater than the selected version's. If both records are unreadable: treat the criterion as true.

T02 (rule). When the criterion of T01 is true, redo the read stage once, on the read cache that is valid within this mount (the cache only keeps locations that were read successfully and are not all zeros). R = 1, taken from the "reread once" in the "root slot read fails this time" row of the publish-semantics rule; there is no waiting between the two reads. If the redone pass is false, continue with the redone pass. If it is still true, refuse the writable mount before instance acquisition, with the disks unchanged byte for byte; the read-only mount proceeds as usual.

T03 (rule). If the instance table of the newest root, which the allocator rebuild uses to judge abandonment, is unreadable, it is likewise reread once, and if it is still unreadable the writable mount is refused.

T04 (rule, the "continuation" part). The instance acquisition write no longer writes tail 0. It writes the witness value c_witness read at the moment of acquisition by the same method: over the two slots of every device, the maximum journal_tail over all self-verified system configuration slots whose fsid equals this pool's; 0 when there is none; read once after the first barrier and before the first acquisition write. The rollback write after a failed acquisition carries the same witness value and does not fall back to 0. mkfs writes tail 0; the first acquisition after mkfs reads 0 and writes 0, so the bytes of a newly created file on a new pool are unchanged.

T05 (rule). If at the moment of acquisition some device has not a single self-verified system configuration slot (the pass that reads the witness value fails, while the per-device check pass could read): refuse. The rule text says this refusal is being implemented.

T06 (rule). A newest root that the system configuration did not witness: after the continuation part lands, the next mount sees it in its own c_witness. This form is recorded as an open item.

T07 (rule, ordering). The journal record watermark (instance generation, checkpoint_txg) is compared with instance generation first, in the same order as jsn. Root choice still compares txg first and breaks ties by instance generation. The two places compare different things and each keeps its own order.

T08 (invariant I-7.4, blocks of recent generations not reused). Roots of an abandoned timeline are not in the rollback candidate set; that only governs the choice of rollback target. Today abandoned timelines are only created by crash recovery. The blocks they reference must likewise not be reallocated and must not have their headers wiped before those roots leave the root ring. The pool checker walks every root in the root ring that is judged abandoned, each with its own walk, and only checks whether what it reads back is correct: a checksum violation or a failed walk marks that root red.

T09 (rule). Authoritative state = units + accounting + roots; losing it loses data. Derived state = indexes (trees); losing it only makes things slower. The "roots" entry includes its indirection layer: root record + tree table unit + instance table unit.

T10 (rule). The allocation record tree can be rebuilt entirely from authoritative state; it is not on the authoritative state list.

Part B. What the code does today (function names only)

T11 (newer_publish_witness). c_witness = the maximum journal_tail over the slots that verified_system_configuration_slots returns for every device. A slot that cannot be read or does not self-verify simply drops out of the maximum. If no slot remains, c_witness = 0 and the comparison is NothingWitnessed.

T12 (witnesses_a_publish_newer_than_the_selected_version). NothingWitnessed gives false. The comparison against the selected version's last record gives: c_witness greater than that record's counter. The comparison against the record at the witnessed counter gives: that record's (instance, txg) greater than the selected version's. Undecidable gives true.

T13 (read_stage_settled_at_most_on_the_one_reread). If the first pass is false, the first pass is used. If the first pass is true, the hook before_rereading is called, then the read stage runs again, and this pass reads the system configuration slots again. If the reread pass is true, the mount is refused with NewerStateStillUnreadableAfterOneReread. If the reread pass is false, the reread pass is used. Only the reread pass's own witness is used: the witnesses of the two passes are not merged into a maximum, and the version the reread pass selects is not required to be newer than the first pass's.

T14 (write_system_configuration_slot). The slot generation written on a device = (the largest slot generation among that device's readable self-verified slots) + 1, and the physical slot written = generation mod 2. So when the slot with the largest generation on a device is unreadable, the write lands back on that very slot.

T15 (write_acquired_instance). The tail that the acquisition write carries = the largest journal_tail among the self-verified slots read at acquisition (readable slots only).

T16 (self_verified_system_configurations_of_every_device). At acquisition, a device with zero self-verified slots makes the acquisition fail. A device with at least one readable self-verified slot passes.

T17 (first_txg_of_new_instance and target_for_publish). A new instance's first checkpoint_txg = max(largest txg among readable roots in the root ring, largest checkpoint_txg among self-verified journal records) + 1. The root ring slot that a publish writes is determined by txg alone. So a hidden root is overwritten when the new instance publishes the txg that maps to its slot. Only a hidden root whose txg is higher than every publish the new instance has made can later override it; when the hidden roots are no more than the new instance's publishes, their root slots are overwritten one by one.

T18 (next_counter in the writable mount). The new instance's next journal counter = the largest readable record counter + 1. It is not compared with the witness.

T19 (isolate_slots_referenced_only_by_abandoned_roots). For each abandoned root, placements_referenced_by_root is called once. If it returns None (the tree table or the allocation record tree of that root cannot be read or decoded), the mount adds 1 to an unreadable counter and continues: none of that root's placements are isolated, nothing is reread, no error is returned, and the mount is not refused.

T20 (the two rereads that exist today). The read-stage reread of T13 concerns the criterion of T01. The shadow ledger's only reread is of the newest root's instance table (the hook before_rereading is called right before it). Neither touches an abandoned root's own tree table or allocation record tree.

T21 (read_mapped_tree_node_via_hint_then_central_mapping and tree_table_entry_count). Allocation record tree nodes: the two copies are read at the hinted locations; if both fail, the central mapping is looked up and the mapped locations are read once more (in the histories below, the same slot). Tree table: each copy is read once, with no mapping fallback.

Part C. Histories and measured results (C331 family)

T22 (common history). Instance 1: warm-ups txg 1 and 2, A at txg 3. Reopen as instance 2: row txg 4, warm-up txg 5, B at txg 6, C at txg 7; all acknowledged. With journal counters: A = (1, 3, jsn 3), B = (2, 6, jsn 6), C = (2, 7, jsn 7). Two devices, two system configuration slots per device.

T23 (order of system configuration slot reads within one writable mount, from a call-stack trace). Read 1 chooses the system configuration. Read 2 checks the device table. Read 3 is the first-pass witness read. Read 4 is the reread-pass witness read. Read 5 comes after the read stage (it computes the effective rollback floor). Moving the fault of V1 from read 3 to read 5 turned the outcome back into a refusal of the writable mount (both passes read witness 7).

T24 (the original C331 form, today). When all roots and records of instance 2 stay unreadable at the new-instance mount: refuse writable (witness 7 is greater than A's last record 3, still true after the one reread); the new instance has not written a single byte, so there is no acknowledged write that can be overridden; after the fault is lifted the writable mount chooses C. When only the roots are hidden and the records are readable: also refuse, because replay does not cross instances and the selected version is still A. When the fault hits only once, or only the reread can read: the reread chooses C; the new instance writes E, acknowledged; after the fault is lifted and the pool reopened, root choice picks E.

T25 (row V1, today). The four system configuration slots are unreadable only on read 3 (the first-pass witness read); all roots and records of instance 2 are unreadable. c_witness is read as 0, NothingWitnessed, false, no reread; writable mount from A; instance 3 writes E (txg 6), acknowledged. The faults are lifted and the pool reopened: root choice picks C (2, 7), and E is overridden. The loss is the new instance's. The read-only mount picks C and cannot read the file back; the writable mount reports UnitUnreadable (instance 3's row reused C's instance table area).

T26 (row V2, today). Same as V1, but the fault hits read 4 (the reread-pass witness read): the first pass read witness 7 and judged true. The reread pass still selected A, but read c_witness 0, judged false, and the mount continued with the reread pass. Same outcome as V1: E is overridden; the loss is the new instance's.

T27 (row V3, today). On both devices, the slot that witnessed the newest publish C is persistently unreadable; C's root and records are unreadable. c_witness falls back to 6, equal to B's last record, false; writable mount from B; instance 3's row txg 7 overwrites C's root slot; E survives. The loss is the older instance's: C is lost and the mount does not refuse.

T28 (row V4, today). Two faults that do not happen at the same time. At mount k, on both devices the slot that witnessed C is temporarily unreadable (C's root is readable and root choice picks C); the acquisition write lands back on that slot (T14) and writes tail 6, erasing witness 7; the mount crashes before the row write. After the crash all four slots carry tail 6. At mount k+1, C's root and records are temporarily unreadable: c_witness 6 equals B's last record, false, writable mount from B; C (acknowledged) is lost; the new instance's write E survives. The loss is the older instance's.

T29 (why V1's final reopen is not refused either). The new instance's next_counter (T18) is not compared with the witness. Instance 3's records start from jsn 4, and each rotation write carries the counter of that publish. At the end the tails on disk are only 6 and 5, below C's last record 7, so the final reopen's read stage judges false and lets C through.

T30 (forms that today's code already blocks; reference only, not grid rows). (i) A newest root D that the system configuration never witnessed (after D's root was written with FUA, the rotation write failed and was not acknowledged; crash; D's root and records unreadable): witness 7 equals C's last record, false, writable mount from C; instance 3's row txg 8 lands on D's root slot and overwrites D; E survives; nothing acknowledged is lost (D was never acknowledged). (ii) Only device 1 has both system configuration slots persistently unreadable, and all roots and records of instance 2 are unreadable: device 0 still witnesses 7, refuse writable. (iii) Roots and records readable only on the reread, unreadable again afterwards: the reread chooses C; afterwards the few places that read the disk directly cannot see instance 2's roots, and the first txg is set to 8 by the reread pass's records; E survives.

T31 (scope of the measurements). All of the above were built only with this one set of parameters: two devices, region ownership [0, 1, 0], one record per overwrite, history instance 1 to instance 2 to instance 3. V1 and V2 need all four slots to be unreadable exactly at that one witness read.

Part D. History and measured results (C393 family)

T32 (history). A and B (instance 1); reopen, acquire instance 2; C = (2, 8); crash recovery abandons C (C's root slot and data units temporarily unreadable, and the system configuration slot that witnessed C broken; recovery lands on (2, 7); instance 3 writes its row at txg 9 and a warm-up at txg 10); C becomes readable again; then a writable reopen (instance 4), and this time one of C's accounts (the tree table, or the allocation record tree) is unreadable in both copies (block device error).

T33. In every arm, the instance 4 mount settles on the first read: the selected version (3, 10) is the one the system configuration witnessed (jsn 10), so the criterion of T01 is false; the newest root (3, 10)'s instance table is readable. C (2, 8) is older than the selected version. The hook before_rereading is called 0 times in every arm; in a positive control on the same history, where (3, 10)'s instance table is unreadable, it is called once and isolation is 14 / 14.

T34. C exclusively holds 14 slots (the readable control isolates 14 on device 0 and 14 on device 1). If the shadow ledger does not isolate them, after the reopen the allocator treats them as free, and the first overwrite after the reopen (txg 14) writes a new data unit on C's data unit slot. C's root slot is only overwritten at txg 32, so at txg 14 C is still in the root ring and I-7.4 is red.

T35 (measured arms today, instance 4 mount; isolation is device 0 / device 1; counter is the unreadable counter of T19).
 R1 control, both accounts readable: mount OK; counter 0; isolation 14 / 14; nothing of C reused through txg 31; I-7.4 holds.
 R2 allocation record tree root, unreadable on every read: mount OK, not refused; counter 1; isolation 0 / 0; reused at txg 14 (C's data unit); checker red, only I-7.4.
 R3 allocation record tree, unreadable on every read until the product's reread hook (if the product rereads, the hook lifts the fault): same as R2; hook called 0 times.
 R4 allocation record tree, each copy fails only on its first read: mount OK; counter 0; isolation 14 / 14; nothing reused through txg 31; I-7.4 holds.
 R5 tree table, unreadable on every read: mount OK, not refused; counter 1; isolation 0 / 0; reused at txg 14; red, only I-7.4.
 R6 tree table, unreadable until the reread hook: same as R5; hook called 0 times.
 R7 tree table, each copy fails only on its first read: mount OK; counter 1; isolation 0 / 0; reused at txg 14; red, only I-7.4.

T36 (why R4 and R7 differ). The allocation record tree root is a mapped node: both hinted copies fail, then the central mapping fallback reads the same slot once more and succeeds (reads per copy: 2 and 1). That second read is the mapping fallback, not the product's reread. The tree table is exempt from the mapping and is read once per copy (1 and 1); there is no second read. The product-level "reread once" (T02, R = 1) has no effect on this read: the hook is called 0 times.

T37 (the investigator's experiment on a copy, used to locate the cause, not a proposed fix). The shadow ledger was changed so that when placements_referenced_by_root returns None, it is called once more. R7 flipped to isolation 14 / 14, counter 0, nothing reused through txg 31, I-7.4 holds. R2, R3, R5 and R6 kept isolation 0 / 0, counter 1 and reuse at txg 14; only the number of reads doubled.

T38 (not covered). The path that recomputes the shadow ledger when the rollback floor F is raised while mounted (admission or unmount) reads an abandoned root's accounts again, so arms that become readable later might get isolated after a raise of F. In this history the reuse at txg 14 happens before any raise of F.

Part E. Candidates

C331 family (some system configuration slot is unreadable in the witness read):
- Jia, unreadable means undecided: if any system configuration slot is unreadable or fails self-verification in the witness read, treat the criterion of T01 as true and go to the reread; if the reread still has an unreadable slot, refuse writable. (This is the change the investigator made on a copy; V1 to V3 flipped to refuse.)
- Yi, judge per device: only when both slots of one device are unreadable is it treated as undecided; when every device has at least one self-verified slot, take the maximum of the readable ones as today.
- Bing, merge the two passes: the reread pass's c_witness is the maximum of what the two passes read; everything else as today (this only fixes the V2 cell).
- Ding, keep today: register "the witness read and the roots unreadable at the same time" as a multiple fault that is not guaranteed, and write it into the scope.
- The V4 branch (the witness value of the acquisition write): each candidate states how it takes that value. The rule already has "at acquisition some device has not a single self-verified system configuration slot: refuse" (T05); check whether that covers V4.

C393 family (an abandoned root's account is unreadable), judged once for the tree table and once for the allocation record tree:
- (a) refuse writable and report: if it is still unreadable after one reread, refuse writable; read-only as usual.
- (d) conservative isolation: when it is unreadable, treat every slot written by the instance of the abandoned root as isolated as a whole, not reused, until that root leaves the root ring (state where "slots written" come from).

QUESTIONS

Q1. Grid 1 (C331 family). Rows: V1, V2, V3, V4. Columns: Today, Jia, Yi, Bing, Ding. That is 20 cells, numbered 1.1 to 1.20 in this order: V1-Today, V1-Jia, V1-Yi, V1-Bing, V1-Ding, V2-Today, and so on to V4-Ding. For each cell write one block of lines:
 1.n Row-Column: LABEL
 mount: which mount the label refers to (new-instance mount, final reopen, mount k, or mount k+1)
 numbers: c_witness on the first pass, c_witness on the reread pass (or "no reread"), the selected version and its last record counter; for V4 also the tail written by the acquisition write at mount k
 facts: the fact ids you used
 assumption: only when the candidate's definition does not settle the cell
 overturned if: one observation that would make this label wrong

Q2. Grid 2 (C393 family). Rows: R1 to R7 of T35. Columns: Today, (a), (d). That is 21 cells, numbered 2.1 to 2.21 in this order: R1-Today, R1-(a), R1-(d), R2-Today, and so on to R7-(d). For each cell write one block of lines:
 2.n Row-Column: LABEL
 numbers: isolation device 0 / device 1, the unreadable counter, and the txg of the first reuse of C's units (or "none through txg 31")
 facts: the fact ids you used
 assumption: only when the candidate's definition does not settle the cell; for (a) in R3 and R6 say whether your reread calls the hook before_rereading; for (d) say where "slots written by the instance" come from
 overturned if: one observation that would make this label wrong

Q3. V4 only. For each of Today, Jia, Yi, Bing, Ding (numbered 3.1 to 3.5), give: (i) the witness value that the acquisition write writes at mount k; (ii) whether the refusal of T05 fires at mount k, with the number of self-verified system configuration slots each device has at that moment; (iii) whether that is enough to keep C. Each with facts and "overturned if".

Q4. List every cell where you picked a label under an assumption, and every cell where two labels seemed possible. For each, name the missing definition. Number them 4.1, 4.2, and so on.

Do not add any section beyond Q1 to Q4.
