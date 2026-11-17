## E209 Cause of the spike —— run (enough to decide): training-point artifacts of the first and second segments (27) and test-point artifacts of the third segment (48) are out, and the combined reports of all three segments are re-issued and read; rows one, two, three and five are judged "does not hold", row four is judged "holds" at the five points that reproduce the spike, `files_10_units_1000` does not reproduce the spike so rows one to four are "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`) there; the fourth segment (conditional) is not started and marked "not run after enough to decide"; the three-way argument verdict: none of the three registration readings changes the judgement of the five rows

[中文](209-尖峰成因.md) · **English** · [日本語](209-スパイクの原因.ja.md)

Pre-run registration `research/prompts/e209-preregistration.md`, fork sheet `research/prompts/e207-spike-cause-questions.md` (five rows, hanging under the row "spike counted separately" of `research/prompts/e203-b8-criterion-r1-forks.md`). The question: in the measurement window of E207 (Steps of scans under the strict criterion) some operations have a `final_nodes` far above the median of the same point and state (flagged operations); in the publish, which step contributes the extra steps, and what decides which operation shows it. The five rows: the flag follows "touching old nodes not yet carried out in this mount"; the flag follows only the ordinal after the mount; the flag follows a stored quantity crossing a threshold; after a remount there is no flag because the remount step already carried those nodes out; in which step of the publish the extra steps land.

**This page records where the training-point artifacts of the first segment (seven artifacts) and the second segment (20 artifacts) and the five test points of the third segment (48 artifacts) stand, and the readings of the three combined reports: the first segment reads `-r3`, the second segment reads `-r2`, the third segment reads the third-segment combined report (75 inputs), all re-issued after the two model changes. Of the five rows, one, two, three and five are judged "does not hold" by the registration's "one mismatch is enough", and row four "holds" at the five points that reproduce the spike; the geometry sensitivity is read (all five rows carry `stable=false`, and the only flip in the registration's sense is the reproduction gate; item by item in "Readings and row-by-row judgements of the third-segment combined report"). What is judged is each row of the fork sheet, not whether a decision is overturned or established.**

### Current state

- Apparatus: `scripts/experiment-apparatus/e209_spike_cause_counters.py`, `e209_spike_cause_histories.rs`, `e209_spike_cause_scripts.rs`, `e209_spike_cause.rs` (model, bin `e209-spike-cause`), `e209_spike_cause_library.py` and the entry `e209-spike-cause-run.sh`; mutation table `scripts/mutation/tables/e209_spike_cause.tsv` and the counters script's own `e209_spike_cause_counters.mutations.tsv`.
- First-segment artifacts (each has `grep -c '^E209RESULT name=item_done'` equal to 1 and `grep -c window_refused` equal to 0): `research/results/e209-spike-cause-files_100_units_100-head-first_mount_first-2026-11-13.out` (2298 lines), `research/results/e209-spike-cause-files_100_units_100-head-first_repeated-2026-11-13.out` (2222 lines), `research/results/e209-spike-cause-files_100_units_100-head-mount_then_first-2026-11-13.out` (2264 lines), `research/results/e209-spike-cause-files_100_units_100-head-mount_twice_first-2026-11-13.out` (1821 lines), `research/results/e209-spike-cause-files_100_units_100-head-walk_forward-2026-11-13.out` (3546 lines), `research/results/e209-spike-cause-files_100_units_100-plain-walk_forward-2026-11-13.out` (81 lines); two twin artifacts (see "Entry defect" below).
- Source diff of the counting build against the original tree: `research/results/e209-spike-cause-counters-diff-2026-11-13.diff` (2163 lines, the inserted counting lines) and `research/results/e209-spike-cause-counters-diff-2026-11-13-r2.diff` (0 bytes; written by another call of the entry's `publish_diff`; the plain build copies without inserting, consistent with it being empty; this is an inference from reading the entry, which call wrote it was not checked one by one).
- Second-segment artifacts (task 15649 exited 0: judged 20, reused 0, refused 0): each has `item_done` equal to 1 and `window_refused` equal to 0; line counts: `research/results/e209-spike-cause-files_100_units_100-head-walk_backward-2026-11-13.out` (3489 lines), `research/results/e209-spike-cause-files_100_units_100-head-walk_from_middle-2026-11-13.out` (3461 lines), `research/results/e209-spike-cause-files_100_units_100-head-create_then_first-2026-11-13.out` (1340 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_create_1-2026-11-13.out` (1593 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_create_2-2026-11-13.out` (1826 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_create_4-2026-11-13.out` (2102 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_create_8-2026-11-13.out` (3099 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_create_16-2026-11-13.out` (4969 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_young_1-2026-11-13.out` (1631 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_young_4-2026-11-13.out` (2552 lines), `research/results/e209-spike-cause-files_100_units_100-head-filler_young_8-2026-11-13.out` (3964 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_1-2026-11-13.out` (1731 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_2-2026-11-13.out` (2006 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_4-2026-11-13.out` (2708 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_8-2026-11-13.out` (4066 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_16-2026-11-13.out` (6802 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_32-2026-11-13.out` (12252 lines), `research/results/e209-spike-cause-files_100_units_100-head-presweep_64-2026-11-13.out` (23508 lines), `research/results/e209-spike-cause-files_100_units_100-head-young_mount_first-2026-11-13.out` (2109 lines), `research/results/e209-spike-cause-files_100_units_100-head-old_mount_old-2026-11-13.out` (1430 lines). The check command and its output as printed:
```
for s in walk_backward walk_from_middle create_then_first filler_create_1 filler_create_2 filler_create_4 filler_create_8 filler_create_16 filler_young_1 filler_young_4 filler_young_8 presweep_1 presweep_2 presweep_4 presweep_8 presweep_16 presweep_32 presweep_64 young_mount_first old_mount_old; do f=research/results/e209-spike-cause-files_100_units_100-head-$s-2026-11-13.out; echo "$s item_done=$(grep -c '^E209RESULT name=item_done' $f) window_refused=$(grep -c window_refused $f) lines=$(wc -l < $f)"; done
walk_backward item_done=1 window_refused=0 lines=3489
walk_from_middle item_done=1 window_refused=0 lines=3461
create_then_first item_done=1 window_refused=0 lines=1340
filler_create_1 item_done=1 window_refused=0 lines=1593
filler_create_2 item_done=1 window_refused=0 lines=1826
filler_create_4 item_done=1 window_refused=0 lines=2102
filler_create_8 item_done=1 window_refused=0 lines=3099
filler_create_16 item_done=1 window_refused=0 lines=4969
filler_young_1 item_done=1 window_refused=0 lines=1631
filler_young_4 item_done=1 window_refused=0 lines=2552
filler_young_8 item_done=1 window_refused=0 lines=3964
presweep_1 item_done=1 window_refused=0 lines=1731
presweep_2 item_done=1 window_refused=0 lines=2006
presweep_4 item_done=1 window_refused=0 lines=2708
presweep_8 item_done=1 window_refused=0 lines=4066
presweep_16 item_done=1 window_refused=0 lines=6802
presweep_32 item_done=1 window_refused=0 lines=12252
presweep_64 item_done=1 window_refused=0 lines=23508
young_mount_first item_done=1 window_refused=0 lines=2109
old_mount_old item_done=1 window_refused=0 lines=1430
```
- First-segment combined reports: the old report `research/results/e209-spike-cause-report-first-2026-11-13.out` (88 lines, produced before the constants check was fixed; `constants_mismatches=1` in its lines 7 and 88 is an apparatus defect, below, "The constants check"), and the re-issued one in kept form with the `E7RESULT ` prefix and closing line added, `research/results/e209-spike-cause-report-first-2026-11-13-r2.out` (89 lines, again 7 inputs: five head items, the new twin, plain; `--crates-root` given for the row positions). Without prefix and closing line the two differ only in lines 7 and 88 (command and output under "The constants check"); the reading lines copied below are taken from `-r2`, line numbers are the same as in the old report.
- Second-segment combined report (task 15735, reading the 27 training-point artifacts of the first and second segments, without the old twin): raw form `research/results/e209-spike-cause-report-second-2026-11-13-raw.out` (407 lines) and kept form `research/results/e209-spike-cause-report-second-2026-11-13.out` (408 lines, with the `E7RESULT ` prefix and closing line added); the readings and the row-by-row judgements are under "Readings and row-by-row judgements of the second-segment combined report" below.
- The three re-issued combined reports (after the two model changes, per-round share and steps per extra node; outcomes handed over by the main agent: tasks 15811-15813 and 15815-15817 all exit 0, model unit tests 15807 "42 passed; 0 failed", mutation table 15808 "caught 19, not red 0, invalid 0", entry self-test 15809 `failures=0` (22 cells), break switch `memory-table-lacks-third` 15810 red on `the_run_reports_its_memory_before_the_copy_and_the_compile` with `failures=1`): first segment `research/results/e209-spike-cause-report-first-2026-11-13-r3.out` (89 lines) and raw form `research/results/e209-spike-cause-report-first-2026-11-13-r3-raw.out` (88 lines); second segment `research/results/e209-spike-cause-report-second-2026-11-13-r2.out` (408 lines) and raw form `research/results/e209-spike-cause-report-second-2026-11-13-r2-raw.out` (407 lines); third segment `research/results/e209-spike-cause-report-third-2026-11-13.out` (1052 lines, reading the 75 artifacts of the first, second and third segments) and raw form `research/results/e209-spike-cause-report-third-2026-11-13-raw.out` (1051 lines). The kept forms are the raw forms plus the `E7RESULT ` prefix and a closing line. The readings this page rests on are the kept forms of these three re-issued reports; the older first-segment `-r2`, the old second-segment report and its raw form (named in the two items above) are not deleted or moved, and compared with the re-issued ones:
```
diff research/results/e209-spike-cause-report-first-2026-11-13-r2.out research/results/e209-spike-cause-report-first-2026-11-13-r3.out | grep -c '^<'
3
```
```
diff research/results/e209-spike-cause-report-second-2026-11-13.out research/results/e209-spike-cause-report-second-2026-11-13-r2.out | grep -c '^<'
15
```
```
diff <(sed -E 's/round_shares=[^ ]*//; s/steps_per_extra_node=[^ ]*//' research/results/e209-spike-cause-report-first-2026-11-13-r2.out) <(sed -E 's/round_shares_of_the_raw_difference=[^ ]*//; s/steps_per_extra_node=[^ ]*//' research/results/e209-spike-cause-report-first-2026-11-13-r3.out) | wc -l
0
```
```
diff <(sed -E 's/round_shares=[^ ]*//; s/steps_per_extra_node=[^ ]*//' research/results/e209-spike-cause-report-second-2026-11-13.out) <(sed -E 's/round_shares_of_the_raw_difference=[^ ]*//; s/steps_per_extra_node=[^ ]*//' research/results/e209-spike-cause-report-second-2026-11-13-r2.out) | wc -l
0
```
The byte-different lines are 3 in the first segment and 15 in the second; they are only the `round_shares` field (now `round_shares_of_the_raw_difference`) of the `extra_steps` lines and the `steps_per_extra_node` field of the `extra_steps_per_node` lines (with those two fields removed, 0 lines differ); neither field enters any criterion, so the criterion readings are the same old and new, and the result lines copied whole in this page now come from the re-issued reports.
- Third-segment artifacts (task 15736 exit 0: judged fresh 48, of which forced fresh 0, reused 0, refused 0; longest item 235 seconds; the last attempt, the 7th, peaked at 35.00 GiB under the cap of 46656M): five test points `files_75_units_100`, `files_200_units_100`, `files_100_units_100_unit_area_2_pow_17`, `files_100_units_100_unit_area_2_pow_20` with ten items each and `files_10_units_1000` with eight, 48 artifacts, each with `item_done` equal to 1 and `window_refused` equal to 0. Check command and output as printed (the file name on each output line is one of the 48 artifacts, all under `research/results/`):
```
cd research/results; for f in $(ls | grep 'e209-spike-cause-files' | grep -v 'files_100_units_100-'); do echo "$f item_done=$(grep -c '^E209RESULT name=item_done' $f) window_refused=$(grep -c window_refused $f) lines=$(wc -l < $f)"; done
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-filler_create_4-2026-11-13.out item_done=1 window_refused=0 lines=2102
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-filler_young_4-2026-11-13.out item_done=1 window_refused=0 lines=2552
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-first_mount_first-2026-11-13.out item_done=1 window_refused=0 lines=2298
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-first_repeated-2026-11-13.out item_done=1 window_refused=0 lines=2222
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-mount_then_first-2026-11-13.out item_done=1 window_refused=0 lines=2264
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-mount_twice_first-2026-11-13.out item_done=1 window_refused=0 lines=1821
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-presweep_32-2026-11-13.out item_done=1 window_refused=0 lines=12252
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-presweep_8-2026-11-13.out item_done=1 window_refused=0 lines=4066
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-walk_backward-2026-11-13.out item_done=1 window_refused=0 lines=3489
e209-spike-cause-files_100_units_100_unit_area_2_pow_17-head-walk_forward-2026-11-13.out item_done=1 window_refused=0 lines=3544
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-filler_create_4-2026-11-13.out item_done=1 window_refused=0 lines=2102
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-filler_young_4-2026-11-13.out item_done=1 window_refused=0 lines=2552
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-first_mount_first-2026-11-13.out item_done=1 window_refused=0 lines=2298
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-first_repeated-2026-11-13.out item_done=1 window_refused=0 lines=2222
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-mount_then_first-2026-11-13.out item_done=1 window_refused=0 lines=2263
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-mount_twice_first-2026-11-13.out item_done=1 window_refused=0 lines=1821
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-presweep_32-2026-11-13.out item_done=1 window_refused=0 lines=12252
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-presweep_8-2026-11-13.out item_done=1 window_refused=0 lines=4066
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-walk_backward-2026-11-13.out item_done=1 window_refused=0 lines=3489
e209-spike-cause-files_100_units_100_unit_area_2_pow_20-head-walk_forward-2026-11-13.out item_done=1 window_refused=0 lines=3544
e209-spike-cause-files_10_units_1000-head-filler_create_4-2026-11-13.out item_done=1 window_refused=0 lines=1960
e209-spike-cause-files_10_units_1000-head-filler_young_4-2026-11-13.out item_done=1 window_refused=0 lines=2196
e209-spike-cause-files_10_units_1000-head-first_mount_first-2026-11-13.out item_done=1 window_refused=0 lines=2118
e209-spike-cause-files_10_units_1000-head-first_repeated-2026-11-13.out item_done=1 window_refused=0 lines=1915
e209-spike-cause-files_10_units_1000-head-mount_then_first-2026-11-13.out item_done=1 window_refused=0 lines=2421
e209-spike-cause-files_10_units_1000-head-mount_twice_first-2026-11-13.out item_done=1 window_refused=0 lines=1720
e209-spike-cause-files_10_units_1000-head-walk_backward-2026-11-13.out item_done=1 window_refused=0 lines=2994
e209-spike-cause-files_10_units_1000-head-walk_forward-2026-11-13.out item_done=1 window_refused=0 lines=3018
e209-spike-cause-files_200_units_100-head-filler_create_4-2026-11-13.out item_done=1 window_refused=0 lines=2563
e209-spike-cause-files_200_units_100-head-filler_young_4-2026-11-13.out item_done=1 window_refused=0 lines=3249
e209-spike-cause-files_200_units_100-head-first_mount_first-2026-11-13.out item_done=1 window_refused=0 lines=2789
e209-spike-cause-files_200_units_100-head-first_repeated-2026-11-13.out item_done=1 window_refused=0 lines=2721
e209-spike-cause-files_200_units_100-head-mount_then_first-2026-11-13.out item_done=1 window_refused=0 lines=2329
e209-spike-cause-files_200_units_100-head-mount_twice_first-2026-11-13.out item_done=1 window_refused=0 lines=1763
e209-spike-cause-files_200_units_100-head-presweep_32-2026-11-13.out item_done=1 window_refused=0 lines=12477
e209-spike-cause-files_200_units_100-head-presweep_8-2026-11-13.out item_done=1 window_refused=0 lines=4631
e209-spike-cause-files_200_units_100-head-walk_backward-2026-11-13.out item_done=1 window_refused=0 lines=3756
e209-spike-cause-files_200_units_100-head-walk_forward-2026-11-13.out item_done=1 window_refused=0 lines=3786
e209-spike-cause-files_75_units_100-head-filler_create_4-2026-11-13.out item_done=1 window_refused=0 lines=2007
e209-spike-cause-files_75_units_100-head-filler_young_4-2026-11-13.out item_done=1 window_refused=0 lines=2318
e209-spike-cause-files_75_units_100-head-first_mount_first-2026-11-13.out item_done=1 window_refused=0 lines=2223
e209-spike-cause-files_75_units_100-head-first_repeated-2026-11-13.out item_done=1 window_refused=0 lines=2034
e209-spike-cause-files_75_units_100-head-mount_then_first-2026-11-13.out item_done=1 window_refused=0 lines=2134
e209-spike-cause-files_75_units_100-head-mount_twice_first-2026-11-13.out item_done=1 window_refused=0 lines=1719
e209-spike-cause-files_75_units_100-head-presweep_32-2026-11-13.out item_done=1 window_refused=0 lines=10882
e209-spike-cause-files_75_units_100-head-presweep_8-2026-11-13.out item_done=1 window_refused=0 lines=3603
e209-spike-cause-files_75_units_100-head-walk_backward-2026-11-13.out item_done=1 window_refused=0 lines=3129
e209-spike-cause-files_75_units_100-head-walk_forward-2026-11-13.out item_done=1 window_refused=0 lines=3223
```
These 48 artifacts are read by the third-segment combined report (75 inputs of the first, second and third segments; readings and row-by-row judgements in "Readings and row-by-row judgements of the third-segment combined report" below).

### Entry defect: the old twin artifact lacks one line (fixed)

The twin artifact lacked line 57 `name=point` of the original item, and its header recorded `test_threads=31` (only one test thread was used): in libtest's single-thread mode the entry did not strip the prefix `test <case name> ... ` that is glued in front of the first line, so that line no longer started with `E209RESULT` and was dropped; the header also recorded the default concurrency. The entry is fixed and the twin artifact re-issued: `research/results/e209-spike-cause-files_100_units_100-head-walk_forward-twin-2026-11-13-r2.out` (3546 lines, header `twin=yes … test_threads=1`). The old `research/results/e209-spike-cause-files_100_units_100-head-walk_forward-twin-2026-11-13.out` (3545 lines) is kept and is not an input of the combined report. `diff` against the original artifact `research/results/e209-spike-cause-files_100_units_100-head-walk_forward-2026-11-13.out`: the new twin differs only on line 1 (header `twin=no … test_threads=31` versus `twin=yes … test_threads=1`); the old twin differs on line 1 and on line 57 of the original, `name=point` (`57d56`, absent in the old twin). The new twin is byte-identical to the original apart from the header; `name=determinism_twin … identical=true` of the combined report reads the new twin.

### Readings

Stop clauses and differential checks (the pins of the apparatus against the implementation):

`e209-spike-cause-report-first-2026-11-13-r3.out` line 8:
```
E209RESULT name=determinism_twin item=files_100_units_100__head__walk_forward identical=true first_difference_line=-
```

`e209-spike-cause-report-first-2026-11-13-r3.out` line 9:
```
E209RESULT name=plain_versus_counts item=files_100_units_100__plain__walk_forward image_sha256_equal=true
```

`e209-spike-cause-report-first-2026-11-13-r3.out` line 88:
```
E209RESULT name=verdict items_read=6 items_without_item_done=zero items_refused=zero constants_mismatches=zero history_mismatches=zero script_sequence_mismatches=zero closure_model_mismatches=zero final_set_not_covering=zero final_nodes_mismatches=zero edge_walk_mismatches=zero live_units_shortfalls=zero negative_control_over_four_rounds=zero twin_differences=zero plain_differences=zero reproduction_at_the_training_point=reproduced
```

**The constants check**: `e209-spike-cause-report-first-2026-11-13-r3.out` line 7:
```
E209RESULT name=item_checks item=files_100_units_100__plain__walk_forward done=true refused=false constants=- history=ok script_sequence=ok closure_publishes=0 closure_mismatches=- final_not_covering=0 final_nodes_mismatches=0 edge_walk_mismatches=0 live_units=ok
```

This is the plain build: it copies without inserting, so its artifact has no `name=constants` line (registration section seven, (2), item 1 says "compare the model's local constants with the header of the counting build's artifact"), yet the apparatus checked the constants of every item and found `None≠812` and two more for plain, giving `constants_mismatches=1` in the old report. It is an apparatus defect, not an implementation constant mismatch: the five head items have `constants=-` in their `item_checks` (lines 1, 2, 4, 5, 6). Fix: `constants_mismatches` in `scripts/experiment-apparatus/e209_spike_cause.rs` does not check items with `arm == "plain"`; the unit test `the_constants_and_the_history_are_checked_against_the_registered_values` gained a plain case and goes red without this line. In the re-issued `-r2` this cell is `zero` (line 7 above has `constants=-`, line 88 under "Stop clauses and cross-checks" has `constants_mismatches=zero`). The old report is not edited; its lines 7 and 88 are:

`e209-spike-cause-report-first-2026-11-13.out` line 7:
```
E209RESULT name=item_checks item=files_100_units_100__plain__walk_forward done=true refused=false constants=leaf_slots=None≠812,internal_fanout=None≠169,settling_rounds_at_most=None≠64 history=ok script_sequence=ok closure_publishes=0 closure_mismatches=- final_not_covering=0 final_nodes_mismatches=0 edge_walk_mismatches=0 live_units=ok
```

`e209-spike-cause-report-first-2026-11-13.out` line 88:
```
E209RESULT name=verdict items_read=6 items_without_item_done=zero items_refused=zero constants_mismatches=1 history_mismatches=zero script_sequence_mismatches=zero closure_model_mismatches=zero final_set_not_covering=zero final_nodes_mismatches=zero edge_walk_mismatches=zero live_units_shortfalls=zero negative_control_over_four_rounds=zero twin_differences=zero plain_differences=zero reproduction_at_the_training_point=reproduced
```

The difference between the two reports (command and output; the output shows only these two places):
```
diff <(sed 's/^E7RESULT //' research/results/e209-spike-cause-report-first-2026-11-13-r2.out | grep -v '^name=done') research/results/e209-spike-cause-report-first-2026-11-13.out | grep '^[0-9]'
7c7
88c88
```

Q0 reproduction gate (registration section five, first segment, "gate"):

`e209-spike-cause-report-first-2026-11-13-r3.out` line 10:
```
E209RESULT name=reproduction_gate point=files_100_units_100 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
```

Readings of the five rows (this section is from the first segment only, a preview; the readings and judgements of the second-segment combined report are under "Readings and row-by-row judgements of the second-segment combined report" below). Row one (freshness rule, four variants):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 51-54:
```
E209RESULT name=freshness_rule variant=mount_wide threshold=0 flagged_in_training=2 training_failures=15 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=1 script_unflagged=12 script_mismatches=12 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=2:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=4:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=2:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_rule variant=mount_narrow threshold=42 flagged_in_training=2 training_failures=0 training_failure_labels=- script_flagged=1 script_unflagged=12 script_mismatches=0 script_mismatch_labels=- geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=too_few_flagged_operations
E209RESULT name=freshness_rule variant=ever_wide threshold=0 flagged_in_training=2 training_failures=15 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=1 script_unflagged=12 script_mismatches=12 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=0:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=0:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_rule variant=ever_narrow threshold=42 flagged_in_training=2 training_failures=0 training_failure_labels=- script_flagged=1 script_unflagged=12 script_mismatches=0 script_mismatch_labels=- geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=too_few_flagged_operations
```

Of the four variants the two wide readings have 15 training failures (`training_failures=15`, verdict `does_not_hold`); the two narrow readings have no training failure and get `too_few_flagged_operations` (fewer than 3 flagged operations in the test sets: `script_flagged=1`, `geometry_flagged=0`). Only 2 flagged operations are in training (`flagged_in_training=2`). The first segment's training set has only the scripts `walk_forward` and `first_repeated`, while the registration's training set is four scripts (two more in the second segment); the whole test-geometry family (the five test points of the third segment) has not run, so `geometry_flagged=0` means not run, not "no flag". The judgement of this cell is in "Readings and row-by-row judgements of the second-segment combined report" below.

Row two (ordinal):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 57-58:
```
E209RESULT name=ordinal_equality point=files_100_units_100 script_pairs=10 flag_depends_only_on_the_ordinal=true first_difference=-
E209RESULT name=ordinal_shift point=files_100_units_100 walk_forward_first_flagged_ordinal=1 fillers=0 verdict=neither
```

Row three (stored quantity):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 67-76:
```
E209RESULT name=stored_quantity quantity=stale_nodes relative=false threshold=0.0000 pooled_errors=16 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=stale_nodes relative=true threshold=0.0000 pooled_errors=16 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=chain_depth relative=false threshold=23.0000 pooled_errors=0 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=too_few_distinct_values
E209RESULT name=stored_quantity quantity=chain_depth relative=true threshold=0.3770 pooled_errors=0 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=too_few_distinct_values
E209RESULT name=stored_quantity quantity=released_backlog relative=false threshold=4502.0000 pooled_errors=4 flagged_values=1 unflagged_values=14 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=released_backlog relative=true threshold=73.8033 pooled_errors=4 flagged_values=1 unflagged_values=14 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=false threshold=134.0000 pooled_errors=0 flagged_values=1 unflagged_values=9 geometry_errors=0 verdict=too_few_distinct_values
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=true threshold=2.1967 pooled_errors=0 flagged_values=1 unflagged_values=9 geometry_errors=0 verdict=too_few_distinct_values
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=0 first_flagged=true stale_nodes=0 publishes_since_flag=134 final_nodes=53 first_flagged_ordinal=1 first_flagged_selector=first
E209RESULT name=presweep point=files_100_units_100 flagged_counts=0 unflagged_counts=- between=- verdict=threshold_sides_not_both_sampled
```

Row four (carried out by the remount):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 59-82:
```
E209RESULT name=mount_carries point=files_100_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6
E209RESULT name=mount_carried_step point=files_100_units_100 carried_nodes_by_first_step=write_row:51,warm_up_1:0,warm_up_2:0,warm_up_3:0,warm_up_4:0,warm_up_5:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_first step_index=4 flagged=false fresh_mount_wide=2 fresh_ever_wide=0 nodes=7 in_the_mount_publishes=5 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_old step_index=5 flagged=false fresh_mount_wide=0 fresh_ever_wide=0 nodes=7 in_the_mount_publishes=5 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_mid step_index=6 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=9 in_the_mount_publishes=5 touched_before_the_mount=9 not_in_the_mount_publishes=4
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_first step_index=3 flagged=false fresh_mount_wide=8 fresh_ever_wide=0 nodes=13 in_the_mount_publishes=11 touched_before_the_mount=0 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_old step_index=4 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=7 touched_before_the_mount=0 not_in_the_mount_publishes=4
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_mid step_index=5 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=11 touched_before_the_mount=0 not_in_the_mount_publishes=0
```

Row five (which step of the publish holds the extra steps):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 61-66:
```
E209RESULT name=extra_steps operation=files_100_units_100__head__first_mount_first#1 selector=first comparable_first=files_100_units_100__head__first_repeated#2 comparable_second=files_100_units_100__head__mount_then_first#2 steps=11164 steps_first=3730 steps_second=4922 extra=7434 noise=1192 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.771 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__first_mount_first#1 extra_nodes=46 steps_per_extra_node=161.609
E209RESULT name=extra_steps operation=files_100_units_100__head__first_repeated#1 selector=first comparable_first=files_100_units_100__head__first_mount_first#4 comparable_second=files_100_units_100__head__mount_then_first#2 steps=11164 steps_first=6614 steps_second=4922 extra=4550 noise=1692 top_rows=allocator_release_linear_find:1800,mapped_units_whole_table:900,record_intersection_filter:866 top_share=0.784 round_shares_of_the_raw_difference=1:release_not_covered:0.000,2:settled:0.000,3:none:0.000 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__first_repeated#1 extra_nodes=46 steps_per_extra_node=98.913
E209RESULT name=extra_steps operation=files_100_units_100__head__walk_forward#1 selector=first comparable_first=files_100_units_100__head__first_mount_first#4 comparable_second=files_100_units_100__head__first_repeated#2 steps=11164 steps_first=6614 steps_second=3730 extra=4550 noise=2884 top_rows=allocator_release_linear_find:1800,mapped_units_whole_table:900,record_intersection_filter:866 top_share=0.784 round_shares_of_the_raw_difference=1:release_not_covered:0.000,2:settled:0.000,3:none:0.000 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__walk_forward#1 extra_nodes=46 steps_per_extra_node=98.913
```

Flag-line sensitivity and geometry sensitivity (one sample point only; `stable=true` only says nothing flipped at one point):

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 83-87:
```
E209RESULT name=geometry_sensitivity quantity=reproduction_gate points=1 per_point=files_100_units_100:reproduced stable=true
E209RESULT name=geometry_sensitivity quantity=freshness_rule_mount_wide points=1 per_point=files_100_units_100:mismatches_27 stable=true
E209RESULT name=geometry_sensitivity quantity=ordinal_equality points=1 per_point=files_100_units_100:true stable=true
E209RESULT name=geometry_sensitivity quantity=mount_carries points=1 per_point=files_100_units_100:mount_carried_the_nodes stable=true
E209RESULT name=geometry_sensitivity quantity=extra_steps_location points=1 per_point=files_100_units_100:not_resolved stable=true
```

The per-operation rows (40 lines `name=operation_row`, lines 11-50) are in the same file; `name=freshness_manipulation`:

`e209-spike-cause-report-first-2026-11-13-r3.out` lines 55-56:
```
E209RESULT name=freshness_manipulation_script script=walk_forward flagged_operations=1 selectors=first
E209RESULT name=freshness_manipulation scripts=1 position_follows=false count_follows=false retouched_operations=23 retouched_and_flagged=3 retouched_flagged_labels=files_100_units_100__head__first_mount_first#1,files_100_units_100__head__first_repeated#1,files_100_units_100__head__walk_forward#1
```

### Readings and row-by-row judgements of the second-segment combined report (27 artifacts of the first and second segments)

After the three combined reports were re-issued, the lines copied in this section come from the kept form of the re-issued second-segment report `e209-spike-cause-report-second-2026-11-13-r2.out` (reading the 27 training-point artifacts of the first and second segments, without the old twin), with the `E7RESULT ` prefix removed; the line numbers are those of the raw form `-r2-raw.out` and also those of the old second-segment combined report `e209-spike-cause-report-second-2026-11-13.out` (the two differ only in the `round_shares` and `steps_per_extra_node` fields, command and output in "Current state"). In the re-issued report the `round_shares_of_the_raw_difference=` field of the `extra_steps` lines is each round's difference over the sum of the round differences, and `steps_per_extra_node` reports `not_defined_without_positive_extra` when the extra steps are not positive; the readings of these two fields can be read, but they still enter no criterion, and the criterion readings are as in this report. The row-by-row judgements follow registration sections six and eleven and judge this row of the fork sheet.

Stop clauses, differential checks and the reproduction gate:

`e209-spike-cause-report-second-2026-11-13-r2.out` line 407:
```
E209RESULT name=verdict items_read=26 items_without_item_done=zero items_refused=zero constants_mismatches=zero history_mismatches=zero script_sequence_mismatches=zero closure_model_mismatches=zero final_set_not_covering=zero final_nodes_mismatches=zero edge_walk_mismatches=zero live_units_shortfalls=zero negative_control_over_four_rounds=zero twin_differences=zero plain_differences=zero reproduction_at_the_training_point=reproduced
```

`e209-spike-cause-report-second-2026-11-13-r2.out` line 30:
```
E209RESULT name=reproduction_gate point=files_100_units_100 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
```

All stop counts are `zero`, `items_read=26` (of the 27 inputs the twin item has the same name as the original, one item), `reproduction_at_the_training_point=reproduced`; the registered stop clauses do not hold anything back.

**Row one (freshness rule): decidable, does not hold** (registration section eleven, "enough to decide": one mismatch in row one is enough).

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 319-326:
```
E209RESULT name=freshness_rule variant=mount_wide threshold=0 flagged_in_training=4 training_failures=27 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=17 script_unflagged=230 script_mismatches=230 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=2:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=4:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=2:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_rule variant=mount_narrow threshold=18 flagged_in_training=4 training_failures=0 training_failure_labels=- script_flagged=17 script_unflagged=230 script_mismatches=1 script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_rule variant=ever_wide threshold=0 flagged_in_training=4 training_failures=27 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=17 script_unflagged=230 script_mismatches=230 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=0:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=0:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_rule variant=ever_narrow threshold=18 flagged_in_training=4 training_failures=0 training_failure_labels=- script_flagged=17 script_unflagged=230 script_mismatches=1 script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0 geometry_flagged=0 geometry_unflagged=0 geometry_mismatches=0 geometry_mismatch_labels=- verdict=does_not_hold
E209RESULT name=freshness_manipulation_script script=walk_forward flagged_operations=1 selectors=first
E209RESULT name=freshness_manipulation_script script=walk_backward flagged_operations=1 selectors=mid
E209RESULT name=freshness_manipulation_script script=walk_from_middle flagged_operations=1 selectors=mid
E209RESULT name=freshness_manipulation scripts=3 position_follows=true count_follows=false retouched_operations=267 retouched_and_flagged=21 retouched_flagged_labels=files_100_units_100__head__first_mount_first#1,files_100_units_100__head__first_repeated#1,files_100_units_100__head__walk_forward#1,files_100_units_100__head__walk_backward#3,files_100_units_100__head__walk_from_middle#1,files_100_units_100__head__create_then_first#2,files_100_units_100__head__filler_create_1#2,files_100_units_100__head__filler_create_2#3,files_100_units_100__head__filler_create_4#5,files_100_units_100__head__filler_create_8#9,files_100_units_100__head__filler_create_16#17,files_100_units_100__head__filler_young_1#2,files_100_units_100__head__filler_young_4#5,files_100_units_100__head__filler_young_8#9,files_100_units_100__head__presweep_1#1,files_100_units_100__head__presweep_2#1,files_100_units_100__head__presweep_4#1,files_100_units_100__head__presweep_8#1,files_100_units_100__head__presweep_16#1,files_100_units_100__head__presweep_32#1,files_100_units_100__head__presweep_64#1
```

All four variants have `verdict=does_not_hold`. Two wide readings: the minimum freshness among flagged operations is 0 (`threshold=0`), so "freshness ≥ 0" predicts a flag for every operation and 27 unflagged operations in training are wrong (`training_failures=27`); registration Q3's re-touch: 267 operations have freshness 0, of which 21 are flagged (`retouched_operations=267`, `retouched_and_flagged=21`), i.e. "re-touch still flags". Two narrow readings: 0 failures in training (`threshold=18`, `flagged_in_training=4`), 246 of the 247 operations of the test set predicted right, `script_mismatches=1`, label `old_mount_old#1:fresh=42:observed_flag=0`. First, is this an apparatus defect? The operation is the only user operation in the 0th mount of `old_mount_old` (line 316, `final_nodes=53 flagged=false`):

`e209-spike-cause-report-second-2026-11-13-r2.out` line 316:
```
E209RESULT name=operation_row item=files_100_units_100__head__old_mount_old mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_old final_nodes=53 flagged=false flagged_at_the_low_line=false flagged_at_the_high_line=false median_inflated=false fresh_mount_wide=0 fresh_mount_narrow=42 fresh_ever_wide=0 fresh_ever_narrow=42 seed_nodes=11 pulled_nodes=42 stale_nodes=0 chain_depth=23 released_backlog=3392 publishes_since_flag=134
```

The flag line of reading 1 is `max(28, 4 × lower median of the window)`; the window holds only this one operation, so the lower median is 53 and the line 212, and this 53-node operation is not flagged under the registered reading. The apparatus computed the registered formula correctly (`spike_line`, `lower_median`, pinned by the unit test `the_spike_flag_uses_the_lower_median_and_the_floor`); it is not an apparatus defect but a property of E207's (Steps of scans under the strict criterion) literal reading, which cannot flag when the window holds one operation. So this one mismatch carries an artefact of window size 1; the registration's "one mismatch means does not hold" is applied literally and judges it does not hold; whether to set up a separate flag reading for windows with a single operation is for the main agent. `geometry_flagged=0` means the third segment's test-geometry artifacts had not been read by a combined report, not "no flag". The position follows and the count does not (lines 324-326, `position_follows=true`, `count_follows=false`). **Can the remaining quantities flip it: no** (a larger test set does not remove mismatches, and the 27 training failures of the wide readings are fixed inside the training set).

**Row two (flag follows only the ordinal): decidable, does not hold.**

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 327-336:
```
E209RESULT name=ordinal_equality point=files_100_units_100 script_pairs=300 flag_depends_only_on_the_ordinal=false first_difference=create_then_first/filler_create_16#2
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_create_1 filler_count=1 first_ordinal=2 first_flagged_ordinal=2 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_create_16 filler_count=16 first_ordinal=17 first_flagged_ordinal=17 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_create_2 filler_count=2 first_ordinal=3 first_flagged_ordinal=3 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_create_4 filler_count=4 first_ordinal=5 first_flagged_ordinal=5 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_create_8 filler_count=8 first_ordinal=9 first_flagged_ordinal=9 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_young_1 filler_count=1 first_ordinal=2 first_flagged_ordinal=2 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_young_4 filler_count=4 first_ordinal=5 first_flagged_ordinal=5 first_flagged_selector=first
E209RESULT name=ordinal_shift_row point=files_100_units_100 script=filler_young_8 filler_count=8 first_ordinal=9 first_flagged_ordinal=9 first_flagged_selector=first
E209RESULT name=ordinal_shift point=files_100_units_100 walk_forward_first_flagged_ordinal=1 fillers=8 verdict=follows_the_operation
```

`flag_depends_only_on_the_ordinal=false` (of 300 script pairs the first difference is at `create_then_first/filler_create_16#2`); shifting: in each of the eight filler scripts the first flagged ordinal is `n + 1` with selector `first`, and `ordinal_shift` judges `follows_the_operation`. The flag follows the touched operation, not the ordinal; the "does not hold" conditions of registration Q4 and Q5 are both met. **The remaining quantities cannot flip it** (one difference is enough; the third segment only adds geometry sensitivity).

**Row three (stored quantity crosses a threshold): decidable, does not hold.**

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 374-390:
```
E209RESULT name=stored_quantity quantity=stale_nodes relative=false threshold=0.0000 pooled_errors=241 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=stale_nodes relative=true threshold=0.0000 pooled_errors=241 flagged_values=1 unflagged_values=1 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=chain_depth relative=false threshold=23.0000 pooled_errors=39 flagged_values=4 unflagged_values=5 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=chain_depth relative=true threshold=0.3770 pooled_errors=39 flagged_values=4 unflagged_values=5 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=released_backlog relative=false threshold=17788.0000 pooled_errors=22 flagged_values=10 unflagged_values=134 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=released_backlog relative=true threshold=291.6066 pooled_errors=22 flagged_values=10 unflagged_values=134 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=false threshold=150.0000 pooled_errors=20 flagged_values=6 unflagged_values=82 geometry_errors=0 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=true threshold=2.4590 pooled_errors=20 flagged_values=6 unflagged_values=82 geometry_errors=0 verdict=does_not_hold
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=0 first_flagged=true stale_nodes=0 publishes_since_flag=134 final_nodes=53 first_flagged_ordinal=1 first_flagged_selector=first
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=1 first_flagged=false stale_nodes=0 publishes_since_flag=0 final_nodes=9 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=2 first_flagged=false stale_nodes=0 publishes_since_flag=1 final_nodes=9 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=4 first_flagged=false stale_nodes=0 publishes_since_flag=3 final_nodes=7 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=8 first_flagged=false stale_nodes=0 publishes_since_flag=7 final_nodes=9 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=16 first_flagged=false stale_nodes=0 publishes_since_flag=15 final_nodes=9 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=32 first_flagged=false stale_nodes=0 publishes_since_flag=31 final_nodes=9 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep_point point=files_100_units_100 presweep_count=64 first_flagged=false stale_nodes=0 publishes_since_flag=63 final_nodes=11 first_flagged_ordinal=1 first_flagged_selector=cold_1
E209RESULT name=presweep point=files_100_units_100 flagged_counts=0 unflagged_counts=1,2,4,8,16,32,64 between=0..1 verdict=threshold_between_two_sample_points
```

All eight cells are `verdict=does_not_hold` (20 to 241 pooled errors), registration Q6 "all 8 cells do not hold ⇒ row three does not hold". `name=presweep` literally judges `threshold_between_two_sample_points` (`flagged_counts=0`, `unflagged_counts=1,2,4,8,16,32,64`), but this cannot be read as a quantity threshold: in every `presweep_m` script (m ≥ 1) `first` is unflagged because the flag falls on the earlier cold-file operation `cold_1` (the two lines below: the first operation of `presweep_1` and `presweep_2`, `overwrite_cold_1`, `final_nodes=53 flagged=true`), each script still has exactly one flagged operation, and `stale_nodes` is 0 for m = 0 to 64 (the `presweep_point` lines): the same batch of old nodes, whichever operation touches it first is flagged.

`e209-spike-cause-report-second-2026-11-13-r2.out` line 163:
```
E209RESULT name=operation_row item=files_100_units_100__head__presweep_1 mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_cold_1 final_nodes=53 flagged=true flagged_at_the_low_line=true flagged_at_the_high_line=false median_inflated=false fresh_mount_wide=0 fresh_mount_narrow=42 fresh_ever_wide=0 fresh_ever_narrow=42 seed_nodes=11 pulled_nodes=42 stale_nodes=0 chain_depth=23 released_backlog=3392 publishes_since_flag=134
```

`e209-spike-cause-report-second-2026-11-13-r2.out` line 167:
```
E209RESULT name=operation_row item=files_100_units_100__head__presweep_2 mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_cold_1 final_nodes=53 flagged=true flagged_at_the_low_line=true flagged_at_the_high_line=false median_inflated=false fresh_mount_wide=0 fresh_mount_narrow=42 fresh_ever_wide=0 fresh_ever_narrow=42 seed_nodes=11 pulled_nodes=42 stale_nodes=0 chain_depth=23 released_backlog=3392 publishes_since_flag=134
```

**The remaining quantities cannot flip it**: the pooling of the eight cells uses only the training set and the script test set; the test geometry added by the third segment does not enter the pooling.

**Row four (carried out by the remount): decidable at the training point, holds; whether it is stable is read in the next section from the third-segment combined report.**

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 337-338:
```
E209RESULT name=mount_carries point=files_100_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6
E209RESULT name=mount_carried_step point=files_100_units_100 carried_nodes_by_first_step=write_row:51,warm_up_1:0,warm_up_2:0,warm_up_3:0,warm_up_4:0,warm_up_5:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 391-401:
```
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_first step_index=4 flagged=false fresh_mount_wide=2 fresh_ever_wide=0 nodes=7 in_the_mount_publishes=5 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_old step_index=5 flagged=false fresh_mount_wide=0 fresh_ever_wide=0 nodes=7 in_the_mount_publishes=5 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__first_mount_first operation=overwrite_mid step_index=6 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=9 in_the_mount_publishes=5 touched_before_the_mount=9 not_in_the_mount_publishes=4
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_first step_index=3 flagged=false fresh_mount_wide=8 fresh_ever_wide=0 nodes=13 in_the_mount_publishes=11 touched_before_the_mount=0 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_old step_index=4 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=7 touched_before_the_mount=0 not_in_the_mount_publishes=4
E209RESULT name=mount_variant item=files_100_units_100__head__mount_twice_first operation=overwrite_mid step_index=5 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=11 touched_before_the_mount=0 not_in_the_mount_publishes=0
E209RESULT name=mount_variant item=files_100_units_100__head__young_mount_first operation=overwrite_first step_index=4 flagged=false fresh_mount_wide=2 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=9 touched_before_the_mount=5 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__young_mount_first operation=overwrite_old step_index=5 flagged=false fresh_mount_wide=4 fresh_ever_wide=0 nodes=13 in_the_mount_publishes=7 touched_before_the_mount=7 not_in_the_mount_publishes=6
E209RESULT name=mount_variant item=files_100_units_100__head__young_mount_first operation=overwrite_mid step_index=6 flagged=false fresh_mount_wide=0 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=9 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__old_mount_old operation=overwrite_old step_index=3 flagged=false fresh_mount_wide=2 fresh_ever_wide=0 nodes=7 in_the_mount_publishes=5 touched_before_the_mount=7 not_in_the_mount_publishes=2
E209RESULT name=mount_variant item=files_100_units_100__head__old_mount_old operation=overwrite_first step_index=4 flagged=false fresh_mount_wide=2 fresh_ever_wide=0 nodes=11 in_the_mount_publishes=9 touched_before_the_mount=7 not_in_the_mount_publishes=2
```

The premise holds (the first overwrite after the mount is not flagged, `first_after_the_mount_flagged=false`; the 11 `mount_variant` lines for the overwrites after the mount in `first_mount_first`, `mount_twice_first`, `young_mount_first` and `old_mount_old` are all `flagged=false`, no variant flips); `share=0.962` (51 of the 53 nodes of the spike were carried out by the mount itself); `located_step=write_row` (all 51 in the mount's row-write publish). Registration Q8 "the mount carried out those nodes" and Q9 "located in one step" both hold. Positions (line numbers taken today):

```
grep -n 'merge_into_the_rewrite_set_walking_the_release_chain(\|let settled = settle_the_allocation_record_tree(' crates/singlefs-core/src/transaction.rs
2046:            merge_into_the_rewrite_set_walking_the_release_chain(
2070:        merge_into_the_rewrite_set_walking_the_release_chain(
13435:    let settled = settle_the_allocation_record_tree(
14087:            merge_into_the_rewrite_set_walking_the_release_chain(
14116:        merge_into_the_rewrite_set_walking_the_release_chain(
14133:fn merge_into_the_rewrite_set_walking_the_release_chain(
grep -n 'fn row_publish_plan_on_file_version' crates/singlefs-core/src/mount.rs
5841:fn row_publish_plan_on_file_version<'plan>(
```
The mount's row-write publish goes through `row_publish_plan_on_file_version` at `crates/singlefs-core/src/mount.rs:5841` (same as `plan_position` in report line 338); the fixed-point call site is `crates/singlefs-core/src/transaction.rs:13435`; `first_merge_site=release_not_covered` is the merge at the release site, at `transaction.rs:14087` (the placement site is `:14116`). **The remaining quantities cannot flip it** (decidable at the training point); the third segment only adds the number of sample points of `name=geometry_sensitivity quantity=mount_carries` (now `points=1`).

**Row five (which step holds the extra steps): decidable, does not hold.**

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 339-373:
```
E209RESULT name=extra_steps operation=files_100_units_100__head__first_mount_first#1 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_repeated#2 steps=11164 steps_first=3736 steps_second=3730 extra=7428 noise=6 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.771 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=spread_over_many_rows
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__first_mount_first#1 extra_nodes=46 steps_per_extra_node=161.478
E209RESULT name=extra_steps operation=files_100_units_100__head__first_repeated#1 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11164 steps_first=3736 steps_second=6614 extra=7428 noise=2878 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.771 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__first_repeated#1 extra_nodes=46 steps_per_extra_node=161.478
E209RESULT name=extra_steps operation=files_100_units_100__head__walk_forward#1 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11164 steps_first=3736 steps_second=6614 extra=7428 noise=2878 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.771 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__walk_forward#1 extra_nodes=46 steps_per_extra_node=161.478
E209RESULT name=extra_steps operation=files_100_units_100__head__walk_backward#3 selector=mid comparable_first=files_100_units_100__head__filler_create_1#4 comparable_second=files_100_units_100__head__filler_create_16#19 steps=47258 steps_first=41642 steps_second=26916 extra=5616 noise=14726 top_rows=allocator_release_linear_find:2160,mapped_units_whole_table:1080,record_intersection_filter:1080 top_share=0.769 round_shares_of_the_raw_difference=1:release_not_covered:-0.001,2:placement_not_covered:0.501,3:settled:0.500 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__walk_backward#3 extra_nodes=18 steps_per_extra_node=312.000
E209RESULT name=extra_steps operation=files_100_units_100__head__walk_from_middle#1 selector=mid comparable_first=files_100_units_100__head__filler_create_1#4 comparable_second=files_100_units_100__head__filler_create_16#19 steps=30912 steps_first=41642 steps_second=26916 extra=-10730 noise=14726 top_rows=mapped_units_whole_table:120,record_intersection_filter:120,allocation_record_tree_node_set:54 top_share=0.000 round_shares_of_the_raw_difference=1:release_not_covered:0.000,2:settled:0.000,3:none:0.000 verdict=no_extra_steps
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__walk_from_middle#1 extra_nodes=26 steps_per_extra_node=not_defined_without_positive_extra
E209RESULT name=extra_steps operation=files_100_units_100__head__create_then_first#2 selector=first comparable_first=files_100_units_100__head__first_mount_first#4 comparable_second=files_100_units_100__head__first_repeated#2 steps=11162 steps_first=6614 steps_second=3730 extra=4548 noise=2884 top_rows=allocator_release_linear_find:1800,mapped_units_whole_table:900,record_intersection_filter:866 top_share=0.784 round_shares_of_the_raw_difference=1:release_not_covered:0.000,2:settled:0.000,3:none:0.000 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__create_then_first#2 extra_nodes=46 steps_per_extra_node=98.870
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_create_1#2 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11162 steps_first=3736 steps_second=6614 extra=7426 noise=2878 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.772 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_create_1#2 extra_nodes=46 steps_per_extra_node=161.435
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_create_2#3 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11162 steps_first=3736 steps_second=6614 extra=7426 noise=2878 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.772 round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_create_2#3 extra_nodes=46 steps_per_extra_node=161.435
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_create_4#5 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11250 steps_first=3736 steps_second=6614 extra=7514 noise=2878 top_rows=allocator_release_linear_find:2880,record_intersection_filter:1454,mapped_units_whole_table:1440 top_share=0.768 round_shares_of_the_raw_difference=1:release_not_covered:-0.018,2:settled:1.018 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_create_4#5 extra_nodes=46 steps_per_extra_node=163.348
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_create_8#9 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11390 steps_first=3736 steps_second=6614 extra=7654 noise=2878 top_rows=allocator_release_linear_find:2880,record_intersection_filter:1524,mapped_units_whole_table:1440 top_share=0.764 round_shares_of_the_raw_difference=1:release_not_covered:-0.015,2:settled:1.015 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_create_8#9 extra_nodes=46 steps_per_extra_node=166.391
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_create_16#17 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11058 steps_first=3736 steps_second=6614 extra=7322 noise=2878 top_rows=allocator_release_linear_find:2760,record_intersection_filter:1460,mapped_units_whole_table:1380 top_share=0.765 round_shares_of_the_raw_difference=1:release_not_covered:-0.016,2:settled:1.016 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_create_16#17 extra_nodes=44 steps_per_extra_node=166.409
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_young_1#2 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=11152 steps_first=3736 steps_second=6614 extra=7416 noise=2878 top_rows=allocator_release_linear_find:2880,mapped_units_whole_table:1440,record_intersection_filter:1410 top_share=0.773 round_shares_of_the_raw_difference=1:release_not_covered:-0.021,2:settled:1.021 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_young_1#2 extra_nodes=46 steps_per_extra_node=161.217
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_young_4#5 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=21074 steps_first=3736 steps_second=6614 extra=17338 noise=2878 top_rows=allocator_release_linear_find:6540,record_intersection_filter:3418,mapped_units_whole_table:3270 top_share=0.763 round_shares_of_the_raw_difference=1:release_not_covered:-0.005,2:placement_not_covered:0.323,3:settled:0.682 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_young_4#5 extra_nodes=44 steps_per_extra_node=394.045
E209RESULT name=extra_steps operation=files_100_units_100__head__filler_young_8#9 selector=first comparable_first=files_100_units_100__head__create_then_first#3 comparable_second=files_100_units_100__head__first_mount_first#4 steps=20510 steps_first=3736 steps_second=6614 extra=16774 noise=2878 top_rows=allocator_release_linear_find:6300,record_intersection_filter:3328,mapped_units_whole_table:3150 top_share=0.762 round_shares_of_the_raw_difference=1:release_not_covered:-0.006,2:placement_not_covered:0.325,3:settled:0.681 verdict=not_resolved
E209RESULT name=extra_steps_per_node operation=files_100_units_100__head__filler_young_8#9 extra_nodes=42 steps_per_extra_node=399.381
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_1#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_2#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_4#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_8#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_16#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_32#1 comparables=0 verdict=no_comparable_operation
E209RESULT name=extra_steps operation=files_100_units_100__head__presweep_64#1 comparables=0 verdict=no_comparable_operation
```

Verdict counts of the `extra_steps` lines (command and output as printed):
```
sed -n '339,373p' research/results/e209-spike-cause-report-second-2026-11-13.out | grep '^E7RESULT E209RESULT name=extra_steps ' | grep -o 'verdict=[a-z_]*' | sort | uniq -c
      7 verdict=no_comparable_operation
      1 verdict=no_extra_steps
     12 verdict=not_resolved
      1 verdict=spread_over_many_rows
```
The only flagged operation with enough resolution is `first_mount_first#1` (`extra=7428`, `noise=6`): its three largest rows (`allocator_release_linear_find`, `mapped_units_whole_table`, `record_intersection_filter`) hold only `top_share=0.771` of `extra`, below 0.9, recorded `spread_over_many_rows`; registration Q11 "any 'spread' ⇒ does not hold". The other 12 flagged operations are `not_resolved` because `noise` is large (2878, 2884, 14726) and by the registration do not enter the judgement; the reason is that the registered way of choosing comparables (other script items at the same point, same selector, same declared units, not flagged, the first two by item name) does not restrict the mount order, so `first_mount_first#4` (an operation after the mount, 6614 steps) was taken as a comparable and raised the noise: a limit of the registered wording, not an apparatus defect (`pick_comparables`, pinned by the unit test `the_comparables_are_the_first_two_unflagged_operations_by_item_name`). One `no_extra_steps` (`walk_from_middle#1`, whose comparable is a `mid` operation in a filler script that takes more steps than the flagged one) and seven `no_comparable_operation` lines (the flag of `presweep_m#1` is on selector `cold_1`, and no other item at the point has an unflagged `cold_1`, `comparables=0`) follow the registration "a flagged operation without comparables does not enter row five, the count is reported". The negative `steps_per_extra_node=-412.692` (line 348) and the large numbers in the `round_shares` field (e.g. `2:settled:628.302`) in the old second-segment combined report are apparatus defects, fixed; in the re-issued report line 348 is `steps_per_extra_node=not_defined_without_positive_extra` and line 339 has `round_shares_of_the_raw_difference=1:release_not_covered:-0.022,2:settled:1.022`, next section. Registration Q12 (the row's location in `crates/`) is required only for operations "located in three rows"; there is none here, so it does not apply. **The remaining quantities cannot flip it** (one "spread" is enough).

Geometry sensitivity (every quantity has one sample point only; `stable=true` only says nothing flipped at one point):

`e209-spike-cause-report-second-2026-11-13-r2.out` lines 402-406:
```
E209RESULT name=geometry_sensitivity quantity=reproduction_gate points=1 per_point=files_100_units_100:reproduced stable=true
E209RESULT name=geometry_sensitivity quantity=freshness_rule_mount_wide points=1 per_point=files_100_units_100:mismatches_257 stable=true
E209RESULT name=geometry_sensitivity quantity=ordinal_equality points=1 per_point=files_100_units_100:false stable=true
E209RESULT name=geometry_sensitivity quantity=mount_carries points=1 per_point=files_100_units_100:mount_carried_the_nodes stable=true
E209RESULT name=geometry_sensitivity quantity=extra_steps_location points=1 per_point=files_100_units_100:no_comparable_operation,no_extra_steps,not_resolved,spread_over_many_rows stable=true
```

Whether rows two, four and five are stable can only be told by the third segment, see the next section.

### Readings and row-by-row judgements of the third-segment combined report (75 artifacts of the first, second and third segments)

The third-segment combined report reads 75 inputs of the first, second and third segments (7 of the first, 20 of the second, 48 of the third, each input written out in full, without the old twin); artifacts and kept forms are in "Current state"; the lines copied below come from the kept form `e209-spike-cause-report-third-2026-11-13.out` (1052 lines) with the `E7RESULT ` prefix removed, and the line numbers are those of the raw form `-raw.out` (1051 lines). The row-by-row judgements follow registration sections six, eight, ten and eleven, judge this row of the fork sheet, and do not judge whether a decision is overturned or established. Registration line numbers all refer to lines of registration `research/prompts/e209-preregistration.md`.

Stop clauses, cross-checks, reproduction gate:

`e209-spike-cause-report-third-2026-11-13.out` line 1051:
```
E209RESULT name=verdict items_read=74 items_without_item_done=zero items_refused=zero constants_mismatches=zero history_mismatches=zero script_sequence_mismatches=zero closure_model_mismatches=zero final_set_not_covering=zero final_nodes_mismatches=zero edge_walk_mismatches=zero live_units_shortfalls=zero negative_control_over_four_rounds=zero twin_differences=zero plain_differences=zero reproduction_at_the_training_point=reproduced
```

`e209-spike-cause-report-third-2026-11-13.out` lines 83-88:
```
E209RESULT name=reproduction_gate point=files_100_units_100 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
E209RESULT name=reproduction_gate point=files_100_units_100_unit_area_2_pow_17 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
E209RESULT name=reproduction_gate point=files_100_units_100_unit_area_2_pow_20 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
E209RESULT name=reproduction_gate point=files_10_units_1000 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=0 flagged_at_28=0 flagged_at_56=0 verdict=not_reproduced
E209RESULT name=reproduction_gate point=files_200_units_100 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=1 verdict=reproduced
E209RESULT name=reproduction_gate point=files_75_units_100 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=1 flagged_at_28=1 flagged_at_56=0 verdict=reproduced
```

All stop counts are `zero`, `items_read=74` (in the 75 inputs the twin has the same name as its original, so it is the same item), `reproduction_at_the_training_point=reproduced`; the registered stop clauses (lines 368-374) do not stop. Five of the six points of the reproduction gate are `reproduced`; `files_10_units_1000` is `not_reproduced` (`flagged_at_14=0 flagged_at_28=0 flagged_at_56=0`): the first operation of its `walk_forward` has `final_nodes=17`, flag line 28, and no operation is flagged; the command below counts 0 `flagged=true` among its 54 operation rows:
```
grep -c 'name=operation_row item=files_10_units_1000' research/results/e209-spike-cause-report-third-2026-11-13.out; grep 'name=operation_row item=files_10_units_1000' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -c ' flagged=true'
54
0
```

Registration Q0 (line 258): both lines 0 is "not reproduced", and rows one to four at that point are "the apparatus cannot reach it" (denominator 0, reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`, not "not hit" or "the spike does not exist"), row five has no flagged operation to judge; invalidation clause 1 (line 361): only that point is set aside, not everything; F3 (line 346) fires only if both `files_100_units_100` and `files_200_units_100` have no flagged operation, and both are `reproduced` (lines 83 and 87), so F3 does not fire.

Verdict lines read one by one (command and verbatim output):

```
for f in first-2026-11-13-r3 second-2026-11-13-r2 third-2026-11-13; do g=research/results/e209-spike-cause-report-$f.out; echo "$g verdict_lines=$(grep -c 'name=verdict' $g) false_or_not_run_in_the_verdict_line=$(grep 'name=verdict' $g | grep -c 'false\|not_run') non_zero_fields=$(grep 'name=verdict' $g | grep -o ' [a-z_]*=[^ ]*' | grep -v '=zero$' | tr '\n' ' ')"; done
research/results/e209-spike-cause-report-first-2026-11-13-r3.out verdict_lines=1 false_or_not_run_in_the_verdict_line=0 non_zero_fields= name=verdict  items_read=6  reproduction_at_the_training_point=reproduced
research/results/e209-spike-cause-report-second-2026-11-13-r2.out verdict_lines=1 false_or_not_run_in_the_verdict_line=0 non_zero_fields= name=verdict  items_read=26  reproduction_at_the_training_point=reproduced
research/results/e209-spike-cause-report-third-2026-11-13.out verdict_lines=1 false_or_not_run_in_the_verdict_line=0 non_zero_fields= name=verdict  items_read=74  reproduction_at_the_training_point=reproduced
```

Each of the three re-issued combined reports has exactly one `name=verdict`; the verdict line has no `false` or `not_run` field, and the counts for violations, mismatches, ambiguities and failures are all `zero`; `items_read` and `reproduction_at_the_training_point=reproduced` are values the registration expected. Two "not passed" fields lie outside the verdict line: line 86 `reproduction_gate ... verdict=not_reproduced` (`files_10_units_1000`; registration Q0 names this value and its consequence, expected) and the `stable=false` of the five `geometry_sensitivity` lines 1046-1050 (read one by one under "geometry sensitivity" below; command and verbatim output):
```
grep -c 'name=geometry_sensitivity.*stable=false' research/results/e209-spike-cause-report-third-2026-11-13.out
5
```

No crash judging stage was run (no `gpu`), so `NOT_RUN` does not apply.

**Row one (freshness rule): decidable, does not hold** (registration section eleven "stop when enough to decide", line 382: one failure of row one is enough; one mismatch on the test set means does not hold, Q2 line 260, F1 line 344).

`e209-spike-cause-report-third-2026-11-13.out` lines 831-834:
```
E209RESULT name=freshness_rule variant=mount_wide threshold=0 flagged_in_training=4 training_failures=27 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=17 script_unflagged=230 script_mismatches=230 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=2:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=4:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=2:observed_flag=0 geometry_flagged=31 geometry_unflagged=423 geometry_mismatches=423 geometry_mismatch_labels=files_75_units_100__head__filler_create_4#1:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#2:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#3:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#4:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#6:fresh=0:observed_flag=0 verdict=does_not_hold
E209RESULT name=freshness_rule variant=mount_narrow threshold=18 flagged_in_training=4 training_failures=0 training_failure_labels=- script_flagged=17 script_unflagged=230 script_mismatches=1 script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0 geometry_flagged=31 geometry_unflagged=423 geometry_mismatches=1 geometry_mismatch_labels=files_75_units_100__head__first_mount_first#1:fresh=28:observed_flag=0 verdict=does_not_hold
E209RESULT name=freshness_rule variant=ever_wide threshold=0 flagged_in_training=4 training_failures=27 training_failure_labels=files_100_units_100__head__first_repeated#2:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#3:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#4:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#5:fresh=0:observed_flag=0,files_100_units_100__head__first_repeated#6:fresh=0:observed_flag=0 script_flagged=17 script_unflagged=230 script_mismatches=230 script_mismatch_labels=files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#1:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#2:fresh=0:observed_flag=0,files_100_units_100__head__first_mount_first#3:fresh=0:observed_flag=0,files_100_units_100__head__mount_then_first#1:fresh=0:observed_flag=0 geometry_flagged=31 geometry_unflagged=423 geometry_mismatches=423 geometry_mismatch_labels=files_75_units_100__head__filler_create_4#1:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#2:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#3:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#4:fresh=0:observed_flag=0,files_75_units_100__head__filler_create_4#6:fresh=0:observed_flag=0 verdict=does_not_hold
E209RESULT name=freshness_rule variant=ever_narrow threshold=18 flagged_in_training=4 training_failures=0 training_failure_labels=- script_flagged=17 script_unflagged=230 script_mismatches=1 script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0 geometry_flagged=31 geometry_unflagged=423 geometry_mismatches=1 geometry_mismatch_labels=files_75_units_100__head__first_mount_first#1:fresh=28:observed_flag=0 verdict=does_not_hold
```

The two wide variants have `training_failures=27` and `script_mismatches=230`; the two narrow variants have 0 training failures, 1 mismatch on the script test set (`old_mount_old#1`, already read by the second-segment combined report) and on the test geometry `geometry_flagged=31`, `geometry_unflagged=423`, `geometry_mismatches=1`, the one mismatch being `files_75_units_100__head__first_mount_first#1:fresh=28:observed_flag=0`. All four variants are `does_not_hold`. F6 (line 349) does not fire: the test geometry has 31 flagged operations (≥ 3).
The two narrow-reading mismatches have the same cause (registration reading 1, line 24: flag line `max(28, 4 × lower median of the window)`, window = all script operations of one mount in one item): when the window holds one or two operations the lower median is the smaller of them and the 4-times line moves with the window. Command and verbatim output (the operations of each mount of each item; mount 0 is the first batch after the history):
```
grep 'name=operation_row item=files_100_units_100__head__first_mount_first ' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -o 'mount_epoch=[0-9]* ordinal_user=[0-9]* step_index=[0-9]* operation=[a-z_0-9]* final_nodes=[0-9]* flagged=[a-z]*'
mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_first final_nodes=53 flagged=true
mount_epoch=0 ordinal_user=2 step_index=2 operation=overwrite_old final_nodes=13 flagged=false
mount_epoch=1 ordinal_user=1 step_index=4 operation=overwrite_first final_nodes=9 flagged=false
mount_epoch=1 ordinal_user=2 step_index=5 operation=overwrite_old final_nodes=9 flagged=false
mount_epoch=1 ordinal_user=3 step_index=6 operation=overwrite_mid final_nodes=13 flagged=false
```

```
grep 'name=operation_row item=files_75_units_100__head__first_mount_first ' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -o 'mount_epoch=[0-9]* ordinal_user=[0-9]* step_index=[0-9]* operation=[a-z_0-9]* final_nodes=[0-9]* flagged=[a-z]*'
mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_first final_nodes=39 flagged=false
mount_epoch=0 ordinal_user=2 step_index=2 operation=overwrite_old final_nodes=11 flagged=false
mount_epoch=1 ordinal_user=1 step_index=4 operation=overwrite_first final_nodes=9 flagged=false
mount_epoch=1 ordinal_user=2 step_index=5 operation=overwrite_old final_nodes=7 flagged=false
mount_epoch=1 ordinal_user=3 step_index=6 operation=overwrite_mid final_nodes=13 flagged=false
```

```
grep 'name=operation_row item=files_100_units_100__head__old_mount_old ' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -o 'mount_epoch=[0-9]* ordinal_user=[0-9]* step_index=[0-9]* operation=[a-z_0-9]* final_nodes=[0-9]* flagged=[a-z]*'
mount_epoch=0 ordinal_user=1 step_index=1 operation=overwrite_old final_nodes=53 flagged=false
mount_epoch=1 ordinal_user=1 step_index=3 operation=overwrite_old final_nodes=9 flagged=false
mount_epoch=1 ordinal_user=2 step_index=4 operation=overwrite_first final_nodes=11 flagged=false
```

At the training point the first mount of `first_mount_first` holds two operations, 53 and 13, lower median 13, flag line `max(28, 52) = 52`, so the 53-node `first` is flagged (by 1 only); in the same item at `files_75_units_100` the two operations are 39 and 11, flag line `max(28, 44) = 44`, so 39 nodes are not flagged; the first mount of `old_mount_old` has a single operation, 53, flag line 212, not flagged. The apparatus computed the registered formula correctly and this is not an apparatus defect; the registration's "one mismatch means does not hold" is applied literally. The position follows the script and the count does not (line 838 `position_follows=true`, `count_follows=false`, 21 flagged among 267 retouched operations). **The remaining quantities cannot flip it** (mismatches only grow; the training failures of the wide readings are fixed on the training set).

**Row two (the flag follows only the ordinal): decidable, does not hold.** At the five points that reproduce, `flag_depends_only_on_the_ordinal=false` and `ordinal_shift` is `follows_the_operation` (registration Q4 line 262: any difference means does not hold; Q5 line 263: "follows the operation" means does not hold):

`e209-spike-cause-report-third-2026-11-13.out` line 839:
```
E209RESULT name=ordinal_equality point=files_100_units_100 script_pairs=300 flag_depends_only_on_the_ordinal=false first_difference=create_then_first/filler_create_16#2
```

`e209-spike-cause-report-third-2026-11-13.out` line 848:
```
E209RESULT name=ordinal_shift point=files_100_units_100 walk_forward_first_flagged_ordinal=1 fillers=8 verdict=follows_the_operation
```

`e209-spike-cause-report-third-2026-11-13.out` line 886:
```
E209RESULT name=ordinal_equality point=files_100_units_100_unit_area_2_pow_17 script_pairs=45 flag_depends_only_on_the_ordinal=false first_difference=filler_create_4/first_mount_first#1
```

`e209-spike-cause-report-third-2026-11-13.out` line 889:
```
E209RESULT name=ordinal_shift point=files_100_units_100_unit_area_2_pow_17 walk_forward_first_flagged_ordinal=1 fillers=2 verdict=follows_the_operation
```

`e209-spike-cause-report-third-2026-11-13.out` line 906:
```
E209RESULT name=ordinal_equality point=files_100_units_100_unit_area_2_pow_20 script_pairs=45 flag_depends_only_on_the_ordinal=false first_difference=filler_create_4/first_mount_first#1
```

`e209-spike-cause-report-third-2026-11-13.out` line 909:
```
E209RESULT name=ordinal_shift point=files_100_units_100_unit_area_2_pow_20 walk_forward_first_flagged_ordinal=1 fillers=2 verdict=follows_the_operation
```

`e209-spike-cause-report-third-2026-11-13.out` line 932:
```
E209RESULT name=ordinal_equality point=files_200_units_100 script_pairs=45 flag_depends_only_on_the_ordinal=false first_difference=filler_create_4/first_mount_first#1
```

`e209-spike-cause-report-third-2026-11-13.out` line 935:
```
E209RESULT name=ordinal_shift point=files_200_units_100 walk_forward_first_flagged_ordinal=1 fillers=2 verdict=follows_the_operation
```

`e209-spike-cause-report-third-2026-11-13.out` line 954:
```
E209RESULT name=ordinal_equality point=files_75_units_100 script_pairs=45 flag_depends_only_on_the_ordinal=false first_difference=filler_create_4/first_repeated#1
```

`e209-spike-cause-report-third-2026-11-13.out` line 957:
```
E209RESULT name=ordinal_shift point=files_75_units_100 walk_forward_first_flagged_ordinal=1 fillers=2 verdict=follows_the_operation
```

The one point `files_10_units_1000`:
`e209-spike-cause-report-third-2026-11-13.out` line 926:
```
E209RESULT name=ordinal_equality point=files_10_units_1000 script_pairs=28 flag_depends_only_on_the_ordinal=true first_difference=-
```

`e209-spike-cause-report-third-2026-11-13.out` line 929:
```
E209RESULT name=ordinal_shift point=files_10_units_1000 walk_forward_first_flagged_ordinal=- fillers=2 verdict=neither
```

Its `flag_depends_only_on_the_ordinal=true` holds vacuously: none of the 54 operation rows of that point is flagged (command above), the flags of the 28 script pairs are all "not flagged", so of course equal at every ordinal; `ordinal_shift` is `neither` (no first flagged ordinal to compare). Under Q0 and invalidation clause 1 row two at that point is "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`), and this `true` is not taken as a value of "the flag follows only the ordinal". **The remaining quantities cannot flip it.**

**Row three (a stored quantity crossing a threshold): decidable, does not hold.** All eight `stored_quantity` cells are `does_not_hold`, with 20-241 pooled errors (≥ 1) and 31-423 test-geometry errors (registration Q6 line 264: all eight cells wrong at least once after pooling means row three does not hold; F7 line 350 fires only with 0 pooled errors and at least 1 test-geometry error, so it does not fire):

`e209-spike-cause-report-third-2026-11-13.out` lines 970-977:
```
E209RESULT name=stored_quantity quantity=stale_nodes relative=false threshold=0.0000 pooled_errors=241 flagged_values=1 unflagged_values=1 geometry_errors=423 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=stale_nodes relative=true threshold=0.0000 pooled_errors=241 flagged_values=1 unflagged_values=1 geometry_errors=423 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=chain_depth relative=false threshold=23.0000 pooled_errors=39 flagged_values=4 unflagged_values=5 geometry_errors=36 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=chain_depth relative=true threshold=0.3770 pooled_errors=39 flagged_values=4 unflagged_values=5 geometry_errors=37 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=released_backlog relative=false threshold=17788.0000 pooled_errors=22 flagged_values=10 unflagged_values=134 geometry_errors=31 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=released_backlog relative=true threshold=291.6066 pooled_errors=22 flagged_values=10 unflagged_values=134 geometry_errors=31 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=false threshold=150.0000 pooled_errors=20 flagged_values=6 unflagged_values=82 geometry_errors=33 verdict=does_not_hold
E209RESULT name=stored_quantity quantity=publishes_since_flag relative=true threshold=2.4590 pooled_errors=20 flagged_values=6 unflagged_values=82 geometry_errors=34 verdict=does_not_hold
```

`presweep` (registration Q7 line 265) is literally `threshold_between_two_sample_points` at the five points that reproduce, and `threshold_sides_not_both_sampled` at `files_10_units_1000` (at `m = 0` its `first` is not flagged either, so there is no flagged side):

`e209-spike-cause-report-third-2026-11-13.out` line 986:
```
E209RESULT name=presweep point=files_100_units_100 flagged_counts=0 unflagged_counts=1,2,4,8,16,32,64 between=0..1 verdict=threshold_between_two_sample_points
```

`e209-spike-cause-report-third-2026-11-13.out` line 1007:
```
E209RESULT name=presweep point=files_100_units_100_unit_area_2_pow_17 flagged_counts=0 unflagged_counts=8,32 between=0..8 verdict=threshold_between_two_sample_points
```

`e209-spike-cause-report-third-2026-11-13.out` line 1017:
```
E209RESULT name=presweep point=files_100_units_100_unit_area_2_pow_20 flagged_counts=0 unflagged_counts=8,32 between=0..8 verdict=threshold_between_two_sample_points
```

`e209-spike-cause-report-third-2026-11-13.out` line 1035:
```
E209RESULT name=presweep point=files_200_units_100 flagged_counts=0 unflagged_counts=8,32 between=0..8 verdict=threshold_between_two_sample_points
```

`e209-spike-cause-report-third-2026-11-13.out` line 1045:
```
E209RESULT name=presweep point=files_75_units_100 flagged_counts=0 unflagged_counts=8,32 between=0..8 verdict=threshold_between_two_sample_points
```

`e209-spike-cause-report-third-2026-11-13.out` line 1025:
```
E209RESULT name=presweep point=files_10_units_1000 flagged_counts=- unflagged_counts=0 between=- verdict=threshold_sides_not_both_sampled
```

As in the second segment's reading of the training point, this literal judgement is not read as a quantity threshold: in every `presweep_m` (m ≥ 1) the flag has only moved to the earlier `cold_1` operation (ordinal 1), each script still has one flagged operation; and `stale_nodes` is 0 at every sample point of `presweep` (flagged and not flagged alike). Command and verbatim output:
```
grep 'name=presweep_point' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -o 'point=[a-z0-9_]* presweep_count=[0-9]* first_flagged=[a-z]* stale_nodes=[0-9]*'
point=files_100_units_100 presweep_count=0 first_flagged=true stale_nodes=0
point=files_100_units_100 presweep_count=1 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=2 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=4 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=8 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=16 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=32 first_flagged=false stale_nodes=0
point=files_100_units_100 presweep_count=64 first_flagged=false stale_nodes=0
point=files_100_units_100_unit_area_2_pow_17 presweep_count=0 first_flagged=true stale_nodes=0
point=files_100_units_100_unit_area_2_pow_17 presweep_count=8 first_flagged=false stale_nodes=0
point=files_100_units_100_unit_area_2_pow_17 presweep_count=32 first_flagged=false stale_nodes=0
point=files_100_units_100_unit_area_2_pow_20 presweep_count=0 first_flagged=true stale_nodes=0
point=files_100_units_100_unit_area_2_pow_20 presweep_count=8 first_flagged=false stale_nodes=0
point=files_100_units_100_unit_area_2_pow_20 presweep_count=32 first_flagged=false stale_nodes=0
point=files_10_units_1000 presweep_count=0 first_flagged=false stale_nodes=0
point=files_200_units_100 presweep_count=0 first_flagged=true stale_nodes=0
point=files_200_units_100 presweep_count=8 first_flagged=false stale_nodes=0
point=files_200_units_100 presweep_count=32 first_flagged=false stale_nodes=0
point=files_75_units_100 presweep_count=0 first_flagged=true stale_nodes=0
point=files_75_units_100 presweep_count=8 first_flagged=false stale_nodes=0
point=files_75_units_100 presweep_count=32 first_flagged=false stale_nodes=0
```

```
grep 'overwrite_cold_1' research/results/e209-spike-cause-report-third-2026-11-13.out | grep 'flagged=true' | grep -o 'item=[a-z0-9_]*__head__presweep_[0-9]* mount_epoch=0 ordinal_user=[0-9]*'
item=files_100_units_100__head__presweep_1 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_2 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_4 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_8 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_16 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_32 mount_epoch=0 ordinal_user=1
item=files_100_units_100__head__presweep_64 mount_epoch=0 ordinal_user=1
item=files_75_units_100__head__presweep_32 mount_epoch=0 ordinal_user=1
item=files_75_units_100__head__presweep_8 mount_epoch=0 ordinal_user=1
item=files_200_units_100__head__presweep_32 mount_epoch=0 ordinal_user=1
item=files_200_units_100__head__presweep_8 mount_epoch=0 ordinal_user=1
item=files_100_units_100_unit_area_2_pow_17__head__presweep_32 mount_epoch=0 ordinal_user=1
item=files_100_units_100_unit_area_2_pow_17__head__presweep_8 mount_epoch=0 ordinal_user=1
item=files_100_units_100_unit_area_2_pow_20__head__presweep_32 mount_epoch=0 ordinal_user=1
item=files_100_units_100_unit_area_2_pow_20__head__presweep_8 mount_epoch=0 ordinal_user=1
```

`overwrite_cold_1` is the flagged operation at ordinal 1 in every `presweep_m` (m ≥ 1) item (7 at the training point, 2 at each of the four test points), and the `first` step is not flagged. For the two `stale_nodes` cells the flagged values and the not-flagged values each have only 1 distinct value (`flagged_values=1 unflagged_values=1`), so they would not meet "at least 2 distinct values each" anyway. **The remaining quantities cannot flip it** (pooling uses only the training set and the script test set, the test geometry does not enter pooling; the `presweep` sequence is required only at the training point and the two test-point items give the geometry sensitivity).

**Row four (carried out by the remount): holds (five points that reproduce), one point the apparatus cannot reach (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`).** Registration Q8 (line 266): premise holds (`first` is not flagged after the mount) and `share ≥ 0.9` means "the mount carried those nodes"; Q9 (line 267): the largest step holding at least 90% means "located in one step". The five points that reproduce have `share` 0.949-0.981, all located in the one step `write_row`:

`e209-spike-cause-report-third-2026-11-13.out` lines 849-850:
```
E209RESULT name=mount_carries point=files_100_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6
E209RESULT name=mount_carried_step point=files_100_units_100 carried_nodes_by_first_step=write_row:51,warm_up_1:0,warm_up_2:0,warm_up_3:0,warm_up_4:0,warm_up_5:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

`e209-spike-cause-report-third-2026-11-13.out` lines 890-891:
```
E209RESULT name=mount_carries point=files_100_units_100_unit_area_2_pow_17 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6
E209RESULT name=mount_carried_step point=files_100_units_100_unit_area_2_pow_17 carried_nodes_by_first_step=write_row:51,warm_up_1:0,warm_up_2:0,warm_up_3:0,warm_up_4:0,warm_up_5:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

`e209-spike-cause-report-third-2026-11-13.out` lines 910-911:
```
E209RESULT name=mount_carries point=files_100_units_100_unit_area_2_pow_20 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6
E209RESULT name=mount_carried_step point=files_100_units_100_unit_area_2_pow_20 carried_nodes_by_first_step=write_row:51,warm_up_1:0,warm_up_2:0,warm_up_3:0,warm_up_4:0,warm_up_5:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

`e209-spike-cause-report-third-2026-11-13.out` lines 936-937:
```
E209RESULT name=mount_carries point=files_200_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=105 carried_by_the_mount=103 share=0.981 verdict=mount_carried_the_nodes remount_publishes=4
E209RESULT name=mount_carried_step point=files_200_units_100 carried_nodes_by_first_step=write_row:103,warm_up_1:0,warm_up_2:0,warm_up_3:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

`e209-spike-cause-report-third-2026-11-13.out` lines 958-959:
```
E209RESULT name=mount_carries point=files_75_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=39 carried_by_the_mount=37 share=0.949 verdict=mount_carried_the_nodes remount_publishes=4
E209RESULT name=mount_carried_step point=files_75_units_100 carried_nodes_by_first_step=write_row:37,warm_up_1:0,warm_up_2:0,warm_up_3:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

The locations of the plan function and the fixed-point call sites, fetched now (`row_publish_plan_on_file_version` at `crates/singlefs-core/src/mount.rs:5841`, the empty plan at `:5882`; the call site `transaction.rs:13435` and merge sites `:14087`, `:14116`; this segment did not change `crates/`):
```
grep -n 'fn row_publish_plan_on_file_version\|fn empty_publish_plan_after_file_version' crates/singlefs-core/src/mount.rs
5841:fn row_publish_plan_on_file_version<'plan>(
5882:pub(crate) fn empty_publish_plan_after_file_version<'plan>(
```

```
grep -n 'merge_into_the_rewrite_set_walking_the_release_chain(\|let settled = settle_the_allocation_record_tree(' crates/singlefs-core/src/transaction.rs
2046:            merge_into_the_rewrite_set_walking_the_release_chain(
2070:        merge_into_the_rewrite_set_walking_the_release_chain(
13435:    let settled = settle_the_allocation_record_tree(
14087:            merge_into_the_rewrite_set_walking_the_release_chain(
14116:        merge_into_the_rewrite_set_walking_the_release_chain(
14133:fn merge_into_the_rewrite_set_walking_the_release_chain(
```

`files_10_units_1000`:
`e209-spike-cause-report-third-2026-11-13.out` lines 930-931:
```
E209RESULT name=mount_carries point=files_10_units_1000 first_after_the_mount_flagged=false nodes_of_the_spike=17 carried_by_the_mount=15 share=0.882 verdict=partly_carried remount_publishes=4
E209RESULT name=mount_carried_step point=files_10_units_1000 carried_nodes_by_first_step=write_row:15,warm_up_1:0,warm_up_2:0,warm_up_3:0 located_step=write_row verdict=located_one_step first_merge_site=release_not_covered plan_function=row_publish_plan_on_file_version plan_position=crates/singlefs-core/src/mount.rs:5841
```

At that point `first_after_the_mount_flagged=false`, `nodes_of_the_spike=17` (the `seed ∪ pulled` count of the first operation, which itself has `final_nodes=17 < 28` and is not flagged), `carried_by_the_mount=15`, `share=0.882`, literally `partly_carried` under Q8 "otherwise partly"; but the spike does not reproduce at that point, and under Q0 and invalidation clause 1 it is "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`): the premise that the batch of `first` in `walk_forward` is the spike does not hold, so this 0.882 is not taken as a "partly" judgement. Both readings are written side by side and the registration is not chosen for. **The remaining quantities cannot flip it** (the five points that reproduce already decide "holds").

**Row five (which step holds the extra steps): decidable, does not hold.** Judgement counts per point (command and verbatim output):

```
grep 'name=extra_steps ' research/results/e209-spike-cause-report-third-2026-11-13.out | grep -o 'operation=files_[0-9a-z_]*__head\|verdict=[a-z_]*$' | sed 's/__head//' | paste - - | tr '\t' ' ' | sort | uniq -c
      2 operation=files_100_units_100_unit_area_2_pow_17 verdict=no_comparable_operation
      5 operation=files_100_units_100_unit_area_2_pow_17 verdict=not_resolved
      1 operation=files_100_units_100_unit_area_2_pow_17 verdict=spread_over_many_rows
      2 operation=files_100_units_100_unit_area_2_pow_20 verdict=no_comparable_operation
      5 operation=files_100_units_100_unit_area_2_pow_20 verdict=not_resolved
      1 operation=files_100_units_100_unit_area_2_pow_20 verdict=spread_over_many_rows
      7 operation=files_100_units_100 verdict=no_comparable_operation
      1 operation=files_100_units_100 verdict=no_extra_steps
     12 operation=files_100_units_100 verdict=not_resolved
      1 operation=files_100_units_100 verdict=spread_over_many_rows
      2 operation=files_200_units_100 verdict=no_comparable_operation
      2 operation=files_200_units_100 verdict=not_resolved
      5 operation=files_200_units_100 verdict=spread_over_many_rows
      2 operation=files_75_units_100 verdict=no_comparable_operation
      2 operation=files_75_units_100 verdict=no_extra_steps
      2 operation=files_75_units_100 verdict=not_resolved
```

Eight flagged operations have enough resolution and all are `spread_over_many_rows` (command and verbatim output; `top` is the share of the three largest rows in `extra`, threshold 0.9):
```
grep 'name=extra_steps ' research/results/e209-spike-cause-report-third-2026-11-13.out | grep spread_over | sed -E 's/.*operation=([^ ]*) .*noise=([0-9]+).*top_share=([0-9.]+).*/\1 noise=\2 top=\3/'
files_100_units_100__head__first_mount_first#1 noise=6 top=0.771
files_100_units_100_unit_area_2_pow_17__head__walk_backward#3 noise=48 top=0.717
files_100_units_100_unit_area_2_pow_20__head__walk_backward#3 noise=50 top=0.717
files_200_units_100__head__filler_young_4#5 noise=3216 top=0.777
files_200_units_100__head__first_mount_first#1 noise=2558 top=0.775
files_200_units_100__head__first_repeated#1 noise=658 top=0.777
files_200_units_100__head__walk_backward#4 noise=44 top=0.776
files_200_units_100__head__walk_forward#1 noise=3216 top=0.777
```

Registration Q11 (line 269): any "spread" means does not hold; F8 (line 351) is open only if all flagged operations lack resolution, and does not fire; Q12 (line 270) is required only of operations "located to three rows", and there is none, so it does not apply. The training point, the two U points and F = 200 each have a spread operation; `files_75_units_100` has no flagged operation with enough resolution (2 `no_comparable_operation`, 2 `no_extra_steps`, 2 `not_resolved`), so nothing is judged there and nothing is judged otherwise; `files_10_units_1000` has no flagged operation, so row five has nothing to judge. The 26 `not_resolved` and 15 `no_comparable_operation` have the same causes as in the second segment (the comparable wording does not restrict the mount order; the flag of `presweep_m#1` is on `cold_1`). **The remaining quantities cannot flip it** (one spread is enough for does not hold).

**Geometry sensitivity.** Lines 1046-1050:

`e209-spike-cause-report-third-2026-11-13.out` lines 1046-1050:
```
E209RESULT name=geometry_sensitivity quantity=reproduction_gate points=6 per_point=files_100_units_100:reproduced,files_100_units_100_unit_area_2_pow_17:reproduced,files_100_units_100_unit_area_2_pow_20:reproduced,files_10_units_1000:not_reproduced,files_200_units_100:reproduced,files_75_units_100:reproduced stable=false
E209RESULT name=geometry_sensitivity quantity=freshness_rule_mount_wide points=6 per_point=files_100_units_100:mismatches_257,files_100_units_100_unit_area_2_pow_17:mismatches_92,files_100_units_100_unit_area_2_pow_20:mismatches_92,files_10_units_1000:mismatches_54,files_200_units_100:mismatches_91,files_75_units_100:mismatches_94 stable=false
E209RESULT name=geometry_sensitivity quantity=ordinal_equality points=6 per_point=files_100_units_100:false,files_100_units_100_unit_area_2_pow_17:false,files_100_units_100_unit_area_2_pow_20:false,files_10_units_1000:true,files_200_units_100:false,files_75_units_100:false stable=false
E209RESULT name=geometry_sensitivity quantity=mount_carries points=6 per_point=files_100_units_100:mount_carried_the_nodes,files_100_units_100_unit_area_2_pow_17:mount_carried_the_nodes,files_100_units_100_unit_area_2_pow_20:mount_carried_the_nodes,files_10_units_1000:partly_carried,files_200_units_100:mount_carried_the_nodes,files_75_units_100:mount_carried_the_nodes stable=false
E209RESULT name=geometry_sensitivity quantity=extra_steps_location points=6 per_point=files_100_units_100:no_comparable_operation,no_extra_steps,not_resolved,spread_over_many_rows,files_100_units_100_unit_area_2_pow_17:no_comparable_operation,not_resolved,spread_over_many_rows,files_100_units_100_unit_area_2_pow_20:no_comparable_operation,not_resolved,spread_over_many_rows,files_10_units_1000:-,files_200_units_100:no_comparable_operation,not_resolved,spread_over_many_rows,files_75_units_100:no_comparable_operation,no_extra_steps,not_resolved stable=false
```

The apparatus `stable` is point-by-point equality of the value strings (`geometry_sensitivity_line`), and all five rows are `stable=false`; read row by row under registration F10 (line 353: a flip means the same row and the same variant judged differently at two sample points, with the knob and the two sample points listed) and invalidation clause 1, with two readings side by side:

| Quantity (which row) | Source of `stable=false` | Among the five points that reproduce | Against `files_10_units_1000`: literal / under invalidation clause 1 |
|---|---|---|---|
| `reproduction_gate` (Q0) | five `reproduced`, one `not_reproduced` | agree | literal: judged differently, the knob being "files F and units per file K in opposite directions" (registration section eight, table row three), the two points `files_10_units_1000` (F = 10, K = 1000) against `files_100_units_100` (F = 100, K = 100); this is a flip of the reproduction gate itself, not of any row's judgement |
| `freshness_rule_mount_wide` (row one) | mismatch counts differ per point (257, 92, 92, 54, 91, 94), the apparatus compares count strings | every point has ≥ 1 mismatches, all does not hold, no flip | the wide reading there has 54 mismatches, also does not hold, but under invalidation clause 1 row one is not judged at that point |
| `ordinal_equality` (row two) | `files_10_units_1000` is `true`, the rest `false` | agree (all `false`) | literal: `true` against `false`; that point holds vacuously (no flagged operation) and under invalidation clause 1 is "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`), not a judgement flip |
| `mount_carries` (row four) | `files_10_units_1000` is `partly_carried`, the rest `mount_carried_the_nodes` | agree (`share` 0.949-0.981) | literal: `partly_carried` against `mount_carried_the_nodes`; the spike does not reproduce there, and under invalidation clause 1 it is "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`) |
| `extra_steps_location` (row five) | the kinds of judgement that appear differ per point (four at the training point, no `spread_over_many_rows` at F = 75, none at `files_10_units_1000`) | the only judgement of operations with enough resolution is "spread"; no point judges "holds" or "located to three rows"; F = 75 has no operation with enough resolution, so nothing is judged | no flagged operation there, so row five has nothing to judge |

Conclusion: **a flip in the registration's sense**: among the five points that reproduce (F = 75, 100, 200; U = 2¹⁷, 2¹⁹, 2²⁰) the judgements of rows one to five do not differ anywhere; the flip is only at `files_10_units_1000`, and literally under F10 it is one unstable quantity, the reproduction gate (knob K and F in opposite directions, the two points `files_10_units_1000` and `files_100_units_100`); rows one to four at that point are "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`) under invalidation clause 1 and row five has no flagged operation, so these rows are not recorded as judgement flips; if the literal values at that point are also counted as "judged differently" (F10 says "any quantity"), rows two and four are also recorded "unstable" with the same knob. The registration does not fix whether that point counts as a sample point; both readings are written, and "this is a property of the policy" is not written. The judgements of the five rows (does not hold, does not hold, does not hold, holds, does not hold) do not flip at any of the five points that reproduce.

**The fourth segment (conditional) is not started**: registration section five (line 239) and the segment table (line 248): it starts only if some row flips between F = 75, 100 and 200 and a larger geometry is needed to separate the flip point. Between F = 75, 100 and 200 the judgements of the five rows agree (row five has no operation with enough resolution at F = 75, which is not a different judgement), and the only flip is at the point where K and F go in opposite directions, which `files_250_units_100` and `files_300_units_100` do not move, so they cannot separate it; the five rows are all decidable (registration section eleven "stop when enough to decide", lines 376-383). The `walk_forward` and `mount_then_first` of the two fourth-segment geometries are marked "not run after enough to decide", corresponding to the geometry sensitivity of the five fork-sheet rows; whether to start it is for the main agent.


### Apparatus defects (two read from the second-segment combined report, fixed; the three combined reports re-issued)

- `round_shares`: `round_shares` in `scripts/experiment-apparatus/e209_spike_cause.rs` divided the per-round difference of the `round_row` steps by the B8 model's `extra`: `round_row` is measured raw steps (the two rounds of `first_mount_first#1` total as below, command and output as printed), `extra` is the model steps computed by `by_delta_rows` from the state, so the units differ and the shares came out as large numbers such as `-13.819` and `628.302`. Changed to divide by the sum of the round differences (a sum that is not positive is recorded as 0) and the field renamed `round_shares_of_the_raw_difference`; unit test `the_round_shares_are_shares_of_the_sum_of_the_round_differences`, mutation row `x3_round_share_divides_by_the_round_count`.
```
f=research/results/e209-spike-cause-files_100_units_100-head-first_mount_first-2026-11-13.out; grep 'name=round_row' $f | grep 'mount_epoch=0 operation=overwrite_first operation_index=1 ' | awk '{for(i=1;i<=NF;i++){if($i~/^round=/)r=$i; if($i~/^steps=/){split($i,a,"=");s[r]+=a[2]}}} END{for(k in s)print k,s[k]}'
round=1 5276590
round=2 11513136
```
- `steps_per_extra_node`: it also produced a ratio for a non-positive extra step count (line 348: `extra=-10730`, `extra_nodes=26`, giving `-412.692`), which is meaningless. Changed so that the ratio exists only when both the extra steps and the extra nodes are positive, otherwise `not_defined_without_positive_extra`; unit test `the_steps_per_extra_node_exists_only_for_a_positive_extra_in_steps_and_in_nodes`, mutation row `x4_steps_per_extra_node_allows_a_negative_extra`.
- Both changes touch only fields and numbers in the report, no criterion and no judgement; the unit tests after the change (42 passed), the mutation table (all 19 caught) and the three re-issued combined reports are done (outcomes in "Current state"); the `-13.819`, `628.302` and `-412.692` in the two items above come from the old second-segment combined report `e209-spike-cause-report-second-2026-11-13.out`, and in the re-issued second-segment `-r2` the same places are `-0.022` and `1.022` on line 339 and `not_defined_without_positive_extra` on line 348; this page now reads the re-issued ones.

### Rerun

```bash
# Combined reports (the model belongs to the main agent, scheduled through task-server): one each for the 7 artifacts of the first segment, the 27 of the first and second segments and the 75 of the first, second and third segments; the output is written straight to a new file (`--out`) and then converted to the kept form with the E7RESULT prefix and closing line
task-server/task submit --cwd scripts/experiment-apparatus -- cargo run --release --bin e209-spike-cause -- report --crates-root /home/fy5090/code/singlefs --out <new file> <inputs...>
bash scripts/experiment-apparatus/replay.sh E209    # replay-table rows E209, three rows: @driver_e209_report_first (kept `-r3`), @driver_e209_report_second (kept `-r2`), @driver_e209_report_third (kept third-segment combined report); all three kept artifacts are stored
```

The inputs, commands and kept forms of the three combined reports are in "Current state"; "Readings and row-by-row judgements of the second-segment combined report" on this page reads the re-issued second-segment `-r2`, and "Readings and row-by-row judgements of the third-segment combined report" reads the third-segment combined report.

### Measurement basis

- Raw output is kept in the artifacts named above; the combined report is a pure-reading recomputation of the artifacts by the model, no project code is started.
- A flagged operation follows the E207 reading: `final_nodes ≥ max(28, 4 × lower median of final_nodes of all operations in the same window of the same point and state)`; the floor is additionally swept at 14, 28 and 56.
- Freshness: the nodes dirtied at the first merge of the rewrite set in this publish fixed point plus the nodes newly pulled in along the release chain, minus the nodes already carried out by the settled sets since the mount (or since the pool was created); counted as a number.

### What it cannot answer

- The narrow readings of row one have two mismatches in all on the test sets, both because the flag line of reading 1 moves with the window when the window holds few operations: at the training point `old_mount_old#1` (the window holds a single operation, flag line `max(28, 4 × 53)`) and `first_mount_first#1` of `files_75_units_100` (two operations in the window, 39 and 11, lower median 11, flag line 44, so 39 nodes are not flagged while the narrow reading `fresh=28 ≥ 18` predicts flagged). The three-way argument verdict (`research/prompts/e203-b8-criterion-r2-main-verification.md`, section two, item 4): the flag line degenerates in a mount window with very few operations, both legs file it under "the discriminator cannot be observed" (the second row of the table "A criterion can also be written wrongly: after a hit, first judge which kind it is" in `.claude/rules/evidence-discipline.md`), the "does not hold" of row one does not rest on it (the 27 failures of the wide readings on the training set are another cause), and the judgement of row one does not change; judging row one again with a flag line that uses only the lower bound 28 gives 0 mismatches over 742 operations (task 16036), which the legs worked out after the run, the registration does not allow changing the criterion afterwards (`research/prompts/e209-preregistration.md:344`, `:366`), it does not count as mechanism evidence, and the user decided on 2026-11-13 to leave it to the optimization milestone (closeout table of `.claude/kb/milestone/04-fourth-txn.md`). In row five the eight operations with enough resolution are all "spread", and another 26 are `not_resolved` because of a large `noise`, whose cause is that the registered comparable wording does not restrict the mount order and the comparable is taken from an operation after the mount; in the same item of the verdict, the defense judged by upper-bound arithmetic that restricting the mount order cannot change row five and the attacker did not hit (one sampling), so the change "restrict the mount order" is not adopted and the judgement of row five does not change.
- The registration (section eight and F10 of section ten) does not fix it: when the spike does not reproduce at a point (Q0 judges "not reproduced"), whether that point's values enter the geometry-sensitivity comparison. The three-way argument verdict (section two, item 4): the attacker files it under "a hit attributed to the wrong criterion" (Q0 is not among the quantities named in section eight of the registration, so "the K direction was never scanned" should be recorded), the defense files it under "a hit that does not tell the arms apart"; the two ways of filing lead to the same handling: the point `files_10_units_1000` is recorded as "the apparatus cannot reach it" (reproduction-gate test `the_reproduction_gate_tells_the_low_line_from_the_main_line`) by void clause 1, and the geometry-sensitivity row states that what is unstable is the one point where K and F go in opposite directions. This page writes both readings side by side in the geometry-sensitivity table of "Readings and row-by-row judgements of the third-segment combined report".
- Sample points stay at small geometry (F ≤ 200); the point where E207 reproduced the spike (F = 1000) is not part of this experiment; at `files_10_units_1000` (F = 10, K = 1000, same total units as the training point) the spike does not reproduce, so the "large K" side has no decidable value.
- The fourth segment (`walk_forward` and `mount_then_first` of `files_250_units_100` and `files_300_units_100`) is not started and no artifact of it has run; it is marked "not run after enough to decide": registration section five (lines 239 and 248) starts it only if some row flips between F = 75, 100 and 200, and between F = 75, 100, 200 and the two U points the judgements of rows one to four agree (row five has no operation with enough resolution at F = 75, which is not a different judgement); the flip appears only at the one point F = 10, K = 1000, which a larger F cannot separate; it corresponds to the geometry sensitivity of all five fork-sheet rows (each already decidable; registration section eleven "stop when enough to decide", lines 376-383).
- Line 88 of the old first-segment combined report has `constants_mismatches=1`, an apparatus defect; the old report is kept unedited; in the re-issued `-r3` the constants cell is `zero`.

### Criteria and failure clauses

See sections six, ten and eleven of the pre-run registration `research/prompts/e209-preregistration.md`; this page copies them clause by clause once the readings of the matching segment exist.

### 2026-11-14 re-run artifacts

The re-run now judges byte-identical; the new artifacts are in the repository and the replay-table rows point to the new names (the E129 device probe has no replay-table row):

- `research/results/e209-spike-cause-report-third-2026-11-14.out` (byte-identical to `e209-spike-cause-report-third-2026-11-13.out`)

### Decisions affected

| Decision item | Relation | Look back |
|---|---|---|
| D8 (core index structure) settled item 16 | preparation | 2026-11-13 not affected: the three-way argument verdict (`research/prompts/e203-b8-criterion-r2-main-verification.md`, section two, item 5): this page stays "preparation" for this item and is not promoted to support; looked back after the third-segment combined report was read: rows one, two, three and five judged "does not hold" and row four judged "holds" at the five points that reproduce, and none of the three registration readings can change the judgement of these five rows; these are readings of the cause of the spike and do not touch this item; its basis does not cite E209 (Cause of the spike) and `.claude/decision-links-pending` currently has no entry |
