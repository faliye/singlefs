## E16 Role of Journal: WAL vs Intent Log — Accounting Model Half Run (2026-08-28; 08-31 Supplemented Ancestor Count; 09-03 Supplemented Byte Basis; 09-28 Fifth Run: Peak Location and Scope Sentences Under Today's Unit Widths; 2026-10-04 Re-run for Named Entries 67 → 64: Artifacts and 23 Unit Tests Produced, Mutation Table 18 Items Run Before and After, 18/18 Caught After), Latency Half Unrun

[中文](16-journal的角色WALvs意图日志.md) · **English** · [日本語](16-ジャーナルの役割-WAL-vs-インテントログ.ja.md)

**Background**: [decisions.md](../decisions.md) D23 (Journal Role & Format) settled item 1 states: "**Role must be determined before sizing can be determined**".

### First Modeling Collision: The WAL Side is Actually Two Arms

D23 (Journal Role & Format) settled: "**Allocation precedes journal: journal records are publication directives for items already written to disk**".
⇒ **"WAL fsync = appending a single record" is unimplementable in this project**—blocks named in records must already reside on disk.
WAL thus splits into two fundamentally distinct arms, differing not in speed, but in **whether replay requires an allocator**:

| Arm | What is Written at fsync | Replay Requires Allocator | Status with D23 (Journal Role & Format) Settled Items |
|---|---|---|---|
| `intent` Intent Log | Dirty leaves + all ancestors + root slot + cursor record | No | Compatible |
| `wal_full` WAL / Record names down to layer below root | Dirty leaves + all ancestors + record (**does not publish root**) | No | Compatible |
| `wal_leaf` WAL / Record only names leaves | **Writes dirty leaves only** + record; ancestors deferred to checkpoint | **Yes** | **Conflict** |

### Empirical Measurements (200,000 Operations × 3 Workloads × 3 fsync Tiers × 2 Checkpoint Intervals = 18 Cells × 3 Arms)

Write amplification (WA) = device blocks written / user write count; E16 (Role of Journal: WAL vs Intent Log) takes checkpoint interval 1000.
"fsync blocks" is p50, **measured in blocks, not time**.

| Workload | fsync | intent WA | wal_full WA | wal_leaf WA | intent Ring | wal_full Ring | wal_leaf Ring | intent fsync Blocks | wal_leaf fsync Blocks | intent Root Pubs | WAL Arm Root Pubs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| rand | Every | 7.000 | 6.002 | **4.101** | 48 B | 104048 B | 107120 B | 7 | **2** | 200000 | 200 |
| rand | Every 10 | 4.265 | 4.167 | **3.201** | 48 B | 59728 B | 63920 B | 43 | **11** | 20000 | 200 |
| rand | No fsync | 3.101 | 3.101 | 3.101 | 48 B | 3120 B | 3120 B | 0 | 0 | 200 | 200 |
| metaheavy | Every | 4.968 | 4.469 | **2.149** | 48 B | 104048 B | 160072 B | 10 | **3** | 200000 | 200 |
| metaheavy | Every 10 | 2.776 | 2.727 | **1.698** | 48 B | 10448 B | 116872 B | 56 | **21** | 20000 | 200 |
| seq | Every | 1.750 | 1.625 | **1.134** | 48 B | 104048 B | 496072 B | 14 | **9** | 200000 | 200 |

Across the grid, maximum ratio of total blocks written between arms is **3.71×** (attained on `multistream`, see section "Multistream Workload: Batching Widens Rather Than Consumes Deferred Gains (2026-08-28 Supplement)").

### Multistream Workload: Batching Widens Rather Than Consumes Deferred Gains (2026-08-28 Supplement)

**Why supplement this workload**: The original three workloads in E16 (Role of Journal: WAL vs Intent Log) could not answer "how much gain from deferring ancestors remains in the presence of group commits"—
`rand` is uniformly random across keyspace (operations rarely share ancestors, yet do not advance sequentially like real multi-stream),
and `seq` is a single spine (shares everything). Neither matches "multiple streams in parallel, advancing sequentially within their own subtrees".
⇒ Added `multistream`: 16 streams take turns, each sequentially appending within its subtree;
**subtrees diverge at the layer of "children of root", sharing only the root spine**.

| Workload | fsync Every | fsync Every 10 | What Batching Did to Gap |
|---|---|---|---|
| seq | 1.54× | **1.05×** | Almost eliminated |
| rand | 1.71× | 1.33× | Consumed half |
| metaheavy | 2.31× | 1.63× | Consumed partially |
| **multistream** | 3.40× | **3.71×** | **Widened rather than dropped** |

(Ratio of write amplification between Arm A=`intent` and Arm B=`wal_leaf`, checkpoint interval 1000.)

**Mechanism**: **Batching helps Arm A only when batched operations share ancestors.**
In `seq` they share the entire spine; in `multistream` they share only root ⇒ batching 10 operations requires writing 10 mutually disjoint spines,
saving Arm A nothing; while Arm B defers all of them to checkpoint.
Blocks per fsync: Arm A 7 → 43 (10× batching causes 6.1× block count), Arm B 2 → 11 (5.5×).

⇒ **The claim that "deferring ancestors and group commits are substitutes" is false when streams do not share spines**,
and represents the largest gap in the E16 (Role of Journal: WAL vs Intent Log) grid (`max_ratio` rose from 2.31 to **3.71**).

#### ⚠️ "Batching Does Not Consume Deferred Gains" Has Bounded Scope, Discovered Only via Sweep

The cell "16 streams × batch 10" fell right before the peak; looking at it in isolation reads a bounded finding as a universal statement.
After performing a 2D sweep across stream count × batch size (7 × 8 = 56 cells), the curve is **non-monotonic**:

| Streams＼Batch | 1 | 2 | 5 | 10 | 20 | 50 | 100 | 200 | Peak Position |
|---|---|---|---|---|---|---|---|---|---|
| 2 | 3.50 | **3.67** | 2.34 | 1.73 | 1.39 | 1.16 | 1.08 | 1.04 | Batch=2 |
| 4 | 3.50 | **3.67** | 3.33 | 2.28 | 1.67 | 1.28 | 1.14 | 1.07 | Batch=2 |
| 8 | 3.50 | 3.67 | **3.83** | 3.37 | 2.24 | 1.52 | 1.25 | 1.13 | Batch=5 |
| 16 | 3.50 | 3.67 | 3.83 | **3.91** | 3.38 | 1.99 | 1.49 | 1.24 | Batch=10 |
| 32 | 3.50 | 3.67 | 3.83 | 3.91 | **3.95** | 2.92 | 1.95 | 1.48 | Batch=20 |
| 64 | 3.50 | 3.67 | 3.83 | 3.91 | 3.95 | **3.98** | 2.89 | 1.95 | Batch=50 |
| 128 | 3.50 | 3.67 | 3.83 | 3.91 | 3.95 | **3.98** | 3.95 | 2.89 | Batch=50 |

**Peak positions fall row-by-row on "batch ≈ stream count".** ⇒ Accurate formulation:

> **Batching does not consume gains of deferred ancestors only holds when "batch size ≤ concurrent stream count";
> once batch exceeds stream count, ratio collapses monotonically toward 1.**

**This curve was predicted prior to measurement**, not post-hoc fit: Arm A per batch ≈ `batch + min(streams, batch) × height + 2`,
Arm B ≈ `batch + 1` ⇒ ratio must peak at batch ≈ streams. Across 56 cells relative error to prediction is **median 9.6%, max 20.5%**,
with prediction **systematically high** (ignoring top-level ancestor sharing). ⇒ Curve shape matches; do not directly cite analytical formula for absolute values.

⚠️ **Deterministic experiments require parameter sweeping, not re-runs.**
`.claude/singlefs-ai-sop/rules/test-discipline.md` "single observations do not count" governs **repeated measurements**,
whereas deterministic accounting models have zero measurement noise—**"byte-identical runs" prove nothing about robustness**.
The corresponding risk is **testing only one point in parameter space**, which only sweeps can guard against.

⚠️ **This does not reopen Axis 2 of [decisions.md](../decisions.md) D23 (Journal Role & Format).** Axis 2 was **structurally** eliminated by Axis 1
(deferring ancestors ⇒ new root cannot be self-consistent ⇒ cannot publish root), independent of gain magnitude.
E16 (Role of Journal: WAL vs Intent Log) changes the strength of premise "③ Gain Shape" in that decision, not conclusion.

The 56-cell sweep table above used original apparatus geometry (fanout fixed at 128, height 4); under the 2026-09-28 real widths this scope statement is no longer universally valid, see section "Fifth Run: Peak Placement, Formula Error, and Scope Sentences Under 2026-09-28 Unit Widths".

### Fifth Run: Peak Placement, Formula Error, and Scope Sentences Under 2026-09-28 Unit Widths

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 -->
The labels `G0`–G5 below are local labels from Section 5 of pre-run registry `e16-r5-prereg.md`, not kb identifiers.

**Independent Apparatus**: `scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs` (sharing no code with `crates/`, transferring semantics of both arms directly, adopting fanouts 144/147, tree heights derived from file size, and record widths—apparatus uses 64 entries/record; fifth run used 67 entries/record, described in Sixth Run section). Original apparatus `sweep()` was run unchanged as positive control (PC1), artifact `e16-journal-2026-09-28.out`, cell-by-cell identical to 2026-08-28 56-cell sweep.

**Under fanouts 144/147, tree height is determined by file size** (`.claude/kb/decisions/08-核心索引结构.md` (core index structure) settled item 14): G1=4 GiB (L=3), G2=1 TiB (L=4, real baseline), G4=16 TiB (L=5).

**On real baseline G2, "peak falls on batch ≈ streams" is not a universal statement**: for stream count ≤ 16 (9 rows), peaks still fall within [streams ÷ 2, 2 × streams]; for stream count ≥ 24 (12 rows), peaks fall directly at batch = 1, declining monotonically with no internal peak. Shapes across three geometries (G1/G4) differ mutually (15 of 21 stream counts classify differently across geometries)—**peak location changes with tree height, not a height-invariant constant**.

**Evaluating the two halves of original statement separately**: "Batching does not consume gains of deferred ancestors when batch ≤ streams" fails on 12 of 21 rows in G2 (adjacent batches drop >1% tolerance, drop range 1.3%–4.7%, mostly around batch = 2); "ratio collapses monotonically toward 1 beyond stream count" holds on all 21 rows in both "no longer rising" and "settling within 1/4 of peak excess at 16× streams", but phrasing "collapses toward 1" is imprecise—at batch = 16384, 12 rows (streams 2–32) have ratios that **dip below 1** (down to 0.9927–0.9968), not merely approaching 1.

**Prediction formula error under fifth run widths is far larger than original 9.6% / 20.5%**: across 56-cell subset (G1/G2/G4 × two denominators), median error is 13.0%–22.1%, max error 33.4%–93.9%, all exceeding bounds across three geometries and two denominators; 98.2% of cells remain systematically higher, direction intact, magnitude altered. Denominator measurement basis was re-verified: original "9.6%, 20.5%" used **predicted ratio as denominator** (not model ratio); replication command and figures reside in `name=q3_summary` rows.

**Current formulation**: Batching does not consume deferred ancestor gains only holds when concurrent streams ≤ 16 (specific geometry of 1 TiB file, 11 children under root); with more streams, gains peak at batch = 1. Beyond stream count, ratio ceases to rise and settles within 1/4 of peak excess near 16× streams, but for streams ≤ 32 (fifth run with 67 entries/record; streams ≤ 64 with 64 entries/record, see Sixth Run section), ratio actually dips below 1 at far batch tail (≥16384). This formulation left one owed check in Segment 1: tree height and children under root were conflated—on G1 (L=3), none of 21 rows fall on "batch ≈ streams". Segment 2 (supplement) discharges this check: governing variable is children under root, not tree height.

**Segment 2: "Peaks falling at batch = 1 for streams ≥ 24" is governed by children under root, not tree height.** ⚠️ Struck by third party, pending remake (verdict `research/prompts/d23-e16r5-r1-main-verification.md`): the attacker swept tree height, root children, fanout separately; tree height acted as switch (height ≤ 3 converges from batch = 1 across all streams, height ≥ 5 rises then converges across all swept streams), fanout had zero effect, root children only set threshold at height 4; separated sweep data was measured only on adversarial replica, main agent re-ran byte-identical, repository apparatus not remade (user settled on correcting miscopied numbers without third segment). Segment 2 ran grids across fanout and record width knobs; four fanout points (`one_tebibyte_file`, `full_root_level_four`, `small_fanout_one_tebibyte_file`, `large_fanout_one_tebibyte_file`) keep height constant at L=4, varying only children under root—`one_tebibyte_file`, `small_fanout_one_tebibyte_file`, `large_fanout_one_tebibyte_file` all have 1 TiB unit count (33692212), with root children 11 (real fanout 144/147), 17 (small fanout 128/128), 2 (large fanout 296/296); `full_root_level_four` (G3, saturated) has 457419312 units, 147 root children. Stream count where peak disappearance (Class C) begins shifts monotonically backward with root children: root children = 2 has all 21 rows (streams 2–1024) in Class C; = 11 (real baseline) starts Class C at streams 24; = 17 starts Class C at streams 32; = 147 (G3) extends Class A to streams 128, plateau to 128, Class B to 256, entering Class C only at streams 384—at identical tree height, raising root children from 11 to 147 delays Class C start from stream count 24 to 384 (16×). Conversely, tree height knob **without controlling root children** across four points (G1 L=3 with 7 root children all Class C, G2 L=4 with 11 root children Class C from 24, G4 L=5 with 2 root children complex, G5 L=2 with 58 root children all Class C) is non-monotonic—G5 has shortest height (L=2) yet non-trivial root children (58) and is all Class C, inexplicable by tree height alone. ⇒ Start of peak disappearance is governed by root children; tree height is not the direct variable; trees with few root children, regardless of height, lose deferred ancestor gains once batch exceeds 1. Artifacts (`name=q1_summary`, `name=row_summary` under `geometry=full_root_level_four`/`small_fanout_one_tebibyte_file`/`large_fanout_one_tebibyte_file`) match independently calculated root children (17, 2).

Among three points of record width knob (fifth run's 67 entries/record (apparatus 64 entries/record), `record_width_knob_minimum`=1 entry/record, `record_width_knob_maximum`=1,000,000 entries/record), 1 entry/record has all 21 rows in Class C (Arm B record overhead pushed extremely high by this knob, eliminating Arm A advantage over Arm B starting from batch = 1); 1,000,000 entries/record matches 67 entries/record classifications row-by-row (and row-by-row identical in 64 entries/record artifacts)—record width has minimal impact on peak location unless "small enough that nearly every unit requires a record".

Artifacts: `e16-fifth-run-peak-under-today-widths-2026-09-28.out` (Segment 1 full grid, G1/G2/G4 × 21 streams × avg 54 batches = 3423 cells), `research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out` (PC3 differential), `e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` (Segment 2, three knob groups × 11 geometry points × 21 streams × avg 54 batches = 12551 cells). Re-run: `bash scripts/experiment-apparatus/replay.sh E16R5`. Mutation table `scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv`, all 17 items caught during fifth run (table has 18 items, see Sixth Run section) (`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e16-fifth-run-peak-under-today-widths e16_fifth_run_peak_under_today_widths.rs ../mutations/e16_fifth_run_peak_under_today_widths.tsv`; added item 17 anchors Segment 2 record width knob's new `RecordFormat` branch).

**What it cannot answer**: `crates/` publication names every rewritten unit (not just dirty leaves); a single write modifies other trees (upper-stage leaves, tree table, central mapping tree, allocation record tree, accounting tree)—these two structural differences were unmodeled here, altering actual block counts per fsync, but are not "width" and outside this scope; Q1 classification "encryption" (densification) re-evaluation step (encrypting integers between adjacent batch values to verify classification was not sampling artifact) was omitted, sampling only two boundary-prone spots; whether root children linearly affects Class C start at intermediate values (e.g. 30, 50) was not swept—taking only 2, 11, 17, 147 suffices to judge "who the governing variable is", but not "exact functional relationship".

### Sixth Run: Named Entries 67 → 64 (2026-10-04, Fifth Run Apparatus and Geometry Kept Intact)

<!-- doc-lint:not-numbers PC1 PC2 PC3 PC4 F7 S1 V7 -->
PC1–PC4, F7, S1, V7 are control / failure / shutdown / voiding clause numbers internal to pre-run registries `e16-r5-prereg.md` and `e16-r6-prereg.md`, while G1–G5 are geometry labels from same registry, none being kb identifiers.
D23 (Journal Role & Format) settled item 4 sets record header to 487 (new root segment 364), where a 4096 record accommodates 64 named entries (settled items 12, 17); in apparatus Arm B names k leaves per fsync writing ⌈k ÷ 64⌉ records (fifth run was ⌈k ÷ 67⌉). Geometries, semantics of both arms, criteria, tolerances, and bandwidth remain unchanged; re-run registered in `e16-r6-prereg.md`, replay key remains `E16R5`, issue tracking `m3-r1-batch9-reruns-questions.md` row 4.
Modifications limited to: `JOURNAL_HEADER_BYTES` 487 and `JOURNAL_NAMED_ENTRIES_PER_RECORD` 64 (with three comment paragraphs), batch axis merged with 63 and 65 (total 81 batch points, straddling threshold 64 / 65), four unit test pinned values (named entries, Arm B block counts on B=63–73 across nine batches, header width, PC4 batch set {1, 2, 5, 10, 20, 50, 64, 65, 67}), mutation table (deleted "67→72", added "64→72" and "64→67", 18 items total). All 23 unit tests green (`cargo test --release --bin e16-fifth-run-peak-under-today-widths` output `test result: ok. 23 passed`).

**Baselines and Shutdown Clauses**: Prior to modifying apparatus, `bash scripts/experiment-apparatus/replay.sh E16R5` judged all three lines "byte-identical" (apparatus 311 / 67, `crates/` 487 / 64, registered hardcoded exception: comparison baseline only, no conclusions); after modification but before artifact generation, nine lines of output from `grep -nE '^pub const (DATA_UNIT_BYTES|DATA_UNIT_PAYLOAD_OFFSET|EXTENT_TREE_INTERNAL_FANOUT|EXTENT_TREE_LOWER_LEAF_DATA_UNITS|JOURNAL_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NAMED_ENTRY_BYTES|JOURNAL_NAMED_ENTRIES_PER_RECORD|TEST_IMAGE_DEFAULT_BYTES):' crates/singlefs-format/src/lib.rs` matched command 2 of Section 13 verbatim (S1 passed).
PC1 and PC2 (original apparatus current run and error denominator verification) were unrun: original apparatus had zero changes and does not read named entry count. PC3 passed: `pc3` mode artifact was byte-for-byte identical to registered `e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out` (58 lines), so no new pc3 artifact produced, keeping `replay.sh` pc3 line intact.

**Artifacts** (after updating `replay.sh` registry line, `bash scripts/experiment-apparatus/replay.sh E16R5` judges all three lines "byte-identical", V7 untriggered):

```
$ grep -oE '^E7RESULT name=[a-z_0-9]*' research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out | sort | uniq -c
   3543 E7RESULT name=cell
      1 E7RESULT name=config
      1 E7RESULT name=done
      3 E7RESULT name=geometry_config
     21 E7RESULT name=geometry_sensitivity
      3 E7RESULT name=q1_summary
      3 E7RESULT name=q2_summary
      6 E7RESULT name=q3_summary
      6 E7RESULT name=q4_summary
     63 E7RESULT name=row_summary
```

Segment 1 `e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` (3650 lines, `name=done emitted=3650`; fifth run was 3530 lines); Segment 2 `e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out` (13366 lines, `name=done emitted=13366`; fifth run was 12926 lines, `name=cell` 12991 lines, `geometry_config` 11, `q1_summary` 11, `row_summary` 231).
Comparing cell-by-cell against fifth run artifacts (Q7, `diff <(grep 'name=cell' old) <(grep 'name=cell' new)`): Segment 1 deleted 654 lines and added 774 lines, the 120 added lines corresponding entirely to new batches 63 and 65; Segment 2 deleted 1962 lines and added 2402 lines, the 440 added lines also solely on batches 63 and 65. Cells differing between segments fall on the 18 batches where `⌈B ÷ 64⌉ ≠ ⌈B ÷ 67⌉` (65, 67, 129, 193, 200, 257, 385, 513, 769, 1025, 1536, 2048, 3072, 4096, 6144, 8192, 12288, 16384), with modified fields restricted to Arm B three block counts, ratio, and two error denominators; cells with B ≤ 64, existing batches of two record-width samples (1 entry/record and 10⁶ entries/record), and geometry config lines remain verbatim identical, with 0 points outside permitted set. **Artifacts altered.**

**Q1–Q6 Re-evaluated by Geometry Point (Compared Against Fifth Run Artifact Verdicts)**:

| Geometry Point | Q1 Peak Placement Classification | Q2 2D Maximum | Q3 Error (Two Denominators, 6 Rows Total) | Q5 | Q6 ①② | Q6 ③ Rows with Minimum Ratio Dipping Below 1 |
|---|---|---|---|---|---|---|
| G1 | 21 rows all Class C, unchanged | Unchanged | Unchanged | Unchanged | Unchanged | 14 → 15 (added streams=128 row) |
| G2 | Partitioned (streams 2–16 Class A, 24–1024 Class C), unchanged | Unchanged | Unchanged | Unchanged | Unchanged | 12 → 13 (added streams=64 row) |
| G4 | Partitioned, **streams=128 row changed from wide plateau to Class B**, other rows unchanged | Unchanged | Unchanged | Unchanged | Unchanged | 11 → 11 |

Segment 2 six sample points: G3 (`full_root_level_four`), G5 (`two_fifty_six_mebibyte_file`), fanouts 128 / 296, records 1 entry/record and 10⁶ entries/record, row classifications (Q1), Q3, Q5, Q6 ①② remain unchanged; G3 2D maximum (Q2) changed from 3.985294 (batches 56–67) to 3.984615 (batches 56–64), both axes remaining within interior (`streams_interior` and `batch_interior` still `true`), upper bound of peak set on G3 rows 96, 128, 192, 256 changed from 67 to 64 (peak ratio dropped slightly, e.g. row 96 from 3.985294 to 3.984615); rows dipping below 1: in Segment 2 tree height knob G1 and G2 match Segment 1 table (14 → 15, 12 → 13), two other copies of 1 TiB (fanout knob, record width knob) each 12 → 13, fanout 296 point 13 → 14 (added streams=96 row), other points unchanged. The 10⁶ entries/record point matched 64 entries/record 21 row categories row-by-row (matching 67 entries/record in fifth run as well); 1 entry/record point had all 21 rows in Class C, unchanged.
Full line evidence (artifact verbatim):

```
research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out:3496:E7RESULT name=row_summary geometry=sixteen_tebibyte_file streams=128 classification=宽平台 peak_ratio=4.013520 peak_batch_low=5 peak_batch_high=67 moved_diagonal_ratio=NA rising_steps_before_peak=0 falling_steps_after_peak=21 opposite_direction_steps=0 final_batch=16384 final_ratio=1.015437 ratio_non_decreasing_through_streams_holds=false worst_drop_fraction_through_streams=0.014368 worst_drop_batch_through_streams=68 ratio_non_increasing_beyond_streams_holds=true settles_near_one_beyond_sixteen_streams_holds=true settle_beyond_sixteen_streams_batch=2048 settle_beyond_sixteen_streams_margin=0.178114 row_minimum_ratio=1.015437 row_minimum_batch=16384 row_dips_below_one=false
research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out:3616:E7RESULT name=row_summary geometry=sixteen_tebibyte_file streams=128 classification=乙行 peak_ratio=4.013520 peak_batch_low=5 peak_batch_high=63 moved_diagonal_ratio=0.0391 rising_steps_before_peak=0 falling_steps_after_peak=22 opposite_direction_steps=0 final_batch=16384 final_ratio=1.014766 ratio_non_decreasing_through_streams_holds=false worst_drop_fraction_through_streams=0.014449 worst_drop_batch_through_streams=65 ratio_non_increasing_beyond_streams_holds=true settles_near_one_beyond_sixteen_streams_holds=true settle_beyond_sixteen_streams_batch=2048 settle_beyond_sixteen_streams_margin=0.177548 row_minimum_ratio=1.014766 row_minimum_batch=16384 row_dips_below_one=false
research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out:2325:E7RESULT name=row_summary geometry=one_tebibyte_file streams=64 classification=丙行 peak_ratio=3.500000 peak_batch_low=1 peak_batch_high=1 moved_diagonal_ratio=NA rising_steps_before_peak=0 falling_steps_after_peak=30 opposite_direction_steps=0 final_batch=16384 final_ratio=1.000451 ratio_non_decreasing_through_streams_holds=false worst_drop_fraction_through_streams=0.047619 worst_drop_batch_through_streams=2 ratio_non_increasing_beyond_streams_holds=true settles_near_one_beyond_sixteen_streams_holds=true settle_beyond_sixteen_streams_batch=1024 settle_beyond_sixteen_streams_margin=0.124760 row_minimum_ratio=1.000451 row_minimum_batch=16384 row_dips_below_one=false
research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out:2405:E7RESULT name=row_summary geometry=one_tebibyte_file streams=64 classification=丙行 peak_ratio=3.500000 peak_batch_low=1 peak_batch_high=1 moved_diagonal_ratio=NA rising_steps_before_peak=0 falling_steps_after_peak=29 opposite_direction_steps=0 final_batch=16384 final_ratio=0.999790 ratio_non_decreasing_through_streams_holds=false worst_drop_fraction_through_streams=0.047619 worst_drop_batch_through_streams=2 ratio_non_increasing_beyond_streams_holds=true settles_near_one_beyond_sixteen_streams_holds=true settle_beyond_sixteen_streams_batch=1024 settle_beyond_sixteen_streams_margin=0.124760 row_minimum_ratio=0.999790 row_minimum_batch=16384 row_dips_below_one=true
research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out:4673:E7RESULT name=q2_summary geometry=full_root_level_four maximum_ratio=3.985294 streams_lower=64 streams_upper=128 batch_lower=56 batch_upper=67 streams_interior=true batch_interior=true
research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out:4833:E7RESULT name=q2_summary geometry=full_root_level_four maximum_ratio=3.984615 streams_lower=64 streams_upper=128 batch_lower=56 batch_upper=64 streams_interior=true batch_interior=true
```

⇒ **Verdicts altered (registered F7)** in two specified locations: G4 Q1 at streams=128 changed from wide plateau to Class B (peak set [5, 67] became [5, 63], `B_lo ÷ S` = 0.0391, new threshold 64 falling outside lower edge of `[S ÷ 2, 2S]` = [64, 256]); Q6 ③ "minimum ratio dipping below 1" gained one row (G1 streams=128, G2 streams=64, both with minimum ratio at batch 16384: 1.000192 → 0.999531, 1.000451 → 0.999790, changing from "not dipping" to "dipping below 1"). Other cells (G1, G2 Q1, three geometries Q2, Q3, Q5, Q6 ①②) unchanged. This is a legitimate finding; retroactive modifications to tolerance, bandwidth, or batch axis are forbidden.
Record width knob Q1 verdicts between 67 and 64: G1, G2 do not flip; G4 streams=128 flips (varying with record width, see "Verdicts altered (registered F7)" paragraph); Segment 2 six sample points do not flip. Cross-geometry stability rows (`name=geometry_sensitivity`) matched verbatim across 20 of 21 rows, `streams=128` row `sixteen_tebibyte_file` changing from wide plateau to Class B, `stable=false` intact; `stable=true` rows remained 6 both times (Segment 2 remained 27).
Q1 final sentence "densification re-verdict" was omitted again this round (omitted since fifth run).

**Mutations**: Mutation table 18 items pre-modification (old apparatus, old table 17 items) and post-modification full table runs conducted by main agent, logs `e16-mutate-before-2026-10-04-h487.log`, `e16-mutate-after-2026-10-04-h487.log`, command: `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e16-fifth-run-peak-under-today-widths e16_fifth_run_peak_under_today_widths.rs ../mutations/e16_fifth_run_peak_under_today_widths.tsv`.

```
$ for p in before after; do f=research/results/e16-mutate-$p-2026-10-04-h487.log; echo "$p: $(for s in '✅' '⏭' '❌' '💥' '⚠️' '⏱' '🧱'; do printf '%s=%s ' "$s" "$(grep -c "^$s" $f)"; done)"; done
before: ✅=17 ⏭=0 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0 
after: ✅=18 ⏭=0 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0 
$ grep -E '^(计数：|已还原)' e16-mutate-after-2026-10-04-h487.log
计数：内存撞顶 0 条（上限 2G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条
已还原，基线仍全绿
```

Pre-modification: **caught 17, invalid 0, not red 0** (17 in table); post-modification: **caught 18, invalid 0, not red 0** (18 in table); sums match respective table row counts, concluding with baseline fully green, exit code 0; memory cap, timeout, queue fail, total cap drops all 0. Registered voiding clause V6 (mutation table incomplete voids only "all N caught" statement, not artifacts) untriggered: both completed full table. Registered Section 9 "except two rows, other 16 must have identical status": comparing logs by mutant name shows differences exactly matching registry—pre-run had "record packing old width items (67→72)" absent in post-run; post-run had "record packing previous gen items (64→67)" and "record packing old width items (64→72)" absent in pre-run; remaining 16 items were "caught" both times; two added items were both "caught", invalid count remaining 0 vs 0.

### A Deduction: Group Commit and "Reducing fsync Frequency" are Identical on the Device Write Side

**This is deduction, not empirical measurement**, but governs how to read the four-workload comparison table in "Multistream Workload":

Under checkpoint publish semantics, a single publish writes the COW set of **this batch's dirty set** + root slot + one record.
"Batching C operations before issuing one fsync" versus "C concurrent fsyncs served by one publish"—
**both share identical dirty sets ⇒ device writes are completely identical**. They differ in only two aspects:

| Difference | Explanation |
|---|---|
| **Denominator** | "Batching C operations before fsync" serves 1 fsync call per publish; "C concurrent fsyncs served by one publish" serves C. ⇒ When discussing **cost per fsync**, the denominator must be clarified |
| **Which operations batch into the same batch** | Concurrent threads batch operations from **different streams**; sequential frequency reduction batches adjacent operations from the **same stream**. ⇒ **This aspect truly impacts outcomes**, and is a property of workload, not concurrency itself |

⇒ **`multistream` is the extreme case of the second aspect**, used by E16 (Role of Journal: WAL vs Intent Log) to cover "group commit fully available",
rather than modeling threads directly. ⚠️ **It must not impersonate latency under real concurrency**—E16 (Role of Journal: WAL vs Intent Log) calculates only block counts.

### Four Decisive Figures from the 08-28 Round

1. **Without fsync, write amplification across all three arms is cell-by-cell identical** (Positive Control 1). ⇒ **Marginal gain of WAL is exactly 0**—
   what it buys exists only where group commits cannot reach.
2. **`wal_leaf` is the sole form buying substantial write amplification gains**: blocks per fsync drop to 1/3.5 (rand 7→2),
   1/3.3 (metaheavy 10→3), reducing write amplification by **35–57%**.
3. **Ring size differs by 3 orders of magnitude**: `intent` is constantly 48 bytes (independent of throughput, workload, interval),
   the two WAL arms require 10⁴–10⁵ bytes, with **lower bound proportional to fsync rate (`wal_full`) or throughput (`wal_leaf`)**.
   ⚠️ **Scope annotation (2026-09-03, caught by the backward leg of the three-way argument)**: these two figures were measured under this model's
   measurement basis where "records only record 48-byte cursor"; D23 (Journal Role & Format) settled item 12 (2026-09-01) fixed records to **4 KiB fixed length +
   named entries** ⇒ under settled format, both arms write one 4 KiB record per fsync with identical slot consumption rates,
   the true difference lying in **retention duration** (Arm A records die upon root publish; Arm B records must survive until checkpoint).
   Recomputed accounting under settled format resides in "Maintenance Cost Half".
4. **Ancestor amortization: 0.0567 blocks/operation on multistream workload with checkpoint interval 1000** (2026-08-31 supplement).

   **Why supplemented**: [decisions.md](../decisions.md) D10 (True Answer to Random Writes / Fragmentation) took "intra-checkpoint amortization compressing ancestors to
   **0.069 blocks/operation**" as evidence to unblock format freeze, yet that number
   **never existed in E16 (Role of Journal: WAL vs Intent Log) artifacts**—
   checked on 2026-08-31 across four fields and relaxed intervals of the 08-28 artifact with **zero hits**.
   ⇒ Added ancestor block counter and re-ran to make it a measurable number.

   | Workload | ckpt Interval | Ancestor Blocks/Op (Ancestors Deferred to Checkpoint) |
   |---|---|---|
   | `seq` | 1000 | **0.0083** |
   | `seq` | 100 | 0.0126 |
   | **`multistream`** | **1000** | **0.0567** |
   | `multistream` | 100 | 0.4966 |
   | `metaheavy` | 1000 | 0.6476 |
   | `rand` | 1000 | 2.0990 |

   ⚠️ **Measurement basis is load-bearing; citing the naked number alone is forbidden**: the ancestor amortization table measures the side where **ancestors are deferred to checkpoint**
   (`wal_leaf`, or any arm not publishing roots on every fsync).
   **When ancestors are not deferred, the same cell is 4.0000 blocks/operation** (`intent` / `wal_full` every fsync)—**a 70× difference**.
   ⇒ "Intra-checkpoint amortization compressing ancestors to zero point zero something" holds only on premise of **deferring ancestors**;
   treating it as an inherent property of coarse workloads discards the premise.

   ⇒ **D10 (True Answer to Random Writes / Fragmentation)'s `0.069` should be `0.0567`** (multistream / ckpt=1000).
   Direction unchanged (slightly smaller than 0.069). ⚠️ **`0.069` has zero hits in E16 (Role of Journal: WAL vs Intent Log) artifacts or prose**—
   E16 (Role of Journal: WAL vs Intent Log) never produced that number; citing amortization must cite `0.0567`.

   **Internal control for this change**: the addition was solely a counter; **existing 78 result lines with the new field stripped are
   byte-for-byte identical to the 08-28 artifact**—adding the counter disturbed none of the existing numbers.

5. **Root ring slot consumption rate differs by 1000×**: `intent` publishes a root on every fsync (200,000 times),
   while WAL arms publish only at checkpoints (200 times). Root ring slots are finite (D22 (Synthesizing Unit Atomicity) settled item 2 sets R=3, slot count per region residing in system configuration); this represents a resource rate disparity,
   not merely a block write count disparity.

### Three Modeling Errors in Earlier Versions, All Caught by the Other Two Legs of Third-Party Review

**The most important takeaway from the 08-28 round is not its four decisive figures, but the shape of these three errors.**

| # | Error | Consequence | Who Caught It |
|---|---|---|---|
| 1 | `wal_full` records billed by **dirty leaves**, whereas D23 (Journal Role & Format) explicitly specifies records name **subtree roots** (bounded by fanout ≤128) | Overreported ring peak by up to **47×**, where ring size was the very quantity E16 (Role of Journal: WAL vs Intent Log) sought to produce | Both cloud legs caught **independently** |
| 2 | After `wal_full` fsync cleared dirty set, checkpoint trigger inspected only dirty set | **Not a single root published across 200,000 operations** (`root=0 ckpts=0`), invisible to conservation checks, inter-arm comparisons, unit tests | Forward-deduction leg |
| 3 | Clearing ring without publishing root | Underreported `wal_full` ring peak by **4–5 orders of magnitude** | Forward-deduction leg |

⇒ **Common pattern of all three: checks compared the three arms against each other without pinning absolute values.** When all three arms failed together,
relative comparisons remained equal. Remedied by adding to conservation check: "every arm must publish root at least once every `ckpt_interval` operations",
and writing independent unit tests for record billing and ring truncation rules.

### A Format Requirement Forced Out by the 08-28 Round

> ⚠️ **Every item named in an fsync record must carry its own checksum.**

D4 (Checksum Location) settled checksums inlined into **parent pointers**, but for items named in an fsync record, **their parent is not yet on disk at this moment**—
`wal_full` names subtree roots whose parent is the unissued root; `wal_leaf` names leaves whose parent is an unwritten internal node.
⇒ **The record must temporarily act as parent**, otherwise those blocks have no integrity credentials during replay.
Each item therefore expanded from 24 to 56 bytes. **Checkpoint records do not need this**: root is published in the same checkpoint.
⇒ **This is an overhead forced into format by WAL, nowhere documented prior to the 08-28 round.**

### Post-Hoc Reformulation (2026-08-29): Four Workloads are Not Four Phenomena, but Four Points on a Single Formula

**This was derived backwards when D25 (Target Workload Priority) was settled, not originally tested by E16 (Role of Journal: WAL vs Intent Log).**

> **Blocks per fsync = touched leaves + Σ distinct nodes touched at each layer + 1 root slot + 1 record**

Zero fitting, closing cell-by-cell (arm=intent, ckpt_interval=1000):

| Workload | Batch | Predicted | Measured | Difference |
|---|---|---|---|---|
| rand | 1 | 1 + 4 + 1 + 1 = 7.00 | 7.00 | 0.00 |
| multistream | 1 | 1 + 4 + 1 + 1 = 7.00 | 7.00 | 0.00 |
| seq | 1 | 8 + 4 + 1 + 1 = 14.00 | 14.00 | 0.00 |
| metaheavy | 1 | 2 + 5.937 + 1 + 1 = 9.937 | 9.94 | 0.003 |
| seq | 10 | 80 + 4 + 1 + 1 = 86.00 | 86.50 | 0.50 |
| multistream | 10 | 10 + (10+10+10+1) + 1 + 1 = 43.00 | 43.00 | 0.00 |

The 5.937 of metaheavy is calculated probabilistically, not reverse-solved: hot zone = 1/1024 of leaves = 262,144 leaves;
two random hot leaves span 2048 nodes at layer 1, 16 at layer 2, 1 at layer 3 and root
⇒ expected distinct nodes = 1.9995 + 1.9375 + 1 + 1 = 5.937.

⚠️ **This yields a critique of E16 (Role of Journal: WAL vs Intent Log) design: `fsync_every=1` is a degenerate cell.**
There, dirty sets of any workload contain only a single spine ⇒ non-user blocks are constantly 6.00 (= HEIGHT 4 + root slot 1 + record 1),
**rendering stream count completely invisible**. Looking only at that tier concludes "stream count does not matter", a modeling degeneracy rather than an intrinsic property:
at `fsync_every=10`, non-user blocks for seq are **6.50**, while multistream is **33.00**, a 5× difference.

⇒ When using E16 (Role of Journal: WAL vs Intent Log) figures to support workload conclusions, **the batch tier must be explicitly stated**.
Settlement in [decisions.md](../decisions.md) D25 (Target Workload Priority).

### Measurement Basis and Known Limitations (Must Not Be Ignored)

- Measures **device blocks written, not time**. Real fsync latency includes FUA round trips and queue scheduling, **which E16 (Role of Journal: WAL vs Intent Log) cannot reach**—
  that half must wait for E12 (Batched Sequential Append vs Unbatched Random Page RMW) harness and transaction layer. **Using E16 (Role of Journal: WAL vs Intent Log) figures to discuss latency is misuse.**
- A record is billed as "occupying at least one block" (synchronous publish directives do not merge with subsequent records).
- ⚠️ **"Peak ring occupancy" does not equal "how large ring should be allocated".** E16 (Role of Journal: WAL vs Intent Log) measures occupancy; ring **lower bounds** have other origins:
  [decisions.md](../decisions.md) D23 (Journal Role & Format) Deadlock 3 requires "worst-case journal occupancy of any operation ≤ ring size / F, F ≥ 2",
  and C42 (Residual record posing as valid prefix) requires an explicit in-flight record cap (XFS equivalent being `XLOG_MAX_ICLOGS 8`,
  inspected 2026-08-28 in `fs/xfs/libxfs/xfs_log_format.h`).
  ⇒ **The 48 bytes of `intent` is occupancy, not that it can be sized at 48 bytes.**
- Geometry: complete 128-ary tree, 4 internal layers, 4 KiB block, 1 block root slot; record `48 + per_entry(24 or 24+32)` bytes;
  intent log record fixed 48 bytes (model parameter; settled format is 4 KiB fixed record, see D23 (Journal Role & Format) settled item 12 and scope annotation in "Maintenance Cost Half"). Leaf = user data block, one operation dirtying 1 (rand) / 2 (metaheavy) / 8 (seq) leaves.
  **Tree height, fanout, and record field widths are model parameters, not empirical measurements.**
- **COW cost of accounting blocks themselves is unmodeled** (D22 (Synthesizing Unit Atomicity) settled item 3 settled accounting structures follow COW, an expense absent from model). If scaling with checkpoint count,
  `intent` with the highest checkpoint count would see its disadvantage widen further.

### Experimental Discipline of the 08-28 Round

- Three arms **share identical operation stream**; matching fingerprints required for validity (mismatch voids entire round, exit 4).
- **Adding `multistream` workload left the 18 cells of the original three workloads cell-by-cell identical** (verified pairwise).
  Whether adding an arm or workload perturbs other cells is the most frequent error in this class of accounting model.
- `multistream` includes a unit test pinning discriminative power: first 16 operations must touch **16 distinct subtree roots**,
  whereas the same number of `seq` operations touch only **1**. ⚠️ Without pinning this, `multistream` could be a disguise for `seq`.
  Two mutants (hardcoding stream ID to 0, swapping stream and offset) proved it turns red.
- **Positive Control 1 run for every arm**: without fsync, COW writes and root slot writes across three arms must be cell-by-cell equal.
- **Fault injection performed on every arm**: injecting "omit journal write" causes conservation check to judge red, all three arms turning red.
- **All 24 unit tests proved to turn red via mutation testing**; tracked mutation table `scripts/mutation/tables/e16_journal.tsv`
  currently has all 9 items caught (3 from 08-31 ancestor count + 3 from 2026-09-03 byte basis +
  3 reconstructed 08-28 historical mutants from test comments: interval × 2, billing by dirty leaves, clearing ring without root publish).
  ⚠️ The verbal claim of "14 items caught" from 08-28 round only retains those 3 reconstructed items in table for replay—
  failure pattern of C40 (mutation table not saved) occurred on E16 (Role of Journal: WAL vs Intent Log) as well, recorded faithfully.
  ⚠️ Running mutations across three rounds revealed items where "not a single test turned red", exposing the same blind spot (only comparing mutually, not pinning absolute values).
- Two execution runs **byte-for-byte identical**; integrity gate `emitted=60` == 60 lines captured.

Code `scripts/experiment-apparatus/e16_journal.rs`, `cargo run --release --bin e16-journal`;
output `e16-journal-2026-08-31.out` (including ancestor counter added 2026-08-31;
stripping that field yields byte-identical match with `e16-journal-2026-08-28.out`)
and `e16-sweep-2026-08-28.out` (56-cell sweep).
Re-run: pass E16 (Role of Journal: WAL vs Intent Log) ID to `scripts/experiment-apparatus/replay.sh`.

### Byte Measurement Basis (2026-09-03 Supplement, Prerequisite for C27 (Format Parameters Settled on a Block-Count Measurement Basis))

**Why supplemented**: When C27 (Format Parameters Settled on a Block-Count Measurement Basis) was established, it noted that using block-count conclusions to settle fanout or
direction for D11 (Message Buffers in Index Nodes) could be inverted—larger nodes touch fewer **node counts** (shorter tree),
yet more **bytes** (each node larger). E16 (Role of Journal: WAL vs Intent Log) previously had only block-count basis.

**Criteria (Hardcoded Pre-Run)**:

1. Absolute value: ancestor **node count** for single-operation fsync must equal tree height independently calculated under that geometry
   (fanout = (node_bytes − 64) / 40, height = minimal h where fanout^h ≥ leaves);
   ancestor **bytes** must equal height × node_bytes.
2. Direction criterion (premise of C27 (Format Parameters Settled on a Block-Count Measurement Basis) itself): sweeping from 512 B to 64 KiB, ancestor node count is **monotonically non-increasing** while
   ancestor bytes are **higher at large node end**—two measurement bases must produce **different orderings**.
   If both bases order identically ⇒ premise of C27 (Format Parameters Settled on a Block-Count Measurement Basis) is invalidated, recorded faithfully with owed check revised.
3. Consistency: ancestor count calculated by parameterized geometry on (fanout 128, height 4) must match constant code of main grid
   point-by-point on identical dirty leaf set—pinned together by test, no drift permitted.

**Empirical Measurements (`e16-journal bytes`, 5 runs md5 identical; all three criteria met)**:

| Node Bytes | Fanout | Tree Height | Single-Op Ancestor Nodes | Single-Op Ancestor Bytes | Batch 10 Multistream Ancestor Nodes | Batch 10 Multistream Ancestor Bytes |
|---|---|---|---|---|---|---|
| 512 | 11 | 9 | **9** | **4.5 KiB** | 70 | 35 KiB |
| 1024 | 24 | 7 | 7 | 7 KiB | 52 | 52 KiB |
| 4096 | 100 | 5 | 5 | 20 KiB | 33 | 132 KiB |
| 16384 | 408 | 4 | 4 | 64 KiB | 24 | 384 KiB |
| 65536 | 1636 | 3 | **3** | **192 KiB** | 21 | 1.31 MiB |

(Leaves fixed at 128⁴ ≈ 268 million 4 KiB data blocks ≈ 1 TiB; fanout = (node_bytes − 64)/40.)

**Criterion 2 holds; premise of C27 (Format Parameters Settled on a Block-Count Measurement Basis) substantiated by E16 (Role of Journal: WAL vs Intent Log)'s own machine**:
512 B → 64 KiB saves 3× by node count (9→3), but is **42.7× more expensive** by byte count (4.5 KiB→192 KiB);
batch 10 multistream shows same direction (nodes 3.3× cheaper, bytes 38.4× more expensive).
**The two measurement bases order node sizes inversely** ⇒ citing any block-count conclusion of E16 (Role of Journal: WAL vs Intent Log) to settle
fanout, node size, D11 (Message Buffers in Index Nodes) or similar format parameters requires providing byte basis simultaneously.
Re-run: `cargo run --release --bin e16-journal bytes`, artifact
`e16-bytes-2026-09-03.out` (7 lines, ending `emitted=7`), byte-compared against second E16 (Role of Journal: WAL vs Intent Log) row in replay table.
One archival note: first version of config line bypassed Emitter, summary reporting 6 but receiving 7;
replay integrity gate turned red on the spot—direct evidence of gate operation.

### Maintenance Cost Half (2026-09-03, Tripartite Review, Criterion 3 Concluded)

**Verdict: Criterion 3 as originally posed (giving two numbers) cannot answer the question it aims to answer—three legs independently and consistently judged
"invariant count / crash point category count" is not a reasonable proxy for maintenance cost, and where inventory boundaries are drawn reverses counting direction.**

Three legs (prompts and source archived in `research/prompts/`: `_e16-maintenance-background.md`,
`e16-maintenance-local-output.md`, `e16-maintenance-sonnet-output.md`,
`e16-maintenance-opus-output.md`):

| Leg | Stance | Count | Verdict on Proxy Metric |
|---|---|---|---|
| Local (Qwen) | Counterexample | Arm A: 10 invariants / 6 crash classes; Arm B: 14 / 10 | Unfair: Arm B allocator implementation difficulty and Arm A root ring wear cannot fit into item count |
| Cloud A (Sonnet) | Forward Deduction | Arm A: 15 / 12; Arm B: 20 / 14 | Direction reliable, magnitude systematically underestimated: expensive parts of Arm B almost entirely fall on "requiring second independent implementation to verify" |
| Cloud B (Opus) | Counter Deduction | Factoring in obligations leaking from publish rate, **Arm A has more** (~ +7 vs +4) | Ordering reversible ⇒ question formulation fundamentally flawed |

**Divergence Points and Verification**: Inconsistent counting direction stems from inventory boundary—counting crash correctness obligations solely on journal structures
(Local, Forward), Arm B has more; factoring in obligations leaking from "publish rate = fsync rate" into accounting and root ring (Counter), Arm A has more.
Both boundaries are valid ⇒ proving counts are unusable. **This is not an illusion of consensus**: reasons across the three legs differ mutually
(implementation difficulty / independent verification hurdle / boundary dependence); only the verdict is unanimous.

**Load-Bearing Structural Answer (Three legs arrived at the same point from different directions)**:
Decisive cost of Arm B is **replay requiring an allocator ⇒ O2 of D13 (Verification Route) (independent parser + checker)
must implement an allocator independently**—the sole surviving reason after D23 (Journal Role & Format) rejected logical intent logging,
inherited verbatim by Arm B; this relies on no item counts. Arm A's corresponding balance is **publish rate = fsync rate** leaking into other structures:
Rider Clauses 2 and 4 of D16 (Publish Semantics) settled item 6 (generation segment ≥ 48 bits, cross-publish merge failure) exist only under Arm A;
protection window of I-7.4 (Blocks in Recent K Generations Not Reused) collapses to milliseconds under 2785 publishes/s, while "worst-case block reuse delay requirements"
is absent from repository (E50 (Upper and Lower Bounds on Root Ring Slot Count) stated it cannot be compared); every fsync has one more ordering point than Arm B (root slot FUA).

**One error in background material, caught by the backward leg and verified verbatim by main agent**: materials fed to three legs
described "fixed 48-byte record" in E16 (Role of Journal: WAL vs Intent Log) model as Arm A's settled format, whereas D23 (Journal Role & Format) settled item 12
settled on **4 KiB fixed records** (Local and Forward legs inherited this premise, contaminating counting details;
structural argument does not depend on record size). Scope annotations added to Item 3 of "Decisive Four Figures" and geometry line of "Measurement Basis and Known Limitations".

**The two balances hung out here were cleared by E91 (Ring and Admission Accounting Under Settled Format)**:
① Under settled format, true difference in retention duration is 1 vs 10,923 records (4 KiB vs 42.7 MiB, T_dirty branch arriving first);
② Arm A's unfillable watermark at T_time = 5 s full window, delayed 8 generations, is 501,364 blocks ≈ 1.91 GiB
(deferred form 28 MiB, ratio 70.6×). Figures and measurement basis reside in E91 (Ring and Admission Accounting Under Settled Format) text.

**Does not reopen D23 (Journal Role & Format) settled item 1**—Axis 2 was structurally eliminated by Axis 1 (deferring ancestors ⇒
new root cannot be self-consistent ⇒ cannot publish root), independent of cost magnitude; three legs agreed on this point.

### The Half Not Yet Done

- **fsync Latency Distribution** (p50/p99, real time): requires devices, requires transaction layer.
- (The two maintenance cost recalculations were cleared by E91 (Ring and Admission Accounting Under Settled Format), 2026-09-03.)
- (Segment 2 under fifth run unit widths—G3, G5, fanout 128/296, records 1 entry and 10⁶ entries—supplemented 2026-09-28: "tree height" and "children under root" separated; at same height (L=4), raising root children from 11 to 147 delays peak disappearance from stream count 24 to 384; governing variable is root children, not tree height. Remaining is intermediate values of root children (e.g. 30, 50) not swept, only judging "who governing variable is", not "exact functional relationship".)

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 1 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 17 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D10 (True Answer to Random Writes / Fragmentation) | Preparation | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) | Preparation | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D13 (Verification Route) | No Effect | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | Preparation | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) | No Effect | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 1 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 2 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 3 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 4 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 5 | Support | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) | No Effect | 2026-10-26 unaffected: only the trilingual language-switch row and markup were updated; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E16 (Role of Journal: WAL vs Intent Log) are collected in [experiments-history.md](../experiments-history.md).
