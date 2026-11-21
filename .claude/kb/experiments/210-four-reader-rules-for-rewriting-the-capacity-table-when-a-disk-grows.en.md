## E210 Four reader rules for rewriting the capacity table when a disk grows —— run (rows 2 and 3 sufficient to judge; row 1 sufficient within the apparatus, two of its cells disagree with a second apparatus written by the attacking side and are not judged; the registration's literal wording of voiding clause 1 triggered and its disposition awaits the main agent): one artifact per segment is stored, 40 unit tests pass, all 27 mutation rows caught

[中文](210-盘变大改容量表的读者规则四臂.md) · **English** · [日本語](210-ディスク拡大時の容量表書き換えにおける読者規則四案.ja.md)

Pre-run registration `research/prompts/e210-preregistration.md` (including the revisions in section twelve), fork sheet `research/prompts/m4-capacity-grow-r1-forks.md` with three rows. The question: the system configuration of each disk in the pool (two rotating slots per disk) carries a per-disk capacity table; the writable mount after a disk grows writes the new table into every slot of every disk, and power may fail halfway; after the power loss the user may leave things alone, shrink the disks back, grow them again, or swap a disk back to an earlier whole-disk image. Which table the reader takes as the table in force when the tables in the slots are not all the same, which disk faces are refused for the whole pool, and which disks a read-only mount names, are the four candidate reader rules of row 1 (Arms A, B, C and D) plus a baseline arm; row 2 counts how many system configuration slot writes and barriers one writable mount that grows the disks performs under each rule; row 3 counts the whole slots usable after the growth under the two ways of recording nothing and recording up to the boundary when the growth crosses a root level boundary of the allocation record tree.

**This page records the numbers the artifacts measured and the result of the cross-check. It does not judge whether any decision is overturned or established. Two cells of the leak half of row 1 disagree with a second enumeration model written by the attacking side (the writable leaks of Arms A and D, and the read-only leak of Arm C); the page lists them side by side marked "the two apparatuses disagree, not judged" and does not assume either is right.**

### Current state

- Apparatus: `scripts/experiment-apparatus/e210_capacity_grow_reader_rules/main.rs` (bin `e210-capacity-grow-reader-rules`): an independently hand-written counting model that shares no code with `crates/` and is not a checked-in apparatus; the command-line argument selects one artifact segment: `write-counts-and-root-level` (rows 2 and 3) and `reader-rules-on-histories` (row 1).
- Unit tests: 38 in the apparatus plus 2 for the shared thread default brought in by `include!`, 40 in all, run by the main agent: `test result: ok. 40 passed; 0 failed`. The mutation table `scripts/mutation/tables/e210_capacity_grow_reader_rules.tsv` with 27 rows was run: "计数：抓到 27 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 27 条、复用 0 条、拒算 0 条" (27 caught, 0 not red, 0 invalid, 0 unfinished, 0 name-blind, 0 anchor lost, 27 judged now, 0 reused, 0 refused) and "已还原，基线仍全绿" (restored, baseline still all green; verbatim in the run log).
- Artifacts (built and run by the main agent, exit 0): `research/results/e210-capacity-grow-reader-rules-write-counts-and-root-level-2026-11-18.out` (32 lines, `done emitted=32`) and `research/results/e210-capacity-grow-reader-rules-reader-rules-on-histories-2026-11-18.out` (165 lines, `done emitted=165`); the two E210 rows of `scripts/experiment-apparatus/replay.sh` point at them. Voiding clause 2 of section eleven of the registration (two runs byte-identical) and the naming check have no result yet and are left to the main agent; there is no result for them when this page is written.
- Not done this time: the third segment of section five of the registration (geometry sensitivity and the self-test of discriminative power, that is, stability) is not dispatched, and the artifact carries two `name=not_run_this_segment` lines; the side quantities not run once sufficiency is reached (witnesses and distributions, whole-pool refusals and read-only refusals and the class-size histogram, the write count of the abandon branch of Arm D, the rotation at the end of a publish, the points of the per-disk greedy variant of Arm B and of the one-above-the-boundary row that no mutation needs) are each marked `name=not_run_after_sufficient` (two lines for row 1, three for rows 2 and 3).
- The hand-derived outcomes of items 8 to 13 of section four of the registration each take one `hand_derivation` line in the row 1 artifact, seven in all with `holds=true`, and the verdict line has `hand_derivations_overturned=zero`; a hand derivation is not a conclusion.

### Measurement basis

1. The pool has two disks (device identities 0 and 1) with exactly two slots each; disk sizes and capacity tables are both counted in whole slots (16 KiB per slot). The reader only recognizes valid slots (slot generation, table, intent marker); a torn slot cannot be read.
2. A history is a tuple over five dimensions: family, shape, footprint, power-loss point, user action; each history has two mounts, writable and read-only. The power-loss points are the enumeration after each arm's write order is cut into persistence segments by the pool barriers: each system configuration slot write takes one of three states (not persisted, persisted, neither old nor new readable), plus one end-of-use point. The four-write order (Arms A, B, D and the baseline arm) has 18 raw states, the six-write order (Arm C) has 27; the history tuples are 1487 writable for the four-write order and 2207 for the six-write order, read-only the same (the five `name=history_space` lines of the artifact read the same numbers).
3. Five arms: Arm A (take the entry-wise lowest when the tables in the valid slots form a chain, refuse the whole pool otherwise), Arm B (take the entry-wise highest), Arm C (write one round of intent-marked slots first; when a slot on the disks carries a marker, take the old table), Arm D (read with the lowest, compare against the probe with the highest, and a growth that was not fully written may be abandoned by writing the old table back), the baseline arm (accept only when all the tables are equal).
4. Three control readers, each run once on the history set of every arm: allow everything (must leak), reject every writable mount (must misrefuse), and the class judge (the answer is given by construction; its leaks and avoidable misrefusals must be 0).
5. Two predicates: a leak (some unit's placement goes past the end of some disk as it is now, and the writable mount is made, or the read-only mount is made without naming that disk); a misrefusal (the writable mount is refused while no placement goes out of bounds). Misrefusal is reported under two readings side by side: the literal reading and the avoidable reading (no history in that observation class goes out of bounds); the literal reading is always true on every arm, so it cannot tell the arms apart; the misrefusal half of row 1 is flipped on the avoidable reading, which the main agent has accepted.
6. Row 2: the write-order generator counts the slot writes and the pool barriers preceded by a write, with no hand-entered constants.
7. Row 3: the level arithmetic is written a second time, not copied from `crates/`, and is put through a differential check against the four level readings pinned by the unit tests in `crates/`; six sampling points: the three of below the boundary, crossing one, crossing two, plus a synthetic control point and the two one-above-the-boundary points that the mutations need. How Arm B's "record up to the boundary" is split over the two disks is the reading of item 2 of the last column of section one of the registration (largest total; on equal totals the smaller difference between the two disks; on equal again the group met first; the revisions paragraph has brought the registration's wording into line with the apparatus).
8. Determinism: the model has no random source, clock or concurrency.

### Artifact readings

**Row 2 (`…write-counts-and-root-level-2026-11-18.out`, lines 2 to 7, whole lines):**

```text
E7RESULT name=write_count arm=chain_then_lowest writes=4 pool_barriers=2 reference_writes=4 reference_pool_barriers=2 extra_writes=0 extra_pool_barriers=0 more_than_the_reference=false
E7RESULT name=write_count arm=highest_table writes=4 pool_barriers=2 reference_writes=4 reference_pool_barriers=2 extra_writes=0 extra_pool_barriers=0 more_than_the_reference=false
E7RESULT name=write_count arm=old_table_while_intent_marker writes=6 pool_barriers=3 reference_writes=4 reference_pool_barriers=2 extra_writes=2 extra_pool_barriers=1 more_than_the_reference=true
E7RESULT name=write_count arm=lowest_to_read_highest_to_probe writes=4 pool_barriers=2 reference_writes=4 reference_pool_barriers=2 extra_writes=0 extra_pool_barriers=0 more_than_the_reference=false
E7RESULT name=write_count arm=baseline_only_if_every_table_is_equal writes=4 pool_barriers=2 reference_writes=4 reference_pool_barriers=2 extra_writes=0 extra_pool_barriers=0 more_than_the_reference=false
E7RESULT name=write_count_control plan=four_rounds writes=8 pool_barriers=4 expected_writes=8 expected_pool_barriers=4 matches=true
```

One growth by Arm C writes 2 more slots and 1 more barrier; the other three arms and the baseline arm equal the number acquisition write of a mount that does not grow (4 writes, 2 barriers); the four-round control reads 8 and 4 and passes. Row 2 is sufficient to judge.

**Row 3 (same file, lines 15 to 22, whole lines):**

```text
E7RESULT name=crates_cross_check per_disk_slots=262144 root_level=2 crates_root_level=2 crates_test=two_four_gibibyte_devices_give_a_tree_of_height_three matches=true
E7RESULT name=crates_cross_check per_disk_slots=67108864 root_level=3 crates_root_level=3 crates_test=two_one_tebibyte_devices_give_a_tree_of_height_four matches=true
E7RESULT name=crates_cross_check per_disk_slots=68208 root_level=1 crates_root_level=1 crates_test=half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary matches=true
E7RESULT name=crates_cross_check per_disk_slots=68209 root_level=2 crates_root_level=2 crates_test=half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary matches=true
E7RESULT name=root_level_point label=below_boundary old_records=60000,60000 new_probe=68208,68208 old_root_level=1 new_root_level=1 root_level_changes=false chain_total_slots=136416 highest_total_slots=136416 highest_split=68208,68208 highest_minus_chain_slots=0 chain_total_unit_area_slots=36064 highest_total_unit_area_slots=36064 highest_minus_chain_unit_area_slots=0 growth_available_slots=16416 highest_keeps_fraction_of_the_growth=0.0000 diverges=false
E7RESULT name=root_level_point label=crosses_one_boundary old_records=60000,60000 new_probe=262144,262144 old_root_level=1 new_root_level=2 root_level_changes=true chain_total_slots=120000 highest_total_slots=137228 highest_split=68208,69020 highest_minus_chain_slots=17228 chain_total_unit_area_slots=19648 highest_total_unit_area_slots=36876 highest_minus_chain_unit_area_slots=17228 growth_available_slots=404288 highest_keeps_fraction_of_the_growth=0.0426 diverges=true
E7RESULT name=root_level_point label=crosses_two_boundaries old_records=60000,60000 new_probe=67108864,67108864 old_root_level=1 new_root_level=3 root_level_changes=true chain_total_slots=120000 highest_total_slots=137228 highest_split=68208,69020 highest_minus_chain_slots=17228 chain_total_unit_area_slots=19648 highest_total_unit_area_slots=36876 highest_minus_chain_unit_area_slots=17228 growth_available_slots=134097728 highest_keeps_fraction_of_the_growth=0.0001 diverges=true
E7RESULT name=root_level_point label=synthetic_control_crossing old_records=68208,68208 new_probe=262144,262144 old_root_level=1 new_root_level=2 root_level_changes=true chain_total_slots=136416 highest_total_slots=137228 highest_split=68208,69020 highest_minus_chain_slots=812 chain_total_unit_area_slots=36064 highest_total_unit_area_slots=36876 highest_minus_chain_unit_area_slots=812 growth_available_slots=387872 highest_keeps_fraction_of_the_growth=0.0021 diverges=true
```

Below the boundary the two ways are the same (difference 0), crossing one and crossing two boundaries each differ by 17228 whole slots (Arm B 137228 against Arm A 120000), the synthetic control point differs by 812, and all four differential checks against the level readings pinned in `crates/` are `matches=true`. Row 3 is sufficient to judge; Arm B's split over the two disks takes the registration's reading, and the other reading (the per-disk greedy variant, 137140) is marked `not_run_after_sufficient` and was not run. The three level unit tests in `crates/` that the main agent submitted separately were green in the previous segment (three ok) and were not resubmitted when this page was written.

**Row 1 (`…reader-rules-on-histories-2026-11-18.out`, lines 7 to 31, whole lines; denominator = writable tuples, 1487 for Arms A, B, D and the baseline arm, 2207 for Arm C):**

```text
E7RESULT name=reading arm=chain_then_lowest reading=leak_writable count=103 denominator=1487 exists=true
E7RESULT name=reading arm=chain_then_lowest reading=leak_read_only count=103 denominator=1487 exists=true
E7RESULT name=reading arm=chain_then_lowest reading=misrefusal_literal count=164 denominator=1487 exists=true
E7RESULT name=reading arm=chain_then_lowest reading=misrefusal_avoidable count=30 denominator=1487 exists=true
E7RESULT name=reading arm=chain_then_lowest reading=writable_refused count=287 denominator=1487 exists=true
E7RESULT name=reading arm=highest_table reading=leak_writable count=0 denominator=1487 exists=false
E7RESULT name=reading arm=highest_table reading=leak_read_only count=0 denominator=1487 exists=false
E7RESULT name=reading arm=highest_table reading=misrefusal_literal count=550 denominator=1487 exists=true
E7RESULT name=reading arm=highest_table reading=misrefusal_avoidable count=284 denominator=1487 exists=true
E7RESULT name=reading arm=highest_table reading=writable_refused count=776 denominator=1487 exists=true
E7RESULT name=reading arm=old_table_while_intent_marker reading=leak_writable count=0 denominator=2207 exists=false
E7RESULT name=reading arm=old_table_while_intent_marker reading=leak_read_only count=0 denominator=2207 exists=false
E7RESULT name=reading arm=old_table_while_intent_marker reading=misrefusal_literal count=431 denominator=2207 exists=true
E7RESULT name=reading arm=old_table_while_intent_marker reading=misrefusal_avoidable count=66 denominator=2207 exists=true
E7RESULT name=reading arm=old_table_while_intent_marker reading=writable_refused count=756 denominator=2207 exists=true
E7RESULT name=reading arm=lowest_to_read_highest_to_probe reading=leak_writable count=103 denominator=1487 exists=true
E7RESULT name=reading arm=lowest_to_read_highest_to_probe reading=leak_read_only count=0 denominator=1487 exists=false
E7RESULT name=reading arm=lowest_to_read_highest_to_probe reading=misrefusal_literal count=164 denominator=1487 exists=true
E7RESULT name=reading arm=lowest_to_read_highest_to_probe reading=misrefusal_avoidable count=30 denominator=1487 exists=true
E7RESULT name=reading arm=lowest_to_read_highest_to_probe reading=writable_refused count=287 denominator=1487 exists=true
E7RESULT name=reading arm=baseline_only_if_every_table_is_equal reading=leak_writable count=0 denominator=1487 exists=false
E7RESULT name=reading arm=baseline_only_if_every_table_is_equal reading=leak_read_only count=0 denominator=1487 exists=false
E7RESULT name=reading arm=baseline_only_if_every_table_is_equal reading=misrefusal_literal count=885 denominator=1487 exists=true
E7RESULT name=reading arm=baseline_only_if_every_table_is_equal reading=misrefusal_avoidable count=619 denominator=1487 exists=true
E7RESULT name=reading arm=baseline_only_if_every_table_is_equal reading=writable_refused count=1111 denominator=1487 exists=true
```

Controls and self-checks (lines 155 to 158 and 161 to 164, whole lines):

```text
E7RESULT name=verdict check=positive_controls allow_everything_leaks_on_every_arm=true reject_every_writable_mount_misrefuses_on_every_arm=true
E7RESULT name=verdict check=negative_controls negative_control_violations=zero
E7RESULT name=verdict check=decision_is_function_of_observation differing_class_violations=zero
E7RESULT name=verdict check=literal_minus_avoidable at_least_one_arm_literal_exceeds_avoidable=true
E7RESULT name=verdict check=out_of_model out_of_model_violations=zero
E7RESULT name=verdict check=history_counts history_count_mismatches=zero
E7RESULT name=verdict check=hand_derivations hand_derivations_overturned=zero
E7RESULT name=verdict check=arms_distinguishable any_two_of_the_four_candidate_arms_differ=true
```

Judgment per item under the thresholds of section six of the registration (the misrefusal half flipped on the avoidable reading):

- Leak (writable): Arms A and D 103 / 1487 each, exists; Arms B, C and the baseline arm 0 (0 / 1487, 0 / 2207, 0 / 1487): none in this history space, not written as "never".
- Leak (read-only): Arm A 103 / 1487, exists; Arms B, C, D and the baseline arm 0.
- Misrefusal (avoidable): Arm A 30, B 284, C 66, D 30, baseline arm 619, exists on all five arms; the literal reading is 164, 550, 431, 164, 885, likewise on all five arms, and as the registration expected it cannot tell the arms apart.
- Every candidate has at least one flipping observation (Arms A and D have leaks and misrefusals, Arms B and C have misrefusals); the four candidates are not all pairwise identical on the four readings (`any_two_of_the_four_candidate_arms_differ=true`), so item 11 of section ten does not trigger. Arm C differs from Arm B in the amount of misrefusal and in the write count (row 2), not in leaks.
- Hand derivations (items 9 and 11 of section four of the registration): the leak of Arms A and D on the early image holds (lines 143 and 146 of the artifact, `holds=true`, 99 writable leaks on the early image), the witness histories are on lines 149 and 150; all seven hand derivations are `holds=true`.

### Voiding clause 1: the invariance check

The registration's literal wording (item 4 of section ten, kept verbatim by the revisions paragraph): line 159 of the artifact, `name=verdict check=invariance invariance_mismatches=5`, one mismatch on each of the five arms, coming from the form where the "grow again, 100 on every disk" lands on the renumbered integers (`variant=alternative_integers_with_fixed_growth_step`). The mismatched fields are listed field by field in the 20 `name=invariance_detail` lines:

```text
$ grep -c 'name=invariance_detail .* field=classes ' <row 1 artifact>
20
$ grep -c 'name=invariance_detail' <row 1 artifact>
20
```

That is, every detail line is `field=classes`: the number of observation classes goes from 474 to 490 for Arms A, B, D and the baseline arm and from 684 to 716 for Arm C; none of the five reading counts, the counts on the early image, the trajectories, the number of classes with differing decisions or the whole-pool refusal count mismatches anywhere. The mechanism agrees with what the revisions paragraph wrote: once the probe of "grow again" has 100 added, cross-shape probes that were equal in the main grid are no longer equal on the renumbered integers, the observation classes multiply, and the reader's decisions do not change.

The order-preserving criterion that the revisions paragraph added: line 160 of the artifact, `name=verdict check=invariance_order_preserving order_preserving_mismatches=zero`; the three other parity combinations and the order-preserving form make 20 lines of `equal_to_main=true`, and only the 5 lines of the add-100-directly form are `false` (`grep -c 'name=invariance arm=.* equal_to_main=true'` = 20, `… equal_to_main=false` = 5).

Disposition: the registration's literal wording triggers voiding clause 1 of section eleven by its letter; the order-preserving criterion was added by the revisions paragraph as "only tighten, add one", and it passes. **Whether this clause still voids is for the main agent to judge**; this page does not judge it and does not change the voiding clause itself. If it is left unchanged, the readings of row 1 are written as they are and the whole round is redone; if the order-preserving form is taken as authoritative, the readings above stand.

### Cross-check of the two apparatuses: cells that disagree

The enumeration model written separately by the attacking side (`capacity_growth_reader_rules.py`, sharing no code with E210; two-run model output) gives, on the same candidate rules, the following against E210:

| Cell | E210 | Attacker's model | Where they differ | State |
|---|---|---|---|---|
| Arm A writable leak | 103 / 1487 | 0 | E210 has no lagging-disk branch (the `standard_outcome` and `arm_decision` of `e210_capacity_grow_reader_rules.rs:890`–`:938` have a single refusal cause, "probe below the comparison table"); the attacker's `capacity_growth_reader_rules.py:222`–`:227` refuses the writable mount for a disk that is behind the selected version and lacks its units. `crates/` has this branch today: `mount.rs:8237` (`WritableMountRefusedByDevicesWithoutTheSelectedVersion`), the list computed by `what_a_device_behind_the_version_lacks` at `mount.rs:7247`; it also has a geometry check that judges the span of an allocation record against the disk's size as it is now (`mount.rs:9993` rebuilding the previous version, then `recovery.rs:4470`, then `:2866`), which neither apparatus builds | the two apparatuses disagree, not judged |
| Arm D writable leak | 103 / 1487 | 0 | Same cause as Arm A writable leak; Arm D is made through the abandon branch | the two apparatuses disagree, not judged |
| Arm C read-only leak | 0 / 2207 | 9 to 223 over five variants | History space: E210's images are only the early image and the front image, neither carrying an intent marker (`e210_capacity_grow_reader_rules.rs:603`–`:609`); the attacker's image may be taken in the middle of a mount that succeeds, with marked slots, and after it is put back into the old-size disk Arm C takes the old table of the marked slot and does not name that disk (`capacity_growth_reader_rules.py:448`–`:452`). `crates/` has neither the capacity table nor the intent marker (`grep -rn '容量表\|capacity_table' crates/ --include=*.rs \| wc -l` reads 0), so there is no implementation to check against | the two apparatuses disagree, not judged |
| Arm A read-only leak | 103 / 1487 | 485 | Both exist; the numbers are not comparable: the history spaces, the numbers of mounts and the ways the disks change all differ | existence agrees, numbers not compared |
| Arm B writable and read-only leaks, Arm D read-only, Arm C writable | 0 | 0 | Agree | agree |

Read-only side of `crates/`: `mounted_read.rs:635`–`:637` names a disk in a read-only mount only when neither of its slots can be chosen, and does not name a lagging disk, which agrees with both sides; where the read-only mount path references the geometry check directly I only grepped `mounted_read.rs`, with zero hits, and did not trace whether `open_pool_for_read` (`mounted_read.rs:389`) reaches `recovery.rs:2866` indirectly. What each reading would need: for the two writable cells, E210 is right iff the swapped-in image disk does not lack the units of the selected version and its journal tail is not behind (E210 builds no units or tail and cannot show this); the attacker is right iff it lacks them, and today's `crates/` reads an "earlier whole-disk image" as lacking them. For the Arm C read-only cell, what is needed is whether the fork sheet's "swap back to an earlier whole-disk image" includes an image taken in the middle of a growth and carrying a marker; the E210 registration (item 4 of section one, unsettled items) already says only two time points are taken.

### Classification of the 103 writable leaks (103 each for Arms A and D)

The artifact has the count at every power-loss point and the count on the early image (`name=trajectory arm=chain_then_lowest reading=leak_writable`: 5 at each of points 0 to 17 and 13 at the end of use, 5 times 18 plus 13 equals 103; `early_image_leak_writable=99` of `name=hand_derivation claim=chain_then_lowest_leaks_on_early_image`), but no per-history detail; the classification table computed by hand from the apparatus source has three numbers that agree with the artifact, and it is derived, not an artifact line:

| Class | Shape | Footprint | Power-loss point | User action | Count |
|---|---|---|---|---|---|
| Early image | Family D same-grow | full | points 0 to 17 | disk 0 or disk 1 swapped back to the early image | 2 × 18 = 36 |
| Early image | Family D one-grows | full | points 0 to 17 | disk 0 or disk 1 swapped back to the early image | 36 |
| Early image | Family D crossed | full | points 0 to 17 | disk 0 only (the front size of disk 1 is 100, no overrun) | 18 |
| Early image | Family D same-grow, one-grows, crossed and Family S same-grow, one-grows | full | end of use | 2 + 2 + 2 + 2 + 1 | 9 |
| Front image | Family D same-grow, one-grows, crossed | full | end of use | 2 + 1 + 1 | 4 |

Actions with no leak: leave alone, shrink, grow again, footprint "few", Family S at points 0 to 17. All 103 are "one disk swapped back to an old whole-disk image", the same thing as the lagging-disk branch in the cross-check table.

### Rerun

```bash
bash scripts/experiment-apparatus/replay.sh E210
```

Criterion `exact`, each of the two rows against one artifact.

### Paths and conclusions register

The level readings of row 3 are one quantity computed by two paths that share no code and then compared:

| # | Path | How it is computed | Source location | Shared items read |
|---|---|---|---|---|
| 1 | The level arithmetic the apparatus writes separately | The smallest root level R ≥ 1 such that the sum over disks of ⌈whole slots ÷ slots covered by one cell of level R−1⌉ does not exceed the fan-out 169; slots covered by one cell = 812 × 169 to the power R−1; the inputs are the six sampling points of the registration and the four inputs for the cross-check, not a sweep of the domain | `scripts/experiment-apparatus/e210_capacity_grow_reader_rules/write_counts_and_root_levels.rs:42 root_level_of` | The leaf coverage 812 and fan-out 169 (copied from two constants in `crates/singlefs-format/src/lib.rs`) and the level definition itself (item 7 of section three of the registration, read from `allocation_record_tree.rs`); if these are wrong both paths are wrong together |
| 2 | The geometry evaluation of `crates/` | `AllocationRecordTreeGeometry::of_devices` takes the absolute slot count of each disk and finds the same smallest root level; this page only reads the four readings its unit tests assert and does not call it | `crates/singlefs-core/src/allocation_record_tree.rs:97 of_devices` | The same two constants and the level definition |

The cross-path comparison is pinned in the apparatus unit test `root_level_arithmetic_agrees_with_the_four_readings_pinned_in_the_crates`, and the four `name=crates_cross_check` lines of the artifact are all `matches=true`; the outputs of the two paths are not connected and compared directly.

### What it cannot answer

- The capacity table is not implemented in `crates/` yet: the reader-rule side has no implementation to check against, only the write order, the barriers, the generation numbers and the root level arithmetic have implementations in `crates/`.
- It only counts "made or refused, which disks named", does not simulate the writes after a mount; the histories have no versions, unit contents or journal tail, so it cannot answer whether a swapped-in image disk lacks the units of the selected version, and neither the lagging-disk branch of the writable mount (`mount.rs:8237`) nor the geometry check by the disk's current size (`recovery.rs:2866`) is in the model.
- Images are taken at only two time points (before the growth, and after the growth and its writes), neither carrying an intent marker; an image taken in the middle of a growth is not in the histories, and the 0 read-only leaks of Arm C hold only in this history space.
- The reader rules do not look at slot numbers or generation numbers, only at the order of sizes; disk sizes and table entries use small integers, and real slot counts are used only in the row 3 group of quantities.
- What the marker slot of Arm C carries, how "take the old table" works with several slots, how Arm D's "not fully written" is recognized, and how Arm B is split over two disks are the readings written in items 2 and 3 of the last column of section one of the registration; if someone holds that an arm should carry a stricter guard, that is another arm, to be named and registered anew.
- Stability (geometry sensitivity and the self-test of discriminative power) is not done this time, so row 1 cannot write "at which sampling points it holds".
- The disposition of voiding clause 1, the choice between the cells that disagree, and the per-history detail of the 103 (a new diagnostic segment and a full rerun are needed) are not done and are left to the main agent.

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D12 (target media) settled item 4 | Preparation | 2026-11-20 unaffected: the split only changed the file layout of the apparatus source; conclusions, numbers and artifacts unchanged; previously, 2026-11-18 not affected: this page prepares "a device whose capacity shrinks is refused, a device that grows is admitted and recorded", with the readings of each reader rule on the power-loss histories of growing a disk and rewriting the table; two cells disagree with the other apparatus and the disposition of voiding clause 1 is open, so it is not raised to support; the basis of this item does not cite E210 (four reader rules for rewriting the capacity table when a disk grows), and `.claude/decision-links-pending` has no entry now |
| D22 (synthesizing unit atomicity) settled item 8 | Preparation | 2026-11-20 unaffected: the split only changed the file layout of the apparatus source; conclusions, numbers and artifacts unchanged; previously, 2026-11-18 not affected: the write order, barriers and power-loss points of the apparatus are built after this item's two number acquisition write rounds, and the row 2 reading (Arm C writes 2 more slots and 1 more barrier) does not judge it; the basis does not cite this experiment |
| D22 (synthesizing unit atomicity) settled item 16 | Preparation | 2026-11-20 unaffected: the split only changed the file layout of the apparatus source; conclusions, numbers and artifacts unchanged; previously, 2026-11-18 not affected: the generation numbers and slot placement of the apparatus are built after this item's slot rotation rule and do not judge it; the basis does not cite this experiment |
| D13 (verification route) settled item 4 | Does not affect | 2026-11-20 unaffected: the split only changed the file layout of the apparatus source; conclusions, numbers and artifacts unchanged; previously, 2026-11-18 not affected: only its persistence segments and third state are borrowed to enumerate the power-loss points, and no clause of it is judged |
| D2 (RAID stripe policy) settled item 3 | Does not affect | 2026-11-20 unaffected: the split only changed the file layout of the apparatus source; conclusions, numbers and artifacts unchanged; previously, 2026-11-18 not affected: the registration lists it as one of the clauses under test, and the measured quantities are the reader rules and the write counts, not its direction |

## Revision History

None (initial output).
