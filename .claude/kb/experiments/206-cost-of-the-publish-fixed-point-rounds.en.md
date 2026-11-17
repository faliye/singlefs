## E206 Cost of the publish fixed point rounds —— partly run: segment one has 40 items (39 complete, 1 void); void clause 1 records one loss, the round-by-round readings of P1 are void and kept as a lead only; F1 hit, P1 flips; P2a is judgeable; the premise of P3 does not hold (the readings come from the round-by-round readings of P1, kept as a lead only); the 33 items of segment two are not run, judgeable already

[中文](206-发布固定点轮数的代价.md) · **English** · [日本語](206-パブリッシュ固定点ラウンド数のコスト.ja.md)

<!-- doc-lint:not-numbers P1 P2 P3 P2a P2b B1 B4 B8 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 F1 F2 F3 F4 F5 F6 F7 F8 W1 W2 W3 W4 U -->

Pre-run registration `research/prompts/e206-preregistration.md` (revision 1 in section 12 records three things added after the artifacts), fork sheet `research/prompts/e203-forks-r2-forks.md`, rows P1, P2 and P3. It asks how many rounds the fixed point of one publish takes, what shape of history makes it take many, how that changes publish after publish, and what it costs to push the rounds down (P2) or to make each round cheaper (P3). Once this is measured, the B8 cell of E203 (Step cost of the scans before admission) is judged again by its original criterion (revision 8 in section 12 of the registration of E203 (Step cost of the scans before admission)); the readings are in the section "Readings for the B8 cell" below.

**This page records the 40 artifacts of segment one and the merged report (task 11102).** The last line of the merged report `research/results/e206-fixed-point-cost-report-2026-11-09.out` is `name=report_done items=40 complete=39`. The 33 items of segment two and segment three are not run (judgeable already, so not run). This page does not judge whether a conclusion can overturn or establish a decision, and does not choose among P1, P2 and P3; "which one" goes to the user.

### Current state

- Segment one has 40 items: 16 points with the two arms p1 and p2a make 32 items, `single_step` (`base`, `files_100`, `files_1000`) and `closure_off` (the same three points) make 6, and `plain` (`base`, `files_1000`) makes 2. They ran as three groups of `--item` (registration revision 1), and verdicts go through prune-server. 39 items have `name=item_done`; `files_1000_units_100__p1` has none, because the first overwrite in its measurement window was refused (next entry).
- **F1 hit**: at the first overwrite in the measurement window of `files_1000_units_100__p1` the fixed point ran the full `fixed_point_rounds=64` and returned `AllocationRecordTreeRewriteSetDidNotSettle`. This is a refusal on the product path (limit `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`, `transaction.rs:13466`); registration F1 requires handing it to the main agent on the spot, and it is in the report of the experiment runner. The estimate of section 4, item 12 of the registration (today's implementation returns this error at `files_1000_units_100`) hit on that point; the same estimate for `files_30000` (about 74 rounds, past the limit) did not hit, the measured value is 38 rounds (void, kept as a lead only). That refusal of F1 is the outcome of the unmodified fixed point loop of the P1 arm and does not depend on the `single_step` control, so it is reported as it is.
- **Void clause 1 records one loss**: the `single_step` control has `holds=false` at three points (lines 569, 571 and 573 of the merged report; these are line numbers in the 753-line original report of task 11102, in the retained report of 2026-11-13 they are lines 607, 609 and 611), which literally triggers void clause 1; the round-by-round readings of P1 are void as a whole and the criterion is not relaxed. Below, every number that comes from the round-by-round readings of the P1 arm (lines with `arm=p1` or `item=…__p1`, including the Q1 peaks 38, 29, 22 and 11) is a void reading and serves only as a lead; the whole lines are copied unchanged and not deleted. The choice of P2a rests on its own readings and the comparison of fixed points (judged by the main agent, in the milestone four pre-development record `records/2026-10-13-里程碑四预开发.md`, line 1579).
- Void clause 6 of the registration: this point and this arm are void (`name=void_6`). **P2a has no comparison partner at that point**: `files_1000_units_100__p2a` is complete (`name=item_complete … complete=true`), but the fixed point agreement rate of Q5, the net steps of Q5 and the write amplification of Q6 are all P2a compared with P1 at the same point, P1 has no readings there, so these three quantities have no row at that point.
- On 2026-11-09 the user decided in a popup "implement P2a now", and the 33 items of segment two are not run (judgeable already, so not run). Segment two is 24 items (p1 and p2a at 12 points) plus 9 items (P2b16, P2b256 and `whole` at the end points of three axes), 33 items by the count of section 5 of the registration; the 36 items written in the popup were wrong (`records/2026-10-13-里程碑四预开发.md` line 1579).
- The three `fork_row` lines of the merged report: P1 `enough_readings=false`, P2 `enough_readings=true`, P3 `enough_readings=false`. The flag only counts whether the readings of p1 (P1, P3) or p2a (P2) are complete at all 16 points of segment one (`fork_rows_with_enough_readings`, `e206_fixed_point_cost.rs:1872`): P1 and P3 are false because `files_1000_units_100__p1` is void, not because other points lack readings.
- Apparatus: the measured path is `scripts/experiment-apparatus/e206_fixed_point_cost_counters.py`, `e206_fixed_point_cost_histories.rs`, the entry `e206-fixed-point-cost-run.sh` and the verdict library `e206_fixed_point_cost_library.py`; the derived path is `scripts/experiment-apparatus/e206_fixed_point_cost.rs` (bin name `e206-fixed-point-cost`). Mutation tables: the model `scripts/mutation/tables/e206_fixed_point_cost.tsv`, the counter insertion script `e206_fixed_point_cost_counters.mutations.tsv`, the history test cases `scripts/mutation/tables/e206_fixed_point_cost_histories.tsv` (11 rows, **not shown to go red**, registration revision 1).

### Results of segment one

The lines below are copied whole from `research/results/e206-fixed-point-cost-report-2026-11-09.out` (the 753 lines starting with `name=` in the log of task 11102; `e206-fixed-point-cost-report-2026-11-09-r2.out` in the same directory is the retained form with the prefix `E7RESULT ` on every line and `name=done` at the end). The rerun table now compares `research/results/e206-fixed-point-cost-report-2026-11-13.out` of 2026-11-13 (792 lines, the whole report recomputed by today's apparatus from the 40 artifacts, prefix `E7RESULT ` on every line, last line `E7RESULT name=done emitted=792`): it has 38 more lines `name=by_delta_pooled_fallback` than `-r2` (a reading the apparatus gained later, unrelated to the lines below) and a different count on the closing line, and every other line is byte-identical (the two `diff`s show only these); after the prefix is removed, every line quoted on this page is found in it word for word (the order of the two items in the line `question=q4 arm=p1` under "Judgment lines" was corrected on 2026-11-13 to this report, see Historical Versions). Nothing else was changed in copying; product lines are quoted from the 40 artifacts of segment one themselves (each line has the prefix `E206RESULT `). Lines that come from the round-by-round readings of the P1 arm (void clause 1) are copied and not deleted, and the introducing sentence of each block marks them "void, kept as a lead only".

**F1: the one that hit the limit** (the outcome of the P1 loop, independent of the `single_step` control, reported as it is; the last three lines of `research/results/e206-fixed-point-cost-files_1000_units_100-p1-2026-11-09.out`, 3535 lines in all, no `name=item_done`):

```text
E206RESULT name=publish item=files_1000_units_100__p1 phase=window state=same_mount op=W1 op_index=1 publish=same_mount/W1/1/0 had_caller_segment=true fixed_point_rounds=64
E206RESULT name=window_refused item=files_1000_units_100__p1 phase=window state=same_mount op=W1 op_index=1 change=overwrite cause=AllocationRecordTreeRewriteSetDidNotSettle
```

```text
name=void_6 item=files_1000_units_100__p1 problem=refused:AllocationRecordTreeRewriteSetDidNotSettle voided=this_point_this_arm
name=q1 item=files_1000_units_100__p1 point=files_1000_units_100 arm=p1 peak_rounds=64 at=same_mount/W1/1 verdict=hits_the_limit first_overwrite_rounds=64 window_rest_peak_rounds=0 history_peak_rounds=22
name=f8 item=files_1000_units_100__p1 refused=AllocationRecordTreeRewriteSetDidNotSettle refused_for_not_settling=true
name=peaks item=files_1000_units_100__p1 peak_round_work=19004 peak_final_roles=171 peak_final_nodes=415 peak_effective_rounds=19.082
name=q9 item=files_1000_units_100__p1 state=same_mount publishes=1 peak=64 rounds_at_least_3=1 rounds_at_least_5=1 rounds_at_least_9=1 last=64 falls_back=false last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=remounted publishes=0 peak=0 rounds_at_least_3=0 rounds_at_least_5=0 rounds_at_least_9=0 last=0 falls_back=true last_window_at_most_4=true recurs=false
name=q10_many_rounds item=files_1000_units_100__p1 at=same_mount/W1/1 rounds=64 release_chain_depth=85 release_chain_nodes=431 active_leaves=[273, 273] rewritten_nodes=415
name=q10 item=files_1000_units_100__p1 chain_explained_ratio=0.000 release_exit_share=1.000 placement_exit_share=0.000
name=f2 item=files_1000_units_100__p1 rounds_at_least_5=6
name=q7 item=files_1000_units_100__p1 largest_round=3036 sum_of_rounds=60850 sum_over_final_roles=355.848 premise_ratio=0.045 premise=holds memory_bytes=48056 maintenance_steps=19458 unread=0
name=q8 item=files_1000_units_100__p1 state=same_mount first_publish=2701726 peak=2701726 between_rounds_first_publish=61910 between_rounds_peak=61910 unread=0
name=q8 item=files_1000_units_100__p1 state=remounted first_publish=unread peak=unread between_rounds_first_publish=unread between_rounds_peak=unread unread=0
```

**What shape of history makes the fixed point take many rounds** (the four points that Q1 judges "many" and the one that hits the limit, one line each for p1 and p2a; the p1 peaks 11, 22, 29 and 38 are void, kept as a lead only, and the line that hit the limit is the refusal of F1, reported as it is):

```text
name=q1 item=data_units_10000__p1 point=data_units_10000 arm=p1 peak_rounds=11 at=//0 verdict=many first_overwrite_rounds=3 window_rest_peak_rounds=3 history_peak_rounds=11
name=q1 item=data_units_10000__p2a point=data_units_10000 arm=p2a peak_rounds=3 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_10000__p1 point=files_10000 arm=p1 peak_rounds=22 at=same_mount/W1/1 verdict=many first_overwrite_rounds=22 window_rest_peak_rounds=6 history_peak_rounds=6
name=q1 item=files_10000__p2a point=files_10000 arm=p2a peak_rounds=4 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=4
name=q1 item=files_1000_units_100__p1 point=files_1000_units_100 arm=p1 peak_rounds=64 at=same_mount/W1/1 verdict=hits_the_limit first_overwrite_rounds=64 window_rest_peak_rounds=0 history_peak_rounds=22
name=q1 item=files_1000_units_100__p2a point=files_1000_units_100 arm=p2a peak_rounds=4 at=remounted/W2/13 verdict=not_many first_overwrite_rounds=3 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_1000_units_10__p1 point=files_1000_units_10 arm=p1 peak_rounds=29 at=same_mount/W1/1 verdict=many first_overwrite_rounds=29 window_rest_peak_rounds=3 history_peak_rounds=5
name=q1 item=files_1000_units_10__p2a point=files_1000_units_10 arm=p2a peak_rounds=3 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_30000__p1 point=files_30000 arm=p1 peak_rounds=38 at=same_mount/W1/1 verdict=many first_overwrite_rounds=38 window_rest_peak_rounds=7 history_peak_rounds=8
name=q1 item=files_30000__p2a point=files_30000 arm=p2a peak_rounds=5 at=//0 verdict=not_many first_overwrite_rounds=3 window_rest_peak_rounds=4 history_peak_rounds=5
```

All `name=q1` lines by arm and class (command and original output):

```text
$ grep '^name=q1 ' e206-fixed-point-cost-report-2026-11-09.out | sed -E 's/.*arm=([a-z_0-9]+) .*verdict=([a-z_]+) .*/\1 \2/' | sort | uniq -c
      3 closure_off not_many
      1 p1 hits_the_limit
      4 p1 many
     11 p1 not_many
     16 p2a not_many
      1 single_step many
      2 single_step not_many
```

Where the peak round count falls (readings of p1, void, kept as a lead only): the peaks of the four p1 points `files_10000`, `files_1000_units_10`, `files_30000` and `files_1000_units_100` are all at `same_mount/W1/1` (the first overwrite in the measurement window). The peak 6 of `files_1000_unit_area_2_pow_24__p1` is at `same_mount/W2/16`; the 4 of `data_units_1000__p1` is at `same_mount/W3/2`; the 11 of `data_units_10000__p1` is in the history (`at=//0` is the history segment, with no window position). Same mount against remount:

```text
name=sensitivity_mount point=files_1000 same_mount_peak=4 remounted_peak=4
name=sensitivity_mount point=files_10000 same_mount_peak=22 remounted_peak=6
```

How it changes publish after publish (Q9, the four many-round points of p1 and the one that hits the limit; readings of p1, void, kept as a lead only):

```text
name=q9 item=data_units_10000__p1 state=same_mount publishes=105 peak=5 rounds_at_least_3=14 rounds_at_least_5=1 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=data_units_10000__p1 state=remounted publishes=105 peak=3 rounds_at_least_3=18 rounds_at_least_5=0 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_10000__p1 state=same_mount publishes=105 peak=22 rounds_at_least_3=52 rounds_at_least_5=13 rounds_at_least_9=1 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_10000__p1 state=remounted publishes=105 peak=6 rounds_at_least_3=51 rounds_at_least_5=10 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=same_mount publishes=1 peak=64 rounds_at_least_3=1 rounds_at_least_5=1 rounds_at_least_9=1 last=64 falls_back=false last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=remounted publishes=0 peak=0 rounds_at_least_3=0 rounds_at_least_5=0 rounds_at_least_9=0 last=0 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_10__p1 state=same_mount publishes=104 peak=29 rounds_at_least_3=40 rounds_at_least_5=1 rounds_at_least_9=1 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_10__p1 state=remounted publishes=104 peak=4 rounds_at_least_3=38 rounds_at_least_5=0 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_30000__p1 state=same_mount publishes=104 peak=38 rounds_at_least_3=58 rounds_at_least_5=26 rounds_at_least_9=3 last=4 falls_back=false last_window_at_most_4=false recurs=true
name=q9 item=files_30000__p1 state=remounted publishes=104 peak=7 rounds_at_least_3=54 rounds_at_least_5=25 rounds_at_least_9=0 last=2 falls_back=false last_window_at_most_4=false recurs=false
```

The release chain of the long-round publishes (Q10; `chain_explained_ratio` is the share with `|rounds − (release_chain_depth + 2)| ≤ 2`, judged only on publishes with `rounds ≥ 9`; readings of p1, void, kept as a lead only):

```text
name=q10_many_rounds item=files_10000__p1 at=same_mount/W1/1 rounds=22 release_chain_depth=21 release_chain_nodes=73 active_leaves=[44, 44] rewritten_nodes=73
name=q10_many_rounds item=files_1000_units_100__p1 at=same_mount/W1/1 rounds=64 release_chain_depth=85 release_chain_nodes=431 active_leaves=[273, 273] rewritten_nodes=415
name=q10_many_rounds item=files_1000_units_10__p1 at=same_mount/W1/1 rounds=29 release_chain_depth=28 release_chain_nodes=87 active_leaves=[47, 47] rewritten_nodes=87
name=q10_many_rounds item=files_30000__p1 at=same_mount/W1/1 rounds=38 release_chain_depth=36 release_chain_nodes=153 active_leaves=[129, 129] rewritten_nodes=155
$ grep '^name=q10 ' e206-fixed-point-cost-report-2026-11-09.out | grep -v not_seen
name=q10 item=files_10000__p1 chain_explained_ratio=1.000 release_exit_share=0.955 placement_exit_share=0.000
name=q10 item=files_1000__single_step chain_explained_ratio=0.000 release_exit_share=0.756 placement_exit_share=0.152
name=q10 item=files_1000_units_100__p1 chain_explained_ratio=0.000 release_exit_share=1.000 placement_exit_share=0.000
name=q10 item=files_1000_units_10__p1 chain_explained_ratio=1.000 release_exit_share=0.966 placement_exit_share=0.000
name=q10 item=files_30000__p1 chain_explained_ratio=0.667 release_exit_share=0.895 placement_exit_share=0.053
```

### Fork table

Each cell points to a whole line above or below; "the remaining quantities" means segments two and three of section 5 of the registration. The three fork sheet lines and the closing line at the end of the merged report:

```text
name=fork_row row=P1 enough_readings=false
name=fork_row row=P2 enough_readings=true
name=fork_row row=P3 enough_readings=false
name=report_done items=40 complete=39
```

| Candidate | State | Basis | Still missing |
|---|---|---|---|
| P1 today: no target for the fixed point rounds, only the existing limit in the source | **Flipped** (registration Q1 and F1: if any point hits the limit, P1 flips directly). That refusal of F1 is the outcome of the P1 loop, does not depend on the `single_step` control, and is reported as it is. **Void clause 1 records one loss**: the control `single_step` has `holds=false` at three points, the round-by-round readings of P1 are void and the criterion is not relaxed, see "Controls and void readings"; the 4 points Q1 judges "many" (peaks 38, 29, 22, 11), the `data_units` axis of Q2 with α 0.695 recorded "grows" and the 7 `not_held_down` points of Q4 come from void readings and serve only as a lead. `fork_row row=P1 enough_readings=false` only because `files_1000_units_100__p1` is void | `name=verdict question=q1 arm=p1 … points_at_the_limit=files_1000_units_100`; `name=q2 arm=p1 axis=data_units … alpha=0.695 band=grows`; `name=q2 arm=p1 axis=files … alpha=0.496 band=between` | Already flipped; the remaining quantities can only fill in the "what shape" half: the 33 items of segment two (including the 12 order points: sequential, clustered, scattered, create-then-delete and so on) are not run, judgeable already; the P1 readings of `files_1000_units_100` cannot be completed (stops at the limit) |
| P2 push the rounds down (P2a walks the release chain to closure each round; P2b rewrites the whole tree periodically) | **P2a is judgeable** (`enough_readings=true`): Q4 `held_down` (at 16 points p2a has at most 5 rounds, `E*` at most 3.520); Q5 fixed point agreement rate of p2a is `ratio=1.0000` on all 15 lines; Q6 write amplification is `ratio=1.0000` on 15 lines. **Net steps**: in the linear lookup form 5 points cost more and 10 points save; in the indexed form the same 5 points cost more (Q5 section below); the net steps are the difference between the P2a and P1 round-by-round readings, and the P1 side is void, kept as a lead only. P2b not run (registration: needed only if P2a flips) | `name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-`; the `__p2a` lines of `name=q5_same_final_set`, `name=q5_net_steps` and `name=q6` | Registration Q5 says "agreement rate < 1 or costs more in the indexed form, P2a flips and P2 is left with P2b"; the 5 points that cost more are all small points where P1 peaks at no more than 3 rounds, and the registration does not say whether "costs more" is judged per point or over all points, this page does not judge it. The user has decided to implement P2a first, and Q6 of P2b is not measured first |
| P3 keep the rounds, cut the cost of each (by formula, computed by the model) | **Premise of Q7 does not hold** (the input of Q7 is the round-by-round readings of the P1 arm, void, kept as a lead only): `ρ` is above 0.5 at the 15 complete p1 points (3.026 to 149.898), 15 lines of `name=f6 … premise_ratio_over_the_bound=true`; the only `ρ=0.045` is the void `files_1000_units_100__p1` (one refused publish). By Q7 of the registration "if any one fails, P3 flips" and F6 read literally: flipped. `fork_row row=P3 enough_readings=false`, for the same reason as P1 | `name=q7` and `name=f6` whole lines in the Q7 section below | The readings of `files_1000_units_100__p1` are still missing, P1 hits the limit there and they cannot be filled in; the remaining quantities cannot bring `ρ` at the 15 points below 0.5 |

#### Readings of P2a

```text
name=verdict question=q1 arm=p2a points_many=- points_at_the_limit=-
name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-
name=verdict question=q2 arm=p2a grows_on=data_units
name=verdict question=q10 arm=p2a points_the_chain_does_not_explain=-
name=q2 arm=p2a axis=files low_end=39 high_end=95 alpha=0.129 band=does_not_grow local_slopes=0.000,0.208,0.178
name=q2 arm=p2a axis=data_units low_end=39 high_end=1272 alpha=0.504 band=grows local_slopes=0.931,0.578,0.004
name=q2 arm=p2a axis=unlinks low_end=39 high_end=57 alpha=0.055 band=does_not_grow local_slopes=0.000,0.032,0.133
name=q3_final_roles arm=p2a axis=files low_end=22 high_end=84 alpha=0.194 band=does_not_grow local_slopes=0.000,0.056,0.526
name=q3_final_roles arm=p2a axis=data_units low_end=22 high_end=441 alpha=0.434 band=between local_slopes=0.714,0.570,0.017
name=q3_final_roles arm=p2a axis=unlinks low_end=22 high_end=26 alpha=0.024 band=does_not_grow local_slopes=0.000,0.000,0.073
name=f4 arm=p2a points=-
```

What `points_not_held_down=-` in the Q4 line of P2a means: at 16 points `R* ≤ 8` and `E* ≤ 4`. The largest `R*` is the 5 rounds of `files_30000__p2a` (38 rounds for P1 at the same point), the other points have 3 to 4 rounds. The `data_units` axis of `q2` is also recorded "grows" for P2a (α 0.504): the number of roles that really must be written on that axis grows by itself (α 0.434 on the `data_units` axis of `q3_final_roles`, the same for both arms), P2a cannot move it, and F5 has `triggered=false` on all three axes.

Q5: fixed points of P2a and P1 on the same history, compared one by one (command and original output):

```text
$ grep '^name=q5_same_final_set item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'ratio=1.0000 first_difference=None same_image_at_the_history_end=true final_set_differs=false'
15
$ grep '^name=q6 item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'ratio=1.0000 verdict=acceptable'
15
$ grep '^name=q5_net_steps item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -o 'linear=[a-z_]* indexed=[a-z_]*' | sort | uniq -c
      5 linear=costs_more indexed=costs_more
     10 linear=saves indexed=saves
```

The net steps, whole lines (`net_steps_linear` is the linear lookup form, `net_steps_indexed` is the indexed form; a negative number is a saving):

```text
name=q5_net_steps item=base__p2a publishes_with_at_least_3_rounds=26 net_steps_linear=1338 net_steps_indexed=936 linear=costs_more indexed=costs_more
name=q5_net_steps item=data_units_10000__p2a publishes_with_at_least_3_rounds=32 net_steps_linear=-10334626 net_steps_indexed=-10349720 linear=saves indexed=saves
name=q5_net_steps item=data_units_1000__p2a publishes_with_at_least_3_rounds=37 net_steps_linear=-3836827 net_steps_indexed=-3838860 linear=saves indexed=saves
name=q5_net_steps item=data_units_100__p2a publishes_with_at_least_3_rounds=29 net_steps_linear=1492 net_steps_indexed=1044 linear=costs_more indexed=costs_more
name=q5_net_steps item=files_10000__p2a publishes_with_at_least_3_rounds=103 net_steps_linear=-372091510 net_steps_indexed=-372193553 linear=saves indexed=saves
name=q5_net_steps item=files_1000__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-21674834 net_steps_indexed=-21681554 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_14__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-6144074 net_steps_indexed=-6147399 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_20__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-140290898 net_steps_indexed=-140297618 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_24__p2a publishes_with_at_least_3_rounds=86 net_steps_linear=-2071221471 net_steps_indexed=-2071231185 linear=saves indexed=saves
name=q5_net_steps item=files_1000_units_10__p2a publishes_with_at_least_3_rounds=78 net_steps_linear=-328136791 net_steps_indexed=-328222922 linear=saves indexed=saves
name=q5_net_steps item=files_100__p2a publishes_with_at_least_3_rounds=47 net_steps_linear=2432 net_steps_indexed=1692 linear=costs_more indexed=costs_more
name=q5_net_steps item=files_30000__p2a publishes_with_at_least_3_rounds=112 net_steps_linear=-2662924587 net_steps_indexed=-2663330038 linear=saves indexed=saves
name=q5_net_steps item=unlinks_10000__p2a publishes_with_at_least_3_rounds=51 net_steps_linear=-41746236 net_steps_indexed=-41750799 linear=saves indexed=saves
name=q5_net_steps item=unlinks_1000__p2a publishes_with_at_least_3_rounds=27 net_steps_linear=1390 net_steps_indexed=972 linear=costs_more indexed=costs_more
name=q5_net_steps item=unlinks_100__p2a publishes_with_at_least_3_rounds=23 net_steps_linear=1186 net_steps_indexed=828 linear=costs_more indexed=costs_more
```

#### Readings of P3

```text
name=q7 item=base__p1 largest_round=1391 sum_of_rounds=2647 sum_over_final_roles=120.318 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7 item=data_units_10000__p1 largest_round=1033610 sum_of_rounds=1379231 sum_over_final_roles=3127.508 premise_ratio=149.898 premise=fails memory_bytes=496920 maintenance_steps=308502 unread=0
name=q7 item=data_units_1000__p1 largest_round=85778 sum_of_rounds=107776 sum_over_final_roles=254.189 premise_ratio=78.074 premise=fails memory_bytes=51704 maintenance_steps=25396 unread=0
name=q7 item=data_units_100__p1 largest_round=7690 sum_of_rounds=10104 sum_over_final_roles=88.632 premise_ratio=12.286 premise=fails memory_bytes=6920 maintenance_steps=2894 unread=0
name=q7 item=files_10000__p1 largest_round=6089 sum_of_rounds=16116 sum_over_final_roles=191.857 premise_ratio=3.290 premise=fails memory_bytes=13224 maintenance_steps=5996 unread=0
name=q7 item=files_1000__p1 largest_round=2373 sum_of_rounds=5609 sum_over_final_roles=224.360 premise_ratio=3.379 premise=fails memory_bytes=4984 maintenance_steps=2164 unread=0
name=q7 item=files_1000_unit_area_2_pow_14__p1 largest_round=2041 sum_of_rounds=4488 sum_over_final_roles=195.130 premise_ratio=3.333 premise=fails memory_bytes=4016 maintenance_steps=1775 unread=0
name=q7 item=files_1000_unit_area_2_pow_20__p1 largest_round=2375 sum_of_rounds=5617 sum_over_final_roles=224.680 premise_ratio=3.379 premise=fails memory_bytes=4984 maintenance_steps=2164 unread=0
name=q7 item=files_1000_unit_area_2_pow_24__p1 largest_round=2617 sum_of_rounds=6244 sum_over_final_roles=231.259 premise_ratio=3.037 premise=fails memory_bytes=5640 maintenance_steps=2460 unread=0
name=q7 item=files_1000_units_100__p1 largest_round=3036 sum_of_rounds=60850 sum_over_final_roles=355.848 premise_ratio=0.045 premise=holds memory_bytes=48056 maintenance_steps=19458 unread=0
name=q7 item=files_1000_units_10__p1 largest_round=7041 sum_of_rounds=18889 sum_over_final_roles=190.798 premise_ratio=3.243 premise=fails memory_bytes=15552 maintenance_steps=7029 unread=0
name=q7 item=files_100__p1 largest_round=1457 sum_of_rounds=2823 sum_over_final_roles=128.318 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7 item=files_30000__p1 largest_round=13346 sum_of_rounds=36363 sum_over_final_roles=211.413 premise_ratio=3.026 premise=fails memory_bytes=27112 maintenance_steps=13760 unread=0
name=q7 item=unlinks_10000__p1 largest_round=2147 sum_of_rounds=6351 sum_over_final_roles=244.269 premise_ratio=3.360 premise=fails memory_bytes=6056 maintenance_steps=2626 unread=0
name=q7 item=unlinks_1000__p1 largest_round=1473 sum_of_rounds=2899 sum_over_final_roles=131.773 premise_ratio=3.440 premise=fails memory_bytes=2872 maintenance_steps=1026 unread=0
name=q7 item=unlinks_100__p1 largest_round=1395 sum_of_rounds=2651 sum_over_final_roles=120.500 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7_sum_of_rounds arm=p1 axis=files low_end=2647 high_end=16116 alpha=0.262 band=between local_slopes=0.028,0.298,0.458
name=q7_memory_bytes arm=p1 axis=files low_end=2600 high_end=13224 alpha=0.235 band=does_not_grow local_slopes=0.000,0.283,0.424
name=q7_sum_of_rounds arm=p1 axis=data_units low_end=2647 high_end=1379231 alpha=0.906 band=grows local_slopes=0.582,1.028,1.107
name=q7_memory_bytes arm=p1 axis=data_units low_end=2600 high_end=496920 alpha=0.760 band=grows local_slopes=0.425,0.873,0.983
name=q7_sum_of_rounds arm=p1 axis=unlinks low_end=2647 high_end=6351 alpha=0.127 band=does_not_grow local_slopes=0.001,0.039,0.341
name=q7_memory_bytes arm=p1 axis=unlinks low_end=2600 high_end=6056 alpha=0.122 band=does_not_grow local_slopes=0.000,0.043,0.324
$ grep '^name=f6 ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'premise_ratio_over_the_bound=true roles_changed_over_twice_the_final=false'
15
name=f6 item=files_1000_units_100__p1 premise_ratio_over_the_bound=false roles_changed_over_twice_the_final=true
```

`ρ` is `Σ_r records_changed_between_rounds_r ÷ W` of registration Q7, `premise_ratio` in the table; `premise=fails` means `ρ > 0.5`. α of `Σ_r`: 0.262 on the `files` axis (between), 0.906 on the `data_units` axis (grows), 0.127 on the `unlinks` axis (does not grow); α of the memory peak: 0.235, 0.760, 0.122. Registration Q7 needs all three axes not growing, the premise holding and memory not growing before P3 stands. These are derived: the formula of P3 is in section 5 of the registration, and the input is measured (the between-rounds difference readings) and comes from the round-by-round readings of the P1 arm, void, kept as a lead only.

### Readings for the B8 cell

E203 (Step cost of the scans before admission), revision 8 in section 12 of its registration: the one cell left of B8 (`files_10000`, overwrite in the same mount, round 1, fixed point 22 rounds, Q6 α 0.620) is recorded "unstable", and once this fixed point measurement is done it is judged again by the original criterion; section 3 "B8" of the verdict `research/prompts/e203-forks-r2-main-verification.md` hands the choice of reading, "still grows or unstable", to the user. **The original criterion** (registration of E203 (Step cost of the scans before admission): Q6, F2 and item 1 of section 8):

- Q6: one class per axis; if α ≤ 0.25 on all axes, record "B8 does not grow with the total"; if α ≥ 0.5 on any axis, record "B8 still grows with the total"; in between, no judgment.
- F2: if α ≥ 0.5 on any axis, in any mount state, on any measured publish, the B8 row flips.
- Item 1 of section 8: if the two mount states fall in different classes, that cell is recorded "unstable".

Q8 of E206 (Cost of the publish fixed point rounds) feeds the per-round inputs of each publish to the B8 column of E203 (Step cost of the scans before admission). `q8_first_publish` takes the first overwrite W1 in the measurement window (same mount), `q8_peak` takes the maximum over all publishes in the measurement window (same mount plus remount), and both readings are reported; `q8_p3_*` is the same publishes with the between-rounds difference input of P3 (registration Q7, the P3 formula of section 5). Whole lines (the two arms P1 and P2a, three axes):

```text
name=q8_first_publish arm=p1 axis=files low_end=2102 high_end=152344 alpha=0.620 band=grows local_slopes=0.045,0.475,1.340
name=q8_peak arm=p1 axis=files low_end=3763 high_end=152344 alpha=0.536 band=grows local_slopes=0.030,0.356,1.221
name=q8_p3_first_publish arm=p1 axis=files low_end=1787 high_end=16538 alpha=0.322 band=between local_slopes=0.048,0.314,0.605
name=q8_p3_peak arm=p1 axis=files low_end=2794 high_end=16538 alpha=0.257 band=between local_slopes=0.030,0.297,0.445
name=q8_first_publish arm=p1 axis=data_units low_end=2102 high_end=5958 alpha=0.151 band=does_not_grow local_slopes=0.036,0.374,0.043
name=q8_peak arm=p1 axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
name=q8_p3_first_publish arm=p1 axis=data_units low_end=1787 high_end=3449 alpha=0.095 band=does_not_grow local_slopes=0.037,0.208,0.040
name=q8_p3_peak arm=p1 axis=data_units low_end=2794 high_end=1379969 alpha=0.898 band=grows local_slopes=0.565,1.023,1.107
name=q8_first_publish arm=p1 axis=unlinks low_end=2102 high_end=12789 alpha=0.261 band=between local_slopes=0.031,0.083,0.670
name=q8_peak arm=p1 axis=unlinks low_end=3763 high_end=12789 alpha=0.177 band=does_not_grow local_slopes=0.000,0.063,0.467
name=q8_p3_first_publish arm=p1 axis=unlinks low_end=1787 high_end=6577 alpha=0.189 band=does_not_grow local_slopes=0.026,0.078,0.462
name=q8_p3_peak arm=p1 axis=unlinks low_end=2794 high_end=6577 alpha=0.124 band=does_not_grow local_slopes=0.001,0.045,0.326
name=q8_first_publish arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526
name=q8_peak arm=p2a axis=files low_end=3763 high_end=15440 alpha=0.204 band=does_not_grow local_slopes=0.030,0.350,0.233
name=q8_p3_first_publish arm=p2a axis=files low_end=1787 high_end=16185 alpha=0.319 band=between local_slopes=0.048,0.312,0.598
name=q8_p3_peak arm=p2a axis=files low_end=2794 high_end=16185 alpha=0.254 band=between local_slopes=0.030,0.181,0.551
name=q8_first_publish arm=p2a axis=data_units low_end=2102 high_end=4040 alpha=0.095 band=does_not_grow local_slopes=0.036,0.201,0.047
name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
name=q8_p3_first_publish arm=p2a axis=data_units low_end=1787 high_end=3430 alpha=0.094 band=does_not_grow local_slopes=0.037,0.206,0.040
name=q8_p3_peak arm=p2a axis=data_units low_end=2794 high_end=1379969 alpha=0.898 band=grows local_slopes=0.565,1.023,1.107
name=q8_first_publish arm=p2a axis=unlinks low_end=2102 high_end=7184 alpha=0.178 band=does_not_grow local_slopes=0.031,0.083,0.420
name=q8_peak arm=p2a axis=unlinks low_end=3763 high_end=7184 alpha=0.094 band=does_not_grow local_slopes=0.000,0.063,0.217
name=q8_p3_first_publish arm=p2a axis=unlinks low_end=1787 high_end=4062 alpha=0.119 band=does_not_grow local_slopes=0.026,0.078,0.252
name=q8_p3_peak arm=p2a axis=unlinks low_end=2794 high_end=4186 alpha=0.059 band=does_not_grow local_slopes=0.001,0.045,0.130
```

The α and classes copied out of the whole lines above (not recomputed):

| Arm | Axis | `q8_first_publish` | `q8_peak` |
|---|---|---|---|
| P1 | `files` | 0.620 grows (2102 → 152344) | 0.536 grows (3763 → 152344) |
| P1 | `data_units` | 0.151 does not grow | 0.978 grows (3763 → 3231038) |
| P1 | `unlinks` | 0.261 between | 0.177 does not grow |
| P2a | `files` | 0.289 between (2102 → 15440) | 0.204 does not grow (3763 → 15440) |
| P2a | `data_units` | 0.095 does not grow | 0.978 grows (3763 → 3231038) |
| P2a | `unlinks` | 0.178 does not grow | 0.094 does not grow |

By the letter of the original criterion:

- **P1** (the input is the round-by-round readings of the P1 arm, void, kept as a lead only): under both readings some axis has α ≥ 0.5 (`files` 0.620 for `q8_first_publish`, `files` 0.536 and `data_units` 0.978 for `q8_peak`), Q6 records "B8 still grows with the total" and F2 hits. The 2102 → 152344 on the `files` axis is the same cell and the same numbers as on the page of E203 (Step cost of the scans before admission) (`files_10000`, overwrite in the same mount, round 1, 22 rounds).
- **P2a**: under the `q8_first_publish` reading no axis reaches 0.5, but 0.289 on the `files` axis is above 0.25, which fails "all axes ≤ 0.25", so it is in neither class (the original criterion gives no conclusion for this class); for the same publish P1 is 152344 and P2a is 15440. Under the `q8_peak` reading the `data_units` axis has α 0.978 ≥ 0.5, Q6 records "still grows" and F2 hits.
- The peak 3231038 on the `data_units` axis is exactly the same in the two arms P1 and P2a (`name=q8 item=data_units_10000__p1` and `__p2a`, `state=same_mount … peak=3231038`); the publish with the largest `final_roles` in the measurement window is the first one of `same_mount` W2 (66 roles, 3 rounds in both arms, worked out from the `name=settle` lines, **not checked** which one `q8_peak` takes); P2a cannot move it, it does not come from the number of fixed point rounds. On the page of E203 (Step cost of the scans before admission) all 6 cells of the D axis of Q6 are "does not grow"; `q8_peak` of E206 (Cost of the publish fixed point rounds) takes all publishes in the measurement window (including the W2 cold file and the W3 create-delete), a publish set different from the five-round overwrite and unlink of E203 (Step cost of the scans before admission), so the two measurement bases cannot be compared directly.
- The `q8_p3_*` lines (P3 between-rounds difference input, the P1 arm side void, kept as a lead only): on the history of the P1 arm, first_publish 0.322 and peak 0.257 on the `files` axis are both between; 0.095 does not grow and 0.898 grows on the `data_units` axis; 0.189 and 0.124 on the `unlinks` axis both do not grow; these are derived.
- The remount column is in the per-point lines `name=q8 … state=remounted`, where the two arms are the same or P2a is smaller; the remount first_publish of `files_10000` is 8818, against 152344 (P1) and 15440 (P2a) in the same mount. Whether item 1 of section 8, "the two mount states fall in different classes, record unstable", triggers at this cell needs α of `state=remounted` computed separately; the report does not print that line, **not judged**.

This section only states the numbers and the original criterion; "choose a reading", "B8 still grows or unstable" and "which of P1 and P2a" are not here.

### Controls and void readings

The outcome of each void clause of section 11 of the registration (whole lines above and below; the command and original output of each clause are in the report of the experiment runner):

| Clause | Reading | Outcome |
|---|---|---|
| 1 `single_step` control | `holds=false` at three points | **Records one loss**: the round-by-round readings of P1 are void as a whole by the letter, the criterion is not relaxed; the cause is below |
| 2 `closure_off` control | `holds=true` at three points | P2a is not void |
| 3 P3 control | 38 lines of `control_p3` with `holds=true` | Not void |
| 4 negative control | `holds=true` on all three axes of p1 and p2a; six p2b lines `result=unread` (P2b not run) | p1 and p2a are not void |
| 5 `--build plain` disk comparison | `same=true` at the two points `base` and `files_1000` | Not void |
| 6 missing `item_done` | `files_1000_units_100__p1` | This point and this arm are void |
| 7 reconciliation | 8 of 24 lines `equal=false` (all W2, 30 against 32) | Found to be a criterion that undercounts, not a counting error; group 2 is not void, see below |

```text
name=anchor_measured anchor_identifier=files_10000_first_overwrite rounds=22 expected_rounds=22 round_work=1201 expected_round_work=1201 per_round_roles=[11, 24, 28, 32, 36, 40, 42, 44, 46, 50, 54, 58, 62, 64, 66, 70, 74, 76, 78, 80, 82, 84] matches=true
name=anchor_measured anchor_identifier=files_10000_second_overwrite rounds=Some(2) expected_rounds=2 matches=true
name=anchor_measured anchor_identifier=base_first_overwrite rounds=2 expected_rounds=2 round_work=21 expected_round_work=21 per_round_roles=[7, 14] matches=true
name=control_single_step point=base holds=false voids=p1_round_readings_if_false
name=control_closure_off point=base holds=true voids=p2a_if_false
name=control_single_step point=files_100 holds=false voids=p1_round_readings_if_false
name=control_closure_off point=files_100 holds=true voids=p2a_if_false
name=control_single_step point=files_1000 holds=false voids=p1_round_readings_if_false
name=control_closure_off point=files_1000 holds=true voids=p2a_if_false
name=void_5 point=base plain=Some("a97bb56210604ecb68cd4f689ec0f525a646552bc144c6b0c303b5d4079306bf") today=Some("a97bb56210604ecb68cd4f689ec0f525a646552bc144c6b0c303b5d4079306bf") same=true voids=all_measured_if_false
name=void_5 point=files_1000 plain=Some("c82a604c4590edda8a2f38f2f8bbad0096f5ccae6508337bf5bf6f574025af19") today=Some("c82a604c4590edda8a2f38f2f8bbad0096f5ccae6508337bf5bf6f574025af19") same=true voids=all_measured_if_false
name=control_negative arm=p1 axis=files low_final_roles=10 high_final_roles=13 alpha=0.038 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p1 axis=data_units low_final_roles=10 high_final_roles=10 alpha=0.000 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p1 axis=unlinks low_final_roles=10 high_final_roles=14 alpha=0.049 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p2a axis=files low_final_roles=10 high_final_roles=13 alpha=0.038 rounds=2 holds=true voids=p2a_if_false
name=control_negative arm=p2a axis=data_units low_final_roles=10 high_final_roles=10 alpha=0.000 rounds=2 holds=true voids=p2a_if_false
name=control_negative arm=p2a axis=unlinks low_final_roles=10 high_final_roles=14 alpha=0.049 rounds=2 holds=true voids=p2a_if_false
$ grep '^name=control_p3 ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'holds=true'
38
```

**The `single_step` control has `holds=false` at three points.** The four conditions of `single_step_control_holds` in the model (`e206_fixed_point_cost.rs:1259`): the fixed points are the same each time, every settled fixed point has at least as many rounds as nodes, at least one publish in the measurement window takes more rounds than p1, and `image_sha256` at the end of the history is the same. The experiment runner wrote a separate script that checks them one by one (the script `single-step-control-check.py` is in the experiment runner's scratch directory, started from the `research/results` directory; in brackets are the history index, the p1 rounds, the `single_step` rounds and the node count, and for a window entry the op, op_index, rounds and node count):

```text
$ python3 -I single-step-control-check.py
point=base history_calls=44/44 same_set_history=True window_publishes=210/210 same_set_window=True window_slower=210 image_sha256_equal=True settled_with_rounds_below_final_nodes=1 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5')]
point=files_100 history_calls=134/134 same_set_history=True window_publishes=208/208 same_set_window=True window_slower=208 image_sha256_equal=True settled_with_rounds_below_final_nodes=1 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5')]
point=files_1000 history_calls=1043/1043 same_set_history=True window_publishes=208/208 same_set_window=True window_slower=208 image_sha256_equal=True settled_with_rounds_below_final_nodes=2 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5'), (917, 'p1', '3', 'single_step', '8', 'final_nodes', '9')]
```

Three conditions hold at all three points: the fixed points are the same each time (`final_set_hash` and the role count, in the history and in the window), every publish in the window is slower, and the disk images at the end of the history are the same; the one that fails is the second: `base` and `files_100` each have 1 settled fixed point, `files_1000` has 2, where the rounds of `single_step` are fewer than its nodes. Index 0 is the first fixed point in the history (both arms have 2 rounds, 5 nodes and the same hash, so `single_step` did not slow it down; whether it is the same-shaped line publish fixed point at `:1903` was not checked, and the patch for that one is not in section 5 of the registration); index 917 of `files_1000` has 3 rounds for p1, 8 for `single_step` and 9 nodes: the patch merges only one node per round and the nodes covered by the plan of round 1 are not in the difference set, hence 8 rounds for 9 nodes, so the assumption of the registration "at least one node per round" does not match the way the loop is written. This cause was derived by the experiment runner from reading the code; the registration does not say it and no other reading proves it.

Judgment (the main agent, `records/2026-10-13-里程碑四预开发.md` line 1579): `holds=false` at three points (lines 569, 571 and 573 of the merged report, line numbers in the original report; lines 607, 609 and 611 in the retained report of 2026-11-13) literally triggers void clause 1, it records one loss, the round-by-round readings of P1 are void and the criterion is not relaxed; that refusal of F1 is the outcome of the P1 loop and does not depend on this control.

**The 8 lines of the W2 reconciliation with `equal=false`** (30 against 32, the same pair of numbers for p1, p2a, `single_step` and `closure_off` of `base` and `files_100`):

```text
name=reconciliation item=base__closure_off op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__p1 op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__p2a op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__single_step op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__closure_off op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__p1 op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__p2a op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__single_step op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
```

The `plain` build inserts no counters, and its W2 `image_changed_unit_slots=30` equals the disk reading of p1 (`research/results/e206-fixed-point-cost-base-plain-2026-11-09.out`), so the disk side is 30 and the `slots_placed` the counter printed is 32 (W1 and W3 are both equal). Void clause 7 of the registration first checks both sides by stop clause 2 and voids only if a counting error is found. Both sides were checked: the whole line of probe task 11147 (copied from line 1580 of `records/2026-10-13-里程碑四预开发.md`) equals, field by field, the expectation the investigator wrote before the run:

```text
E206PROBE name=op_summary item=base__p1 phase=window state=same_mount op=W2 op_index=1 counter_placed=32 allocator_placed=32 image_changed=30 placed_slots=32 placed_unchanged=2 changed_not_placed=0 placed_changed_without_a_write=0 placed_unchanged_without_a_write=0 placed_unchanged_not_all_zero=0
```

Counterfactual task 11148 (the first content of W2 made longer so that the tail slots are not all zero): `counter_placed=32 allocator_placed=32 image_changed=32 placed_slots=32 placed_unchanged=0`. Judgment: it is not a counting error but a criterion that undercounts, because `image_changed_unit_slots` measures the slots whose content changed, and rewriting an all-zero tail slot does not count as a change (32 slots placed, 2 unchanged), so group 2 is not void; changing the criterion would be a registration revision, which this time was not done. For the 32 lines of W3, `counter_placed=116 allocator_placed=34 image_changed=116`: `counter` equals `image`, and the smaller `allocator` is the allocator range the probe takes (derived, not checked). The probe copy has been deleted.

**Stop clause 3 (the source sha256 recorded in the artifact header does not match the registration)**: of the 40 artifacts, the headers of 34 record the same `transaction.rs` and `allocator.rs` as the registration; 6 artifacts, p1 and p2a at the three points `files_30000`, `files_1000_units_100` and `files_1000_unit_area_2_pow_24`, record other values (`transaction.rs` is `602f9bbb…` everywhere, `allocator.rs` is `3c91258f…` for p1 and `ff676177…` for p2a), and within each of the three points the `allocator.rs` of p1 and p2a also differ. Cause: `header_for` in the entry `scripts/experiment-apparatus/e206-fixed-point-cost-run.sh:170` reads `$source_tree` (the repository), so the header records the repository at the moment the header was written, not the copy that was compiled in, and those files of `crates/` were changed in the repository afterwards. The experiment runner checked the copies: in the three copy groups (`small`, `bounds`, `ends`) these three files all equal the original of `git show HEAD:` with `research/results/e206-fixed-point-cost-counters-diff-2026-11-09.diff` applied (1989 lines, the diff of the same name in the three copy groups is byte-identical), and the sha256 of the three files at HEAD is exactly what the registration wrote. So what was compiled in is the source of the registration: the header is wrongly recorded, not wrongly compiled. This evidence is "the present state of the three copy groups", not a record from compile time. The patch of the investigator (make the header read the copy) was not applied: once it reads the copy, the `counts` artifacts trigger stop clause 3 on every run. The debt goes to a registration revision.

### Criteria and failure clauses

The verdict lines (whole lines, all `name=verdict` in the merged report):

```text
name=verdict question=q1 arm=p1 points_many=data_units_10000,files_10000,files_1000_units_10,files_30000 points_at_the_limit=files_1000_units_100
name=verdict question=q4 arm=p1 verdict=not_held_down points_not_held_down=data_units_10000,files_10000,files_1000_unit_area_2_pow_24,files_1000_units_10,files_1000_units_100,files_30000,unlinks_10000
name=verdict question=q2 arm=p1 grows_on=data_units
name=verdict question=q10 arm=p1 points_the_chain_does_not_explain=files_1000_units_100,files_30000
name=verdict question=q1 arm=p2a points_many=- points_at_the_limit=-
name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-
name=verdict question=q2 arm=p2a grows_on=data_units
name=verdict question=q10 arm=p2a points_the_chain_does_not_explain=-
```

- F1: hit (above), reported as it is, independent of the `single_step` control.
- F2 (the source comment "settles in three or four rounds"): hit, which only records that the comment and the readings disagree and does not judge an arm. Number of publishes with `rounds ≥ 5`, for each point of the p1 arm (those with `0` are not listed; including the `single_step` and `closure_off` controls; the readings of p1 and `single_step` are void, kept as a lead only):

```text
$ grep '^name=f2 ' e206-fixed-point-cost-report-2026-11-09.out | grep -v 'rounds_at_least_5=0'
name=f2 item=base__single_step rounds_at_least_5=253
name=f2 item=data_units_10000__p1 rounds_at_least_5=2
name=f2 item=files_10000__p1 rounds_at_least_5=232
name=f2 item=files_1000__closure_off rounds_at_least_5=4
name=f2 item=files_1000__p1 rounds_at_least_5=4
name=f2 item=files_1000__single_step rounds_at_least_5=1250
name=f2 item=files_1000_unit_area_2_pow_14__p1 rounds_at_least_5=7
name=f2 item=files_1000_unit_area_2_pow_20__p1 rounds_at_least_5=4
name=f2 item=files_1000_unit_area_2_pow_24__p1 rounds_at_least_5=13
name=f2 item=files_1000_units_100__p1 rounds_at_least_5=6
name=f2 item=files_1000_units_10__p1 rounds_at_least_5=2
name=f2 item=files_100__single_step rounds_at_least_5=341
name=f2 item=files_30000__p1 rounds_at_least_5=1202
name=f2 item=files_30000__p2a rounds_at_least_5=1
name=f2 item=unlinks_10000__p1 rounds_at_least_5=29
```

- F3 (P2a is not the original fixed point): not hit; the fixed points of P2a and P1 are the same on all 15 lines and the disk images at the end of the history are the same (the commands in the section "Readings of P2a").
- F4 (P2a cannot reach the round count): not hit, `name=f4 arm=p2a points=-`.
- F5 (the part shared by the three arms grows): not hit, `triggered=false` on all three axes; α 0.434 on the `data_units` axis of `q3_final_roles` is between.
- F6 (premise of P3 does not hold): hit (the readings come from the round-by-round readings of the P1 arm, void, kept as a lead only); `ρ` is above 0.5 at the 15 complete p1 points; `roles_changed_over_twice_the_final` is `false` at the 15 points.
- F7 (the release chain does not explain the long rounds): hit (from the round-by-round readings of the P1 arm, void, kept as a lead only), `name=verdict question=q10 arm=p1 points_the_chain_does_not_explain=files_1000_units_100,files_30000`: the 64 rounds of `files_1000_units_100` are cut by the limit (`release_chain_depth=85`, +2 exceeds 64), and `chain_explained_ratio=0.667` for `files_30000` (`release_exit_share=0.895`, `placement_exit_share=0.053`). `chain_explained_ratio=1.000` for `files_10000` and `files_1000_units_10`. The assumption of item 12 of section 4 of the registration (rounds roughly equal the active leaves per disk) did not match at these two points: the prediction (about 74 for `files_30000`, about 247 for `files_1000_units_100`) against the measured 38 rounds (void, kept as a lead only) and the 64 rounds refused (F1, reported as it is), with 129 and 273 active leaves.
- F8 (the history cannot be walked to the end): the only refusal is that of F1, which was refused in the measurement window, not in the history; `history_refused` has 0 lines in the 40 artifacts (`grep -c 'name=history_refused'`, no file is non-zero).

### Rerun

```bash
# Merged report: recompute from the 40 artifacts and compare byte for byte with the retained artifact (exact); the history is not rerun. The registration row is written and has not been run yet
bash scripts/experiment-apparatus/replay.sh --item E206.1
# One of the 40 items: the measured path, the copy is made by the entry
bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --item files_10000__p1 --work <new directory>
bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --list-items
# The original command of the merged report
cargo run --release --bin e206-fixed-point-cost -- report <40 artifacts>
```

Each command is submitted by the main agent through `task-server/task submit`; the work directory is new and under a shared root. A whole run of segment one also has `--segment first`.

### Measurement basis

- Merged report (the one the rerun table compares): `research/results/e206-fixed-point-cost-report-2026-11-13.out` (792 lines, the whole report recomputed by the apparatus of 2026-11-13, prefix `E7RESULT ` on every line); the one of task 11102: `research/results/e206-fixed-point-cost-report-2026-11-09.out` (753 lines, log of task 11102); retained form `research/results/e206-fixed-point-cost-report-2026-11-09-r2.out` (prefix `E7RESULT ` added to every line and `name=done emitted=754` at the end, derived from the previous file with the same awk, not a rerun).
- Counter insertion diff: `research/results/e206-fixed-point-cost-counters-diff-2026-11-09.diff` (1989 lines, `diff_file=` in the headers of the 38 artifacts of the counting build); `research/results/e206-fixed-point-cost-counters-diff-2026-11-09-r2.diff` is an empty file, the diff of the `plain` build (headers of the two `plain` artifacts have `diff_file=e206-fixed-point-cost-counters-diff-2026-11-09-r2.diff`, `diff_lines=0`).
- The 40 artifacts and their completion markers are in the work tree (none archived):

```text
$ ls e206-fixed-point-cost-*-2026-11-09.out | grep -v 'report'
e206-fixed-point-cost-base-closure_off-2026-11-09.out
e206-fixed-point-cost-base-p1-2026-11-09.out
e206-fixed-point-cost-base-p2a-2026-11-09.out
e206-fixed-point-cost-base-plain-2026-11-09.out
e206-fixed-point-cost-base-single_step-2026-11-09.out
e206-fixed-point-cost-data_units_10000-p1-2026-11-09.out
e206-fixed-point-cost-data_units_10000-p2a-2026-11-09.out
e206-fixed-point-cost-data_units_1000-p1-2026-11-09.out
e206-fixed-point-cost-data_units_1000-p2a-2026-11-09.out
e206-fixed-point-cost-data_units_100-p1-2026-11-09.out
e206-fixed-point-cost-data_units_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_10000-p1-2026-11-09.out
e206-fixed-point-cost-files_10000-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000-closure_off-2026-11-09.out
e206-fixed-point-cost-files_1000-p1-2026-11-09.out
e206-fixed-point-cost-files_1000-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000-plain-2026-11-09.out
e206-fixed-point-cost-files_1000-single_step-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_14-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_14-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_20-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_20-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_24-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_24-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_units_100-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_units_10-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_units_10-p2a-2026-11-09.out
e206-fixed-point-cost-files_100-closure_off-2026-11-09.out
e206-fixed-point-cost-files_100-p1-2026-11-09.out
e206-fixed-point-cost-files_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_100-single_step-2026-11-09.out
e206-fixed-point-cost-files_30000-p1-2026-11-09.out
e206-fixed-point-cost-files_30000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_10000-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_10000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_1000-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_1000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_100-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_100-p2a-2026-11-09.out
```

- Points: `U` is 2^17 by default, the axes `u`, `D` and `F` are in the sampling table of section 5 of the registration; history order and the measurement windows W1 to W4 are in section 5 of the registration. The definitions of "one fixed point", "round", "round work" and "effective rounds" are word for word in section 1 of the registration, "readings".
- Both the roles rewritten per round `A_r` and the node count `N_r` are reported; the criteria only use `A_r`.
- Wall clock is not measured (reading 8 of the registration).

### What it cannot answer

- It measures only on the 6 MiB ring, two disks and the history shapes of section 5 of the registration; the 33 items of segment two (12 order points: sequential, clustered, scattered, create-then-delete, one inode at a time, remount every 1000; plus P2b16, P2b256 and `whole` at the end points of three axes) and the full run points of segment three are not run, judgeable already, so whether the "what shape" question flips the verdict of segment one on those orders, this page cannot answer.
- P2b is not measured: the write amplification of P2b (Q6) has only the 15 lines of P2a (ratio 1.0000, P2a writes no more to begin with), and the write amplification of the two P2b classes has no readings.
- P3 is a formula, not an implementation: `ρ`, the sum of between-rounds differences and the memory of Q7 are computed by the model from the round-by-round readings, and "touch only what changed each round" has never been implemented.
- The round-by-round readings are void: the `single_step` control has `holds=false` at three points (previous section), void clause 1 records one loss, the round-by-round readings of P1 are void as a whole and serve only as a lead, and the criterion is not relaxed. The refusal of F1 is an observed outcome (`window_refused`); it depends on the unmodified loop of the P1 arm and not on the patch of the `single_step` arm.
- The 32 against 30 of the W2 reconciliation was found to be a criterion that undercounts, not a counting error (previous section); the criterion was not changed, and changing it would be a registration revision. For W3, the `allocator_placed=34` against `counter_placed=116` is the allocator range the probe takes, derived, not checked.
- `source_sha256` in the header records the repository at the time of writing the header on 6 artifacts, not the compiled-in copy (previous section); to find the source at compile time one can only rely on the present state of the three copy groups; the patch that makes the header read the copy was not applied, and the owed check goes to a registration revision.
- The P1 readings of `files_1000_units_100` cannot be completed: the fixed point does not settle by 64 rounds, the publish after the error was returned did not happen, and P1 at this point has only this one for Q5, Q6, Q7 and Q9 (a single publish).
- It does not judge whether a conclusion can overturn or establish a decision, and does not choose among P1, P2 and P3.
- The B8 α of the remount was not computed separately (previous section).
- The arm definitions hold only for the tree recorded in `source_sha256` in the header of each of the 40 artifacts: the allocator and fixed point under test have since been reimplemented (the fixed point of `transaction.rs` now merges into the rewrite set through the product's own release chain closure), and E206 is only scanned back, not rerun; on today's tree `p2a` is the product as written today, while `p1` and `closure_off` remove the closure the product already has and are no longer "today's code". An arm run on today's tree is not compared with the same-named arm of these 40 artifacts (registration revision 4).

### Paths and conclusions register

| # | Path | How computed | Source location | Shared items read |
|---|---|---|---|---|
| 1 | The measured path | Counters inserted into a copied implementation, the history test cases run, per-round readings printed; the samples are the enumerated points of the sampling table in section 5 of the registration (16 points of segment one, p1 and p2a each, 3 points for the controls), because the three axes span three orders of magnitude plus shape and geometry sensitivity, not because the domain was swept | `scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs:640 run_point_under_arm` | The code under test itself; 812 slots per leaf, fanout 169, limit 64 and two disks all come from the implementation (16 `name=constants` lines of the artifact headers are the same: `leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2`) |
| 2 | The derived path | Local formulas take the readings of the measured path and compute P3 and the B8 column; the abstract fixed point is computed separately on the graphs of section 7 of the registration | `scripts/experiment-apparatus/e206_fixed_point_cost.rs:1084 p3_reading` | 812 slots per leaf, fanout 169, limit 64 and two disks as local constants, compared one by one with the artifact header; the per-round readings; the B8 formulas of the registration of E203 (Step cost of the scans before admission) |

The comparison of the two paths is in `model_mismatches` (`e206_fixed_point_cost.rs:1327`: round work equals the sum of the per-round roles, the artifact header constants equal the local ones); the merged report has no `name=stop_condition_2` line. The existing readings of item 2 of ② in section 7 of the registration are reproduced on these artifacts (the three lines `name=anchor_measured … matches=true` above): `files_10000__p1` in the same mount, W1 first publish, 22 rounds, round work 1201, per-round roles 11 to 84; second publish, 2 rounds; `base__p1` 2 rounds, round work 21, roles 7 and 14. The four B8 numbers of E203 (Step cost of the scans before admission) (2102, 2098, 152344, 4226) have no corresponding lines in this report and are checked by the `e203-anchors` subcommand of the model, **not run this time**.

### 2026-11-14 re-run artifacts

The re-run now judges byte-identical; the new artifacts are in the repository and the replay-table rows point to the new names (the E129 device probe has no replay-table row):

- `research/results/e206-fixed-point-cost-report-2026-11-14.out` (byte-identical to `e206-fixed-point-cost-report-2026-11-13.out`)

### Affected decisions

| Decision item | Relation | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 14 | supports | 2026-11-13 not affected: that day only the retained form of the merged report was replaced by the whole report recomputed by today's apparatus (38 more lines `name=by_delta_pooled_fallback`, every other line byte-identical); readings, artifacts and conclusions are unchanged; the basis of this item still cites this page; 2026-11-12 not affected: that day only "What it cannot answer" was extended to say each arm is read against the tree recorded in its own artifact header, and the readings, artifacts and conclusions are unchanged; the 2026-11-09 look-back was a change: the bullet "Publish path fixed point" in the settlement of this item is rewritten so that the release chain is walked to its closure in one go (`merge_into_the_rewrite_set_walking_the_release_chain`), and the basis cites this page: in the arm that walks the closure the fixed points are identical one by one to those without the closure and no unit is written more, the rounds stay within the registered thresholds, and the net steps are counted over all points, where this arm stands (the user settled it on 2026-11-09); the readings, artifacts and conclusions of this page are unchanged |
| D8 (Core Index Structure) settled item 16 | groundwork | 2026-11-13 not affected: that day only the retained form of the merged report was replaced by the whole report recomputed by today's apparatus (38 more lines `name=by_delta_pooled_fallback`, every other line byte-identical); readings, artifacts and conclusions are unchanged; 2026-11-12 not affected: that day only "What it cannot answer" was extended to say each arm is read against the tree recorded in its own artifact header, and the readings, artifacts and conclusions are unchanged; the reason of the 2026-11-09 look-back still holds: this page only records readings and neither the settlement nor the scope is changed; the sentence in its scope that the step cost before admission does not grow with the total containers and records on disk depends on the fixed point: P1 hits the limit at `files_1000_units_100` (F1, reported as it is), while the 22 and 38 rounds on the `files` axis and "B8 still grows in the P1 arm" (section on B8 above) come from the void round-by-round readings of P1 and serve only as a lead, and in the P2a arm the `files` axis is between; the basis of this item does not cite E206 (Cost of the publish fixed point rounds), and whether it should be raised to support or overturn goes to the main agent |
| D28 (Mount-Time Commitment Volume) settled item 1 | groundwork | 2026-11-13 not affected: that day only the retained form of the merged report was replaced by the whole report recomputed by today's apparatus (38 more lines `name=by_delta_pooled_fallback`, every other line byte-identical); readings, artifacts and conclusions are unchanged; 2026-11-12 not affected: that day only "What it cannot answer" was extended to say each arm is read against the tree recorded in its own artifact header, and the readings, artifacts and conclusions are unchanged; the reason of the 2026-11-09 look-back still holds: the clause is not changed; the fixed point described by the wiring of this item runs between two acceptances, and P1 running the full 64 rounds at `files_1000_units_100` and returning `AllocationRecordTreeRewriteSetDidNotSettle` is a refusal between the two acceptances of the wiring of this item; the basis of this item does not cite E206 (Cost of the publish fixed point rounds), and whether it should be raised to support or overturn goes to the main agent |

## Historical Versions

### 2026-11-13

- Retained form of the merged report renewed: `replay.sh E206` of task 16216 judged "does not match" because the apparatus later added a `name=by_delta_pooled_fallback` line for each point; today's apparatus recomputed 792 lines, `research/results/e206-fixed-point-cost-report-2026-11-13.out` is new, the registration line now points to it, and `-r2` and the original report of 11-09 are kept. Compared with `-r2`, it has only 38 more lines `name=by_delta_pooled_fallback` and a different count on the closing line, every other line is byte-identical. The two items in the line `question=q4 arm=p1` on this page were put in the order of the new report (in the original report of 11-09 `files_1000_units_100` comes before `files_1000_units_10`, in `-r2` and the new report it is the other way round, the same set); for the three lines of void clause 1 the line numbers 607, 609 and 611 of the retained report were added. Readings, artifacts and conclusions are unchanged; the decision table was reviewed line by line up to 2026-11-13.

### 2026-11-12

- Added one item to "What it cannot answer": the arm definitions hold only for the tree recorded in `source_sha256` in the header of each of the 40 artifacts; on today's tree `p1` and `closure_off` mean "remove the release chain closure the product already has", and E206 is only scanned back, not rerun. The readings, artifacts and conclusions did not change. Basis: registration revision 4.

### 2026-11-09

- Changed to current values: the 40 artifacts of segment one and the merged report (task 11102) are in the page; F1 hit, P1 flips; P2a is judgeable; the premise of P3 does not hold; `files_1000_units_100__p1` is void and P2a has no comparison partner at that point; the user decided "implement P2a now", and the 33 items of segment two are not run (judgeable already); the readings for the B8 cell are in the page; controls and void readings are in the page: the `single_step` control has `holds=false` at three points, void clause 1 records one loss, the round-by-round readings of P1 are void and serve only as a lead; the 32 against 30 of the W2 reconciliation was found to be a criterion that undercounts, not a counting error, and group 2 is not void; the `source_sha256` mismatch in the headers of 6 artifacts is the entry reading `$source_tree` and recording the repository, the header is wrongly recorded and not wrongly compiled, the patch was not applied, and the owed check goes to a registration revision. The 40 artifacts, the merged report with its `-r2` retained form and the two diffs are named. Registration revision 1 recorded the new mutation table (not shown to go red), the renaming of the apparatus and the three groups of segment one.
- Created, not run: the apparatus, unit tests and mutation tables are written, and the page exists so the gate recognizes the number.
