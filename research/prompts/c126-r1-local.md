You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italics. Number your answers 1 to 6 and keep each under 200 words. Your stance: attack the writer's preferred candidate (alpha) by constructing concrete sequences; if you cannot construct one, say so plainly.

Settled facts (do not dispute them; use them):

F1. A writable mount is admitted only if: writable devices >= lower bound of w AND the mount holds exclusive opens on more than half of the pool's devices AND the instance table is readable AND the reserve for instance switching can be obtained. If any conjunct fails the mount is read-only.

F2. An instance switch is a recovery performed inside the running mount: take a new instance number, write an instance-table row (old instance, last published txg, last applied transaction number W), and re-issue the in-flight checkpoint. Transactions with number <= W keep their effect (their journal records were already appended); transactions with number > W are redone under the new write order or reported as errors to callers that have not returned yet; all fixed-point units (COW nodes of the accounting trees and allocation-record tree written at checkpoint time) are rewritten under the new instance. Stated cost of one switch: one row + rewrite of the whole instance-table chain + rewrite of the data units of transactions > W + one fixed-point redo. The reserve for switching is an in-memory space quantity recomputed at every mount, never stored on disk. Reserve size = N_switch times the worst-case cost of one switch, with N_switch = 3. The worst-case cost of one fixed-point redo has no computable definition today; that gap is the debt under review. The reserve guards the allocation-failure path only, not the write-failure path. Allocation failures are routed three ways: a foreground user-data allocation failure returns ENOSPC to the caller and the checkpoint publishes normally; a background job's allocation failure pauses that job; only failure to obtain the switch reserve turns the mount read-only.

F3. Recovery replays only journal records whose (instance number, checkpoint txg) is strictly greater than the chosen root's watermark; jsn must be strictly consecutive and replay stops at the first gap. The chosen root is the newest readable self-verified root. W is the largest transaction number of the chosen root's instance applied by this replay. The recovered in-memory state (allocator, accounting, deferred-release queue) is rebuilt from the chosen root plus the applied records.

F4. Instance table: packed-record type 4, one page is 32768 bytes, one record is 64 bytes, floor((32768 - 135) / 64) = 509 records per page including the chain-pointer record; mkfs writes one empty page holding a single "no next page" chain-pointer record; rows are unique per instance number, later writes overwrite; rows are written only at recovery, rollback and switch, in the same publication as the first new root; every row write COW-rewrites the whole chain. The root record holds the pointer to the instance table directly (83 bytes wide: header 47 + two 14-byte location entries + instance number 4 + birth sequence 4). Two location entries means two copies. An open debt says the copy count was never enumerated separately, and another says the 64-byte row cannot hold an 83-byte chain pointer so row width and records per page must be recomputed.

F5. Admission inequality: available = sum over devices (capacity - allocated - unreclaimable - deferred releases) - pending deletions - committed reserves.

F6. Commit-internal blocks (COW nodes of accounting trees) are allocated from a clustered segment. A counting model measured: with clustering, one fsync of 12 units converges in 2 rounds and 28 metadata blocks; 1024 units in 3 rounds and 1275 blocks; fully scattered allocation takes 65 rounds and 5909 blocks. The model is a lower bound. A runtime reading of the convergence round count is still owed.

F7. Deadlock 2 (checkpoint needs COW allocation, disk full, cannot publish a root) is handled by a reserve pool with reserve >= checkpoint cost; a model run showed a 64-block reserve reduced stalls from 199 to 0. The reserve pool is a pure tax on ordinary allocation; false ENOSPC is handled by admission, not by the reserve pool.

F8. Deadlock 3: any single operation's worst-case journal occupancy must be <= ring size / F with F >= 2, computed at mkfs and re-verified at mount. An in-flight record count limit is stored in the superblock.

F9. Checkpoint trigger: elapsed >= T_time or dirty bytes >= T_dirty, T_dirty = 2 GiB initial value. In-flight checkpoint depth is two: one open, one publishing. Worst-case loss on crash is T_time + N_switch times (T_retry + T_redo); every switch waits out T_retry and then redoes the in-flight checkpoint.

F10. Allocation-record entries have exactly two states: allocated, or freed with a release generation. There is no third state, so a reserve cannot be expressed on disk.

F11. Releases performed inside checkpoint C do not enter the allocatable set before C is published (deferred releases).

F12. In a counting model of the near-full delete-then-create loop (capacity 4000 blocks), an admission-only hold that df does not subtract produced 24 false ENOSPC events once the hold exceeded 60 blocks; subtracting the same hold from df turned them back into true ENOSPC.

The three questions:

Q1. Does the redo after a switch need any block space beyond what the original checkpoint attempt already had? The writer's argument: the switch is a recovery; the rebuilt state comes from the chosen root plus records <= W; the failed attempt's fixed-point units and the units of transactions > W never entered any published allocation record, so in the rebuilt state they are free; the redo therefore reuses the same space and draws on the checkpoint reserve pool of F7. Attack this. Look at: transactions <= W whose allocations are now applied into the rebuilt state; the open (second) checkpoint's transactions that already wrote units; deferred releases of F11; foreground transactions arriving during the redo; old roots in the root ring still referencing the old chain pages and old fixed-point units.

Q2. Proposed computable definition. Switch reserve = sum over k = 1..N_switch of copies times 32768 times ceil((rows0 + k + 1) / 509), copies = 2, rows0 = row count read at mount, +1 for the chain-pointer record; fixed-point redo gets no separate term (by Q1). For the first transaction's geometry rows0 = 0, so each switch costs 65536 bytes and three switches 196608 bytes = 12 blocks of 16 KiB. What does the formula miss? Consider journal records of the redo (they live in the ring, not in block space), root slots (not allocated), mapping entries (one per unit; instance-table units are exempt), data accumulated by the open checkpoint, whether row recycling bounds rows0, and the recomputation of 509 once the row width changes.

Q3. Where should this quantity live: inside the committed reserves term of F5, or as a separate term? Given F12, must df subtract it? How would you build a counting-model self-test where a pool with fill ratio just below the threshold mounts writable and just above it does not?

Candidates: alpha = chain rewrite only, fixed-point redo covered by the checkpoint reserve pool (writer's preference). beta = alpha plus N_switch times a separate worst-case checkpoint cost. gamma = beta plus N_switch times T_dirty for rewriting the data units of transactions > W.

Answer these six items:
1. Q1: give a concrete sequence (transaction numbers, W, which block) in which the redo needs a block that the rebuilt state still counts as allocated, or a block the rebuilt state cannot provide although the reserve pool is intact. If you cannot, say so.
2. Q2: check the arithmetic and each assumption of the formula; state what is missing.
3. Q2: is 12 blocks of 16 KiB correct for the first transaction's geometry?
4. Q3: which term of F5 should hold the reserve, and must df subtract it?
5. Journal ring: after the redo, the records of transactions <= W and the redo's records coexist in the ring; does F8's factor F >= 2 and the in-flight record limit cover this, or is a separate bound needed?
6. Name any settled fact above that the writer's candidate alpha contradicts, quoting which one.
