# Background dossier: D26 open items (background compaction and placement reclamation)

All facts below were verified today (2026-09-03) by reading the decision texts verbatim or by
running the cited experiments. Do not assume facts beyond this dossier; if a proposal depends on
something not listed here, flag it as an unverified dependency.

## Verified facts from settled decisions

F1. Project discipline splits accounting into three cells: (cell 1) runtime decision paths must
never traverse, cost must not grow with disk capacity; (cell 2) checker/audit MUST traverse and
must not share the runtime's algorithm; (cell 3) background, resumable, non-decision work
(snapshot destroy, compaction, scrub) has NO restrictions. Compaction is explicitly named in
cell 3.

F2. D26 is half-settled: long-term fragmentation is answered by two stacked legs. Leg 1: a
resident, automatic background compaction plus our own placement and reclamation layer, on all
media, explicitly NOT format-level. Leg 2: structural aging resistance in the index itself,
format-level, currently blocking D8's node-format freeze. NVMe FDP/ZNS demoted to optional
acceleration. Six open items remain (trigger criterion; transaction shape; does compaction
reserve space; form of structural aging resistance; interaction with snapshot accounting;
runtime observability).

F3. D8 (core index): one btree implementation, multiple keyspaces, log-structured 16 KiB nodes
(pinned constant), write-buffer front end. Extent-tree key is (locality_id, inode, offset);
locality_id is inherited at creation, never updated on rename. D11 settled: internal nodes keep
a message buffer, epsilon = 0.65 (fanout 119, buffer 665 entries per 16 KiB node).

F4. D8 settled item 5: unbounded operations (truncate, big deletes) write an intent as an
ORDINARY KEY into a logged-ops tree riding the same transaction layer, then proceed in batches,
resumable; after a crash there are exactly three states (no intent / intent + partial work,
idempotently redone / work done, intent not yet deleted). Deleting the intent commits in the
same transaction as the last batch.

F5. D13: crash-replay sampling is defined over a CLOSED enum CommitStep { WriteDataBlock,
WriteStripeParity, WriteInternalNode, CasRootPointer, AppendRootRecord, ZoneFinish, ZoneReset }.
Any new structural commit step must be a diff to this shared enum; parameters (how many bytes,
how many repetitions) are not new steps.

F6. D3: principle: "freeing space must itself never require allocating space". Settled item 2:
deletion writes tombstone units, and that cost is prepaid by admission control at WRITE time
("the tombstone is part of the worst case of undoing yourself"). Settled item 4: the admission
inequality is: available = sum over devices (capacity - allocated - unreclaimable -
defer-pending-release) - pending-delete - committed-reservations. Settled item 6: admission
reads published stats minus in-flight approved reservations; releases within the window earn no
credit; the in-flight overlay is pure in-memory derived state. Settled item 5: blocks generated
by the commit itself (tree nodes, accounting nodes) must be allocated from a contiguous bump
segment; E81 measured that scattering them turns a 12-unit fsync into 5909 metadata blocks (65
fixpoint rounds), clustering cuts it to 2 rounds / 28 blocks.

F7. D5 (snapshots): ZFS birth-txg + deadlist model. Reference condition: snapshot S references
block b iff birth(b) <= S.txg < death(b). Kill rule: birth > prev_snap_txg means free
immediately, else append to the live head's deadlist; snapshot creation hands the head deadlist
to the new snapshot; destroying the oldest merges its deadlist into the next newer side and
frees entries filtered by birth > prev(S).txg. birth is defined as "the checkpoint number at
which the block was PUBLISHED".

F8. D9 settled item 5: the authoritative record of a block's physical address was moved OUT of
the encrypted structures into a plaintext logical-identity-to-physical mapping layer plus a
plaintext reverse index, precisely so keyless maintenance can move blocks; this was paid for
D3's resident-compaction promise. D9 settled item 7: the keyless side can MOVE but not RECLAIM.
D9 open item 9: today the mapping layer exists only when encryption is on; check C34 records
three conflicting statements about whether the mapping layer is authoritative/derived/
unconditional. D19: block pointers carry position entries (device id + offset + ciphertext
checksum) inline in the referencing trees.

F9. D21 open item 1: whether the reverse index is authoritative or derived is open; if derived,
compaction cannot run until the index is rebuilt after loss (a stall window with zero test
coverage today). It does not block the first transaction (zero shared units then).

F10. D2 settled item 11: partially-freed wide stripes are pinned and lazily re-laid by leg 1 of
D26 under a budget; pinned cells are counted in the "unreclaimable" statistic. D12 settled item
5: the first layout line is pure SSD (no FDP/ZNS to lean on in phase 1).

F11. D16 new rule 2: space freed within checkpoint C must not enter the allocatable set until C
publishes.

## Verified experiment results (run today, mutation-tested, 5-run byte-identical)

E93 (aging placement model; 8192 fixed-size objects, 10240 slots = 80 percent fill, segment 64
slots, 64 dirty objects per checkpoint, 2000 checkpoints, 5 seeds, medians):
- Fragmentation metric F = runs / L (runs = physically contiguous segments when traversing all
  objects in key order) is maintainable in O(1) per move; an independent full-scan audit agreed
  at every sample point in every arm.
- Without compaction, aging saturates fast: first-fit reaches 99.9 percent (uniform) and 97.6
  percent (8-run workload) of max fragmentation, most of it within 500 checkpoints.
- Checkpoint-clustered bump-segment allocation WITHOUT compaction is not self-sustaining:
  91.8-93.8 percent of allocations fall back because scattered frees almost never empty a whole
  segment. Compaction budget is the only force that restored empty-segment supply (fallback 88
  to 50 percent as budget grows).
- Write-path neighbor co-rewrite (each dirty key run extended by up to R clean neighbors,
  placed contiguously) is a real no-format-change aging-resistance form but SATURATES around
  F = 66 percent and its budget is capped by workload geometry (on the run workload it can only
  spend 1.5x writes even with R = 2). At equal extra-write budget it and compaction split wins
  (neighbor wins the 3x uniform cell 5497 vs 6095 and the 1.5x run cell 5111 vs 6073; compaction
  wins the 5x uniform cell 4791 vs 5460); compaction alone keeps improving with budget (down to
  F = 10-21 percent at 5x on the run workload).
- E93 explicitly does NOT replace E10 (the real-filesystem aging experiment, still owed): no
  time axis, no message-buffer batching statistics, and no arm for format-level node-layout
  changes.

E94 (move touch-set model; persistent COW tree, 4096 keys, fanout 16, K retained snapshots with
rotation destroying the oldest, u updates between snapshots plus u post-snapshot updates, batch
of 64 physically adjacent occupied slots, 5 seeds, medians):
- Architecture ptr_rewrite (position entries live in every referencing tree's pointers, the
  D19 literal shape): moving the batch COWs 106/181/244 tree nodes at K = 1/4/16 (u = 1024),
  rewrites ALL K+1 root records every batch, and rewrites 11-36 deadlist entries (deadlist
  entries keyed by position). Snapshot roots are published identity anchors; per-batch root
  drift breaks anything pinned to them.
- Architecture central_map (single logical-identity-to-physical mapping, the D9-item-5 shape
  made unconditional): 13-62 mapping-tree nodes depending only on batch key dispersion,
  K-invariant, exactly 1 root reference, zero deadlist rewrites. Node ratio vs ptr_rewrite:
  1.7x to 5.4x on aged batches, 13.8x on key-clustered batches.
- Architecture skip_shared (move only extents referenced by exactly one tree): movable fraction
  of an aged physical batch is 0 to 17 percent, and it DROPS as the system gets quieter (at u =
  64 essentially zero). The ceiling is low and inverted (old quiet disks need compaction most
  and can move least).
- Negative arm: reassigning birth on move produces violations exactly equal to the number of
  moved versions referenced by at least one snapshot (independently counted) in all 60 cells.
  birth belongs to the logical version, not to the position.
- E74 (earlier, referenced): allocation-record btree keyed by (device, offset) puts a
  physically-adjacent compaction batch at the cheap end (2.6x COW amplification vs 964x for
  key-scattered batches); the tree itself costs 0.055 percent of capacity.

## The six proposals under review

P1 (item 2, transaction shape): A compaction batch is an ORDINARY transaction sequence: write
new locations (existing steps WriteDataBlock / WriteInternalNode), update references and
accounting incrementally, publish; the unbounded whole rides D8-item-5's intent mechanism
(intent = ordinary key, batches, resumable). No new CommitStep enum member is needed, so leg 1
stays non-format-level. If some future media line needs a structural step (e.g. zone finish),
that member already exists for that line and is not compaction-specific.

P2 (item 3, does compaction reserve space): The principle "freeing space never allocates" governs
the FREEING path; compaction is not a freeing path but a space-consuming operation, so the
principle is neither violated nor applicable. Compaction batches go through the same admission
control as any writer (D3 items 4/6); the batch's worst case (new locations + metadata COW)
counts into "committed reservations" while in flight; old locations free via defer after
publish. Under a full disk compaction stalls, which is acceptable for correctness because
deletion never depends on compaction (tombstone cost prepaid at write time, D3 item 2). Open
sub-question for the user: whether to keep a small STANDING reserve (a few empty segments) so
commit-intrinsic clustering (D3 item 5, E81) never degrades - E93 shows compaction is what
sustains empty-segment supply.

P3 (item 5, interaction with snapshot accounting): Three sub-answers. (a) birth NEVER changes on
move - it is a property of the logical version (E94 negative arm). (b) whether deadlist entries
must be rewritten depends on their key: position-keyed entries must be rewritten per move under
ptr_rewrite (11-36 per 64-batch), logical-identity-keyed entries are untouched; the deadlist key
choice must be settled together with the position-authority architecture. (c) allocation-record
rewrite cost is already quantified by E74 (cheap end, thanks to the position-keyed key). The
larger question - where position authority lives (ptr_rewrite vs central_map vs skip-shared) -
sits at the intersection of C34 / D9 open item 9 / D21 open item 1 and is NOT settled by this
round; E94's numbers (K-linear cost, per-batch rewriting of all snapshot roots, identity drift
of published snapshot roots vs K-invariant single-root central map) are handed to the user as
the decisive input.

P4 (item 4, form of structural aging resistance): Leg 2 takes a POLICY form, not a format form:
locality_id key prefix (already settled in D8 item 3) + clustered bump-segment placement +
optional write-path neighbor co-rewrite radius R (runtime policy knob). No new node-layout
field, no competition with D11's message buffer for node bytes, stacking (not replacing)
locality_id. Consequence: item 4's verdict "changes the first transaction's bytes: YES" flips
to NO, and D8's node-format freeze is no longer blocked by D26. Risk exposure stated: E10 (real
filesystem, time axis) still owed; message-buffer batching statistics not modeled; if E10 later
shows the policy form is insufficient and node layout must change, the flip reverts.

P5 (item 1, trigger criterion): FORM settles now, thresholds stay open pending E10. Dual
watermark, both quantities maintained O(1) on the commit path and audited by the checker's
independent full scan (E93 validated both properties for F): (a) empty-segment watermark (count
of fully-free clustering segments) with hysteresis - guards allocation self-sustainability and
commit-intrinsic clustering (E81); (b) fragmentation degree F = runs / L - guards scan
locality. Item 1 narrows from "criterion unknown" to "threshold values unknown".

P6 (item 6, observability): the observables are the incremental F reading, the empty-segment
watermark, the fallback counter (allocations that could not enter a clustering segment), the
compaction sweep cursor, and per-round moved bytes / batch count. All runtime readings, nothing
persisted (the existing verdict "does not change first-transaction bytes: NO" stands; persisting
any of them reopens the item as already recorded).

# Your task (counterexample leg)

You are one of three independent reviewers. Your assigned stance is: hunt for counterexamples
and cross-decision conflicts. Do not restate the proposals and do not praise them. Answer in
English. Do not use any markdown emphasis (no asterisks, no bold, no italics).

For each proposal P1 to P6, do exactly this:
1. Name one concrete scenario or one settled clause from the dossier (cite the fact number F1
   to F11 or the experiment E93/E94/E74) under which the proposal would be wrong or incomplete.
   If you cannot find one from the dossier alone, say "no counterexample found from the dossier"
   and name what additional fact you would need.
2. State whether the counterexample, if it holds, changes the proposal's verdict about the
   first transaction's bytes (P4 in particular claims a flip from YES to NO).
3. Give a one-line severity: blocks the proposal / weakens it / cosmetic.

Then answer one closing question in at most five sentences: among P1 to P6, which single
proposal depends most heavily on a fact that is NOT in this dossier, and what is that fact?
