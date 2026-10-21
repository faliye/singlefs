# Milestone "Small File Packing": Action Items, Acceptance and Sequence

[中文](05-fifth-txn.md) · **English** · [日本語](05-fifth-txn.ja.md)

**Measurement basis**: Planning document, established 2026-10-11 per user settlement (original quote: "I think Milestone 5 needs small file packing"; main agent suggested placing packing before concurrency, changing concurrency to Milestone 6, user settled: "Yes, arrange it that way"), currently a draft. **It does not require correctness**: revised before starting each item. Implements D27 (small-data packing container): 13 settled items, with the form being background compaction migrating objects ≤ 4 KiB into shared containers, write path not changing a single byte, and containers being a new unit class. Prerequisites written in "Entry" section: migration and reclamation piggyback on persistent compaction and region scan of D26 (background compaction and placement reclamation) (Milestone 4 Exit 2); container corruption relies on two replicas under D2 (RAID Stripe Policy) settled item 6; if any item is missing when Milestone 3 ([03-third-txn.md](03-third-txn.md)) and Milestone 4 ([04-fourth-txn.md](04-fourth-txn.md)) close out, this document is rescheduled according to what is missing.

**Reading method**: Four cells per item—"Original words" quotes user statement verbatim; "Existing in repository" details which clauses, owed checks, and code this item rests upon in the repository (checked 2026-10-11, with citations); "Action items" table: number, what to do, underlying clauses and owed checks, acceptance (each proving red beforehand), dependencies; "To settle before kickoff" lists unsettled questions requiring three-party review or user decision before kickoff. Statements labeled "inferred" are judgments unmeasured by the main agent. Action item numbering is written as "Item.Point" (I.1, II.3), used solely within this document.

**No new transaction kinds added**: Packing is a form of relocation within background compaction (D27 (small-data packing container) settled items 1, 6), following the intent mechanism and region scan landed in Milestone 4 GC, adding no `CommitStep` members; remaining a single transaction layer (`.claude/rules/fs-design.md` "one transaction layer, shared across all structures").

**Boundary with write-path packing**: Stuffing several files into one unit or inlining into index nodes on the write path was judged negative by D14 (Dual-Track (Large/Small Files / Persistent/Temporary)) settled item 3 (criterion being atomicity: multiple files sharing an atomic unit); this document does not touch that "no".

## Entry (Refined 2026-10-17, main agent draft)

Divided into two tiers: Stage 2 (Implementation) does not start until all hard entry criteria are satisfied; action items satisfying soft entry criteria may start early, restricted to those not modifying `crates/`.

### Hard Entry

| # | Entry | How Satisfaction Is Judged | Status Checked 2026-10-17 |
|---|---|---|---|
| En.1 | All five exit items of Milestone 3 achieved | [03-third-txn.md](03-third-txn.md) exit table has completion records for all five rows | Unsatisfied: grouping table Stage 3 items I.1, VI.2 not started, Stage 4 not started; C147 (invariant for tree ID watermark not yet present), C395 (generation fields guarded only by acceptance assertions), C446 (unimplemented invariants lack prerequisite registration and expiration) still owed |
| En.2 | Milestone 4 exits 1, 2, 3, 5, 6, 7 achieved (Exit 4 long-term aging moved to Milestone 9) | Respective rows in [04-fourth-txn.md](04-fourth-txn.md) exit table | Unsatisfied: code in worktree `~/.local/share/singlefs/work/m4-predev` (branch `m4-predev`); truncation still missing a batch of implementations (`records/2026-10-13-里程碑四预开发.md` "IV. Truncation (Item 6)" truncation #9); verification stage incomplete; no file starting with `04-` under `.claude/kb/layout/`; crash stream full domain not run |
| En.3 | Milestone 4 changes merged into master and committed | `git log master` contains Milestone 4 commits; m4-predev worktree has no uncommitted Milestone 4 changes | Unsatisfied: m4-predev worktree `git status --short` shows 285 entries, all uncommitted |
| En.4 | Every row in "Compaction Interfaces Delivered by Milestone 4" table exists on master | Stage 0 checked row-by-row on master | Unsatisfied: present only in m4-predev worktree |
| En.5 | D2 (RAID Stripe Policy) settled item 6 `w ≥ 2` implemented | Progress row of `.claude/kb/decisions/02-RAID条带策略.md` settled item 6 | Satisfied: that row writes "**Progress: Implemented.**" |

Long-term aging of E10 (Aging: Structural Aging Resistance vs Post-Hoc Compaction) is not an entry gate: user settled 2026-10-17 that together with original Exit 4 of Milestone 4 it is moved to Milestone 9 ([09-ninth-txn.md](09-ninth-txn.md) "V. Long-Term Aging"; original quote: "Does not count; real device already moved to Milestone 9, if Milestone 4 has residue move together"). IV.1 and V.2 do not wait for it, measuring on unaged disks.

### Soft Entry

| Action Item | What Is Required to Start | Status Checked 2026-10-17 |
|---|---|---|
| I.1 Field table | None | Can start |
| I.2 smallfile three columns measurement | Write path on master; rerun once per measurement basis of D25 (Target Workload Prioritization) after Milestone 4 merge, then write values into D25 (Target Workload Prioritization) | Can start |
| I.3 Evidence | I.1, I.2 | Awaiting I.1, I.2 |
| I.4 Knob values | I.1 | Awaiting I.1 |
| II.1 Design round | I.1; compaction interfaces discussed based on m4-predev worktree, materials specifying tree basis, rechecked against master after Milestone 4 merge | Awaiting I.1 |

### Compaction Interfaces Delivered by Milestone 4

Checked in m4-predev worktree on 2026-10-17 (unmerged to master, not counting as settled):

| Interface | Placement | How Packing Uses It |
|---|---|---|
| Region scan | `scan_region` in `crates/singlefs-core/src/region_scan.rs` | III.2 discovers completely dead containers |
| Segment selection and relocation plan | `choose_region`, `plan_batch`, `CompactionSwitches` in `crates/singlefs-core/src/compaction_plan.rs` | Compaction selects by segment, packing proceeds along central mapping key order, one mapping leaf per transaction (D27 (small-data packing container) settled item 6); whether planning function is duplicated separately settled in II.1 |
| Relocation batch publish | `publish_relocation_batch` in `crates/singlefs-core/src/transaction.rs` | III.1 publish of a batch of packing |
| Compaction entry | `run_compaction` in `crates/singlefs-core/src/mounted_session.rs` | Where packing loop hooks, how rate limiting connects, settled in II.1 |
| Intent carrier | `crates/singlefs-core/src/logged_operations.rs`: kind code 1 compaction, 2 truncation, readers reject other codes | Whether packing reuses code 1 or introduces new code settled in II.1 |
| Verus verification | `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`: P1 segment selection soundness, P2 plan soundness, P3 progress, P4 abstract identity lemma | Whether packing identity in Exit 6 borrows P4 settled in II.1 |
| Unit class code | `crates/singlefs-core/src/unit.rs`: `UNIT_CLASS_INDEX_NODE` = 2, `UNIT_CLASS_PACKED` = 3; 4–15 have no constants | II.1 picks one code |
| incompat bit | `crates/singlefs-core/src/system_configuration.rs`: bit 0 (retired), bit 1 in use; verdict `research/prompts/m4-trunc-r2-main-verification.md` assigned bit 2 for truncation, code not yet written | II.2 takes next free bit, checked upon kickoff |

## Exit (Refined 2026-10-17, main agent draft, subject to user settlement)

Nine items, all must be achieved to complete, with acceptance for each item listed in respective action item table:

| # | Exit | Attributed Item |
|---|---|---|
| 1 | Prerequisite three debts cleared: C361 (header width and slot width of new packing container class lack field table), C174 (three columns in smallfile row unmeasured), C391 (small-data packing container owes D25 proof); if I.3 evidence fails, halt at I.3, whether to proceed with packing left to user | I |
| 2 | Knobs quantified: C202 (idle overhead and hysteresis K of packing patrol lack values), C105 (write amplification of packing container rewritten by publish frequency), C106 (upper bound on open container count) each has a counting experiment completed, values written into D27 (small-data packing container) body | I |
| 3 | New unit class landed: occupies one code in unit class registry 4–15 (D18 (information carried in a block) settled item 11); incompat bit registered in D15 (format freeze policy) settled item 4 table and `.claude/kb/feature-bits.md`; validity invariants of new class implemented in pool-level checker and GPU judge, differential item-by-item identical (those unable to run on GPU registered in non-assertion table with owed check); each invariant has one bad image, `crates/mutations.tsv` one row; on-disk format registered in file starting with `05-` under `.claude/kb/layout/`; C154 (stripe member table lacks registration of packed record type) cleared or rewritten | II |
| 4 | Packing and reclamation landed: compactor moves objects ≤ 4 KiB into containers along central mapping key order (one transaction per mapping leaf, one container per size class, hysteresis by birth generation, continuous patrol with rate limiting), completely dead containers reclaimed by region scan with discard issued; slot constantly COW, world modifying already-written slot in-place must turn pool-level checker red (D27 (small-data packing container) settled item 10); containers with only partial objects deleted not released, remaining slots readable as normal | III |
| 5 | Interoperability with Milestone 4 landed features each has one test case and one must-red world: scan reconstruction (Milestone 4 Exit 6) recognizes container slots; objects referenced by snapshots when packed take shared block migration branch, old placement entering deadlist; truncation encountering objects inside container does not modify slot in place; rollback crossing packing batch leaves mapping not pointing to released placements | III |
| 6 | Abstract identity: model differential with packing compared against RefFS, results item-by-item identical to non-packed; random history incorporates packing operations; Verus proves packing plan sound, extending P4 to placements inside container slots (user settled 2026-10-17 into exit criteria) | III |
| 7 | Four crash streams connected to crash amplification and judged by GPU: packing, reclamation, packing interleaved with snapshot, packing interleaved with truncation; registered in `crash-case:` rows of `.claude/gate.d/stage-inputs.tsv` and `.claude/gate.d/crash-point-replay-items.tsv`, full domain 0 red; core part of light tier adds packing item | III |
| 8 | Ledgers match: space benefit, write byte ratio, loss radius measured on smallfile workload, matching closed forms of E114 (Ledger of Small-Data Packing Containers), E116 (Packing Container Ledger: Metadata Writes and Compaction Strategy), E149 (Costs of Three Repair Strategies for Damaged Packing Containers); discrepancies investigated on both sides | IV |
| 9 | Write amplification evaluation (user added 2026-10-17: "After completion conduct a wa evaluation to see how data looks"): after first eight items complete, measure write amplification of this version per measurement basis in "V. Write Amplification Evaluation", packing on and off one pass each, compared against Milestone 4 closeout version and compared with six systems using E152 (per-milestone file performance against six file systems) apparatus; observations only, data presented to user, modifications through three-party review | V |

## Grouping and Sequence

| Stage | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Evaluate hard entry criteria in "Entry" section row by row; inspect "Compaction Interfaces Delivered by Milestone 4" table row by row on master; settle exit criteria | Soft entry items (I.1, I.2, I.4) do not wait for hard entry, can be done during Milestone 4 closeout wait | Not started |
| 1 Debt Clearance and Design | I.1 ∥ I.2 → I.3 ∥ I.4 → II.1 (design round for new class field table, invariants, and packing plan) | Cap and benefits can only be recomputed once field table is settled; format opens only after evidence passes | Not started |
| 2 Implementation | II.2 (new class, checker, GPU judge, layout) → III.1 (packing) ∥ III.2 (reclamation) → III.4 (interoperability) ∥ III.5 (identity) | Packing and reclamation each attach to compactor, intersecting at compaction scan branch; interoperability requires packing landed first | Not started |
| 3 Coverage | III.3 four streams run full domain on crash amplification; IV.1 measures overall ledger | Exits 7, 8 satisfied in this stage | Not started |
| 4 Evaluation | V.1 → V.2 ∥ V.3 | Measured only after first eight items complete, measuring completed version; Exit 9 satisfied in this stage | Not started |

**Estimates**: None estimated.

## I. Debt Clearance: Field Table, Denominator, Evidence

- **Original words**: "I think Milestone 5 needs small file packing".
- **Existing in repository**: D27 (small-data packing container) settled items 1, 2 specify cap 7, benefit 6.9×, write byte ratio 1.143 calculated under geometry of "container header 107, each slot 47, w = 2", while header width and slot width of new class lack field table (C361 (header width and slot width of new packing container class lack field table)); D25 (Target Workload Prioritization) `smallfile small-file intensive` row leaves leaf/fsync, spine, and write amplification per fsync all "unmeasured" (C174 (three columns in smallfile row unmeasured)); D25 (Target Workload Prioritization) states "permanent format branch serving secondary workload requires separate justification per branch criteria of `.claude/rules/fs-design.md`", five hard requirements also in evidence checklist (C391 (small-data packing container owes D25 proof)); additionally C105 (write amplification of packing container rewritten by publish frequency), C106 (upper bound on open container count), C202 (idle overhead and hysteresis K of packing patrol lack values), C186 (arm added mid-run falls outside pre-run failure clause scope), C240 (uncomputed cost of re-encryption during relocation) (path exists only when encryption enabled).
- **Action items**:

| # | What to Do | Underlying Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I.1 | Field table: container header and slot field table for new class (class identity segment, slot width `W`, slot count, central mapping key five-tuple per slot, container-wide MAC and AAD), cap and benefit recalculated | D27 (small-data packing container) settled items 3, 7, 8, 9, 12; C361 (header width and slot width of new packing container class lack field table) | Field table written into D27 (small-data packing container) body and file starting with `.claude/kb/layout/05-`; E114 (Ledger of Small-Data Packing Containers) rerun per new width, cap and benefit updated to current values | None |
| I.2 | Denominator: three columns measured for smallfile row (this implementation, local machine, measurement basis per D25 (Target Workload Prioritization) table) | C174 (three columns in smallfile row unmeasured) | Three columns updated from "unmeasured" to numbers, with artifacts | None |
| I.3 | Evidence: five hard requirements of `.claude/rules/fs-design.md` evaluated one by one (explicit switching criterion, each branch testable via forced entry, each branch verified independently including crash-point replay, branch observable, branch variables immutable during operation), drafted into three-party round | C391 (small-data packing container owes D25 proof); D25 (Target Workload Prioritization) | Verdict placed in `research/prompts/`, if evidence fails halt at this item, submit to user | I.1, I.2 |
| I.4 | Knob values: hysteresis `K`, patrol rate limiting, idle overhead; open container upper bound and write amplification rewritten by publish frequency | C202 (idle overhead and hysteresis K of packing patrol lack values), C105 (write amplification of packing container rewritten by publish frequency), C106 (upper bound on open container count) | One counting experiment each, pre-run registration hardcoding criteria | I.1 |

- **To settle before kickoff**: Whether to proceed with packing if I.3 evidence fails (user decides); whether C240 (uncomputed cost of re-encryption during relocation) must be calculated prior to encryption (inferred: encryption not in first runnable version, can be deferred).

## II. New Unit Class

- **Existing in repository**: D27 (small-data packing container) settled item 3 assigns new code from unit class registry 4–15 (content class), not reusing code 3, with new class establishing its own validity invariant rather than applying I-1.7 (validity and verdict ordering of packed containers); D18 (information carried in a block) settled item 11 "new unit class ⇒ incompat"; C154 (stripe member table lacks registration of packed record type) notes second-level registry of code 3 already has unregistered occupancies.
- **Action items**:

| # | What to Do | Underlying Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II.1 | Design round (three-party): which code to occupy, class identity segment fields, validity invariant phrasing, release judgment via central mapping inspecting slot by slot to claim slots | D27 (small-data packing container) settled items 3, 4, 5, 10; D19 (Block Pointer Structure & Width Budget) settled items 5, 6 | Verdict placed in `research/prompts/`, settled items written back to kb | I.1 |
| II.2 | Implementation: unit class registration, incompat bit, container header and slot read/write, new invariant into pool-level checker and GPU judge, bad image one each, `crates/mutations.tsv` one row each | II.1 settlement; D15 (format freeze policy) settled item 4 | Invariant reddens only itself; GPU and CPU differential identical item by item; feature bit registered consistently in two locations (code-source-discipline feature-bits cell) | II.1 |

- **To settle before kickoff**: Which code between 4–15 to occupy (settled in II.1).

## III. Packing and Reclamation

- **Existing in repository**: D27 (small-data packing container) settled item 6 specifies five rules (along central mapping key order, one transaction per mapping leaf, one container per size class, hysteresis by birth generation, continuous patrol with rate limiting); D27 (small-data packing container) settled items 4, 5 establish no container index and no live slot counting, completely dead containers discovered via region scan of D26 (background compaction and placement reclamation) settled item 2; D27 (small-data packing container) settled item 10 mandates slot constantly COW, violation means abandoning entire D27 (small-data packing container); D27 (small-data packing container) settled item 13 specifies first version size class table of 36 classes is policy, not format. Compactor and intent mechanism landed in Milestone 4: as of 2026-10-17 present only in m4-predev worktree, placements listed in "Compaction Interfaces Delivered by Milestone 4" table; does not count before merging to master, recheck on master prior to kickoff.
- **Action items**:

| # | What to Do | Underlying Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III.1 | Packing: compactor scans objects ≤ 4 KiB along central mapping key order, packs into containers by size class, one transaction per mapping leaf, position entry updated to point to container placement; old units released per existing release rules after object migration | D27 (small-data packing container) settled items 1, 2, 6, 8, 11, 12, 13 | Fixed script: write N small files → one compaction round → read-back byte-identical → space consumption drops per cap; each new test proves red beforehand | II.2 |
| III.2 | Reclamation: region scan recognizes completely dead containers and releases them; live slots uncounted | D27 (small-data packing container) settled items 4, 5; D26 (background compaction and placement reclamation) settled item 2 | Delete all objects in container → compaction scans region → container released; delete partial objects → container not released, remaining slots readable | III.1 |
| III.3 | Crash streams: four streams (packing, reclamation, packing interleaved with snapshot, packing interleaved with truncation) connected to crash amplification (GPU judged), registered in `crash-case:` rows of `.claude/gate.d/stage-inputs.tsv` and `.claude/gate.d/crash-point-replay-items.tsv`; core part of light tier adds packing item | D13 (verification route) settled items 9, 15 | Four streams full domain 0 red; each stream has one must-red world | III.1, III.2, III.4 |
| III.4 | Interoperability: scan reconstruction recognizes container slots; snapshot-referenced objects when packed take shared block migration branch, old placement entering deadlist; truncation encountering containerized objects does not modify slot in place; rollback crossing packing batch leaves mapping not pointing to released placements | D27 (small-data packing container) settled item 10; Milestone 4 Exits 1, 2, 6, 7; C142 (compaction after first reflink creates silent stale read) | Four cells each have one test case and one must-red world (e.g. modifying slot in place during truncation turns pool-level checker red) | III.1, III.2 |
| III.5 | Identity: model differential compared against RefFS adds packing operations, random history interleaves packing, results identical item by item with non-packed; Verus proves packing plan sound, extending P4 to container slots | D13 (verification route); P4 of `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs` | Differential has seed hit; world必然 producing counterexample (packing modifying a byte) | III.1 |

- **To settle before kickoff**: Whether Milestone 6 parallel commit modifies acquisition and interleaving rules of background transactions—when Milestone 6 item I.9 reopens D8 (core index structure) settled item 16 item 6, packing branch is modified alongside GC, not pre-committed in this document.

## IV. Overall Ledger Measurement

| # | What to Do | Underlying Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| IV.1 | Measure space benefit, write byte ratio, and peak occupancy prior to compaction completion on smallfile workload, compared against closed forms of E114 (Ledger of Small-Data Packing Containers), E116 (Packing Container Ledger: Metadata Writes and Compaction Strategy), E149 (Costs of Three Repair Strategies for Damaged Packing Containers) | Cost sentences in D27 (small-data packing container) settled items 1, 2 | Matching writes back to experiment page; discrepancies investigated on both sides, neither default apparatus nor implementation assumed correct (`.claude/rules/implementation-first.md` rule 4) | III.1, III.2, I.2 |

## V. Write Amplification Evaluation

- **Original words**: "Milestone 5 also needs an addition. After completion conduct a wa evaluation to see how data looks" (2026-10-17).
- **Existing in repository**: D25 (Target Workload Prioritization) workload table has "write amplification per fsync" column (blocks / user blocks), smallfile row measured by I.2; write byte ratio 1.143 of D27 (small-data packing container) settled items 1, 2 is closed form calculated by E114 (Ledger of Small-Data Packing Containers) with container header 107, slot 47, w = 2; E152 (per-milestone file performance against six file systems) is apparatus comparing against six systems by milestone, table generated by `scripts/experiments/e152-tables.py`, summarized in `scripts/experiments/perf-by-milestone.md`; Milestone 9 item I.1 breaks down write amplification per fsync by blocks, bytes, and barriers, reusing apparatus built here ([09-ninth-txn.md](09-ninth-txn.md) "I. Write Amplification Measurement"). C26 (barrier count never measured) remains owed.
- **Action items**:

| # | What to Do | Underlying Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| V.1 | Pre-run registration: measurement basis for write amplification (foreground: units, journal records, root slots, system configuration slots written per fsync in blocks, bytes, barrier counts; background: bytes written by packing and reclamation migration; total: device-side written bytes ÷ user written bytes over fixed window), workloads taken from four rows of D25 (Target Workload Prioritization) table, packing on and off one arm each; measurement basis shared with Milestone 9 I.1, not duplicated there | D25 (Target Workload Prioritization); C26 (barrier count never measured); `.claude/singlefs-ai-sop/rules/test-discipline.md` "experiments must state which decision they measure for and how much measurement suffices" | Pre-run registration completed; counting apparatus includes closed-form comparison, assertions pinning absolute values, and mutation table; QEMU device-side recording matches program counting item by item | Exits 1 to 8 |
| V.2 | Measurement: four workloads, packing on and off, measuring all quantities in V.1; side-by-side with Milestone 4 closeout version and closed-form write byte ratios of E114 (Ledger of Small-Data Packing Containers), E116 (Packing Container Ledger: Metadata Writes and Compaction Strategy) | Cost sentences in D27 (small-data packing container) settled items 1, 2 | Artifacts deposited in repository; discrepancies against closed forms investigated on both sides (`.claude/rules/implementation-first.md` rule 4) | V.1 |
| V.3 | Comparison against six systems: run singlefs arm of this version using E152 (per-milestone file performance against six file systems) apparatus, add milestone "small file packing" section to `scripts/experiments/perf-by-milestone.md` | None | Table generated from artifacts by `scripts/experiments/e152-tables.py`, not hand-copied | V.1 |

- **To settle before kickoff**: Length of "fixed window", which fsync denominator background migration writes are attributed to (hardcoded in V.1 pre-run registration). Evaluation produces observations only; what to change after viewing data opens separate inference rounds per `.claude/rules/three-way-inference.md`, or merges into Milestone 9 "III. Convergence".

**Not in exit**: Concurrency and parallel commit (Milestone 6 [06-sixth-txn.md](06-sixth-txn.md)), directories and POSIX surface (Milestone 11; FUSE mount in Milestone 12), cross-file packing and inlining on write path (judged negative in D14 (Dual-Track (Large/Small Files / Persistent/Temporary)) settled item 3), re-encryption cost when encryption enabled (C240 (uncomputed cost of re-encryption during relocation)), writable clones, compression, third-party pipelines, zoned.
