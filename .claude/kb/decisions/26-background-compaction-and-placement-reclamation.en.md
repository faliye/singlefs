## D26 (Background Compaction and Placement Reclamation) —— Semi-settled (Three Unsettled Items)

[中文](26-后台整理与放置回收.md) · **English** · [日本語](26-バックグラウンド整理と配置回収.ja.md)

D26 (Background Compaction and Placement Reclamation) governs who resolves fragmentation accumulated over long-term operation: what the two legs are, how compaction starts and stops, which transaction form it follows, whether it requests space, how it interacts with accounting, where observability metrics reside, and how reclamation branches into two lines. What it does not govern: allocation placement policy is in D3 (Space Allocation) Settled Items 5 / 8; acceptance inequality is in D28 (Mount-Time Commitments) Settled Item 1; rebalance and relocation unit measurement basis is in D2 (RAID Stripe Policy) Settled Items 8 / 11; destroying snapshots and destroying writable heads are in D6 (Snapshot Implementation Model); index node internal layout belongs to D15 (Format Freeze Policy) Layer 4; specific threshold values are not here, and the values of the three start watermarks do not come from E10 (Aging: Structural Resistance vs Post-hoc Compaction) either.

### Settled Items

**D26 (Background Compaction and Placement Reclamation) items have a single numbering scheme**, continuously numbered with the "Unsettled Items" section, without reordering.

| # | Item | Ruling |
|---|---|---|
| 1 | **Trigger criteria for compaction** | Starts whenever any of three watermarks is breached; stopping uses `R`-local predicate "at the moment intention is deleted, `R` is completely empty by record", checks whether holes suffice prior to issuance, and stops issuing the next one if there is no yield; `R` cannot serve as allocation destination during intention lifetime **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Transactional form of compaction** | Does not add `CommitStep` variants; intention target state written as "no unreleased placements with allocation generation ≤ `c0` exist within `R`" **Status: Settled.** **Progress: Unimplemented.** |
| 3 | **Whether compaction itself requests space** | Follows regular acceptance; retains a resident floor quota from version 1, implemented as the second conjunct of acceptance; floor quota takes "as small as possible" **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Form of structural anti-aging** | Does not enter format: does not store group identity, does not touch index node internal layout; group is calculated solely from key; policy half falls back to D3 (Space Allocation) Settled Item 8 and Settled Item 1 **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **Interaction between compaction and accounting** | Relocation does not change `birth`; allocation record tree has one formula each for source and destination sides; deadlist entries recorded by central mapping key **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Runtime observability points for compaction invocation** | Six observables, three used for triggering persisted into accounting tree, compaction progress not stored (full verification against target state across entire `R` before intention deletion), avoiding blind execution or full scans post-mount **Status: Settled.** **Progress: Unimplemented.** |
| 7 | **Branching of reclamation lines** | Split into two lines: first build self-hosted GC; when testing reveals disk FTL supports GC, offload to disk while SingleFS only tags and issues commands; which line to take is governed by feature bit **Status: Settled.** **Progress: Unimplemented.** |
| 11 | **Where the answer to fragmentation rests** | Two legs superimposed, not either-or; NVMe FDP / ZNS demoted to "use-if-present acceleration", no longer the answer **Status: Settled.** **Progress: No code object.** |
| 12 | **Form of Leg 1** | Boxed in by three settled disciplines: Slot 3 has no commandments, unbounded heavy work follows intention chunking, relocation unit is whole object re-placed as a group **Status: Settled.** **Progress: No code object.** |
| 13 | **Where background task computation sits, how it is enabled, and its cadence** | Defaults to CPU; one set of mount parameters governs all background tasks: enabling GPU sends every task with a GPU path to GPU, can designate GPU cards, set a combined VRAM ceiling, switchable while mounted; results compared across two kernels; GPU error fails without fallback; checks watermarks post-publish plus periodic fallback timer; runs within write bandwidth budget, yielding to foreground **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Trigger Criteria for Compaction

**Ruling**: Three rules.

1. **Start: Three watermarks, breaching any one starts execution.** Completely empty cluster segment count (physical metric, per-device dimension), fragmentation metric `F = runs / L` (runs current measurement basis is count of contiguous runs of free slots in unit area, physical metric, per-device dimension, D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 Item 10), and unreclaimable bytes (per-device dimension). Completely empty cluster segment count follows record measurement basis (all 64 slots in segment have no unreleased allocation records); code `empty_segments` is already in record measurement basis (the code half of C372 (two definitions of the fully empty cluster segment count are opposite) is done), same sentence as D3 (Space Allocation) Settled Item 10 ①. All three metrics are settled by Settled Item 6 as persistent, readable upon mount.
2. **Stop: At the moment intention is deleted, `R` is completely empty by record** (`R`-local predicate); prior to issuance, checks whether hole count in non-empty segments outside `R` (segments storing only commit-endogenous blocks do not count as non-empty) ≥ number of units to relocate in `R`, not issuing if insufficient; if an intention completes but global difference `completely empty cluster segment count(now) − segs0 ≥ 1` is false, the next one is not issued; `segs0` is still written as value into intention record, serving only as observable.
3. **`R` cannot serve as allocation destination during intention lifetime** (first rule of fallback policy). Cost is adding one conjunct to allocation path: "candidate placement is not within `R` of any active compaction intention"; active intention count is bounded (compaction advances one batch at a time) ⇒ this is an O(active intention count) point lookup, not a traversal; intention itself resides in logged_ops tree (D8 (Core Index Structure) Settled Item 5), readable by allocator.

**Threshold values do not enter format**: the values of the three start watermarks do not come from E10 (Aging: Structural Resistance vs Post-hoc Compaction) (its pre-registration `research/prompts/e10-r1-prereg.md` writes the watermarks as a choice made by the registration, not as something measured); adopt conservative defaults made mount-tunable. Changes new-pool file creation bytes: No — the three metrics are settled by Settled Item 6 as statistics in accounting tree; new-pool file creation already writes them; start and stop are judgment forms, `segs0` resides in intention record while new-pool file creation contains no compaction intentions.

**Scope**: Governs start and stop forms, and mutual exclusion of `R`; does not govern specific threshold values. Whether yield exists after intention fulfillment is judged by Rule 2 `R`-local predicate; global difference only decides whether to issue the next one; checker "completely empty segment count strictly increases after every N completed intentions" is owed under C140 (Intention completion does not equal empty segment freed). Where allocation falls when no empty segments exist outside `R` still has no repository clause. State "`R` cannot serve as allocation destination" has no standalone placement on disk, inferred by reading active intentions — isomorphic to C127 (pinned state has no placement in allocation records), but this rule has input (intention record on disk). Marginal return (`Δfragmentation ÷ Δbytes written`) stopping form is unusable: numerator using global `runs` has wrong sign across entire experimental trajectory; denominator using "bytes already written by this compaction batch" is a function of history, non-derivable from on-disk state; writing it into intention record means "declaring increment", hitting D8 (Core Index Structure) Settled Item 5 Hard Requirement 1 verbatim: "Intentions declare target state, never declare increments".

**Evidence**:

- E93 (Placement and Fragmentation Under Aging): Fragmentation maintainable incrementally in O(1); cluster segment policy is non-self-sustaining without compaction feeding empty segments; highest-budget compaction arm has monotonically increasing `runs` trajectory — numerator of marginal return stopping criteria thus has wrong sign at every sample point.
- Three-way argument Round 3 (unmodified output `research/prompts/d26-r3-{sonnet,opus,local}-output.md`): Stopping criteria changes form; fallback policy must add one rule.
- User settlement (three watermarks, yield predicate, `R` mutual exclusion), original wording in change history.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G4: Stopping changed to `R`-local predicate, checks holes pre-issuance, stops issuing next if no yield (on attack model, pre-issuance check yields zero geometric livelocks for four cases, stop-on-no-yield yields zero geometric livelocks for all); C372 (two definitions of the fully empty cluster segment count are opposite) settled to record measurement basis. Section 7 user settlement "reopen: R-local predicate + pre-issuance check + stop on no yield", original wording in change history.

**Owed checks**: C140 (Intention completion does not equal empty segment freed); C146 (fallback policy with no empty segment undefined repo-wide); C243 (Quantity in Termination Predicate Lacks Trajectory); C364 (compaction intent record fields lack field-by-field reconciliation); C372 (two definitions of the fully empty cluster segment count are opposite); C373 (No clause on whether active intention is deleted when stopping predicate evaluates true).

#### Settled Item 2: Transactional Form of Compaction

**Ruling**: Two halves.

| Half | Ruling |
|---|---|
| Commit steps | **Does not add `CommitStep` variants**. Compaction is normal transaction sequence, unbounded work follows D8 (Core Index Structure) Settled Item 5 intention mechanism — that rule verbatim governs "any operation that **may modify unbounded extent count**... writes intention into logged_ops tree first, then advances in batches"; compaction falls within scope |
| Intention target state | **"No unreleased placements with allocation generation ≤ `c0` exist within `R`"**, `c0` = allocation generation at moment of writing this intention. **Does not write "empty region `R`"** |

Three properties making this predicate viable: **Monotonic** — new placements allocated in `R` by foreground necessarily have allocation generation > `c0` ⇒ does not violate target state ⇒ foreground writes no longer block intention completion; **Computable** — both inputs reside in D3 (Space Allocation) Settled Item 7 allocation record entries (value is "allocation generation or release generation 8 + span segments 2", highest bit of span segments already borrowed by "released"); **Zero format bits** — does not add any on-disk fields, `c0` resides in intention record. Changes new-pool file creation bytes: No — new-pool file creation contains no compaction steps; adding variants to enum or writing intention predicates does not alter written bytes.

**logged_ops kind registry and common encoding**: the first 2 key bytes encode kind, followed by 8 bytes of issue txg, little-endian; every entry is 44 bytes (key 10, value 34). Reject 0 and unregistered codes; parse registered kinds using their own field tables.

| Kind code | Intention | Class | Field specification |
|---|---|---|---|
| 1 | Compaction | Advisory | Compaction field table in D26 (Background Compaction and Placement Reclamation) settled item 2 |
| 2 | Truncation | Commitment | Truncation field table in D8 (Core Index Structure) settled item 5 |

**Field table for compaction intention entry** (logged_ops tree entry, serialization in `crates/singlefs-core/src/logged_operations.rs` `to_bytes` and `parse`, little-endian):

| Segment | Field | Width | Offset | Values |
|---|---|---|---|---|
| key | kind | 2 | 0 | Constant 1 = compaction; parse code 2 using the truncation field table; reject 0 and unregistered codes |
| key | issuance txg | 8 | 2 | txg of publication that issued this intention |
| value | `R` device | 4 | 10 | Disk where `R` resides |
| value | `R` start slot | 6 | 14 | Multiple of 64, unaligned rejected |
| value | `R` slot count | 2 | 20 | Constant 64 (one cluster segment), other values rejected |
| value | `c0` | 8 | 22 | Allocation generation at issuance moment |
| value | `segs0` | 8 | 30 | Value for the disk containing `R` in accounting Item 11 (completely empty cluster segment count, D5 (Snapshot / Space Accounting Mechanism) settled item 4) before the issuing publication, observable only; not the new accounting value written by that publication |
| value | zero padding | 6 | 38 | Non-zero rejected |

Total 44 bytes: key 10, value 34 fixed width, Code 2 node holds 369 entries; width not equal to 44 uniformly rejected.

**Sampling boundary**: if issuance opens a new segment, the pre-issue `segs0` can exceed the new accounting count by 1; emptying just one segment then offsets that opening and is not net yield. Follow H3 in `research/prompts/m4-segment-code-r2-main-verification.md` (user decision to sample before issuance); code and cases are in `crates/singlefs-core/src/transaction.rs` and `crates/singlefs-harness/tests/compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs`.
<!-- format-const: LOGGED_OPERATIONS_KEY_BYTES = 10 -->
<!-- format-const: LOGGED_OPERATIONS_VALUE_BYTES = 34 -->
<!-- format-const: LOGGED_OPERATIONS_ENTRY_BYTES = 44 -->
<!-- format-const: LOGGED_OPERATIONS_ENTRIES_PER_NODE = 369 -->

**Scope**: Governs how target state is written and whether commit steps are added. Does not govern "whether fulfillment equals effectiveness" — histories moving back and forth between two cluster segments where every intention completes normally while empty segment supply remains zero belong to Settled Item 1 stopping and yield criteria. One compaction batch rewriting K+1 published roots represents a commit point **combination** not enumerated by crash point sampling, belonging to crash harness. Reason "empty region `R`" cannot be written is non-monotonicity: no clause in entire repository states `R` is unallocatable during intention lifetime (Settled Item 1 Rule 3 added precisely for this), foreground writes could refill `R` ⇒ target state never achieved, intention never deleted.

**Evidence**:

- E94 (Touch Surface of Moving Shared Placement Points): All three location authoritative architectures write only tree nodes, root records, and existing entries, confirming one compaction batch rewrites K+1 published roots.
- Three-way argument Rounds 2 and 3 (unmodified output `research/prompts/d26-r2-{sonnet,opus,local}-output.md`, `research/prompts/d26-r3-{sonnet,opus,local}-output.md`): Attack leg and counterexample leg independently hit non-monotonicity of "empty region `R`".
- User settlement, original wording in change history.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G1: logged_ops single tree, fixed-width union entry, version 1 holds only compaction intentions, field table fixed by it.

**Owed checks**: C91 (redo drift between compaction intent and incremental trigger amount); C140 (Intention completion does not equal empty segment freed); C364 (compaction intent record fields lack field-by-field reconciliation).

#### Settled Item 3: Whether Compaction Itself Requests Space

**Ruling**: Three sentences.

1. **"Releasing space itself requires no space request" holds as an unmodified requirement**, D3 (Space Allocation) principle is **not reopened** — reopening it would dismantle the discipline that specifically targeted btrfs root causes at project initiation. What fails is statement "D3 (Space Allocation) Settled Item 2 + Settled Item 4 **has already fulfilled it**"; what needs reopening is D3 (Space Allocation) Settled Item 2 algorithm for "how much space is needed to unbind self".
2. **Compaction is not release path, it follows regular acceptance**, with batch worst-case occupancy counted into "committed reservation".
3. **Retains a resident floor quota from version 1**, shaped as acceptance **second conjunct** `(available ≥ demand) ∧ (completely empty cluster segment count ≥ N₀)`. **Floor quota cannot enter any term of inequality** — it constrains **geometry**, not scalar. `N₀` takes **minimum value preventing ratchet divergence**, criteria being "smaller is better", not "sufficient is enough" (rationale: D3 (Space Allocation) Settled Item 13 does not segregate data / metadata space, unified global allocator ⇒ "release requesting space" is not as fatal here as in btrfs), numerical value awaiting E10 (Aging: Structural Resistance vs Post-hoc Compaction).

Changes new-pool file creation bytes: No — completely empty cluster segment count settled by Settled Item 6 as statistic in accounting tree; new-pool file creation already writes it; adding a conjunct is a form of acceptance judgment, not altering written bytes.

**Scope**: Governs whether compaction path enters regular acceptance, whether floor quota is retained and its shape; does not govern value of `N₀`. Which term of acceptance inequality accounts for space occupied by floor quota itself is unsettled — under D28 (Mount-Time Commitments) Settled Item 1, subtracting it from available like "committed reservation" double-counts against compaction batch reservation; this slot owes an item. This rule is not free: it drags "completely empty cluster segment count" from `.claude/rules/fs-design.md` Slot 3 (background, no commandments) into **Slot 1** (runtime decision path, requiring immediate answers when queried), consequently owing three things: in-flight overlay (shaped after D28 (Mount-Time Commitments) Settled Item 2), an I-3.1 (Allocated Statistics Match) shaped invariant (incremental reading matches checker full scan), and per-device dimension (Settled Item 6 settled completely empty cluster segment count as physical metric with per-device dimension, while acceptance inequality sums per-device).

**Evidence**:

- E93 (Placement and Fragmentation Under Aging): Cluster segment policy is non-self-sustaining without compaction feeding empty segments, compaction budget is sole force suppressing fragmentation — compaction stall ⇒ empty segment supply halts ⇒ commit-endogenous blocks scatter ⇒ by D3 (Space Allocation) Settled Item 5 evidence, I-8.1 (Ring Geometry Sufficiently Large) worst-case transaction footprint has no usable upper bound under full scattering policy ⇒ release transaction itself cannot issue. This chain is independent of `w`, holding from version 1.
- Three-way argument Round 3 (unmodified output `research/prompts/d26-r3-{sonnet,opus,local}-output.md`): Three legs independently hit same point in two places, retargeting from "reopen D3 (Space Allocation) principle" to "reopen D3 (Space Allocation) Settled Item 2 algorithm".
- User ruling (follows regular acceptance, version 1 retains floor quota, floor quota smaller is better), original wording in change history.

**Owed checks**: C139 (release cannot be issued after compaction halts); C90 (ENOSPC ratchet under pinned stripes); C906 (compaction batches skip ordinary admission, at odds with the clause).

#### Settled Item 4: Form of Structural Anti-Aging

**Ruling**: Format part — **does not store group identity in node header, does not touch index node internal layout**; placement grouping calculated solely from key (key range grouping). Policy part — cluster segment placement, selectable rewrite radius `R`, and group relocation by key governed by D3 (Space Allocation) Settled Item 8 and Settled Item 1, no longer held separately here. Changes new-pool file creation bytes: No — index node bytes for new-pool file creation do not depend on this item; Code 2 header 2-byte reserved remains uniformly 0.

**Scope**: Governs whether registered two format arms are adopted, not "whether a third form exists": future proposals for anti-aging forms requiring on-disk fields establish separate items and experiments, without reopening this item. 1-byte aging generation path remains open at byte level, positioned in Code 2 class identity segment 2-byte reserved. Verdict sensitivity to geometry sample points unchecked.

**Evidence**:

- E127 (Whether to Store Group Identity Under Splits and Merges): In models with split-merge, node count grouping (storing group identity in node header) fails to suppress `runs` below 0.8× of key range grouping (no fields) across any of six equal-budget cells; after completing knob grid, loses across all six cells.
- E95 (Aging Node Layout Arm): Other registered format arm (generational separation, node header carries aging generation) achieves majority in zero of six cells by its own Criterion 5; its "format / policy" boundary is hardcoded label, flipping answer row when registering identical function as policy arm.
- E101 (Cost and Expressiveness of Reserved Bits in Node Header): Reserving 0 / 1 / 2 / 4 byte tag bits in node header drops cost in zero cells across four tiers, tree height unchanged; sole reason verdict cannot flip is unbounded node group count, and that reason vanished with E127 (Whether to Store Group Identity Under Splits and Merges) retirement of node groups.
- Round 5 three-way argument verdict `research/prompts/d26-r5-main-verification.md`: Three legs unanimous; backward leg self-built best implementation reaches only 0.998, disproving "siblings with same parent provided free by tree structure" (Code 2 header lacks parent pointer, derived tree header lacks key range, relocation via mapping does not modify referrers) ⇒ groups calculated solely from key.
- User settlement (lift "policy half not cut either", flip byte judgment, policy falls back to existing policy slots), original wording in change history.

**Owed checks**: C317 (Verdict measured on single geometry sample point unchecked); C102 (no reserved area in index node headers; adding fields is not compat).

#### Settled Item 5: Interaction Between Compaction and Accounting

**Ruling**: Answers to three questions.

1. **Relocation does not change `birth`** (authoritative under D5 (Snapshot / Space Accounting Mechanism) `birth(b)` definition table).
2. **Allocation record tree rewrite volume**: Source side `⌈N/fanout⌉` leaves + one internal node per level (`interior += leaves.min(level_nodes)`); destination side when scattered is `min(N, total leaves)`. Allocation record at destination writes unit birth generation as allocation generation (does not write txg of relocation publication); I-3.10 (Allocation Generation of Allocated Record Equals Birth Generation of Covered Unit) holds unmodified. Does not scale with disk capacity: source side contiguous at N = 4096 on 16 TiB is 13 nodes / 212992 bytes / 2.60×; 256× capacity increase reaches only 19 nodes, all 6 added nodes originating from tree height.
3. **Deadlist entries recorded by version identity** — key takes central mapping key (D19 (Block Pointer Structure & Width Budget) Settled Item 6, Code 1 27 bytes), not by placement, carrying no location hints, sharing identical key with D5 (Snapshot / Space Accounting Mechanism) Settled Item 6 sparse side table.

**Reading of region scan and segment selection** (three layers are `scan_region` → `choose_region` → `plan_batch`):

| Question | Reading |
|---|---|
| Region scan classification priority | Pinned → quarantined → read head → class tag → key and position; quarantined slots (checksum mismatch before release) uniformly judged quarantined without consulting mapping |
| Segment selection qualification | At least one movable unit in `R`, taken according to `scan_region` classification |
| Segments rejected across entire scan | Recorded into current mount ineligible segment set, excluded from further selection; whole-scan rejection stems from corrupted on-disk records, falling under C411 (Allocation record span segment mismatch with unit category uninspected); compaction does not repair it |
| Greedy tie-breaking | Take segment with smallest start slot number |
| `plan_batch` returning empty list | Return value distinguishes "fully cleared" vs "immovable" |

Changes new-pool file creation bytes: No — first runnable milestone lacks snapshots ⇒ deadlist entry count is 0, shared placements are 0; writing either answer across the three questions produces byte-for-byte identical output; adjacent mapping entry encoding evaluates "yes", categorized under D19 (Block Pointer Structure & Width Budget) Settled Item 6, outside this item.

**Scope**: Governs what relocation does to each of three accounting structures, and governs region scanning and segment selection reading (which units in `R` are movable, which segment selected as `R`); two halves parallel, not modifying each other. **Placement policy on destination side is not settled in Settled Item 5**: No clause guarantees contiguity; only modeled policy (lowest free slot first) guarantees non-contiguity in states where compaction executes, and Settled Item 1 Rule 3 excludes the segment most likely to hold contiguous space (`R`) from destinations. Three measurement basis notes: E74 (Allocation Record Entry Count and Compaction Cost) absolute values take placement granularity 32 KiB whereas D3 (Space Allocation) Settled Item 7 settled granularity 16384, and same item updated entry count measurement basis from "allocated placements" to "ever-allocated, not-yet-overwritten placements" ⇒ those numbers are dual legacy measurement basis; "370×" belongs exclusively to 16 TiB tier (4823 / 13 = 371); 1 TiB is 319×, 256 TiB is 432×; D3 (Space Allocation) Settled Item 3 text "that key places compaction on the good end" holds only for source side. `birth` is property of **logical version** ⇒ consumers using it as "how fresh this physical replica is" (D13 (Verification Route) Settled Item 8 generational incremental scrub) are uniformly blind to relocation.

**Evidence**:

- E94 (Touch Surface of Moving Shared Placement Points): Recording deadlist by central mapping key yields zero rewrites on relocation touch surface; recording by pointer rewrites 11–36 entries per batch; under pointer location authority, relocation batch cost scales with snapshot count, rewriting all snapshot roots per batch.
- E74 (Allocation Record Entry Count and Compaction Cost): Compaction cost does not scale with disk capacity; copy-on-write amplification depends entirely on placement clustering — source and destination formulas measured by it.
- 2026-09-08 three-round adversarial evaluation and 2026-09-10 Round 4 (unmodified output `research/prompts/d26-item5-r{1,2,3}-{sonnet,opus,local}-output.md`, `research/prompts/d26-item5-r4-{opus,sonnet,local}-output.md`): ① and ② concluded, ③ direction and blocker set by these rounds.
- User settlement concluded alongside D5 (Snapshot / Space Accounting Mechanism) Settled Item 6, original wording in change history.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G3: Destination allocation record allocation generation writes unit birth generation, I-3.10 (Allocation Generation of Allocated Record Equals Birth Generation of Covered Unit) unaltered by a single word.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G2, G9: Three layers and segment selection qualification "at least one movable unit in `R`" (attacker K-b), region scan classification.
- Verdict `research/prompts/m4-batch2-code-r1-main-verification.md` Section 2 S3 (classification priority, quarantined slots uniformly judged quarantined, decided by main agent), S4 (scan-rejected segments not reselected, `plan_batch` returning empty list distinguishes two cases, decided by main agent); greedy tie-breaking taking smallest start slot number is implementation writer reading for Milestone 4 Batch 2 GC pure functions, accepted by main agent (`records/2026-10-13-里程碑四预开发.md` "Three: Handover and Judgment").

**Owed checks**: C122 (entry count measurement basis after allocation record point-delete); C194 (Generational Incremental Scrub Cannot See Moved Copies); C146 (fallback policy with no empty segment undefined repo-wide); C195 (prev propagation during destroy cascade is written only in the experiment's model revision log); C197 (unwritten whether three writes of migration form single transaction).

#### Settled Item 6: Runtime Observability Points for Compaction Execution

**Ruling**: Six observables — fragmentation metric `F = runs / L` runs, completely empty cluster segment count, fallback count (times allocation fails to enter cluster segment), unreclaimable bytes, compaction progress, and relocation volume per compaction batch. **Three metrics used for triggering are persisted, avoiding recomputation post-mount**:

| Metric | Where stored | Rationale |
|---|---|---|
| runs, completely empty cluster segment count | Accounting tree (D5 (Snapshot / Space Accounting Mechanism) Settled Item 3 carrier: "(metric, dimension tuple, generation) → full value") | Idempotent full value, adding metrics does not change format; D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 metric list expands from nine to eleven, following self-declared reopening clause, reopening already executed |
| Unreclaimable bytes | Accounting tree | Originally Metric 3 of D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 |
| Compaction progress | Not stored: full run verification against target state across entire `R` before intention deletion; entries and allocation records themselves constitute progress | Attacker F4: Resume point following current rollback would skip resurrected placements |
| Fallback count, relocation volume per compaction batch | Runtime counters | Observability only, does not enter judgment, loss has no impact |

Without persistence, post-mount trigger criteria either lack input (blind execution) or require full scan (O(capacity), violating `.claude/rules/fs-design.md` Slot 1). Changes new-pool file creation bytes: Yes — new-pool file creation writes two additional accounting entries; [layout/01-first-txn.md](../layout/01-first-txn.md) Section 5 adds one line.

**Scope**: Governs where these six metrics reside and which are persisted; does not govern threshold values (governed by Settled Item 1). runs and completely empty cluster segment count **are both physical metrics with per-device dimension**: runs measurement basis is count of contiguous runs of free slots in unit area, one row per device (D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 Item 10); completely empty cluster segment count likewise one row per device. Numbers quoted for trigger judgment in this item measured before measurement basis change ("traverse allocated objects in key order, count physical contiguous runs"); the two measurement bases do not measure same quantity and cannot be copied directly. Incremental runs drifts under intention redo; persistence turns it into standard accounting tree problem (one full value per generation, shaped like I-3.1 (Allocated Statistics Match)), audit path being full scan runs.

**Evidence**:

- E93 (Placement and Fragmentation Under Aging): Fragmentation maintainable incrementally in O(1) under its given definition, providing full scan runs audit path used by checker for comparison.
- Three-way argument (unmodified output `research/prompts/d26-open-items-{dossier,forward-answer,attack-answer,local-answer}.md`): Backward leg independently pointed out compaction progress residing in intention record; non-persisted trigger metrics force blind execution or full scan post-mount.
- User settlement (persist), original wording in change history.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G1: logged_ops compaction intention does not store resume point (attacker F4: resume point following current rollback would skip resurrected placements); verifies target state fully across entire `R` before intention deletion.

**Owed checks**: C91 (redo drift between compaction intent and incremental trigger amount); C141 (inputs to halting criteria non-persistent); C326 (whether E93 numbers can still be cited after runs changed measurement basis); C323 (no repository clause for image size); C413 (fragmentation and all-empty segment statistics lack invariants).

#### Settled Item 7: Reclamation Splits into Two Lines, Handed to the Disk When Testing Finds Its FTL Supports GC

**Ruling**: Reclamation (GC) will follow two lines. First implement self-hosted GC of Leg 1; later, when testing reveals disk FTL supports GC, bypass in-code GC and let disk implement GC natively, with SingleFS only tagging and issuing commands. Which line reclamation follows is handled via feature bits in system configuration, carried per disk; tier among incompat, compat_ro, compat unsettled, deferred to Unsettled Item 10. GC design must reserve this branch; feature will definitely be added later; no evaluation, implementation, or verification in near term. Changes new-pool file creation bytes: No — self-hosted GC built first, disk-offloaded line unimplemented near term, version 1 does not write this bit; reassessed once bit is settled.

**Scope**: Governs whether reclamation branches and selection mechanism; does not govern how disk FTL is tested (Unsettled Item 8), which Leg 1 tasks halt when taking disk-offloaded line (Unsettled Item 9), feature bit tier and commands issued to disk (Unsettled Item 10). **Not the same as D10 (True Answer to Random Writes / Fragmentation) Candidate C**: Candidate C verbatim states "NVMe FDP / ZNS hands data placement and GC responsibility to host"; Settled Item 7 "hand to disk" runs in opposite direction — reclamation remains in disk native FTL, host does not participate; FDP / ZNS remains "use-if-present acceleration", unaltered by a single word. Disk-offloaded line does not exist in `crates/`; self-hosted line has, in `crates/singlefs-core`, the intention entry format (`logged_operations.rs`), region scan (`region_scan.rs`), segment selection and planning (`compaction_plan.rs`, including the trigger `compaction_starts_after_a_user_change` that runs at most one batch after a user change is published), the publication entry point (`publish_relocation_batch` in `transaction.rs`) and the explicit entry point `MountedSession::run_compaction` (`mounted_session.rs`); the batch size is not settled by the clauses, so the product path halts before signing any intention (`CompactionRunStop::BatchSizeNotDecidedByTheClauses`), and only a test-only switch that supplies a batch size drives it further. Besides: `crates/singlefs-core/src/system_configuration.rs` holds watermark constants `COMPACTION_WATERMARK_COUNT`, `COMPACTION_WATERMARK_BUILT_IN_DEFAULT` ("three watermarks: low / high / stop line", version 1 uniformly 0 = use built-in defaults, writes 24 bytes of 0), `crates/singlefs-harness/src/mount_transparency.rs` and `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` check only watermark built-in default 0, `crates/singlefs-core/src/allocator.rs` `reclaim_released_up_to` returns released placements to the free pool after reallocatability without moving live data, the block device trait has `discard_at` (`crates/singlefs-core/src/block_device.rs`), which on the product path is called by `discard_the_segments_reusable_as_a_whole` in `crates/singlefs-core/src/mounted_session.rs` after each user-change publish and after the explicit compaction entry point (`discard_at(` also hits `block_device.rs` unit tests, the harness recorder and the fault injection device forwarder); discard timing detailed in Unsettled Item 9 "Timing of discard".

**Evidence**: No experiment: this item is product direction policy judgment; user settled no near-term evaluation, implementation, or verification; neither line exists in code, offering no measurable metrics; items requiring measurement (testing FTL, Leg 1 halted tasks, feature bit tier) fall into Unsettled Items 8 / 9 / 10. User verbatim wording and responses in change history.

**Owed checks**: None.

#### Settled Item 11: Where the Answer to Fragmentation Rests: Two Legs Stacked, Device Interfaces Demoted to Acceleration

**Ruling**: **Fragmentation accumulated over long-term operation resolved by SingleFS itself, not staked on device interfaces.** Two legs **superimposed**, not either-or:

| Leg | What it is | Format-level? |
|---|---|---|
| **One: Ex-post compaction** | Resident, automatic, background compaction invisible to user, plus SingleFS native placement and reclamation layer; reclamation half branches into two lines (Settled Item 7) | No — falls into `.claude/rules/fs-design.md` Slot 3 (background, resumable, non-decision path, no commandments) |
| **Two: Structural anti-aging** | Index structure itself preserves scan locality after aging, producing less fragmentation at source | Settled not to enter format (Settled Item 4) ⇒ does not block D8 (Core Index Structure) node format freeze |

**NVMe FDP / ZNS demoted to "use-if-present acceleration", no longer the answer**: Staking answer to fragmentation on device interfaces is unreliable — on devices lacking FDP / ZNS, it equates to having no answer, while D12 (Target Media) settled not capping media classes beforehand ⇒ those are mainstream media, not corner cases. **Superimposition is current ruling, not final end**: One leg may be demoted later, using E10 (Aging: Structural Resistance vs Post-hoc Compaction) self-contained criterion ("③ If able to reach Level ②, resident background compaction demotes from commitment to optimization"), **no separate criteria permitted**; both pursued prior to that. Changes new-pool file creation bytes: No — both legs are background and placement policies; new-pool file creation does not run them.

**Scope**: Governs where answer rests and whether two legs are superimposed. Leg 1 framework in Settled Item 12; trigger, transaction form, acceptance, accounting interaction in Settled Items 1 / 2 / 3 / 5; reclamation branching in Settled Item 7; Leg 2 form in Settled Item 4. Status of D10 (True Answer to Random Writes / Fragmentation) three candidates documented in D10 (True Answer to Random Writes / Fragmentation), not repeated here. Demotion criterion hinges on unrun experiment; prior to its execution, this "superimposition" has no usable flipping observation.

**Evidence**:

- E93 (Placement and Fragmentation Under Aging): Cluster segment policy non-self-sustaining without compaction feeding empty segments; neighbor rewriting hits ceiling while compaction decreases monotonically with budget — Leg 1 is required, not optional acceleration.
- User settlement (two legs superimposed, FDP / ZNS demoted to acceleration), original wording in change history.

**Owed checks**: C139 (release cannot be issued after compaction halts).

#### Settled Item 12: Form of Leg 1 Is Boxed In by Three Settled Disciplines

**Ruling**: Form of Leg 1 is not an open question; three settled disciplines jointly determine it completely:

| Constraint | Source |
|---|---|
| Compaction belongs to Slot 3, **no commandments** — can traverse, can be slow, excluded from any instantaneous decisions | `.claude/rules/fs-design.md` three-slot discipline, verbatim naming "destroy snapshot, compaction, scrub" |
| Unbounded heavy work follows "write intention → chunking → resumable" | Same slot; backed by C2 (unbounded operation made into single transaction) |
| Relocation unit is "whole object re-placed as a group", not single stripe | D2 (RAID Stripe Policy) Settled Item 8 measurement basis, directly reusable |

**Compaction and rebalancing are two separate tasks, not merged**: Compaction manages fragmentation, rebalancing manages inter-disk balance (explicitly stated in D2 (RAID Stripe Policy)). Changes new-pool file creation bytes: No — all three reuse existing disciplines, introducing zero on-disk fields.

**Scope**: Governs Leg 1 overarching framework, not individual clauses — trigger in Settled Item 1, transaction form in Settled Item 2, acceptance in Settled Item 3, accounting interaction and region scan / segment selection in Settled Item 5, observability points in Settled Item 6, reclamation branching in Settled Item 7. Slot 3 no commandments covers only compaction background path; once a metric is dragged into runtime decision paths (Settled Item 3 dragging completely empty cluster segment count is an example), it no longer enjoys this exemption.

**Evidence**:

- E74 (Allocation Record Entry Count and Compaction Cost): Compaction cost does not scale with disk capacity, but copy-on-write amplification differs 370× depending entirely on placement clustering — reused measurement basis "relocation unit is whole object re-placed as a group" underpins this slot.
- E93 (Placement and Fragmentation Under Aging): Compaction scales down continuously with budget without saturation, supporting design as resident background task rather than one-off tool.
- User ruling (Leg 1 form needs no further decision), original wording in change history.

**Owed checks**: C2 (unbounded operation made into single transaction).

#### Settled Item 13: Where Background Task Computation Sits, How It Is Enabled, and Its Cadence

**Ruling**: Eight rules.

1. **Where computation sits**: Defaults to CPU. Mount parameter `background_compute` (accepts `cpu` or `gpu`, default `cpu`; CLI takes `--background-compute=gpu`) selects GPU. This set of mount parameters (this rule and Rules 2 and 3) exists only once and governs all background tasks, not set per task: with `gpu` on, every background task that has a GPU path (GC relocation, packing relocation, and other background tasks that later gain a GPU path) runs on GPU, and tasks without a GPU path keep computing on CPU. Resides only in memory: not written to system configuration, does not occupy feature bits (D24 (Offloading Heavy Background Work to GPU) Settled Item 2 Rule 1), supplied per mount. Switchable via mounted session control interface during runtime (one switch flips all background tasks), taking effect from next batch; immutable mid-batch (`.claude/rules/fs-design.md` "branch variables forbidden from changing mid-operation"). GC relocation puts three stages on the GPU: checksum verification in the scan, unit header parsing and reverse mapping lookup in the region scan, and segment selection scoring; the compaction publish half stays on the CPU as before. Once `gpu` is on it keeps running as on, with no automatic switch back to CPU depending on whether a given card or submission interval pays off; the cards and submission intervals that lose are written into the documentation and into the control group of Rule 7.
2. **Which cards to use**: Mount parameter `background_gpu_devices` is a list of cards (can designate multiple); when cards are named, only the named cards are used; if omitted, all discrete GPU cards on the host are used, and integrated GPUs and software Vulkan adapters do not count as "GPU cards" and are not part of "all". Every participating card is used, and work is split by the host CPU of one submission measured on each card at mount time; cards that lose as measured by Rule 7 are still used, and this is reported.
3. **VRAM ceiling**: Mount parameter `background_gpu_memory_limit` is a list of byte values with units, assigned one per card sequentially, corresponding 1-to-1 with participating cards (corresponds to `background_gpu_devices` list if supplied, or to the list of all discrete GPU cards from Rule 2 if omitted), without padding: if `background_gpu_devices` designates three cards, `background_gpu_memory_limit` must be omitted or provide exactly three values; providing only one errors; in GPU mode, VRAM used by all background tasks combined on that card does not exceed that one value: the value is a ceiling, not a reservation, so using less is fine (if the card has less free right now, the tasks still run and use only what is free), and usage must never exceed it at any moment, with batching, buffers and staging areas all counted in. If omitted, no limit set, background tasks use available capacity on card during execution. GPU mode batches consecutive calculation batches to GPU at once, bounded by this limit.
4. **Configuration errors and GPU unavailability fail with error, no fallback**: At mount or switch time, if `background_gpu_memory_limit` supplied with count unequal to participating cards, a value exceeds total card VRAM, GPU absent, driver mismatched, or designated card nonexistent, mount (or switch) rejected with error; if GPU encounters error while mounted, currently calculating batch aborted (intention remains on disk, target state monotonic, Settled Item 2), that background task stops with error without switching back to CPU; while it is stopped, any switch request that passes validation (the same validation as at mount time) lifts the stop on the spot, and the new settings apply from the next batch; a switch request already received before the error but not yet in effect also counts at the moment of the stop, and lifts the stop accordingly. The only exception is Rule 8: entries that stay uncertain for several consecutive rounds are handed to the CPU one by one to judge, while the background task keeps running on GPU.
5. **Start timing**: Evaluates Settled Item 1 three watermarks after each publish; periodic fallback timer checks again at mount-parameterized interval. Both modes follow identical start rules.
6. **Cadence after starting**: GC does not run on the calling thread of user changes (no batch runs before a user change returns); after it moves off, three guarantees still hold: a single publish pass has a bounded budget, the judgement on signing the next intent is kept across passes, and a compaction batch does not slip in between a user change raising F and its re-judgement. GC write bandwidth budget bounded by mount parameter; does not issue next batch when foreground fsync is waiting, and this discriminator only becomes visible once GC has moved off the calling thread.
7. **Worthwhile evaluated by saved CPU time**: The purpose of GPU mode is to take less host CPU time; running faster on large data is only a by-product. Benefit is host CPU time saved by the background task (its CPU time in CPU mode minus its CPU time in GPU mode, latter including data preparation, submission, result awaiting, comparing the two results, and the entries Rule 8 hands to the CPU to judge); pursued even if wall clock equals CPU mode; D24 (Offloading Heavy Background Work to GPU) Settled Item 3 wall clock ratio criterion does not apply to background tasks covered by this item. Post-implementation benchmark runs comparison: identical workload, identical pool, running CPU and GPU modes respectively, reporting GC wall clock, host CPU time, foreground fsync latency, across disk capacity tiers to evaluate scaling.
8. **How GPU results are re-verified**: Two independently written GPU kernels (sharing no kernel function) each compute once over the same batch of input, and the two results are compared entry by entry. Entries that match in both runs count as confirmed and go to the CPU: along the publish path the CPU checks each entry against the plan and against what it reads back from disk (`relocation_checked_against_the_version` in `crates/singlefs-core/src/transaction.rs`), then updates the bitmap, without separately re-checking the GPU results by sampling; entries that differ are marked with uncertain status, and this batch neither reclaims nor relocates them, leaving them to be recomputed in the next round. An entry that stays uncertain for 2 consecutive rounds (N = 2) is handed to the CPU implementation to judge once on its own, and the CPU verdict is applied; the count of consecutive uncertain rounds lives only in memory and is reset on remount. Each batch reports how many entries were confirmed, uncertain and handed to the CPU, so this is visible at run time. With a single card both kernels run on that card; with several cards work is split per Rule 2 and the two kernels may run on separate cards; multiple cards must never be required, and one card or several must both work with identical verdicts.

GPU path and CPU path are two implementations (D24 (Offloading Heavy Background Work to GPU) Settled Item 2 Rule 3): run-time re-verification follows Rule 8, with two GPU kernels each computing once and comparing, the CPU implementation handles entries that stay uncertain, and the two implementations are run against each other as a differential in tests. GPU path defaults to WGSL via `wgpu` (checker tier crash judge already uses it, `Cargo.lock` has `wgpu 30.0.1`, requiring no CUDA toolchain). Product GPU code lives in a separate crate `singlefs-background-gpu`: it depends on the core layer, the core layer does not depend on it, and the caller hands its implementation in at mount time; it shares no code with the GPU judge of the checker tier, which the checker-implementation-disjoint cell of the gate checks (user settlement 2026-10-26, verdict G7 of round 3 of VII·2). Changes new-pool file creation bytes: No — all eight rules govern mount parameters and background behaviors, not written to disk.

**Scope**: Governs switch, card selection, VRAM, error handling, start timing, and cadence forms. Rules 1 to 4 and Rules 7 and 8 govern every background task with a GPU path, today GC relocation and packing relocation; Rules 5 and 6 govern only GC, packing start and rate limiting are in D27 (Small-Data Packing Container) Settled Item 6. The three candidates of D24 (Offloading Heavy Background Work to GPU) remain deferred (D24 (Offloading Heavy Background Work to GPU) Settled Item 1); when reopened and given a GPU path they join the same switch, and which criterion judges their worth is settled at reopening. Which GC stages are computed on the GPU is settled in Rule 1: by the yardstick of Rule 7, once the CPU side switches to hardware CRC and lookup by identity (index building not counted), with submissions issued back to back 64 of 84 cells do not lose on the RTX 5090 and 10 cells do not lose on the RTX 5060 Ti, and at a 50 ms submission interval neither card has a single cell that does not lose (the fourth rerun of E185 (Wall-Clock Breakdown of a Batch of GC Migrations); the table counted by the main agent is in section two of `research/prompts/m4-gpugc-design-r3-main-verification.md`); Rule 1 still keeps running as on, with no automatic switch. Does not govern: which packing phases run on GPU (Milestone 7 design round); periodic fallback interval, write bandwidth budget default (awaiting C412 (wall-clock cost of compaction unmeasured) and E10 (Aging: Structural Resistance vs Post-hoc Compaction)). ⚠️ Rule 4 halts GC on runtime error: during stoppage, space only decreases without increasing; coexist reading with D3 (Space Allocation) Settled Item 14 "background compaction resident, automatic" is that this occurs only when user explicitly enables GPU, default `cpu` unaffected. ⚠️ The two kernels of Rule 8 catch errors that only one of them makes (a bug in one kernel, a bit flip without ECC, a nondeterministic race) but not errors both make together: when something the two share goes wrong (the same driver and shader compiler, the same preparation of the input table, the same misreading of the rules), the two still agree, and that class is caught by the differential between the GPU and CPU implementations in tests and by crash-point replay. ⚠️ The count of consecutive uncertain rounds in Rule 8 lives only in memory and is reset on remount: if an entry only ever gets one uncertain round computed per mount, it never reaches the CPU judgement.

**Evidence**:

- E185 (Wall-Clock Breakdown of a Batch of GC Migrations) third and fourth reruns: the host CPU of one GPU submission, and the number of cells that do not lose broken down by CPU implementation, card and submission interval (`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`; the table counted by the main agent is in section two of the round 3 verdict); the stages Rule 1 puts on the GPU, and the losing cards and intervals in the scope, are measured by it.
- Three-way verdicts `research/prompts/m4-gpugc-design-r1-main-verification.md`, `research/prompts/m4-gpugc-design-r2-main-verification.md`, `research/prompts/m4-gpugc-design-r3-main-verification.md` (three rounds; fork sheet `research/prompts/m4-gpugc-design-r1-forks.md`): GC staying on the calling thread puts the GPU on the foreground path once enabled, batches cannot be accumulated across batches, equal-weight rotation over all cards is out, and the parenthetical of Rule 8 conflicts with the entry-by-entry check on the publish path, all broken through; whether to move off the calling thread, how to split across several cards, N and where the count lives, and switching while stopped went to the user.
- User settlement (2026-10-19 four dialog questions and follow-ups; 2026-10-20 parameters unified across all background tasks, benefit measure extended to all background tasks; 2026-10-21 two GPU kernels each compute once and compare, uncertain entries wait for the next round, entries uncertain for consecutive rounds go to the CPU to judge; 2026-10-26 two dialogs settled the eight questions of VII·2), original wording in change history.
- No experiment: the defaults of the periodic fallback interval and the write bandwidth budget have not been measured yet; numerical values not here, awaiting C412 (wall-clock cost of compaction unmeasured) and E10 (Aging: Structural Resistance vs Post-hoc Compaction).

**Owed checks**: C412 (wall-clock cost of compaction unmeasured).

### Unsettled Items

| # | Item | Status |
|---|---|---|
| 8 | **Switching criteria between the two reclamation lines** | How to test disk FTL support for GC, when to test, whether test results persist to disk, all unspecified; user settled no near-term evaluation. Changes new-pool file creation bytes: No (2026-09-20, evidence: reclamation and compaction run in background, new-pool file creation does not run them, see paragraph below table) Modifies format: Boundary undetermined (2026-09-21, evidence: prompt text contains "whether test results persist to disk", which itself is unsettled — persisting requires allocating on-disk field, non-persisting modifies zero bytes) **Status: Unsettled.** **Progress: Unimplemented.** |
| 9 | **Scopes of the two reclamation lines** | Which Leg 1 tasks halt when taking disk-offloaded line unspecified; user settled no near-term evaluation. Changes new-pool file creation bytes: No (2026-09-20, evidence: same as Unsettled Item 8, halting tasks only alters background behavior) Modifies format: Does not modify (2026-09-21, evidence: prompt addresses "which Leg 1 tasks halt", halting background behavior; D26 (Background Compaction and Placement Reclamation) Settled Item 6 settled three metrics as statistics in accounting tree, halting tasks does not alter width or semantics) **Status: Unsettled.** **Progress: Unimplemented.** |
| 10 | **Feature bit tiers and device commands for reclamation lines** | Recorded tentatively as feature bit, tier and commands remain unsettled; user settled no near-term evaluation. Changes new-pool file creation bytes: No (2026-09-20, evidence: system configuration feature bit tiers written by new-pool file creation currently all 0, adding tier alters no written bytes) Modifies format: Modifies (2026-09-21, evidence: feature bits are bits in system configuration three bitmaps; prompt "occupies separate bit or follows D12 (Target Media) Settled Item 1 layout bit" corresponds to distinct bit assignments) **Status: Unsettled.** **Progress: Unimplemented.** |

Unsettled Items 8, 9, 10 change new-pool file creation bytes: No — reclamation and compaction run in background, new-pool file creation does not run them (D12 (Target Media) Settled Item 5 "Chaining with D26 (Background Compaction and Placement Reclamation)" verbatim: "Compaction remains background, first runnable target milestone does not run it"); Item 10 adds: self-hosted GC built first, disk-offloaded line unimplemented near term, version 1 does not write this bit, reassessed once bit is settled.

#### Unsettled Item 8: Switching Criteria Between Reclamation Lines

**Prompt**: How to test disk FTL GC support, whether tested at mkfs or mount, whether test results persist to disk. User answered input to criterion is "if testing reveals disk ftl supports gc", omitting what to test or when to test, and settled no near-term evaluation (original wording and full responses in change history).

**Scope**: Settled Item 13 only governs where background task computation sits, how it is enabled and its cadence, and does not answer how to test whether the disk FTL supports GC.

**This branch governed by `.claude/rules/fs-design.md` "Five Hard Requirements"**, which require: switching criteria must be explicit, computable, written down; each branch must be forceable by test; each branch verified separately, including crash-point replay; branch must be observable; branch variables forbidden from changing mid-operation. ⇒ "Disk FTL supports GC" test result cannot change mid-operation.

**Which built first is settled**: Build self-hosted line first, add disk-offloaded line later (Settled Item 7); this item must be answered before disk-offloaded line connects.

**Phase 1**: D12 (Target Media) Settled Item 5 settled first layout line as standard SSD, verbatim "requires no FDP / ZNS / any special device interfaces"; which line standard SSD falls into depends on this item.

#### Unsettled Item 9: Scopes of the Two Reclamation Lines

**Prompt**: Which Leg 1 tasks halt when taking "hand to disk" line. User answered "we do not implement gc ourselves. gc part handed to ftl", "we tag, issue commands", omitting which Leg 1 tasks constitute "gc part", and settled no near-term evaluation.

**Scope**: Settled Item 13 only governs where background task computation sits, how it is enabled and its cadence, and does not answer which Leg 1 tasks halt when taking the disk-offloaded line.

**Tasks attached to Leg 1** (listed by settled item index and chain tables, completeness not guaranteed):

| Task | Source | Halts when taking hand-to-disk line? |
|---|---|---|
| Compaction: move live data, produce completely empty cluster segments | Settled Item 1 (Stopping uses `R`-local predicate "at moment intention is deleted, `R` is completely empty by record") | Unsettled |
| Lazy relocation of partial stripes | D2 (RAID Stripe Policy) Settled Item 11 | Unsettled |

⚠️ **Broadest reading (halting all Leg 1) collides with Settled Item 3 evidence**: That chain leads from "compaction stalls" to "release transaction itself cannot issue", making `(completely empty cluster segment count ≥ N₀)` the second conjunct of allocation acceptance; it counts metadata blocks and transaction footprint after commit-endogenous blocks scatter, bypassing disk FTL, unaffected by how capable disk FTL is — this deduction is main agent interpretation, unvetted by tripartite evaluation. ⇒ This item must be answered jointly with Settled Item 3 and D3 (Space Allocation) Settled Item 5 (placement of commit-endogenous blocks).

**Timing of discard**: Disk FTL knows blocks are unused only upon receiving discard. When discard issues is settled (verdict `research/prompts/m4-gc-r1-main-verification.md` Section 3 G5): post-publish return, slots where "reallocatable" predicate evaluates true (reclaimed, not held by raised F, unquarantined, F effective) issued in 64-slot aligned full segments; block device trait adds `discard_at` (default implementation returns "unsupported" and increments count). The only unsettled aspect in this item remains scope of disk-offloaded line itself.

**Discard version 1 behavior** (`crates/singlefs-core/src/block_device.rs` current):

| Question | Version 1 |
|---|---|
| `discard_at` returns | `Applied` or `Unsupported`; `Unsupported` alters zero bytes, omitted from recording stream, counted in `unsupported_calls` |
| Length 0 | Returns `Applied`, excluded from `applied_calls` (verdict `research/prompts/m4-batch2-code-r1-main-verification.md` Section 2 S5, main agent decided: length 0 performs no action) |
| Backend | File backend reuses hole punching, returns `Unsupported` when `fallocate` reports punch hole unsupported; `O_DIRECT` backend returns `Unsupported` in version 1 |
| State count in crash-point replay | Follows D13 (Verification Route) Settled Item 4 literal: spans longer than one sector with pre-existing data take third state (verdict `research/prompts/m4-batch2-code-r1-main-verification.md` Section 2 S6, main agent decided) |
| Callers on product path | `discard_the_segments_reusable_as_a_whole` in `mounted_session.rs`, called once after each user-change publish and once after the explicit compaction entry point (segments holding orphan heads the sweep has not yet cleared are not discarded) |

Two questions remain open, answered when `.claude/kb/milestone/04-fourth-txn.md` II.2 issues discard on product path: whether "single discard partially landing within segment" is another form of third state; and whether host re-runs (`SparseBlockDevice` returning `Applied`) and VM (`O_DIRECT` backend returning `Unsupported`) share identical outcome.

#### Unsettled Item 10: Feature Bit Tiers and Device Commands for Reclamation Lines

**Prompt**: Which tier reclamation line feature bits use among incompat (unsupported cannot mount), compat_ro (unsupported mounts read-only), compat (unsupported mounts freely); whether occupying separate bit or following D12 (Target Media) Settled Item 1 layout bit; what commands issued to disk. User settled no near-term evaluation.

**Scope**: Settled Item 13 only governs where background task computation sits, how it is enabled and its cadence (its mount parameters occupy no feature bit), and does not answer which tier the reclamation line takes or what commands are sent to the disk.

**How tier is judged**: `.claude/rules/fs-design.md` "value of format branch lies in whether it turns 'misused' into 'cannot mount'". Main agent objection (upon which user selected "feature bit, tier unsettled"): if on-disk bytes written by both lines are identical, differing only in whether host relocates data and issues discard, implementations unaware of this bit could correctly read/write this disk, where incompat blocking yields no safety gains and compat should be used; if disk-offloaded line causes disk to lack prerequisites relied upon by self-hosted line (such as completely empty cluster segments required for Settled Item 3 acceptance), implementations unaware of this bit writing under self-hosted assumptions cause corruption, warranting compat_ro or incompat. ⇒ Tier follows answer to Unsettled Item 9. Feature bits written at mkfs, satisfying "branch variables forbidden from changing mid-operation" cited in Unsettled Item 8.

**Implementation form**: System configuration bootstrap header has incompat / compat_ro / compat three bitmaps, 256 bits each ([layout/01-first-txn.md](../layout/01-first-txn.md) "I. Planted by mkfs"), version 1 having only incompat bit 0 = 1 (first pure SSD layout line); `crates/singlefs-core/src/system_configuration.rs` checks only incompat bitmap (`incompat_bits_are_mountable`), `grep -rn "compat_ro" crates --include=*.rs` yielding zero hits. Main agent reading (user unspecified, not settled): commands issued to disk include at least discard, rationale detailed in Unsettled Item 9 "Timing of discard".

### Interactions with Other Decisions

| Decision | Interaction Point |
|---|---|
| D10 (True Answer to Random Writes / Fragmentation) | D26 (Background Compaction and Placement Reclamation) Settled Item 11 provides answer to D10 (True Answer to Random Writes / Fragmentation) Settled Item 1: two legs superimposed, Candidate C demoted to acceleration. After reclamation splits into two lines, self-hosted layer no longer treats all media uniformly; Candidate C hands placement and GC to host, opposite to Settled Item 7 "hand to disk" |
| D3 (Space Allocation) | Leg 1 undertakes commitment in Rule 2 for "resident, automatic, background compaction invisible to user"; Settled Item 3 reopens D3 (Space Allocation) Settled Item 2 algorithm for "how much space is needed to unbind self", not principle itself |
| D8 (Core Index Structure) | Settled Item 4 settles not entering format ⇒ Leg 2 does not block node format freeze; compaction unbounded work follows D8 (Core Index Structure) Settled Item 5 intention mechanism |
| D15 (Format Freeze Policy) | Index node internal layout is Layer 4; Settled Item 4 judges not touching that layer |
| D21 (Boundary Between Authoritative and Derived States) | Settled Item 1 judged authoritative state: reverse mapping主体 resides in units, compaction does not wait for index rebuild |
| D9 (Encryption) | Settled Item 5 settled non-key side performs scrub only, neither moving nor reclaiming ⇒ compaction completely halts during disk locked period. This is boundary accepted by settled ruling, not gap in D26 (Background Compaction and Placement Reclamation) |
| D2 (RAID Stripe Policy) | Settled Item 8 relocation unit measurement basis reusable (D26 (Background Compaction and Placement Reclamation) Settled Item 12); Settled Item 11 attaches "lazy relocation of partial stripes" to Leg 1 — first concrete responsibility of Leg 1 |
| D12 (Target Media) | Settled Item 5 settled first layout line as pure SSD ⇒ Phase 1 has no FDP / ZNS to rely upon; "Phase 1 main battlefield" holds only if standard SSD falls to self-hosted line, which line depending on Unsettled Item 8 |
| D5 (Snapshot / Space Accounting Mechanism) | Settled Item 5 "birth unchanged" concludes its `birth(b)` definition; Settled Item 6 adds two metrics (runs, completely empty cluster segment count) to Settled Item 4 metric list, expanding from nine to eleven |
| [layout/01-first-txn.md](../layout/01-first-txn.md) | Settled Item 6 causes new-pool file creation to write two additional accounting entries, adding one line to Section 5 |

## Historical Versions

D26 (Background Compaction and Placement Reclamation) historical entries are consolidated in [decisions-history.md](../decisions-history.md).
