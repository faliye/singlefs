## E205 Cost of three fixes for the inverted root after multiple read failures — run (judgeable), **conclusion retired**: the apparatus, the test-only switch and the rerun are all deleted; the artifacts and cost figures are in git history

[中文](205-多处读坏倒挂三条改法的代价.md) · **English** · [日本語](205-複数箇所の読み取り不良による逆転ルートの3つの修正案のコスト.ja.md)

Pre-run registration `research/prompts/e205-preregistration.md` (697 lines, section 12 revisions 1–15); fork sheet `research/prompts/m3-close-b19-c971-forks.md` with five rows: Jia merge-all-zero, Jia merge-read-bad-only, Yi refuse-directly, Yi into-undecidable, Bing. The question is the shape of C971 (a writable mount writes an inverted new-instance root after multiple read failures): before the user decides whether to fix it and which way, a cost figure for every path. The apparatus (7299 lines, source `git show 84b4ae23f:crates/singlefs-verification/src/bin/e205_c971_inverted_first_txg_arm_costs.rs`): Jia writes the image for real through the test-only switch for the first txg of the new instance in `mount.rs`, Yi is computed from the reads at mount time, Bing goes through the product entry `mount_writable`. **This experiment is retired**: user settlement 「退场」 (retire) (verdict `research/prompts/m3-close-b20-code-r2-main-verification.md` section four); the apparatus, the test-only switch, the switch's test target, the two lines of E205 (Cost of three fixes for the inverted root after multiple read failures) in `replay.sh` and the mutation rows pointing at them have all been deleted from the repository, and it is no longer rerun; the artifacts and cost figures stay in git history, and the listed readings are from before the retirement.

**This page records the readings of four artifacts: segment one six cells (rerun under registration revision 15), segment two torn rotation, segment three S = 8 two groups, segment three S = 4 and 16 four groups.** Every quantity of the registration has been run and all five fork-sheet rows are judgeable. In the first artifact of segment one (task 9948) the positive control of the Bing row failed, so by registration section 11 that row's readings were void; the rerun artifact replaces it. The first artifact exists only in the repository history (`git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-six-cells.out`) and is used below only for a line-by-line comparison, its readings are not cited.

### Current state

- All four artifacts were submitted by the main agent through the task service (hand-over note, does not affect the artifacts):
  - Segment one six cells (rerun after registration revision 15): `research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out`, 432 lines, task 10127 (finished on the first attempt, 101.889 seconds), last line `E7RESULT name=done result_lines=431`.
  - Segment two torn rotation: `research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out`, 609 lines, task 9949, last line `E7RESULT name=done result_lines=608`; artifact line see `git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out` line 608 (name=verdict).
  - Segment three S = 8: `research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out`, 60 lines, task 9950, last line `E7RESULT name=done result_lines=59`; artifact line see `git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out` line 59 (name=verdict).
  - Segment three S = 4, 16: `research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out`, 106 lines, task 10016 (the first two attempts hit the memory limit, the third finished in 2465.831 seconds), last line `E7RESULT name=done result_lines=105`.
- The first artifact of segment one (task 9948, `git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-six-cells.out`, 434 lines) compared line by line with the rerun (command and output verbatim):

```text
$ diff <(git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-six-cells.out | sed -n 1,430p) <(sed -n 1,430p research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out) | grep -E '^[0-9]' | tr '\n' ' '
224c224 233c233 246,249c246,249 393c393 406c406 415c415 424c424 
```

  Only 10 of the first 430 lines differ, the other 420 are identical character for character, and neither has non-`E7RESULT` lines. The 10 changed lines are exactly what registration revision 15 changed: lines 224 and 233 are the two G hide-one cells (Jia merge-read-bad-only, root read back all-zero × two kinds of bad system configuration), where only two fields differ, `model_inverted=true` becoming `false` and `fixed_crash_point_model_against_the_written_image=differs` becoming `equal` (the S9 model gained the "the slot is overwritten by root" step); lines 246–249 are the four inverted-image controls, `written_roots=[not_written] publish:ReleaseTargetAlreadyReleased control=CONTROL_FAILED` becoming the really written `written_roots=[(2,5),(2,6)]` and `control=as_expected` (built as revision 15 says); lines 393, 406, 415 and 424 are the four control cells, each gained a `chosen=` field, and the `refusal=none control=CONTROL_FAILED` of line 424 became `refusal=none chosen=1,5 control=as_expected` (the expectation of the G hide-one cell is now "mount succeeds, picks (1,5)"). The two S9 `name=stop_clause` lines (old 431–432) are gone in the rerun, the verdict line's `controls_failed` went from 4 to 0 and `stop_clause_mismatches` from 2 to 0. The Bing six cells, Jia, Yi and control-anchor lines are unchanged character for character, so the line numbers below are the same in both files.
- Every artifact has 0 non-`E7RESULT` lines (the `grep -vc` command under "Verdict Lines and Counts" below).
- Per-cell judgement sentences in the task logs: segment one (log `~/.local/state/singlefs/task-server/logs/task-b0f6b921-10127.attempt-1.log`) "e205：入口 fault_cell_verdict 逐格判定：现判 59、复用 45、拒算 0；现判里判红 0 格（记进红表，下一趟照判）", "e205：入口 control_cell_verdict 逐格判定：现判 44、复用 0、拒算 0；现判里判红 0 格（记进红表，下一趟照判）", "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：现判（键对不上：第一段 148 件切片多线程判一遍、单线程再判一遍）；逐件逐字相同，记绿" (the single-thread differential of segment one), "e205：six-cells 跑完：现判 103、复用 45、拒算 0"; the 45 reused are verdicts with the same key already written to the library by the interrupted previous run (task 10124). Segment two (log `task-b0f6b921-9949.attempt-3.log`) "e205：入口 torn_state_verdict 逐格判定：现判 513、复用 0、拒算 0；现判里判红 0 格", segment three S = 8 (`task-b0f6b921-9950.attempt-3.log`) "e205：入口 skip_state_verdict 逐格判定：现判 33254、复用 0、拒算 0；现判里判红 0 格", segment three S = 4, 16 (`task-b0f6b921-10016.attempt-3.log`) "e205：入口 skip_state_verdict 逐格判定：现判 119104、复用 0、拒算 0；现判里判红 0 格（记进红表，下一趟照判）", "e205：first-txg-skips s4-s16 跑完：现判 119104、复用 0、拒算 0".
- None of the four goes through the crash judging stage entries, so the NOT_RUN count does not apply.

### Verdict Lines and Counts

```text
$ grep -n 'name=verdict' research/results/e205-c971-inverted-first-txg-arm-costs-*.out
research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out:105:E7RESULT name=verdict segment=first_txg_skips groups=s4-s16 constants_equal=true histories=24 states=119104 unmount_rule_mismatches=0 stop_clause_mismatches=0 voided_states=0
research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out:431:E7RESULT name=verdict segment=six_cells constants_equal=true thread_cross_check_identical=true cell_jobs=104 control_jobs=44 mounts_of_the_cells=132 controls_failed=0 six_cell_anchor_failed_arms=[] stop_clause_mismatches=0 voided_states=0
```

The other two artifacts have been archived out of the working tree; their verdict lines are in `git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out` line 59 (name=verdict): `groups=s8 constants_equal=true histories=12 states=33254 unmount_rule_mismatches=0 stop_clause_mismatches=0 voided_states=0`; and `git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out` line 608 (name=verdict): `constants_equal=true states=513 voided_segments=0`.

The four verdict lines read one by one: segment one (rerun) has `controls_failed=0`, `stop_clause_mismatches=0`, `voided_states=0`, `six_cell_anchor_failed_arms=[]`, `constants_equal=true`, `thread_cross_check_identical=true`; no `false` or `not_run` field, and every integer count for violations, mismatches, ambiguities and failures is 0 (the `controls_failed=4` and `stop_clause_mismatches=2` that were not 0 in the first artifact are 0 in the rerun, see the line-by-line comparison). Segment two has `voided_segments=0`. Both segment three artifacts (S = 8 and S = 4, 16) have `unmount_rule_mismatches=0`, `stop_clause_mismatches=0`, `voided_states=0`. Both the segment-one and the segment-three S = 4, 16 artifacts have 0 `CONTROL_FAILED` lines (the `grep -c` is under Bing).

Number of result lines of each kind in each artifact (`awk` takes the `name=` field, then `sort | uniq -c`, output verbatim):

```text
$ awk '{match($0,/name=[a-z_0-9]+/); print substr($0,RSTART,RLENGTH)}' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | sort | uniq -c
    202 name=cell
      1 name=config
      9 name=constant
      1 name=constants_verdict
      4 name=control_cell
      6 name=discrimination
      1 name=done
      4 name=inverted_image_control
     18 name=jia_anchor
      8 name=jia_six_cells_image
      8 name=jia_six_cells_inversion
      4 name=negative_control
      9 name=right_after_make_filesystem
     28 name=s7
     96 name=six_cell_anchor
      4 name=today_d22_clause
      4 name=today_refused_afterwards
      6 name=today_six_cells_reading
      1 name=today_six_cells_reading_other_five
      1 name=verdict
      9 name=yi_hand_made_image
      8 name=yi_six_cells
$ git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out | awk '{match($0,/name=[a-z_0-9]+/); print substr($0,RSTART,RLENGTH)}' | sort | uniq -c
      1 name=config
      9 name=constant
      1 name=constants_verdict
      4 name=discrimination
      1 name=done
     57 name=torn_segment
    513 name=torn_state
      1 name=verdict
      2 name=yi_offline_way_out
     18 name=yi_torn_over_refusals
      2 name=yi_torn_over_refusals_total
$ git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out | awk '{match($0,/name=[a-z_0-9]+/); print substr($0,RSTART,RLENGTH)}' | sort | uniq -c
      1 name=bad_slot_txgs_against_the_allocator
      1 name=config
      9 name=constant
      1 name=constants_verdict
      8 name=discrimination
      1 name=done
     12 name=jia_family_image
     12 name=jia_skip_family
      2 name=s8
     12 name=skip_trajectory
      1 name=verdict
$ awk '{match($0,/name=[a-z_0-9]+/); print substr($0,RSTART,RLENGTH)}' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | sort | uniq -c
      1 name=bad_slot_txgs_against_the_allocator
      1 name=config
      9 name=constant
      1 name=constants_verdict
     16 name=discrimination
      1 name=done
     24 name=jia_family_image
     24 name=jia_skip_family
      4 name=s8
     24 name=skip_trajectory
      1 name=verdict
```

Self-test of discriminative power (registration section 8: moving the threshold to the observed value plus one must flip yes to no, and moving an observed 0 to ≥ 0 must flip no to yes) and the count of non-`E7RESULT` lines:

```text
$ for x in research/results/e205-*.out; do echo $x $(grep -c 'name=discrimination' $x) $(grep 'name=discrimination' $x | grep -c 'flips=true'); done
research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out 16 16
research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out 6 6
$ grep -vc '^E7RESULT' research/results/e205-*.out
research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out:0
research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out:0
```

For the two archived artifacts (segment two torn rotation, segment three S = 8) the same two commands wrapped in `git show 84b4ae23f:<path> |`, output verbatim: torn rotation has 4 self-test lines, 4 with `flips=true`, 0 non-`E7RESULT` lines; S = 8 has 8 self-test lines, 8 with `flips=true`, 0 non-`E7RESULT` lines.

### Results

#### Bing (Today's Implementation, Product Entry)

- The six-cell reading reproduces the replica model completely: at the G main point (S = 8, two roots C and D hidden) the cell with the root reading back all-zero × the system configuration reading back all-zero is compared field by field with the replica line, `reproduced=true differing_fields=[]`; the other five cells are compared on the fields the replica gives, `reproduced_cells=5 of=5`. All six cells are form Yi (`instance_and_txg_order_crossed_on_roots=true`), the pool-level checker is red on 6 invariants (`I-3.1,I-3.21,I-3.5,I-3.9,I-4.2,I-8.14`, the same set as the replica line), the write-row txg is 5 (the anchor of registration section 7, D23 (journal role and format) settled item 14 note 3), `choose_root` picks (1,6) (D; the confirmed C is not lost), and the later fault-free writable mount is refused (`NewerStateStillUnreadableAfterOneReread`). Line 48 is the replica cell, copied whole below:

```text
$ sed -n '48p;382p;388,390p' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
E7RESULT name=cell geometry=main_point root_and_record_fault=all_zero newest_system_configuration_fault=all_zero arm=today mount=ok instance=2 chosen=(1,4) effective=(1,4) row_txg=5 row_txg=5 apparatus_first_txg=5 apparatus_highest_record_txg=4 branch=first_txg=past_the_highest_root_and_record_in_the_ring system_configuration_witness_undecidable=false system_configuration=[all_zero/ok,all_zero/ok] refuse_directly=true into_undecidable=true bad_slot_txgs_against_the_allocator=equal crash_point=fixed persisted_writes=45/69 roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(1,6),(2,5)] records=[(1,1),(1,2),(1,3),(1,4),(2,5),(2,6)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=true instance_and_txg_order_crossed_on_roots_and_records=true inverted=true checker_red=[I-3.1,I-3.21,I-3.5,I-3.9,I-4.2,I-8.14] checker_red_count=6 choose_root=(1,6) content=content_4 confirmed_lost=false mount_after=refused:NewerStateStillUnreadableAfterOneReread mount_after_content=not_mounted mount_after_confirmed_lost=true
E7RESULT name=today_six_cells_reading cell=root_and_record_fault=all_zero newest_system_configuration_fault=all_zero reproduced=true differing_fields=[] model_line=research/prompts/e205-preregistration.md:336
E7RESULT name=today_six_cells_reading_other_five reproduced_cells=5 of=5
E7RESULT name=today_d22_clause geometry=main_point cells_with_checker_red=6 judged_at_main_point=true clause_holds=false
E7RESULT name=today_refused_afterwards geometry=main_point cells_refused_afterwards=6
```

- "Bing, the D22 sentence": `cells_with_checker_red=6 judged_at_main_point=true clause_holds=false` (line 389): the scope sentence of D22 (unit atomicity composition) settled item 8, "before the floor raise the pool-level checker is not red at all on that shape", reads 6 red cells on the checked-in apparatus; handled as the "clause may be wrong" case of registration section 10, not voided, handed to the main agent. "Bing refused afterwards" is 6 cells (line 390). The small and large ring groups give the same 6 / 6 (lines 404, 405, 413, 414).
- **Positive and negative controls both passed, so the Bing row's readings are valid (the void clause of registration sections 10 and 11 does not fire):**
  - The inverted-image control was built in all four geometries (built as registration revision 15 says: take the allocator before C is written in the starting history, instance 2 writes txg 5 and 6, then copy the journal records of the two publishes C and D back from the starting image): `written_roots=[(2,5),(2,6)]`, the pool-level checker is red only on `I-8.14`, `inverted=true`, form Yi judged on roots and records together holds, `control=as_expected`; form Yi judged on roots alone does not occur by construction (`instance_and_txg_order_crossed_on_roots=false`), registration revision 2 already says this control is judged by the merged key.
  - The "system configuration not corrupted" cell has `control=as_expected` in all four geometries: G main point, small ring and large ring refuse with `NewerStateStillUnreadableAfterOneReread` (`chosen=none`); G hide-one mounts successfully and picks (1,5) (`refusal=none chosen=1,5`, the expectation registration revision 15 gives this group: the counter of the selected version C is not below the system configuration witness value, so the product does not refuse).
  - The negative control mounts in all four geometries with 0 red pool-level checker lines; the verdict line has `controls_failed=0` (44 control jobs plus the four control cells); the whole artifact has 0 `CONTROL_FAILED` lines and 144 `control=as_expected` lines.

```text
$ grep -n -E 'name=(negative_control|inverted_image_control|control_cell)' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
242:E7RESULT name=negative_control geometry=main_point mount=ok instance=2 chosen=(1,6) effective=(1,6) row_txg=7 checker_red=[] control=as_expected
243:E7RESULT name=negative_control geometry=small_ring mount=ok instance=2 chosen=(1,6) effective=(1,6) row_txg=7 checker_red=[] control=as_expected
244:E7RESULT name=negative_control geometry=large_ring mount=ok instance=2 chosen=(1,6) effective=(1,6) row_txg=7 checker_red=[] control=as_expected
245:E7RESULT name=negative_control geometry=hide_one mount=ok instance=2 chosen=(1,6) effective=(1,6) row_txg=7 checker_red=[] control=as_expected
246:E7RESULT name=inverted_image_control geometry=main_point written_roots=[(2,5),(2,6)] roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(2,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(1,6)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=true inverted=true checker_red=[I-8.14] checker_red_count=1 choose_root=(2,6) content=content_6 confirmed_lost=false control=as_expected
247:E7RESULT name=inverted_image_control geometry=small_ring written_roots=[(2,5),(2,6)] roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(2,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(1,6)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=true inverted=true checker_red=[I-8.14] checker_red_count=1 choose_root=(2,6) content=content_6 confirmed_lost=false control=as_expected
248:E7RESULT name=inverted_image_control geometry=large_ring written_roots=[(2,5),(2,6)] roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(2,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(1,6)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=true inverted=true checker_red=[I-8.14] checker_red_count=1 choose_root=(2,6) content=content_6 confirmed_lost=false control=as_expected
249:E7RESULT name=inverted_image_control geometry=hide_one written_roots=[(2,5),(2,6)] roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(2,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(1,6)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=true inverted=true checker_red=[I-8.14] checker_red_count=1 choose_root=(2,6) content=content_6 confirmed_lost=false control=as_expected
393:E7RESULT name=control_cell geometry=main_point root_and_record_fault=device_error newest_system_configuration_fault=none refusal=NewerStateStillUnreadableAfterOneReread chosen=none control=as_expected
406:E7RESULT name=control_cell geometry=small_ring root_and_record_fault=device_error newest_system_configuration_fault=none refusal=NewerStateStillUnreadableAfterOneReread chosen=none control=as_expected
415:E7RESULT name=control_cell geometry=large_ring root_and_record_fault=device_error newest_system_configuration_fault=none refusal=NewerStateStillUnreadableAfterOneReread chosen=none control=as_expected
424:E7RESULT name=control_cell geometry=hide_one root_and_record_fault=device_error newest_system_configuration_fault=none refusal=none chosen=1,5 control=as_expected
$ grep -c CONTROL_FAILED research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
0
$ grep -c 'control=as_expected' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
144
```

- One more: at G hide-one (only D hidden) none of the six Bing cells is inverted (`inverted=false`; the command under "Stops and Checks" counts 6 cells), the pool-level checker is red on 0 and the later writable mount succeeds; the registration section 8 expectation "the inversion moves from form Yi to form Jia" did not appear: the new instance's first txg 6 equals D's txg, so the root and record it writes land on D's own slot and overwrite D (the `roots=` of artifact line 183 has no (1,6)).

#### Jia Merge-All-Zero and Jia Merge-Read-Bad-Only

- Six cells (segment one, real writes through the switch, fixed crash point):

```text
$ grep -n -E 'name=(jia_six_cells_inversion|jia_six_cells_image|yi_six_cells)' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
394:E7RESULT name=jia_six_cells_inversion arm=past_read_bad_or_all_zero geometry=main_point cells_still_inverted=0 cells=[] covers_the_six_cells=true
395:E7RESULT name=jia_six_cells_image arm=past_read_bad_or_all_zero geometry=main_point cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=6 cells_with_checker_red=0 cells_refused_afterwards=0
398:E7RESULT name=jia_six_cells_inversion arm=past_read_bad geometry=main_point cells_still_inverted=2 cells=[root_and_record_fault=all_zero/newest_system_configuration_fault=all_zero,root_and_record_fault=all_zero/newest_system_configuration_fault=not_self_verified] covers_the_six_cells=false
399:E7RESULT name=jia_six_cells_image arm=past_read_bad geometry=main_point cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=4 cells_with_checker_red=2 cells_refused_afterwards=2
402:E7RESULT name=yi_six_cells yi=refuse_directly geometry=main_point cells_refused=6 of=6 covers_the_six_cells=true
403:E7RESULT name=yi_six_cells yi=into_undecidable geometry=main_point cells_refused=6 of=6 covers_the_six_cells=true
407:E7RESULT name=jia_six_cells_inversion arm=past_read_bad_or_all_zero geometry=small_ring cells_still_inverted=0 cells=[] covers_the_six_cells=true
408:E7RESULT name=jia_six_cells_image arm=past_read_bad_or_all_zero geometry=small_ring cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=6 cells_with_checker_red=0 cells_refused_afterwards=0
409:E7RESULT name=jia_six_cells_inversion arm=past_read_bad geometry=small_ring cells_still_inverted=2 cells=[root_and_record_fault=all_zero/newest_system_configuration_fault=all_zero,root_and_record_fault=all_zero/newest_system_configuration_fault=not_self_verified] covers_the_six_cells=false
410:E7RESULT name=jia_six_cells_image arm=past_read_bad geometry=small_ring cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=4 cells_with_checker_red=2 cells_refused_afterwards=2
411:E7RESULT name=yi_six_cells yi=refuse_directly geometry=small_ring cells_refused=6 of=6 covers_the_six_cells=true
412:E7RESULT name=yi_six_cells yi=into_undecidable geometry=small_ring cells_refused=6 of=6 covers_the_six_cells=true
416:E7RESULT name=jia_six_cells_inversion arm=past_read_bad_or_all_zero geometry=large_ring cells_still_inverted=0 cells=[] covers_the_six_cells=true
417:E7RESULT name=jia_six_cells_image arm=past_read_bad_or_all_zero geometry=large_ring cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=6 cells_with_checker_red=0 cells_refused_afterwards=0
418:E7RESULT name=jia_six_cells_inversion arm=past_read_bad geometry=large_ring cells_still_inverted=2 cells=[root_and_record_fault=all_zero/newest_system_configuration_fault=all_zero,root_and_record_fault=all_zero/newest_system_configuration_fault=not_self_verified] covers_the_six_cells=false
419:E7RESULT name=jia_six_cells_image arm=past_read_bad geometry=large_ring cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=4 cells_with_checker_red=2 cells_refused_afterwards=2
420:E7RESULT name=yi_six_cells yi=refuse_directly geometry=large_ring cells_refused=6 of=6 covers_the_six_cells=true
421:E7RESULT name=yi_six_cells yi=into_undecidable geometry=large_ring cells_refused=6 of=6 covers_the_six_cells=true
425:E7RESULT name=jia_six_cells_inversion arm=past_read_bad_or_all_zero geometry=hide_one cells_still_inverted=0 cells=[] covers_the_six_cells=true
426:E7RESULT name=jia_six_cells_image arm=past_read_bad_or_all_zero geometry=hide_one cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=0 cells_with_checker_red=0 cells_refused_afterwards=0
427:E7RESULT name=jia_six_cells_inversion arm=past_read_bad geometry=hide_one cells_still_inverted=0 cells=[] covers_the_six_cells=true
428:E7RESULT name=jia_six_cells_image arm=past_read_bad geometry=hide_one cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=0 cells_with_checker_red=0 cells_refused_afterwards=0
429:E7RESULT name=yi_six_cells yi=refuse_directly geometry=hide_one cells_refused=6 of=6 covers_the_six_cells=true
430:E7RESULT name=yi_six_cells yi=into_undecidable geometry=hide_one cells_refused=6 of=6 covers_the_six_cells=true
```

  - Jia merge-all-zero: every geometry has `cells_still_inverted=0` (all inversions gone), the pool-level checker is red on 0, and 0 cells have the later same-mode writable mount refused; the price is that all 6 cells lose the confirmed C (`cells_losing_the_last_confirmed_overwrite_where_today_keeps_it=6`, Bing does not lose it in the same cells). The write-row txg is 3S (24 at S = 8, 12 for the small ring, 48 for the large ring, also 24 for hide-one; the `six_cell_anchor` lines agree with the artifact, see "Stops and Checks" below).
  - Jia merge-read-bad-only: the four cells with the root and record read badly (device read error, readable but self-verification fails) lose the inversion (write row 7); the two cells with the root reading back all-zero keep the inversion (`cells_still_inverted=2`, write row 5, same as Bing), the pool-level checker is red on those two and the later writable mount is refused (`cells_with_checker_red=2 cells_refused_afterwards=2`); 4 cells lose the confirmed C (Bing does not in the same cells). The small and large rings are the same. At hide-one Bing itself is not inverted and both modes give 0.
  - Yi in the six cells: both readings refuse 6 / 6 in all four geometries (`yi_six_cells`, above).
  - Jia, the half comparison "Bing does not lose C in the same cells": Bing's positive control passed and its readings are valid (see Bing); over four geometries × six cells, 24 cells in all, Bing has `confirmed_lost=false`, none loses the confirmed C, while Jia merge-all-zero loses it in 6 / 6 / 6 cells of the three geometries that hide two roots and Jia merge-read-bad-only in 4 / 4 / 4, so the "Jia buys losing a confirmed write" half of the comparison now has a valid Bing base. The count of Bing's 24 cells that do not lose C:

```text
$ grep 'name=cell ' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep 'arm=today ' | grep 'crash_point=fixed' | grep -v 'newest_system_configuration_fault=none' | wc -l
24
$ grep 'name=cell ' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep 'arm=today ' | grep 'crash_point=fixed' | grep -v 'newest_system_configuration_fault=none' | grep -c 'confirmed_lost=false'
24
```

- First writable mount after mkfs (control line `right_after_make_filesystem`, three S × three modes, merged into segment one by registration revision 6):

```text
$ grep -n 'name=right_after_make_filesystem' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
268:E7RESULT name=right_after_make_filesystem slots_per_region=4 arm=today_via_switch expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
269:E7RESULT name=right_after_make_filesystem slots_per_region=4 arm=past_read_bad expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
270:E7RESULT name=right_after_make_filesystem slots_per_region=4 arm=past_read_bad_or_all_zero expected_row_txg=12 row_txg=12 apparatus_first_txg=12 same_bytes_as_today=false genesis_slots_ok=true control=as_expected
271:E7RESULT name=right_after_make_filesystem slots_per_region=8 arm=today_via_switch expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
272:E7RESULT name=right_after_make_filesystem slots_per_region=8 arm=past_read_bad expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
273:E7RESULT name=right_after_make_filesystem slots_per_region=8 arm=past_read_bad_or_all_zero expected_row_txg=24 row_txg=24 apparatus_first_txg=24 same_bytes_as_today=false genesis_slots_ok=true control=as_expected
274:E7RESULT name=right_after_make_filesystem slots_per_region=16 arm=today_via_switch expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
275:E7RESULT name=right_after_make_filesystem slots_per_region=16 arm=past_read_bad expected_row_txg=1 row_txg=1 apparatus_first_txg=1 same_bytes_as_today=true genesis_slots_ok=true control=as_expected
276:E7RESULT name=right_after_make_filesystem slots_per_region=16 arm=past_read_bad_or_all_zero expected_row_txg=48 row_txg=48 apparatus_first_txg=48 same_bytes_as_today=false genesis_slots_ok=true control=as_expected
```

  Jia merge-all-zero has write-row txg 12 / 24 / 48 ≠ 1 and an image that differs from today's mode (`same_bytes_as_today=false`), recorded as "changes the bytes of new-pool file creation"; Jia merge-read-bad-only has write-row txg 1 and a byte-identical image.

- Segment three S = 8 (two groups in opposite directions: p = 2 fills the ring slowly, p = 24 fills a whole ring in one mount; "today's value" is the Bing mode):

```text
$ grep -n -E 'name=(s8|jia_skip_family|bad_slot_txgs_against_the_allocator)' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out
24:E7RESULT name=s8 slots_per_region=8 overwrites=2 arm=past_read_bad history=no_fault same_as_today_at_every_mount_boundary=true
25:E7RESULT name=s8 slots_per_region=8 overwrites=24 arm=past_read_bad history=no_fault same_as_today_at_every_mount_boundary=true
26:E7RESULT name=jia_skip_family arm=today family=no_fault slots_per_region=8 overwrites=2 states=115 mounted_states=115 refusals=[] skip_maximum=0 skip_mean=0/115 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
28:E7RESULT name=jia_skip_family arm=today family=single_fault slots_per_region=8 overwrites=2 states=1944 mounted_states=1932 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=0 skip_mean=0/1932 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
30:E7RESULT name=jia_skip_family arm=past_read_bad family=no_fault slots_per_region=8 overwrites=2 states=115 mounted_states=115 refusals=[] skip_maximum=0 skip_mean=0/115 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
33:E7RESULT name=jia_skip_family arm=past_read_bad family=single_fault slots_per_region=8 overwrites=2 states=1944 mounted_states=1932 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=23 skip_mean=14904/1932 skip_positive_states=1242 skip_at_ring_length_less_one=54 changes_behaviour_without_a_fault=true more_refusals_than_today=false
36:E7RESULT name=jia_skip_family arm=past_read_bad_or_all_zero family=no_fault slots_per_region=8 overwrites=2 states=107 mounted_states=107 refusals=[] skip_maximum=23 skip_mean=2092/107 skip_positive_states=103 skip_at_ring_length_less_one=14 changes_behaviour_without_a_fault=true more_refusals_than_today=false
39:E7RESULT name=jia_skip_family arm=past_read_bad_or_all_zero family=single_fault slots_per_region=8 overwrites=2 states=1800 mounted_states=1788 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=23 skip_mean=37224/1788 skip_positive_states=1788 skip_at_ring_length_less_one=408 changes_behaviour_without_a_fault=true more_refusals_than_today=false
42:E7RESULT name=jia_skip_family arm=today family=no_fault slots_per_region=8 overwrites=24 states=483 mounted_states=483 refusals=[] skip_maximum=0 skip_mean=0/483 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
44:E7RESULT name=jia_skip_family arm=today family=single_fault slots_per_region=8 overwrites=24 states=8568 mounted_states=8556 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=0 skip_mean=0/8556 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
46:E7RESULT name=jia_skip_family arm=past_read_bad family=no_fault slots_per_region=8 overwrites=24 states=483 mounted_states=483 refusals=[] skip_maximum=0 skip_mean=0/483 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
49:E7RESULT name=jia_skip_family arm=past_read_bad family=single_fault slots_per_region=8 overwrites=24 states=8568 mounted_states=8556 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=23 skip_mean=65688/8556 skip_positive_states=5474 skip_at_ring_length_less_one=238 changes_behaviour_without_a_fault=true more_refusals_than_today=false
52:E7RESULT name=jia_skip_family arm=past_read_bad_or_all_zero family=no_fault slots_per_region=8 overwrites=24 states=487 mounted_states=487 refusals=[] skip_maximum=23 skip_mean=1127/487 skip_positive_states=93 skip_at_ring_length_less_one=5 changes_behaviour_without_a_fault=true more_refusals_than_today=false
55:E7RESULT name=jia_skip_family arm=past_read_bad_or_all_zero family=single_fault slots_per_region=8 overwrites=24 states=8640 mounted_states=8628 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=23 skip_mean=107019/8628 skip_positive_states=8349 skip_at_ring_length_less_one=495 changes_behaviour_without_a_fault=true more_refusals_than_today=false
58:E7RESULT name=bad_slot_txgs_against_the_allocator segment=first_txg_skips groups=s8 states=[equal:33182,not_mounted:72]
```

  - No-fault family: Jia merge-read-bad-only has skip 0 at both p = 2 and p = 24, `same_as_today_at_every_mount_boundary=true` (stop S8 passes); Jia merge-all-zero at p = 2 has skip > 0 in 103 of 107 states, maximum 23 (= 3S − 1), mean 2092/107 (about 19.55); at p = 24, 93 of 487 states are > 0, maximum 23, mean 1127/487 (about 2.31). Jia merge-all-zero changes behaviour even without a fault (`changes_behaviour_without_a_fault=true`).
  - Single-fault family (one root ring slot × three kinds of bad read): Jia merge-read-bad-only at p = 2 has 1242 of 1944 states (1932 mounted) > 0, maximum 23, mean 14904/1932 (about 7.71); at p = 24, 5474 of 8568 states (8556 mounted) > 0, mean 65688/8556 (about 7.68). Jia merge-all-zero at p = 2 has all 1788 of 1800 states (1788 mounted) > 0, mean 37224/1788 (about 20.82); at p = 24, 8349 of 8640 states (8628 mounted) > 0, mean 107019/8628 (about 12.40). Both modes have states with 23 = 3S − 1 (the first publish lands on the slot of the newest readable root and overwrites the root of the current version): Jia merge-read-bad-only 54 / 238 states, Jia merge-all-zero 408 / 495 states.
  - The number of refused states equals Bing's: in each single-fault group 12 states are refused with `NewerStateStillUnreadableAfterOneReread`, `more_refusals_than_today=false`.
  - Image consequences of both families: the 12 `jia_family_image` lines of the three modes all have red states 0 and lost confirmed writes 0 (the `grep -vc` below counts 0); the six-cell price of Jia losing C does not appear in the family histories.

- Skip trajectory (8 mounts, real writes, model compared item by item 12 / 12 `model_column=equal`):

```text
$ grep -n 'name=skip_trajectory' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out
12:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=today history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
13:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=today history=persistent_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
14:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=past_read_bad_or_all_zero history=no_fault skips=[23,18,18,18,18,18,18,18] peak=23 positive_mounts=8 final=18 mean=149/8 model=[23,18,18,18,18,18,18,18] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=true
15:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=past_read_bad_or_all_zero history=persistent_fault skips=[23,22,17,17,17,17,17,17] peak=23 positive_mounts=8 final=17 mean=147/8 model=[23,22,17,17,17,17,17,17] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=true
16:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
17:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=2 arm=past_read_bad history=persistent_fault skips=[3,17,17,17,17,17,17,17] peak=17 positive_mounts=8 final=17 mean=122/8 model=[3,17,17,17,17,17,17,17] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=true
18:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=today history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
19:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=today history=persistent_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
20:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=past_read_bad_or_all_zero history=no_fault skips=[23,0,0,0,0,0,0,0] peak=23 positive_mounts=1 final=0 mean=23/8 model=[23,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
21:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=past_read_bad_or_all_zero history=persistent_fault skips=[23,23,20,20,20,20,20,20] peak=23 positive_mounts=8 final=20 mean=166/8 model=[23,23,20,20,20,20,20,20] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=true
22:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=false
23:E7RESULT name=skip_trajectory slots_per_region=8 overwrites=24 arm=past_read_bad history=persistent_fault skips=[3,20,20,20,20,20,20,20] peak=20 positive_mounts=8 final=20 mean=143/8 model=[3,20,20,20,20,20,20,20] model_column=equal cycles=[ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised,ok|floor_raised] every_mount_skips=true
```

  Jia merge-all-zero without a fault skips at every mount at p = 2 (`[23,18,18,18,18,18,18,18]`, `positive_mounts=8`) and only at the first mount at p = 24 (`[23,0,0,…]`, `positive_mounts=1`); on a history with one root ring slot persistently bad both modes skip at every mount (Jia merge-read-bad-only `[3,17,17,…]` / `[3,20,20,…]`, Jia merge-all-zero `[23,22,17,…]` / `[23,23,20,…]`). Bing is all 0.

- Segment three S = 4, 16 (four groups: S = 4 at p = 2 and p = 12, S = 16 at p = 2 and p = 48; p = 3S fills a whole ring in one mount). Read the same way as S = 8; Bing's mode (`arm=today`) has the same-shaped lines for each group, all with skip 0. Below, the family lines of the two Jia modes (`arm=past_read_bad` is Jia merge-read-bad-only, `arm=past_read_bad_or_all_zero` is Jia merge-all-zero), with the constant `name`, `states` and `mounted_states` fields removed, line numbers are the artifact's:

```text
$ grep -n 'name=jia_skip_family' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | grep -v 'arm=today' | sed -E 's/E7RESULT name=jia_skip_family //; s/ states=[0-9]+ mounted_states=[0-9]+//'
44:arm=past_read_bad family=no_fault slots_per_region=4 overwrites=2 refusals=[] skip_maximum=0 skip_mean=0/115 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
47:arm=past_read_bad family=single_fault slots_per_region=4 overwrites=2 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=11 skip_mean=3564/960 skip_positive_states=594 skip_at_ring_length_less_one=54 changes_behaviour_without_a_fault=true more_refusals_than_today=false
50:arm=past_read_bad_or_all_zero family=no_fault slots_per_region=4 overwrites=2 refusals=[] skip_maximum=11 skip_mean=856/107 skip_positive_states=103 skip_at_ring_length_less_one=14 changes_behaviour_without_a_fault=true more_refusals_than_today=false
53:arm=past_read_bad_or_all_zero family=single_fault slots_per_region=4 overwrites=2 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=11 skip_mean=8028/888 skip_positive_states=888 skip_at_ring_length_less_one=228 changes_behaviour_without_a_fault=true more_refusals_than_today=false
60:arm=past_read_bad family=no_fault slots_per_region=4 overwrites=12 refusals=[] skip_maximum=0 skip_mean=0/291 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
63:arm=past_read_bad family=single_fault slots_per_region=4 overwrites=12 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=11 skip_mean=9372/2544 skip_positive_states=1562 skip_at_ring_length_less_one=142 changes_behaviour_without_a_fault=true more_refusals_than_today=false
66:arm=past_read_bad_or_all_zero family=no_fault slots_per_region=4 overwrites=12 refusals=[] skip_maximum=11 skip_mean=275/295 skip_positive_states=45 skip_at_ring_length_less_one=5 changes_behaviour_without_a_fault=true more_refusals_than_today=false
69:arm=past_read_bad_or_all_zero family=single_fault slots_per_region=4 overwrites=12 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=11 skip_mean=15279/2580 skip_positive_states=2409 skip_at_ring_length_less_one=279 changes_behaviour_without_a_fault=true more_refusals_than_today=false
76:arm=past_read_bad family=no_fault slots_per_region=16 overwrites=2 refusals=[] skip_maximum=0 skip_mean=0/115 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
79:arm=past_read_bad family=single_fault slots_per_region=16 overwrites=2 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=47 skip_mean=60912/3876 skip_positive_states=2538 skip_at_ring_length_less_one=54 changes_behaviour_without_a_fault=true more_refusals_than_today=false
82:arm=past_read_bad_or_all_zero family=no_fault slots_per_region=16 overwrites=2 refusals=[] skip_maximum=47 skip_mean=4564/107 skip_positive_states=103 skip_at_ring_length_less_one=14 changes_behaviour_without_a_fault=true more_refusals_than_today=false
85:arm=past_read_bad_or_all_zero family=single_fault slots_per_region=16 overwrites=2 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=47 skip_mean=160416/3588 skip_positive_states=3588 skip_at_ring_length_less_one=768 changes_behaviour_without_a_fault=true more_refusals_than_today=false
92:arm=past_read_bad family=no_fault slots_per_region=16 overwrites=48 refusals=[] skip_maximum=0 skip_mean=0/867 skip_positive_states=0 skip_at_ring_length_less_one=0 changes_behaviour_without_a_fault=false more_refusals_than_today=false
95:arm=past_read_bad family=single_fault slots_per_region=16 overwrites=48 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=47 skip_mean=485040/30948 skip_positive_states=20210 skip_at_ring_length_less_one=430 changes_behaviour_without_a_fault=true more_refusals_than_today=false
98:arm=past_read_bad_or_all_zero family=no_fault slots_per_region=16 overwrites=48 refusals=[] skip_maximum=47 skip_mean=4559/871 skip_positive_states=189 skip_at_ring_length_less_one=5 changes_behaviour_without_a_fault=true more_refusals_than_today=false
101:arm=past_read_bad_or_all_zero family=single_fault slots_per_region=16 overwrites=48 refusals=[NewerStateStillUnreadableAfterOneReread:12] skip_maximum=47 skip_mean=789459/31092 skip_positive_states=30597 skip_at_ring_length_less_one=927 changes_behaviour_without_a_fault=true more_refusals_than_today=false
```

  - No-fault family: Jia merge-read-bad-only has skip 0 in all four groups (`skip_positive_states=0`), `same_as_today_at_every_mount_boundary=true` (four `name=s8` lines, lines 36–39, stop S8 passes); Jia merge-all-zero at S = 4, p = 2 has 103 of 107 states with skip > 0, maximum 11 (= 3S − 1), mean 856/107 (about 8.00), at p = 12 45 of 295 states > 0, maximum 11, mean 275/295 (about 0.93); at S = 16, p = 2 103 of 107 states > 0, maximum 47, mean 4564/107 (about 42.65), at p = 48 189 of 871 states > 0, maximum 47, mean 4559/871 (about 5.23). All four groups have `changes_behaviour_without_a_fault=true`.
  - Single-fault family: Jia merge-read-bad-only at S = 4, p = 2 has 594 of 960 mounted states > 0, mean 3564/960 (about 3.71), at p = 12 1562 of 2544 states > 0, mean 9372/2544 (about 3.68); at S = 16, p = 2 2538 of 3876 states > 0, mean 60912/3876 (about 15.72), at p = 48 20210 of 30948 states > 0, mean 485040/30948 (about 15.67). Jia merge-all-zero at S = 4, p = 2 has all 888 of 888 states > 0, mean 8028/888 (about 9.04), at p = 12 2409 of 2580 states > 0, mean 15279/2580 (about 5.92); at S = 16, p = 2 all 3588 of 3588 states > 0, mean 160416/3588 (about 44.71), at p = 48 30597 of 31092 states > 0, mean 789459/31092 (about 25.39). The maximum is always 3S − 1 (11 and 47); states equal to 3S − 1: Jia merge-read-bad-only 54 / 142 / 54 / 430, Jia merge-all-zero 228 / 279 / 768 / 927 (in the order S = 4 at p = 2 and p = 12, S = 16 at p = 2 and p = 48).
  - The number of refused states equals Bing's: in each single-fault group 12 states are refused with `NewerStateStillUnreadableAfterOneReread`, `more_refusals_than_today=false` (the 12 lines above).
  - Image consequences of both families: the 24 `jia_family_image` lines (three modes × two families × four groups) have 0 red states and 0 lost confirmed writes:

```text
$ grep 'name=jia_family_image' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | grep -vc 'checker_red_states=0 checker_red=\[\] confirmed_lost_states=0'
0
$ grep -n 'name=bad_slot_txgs_against_the_allocator' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out
104:E7RESULT name=bad_slot_txgs_against_the_allocator segment=first_txg_skips groups=s4-s16 states=[equal:118960,not_mounted:144]
```

  - Skip trajectory (8 mounts, real writes, model compared item by item 24 / 24 `model_column=equal`; the 16 lines of the two Jia modes, with the per-mount `cycles=` field removed):

```text
$ grep -n 'name=skip_trajectory' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | grep -v 'arm=today' | sed -E 's/E7RESULT name=skip_trajectory //; s/ cycles=.*//'
14:slots_per_region=4 overwrites=2 arm=past_read_bad_or_all_zero history=no_fault skips=[11,6,6,6,6,6,6,6] peak=11 positive_mounts=8 final=6 mean=53/8 model=[11,6,6,6,6,6,6,6] model_column=equal
15:slots_per_region=4 overwrites=2 arm=past_read_bad_or_all_zero history=persistent_fault skips=[11,10,5,5,5,5,5,5] peak=11 positive_mounts=8 final=5 mean=51/8 model=[11,10,5,5,5,5,5,5] model_column=equal
16:slots_per_region=4 overwrites=2 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal
17:slots_per_region=4 overwrites=2 arm=past_read_bad history=persistent_fault skips=[3,5,5,5,5,5,5,5] peak=5 positive_mounts=8 final=5 mean=38/8 model=[3,5,5,5,5,5,5,5] model_column=equal
20:slots_per_region=4 overwrites=12 arm=past_read_bad_or_all_zero history=no_fault skips=[11,0,0,0,0,0,0,0] peak=11 positive_mounts=1 final=0 mean=11/8 model=[11,0,0,0,0,0,0,0] model_column=equal
21:slots_per_region=4 overwrites=12 arm=past_read_bad_or_all_zero history=persistent_fault skips=[11,11,8,8,8,8,8,8] peak=11 positive_mounts=8 final=8 mean=70/8 model=[11,11,8,8,8,8,8,8] model_column=equal
22:slots_per_region=4 overwrites=12 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal
23:slots_per_region=4 overwrites=12 arm=past_read_bad history=persistent_fault skips=[3,8,8,8,8,8,8,8] peak=8 positive_mounts=8 final=8 mean=59/8 model=[3,8,8,8,8,8,8,8] model_column=equal
26:slots_per_region=16 overwrites=2 arm=past_read_bad_or_all_zero history=no_fault skips=[47,42,42,42,42,42,42,42] peak=47 positive_mounts=8 final=42 mean=341/8 model=[47,42,42,42,42,42,42,42] model_column=equal
27:slots_per_region=16 overwrites=2 arm=past_read_bad_or_all_zero history=persistent_fault skips=[47,46,41,41,41,41,41,41] peak=47 positive_mounts=8 final=41 mean=339/8 model=[47,46,41,41,41,41,41,41] model_column=equal
28:slots_per_region=16 overwrites=2 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal
29:slots_per_region=16 overwrites=2 arm=past_read_bad history=persistent_fault skips=[3,41,41,41,41,41,41,41] peak=41 positive_mounts=8 final=41 mean=290/8 model=[3,41,41,41,41,41,41,41] model_column=equal
32:slots_per_region=16 overwrites=48 arm=past_read_bad_or_all_zero history=no_fault skips=[47,0,0,0,0,0,0,0] peak=47 positive_mounts=1 final=0 mean=47/8 model=[47,0,0,0,0,0,0,0] model_column=equal
33:slots_per_region=16 overwrites=48 arm=past_read_bad_or_all_zero history=persistent_fault skips=[47,47,44,44,44,44,44,44] peak=47 positive_mounts=8 final=44 mean=358/8 model=[47,47,44,44,44,44,44,44] model_column=equal
34:slots_per_region=16 overwrites=48 arm=past_read_bad history=no_fault skips=[0,0,0,0,0,0,0,0] peak=0 positive_mounts=0 final=0 mean=0/8 model=[0,0,0,0,0,0,0,0] model_column=equal
35:slots_per_region=16 overwrites=48 arm=past_read_bad history=persistent_fault skips=[3,44,44,44,44,44,44,44] peak=44 positive_mounts=8 final=44 mean=311/8 model=[3,44,44,44,44,44,44,44] model_column=equal
$ grep 'name=skip_trajectory' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | grep -c 'model_column=equal'
24
```

  The direction at S = 4 and 16 matches S = 8: Jia merge-all-zero without a fault skips at every mount at p = 2 (`positive_mounts=8`) and only at the first mount at p = 12 and p = 48 (`positive_mounts=1`); on a history with one root ring slot persistently bad both modes skip at every mount (`positive_mounts=8`); Jia merge-read-bad-only without a fault 0. Bing is all 0.
  - Geometry sensitivity (registration section 8: each group of the two Jia families is noted flip or not): across the six groups S = 4, 8, 16 × p ∈ {2, 3S} the direction of no threshold changes in any group. Behaviour change without a fault: Jia merge-all-zero has ≥ 1 state in all six groups (S = 8 at p = 2, p = 24 gives 103, 93 states, S = 4 at p = 2, p = 12 gives 103, 45, S = 16 at p = 2, p = 48 gives 103, 189, all the `skip_positive_states` in the lines above and earlier), Jia merge-read-bad-only has 0 in all six; states with skip equal to 3S − 1 without a fault: Jia merge-all-zero ≥ 1 in all six (14 at p = 2, 5 at p = 3S); states with skip > 0 and equal to 3S − 1 in the single-fault family: ≥ 1 for both modes in all six; refusal causes the switch mount adds: none in any of the six; image consequences: 36 lines (the 12 of S = 8 plus the 24 above) have 0 red states and 0 lost-confirmed states; whether the number of positive mounts is ≥ 2: the same p gives the same sign across the three S (no-fault Jia merge-all-zero 8 at p = 2, 1 at p = 3S, both modes 8 on the persistently bad history). The six groups are noted "same direction, no flip".

- The state counts of segment three against registration section 4, to be read together with "State Count Check" below:

```text
$ grep 'name=jia_skip_family' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out | sed -E 's/.*arm=([a-z_]+) family=([a-z_]+) slots_per_region=([0-9]+) overwrites=([0-9]+) states=([0-9]+).*/\1 \2 S\3 p\4 states=\5/'
today no_fault S8 p2 states=115
today single_fault S8 p2 states=1944
past_read_bad no_fault S8 p2 states=115
past_read_bad single_fault S8 p2 states=1944
past_read_bad_or_all_zero no_fault S8 p2 states=107
past_read_bad_or_all_zero single_fault S8 p2 states=1800
today no_fault S8 p24 states=483
today single_fault S8 p24 states=8568
past_read_bad no_fault S8 p24 states=483
past_read_bad single_fault S8 p24 states=8568
past_read_bad_or_all_zero no_fault S8 p24 states=487
past_read_bad_or_all_zero single_fault S8 p24 states=8640
$ grep 'name=jia_family_image' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out | grep -vc 'checker_red_states=0 checker_red=\[\] confirmed_lost_states=0'
0
```

#### Yi Refuse-Directly and Yi Into-Undecidable

- Six cells: both readings refuse 6 / 6 in all four geometries (the eight `yi_six_cells` lines above). The nine judgements on the three hand-made images all match the pinned answers (`yi_hand_made_image` 9 / 9 `as_expected`, command under "Stops and Checks" below).
- Torn-rotation family (segment two): three S (4, 8, 16) × 19 system configuration write segments × 9 states per segment = 513 states, each segment `states=9 expected_states=9` (the void clause does not fire, `voided_segments=0`); Bing mounts writably in all 513 states (`today_refused_states=0` on every line):

```text
$ sed -n '582,608p' research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=4 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=65 conflicting_with_c596=65
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=4 kind=acquisition states=36 today_refused_states=0 over_refused_states=20 conflicting_with_c596=20
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=4 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=10 conflicting_with_c596=10
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=8 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=65 conflicting_with_c596=65
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=8 kind=acquisition states=36 today_refused_states=0 over_refused_states=20 conflicting_with_c596=20
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=8 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=10 conflicting_with_c596=10
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=16 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=65 conflicting_with_c596=65
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=16 kind=acquisition states=36 today_refused_states=0 over_refused_states=20 conflicting_with_c596=20
E7RESULT name=yi_torn_over_refusals yi=refuse_directly slots_per_region=16 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=10 conflicting_with_c596=10
E7RESULT name=yi_torn_over_refusals_total yi=refuse_directly over_refused_states=285 over_refuses_within_the_promise=true conflicting_with_c596=285 conflicts_with_settled_clauses=true
E7RESULT name=discrimination quantity=yi_torn_over_refusals.refuse_directly observed=285 threshold=1 judged=true moved_threshold=286 judged_at_moved=false flips=true
E7RESULT name=discrimination quantity=yi_torn_c596_conflicts.refuse_directly observed=285 threshold=1 judged=true moved_threshold=286 judged_at_moved=false flips=true
E7RESULT name=yi_offline_way_out yi=refuse_directly over_refused_states=285 walked_out_states=255 outcomes=[refused:TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion:30,rewritten:255] attached_quantity=true
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=4 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=13 conflicting_with_c596=13
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=4 kind=acquisition states=36 today_refused_states=0 over_refused_states=4 conflicting_with_c596=4
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=4 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=2 conflicting_with_c596=2
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=8 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=13 conflicting_with_c596=13
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=8 kind=acquisition states=36 today_refused_states=0 over_refused_states=4 conflicting_with_c596=4
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=8 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=2 conflicting_with_c596=2
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=16 kind=publish_end_rotation states=117 today_refused_states=0 over_refused_states=13 conflicting_with_c596=13
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=16 kind=acquisition states=36 today_refused_states=0 over_refused_states=4 conflicting_with_c596=4
E7RESULT name=yi_torn_over_refusals yi=into_undecidable slots_per_region=16 kind=floor_raise_pre_write states=18 today_refused_states=0 over_refused_states=2 conflicting_with_c596=2
E7RESULT name=yi_torn_over_refusals_total yi=into_undecidable over_refused_states=57 over_refuses_within_the_promise=true conflicting_with_c596=57 conflicts_with_settled_clauses=true
E7RESULT name=discrimination quantity=yi_torn_over_refusals.into_undecidable observed=57 threshold=1 judged=true moved_threshold=58 judged_at_moved=false flips=true
E7RESULT name=discrimination quantity=yi_torn_c596_conflicts.into_undecidable observed=57 threshold=1 judged=true moved_threshold=58 judged_at_moved=false flips=true
E7RESULT name=yi_offline_way_out yi=into_undecidable over_refused_states=57 walked_out_states=51 outcomes=[refused:TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion:6,rewritten:51] attached_quantity=true
E7RESULT name=verdict segment=torn_rotation constants_equal=true states=513 voided_segments=0
$ grep 'name=torn_segment' research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out | grep -c 'states=9 expected_states=9'
57
```

  - Yi refuse-directly over-refuses 285 states (95 per S: publish-end rotation 65, number acquisition 20, floor-raise-first-write 10), Yi into-undecidable over-refuses 57 states (19 per S: 13 / 4 / 2); this matches the prediction: in each 9-state segment Yi refuse-directly refuses 5 states and Yi into-undecidable refuses 1.
  - All over-refused states fall on the conflict cells with what D22 settled item 8 scope and C596 (a system configuration image whose witness cannot be decided is left with read-only mounts only) settled (`conflicting_with_c596` 285 / 285 and 57 / 57, `conflicts_with_settled_clauses=true`): no device read error, every disk has at least one slot that verifies itself, the bad slots are torn, a shape on which writable mount is promised today. The failure clause "Yi over-refuses and conflicts with settled clauses" fires (a legitimate result).
  - Attached (not in the judgement): the offline way out `rewrite_system_configuration_slots_for_the_asserted_version` gets through 255 / 285 and 51 / 57, and the remaining 30 and 6 states are refused with `TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`.

### Stops and Checks

- Stop S9 of registration section 11 (real write disagrees with the arithmetic model) fires on 0 cells in the rerun: the artifact has no `name=stop_clause` line, and the verdict line has `stop_clause_mismatches=0`. The two cells that disagreed in the first artifact (G hide-one, Jia merge-read-bad-only, root reading back all-zero × the two system configuration bad reads, lines 224 and 233) were a model that missed the step "the new instance's first root overwrites, by the slot formula, the root that lived in that slot": in the real write the new instance's first txg 6 lands on D's own slot and overwrites D (`roots=` has no (1,6), Bing's mode in the same cell too), so the inversion does not hold; registration revision 15 added that step to the model, and in the rerun these two cells have `model_inverted=false` and `fixed_crash_point_model_against_the_written_image=equal`. Commands and whole lines:

```text
$ grep 'name=cell geometry=hide_one' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep 'arm=today ' | grep 'crash_point=fixed' | grep -v 'newest_system_configuration_fault=none' | grep -c 'inverted=false'
6
$ sed -n '183p;224p;233p' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
E7RESULT name=cell geometry=hide_one root_and_record_fault=device_error newest_system_configuration_fault=all_zero arm=today mount=ok instance=2 chosen=(1,5) effective=(1,5) row_txg=6 row_txg=6 apparatus_first_txg=6 apparatus_highest_record_txg=5 branch=first_txg=past_the_highest_root_and_record_in_the_ring system_configuration_witness_undecidable=false system_configuration=[all_zero/ok,all_zero/ok] refuse_directly=true into_undecidable=true bad_slot_txgs_against_the_allocator=equal crash_point=fixed persisted_writes=45/48 roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(1,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(2,6),(2,7)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=false inverted=false checker_red=[] checker_red_count=0 choose_root=(2,6) content=content_3 confirmed_lost=false mount_after=ok:chosen=(2,6),row=(3,8) mount_after_content=content_3 mount_after_confirmed_lost=false
E7RESULT name=cell geometry=hide_one root_and_record_fault=all_zero newest_system_configuration_fault=all_zero arm=past_read_bad mount=ok instance=2 chosen=(1,5) effective=(1,5) row_txg=6 row_txg=6 apparatus_first_txg=6 apparatus_highest_record_txg=5 branch=first_txg=past_ring_slots_read_bad_at_mount system_configuration_witness_undecidable=false system_configuration=[all_zero/ok,all_zero/ok] refuse_directly=true into_undecidable=true bad_slot_txgs_against_the_allocator=equal crash_point=fixed persisted_writes=45/48 roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(1,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(2,6),(2,7)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=false inverted=false checker_red=[] checker_red_count=0 choose_root=(2,6) content=content_3 confirmed_lost=false mount_after=ok:chosen=(2,6),row=(3,8) mount_after_content=content_3 mount_after_confirmed_lost=false model_inverted=false model_lost=false fixed_crash_point_model_against_the_written_image=equal
E7RESULT name=cell geometry=hide_one root_and_record_fault=all_zero newest_system_configuration_fault=not_self_verified arm=past_read_bad mount=ok instance=2 chosen=(1,5) effective=(1,5) row_txg=6 row_txg=6 apparatus_first_txg=6 apparatus_highest_record_txg=5 branch=first_txg=past_ring_slots_read_bad_at_mount system_configuration_witness_undecidable=false system_configuration=[not_self_verified/ok,not_self_verified/ok] refuse_directly=true into_undecidable=true bad_slot_txgs_against_the_allocator=equal crash_point=fixed persisted_writes=45/48 roots=[(0,0),(1,1),(1,2),(1,3),(1,4),(1,5),(2,6)] records=[(1,1),(1,2),(1,3),(1,4),(1,5),(2,6),(2,7)] roots_of_two_instances_at_one_txg=false instance_and_txg_order_crossed_on_roots=false instance_and_txg_order_crossed_on_roots_and_records=false inverted=false checker_red=[] checker_red_count=0 choose_root=(2,6) content=content_3 confirmed_lost=false mount_after=ok:chosen=(2,6),row=(3,8) mount_after_content=content_3 mount_after_confirmed_lost=false model_inverted=false model_lost=false fixed_crash_point_model_against_the_written_image=equal
$ grep -c 'name=stop_clause' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
0
$ grep 'name=cell ' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep -c 'model_inverted'
48
$ grep 'name=cell ' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep 'model_inverted' | grep -c 'fixed_crash_point_model_against_the_written_image=equal'
48
```

  Reading: for the 48 cells of the two Jia modes (four geometries × six cells × two modes) the inversion judgement and image of the real write equal the revised arithmetic model cell by cell (48 / 48 `equal`), no "model mismatch" in the model column; in the first artifact the S9 checks at the G main point also all passed.
- Absolute-value anchors and controls:

```text
$ grep -c 'name=six_cell_anchor' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
96
$ grep 'name=six_cell_anchor' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep -c 'control=as_expected'
96
$ grep -c 'name=jia_anchor' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
18
$ grep 'name=jia_anchor' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep -c 'control=as_expected'
18
$ grep -c 'name=yi_hand_made_image' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out
9
$ grep 'name=yi_hand_made_image' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep -c 'control=as_expected'
9
$ grep 'name=s7' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | grep -c 'same_outcome=true same_whole_image=true same_fixed_image=true'
28
```

  The 96 six-cell anchor lines, the 18 `jia_anchor` lines and the 9 `yi_hand_made_image` lines are all `as_expected`, S7 (Bing through the switch's today mode is byte-identical with the product entry) 28 / 28; S3 `bad_slot_txgs_against_the_allocator` in segment three is `equal:33182,not_mounted:72` for S = 8 and `equal:118960,not_mounted:144` for S = 4, 16 (`not_mounted` 72 and 144 = 6 and 12 single-fault groups × 12 refused states); the rerun artifact has no `name=stop_clause` line at all (S1, S2, S3, S5, S6, S7, S8, S9 did not fire), and the four `name=s8` lines of S = 4, 16 also have `same_as_today_at_every_mount_boundary=true`; in every `name=cell` line `apparatus_first_txg` equals the write-row txg (with command and verbatim output; 6 of the 202 lines are refused control cells with `row_txg=none`, which have no write row, the remaining 196 are equal line by line):

```text
$ grep 'name=cell ' research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out | awk '{r="";a=""; for(i=1;i<=NF;i++){ if($i ~ /^row_txg=/ && r=="") r=substr($i,9); if($i ~ /^apparatus_first_txg=/) a=substr($i,21)}; n++; if(r!=a){d++; if(r!="none") bad++}} END{print "lines="n, "differing="d, "differing_with_a_write_row="bad+0}'
lines=202 differing=6 differing_with_a_write_row=0
```
### State Count Check

- The 513 states of segment two against "at most 540" in registration section 6: 540 = 3 groups of S × 20 segments × 9 states; the registration states an upper bound and the history decides the real count (revision 4 takes at most 20 segments); the real count is 19 segments per group of S (publish-end rotation 13, number acquisition 4, floor-raise-first-write 2), 3 × 19 × 9 = 513, inside the bound, and the 27 missing states are 1 segment fewer in each of the 3 groups of S. Every segment has `states=9 expected_states=9`, the void clause did not fire.
- The 33254 states of segment three S = 8 against 32910 in registration section 4: 344 more. Breakdown (counted from the output of the `s8_states` command above): the registration writes `≤ 4(18 + 3p) + 1` for the no-fault family, 97 at p = 2 and 361 at p = 24; the real counts are 115 and 483 for Bing and Jia merge-read-bad-only (18 and 122 over) and 107 and 487 for Jia merge-all-zero (10 and 126 over), 416 more over the three modes. For the single-fault family the registration has 29 and 117 clean states with 72 states each (3S slots × 3 kinds of bad read): Bing and Jia merge-read-bad-only have 27 and 119 clean states, 146 over both p, the same as the registration's 29 + 117 = 146; Jia merge-all-zero has 25 and 120, 145 in total, one clean state fewer (72 states fewer). Total 416 − 72 = 344.
- Cause (inferred): registration section 4 says the segment count "≤ 18 + 3p" is an upper-bound estimate with the number acquisition and floor-raise segments not counted one by one; the real no-fault counts exceed that "upper bound", which means each mount has more segment boundaries than estimated. The artifact does not print the segment count per mount, so it was not checked segment by segment. All judgements in registration section 6 are count thresholds (reaching 1 or 2) and do not depend on the number of states, so the difference changes no row's judgement.
- The 119104 states of segment three S = 4, 16 against the 118308 derived by the registration: 796 more. Where the derivation comes from: registration section 4 gives 151218 over the three modes, minus 32910 for the two groups of S = 8 (per mode 97 + 361 no-fault states plus (29 + 117) × 72 single-fault states, i.e. 10970 per mode, 32910 over three), giving 118308; the registration formulas give the same directly, per mode 97 + 217 + 97 + 649 = 1060 no-fault states and 29 × 36 + 69 × 36 + 29 × 144 + 213 × 144 = 38376 single-fault states, 3 × (1060 + 38376) = 118308. The real counts added up by mode and family (command and output verbatim):

```text
$ grep 'name=jia_skip_family' research/results/e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out | sed -E 's/.*arm=([a-z_]+) family=([a-z_]+) slots_per_region=.* states=([0-9]+) mounted.*/\1 \2 \3/' | awk '{s[$1" "$2]+=$3} END{for(k in s) print k, s[k]}' | sort
past_read_bad no_fault 1388
past_read_bad_or_all_zero no_fault 1380
past_read_bad_or_all_zero single_fault 38196
past_read_bad single_fault 38376
today no_fault 1388
today single_fault 38376
```

  Where the difference comes from: the no-fault family has 1388 + 1388 + 1380 = 4156 over the three modes against the registration's 3 × 1060 = 3180, 976 more (as for S = 8, the cause is inferred, the artifact does not print the segment count per mount: the segment boundaries per mount are more than the registration's "≤ 18 + 3p" estimate; Bing and Jia merge-read-bad-only have 328 more per mode, Jia merge-all-zero 320 more); in the single-fault family Bing and Jia merge-read-bad-only have 38376, equal to the registration (the clean-state count of each group differs from the registration by ±2, and weighted by 3S × 3 kinds of bad read the differences cancel), Jia merge-all-zero has 38196, 180 fewer than the registration (S = 4 has 4 clean states fewer at p = 2 and 3 more at p = 12, S = 16 has 4 fewer at p = 2 and 3 more at p = 48, weighted by 36 and 144 fault states per clean state: −144 + 108 − 576 + 432 = −180). 976 − 180 = 796. The state count difference affects no row's judgement.

### Fork table

| Candidate | State | Which artifact lines it rests on |
|---|---|---|
| Jia merge-all-zero | Judgeable | Segment one G main point six cells: `six-cells-2026-11-07.out` lines 394, 395 (0 cells inverted; 6 cells lose C; pool-level checker red on 0; refused afterwards 0); small ring, large ring, hide-one lines 407–408, 416–417, 425–426; segment three S = 8 no-fault and single-fault families, p = 2 and p = 24: `first-txg-skips-s8.out` lines 36, 39, 52, 55, trajectories lines 12–23; segment three S = 4, 16 four groups: `first-txg-skips-s4-s16.out` lines 50, 53, 66, 69, 82, 85, 98, 101, trajectories lines 14, 15, 20, 21, 26, 27, 32, 33; new-pool bytes `six-cells-2026-11-07.out` lines 270, 273, 276 |
| Jia merge-read-bad-only | Judgeable | Segment one G main point six cells: lines 398, 399 (2 cells stay inverted; 4 cells lose C; pool-level checker red on 2 cells; refused afterwards 2 cells); the other three groups lines 409–410, 418–419, 427–428; segment three S = 8: `first-txg-skips-s8.out` lines 24, 25, 30, 33, 46, 49; S = 4, 16: `first-txg-skips-s4-s16.out` lines 44, 47, 60, 63, 76, 79, 92, 95, trajectories lines 16, 17, 22, 23, 28, 29, 34, 35; new-pool bytes lines 269, 272, 275 |
| Yi refuse-directly | Judgeable | Segment two torn rotation, three S: `torn-rotation.out` lines 582–594 (over-refuses 285, all in conflict with C596's settled clause); six cells `six-cells-2026-11-07.out` lines 402, 411, 420, 429 (6 / 6 refused in each of four geometries) |
| Yi into-undecidable | Judgeable | `torn-rotation.out` lines 595–607 (over-refuses 57, all in conflict with C596's settled clause); six cells `six-cells-2026-11-07.out` lines 403, 412, 421, 430 (6 / 6 refused in each of four geometries) |
| Bing | Judgeable: the checked-in apparatus reproduces the minimal-history six cells, positive and negative controls passed | `six-cells-2026-11-07.out` lines 48, 382 (the G main point cell equals the replica field by field) and line 388 (the other five 5 / 5), lines 389–390 (pool-level checker red on 6 cells, refused afterwards 6 cells); positive controls lines 246–249 and lines 393, 406, 415, 424, negative controls lines 242–245, verdict line 431 `controls_failed=0` |

"Judgeable" for the two Jia rows: both the real-write six-cell inversion readings and the skip distributions of the two families exist (registration section 1, "to which quantities the judgeable conditions of the five fork-sheet rows fall"); the half of the comparison that says "Bing in the same cell loses no C and is not red" uses Bing's rows as the base, Bing's positive control passed and its readings are valid, Bing has 24 cells that do not lose C (see the command in the "Jia Merge-All-Zero and Jia Merge-Read-Bad-Only" section), so that half no longer rests on the replica reading. Every quantity of the registration has been run, none is left as "not run after judgeable"; the four groups S = 4, 16 of segment three have been run, and the geometry sensitivity of the two Jia rows is judged on six groups (S, p) (same direction, no flip).

#### Fork table for the user (four columns, with the cost figures of each candidate)

| Candidate | Flip observation | Judgeable condition | State |
|---|---|---|---|
| Jia merge-all-zero | Whether the inversion disappears on the six cells of the C971 minimal history; the distribution of txg skips (txgs skipped per writable mount, maximum and mean) on no-fault and single-fault histories; whether a confirmed publish is lost | Six-cell inversion readings and skip distributions measured on the checked-in apparatus, at least one geometry each for the single-fault and the no-fault family | Decided by the user: Yi into-undecidable. Judgeable. Cost figures: 0 of the six cells inverted (0 in all four geometries), but all 6 / 6 six cells lose the confirmed C (Bing does not lose it in the same cells; small ring and large ring also 6 / 6, hide-one 0), pool-level checker red on 0, refused afterwards 0; the write-row txg is 3S, and the bytes of a newly made pool change (write-row txg of the first mount after mkfs 12 / 24 / 48, not 1); it also skips in the no-fault family, maximum 3S − 1 (11 / 23 / 47), S = 8 at p = 2 has 103 of 107 states > 0, mean about 19.55; in the single-fault family S = 8 at p = 2 all 1788 mounted states > 0, mean about 20.82; the skip comes at every mount (no-fault p = 2 8 / 8 mounts, no-fault p = 3S only the first, persistent bad read 8 / 8); the image consequences of both families 36 lines with 0 red and 0 lost-confirmed; the number of refused states equals Bing's; all six groups (S, p) in the same direction |
| Jia merge-read-bad-only | Same as Jia merge-all-zero | Same as Jia merge-all-zero | Decided by the user: Yi into-undecidable. Judgeable. Cost figures: 2 of the six cells stay inverted (root reading back all-zero × two kinds of bad system configuration, write-row txg 5, same as Bing), pool-level checker red on those 2 cells, refused afterwards 2, the other four inversions disappear; the confirmed C is lost in 4 cells (small ring and large ring also 4, hide-one 0); the bytes of a newly made pool do not change (write-row txg 1); skip is 0 in all six groups of the no-fault family; the single-fault family has skip > 0 (S = 8 at p = 2: 1242 of 1932 mounted states, mean about 7.71), maximum 3S − 1; on a persistently bad history it skips at every mount; image consequences of both families 0 red and 0 lost-confirmed; the number of refused states equals Bing's; six groups in the same direction |
| Yi refuse-directly | Whether it refuses on the six cells; how many writable mounts are over-refused after an ordinary power-loss tear of the system configuration rotation (single fault within the promise) (and the number of conflict cells with what C596 (a system configuration image whose witness cannot be decided is left with read-only mounts only) settled) | Over-refusal count measured on the torn-rotation family | Decided by the user: Yi into-undecidable. Judgeable. Cost figures: 6 / 6 refused in all four geometries of the six cells; over-refuses 285 of the 513 torn-rotation states (95 per S: publish-end rotation 65, number acquisition 20, floor-raise-first-write 10), 285 / 285 in conflict with what C596 settled; the offline way out succeeds in 255 / 285 (attached, not in the judgement) |
| Yi into-undecidable | Same as Yi refuse-directly | Same as Yi refuse-directly | Decided by the user: Yi into-undecidable. Judgeable. Cost figures: 6 / 6 refused in all four geometries of the six cells; over-refuses 57 of the 513 torn-rotation states (19 per S: 13 / 4 / 2), 57 / 57 in conflict with what C596 settled; the offline way out succeeds in 51 / 57 (attached, not in the judgement) |
| Bing | Whether this shape reproduces the replica model's readings on the checked-in apparatus (how many pool-level checker lines are red, the `mount_after` outcome) | The checked-in apparatus reproduces the minimal-history six cells | Judgeable. Cost figures: the G main point cell equals the replica field by field (`reproduced=true differing_fields=[]`), the other five 5 / 5; all six cells are inverted (form Yi), pool-level checker red on 6 lines (I-3.1, I-3.21, I-3.5, I-3.9, I-4.2, I-8.14), write-row txg 5, picks (1,6) without losing the confirmed C, the later fault-free writable mount refused 6 / 6 (`NewerStateStillUnreadableAfterOneReread`); small ring and large ring also 6 / 6; at G hide-one (only D hidden) Bing is not inverted, 0 red, 0 refused; the scope sentence of D22 settled item 8, "before the floor raise the pool-level checker is not red at all on that shape", reads 6 red cells on the checked-in apparatus |

This table lists only each candidate's readings and cost figures and does not judge which path is better; that is inference and goes through the three-way process.

### Rerun

No longer rerun: the apparatus, the test-only switch, the switch's test target and the two lines of E205 (Cost of three fixes for the inverted root after multiple read failures) in `replay.sh` (the registration and the driver) have all been deleted, and the repository holds nothing needed for a rerun. The four artifacts (the "Current state" section above) and the apparatus source are in git history. The first artifact of segment one (`-six-cells.out`) has void readings because its Bing positive control failed; it is not byte-identical with the rerun (line-by-line comparison above: 10 of the first 430 lines differ), and the rerun is the one that carries weight.

### Measurement basis

- The definitions of "inversion", "form Yi", "skip", "lost", "over-refusal" and "conflict cell" are verbatim in registration section 1, "Reading rules fixed in advance".
- The two Jia modes write for real through the switch: N and the bad slots are taken from the switch's own pass over the root ring (registration revision 14); the read faults of E205 do not change during a mount, so the two passes read the same.
- Every judgement is a count threshold (≥ 1, ≥ 2, ≠ 1), no ratios are reported; the decimals in parentheses after the means are the reader's conversion, not artifact fields.
- The apparatus is deterministic: no real I/O, timing or random source enters a judgement; threads only slice, and the 148 items of segment one are identical letter for letter under sliced multi-threading and under one thread (verdict line of the rerun `thread_cross_check_identical=true`).
- The judgements go into the apparatus-items family of prune-server and are reused by unit key; segment two and the two segment-three artifacts reused 0 (first run), the segment-one rerun reused 45 and judged 103 fresh (the reused ones are verdicts with the same key already written by the interrupted previous run; the apparatus code key changed, so none of the verdicts of the first artifact's run hit again).

### What it cannot answer

- The Bing row: the journal records of the two publishes C and D in the inverted-image control were copied back from the starting image after registration revision 15 relaxed "only the write entry" (they are not written by the writer); this control only shows that the judgement (pool-level checker judges I-8.14, apparatus judges inversion) recognizes such an image, it does not show that the product writes such a record layout. Bing's reading is one image at a fixed crash point.
- Jia's sampling: segment three is the six groups S = 4, 8, 16 × p ∈ {2, 3S}, crash points only at segment boundaries; the geometry sensitivity is judged on these six points and says nothing about other S, p values.
- There is only one fixed crash point (the "whole mount written" point is recorded separately); the "whole mount written" rows are reported with the segment and are not in the judgement.
- Jia has 0 red and 0 lost confirmed writes in the family histories, which only says they did not appear in these family histories, not that other shapes cannot.
- Yi was measured only on the torn-rotation family and the six cells; it does not cover other system configuration fault shapes, and the operational cost of the offline way out after a Yi refusal was not measured, only how many states get through.
- The G hide-one geometry did not produce form Jia (Bing is not inverted; in the real write the new instance's first txg overwrites D's slot); the sampling point of the registration section 9 mutation "inversion judgement relaxed" was replaced by a hand-given key table under registration revision 15 and no longer depends on this cell.
- It does not judge which path is better, nor whether the D22 settled item 8 scope sentence should change; that is inference and goes through the three-way process.

### Paths and conclusions register

| # | Path | How computed | Source location | Shared items read |
|---|---|---|---|---|
| 1 | Switch real write | A writable mount through the test-only switch in `crates/`, measuring the really written image and the write-row txg; the six cells are enumerated points of 6 bad-read cells × 4 geometries, segment three is the two groups of p at S = 8 | No implementation: the test-only switch has been deleted, the experiment is retired, the source is in git history | Root ring placement formula (`target_for_publish` in `root_ring.rs`), the reads of `every_root_ring_slot_read_once`, the formula of D23 settled item 14 note 3 |
| 2 | Apparatus computation and arithmetic model | On the same image and the same read faults the apparatus computes the first txg with the formula of registration section 1, and derives the skip sequence and inversion by recording the txg each slot holds with the placement formula | No implementation: the apparatus has been deleted, the experiment is retired, the source is in git history | The same starting history and read fault injection, the constants 3 and S, "common rules for the two readings" in registration section 1; the apparatus writes its own copy of the placement formula (`ring_slot_of_txg`) |

The comparison of the two paths is wired as stop clauses S6 (switch write-row txg equals the apparatus computation) and S9 (real write equals the arithmetic model); this time S6 has no mismatch and S9 has 0 mismatching cells in the rerun (the first artifact had 2 at G hide-one, cause and revision in "Stops and Checks"). In the apparatus the comparison is a stop judgement, not an `assert_eq!`, so one pinned assertion is still owed.

### Affected decisions

| Decision item | Relation | Review |
|---|---|---|
| D22 (unit atomicity composition) settled item 8 | supports | 2026-11-19 unaffected: only wording and reference formatting were adjusted; conclusions, figures, and artifacts are unchanged; previously, 2026-11-09 changed: E205 (Cost of three fixes for the inverted root after multiple read failures) is retired (the apparatus, the switch and the rerun are all deleted; the readings and cost figures are in git history); the scope of the item has three other changes (the newest slot on both disks staying bad after number acquisition is refused, another pool having the largest generation number on every disk leaves only the offline way out, the inversion after a lost write is not covered), which the readings of this page did not cause; Yi into-undecidable added to the scope on 2026-11-07 and the sentence on the Bing row's reading stay as they were; the basis section of the item cites E205 (Cost of three fixes for the inverted root after multiple read failures) |
| D16 (publish semantics) settled item 1 | groundwork | 2026-11-19 unaffected: only wording and reference formatting were adjusted; conclusions, figures, and artifacts are unchanged; previously, 2026-11-09 not affected: the retirement of E205 (Cost of three fixes for the inverted root after multiple read failures) only deletes the apparatus and the rerun, and the readings do not change; Jia's rule is the same as the item's "raise the floor's upper bound"; the number of empty publishes at each normal unmount in segment three matches the rule "3 when the current txg mod 3 is 1, 2 when it is 0 or 2" (`unmount_rule_mismatches=0`); the item's basis section does not cite E205 (Cost of three fixes for the inverted root after multiple read failures) |
| D23 (journal role and format) settled item 14 | groundwork | 2026-11-19 unaffected: only wording and reference formatting were adjusted; conclusions, figures, and artifacts are unchanged; previously, 2026-11-09 not affected: the retirement of E205 (Cost of three fixes for the inverted root after multiple read failures) only deletes the apparatus and the rerun, and the readings do not change; the first-txg formula of note 3 holds at Bing's three anchors (G main point 5, G hide-one 6, first mount after mkfs 1); the item's basis section does not cite E205 (Cost of three fixes for the inverted root after multiple read failures) |

## Historical Versions

### 2026-11-09

- Retired: user settlement 「退场」 (retire); the apparatus, the test-only switch, the switch's test target, the two lines of E205 (Cost of three fixes for the inverted root after multiple read failures) in `replay.sh` and the mutation rows pointing at them are all deleted; the title state gains "conclusion retired", the rerun section now says it is no longer rerun, the source location of the two rows of the paths and conclusions register now says "No implementation", and the three rows of the affected decisions were reviewed again; the readings, artifacts and state counts do not change.

### 2026-11-07

- Created the page and wrote in the readings of three artifacts: segment one six cells, segment two torn rotation, segment three S = 8; the two Jia rows and the two Yi rows of the fork sheet are judgeable, the Bing row's positive control failed and its readings are void; the four groups of segment three with S = 4 and 16 are not run after judgeable.
- After registration revision 15 the segment-one rerun and segment three S = 4, 16 were added: the Bing row's positive and negative controls pass (`controls_failed=0`, `stop_clause_mismatches=0`), its readings are valid, the checked-in apparatus reproduces the minimal-history six cells, and all five fork-sheet rows are judgeable; the four groups of segment three with S = 4 and 16 have been run, and the geometry sensitivity of the two Jia rows is in the same direction over six groups. The title state is changed to "run (judgeable)".
