## E161 Deduplication and Segmentation Timing of Crash Amplification — Feasibility Tier Run (2026-09-27), Second Rerun Run (sufficient to decide, 2026-10-05), Third Rerun Run (sufficient to decide, 2026-10-26)

[中文](161-崩溃放量的去重与分段耗时.md) · **English** · [日本語](161-クラッシュ増幅の重複排除と段階別所要時間.ja.md)

Pre-registration `research/prompts/e161-preregistration.md`; question list `m3-gpu-dedup-r1-forks.md` (G1–G5). As directed by the user, this run only checks whether the apparatus can run and is usable; **all G1–G5 rows remain unjudged (only the feasibility tier ran)**. Registration section 12.2 documents the narrowing. Checked-in apparatus `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` (`feasibility` mode). In the first run all 13 deterministic unit tests passed; 12 mutations M1–M12 appended to `crates/mutations.tsv` are assigned to the submission-time checker-tier-crates-mutation-replay gate and were not run here.

Second rerun (2026-10-05): a barrier was added after system configuration rotation and before publish returns (at the end of `persist_the_root_then_rotate_the_system_configuration` in `crates/singlefs-core/src/transaction.rs`, commit c3540c02). The new-pool file creation stream changed from 10 to 11 segments, and the first registration's literal domain “all states in segments with fewer than 26 writes” became 16,777,260 states. The feasibility tier was rerun after redefining the domain. Four question rows: row 1 has three-way equality per segment, row 2 domain is fixed and finishes, **row 3 has seven cells reversed relative to the first run**, row 4 has all 14 inline tests passing and 16 mutations assigned to the submission gate. See “Second rerun.” In the second rerun the apparatus had 14 inline tests and 16 mutations M1–M16 in `crates/mutations.tsv`.

Third rerun: the two-round number acquisition write (`ACQUISITION_WRITE_ROUNDS` in `crates/singlefs-core/src/transaction.rs`) adds one 2-write segment to every number acquisition, so the new-pool file creation stream goes from 11 to 12 segments and the fixed-script overwrite stream from 78 to 80 segments; the apparatus's pinned values were redefined accordingly and the sampled segments of the fixed-script overwrite stream moved to 12–28. Per the registration revisions, the feasibility tier's judging stage was built with the `gpu` feature and ran on the GPU after a quiesce. Four question rows: row 1 has all four numbers equal per segment, row 2 has a fixed domain that finishes with an object in every cell, **row 3 has 0 cells flipped relative to the second rerun** (the timing cells ran at a final load of 60.16 on 32 cores, and the registration has no clause voiding them because of load), row 4 has all 15 inline tests passing and 16 mutations assigned to the submission gate. See “Third rerun”; the apparatus now has 15 inline tests and 16 mutations M1–M16 in `crates/mutations.tsv`.

### What was done

- Sampling domain (registration 12.2 F1): all states in new-pool file-creation stream segments with fewer than 26 writes plus the fully persisted state, 37 total; the full fast-tier domain for arm 甲二 for both streams, 54 + 84; and the 4096 smallest segment-local indices for each of two 26-write segments (new-pool creation segment 6, fixed script overwrite segment 9).
- For each state do four things under 5.1: replay and time by phase (same order as `evaluate_state_recording_findings` in `crates/singlefs-checker-tier/src/crash.rs`); rerun recovery twice and checker twice on recording readers (one normal, one tree walk with unit candidates cleared); unit-level check replay; emit six keys K2-consult, K2-ignore, K2-pair, K4-walk, K4-units, K4-full.
- Stop conditions S1–S6, all section-seven anchors and 5.3 positive controls were tested only on this small domain (12.2 F2).
- End-to-end apparatus completed: artifact 171 lines, final `name=done emitted=171`, exit 0.

### Result rows copied from artifacts (rows moved out of the workspace represented by one-line pointers)

Artifact `e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`.

Stop conditions and positive controls (for the reasons for every `false` field in the verdict row, see the paragraph “Individual explanations of the 12 `false` fields in the verdict row (line 170)” at the end of the section “Results copied as whole rows from the artifacts”):

```
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 17 行（name=stop_clause_crates）
E7RESULT name=stop_clause_crates_per_state cell=first_segment_head states=4096 clause_S4_holds=true
E7RESULT name=stop_clause_crates_per_state cell=second_segment_head states=4096 clause_S4_holds=true
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 26 行（name=stop_clause_determinism）
E7RESULT name=stop_clause_determinism cell=second_small_and_head control_states=4096 compared_states=4096 recorded_consult_report_differs=0 recorded_ignore_report_differs=0 recorded_verdicts_differ=0 second_run_recovery_outcome_changed=0 second_run_verdict_changed=0 walk_unit_reads_outside_normal_unit_reads=0 clause_S5_holds=true clause_S6_holds=true
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 27 行（name=positive_control_same_state_twice）
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 38 行（name=positive_control_refeed）
E7RESULT name=positive_control_device_memory cell=first_segment_head p1_grew=32768 p3_grew=147 p2_grew=7 p4_per_state_grew=48 pc5_holds=true
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 167 行（name=positive_control_timing）
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 60 行（name=positive_control_remove_one_position）
```

Reuse arms (readings only to establish apparatus usability, not decisions):

```
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 32 行（name=reuse_arm）
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 33 行（name=reuse_arm）
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 154 行（name=reuse_arm）
```

Inconsistency and hit counts for each arm in each cell (counted from `name=reuse_arm`; command under “Rerun”): K2-consult, K2-ignore, K2-pair and K4-full have zero inconsistencies in all five cells. K4-walk has 3 in `first_small` and 3 in `first_quick`; K4-units has 6 in each of those and 3 in `second_quick`, zero in other cells. K4-full has zero hits in every cell (one key per state).

Two G1 counts and rough G3 counts:

```
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 28 行（name=g1）
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 37 行（name=g3）
```

Across five cells, pool-level checker takes 0.74–0.91 of per-state total time, the two recovery passes together 0.07–0.17, record checker 0.02–0.08, oracle less than 0.003 (`name=g3`). Q3g (each unit-read replay + bitwise CRC in header scan + record checks) exceeds total per-state time in all five cells (`unit_events_share_ratio` 1.32–1.62). Q3f exceeds checker time in 10.8% and 22.2% of states in `first_small` and `first_quick`, exceeding V3's 1%; under V3 Q3f–Q3i are invalidated in those two cells.

The 12 `false` fields in verdict line 170 are all expected by registration, not apparatus failures:

- Six `first_segment_head_k2_consult_constant_key_separates=false`, `first_segment_head_k2_ignore_constant_key_separates=false`, `first_segment_head_k4_full_constant_key_separates=false`, `second_segment_head_k2_consult_constant_key_separates=false`, `second_segment_head_k2_ignore_constant_key_separates=false`, `second_segment_head_k4_full_constant_key_separates=false`: the 4096 states at each 26-write segment head have only one outcome and judgment (`distinct_outcomes=1`), so replacing the key with a constant cannot distinguish outcomes; PC2b / PC4b cannot be tested in these two cells. The same arms are `true` on `first_small`, `first_quick`, `second_quick`.
- `first_k4_full_pc2c_pc4c_found_a_position=false`: K4-full has one key per state in the new-pool creation control sample (0 hits). Removing any position produces no same-key/different-judgment case, so PC4c cannot find a discriminating position. Registration 11.1's next sentence says this does not invalidate the run; only the arm's “inconsistency = 0” cannot close G4.
- Five `second_k2_consult_pc2c_pc4c_found_a_position=false`, `second_k2_ignore_pc2c_pc4c_found_a_position=false`, `second_k4_walk_pc2c_pc4c_found_a_position=false`, `second_k4_units_pc2c_pc4c_found_a_position=false`, `second_k4_full_pc2c_pc4c_found_a_position=false`: fixed script overwrite control sample contains only the 4096 states at segment 9's head (feasibility excludes segments 10–22); only one outcome, so no removed position creates an inconsistency.
- Five `g1`–`g5` fields are `not_judged_feasibility_only`: feasibility tier does not judge G rows (12.2 F3).

### Second rerun: sampling domain redefined for 11-segment stream (2026-10-05)

Rerun registration `research/prompts/e161-r2-prereg.md` (sha256 `ccd7e3f2b014855e815867e05f62b10eecaed94f4c390926c1ad4617f48de5a0`), question list `e161-r2-questions.md` four rows; first registration `research/prompts/e161-preregistration.md` unchanged. Domain uses interpretation A (1.4): within sampled segments, sample only 24-write segments, fully enumerate segments with fewer than 24 writes; `STRIDED_SEGMENT_WRITES` changes 26 to 24. New-pool file-creation stream has 11 segments of lengths `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`, small domain 45 states excluding fully persisted. Fixed-script overwrite sampled segments change to 11–26, small domain 393,261, quick tier 52. For each 24-write segment (new-pool stream segment 7 and overwrite stream segment 11), take the 4096 smallest segment-local indices.

Runs and readings (apparatus `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`, sha256 `bb2e28924e290de50aa188f7b9dc4b19a087fe7261b0f0a487f39008917e12d9`, workspace file at run time):

| Step | Executor | Command and reading |
|---|---|---|
| Inline tests | Main agent, prefix `SINGLEFS_HEAVY_TESTS=user-request`, 12G, 1800-second limit, `--test-threads=4` | `cargo test --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split` exit 0, `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.16s`; current `grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` gives 14; peak memory 2,230,321,152 bytes (dated row in `~/.local/share/singlefs/memory-peaks.tsv`) |
| Feasibility tier | Main agent, prefix `SINGLEFS_HEAVY_TESTS=user-request`, `E161_THREADS=10`, 8G, 1800-second limit | Exit 0, artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` 186 lines, final `E7RESULT name=done emitted=186`; peak memory 1,647,312,896 bytes (line 7096 of same file) |
| Judgment | Executor, reads the two artifacts only | `e161-crash-state-dedup-and-time-split-feasibility-r2-judge-2026-10-05.py` (sha256 `2bb9e5fc8478d432ec8798f12a46d66af23338e28fb82528bb3ce431f4310acb`), three outputs: `e161-crash-state-dedup-and-time-split-feasibility-r2-pcr3-2026-10-05.out`, `e161-crash-state-dedup-and-time-split-feasibility-r2-compare-2026-10-05.out`, `e161-crash-state-dedup-and-time-split-feasibility-r2-rows-2026-10-05.out` |

Counts by artifact row family (one command):

```
sed 's/^E7RESULT //' research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out | awk '{print $1}' | sort | uniq -c
     89 name=anchor_segment
      2 name=anchor_stream
      1 name=config
      5 name=determinism
      1 name=done
      5 name=g1
      5 name=g3
      5 name=g5_contents
      2 name=positive_control_device_memory
      1 name=positive_control_refeed
     10 name=positive_control_remove_one_position
      2 name=positive_control_same_state_twice
      6 name=positive_control_timing
     30 name=reuse_arm
      1 name=stop_clause_constants
      3 name=stop_clause_crates
      2 name=stop_clause_crates_per_state
      2 name=stop_clause_determinism
      1 name=stop_clause_shared_prefix
      2 name=stop_clause_stream
     10 name=stop_clause_tally
      1 name=verdict
```

#### Row 1: do current segment lengths, plan counts, crates and independent anchors still agree per segment? Yes, sufficient to decide

Artifact lines 3, 4, 5 (segment shape and prefix, registration S2):

```
E7RESULT name=stop_clause_stream stream=first writes=41 segments=11 roots=3 segment_lengths_match=true writes_match=true roots_match=true
E7RESULT name=stop_clause_stream stream=second writes=481 segments=78 roots=19 segment_lengths_match=true writes_match=true roots_match=true
E7RESULT name=stop_clause_shared_prefix bases_identical=true shared_prefix_segments=11 shared_prefix_segments_identical=true clause_S2_holds=true
```

Compare four numbers per segment: apparatus plan counts individually, crates expands only that segment, apparatus formula, and independent anchors (output of arithmetic script under `/tmp` copied to registration section thirteen). First four lines of `e161-crash-state-dedup-and-time-split-feasibility-r2-rows-2026-10-05.out`:

```
== 行 1：Q1b、Q1c
first 段数 11 四个数不全相等的段数 0 逐个数 true 的段数 11
second 段数 16 四个数不全相等的段数 0 逐个数 true 的段数 16
second 的 anchor_segment 行数 78 three_agree 非 true 的段 []
```

New-pool stream has 0 unequal segments; all 11 counted individually, including 24-write segment 7 with 16,777,215 states (artifact line 13 `counted_by_the_plan=16777215 counted_by_the_crates=16777215 counted_by_the_formula=16777215 three_agree=true`). Inline test `first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor` passes, but does not count segment 7 individually; that count occurs only in feasibility. Fixed-script overwrite sampled segments 11–26 likewise have all four numbers equal.

#### Row 2: after redefining the sampling domain, is it fixed, runnable and does every criterion have an object? Sufficient to decide

Artifact lines 17 and 96 (domain sizes and anchors, full rows):

```
E7RESULT name=anchor_stream stream=first sampled_segment_states=[8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8] domain_states=16777260 by_publish=[12, 12, 16777227, 8] d1_small=45 d1_stride=7260 d1_quick=46 d1_heads=4096 recorded_writes=41 in_place_overwrites=8 replays=0 w=49 segments_match=true domain_states_match=true by_publish_match=true d1_small_match=true d1_stride_match=true d1_quick_match=true d1_heads_match=true in_place_overwrites_match=true w_formula_holds=true
E7RESULT name=anchor_stream stream=second sampled_segment_states=[16777215, 3, 1, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1] domain_states=17170476 by_publish=[0, 0, 0, 16777219, 262163, 65547, 65547, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] d1_small=393261 d1_stride=7260 d1_quick=52 d1_heads=4096 recorded_writes=481 in_place_overwrites=46 replays=0 w=527 segments_match=true domain_states_match=true by_publish_match=true d1_small_match=true d1_stride_match=true d1_quick_match=true d1_heads_match=true in_place_overwrites_match=true w_formula_holds=true whole_stream_states=1678377204 whole_stream_largest_segment_states=268435455 whole_stream_states_match=true whole_stream_largest_segment_match=true
```

Runnable: both inline-test and feasibility wrappers exit 0 (not 250 or 253); peaks 2,230,321,152 and 1,647,312,896 bytes. Every cell has an object: latter part of `e161-crash-state-dedup-and-time-split-feasibility-r2-rows-2026-10-05.out` ends “number of cells without an object 0 []”. K4-full has ≥ 2 distinct outcomes only in three cells: new-pool small 8, new-pool fast 8, overwrite fast 5; both segment-head cells have 1. Interpretations A and B give the same domain; B was deleted by the main agent before the apparatus was written. Their comparison remains only in registration 1.4 and section-thirteen arithmetic; the apparatus implements A only.

#### Row 3: do first-run conclusions still have the same direction? Seven cells reversed, sufficient to decide

Comparison is fixed in registration section six: pair by key; compare direction by category for booleans, pinned zero, required positive, outcome count, unreadable and G-row judgment; do not compare timings or counts. First demonstrate the comparison script fails as intended, conclusion from `…-r2-pcr3-2026-10-05.out`:

```
== PCr3 结果：① 翻面 0 格（要求 0）；② 翻面 2 格（要求 2）
== PCr3 通过
```

Two alterations: first artifact verdict line 170 `clause_S2_holds` true → false, and line 29 `inconsistent` 0 → 3; script reports exactly those two cells. Actual comparison `e161-crash-state-dedup-and-time-split-feasibility-r2-compare-2026-10-05.out` (first `…-feasibility-2026-09-27.out` versus second):

```
键数 第1次/第2次: 85 85
比了的字段数（按类）: {'布尔': 153, '钉零': 88, '读不到': 5, '要正': 70, '结局种数': 30, 'G行判定': 5} 合计 351
翻面格数: 7
  翻面 [要正] 键=positive_control_remove_one_position cell=first_small_and_head arm=k4_full 字段=discriminating_positions 第1次(行64)=0 第2次(行144)=1
  翻面 [布尔] 键=positive_control_remove_one_position cell=first_small_and_head arm=k4_full 字段=found_at_least_one 第1次(行64)=false 第2次(行144)=true
  翻面 [要正] 键=reuse_arm cell=first_segment_head arm=k4_units 字段=hits 第1次(行44)=3640 第2次(行124)=0
  翻面 [布尔] 键=reuse_arm cell=first_segment_head arm=k4_units 字段=keys_at_or_above_half 第1次(行44)=false 第2次(行124)=true
  翻面 [要正] 键=reuse_arm cell=second_segment_head arm=k4_units 字段=hits 第1次(行143)=3640 第2次(行158)=0
  翻面 [布尔] 键=reuse_arm cell=second_segment_head arm=k4_units 字段=keys_at_or_above_half 第1次(行143)=false 第2次(行158)=true
  翻面 [布尔] 键=verdict scope=feasibility 字段=first_k4_full_pc2c_pc4c_found_a_position 第1次(行170)=false 第2次(行185)=true
只在一边的键数: 0
只在一边的字段数: 15
```

One verdict field reversed, the other six occur in reuse arms and position-removal controls. All five G fields `g1`–`g5` remain `not_judged_feasibility_only` in both, no reversal. Fifteen fields appearing only in run two (`d1_heads`, `d1_heads_match`, `whole_stream_*`, six `pc3_segment_holds` rows, `shared_prefix_segments`, listed in comparison output) are additional tightened checks and cannot be compared. Full rows for reversed cells (second artifact lines 124, 144, 158; first lines 44, 64, 143):

```
e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 44 行
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 44 行（name=reuse_arm）
research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out 第 124 行
E7RESULT name=reuse_arm cell=first_segment_head arm=k4_units states=4096 inconsistent=0 hits=0 distinct_keys=4096 keys_over_states=1.000000 keys_at_or_above_half=true constant_key_inconsistent=0 distinct_outcomes=1 new_keys_peak_after_cold_start=0 new_keys_positive_intervals=1/1 new_keys_final=4096 new_keys_last_four=[4096] inconsistent_peak_after_cold_start=0 inconsistent_positive_intervals=0/1 inconsistent_final=0 inconsistent_last_four=[0]
e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 64 行
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 64 行（name=positive_control_remove_one_position）
research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out 第 144 行
E7RESULT name=positive_control_remove_one_position cell=first_small_and_head arm=k4_full control_states=4141 positions=4169 discriminating_positions=1 first_ten=[13:1:4194304:512] inconsistent_without_removal=0 found_at_least_one=true
```

The cell for the head of the overwrite fixed-script stream segment (use `awk 'NR==143'` to extract it from the first artifact and `awk 'NR==158'` to extract it from the second artifact):

```
e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 143 行
产物行见 git show 5d0b7e7c1667:research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out 第 143 行（name=reuse_arm）
research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out 第 158 行
E7RESULT name=reuse_arm cell=second_segment_head arm=k4_units states=4096 inconsistent=0 hits=0 distinct_keys=4096 keys_over_states=1.000000 keys_at_or_above_half=true constant_key_inconsistent=0 distinct_outcomes=1 new_keys_peak_after_cold_start=0 new_keys_positive_intervals=1/1 new_keys_final=4096 new_keys_last_four=[4096] inconsistent_peak_after_cold_start=0 inconsistent_positive_intervals=0/1 inconsistent_final=0 inconsistent_last_four=[0]
```

Known domain differences for reversed cells (registration section ten F3 table, inferred from segment shapes at design time, not checked state by state): new-pool position-removal control (including verdict field) uses small domain plus head. Small domain grows 37 → 45 (eight added states are torn or unpersisted second warm-up rotation, with no units persisted); all 4096 head states change (first run: nine combinations of two rotations times subsets of roughly nine earlier unit writes; this run: all subsets of first 12 unit writes with rotation fully persisted). Control states 4133 → 4141. Both K4-units head cells likewise have entirely changed head states. Cells without reversals include quick tiers (new-pool 54 → 46, overwrite 84 → 52), timing positive control (same write batch, segment 14 → 16), and other reuse arms; all judgment fields on fast and timing controls remain same direction. **No attribution is made here**: domain change, implementation change or both cannot be separated. `crates` changed after first artifact too: `git log --oneline b970d824..HEAD -- crates/singlefs-core crates/singlefs-checker crates/singlefs-checker-tier/src/crash.rs` has 7 entries; `git diff HEAD --stat -- crates` ends `109 files changed, 37442 insertions(+), 2566 deletions(-)`, including changes to `crates/singlefs-core/src/mount.rs` 955 lines, `crates/singlefs-checker/src/walk.rs` 1537, `crates/singlefs-checker/src/image.rs` 90, `crates/singlefs-checker-tier/src/crash.rs` 25. A supplemental one-thread run (C3b) was still owed at this point to rule out thread count.

Verdict line 185's fields valued `false` or `not_judged_feasibility_only` (the only verdict row listed by `grep -n 'name=verdict'`):

- Six `first_segment_head_k2_consult_constant_key_separates`, `first_segment_head_k2_ignore_constant_key_separates`, `first_segment_head_k4_full_constant_key_separates`, `second_segment_head_k2_consult_constant_key_separates`, `second_segment_head_k2_ignore_constant_key_separates`, `second_segment_head_k4_full_constant_key_separates`: both segment-head cells have one outcome (`distinct_outcomes=1`); constant keys cannot separate different outcomes, so this control cannot be tested there. All six were also false in the first artifact, as expected.
- Five `second_k2_consult_pc2c_pc4c_found_a_position`, `second_k2_ignore_pc2c_pc4c_found_a_position`, `second_k4_walk_pc2c_pc4c_found_a_position`, `second_k4_units_pc2c_pc4c_found_a_position`, `second_k4_full_pc2c_pc4c_found_a_position`: fixed script overwrite control sample has only 4096 head states, all five `discriminating_positions` are 0 (lines 174–178); also false in first artifact, expected.
- `first_k4_full_pc2c_pc4c_found_a_position`: false in first, true this time, therefore not among false fields; the verdict cell reversed in row 3.
- Five `g1`–`g5` are `not_judged_feasibility_only`: feasibility does not judge G rows, expected.

Verdict row contains only boolean fields and the five G fields, no integer fields or `not_run`. Outside verdict, reuse-arm `inconsistent` is 7 in K4-walk and K4-units on `first_small` and `first_quick` (first run 3 and 6), and 6 on `second_quick` K4-units (first 3). They remain in “≥ 1” category, no reversal (expected, only apparatus-usability readings). Other violation-like counts have no nonzero rows: `grep -c ' [a-z_0-9]*\(differs\|collisions\|mismatches\|changed\|_differ\)=[1-9]' research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` returns 0; replacing `[1-9]` with `0` returns 17 rows.

#### Row 4: inline tests and mutations

All 14 inline tests pass (`test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.16s`; passed count equals current grep count 14, failed 0, ignored 0), including new `second_stream_shape_and_sampled_domains_have_the_registered_sizes` and A4's `stride_and_quick_domains_have_the_registered_sizes`. Current `grep -c '[E]161 crash_state_dedup' crates/mutations.tsv` counts 16 mutations (M1–M12 sample points rewritten, M13–M16 new). Caught, invalid and not-failing counts belong to submission gate checker-tier-crates-mutation-replay; not run here.

### Third rerun: segment shapes and pinned values redefined after the two-round number acquisition write (2026-10-26)

Rerun registration `research/prompts/e161-r3-prereg.md` (sha256 `7499335b7e0ce8b273e0a50f00914462c65e60286b52bb763cc5b399ee0d227c`); the question list is the four rows of registration 1.2. The three revisions in registration section twelve (12.1 changes how command 3 is started, 12.2 changes how command 2 is started, 12.3 is an erratum) were all written before any valid artifact, and the reading follows them. The cause is the two-round number acquisition write (`ACQUISITION_WRITE_ROUNDS` in `crates/singlefs-core/src/transaction.rs`: every number acquisition writes in two rounds, each round writes the system configuration slot once on each of the two disks, and a pool barrier follows each round), so each number acquisition in the recorded stream turns from one 2-write segment into two. The new-pool file creation stream becomes 12 segments `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]` with 43 writes; its small domain excluding the fully persisted state is 53 states and its quick tier 54. The fixed-script overwrite stream becomes 80 segments with 485 writes; its sampled segments move, with the same intent as the second rerun, to 12–28 (17 segments), small domain 393,269 states, quick tier 60. For each of the two 24-write segments (new-pool file creation stream segment 8, fixed-script overwrite stream segment 12) the 4096 states with the smallest in-segment indices are taken. The segment of timing positive control PC3 moves to fixed-script overwrite stream segment 18 (the same batch of writes: the 18 unit writes of the row write).

What ran and the readings (apparatus `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`, sha256 `cc3954157aad6edf6a030369f0856c27589c4ea13c73dcd59e4a190af377fdab`, 6,379 lines, the workspace file at run time):

| Step | Who ran it | Command and readings |
|---|---|---|
| Inline tests (registration 12.2.1) | Main agent, prefix `SINGLEFS_HEAVY_TESTS=commit`, with `--features verdict-store,gpu`, quiesced through `scripts/crash/gpu-quiesce-run.sh`, 12G, time limit 1800 s, `--test-threads=1 --show-output` | Exit code 0, `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.25s`; current `grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` count 15; memory peak 6,287,261,696 bytes (the row with peak 6287261696 in `~/.local/share/singlefs/memory-peaks.tsv`) |
| Feasibility tier (registration 12.1.1) | Main agent, with the same feature and quiesce, `E161_THREADS=31`, `capped.sh 31`, 8G, time limit 1800 s | Exit code 0, artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out` (sha256 `f764a5cb503e8fdfc5a27cb83c3164e34ac526dc0491f7707bd39bd795ea62ac`) with 189 lines, final row `E7RESULT name=done emitted=189 loadavg1=60.16 cpus=32`; memory peak 6,884,950,016 bytes (the row with peak 6884950016 in the same file), 80.2% of 8G |
| Judging script | Executor, reads the two artifacts only | `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-judge-2026-10-26.py` (sha256 `5999529a06080cd0fbb62fced09fa3483ee200e26120443b0532dc04d2b276d3`, adapted from the second-rerun judging script: renaming table, anchors and the Q2c numbers changed); three outputs: `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-pcr3-2026-10-26.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-compare-2026-10-26.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-rows-2026-10-26.out` |

The first feasibility-tier trial run (2026-10-25) did not run the judging stage and is not an artifact (registration 12.1.4): it was started exactly as registration 5.8 command 3 without compiling in the `gpu` feature while the multi-host configuration had GPU enabled, so the judging stage of the crates enumeration only recorded and did not judge, and the apparatus exited 77. Output `~/.local/share/singlefs/work/m3-close-b16-e161-r3-run/c3-try1-not-run-without-gpu-feature.out` (sha256 `5a912d04bbde9d3320e31d3745fd6ac938860c709c2bc2d43ba755036e92f064`, 183 lines, last line `name=done emitted=183 loadavg1=14.24 cpus=32`), last line of the err file `exit=77`; it counts only as history, not as a source. The valid run was restarted per 12.1.1 with the feature and the quiesce.

Row counts per family in the artifact (counted by one command; compared with the second rerun only `anchor_segment` changes, 89 to 92, that is 12 + 80 segments):

```
sed 's/^E7RESULT //' research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out | awk '{print $1}' | sort | uniq -c
     92 name=anchor_segment
      2 name=anchor_stream
      1 name=config
      5 name=determinism
      1 name=done
      5 name=g1
      5 name=g3
      5 name=g5_contents
      2 name=positive_control_device_memory
      1 name=positive_control_refeed
     10 name=positive_control_remove_one_position
      2 name=positive_control_same_state_twice
      6 name=positive_control_timing
     30 name=reuse_arm
      1 name=stop_clause_constants
      3 name=stop_clause_crates
      2 name=stop_clause_crates_per_state
      2 name=stop_clause_determinism
      1 name=stop_clause_shared_prefix
      2 name=stop_clause_stream
     10 name=stop_clause_tally
      1 name=verdict
```

#### Third rerun row 1: do the new-pool file creation stream's segment lengths and plan counts still agree segment by segment with crates, the formula and the independent anchors? Yes, sufficient to decide

Artifact lines 3–5 (segment shape and prefix, registration S2):

```
E7RESULT name=stop_clause_stream stream=first writes=43 segments=12 roots=3 segment_lengths_match=true writes_match=true roots_match=true
E7RESULT name=stop_clause_stream stream=second writes=485 segments=80 roots=19 segment_lengths_match=true writes_match=true roots_match=true
E7RESULT name=stop_clause_shared_prefix bases_identical=true shared_prefix_segments=12 shared_prefix_segments_identical=true clause_S2_holds=true
```

Artifact lines 6–17 (the 12 segments of the new-pool file creation stream: apparatus plan counted one by one, crates expanding only that segment, apparatus formula):

```
E7RESULT name=anchor_segment stream=first segment=0 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true
E7RESULT name=anchor_segment stream=first segment=1 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true
E7RESULT name=anchor_segment stream=first segment=2 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=0 counted_by_the_plan=3 counted_by_the_crates=3 counted_by_the_formula=3 three_agree=true
E7RESULT name=anchor_segment stream=first segment=3 in_domain_counted_one_by_one=true writes=1 in_place_overwrites=0 counted_by_the_plan=1 counted_by_the_crates=1 counted_by_the_formula=1 three_agree=true
E7RESULT name=anchor_segment stream=first segment=4 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true
E7RESULT name=anchor_segment stream=first segment=5 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=0 counted_by_the_plan=3 counted_by_the_crates=3 counted_by_the_formula=3 three_agree=true
E7RESULT name=anchor_segment stream=first segment=6 in_domain_counted_one_by_one=true writes=1 in_place_overwrites=0 counted_by_the_plan=1 counted_by_the_crates=1 counted_by_the_formula=1 three_agree=true
E7RESULT name=anchor_segment stream=first segment=7 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true
E7RESULT name=anchor_segment stream=first segment=8 in_domain_counted_one_by_one=true writes=24 in_place_overwrites=0 counted_by_the_plan=16777215 counted_by_the_crates=16777215 counted_by_the_formula=16777215 three_agree=true
E7RESULT name=anchor_segment stream=first segment=9 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=0 counted_by_the_plan=3 counted_by_the_crates=3 counted_by_the_formula=3 three_agree=true
E7RESULT name=anchor_segment stream=first segment=10 in_domain_counted_one_by_one=true writes=1 in_place_overwrites=0 counted_by_the_plan=1 counted_by_the_crates=1 counted_by_the_formula=1 three_agree=true
E7RESULT name=anchor_segment stream=first segment=11 in_domain_counted_one_by_one=true writes=2 in_place_overwrites=2 counted_by_the_plan=8 counted_by_the_crates=8 counted_by_the_formula=8 three_agree=true
```

The row-1 part of judging output `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-rows-2026-10-26.out` (the fourth number is the independent anchor of registration section seven A2, A3):

```
== 行 1：Q1a、Q1b、Q1c
Q1a first writes 43 segments 12 roots 3 segment_lengths_match true
Q1a second writes 485 segments 80 roots 19 segment_lengths_match true
first 段数 12 四个数不全相等的段数 0 逐个数 true 的段数 12 F1 段数 0 缺行段数 0
second 段数 17 四个数不全相等的段数 0 逐个数 true 的段数 17 F1 段数 0 缺行段数 0
second 的 anchor_segment 行数 80 three_agree 非 true 的段 []
```

Q1a: the segment shape is 43, 12, 3, true, and S2 does not trigger; Q1b: 0 unequal segments, 0 segments of “apparatus = crates ≠ formula” (F1); Q1c: all 12 segments counted one by one, including segment 8 with 24 writes and 16,777,215 states; Q1d: inline test `first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor` is ok (segment 8 is not counted one by one in that test). The segments added by the number acquisition (registration A1b): new-pool file creation stream segments 0, 1 and fixed-script overwrite stream segments 16, 17 each have 2 writes, 2 in-place overwrites and 8 states (artifact lines 6, 7, 35, 36), and the write kind printed by the crates enumeration is system configuration slot for all of them (feasibility-tier raw standard output, `grep '^CRASH_FLOW_COVERAGE' <raw standard output> | awk '{print $2, $3, $4, $6}' | sort | uniq -c | grep -E 'segment=(0|1|16|17) '`):

```
      3 entry=e161_crates_comparison step=SystemConfigurationSlot segment=0 domain=quick
      1 entry=e161_crates_comparison step=SystemConfigurationSlot segment=16 domain=quick
      1 entry=e161_crates_comparison step=SystemConfigurationSlot segment=17 domain=quick
      3 entry=e161_crates_comparison step=SystemConfigurationSlot segment=1 domain=quick
```

Geometry sensitivity (the “stream” cell of registration 8.2, supplementary): the 17 sampled segments of the fixed-script overwrite stream also have all four numbers equal, no flip, two points recorded; the “number of acquisition rounds” cell (one round gives 11–26, two rounds give 12–28) is two points computed at registration time, and the same rule does not flip.

#### Third rerun row 2: after the sampled segments move to 12–28, is the domain fixed, does it finish, and does every criterion cell have an object? Sufficient to decide

Artifact lines 18 and 99 (domain sizes and anchors, full rows):

```
E7RESULT name=anchor_stream stream=first sampled_segment_states=[8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8] domain_states=16777268 by_publish=[20, 12, 16777227, 8] d1_small=53 d1_stride=7260 d1_quick=54 d1_heads=4096 recorded_writes=43 in_place_overwrites=10 replays=0 w=53 segments_match=true domain_states_match=true by_publish_match=true d1_small_match=true d1_stride_match=true d1_quick_match=true d1_heads_match=true in_place_overwrites_match=true w_formula_holds=true
E7RESULT name=anchor_stream stream=second sampled_segment_states=[16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1] domain_states=17170484 by_publish=[0, 0, 0, 16777219, 262171, 65547, 65547, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] d1_small=393269 d1_stride=7260 d1_quick=60 d1_heads=4096 recorded_writes=485 in_place_overwrites=50 replays=0 w=535 segments_match=true domain_states_match=true by_publish_match=true d1_small_match=true d1_stride_match=true d1_quick_match=true d1_heads_match=true in_place_overwrites_match=true w_formula_holds=true whole_stream_states=1678377220 whole_stream_largest_segment_states=268435455 whole_stream_states_match=true whole_stream_largest_segment_match=true
```

Summary rows and the state counts of the five cells from the row-2 part of the judging output:

```
Q2a 对不上的格数 0
1 first_small states ['53'] 期望 53 有对象
1 first_quick states ['54'] 期望 54 有对象
1 first_segment_head states ['4096'] 期望 4096 有对象
1 second_quick states ['60'] 期望 60 有对象
1 second_segment_head states ['4096'] 期望 4096 有对象
5 timing 行数 6 states ['1000'] pc3_segment_holds ['true'] 判决行有 second_segment_18_pc3 True
没有对象的格数 0 []
```

Q2a: all eight cells (`d1_small`, `d1_stride`, `d1_quick`, `d1_heads` of each stream) equal registration A4 and every `*_match` is true; in the inline tests, the A4 test `stride_and_quick_domains_have_the_registered_sizes` and `second_stream_shape_and_sampled_domains_have_the_registered_sizes` are both ok. Q2b, it finishes: the wrapper exit codes of the feasibility tier and of the inline tests are both 0 (not 250 or 253); peaks are in the table. Q2c: every one of the nine items has an object (last line `没有对象的格数 0 []`); K4-full has ≥ 2 distinct outcomes only in three cells (new-pool file creation small domain 8, quick tier 8, fixed-script overwrite quick tier 5), and all six arms are 1 in both segment-head cells. The “threshold of interpretation A” cell of geometry sensitivity (19 and 24 give 53 and 393,269; 25 and 26 give the whole domain) is registration-time arithmetic; the domain guard is enforced by mutation M13, at submission time.

#### Third rerun row 3: after the sampled segments moved, do the verdicts and G-row cells of the second-rerun artifact still point the same way? Same direction (0 cells flipped), sufficient to decide

The comparison is the same as in the second rerun (registration section six, row 3), against the second-rerun artifact: it is staged as deleted in the workspace, so per V9 it is taken from git (`git show HEAD:research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`), sha256 `265d97ff420579077920ba38fc0624b21e62fc9f169a659e451cf02faf83d8cb`, 186 lines, equal to its own final `emitted=186`, so V9 does not trigger; the new artifact has 189 lines, equal to `emitted=189`, so V10 does not trigger. Renaming table `second_segment_16_pc3` ↔ `second_segment_18_pc3`. First the comparison script is shown to fail as intended (PCr3, altering two places of the second-rerun artifact: verdict-row `clause_S2_holds` and `inconsistent` in one `reuse_arm` row); conclusion rows of `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-pcr3-2026-10-26.out`:

```
== PCr3 结果：① 翻面 0 格（要求 0）；② 翻面 2 格（要求 2）
== PCr3 通过
```

The actual comparison `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-compare-2026-10-26.out` (second-rerun artifact against the new artifact), in full:

```
键数 比对对象/这一次: 85 85
比了的字段数（按类）: {'布尔': 163, '钉零': 88, '读不到': 5, '要正': 70, '结局种数': 30, 'G行判定': 5} 合计 361
翻面格数: 0 其中判决行（Q3a）: 0 其余（Q3b）: 0
只在一边的键数: 0
只在一边的字段数: 0
```

Q3a: 0 verdict-row flips; Q3b: 0 flips in the other cells; Q3c: 0 keys and fields present on one side only. The cell-by-cell reading after the judging stage moved to the GPU follows registration 12.1.3: the S3 observation is still in the recording stage on the CPU, and `clause_S3_holds` is true in all three `stop_clause_crates` cells and in the verdict row; the crates counts that S4 compares are now produced by the GPU judge, `clause_S4_holds` is true in all three cells and in the verdict row, and the S4 stop clause does not trigger; S4 on the segment heads (`stop_clause_crates_per_state`) still goes through the crates single-state entry, and both cells are true. Artifact lines 100, 103, 106, 148, 151:

```
E7RESULT name=stop_clause_crates cell=first_small domain_states=53 crates_observed_states=53 compared_states=53 writes_table_differs=false persisted_mismatches=0 clause_S3_holds=true clause_S4_holds=true
E7RESULT name=stop_clause_crates cell=first_quick domain_states=54 crates_observed_states=54 compared_states=54 writes_table_differs=false persisted_mismatches=0 clause_S3_holds=true clause_S4_holds=true
E7RESULT name=stop_clause_crates_per_state cell=first_segment_head states=4096 clause_S4_holds=true
E7RESULT name=stop_clause_crates cell=second_quick domain_states=60 crates_observed_states=61 compared_states=60 writes_table_differs=false persisted_mismatches=0 clause_S3_holds=true clause_S4_holds=true
E7RESULT name=stop_clause_crates_per_state cell=second_segment_head states=4096 clause_S4_holds=true
```

The timing cells (the six PC3 rows, artifact lines 182–187) are compared as booleans per 12.1.3 with thresholds unchanged; all four booleans in all six rows are true, the same direction as the second rerun:

```
E7RESULT name=positive_control_timing busy_zone=recovery_consult states=1000 recovery_consult_delta_elapsed_ns=1980103594 recovery_ignore_delta_elapsed_ns=-8382645 oracle_delta_elapsed_ns=-1253735 pool_checker_delta_elapsed_ns=-221367603 record_checker_delta_elapsed_ns=-4823272 unit_check_replay_delta_elapsed_ns=-53491491 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
E7RESULT name=positive_control_timing busy_zone=recovery_ignore states=1000 recovery_consult_delta_elapsed_ns=-21528113 recovery_ignore_delta_elapsed_ns=2000065252 oracle_delta_elapsed_ns=-1437296 pool_checker_delta_elapsed_ns=-271226199 record_checker_delta_elapsed_ns=-5221362 unit_check_replay_delta_elapsed_ns=-64790073 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
E7RESULT name=positive_control_timing busy_zone=oracle states=1000 recovery_consult_delta_elapsed_ns=-17296161 recovery_ignore_delta_elapsed_ns=-6956977 oracle_delta_elapsed_ns=1999244174 pool_checker_delta_elapsed_ns=-198254945 record_checker_delta_elapsed_ns=-4099750 unit_check_replay_delta_elapsed_ns=-51198403 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
E7RESULT name=positive_control_timing busy_zone=pool_checker states=1000 recovery_consult_delta_elapsed_ns=-14218155 recovery_ignore_delta_elapsed_ns=-6597880 oracle_delta_elapsed_ns=-698147 pool_checker_delta_elapsed_ns=1829881171 record_checker_delta_elapsed_ns=-3410923 unit_check_replay_delta_elapsed_ns=-48542364 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
E7RESULT name=positive_control_timing busy_zone=record_checker states=1000 recovery_consult_delta_elapsed_ns=-14649675 recovery_ignore_delta_elapsed_ns=-8035039 oracle_delta_elapsed_ns=-755193 pool_checker_delta_elapsed_ns=-259918876 record_checker_delta_elapsed_ns=1998385277 unit_check_replay_delta_elapsed_ns=-56074330 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
E7RESULT name=positive_control_timing busy_zone=unit_check_replay states=1000 recovery_consult_delta_elapsed_ns=-19702880 recovery_ignore_delta_elapsed_ns=-7250629 oracle_delta_elapsed_ns=-1206238 pool_checker_delta_elapsed_ns=-204092726 record_checker_delta_elapsed_ns=-4146332 unit_check_replay_delta_elapsed_ns=1955902394 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_segment_holds=true pc3_zone_holds=true
```

Timing environment (the sources 12.1.3 asks to copy): final row `loadavg1=60.16 cpus=32`, ratio 1.88; quiesce lines and exit code from the feasibility-tier err file, and the judging-stage and verdict-library lines from the raw standard output, verbatim:

```
GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=1
GPU_QUIESCE step=exit code=0
exit=0
CRASH_POINT_REPLAY_JUDGE route=gpu cards_used=1 blocks=12 states=53 red_states=0 compile_seconds=1.7 judge_seconds=0.2 states_per_second=305
CRASH_CASE_BLOCKS blocks=12 reused=0 fresh=12 red=0 refused=0
CRASH_POINT_REPLAY_JUDGE route=gpu cards_used=1 blocks=12 states=53 red_states=0 compile_seconds=1.7 judge_seconds=0.2 states_per_second=325
CRASH_CASE_BLOCKS blocks=13 reused=1 fresh=12 red=0 refused=0
CRASH_POINT_REPLAY_JUDGE route=gpu cards_used=1 blocks=18 states=61 red_states=0 compile_seconds=1.7 judge_seconds=7.4 states_per_second=8
CRASH_CASE_BLOCKS blocks=18 reused=0 fresh=18 red=0 refused=0
```

`route=gpu` in three places: the judging stage judged afresh on the GPU (12, 12 and 18 blocks fresh, 0, 1 and 0 blocks reused). The registration has no clause that voids timing cells because of load: of the positive controls V1 covers, everything PC3 must show was shown; V3 (`state_shorter_than_its_segments_share_ratio` is 0.000000 in every cell, and the largest `replaying_longer_than_the_checker_share_ratio` is 0.008300 in `first_segment_head`, under 1%) does not trigger; so the PC3 cells of this run count as registered. The timing load gate of the replay table (“timing row load gate” in the header of `scripts/experiment-apparatus/replay.sh`, which records a replay row as not judged this time when the ratio exceeds 0.5) governs replays, not this registration's clauses; whether this artifact's timing cells should be judged differently because of load is handed to the main agent and was not judged. In the `busy_zone=pool_checker` row the busy zone added 1,829,881,171 ns, only about 30 ms above the 1,800,000,000 threshold (second rerun: 2,076,969,781); the cause was not investigated. With no flipped cells, registration F3 does not trigger, and Q3d and C3b (one thread) were not run after the rows became sufficient to decide.

Verdict row (line 188, the only verdict row listed by `grep -n 'name=verdict'`):

```
E7RESULT name=verdict scope=feasibility clause_S1_holds=true clause_S2_holds=true clause_S3_holds=true clause_S4_holds=true clause_S5_holds=true clause_S6_holds=true anchors_a1_to_a7_hold=true first_small_k2_consult_constant_key_separates=true first_small_k2_ignore_constant_key_separates=true first_small_k4_full_constant_key_separates=true first_small_pc1=true first_segment_head_k2_consult_constant_key_separates=false first_segment_head_k2_ignore_constant_key_separates=false first_segment_head_k4_full_constant_key_separates=false first_segment_head_pc5=true first_quick_k2_consult_constant_key_separates=true first_quick_k2_ignore_constant_key_separates=true first_quick_k4_full_constant_key_separates=true first_k2_consult_pc2c_pc4c_found_a_position=true first_k2_ignore_pc2c_pc4c_found_a_position=true first_k4_walk_pc2c_pc4c_found_a_position=true first_k4_units_pc2c_pc4c_found_a_position=true first_k4_full_pc2c_pc4c_found_a_position=true second_segment_head_k2_consult_constant_key_separates=false second_segment_head_k2_ignore_constant_key_separates=false second_segment_head_k4_full_constant_key_separates=false second_segment_head_pc5=true second_quick_k2_consult_constant_key_separates=true second_quick_k2_ignore_constant_key_separates=true second_quick_k4_full_constant_key_separates=true second_k2_consult_pc2c_pc4c_found_a_position=false second_k2_ignore_pc2c_pc4c_found_a_position=false second_k4_walk_pc2c_pc4c_found_a_position=false second_k4_units_pc2c_pc4c_found_a_position=false second_k4_full_pc2c_pc4c_found_a_position=false second_segment_18_pc3=true g1=not_judged_feasibility_only g2=not_judged_feasibility_only g3=not_judged_feasibility_only g4=not_judged_feasibility_only g5=not_judged_feasibility_only
```

The verdict-row fields valued `false` or `not_judged_feasibility_only` all have the same values as in the second rerun and are within the registration's expectation:

- `first_segment_head_k2_consult_constant_key_separates=false`, `first_segment_head_k2_ignore_constant_key_separates=false`, `first_segment_head_k4_full_constant_key_separates=false`, `second_segment_head_k2_consult_constant_key_separates=false`, `second_segment_head_k2_ignore_constant_key_separates=false`, `second_segment_head_k4_full_constant_key_separates=false`: in both segment-head cells all six arms have only one outcome (`distinct_outcomes=1` each), so a constant key cannot separate different outcomes either, and this control cannot be tested in the two segment-head cells.
- `second_k2_consult_pc2c_pc4c_found_a_position=false`, `second_k2_ignore_pc2c_pc4c_found_a_position=false`, `second_k4_walk_pc2c_pc4c_found_a_position=false`, `second_k4_units_pc2c_pc4c_found_a_position=false`, `second_k4_full_pc2c_pc4c_found_a_position=false`: the control sample of the fixed-script overwrite stream has only the 4096 segment-head states, and `discriminating_positions` is 0 for all five arms (artifact lines 177–181).
- The five fields `g1`–`g5` are `not_judged_feasibility_only`: the feasibility tier does not judge G rows.

The verdict row has no integer fields and no `not_run`. Outside the verdict row, reuse-arm `inconsistent` is 7 on K4-walk and K4-units of `first_small` and `first_quick`, and 6 on K4-units of `second_quick`, the same as the second rerun and in the same “≥ 1” class (readings of apparatus usability only); other violation-class counts have no non-zero rows: `grep -c ' [a-z_0-9]*\(differs\|collisions\|mismatches\|changed\|_differ\)=[1-9]' research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out` outputs 0, and the same command with `[1-9]` replaced by `0` outputs 17 (rows).

#### Third rerun row 4: inline tests and mutations. The inline half is sufficient to decide; the mutation half runs at submission

Per registration 12.2.2, “all pass” for the inline tests requires all five items, checked now in log `~/.local/share/singlefs/work/m3-close-b16-e161-r3-run/e161-r3-c2-gpu-inline-tests.out` (inline-test scratch log, original not retained in the repository; the 9 checked lines are copied verbatim): ① `exit=0` with `GPU_QUIESCE step=command result=exit-0`; ② `test result: ok.`, 15 passed equal to the current `grep -c` count 15, failed 0, ignored 0; ③ `grep -cE 'NOT_RUN_[A-Z_]+ test='` outputs 0; ④ `grep -c 'judge_stage=not_run'` outputs 0; ⑤ exactly one judged row. Q4a is recorded as all pass. The corresponding log lines, verbatim:

```
GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1
running 15 tests
CRASH_POINT_REPLAY_JUDGE route=gpu cards_used=0 blocks=0 states=0 red_states=0 compile_seconds=0.0 judge_seconds=0.0 states_per_second=0
CRASH_CASE_BLOCKS blocks=12 reused=12 fresh=0 red=0 refused=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.25s
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=0
GPU_QUIESCE step=exit code=0
exit=0
```

The row of ⑤ is log line 272: `E7RESULT name=stop_clause_crates cell=test-first-small domain_states=53 crates_observed_states=53 compared_states=53 writes_table_differs=false persisted_mismatches=0 clause_S3_holds=true clause_S4_holds=true`.

In this run the crates judging stage came entirely from the verdict library (`reused=12 fresh=0`; the judge judged 0 states afresh), a case registration 12.2.2 accepts. Mutations: the 16 rows of `crates/mutations.tsv` starting with `E161 crash_state_dedup` (M14 renamed to `PC3 的段改回 16`, M15 to `覆盖写固定脚本流第一个取样段 12 改成 11`) each have an original that matches exactly once in the apparatus (Q4c, counted row by row by name, `count=1`); the caught, invalid and not-failing counts (Q4b) belong to the submission gate checker-tier-crates-mutation-replay and were not run.

### Rerun

Entry point is `driver_e161_feasibility` in `scripts/experiment-apparatus/replay.sh` (sets thread environment variable to 31, memory cap follows `REPLAY_MEMORY_CAP`, default 8G, adds `--features verdict-store,gpu` and starts through the quiesce wrapper when the multi-host configuration enables GPU, retains only `E7RESULT` rows; timing fields end `_elapsed_ns` or `_ratio`, structural comparison in `timing` mode; a replay whose final-row load ratio exceeds 0.5 is recorded as not judged this time). Registered row with driver `@driver_e161_feasibility` now points to third rerun artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out`; second rerun artifact `e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` serves only as the third rerun's row-3 control and has been moved out of the workspace (take it with `git show 841b3beab:<path>`); first `e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` serves only as the second rerun's row-3 control and is archived (take it with `git show 5d0b7e7c1667:<path>`). Replay and direct run (8G sufficed in the third rerun, peak 6,884,950,016 bytes):

```
bash scripts/experiment-apparatus/replay.sh E161
unset GPU_CARDS_BY_PRIORITY; E161_THREADS=31 bash scripts/hosts/capped.sh 31 bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --features verdict-store,gpu --bin e161_crash_state_dedup_and_time_split -- feasibility | grep '^E7RESULT '
```

Both commands are checker tier, run by the main agent only on submission with `SINGLEFS_HEAVY_TESTS=commit` (or `=user-request` at user request).

Count arm inconsistencies and hits: `grep 'name=reuse_arm' research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out | awk '{print $3, $4, $6, $7}'`. Third rerun row-3 comparison: first `git show 841b3beab:research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out > <scratch directory>/r2.out`, then `python3 research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-judge-2026-10-26.py compare <scratch directory>/r2.out research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out`; rows 1 and 2 use the `row-one` and `row-two` subcommands of the same script, PCr3 uses `pcr3`. Second rerun row-3 comparison: `python3 e161-crash-state-dedup-and-time-split-feasibility-r2-judge-2026-10-05.py compare e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`.

### Path and conclusion registration

| # | Path | Calculation | Source placement | Shared inputs |
|---|---|---|---|---|
| 1 | **Apparatus enumeration plan** | Apparatus's torn-image table and mixed-radix plan generate and count persisted sets per segment (individually in sampled segments) | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1175` `fn persisted_of`, `:2855` `fn count_distinct_persisted_sets` | Recorded stream write table and segments (`writes_and_segments` output), base image, D13 (Verification Route) settled item 4's three in-place-overwrite criteria |
| 2 | **crates enumeration plan** | crates-private `CrashStatePlan` and `crash_state_count_with_torn_in_place_overwrites` enumerate by segment and pass to observer | `crates/singlefs-checker-tier/src/crash.rs:2425` `fn persisted_writes_of_state`, `crates/singlefs-checker-tier/src/crash.rs:2279` `fn crash_state_count_with_torn_in_place_overwrites` | Same recorded write table and segments, base image and D13 settled item 4 |
| 3 | **Section-four closed form** | 3^m · 2^(n−m) − 1 per segment | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1222` `fn formula_segment_state_count` | Segment length and in-place overwrite count (from path 1 judgment) |
| 4 | **Apparatus replay counts** | Apparatus calls public functions in `evaluate_state_recording_findings` order and records its own `CrashPointReplayTally` | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2014` `fn evaluate_baseline` | crates `recover`, `check_pool_image`, `check_records`, `oracle_violation_for_versions` |
| 5 | **crates replay counts** | crates per-segment enumeration or single-state entry `evaluate_state_for_versions` | `crates/singlefs-checker-tier/src/crash.rs:1634` `fn evaluate_state_recording_findings` | Same four public functions as path 4 |

Cross-path assertions: `segment_count_checks` (`:2907`) compares paths 1, 2, 3 per segment, artifact `name=anchor_segment` reports `three_agree`; `compare_with_the_crates` (`:2983`) compares persisted sets and write tables from paths 1 and 2 per state; `tally_text` (`:2514`) builds and compares path 4/5 counts verbatim. Paths 4 and 5 call the same public functions, so equality proves apparatus ordering and count definitions were transcribed correctly, not that recovery or checker itself is correct. Paths 1 and 2 share the write table and in-place-overwrite criterion; if the criterion is wrong both err together.

### What it cannot answer

- **All G1–G5 remain unjudged**: feasibility only; ratios, inconsistencies, shares and byte counts establish only that the apparatus can produce numbers. Registration 5.5 segment one (all 1.44 million states in fixed script overwrite segments 10–22, stride 2311 samples from both 26-write segments), segments two and three were not run. Section-eight geometry sensitivity (arm 甲二 quick tier versus full run, one thread, mask width), weighted G3 estimate and G5 extrapolation were outside this run.
- **K4-walk and K4-units have inconsistencies in small domain** (three `name=reuse_arm` rows above). This class is expected by registration (4.12 says judgment also reads journal and scan direction, so same-key/same-judgment does not follow), but feasibility does not decide G4 and Q4d (diagnosis by invariant) was not run.
- **G3 upper bound Q3g is overcounted in this apparatus**: it computes header-scan CRC bitwise and U on every reread, making replay total exceed per-state total; V3 triggers in `first_small` and `first_quick`. Before judging G3, check whether replay measurement basis does more work than checker itself.
- **PC2b / PC4b cannot be tested in the two segment-head cells**: their 4096 states have only one outcome.
- **16 mutations not run**: E161's 16 rows in `crates/mutations.tsv` (M1–M12 sample rewrites, M13–M16 added for second rerun) belong to submission gate checker-tier-crates-mutation-replay. M13 must be “caught,” not “timeout” or “memory cap,” for the domain guard to work (PCr2a); this has not yet been observed.
- **Seven reversals in second-rerun row 3 have no attribution**: they occur in cells whose content completely changed (new-pool small + head position-removal control and both K4-units head cells), so changed sampling domain, changed crates since first artifact (`git log` 7 entries, `git diff HEAD --stat -- crates` 109 files), or both cannot be distinguished. Thread count has since been ruled out: C3b was run by main agent on 2026-10-05 (`E161_THREADS=1`, exit 0, `e161-crash-state-dedup-and-time-split-feasibility-r2-threads1-2026-10-05.out`, 186 lines, final `name=done emitted=186`). After removing timing fields ending `_elapsed_ns` and `_ratio`, line-by-line comparison with the ten-thread run differs only in line 1 `name=config`, `threads=10` / `threads=1`; remaining 185 lines match.
- **Second rerun still does not judge g1–g5**: verdict fields remain `not_judged_feasibility_only`; segment-head cells and overwrite control still have one outcome, so PC2b, PC4b and position-removal controls cannot be tested there (11 `false` verdict fields).
- **The third rerun's timing cells ran under high load**: the feasibility tier's final row is `loadavg1=60.16 cpus=32` (ratio 1.88), at which the replay table's timing load gate would record the row as not judged this time. The registration has no clause voiding them because of load, so the six PC3 rows count as registered and all keep the second rerun's direction, but the `busy_zone=pool_checker` row is only about 30 ms above the 1.8-second threshold. Whether PC3 still passes at low load, and whether this artifact's timing cells should be judged differently, was not measured and is handed to the main agent.
- **In the third rerun's inline tests the crates judging stage did not judge afresh**: the crates counts compared by `device_tally_equals_the_crates_tally_on_the_first_stream_small_domain` came entirely from the verdict library (`reused=12 fresh=0`); the GPU judge judged afresh in the feasibility-tier run (12, 12 and 18 blocks in the three cells).
- **The third rerun still does not judge g1–g5**, and the 16 mutations (M14 and M15 originals changed with the constants) still belong to the submission gate; the one-thread supplementary run (C3b) and the known-change notes for flipped cells (Q3d) were not run after the rows became sufficient to decide, because row 3 has no flipped cells.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 4 | Background | 2026-10-26 unaffected: the third rerun changed the artifact; the four numbers of the segment model are still equal segment by segment on the 12 segments of the new-pool file creation stream and the 17 sampled segments of the fixed-script overwrite stream, and the state-by-state differential against crates S3 and S4 all passes, so the background material is unchanged |
| D13 (Verification Route) settled item 7 | Background | 2026-10-26 unaffected: in the third rerun the two recorded recoveries and checker verdicts equal the baseline run item by item (S5 and S6 true), so the background material is unchanged |
| D13 (Verification Route) settled item 16 | Supports | 2026-10-26 unaffected: in the third rerun the full per-segment state count, the closed form, the apparatus's one-by-one count and the crates plan are still equal segment by segment, and the state count still grows as a power of the segment's write count (16,777,215 for a 24-write segment), so the supported premise is unchanged; the item's basis still points to the second-rerun section |
| D24 (Offloading Heavy Background Work to GPU) | No impact | 2026-10-26 unaffected: the third rerun's crates judging stage ran on the GPU, but G rows are not judged and no cost or benefit of offloading to the GPU was measured |

## Historical Versions

### 2026-09-27: feasibility tier

- Page created. Under user decision 2026-09-27 only feasibility was run (registration 12.2); originally planned stop-clauses and segment-one long runs have no checked-in artifacts.

### 2026-10-05: second rerun

- Cause: barrier added before publish returns split new-pool file creation into 11 segments, making the first registration's literal domain 16,777,260 states; apparatus inline tests failed or hit the memory cap. Register `research/prompts/e161-r2-prereg.md`, domain interpretation A (threshold 26 → 24). Apparatus updated to 14 inline tests and 16 mutation rows; feasibility artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` has 186 lines.
- Conclusions: row 1 three-way per-segment equality, row 2 fixed runnable domain, row 3 seven reversed cells (one verdict `first_k4_full_pc2c_pc4c_found_a_position`, six in reuse arms and position-removal control), row 4 inline tests all pass and mutations assigned to submission gate. First artifact retained unchanged as control; `replay.sh` registration points to new artifact.
- Not done: supplemental one-thread run, 16 mutations, G1–G5. See “Second rerun” and “What it cannot answer.”

### 2026-10-26: third rerun

- Cause: the two-round number acquisition write made the new-pool file creation stream 12 segments and the fixed-script overwrite stream 80 segments, so the second rerun's pinned values no longer matched. Registered `research/prompts/e161-r3-prereg.md`; the sampled segments moved with the same intent to 12–28 and the PC3 segment moved to 18; the apparatus has 15 inline tests and 16 mutation rows (rows M14 and M15 changed); feasibility artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out` has 189 lines, and its judging stage was built with the `gpu` feature and ran on the GPU after a quiesce; the judging script and its three outputs are the four files under `research/results/` that start with `e161-crash-state-dedup-and-time-split-feasibility-r3-` and end with `-2026-10-26`.
- Conclusions: row 1 has all four numbers equal per segment, row 2 has a fixed domain that finishes with an object in every cell, row 3 has 0 cells flipped relative to the second rerun, row 4 has all inline tests passing and mutations assigned to the submission gate. The second rerun artifact has been moved out of the workspace and was taken from git as the row-3 control; the `replay.sh` registration row now points to the new artifact with 31 threads. The first feasibility-tier trial run (2026-10-25) did not compile in the `gpu` feature, did not run the judging stage and exited 77; it is not an artifact.
- Not done: the one-thread supplementary run and Q3d (not run after the rows became sufficient to decide), 16 mutations, G1–G5; the timing cells are not judged differently and not rerun because of the final load of 60.16 (main agent decision on 2026-10-26: the registration has no clause voiding cells by load, and changing the criteria afterwards for the load would be a new run; the load and the margin of the pool_checker cell are written under "What it cannot answer").
