## E91 Ring and Admission Accounting Under Settled Format — Run (2026-09-03, Pure Arithmetic + Occupancy Simulation, 12 Unit Tests / All 8 Mutations Caught): **Three Old Ring Lower Bounds Each Reproduce Under Their Respective Measurement Bases Without Contradiction; Under Settled Measurement Basis, Constrained Value of I-8.1 (Ring Geometry Sufficiently Large) Is 14776 KiB (F=2); True Difference in Retention Duration Is 1 Record vs 10923 Records; Acceptance / Admission Watermark for Path A Is 1.91 GiB**

[中文](91-已定格式下的环账与准入账.md) · **English** · [日本語](91-確定フォーマット下でのリング会計と受入会計.ja.md)

**What it settles is the accounts posted by the three-way argument on maintenance cost in E16 (Role of Journal: WAL vs Intent Log)**:
Three mutually contradictory ring lower bounds coexisted in the repository (8 KiB in E75 (Record Size and Ring Geometry),
8 MiB derived from D16 (Publish Semantics) settled item 5, and never recalculated under the new semantics of D23 (Journal Role & Format) settled item 14), differing by 1000×; "the true difference between Path A and Path B lies in retention duration" had never been calculated under the settled format;
"ancestors not deferred = 4.0 blocks/operation" had never been substituted into the admission / acceptance formula of E19 (Spurious ENOSPC Under Defer Window).

**During design, the main agent already verified a fourth inconsistency, entered as a hypothesis to test rather than a criterion**:
D16 (Publish Semantics) settled item 5 derived "occupancy per checkpoint = T_dirty / 512 = 4 MiB" using the
32 B/entry measurement basis (ENTRY=32 in E70 (Feasible Region for Two Checkpoint Thresholds)),
whereas D23 (Journal Role & Format) settled item 12, settled later, specifies a record format of fixed 4 KiB length,
78 B header, and 56 B per entry. Whether the two actually yield different ring lower bounds is judged by the model.

### Verbatim Cited Clauses (verify-before-claiming)

- D23 (Journal Role & Format) settled item 12: Record fixed 4 KiB length written into system configuration;
  "A 4096-byte record fits (4096 − 99) / 56 = 71 entries" (E75 (Record Size and Ring Geometry) actually measured that all four header readings 78/91/95/99 yield 71 entries).
- D23 (Journal Role & Format) settled item 14: Recovery only applies records whose (instance generation, checkpoint_txg)
  is strictly greater than that of the selected root—**once the root is persisted, records below the watermark are dead**.
- D16 (Publish Semantics) settled item 6: Each publish increments checkpoint_txg by 1; **an fsync trigger is also a publish**.
- D16 (Publish Semantics) settled item 5: T_dirty = 2 GiB (initial value); T_time feasible interval [200 ms, 5 s], taking 5 s.
- I-8.1 (Ring Geometry Sufficiently Large): Ring ≥ F × worst-case journal occupancy of any transaction, F ≥ 2.
- D23 (Journal Role & Format) admission / acceptance rule: Remaining space > churn per checkpoint × (defer generations + 1)
  + cost of one checkpoint.
- E16 (Role of Journal: WAL vs Intent Log) measured: Ancestors not deferred 4.0000 blocks/operation;
  deferred (multistream ckpt=1000) 0.0567 blocks/operation. E44 (Host Testing and Cost of Sequence Number Bit Width): Local machine 2785 fsync/sec.

### Three Accounts (Pure Arithmetic + Occupancy Simulation with Independent Routes)

1. **Reconciliation of ring lower bounds**: The same model recalculates each of the three published numbers under its respective measurement basis—
   target workload 12-entry transaction and 62,000-entry transaction (the two cells of E75 (Record Size and Ring Geometry)),
   T_dirty full-window checkpoint transaction at 32 B/entry (the cell of D16 (Publish Semantics) settled item 5)—all three numbers must be **reproduced verbatim**;
   then calculate the true constrained value of I-8.1 (Ring Geometry Sufficiently Large) under the settled format measurement basis (56 B/entry, 71 entries/record, T_dirty full window).
2. **Retention duration (Path A vs deferred ancestor form)**: Under watermark semantics, simulate fsync stream and checkpoint triggering,
   recording peak bytes of live records in the ring (above watermark) and lifetime of a single record;
   Path A publishes on each fsync; deferred form publishes only at checkpoint (earlier of T_time and T_dirty).
   Closed-form and simulation must match exactly.
3. **Admission / acceptance watermark**: Substitute 4.0 and 0.0567 respectively into the admission / acceptance formula, scanning defer generations × operations per window;
   simultaneously use a small pending-queue simulation in the style of E19 (Spurious ENOSPC Under Defer Window) to find the onset of spurious ENOSPC, comparing against closed form.

### Criteria (Hardcoded Before Run)

1. **Absolute values are all pinned by independent arithmetic**: 71 entries per record (independently calculated from (4096−78)/56);
   T_dirty full-window entries = 2 GiB / 16 KiB node; record count, occupancy, and ring lower bound must match manual calculation one by one.
2. **Three published numbers each reproduce verbatim under their respective measurement bases** (8 KiB / 6992 KiB / 4 MiB→8 MiB).
   If any fails to reproduce ⇒ that number itself has an error, recorded faithfully, without modifying criteria of E91 (Ring and Admission Accounting Under Settled Format).
3. **Simulation matches closed form exactly**: Peak occupancy of Account 2 and onset of spurious ENOSPC of Account 3 must differ between simulated and closed-form values
   by ≤ 1 record / 1 block. Not equal ⇒ implementation flaw, whole run invalidated.
4. **Positive controls**: When churn takes original parameters of E19 (Spurious ENOSPC Under Defer Window)
   (200 operations × 4 blocks per window), simulation of Account 3 must reproduce the behavioral shape of E19 (Spurious ENOSPC Under Defer Window) (no spurious ENOSPC above watermark,
   appearing below); Account 2 in the degenerate tier of "truncate upon each publish" (publish rate→∞) must collapse its peak to single-transaction occupancy.

### Failure Clauses (Hardcoded Before Run)

- If Criterion 2 shows that the derived number in authoritative settled text does not match the settled format measurement basis (the hypothesis entered at the outset),
  **faithfully record and attach a scope note + decisions-history to that text**, without altering the T_dirty / T_time settlement itself,
  and without making a ring size decision on the spot.
- Deterministic model: 5 runs of md5 only prove absence of hidden state.

### Measurement Basis and Known Limitations

- Pure arithmetic + counting simulation, no device. fsync rate 2785/sec is a local machine figure (E44 (Host Testing and Cost of Sequence Number Bit Width)), not a universal value;
  churn amounts 4.0 and 0.0567 are tied to tree geometry of E16 (Role of Journal: WAL vs Intent Log) (complete 128-way, height 4).
- In-flight record count upper bound (named by I-8.3 (Strictly Continuous Replay Prefix), value unsettled) scanned as parameter without settling its value.
- Ring size itself is not settled—what is produced is the constrained value of I-8.1 (Ring Geometry Sufficiently Large) and the respective scopes of the three old numbers.
- Prerequisites: None.

### Actual Measurement (2026-09-03, 5 md5 Runs Byte-Identical; 24/24 Admission / Acceptance Cells Match Closed Form Exactly)

**Account 1 (Reconciliation of ring lower bounds, Criterion 2 all pass)**:

| Measurement Basis | Transaction | Record Count | Occupancy | F=2 Ring Lower Bound |
|---|---|---|---|---|
| E75 (Record Size and Ring Geometry) target workload | 12 entries | 1 | 4 KiB | **8 KiB** (reproduced verbatim) |
| E75 (Record Size and Ring Geometry) worst-case sample | 62,000 entries | 874 | 3496 KiB | **6992 KiB** (reproduced verbatim) |
| D16 (Publish Semantics) settled item 5 old basis (32 B/entry) | T_dirty full window | — | 4 MiB | **8 MiB** (reproduced verbatim) |
| **Settled measurement basis (56 B/entry, 71 entries/record)** | T_dirty full window 131,072 entries | **1847** | **7388 KiB** | **14776 KiB (F=3: 22164 KiB)** |

Three old numbers **each reproduce verbatim under their respective measurement bases**—not that anyone calculated wrongly, but that each answered a different question.
The hypothesis entered at the outset is judged true: the derived number in D16 (Publish Semantics) settled item 5 was **undercounted by 1.80×** under the old basis
(4096 → 7388 KiB); a scope note has been attached to that text (see decisions-history).
The constrained value of I-8.1 (Ring Geometry Sufficiently Large) under settled semantics is dominated by the **T_dirty full-window checkpoint transaction**;
it is identical for both Path A and Path B forms (the worst-case transaction is shared).

**Account 2 (Retention duration, Criteria 3 and 4 all pass)**: Under watermark semantics (D23 (Journal Role & Format) settled item 14),
Path A publishes per beat ⇒ peak live records in ring is **constantly 1 record (4 KiB)**, single record lifetime 1 beat (≈ 0.36 ms @ 2785 fsync/sec);
deferred form peak is **10,923 records = 42.7 MiB**, lifetime 3922 ms—the earlier trigger is the **T_dirty branch**
(target workload dirties 192 KiB per beat ⇒ 10,923 beats < 13,925 beats of T_time), simulation and closed form matching exactly.
⇒ Today's reading of the old measurement basis "48 B vs 10⁴–10⁵ B" in E16 (Role of Journal: WAL vs Intent Log) is
**1 : 10,923 records**; Path B form ring must also hold the entire window of live records, while Path A is constrained only by the shared worst-case transaction lower bound.

**Account 3 (Admission / acceptance watermark, Criteria 3 and 4 all pass)**: Substituting churn into the D23 (Journal Role & Format) admission / acceptance formula,
all 24 cells match closed form exactly cell by cell. Representative cell (T_time = 5 s full window 13,925 operations, defer 8 generations):

| Form | Churn | Unfillable Watermark |
|---|---|---|
| Path A (ancestors not deferred) | 4.0000 blocks/operation | **501,364 blocks ≈ 1.91 GiB** |
| Deferred ancestors | 0.0567 blocks/operation | 7165 blocks ≈ 28 MiB |

The ratio is 70.6×, consistent with the 70× measurement basis of E16 (Role of Journal: WAL vs Intent Log) (another manifestation of the same pair of numbers).
Path A's watermark does not scale with disk capacity (it is constant), but 1.91 GiB represents a significant proportion on small disks—
this is the quantitative realization of the account in backward leg 1.1, recorded faithfully without reopening any decisions on the spot.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e91_ring_admission.rs`
  (`cargo run --release --bin e91-ring-admission`), artifact
  `e91-ring-admission-2026-09-03.out` (32 lines, ending `emitted=32`).
  Re-run: pass E91 (Ring and Admission Accounting Under Settled Format) ID to `scripts/experiment-apparatus/replay.sh`, byte comparison.
- **12 unit tests all pass; all 8 mutations caught** (`scripts/mutation/tables/e91_ring_admission.tsv`),
  including entry count omitting header subtraction, rounding direction, old basis factor, watermark non-truncation, inverted earlier trigger, missing trigger branch, missing +1 in formula, queue non-dequeue.
- Deterministic model: 5 md5 runs byte-identical (proves only absence of hidden state).

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) settled item 5 | Supports | 2026-09-20 Unaffected: hypothesis entered at outset judged true—derived number in settled item 5 undercounted by 1.80× under old entry measurement basis (4096 vs 7388 KiB); those two derived numbers in the scope of settled item 5 (1847 records, 7388 KiB, ring lower bounds 14776 / 22164 KiB) originate from here; settlements of `T_dirty` and `T_time` themselves not modified; repository-wide renaming on same date "系统配置" → "系统配置" only touched wording, conclusions, numbers, and artifacts unchanged |
| D23 (Journal Role & Format) settled item 14 | Does not affect | 2026-09-20 Unaffected: served only as input—watermark semantics "recovery only applies records strictly greater than selected root" and admission / acceptance formula taken from it; E91 (Ring and Admission Accounting Under Settled Format) produces constrained value of I-8.1 (Ring Geometry Sufficiently Large) and respective scopes of three old lower bounds, without modifying any item of D23 (Journal Role & Format); repository-wide renaming on same date "系统配置" → "系统配置" only touched wording, conclusions, numbers, and artifacts unchanged |

## Historical Versions

### 2026-09-03
- Established and run on same day. Source: two accounts posted by "maintenance cost half" of E16 (Role of Journal: WAL vs Intent Log)
  and coexistence of three lower bounds pointed out by backward leg 1.6; criteria hardcoded before run;
  fourth inconsistency verified by main agent during design (derived number in D16 (Publish Semantics) settled item 5 using old entry measurement basis)
  entered as hypothesis to test, judged true in actual measurement.
