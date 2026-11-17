## E203 Step cost of the scans before admission — partly run (enough to judge): B1 and B4 are enough to judge; B8 waits for the fixed-point measurement; Y1 is left to the unscoped run of the experiment replay

[中文](203-准入之前扫类的步代价.md) · **English** · [日本語](203-受理前スキャン系のステップコスト.ja.md)

Pre-registration `research/prompts/e203-preregistration.md` (section 12, revisions 1–8). Fork sheet `research/prompts/m4-admission-delta-forks.md`, three rows: B1 today, B8 item by item by delta, B4 change the scope. It measures the steps and wall clock of the scans each publish runs before the two acceptances, across three orders of magnitude of unit area and record counts, including X2, the unit area of 16 TiB per disk (2³⁰ − 1408 slots, with u, D and F at 10 each).

**This page records the 18 points rerun under revision 7 (one artifact each for the counting and timing builds, 36 artifacts in all) and their merged report (task 10180).** Two verdict rounds: `research/prompts/e203-forks-r1-main-verification.md` and `research/prompts/e203-forks-r2-main-verification.md`; where they disagree, the second round holds.

### Current state

- All three segments ran: the first segment's 8 points (U axis and u axis), the second segment's 9 points (D axis, F axis and the diagonal point X1), and X2 in the third segment. Y1 of the second segment (`many_unlinks_unit_area_2_pow_16`) did not run: revision 6 moved it into the unscoped run of the experiment replay.
- Two rows are enough to judge: B1 and B4 (B4 only for the overwrite class). B8 is recorded as "unstable" (revision 8) and waits for the fixed-point measurement: fork sheet `research/prompts/e203-forks-r2-forks.md` (three rows P1, P2, P3), registered as E206 (Cost of the publish fixed point rounds): segment one has run, its P1 per-round readings are void and kept as clues only, segment two is not run (judgeable already).
- This page does not judge whether a conclusion can overturn or establish a decision; the tables handed to the user in the two verdict rounds still wait for the user.
- Apparatus: measuring path `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py`, `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs` and entry `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh`; derived path `scripts/experiment-apparatus/e203_admission_scan_cost.rs` (bin name `e203-admission-scan-cost`), mutation table `scripts/mutation/tables/e203_admission_scan_cost.tsv`.

### Results of the three segments

The rows below are copied whole from `research/results/e203-admission-scan-cost-report-2026-11-13.out` (2755 lines, the whole report recomputed on 2026-11-13 by today's apparatus from the 36 artifacts, prefix `E7RESULT ` on every line, last line `E7RESULT name=done emitted=2755`; the rerun table compares this one). The report of task 10180 (`research/results/e203-admission-scan-cost-report-2026-11-07.out`, the 2643 lines starting with `name=` in the log of task 10180 with the prefix and one `name=done` line, 2644 lines in all) is kept; the new one has 111 more lines: 108 lines `name=by_delta_pooled_fallback`, one line `name=verdict quantity=by_delta_pooled_fallback`, one line `name=verdict clause=void_frozen_publish` and one line `name=three_two_row row=pooled_fallback_by_the_leaf`; the numbers in the `quantity=three_two` line changed (see "Criteria and failure clauses" below) and so did the count on the closing line; every other line is byte-identical (the two `diff`s show only these). Nothing else was changed in copying.

First segment (U axis, u axis):

```text
E7RESULT name=q1 axis=U state=same_mount kind=overwrite row=whole_allocator_copy_before_the_judgement low_end=101959 high_end=103286290 alpha=0.998444 band=GrowsWithTheAxis
E7RESULT name=q2 axis=U state=same_mount kind=overwrite low_end=350496 high_end=309991237 alpha=0.978861 band=GrowsWithTheAxis
E7RESULT name=q2 axis=u state=same_mount kind=overwrite low_end=2503416 high_end=10171494 alpha=0.202951 band=DoesNotGrow
E7RESULT name=q3 axis=u state=same_mount kind=overwrite alphas=0.772315,0.619106,0.641893,0.670220,0.626462 band=GrowsWithTheAxis
E7RESULT name=q10 axis=u state=same_mount kind=overwrite segment=before_landing alphas=0.787311,0.757060,0.771941,0.790239,0.789375 band=GrowsWithTheAxis
```

Second segment (D axis, F axis, X1):

```text
E7RESULT name=q2 axis=D state=same_mount kind=overwrite low_end=2503416 high_end=8252955 alpha=0.172692 band=DoesNotGrow
E7RESULT name=q2 axis=F state=same_mount kind=overwrite low_end=2503416 high_end=163060965 alpha=0.604606 band=GrowsWithTheAxis
E7RESULT name=q6 axis=F state=same_mount kind=overwrite low_end=2102 high_end=152344 alpha=0.620064 band=GrowsWithTheAxis
E7RESULT name=q6 axis=F state=remounted kind=overwrite low_end=2270 high_end=4574 alpha=0.101423 band=DoesNotGrow
E7RESULT name=q5 axis=diagonal state=same_mount predicted_steps=34830876 measured_steps=62106461 nanos_ratios=1.178,0.933,1.000,1.347,0.709 passed=false
```

Third segment (X2):

```text
E7RESULT name=q6_top_rows point=user_scale_unit_area state=same_mount kind=overwrite rows=allocator_release_linear_find:1012,record_intersection_filter:354,mapped_units_whole_table:322
E7RESULT name=q9 point=user_scale_unit_area state=same_mount kind=overwrite steps_between=13220563557 nanos_of_each_valid_round=2959498030,2964803058,3551101363,3550223622,3542668057
E7RESULT name=q9 point=user_scale_unit_area state=remounted kind=overwrite steps_between=19830857527 nanos_of_each_valid_round=4475982759,2961183237,3020373367,3135067922,3604622355
```

On X2 the caller's whole copy (`whole_allocator_copy_before_the_judgement`) takes 1.390–1.719 seconds on that row (measured: three publish kinds × two mount states × five rounds, 30 readings in all, from `exclusive_nanos` in the timing artifact).

### Fork table

| Candidate | Status | Basis | Still missing |
|---|---|---|---|
| B1 today | Enough to judge: cannot stay | It grows with disk capacity on the U axis (Q2 six cells, α 0.95–1.01, measured), which already breaks "cost must not grow with disk capacity" on the runtime decision path of `.claude/rules/fs-design.md`; on X2 the whole-copy row takes 1.390–1.719 seconds (measured). F1 is recorded unknown: the verdict does not allow the X2 reading to rejudge F1, so that reading stands only as a lower bound | ③2 is literally short by 2 unread cells (see "Criteria and failure clauses"); the u axis goes to 13 containers only, provisional |
| B8 item by item by delta | Unstable, judged neither flipped nor standing (revision 8) | Only one cell is left: Q6, F axis, same mount, overwrite, α 0.620, only in round 1 of files_10000 (22 fixed-point rounds); rounds 2–5 of that point and the remount do not grow. By the letter of F2 this is still a hit and is recorded as such. Revision 7 changed s36 to ask by delta, a relaxation against the original criterion, and the new definition of B8 has seen zero rounds of attack | The counting experiment on the fixed-point rounds; once measured, rejudge under the original criterion |
| B4 change the scope | Enough to judge (overwrite class): can only be handed over as a clause change | The B4 arm of Q8 does not grow in all six cells on all four axes and F5 did not fire; but the formula judgement for overwrite is the ENOSPC acceptance of D28 (Mount-Time Commitments) settled item 1, whose demand comes from the result computed between the two judgements, so the reading oversteps, as both verdict rounds judge. The conclusion covers only the listed steps (F3 fired). The residual of the segment before the placement judgement grows in all five rounds on the u axis for overwrite, right inside the scope, source not measured | The reading for deletion publishes has seen zero rounds of attack; the Q9 user-scale numbers do not count (revision 8) |

Counts by family (one command counts them from the artifact; the command and its raw output are in the runner's report): on Q2 the U axis grows in 6 cells, the u and D axes do not grow in 6 cells each, the F axis has 1 growing, 1 between and 4 not growing; on Q3 the U and F axes grow in 6 cells each, the u axis has 5 growing and 1 unstable, the D axis 2 growing and 4 unstable; on Q6 the U and D axes do not grow in 6 cells each, the u axis has 5 not growing and 1 between, the F axis 5 not growing and 1 growing; on Q8 the B4 arm does not grow in 6 cells on each of four axes, and the B1 arm grows in 6 cells on the U axis and does not grow in 6 cells on each of the other three.

### Replay

```bash
# Merged report: recompute from the 36 artifacts, compare byte for byte with the retained artifact (exact); histories are not rerun
bash scripts/experiment-apparatus/replay.sh E203
# One of the 18 points: measuring path, source tree is the frozen copy from the revision 7 run
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build counts --work <new directory> --source <frozen source tree> --item base
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build timing --work <new directory> --source <frozen source tree> --item base
# The original command of the merged report
cargo run --release --bin e203-admission-scan-cost -- report <18 counting artifacts> --timing <18 timing artifacts>
```

Each command is submitted by the main agent through `task-server/task submit`. The Y1 row is registered in `scripts/experiment-apparatus/replay.sh` and runs only in the unscoped run of the experiment replay (revision 6); the scoped run `replay.sh E203` does not reach it. Once it has readings, the judgements that read the container-count axis are judged again under the original criterion.

### Measurement basis

- Merged report: `research/results/e203-admission-scan-cost-report-2026-11-13.out` (2755 lines, the one the rerun table compares); the one of task 10180, `research/results/e203-admission-scan-cost-report-2026-11-07.out` (2644 lines), is kept.
- The retained expectation of the Y1 row: `research/results/e203-admission-scan-cost-many-unlinks-completion.out` (three hand-written completion lines, not measured numbers; the replay table needs something to compare with, and the same three lines are printed only once both builds are done).
- The 12 raw outputs still in the worktree:
  - `research/results/e203-admission-scan-cost-counts-diagonal-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-diagonal-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_1-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_100-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_1000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_1-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_100-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_1000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-unlinks_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07-r2.out`
- The 24 archived artifacts, each pointing at its completion-marker line:

```text
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-base-2026-11-07.out 第 6067 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-07.out 第 6164 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_1000-2026-11-07.out 第 6326 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_100-2026-11-07.out 第 5930 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_1-2026-11-07.out 第 5922 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_10000-2026-11-07.out 第 7550 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_1000-2026-11-07.out 第 6476 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_100-2026-11-07.out 第 6126 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_1-2026-11-07.out 第 6028 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_14-2026-11-07.out 第 5758 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_20-2026-11-07.out 第 6067 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_24-2026-11-07.out 第 6236 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_10000-2026-11-07.out 第 7002 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_1000-2026-11-07.out 第 6204 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_100-2026-11-07.out 第 6006 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_1-2026-11-07.out 第 6004 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07-r2.out 第 6236 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-base-2026-11-07.out 第 1659 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_14-2026-11-07.out 第 1661 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_20-2026-11-07.out 第 1659 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_24-2026-11-07.out 第 1651 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_1000-2026-11-07.out 第 1651 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_100-2026-11-07.out 第 1665 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_1-2026-11-07.out 第 1663 行（name=item_done）
```

- "Before acceptance" is cut into three segments reported separately: caller segment (entry to the call of prepare), before the placement judgement, between the two judgements; after acceptance is reported as a count only. A scan step is one step per entry of the innermost loop body; a whole copy counts the sum of the element counts of every collection in the copied value.
- Timing uses a release build, a single thread, and drops the `lookup-work-counters` feature; each point is measured five rounds in the same mount and after a remount.
- Both s25 lookups now record calls (revision 7); only the call count was added, the steps did not change in a single cell (revision 8; the main agent compared the old and new base counting artifacts: 1412 shared rows, 60 differ in calls, 0 differ in steps).
- The query count of s36 is the number of roles entering or leaving `mapped_units` in that round (revision 7); under the formula before revision 7, F2 fired in task 9595 and 12 cells on the D and F axes were recorded as growing, already logged as a loss.
- The point axes of the timing report come from the counting artifact of the same name (revision 7).
- Positive control: each arm has a switch that turns off the step it saves and makes it grow once with the unit area; negative control: the steps assembling the units for the placement judgement depend only on this publish's roles.

### Criteria and failure clauses

- Criteria: growth order α in three bands (α ≥ 0.5 grows with the axis, α ≤ 0.25 does not, in between no verdict); at the user-scale shape the scan wall clock of one overwrite before acceptance is "≥ 1 ms" if all five rounds are ≥ 2 ms, "< 1 ms" if all are ≤ 0.5 ms, otherwise unstable.
- Verdict rows (copied from the merged report):

```text
E7RESULT name=verdict clause=void_frozen_publish points_voided_in_counts=0 points_voided_in_timing=0 cells_without_the_reading_in_counts=540 cells_without_the_reading_in_timing=540
E7RESULT name=verdict quantity=by_delta_pooled_fallback cells_checked=108 cells_with_pooled_fallback_calls=0
E7RESULT name=verdict quantity=controls all_passed=true
E7RESULT name=verdict quantity=three_two cells_checked=14578 disagreements=0 cells_over_the_bound=0 cells_with_the_scale_unread=2 rows_reconciled=27 rows_equal_only=18 rows_with_a_bounded_part=8 rows_steps_only=1 rows_not_reconciled=0
E7RESULT name=verdict clause=stop_2 constants_not_matching_or_unread=0 three_two_disagreements=0 three_two_cells_over_the_bound=0
E7RESULT name=verdict quantity=Q6 result=GrowsOnSomeAxis
E7RESULT name=verdict quantity=Q8_b4_arm result=DoesNotGrowOnAnyAxis
E7RESULT name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond reference_only_because_data_or_files_axes_are_missing=false not_judged_because_the_extrapolation_is_not_trusted=true
E7RESULT name=verdict quantity=Q5 cells_passed=2 cells_failed=8
E7RESULT name=verdict clause=F1 fired=unknown meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1
E7RESULT name=verdict clause=F2 fired=true meaning=Q6_B8_still_grows_on_some_axis
E7RESULT name=verdict clause=F3 fired=true meaning=residual_grows_on_some_axis_in_all_five_rounds
E7RESULT name=verdict clause=F4 fired=false unlinks_not_two_publishes=0
E7RESULT name=verdict clause=F5 fired=false meaning=B4_arm_before_the_landing_judgement_still_grows
E7RESULT name=verdict clause=F6 fired=true meaning=some_Q5_cell_failed
```

- F1: `fired=unknown`. F6 fired, so Q4 is recorded "extrapolation not trusted, not judged" (`not_judged_because_the_extrapolation_is_not_trusted=true`); `reference_only_because_data_or_files_axes_are_missing=false` says the D and F axes both have readings, not that Q4 was judged.
- F2: `fired=true`, only for the one cell left to B8, and that cell is recorded "unstable" (revision 8, fork table).
- F3: `fired=true`. The conclusions cover only the listed steps. The residual's share of the segment wall clock hardly changes along the axes, so the registration's way out, "complete the list", may not reach it; the source of the residual was not measured.
- F4: `fired=false`, every measured unlink was judged twice. F5: `fired=false`, the B4 arm does not grow on any of the four axes.
- F6: `fired=true`, 8 of the 10 Q5 cells failed; as registered, the third segment ran, and Q4 still uses measurements only for the U part.
- ③2: `cells_with_the_scale_unread=2` is not 0, and revision 4 says "unread cells not at 0 do not count as ③2 fully passed". The two cells are `multi_level_tree_planning` and `mapped_units_whole_table` in round 4 of the remounted overwrite at data_units_10000. By the letter this condition is not met; the verdict that B1 is enough to judge rests on the U axis and the whole-copy readings, not on these two cells.

### What it cannot answer

- The `by_delta_pooled_fallback` lines and the `pooled_fallback_by_the_leaf` reconciliation row added on 2026-11-13 are readings the apparatus gained later and they use the same 36 artifacts of 2026-11-07, the histories were not rerun; the line `void_frozen_publish` has `cells_without_the_reading_in_counts=540` and `cells_without_the_reading_in_timing=540`, which means the 36 artifacts do not carry the frozen-publish field, so the void clause was not triggered but also not checked; whether `pooled_fallback_calls=0` is a 0 recorded because no such line is in the artifacts or a real "did not happen", this page does not tell.
- Only on a 6 MiB ring, two disks, overwrite and unlink; ring length, disk count, truncation, snapshots and compaction moves are not swept (registration section 8, item 4).
- X2 only pushes the unit area to 16 TiB per disk, not the record counts; on X2 the fixed point takes only 2–3 rounds, while 22 rounds were measured elsewhere and the source limit is 64. The shape of 16 TiB per disk after 10⁶ unlinks is extrapolated and Q5 failed its check: neither the Q4 nor the Q9 user-scale numbers count.
- Containers are measured only to 13 tombstone containers (Y1 did not run); judgements reading the container-count axis are all marked provisional, and growth above 13 is not measurable in this segment.
- How the 22 rounds arise and what a bound on rounds costs were not measured here (the fixed-point measurement).
- The whole copy is 95.7% of the scan sum at the baseline (attacker, derived) and flattens the other rows: counting Q2 does not grow on the u and D axes while timing Q3 grows; the two readings do not conflict, and this is recorded without changing a verdict. The three rows F2 lists are the X2 point the model picks by the sum over the four axes, not the high end of the F axis.
- The cost of copying bytes belongs to E199 (Cost of Copying the Previous Version at Publish); this page asks only about scans.
- Some readings are the apparatus's own operationalisation (the registration did not spell them out): B8's `m_round` is the placements released in a fixed-point round plus the roles newly allocated; `dirty` is the number of records that differ before and after one call; `dm` is twice the rewritten roles (one out, one in); in the maintenance term `m_rec` is the same sum in the last round and `m_seg` takes it as an upper bound; tree heights are read from the root node headers.
- s9, s10 and s17 count record numbers only, because the shape copies sit in Verus blocks where nothing can be inserted; s3's timer covers the whole writer-call statement (the copy is inside a Verus block) and so includes the writer's non-copy work; these four rows are copy rows and stay out of the scan sum.
- s32 counts only the release of the instance-table chain (overwrites and unlinks at the sample points write no row, so it mostly is not reached; inferred, not measured).

### Paths and conclusions register

| # | Path | How computed | Source location | Shared items read |
|---|---|---|---|---|
| 1 | Measuring path (B1 counts) | Counters only inserted into a copy of the implementation, the history cases print `E203RESULT` rows per point and state; the sample points are 18 enumerated points (Y1 did not run), not the whole domain | `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:653 edits`, `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs:674 run_the_point` | the code under test itself; the implementation's own constants; the `PoolUnderTest` pool setup |
| 2 | Derived path (counting model) | Pure arithmetic: B8 and B4 step formulas, resident memory, α and banding, extrapolation; anchor unit tests pin absolute values; the report reading the artifacts wires up Q1–Q13 and the failure clauses | `scripts/experiment-apparatus/e203_admission_scan_cost.rs:187 by_delta_rows`, `scripts/experiment-apparatus/e203_admission_scan_cost.rs:471 narrowed_steps_by_segment`, `scripts/experiment-apparatus/e203_admission_scan_cost.rs:3403 full_report` | local constants (sources noted in the file header, the implementation's values in the artifact header compared one by one); the B8 column of registration section 5 |

The reconciliation of the two paths (registration section 7, check ③2, revision 4) now covers all 27 rows in today's merged report (the one of 2026-11-07 has 26 rows): 18 rows equal cell by cell, 8 rows with an upper-bound part, 1 row compared on steps only (`rows_steps_only=1`, the new `pooled_fallback_by_the_leaf`: relation `EveryCallTakesAStep`, 540 cells checked, 0 unequal, 0 over the bound), `rows_not_reconciled=0`; 0 cells unequal, 0 over the bound, 2 unread (previous section).

### Affected decisions

| Decision item | Relation | Review |
|---|---|---|
| D8 (core index structure) settled item 16 | Preparation | 2026-11-17 not affected: only a stale statement of the current state and a pointer to the artifacts changed; conclusions, numbers and artifacts are unchanged; 2026-11-13 not affected: that day only the merged report was replaced by the whole report recomputed by today's apparatus (a batch of `by_delta_pooled_fallback` lines is new and the ③2 reconciliation goes from 26 rows to 27); readings, artifacts and conclusions are unchanged; 2026-11-08 unaffected: this page only records readings, the decision and its scope are unchanged; "the step cost before acceptance does not grow with the total container or record count" does not hold on the U and F axes as measured (B1), and whether to change the scope is the B4 row, waiting for the user; the basis of this item does not cite E203 |
| D28 (Mount-Time Commitments) settled item 1 | Does not affect | 2026-11-17 not affected: only a stale statement of the current state and a pointer to the artifacts changed; conclusions, numbers and artifacts are unchanged; 2026-11-13 not affected: that day only the merged report was replaced by the whole report recomputed by today's apparatus (a batch of `by_delta_pooled_fallback` lines is new and the ③2 reconciliation goes from 26 rows to 27); readings, artifacts and conclusions are unchanged; 2026-11-08 unaffected: the verdict cites it to judge that the formula judgement is the ENOSPC acceptance; the clause does not change |

## Historical Versions

### 2026-11-13

- Merged report renewed: `replay.sh E203` of task 16215 judged "does not match" because the apparatus later gained the `by_delta_pooled_fallback` reading and the `pooled_fallback_by_the_leaf` reconciliation row; today's apparatus recomputed 2755 lines, `research/results/e203-admission-scan-cost-report-2026-11-13.out` is new, the registration line now points to it, and the one of 2026-11-07 is kept. It has 111 lines more than the one of 2026-11-07 (108 lines `by_delta_pooled_fallback`, two verdict lines, one `three_two_row` line); in the `three_two` verdict line `cells_checked=14038` became 14578, `rows_reconciled=26` became 27 and `rows_steps_only=1` was added; every other line is byte-identical. The verdict lines and the reconciliation sentence on this page follow the new report, the two new verdict lines were copied into the verdict-lines section, and one item was added to "What it cannot answer" on where the new readings come from; readings, artifacts and conclusions are unchanged; the decision table was reviewed line by line up to 2026-11-13.

### 2026-11-08

- Brought to current values: the three segments of the revision 7 rerun, the fork table, the criteria outcomes, the paths register and the decision review are written; the 36 artifacts are named and the merged report artifact is added. The false fields in the verdict rows, `fired=false` (F4, F5) and `reference_only_because_data_or_files_axes_are_missing=false` (Q4), are explained under "Criteria and failure clauses".

### 2026-11-05

- File created, registering E203 (Step cost of the scans before admission) pre-registration and first-segment apparatus; the artifact has not been run once.
- Close of the first segment: the self-test's guard count in the counter-insertion script now runs per file and names are full words; the model gained the user-scale steps, the ③2 reconciliation on five rows, void clause 8, unit tests for stop clauses 3 and 4 and end-to-end cases for the failure clauses; the artifact is still unrun.
