## E167 Crash Enumeration Partitioning for Multiple In-Flight Publishes — partially run (one cell added in segment three) (2026-10-01)

[中文](167-多在飞发布的崩溃枚举切法.md) · **English** · [日本語](167-複数の実行中公開に対するクラッシュ列挙の区切り方.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q7 K1 K2 P1 P2 P3 G1 G2 G3 G4a G4b G5 L3a L3b L7a L7b S1 S2 S3 F1 F2 F3 F4 F5 F6 V1 V2 V3 V4 V5 V6 PC1 PC2 PC3 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 M29 M30 M31 M32 M33 M34 M35 -->

```text
Local arm names and notation on this page use lowercase (l1, l2, l4, l5, ...). Preregistration, apparatus, artifacts and commands use the corresponding uppercase forms, which resemble global registration identifiers. KB prose uses lowercase only to distinguish them; match uppercase when reading artifacts.
```

Preregistration `e167-preregistration.md` (section 12 amendments 1–6, segment two amendments 7–9, segment three amendment 10); Q7 of `m3-concurrency-questions.md` asks how crash enumeration should cut multiple in-flight publications. Q1's three candidates P1 (parallel preparation, serial persistence), P2 (multiple transactions in one publication), P3 (multiple publications in flight) are separated by cell, not judged. The apparatus is an independent hand-written counting model `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs` under `research/`, sharing no code with `crates/` and not timing execution; mutation table `scripts/mutation/tables/e167_multi_inflight_crash_cut.tsv`.

Artifacts (under `research/results/`):

- `e167-multi-inflight-crash-cut-2026-10-01-r2.out`: official first-segment artifact, 200 `E7RESULT` lines, `void_conditions=0 halt_conditions=0`. Replay registration points to it.
- `e167-multi-inflight-crash-cut-2026-10-01.out`: first run that day, `void_conditions=1` (`void_list=threshold_shift:l6`). The l6 discriminative power self-test shifted the threshold to the minimum K1 coverage count, which is zero; amendment 5's zero-denominator rule classified the shifted result as `no_invalid_prefix_in_truth`, not `covers_every_invalid_prefix`, and the self-check found no flip. The apparatus was changed to shift to maximum K1 coverage (13), then saved as `-r2`. Except `threshold_shift`, `scope=run`, and `name=done` rows, the two match line by line (diff checked). The first output is invalid under registration but retained.
- `e167-multi-inflight-crash-cut-mutate-2026-10-01.log`: verbatim mutation-table output on the source before the l6 change (12 caught); `e167-multi-inflight-crash-cut-mutate-2026-10-01-r2.log`: verbatim output on final source, the source of the three mutation counts.
- `e167-multi-inflight-crash-cut-python-recount-2026-10-01.py`, its `.out` and `-two-state.out`: independent recount script (registration section 1 definitions only, does not read the Rust apparatus) and verbatim outputs for three-state system configuration slot writes (`.out`) and two-state writes (`-two-state.out`, `THIRD=0`); cell values pinned in unit tests come from this script.
- Segment-two artifacts: `e167-multi-inflight-crash-cut-second-2026-10-01-r3.out`, the official result, has 137 `E7RESULT` lines, `void_conditions=0 halt_conditions=0`, and is registered for replay (E167 `|e167-multi-inflight-crash-cut|second|…|exact`). `e167-multi-inflight-crash-cut-second-2026-10-01-r2.out` is byte-identical (cmp checked). The first run `e167-multi-inflight-crash-cut-second-2026-10-01.out` has `void_conditions=1` (`void_list=threshold_shift:l6:g4a_free`) and is retained as invalid: l6's apparatus self-check shifted only the “best stream” side, which already judged coverage and did not flip; this was a self-check error. It was changed to shift each direction once and shift the K2 side additionally, producing `-r2` and `-r3`; except `threshold_shift`, `scope=run`, and `name=done`, these match the first run line by line (cmp checked).
- `e167-multi-inflight-crash-cut-s3-estimate-2026-10-01.out`: scale estimate required by S3 (`s3` subcommand, six lines). `e167-multi-inflight-crash-cut-mutate-2026-10-01-r3.log`: first mutation run after adding ten segment-two mutations M13–M22; M18 did not turn red (21 caught, one not red of 22). `e167-multi-inflight-crash-cut-mutate-2026-10-01-r4.log`: final-source run after adding a unit test, source of the three mutation counts.
- `e167-multi-inflight-crash-cut-ordered-count-python-2026-10-01.py` and its `.out`: independent Σ_ord interleaving count (registration section 1 and segment-two amendment 7 only; does not read Rust apparatus) and verbatim output.
- Segment-three artifacts (adds cell l11): `e167-multi-inflight-crash-cut-third-2026-10-01.out`, 30 `E7RESULT` lines, `void_conditions=0 halt_conditions=0`, registered for replay; `e167-multi-inflight-crash-cut-mutate-2026-10-01-r5.log`, verbatim final-source output for 35 mutations; `e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.py` and its `.out` (one disk), `e167-multi-inflight-crash-cut-python-l11-two-disk-recount-2026-10-01.py` and `-a-ordered.out`, `-b-ordered.out`, `-a-free.out` (two disks, one unit): independent l11 recounts and verbatim output.

### What segment one did

- Stop conditions S1 (two registry entries) and S2 (manual publication attribution) were actually run with the `halt` subcommand before output. For S1 the model counted both enumerated deduplicated vectors and closed forms for `24+2+1+2`, its kind string, 35 operations, 16777223 states, and the entire stream `2+2+1+2+2+1+2+24+2+1+2`, 57 operations, 16777240 states; all matched. For S2 model attribution on the serial stream matched a manual derivation of `publishes_of_one_recording` in `crates/singlefs-harness/src/memory_pool.rs` item by item. S1 also read (without running) the tests pinning the registration table: `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` pins `(35, "24+2+1+2".to_string(), 16_777_223)` and `(57, "2+2+1+2+2+1+2+24+2+1+2".to_string(), 16_777_240)`; `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` pins `const FULL_STATES: u64 = 16_777_260;`.
- Calculated all section-seven class-C anchors (five values each for G1, G3, G4a, G4b, G5) and arithmetic for capability cells. G4a/G4b calculate only anchors in segment one; P3 two-disk cells belong to segment two.
- Twelve cells: G1 (one disk, u = 2, three-state system configuration slot write), G3 (one disk, u = 1), G5 (G1 with two-state system configuration slot write), each with four candidate cells (P1, P2, P3 free interleaving, P3 Σ_ord). G1 Σ_ord is G2 in registration. Each cell reports truth (two algorithms), K1 per-stream distribution, K2, two invalid-prefix levels, covering segment strings, capability, and exact reachable set.
- Positive controls PC1, PC2, PC3, discriminative power self-test, geometry sensitivity and each failure condition were judged. l8, l9 and G4a/G4b P3 cells were not run (segment two or omitted after sufficient evidence).

### Result lines copied intact from artifacts

G1's two P3 cells (free interleaving and Σ_ord), `research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out`:

```
56:E7RESULT name=cell_truth cell=g1_free sampling_point=G1 candidate=many_publishes_in_flight_free disks=1 units=2 table_writes=10 streams=12870 truth_by_enumeration=64 truth_by_search=64 methods_agree=true cross_transaction_states=49
57:E7RESULT name=arm_by_root_publish cell=g1_free streams=12870 states_minimum=15 states_maximum=29 states_on_serial_stream=15 states_on_minimum_cross_miss_stream=29 closed_form_mismatches=0 cross_misses_minimum=27 cross_misses_maximum=42 streams_with_cross_misses=12870 minimum_cross_miss_stream=a1.a2.b1.b2.|.rA.|.|.rB.RA.|.RB.SA.SB.|.| non_cross_misses_maximum=11 streams_with_non_cross_misses=12870
58:E7RESULT name=arm_by_transaction cell=g1_free states=64 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=0 identity_ok=true
59:E7RESULT name=invalid_prefix cell=g1_free kind=recorded truth_count=16 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=13 arm_by_transaction_covered=16 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=covers_every_invalid_prefix
60:E7RESULT name=invalid_prefix cell=g1_free kind=all_writes truth_count=6 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=6 arm_by_transaction_covered=6 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=covers_every_invalid_prefix
65:E7RESULT name=cell_truth cell=g1_ordered sampling_point=G2 candidate=many_publishes_in_flight_ordered disks=1 units=2 table_writes=10 streams=3747 truth_by_enumeration=46 truth_by_search=46 methods_agree=true cross_transaction_states=35
66:E7RESULT name=arm_by_root_publish cell=g1_ordered streams=3747 states_minimum=15 states_maximum=29 states_on_serial_stream=15 states_on_minimum_cross_miss_stream=29 closed_form_mismatches=0 cross_misses_minimum=13 cross_misses_maximum=28 streams_with_cross_misses=3747 minimum_cross_miss_stream=a1.a2.b1.b2.|.rA.|.|.rB.RA.|.RB.SA.SB.|.| non_cross_misses_maximum=7 streams_with_non_cross_misses=3747
67:E7RESULT name=arm_by_transaction cell=g1_ordered states=64 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=18 identity_ok=true
68:E7RESULT name=invalid_prefix cell=g1_ordered kind=recorded truth_count=1 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=1 arm_by_transaction_covered=1 arm_by_transaction_not_in_truth=15 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=covers_every_invalid_prefix
69:E7RESULT name=invalid_prefix cell=g1_ordered kind=all_writes truth_count=2 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=2 arm_by_transaction_covered=2 arm_by_transaction_not_in_truth=4 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=covers_every_invalid_prefix
```

The two actual-baseline (P1 serial, one disk, u = 2) and P2 (multiple transactions in one publication) cells:

```
38:E7RESULT name=cell_truth cell=g1_serial sampling_point=G1 candidate=serial_pipeline disks=1 units=2 table_writes=10 streams=1 truth_by_enumeration=15 truth_by_search=15 methods_agree=true cross_transaction_states=7
39:E7RESULT name=arm_by_root_publish cell=g1_serial streams=1 states_minimum=15 states_maximum=15 states_on_serial_stream=15 states_on_minimum_cross_miss_stream=15 closed_form_mismatches=0 cross_misses_minimum=0 cross_misses_maximum=0 streams_with_cross_misses=0 minimum_cross_miss_stream=a1.a2.|.rA.|.RA.SA.|.b1.b2.|.rB.|.RB.SB.| non_cross_misses_maximum=0 streams_with_non_cross_misses=0
40:E7RESULT name=arm_by_transaction cell=g1_serial states=64 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=49 identity_ok=true
47:E7RESULT name=cell_truth cell=g1_one_publish sampling_point=G1 candidate=one_publish_many_transactions disks=1 units=2 table_writes=8 streams=1 truth_by_enumeration=22 truth_by_search=22 methods_agree=true cross_transaction_states=15
48:E7RESULT name=arm_by_root_publish cell=g1_one_publish streams=1 states_minimum=22 states_maximum=22 states_on_serial_stream=not_defined states_on_minimum_cross_miss_stream=22 closed_form_mismatches=0 cross_misses_minimum=0 cross_misses_maximum=0 streams_with_cross_misses=0 minimum_cross_miss_stream=a1.a2.b1.b2.|.rA.rB.|.R.S.| non_cross_misses_maximum=0 streams_with_non_cross_misses=0
49:E7RESULT name=arm_by_transaction cell=g1_one_publish states=28 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=6 identity_ok=true
50:E7RESULT name=invalid_prefix cell=g1_one_publish kind=recorded truth_count=1 arm_by_root_publish_covered_minimum=1 arm_by_root_publish_covered_maximum=1 arm_by_transaction_covered=1 arm_by_transaction_not_in_truth=3 arm_by_root_publish_judgement=covers_every_invalid_prefix arm_by_transaction_judgement=covers_every_invalid_prefix
51:E7RESULT name=invalid_prefix cell=g1_one_publish kind=all_writes truth_count=1 arm_by_root_publish_covered_minimum=1 arm_by_root_publish_covered_maximum=1 arm_by_transaction_covered=1 arm_by_transaction_not_in_truth=2 arm_by_root_publish_judgement=covers_every_invalid_prefix arm_by_transaction_judgement=covers_every_invalid_prefix
```

G3 (u = 1) and G5 (two-state system configuration slot write), P3 free-interleaving cells:

```
92:E7RESULT name=cell_truth cell=g3_free sampling_point=G3 candidate=many_publishes_in_flight_free disks=1 units=1 table_writes=8 streams=3432 truth_by_enumeration=36 truth_by_search=36 methods_agree=true cross_transaction_states=25
93:E7RESULT name=arm_by_root_publish cell=g3_free streams=3432 states_minimum=11 states_maximum=17 states_on_serial_stream=11 states_on_minimum_cross_miss_stream=17 closed_form_mismatches=0 cross_misses_minimum=11 cross_misses_maximum=20 streams_with_cross_misses=3432 minimum_cross_miss_stream=a1.|.rA.b1.|.|.rB.RA.|.RB.SA.SB.|.| non_cross_misses_maximum=9 streams_with_non_cross_misses=3432
128:E7RESULT name=cell_truth cell=g5_free sampling_point=G5 candidate=many_publishes_in_flight_free disks=1 units=2 table_writes=10 streams=12870 truth_by_enumeration=49 truth_by_search=49 methods_agree=true cross_transaction_states=36
129:E7RESULT name=arm_by_root_publish cell=g5_free streams=12870 states_minimum=13 states_maximum=24 states_on_serial_stream=13 states_on_minimum_cross_miss_stream=24 closed_form_mismatches=0 cross_misses_minimum=19 cross_misses_maximum=30 streams_with_cross_misses=12870 minimum_cross_miss_stream=a1.a2.b1.b2.|.rA.|.|.rB.RA.|.SA.RB.|.SB.| non_cross_misses_maximum=9 streams_with_non_cross_misses=12870
```

Covering segment sequences and geometry sensitivity (G1 and G2 give the same judgment; G1 and G5 give the same judgment):

```
62:E7RESULT name=cover cell=g1_free set=arm_by_transaction target_states=64 cover_segments=1 cover_largest_segment_writes=10 cover_states=2304 covers_target=true cover_equals_target=false exhaustive_agrees=not_run_more_than_8_writes
173:E7RESULT name=geometry_sensitivity pair=G1_vs_G2 l2_judgement_free=leaks_on_every_stream l2_judgement_ordered=leaks_on_every_stream same_judgement=yes cross_misses_minimum_free=27 cross_misses_minimum_ordered=13 cross_misses_maximum_free=42 cross_misses_maximum_ordered=28 invalid_prefix_recorded_truth_free=16 invalid_prefix_recorded_truth_ordered=1 invalid_prefix_all_writes_truth_free=6 invalid_prefix_all_writes_truth_ordered=2
176:E7RESULT name=geometry_sensitivity pair=G1_vs_G5_free system_configuration_third_state_changes_judgement=no l2_judgement_three_state=leaks_on_every_stream l2_judgement_two_state=leaks_on_every_stream cross_misses_minimum_three_state=27 cross_misses_minimum_two_state=19 cross_misses_maximum_three_state=42 cross_misses_maximum_two_state=30 recorded_truth_three_state=16 recorded_truth_two_state=12
```

Capacity anchors (K2's covering segment sequences on G1 and G4a are measured; the real publication shape is arithmetic only):

```
31:E7RESULT name=anchor table=capacity geometry=G1 quantity=arm_by_transaction_cover_one_segment_states expected=2304 actual=2304 cover_segments=1 cover_largest_segment_writes=10 covers_target=true within_u32=true ok=true
32:E7RESULT name=anchor table=capacity geometry=G4a quantity=arm_by_transaction_cover_one_segment_states expected=1327104 actual=1327104 cover_segments=1 cover_largest_segment_writes=18 covers_target=true within_u32=true ok=true
33:E7RESULT name=anchor table=capacity geometry=real_publish_shape quantity=real_shape_arm_by_transaction_cover_one_segment_states expected=1459166279268040704 actual=1459166279268040704 exceeds_u32=true within_u64=true ok=true
36:E7RESULT name=anchor table=capacity geometry=real_publish_shape quantity=real_shape_k1_witness_first_segment_nonzero_states expected=281474976710655 actual=281474976710655 exceeds_u32=true within_u64=true ok=true
37:E7RESULT name=anchor table=capacity geometry=real_publish_shape quantity=real_shape_k1_witness_first_segment_writes expected=48 actual=48 within_enumerated_writes_limit=true ok=true
```

Positive controls and discriminative power self-test:

```
148:E7RESULT name=positive_control control=serial_concatenation cell=g1_free arm=arm_by_root_publish witness=b1 witness_in_truth=true witness_missed=true misses_against_truth=49 cross_misses=42 ok=true
149:E7RESULT name=positive_control control=serial_concatenation cell=g1_free arm=arm_by_transaction witness=b1 witness_in_truth=true witness_missed=true misses_against_truth=49 cross_misses=42 ok=true
150:E7RESULT name=positive_control control=delete_one_state cell=g1_free deleted=a1+b1 arm_by_root_publish_misses_before=37 arm_by_root_publish_misses_after=38 arm_by_transaction_misses_before=0 arm_by_transaction_misses_after=1 ok=true
151:E7RESULT name=positive_control control=delete_one_invalid_prefix cell=g1_free deleted=a1+a2+b1+b2+rB arm_by_root_publish_covered_before=1 arm_by_root_publish_covered_after=0 arm_by_transaction_covered_before=16 arm_by_transaction_covered_after=15 ok=true
170:E7RESULT name=threshold_shift quantity=l2_cross_misses cell=g1_free threshold_before=0 judgement_before=leaks_on_every_stream threshold_after=42 judgement_after=never_leaks flips_as_required=true
171:E7RESULT name=threshold_shift quantity=l6_recorded_prefixes_best_stream cell=g1_free covered=13 required_before=16 judgement_before=misses_invalid_prefixes required_after=13 judgement_after=covers_every_invalid_prefix flips_as_required=true
```

Counts of each line type in the artifact (one command, `research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out`):

```
$ sed 's/^E7RESULT name=\([a-z_0-9]*\).*/\1/' research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out | sort | uniq -c
     32 anchor
     12 arm_by_root_publish
     12 arm_by_transaction
     12 capacity
     12 cell_truth
      1 config
     24 cover
      1 done
      9 failure_clause
      5 geometry_sensitivity
      4 halt_check
     24 invalid_prefix
     24 positive_control
     12 reference
      3 threshold_shift
     13 verdict
```

Each of 12 cells has one `cell_truth`, `arm_by_root_publish`, `arm_by_transaction`, `capacity`, and `reference` line (12 each) and two `invalid_prefix` and `cover` lines (24 each). Twenty-four `positive_control` lines = two per each of three P2 cells + four per each of three P3 free-interleaving cells + two per each of three P3 Σ_ord cells (PC2/PC3 witness states are not in the truth on P1's serial stream, so those controls do not run).

### Judgment by row

| Quantity | Reading | Judgment |
|---|---|---|
| V1 actual baseline | P1 and P2 in G1, G3, G5: K1 state count = truth, zero misses and extra states (`name=arm_by_root_publish cell=g1_serial` line 39; `cell=g1_one_publish` line 48). | Pass |
| V2 positive controls | PC1 reports witness `b1` missed by both K1 and K2 in all three G1/G3/G5 free-interleaving cells; PC2 adds exactly one miss; PC3 reduces coverage by exactly one (all 72 `ok=` fields in output are `true`). | Pass |
| V3 two truth algorithms | All 12 cells `methods_agree=true`. | Pass |
| V4 K2 misses on one disk | All 12 cells `cross_misses=0`. | Pass, consistent with the theorem derived in registration section 4 item 6 |
| V5, V6 | All 32 class-C anchors `ok=true`; all 12 cells `identity_ok=true`, `closed_form_mismatches=0`. | Pass |
| l2 K1 cross-transaction misses | P3 G1: min 27, max 42, all 12870 streams miss; G2: min 13, max 28, all 3747 miss. G3 min 11, G3 Σ_ord min 5, G5 min 19, G5 Σ_ord min 9 likewise miss on every stream. P1/P2 are always 0. | K1 misses on every recorded stream at every P3 sample point (minimum > 0). |
| L3a, L3b K2 misses | All 12 one-disk cells are 0; two disks deferred to segment two. | No separate one-disk judgment (derivable by definition). |
| l4 state count | K1: G1 15–29 per stream, 15 on serial; K2: 64 on G1 and G2 (but truth is 46); P2 K1 22, K2 28. | Report count |
| l5 extra K2 states | G1 free 0; G2 18; G3 Σ_ord 10; G5 Σ_ord 13; P1 on G1 49. | Report count; extras are in K2's product but not truth. |
| l6 invalid prefixes | P3 G1: 16 by recorded prefix, 6 by all writes; K1 covered min 0, max 13 (recorded) and 6 (all writes); K2 covers all. G2: 1 recorded, 2 all writes. P1 always 0; F4 does not fire (both levels nonzero in all three P3 free cells). | K1 does not cover all at either level; K2 covers all. |
| L7a K2 covering segment sequence | G1 one 10-write segment, 2304 states; G4a one 18-write segment, 1327104 states. P3 truth itself also needs one 10-write segment, 2304 states on G1/G2. | Consistent with derivation in section 4 item 7; F6 does not fire. |
| l10 | All 12 cells `exact_not_in_truth=0`, `truth_not_in_exact=0`. | F1 does not fire. |
| F3 | G1/G2, G3/G3 Σ_ord, G5/G5 Σ_ord each yield the same judgment (`leaks_on_every_stream`). | Does not fire. |

### Verdict lines identified

`grep -n 'name=verdict' research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out` gives 13 lines (187–199): no boolean `false` or `not_run` fields. Integer counters containing `misses` are results, not violations: `arm_by_root_publish_cross_misses_minimum` is positive in six cells `g1_free` (27), `g1_ordered` (13), `g3_free` (11), `g3_ordered` (5), `g5_free` (19), `g5_ordered` (9), all the registered expected “K1 misses on P3.” Outside verdict lines, 19 lines contain `=false`: one `exceeds_u32=false` (16777228 states in a single-transaction chain fit u32), 18 `cover_equals_target=false` (covering segment sequence exceeds the target, derived in registration section 4 item 7). Twelve lines contain `not_run_more_than_8_writes` (ordered-partition exhaustive search is omitted for > 8 writes, amendment 3).

### Unit tests and mutations

37 unit tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs` gives 37; `cargo test --release` reports `37 passed; 0 failed`). Twelve mutations (`scripts/mutation/tables/e167_multi_inflight_crash_cut.tsv`): **12 caught, 0 invalid, 0 not red**; 💥, ⚠️, ⏱, 🧱 each 0; final counts “0 memory ceiling hits (2G cap), 0 timeouts” and “restored, baseline still all green” (`e167-multi-inflight-crash-cut-mutate-2026-10-01-r2.log`). M10 (region attribution) changes no first-segment output lines and is caught by unit test `root_slots_of_the_two_disk_geometries_follow_the_three_region_assignment`. “All 12 caught” says only the written assertions were mutation-tested, not that every piece of code relevant to conclusions was tested.

### Replay

`bash scripts/experiment-apparatus/replay.sh E167` (crash enumeration cuts for multiple in-flight publications): (registered row E167 (crash enumeration cuts for multiple in-flight publications) `|e167-multi-inflight-crash-cut||e167-multi-inflight-crash-cut-2026-10-01-r2.out|exact`). Deterministic model: five consecutive runs of the same binary yield the identical `sha256sum`, demonstrating that there is no hidden state, rather than mere statistical stability. Termination clause subcommand: `./target/release/e167-multi-inflight-crash-cut halt`.

### Paths and conclusions register

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Truth: enumeration | For every issued stream in Σ(P), segment by device, take its enumeration domain and union them. | `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs:609 truth_by_enumeration` (calls `:296 operation_segments`, `:384 crash_states_of_segments`) | Write table and chain generator (order in D16 (Publish Semantics) settled item 7), segmentation rules, interleaving enumerator. |
| 2 | Truth: per-state search | For each vector in all states, search interleavings of the two chains for an issued stream putting it in the enumeration domain. | `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs:703 some_stream_realizes_target` | Write table and chain generator; separately implemented stepwise segmentation; Σ_ord waiting table shared with path 1. |
| 3 | Truth: independent recount | Python script recalculates Σ and T cardinalities, K1 distribution, misses and two prefix levels from definitions. | No implementation: one-off script `e167-multi-inflight-crash-cut-python-recount-2026-10-01.py` archived out of tree in 31128273 (recoverable from git history); apparatus test `many_publishes_in_flight_cells_match_the_independent_python_recount` pins its readings. | Reads only registration section 1 definitions, not Rust apparatus. |
| 4 | Exact reachable set | Take downward-closed sets under device partial order; tearing applies only to maximal elements. | `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs:752 must_persist_before_masks`, `:829 exact_reachable_states` | Write table and stream; does not read segmentation rules. |
| 5 | Covering segment sequence | Merge method: strongly connected components of partial order in topological order; G3 also checked by a differential against exhaustive ordered partitions. | `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs:976 cover_by_merging`, `:1039 cover_by_exhaustive_ordered_partitions` | Write table and closed form; coverage checked against definition without partial order. |

Path 1 versus 2 is asserted state by state for each cell (`methods_agree`); path 1 versus 3 uses values pinned in tests (apparatus does not read script); the two path-5 methods are compared on G3 in `merging_agrees_with_exhaustive_ordered_partitions_on_the_u1_cells`. Shared inputs: paths 1 and 2 share the write table, chain write order and Σ_ord constraint definition, so errors there could affect both while every cell agrees. Class-C anchors (two S1 registration-table entries and closed-form table) and path 3's counts pin these (chain and interleaving independently implemented). The conclusion is universal (“K1 misses on every recorded stream”), and values exhaust all issued streams in Σ(P) (12870 G1, 3747 G2), not a sample.

### What this cannot answer

- Truth is “the union of D13 settled item 4's enumeration domain over every issued stream allowed by the Q1 candidate”: K1 consumes one stream, while truth unions many; one stream necessarily cannot contain the union (G1 K1 max 29 states, truth 64). This reading is fixed in registration section 1 and accepted by the main agent. “K1 misses on every stream” is the reading under that definition. If crash enumeration judges each recorded stream separately, with many recordings jointly covering Σ, then a single stream's “misses” are other streams' states; whether that is a defect belongs to Q1/Q2, not this experiment.
- K2 does not miss on one-disk geometry by definition; whether it misses on two disks (G4a/G4b, L3b) was not measured in segment one and belongs to segment two. “K2 does not miss” cannot be concluded from this segment.
- K2 has 18 states beyond truth on G2 (10 for G3 Σ_ord, 13 for G5 Σ_ord): combinations where A and B transactions are separately reachable but their combination is unreachable under issued-order constraints such as rA before rB. Whether enumerating these causes gate false positives was not measured.
- Σ_ord includes only rA before rB, RA before RB, SA before SB (fixed in registration section 1); other additional disciplines (Q2 candidates) have no separate cells.
- The capability claim that state counts exceed the crash amplification apparatus is arithmetic only: on the real publication shape, the witness stream's first K1 segment has 48 writes and 2^48 − 1 states, exceeding the u32 within-segment ordinal asserted in `crates/singlefs-checker-tier/src/crash_verify_gpu.rs` when this was written. K1's largest segment for the real shape (l8 dynamic programming) and K2 cost after adding product expansion were not measured.
- Structural constants (2048 writes, u32, u64) are copied from registration section 3 and not cross-checked against `crates/` in the apparatus, which is not committed apparatus.
- Exhaustive ordered partitioning stops at ≤ 8 writes: covering sequences for G1/G5 (10 writes) only use the merging method; G3 is the differential.
- Merging consecutive barriers on one disk or not gives the same state set (unit test `doubled_barriers_on_one_disk_do_not_change_the_crash_state_set`); device tearing whole writes in other forms is outside the model.

## Segment two (l1–l4, l6, l8, l10 on two-disk geometries G4a/G4b)

### What segment two did

- **S3 first** (registration section 11): the `s3` subcommand reports scale before results, artifact `e167-multi-inflight-crash-cut-s3-estimate-2026-10-01.out`:

```
产物行见 git show 5d0b7e7c1667:research/results/e167-multi-inflight-crash-cut-s3-estimate-2026-10-01.out 第 1 行（name=s3_estimate）
产物行见 git show 5d0b7e7c1667:research/results/e167-multi-inflight-crash-cut-s3-estimate-2026-10-01.out 第 3 行（name=s3_estimate）
```

  The 155117520 free interleavings have only 19719 (G4a) and 25252 (G4b) distinct segment signatures and 243158 distinct partial orders; the per-state depth search has 1327104 state vectors and a search space of 12288. Apparatus stderr timings (not in output): 1.15 seconds (G4a) and 1.20 seconds (G4b) to visit all 155117520 leaves per free cell, five threads; full `second` run 17.42 seconds, peak 282104 KiB (`/usr/bin/time -v`, draft `second-r3.err`, not committed). Both algorithms finish, S3 does not stop, no sample point reduced.
- **Apparatus:** added `s3` and `second` subcommands to the first-segment source (default first-segment output remains byte-identical to committed `-r2`, diff zero). Two-disk Σ_free streams are not stored: depth-first search aggregates K1 distributions by “segment signature × count”; T_exact uses downward-closed-set recursion and algorithm (b) uses fixed arrays. The measurement basis and readings are in amendments 7–9: two-disk Σ_ord pairs record and system-configuration writes per disk, root-slot writes by kind; G4 Σ_ord has 37834712 streams.
- Eight cells: G4a and G4b each P1 (`g4a_serial`), P2 (`g4a_one_publish`), P3 free (`g4a_free`), P3 Σ_ord (`g4a_ordered`), lines 2–99; G5 and l9 not run (incidental quantities omitted after sufficient evidence).

### Segment-two result lines copied intact from the artifact

G4a P3 free interleaving (truth, K1 distribution, K2, invalid prefixes, covering segments, capacity, exact reachable set):

```
23:E7RESULT name=cell_truth cell=g4a_free sampling_point=G4a candidate=many_publishes_in_flight_free disks=2 units=2 table_writes=18 streams=155117520 truth_by_enumeration=1327104 truth_by_search=1327104 methods_agree=true cross_transaction_states=1324801
24:E7RESULT name=stream_classes cell=g4a_free interleavings=155117520 interleavings_by_lattice_count=155117520 counts_agree=true distinct_segment_signatures=19719 distinct_order_masks=243158 state_vector_universe=1327104
25:E7RESULT name=arm_by_root_publish cell=g4a_free streams=155117520 states_minimum=47 states_maximum=1327104 states_on_serial_stream=55 states_on_minimum_cross_miss_stream=1327104 closed_form_mismatches=0 cross_misses_minimum=0 cross_misses_maximum=1324774 streams_with_cross_misses=155115151 minimum_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.SA@1.|1.RB@1.SB@0.SB@1.|0.|1.|0.|1 non_cross_misses_maximum=2287 streams_with_non_cross_misses=155115151
26:E7RESULT name=arm_by_transaction cell=g4a_free states=784 cross_misses=1324072 true_cross_misses=0 non_cross_misses=2248 states_not_in_truth=0 identity_ok=true
27:E7RESULT name=invalid_prefix cell=g4a_free kind=recorded truth_count=248832 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=248832 arm_by_transaction_covered=180 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=misses_invalid_prefixes
28:E7RESULT name=invalid_prefix cell=g4a_free kind=all_writes truth_count=1150 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=1150 arm_by_transaction_covered=26 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=misses_invalid_prefixes
29:E7RESULT name=cover cell=g4a_free set=truth target_states=1327104 cover_segments=1 cover_largest_segment_writes=18 cover_states=1327104 covers_target=true cover_equals_target=true exhaustive_agrees=not_run_more_than_8_writes
30:E7RESULT name=cover cell=g4a_free set=arm_by_transaction target_states=784 cover_segments=1 cover_largest_segment_writes=18 cover_states=1327104 covers_target=true cover_equals_target=false exhaustive_agrees=not_run_more_than_8_writes
31:E7RESULT name=capacity cell=g4a_free table_writes=18 arm_by_root_publish_largest_segment_writes=18 arm_by_root_publish_largest_segment_states=1327104 arm_by_root_publish_total_states_maximum=1327104 arm_by_transaction_cover_largest_segment_states=1327104 arm_by_transaction_cover_total_states=1327104 writes_within_limit=true segments_within_u32=true segments_within_u64=true
32:E7RESULT name=reference cell=g4a_free exact_states=784 exact_not_in_truth=0 truth_not_in_exact=1326320
```

G4b P3 free interleaving (A's txg ≡ 2, RA/RB both on disk 0):

```
72:E7RESULT name=cell_truth cell=g4b_free sampling_point=G4b candidate=many_publishes_in_flight_free disks=2 units=2 table_writes=18 streams=155117520 truth_by_enumeration=1327104 truth_by_search=1327104 methods_agree=true cross_transaction_states=1324801
74:E7RESULT name=arm_by_root_publish cell=g4b_free streams=155117520 states_minimum=47 states_maximum=1327104 states_on_serial_stream=55 states_on_minimum_cross_miss_stream=1327104 closed_form_mismatches=0 cross_misses_minimum=0 cross_misses_maximum=1324774 streams_with_cross_misses=155115956 minimum_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.|1.SA@1.|0.RB@0.SB@0.|1.SB@1.|0.|1 non_cross_misses_maximum=2287 streams_with_non_cross_misses=155115956
75:E7RESULT name=arm_by_transaction cell=g4b_free states=784 cross_misses=1324072 true_cross_misses=0 non_cross_misses=2248 states_not_in_truth=0 identity_ok=true
76:E7RESULT name=invalid_prefix cell=g4b_free kind=recorded truth_count=248832 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=248832 arm_by_transaction_covered=180 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=misses_invalid_prefixes
77:E7RESULT name=invalid_prefix cell=g4b_free kind=all_writes truth_count=1150 arm_by_root_publish_covered_minimum=0 arm_by_root_publish_covered_maximum=1150 arm_by_transaction_covered=26 arm_by_transaction_not_in_truth=0 arm_by_root_publish_judgement=misses_invalid_prefixes arm_by_transaction_judgement=misses_invalid_prefixes
81:E7RESULT name=reference cell=g4b_free exact_states=784 exact_not_in_truth=0 truth_not_in_exact=1326320
```

Two P3 Σ_ord cells (G4a_ordered, G4b_ordered) and actual baselines (P1 serial, P2 multiple transactions in one publication, G4a):

```
38:E7RESULT name=cell_truth cell=g4a_ordered sampling_point=G4a_ordered candidate=many_publishes_in_flight_ordered disks=2 units=2 table_writes=18 streams=37834712 truth_by_enumeration=1327104 truth_by_search=1327104 methods_agree=true cross_transaction_states=1324801
41:E7RESULT name=arm_by_transaction cell=g4a_ordered states=784 cross_misses=1324072 true_cross_misses=0 non_cross_misses=2248 states_not_in_truth=0 identity_ok=true
47:E7RESULT name=reference cell=g4a_ordered exact_states=568 exact_not_in_truth=0 truth_not_in_exact=1326536
87:E7RESULT name=cell_truth cell=g4b_ordered sampling_point=G4b_ordered candidate=many_publishes_in_flight_ordered disks=2 units=2 table_writes=18 streams=37834712 truth_by_enumeration=1327104 truth_by_search=1327104 methods_agree=true cross_transaction_states=1324801
90:E7RESULT name=arm_by_transaction cell=g4b_ordered states=784 cross_misses=1324072 true_cross_misses=0 non_cross_misses=2248 states_not_in_truth=0 identity_ok=true
96:E7RESULT name=reference cell=g4b_ordered exact_states=568 exact_not_in_truth=0 truth_not_in_exact=1326536
2:E7RESULT name=cell_truth cell=g4a_serial sampling_point=G4a candidate=serial_pipeline disks=2 units=2 table_writes=18 streams=1 truth_by_enumeration=55 truth_by_search=55 methods_agree=true cross_transaction_states=27
4:E7RESULT name=arm_by_transaction cell=g4a_serial states=784 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=729 identity_ok=true
11:E7RESULT name=cell_truth cell=g4a_one_publish sampling_point=G4a candidate=one_publish_many_transactions disks=2 units=2 table_writes=15 streams=1 truth_by_enumeration=280 truth_by_search=280 methods_agree=true cross_transaction_states=249
13:E7RESULT name=arm_by_transaction cell=g4a_one_publish states=370 cross_misses=0 true_cross_misses=0 non_cross_misses=0 states_not_in_truth=90 identity_ok=true
```

Positive control (G4a free-interleaving grid), self-demonstration of discriminative power, and geometric sensitivity:

```
33:E7RESULT name=positive_control control=serial_concatenation cell=g4a_free arm=arm_by_root_publish witness=b1@0 witness_in_truth=true witness_missed=true misses_against_truth=1327049 cross_misses=1324774 ok=true
34:E7RESULT name=positive_control control=serial_concatenation cell=g4a_free arm=arm_by_transaction witness=b1@0 witness_in_truth=true witness_missed=true misses_against_truth=1327049 cross_misses=1324774 ok=true
35:E7RESULT name=positive_control control=delete_one_state cell=g4a_free deleted=a1@0+b1@0 arm_by_root_publish_misses_before=1324710 arm_by_root_publish_misses_after=1324711 arm_by_transaction_misses_before=1324072 arm_by_transaction_misses_after=1324073 ok=true
36:E7RESULT name=positive_control control=delete_one_invalid_prefix cell=g4a_free deleted=a1@0+a1@1+a2@0+a2@1+b1@0+b1@1+b2@0+b2@1+rB@0+rB@1 arm_by_root_publish_covered_before=864 arm_by_root_publish_covered_after=863 arm_by_transaction_covered_before=180 arm_by_transaction_covered_after=179 ok=true
37:E7RESULT name=positive_control control=delete_one_true_cross_state cell=g4a_free deleted=a1@0+b1@0 arm_by_transaction_true_misses_before=0 arm_by_transaction_true_misses_after=1 ok=true
100:E7RESULT name=threshold_shift quantity=l2_cross_misses cell=g4a_free threshold_before=0 judgement_before=leaks_on_some_streams threshold_after=1324774 judgement_after=never_leaks flips_as_required=true
101:E7RESULT name=threshold_shift quantity=l3b_true_cross_misses cell=g4a_free status=not_applicable reason=l3b_is_zero_so_there_is_no_positive_judgement_to_flip
103:E7RESULT name=threshold_shift quantity=l6_recorded_prefixes_best_stream cell=g4a_free covered=248832 required_before=248832 judgement_before=covers_every_invalid_prefix required_after=248833 judgement_after=misses_invalid_prefixes flips_as_required=true
104:E7RESULT name=threshold_shift quantity=l6_recorded_prefixes_arm_by_transaction cell=g4a_free covered=180 required_before=248832 judgement_before=misses_invalid_prefixes required_after=180 judgement_after=covers_every_invalid_prefix flips_as_required=true
105:E7RESULT name=geometry_sensitivity pair=G4a_vs_G4b_free same_judgement=yes l2_judgement_g4a=leaks_on_some_streams l2_judgement_g4b=leaks_on_some_streams cross_misses_minimum_g4a=0 cross_misses_minimum_g4b=0 cross_misses_maximum_g4a=1324774 cross_misses_maximum_g4b=1324774 l3a_g4a=1324072 l3a_g4b=1324072 l3b_g4a=0 l3b_g4b=0 recorded_truth_g4a=248832 recorded_truth_g4b=248832 all_writes_truth_g4a=1150 all_writes_truth_g4b=1150
```

l8 (real publication shape `real_publish_shape`, two transactions each `24+2+1+2`, Σ_free has C(70, 35) interleavings, dynamic programming) and dynamic-programming differential against small two-disk geometries:

```
108:E7RESULT name=capacity_real_shape shape=real_publish_shape objective=largest_segment_states interleavings=seventy_choose_thirty_five table_writes=58 segment_writes=58 segment_in_place_writes=4 segment_states=1459166279268040703 exceeds_u32=true within_u64=true within_enumerated_writes_limit=true dynamic_programming_agrees_with_replay=true stream=a1@0.a1@1.a2@0.a2@1.a3@0.a3@1.a4@0.a4@1.a5@0.a5@1.a6@0.a6@1.a7@0.a7@1.a8@0.a8@1.a9@0.a9@1.a10@0.a10@1.a11@0.a11@1.a12@0.a12@1.|0.b1@0.|1.b1@1.b2@0.b2@1.b3@0.b3@1.b4@0.b4@1.b5@0.b5@1.b6@0.b6@1.b7@0.b7@1.b8@0.b8@1.b9@0.b9@1.b10@0.b10@1.b11@0.b11@1.b12@0.b12@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.SA@1.|1.RB@1.SB@0.SB@1.|0.|1.|0.|1
110:E7RESULT name=capacity_dynamic_programming_check geometry=G4a dynamic_programming_largest_expansion=1327104 signatures_largest_expansion=1327104 dynamic_programming_most_writes=18 signatures_most_writes=18 agrees=true
111:E7RESULT name=capacity_dynamic_programming_check geometry=G4b dynamic_programming_largest_expansion=1327104 signatures_largest_expansion=1327104 dynamic_programming_most_writes=18 signatures_most_writes=18 agrees=true
```

Line 109 (`objective=most_writes_in_one_segment`) reports `segment_writes=58 segment_in_place_writes=4 segment_states=1459166279268040703`; witness stream is the same as line 108.

Counts by category (one command, `research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out`):

```
$ sed 's/^E7RESULT name=\([a-z_0-9]*\).*/\1/' research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out | sort | uniq -c
      8 arm_by_root_publish
      8 arm_by_transaction
      8 capacity
      2 capacity_dynamic_programming_check
      2 capacity_real_shape
      8 cell_truth
      1 config
     16 cover
      1 done
     16 failure_clause
      3 geometry_sensitivity
     16 invalid_prefix
     22 positive_control
      8 reference
      4 stream_classes
      5 threshold_shift
      9 verdict
```

Each of eight cells has one `cell_truth`, `arm_by_root_publish`, `arm_by_transaction`, `capacity`, and `reference` line, two `invalid_prefix` and `cover` lines. Four `stream_classes` lines belong only to P3. Twenty-two `positive_control` lines = three per two P2 cells + five per two P3 free cells + three per two Σ_ord cells (P1 not run). Sixteen `failure_clause` = F1 1 + F2 4 + F5 4 + F3 2 + F4 4 + F6 1.

### Segment-two judgment by row

| Quantity | Reading | Judgment |
|---|---|---|
| V1 actual baseline | G4a/G4b P1/P2: K1 states = truth (55, 280), zero misses and extras (lines 3, 12, etc.). | Pass |
| V3 two truth algorithms | Eight `methods_agree=true` (four G4 P3 truth cells all 1327104, the entire vector set; P1 55, P2 280); four P3 cells `counts_agree=true` (two interleaving counts agree). | Pass |
| V5, V6 | Eight `identity_ok=true`; four P3 `closed_form_mismatches=0`; apparatus `self_checks_passed=81`, `void_conditions=0`. | Pass |
| V2 positive controls | 22 `ok=true` lines: PC1 reports `b1@0` missed on K1 and K2 in free G4a/G4b; PC2 adds exactly one miss; PC3 reduces coverage by one. Added PC2 counterpart for L3b deletes one truth ∩ exact ∩ cross-transaction state from K2, changing L3b 0 → 1; all six P2/P3 cells of eight are `ok=true`. | Pass |
| l1 truth |T| | Four P3 cells are 1327104 = all state vectors (2^14 · 3^4): both free and Σ_ord have a stream merging all 18 writes into one segment (line 29, one 18-write cover, `cover_equals_target=true`). | Report count |
| l2 K1 cross-transaction misses | P3 G4a/G4b free and G4a/G4b Σ_ord min 0, max 1324774; positive-miss streams 155115151, 155115956, 37833148, 37833930 respectively, out of 155117520 and 37834712 streams: 2369, 1564, 1564, 782 streams miss nothing. P1/P2 always 0. | All four `leaks_on_some_streams`, not every stream. The stream attaining zero is line 25's `minimum_cross_miss_stream` (disk barriers staggered, one disk always blocked, all writes merged). F3 does not trigger on G4; free and Σ_ord agree. |
| L3a K2 cross misses versus T | All four P3 cells 1324072, P1/P2 0. | Report count |
| L3b K2 true cross misses versus T ∩ T_exact | All eight `true_cross_misses=0`. | K2 misses no reachable cross-transaction persistence subset on G4a/G4b. |
| F5 | G4a/G4b free and Σ_ord all `triggered=yes` (L3a = 1324072, L3b = 0). | All 1324072 K2 cross states missed on G4 are the “actually unreachable” class acknowledged by D13 settled item 4: exact reachable set is only 784 on free (= K2's 784, `exact_states=784`, `exact_not_in_truth=0`) and 568 on Σ_ord. |
| F2 | Four `triggered=no` (l2 min 0). | Not triggered: not both arms miss. |
| l4 state count | K1 per stream 47–1327104 (G4a/G4b free and Σ_ord), 55 on σ_ser; K2 784; P2 K1 280, K2 370. | Per-stream closed forms agree; K2 equals product closed form. |
| l5 extra K2 states | Four P3 `states_not_in_truth=0` (truth is already the entire set); P1 729, P2 90. | Report count |
| l6 invalid prefixes | P3 G4a/G4b free and Σ_ord: truth 248832 recorded, 1150 all-writes; K1 per-stream coverage min 0, max 248832 and 1150 (best covers all); K2 covers 180 and 26, not all. P1 both 0 (F4 only reports the count); P2 each 3. | K1 min < truth count (some streams miss), best covers all. K2 misses invalid prefixes (180 / 248832, 26 / 1150), but they are all actually unreachable (F5). |
| L7a, L7b | Four G4 P3 cells: truth itself covered by one 18-write segment, 1327104 states; K2's cover is the same (`cover_equals_target=false`, cover is entire set); maximum 1327104 fits u32. | Consistent with registration section 4 item 7, F6 not triggered. |
| l8 real publication shape | K1 on Σ_free has a segment of at most 58 writes (all writes), `segment_states=1459166279268040703` (2^54 · 3^4 − 1), above 2^32 − 1, within u64 and 2048-write limit. Dynamic-programming witness was replay-checked (`dynamic_programming_agrees_with_replay=true`); G4a/G4b dynamic programming equals maximum over all signatures (lines 110–111). | Larger than the ≥ 48 writes inferred in section 4 item 7: an issued stream merges all 58 real-shape writes into one segment; on it K1 and K2 cover sequences are of the same order. F6 not triggered. |
| l10 | Eight `exact_not_in_truth=0` (F1 not triggered); `truth_not_in_exact`: P1/P2 0, P3 free 1326320, Σ_ord 1326536. | F1 not triggered; D13 item 4's extra unreachable states on two disks are only counted. |
| Discriminative power self-test | l2 threshold 0 → 1324774: `leaks_on_some_streams` → `never_leaks`; l6 flips once in each direction on K1's best stream (covered → missed) and K2 (missed → covered). L3b is 0 on both geometries with no positive verdict to flip, `not_applicable` (lines 101–102); confidence in L3b = 0 rests on its PC2 counterpart. | Pass |
| Geometry sensitivity | G4a/G4b agree on l2, L3b and both truth-prefix counts (line 105 `same_judgement=yes`), and free/Σ_ord agree (lines 106–107). | Instability condition not triggered. |

### Segment-two verdict lines identified

`grep -n 'name=verdict' research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out` gives nine lines (128–136), with no boolean `false` or `not_run` fields. Counters containing `misses` or `not_in`: `arm_by_root_publish_cross_misses_maximum` is 1324774 in the four `g4a_free`, `g4a_ordered`, `g4b_free`, `g4b_ordered` cells, and `arm_by_transaction_cross_misses` is 1324072 in the same four: expected readings (K1 misses on some streams, K2 misses model truth), not violations. `truth_not_in_exact` in these four is 1326320 or 1326536, the extra unreachable states acknowledged by D13 item 4, registered as report-only. `arm_by_transaction_true_cross_misses=0` and `exact_not_in_truth=0` in all eight. Eight lines contain `=false`, all `cover_equals_target=false` (K2's cover exceeds K2, derived in registration section 4 item 7).

### Segment-two unit tests and mutations

52 tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs`; `cargo test --release --bin e167-multi-inflight-crash-cut` reports `52 passed; 0 failed`), 15 added since segment one: two-disk Σ_ord interleaving counts match independent Python (`e167-multi-inflight-crash-cut-ordered-count-python-2026-10-01.out`: G4 free 155117520, Σ_ord 37834712); depth-search leaves match the number satisfying five ordering constraints in Σ_free; streaming and first-segment in-memory paths match on seven one-disk cells; fixed-array and set-based search match per state; signatures and partial-order immediate predecessors match stream functions on sampled two-disk streams; downward-closed recursion matches subset scan; dynamic-programming maximum segment matches brute force; real shape has 58 writes in one segment, etc. Twenty-two mutations (`scripts/mutation/tables/e167_multi_inflight_crash_cut.tsv`, M1–M12 segment one, M13–M22 new): **22 caught, 0 invalid, 0 not red**; 💥, ⚠️, ⏱, 🧱 all 0; “0 memory ceiling hits (4G cap), 0 timeouts” and “restored, baseline still all green” (`e167-multi-inflight-crash-cut-mutate-2026-10-01-r4.log`). Prior run `-r3.log`, same table with one fewer test, failed to catch M18: merging thread signature tables chose lexicographically largest instead of smallest stream and assertions missed it. This was “assertion lacks coverage” under `mutation-sampling.md`; after adding `pass_one_keeps_the_smallest_path_and_the_full_count_of_every_signature_whatever_the_thread_split`, `-r4.log` caught M18. All 22 caught establishes only that written assertions were mutation-tested, not that all code relevant to conclusions was tested.

### Segment-two replay

`bash scripts/experiment-apparatus/replay.sh E167` (crash enumeration cuts for multiple in-flight publications): (when segment two finished there were two registered rows: the first segment `…-2026-10-01-r2.out` and the second segment E167 (crash enumeration cuts for multiple in-flight publications) `|e167-multi-inflight-crash-cut|second|e167-multi-inflight-crash-cut-second-2026-10-01-r3.out|exact`). Determinism: running `-r3` once each with 5 threads and 2 threads passes `cmp`, demonstrating that thread count does not affect output, rather than mere statistical stability.

### What segment two cannot answer

- **Truth on two disks is the entire state-vector set**: D13 item 4 segments per device; a stream staggers the two transactions' barriers so one disk is always blocked and all 18 writes form one segment. Its enumeration domain is all 1327104 vectors; only 784 (free) or 568 (Σ_ord) are exactly reachable by device partial order. Thus K1 and K2 misses versus T use a reference far larger than truly reachable states. L3b uses T ∩ T_exact for “reachable.” Segment-two registration did not include K1 misses versus T ∩ T_exact (L3b is K2 only); segment three adds l11 (maximum 702 / 729, misses on some streams only).
- **K2 misses of invalid prefixes** (l6 coverage 180 / 248832) likewise use the whole-set reference: K2's product contains only 180 recorded invalid prefixes; the other 248652 are in the model enumeration domain but not the exact reachable set. “K2 coverage of invalid prefixes versus T_exact” was not computed because registration did not specify it.
- The two-disk Σ_ord interpretation was added by amendment 7 (records and system-configuration writes paired per disk, root-slot writes by kind); other readings, such as both A records preceding either B record, were not run and results are unknown.
- Structural constants (2048 writes, u32, u64) were copied from registration section 3, not cross-checked against `crates/` by this non-committed apparatus. Two-disk P3 truth and K1 distribution have no closed form, pinned by per-state equality of l1's two algorithms and small-geometry unit cross-checks. No independent Python recount of G4 P3 (155117520 streams are too many for Python); independent counting covers interleaving totals only.
- One-disk geometry, G5 and l9: see segment one and registration.

## Segment three (one added cell: K1 cross-transaction misses l11 versus T ∩ T_exact, G4a/G4b)

### What segment three did

- **Added quantity** (registration section 12 segment-three amendment 10, based on the main agent's explicitly requested cell and segment-two fork-table missing l2 row): l11(σ) = |(T ∩ T_exact ∩ C) \ Dom4(σ)|, same measurement basis as L3b (reference T ∩ T_exact, count only cross-transaction states). Segment-two l2 referenced all T state vectors (1327104 on two disks), inflating K1 misses through the model's own over-enumeration; l11 restricts reference to exactly reachable cross-transaction states. Each cell reports minimum, maximum and number of streams with positive misses, using l2's judgment function and zero threshold.
- **Apparatus:** added `third` subcommand to the same source (only G4a/G4b free and Σ_ord, with reuse of segment-two truth algorithms, T_exact and K2 product). Two l11 methods must agree for each segment signature: A tests membership by the definition “there exists a current segment,” B enumerates Dom4(σ) and counts hits against the target bitset. On σ_ser, first-segment `state_is_in_crash_states_of_segments` checks once more. Outputs of segment two and default segment one are unchanged: binary from final segment-three source passes bytewise `cmp` against `-second-2026-10-01-r3.out` and `-2026-10-01-r2.out`, respectively.
- **Positive controls** (eight lines, each cell): PC1's serial concatenation misses witness `a1@0+b1@0` on K1 (only the first unit write of each A/B persists, cross-transaction; b1 alone is not cross-transaction and is not a witness). PC2 deletes a target state from the domain of the minimum-miss stream and increases l11 by exactly one. Discriminability: move threshold 0 to maximum, verdict flips `leaks_on_some_streams` → `never_leaks` in all four.
- Registered l2, L3a, l4–l10, G5 are not rerun in segment three; segment-two numbers unchanged.

### Segment-three result lines copied intact from the artifact

`research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out`, lines 2–30 (line 1 is `name=config`):

```
2:E7RESULT name=true_cross_reference cell=g4a_free sampling_point=G4a candidate=many_publishes_in_flight_free truth_states=1327104 exact_states=784 truth_and_exact_states=784 truth_cross_states=1324801 truth_and_exact_cross_states=729
3:E7RESULT name=arm_by_root_publish_true_cross cell=g4a_free streams=155117520 signatures=19719 true_cross_states=729 true_cross_misses_minimum=0 true_cross_misses_maximum=702 streams_with_true_cross_misses=155115151 streams_with_zero_true_cross_misses=2369 signatures_with_true_cross_misses=19718 true_cross_misses_on_serial_stream=702 minimum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.SA@1.|1.RB@1.SB@0.SB@1.|0.|1.|0.|1 maximum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.|1.rA@0.rA@1.|0.|1.RA@0.SA@0.SA@1.|0.|1.b1@0.b1@1.b2@0.b2@1.|0.|1.rB@0.rB@1.|0.|1.RB@1.SB@0.SB@1.|0.|1 two_methods_agree=yes judgement=leaks_on_some_streams arm_by_transaction_true_cross_misses=0
4:E7RESULT name=positive_control control=serial_concatenation_l11 cell=g4a_free witness=a1@0+b1@0 witness_in_true_cross_target=true witness_missed=true true_cross_misses_on_serial_stream=702 ok=true
5:E7RESULT name=positive_control control=delete_one_true_cross_state_from_k1 cell=g4a_free deleted=a1@0+b1@0 true_cross_misses_before=0 true_cross_misses_after=1 ok=true
6:E7RESULT name=true_cross_reference cell=g4a_ordered sampling_point=G4a_ordered candidate=many_publishes_in_flight_ordered truth_states=1327104 exact_states=568 truth_and_exact_states=568 truth_cross_states=1324801 truth_and_exact_cross_states=525
7:E7RESULT name=arm_by_root_publish_true_cross cell=g4a_ordered streams=37834712 signatures=6666 true_cross_states=525 true_cross_misses_minimum=0 true_cross_misses_maximum=498 streams_with_true_cross_misses=37833148 streams_with_zero_true_cross_misses=1564 signatures_with_true_cross_misses=6665 true_cross_misses_on_serial_stream=498 minimum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.SA@1.|1.RB@1.SB@0.SB@1.|0.|1.|0.|1 maximum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.|1.rA@0.rA@1.|0.|1.RA@0.SA@0.SA@1.|0.|1.b1@0.b1@1.b2@0.b2@1.|0.|1.rB@0.rB@1.|0.|1.RB@1.SB@0.SB@1.|0.|1 two_methods_agree=yes judgement=leaks_on_some_streams arm_by_transaction_true_cross_misses=0
8:E7RESULT name=positive_control control=serial_concatenation_l11 cell=g4a_ordered witness=a1@0+b1@0 witness_in_true_cross_target=true witness_missed=true true_cross_misses_on_serial_stream=498 ok=true
9:E7RESULT name=positive_control control=delete_one_true_cross_state_from_k1 cell=g4a_ordered deleted=a1@0+b1@0 true_cross_misses_before=0 true_cross_misses_after=1 ok=true
10:E7RESULT name=true_cross_reference cell=g4b_free sampling_point=G4b candidate=many_publishes_in_flight_free truth_states=1327104 exact_states=784 truth_and_exact_states=784 truth_cross_states=1324801 truth_and_exact_cross_states=729
11:E7RESULT name=arm_by_root_publish_true_cross cell=g4b_free streams=155117520 signatures=25252 true_cross_states=729 true_cross_misses_minimum=0 true_cross_misses_maximum=702 streams_with_true_cross_misses=155115956 streams_with_zero_true_cross_misses=1564 signatures_with_true_cross_misses=25251 true_cross_misses_on_serial_stream=702 minimum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.|1.SA@1.|0.RB@0.SB@0.|1.SB@1.|0.|1 maximum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.|1.rA@0.rA@1.|0.|1.RA@0.SA@0.SA@1.|0.|1.b1@0.b1@1.b2@0.b2@1.|0.|1.rB@0.rB@1.|0.|1.RB@0.SB@0.SB@1.|0.|1 two_methods_agree=yes judgement=leaks_on_some_streams arm_by_transaction_true_cross_misses=0
12:E7RESULT name=positive_control control=serial_concatenation_l11 cell=g4b_free witness=a1@0+b1@0 witness_in_true_cross_target=true witness_missed=true true_cross_misses_on_serial_stream=702 ok=true
13:E7RESULT name=positive_control control=delete_one_true_cross_state_from_k1 cell=g4b_free deleted=a1@0+b1@0 true_cross_misses_before=0 true_cross_misses_after=1 ok=true
14:E7RESULT name=true_cross_reference cell=g4b_ordered sampling_point=G4b_ordered candidate=many_publishes_in_flight_ordered truth_states=1327104 exact_states=568 truth_and_exact_states=568 truth_cross_states=1324801 truth_and_exact_cross_states=525
15:E7RESULT name=arm_by_root_publish_true_cross cell=g4b_ordered streams=37834712 signatures=9025 true_cross_states=525 true_cross_misses_minimum=0 true_cross_misses_maximum=498 streams_with_true_cross_misses=37833930 streams_with_zero_true_cross_misses=782 signatures_with_true_cross_misses=9024 true_cross_misses_on_serial_stream=498 minimum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.b1@0.|1.b1@1.b2@0.b2@1.|0.rA@0.|1.rA@1.|0.rB@0.|1.rB@1.RA@0.|0.SA@0.|1.SA@1.|0.RB@0.SB@0.|1.SB@1.|0.|1 maximum_true_cross_miss_stream=a1@0.a1@1.a2@0.a2@1.|0.|1.rA@0.rA@1.|0.|1.RA@0.SA@0.SA@1.|0.|1.b1@0.b1@1.b2@0.b2@1.|0.|1.rB@0.rB@1.|0.|1.RB@0.SB@0.SB@1.|0.|1 two_methods_agree=yes judgement=leaks_on_some_streams arm_by_transaction_true_cross_misses=0
16:E7RESULT name=positive_control control=serial_concatenation_l11 cell=g4b_ordered witness=a1@0+b1@0 witness_in_true_cross_target=true witness_missed=true true_cross_misses_on_serial_stream=498 ok=true
17:E7RESULT name=positive_control control=delete_one_true_cross_state_from_k1 cell=g4b_ordered deleted=a1@0+b1@0 true_cross_misses_before=0 true_cross_misses_after=1 ok=true
18:E7RESULT name=threshold_shift quantity=l11_true_cross_misses cell=g4a_free threshold_before=0 judgement_before=leaks_on_some_streams threshold_after=702 judgement_after=never_leaks flips_as_required=true
19:E7RESULT name=threshold_shift quantity=l11_true_cross_misses cell=g4a_ordered threshold_before=0 judgement_before=leaks_on_some_streams threshold_after=498 judgement_after=never_leaks flips_as_required=true
20:E7RESULT name=threshold_shift quantity=l11_true_cross_misses cell=g4b_free threshold_before=0 judgement_before=leaks_on_some_streams threshold_after=702 judgement_after=never_leaks flips_as_required=true
21:E7RESULT name=threshold_shift quantity=l11_true_cross_misses cell=g4b_ordered threshold_before=0 judgement_before=leaks_on_some_streams threshold_after=498 judgement_after=never_leaks flips_as_required=true
22:E7RESULT name=geometry_sensitivity quantity=l11 pair=G4a_vs_G4b_free same_judgement=yes l11_judgement_left=leaks_on_some_streams l11_judgement_right=leaks_on_some_streams l11_minimum_left=0 l11_minimum_right=0 l11_maximum_left=702 l11_maximum_right=702 l11_streams_with_misses_left=155115151 l11_streams_with_misses_right=155115956
23:E7RESULT name=geometry_sensitivity quantity=l11 pair=G4a_vs_G4a_ordered same_judgement=yes l11_judgement_left=leaks_on_some_streams l11_judgement_right=leaks_on_some_streams l11_minimum_left=0 l11_minimum_right=0 l11_maximum_left=702 l11_maximum_right=498 l11_streams_with_misses_left=155115151 l11_streams_with_misses_right=37833148
24:E7RESULT name=geometry_sensitivity quantity=l11 pair=G4b_vs_G4b_ordered same_judgement=yes l11_judgement_left=leaks_on_some_streams l11_judgement_right=leaks_on_some_streams l11_minimum_left=0 l11_minimum_right=0 l11_maximum_left=702 l11_maximum_right=498 l11_streams_with_misses_left=155115956 l11_streams_with_misses_right=37833930
25:E7RESULT name=verdict cell=g4a_free sampling_point=G4a arm_by_root_publish_l11_judgement=leaks_on_some_streams arm_by_root_publish_true_cross_misses_minimum=0 arm_by_root_publish_true_cross_misses_maximum=702 arm_by_root_publish_streams_with_true_cross_misses=155115151 arm_by_transaction_true_cross_misses=0 arm_by_transaction_l3b_judgement=never_leaks
26:E7RESULT name=verdict cell=g4a_ordered sampling_point=G4a_ordered arm_by_root_publish_l11_judgement=leaks_on_some_streams arm_by_root_publish_true_cross_misses_minimum=0 arm_by_root_publish_true_cross_misses_maximum=498 arm_by_root_publish_streams_with_true_cross_misses=37833148 arm_by_transaction_true_cross_misses=0 arm_by_transaction_l3b_judgement=never_leaks
27:E7RESULT name=verdict cell=g4b_free sampling_point=G4b arm_by_root_publish_l11_judgement=leaks_on_some_streams arm_by_root_publish_true_cross_misses_minimum=0 arm_by_root_publish_true_cross_misses_maximum=702 arm_by_root_publish_streams_with_true_cross_misses=155115956 arm_by_transaction_true_cross_misses=0 arm_by_transaction_l3b_judgement=never_leaks
28:E7RESULT name=verdict cell=g4b_ordered sampling_point=G4b_ordered arm_by_root_publish_l11_judgement=leaks_on_some_streams arm_by_root_publish_true_cross_misses_minimum=0 arm_by_root_publish_true_cross_misses_maximum=498 arm_by_root_publish_streams_with_true_cross_misses=37833930 arm_by_transaction_true_cross_misses=0 arm_by_transaction_l3b_judgement=never_leaks
29:E7RESULT name=verdict scope=run self_checks_passed=32 void_conditions=0 halt_conditions=0 void_list=none halt_list=none
30:E7RESULT name=done emitted=30
```

Counts by category (one command, `research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out`):

```
$ sed 's/^E7RESULT name=\([a-z_0-9]*\).*/\1/' research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out | sort | uniq -c
      4 arm_by_root_publish_true_cross
      1 config
      1 done
      3 geometry_sensitivity
      8 positive_control
      4 threshold_shift
      4 true_cross_reference
      5 verdict
```

### Segment-three judgment by row

| Quantity | Reading | Judgment |
|---|---|---|
| l11 K1 cross misses versus T ∩ T_exact (G4a/G4b free verdict cells) | Target |T ∩ T_exact ∩ C| = 729 (lines 2, 10 `truth_and_exact_cross_states=729`); min 0, max 702, positive streams 155115151 (G4a) and 155115956 (G4b), no-miss 2369 and 1564 streams, σ_ser 702 (lines 3, 11). | Both `leaks_on_some_streams` on reachable cross-transaction states, not every stream. Zero-miss streams have the same shape as segment-two l2 minimum (disk barriers staggered, one disk always blocked, all writes in one segment, same `minimum_cross_miss_stream` shape). Most others miss; at most 702 of 729 target states. |
| l11 Σ_ord (G4a_ordered/G4b_ordered) | Target 525 (lines 6, 14); min 0, max 498, positive streams 37833148 and 37833930, no-miss 1564 and 782; σ_ser 498 (lines 7, 15). | Same `leaks_on_some_streams`; F3-style comparison: free and Σ_ord judgments match (lines 23, 24 `same_judgement=yes`), as do G4a/G4b (line 22). |
| Relationship to l2 | Positive stream counts equal segment-two l2's `streams_with_cross_misses` cell by cell (155115151, 155115956, 37833148, 37833930); maximum drops from 1324774 to 702 (free) or 498 (Σ_ord). | Same judgment; l2 misses are inflated by reference T (entire set), l11 maximum counts misses among reachable states. |
| L3b alongside (K2) | `arm_by_transaction_true_cross_misses=0` in four cells (lines 3, 7, 11, 15). | K2's true misses versus T ∩ T_exact remain 0; segment-two L3b has the same value in eight cells. |
| Two methods agree | Four `two_methods_agree=yes`; `self_checks_passed=32`, `void_conditions=0`. | Pass |
| Positive controls | PC1 `ok=true` in four cells (σ_ser misses 702 or 498, witness `a1@0+b1@0` is in target and missed); PC2 `ok=true` in four (misses 0 → 1). | Pass |
| Discriminative power self-test | Threshold 0 → maximum flips all four `leaks_on_some_streams` to `never_leaks` (lines 18–21). | Pass |

For the first half of Q7's flip observation, “some candidate misses a cross-transaction persistence subset”: on two-disk G4a/G4b, K2 (cut by transaction) misses no reachable cross-transaction persistence subset (L3b = 0); K1 (as before, cut at root-slot writes) misses reachable cross-transaction subsets depending on the recorded issued stream (max 702 / 729) and misses none only on the best stream. This does not establish or refute a decision, which requires the three-party process; it is this E167 cell's reading.

### Segment-three verdict lines identified

`grep -n 'name=verdict' research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out` gives five lines (25–29), no boolean `false` or `not_run`. Counters containing `misses`: `arm_by_root_publish_true_cross_misses_maximum` (702 for g4a_free/g4b_free, 498 for g4a_ordered/g4b_ordered) and `arm_by_root_publish_streams_with_true_cross_misses` (155115151, 37833148, 155115956, 37833930) are registered expected readings (K1 misses on some streams), not violations. `arm_by_transaction_true_cross_misses=0` in all four. No `ok=false`, `=false`, or `not_run` in output (`grep -c 'ok=false'` = 0).

### Segment-three unit tests and mutations

59 tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs`; `cargo test --release` reports `59 passed; 0 failed`), seven more than segment two: hand-worked target-set example; hand-worked segment-sequence membership (three writes, two segments); three membership methods (mask, table, enumeration) agree vector by vector on sampled one-disk streams; l11 in four one-disk cells matches independent Python and in-memory per-stream enumeration including min/max streams; two ordered two-disk one-unit cases (A txg residues 0 and 2) and one free case; all self-checks pass in one segment-three cell. Independent Python: `e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.py` (one-disk G1/G3, matching `.out`; T_exact = T on one disk so values equal segment-one l2) and `e167-multi-inflight-crash-cut-python-l11-two-disk-recount-2026-10-01.py` (one unit on two disks, verbatim `-a-ordered.out`, `-b-ordered.out`, `-a-free.out`; T_exact only 184 / 256 versus T = 82944, so exact filtering is sensitive). Python cannot recount full G4 (155117520 streams); agreement of two methods, one-disk and two-disk one-unit cross-checks, and positive controls pin it. Thirty-five mutations (M1–M22 prior segments, M23–M35 thirteen added): **35 caught, 0 invalid, 0 not red**, 💥, ⚠️, ⏱, 🧱 each 0; “0 memory ceiling hits (4G cap), 0 timeouts” and “restored, baseline still all green” (`e167-multi-inflight-crash-cut-mutate-2026-10-01-r5.log`, `mutate.sh` exit 0). M27 (positive-miss stream threshold), M24 (drop T_exact filter), M33 (check no greater than l2) have the same values as original on four one-disk cells (T_exact = T, l11 = l2). Two-disk one-unit samples added under `mutation-sampling.md` category 3 caught them. All 35 caught establishes only written assertions were mutation-tested, not coverage of all conclusion-related code.

### Segment-three replay

`bash scripts/experiment-apparatus/replay.sh E167` (crash enumeration cuts for multiple in-flight publications): (there are three rows, adding E167 (crash enumeration cuts for multiple in-flight publications) `|e167-multi-inflight-crash-cut|third|e167-multi-inflight-crash-cut-third-2026-10-01.out|exact`). Deterministic model: running N times demonstrates that there is no hidden state, rather than mere statistical stability.

### What segment three cannot answer

- Reference is T ∩ T_exact: T_exact is registration section 2's exact reachable set under device partial order (FUA writes precede every later write; writes before a barrier precede writes issued after it; torn in-place writes only at maximal elements). Whether real devices persist only according to this partial order is outside the model.
- No K1 misses of non-cross-transaction states versus T ∩ T_exact (only C is counted, same as L3b), nor K1 coverage of invalid prefixes versus T_exact.
- No independent Python recount of l11 on G4 (155117520 streams); independent recount covers one-disk G1/G3 and two-disk one-unit only.
- “Misses on some streams” counts every Σ_free / Σ_ord stream once without probability weighting; which streams the harness records is outside E167.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route), settled item 4 | Supporting material | 2026-10-18 unaffected: in the path register, the archived-out-of-tree one-off script is described as “no implementation” with archive commit specified; conclusions and numbers unchanged. |
| D13 (Verification Route), settled item 2 | Supporting material | 2026-10-18 unaffected: in the path register, the archived-out-of-tree one-off script is described as “no implementation” with archive commit specified; conclusions and numbers unchanged. |
| D16 (Publish Semantics), settled item 7 | Supporting material | 2026-10-18 unaffected: in the path register, the archived-out-of-tree one-off script is described as “no implementation” with archive commit specified; conclusions and numbers unchanged. |
| D16 (Publish Semantics), settled item 8 | Supporting material | 2026-10-18 unaffected: in the path register, the archived-out-of-tree one-off script is described as “no implementation” with archive commit specified; conclusions and numbers unchanged. |

“Should this cell be elevated to support?” The **evidence** sections of D13 items 4 and 2 and D16 items 7 and 8 do not cite E167 (Crash Enumeration Partitioning for Multiple In-Flight Publishes) (`grep -n 'E167（多在飞发布的崩溃枚举切法）' .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/16-发布语义.md` has no hits). On two disks K2 misses no reachable cross-transaction states (L3b = 0), while K1 misses none on some streams. Whether to elevate to support and how to judge Q7 are for the main agent after the three-party process; decision files were not changed.

## Historical Versions

### 2026-10-01: page created, segment one run

Apparatus, 37 unit tests, 12 mutations (12 caught), amendments 1–6, artifacts (first run invalid `void_conditions=1`, `-r2` official), and replay registration landed the same day. Segment two (l1, l2, l3, l4, l6, l10 and l8 on G4a/G4b) had not run.

### 2026-10-01: segment two run and handed back

S3 first (feasible); 52 tests, 22 mutations (22 caught), segment-two amendments 7–9, artifact `-second-2026-10-01-r3.out` (first invalid `void_conditions=1`, `-r2` and `-r3` identical), and replay registration landed the same day. The four decision-impact rows were reviewed against segment-two readings, all still supporting material, not elevated to support.

### 2026-10-01: one cell added to segment two (l11), run

Added K1 cross-transaction misses versus T ∩ T_exact (segment-three amendment 10): `third` subcommand, 59 tests, 35 mutations (35 caught), artifact `-third-2026-10-01.out` (`void_conditions=0`) and replay registration landed the same day. Prior outputs/readings unchanged (`second` and default reruns passed `cmp`). Four decision-impact rows reviewed against segment three, still supporting material.
