## E139 Tightened Form of Per-Disk Rollback Floor — Run (2026-09-12); Rerun 1 Run (2026-10-04, Reclamation Threshold Changed to One-Ring Window); Rerun 2 Run (2026-10-05, Candidate Floor Adding System Configuration F, Reclamation Threshold and Other Five Locations Returning to Whole Ring)

[中文](139-按盘回退下界的收严形态.md) · **English** · [日本語](139-ディスク別ロールバック下限の厳格形態.ja.md)

<!-- doc-lint:not-numbers G2S G2R G2SW H1 H2 H3 RGall -->

Note: Counts, verdict rows, and "What It Cannot Answer" in section "Rerun 1" are figures under the old candidate floor and window threshold measurement basis; Rerun 2 (section of the same name) changed the candidate floor to clause form and added the RGall arm; new figures for the same batch of cells are governed by section Rerun 2, and the two rewritten sentences in the old section (direction of the rollback candidate set row, item 3 of What It Cannot Answer) have been updated according to the new artifacts.

**What is being asked**: Direction settled by user after D16 (Publish Semantics) settled item 1 Round 3 three-way argument — tightened form of per-disk rollback floor
(acceptance charges by worst-case residue, bounds and reservation pools reserve margin for k_tol = 2 root slot write failures, reservation pools and residue calculated by declared worst-case empty publish overhead c_max, minimum retention counted by 4 distinct states) —
across the three worlds struck in Round 3 (root slot write failure during push-empty, one root slot permanently unwritable thereafter, empty publish overhead expanding after mount) and existing fault worlds of E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty), whether it meets the two hard requirements of D3 (Space Allocation) settled item 9, and what the respective bounds and costs are.
"4" establishes one arm under each of two readings: reading by root count (G2R, reading of E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)) and reading by user-clarified state count (G2S).

**Hardcoded before running**: `e139-preregistration.md`. Three changes made while coding apparatus (arithmetic of G2S lag sentence, precise definition of lag, active metadata recorded back into reservation pool), all prior to first run;
one change after first run (when non-empty valid roots < 4, upper bound under state-count reading takes oldest valid root rather than oldest non-empty root); artifact of first run preserved as `e139-tightened-floor-2026-09-12-run1.out`,
the two artifacts differing by 8 rows, all in S = 1 (five fault world cells, two `near_full` lag cells, one summary row), see "Historical Versions" at end. The author of the registration held two leanings (G2R passes all; in near-full disk deletion-creation cycles for G2S, blocks deleted this round require another 3 state-changing windows before being returned); directions were both right, but both require qualifications: G2R's "passes all" depends on accounting measurement basis of active metadata recorded back into reservation pool, and before mount-time reservation of instance switching (C126 (Worst-case amount of switching reservation has no measurement basis)) has figures, whether Item 1 is fulfilled cannot be judged; failure windows of G2S reach up to window 6 in reservation sweep, and lag has no upper bound (D16 (Publish Semantics) settled item 1 Round 4 three-way argument, `research/prompts/d16-item1-r4-main-verification.md`; what changed see "Postscript After Round 4 Three-way Argument").

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):
```bash
bash scripts/experiment-apparatus/replay.sh E139
# Run apparatus directly:
cd scripts/experiment-apparatus && cargo run --release --bin e139_tightened_floor
# Diagnostic tests added post-run (first and last failure windows across 18 cells of reservation sweep):
cd scripts/experiment-apparatus && cargo test --release --bin e139_tightened_floor states_arm_reserve_sweep_failure_windows_end_by_window_six -- --nocapture | grep ^RSWEEP_WINDOWS
```
Code: `scripts/experiment-apparatus/e139_tightened_floor.rs` (adapted from source of E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)),
raw output: `e139-tightened-floor-2026-09-12.out` (second run, 901 lines, final line `emitted=901`; native release ~67 seconds).
`replay.sh` registration row points to Rerun 2 artifact `e139-tightened-floor-2026-10-05.out` (6271 lines, exit code 0): deleting rows starting with `name=f_`, `name=verdict_f_` and 2230 lines following `name=done` matches Rerun 1 artifact `e139-tightened-floor-2026-10-04.out` (2231 lines, native release ~190 seconds, preserved without deletion) line-by-line; Rerun 1 artifact deleting new arm rows (starting with `name=w_` and `name=verdict_window`) and 900 lines following `name=done` matches that report verbatim; old arm rows transcribed verbatim in "Verdict" and "Cost" sections can all be found in all three artifacts.

### Rerun 1 (Reclamation Threshold Changed to One-Ring Window)

Rerun registration `e139-r1-prereg.md` (written prior to apparatus changes; after apparatus and unit tests were written, before artifact was run once, "Revisions" R1–R4 were added; Section 6 criteria and Section 10 failure clauses untouched by a single word).
Run on 2026-10-04, artifact `e139-tightened-floor-2026-10-04.out` (2231 lines, final line `emitted=2231`, native release ~190 seconds); apparatus `scripts/experiment-apparatus/e139_tightened_floor.rs` modified in-place,
adding seven `WindowArm` arms (G2SW, reading arms G2SW_Q and G2SW_L, old threshold control G2SW_ringgate, three positive controls shortening one row of window `_short`), leaving behaviors of original eight arms untouched.
Issue ticket `m3-r1-batch9-reruns-questions.md` row 1 asks two halves:

1. After reclamation threshold changed to one-ring window (D16 (Publish Semantics) settled item 1 "oldest valid root in one-ring window"), whether Arm G2S plus window (hereinafter G2SW) still maintains zero fake candidates, zero unrecoverable states in those original fault worlds, and whether false ENOSPC across cells remains 0;
2. Whether the sentence in settled item 1 settlement "form adopts Arm G2S of E139 (Tightened Form of Per-Disk Rollback Floor)" matches apparatus clause by clause (owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus)).

**Readings** (read line by line per Section 6 of registration, each row reporting independently; no judgment on whether these figures overturn or establish D16 (Publish Semantics) settled item 1, which is deduction requiring tripartite argument):

| Quantity | Reading | Pre-run clause |
|---|---|---|
| First half: G2SW fake candidates | Across 60 rows of `w_activation`, 46 rows > 0 (`activation_faults` 8 rows, `activation_crash_no_loss` 8 rows, `skip_then_raise_r1` / `r2` / `r3` 10 rows each; lost disk 0 has 26 rows, lost disk 1 has 20 rows); rows of `w_cell` (except 3 rows of `rollback_first_publish`), `w_near_full`, `w_drain_failure`, `w_broken_slot`, `w_cost_growth` all 0; 3 rows of `rollback_first_publish` reported only without judgment per original registration (fake candidates 37 / 173 / 814, unrecoverable 24, identical across four arms) | Failure clause F1 triggered: recorded under weakest reading as "G2SW loses in these cells" |
| First half: G2SW unrecoverable | Across `w_activation`, 3 rows > 0: `skip_then_raise_r2` S = 4 lost disk 0 failed 2 times (16), `skip_then_raise_r3` S = 4 lost disk 0 failed 1 time (16) and 2 times (64); remaining 108 judged rows are 0 | F1 |
| First half: false ENOSPC in judged cells | Judged cells have 0 rows > 0; unjudged `cost_growth` overhead rising to c_max + 3 has two rows (S = 4, 16, c₀ = 1) at 32 times each, identical to original G2S | First half does not flip on this item |
| Old threshold control G2SW_ringgate | Across 108 judged rows: fake candidates, unrecoverable, false ENOSPC in judged cells all 0 (3 rows of `rollback_first_publish` identical to G2SW, reported only without judgment) | Root cause lies in reclamation threshold item, not in W2–W4 or their shared premises |
| Second half: clause-by-clause match | 13 rows in settled item 1 table plus "non-empty" paragraph total 14 items: 5 Class C, 5 Class B, 4 Class A; another Class C item on D23 (Journal Role & Format) side, outside that table (see "Clause-by-Clause Match" subsection) | F2 triggered: Class C >= 1 |
| Reading arms | Across G2SW_Q, G2SW_L and G2SW, line-by-line verdicts have 0 rows differing | F5 untriggered |
| Discriminative power self-test | Across 111 result rows with identical parameters between G2SW and G2SW_ringgate, 55 rows differ, landing in five classes of injected root slot write failure worlds; six classes of injected worlds all have positive threshold difference counts > 0 | F4 untriggered |
| Positive control for window item | All three `_short` arms across 6 rows in `steady` S = 4, 16 have fake candidates > 0 | F3 untriggered |
| Geometric sensitivity | H1 (broken slot swapped to disk 1) has 0 rows with verdict changes across four arms; H2 (skipped numbers swapped to disk 0) has 15 rows changed each for G2SW, G2SW_Q, G2SW_L, and 0 rows for G2SW_ringgate | F7 triggered: those 15 rows recorded as "unstable", passing no "zero" or "non-zero" conclusion |

Verdict rows transcribed verbatim from `e139-tightened-floor-2026-10-04.out` (all 37 rows from `grep -n 'name=verdict_window' e139-tightened-floor-2026-10-04.out`; original three `name=verdict` rows unchanged verbatim, not re-copied):

```text
E7RESULT name=verdict_window arm=G2SW_tightened_states_window rows=111 rows_with_fake_candidates=46 rows_with_unrecoverable=3 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_window arm=G2SW_Q_disk_hold rows=111 rows_with_fake_candidates=46 rows_with_unrecoverable=3 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_window arm=G2SW_L_ring_lag rows=111 rows_with_fake_candidates=46 rows_with_unrecoverable=3 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_window arm=G2SW_ringgate rows=111 rows_with_fake_candidates=0 rows_with_unrecoverable=0 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_window_short arm=G2SW_short rows=2 rows_where_the_detector_saw_zero_fake_candidates=0
E7RESULT name=verdict_window_short arm=G2SW_Q_short rows=2 rows_where_the_detector_saw_zero_fake_candidates=0
E7RESULT name=verdict_window_short arm=G2SW_L_short rows=2 rows_where_the_detector_saw_zero_fake_candidates=0
E7RESULT name=verdict_window_gate_reach class=steady injects_root_write_failures=0 rows=3 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=torn_newest injects_root_write_failures=0 rows=3 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=disk_loss injects_root_write_failures=0 rows=6 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=slot_write_fail injects_root_write_failures=1 rows=3 rows_with_positive_gate_difference=3 positive_samples_total=840
E7RESULT name=verdict_window_gate_reach class=admin_rollback injects_root_write_failures=0 rows=3 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=rollback_first_publish injects_root_write_failures=0 rows=3 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=activation_faults injects_root_write_failures=1 rows=12 rows_with_positive_gate_difference=8 positive_samples_total=5296
E7RESULT name=verdict_window_gate_reach class=activation_crash_no_loss injects_root_write_failures=1 rows=12 rows_with_positive_gate_difference=8 positive_samples_total=10192
E7RESULT name=verdict_window_gate_reach class=skip_then_raise injects_root_write_failures=1 rows=36 rows_with_positive_gate_difference=36 positive_samples_total=71072
E7RESULT name=verdict_window_gate_reach class=near_full injects_root_write_failures=0 rows=6 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach class=drain_failure injects_root_write_failures=1 rows=12 rows_with_positive_gate_difference=6 positive_samples_total=14592
E7RESULT name=verdict_window_gate_reach class=broken_slot injects_root_write_failures=1 rows=4 rows_with_positive_gate_difference=4 positive_samples_total=62528
E7RESULT name=verdict_window_gate_reach class=cost_growth injects_root_write_failures=0 rows=8 rows_with_positive_gate_difference=0 positive_samples_total=0
E7RESULT name=verdict_window_gate_reach_summary injection_classes=6 injection_classes_without_window_effect=0
E7RESULT name=verdict_window_self_proof_class class=activation_faults rows_differing_from_ringgate=8
E7RESULT name=verdict_window_self_proof_class class=activation_crash_no_loss rows_differing_from_ringgate=8
E7RESULT name=verdict_window_self_proof_class class=skip_then_raise rows_differing_from_ringgate=30
E7RESULT name=verdict_window_self_proof_class class=drain_failure rows_differing_from_ringgate=5
E7RESULT name=verdict_window_self_proof_class class=broken_slot rows_differing_from_ringgate=4
E7RESULT name=verdict_window_self_proof rows_compared=111 rows_differing_from_ringgate=55
E7RESULT name=verdict_window_readings reading_arm=G2SW_Q_disk_hold rows_compared=111 rows_with_a_judgement_different_from_window=0
E7RESULT name=verdict_window_readings reading_arm=G2SW_L_ring_lag rows_compared=111 rows_with_a_judgement_different_from_window=0
E7RESULT name=verdict_window_geometry point=h1 arm=G2SW_tightened_states_window rows_compared=4 rows_with_a_judgement_different_from_the_original_point=0
E7RESULT name=verdict_window_geometry point=h2 arm=G2SW_tightened_states_window rows_compared=36 rows_with_a_judgement_different_from_the_original_point=15
E7RESULT name=verdict_window_geometry point=h1 arm=G2SW_Q_disk_hold rows_compared=4 rows_with_a_judgement_different_from_the_original_point=0
E7RESULT name=verdict_window_geometry point=h2 arm=G2SW_Q_disk_hold rows_compared=36 rows_with_a_judgement_different_from_the_original_point=15
E7RESULT name=verdict_window_geometry point=h1 arm=G2SW_L_ring_lag rows_compared=4 rows_with_a_judgement_different_from_the_original_point=0
E7RESULT name=verdict_window_geometry point=h2 arm=G2SW_L_ring_lag rows_compared=36 rows_with_a_judgement_different_from_the_original_point=15
E7RESULT name=verdict_window_geometry point=h1 arm=G2SW_ringgate rows_compared=4 rows_with_a_judgement_different_from_the_original_point=0
E7RESULT name=verdict_window_geometry point=h2 arm=G2SW_ringgate rows_compared=36 rows_with_a_judgement_different_from_the_original_point=0
```

**Pre-modification reproduction and regression** (Registration Section 6 rows 1, 2): Before changing apparatus, run once with the source code of that time; output sha256 and `git show 3cff909b^:research/results/e139-tightened-floor-2026-09-12.out | sha256sum` are both `bbea1d31ab074141736b27dc3246967b7fec6e7b5168f309499ddf7274ab778f` (invalidation clause V1 untriggered). After modification, deleting new arm rows and comparing line by line against old artifact:

```bash
diff <(git show 3cff909b^:research/results/e139-tightened-floor-2026-09-12.out | grep -v 'name=done') <(grep -v -E '^E7RESULT name=(w_|verdict_window)' e139-tightened-floor-2026-10-04.out | grep -v 'name=done') | wc -l
```

Outputs `0`: old 900 lines appear as is, order unchanged (V2 untriggered). New arm rows count 1330 lines (`grep -c -E '^E7RESULT name=(w_|verdict_window)'`), and old 901 lines minus `name=done` 900 lines plus new arm 1330 lines plus new `name=done` single line = 2231.
Apparatus ran twice successively, artifacts byte-for-byte identical (sha256 `2563dde0002f65c266a5223ecc3973a78e715ab1ba0a5e02f2ecbabd8448bdd2`); this is a deterministic model, demonstrating solely the absence of hidden state; evidence strength derives from unit tests and mutations.

**Line count self-check**: counting each class of result rows by "arm × world × geometry" (command and raw output; each result row has another `name=w_trajectory` row, where `grep -c '^E7RESULT name=w_trajectory '` outputs 610 = 604 + 6):

```bash
grep -o '^E7RESULT name=w_[a-z0-9_]* arm=[A-Za-z0-9_]*' e139-tightened-floor-2026-10-04.out | sed 's/^E7RESULT //' | sort | uniq -c
```

```text
     60 name=w_activation arm=G2SW_L_ring_lag
     60 name=w_activation arm=G2SW_Q_disk_hold
     60 name=w_activation arm=G2SW_ringgate
     60 name=w_activation arm=G2SW_tightened_states_window
     36 name=w_activation_h2 arm=G2SW_L_ring_lag
     36 name=w_activation_h2 arm=G2SW_Q_disk_hold
     36 name=w_activation_h2 arm=G2SW_ringgate
     36 name=w_activation_h2 arm=G2SW_tightened_states_window
      4 name=w_broken_slot arm=G2SW_L_ring_lag
      4 name=w_broken_slot arm=G2SW_Q_disk_hold
      4 name=w_broken_slot arm=G2SW_ringgate
      4 name=w_broken_slot arm=G2SW_tightened_states_window
      4 name=w_broken_slot_h1 arm=G2SW_L_ring_lag
      4 name=w_broken_slot_h1 arm=G2SW_Q_disk_hold
      4 name=w_broken_slot_h1 arm=G2SW_ringgate
      4 name=w_broken_slot_h1 arm=G2SW_tightened_states_window
     21 name=w_cell arm=G2SW_L_ring_lag
      2 name=w_cell arm=G2SW_L_short
     21 name=w_cell arm=G2SW_Q_disk_hold
      2 name=w_cell arm=G2SW_Q_short
     21 name=w_cell arm=G2SW_ringgate
      2 name=w_cell arm=G2SW_short
     21 name=w_cell arm=G2SW_tightened_states_window
      8 name=w_cost_growth arm=G2SW_L_ring_lag
      8 name=w_cost_growth arm=G2SW_Q_disk_hold
      8 name=w_cost_growth arm=G2SW_ringgate
      8 name=w_cost_growth arm=G2SW_tightened_states_window
      6 name=w_crit1_push arm=G2SW_L_ring_lag
      6 name=w_crit1_push arm=G2SW_Q_disk_hold
      6 name=w_crit1_push arm=G2SW_ringgate
      6 name=w_crit1_push arm=G2SW_tightened_states_window
     12 name=w_drain_failure arm=G2SW_L_ring_lag
     12 name=w_drain_failure arm=G2SW_Q_disk_hold
     12 name=w_drain_failure arm=G2SW_ringgate
     12 name=w_drain_failure arm=G2SW_tightened_states_window
      6 name=w_near_full arm=G2SW_L_ring_lag
      6 name=w_near_full arm=G2SW_Q_disk_hold
      6 name=w_near_full arm=G2SW_ringgate
      6 name=w_near_full arm=G2SW_tightened_states_window
```

All four full-cell arms have consistent row counts per class; 21 rows of `w_cell` = six basic worlds × S ∈ {1, 4, 16}, `disk_loss` having one extra dropped disk; 60 rows of `w_activation` = five variants × S ∈ {4, 16} × 2 dropped disks × failure count 0 / 1 / 2; positive control `_short` enters only S = 4, 16 of `steady`.
Thirteen classes of worlds (original registration "Worlds" table) all accounted for: six classes of `w_cell`, `activation_faults` / `activation_crash_no_loss` / `skip_then_raise` r ∈ {1, 2, 3} of `w_activation`, `near_full`, `drain_failure`, `broken_slot`, `cost_growth`. Reservation sweep does not belong to fault worlds, halted after sufficient for verdict per registration.

**Unit tests and mutations**: 36 unit tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e139_tightened_floor.rs` outputs 36 = original 21 + new 15), 5 consecutive runs each showing `test result: ok. 36 passed`; new 15 include K1–K9 of registration Section 7 (one per anchor, expected values copied from output of independent audit script `anchors.py` at end of Section 13) and 6 supplemented tests (anchor at rollback target after rollback, rollback target taking oldest candidate root in window, newest valid root on surviving disk in window after disk drop, upper bound difference counter, skipped-number prefix copy matching original prefix, steady four new arms verbatim identical to original G2S).
Mutation table `scripts/mutation/tables/e139_tightened_floor.tsv` has 26 mutations (original 16 + new 10):

```bash
cd scripts/experiment-apparatus && MUTATE_JOBS=3 MUTATE_CARGO_JOBS=2 bash ../mutation/mutate.sh e139_tightened_floor e139_tightened_floor.rs ../mutations/e139_tightened_floor.tsv
```

Caught 26, invalid 0, unred 0 (by symbol counts at start of each result line: `✅` 26, `⏭` 0, `❌` 0, `💥` 0, `⚠️` 0, `⏱` 0, `🧱` 0); closing summary: "Counts: memory ceiling hits 0 (2G limit), timeouts 0, unscheduled 0, squeezed out by total limit 0", "Restored, baseline still all green".
"All 26 mutations caught" indicates only that code with assertions was tested by mutations; MW2 (reclamation window anchor taking max txg prior to rollback) was caught solely by the supplemented unit test for anchor after rollback. Original M1 source and replacement text added `| Arm::TightenedStatesWindowed(_)` (registration revision R1; apparatus added an `Arm` variant, after which original anchor had 0 hits, mutation point unchanged).

**Detector positive controls** (Registration Section 6 row 4, original three `name=verdict` rows in old artifact verbatim unchanged): `crit2_a_drain_false_enospc=3424`, `crit2_a_broken_false_or_stall=64`, `crit2_g_near_full_wad_cost5=48`, `crit2_g2r_mount_growth_false_or_stall=32`, `crit2_d_activation_fakes=584`, `crit2_gtxgcap_skip_retention=12576`, all six cells > 0 ⇒ false ENOSPC, stalls, fake candidates, minimum retention detectors distinguish differences.

**Four questions to ask first when struck** (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` "Criteria can be written wrong: determine which kind first upon hit"):

| Question | Answer |
|---|---|
| Does it distinguish arms | Distinguishes: G2SW_ringgate and G2SW differ only in reclamation threshold (W2–W4 identical); G2SW_ringgate judged 108 rows all 0, G2SW 46 rows > 0; hit was not caused by premises shared across arms |
| Can the judged system observe what the criterion distinguishes when making decisions | Can observe: at reclamation, there is a valid root outside the window; excluding it from threshold (G2SW) or retaining it to press down threshold (G2SW_ringgate) is a choice of threshold formula, not unobservable information |
| Which clause of the criterion's literal text is satisfied | Section 6 row 8: roots in candidate set (valid ∧ txg ≥ F ∧ in window) reference reused blocks; row 9: root selected during crash recovery is reused |
| Does the fix given in pre-run clauses still hold on hit cells | Pre-run reverse acceptance clause wrote "G2SW certain cell > 0 ⇒ accept, pass structure verbatim to main agent, do not alter window definition or criterion for it"; no fix was given, this question is inapplicable |

**Where fake candidates come from** (deduced, based on `w_detail_activation` rows in artifact and apparatus `reuse_bound` formula; step-by-step states from printout of a temporary diagnostic unit test; diagnostic unit test deleted, no artifact retained): on root slot write failure, apparatus advances txg by one cell before publishing, leaving root of previous revolution in failed slot (`publish`, lines 1175–1190 of apparatus).
Disk 1 hosts only region 1; a single write failure in region 1 leaves disk 1 with one fewer new root than disk 0, and the old root left in that slot drops outside window (txg + 3S > N, N being new root on disk 0): the oldest valid root in window passes over it, reclamation threshold (line 946) exceeds whole-ring threshold of G2SW_ringgate (still pressed down by this old root); blocks with release generation falling between the two are reused, losing blocks referenced by old root.
Subsequently dropping disk 0: recovery selects newest root on disk 1, anchor rewinds to it (lines 1470 onwards, `newest_durable_txg` = selected root); window covers that reused old root again, making it a fake candidate by candidate set definition; when old root itself is selected root, it becomes unrecoverable.
Minimal example in artifact: `grep -n 'name=w_detail_activation line=w_activation variant=activation_faults arm=G2SW_tightened_states_window s=4 seed=0 failures=0 crash_window=2 lost_disk=0' e139-tightened-floor-2026-10-04.out`
Outputs `1106:E7RESULT name=w_detail_activation line=w_activation variant=activation_faults arm=G2SW_tightened_states_window s=4 seed=0 failures=0 crash_window=2 lost_disk=0 fakes=1 unrecoverable=0`
(`failures=` is sequence number of failed root slot write, 0 is first root slot write after prefix, not "no failures"). Diagnostic printout of same cell: roots in ring 34, 36 … 45, 47 (35 and 46 unwritten), N = 47, G2SW threshold 36, whole-ring threshold 34; reused block release generation 35; damaged txgs recorded as 34 and 35;
after dropping disk 0, selected 43, candidates 37, 40, 43, 34, where 34 is fake candidate; original arm G2S and G2SW_ringgate had zero reused blocks in same cell.
Registration Section 4 leaning "deduced (designer)" (unrecoverable emerging in cells with `skip_then_raise` dropping disk 0) matched: all 3 unrecoverable rows are `skip_then_raise` S = 4 dropping disk 0; the half in the same item where "broken slot falls on disk 1" (H1) did not match: `w_broken_slot_h1` four arms 0 rows verdict changed, fake candidates and unrecoverable all 0.

**`broken_slot` rows unpressured**: G2SW's `w_broken_slot` and `w_broken_slot_h1` each 4 rows have `max_publications=0` (original G2S same cell `max_publications=4`): window excludes old root left in broken slot from threshold, pool never reaches insufficient acceptance under broken slot. The 0 false ENOSPC in these four rows is an "unpressured" 0, not "still zero under pressure"; window item indeed functioned in these rows (positive threshold difference count in `broken_slot` class of `verdict_window_gate_reach` was 62528). Cells judged for false ENOSPC under pressure are `near_full` (`admissions_under_pressure` 6504–9672), `drain_failure` (`max_publications` 2–6), `cost_growth` (`true_enospc` 24–32).
```bash
grep -E '^E7RESULT name=w_broken_slot arm=G2SW_tightened_states_window s=4 c0=1 ' e139-tightened-floor-2026-10-04.out
grep -E '^E7RESULT name=broken_slot arm=G2S_tightened_states s=4 c0=1 ' e139-tightened-floor-2026-10-04.out
```
```text
E7RESULT name=w_broken_slot arm=G2SW_tightened_states_window s=4 c0=1 c_max=5 seeds=8 seeds_with_no_space=0 seeds_halted=0 first_failure_window=0 false_enospc_arm=0 false_enospc_raw=0 write_after_delete_failed=0 max_publications=0 bound=8 stalls=0 fakes=0 unrecoverable=0 retention_violations=0 injected=728 forced_per_window_x100=0 max_lag=52
E7RESULT name=broken_slot arm=G2S_tightened_states s=4 c0=1 c_max=5 seeds=8 seeds_with_no_space=0 seeds_halted=0 first_failure_window=0 false_enospc_arm=0 false_enospc_raw=0 write_after_delete_failed=0 max_publications=4 bound=8 stalls=0 fakes=0 unrecoverable=0 retention_violations=0 injected=752 forced_per_window_x100=3 max_lag=52
```

**Clause-by-Clause Match** (Registration Section 6 row 11; readings per main agent: Class A = constructed and identical, Class B = absent from apparatus, Class C = constructed but differing; Class C counts as non-matching, Class B listed individually).
Apparatus line numbers refer to `scripts/experiment-apparatus/e139_tightened_floor.rs`, implementation line numbers refer to `crates/` state on 2026-10-04 recorded in Section 3 of registration. D16 (Publish Semantics) settled item 1 table has 13 rows plus "non-empty" paragraph total 14 items; when multiple classes appear in a row, recorded under heaviest class (Class C > Class B > Class A), individual details written in Differences column. Result: 5 Class C, 5 Class B, 4 Class A.

| Clause | Class | Lines in apparatus | Differences and direction | Basis in E139 (Tightened Form of Per-Disk Rollback Floor) |
|---|---|---|---|---|
| Reallocatable | A | 946 (`reuse_bound` window arm formula `floor_active.max(oldest valid root in window)`), 1014 (`allocate`) | Formulas match verbatim; how F_active is taken belongs to "Activation" row | Old threshold form is (Arm G2S); window half is not (no experiment, tripartite verdict) |
| Oldest valid root in ring | A | 712 (judging valid by instance table), 728 and 740 (minimum txg of self-proving valid roots on surviving disks), 1183 (unmodified on write failure), 1201 (recorded only on successful root write, unpersisted publishes do not enter ring) | None | Yes (`df` lag and old threshold of Arm G2S both take it) |
| Oldest valid root in one-ring window | C | 181 (predicate `txg + window_length > anchor`), 802 and 807 (minimum in window, fallback to F_active when empty), 1201 and 1470 (anchor = newest root recorded by process / selected root after recovery) | Class C: after administrator rollback apparatus publishes no forward roots, anchor stops at rollback target (1498 onwards); clause and implementation take each root recorded, publishing forward root on rollback (Registration Section 11 difference ⑦); apparatus anchor is lower, window further back, threshold no higher than implementation, reclaiming less. Class B: unreadable slots at mount merged into threshold, in-process root recording rereads first, failed self-proof calculated by recorded max txg; apparatus has no read errors, unmodeled (⑤). Class B: among six locations sharing window, checker candidate filtering, checker I-3.1 (Allocated Statistics Match) Ding-defer threshold, shadow accounting candidate filtering are unmodeled in apparatus, allocator threshold (946), rollback candidate set (794), raising F upper bound (849) are present | No (no experiment: window form measured on tripartite replica) |
| Rollback floor F | B | 257–262 (root record carries F), 1248 (monotonically non-decreasing), 1290 onwards (raised only when acceptance insufficient) | Unmodeled: system configuration separately records a copy of F; normal unmount raises to txg of active version (apparatus has no system configuration, no unmount) | Partial: F accompanying root record, raised during acceptance follows E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty); system configuration copy and unmount are not (tripartite prototype) |
| Upper bound for raising F | C | 849, 827, 839 (min(newest on each disk in window, 4th newest non-empty in window); taking oldest valid root in window if fewer than 4; Reading P), 854 and 917 (disposal target = min(release generation, 4th newest non-empty root)) | Class C: implementation valid roots additionally filter `txg >= current_floor` (`crates/singlefs-core/src/mount.rs` `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`), apparatus does not filter (Registration Section 11 difference ②); apparatus "4th newest" and "oldest valid root" may be older, upper bound no higher than implementation, raising less. Class C: "non-empty" apparatus uses ground truth set (see "non-empty" row). Class B: unreadable slots at mount in window treated as potentially non-empty, F not passing it, apparatus unmodeled; unmount raising F does not evaluate upper bound, apparatus has no unmount | Measurement basis min(newest on each disk, 4th newest non-empty), taking oldest valid root if fewer than 4 is (Arm G2S, user 2026-09-12 measurement basis); window, unreadable, unmount locations are not |
| Root slot read error this time | B | None | Unmodeled: apparatus has no read errors, nor "this process knows slot where root lives"; handling read errors across raising F, rollback, calculating F_active unreached (⑤); C563 (A bad read of one root ring slot is taken as no root; other callers not judged) owed | No (user settlement and tripartite verdict) |
| Activation | C | 1434 (`floors_after_recovery`) | Apparatus: takes max F carried by valid roots on each surviving disk, takes min, takes no higher than candidate floor; clause (SysPre): max of newest persistent valid root F on surviving disks, max F on abandoned timelines, max readable F in system configuration, taking max of three (difference ③). Apparatus F_active no higher than clause, reclamation threshold no higher than clause, reclaiming less | Apparatus method is (following E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)); clause method is not (tripartite prototype) |
| Rollback candidate set | C | 930 (`candidacy_floor`), 1064 (`count_fake_candidates`), 1498 (`admin_rollback` target selection), 794 (window), 712 (instance table) | Window and judging valid by instance table match; lower bounds differ: apparatus candidate floor = max F carried by all self-proving roots on surviving disks (including abandoned timelines); clause is txg >= F_active (⑥); apparatus candidate set is no smaller than clause (originally miswritten "no larger", inverted direction: Rerun 2 measured old floor candidate set strictly larger than clause form 53392 times, strictly smaller 0 times), fake candidates leaning high; rollback does not publish forward roots (⑦, same as "oldest valid root in one-ring window" row) | Window half is not; candidate floor method is (following `candidacy_floor` of E135 (Dynamic Rollback Floor) and E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)) |
| Chain of raising F | B | 1248, 1228, 1169 (new F persists into next publish) | Unmodeled: implementation updates system configuration across barrier after raising F; apparatus only updates in root slot | Partial: root slot behavior follows E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty); system configuration update is not |
| Normal unmount | Class B | None | Not modeled: apparatus has no unmount, no unmount mark, no version with 0 tree table rows | No (tripartite prototype) |
| Acceptance | B | 1008 (`user_allocatable`: reallocatable + live metadata − reservation pool, then capped at `df`), 1350 onwards `run_user_window` (first publishes the window being accumulated, pushes empty to raise F, then judges), 34–36 and 250 (B = 4 + 2 × 2 = 8) | Matches; Class B: writable mount pushes after writing the row, and pushes empty to raise F when a deletion-class publish of mount resumption is refused for space; the apparatus has no mount | Yes (B = 8, reservation pool and residue formulas) |
| `df` | Class A | 989 (unused + released), 1003 (`df_arm`), 261 and 270 (reservation pool 10 + 7 c_max, residue 5 + 6 c_max), 973 and 862 (lag and its lower bound, reading a) | Matches item by item; `crates/` has no lag item (`grep -rn '滞后量\|lag_blocks' crates/ --include=*.rs \| wc -l` outputs 0), this row has no implementation to compare against, only the apparatus | Yes |
| Ring depth lower limit | A | 43 (S in {1, 4, 16}), artifact rows with `s=1` | Clause S >= k_tol + 2 = 4; S = 1 is below ring depth lower limit, judged and reported as such; S = 4, 16 are within scope | Yes (ring depth threshold is probe number, see "Postscript After Round 4 Tripartite Argument") |
| How "non-empty" is recognized on disk | C | 1196 (records `nonempty_txgs` on publish), 827, 854 | Apparatus uses ground truth set (whether this publish modified user-visible state), deduplicating by txg; clause compares root pointers of two trees in tree table against preceding valid root, deduplicating by content (difference ④). In history of this apparatus without forward roots after rollback, states of each non-empty root are distinct, two counting methods yield identical number (deduced, unpinned by unit test) | No (user settlement, tripartite verdict) |
| (Outside settled item 1 table) D23 (Journal Role & Format) Root slot write failure | C | 1175–1190 (`publish` failure branch, line 1183 advances txg by one cell before publishing) | Apparatus advances txg and republishes (target form of D23 (Journal Role & Format) line 399); implementation in this version freezes failed publish, republishing same txg as is (same decision line 402). Registered difference (Difference ①), untriggering halt; impact see "Impact of Difference ① (Root Slot Write Failure Advancing txg) on Conclusion" | No |

Table ①–⑧ are sequence numbers of "Registered Differences Not Triggering Halt" in Section 11 of registration: ① root slot write failure advancing txg, ② valid roots for upper bound filtering `txg >= current_floor`, ③ "Activation" method, ④ "Non-empty" method, ⑤ unreadable slots and in-process rereading, ⑥ candidate set lower bound taking F_candidate, ⑦ administrator rollback not publishing forward roots, ⑧ implementation mount treating only `Unreadable` as unreadable slots (difference on implementation side, apparatus has no counterpart, not in table).
Direction summary: anchor of oldest valid root in window (⑦), activation (③), raising F upper bound (②) three Class C items cause apparatus to reclaim less and raise F less, safer than clause and implementation; rollback candidate set (⑥) causes apparatus fake candidates to lean high (originally written leaning low, direction inverted, see "Rerun 2"): zeros counted by apparatus remain zero under clause form; non-zeros counted by apparatus may be generated by candidate floor adopting old form; "non-empty" (④) has identical values under both methods in this apparatus's history.

**Impact of Difference ① (Root Slot Write Failure Advancing txg) on Conclusion** (Registration Section 11: registered, not triggering halt, listed separately here): window threshold and whole-ring threshold differ only when ring contains valid roots outside window; in this apparatus, valid roots outside window have only one source: root slot write failure advancing txg by one cell, leaving root of previous revolution in failed slot (`publish`, line 1183).
In artifact, positive threshold difference counts are all 0 across seven classes of worlds not injecting root slot write failures (`steady`, `torn_newest`, `disk_loss`, `admin_rollback`, `rollback_first_publish`, `near_full`, `cost_growth`) in `verdict_window_gate_reach`, and all > 0 across six injected classes, matching "deduced (designer)" of Section 4.
Implementation in this version (D23 (Journal Role & Format) line 402) freezes entire publish on failure, republishing same txg as is, creating next publish only after republish succeeds (`crates/singlefs-core/src/allocator.rs` `record_root_written_by_this_process` assertion text: failed publish rewinds along with root ring table, republish uses same txg): rewritten into same slot without leaving holes, window is constantly identical to whole ring, and thresholds of G2SW and G2S are equal step-by-step.
Hence the 46 flipped rows of fake candidates in first half refer to window on target form of D23 (Journal Role & Format) line 399 (advancing txg and republishing, failure handling code implemented in later milestones); 2026-10-04 implementation cannot create holes in these rows (deduced: unverified on implementation side, based on apparatus mechanism and artifact positive threshold differences being constantly 0 in non-injected worlds).
Whether settled item 1 sentence and D23 (Journal Role & Format) two failure handling modes should be read together is submitted to main agent.

**Comparison with Settled Item 1 Sentence of D16 (Publish Semantics)** (comparison only, not modifying decision text; whether and how to modify submitted to main agent for tripartite process):

- That sentence: "Form adopts Arm G2S of E139 (Tightened Form of Per-Disk Rollback Floor); after reclamation threshold adopts one-ring window, apparatus lacks window item, this sentence is no longer verbatim identical with apparatus run, recorded as debt that E139 (Tightened Form of Per-Disk Rollback Floor) needs rerun". Apparatus has window item; clause-by-clause match has 5 Class C items; "no longer verbatim identical" still holds and cannot be withdrawn.
- First item of rationale paragraph: "Arm G2S across all fault worlds ... zero fake candidates, zero unrecoverable, false ENOSPC per respective `df` is 0, measured max publishes per admission 7 <= 8, no stalls under pre-run reservation pool": still holds for original arm G2S under old threshold (old 900 rows byte-for-byte unchanged); for G2SW with window, false ENOSPC is 0, no stalls, publishes <= 8 hold, zero fake candidates and zero unrecoverable do not hold (46 rows, 3 rows). Figures for publishes and stalls:

```bash
grep -E "^E7RESULT name=w_(near_full|drain_failure|broken_slot|cost_growth)(_h1)? arm=G2SW_tightened_states_window " e139-tightened-floor-2026-10-04.out | grep -o 'max_publications=[0-9]*' | sort | uniq -c
grep -E "^E7RESULT name=w_[a-z0-9_]+ arm=G2SW_tightened_states_window " e139-tightened-floor-2026-10-04.out | grep -cE ' stalls=[1-9]'
```

```text
      8 max_publications=0
      3 max_publications=2
      2 max_publications=3
     17 max_publications=4
      2 max_publications=5
      2 max_publications=6
0
```

- Two ways out for owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus): "if matching, withdraw 'no longer verbatim'; if not matching, alter that sentence or alter apparatus": does not match; which path to take submitted to main agent. Discriminative self-proof passed: result rows between G2SW_ringgate and G2SW differ in 55 / 111 rows.

**Collateral (Registration Section 6 row 13, for reporting only, not entering verdict)**: Readings P and Q differ in 0 rows across 111 judged rows, but have 6 rows of `retention_violations` in `w_activation` (Reading Q reports 24 / 24 / 24 / 24 / 24 / 24, Reading P reports 0): disk 1 has one fewer new root than disk 0, Reading Q calculates valid root count per surviving disk 1, dropping disk 1 below 4 valid roots (the exact cell pinned by unit test K5). Clause does not specify Reading P or Q, reported here as is.

```text
E7RESULT name=w_trajectory line=w_activation arm=G2SW_tightened_states_window variant=skip_then_raise_r3 s=4 lost_disk=0 failures=2 gate_difference_samples=5760 gate_difference_positive=5760 gate_difference_max=16 gate_difference_max_after_supply=na gate_difference_min=1 gate_difference_last_max=16 ceiling_difference_samples=0 ceiling_difference_positive=0 ceiling_difference_max=na ceiling_difference_max_after_supply=na ceiling_difference_min=na ceiling_difference_last_max=na lag_samples=2880 lag_positive=2880 lag_max=52 lag_max_after_supply=na lag_min=52 lag_last_max=52 df_samples=2880 df_positive=2880 df_max=1076 df_max_after_supply=na df_min=1076 df_last_max=1076 df_below_user_blocks_windows=0
E7RESULT name=w_trajectory line=w_near_full arm=G2SW_tightened_states_window s=16 c0=1 c_max=5 gate_difference_samples=40560 gate_difference_positive=0 gate_difference_max=0 gate_difference_max_after_supply=0 gate_difference_min=0 gate_difference_last_max=0 ceiling_difference_samples=8280 ceiling_difference_positive=0 ceiling_difference_max=0 ceiling_difference_max_after_supply=0 ceiling_difference_min=0 ceiling_difference_last_max=0 lag_samples=12936 lag_positive=12936 lag_max=44 lag_max_after_supply=44 lag_min=16 lag_last_max=32 df_samples=12936 df_positive=6576 df_max=1076 df_max_after_supply=180 df_min=0 df_last_max=8 df_below_user_blocks_windows=6480
```

In the first row, all 5760 threshold-difference readings are positive (the window continues to act after holes appear), and `ceiling_difference_samples=0` (activation cells do not enter the empty-publish loop, so the upper-bound difference has no readings).
Compare the reading arms’ trajectories with G2SW row by row (compare whole rows except the arm name; commands and unmodified output):

```bash
for other in G2SW_Q_disk_hold G2SW_L_ring_lag; do echo -n "$other: "; diff <(grep '^E7RESULT name=w_trajectory ' e139-tightened-floor-2026-10-04.out | grep ' arm=G2SW_tightened_states_window ' | sed 's/ arm=[A-Za-z0-9_]*//') <(grep '^E7RESULT name=w_trajectory ' e139-tightened-floor-2026-10-04.out | grep " arm=$other " | sed 's/ arm=[A-Za-z0-9_]*//') | grep -c '^<'; done
```

```text
G2SW_Q_disk_hold: 0
G2SW_L_ring_lag: 1
```

G2SW_Q's 151 trajectory rows match G2SW line-by-line; G2SW_L differs in only 1 row (`slot_write_fail` S = 1): trajectory sample points are F-raising decision points, where Readings P and Q never took differing values; the two readings differed only in the minimum retention check following each publish and recovery (6 rows of `retention_violations` for G2SW_Q in Collateral section), which does not enter verdict.
Hence the "0 rows differing in verdict" in Section 6 row 12 means the two readings did not traverse divergent paths at decision points in these worlds (unit test K5 constructed a divergence), not that this cell of the clause is inconsequential.

**What Rerun 1 Cannot Answer**:

1. Counting model, no I/O; roots in window taken per apparatus `valid_roots`, unmodeled unreadable slots at mount and in-process rereading (Difference ⑤); among eight registered differences in Section 11, ① determines whether this verdict flip has a counterpart in 2026-10-04 implementation (see "Impact of Difference ① (Root Slot Write Failure Advancing txg) on Conclusion").
2. Admin rollback does not publish forward roots (Difference ⑦), anchor halts at rollback target: window between rollback and next publish differs from implementation; 0 fake candidates in `admin_rollback` world holds only for this form in apparatus.
3. Candidate set lower bound taking F_candidate (Difference ⑥): apparatus candidate set is no smaller than clause's, with fake candidates leaning high (originally written "no larger", "leaning low", inverted direction, see Rerun 2); the 46 flipped rows were counted under this high-leaning measurement basis: after Rerun 2 changed candidate floor to clause form, G2SW fake candidates in the same batch of `activation` rows became 0 in 40 rows and remained > 0 in 6 rows.
4. 4 rows of `broken_slot` and 4 rows of H1 unpressured (`max_publications=0`), false ENOSPC verdict in these rows is a vacuous 0; cells where broken slot falls on disk 1 (H1) did not change verdict from original sample points, but being unpressured, cannot serve as evidence that "window also passes under broken slot".
5. 15 rows of H2 differ in verdict from original sample points, recorded as "unstable" per pre-run clause: cells dropping disk 0 vs disk 1 swap zero and non-zero (in `failures=0` cell of `skip_then_raise`, original sample point dropping disk 0 yields 16 / 32 / 48 fake candidates and dropping disk 1 yields 0; H2 inverts, dropping disk 0 yields 0, dropping disk 1 yields 8), unrecoverable in cells with 1 or 2 failures disappears on H2. Conclusions of "zero" vs "non-zero" cannot be made independent of which disk skipped numbers fall upon.
6. Reservation sweep labeled "unrun after sufficient for verdict": both halves of issue ticket row 1 reached verdicts (F3, F4 untriggered), halted per Section 11 and does not resume; 18 cells of reservation sweep in original artifact are figures from old apparatus; G2SW has no corresponding sweep rows.
7. Difference between Readings P and Q appeared only in unit tests (line-by-line comparison of reading arm trajectories); artifact did not traverse it: identical verdicts for the two reading arms in original classes of worlds does not indicate that unwritten cell in clause can remain unwritten.
8. G2SW did not run in activation cells of S = 1 (original registered activation family swept only S ∈ {4, 16}; S = 1 is below ring depth lower limit); S = 1 rows in `w_cell` and `w_near_full` judged and reported as usual, labeled below ring depth lower limit.

### Rerun 2 (Candidate Floor Adding System Configuration F, Reclamation Threshold and Other Five Locations Returning to Whole Ring)

Rerun registration `e139-r2-prereg.md` (written prior to apparatus changes; after apparatus and unit tests were written, before entire artifact was run once, "Revisions" R1–R5 were added; Section 6 criteria and Section 10 failure clauses untouched by a single word).
Run on 2026-10-05, artifact `e139-tightened-floor-2026-10-05.out` (6271 lines, final line `E7RESULT name=done emitted=6271`, `grep -c '^E7RESULT'` also 6271, exit code 0); apparatus `scripts/experiment-apparatus/e139_tightened_floor.rs` modified in-place (5229 lines, leaving arms and lines from Rerun 1 untouched),
adding: CF with candidate floor taking clause form (`Arm::ClauseFloorWindowed`, used by those seven arms in `f_` rows), four `Arm::WholeRing` arms (RGall, RGall_Q under reading Q, positive controls `RGall_short` and `RGall_Q_short` each with a shortened modifier: reclamation threshold, candidate set, F-raising upper bound, and lag all taking whole ring), `disk_stuck` world (one disk permanently unwritable to roots, stuck disks {1, 0}, occupancy {0.30, 0.70}, S ∈ {4, 16}, three teardown modes, 4 arms), and samplings of three cost items.
Issue ticket `m3-r1-e139-r2-questions.md` has two rows:

1. After apparatus candidate floor changed to clause form (counting system configuration F into lower bound), whether verdicts in Rerun 1 cells change, and how to write the sentence on the experiment page "apparatus candidate set is no larger than clause's" per the new artifact;
2. After reclamation threshold and five other locations sharing window return together to oldest valid root in ring (RGall), whether Arm G2S maintains 0 fake candidates, 0 unrecoverable, and 0 false ENOSPC across those fault worlds of Rerun 1 and on target form advancing txg on root slot write failure; and what the three cost items are (blocks pinned after mount, extra blocks pinned per hole, growth of pinned units over publishes when one disk is stuck).

Invalidation clauses matched first: V1 (prior to changes, 2026-10-05 source code produced 2231 lines, sha256 `2563dde0002f65c266a5223ecc3973a78e715ab1ba0a5e02f2ecbabd8448bdd2`, matching Rerun 1 artifact) passed; V3 (55 unit tests) passed; V2 judged on entire artifact:

```bash
cd scripts/experiment-apparatus && diff <(grep -v -e '^E7RESULT name=f_' -e '^E7RESULT name=verdict_f_' ../../research/results/e139-tightened-floor-2026-10-05.out | grep -v '^E7RESULT name=done') <(grep -v '^E7RESULT name=done' ../../e139-tightened-floor-2026-10-04.out) | wc -l
```

```text
0
```

Outputs `0`: deleting rows starting with `f_`, `verdict_f_` and `name=done`, Rerun 2 artifact matches Rerun 1 line-by-line (`w_` family and all old rows unchanged by apparatus modifications). V4 untriggered (exit code 0); V5, V6, V7, V8 judged by artifact, detailed below in Readings, all 0.

**Readings** (read line by line per Section 6 of registration, each reporting independently; no judgment on whether these numbers overturn or establish D16 (Publish Semantics) settled item 1, which requires tripartite deduction):

| Quantity | Reading | Pre-run clause |
|---|---|---|
| Row 1: CF reached | In `f_` rows, candidate floor difference g > 0 has 53736 samples, landing solely in `activation_faults` (72 of 72 rows), `activation_crash_no_loss` (72 / 72), `skip_then_raise` (432 / 432), and `disk_stuck_h3` (3 of 48 rows); g > 0 sample count is 0 for `steady`, `torn_newest`, `disk_loss`, `slot_write_fail`, `admin_rollback`, `rollback_first_publish`, `near_full`, `drain_failure`, `broken_slot`, `cost_growth`, and `disk_stuck` (stuck disk 1) | F1 untriggered (`cf_never_reached=0`); "counts unchanged" on those eleven classes of worlds indicates only that clause-form lower bound was unreached there |
| Row 1: line-by-line comparison | Comparing 610 rows of `w_` and `f_` rows line by line, 228 rows differ on any of three counts (210 rows changing 3-tuple verdict): all land on G2SW, G2SW_Q, G2SW_L arms, 76 rows each (`activation_faults` 8, `activation_crash_no_loss` 8, `skip_then_raise` 60, including 30 of H2); G2SW_ringgate and three `_short` arms change 0 rows | F2 triggered: counts in that batch of cells in Rerun 1 were under old lower bound measurement basis, not clause measurement basis |
| Row 1: which direction changed | In `f_activation` of G2SW, 46 rows fake candidates 4816 → 200 (40 rows become 0, 6 rows remain > 0, all in `skip_then_raise_r2` / `r3`, S = 4); in `f_activation_h2`, 30 rows 1295 → 0; unrecoverable 96 → 96; false ENOSPC unchanged; G2SW_Q, G2SW_L identical. G2SW still has 3 unrecoverable rows in `f_activation`, same as Rerun 1 | Matches "⑥ alone changed to clause form yields 46 rows → 6 rows" in Verdict W2 section |
| Row 1: attribution | Class A (solely count measurement basis change) 228 rows, Class B (execution path changed) 0 rows, Class C (g never positive yet row changed) 0 rows; `rollback_target_changed` is 0 across all arms | V5 untriggered |
| Row 1: direction of that sentence | Checks where old floor candidate set is strictly larger than clause form: 53392 times; clause form strictly larger than old floor: 0 times; system config F below root F violations: 0 times | Former > 0: that sentence was inverted; old candidate set is no smaller than clause's, with fake candidates leaning high; V6, V7 untriggered |

| Quantity (Row 2 first half, RGall is `RGall_ring_all`, RGall_Q is `RGall_Q_ring_all`) | Reading | Pre-run clause |
|---|---|---|
| RGall, RGall_Q fake candidates, unrecoverable, false ENOSPC in judged cells | Across 123 rows of `verdict_f_window` (111 rows from Rerun 1 plus 12 rows of `f_disk_stuck`), all three items are 0; 12 rows of `f_disk_stuck_h3`, H1, H2 geometric points likewise 0; 3 rows of `rollback_first_publish` (fake candidates 37 / 173 / 814, unrecoverable 24) reported only without judgment per original registration, identical to G2SW, G2SW_ringgate; comparing 24 rows of `f_crit1_push` line-by-line with Rerun 1 `w_crit1_push`, 0 rows differ; RGall rows where `crit1_push` does not hold are 0 | F9 untriggered: Row 2 first half does not flip |
| Opposing arm on same batch of rows | G2SW (CF) has 10 rows with fake candidates (6 rows of `f_activation` plus 4 rows of `f_disk_stuck`), 7 rows unrecoverable (3 rows of `f_activation` plus 4 rows of `f_disk_stuck`; `f_disk_stuck_h3` has another 4 rows of fake candidates and unrecoverable, not entering this 123-row total); G2SW_ringgate is all 0 | In `disk_stuck`, non-zero rows for G2SW occur in teardown dropping the unstuck disk, where surviving disk happens to be the stuck one (`lost_disk_0` or `lost_disk_1`) |
| Positive control | `RGall_short`, `RGall_Q_short` on S = 4, 16 of `steady` both have fake candidates > 0 (`rows_where_the_detector_saw_zero_fake_candidates=0`); all three `_short` arms of G2SW family identical | F3 untriggered: detector observes error of threshold set too high |
| False ENOSPC in unjudged cells | In `cost_growth`, two rows where overhead rises to c_max + 3 (S = 4, 16, c₀ = 1) have 32 occurrences each, identical to G2SW, G2SW_ringgate and Rerun 1 `w_cost_growth`; in `disk_stuck`, RGall and RGall_Q across 6 rows of S = 16, occupancy 0.70 (three teardowns, stuck disks 1 and 0) have 32 / 24 occurrences each, identical to G2SW_ringgate, G2SW is 0; `true_enospc` in `disk_stuck` is all 0 | F8 triggered: out-of-scope observations in 12 rows (6 rows RGall plus 6 rows RGall_Q), not flipping Row 2; `judged_false_enospc` is all 0 |
| RGall and RGall_Q (Readings P and Q) | 0 rows differing in verdict across 175 rows; `disk_without_valid_root_events` is 0 across both readings: across entire artifact, never once did a surviving disk have zero valid roots across whole ring | F5 untriggered; the two readings did not traverse divergent paths on this artifact (unit test K12 constructed a divergence) |
| Collateral: stalls and minimum retention | In `disk_stuck` S = 16, occupancy 0.70, stuck disk 1 three rows: all 8 seeds of RGall, RGall_Q, G2SW_ringgate stall completely (`stalls=8`), G2SW is 0; in `f_drain_failure` S = 4, c₀ = 5, 3 failures single row, RGall, RGall_Q have `retention_violations=24` (G2SW is 0; original G2S in this cell is likewise 24, corresponding to D16 (Publish Semantics) settled item 1 scope where "retaining at least 4 states is fulfilled only when failures <= k_tol") | Reports numbers only |

| Quantity (Row 2 second half, costs of RGall) | Reading | Pre-run clause |
|---|---|---|
| ⓐ Pinned blocks after mount | See "Costs of Various Worlds" table (`verdict_f_cost` row per (class, arm), RGall and RGall_Q identical) | Reports numbers only |
| ⓑ Extra pinned per hole | See "Costs of Various Worlds" table; G2SW (CF) has x = 0 across all 15 classes | V8 untriggered |
| ⓒ Pinned units growth over publishes | See "ⓒ Scale" table: in `disk_stuck`, RGall, RGall_Q, G2SW_ringgate judge "unbounded" (s(2→4) = 1.00) in 7 of 8 rows; 1 row S = 16 occupancy 0.70 scale 4 unreached (all 8 seeds stalled); G2SW judges "bounded" across all 8 rows; on `broken_slot` and H1, across 4 rows of RGall each, 2 rows of S = 4 judge "unbounded" (s(2→4) = 1.00), 2 rows of S = 16 judge "bounded" (0.04–0.09) | F4 untriggered (G2SW does not judge "unbounded"); F11 triggered: RGall pinned units grow with publishes on S = 4 of `broken_slot` and H1; "unbounded" on `disk_stuck` is hardcoded after Section 6 table, deducible from definition, reports numbers only |
| Old roots in holes surviving at most two revolutions in other worlds | In `skip_then_raise_r3`, S = 4, 4 rows each of `f_activation` and `f_activation_h2` (lost disk 0 / 1 × failures 1 / 2) identical across six arms (valid root with txg + 2 × 3S <= N present on surviving disk across 48 samples with 1 failure, 320 with 2 failures), total 48 rows | F10 triggered: "old roots in holes survive at most two revolutions in other worlds" fails on this class, bounded status unmeasured, Row 2 second half records "insufficient for verdict" for it |
| Geometric sensitivity | Three-tuple verdicts of RGall, RGall_Q differ in 0 rows on H1 (4 rows), H2 (36 rows), H3 (12 rows) from original sample points; G2SW differs in 6 rows on H2, 8 rows on H3; G2SW_Q, G2SW_L differ in 6 rows on H2; bounded status: RGall, RGall_Q, G2SW_ringgate have 1 row difference between "unreached" and "unbounded" on H3 and occupancy tiers (S = 16, occupancy 0.70 row), no flip between "bounded" and "unbounded" | F7 triggered literally (`f7_growth_rows_with_a_different_bound_h3=3`, `_fill=3`): RGall 3-tuple verdicts stable, bounded status 1 row difference is unreached scale rather than verdict flip; H2, H3 rows of G2SW recorded as "unstable" |

Costs of various worlds (one row for RGall; in parentheses are G2SW ⓐ peak and period-end; ⓑ lists only classes where x is non-zero). ⓐ "rec" = `pinned_count()` after each recovery, ⓑ "x" = blocks pinned under this arm threshold minus blocks pinned under window threshold, sampled at each root persistence; `after_supply` is peak after consuming initial supply, `na` means never consumed; Σx÷Σh is sum of x divided by sum of slot counts of valid roots outside window on samples where h > 0, max(x÷h) is per-sample maximum:

```text
世界                      ⓐ rec：采样 / 为正 / 峰值 / after_supply / 期末          ⓑ x：采样 / 为正 / 峰值 / after_supply / 期末   Σx÷Σh   max(x÷h)
activation_faults         4096 / 4096 / 611 / na / 611（G2SW 611 / 611）            14336 / 2648 / 78 / na / 78                      6.36     39
activation_crash_no_loss  8192 / 8192 / 663 / na / 637（G2SW 611 / 611）            22528 / 4516 / 78 / na / 78                      5.86     39
skip_then_raise           24576 / 24576 / 663 / na / 663（G2SW 611 / 611）          86016 / 44960 / 195 / na / 195                   13.88    52
slot_write_fail           无恢复                                                    28962 / 420 / 94 / na / 0                        12.49    94
drain_failure             无恢复                                                    97392 / 6680 / 85 / 85 / 0                       1.36     54
broken_slot               无恢复                                                    66672 / 48560 / 1086 / 1086 / 699                314.88   1086
disk_stuck（卡住盘 1）    72 / 72 / 2262 / na / 2262（G2SW 585 / 585）              9936 / 7320 / 1872 / 1157 / 1157                 70.29    1677
disk_stuck_h3（卡住盘 0） 96 / 96 / 1456 / 1208 / 1456（G2SW 598 / 598）            9744 / 5696 / 1456 / 1177 / 0                    32.59    1443
```

`steady`, `torn_newest`, `disk_loss`, `admin_rollback`, `near_full`, `cost_growth`, `rollback_first_publish` have 0 samples where x is positive; ⓐ is identical to G2SW on `torn_newest` (peak 81), `disk_loss` (94), `rollback_first_publish` (611).
RGall and G2SW_ringgate are identical across six of eight classes in this table; `drain_failure` (samples with positive x: 6680 vs 6656) and `disk_stuck_h3` (5696 vs 5704, Σx 3881568 vs 3685624) differ slightly, source of difference uninvestigated individually. Commands for extracting both tables are in "Counting Methods" subsection below; `verdict_f_cost` has 86 rows total (one row per (class, arm), counted via `grep -c 'name=verdict_f_cost'`), table extracts only fields for RGall and G2SW arms.

ⓒ Scale (pinned blocks per seed = `mark*_pinned_sum` in `f_stuck_growth` / `f_broken_slot_growth` / `f_broken_slot_h1_growth` ÷ 8 seeds; revolution k = moment when stuck or broken slot begins + k × 3S txgs):

```text
世界                         S   占用     RGall 刻度 1 / 2 / 4       s(2→4)   RGall 判       G2SW 刻度 1 / 2 / 4   G2SW 判
disk_stuck（卡住盘 1）       4   0.30、0.70  234 / 338 / 546         1.00     没有上界       91 / 91 / 91          有上界
disk_stuck（卡住盘 1）       16  0.30     1014 / 1430 / 2262         1.00     没有上界       403 / 403 / 403       有上界
disk_stuck（卡住盘 1）       16  0.70     1014 / 1206 / 没走到       na       没走到（8 个种子全卡死）  403 / 403 / 403   有上界
disk_stuck_h3（卡住盘 0）    4   0.30、0.70  208 / 260 / 364         1.00     没有上界       39 / 39 / 39          有上界
disk_stuck_h3（卡住盘 0）    16  0.30     832 / 1040 / 1456          1.00     没有上界       195 / 195 / 195       有上界
disk_stuck_h3（卡住盘 0）    16  0.70     832 / 1040 / 1208          1.00     没有上界       195 / 195 / 195       有上界
broken_slot                  4   c₀ = 1、5   247 / 390 / 676         1.00     没有上界       130 / 130 / 130       有上界
broken_slot                  16  c₀ = 1   715 / 176 / 229           0.04     有上界         598 / 598 / 598       有上界
broken_slot                  16  c₀ = 5   715 / 202 / 314           0.09     有上界         598 / 598 / 598       有上界
broken_slot H1               4   c₀ = 1、5   273 / 416 / 702         1.00     没有上界       G2SW 四行全判有上界
broken_slot H1               16  c₀ = 1   741 / 190 / 254           0.05     有上界
broken_slot H1               16  c₀ = 5   741 / 236 / 340           0.08     有上界
```
```text
E7RESULT name=verdict_f_reach_summary positive_samples_total=53736 cf_never_reached=0
E7RESULT name=verdict_f_compare_total rows_compared=610 rows_with_a_count_changed=228 rows_with_a_judgement_changed=210 jia_count_basis_only=228 yi_walk_changed=0 bing_floor_gap_never_positive=0 rows_without_a_first_rerun_counterpart=402
E7RESULT name=verdict_f_crit1_compare rows_compared=24 rows_with_a_differing_line=0 rgall_crit1_rows_not_holding=0
E7RESULT name=verdict_f_direction_total checks_old_set_strictly_larger=53392 checks_new_set_strictly_larger=0 system_configuration_below_root_floor_events=0
E7RESULT name=verdict_f_window arm=G2SW_tightened_states_window rows=123 rows_with_fake_candidates=10 rows_with_unrecoverable=7 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_f_window arm=G2SW_ringgate rows=123 rows_with_fake_candidates=0 rows_with_unrecoverable=0 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_f_window arm=RGall_ring_all rows=123 rows_with_fake_candidates=0 rows_with_unrecoverable=0 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_f_window arm=RGall_Q_ring_all rows=123 rows_with_fake_candidates=0 rows_with_unrecoverable=0 judged_rows_with_false_enospc=0 rollback_first_publish_rows=3 rollback_first_publish_rows_with_fake_candidates=3 rollback_first_publish_rows_with_unrecoverable=3 crit1_push_rows_not_holding=0
E7RESULT name=verdict_f_window_short arm=RGall_short rows=2 rows_where_the_detector_saw_zero_fake_candidates=0
E7RESULT name=verdict_f_window_short arm=RGall_Q_short rows=2 rows_where_the_detector_saw_zero_fake_candidates=0
E7RESULT name=verdict_f_rgall_readings rows_compared=175 rows_with_a_judgement_different_between_p_and_q=0 disk_without_valid_root_events_p=0 disk_without_valid_root_events_q=0
E7RESULT name=verdict_f_growth line=f_broken_slot_growth arm=G2SW_tightened_states_window rows=4 bounded=4 unbounded=0 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_growth arm=G2SW_ringgate rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_growth arm=RGall_ring_all rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_growth arm=RGall_Q_ring_all rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_h1_growth arm=G2SW_tightened_states_window rows=4 bounded=4 unbounded=0 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_h1_growth arm=G2SW_ringgate rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_h1_growth arm=RGall_ring_all rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_broken_slot_h1_growth arm=RGall_Q_ring_all rows=4 bounded=2 unbounded=2 not_judged=0
E7RESULT name=verdict_f_growth line=f_stuck_growth arm=RGall_ring_all rows=8 bounded=0 unbounded=7 not_judged=1
E7RESULT name=verdict_f_growth line=f_stuck_growth arm=RGall_Q_ring_all rows=8 bounded=0 unbounded=7 not_judged=1
E7RESULT name=verdict_f_growth line=f_stuck_growth arm=G2SW_tightened_states_window rows=8 bounded=8 unbounded=0 not_judged=0
E7RESULT name=verdict_f_growth line=f_stuck_growth arm=G2SW_ringgate rows=8 bounded=0 unbounded=7 not_judged=1
E7RESULT name=verdict_f_growth_geometry point=h3 arm=RGall_ring_all rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_growth_geometry point=h3 arm=RGall_Q_ring_all rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_growth_geometry point=h3 arm=G2SW_tightened_states_window rows_compared=4 rows_with_a_different_bound=0
E7RESULT name=verdict_f_growth_geometry point=h3 arm=G2SW_ringgate rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_growth_geometry point=fill arm=RGall_ring_all rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_growth_geometry point=fill arm=RGall_Q_ring_all rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_growth_geometry point=fill arm=G2SW_tightened_states_window rows_compared=4 rows_with_a_different_bound=0
E7RESULT name=verdict_f_growth_geometry point=fill arm=G2SW_ringgate rows_compared=4 rows_with_a_different_bound=1
E7RESULT name=verdict_f_stale rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions=48 rows_in_the_stuck_worlds_with_a_root_beyond_two_revolutions=144
E7RESULT name=verdict_f_clauses f1_cf_never_reached=0 f3_rgall_short_rows_where_the_detector_saw_zero_fake_candidates=0 f4_window_arm_unbounded_disk_stuck_rows=0 f5_rgall_rows_with_different_readings=0 f7_growth_rows_with_a_different_bound_h3=3 f7_growth_rows_with_a_different_bound_fill=3 f7_growth_rows_with_a_different_bound_h1=0 f7_stuck_rows_with_a_different_judgement_between_fills=0 f8_rgall_false_enospc_rows_outside_the_judged_range=12 f9_rgall_rows_with_fakes_unrecoverable_or_judged_false_enospc=0 f10_rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions=48 f11_rgall_broken_slot_unbounded_rows=8 v5_changed_rows_with_a_never_positive_floor_gap=0 v6_checks_new_set_strictly_larger=0 v7_system_configuration_below_root_floor_events=0 v8_window_arm_rows_with_a_nonzero_extra_pinned=0 checks_old_set_strictly_larger=53392 changed_rows_total=228 positive_gap_samples_total=53736
```
In `disk_stuck`, s(1→2) and s(2→4) on RGall are both 1.00 (min and max across 8 seeds identical): every block released after stuck remains in the pinned set. On `broken_slot` S = 16, Scale 2 is lower than Scale 1 (715 → 176), reason uninvestigated individually (unrequired by registration); with opposing verdicts across the two S values, "bounded status on `broken_slot`" cannot conclude independent of S.

Verdict rows transcribed verbatim from `e139-tightened-floor-2026-10-05.out` (297 rows in `grep -n 'name=verdict' research/results/e139-tightened-floor-2026-10-05.out`, where original 40 rows of `name=verdict`, `name=verdict_candidate`, `name=verdict_window*` matched verbatim with Rerun 1 artifact in V2 command, not re-copied; `verdict_f_*` total 257 rows, copying only those used by "Readings" table):
```bash
# Line count self-check: row count per f_ class (rows per arm: f_cell 6 tested arms 21 rows each plus 5 _short arms 2 rows each = 136, f_activation 6 × 60 = 360, f_activation_h2 6 × 36 = 216, f_disk_stuck and f_disk_stuck_h3 4 × 12 = 48 each, f_stuck_marks 32 × 8 = 256)
grep '^E7RESULT' results/e139-tightened-floor-2026-10-05.out | awk '{print $2}' | sort | uniq -c | grep ' name=f_'
# f_ rows without arm= field (except f_config)
grep '^E7RESULT name=f_' results/e139-tightened-floor-2026-10-05.out | grep -v '^E7RESULT name=f_config' | grep -vc ' arm='
```
**Counting Methods** (working directory `research`; artifact `results/e139-tightened-floor-2026-10-05.out`):
```text
    360 name=f_activation
    216 name=f_activation_h2
   1012 name=f_attribution
     24 name=f_broken_slot
     24 name=f_broken_slot_growth
     24 name=f_broken_slot_h1
     24 name=f_broken_slot_h1_growth
    136 name=f_cell
    228 name=f_changed_row
      1 name=f_config
     48 name=f_cost_growth
     36 name=f_crit1_push
     24 name=f_detail_activation
     48 name=f_disk_stuck
     48 name=f_disk_stuck_h3
     72 name=f_drain_failure
     36 name=f_near_full
     74 name=f_nonzero_row
     48 name=f_stale_row
     32 name=f_stuck_growth
    256 name=f_stuck_marks
   1012 name=f_trajectory
0
```
```bash
# Line-by-line comparison of changed rows for fake candidates old -> new, unrecoverable old -> new (summed by line name and arm)
grep '^E7RESULT name=f_changed_row' results/e139-tightened-floor-2026-10-05.out | awk '{l="";a="";fo=0;fn=0;uo=0;un=0;for(i=3;i<=NF;i++){split($i,kv,"=");if(kv[1]=="line")l=kv[2];if(kv[1]=="arm")a=kv[2];if(kv[1]=="fakes_old_new"){split(kv[2],x,"/");fo=x[1];fn=x[2]};if(kv[1]=="unrecoverable_old_new"){split(kv[2],x,"/");uo=x[1];un=x[2]}};key=l" "a;n[key]++;FO[key]+=fo;FN[key]+=fn;UO[key]+=uo;UN[key]+=un}END{for(q in n)print q,"rows="n[q],"fakes_old="FO[q],"fakes_new="FN[q],"unrec_old="UO[q],"unrec_new="UN[q]}' | sort
```
`f_trajectory` and `f_attribution` have 1012 rows each = one row per result row except `f_crit1_push` and `f_detail_activation` (136 + 360 + 36 + 72 + 24 + 48 + 216 + 24 + 48 + 48); `f_changed_row` 228 rows = `rows_with_a_count_changed` of `verdict_f_compare_total`; `f_stale_row` 48 rows = 48 of `verdict_f_stale`.
```text
f_activation G2SW_L_ring_lag rows=46 fakes_old=4816 fakes_new=200 unrec_old=96 unrec_new=96
f_activation G2SW_Q_disk_hold rows=46 fakes_old=4816 fakes_new=200 unrec_old=96 unrec_new=96
f_activation G2SW_tightened_states_window rows=46 fakes_old=4816 fakes_new=200 unrec_old=96 unrec_new=96
f_activation_h2 G2SW_L_ring_lag rows=30 fakes_old=1295 fakes_new=0 unrec_old=0 unrec_new=0
f_activation_h2 G2SW_Q_disk_hold rows=30 fakes_old=1295 fakes_new=0 unrec_old=0 unrec_new=0
f_activation_h2 G2SW_tightened_states_window rows=30 fakes_old=1295 fakes_new=0 unrec_old=0 unrec_new=0
```
```bash
# Total fake candidates, unrecoverable, false ENOSPC (excluding rollback_first_publish) per result row class for RGall, RGall_Q, G2SW, G2SW_ringgate: listing only rows not all zero
for a in RGall_ring_all RGall_Q_ring_all G2SW_tightened_states_window G2SW_ringgate; do for n in f_cell f_activation f_near_full f_drain_failure f_broken_slot f_cost_growth f_activation_h2 f_broken_slot_h1 f_disk_stuck f_disk_stuck_h3; do grep "^E7RESULT name=$n arm=$a " results/e139-tightened-floor-2026-10-05.out | grep -v 'world=rollback_first_publish' | awk -v a=$a -v n=$n '{rows++; for(i=3;i<=NF;i++){split($i,kv,"="); if(kv[1]=="fakes")f+=kv[2]; if(kv[1]=="unrecoverable")u+=kv[2]; if(kv[1]=="false_enospc_arm"&&kv[2]!="na")e+=kv[2]; if(kv[1]=="true_enospc")t+=kv[2]}} END{printf "%s %s rows=%d fakes=%d unrecoverable=%d false_enospc_arm=%d true_enospc=%d\n",a,n,rows,f,u,e,t}'; done; done | grep -v 'fakes=0 unrecoverable=0 false_enospc_arm=0'
```
```text
RGall_ring_all f_cost_growth rows=8 fakes=0 unrecoverable=0 false_enospc_arm=64 true_enospc=240
RGall_ring_all f_disk_stuck rows=12 fakes=0 unrecoverable=0 false_enospc_arm=96 true_enospc=0
RGall_ring_all f_disk_stuck_h3 rows=12 fakes=0 unrecoverable=0 false_enospc_arm=72 true_enospc=0
RGall_Q_ring_all f_cost_growth rows=8 fakes=0 unrecoverable=0 false_enospc_arm=64 true_enospc=240
RGall_Q_ring_all f_disk_stuck rows=12 fakes=0 unrecoverable=0 false_enospc_arm=96 true_enospc=0
RGall_Q_ring_all f_disk_stuck_h3 rows=12 fakes=0 unrecoverable=0 false_enospc_arm=72 true_enospc=0
G2SW_tightened_states_window f_activation rows=60 fakes=200 unrecoverable=96 false_enospc_arm=0 true_enospc=0
G2SW_tightened_states_window f_cost_growth rows=8 fakes=0 unrecoverable=0 false_enospc_arm=64 true_enospc=240
G2SW_tightened_states_window f_disk_stuck rows=12 fakes=320 unrecoverable=32 false_enospc_arm=0 true_enospc=0
G2SW_tightened_states_window f_disk_stuck_h3 rows=12 fakes=640 unrecoverable=32 false_enospc_arm=0 true_enospc=0
G2SW_ringgate f_cost_growth rows=8 fakes=0 unrecoverable=0 false_enospc_arm=64 true_enospc=240
G2SW_ringgate f_disk_stuck rows=12 fakes=0 unrecoverable=0 false_enospc_arm=96 true_enospc=0
G2SW_ringgate f_disk_stuck_h3 rows=12 fakes=0 unrecoverable=0 false_enospc_arm=72 true_enospc=0
```
`f_cost_growth`'s `false_enospc_arm=64` represents two rows of overhead rising to c_max + 3 at 32 each out of 8 rows; rows with `within_declared=1` are 0 (unjudged cells); RGall's `f_cell` has no row not entirely zero, hence does not appear in "Total fake candidates, unrecoverable, false ENOSPC per result row class" output; 3 rows of `rollback_first_publish` reported separately per original registration (`grep '^E7RESULT name=f_nonzero_row' … | grep 'world=rollback_first_publish'`).
```bash
# Cost table (a, b): taking verdict_f_cost rows of RGall and G2SW arms
grep '^E7RESULT name=verdict_f_cost' results/e139-tightened-floor-2026-10-05.out | grep -e 'arm=G2SW_tightened' -e 'arm=RGall_ring' | awk '{c="";a="";delete v;for(i=3;i<=NF;i++){split($i,kv,"=");v[kv[1]]=kv[2];if(kv[1]=="class")c=kv[2];if(kv[1]=="arm")a=kv[2]}; r=(v["holes_samples"]+0>0 && v["hole_slots_sum_at_holes"]+0>0)? sprintf("%.2f",v["extra_pinned_sum_at_holes"]/v["hole_slots_sum_at_holes"]):"na"; print c,a,"sum_x_over_sum_h="r,"max_x_per_h_x100="v["extra_pinned_per_hole_max_x100"]}' | sort
# Scale (c): pinned blocks per seed = mark*_pinned_sum / 8
grep -E '^E7RESULT name=f_(stuck_growth|broken_slot_growth|broken_slot_h1_growth) arm=(RGall_ring_all|G2SW_tightened_states_window) ' results/e139-tightened-floor-2026-10-05.out | awk '{for(i=3;i<=NF;i++){split($i,kv,"=");if(kv[1]=="mark1_pinned_sum")k1=kv[2]/8;if(kv[1]=="mark2_pinned_sum")k2=kv[2]/8;if(kv[1]=="mark4_pinned_sum")k4=kv[2]/8;if(kv[1]=="bound")b=kv[2]};print $2,$3,$4,$5,$6,"marks="k1"/"k2"/"k4,b}' | sort
```
```bash
# Maximum publish count per admission (rows with failures <= 2 separated from failures 3) and stall counts outside disk_stuck
for a in G2SW_tightened_states_window G2SW_ringgate RGall_ring_all RGall_Q_ring_all; do grep -E "^E7RESULT name=f_[a-z0-9_]+ arm=$a " results/e139-tightened-floor-2026-10-05.out | grep ' max_publications=' | awk -v a=$a '{fl="";mp=0;for(i=3;i<=NF;i++){split($i,kv,"=");if(kv[1]=="failures")fl=kv[2];if(kv[1]=="max_publications")mp=kv[2]+0}; if(fl==""||fl+0<=2){if(mp>w)w=mp}else{if(mp>b)b=mp}; n++}END{print a,"rows_with_max_publications="n,"max_within_k_tol(failures<=2 or no failures field)="w,"max_failures_3="b+0}'; done
for a in G2SW_tightened_states_window G2SW_ringgate RGall_ring_all RGall_Q_ring_all; do printf '%s stalls_outside_disk_stuck=' $a; grep -E "^E7RESULT name=f_[a-z0-9_]+ arm=$a " results/e139-tightened-floor-2026-10-05.out | grep -v -e 'name=f_disk_stuck' -e 'name=f_trajectory' -e 'name=f_attribution' -e 'name=f_stuck' | grep -o ' stalls=[0-9]*' | awk -F= '{s+=$2}END{print s+0}'; done
```
```text
G2SW_tightened_states_window rows_with_max_publications=40 max_within_k_tol(failures<=2 or no failures field)=6 max_failures_3=5
G2SW_ringgate rows_with_max_publications=40 max_within_k_tol(failures<=2 or no failures field)=7 max_failures_3=7
RGall_ring_all rows_with_max_publications=40 max_within_k_tol(failures<=2 or no failures field)=7 max_failures_3=7
RGall_Q_ring_all rows_with_max_publications=40 max_within_k_tol(failures<=2 or no failures field)=7 max_failures_3=7
G2SW_tightened_states_window stalls_outside_disk_stuck=0
G2SW_ringgate stalls_outside_disk_stuck=0
RGall_ring_all stalls_outside_disk_stuck=0
RGall_Q_ring_all stalls_outside_disk_stuck=0
```
In the first table, columns rec and x are raw values of `pinned_after_recovery_*` and `extra_pinned_vs_window_gate_*` fields in `verdict_f_cost` rows; Σx÷Σh and max(x÷h) are calculated by the first command (`max_x_per_h_x100` divided by 100); classes with 0 samples are omitted.
In the second table, fields of `f_cost_growth` for RGall are verbatim rows; in the third command, `max_publications` is separated by failures <= 2 vs failures 3.
RGall stalls count 0 in all judged rows; in `f_disk_stuck`, occupancy 0.70 has 8 seeds stalling (reported as unjudged).

**Summary of Failure Clauses and Invalidation Clauses** (registration sections 10 and 11, each judged by artifact): F1 not triggered; F2 triggered (row 1 flipped); F3 not triggered; F4 not triggered; F5 not triggered; F6 absent (K2r, K10a unit tests pass); F7 triggered literally (RGall's three-part verdict has 0 rows changed, bounded-or-not has 1 row "unreached" vs "unbounded", G2SW has changed rows in H2, H3); F8 triggered (12 out-of-range rows); F9 not triggered (0 rows in RGall-judged cells); F10 triggered (48 rows in `skip_then_raise_r3`, S = 4); F11 triggered (RGall judged "unbounded" on `broken_slot` and H1 S = 4, 8 rows); V1–V8 not triggered.

**Unit Tests and Mutations**: 55 unit tests (`tests::` 21, `window_tests::` 15, `second_rerun_tests::` 19) all pass; additionally 1 `#[ignore]` manual rehearse of all `disk_stuck` combinations was run. The apparatus test binary is run directly (wrapped in 100M memory limit):

```text
$ grep -c '^test .* ok$' <单测输出>
55
$ grep '^test result' <单测输出>
test result: ok. 55 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 3.15s
```

Mutation table `scripts/mutation/tables/e139_tightened_floor.tsv` has 42 rows (subsequently deduplicated by removing M16 which was completely identical to M11 in three aspects, current table 41 rows; original 26 plus 16 from this run: MF1–MF5, MR1–MR6, MS1–MS2, MC1–MC3); `mutate.sh` was run in three blocks (14 each, with independent baselines and restores) in the foreground, with all three blocks showing "baseline: all green" and "restored, baseline still all green": 42 caught, 0 invalid, 0 non-failing, memory cap hit 0, timeout 0, unschedulable 0, squeezed out by overall limit 0. The conclusion reports only "42 mutations all caught", not claiming zero blind spots; mutations cover only model and sampling code; row assembly and verdict lines in artifacts (`emit_*`) have no mutations, verified instead via smoke tests in unit tests, V2, and the commands in the "Counting Methods" section. There is no record of running the entire table in a single run in the 2026-10-05 round.

**Comparison Against D16 (Publish Semantics) Settled Item 1** (providing comparison only, not modifying decision text; whether and how to change is left to main agent via tripartite process):

- The 2026-10-05 settlement of settled item 1 already defined the reclamation threshold and five other locations as the oldest valid root in the ring (RGall, settled by user on 2026-10-04), with that sentence stating "form adopts Arm G2S; after the reclamation threshold rolls back to the oldest valid root in the ring, the apparatus candidate floor has not yet incorporated F from system configuration, so this sentence still does not match what runs in the apparatus verbatim, with debt recording that E139 (Tightened Form of Per-Disk Rollback Floor) needs a rerun". In this run, candidate floors in `f_` rows already incorporate system configuration F, and RGall is G2S rolled back to the whole ring according to clauses plus clause-form floor: the half-sentence "candidate floor has not yet incorporated F from system configuration" is obsolete on `f_` rows; whether "does not match what runs in the apparatus verbatim" should be retracted depends on the items below.
- Basis paragraph item 1 "Arm G2S across all fault worlds ... zero fake candidates, zero unrecoverable, false ENOSPC is 0 per cell under its own `df`, measured maximum publish count per admission 7 <= 8, no stall under pre-run reservation pool": for RGall across the 13 classes of worlds from Rerun 1 and `disk_stuck`, fake candidates 0, unrecoverable 0, false ENOSPC in judged cells 0 (`verdict_f_window`), maximum publishes per admission 7 <= 8 (publish count command in the "Counting Methods" section; maximums for rows with failures <= 2 and rows with failures = 3 are both 7), stalls outside `disk_stuck` 0, all hold; on `disk_stuck` (out of range: root slot write failures in a single handling far exceed k_tol = 2) at S = 16, occupancy 0.70 across 6 rows RGall shows false ENOSPC 32 / 24 times and all 8 seeds stall, which is an out-of-range observation (F8). RGall's positive control shows the threshold set high (F3 not triggered).
- That clause on "cost of taking oldest valid root in ring for reclamation threshold" states "when one disk continually fails to write roots, pinned units grow linearly with publishes without upper bound": on apparatus `disk_stuck`, RGall judges "unbounded" across 7 rows (s(2->4) = 1.00), and 1 row at S = 16, occupancy 0.70 stalls before scale 4, whereas under the same parameters G2SW (window plus clause-form floor) has upper bounds across all 8 rows, matching direction; on `broken_slot` and H1, RGall also judges "unbounded" at S = 4, but judges "bounded" at S = 16 (0.04–0.09), which was not documented in the decision text. Numbers in that clause (50 / 80 / 140 / 260, 893 vs 583, 166 vs 26) were measured by both legs under their respective bases; the apparatus basis differs, so they are not compared.
- Regarding "candidate floor adopting old style vs clause form": the candidate set under the old floor is no smaller than that under clause form (53,392 times strictly greater, 0 times strictly smaller); `f_` rows reduce G2SW's fake candidates from 46 rows to 6 rows, indicating that most of the "46 rows of fake candidates" for G2SW in Rerun 1 were artifacts produced by the apparatus candidate floor adopting old style, pointing in the same direction as "⑥ alone can produce fake candidates" in verdict section W2; the remaining 6 rows (`skip_then_raise_r2` / `r3`, S = 4) and 3 rows of unrecoverable remain, while G2SW_ringgate and RGall remain at 0.
- Readings P and Q did not diverge on this artifact (`disk_without_valid_root_events` is 0 on both sides); the cell "no valid root exists across the entire ring of some surviving disk" is unwritten in clauses and unreached by the artifact, so this cell was not judged on the basis of this artifact.

**What Rerun 2 Cannot Answer**:

1. Counting model (same as Rerun 1): no I/O; roots in windows are taken per apparatus `valid_roots`; differences ①②⑤⑦⑧ from registration section 11 remain as before, with new additions: ⑨ apparatus system configuration write does not fail, ⑩ old floor takes maximum F carried by all self-proving roots on surviving disks, ⑪ apparatus RGall does not yet exist in `crates/`, ⑫ apparatus does not model normal unmount, ⑬ in `disk_stuck` what gets stuck is only root slot writes of that disk (apparatus's own fault model; `crates/` does not inject structure-partitioned partial write failures).
2. Clause-form floor is traversed only in `activation_faults`, `activation_crash_no_loss`, `skip_then_raise`, and a portion of `disk_stuck_h3` (g > 0); on the other eleven classes of worlds, row 1 "count unchanged" means unreached, not "unchanged even after alteration".
3. On `disk_stuck`, RGall judging "unbounded" is a cell deduced by definition after the table in registration section 6 (the clause "upper bound on raising F" prevents F from exceeding the newest valid root on the stuck disk, so the threshold cannot exceed it either); the artifact merely gave numbers under apparatus warmup, tight disk, and crash wrap-up, rather than being discovered by criteria; G2SW judging "bounded" on the same parameter set merely proves this criterion discriminates between the two arms.
4. In worlds of the `skip_then_raise_r3`, S = 4 class, old roots in holes survive for more than two cycles (F10), and bounded-or-not for this class was not measured; their `f_` row verdicts are read according to original criteria, but the half stating "pinned units in other worlds are bounded" is empty for this class.
5. Measurement bases of ⓐⓑ differ from attacker leg's "allocated after mount" and forward leg's "block count with release generation falling between two thresholds" (sampling unit of ⓐⓑ is one recovery, while that of the two legs is each root write to disk), so numbers are not compared.
6. `drain_failure` with 3 failures, as well as `disk_stuck` false ENOSPC and minimum reservation violations, are out-of-range cells where numbers are reported only; 3 rows of `rollback_first_publish` are reported without verdict per original registration (C281 (First Publish After Rollback May Reuse Blocks Still Referenced by Abandoned Timeline)).
7. The reservation sweep family was not added for the new arms, marked "unrun after sufficient for judgment"; unit test K15 pinning "RGall_Q with CF disabled is field-by-field equal to original G2S" is an internal equivalence archive within the apparatus, not a differential against `crates/`.
8. Readings P and Q did not diverge on this artifact (see the row "RGall and RGall_Q (Readings P and Q)" in the "Readings" table); this does not indicate that the unwritten cell in clauses can remain unwritten.

### This Is a Counting Model, Not an Implementation

No I/O, random source is hardcoded seed ⇒ running N times is necessarily byte-for-byte identical. Evidence strength derives from **55 unit tests + 42 mutations all caught** (subsequently deduplicated by deleting M16 which was completely identical to M11, current table has 41 rows) (`scripts/mutation/tables/e139_tightened_floor.tsv`; at first run 21 and 16, Rerun 1 added 15 and 10, Rerun 2 added 19 and 16, see "Rerun 1" and "Rerun 2");
mutation M13 (upper bound under state-count reading counting empty roots too) did not turn red on any unit test in Round 1, classified per `.claude/rules/mutation-sampling.md` as sampling point insensitive — original unit tests sampled only on rings without push-empty, where both upper bounds had identical values;
caught after adding unit test `forced_pressure_keeps_four_states_for_states_arm` that raises once more after push-empty roots enter ring. Version 1 of M14 caused tested code not to terminate (region numbers never matching), caught after changing to "failure strikes region 2".
Two cross-apparatus gates (locked block count of E123 (Trade-off Costs Between K and Rollback Depth), N − K of E92 (Worst-Case Block Reuse Delay Requirements)), sharing identical values with E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty).

### Arms

| Arm | Description | Role |
|---|---|---|
| A `ring_persisted` | E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) verbatim: reallocatable ⟺ release generation <= oldest valid root persisted in ring; acceptance does not charge by worst-case residue | Control: target for detector across three worlds where Round 3 judged it loses |
| C `k_runtime`, D `floor_txg` | E135 (Dynamic Rollback Floor) verbatim | Control: cross-apparatus gate; activation calculated by txg |
| G `floor_per_disk` | E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) Arm G verbatim (upper bound by root count, B = 7, formula by c₀) | Control: target for acceptance charge clause |
| G_txgcap | Upper bound counts 4 generations by txg, ignoring disks | Control: target for "taking cap per disk" half (E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) judged "unverified") |
| **G2R** | Per-disk rollback floor + acceptance charging by worst-case residue + B_R = 7 + 2 × 2 = 11 + reservation pool 2 × 5 + 10 c_max, residue 5 + 9 c_max + upper bound min(newest persistent valid root on each disk, 4th newest persistent valid root) | Under test: read by root count |
| **G2S** | Same as G2R, but B_S = 4 + 2 × 2 = 8, reservation pool 2 × 5 + 7 c_max, residue 5 + 6 c_max, upper bound takes 4th newest **non-empty** valid root, `df` further deducts lag (released blocks with release gen > max(4th newest non-empty root, oldest valid root in ring) and not released by empty publishes) | Under test: read by state count |
| G2R_mount | Same as G2R, formula by mount-time c₀ | Control: target for c_max clause |

Allocatable amount and `df` for both tightened arms record currently active metadata back into reservation pool (reservation pool was reserved precisely for it); (c₀, c_max) ∈ {(1, 5), (5, 10)}.

### Verdicts (Per Criteria Hardcoded Before Run)

Transcribed verbatim from `e139-tightened-floor-2026-09-12.out`:

```text
E7RESULT name=verdict crit1_holds=1 crit2_a_drain_false_enospc=3424 crit2_a_broken_false_or_stall=64 crit2_g_near_full_wad_cost5=48 crit2_g2r_mount_growth_false_or_stall=32 crit2_d_activation_fakes=584 crit2_gtxgcap_skip_retention=12576 crit2_a_raw_false_enospc=144
E7RESULT name=verdict_candidate arm=G2R_tightened_roots crit3_fakes=0 crit3_unrecoverable=0 crit4_false_enospc_cells=0 crit5_write_after_delete_cells=0 crit5_bound_exceeded_cells=0 crit6_stall_cells=0 crit7_retention=0 crit8_max_publications_within_tolerance=11 beyond_tolerance_max_publications=9 bound=11
E7RESULT name=verdict_candidate arm=G2S_tightened_states crit3_fakes=0 crit3_unrecoverable=0 crit4_false_enospc_cells=0 crit5_write_after_delete_cells=26 crit5_bound_exceeded_cells=0 crit6_stall_cells=0 crit7_retention=24 crit8_max_publications_within_tolerance=7 beyond_tolerance_max_publications=7 bound=8
E7RESULT name=crit6_min_no_stall_reserve g2r_pair0=54 g2r_pair1=104 g2s_pair0=39 g2s_pair1=74
E7RESULT name=crit9_c281 A_ring_persisted=72 C_k_runtime=72 D_floor_txg=72 G_floor_per_disk=72 G_txg_cap=72 G2R_tightened_roots=72 G2S_tightened_states=72 G2R_mount=72
```

| # | Criterion | Result |
|---|---|---|
| 1 | Apparatus vs pre-run formula | All matched: two cross-apparatus gates; A uses N times, withholding 5 + (N − 2) c; G and G2R across three placement positions use 6 or 7 times, withholding at most 5 + 5c; **G2S uses 3 or 4 times, withholding at most 5 + 2c outside lag, lag 39**; at S = 1 all four arms use 3 times (artifact `crit1_push` rows all `holds=1`) |
| 2 | Discriminative power | All six targets caught: A write failure during push-empty produces false ENOSPC 3424 times, broken slot yields 64 times false or stall; G at c₀ = 5 on near-full disk fails deletion-then-write 48 times; G2R_mount after overhead rising to c_max yields 32 times false or stall; D produces 584 fake candidates in activation interval; **G_txgcap across three cells skipping numbers then raising F exceeds newest root on disk 1 by 4192 times each** — the "taking cap per disk" half judged "unverified" in E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) measured for the first time, G2R / G2S are 0 in same world |
| 3 | Correctness | G2R, G2S have zero fake candidates, zero unrecoverable across all fault worlds (including crash in activation interval without losing disks, skipping numbers then raising F, 1..3 write failures during push-empty, broken slots, growing overhead) |
| 4 | Item 1: No false ENOSPC allowed | G2R, G2S are 0 in every cell (per respective `df`; raw and drain columns of A are non-zero, proving detector turns red) |
| 5 | Item 2: Usable within bounded steps after deletion | **G2R passes every cell**, publish count never exceeds B_R in any cell. **G2S fails deletion-then-write across 26 cells** — near-full disk main sweep and 8 cells of elevated overhead fail in first 3 windows after full disk (windows 1..3 at c₀ = 5, windows 2..3 at c₀ = 1, identical across all 24 seeds, artifact `near_full_failure_windows` rows); 18 cells of reservation sweep in artifact lack window numbers; post-run test measured 9 cells of c₀ = 5 start at 1, end at 3; among 9 cells of c₀ = 1, four cells with reserve = formula + 2..5 have **end window 6**, others end at 3 (`e139-tightened-floor-2026-09-12-reserve-sweep-windows.out`); zero failures from end window up to window 403. Mechanism: blocks deleted this time require another 3 user-state-changing windows before being returned, which is the literal deduction of "retaining 4 rollbackable states"; `df_S` margin minus it yields negative allocatable amount in initial windows, causing subsequent writes to report ENOSPC (true ENOSPC, not false) |
| 6 | Stalls | Both arms 0 when reservation pool >= formula; minimum non-stalling reservation pool in sweep: G2R 54 / 104, G2S 39 / 74 (overhead pairs (1, 5) and (5, 10)); G2S formula is 45 / 80, 39 / 74 is below formula, indicating margin exists |
| 7 | Minimum retention | G2R 0. G2S is 0 in cells with S in {4, 16} and <= 2 failures; 24 times in the two cells with S = 1 and overhead rising to c_max (only 1 non-empty valid root exists, cannot obtain the 4th newest), all concentrated in S = 1, which was registered before run as "outside ring depth lower limit"; 0 across near-full disk cells (average 10–11 non-empty, at least 4, satisfying 4 generations), but among released blocks on near-full disk 34–35 blocks were released by empty publishes, which by definition cannot count as lag, with lag actually deducting 39 blocks |
| 8 | Arithmetic of bounds | G2R measured maximum within tolerance is 11 = B_R (2 failures, c₀ = 1, c_max = 5, `crit8_max_publications_within_tolerance=11`), measured maximum beyond tolerance is 9 <= 11; G2S measured maximum within tolerance is 7 <= B_S = 8 (1 failure, c₀ = 5, c_max = 10), measured maximum beyond tolerance is 7 <= 8 (2 failures) |
| 9 | C281 (First publish after rollback reuses blocks still referenced by abandoned timeline) | 72 across all eight arms, present in every arm |

**Difference between two readings manifested as numbers**: reading by root count, after push-empty candidate set retains only that single state of this deletion (near-full disk cells `min_states=1`, root count no fewer than 5); reading by state count in near-full disk deletion-creation cycle is never fewer than 4 at any time (`min_states=4`; in fault worlds 1..2, where minimum retention check judges F and upper bound, not counting states).
```text
E7RESULT name=near_full_failure_windows arm=G2S_tightened_states s=4 c0=1 seeds_with_failure=24 first_windows=2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2 last_windows=3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3
E7RESULT name=near_full_failure_windows arm=G2S_tightened_states s=4 c0=5 seeds_with_failure=24 first_windows=1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1 last_windows=3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3
E7RESULT name=near_full_failure_windows arm=G2S_tightened_states s=16 c0=1 seeds_with_failure=24 first_windows=2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2/2 last_windows=3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3
E7RESULT name=near_full_failure_windows arm=G2S_tightened_states s=16 c0=5 seeds_with_failure=24 first_windows=1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1 last_windows=3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3/3
E7RESULT name=drain_failure arm=G2S_tightened_states s=4 c0=5 c_max=10 failures=3 cases=160 runs_with_false_enospc=0 runs_with_write_after_delete_failed=8 false_enospc_arm=0 false_enospc_raw=24 write_after_delete_failed=24 max_publications=7 bound=8 stalls=0 fakes=0 unrecoverable=0 retention_violations=24 injected=480
```


**Differences between the two readings crystallized into numbers**: reading by root count, after pushing empty the candidate set contains only that single state of this deletion (on near-full disk cells `min_states=1`, `avg_states` 3.2–4.3, with the mean originating from persisted roots accumulated during reservation-pool push-empty), and writing after deletion will not step over this state to recognize old roots on an abandoned timeline, but this state **has no non-empty root to self-prove it** (`avg_nonempty` 2.2–4.3, originating from empty roots entering the ring during push-empty), with unrecoverable being 0 across cells judged for it; reading by state count, the upper bound directly takes the "4th newest non-empty valid root", so the candidate set contains not only this single state but also old states on the abandoned timeline (`min_states=4`, `avg_states` 10–11), and writing after deletion recognizes back the old states (criterion 5 failed 26 times), requiring lag to withhold these blocks (criterion 7 withheld 24 times in non-reservation-sweep cells, with `avg_lag` 34–35 blocks and `max_lag` 44–52 blocks on near-full disk).

### Costs (Transcribed Verbatim from `e139-tightened-floor-2026-09-12.out`, `near_full` S = 16 Cells, 24 Seeds Total)

```text
E7RESULT name=near_full arm=A_ring_persisted s=16 c0=1 c_max=5 reserve=57 residue=51 hidden_blocks=108 hidden_bytes=1769472 avg_lag_x100=0 max_lag=0 false_enospc_raw=24 false_enospc_arm=0 true_enospc=24 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 max_publications=48 bound=48 admissions_under_pressure=10200 forced_per_window_x100=3622 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=48 avg_candidates_x100=4800 avg_nonempty_x100=230 min_states=1 avg_states_x100=325
E7RESULT name=near_full arm=G_floor_per_disk s=16 c0=1 c_max=5 reserve=16 residue=10 hidden_blocks=26 hidden_bytes=425984 avg_lag_x100=0 max_lag=0 false_enospc_raw=24 false_enospc_arm=0 true_enospc=24 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 max_publications=7 bound=7 admissions_under_pressure=9984 forced_per_window_x100=486 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1039 avg_nonempty_x100=431 min_states=1 avg_states_x100=433
E7RESULT name=near_full arm=G2R_tightened_roots s=16 c0=1 c_max=5 reserve=60 residue=50 hidden_blocks=110 hidden_bytes=1802240 avg_lag_x100=0 max_lag=0 false_enospc_raw=24 false_enospc_arm=0 true_enospc=24 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 max_publications=7 bound=11 admissions_under_pressure=2784 forced_per_window_x100=131 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1490 avg_nonempty_x100=885 min_states=1 avg_states_x100=888
E7RESULT name=near_full arm=G2S_tightened_states s=16 c0=1 c_max=5 reserve=45 residue=35 hidden_blocks=80 hidden_bytes=1310720 avg_lag_x100=3380 max_lag=44 false_enospc_raw=72 false_enospc_arm=0 true_enospc=72 write_after_delete_failed=48 first_failure_window=2 last_failure_window=3 max_publications=4 bound=8 admissions_under_pressure=6552 forced_per_window_x100=114 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1247 avg_nonempty_x100=1033 min_states=4 avg_states_x100=1033
E7RESULT name=near_full arm=A_ring_persisted s=16 c0=5 c_max=10 reserve=245 residue=235 hidden_blocks=480 hidden_bytes=7864320 avg_lag_x100=0 max_lag=0 false_enospc_raw=24 false_enospc_arm=0 true_enospc=24 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 max_publications=48 bound=48 admissions_under_pressure=9624 forced_per_window_x100=3928 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=48 avg_candidates_x100=4800 avg_nonempty_x100=221 min_states=1 avg_states_x100=316
E7RESULT name=near_full arm=G_floor_per_disk s=16 c0=5 c_max=10 reserve=40 residue=30 hidden_blocks=70 hidden_bytes=1146880 avg_lag_x100=0 max_lag=0 false_enospc_raw=48 false_enospc_arm=0 true_enospc=48 write_after_delete_failed=24 first_failure_window=1 last_failure_window=1 max_publications=7 bound=7 admissions_under_pressure=9720 forced_per_window_x100=485 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=981 avg_nonempty_x100=395 min_states=1 avg_states_x100=397
E7RESULT name=near_full arm=G2R_tightened_roots s=16 c0=5 c_max=10 reserve=110 residue=95 hidden_blocks=205 hidden_bytes=3358720 avg_lag_x100=0 max_lag=0 false_enospc_raw=24 false_enospc_arm=0 true_enospc=24 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 max_publications=7 bound=11 admissions_under_pressure=1968 forced_per_window_x100=109 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1433 avg_nonempty_x100=867 min_states=1 avg_states_x100=871
E7RESULT name=near_full arm=G2S_tightened_states s=16 c0=5 c_max=10 reserve=80 residue=65 hidden_blocks=145 hidden_bytes=2375680 avg_lag_x100=3517 max_lag=52 false_enospc_raw=96 false_enospc_arm=0 true_enospc=96 write_after_delete_failed=72 first_failure_window=1 last_failure_window=3 max_publications=4 bound=8 admissions_under_pressure=7320 forced_per_window_x100=97 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1329 avg_nonempty_x100=1146 min_states=4 avg_states_x100=1146
```

Reading method: `hidden_blocks` = reservation pool + worst-case pushing-empty residue, which is the constant under-reported on `df` for this arm compared to the measurement basis of E135 (Dynamic Rollback Floor) (G2S further adds the lag portion of `avg_lag_x100` / `max_lag`); `forced_per_window_x100` is the forced publish count per user window × 100.

- `df` under-reporting: G2R 110 / 205 blocks (1.7 / 3.2 MiB across the two cost pairs; = reservation pool + residue = 15 + 19 c_max), 84 / 135 blocks more than Arm G of E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) — the excess is the margin of k_tol = 2 plus the difference calculated by c_max; independent of S, and linearly related to c_max where c_max has no defined scale. Arm A 108 / 480 blocks, growing linearly with N. G2S 80 / 145 blocks plus lag.
- Forced publishes: G2R approximately 1.1–1.3 times (c₀ = 1), 0.8–1.1 times (c₀ = 5) per user window; Arm A approximately 36–39 times (S = 16); Arm G of E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) approximately 4.9 times. G2R has fewer than G because charging worst-case residue at admission tightens disk space earlier, resulting in fewer handling occurrences (`admissions_under_pressure` 1968–2784 vs 9720–9984).
- Candidate set: G2R averages about 20 roots near full disk, of which about 8.7–8.9 are non-empty, with a minimum of 5 roots; by state count at least 1. G2S averages about 12–13 roots, about 10–11 non-empty, with at least 4 states.
- Format bytes: both tested arms require root records to carry F of 8 bytes, identical to the form in E135 (Dynamic Rollback Floor).

Root slot write failures during pushing-empty and broken slots (numbers for control arms A, G, transcribed verbatim, S = 16, c₀ = 5):

```text
E7RESULT name=drain_failure arm=A_ring_persisted s=16 c0=5 c_max=10 failures=1 cases=104 runs_with_false_enospc=96 runs_with_write_after_delete_failed=96 false_enospc_arm=96 false_enospc_raw=96 write_after_delete_failed=96 max_publications=48 bound=48 stalls=0 fakes=0 unrecoverable=0 retention_violations=0 injected=104
E7RESULT name=drain_failure arm=G_floor_per_disk s=16 c0=5 c_max=10 failures=2 cases=224 runs_with_false_enospc=64 runs_with_write_after_delete_failed=64 false_enospc_arm=64 false_enospc_raw=64 write_after_delete_failed=64 max_publications=7 bound=7 stalls=0 fakes=0 unrecoverable=0 retention_violations=0 injected=448
E7RESULT name=broken_slot arm=A_ring_persisted s=16 c0=5 c_max=10 seeds=8 seeds_with_no_space=8 seeds_halted=8 first_failure_window=67 false_enospc_arm=8 false_enospc_raw=8 write_after_delete_failed=8 max_publications=48 bound=48 stalls=8 fakes=0 unrecoverable=0 retention_violations=0 injected=16 forced_per_window_x100=72 max_lag=0
E7RESULT name=broken_slot arm=G2R_tightened_roots s=16 c0=5 c_max=10 seeds=8 seeds_with_no_space=0 seeds_halted=0 first_failure_window=0 false_enospc_arm=0 false_enospc_raw=0 write_after_delete_failed=0 max_publications=7 bound=11 stalls=0 fakes=0 unrecoverable=0 retention_violations=0 injected=176 forced_per_window_x100=7 max_lag=0
```

### Postscript After Round 4 Three-way Argument (2026-09-12)

D16 (Publish Semantics) settled item 1 Round 4 three-way argument (`research/prompts/d16-item1-r4-main-verification.md`) audited this artifact, and points struck were modified in-place in body text; here recorded are what was modified and what did not enter verdict:

- **Post-run test and artifact**: result rows of `reserve_sweep` do not print failure window numbers; "all 26 cells are first 3 windows" had no field backing for the 18 cells of reservation sweep. Supplemented test `states_arm_reserve_sweep_failure_windows_end_by_window_six` (reads apparatus only, does not modify `main` artifact, rerun still byte-for-byte identical), artifact `e139-tightened-floor-2026-09-12-reserve-sweep-windows.out` (18 rows); mutation M16 (lag counting blocks released by empty publishes too) turns it red.
- **Backward leg probes (uncommitted, clues only)**: four probes on apparatus replica, source diff and artifacts preserved as is in `research/prompts/d16-item1-r4-opus-probes/`, rerun by main agent byte-for-byte identical. They state: ① lag of `df_S` = 4 * blocks released in one window; deleting 32 8-block objects in one window yields `max_lag=1044` — unbounded; ② ring depth threshold k = 2 requires S >= 4, k = 3 requires S >= 5, k = 4 requires S >= 6 ((c₀, c_max) = (5, 10), 8 seeds); S = 2 / 3 within k_tol = 2 violates minimum retention 184 / 224 times, while fault worlds of this artifact swept only S in {4, 16}; ③ under full enumeration of 3 failures (84 combinations), S = 4, c₀ = 1 also violates 160 times, while this artifact takes only first 20 combinations reporting 0; ④ when quantity withheld at mount for instance switching and un-deducted from `df` > 60 blocks (capacity 4000), G2R has 24 false ENOSPC, G2S has 120 at 40 blocks — that quantity is C126 (Worst-case amount of switching reservation has no measurement basis); ⑤ withdrawing correction that "active metadata recorded back into reservation pool", G2R fails deletion-then-write across 24 / 24 seeds in (S, c₀) = (4, 5), (16, 5); ⑥ relaxing publish ceiling from B - 1, bound 7 + 2k / 4 + 2k unexceeded under full enumeration of k = 1..5; ⑦ forms outside this family (F + three out-of-ring anchor roots + judging reallocatable by block lifetime interval) preserve 4 states on near-full disk 8 cells with immediate write after delete, paying cost of pinning blocks alive when anchor was taken and dying later. Entering verdict requires formal measurement by next experiment.
- **Literal wording of Criterion 7**: registration Criterion 7 did not frame 3-failure cells as "report only" like Criterion 5; literally G2S loses once in S = 4, c₀ = 5, 3-failure cell; under k_tol = 2 scope it is out of scope. Both readings submitted to user, criterion unmodified.
- **`crit5_bound_exceeded_cells` cell vacuously true**: `maximum_empty_publications()` returns B - 1 for tightened arms, publishes per admission in apparatus is forever <= B; symptom of incomplete handling is false ENOSPC after truncation and deletion-then-write failure, not exceeding bound.
- **What registration should have written but omitted**: what upper bound takes when non-empty valid roots < 4 (supplemented after first run); tolerance exceptions for Criterion 7.
- `retention_violations` across arms is identical predicate `F > upper_bound`, upper bound taken per arm (G_txgcap newest root on each disk; G2R adds 4th newest root; G2S adds 4th newest non-empty root); it does not count states in candidate set; in fault worlds `min_states` of G2S is 1..2 (`steady` / `torn_newest` / `slot_write_fail` 2, `disk_loss` dropping disk 0 / `admin_rollback` / `rollback_first_publish` 1).
- Withdrawn number: "handling with 3 failures consumes 11 txgs" came from diagnosis run on replica during writing, no artifact retained.

### What It Cannot Answer

1. Wall clock: all counted by publishes.
2. Real devices: counting model, does not touch devices; no numbers on how common broken slots are on real devices.
3. Journal replay window: models root-level references only, not records.
4. Instance switching: models only "advancing txg by one cell, leaving old root in failed slot" half; its own mount-time reservation lacks measurement basis, `df` did not deduct it.
5. c_max is declared upper bound, this project has no measurement basis for it (C83 (commit fixed point unanswered) owes runtime readings); cells exceeding c_max reported only: G2R at c₀ = 1 rising to 8 is all 0 as usual, G2S yields 32 false ENOSPC. Non-empty publish metadata fixed at 5 blocks.
6. Capacity 4000 blocks: proportion of `df` under-reported blocks to capacity cannot transfer to real disks.
7. Slot formulas and disk allocation use layout/01-first-txn.md vision; two disks only.
8. k_tol fixed at 2; other k have formulas only (B = 7 + 2k / 4 + 2k). Combinations of 3 failures took only first 20 (earlier positions), maximum publish count in that cell is not worst-case value of 3 failures.
9. Lag of `df_S` constructed by registration author for state-count measurement basis; user never settled whether `df` should deduct it.
10. All four minimum values of reservation sweep fall on bottom edge of sweep (formula - 6), true requirements of both arms unmeasured.
11. Relationship between ring depth and state count (cell of 3 failures on S = 4) has only one observation, no formula.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) settled item 1 | Supports | 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized only; conclusions, numbers, and artifacts unchanged; prior review 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D3 (Space Allocation) settled item 9 | Supports | 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized only; conclusions, numbers, and artifacts unchanged; prior review 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D23 (Journal Role & Format) | Does not affect | 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized only; conclusions, numbers, and artifacts unchanged; prior review 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |

## Historical Versions

### 2026-10-05
- Rerun 2 (candidate floor adding system configuration F, reclamation threshold and five other locations returning to whole ring): apparatus added clause-form candidate floor CF, four `Arm::WholeRing` arms (RGall, RGall_Q, two `_short`), `disk_stuck` world and sampling of three cost items, 19 unit tests, 16 mutations; new artifact `e139-tightened-floor-2026-10-05.out`, `replay.sh` registration row redirected to it; body added section "Rerun 2", unit tests and mutation counts in "This Is a Counting Model" updated to 55 and 42; in Rerun 1 section, rollback candidate set row and item 3 of "What Rerun 1 Cannot Answer" changed from "apparatus candidate set is no larger than clause's, fake candidates leaning low" to "no smaller, leaning high" with counts (direction originally inverted); identical updates made in direction summary; reviews updated in "Affected Decisions" and added D23 (Journal Role & Format) row.
  - **Rationale**: rerun registration `e139-r2-prereg.md`; issue ticket `m3-r1-e139-r2-questions.md`; artifact `e139-tightened-floor-2026-10-05.out`; owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus).
  - **Pre-change state**: page before this modification (Rerun 1 section already written, uncommitted in same batch; `git show 31e47115:.claude/kb/experiments/139-按盘回退下界的收严形态.md` was version prior to Rerun 1, lacking both Rerun 1 and Rerun 2 sections).

### 2026-10-04
- Rerun 1 (reclamation threshold changed to one-ring window): apparatus added seven arms, 15 unit tests, 10 mutations; new artifact `e139-tightened-floor-2026-10-04.out`, `replay.sh` registration row redirected to it; body added section "Rerun 1", unit tests and mutation counts in "This Is a Counting Model" updated to 36 and 26; two review rows in "Affected Decisions" updated.
  - **Rationale**: rerun registration `e139-r1-prereg.md`; owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus).
  - **Pre-change state**: `git show 31e47115:.claude/kb/experiments/139-按盘回退下界的收严形态.md` (no Rerun 1 section, 21 unit tests, 16 mutations, `replay.sh` pointed to `e139-tightened-floor-2026-09-12.out`).

### 2026-09-12
- Created. Three changes made during apparatus coding (all prior to first run): arithmetic of G2S lag sentence (omitted 8 blocks deleted in each of first two windows), lag excluding blocks released by empty publishes, tightened arms recording active metadata back into reservation pool (without recording back, consecutive disposals between full disk and first deletion-creation cycle had only one deletion, causing G2R at c₀ = 5 to fail deletion-then-write once; `df` reported 6, physically allocatable 76).
- One change after first run: when non-empty valid roots < 4, upper bound under state-count reading changed from "oldest non-empty valid root" to "oldest valid root" — in a 3-slot ring, empty roots older than oldest non-empty root carry older states; skipping them equals losing states.
  Artifact of first run preserved as `e139-tightened-floor-2026-09-12-run1.out`, differing from second run by 8 rows, all in S = 1: five cells below (transcribed verbatim from first run artifact), two `near_full` `avg_lag_x100` cells (c₀ = 1 / 5: 1354 → 1426 / 1728, lag changing after upper bound method changed), and `crit7_retention` in `verdict_candidate` row (65 → 24 = deducting 41 from five cells):
```text
  E7RESULT name=cell arm=G2S_tightened_states world=torn_newest s=1 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=18 checks=9654 min_candidates=2 avg_candidates_x100=299 avg_nonempty_x100=297 min_states=1 avg_states_x100=298 stalls=0
  E7RESULT name=cell arm=G2S_tightened_states world=disk_loss s=1 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=6 checks=2258 min_candidates=1 avg_candidates_x100=297 avg_nonempty_x100=290 min_states=1 avg_states_x100=292 stalls=0
  E7RESULT name=cell arm=G2S_tightened_states world=disk_loss s=1 lost_disk=1 fakes=0 unrecoverable=0 retention_violations=6 checks=2258 min_candidates=2 avg_candidates_x100=298 avg_nonempty_x100=291 min_states=1 avg_states_x100=293 stalls=0
  E7RESULT name=cell arm=G2S_tightened_states world=slot_write_fail s=1 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=6 checks=9654 min_candidates=3 avg_candidates_x100=300 avg_nonempty_x100=298 min_states=1 avg_states_x100=298 stalls=0
  E7RESULT name=cell arm=G2S_tightened_states world=admin_rollback s=1 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=5 checks=9677 min_candidates=1 avg_candidates_x100=298 avg_nonempty_x100=297 min_states=1 avg_states_x100=297 stalls=0
  ```
- Four sentences modified after Round 4 three-way argument (2026-09-12, `research/prompts/d16-item1-r4-main-verification.md`): "two runs differed by only five cells and two rows" changed to 8 rows; "all 26 cells are first 3 windows" changed to main cells 1..3, reservation sweep up to 6 (supplemented test and artifact); "at least 4 states at any time" narrowed to near-full disk; withdrew "consuming 11 txgs". 205 blocks 3.3 MiB changed to 3.2. Literal wording of Criterion 7, `bound_exceeded` being vacuously true, and G2R's two qualifications documented in "Postscript After Round 4 Three-way Argument". Unit tests 20 → 21, mutations 15 → 16.
