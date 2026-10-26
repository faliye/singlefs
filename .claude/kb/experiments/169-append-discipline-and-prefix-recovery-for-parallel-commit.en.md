## E169 Append Discipline and Prefix Recovery for Parallel Commits — Partially Run (segment three)

[中文](169-并行提交的追加纪律与前缀恢复.md) · **English** · [日本語](169-並列コミットの追記規律とプレフィックス復旧.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 H0 H1 H2 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 B1 B2 B3 B4 B5 B6 R1 R2 G1 G2 G3 G4 V1 V2 V3 V4 V5 V6 P1 P2 Q2-1 Q2-2 Q2-3 Q2-4 Q3-1 Q3-2 Q3-3 Q3-4 -->

```text
Local arm names and symbols on this page are lowercase (a1, a2, a3, a4 …). The pre-registration, apparatus, artifacts and commands use the corresponding uppercase forms, which match global registration identifiers. The knowledge-base body uses lowercase only to distinguish them from those identifiers; match against uppercase when reading artifacts.
```

Pre-registration `e169-preregistration.md` (section-twelve revisions 1 and 2 and an observation, written after unit tests and before the artifact). The fork sheet is Q2 (how to append records in transaction-number order) and Q3 (what recovery selects when a prefix contains complete B and incomplete A) in `m3-concurrency-questions.md`. Apparatus `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs`: independently handwritten model, importing no code or constants from `crates/`.

This is the first segment of registration 5.6: two Q1 conditions, “serial persistence” (parallel preparation, serial persistence) and “shared publish” (one publish contains multiple transactions) × four Q2 append arms (甲 whole, 甲 interleaved, 乙 whole, control 丁) × three history families (H0 all complete, 12 kinds; H1 cancellation/narrow, 32 kinds; H2 cancellation/wide, 16 kinds) × two readings of “appended” (R2 issued and covered by a completed barrier or FUA; R1 issued, an arm added by revision 1 item 4) × four Q3 recovery arms (丙0 follows the 2026-10-01 prefix checker, 丙1 truncates before A, 丙2 drops A and keeps B, 丙3 forbids the situation). Segment two (apparatus argument `segment-two`) uses two Q1 conditions “multiple in flight/root ordered” (one publish per transaction, arbitrary interleaving between publishes, F and C issued in txg order) and “multiple in flight/root unordered” (no F/C order constraint), the same four Q2 arms, H0/H1 (all four r values with k = 2; with k = 3 only r = (1, 1, 1) and (2, 1, 1), 20 histories total), two readings and four Q3 arms. k = 2 is enumerated directly, k = 3 deduplicated by configuration (registration 5.6 segment two). H2 under multiple in flight was not run after judgment was sufficient. Segment three (argument `segment-three`) is the registered 5.7 fsync-wait model: discrete events, abstract units, four Q1 conditions × three Q2 arms (甲 whole, 甲 interleaved, 乙 whole; 5.7 supplies no wait rule for control 丁, revision 4 item 1) × two preparation profiles (constant speed, slow head) × two cost samples (P1 device-dominated, P2 CPU-dominated) × r ∈ {1, 2} × two readings × k ∈ {1, 2, 4, 8, 16, 32, 64}, with readings per section-twelve revision 3 (five points settled by main agent) and revision 4. Additionally one path-3 (model 丙0) versus path-4 (`replay_core` in `crates/singlefs-replay-prefix`) direct differential: argument `replay-core-inputs` outputs recovery inputs and 丙0 results for every crash state in segments one and two; a temporary crate with path dependency on unchanged `crates/singlefs-replay-prefix` feeds the same inputs to `replay_core` for individual comparison. The third disjunct “instance switch or transition to read-only already decided” is outside this registration (revision 1 item 5).

Artifacts:

- `research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out`: official artifact, 5650 `E7RESULT` rows, final `E7RESULT name=done emitted=5650`. A rerun of the same binary is byte-identical (`cmp` silent; invalidation V3 not triggered).
- Two attempts before the official artifact produced no output: first memory-wrapper exit 251 (limit specified `auto` but command absent from peak table; no rows run), second stderr redirection hit `noclobber` and apparatus did not start. Both zero-byte outputs were deleted; `-r3` is the artifact.
- `e169-parallel-commit-append-and-prefix-recovery-mutate-2026-10-01.log`: full `mutate.sh` output for segment-one mutation table M1–M13.
- `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out`: official segment-two artifact, 2792 `E7RESULT` rows, ending `E7RESULT name=done emitted=2792`. Same binary rerun byte-identical (`cmp` silent; V3 not triggered). The same binary without arguments produces segment-one output byte-identical to `-r3` (adding segment two did not change segment one).
- First segment-two attempt produced no output: memory-wrapper exit 252 (another executor was already running in the same session and no reason for parallel execution was supplied); no apparatus row ran. The zero-byte file was deleted and segment two uses `-r2`.
- `e169-parallel-commit-append-and-prefix-recovery-mutate-2026-10-01-r2.log`: full `mutate.sh` output for 22 segment-one/two mutations.
- `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out`: official segment-three artifact, 2039 `E7RESULT` rows ending `E7RESULT name=done emitted=2039`. A rerun of the same binary is byte-identical (`cmp` silent, V3 not triggered). After adding segment three, the same binary without arguments and with `segment-two` still matches `-r3` and the segment-two `-r2` bytewise.
- `research/results/e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out`: output from the apparatus run with the `replay-core-inputs` argument, containing 7176 lines and ending with `E7RESULT name=done emitted=7176`. Running the same binary a second time produces output that is identical byte for byte.
- `e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01.out` and `e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01-source.txt`: output (2 rows) of the temporary crate feeding those inputs to `replay_core`, and its source (`Cargo.toml`, `src/main.rs`, build/run commands and then-current SHA-256 of `crates/singlefs-replay-prefix/src/lib.rs`).
- `e169-parallel-commit-append-and-prefix-recovery-mutate-2026-10-01-r3.log`: full `mutate.sh` output for the 29 mutations in segments one, two and three.
- `e169-parallel-commit-append-and-prefix-recovery-t1-crates-check-2026-10-01.out` and `e169-parallel-commit-append-and-prefix-recovery-t1-crates-check-2026-10-01-source.txt`: the other side of stop condition t1, fourteen sequences fed unchanged to the 2026-10-01 `replay_core` in `crates/singlefs-replay-prefix/src/lib.rs` (temporary crate path-depends on it, without changing `crates/`), output and source.

### Results copied as complete artifact rows

The following rows come from `research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out`, prefixed with its line numbers. Every verdict row is identical under R1 and R2: take the verdict rows for each reading for the six quantities, remove `reading=`, and compare line by line; `diff` outputs zero lines (command and output in the report). Only R2 is quoted below.

Stop condition t1, B anchors, positive controls:

```
16:E7RESULT name=t1_summary sequences=14 mismatches=0
311:E7RESULT name=anchor_summary families=B1,B2,B3,B4 checked=294 mismatches=0
4932:E7RESULT name=control_summary controls_not_positive=0
```

The fourteen rows on the `crates/` side (`e169-parallel-commit-append-and-prefix-recovery-t1-crates-check-2026-10-01.out`) apply publish counts 1, 1, 0, 0, 1, 0, 2, 1, 0, 0, 0, 0, 0, 0 and record counts 1, 2, 0, 0, 2, 0, 2, 1, 0, 0, 0, 0, 0, 0, matching registered section-seven a2 expectations and model 丙0 item by item.

Q2 rows (illegal prefixes, legal histories H0 ∪ H1). Of all 16 Q2-1/Q2-2 verdicts, only 甲 interleaved and control 丁 under shared publish hit:

```
5053:E7RESULT name=verdict quantity=Q2-1 q1=shared_publish q2=jia_interleaved reading=r2_covered legal_cells=44 legal_cells_positive=11 legal_states=125 judgement=hit
5054:E7RESULT name=verdict quantity=Q2-2 q1=shared_publish q2=jia_interleaved reading=r2_covered legal_cells=44 legal_cells_positive=20 legal_states=101 judgement=bing3_pairing_out
5101:E7RESULT name=verdict quantity=Q2-1 q1=shared_publish q2=ding_control reading=r2_covered legal_cells=44 legal_cells_positive=11 legal_states=666 judgement=hit
5102:E7RESULT name=verdict quantity=Q2-2 q1=shared_publish q2=ding_control reading=r2_covered legal_cells=44 legal_cells_positive=28 legal_states=522 judgement=bing3_pairing_out
```

All four arms under serial persistence, and 甲 whole and 乙 whole under shared publish, have Q2-1/Q2-2 `legal_states=0` (e.g. `5029:E7RESULT name=verdict quantity=Q2-1 q1=shared_publish q2=jia_whole reading=r2_covered legal_cells=44 legal_cells_positive=0 legal_states=0 judgement=not_hit`). Zero for whole-segment arms follows from their definition (registered section four item 1), so serves only as a4 absolute-value assertion, not a verdict cell; same for 乙 whole under shared publish.

Q3 rows: none of 64 Q3-1 (V_fsync) verdict rows has nonzero `legal_states`; all 16 Q3-3 (V_dangling) verdict rows have `judgement=all_zero`. Only 丙2 hits:

```
5060:E7RESULT name=verdict quantity=Q3-2 q1=shared_publish q2=jia_interleaved reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=20 legal_states=69 judgement=hit
5108:E7RESULT name=verdict quantity=Q3-2 q1=shared_publish q2=ding_control reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=36 legal_states=845 judgement=hit
4992:E7RESULT name=verdict quantity=Q3-4 q1=serial_publishes q2=yi_reserved_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=26 legal_states=76 judgement=hit
5040:E7RESULT name=verdict quantity=Q3-4 q1=shared_publish q2=jia_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=36 legal_states=120 judgement=hit
5064:E7RESULT name=verdict quantity=Q3-4 q1=shared_publish q2=jia_interleaved reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=36 legal_states=281 judgement=hit
5088:E7RESULT name=verdict quantity=Q3-4 q1=shared_publish q2=yi_reserved_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=44 legal_states=322 judgement=hit
5112:E7RESULT name=verdict quantity=Q3-4 q1=shared_publish q2=ding_control reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=44 legal_cells_positive=36 legal_states=1306 judgement=hit
```

Q3-4 for 乙 whole under serial persistence comes from placeholders left by narrow cancellation: the placeholder precedes the next transaction in its publish, and when placeholders are complete but that transaction incomplete, 丙2 applies only the placeholders, partially applying that publish.

Absolute-value assertions (section-seven A class):

```
5125:E7RESULT name=assertion check=A1 q1=serial_publishes q2=jia_whole q3=bing0_today reading=r2_covered legal_cells=44 n2a=0 n2b=0 v_fsync=0 v_dep=0 v_dangling=0 partial_publish=0 n_atom=0 nonzero_quantities=0
5127:E7RESULT name=assertion check=A3 histories=legal cells=704 bing0_n_atom=0 bing1_n_atom=0 bing2_n_atom=0 bing3_n_atom=0
5128:E7RESULT name=assertion check=A3 histories=h2 cells=192 bing0_n_atom=0 bing1_n_atom=0 bing2_n_atom=0 bing3_n_atom=0
5139:E7RESULT name=assertion check=A5_versions q1=serial_publishes q2=yi_reserved_whole reading=r2_covered legal_cells=44 legal_cells_with_differing_states=26 differing_states=76 bing0_today_differs=0 bing1_truncate_before_a_differs=0 bing2_drop_a_keep_b_differs=76 bing3_forbid_differs=0
5141:E7RESULT name=assertion check=A5_versions q1=shared_publish q2=jia_whole reading=r2_covered legal_cells=44 legal_cells_with_differing_states=36 differing_states=120 bing0_today_differs=0 bing1_truncate_before_a_differs=0 bing2_drop_a_keep_b_differs=120 bing3_forbid_differs=0
5143:E7RESULT name=assertion check=A5_versions q1=shared_publish q2=yi_reserved_whole reading=r2_covered legal_cells=44 legal_cells_with_differing_states=44 differing_states=322 bing0_today_differs=0 bing1_truncate_before_a_differs=0 bing2_drop_a_keep_b_differs=322 bing3_forbid_differs=0
5145:E7RESULT name=assertion check=A5_v_dep q1=serial_publishes q2=all reading=r2_covered bing0_today_v_dep=0 bing1_truncate_before_a_v_dep=0 bing3_forbid_v_dep=0
5147:E7RESULT name=assertion check=A5_v_dep q1=shared_publish q2=all reading=r2_covered bing0_today_v_dep=0 bing1_truncate_before_a_v_dep=0 bing3_forbid_v_dep=0
5149:E7RESULT name=assertion check=A6 cells=896 strict_segment_length_peak=6 bound=6 in_flight_limit=65536 in_flight_limit_states=0
5150:E7RESULT name=assertion check=A7 cells=896 a7_violation_points=0
5151:E7RESULT name=assertion check=pending_trajectory cells=896 pending_final_peak=0 pending_never_drained=0
```

All eight a4 rows (N2a/N2b for 甲 whole and 乙 whole in legal cells, both conditions, both readings) are zero (5129–5136). The first half of a5 (the four Q3 arms recover the same version state by state in every 甲 whole and 乙 whole cell) **does not hold**: all differences are in 丙2; 丙0, 丙1 and 丙3 are pairwise equal state by state. Serial persistence × 甲 whole has zero differences (line 5137). The deduction in registered section four item 1 considered only “two partial transactions” and pattern P, not partial application by 丙2 when a complete earlier transaction is followed by an incomplete one, or when placeholders are complete and the following transaction incomplete. A-class mismatch is not invalidation under registration; refer to main agent. The revision's “observation” was recorded from unit-test readings before the artifact.

### Line-by-line verdict (segment one, only for serial persistence and shared publish Q1 conditions; R1/R2 identical)

| Quantity | Serial persistence | Shared publish |
|---|---|---|
| Q2-1 N2a (numbers consecutive but contents belong to two partial transactions) | All four arms 0 | 甲 interleaved 125, control 丁 666 hit; 甲 whole and 乙 whole 0 by definition |
| Q2-2 N2b (pattern P, states 丙3 judges bad) | All four arms 0: 丙3 pairing with all four stands | 甲 interleaved 101, 丁 522: 丙3 pairing with those two eliminated; pairing with 甲 whole and 乙 whole stands |
| Q3-1 V_fsync (fsync returned but recovery loses it) | All four recovery × four append arms 0 | All 0 |
| Q3-2 V_dep (B applied without all of lower-numbered A) | All 0 | Only 丙2: 甲 interleaved 69, 丁 845; 丙2 with 甲 whole or 乙 whole is 0 |
| Q3-3 V_dangling | All 0 | All 0 |
| Q3-4 丙2 partial application | 乙 whole 76 (placeholders), remaining three 0 | All four append arms hit: 甲 whole 120, 甲 interleaved 281, 乙 whole 322, 丁 1306 |

- Q3-1 all-zero has a structural cause: under these two conditions every crash point at which an incomplete transaction appears in the record segment occurs before its barrier completes, when no transaction has returned. This was deduced in advance in registration section four item 2. The “early return” positive control is nonzero for every append arm × recovery arm × reading, demonstrating detection of that violation (lines 4864–4879, values 2–5).
- Under shared publish 丙2 cannot “keep B” without partial application: if a publish has an incomplete transaction and another complete transaction, 丙2 applies only part of that publish, conflicting with D16 (Publish Semantics) settled item 4, “a publish is applied in full or not at all” (registered F6; refer to main agent whether 丙2 is eliminated under shared publish or item 4 reopened, not decided by this experiment).
- Registered F11 (all eliminated) did not trigger: under shared publish 丙1 has zero Q3-1, Q3-2 and Q3-4.

H2 (wide cancellation, listed separately, appearing only in histories forbidden by D23 (Journal Role & Format) settled item 7): under serial persistence, 丙3 has four states with “judged bad and nonempty returned set” for each of 甲 interleaved, 乙 whole and control 丁 (lines 5636, 5638, 5640); under shared publish 丙2 with 甲 interleaved and 丁 has V_dep 13 and 216 (5644, 5648). H2 does not apply to 甲 whole (5634, 5642 `applicable=no`).

### Positive controls

76 `name=control` rows, zero `positive=no` (control_summary). Sample cells follow registration 5.2/5.3: removing each of three Q2 arms' own discipline gives N2a = 6, N2b = 4 at shared publish r = (2, 2); early return 2–5; V_dep control 5–20; N_atom control 5; removing two barriers gives V_dangling 12; 丙3 judges bad 1; 丙2 partial application 10. Each was also run under R1: early-return control 4–7 (4902–4917; under R1 a later transaction can return once the earlier record is issued, producing more crash points with larger returned sets than R2); other control values match R2.

### Geometry sensitivity (registered 8.2 G1/G2)

14 of 480 rows are `stability=unstable` (seven each under R1/R2): Q2-1/Q2-2 for 甲 interleaved and 丁 under shared publish hit at main r = (2, 2), but not r = (1, 1) or k = 3, r = (1, 1, 1); Q2-2 for 甲 interleaved and Q3-2 for 丙2 hit at r = (2, 1) but not r = (1, 2). One record per transaction cannot produce a partial transaction, precisely the registered opposite-direction sample. Record “unstable” per registration; verdicts aggregate all legal cells.

### Segment-two (multiple in flight) results copied as whole artifact rows

The following rows come from `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out`, prefixed by line number. All 192 verdict rows under R1/R2 are line-identical after removing `reading=` (`diff` outputs zero lines); only R2 is quoted.

Layout reading for multiple in flight (not verbatim in registration, inferred from its wording and fixed before segment-two artifact): txg values are consecutive in uncancelled transaction-number order (txg calculated afresh from previous version; narrow-cancelled transactions start no publish). 乙 whole placeholders are written in the publish immediately following the cancelled transaction; if the cancelled transaction is last, start a separate empty publish containing only S, placeholders, F, C, interleaved arbitrarily like other publishes, with F/C ordered by txg under root order (same reading as registration 5.2 “no permanent holes” and revision 2 item 1 serial persistence).

Anchors, path differential, stop condition t3:

```
1955:E7RESULT name=anchor_summary families=B5 checked=32 mismatches=0
1956:E7RESULT name=path_check paths=direct,by_configuration cells=192 state_set_mismatches=0 execution_mismatches=0 trajectory_mismatches=0 quantity_mismatches=0 a7_zero_mismatches=0
1922:E7RESULT name=t3 k=3 cells=128 largest_configurations=19416 limit=100000000 exceeded_cells=0 k3_cells_kept=yes
```

B5 comprises execution counts and deduplicated-state counts for 16 cells in registered `anchors_q1c.py` (example `1936:E7RESULT name=anchor family=B5 q1=multi_flight_roots_unordered q2=yi_reserved_whole r=1,1 quantity=states source=script expected=77 measured=77 matches=yes`). Path differential recalculates all 192 k = 2 cells with configuration deduplication and compares state sets, execution counts, trajectories and quantities to direct enumeration. k = 3 uses configuration deduplication, maximum 19416 configurations; t3 did not trigger.

Q2 rows: only 甲 interleaved and 丁 hit, identically under both subforms:

```
1981:E7RESULT name=verdict quantity=Q2-1 q1=multi_flight_roots_ordered q2=jia_interleaved reading=r2_covered legal_cells=20 legal_cells_positive=1 legal_states=3 judgement=hit
1982:E7RESULT name=verdict quantity=Q2-2 q1=multi_flight_roots_ordered q2=jia_interleaved reading=r2_covered legal_cells=20 legal_cells_positive=5 legal_states=12 judgement=bing3_pairing_out
2077:E7RESULT name=verdict quantity=Q2-1 q1=multi_flight_roots_unordered q2=jia_interleaved reading=r2_covered legal_cells=20 legal_cells_positive=1 legal_states=3 judgement=hit
2078:E7RESULT name=verdict quantity=Q2-2 q1=multi_flight_roots_unordered q2=jia_interleaved reading=r2_covered legal_cells=20 legal_cells_positive=5 legal_states=12 judgement=bing3_pairing_out
2029:E7RESULT name=verdict quantity=Q2-1 q1=multi_flight_roots_ordered q2=ding_control reading=r2_covered legal_cells=20 legal_cells_positive=1 legal_states=3 judgement=hit
2030:E7RESULT name=verdict quantity=Q2-2 q1=multi_flight_roots_ordered q2=ding_control reading=r2_covered legal_cells=20 legal_cells_positive=5 legal_states=22 judgement=bing3_pairing_out
```

甲 whole and 乙 whole have Q2-1/Q2-2 zero in both subforms (e.g. `2101:E7RESULT name=verdict quantity=Q2-1 q1=multi_flight_roots_unordered q2=yi_reserved_whole reading=r2_covered legal_cells=20 legal_cells_positive=0 legal_states=0 judgement=not_hit`); all eight a4 rows are zero (2151–2158). N2a is positive only in H0 r = (2, 2) (3 states); positive N2b cells are H0 r = (2, 1), (2, 2), (2, 1, 1), and H1 r = (2, 1, 1) with t2 or t3 cancelled.

Q3: none of 64 Q3-1 (V_fsync) rows has nonzero `legal_states`. Q3-2 (V_dep) hits only 丁, with 42 for all four Q3 arms (2032/2034/2036/2038; same in root-unordered 2128–2134). Q3-3/Q3-4:

```
2039:E7RESULT name=verdict quantity=Q3-3 q1=multi_flight_roots_ordered q2=ding_control reading=r2_covered bing0=20 bing1=20 bing2=20 bing3=20 legal_cells_where_arms_differ=0 judgement=same_nonzero_across_arms_f8
2111:E7RESULT name=verdict quantity=Q3-3 q1=multi_flight_roots_unordered q2=yi_reserved_whole reading=r2_covered bing0=552 bing1=552 bing2=552 bing3=552 legal_cells_where_arms_differ=0 judgement=same_nonzero_across_arms_f8
2135:E7RESULT name=verdict quantity=Q3-3 q1=multi_flight_roots_unordered q2=ding_control reading=r2_covered bing0=488 bing1=488 bing2=488 bing3=488 legal_cells_where_arms_differ=0 judgement=same_nonzero_across_arms_f8
2016:E7RESULT name=verdict quantity=Q3-4 q1=multi_flight_roots_ordered q2=yi_reserved_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=20 legal_cells_positive=10 legal_states=54 judgement=hit
2112:E7RESULT name=verdict quantity=Q3-4 q1=multi_flight_roots_unordered q2=yi_reserved_whole reading=r2_covered q3=bing2_drop_a_keep_b legal_cells=20 legal_cells_positive=10 legal_states=54 judgement=hit
```

Absolute-value assertions:

```
2149:E7RESULT name=assertion check=A3 histories=legal cells=320 bing0_n_atom=0 bing1_n_atom=0 bing2_n_atom=0 bing3_n_atom=0
2161:E7RESULT name=assertion check=A5_versions q1=multi_flight_roots_ordered q2=yi_reserved_whole reading=r2_covered legal_cells=20 legal_cells_with_differing_states=10 differing_states=54 bing0_today_differs=0 bing1_truncate_before_a_differs=0 bing2_drop_a_keep_b_differs=54 bing3_forbid_differs=0
2167:E7RESULT name=assertion check=A5_v_dep q1=multi_flight_roots_ordered q2=all reading=r2_covered bing0_today_v_dep=42 bing1_truncate_before_a_v_dep=42 bing3_forbid_v_dep=42
2171:E7RESULT name=assertion check=A6 cells=320 strict_segment_length_peak=4 bound=6 in_flight_limit=65536 in_flight_limit_states=0
2172:E7RESULT name=assertion check=A7 cells=320 a7_violation_points=0
2173:E7RESULT name=assertion check=pending_trajectory cells=320 pending_final_peak=0 pending_never_drained=0
2174:E7RESULT name=assertion check=core_failures cells=320 core_failures=0
```

- First half of a5 holds for 甲 whole (2159/2163 are zero), but fails for 丙2 with 乙 whole (54 states, same type as segment one: placeholder records and following transaction in one publish, 丙2 applies only part). Record as A-class mismatch, not invalidation, per main agent's segment-one decision.
- Second half of a5, “丙0, 丙1 and 丙3 have V_dep = 0 in every cell”, **does not hold** under multiple in flight: all 42 states are 丁 k = 3 H0 cells (21 each for r = (1, 1, 1) and (2, 1, 1), same in both subforms); 甲 whole, 甲 interleaved and 乙 whole have zero. 丁 assigns jsn in issue order, so txg 3 records can receive a smaller jsn than txg 2. The 丙0 transcription of `lib.rs` checks txg only within the buffered publish (`:463`); after applying the previous publish it does not stop when the next record has a larger txg, so it skips unapplied txg 2 and applies txg 3. A-class mismatch, not invalidation; refer to main agent. This describes the 2026-10-01 prefix core under undisciplined control 丁, not a candidate arm.

### Line-by-line verdict (segment two: multiple in flight/root ordered or unordered only; R1/R2 identical; subforms equal except Q3-3)

| Quantity | Root ordered | Root unordered |
|---|---|---|
| Q2-1 N2a | 甲 interleaved 3, 丁 3 hit; 甲 whole and 乙 whole 0 | Same |
| Q2-2 N2b (丙3 bad) | 甲 interleaved 12, 丁 22: their 丙3 pairings eliminated; 甲 whole and 乙 whole pairings stand | Same |
| Q3-1 V_fsync | All four recovery × four append arms 0 | All 0 |
| Q3-2 V_dep | Only 丁 hits, 42 for all four recovery arms; zero for all four on three candidate append arms | Same |
| Q3-3 V_dangling | Only 丁 20, same for all four recovery arms: F8, hit does not distinguish arms | 乙 whole 552, 丁 488, same for all four recovery arms: F8; 甲 whole/interleaved 0 |
| Q3-4 丙2 partial application | 乙 whole 54; other three 0 | Same |

- F11 (all eliminated): considering only three candidate append arms, 丙1 and 丙3 trigger neither F4 nor F5 in either subform, so F11 does not trigger. If control 丁 is also counted literally as a Q2 arm, 丙1/丙3 trigger F5 (V_dep 42) and 丙2 F6, making literal F11 hold. But 丙0 has the same V_dep 42 and all four recovery arms match: a hit that does not distinguish arms (`.claude/singlefs-ai-sop/rules/evidence-discipline.md`, “criteria themselves can be wrong: classify a hit first”). Main agent must decide whether 丁 is an F11 input.
- Per registered section-ten F8, V_dangling is a shared premise of Q1 conditions, not a Q3 verdict: in root-unordered, 乙 whole and 丁 persist a later publish's root before an earlier publish's units; in root-ordered only 丁 does. Both 甲 arms are zero under root-unordered; record two points without extrapolation.
- Geometry sensitivity: `name=geometry_summary lines=600 unstable_lines=56` (2775). All eight G4 flips are 乙 whole Q3-3 (root-ordered zero, unordered hit). G1/G2 flips occur for 甲 interleaved/丁 Q2-1/Q2-2 (one record per transaction cannot make a partial transaction, same direction as segment one) and 丁 Q3-2/Q3-3. Verdicts aggregate all legal cells.
- H2: both Q2/Q3 verdict rows under the two multiple-in-flight conditions have values in H0 ∪ H1, so H2 was not run after judgment was sufficient (2776–2791 `name=h2_summary`, 甲 whole `applicable=no`, remaining three `run=not_run_after_decidable`).

### Segment three (fsync wait model), complete rows copied from artifact

Rows below are from `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out`, prefixed by line number. All 384 Q2-3/Q2-4 verdict rows and 128 pairing rows are line-identical under R1/R2 after removing `reading=`; only 20 of 672 cells per reading differ in W_mean (root-unordered × 乙 whole × slow head, both cost points × r = 1/2 × k = 2–32; under R1 later transactions return as soon as the slow head's record is issued). W_max and critical-path components agree in every cell. Only R2 is quoted.

Anchors (registered section-seven B6) and internal assertions:

```
102:E7RESULT name=wait_anchor_summary families=B6 checked=100 mismatches=0
2032:E7RESULT name=wait_assertion check=critical_path_sum cells=1344 mismatches=0
2033:E7RESULT name=wait_assertion check=pending_trajectory cells=1344 pending_final_nonzero=0 pending_peak_after_drain_nonzero=0
2034:E7RESULT name=wait_assertion check=reserved_lag_trajectory cells=448 reserved_lag_final_nonzero=0 reserved_lag_largest_initial_or_rebound=127
```

B6 covers W at k = 1, r = 1 across four conditions × three arms × two profiles × two cost points × two readings (甲 4w + 4b + h, 乙 4w + 4b, plus Δ for slow head), and 8.41 / 8.4 for serial persistence k = 2. The apparatus asserts per cell that critical-path sum n_bar·b + n_dev·w + n_cpu·(h or g) + starting time = W_max (revision 4 item 4); all 1344 pass. 乙 whole's maximum lag (largest issued jsn − smallest reserved unissued jsn) is 127: multiple in flight or shared publish × slow head × r = 2 × k = 64 when t2…t64 finish writing first while t1's two slots remain empty.

Q2-3 (ρ = W_max(64) ÷ W_max(8)), counts by Q1 condition:

```
2035:E7RESULT name=wait_summary quantity=Q2-3 q1=serial_publishes verdict_lines=48 grows=48 bounded=0 between_not_judged=0
2036:E7RESULT name=wait_summary quantity=Q2-3 q1=shared_publish verdict_lines=48 grows=36 bounded=0 between_not_judged=12
2037:E7RESULT name=wait_summary quantity=Q2-3 q1=multi_flight_roots_ordered verdict_lines=48 grows=48 bounded=0 between_not_judged=0
2038:E7RESULT name=wait_summary quantity=Q2-3 q1=multi_flight_roots_unordered verdict_lines=48 grows=48 bounded=0 between_not_judged=0
```

None of 192 Q2-3 verdicts is “bounded”. Main sample (uniform, r = 1), 甲 whole and 乙 whole verdicts:

```
1447:E7RESULT name=wait_verdict quantity=Q2-3 q1=serial_publishes q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=33.640 w_max_k64=269.120 rho=8.0000 judgement=grows
1511:E7RESULT name=wait_verdict quantity=Q2-3 q1=serial_publishes q2=yi_reserved_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=33.600 w_max_k64=268.800 rho=8.0000 judgement=grows
1543:E7RESULT name=wait_verdict quantity=Q2-3 q1=shared_publish q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=4.905 w_max_k64=10.505 rho=2.1417 judgement=grows
1607:E7RESULT name=wait_verdict quantity=Q2-3 q1=shared_publish q2=yi_reserved_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=4.900 w_max_k64=10.500 rho=2.1429 judgement=grows
1551:E7RESULT name=wait_verdict quantity=Q2-3 q1=shared_publish q2=jia_whole profile=uniform point=p2_cpu_dominated r=1 reading=r2_covered w_max_k8=8.550 w_max_k64=39.350 rho=4.6023 judgement=grows
1615:E7RESULT name=wait_verdict quantity=Q2-3 q1=shared_publish q2=yi_reserved_whole profile=uniform point=p2_cpu_dominated r=1 reading=r2_covered w_max_k8=7.100 w_max_k64=35.100 rho=4.9437 judgement=grows
1639:E7RESULT name=wait_verdict quantity=Q2-3 q1=multi_flight_roots_ordered q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=19.605 w_max_k64=149.805 rho=7.6412 judgement=grows
1703:E7RESULT name=wait_verdict quantity=Q2-3 q1=multi_flight_roots_ordered q2=yi_reserved_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=19.600 w_max_k64=142.800 rho=7.2857 judgement=grows
1735:E7RESULT name=wait_verdict quantity=Q2-3 q1=multi_flight_roots_unordered q2=jia_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=12.605 w_max_k64=97.805 rho=7.7592 judgement=grows
1799:E7RESULT name=wait_verdict quantity=Q2-3 q1=multi_flight_roots_unordered q2=yi_reserved_whole profile=uniform point=p1_device_dominated r=1 reading=r2_covered w_max_k8=12.600 w_max_k64=79.800 rho=6.3333 judgement=grows
```

All 12 “between, not judged” rows are shared publish × slow head × P1 (three arms × r = 1/2 × two readings), ρ 1.29–1.59; e.g. `1559:E7RESULT name=wait_verdict quantity=Q2-3 q1=shared_publish q2=jia_whole profile=slow_head point=p1_device_dominated r=1 reading=r2_covered w_max_k8=9.555 w_max_k64=12.355 rho=1.2930 judgement=between_not_judged`. Slow-head Δ = 5 dominates W, so the k-dependent increment (one write per transaction and h) is relatively small.

F9/F10 pairing (128 rows; `grep 'name=wait_pair' 产物 | awk …clause… | sort | uniq -c`, command and raw output in executor report): 120 `f9_both_grow`, 8 `neither_f9_nor_f10` (shared publish × slow head × P1, both arms “between”), zero F10 (one grows, other bounded). Shared growing components: serial persistence and both multiple-in-flight subforms n_bar and n_dev; shared publish × P1 n_dev; shared publish × P2 n_cpu (甲 append h and 乙 reservation g both on critical path). Examples:

```
1831:E7RESULT name=wait_pair q1=serial_publishes profile=uniform point=p1_device_dominated r=1 reading=r2_covered first=jia_whole second=yi_reserved_whole first_judgement=grows second_judgement=grows first_growing=n_bar,n_dev,n_cpu second_growing=n_bar,n_dev shared_growing=n_bar,n_dev clause=f9_both_grow
1871:E7RESULT name=wait_pair q1=shared_publish profile=uniform point=p2_cpu_dominated r=1 reading=r2_covered first=jia_whole second=yi_reserved_whole first_judgement=grows second_judgement=grows first_growing=n_dev,n_cpu second_growing=n_cpu shared_growing=n_cpu clause=f9_both_grow
1927:E7RESULT name=wait_pair q1=multi_flight_roots_unordered profile=uniform point=p1_device_dominated r=1 reading=r2_covered first=jia_whole second=yi_reserved_whole first_judgement=grows second_judgement=grows first_growing=n_bar,n_dev second_growing=n_bar,n_dev shared_growing=n_bar,n_dev clause=f9_both_grow
```

Geometry sensitivity, registration 8.2 G3: `2031:E7RESULT name=wait_geometry_summary id=G3 lines=72 unstable_lines=14`. All 14 flips are shared publish: main P1/uniform/r = 1 is “grows”, with n_dev growing; switching to slow head yields “between” (6 rows), switching to P2 or r = 2 still yields “grows” but adds or replaces the growing component with n_cpu (8 rows). All 54 serial-persistence and multiple-in-flight rows stable.

### Line-by-line verdict (segment three, second clause of Q2; four Q1 conditions judged separately; R1/R2 identical)

| Q1 condition | Q2-3 (3 arms × 2 profiles × 2 points × r = 1/2 × 2 readings, 48 rows each) | Q2-4 shared growing components | Section ten |
|---|---|---|---|
| Serial persistence | All 48 “grows”, ρ 7.09–8.00 | n_bar, n_dev (both 甲 arms also n_cpu; 乙 whole n_cpu always 0) | F9: hit does not distinguish arms |
| Shared publish | 36 “grows”, ρ 2.14–5.76; 12 “between” (slow head × P1 × 3 arms × r = 1/2 × 2 readings, ρ 1.29–1.59) | P1 n_dev, P2 n_cpu | F9 for “grows”; “between” not judged |
| Multiple in flight/root ordered | All 48 “grows”, ρ 5.70–8.00 | n_bar, n_dev | F9 |
| Multiple in flight/root unordered | All 48 “grows”, ρ 5.29–8.00 | n_bar, n_dev | F9 |

- No cell has “one arm grows, the other bounded”: F10 did not trigger, and Q2's second clause reverses no arm under any Q1 condition. Both candidates' W_max grows with k through shared components (each transaction's barrier and FUA, writes on the single-queue device, and each arm's single CPU service desk under shared publish P2). Under F9 record as shared premises, do not judge Q2.
- Under serial persistence n_cpu grows with k for both 甲 arms but not 乙 whole (0), supporting reversal against 甲 under Q2-4's “n_cpu grows in only one Q2 arm” wording. Yet 乙 whole has ρ = 8.00 in that same cell, so F10's “other arm bounded” does not hold and no reversal follows.
- These numbers are a discrete-event model in abstract units with two registered cost samples; local throughput and fsync waits belong to C488 (group commit amortizes write bytes, unamortized half unmeasured), outside this segment.

### Direct differential of path 3 with path 4 (registration section twelve revision 4 item 9)

The following rows are from `research/results/e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out` and `e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01.out`:

```
7172:E7RESULT name=replay_core_input_states segment=one cells=896 states=44254
7173:E7RESULT name=replay_core_input_states segment=two cells=320 states=44306
7174:E7RESULT name=replay_core_input_summary input=all_durable distinct_inputs=6192 states=88560
7175:E7RESULT name=replay_core_input_summary input=strict_segment distinct_inputs=979 states=88560
1:E169DIRECT name=summary input=all_durable distinct_inputs=6192 states=88560 mismatched_inputs=0 mismatched_states=0
2:E169DIRECT name=summary input=strict_segment distinct_inputs=979 states=88560 mismatched_inputs=0 mismatched_states=0
```

- The 88560 crash states in the two official artifacts (896 segment-one cells, 320 segment-two cells, both readings counted) are deduplicated into 6192 and 979 inputs for “all durable records” and “strict contiguous segment”. Model 丙0 and `replay_core` agree for each input on whether an error is reported, number applied, jsn of last applied record and maximum applied transaction number; zero mismatches.
- Discriminative power: add one to the 丙0 applied count in each of 424 input rows with nonzero counts (237 all-durable, 187 strict segment) and feed again; same binary reports 237 and 187 mismatches respectively (done on a draft; output not checked in).
- The 42 states from segment-two a5's second half (multiple in flight × 丁 k = 3, 丙0 skips unapplied txg 2 and applies txg 3) are among these 88560 states and `replay_core` matches 丙0 for each. This is the 2026-10-01 prefix core's result on histories without append discipline, not a transcription error. Ten inputs contain applied records skipping a txg (208 states, counting both readings and both subforms; counted by inspecting the txg of records applied by 丙0 in each input row, command in executor report). Segment-one a5 first-half 丙2 is not covered here (`replay_core` matches only 丙0).
- The differential uses the workspace's 2026-10-01 `crates/singlefs-replay-prefix/src/lib.rs` (sha256 at end of source artifact). Its interface differs from the segment-one t1 comparison version (new types wrap jsn, txg and transaction number, last-record flag is an enum). The old t1 temporary source is inferred not to compile against it, but compilation was not attempted and t1 was not rerun. Whether all fourteen sequences appear among these inputs was not checked.

### Mutations

30 unit tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs`; 24 in segment one, six added in segment two: B5 anchors on each of two enumeration paths, k = 2 per-cell comparison of the paths, k = 3 execution counts without cross-publish constraints equal multinomial coefficients 24! / 8!³ and 25! / (9! 8! 8!), early-return control positive for all four recovery arms under multiple in flight, root-unordered states contain root-ordered states, and segment-two histories 6 + 14 kinds). Mutation table `scripts/mutation/tables/e169_parallel_commit_append_and_prefix_recovery.tsv` contains registered section-nine M1–M15 plus seven “segment-two supplement” mutations (cross-publish order, jsn rank for each enumeration path, configuration-deduplicated return, closed segments and trajectories): all 22 caught, invalid 0, not failing 0. `mutate.sh` ends “memory cap 0 (limit 4G), timeouts 0” and “restored, baseline still green” (`e169-parallel-commit-append-and-prefix-recovery-mutate-2026-10-01-r2.log`). M14 is caught by B5 test (root-ordered 甲 whole r = (1, 1), 3747 executions); M15 by B5, the k = 2 path comparison and segment-one early-return test. `if back_chain_broken(index)` in `lib.rs` (reverse chain) is never broken in the single-instance model; deleting it is an equivalent mutation, documented by `back_chain_is_never_broken_in_a_single_instance_without_residue` and excluded from table.

After segment three: 38 unit tests (same `grep -c` gives `38`; eight added: B6 original numbers, serial k = 2 8.41 / 8.4, four hand-derived absolute values and critical paths, registration 8.2 threshold-movement discriminative power self-test, end-of-trajectory zero over the whole grid). Mutation table adds M16/M17 and five “segment-three supplements” (barrier merging, root ordering waits for prior root slot, R2 coverage, serial persistence waits for prior publish, reservation cost), total 29: caught 29, invalid 0, not failing 0, 💥/⚠️/⏱/🧱 all 0. `mutate.sh` ends “memory cap 0 (limit 16G), timeouts 0”, “restored, baseline still green”, exit 0 (`e169-parallel-commit-append-and-prefix-recovery-mutate-2026-10-01-r3.log`). M16 is caught by threshold movement, root-unordered hand-derived path and serial k = 2 tests; M17 by B6 originals and multi-flight 乙 whole slow head. Direct comparison is in a temporary crate and outside mutation-table coverage; discrimination was tested by corrupting 424 input rows and reporting 424 mismatches (on draft).

### Replay

`bash scripts/experiment-apparatus/replay.sh E169`: four registered rows, segment one without arguments pointing at `e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out`; segment two argument `segment-two` pointing at `e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out`; segment three argument `segment-three` pointing at `e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out`; recovery input argument `replay-core-inputs` pointing at `e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out`; all exact. Individually: from `research/`, `cargo build --release --bin e169-parallel-commit-append-and-prefix-recovery`, then run `./target/release/e169-parallel-commit-append-and-prefix-recovery` (segment one, ~7 MB peak) or with `segment-two` (deterministic single-thread model, local wall clock 15 seconds, `/usr/bin/time -v` max resident 65372 KB including wrapper), `segment-three` (memory wrapper recorded 4 MB peak), or `replay-core-inputs`. Direct comparison is not in `replay.sh`: follow `e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01-source.txt` to create a temporary crate (copy repository-root `Cargo.lock`), run `cargo run --release --offline -- <恢复输入那份产物>`, compare with `...-replay-core-direct-check-2026-10-01.out`.

### Path and conclusion registration

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Apparatus crash enumeration | For each execution split at barriers/FUA, take durable subsets at every crash point, deduplicate by content | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:936 collect_crash_states` | Registration 1.2 write order and segmentation rules; revision 2 item 1 txg and partial-publish reading |
| 5 | Direct multiple-in-flight enumeration | Each within-publish record order × every allowed interleaving between publishes; assign jsn in global issue order, then rank by jsn within each publish to fix ordinal and last record, then segment per path 1 | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:1373 multi_flight_executions`, `:1273 execution_from_merge` | Path 1 segmentation and return conditions (`collect_crash_states`); cross-publish constraints `:1203 next_operation_may_issue`; within-publish write order `:1087 publish_template` |
| 6 | Configuration deduplication | Configuration = progress of each publish, progress at last completed closed-segment point, whether an incomplete closed-segment point was issued, jsn of issued records; calculate crash states per configuration, independently recalculate return conditions and trajectories | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:1413 ConfigurationSearch`, `:1629 visit` | Cross-publish constraints `:1203 next_operation_may_issue`; within-publish order and ordinal/last record precomputed by publish order in `:1087 publish_template` |
| 2 | Embedded registered scripts | Python execution enumeration and closed forms (state and execution counts only) | No implementation: three one-off scripts embedded in pre-registration (`anchors.py`, `anchors_void.py`, `anchors_tongfa.py`), not repository executables (pre-registration archived in 31128273, original in its parent commit) | Same write order and segmentation rules |
| 3 | Model 丙0 | Transcription of `replay_core` main loop, check by check | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:1890 walk_prefix` | Fourteen sequences of registered section-seven a2 |
| 4 | `crates/` prefix core | Direct `replay_core` call | `crates/singlefs-replay-prefix/src/lib.rs:3543 replay_core`; segment-one t1 call using its then-interface and direct-comparison call recorded in their two `-source.txt` artifacts, archived in 31128273 | Fourteen sequences; recovery input for every segment-one/two crash state (emitted by `run_replay_core_inputs` in `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs`: model's durable records, whether named units verified, selected root txg) |
| 7 | Wait model discrete events | Build steps from 5.7 write-order and order constraints, simulate three FIFO service desks for device, appender and reservation, select return time by two 1.2 conditions and trace critical path by reading ④ | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:3411 plan_wait_scenario`, `:3567 simulate_wait`, `:3739 critical_path_of`, `:3825 evaluate_wait` | Cost samples and Δ (`CostPoint::costs`, `PreparationProfile::ready_time_of`); registration 5.7 write order; revisions 3/4 readings |
| 8 | B6 closed form | k = 1, r = 1: 4w + 4b + (h, 甲) + (Δ, slow head); two serial k = 2 values copied from registration | `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs:3944 anchor_b6_single_transaction` | Cost samples and Δ (same `CostPoint::costs`, `ready_time_of` as path 7) |

Paths 1 and 2 are connected by tests (apparatus constants `ANCHOR_B3`, `ANCHOR_B4_SCRIPT` copy script output, plus B1/B2/B4 closed forms); 294 artifact anchors have zero mismatches. Paths 3 and 4 connect twice: both compare fourteen sequences against the apparatus's fixed expectation table (`fixed_sequences`, derived from `lib.rs` in registration, not a direct differential); both separately calculate recovery for inputs from 88560 segment-one/two crash states and a temporary crate compares every input directly, zero mismatches. This assertion is in temporary-crate source (checked in as source text, not in a crate compiled by gates). Paths 3/4 share inputs: durable-record set, whether named units verified and selected root txg are all supplied by the model, so an error there makes both wrong but equal; paths 1/2/5/6 anchors and comparisons pin them. Paths 7/8 are linked by 100 B6 artifact anchors with zero mismatches and unit tests; they share cost constants and Δ, so miscopying constants makes both wrong; unit tests pin literal registered 4.205, 4.7, 9.205 etc. Path 7 also has hand-derived absolute values in tests (root-unordered 甲 whole k = 2: 5.405 and full critical path; shared P2 8.55 / 39.35 and 7.1 / 35.1; multi-flight 乙 whole slow-head return times for both readings). These share only registration 5.7 rules and revisions 3/4 readings with path 7; if the reading itself is wrong both are wrong. Paths 1/2 share write order and segmentation rules and could agree while both misreading them; positive controls and several hand-derived absolute-value tests pin those rules. Values sweep registered histories, not sampled points. Paths 5/6 are linked by artifact `name=path_check` (192 k = 2 cells compared for state sets, execution counts, trajectories, quantities; zero mismatches) and unit test `configuration_search_matches_direct_enumeration_on_two_transaction_multi_flight_histories`; both separately compare against B5's 16 cells from registered `anchors_q1c.py`. They share cross-publish constraints and within-publish order, so misreading either makes both wrong but equal; independent Python B5 and k = 3 multinomial execution-count tests pin them. k = 3 has only path 6, no second comparison path.

### What it cannot answer

- Segment-three wait is a discrete-event model in abstract units: one disk, one FIFO device queue, two chosen P1/P2 cost points, not local timing. Local throughput and per-fsync wait belong to C488 (group commit amortizes write bytes, unamortized half unmeasured). Only H0 ran, with the same r for each transaction and all k calling fsync at time 0; control 丁 has no wait rules (revision 4 item 1).
- In the segment-three equal-time critical-path reading, placement of “cross-publish order constraints” and “ready time” predecessors, and choice among simultaneously latest transactions, are revision 4 items 2/3 additions attacked zero rounds; they change only critical-path components, not W_max or ρ.
- Direct comparison only covers 丙0 (2026-10-01 prefix core); 丙1/丙2/丙3 have no `crates/` implementation to compare. Inputs (durable records, unit verification, selected root) come from the model, so enumeration errors on the model side are invisible to the comparison.
- Under multiple in flight k = 3 ran only r = (1, 1, 1) and (2, 1, 1) (registration 5.6 segment two), with only the configuration-deduplication path; H2 not run after sufficient judgment; only early-return positive control was run for multi-flight, in unit tests (registered 5.2/5.3 control sample cells are serial/shared, already run in segment one).
- Multi-flight readings “txg consecutive by uncancelled transaction number” and “乙 whole opens a separate empty publish with placeholders when the final transaction is cancelled” were inferred from registration, not stated verbatim; alternatives are outside this experiment.
- Third disjunct “instance switch or transition to read-only already decided” is outside registration.
- One disk, one instance, no residual records or ring wraparound; k ≤ 3, r ≤ 2; one shared unit per publish (registration section eleven's unmodeled list and section 8.2 knobs not swept).
- R1/R2 match line by line for serial/shared because earlier transaction records are issued before the later transaction's last barrier; verdicts also match under both multi-flight forms (segment-two artifact). The readings' difference was measured only in the early-return positive control in segment one.
- Cells where “partial application”/V_dep hit 丙2 use registered 丙2 (install the entire new-root segment of the publish carried by the greatest jsn). Another 丙2, e.g. install only through the last complete transaction's root, is outside this experiment.
- H2 semantics (abandon partial publish, reuse txg) are a reading added by revision 2 item 1, attacked zero rounds; H2 is excluded from judgment.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 2 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |
| D13 (Verification Route) settled item 10 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |
| D16 (Publish Semantics) settled item 4 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |
| D16 (Publish Semantics) settled item 7 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |
| D23 (Journal Role & Format) settled item 7 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |
| D23 (Journal Role & Format) settled item 18 | Evidence preparation | 2026-10-11 unaffected: wording only (temporal words, location references, self-reference); conclusions, numbers and artifacts unchanged |

## Historical Versions

### 2026-10-01: page created, segment one run

Apparatus, 24 unit tests, 13 mutations (all caught), official artifact and replay registration landed the same day; section twelve records revisions 1/2 and an observation. Segments two and three not run.

### 2026-10-01: segment two (multiple in flight) run

Added `segment-two` argument (segment-one output unchanged, byte-identical to `-r3`); tests increased to 30, mutations to 22 (all caught); segment-two artifact `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out` and replay registration landed. Section twelve records the main agent's decisions on two segment-one issues. Segment three not run.

### 2026-10-01: segment three (wait model) and direct path 3 versus path 4 comparison run

Section twelve records revision 3 (five readings settled by main agent) and revision 4 (executor's additional readings), after unit tests and before artifacts. Added `segment-three` and `replay-core-inputs` (segment-one/two outputs unchanged, byte-identical to `-r3`/`-r2`); tests increased to 38, mutations to 29. Segment-three artifact `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out`, recovery inputs `research/results/e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out`, direct-comparison output and temporary-crate source landed; two replay entries added. Readings: no Q2-3 cell “bounded” under any of four Q1 conditions, F10 not triggered, both-grow cells recorded under F9 as shared components; model 丙0 and `replay_core` have zero mismatches over 88560 crash states.
