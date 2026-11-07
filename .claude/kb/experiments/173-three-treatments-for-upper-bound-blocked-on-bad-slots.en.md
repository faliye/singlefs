## E173 Three Treatments for Upper Bound Blocked on Bad Slots —— Completed (two knobs in segment three not run after sufficient basis); rerun 1 done (sufficient basis, 2026-11-02, fork sheet row 1 judged B, X6 not run after sufficient basis)

[中文](173-上限卡在读坏槽的三种处置.md) · **English** · [日本語](173-上限が不良スロットに引っかかった場合の3処置.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 H1 H2 H3 H4 H5 K1 K2 K3 K4 K5 K6 K7 PC1 PC2 PC3 PC4 B3 B5 B6 A1 F2 F4 F5 F6 F8 V1 V2 V3 V4 V5 V6 P1 P2 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M6a M6b Mm1 Mm2 Mm3 Mm4 Mm5 Mm6 Mm7 Q1-1 Q1-1b Q1-2 Q1-2b Q1-3 Q1-4 Q1-5 Q1-6 Q2-1 Q2-2 Q2-3 A3 A4 B1 B2 B4 C0 F1 X3 X4 X5 X6 Y1 Z1 Z2 Z3 F7 PC5 PC6 S6 B9 MR1 MR2 MR3 MR4 MR5 MR6 MR7 MR8 MR9 MR10 MR11 MR12 MR13 MR14 MR15 MR16 MR17 MR18 G-min G-1 G-4 C723 -->

```text
Local arm names and notation on this page are written in lowercase (t1, t2 ...): written in the corresponding uppercase in pre-registration, apparatus, artifacts, and commands, isomorphic to global registered identifiers; changed to lowercase in the kb body solely to distinguish them from those identifiers; match by uppercase when reading artifacts.
```

Pre-registration `e173-preregistration.md` (sha256 at start `68fe96ca6be43fc2af1fdbd7a4092326afbf03c125ad6555bc13cee0a0f5ed4d`; after apparatus completion and before running artifacts once, the executor wrote 11 items in Section 12 "Revisions", only supplementing sample points, mutations, criteria, and specifying implementation interpretations, without relaxing original criteria); fork sheet `m3-batch10-code-r1-forks.md` rows Q1 and Q2 (Q3 and Q4 settled by user, not included in this experiment). The tested clauses are the "raising F upper bound" and "root slot read bad this time" rows as well as the "acceptance" row in D16 (Publish Semantics) Settled Item 1; referenced definitions are D16 (Publish Semantics) Settled Item 8 (warm-up), Settled Item 9 (empty publish also writes units), and the "slot count S per region" row in D22 (Synthesizing Unit Atomicity) Settled Item 2.

The apparatus is an independently hand-written count model `scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs`, not an in-tree apparatus, does not depend on `crates/`, and shares no code with the implementation. The "as-of-today" arm adopts the behaviors in the frozen tree of `crates/singlefs-core/src/mount.rs`, `allocator.rs`, and `mounted_session.rs`: the frozen tree is the start snapshot of batch 10 code round 1; after copying into the draft directory, `sha256sum -c --quiet research/prompts/m3-batch10-code-r1-snapshot/crates-sha256.txt` exited with code 0 (all 306 files matched); at the moment work started, sha256 of `crates/singlefs-core/src/mount.rs` in the main workspace was `6f95b6fbc9f6d8e846ce…`, identical to the frozen tree (the main workspace of `allocator.rs` had extra changes from patches being merged; line numbers uniformly take the frozen tree).

**Status**: Decisive conditions for both Q1 and Q2 rows are met; neither voiding clauses V1–V6 nor halting clauses K1–K7 were hit; positive controls PC1–PC4 first passed green across all eight arms (four Q1 arms × two Q2 arms); all 17 unit tests passed and 23 mutation table entries were caught (0 invalid, 0 unkilled). Rerun 1 on added an arm with the reserve pool computed from ckpt_cost; its state and readings are in the "Rerun 1" part below, and the parts outside it describe the original two segments of 2026-10-05. On the user settled to change the clause to follow the code and reopened the two settlements of 2026-10-05: when a bad root slot merged into the ceiling blocks it, the setting is "push empty" (push an empty publish and raise F, then judge again), and a root slot that reads back all zeros is "merged" (merged into the ceiling); D16 (Publish Semantics) settled item 1 has been changed (the user's words are in the same-day block of the D16 (Publish Semantics) section of `.claude/kb/decisions-history.md`); this replaces the two settlements of 2026-10-05 written in the "User settlement" item of this page.

- **Differential (halting clauses K1–K6)**: The values given by the apparatus on the two disk layouts (S = 4, hole-free and with holes) from the frozen tree test cases are identical cell by cell to those pinned by the test case: unreadable with holes has upper bound 6, F raised to 6 (K1); unreadable without holes is 17 (K2, same value for both e and g read failures); reading back all-zeros without holes, all-zeros not merged is 18 and crosses 1 non-empty root, all-zeros merged is 17 (K3); reading back all-zeros or readable with holes are both 6 (K4); not merged on the hole-free disk yields 18 (K5, same value across e, g, z); slots reading back all-zeros are pinned at 5 and 9 in the root ring table (K6). All 5 unit tests for the use case on the frozen tree passed in the copy (`e173-admission-ceiling-bad-ring-slot-dispositions-frozen-tree-use-case-2026-10-05.out`). K7 (structure) was verified item by item by the executor reading lines in the frozen tree copy: placement formula `root_ring.rs:109`, warm-up `mount.rs:3818`, sequence raising F `mount.rs:3362`, B = 8 `mount.rs:2792`, budget judgment `mount.rs:2984`, filtering merged bad slots >= F `mount.rs:2475`, `FloorAlreadyAtTheCeiling` judgment `mount.rs:2854`, all-zeros read back not entering capping table `mount.rs:1350` and `allocator.rs:1129` (taking only the `BadAtMount` class; all-zeros read-back class enters only the reclamation threshold of `allocator.rs:1092`), in-process re-reading `allocator.rs:1219`, identical item by item with the apparatus.
- **Segment One (supplementing Q1 row)**: Four Q1 arms (as-of-today, pushB, pushW, not merged) × H1–H4 × e / g × P1 / P2 / t1 / t2 × three pool types × S = 4 / 8 / 16 × c_max = 5 / 10 × two reserve pool interpretations, 4608 cells, 248832 positions, 258048 mounts, each cell having a position count equal to 3S − 1 (V2). Across persistent read failures (P1, P2) and near-full pools, all four arms experience extra rejections due to read failures and have positions where a single mount is entirely ENOSPC (`stable_hit`, maximum extra rejections 104 = entire request stream for S = 16); hits in this row mostly do not distinguish arms: P2 and t2 turn slots "read bad after written" into slots known by this process to house roots, raising F and reporting its own error (`rej_own_error` column), code-interpretation reserve pool (P_reserve = c) cannot accommodate an F-raising sequence (`rej_raise_no_placement` column), and the four arms yield identical or nearly identical readings in these cells (P2 near-full k = 0, clause interpretation: all four arms have 48 extra-rejected positions; P1, near-full k = 2, code interpretation: not merged 552, other three 528); cells that distinguish arms occur on clause-interpretation pools, e.g., P1, near-full k = 2: extra-rejected position count is 18 for as-of-today, 14 for pushB, 14 for pushW, 6 for not merged (total positions 1296). Only the not-merged arm causes F to cross non-empty roots residing in bad slots (hits on 6 out of 12 points for Q1-2, 0 for the other three arms); the two empty-pushing arms fail to acquire placement points on empty publishes in near-full pools and are rejected (hits on 10 points for Q1-3); on roomy pools, none of the four arms reject or cross, but within that set of pools, raising F and empty pushes are never exercised (F6: `name=judgment` with `no_hit_claim=path_not_exercised` in 48 rows, all in the extra-rejections row of Q1-5); readings indicate that arm dispositions were never exercised on roomy pools and cannot be construed as "a given arm produces no extra rejections on roomy pools". Position counts distinguishing arms are in `e173-admission-ceiling-bad-ring-slot-dispositions-arm-table-2026-10-05.out` (full row readings transcribed in the "Readings for Each Row: Segment One (Q1)" section).
- **Segment Two (supplementing Q2 row)**: z family (H1–H4 × P1 / P2 / t1 / t2) and H5 (young ring, crash leaving holes, holes genuinely all zeros) × two Q2 arms × four Q1 arms, 4896 cells, 124704 positions. On the Q1 not-merged arm, both Q2 arms are identical item by item at every position (F5 self-check: every quantity at every position is identical); on the other three arms, positions where all-zeros not merged causes F to cross more non-empty roots than all-zeros merged exist in the z family (on clause-interpretation pools, P1 near-full k = 0 has 18 positions crossing more, 6 positions losing more states), while positions where all-zeros merged crosses more or loses more are 0 across all cells in segment two (no slots read back all zeros in the disk layouts of segment one with e and g read failures, both Q2 arms being identical position by position); positions where all-zeros merged rejects more than all-zeros not merged exist in z family clause-interpretation near-full k = 2 with 4–6 positions (P1, P2, t2), and H5 clause-interpretation near-full pools (k = 0 and k = 2) each with 4 positions, where all-zeros merged never succeeded in issuing a single request after the first extra rejection.
- **User settlement** (2026-10-05, pop-up, verbatim words in same-date entry under D16 (Publish Semantics) section of `.claude/kb/decisions-history.md`): Q1 adopts as-of-today, bad slots are merged into the upper bound, reporting ENOSPC on this mount if blocked, neither pushing empty nor changing to not merged; Q2 adopts as-of-today not merged, all-zero slots are not merged into the upper bound, F may cross valid roots residing in all-zero slots. Aggregated across the arm table (`e173-admission-ceiling-bad-ring-slot-dispositions-arm-table-2026-10-05.out`, segment one has 15552 positions per arm per reserve pool interpretation, segment two has 7794 positions per group): under the clause interpretation, segment one extra-rejected positions are 270 for as-of-today, 266 for push-empty, 234 for not merged, with F crossing non-empty roots in bad slots occurring only for not merged in 180 positions; in segment two under as-of-today Q1 arm, all-zeros not merged / merged has extra rejections of 117 / 135, F crossing non-empty roots in all-zero slots of 90 / 0; under code interpretation, segment one extra rejections are 1656 / 1656 / 1728 (as-of-today / push-empty / not merged), segment two extra rejections are 864 / 828, crossing 0 / 0. After being reopened on , these two settlements were replaced by "push empty" for bad root slots and "merge" for root slots that read back all zeros (numbers and tables at lines 145–179 of `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-2026-11-02.md`).
- **Segment Three (incidental, not run after sufficient basis)**: The two knobs in reclamation threshold—bad slots pinned to maximum txg and three consecutive bad slots—were not run, labeled in registration Section 5 as "not run after sufficient basis".

Deterministic model: Re-running each of the three artifacts once (`checks` using 2 threads, segment one using 8 threads, segment two using 4 threads) was byte-for-byte identical to the initial artifacts, indicating that there is no hidden state and thread count does not affect output, rather than being merely statistically stable; the strength of judgment criteria relies on unit tests pinning absolute values, positive controls, and mutation tables.

### Rerun 1 (2026-11-02): the reserve pool computed from ckpt_cost

Rerun registration `research/prompts/e173-r1-prereg.md` (written 2026-10-20 before the apparatus; Section 12 revisions 1–20 were added after the apparatus and before the matching artifact, and only supplement sample points, the geometry constant table, anchor values, the S6 re-check, unit tests, mutations and the geometry-sensitivity reading, with the original criteria unchanged by a single character). Fork sheet `research/prompts/m3-close-b19-p5-forks.md` row 1 (C723 (three clause-versus-code reading differences in the bad-slot ceiling experiment)): A, the reserve-pool arm computed from ckpt_cost agrees with the clause reading / B, it differs from the clause. The two readings compared: the code reading has reserve pool P_code = κ (κ is the ckpt_cost computed by `checkpoint_cost_of_the_version_to_build_on` in the core layer, D28 (Mount-Time Commitments) Settled Item 4); the clause reading has P_clause = 10 + 7κ (2 × 5 + (B − 1) × c_max in the "acceptance" and "`df`" rows of D16 (Publish Semantics) Settled Item 1, with c_max taken as the same κ). The apparatus has two parts: segment one is the in-tree apparatus `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs`, which measures κ and the blocks newly taken by one empty publish, c_actual, on real disk layouts, with no arms; segment two is the mode `checkpoint-cost-reserve` added to the original apparatus (`scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs`), crossing both readings with 9 geometries (S = 4 / 8 / 16 × pool size G-min / G-1 / G-4). The original modes `checks`, `first` and `second` were not rerun: when the apparatus was written their outputs were checked byte-identical to the three old artifacts, and afterwards only the geometry constant table and unit tests changed.

**Current state** (segment two readings produced ):

- **Fork sheet row 1 is judged B: the code reading computed from ckpt_cost differs from the clause reading.** The criteria are the Z1 and Z3 thresholds of registration Section 6; the readings are under "Segment two readings" below. Z1 compares 7128 cell pairs one by one: 152 cells hit under the code reading only and 0 under the clause reading only (summary line 28382), in the rows Q1-3 (an empty-publish push finds no landing and is refused) and Q2-2 (merge-zero rejects or sticks more than not-merge-zero); the other rows such as Q1-1, Q1-1b and Q1-2 judge identically cell by cell under both readings. Z3 counts rejections with no read fault: 1 cell where the code reading rejects and the clause reading does not, 0 in the other direction (summary line 28411). Under the first judgment of registration Section 6 Z1, these readings lean towards "change the code"; on the user settled to change the clause to follow the code: the reserve pool in the "Acceptance" and "`df`" rows of D16 (Publish Semantics) settled item 1 is now computed on the spot from ckpt_cost (the user's words are in the same-day block of the D16 (Publish Semantics) section of the decisions history); the extra space the clause reading takes is in segment-one X4 below.
- **The difference varies with pool size and S; it is not a property of the reading** (registration Section 10 F7). Q1-3 hits at G-min and misses at G-4 under the code reading (for S = 4 / 8 / 16 alike), and under the clause reading only S = 16 at G-min hits; the Z1 differences occur at G-min for each of the three S values and at G-1 for S = 16, and are 0 at G-4 for every S and at G-1 for S = 4 and 8. The reverse acceptance "the difference is only on the smallest pool" at the end of registration Section 10 does not hold (S = 16 at G-1 is also non-zero), nor does the one "Z2 has no changed cell" (Z2 has 35 changed cells).
- **Segment one (fork sheet ③B) is sufficient to judge**: κ and c_actual were measured in all 9 geometries. In X3, n_code = ⌊κ ÷ c_actual⌋ is 3, 3, 4 on the three G-min geometries (below 7, not below 3) and at least 21 on the other six; n_clause is at least 22 in all nine. At S = 16 and G-1, n_code = 21 yet Q1-3 still hits under the code reading only, so "n_code < 7" is not a necessary and sufficient condition for the code reading to reject more (inferred: in that geometry one mount pushes 22 empty publishes in total, more than the 21 the reserve holds; see lines 27422 in the Q1-3 part of the segment two readings and 3460 in segment one).
- **Fork sheet ③C (Z2)**: comparing the old and new code readings, of 246 (row, Q1 arm, S) triples 33 changed in "hit at any sample point", all from the old code reading (reserve = c) not hitting to the new code reading (reserve = κ) hitting; on those 33 cells the clause reading hits both before and after, and the new code reading judges the same as the clause reading (one `awk` command counts 33 / 33 / 33 / 33, see the Z2 part). The clause-reading group changed in 2 cells (Q1-3, S = 8, push-B and push-W, hit before and miss now). The user decision on Q1 and Q2 (2026-10-05) rested on the clause-reading numbers, which changed in only 2 cells; the old code-reading numbers changed in 33 cells, and on the user settled to reopen it (this is exactly what the registration Section 6 Z2 text asks): bad root slots are settled as "push empty" and root slots that read back all zeros as "merge", and the two arm tables were re-accumulated under the new reading (reserve pool taken as ckpt_cost), with numbers and tables at lines 145–179 of `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-2026-11-02.md` (the four per-pair and summed tables under the new and old readings are `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-new-sums-2026-11-02.tsv`, `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-old-sums-2026-11-02.tsv`, `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-new-pairs-2026-11-02.tsv`, `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-tables-old-pairs-2026-11-02.tsv`) and in the same-day block of the D16 (Publish Semantics) section of the decisions history.
- **Discrimination power**: the control arm with the reserve reverted to 10 + 7c (c = c_actual), compared with the two arms by κ, differs in readings in 528 of 7128 cells against the code reading and 456 of 7128 against the clause reading (summary lines 28422, 28432), `reverted_reserve_readings_differ=yes`; by geometry, against the code reading S = 4 at G-1 and G-4 differs in 0 cells, and against the clause reading all three pools at S = 4 differ in 0 cells, so not every geometry can tell them apart.
- **The X5 sentence** (registration Section 6 X5, revision 19): on the four geometries `s8_g_min_64mib`, `s16_g_min_128mib`, `s16_g_1` and `s16_g_4`, every judgment of segment two in this part carries this sentence: the model's overwrite counts d + c_actual, 2 blocks fewer than real (segment-one artifact lines 3367, 3436, 3459, 3482, `excess_slots=2`). The last column of the geometry table below marks which ones.
- **Not run after sufficient basis**: X6 (ring window and switch reserve, marked "incidental, not run after sufficient basis" in registration Section 6).

Deterministic model: segment one and segment two both contain only deterministic publishes, which shows there is no hidden state, not statistical stability; strength comes from unit tests pinning absolute values, positive controls and the mutation table.

#### Segment one readings (copied verbatim from `research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out`, 3523 lines, last line `E7RESULT name=done emitted=3523`, first 16 hex digits of sha256 85bed2435162a532)

`name=void`, `name=stop`, `geometry_void` and NOT_RUN are all 0 lines in the artifact:

```
$ grep -c 'name=void\|name=stop\|geometry_void' research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out
0
$ grep -c NOT_RUN research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out
0
```

X3: how many actual empty publishes the two readings' reserve pools hold per geometry (`code_empty_publishes_held` is n_code, `clause_empty_publishes_held` is n_clause; `kappa=` is κ, `largest_actual_empty_publish_slots=` is the maximum c_actual; κ is the minimum over publishes built on versions with trees, and the maximum and the final value are the same in each geometry):

```
3299:E7RESULT name=x3_reserve_fit geometry=s4_g_min_64mib kappa_of_the_geometry=minimum_on_versions_with_trees kappa=18 largest_actual_empty_publish_slots=6 code_reserve=18 clause_reserve=136 code_empty_publishes_held=3 clause_empty_publishes_held=22
3322:E7RESULT name=x3_reserve_fit geometry=s4_g_1 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=130 largest_actual_empty_publish_slots=6 code_reserve=130 clause_reserve=920 code_empty_publishes_held=21 clause_empty_publishes_held=153
3345:E7RESULT name=x3_reserve_fit geometry=s4_g_4 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=543 largest_actual_empty_publish_slots=10 code_reserve=543 clause_reserve=3811 code_empty_publishes_held=54 clause_empty_publishes_held=381
3368:E7RESULT name=x3_reserve_fit geometry=s8_g_min_64mib kappa_of_the_geometry=minimum_on_versions_with_trees kappa=18 largest_actual_empty_publish_slots=6 code_reserve=18 clause_reserve=136 code_empty_publishes_held=3 clause_empty_publishes_held=22
3391:E7RESULT name=x3_reserve_fit geometry=s8_g_1 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=130 largest_actual_empty_publish_slots=6 code_reserve=130 clause_reserve=920 code_empty_publishes_held=21 clause_empty_publishes_held=153
3414:E7RESULT name=x3_reserve_fit geometry=s8_g_4 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=543 largest_actual_empty_publish_slots=10 code_reserve=543 clause_reserve=3811 code_empty_publishes_held=54 clause_empty_publishes_held=381
3437:E7RESULT name=x3_reserve_fit geometry=s16_g_min_128mib kappa_of_the_geometry=minimum_on_versions_with_trees kappa=26 largest_actual_empty_publish_slots=6 code_reserve=26 clause_reserve=192 code_empty_publishes_held=4 clause_empty_publishes_held=32
3460:E7RESULT name=x3_reserve_fit geometry=s16_g_1 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=130 largest_actual_empty_publish_slots=6 code_reserve=130 clause_reserve=920 code_empty_publishes_held=21 clause_empty_publishes_held=153
3483:E7RESULT name=x3_reserve_fit geometry=s16_g_4 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=543 largest_actual_empty_publish_slots=10 code_reserve=543 clause_reserve=3811 code_empty_publishes_held=54 clause_empty_publishes_held=381
```

X4 (reserve-pool share of the unit area, millionths are parts per million; the two disks are identical disk by disk, so the `device=0` line is copied; four lines are copied, S = 4 at G-min and G-4, S = 8 at G-min and S = 16 at G-min, and the other geometries' lines sit next to these four, listed in full by `grep -n 'name=x4_reserve_share'`):

```
3300:E7RESULT name=x4_reserve_share geometry=s4_g_min_64mib kappa_of_the_geometry=minimum_on_versions_with_trees device=0 unit_area_slots=2048 code_reserve_millionths=8789 clause_reserve_millionths=66406 difference_millionths=57617
3346:E7RESULT name=x4_reserve_share geometry=s4_g_4 kappa_of_the_geometry=minimum_on_versions_with_trees device=0 unit_area_slots=211968 code_reserve_millionths=2561 clause_reserve_millionths=17979 difference_millionths=15418
3369:E7RESULT name=x4_reserve_share geometry=s8_g_min_64mib kappa_of_the_geometry=minimum_on_versions_with_trees device=0 unit_area_slots=2048 code_reserve_millionths=8789 clause_reserve_millionths=66406 difference_millionths=57617
3438:E7RESULT name=x4_reserve_share geometry=s16_g_min_128mib kappa_of_the_geometry=minimum_on_versions_with_trees device=0 unit_area_slots=5120 code_reserve_millionths=5078 clause_reserve_millionths=37500 difference_millionths=32422
```

The four geometries where X5 exceeds the maximum c_actual (segment-two judgments on these four carry the X5 sentence):

```
3367:E7RESULT name=x5_overwrite_fixed_point geometry=s8_g_min_64mib count=288 max=8 min=6 last=8 exceeds_the_largest_actual_empty_publish=true excess_slots=2
3436:E7RESULT name=x5_overwrite_fixed_point geometry=s16_g_min_128mib count=576 max=8 min=6 last=8 exceeds_the_largest_actual_empty_publish=true excess_slots=2
3459:E7RESULT name=x5_overwrite_fixed_point geometry=s16_g_1 count=576 max=8 min=6 last=6 exceeds_the_largest_actual_empty_publish=true excess_slots=2
3482:E7RESULT name=x5_overwrite_fixed_point geometry=s16_g_4 count=576 max=12 min=8 last=10 exceeds_the_largest_actual_empty_publish=true excess_slots=2
```

The judgment is unstable between G-min and G-4 (the three `name=stability` lines with one true and one false):

```
3505:E7RESULT name=stability item=code_reserve_holds_the_empty_publishes_of_one_admission pair=s4_g_min_64mib~s4_g_4 left=false right=true stable=false
3509:E7RESULT name=stability item=code_reserve_holds_the_empty_publishes_of_one_admission pair=s8_g_min_64mib~s8_g_4 left=false right=true stable=false
3510:E7RESULT name=stability item=code_reserve_holds_the_empty_publishes_of_one_admission pair=s16_g_min_128mib~s16_g_4 left=false right=true stable=false
```

#### Segment two readings (copied verbatim from `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out`, 28437 lines, first 16 hex digits of sha256 db5aee5389d5e9f5, 31 threads)

The first two lines and the last line (configuration, the check of the geometry constant table against the segment-one artifact `mismatches=0`, the completion marker):

```
1:E7RESULT name=config segment=checkpoint-cost-reserve geometries=9 segment_one_product=research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out data_blocks_per_overwrite=8 admission_budget=8 requests_per_mount=2*3S+8 reserve_readings=ckpt-code:kappa,ckpt-clause:10+7kappa,reverted-10+7c:10+7c metadata_blocks=largest_actual_empty_publish_slots pools=near0,near2,roomy threads=31
2:E7RESULT name=s5_geometry_table mismatches=0 list=none
28437:E7RESULT name=done emitted=28437
```

Completeness (one command counts each kind of result line; the 21384 `name=cell` lines = 6912 + 7344 for the arms by κ, plus 3456 + 3672 for the control arm reverted to 10 + 7c):

```
$ awk '{print $2}' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out | sort | uniq -c | sort -rn
  21384 name=cell
   5508 name=q2_pair
    900 name=judgment
    207 name=z1_reading_split
    144 name=q2_comparison
     84 name=threshold_shift
     82 name=stability
     72 name=anchor
     27 name=z3_zero_point
     18 name=discrimination_reverted_reserve
      4 name=verdict
      2 name=discrimination_reverted_reserve_summary
      1 name=z3_summary
      1 name=z1_summary
      1 name=s5_geometry_table
      1 name=done
      1 name=config
```

Positive controls PC5 / PC6 and NOT_RUN (the 8 PC5 lines with `P=5` are `accepted=false` by design):

```
$ grep -c 'name=anchor id=PC5_.*matches=yes' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
24
$ grep -c 'name=anchor id=PC6_.*matches=yes' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
48
$ grep -c 'name=anchor.*matches=no' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
0
$ grep -c NOT_RUN research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
0
```

The 4 verdict lines (everything `grep -n 'name=verdict'` lists):

```
28433:E7RESULT name=verdict scope=ckpt-first cells=13824 positions=373248 runs=387072 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
28434:E7RESULT name=verdict scope=ckpt-second cells=7344 positions=187056 runs=194400 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
28435:E7RESULT name=verdict scope=reverted-first cells=6912 positions=186624 runs=193536 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
28436:E7RESULT name=verdict scope=reverted-second cells=3672 positions=93528 runs=97200 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
```

The boolean fields of the four verdict lines are all true, with `void_conditions=0` and `halt_conditions=0`, and no false or not_run field. The only `=false` in the whole artifact are the 8 PC5 lines (the expected refusals), and 0 lines elsewhere:

```
$ grep -c '=false' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
8
$ grep '=false' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out | grep -vc 'name=anchor id=PC5'
0
```

##### Z1, Q1-3 and Q2-2 (③A)

Z1 summary line (`only_code_hit_in_refusal_rows=152` is the number of cells hit under the code reading only; `near_full_code_cells_with_cap_active_and_a_non_empty_bad_slot=1944` is the path evidence: near-full cells where the cap is active and the bad slot holds a non-empty root):

```
28382:E7RESULT name=z1_summary cells_paired=7128 unpaired=0 only_code_hit_in_refusal_rows=152 only_code_hit_anywhere=152 only_clause_hit_anywhere=0 code_reading_rejects_or_sticks_more=yes readings_indistinguishable=no clause_reading_worse_on_some_row=no near_full_code_cells_with_cap_active_and_a_non_empty_bad_slot=1944
```

`name=z1_reading_split` has 207 lines (23 quantities × 9 geometries); the 7 lines where hits under the code reading only are non-zero (everything `grep -n 'name=z1_reading_split' <artifact> | grep -E 'only_code_hit=[1-9]|only_clause_hit=[1-9]'` lists; 0 lines with non-zero `only_clause_hit`, the 7 lines total 152):

```
28213:E7RESULT name=z1_reading_split id=Q1-3 S=4 pool_size=g_min_64mib cells=128 only_code_hit=48 only_clause_hit=0 both_hit=0 refusal_row=yes
28216:E7RESULT name=z1_reading_split id=Q1-3 S=8 pool_size=g_min_64mib cells=128 only_code_hit=48 only_clause_hit=0 both_hit=0 refusal_row=yes
28217:E7RESULT name=z1_reading_split id=Q1-3 S=16 pool_size=g_1 cells=128 only_code_hit=48 only_clause_hit=0 both_hit=0 refusal_row=yes
28351:E7RESULT name=z1_reading_split id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes S=8 pool_size=g_min_64mib cells=12 only_code_hit=2 only_clause_hit=0 both_hit=4 refusal_row=yes
28352:E7RESULT name=z1_reading_split id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes S=16 pool_size=g_1 cells=12 only_code_hit=2 only_clause_hit=0 both_hit=4 refusal_row=yes
28354:E7RESULT name=z1_reading_split id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes S=16 pool_size=g_min_128mib cells=12 only_code_hit=2 only_clause_hit=0 both_hit=4 refusal_row=yes
28357:E7RESULT name=z1_reading_split id=Q2-2-merged-all-enospc-beyond-not-merged@z_h1_to_h4 S=4 pool_size=g_min_64mib cells=192 only_code_hit=2 only_clause_hit=0 both_hit=7 refusal_row=yes
```

The six pairs of Q1-3 Y1 judgment lines that are one true and one false (code reading and clause reading by κ, the same (arm, S, pool size); `observed_maximum` is the maximum over bad-slot positions of the number of empty publishes refused in one mount, and one mount has 32 requests at S = 4, 56 at S = 8 and 104 at S = 16; each pair is the clause reading first and the code reading second, three geometries × push-B and push-W):

```
27288:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=4 reserve=ckpt-clause c_max=6 pool_size=g_min_64mib observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27291:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=4 reserve=ckpt-code c_max=6 pool_size=g_min_64mib observed_maximum=32 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27294:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=8 reserve=ckpt-clause c_max=6 pool_size=g_min_64mib observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27297:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=8 reserve=ckpt-code c_max=6 pool_size=g_min_64mib observed_maximum=56 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27299:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=16 reserve=ckpt-clause c_max=6 pool_size=g_1 observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27302:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB S=16 reserve=ckpt-code c_max=6 pool_size=g_1 observed_maximum=101 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27306:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=4 reserve=ckpt-clause c_max=6 pool_size=g_min_64mib observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27309:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=4 reserve=ckpt-code c_max=6 pool_size=g_min_64mib observed_maximum=32 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27312:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=8 reserve=ckpt-clause c_max=6 pool_size=g_min_64mib observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27315:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=8 reserve=ckpt-code c_max=6 pool_size=g_min_64mib observed_maximum=56 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27317:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=16 reserve=ckpt-clause c_max=6 pool_size=g_1 observed_maximum=0 cells_hit=0 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=no no_hit_claim=path_exercised
27320:E7RESULT name=judgment segment=ckpt-first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW S=16 reserve=ckpt-code c_max=6 pool_size=g_1 observed_maximum=104 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
```

The cumulative number of empty publishes pushed in one mount on S = 16 at G-1 for the same pair of readings (`Q1-3-empties-in-one-mount`, the clause reading 27419 first and the code reading 27422 second; the code-reading reserve of that geometry holds 21 on line 3460 of segment-one `name=x3_reserve_fit`):

```
27419:E7RESULT name=judgment segment=ckpt-first id=Q1-3-empties-in-one-mount scope=empty_pushing_arms_near_full arm=pushB S=16 reserve=ckpt-clause c_max=6 pool_size=g_1 observed_maximum=43 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
27422:E7RESULT name=judgment segment=ckpt-first id=Q1-3-empties-in-one-mount scope=empty_pushing_arms_near_full arm=pushB S=16 reserve=ckpt-code c_max=6 pool_size=g_1 observed_maximum=22 cells_hit=24 cells=64 cells_not_applicable=0 cells_with_path_evidence=24 hit=yes no_hit_claim=not_applicable
```

Geometry table (every number is a field of an artifact line: κ, c_actual, n_code and n_clause come from the segment-one `name=x3_reserve_fit` lines and the rest from `z1_reading_split`; the three Z1 columns are cells hit under the code reading only, with 128 cells per geometry for Q1-3, 12 cells for the Q2-2 holes (h5) and 192 cells for the Q2-2 z read faults; the Z3 column is cells where the zero point is rejected under the code reading but not the clause reading, summed over the three pools with 264 cells per pool; the last column, "overwrite under-counted by 2 blocks", marks where the X5 sentence applies):

| Geometry | κ | c_actual | n_code | n_clause | Z1 Q1-3 | Z1 Q2-2 holes | Z1 Q2-2 z | Z3 | Overwrite under-counted by 2 blocks |
|---|---|---|---|---|---|---|---|---|---|
| s4_g_min_64mib | 18 | 6 | 3 | 22 | 48 | 0 | 2 | 0 | no |
| s4_g_1 | 130 | 6 | 21 | 153 | 0 | 0 | 0 | 0 | no |
| s4_g_4 | 543 | 10 | 54 | 381 | 0 | 0 | 0 | 0 | no |
| s8_g_min_64mib | 18 | 6 | 3 | 22 | 48 | 2 | 0 | 1 | yes |
| s8_g_1 | 130 | 6 | 21 | 153 | 0 | 0 | 0 | 0 | no |
| s8_g_4 | 543 | 10 | 54 | 381 | 0 | 0 | 0 | 0 | no |
| s16_g_min_128mib | 26 | 6 | 4 | 32 | 0 | 2 | 0 | 0 | yes |
| s16_g_1 | 130 | 6 | 21 | 153 | 48 | 2 | 0 | 0 | yes |
| s16_g_4 | 543 | 10 | 54 | 381 | 0 | 0 | 0 | 0 | yes |

The two Q2-2 columns differ in amount, not in the direction of the judgment: of the 72 (scope, arm, S, pool size) aggregates, the code reading and the clause reading each have 36 with `hit_merged_all_enospc_beyond_not_merged=yes` (`name=q2_comparison` lines; the scopes are the h5 holes and z read faults); the difference is in "positions where merge-zero is all-ENOSPC beyond not-merge-zero", for example S = 4 at G-min with push-B: 6 positions under the code reading and 4 under the clause reading. The two lines copied in full (the clause reading 28058 first, the code reading 28061 second):

```
28058:E7RESULT name=q2_comparison segment=ckpt-second scope=z_h1_to_h4 arm=pushB S=4 reserve=ckpt-clause c_max=6 pool_size=g_min_64mib cells=48 cells_where_not_merged_has_more=15 cells_where_merged_has_more=0 largest_crossed_excess_of_not_merged=1 largest_lost_excess_of_not_merged=1 crossed_maximum_not_merged=1 crossed_maximum_merged=0 lost_maximum_not_merged=1 lost_maximum_merged=0 all_enospc_positions_not_merged=27 all_enospc_positions_merged=31 extra_rejected_maximum_not_merged=32 extra_rejected_maximum_merged=32 positions_merged_is_all_enospc_against_not_merged=4 positions_merged_rejects_more=4 largest_rejected_excess_of_merged=1 lowest_candidates_not_merged=1 lowest_candidates_merged=1 lowest_final_candidates_not_merged=1 lowest_final_candidates_merged=1 hit_not_merged_has_more=yes hit_merged_has_more=no hit_merged_all_enospc=yes hit_merged_all_enospc_beyond_not_merged=yes
28061:E7RESULT name=q2_comparison segment=ckpt-second scope=z_h1_to_h4 arm=pushB S=4 reserve=ckpt-code c_max=6 pool_size=g_min_64mib cells=48 cells_where_not_merged_has_more=15 cells_where_merged_has_more=0 largest_crossed_excess_of_not_merged=1 largest_lost_excess_of_not_merged=1 crossed_maximum_not_merged=1 crossed_maximum_merged=0 lost_maximum_not_merged=1 lost_maximum_merged=0 all_enospc_positions_not_merged=27 all_enospc_positions_merged=33 extra_rejected_maximum_not_merged=32 extra_rejected_maximum_merged=32 positions_merged_is_all_enospc_against_not_merged=6 positions_merged_rejects_more=6 largest_rejected_excess_of_merged=1 lowest_candidates_not_merged=1 lowest_candidates_merged=1 lowest_final_candidates_not_merged=1 lowest_final_candidates_merged=1 hit_not_merged_has_more=yes hit_merged_has_more=no hit_merged_all_enospc=yes hit_merged_all_enospc_beyond_not_merged=yes
```

Path evidence for misses (F5): of the `hit=no` judgment lines, 332 are `path_exercised` and 72 are `path_not_exercised`; the latter are all on the roomy pool in the extra-rejection row of Q1-5 (floor raising and empty pushing were never reached on the roomy pool), and those 72 lines must not be written as "a reading does not reject more on the roomy pool".

```
$ grep 'name=judgment' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out | grep -o 'hit=[a-z]* no_hit_claim=[a-z_]*' | sort | uniq -c
    332 hit=no no_hit_claim=path_exercised
     72 hit=no no_hit_claim=path_not_exercised
    496 hit=yes no_hit_claim=not_applicable
$ grep 'name=judgment' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out | grep 'no_hit_claim=path_not_exercised' | grep -o 'id=[^ ]* scope=[^ ]*' | sort | uniq -c
     72 id=Q1-5-extra-rejected scope=roomy
```

##### Z3 (③A)

Rejections with no read fault, the two readings compared: of the 27 `name=z3_zero_point` lines (3 S × 3 pool sizes × 3 pools) the only one with a non-zero count where the code reading rejects and the clause reading does not is S = 8, G-min, near-full k = 2 (line 28400; that geometry carries the X5 sentence: the model's overwrite counts d + c_actual, 2 blocks fewer than real); the other 26 lines are 0 in both columns. Summary line 28411:

```
28400:E7RESULT name=z3_zero_point S=8 pool_size=g_min_64mib pool=near2 cells=264 code_rejects_clause_does_not=1 clause_rejects_code_does_not=0 both_reject=130 code_first_rejected_request=2 clause_first_rejected_request=2
28411:E7RESULT name=z3_summary code_rejects_more_without_a_read_fault=yes clause_rejects_more_without_a_read_fault=no cells_code_only=1 cells_clause_only=0
```

Discrimination self-check (the threshold moved to the observed maximum + 1 flips from true to false; one line each for Z1 and Z3, `flips=yes`):

```
28383:E7RESULT name=threshold_shift segment=checkpoint-cost-reserve id=Z1-code-rejects-or-sticks-more arm=all_geometries observed_maximum=48 hit_at_threshold_1=yes shifted_threshold=49 hit_at_shifted_threshold=no flips=yes
28412:E7RESULT name=threshold_shift segment=checkpoint-cost-reserve id=Z3-code-rejects-more-at-the-zero-point arm=all_geometries observed_maximum=1 hit_at_threshold_1=yes shifted_threshold=2 hit_at_shifted_threshold=no flips=yes
```

##### Control arm with the reserve reverted to 10 + 7c (discrimination power, registration revision 10)

Each line is a (against what, S, pool size), the number of the 792 cells whose readings differ and the number whose verdicts differ; the two `_summary` lines are the totals. Against the code reading S = 4 at G-1 and G-4 is 0 (the values 130 and 543 of P = κ and the 52 and 80 of 10 + 7c give the same readings on those cells), and elsewhere it is non-zero:

```
28413:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=4 pool_size=g_1 cells=792 cells_with_differing_readings=0 cells_with_a_differing_verdict=0
28414:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=4 pool_size=g_4 cells=792 cells_with_differing_readings=0 cells_with_a_differing_verdict=0
28415:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=4 pool_size=g_min_64mib cells=792 cells_with_differing_readings=72 cells_with_a_differing_verdict=50
28416:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=8 pool_size=g_1 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=50
28417:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=8 pool_size=g_4 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=49
28418:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=8 pool_size=g_min_64mib cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=0
28419:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=16 pool_size=g_1 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=0
28420:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=16 pool_size=g_4 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=48
28421:E7RESULT name=discrimination_reverted_reserve against=ckpt-code S=16 pool_size=g_min_128mib cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=0
28422:E7RESULT name=discrimination_reverted_reserve_summary against=ckpt-code cells=7128 cells_with_differing_readings=528 reverted_reserve_readings_differ=yes
28423:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=4 pool_size=g_1 cells=792 cells_with_differing_readings=0 cells_with_a_differing_verdict=0
28424:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=4 pool_size=g_4 cells=792 cells_with_differing_readings=0 cells_with_a_differing_verdict=0
28425:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=4 pool_size=g_min_64mib cells=792 cells_with_differing_readings=0 cells_with_a_differing_verdict=0
28426:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=8 pool_size=g_1 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=50
28427:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=8 pool_size=g_4 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=49
28428:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=8 pool_size=g_min_64mib cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=50
28429:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=16 pool_size=g_1 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=50
28430:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=16 pool_size=g_4 cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=48
28431:E7RESULT name=discrimination_reverted_reserve against=ckpt-clause S=16 pool_size=g_min_128mib cells=792 cells_with_differing_readings=76 cells_with_a_differing_verdict=2
28432:E7RESULT name=discrimination_reverted_reserve_summary against=ckpt-clause cells=7128 cells_with_differing_readings=456 reverted_reserve_readings_differ=yes
```

##### Z2 (③C)

The Z2 script `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-z2-2026-10-29.py` reads the segment-two artifact and the two old artifacts already archived out of the working tree (read by `git show` from the parent of commit 59ce5a865; the old code reading is `reserve=code`, the old clause reading `reserve=clause`) and writes `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-z2-2026-11-02.out` (495 lines, first 16 hex digits of sha256 b10a66a0f9b23132). The three summary lines:

```
493:Z2_summary old=code new=ckpt-code compared=246 changed=33 verdict=changed
494:Z2_summary old=clause new=ckpt-clause compared=246 changed=2 verdict=changed
495:Z2_done unpaired=0
```

The changed cells by quantity (command and verbatim output):

```
$ grep 'changed=yes' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-z2-2026-11-02.out | awk '{print $2, $3, $4}' | sort | uniq -c
      2 old=clause new=ckpt-clause id=Q1-3
      3 old=code new=ckpt-code id=Q1-2
      3 old=code new=ckpt-code id=Q1-2b
      3 old=code new=ckpt-code id=Q1-4-crossed
      9 old=code new=ckpt-code id=Q2-1-not-merged-has-more@z_h1_to_h4
      6 old=code new=ckpt-code id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes
      9 old=code new=ckpt-code id=Q2-2-merged-all-enospc-beyond-not-merged@z_h1_to_h4
```

On the 33 cells that changed under the code reading, the old code reading misses and the new one hits; the clause reading hits on the same cells both before and after, so the new code reading judges the same as the clause reading (first count the changed cells, then among them those where the clause reading hit before, those where it hits now, and those where the new code reading and the new clause reading judge alike):

```
$ awk '/^Z2 old=/{for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} k=v["id"]" "v["arm"]" "v["S"]; if(v["old"]=="code"){cn[k]=v["new_any"];cc[k]=v["changed"]} else {kn[k]=v["new_any"];ko[k]=v["old_any"]}} END{for(k in cc) if(cc[k]=="yes"){n++; if(ko[k]=="yes")o++; if(kn[k]=="yes")m++; if(cn[k]==kn[k])s++} print n, o, m, s}' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-z2-2026-11-02.out
33 33 33 33
```

The 2 cells that changed under the clause reading, copied in full:

```
297:Z2 old=clause new=ckpt-clause id=Q1-3 arm=pushB S=8 old_points=5:yes,10:yes new_points=g_1:no,g_4:no,g_min_64mib:no old_any=yes new_any=no changed=yes
300:Z2 old=clause new=ckpt-clause id=Q1-3 arm=pushW S=8 old_points=5:yes,10:yes new_points=g_1:no,g_4:no,g_min_64mib:no old_any=yes new_any=no changed=yes
```

##### F7 geometry sensitivity (registration Section 8, Section 10 F7)

The F7 script `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-f7-2026-11-02.py` reads only the segment-two artifact and compares, per (reading, row, arm), (S, G-min) with (S, G-4) for the three S values and (S = 4, pool) with (S = 16, pool) for the three pool sizes, recording one true and one false as `unstable=yes`; its output is `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-f7-2026-11-02.out` (989 lines, first 16 hex digits of sha256 59612197f503187a). It was first self-checked with an 18-line synthetic judgment file in the draft directory (the code reading hits only at G-min with S = 4): the output was identical, line for line, to the expected five lines; the synthetic file is in the draft directory and is not checked in. Summary:

```
985:F7_summary reading=ckpt-code axis=pool_size_g_min_against_g_4 compared=246 unstable=15
986:F7_summary reading=ckpt-code axis=slots_4_against_16 compared=246 unstable=11
987:F7_summary reading=ckpt-clause axis=pool_size_g_min_against_g_4 compared=246 unstable=11
988:F7_summary reading=ckpt-clause axis=slots_4_against_16 compared=246 unstable=11
989:F7_done rows=164 unpaired=0
```

The 48 unstable lines split by (reading, row) (command and verbatim output). Only Q1-3 and the Q2-2 row of merge-zero all-ENOSPC beyond not-merge-zero are unstable; other rows such as Q1-1, Q1-1b and Q1-2 are stable on both knobs:

```
$ grep 'unstable=yes' research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-f7-2026-11-02.out | awk '{print $2, $3, $4}' | sed 's/arm=[a-zA-Z]*//' | sort | uniq -c
      4 reading=ckpt-clause id=Q1-3 
      9 reading=ckpt-clause id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes 
      9 reading=ckpt-clause id=Q2-2-merged-all-enospc-beyond-not-merged@z_h1_to_h4 
      8 reading=ckpt-code id=Q1-3 
      9 reading=ckpt-code id=Q2-2-merged-all-enospc-beyond-not-merged@h5_holes 
      9 reading=ckpt-code id=Q2-2-merged-all-enospc-beyond-not-merged@z_h1_to_h4 
```

#### Rerun, unit tests and mutations

```
# repo root; segment one is the in-tree apparatus (the main agent's; the registered rerun is driver_e173_checkpoint_cost)
bash scripts/experiment-apparatus/replay.sh E173
# segment two (mode checkpoint-cost-reserve, output redirected to a new file name)
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e173-admission-ceiling-bad-ring-slot-dispositions -- checkpoint-cost-reserve
# Z2 and F7 (read only the segment-two artifact)
python3 -I research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-z2-2026-10-29.py research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
python3 -I research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-f7-2026-11-02.py research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out
# segment two: 30 unit tests and a 33-row mutation table
cd scripts/experiment-apparatus
../../task-server/task submit -- cargo test --release --bin e173-admission-ceiling-bad-ring-slot-dispositions
../../task-server/task submit -- bash ../mutation/mutate.sh e173-admission-ceiling-bad-ring-slot-dispositions e173_admission_ceiling_bad_ring_slot_dispositions.rs ../mutation/tables/e173_admission_ceiling_bad_ring_slot_dispositions.tsv
```

The segment-two registration line is `E173|e173-admission-ceiling-bad-ring-slot-dispositions|checkpoint-cost-reserve|…|exact` (`scripts/experiment-apparatus/replay.sh`); each artifact's completion marker is `E7RESULT name=done emitted=N`, with N equal to the file's line count. Segment two reads the segment-one artifact (the path hard-coded in `SEGMENT_ONE_PRODUCT`); if that artifact is later archived out of the working tree, the segment-two check stops at `segment_one_product_unreadable`, so keep it at archive time or change the apparatus to read through `git show`. The artifact `…-checkpoint-cost-reserve-2026-11-02.out` is 32248184 bytes. One part of the exact comparison on rerun depends on the thread count: `threads=` in the config line takes the registered per-host default heavy thread count (31).

Segment two has 30 unit tests passing (28 in the apparatus file and 2 more heavy-thread-count cases from `include!`; `task-server/task log 4897 --all | grep -c '^test .* ok$'` counts 30, ending with `test result: ok. 30 passed; 0 failed`); segment one has 15 unit tests passing (in-tree apparatus, main agent task 4704). The segment-two mutation table `scripts/mutation/tables/e173_admission_ceiling_bad_ring_slot_dispositions.tsv` has 33 rows (the original 23 + MR9–MR15 + MR16–MR18), and `mutate.sh` ends with `计数：抓到 33 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 33 条、复用 0 条、拒算 0 条` and `已还原，基线仍全绿` (task 4895; the Chinese summary lines are the tool's own wording: caught 33, not red 0, invalid 0, unfinished 0, name blind spots 0, broken anchors 0, judged now 33, reuse 0, refused 0; restored, baseline still all green). The new mutations: MR9 (the code reading's P takes c instead of κ), MR10 (the clause reading's 10 + 7κ changed to 10 + 6κ), MR11 (the new mode's c taken as κ), MR12 (Z1's two directions swapped), MR13 (Z3 compares the read-fault run instead of the zero point), MR14 (the geometry constant table is not checked against the segment-one artifact before the run), MR15 (the code reading's reserve reverted to 10 + 7c), MR16 (a κ of G-4 copied wrong by one), MR17 (G-4's B9 expectation replaced by another pool's numbers), MR18 (free minus P in the B9 expectation written as 61). Segment one's MR1–MR8 are in `crates/mutations.tsv`, with the three counts pending the commit-time gate crates-mutation-replay.

### What Was Done

- **Model**: Two-disk mirror, root ring R = 3 regions (ownership 0 / 1 / 0) × S slots per region, placement txg mod 3 and (txg ÷ 3) mod S; accounting unit is block, overwrite publish newly allocates and releases d + c blocks (d = 8, c = c_max = 5 or 10), empty publish newly allocates and releases c blocks, released blocks carry release gen = publish txg, each recorded root reclaims per threshold max(F_effective, oldest valid root in ring), deductions from an F-raising sequence are credited when that sequence completes; root ring table follows frozen tree classified into "valid root, read bad at mount (carrying oldest possible txg and maximum possible txg), read back all zeros at mount"; in-process root recording re-reads pinned slots first; computing upper bound reads once, and slots known to house roots reporting bad on re-read trigger F-raising's own error; single acceptance allows at most B = 8 publishes; user publish requires free − P_reserve ≥ d + c, empty publish and F-raising sequence require only free ≥ c.
- **Arms**: Q1 four arms (as-of-today, pushB, pushW, not merged), Q2 two arms (all-zeros not merged, all-zeros merged). PushB pushes one empty publish without raising F when the upper bound is blocked on a bad slot (C_merged ≤ F while C0 > F), counting toward pushed count; pushW pushes until unblocked, push-empty not constrained by B, at most 3S times; not merged takes C0.
- **Histories and Read Faults**: H1 crash, H2 raised F, H3 normal unmount, H4 alternating overwrite and empty publish, H5 young ring crash leaving holes (Q2 only); read failures three types e (device error), g (readable but fails self-witnessing), z (successful read returning all zeros), persistence four types P1 (recovered once overwritten), P2 (persists even after written), t1 (transient in mount), t2 (persists until first F-raising sequence finishes); pool three types near-full k = 0, k = 2, roomy, reserve pool two interpretations (code interpretation c, clause interpretation 10 + 7c); S = 4 / 8 / 16, c_max = 5 / 10; each bad slot position j = 1..3S − 1 runs one mount, each mount executing J = 2 × 3S + 8 overwrite requests.
- **Controls and Zero Baseline**: Each cell runs an additional mount with read faults removed as zero baseline (H1–H4 zero baselines across four Q1 arms and two Q2 arms are identical item by item, verified by apparatus); extra rejections due to read failure = difference between rejection counts of the two runs; count of non-empty roots crossed by F, states lost, and rollback candidates computed against ground-truth disk layout (no read faults, each slot read according to genuinely housed content).
- **Revisions**: 11 items in Section 12, highlights: PC1, PC3 pools changed from near-full k = 8 to near-full k = 0 exercising F-raising on first request (k = 8 pool never exercises F-raising across J requests, readings written in artifacts); M5 sample point replaced with a cell identified in unit tests (sample point in registration gave identical readings for two Bs in arithmetic of H1–H4 first request); M6, M9 mutations written to match genuine implementation expressions (M6a, M6b, M9); added mutations M13–M15 and Mm1–Mm7; Q2-2 added position-level readings side by side with all-zeros not merged for H5; PC4 "forcing one round insufficient" taken as treating every iteration in first request as insufficient.

### Replay

```
# 仓根；装置在 scripts/experiment-apparatus/（不依赖 crates/）
# 登记的复跑（scripts/experiment-apparatus/replay.sh 三行：checks、first、second）；没经 replay.sh 跑，三份产物是手工跑同一条命令生成的
bash scripts/experiment-apparatus/replay.sh E173
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e173-admission-ceiling-bad-ring-slot-dispositions -- checks
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e173-admission-ceiling-bad-ring-slot-dispositions -- first
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e173-admission-ceiling-bad-ring-slot-dispositions -- second
# 单测 17 条与变异表 23 条
cd scripts/experiment-apparatus
../../task-server/task submit -- cargo test --release -p e7-index-bench --bin e173-admission-ceiling-bad-ring-slot-dispositions
../../task-server/task submit -- bash ../mutation/mutate.sh e173-admission-ceiling-bad-ring-slot-dispositions e173_admission_ceiling_bad_ring_slot_dispositions.rs ../mutation/tables/e173_admission_ceiling_bad_ring_slot_dispositions.tsv
# 分臂位置数表（只读两段产物）
cd ../..
python3 e173-admission-ceiling-bad-ring-slot-dispositions-arm-table-2026-10-05.py
```

Measured memory packaging peaks: `checks` 10M; segment one and segment two peaks are recorded in corresponding keys in `~/.local/share/singlefs/memory-peaks.tsv`. Segment one takes approximately 1.5 minutes, segment two approximately 1 minute (15 threads); thread count affects wall-clock only, not output. Artifacts: `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out` (285 lines), `…-segment-one-2026-10-05.out` (5311 lines, 4608 cells), `…-segment-two-2026-10-05.out` (7507 lines, 4896 cells), `…-frozen-tree-use-case-2026-10-05.out`, `…-unit-tests-2026-10-05.out`, `…-mutate-2026-10-05.log`, `…-arm-table-2026-10-05.out`; completion marker for each artifact is `E7RESULT name=done emitted=N`, where N equals the file line count.

### Verdicts (copied verbatim from artifacts)

All verdict lines across three artifacts (`grep -n 'name=verdict' <artifact>` listed completely):

```
research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out:284:E7RESULT name=verdict scope=checks self_checks_passed=106 void_conditions=0 void_list=none halt_conditions=0 halt_list=none
research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out:5310:E7RESULT name=verdict scope=first cells=9216 positions=248832 runs=258048 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out:7506:E7RESULT name=verdict scope=second cells=4896 positions=124704 runs=129600 positions_match_expected=true publishes_within_budget_where_the_arm_keeps_it=true q2_arms_agree_where_they_must=true baselines_agree_across_arms=true void_conditions=0 void_list=none halt_conditions=0
```

3 verdict lines, no false or not_run fields; integer counts for voiding and halting clauses are all 0. `cells=` counts cells × two Q2 arms (segment one emits only 4608 lines of `name=cell` for all-zeros not merged arm, the other arm verifying identity position by position).

### Anchors and Positive Controls (copied verbatim from research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out)

`grep -c 'name=anchor'` 114 lines, `grep -c 'matches=yes'` 106 lines, `grep -c 'matches=no'` 0 lines; remaining 8 lines report F-raising counts for registered k = 8 cell of PC1, PC3 (no expected value, reporting readings only).

```
2:E7RESULT name=anchor id=K1 description=有洞那一份 X 读不出：上限 6、F 抬到 6 q1=today q2=zeronot c0=6 c_merged=6 floor_after=6 crossed=0 pushed=yes stop=none expected_floor=6 expected_crossed=none slot_entry=bad_at_mount_oldest_9_highest_21 matches=yes
14:E7RESULT name=anchor id=K2_e description=没洞那一份 X 读不出：上限 17 q1=today q2=zeronot c0=18 c_merged=17 floor_after=17 crossed=0 pushed=yes stop=none expected_floor=17 expected_crossed=none slot_entry=bad_at_mount_oldest_5_highest_17 matches=yes
32:E7RESULT name=anchor id=K3 description=没洞那一份 X 读回全零、全零不并：上限 18、越过 X（越过数 1） q1=today q2=zeronot c0=18 c_merged=18 floor_after=18 crossed=1 pushed=yes stop=none expected_floor=18 expected_crossed=1 slot_entry=all_zero_oldest_5_highest_17 matches=yes
35:E7RESULT name=anchor id=K3 description=没洞那一份 X 读回全零、全零并：上限 17 q1=today q2=zeromerge c0=18 c_merged=17 floor_after=17 crossed=0 pushed=yes stop=none expected_floor=17 expected_crossed=0 slot_entry=all_zero_oldest_5_highest_17 matches=yes
110:E7RESULT name=anchor id=K5_z description=没洞那一份不并：上限 18 q1=nomerge q2=zeronot c0=18 c_merged=18 floor_after=18 crossed=1 pushed=yes stop=none expected_floor=18 expected_crossed=none slot_entry=all_zero_oldest_5_highest_17 matches=yes
116:E7RESULT name=anchor id=K6_no_holes slot_entry=all_zero_oldest_5_highest_17 expected_oldest=5 expected_highest=none matches=yes
117:E7RESULT name=anchor id=K6_holes slot_entry=all_zero_oldest_9_highest_21 expected_oldest=9 expected_highest=none matches=yes
120:E7RESULT name=anchor id=A1 observed=8 expected=8 matches=yes
123:E7RESULT name=anchor id=A3_raise_sequence_by_txg_mod_3 observed=[2, 3, 2] expected=[2, 3, 2] matches=yes
127:E7RESULT name=anchor id=B2_warm_up_by_row_mod_3 observed=[1, 1, 2] expected=[1, 1, 2] matches=yes
132:E7RESULT name=anchor id=B5_mount_facts_today_zeronot observed=N=50 row=51 warmup=1 ring=[41,52] pending_total=127 expected=N=50 row=51 warmup=1 ring=[41,52] pending_total=127 matches=yes
133:E7RESULT name=anchor id=B5_first_request_today_zeronot observed=accepted=true raises=1 ceiling=47 sequence_publishes=3 floor=47 lowest_free_in_the_raise=Some(30) free_above_reserve_after=50 pending_after=77 txg_after=56 expected=accepted=true raises=1 ceiling=47 sequence_publishes=3 floor=47 lowest_free_in_the_raise=Some(30) free_above_reserve_after=50 pending_after=77 txg_after=56 matches=yes
172:E7RESULT name=anchor id=B6_roomy_pool_no_fault_never_rejects observed=cells=384 cells_with_a_rejection=0 expected=cells=384 cells_with_a_rejection=0 matches=yes
173:E7RESULT name=anchor id=PC1_PC3_registered_k8_pool_floor_raises arm=today_zeronot observed=0 note=registered_pool_does_not_reach_the_raise_path
174:E7RESULT name=anchor id=PC1_crossed_by_two_today_zeronot observed=2 expected=2 matches=yes
175:E7RESULT name=anchor id=PC3_states_lost_today_zeronot observed=3 expected=3 matches=yes
176:E7RESULT name=anchor id=PC2_rejected_today_zeronot observed=30 expected=30 matches=yes
177:E7RESULT name=anchor id=PC4_empty_publishes_today_zeronot observed=1 5 expected=1 5 matches=yes
189:E7RESULT name=anchor id=PC4_empty_publishes_pushB_zeronot observed=5 25 expected=5 25 matches=yes
281:E7RESULT name=anchor id=B3_F4_empty_pushes_S4 observed=positions=8 beyond_budget_mismatches=[] within_budget_mismatches=[] expected=positions=8 beyond_budget_mismatches=[] within_budget_mismatches=[] matches=yes
283:E7RESULT name=anchor id=B3_F4_empty_pushes_S16 observed=positions=44 beyond_budget_mismatches=[] within_budget_mismatches=[] expected=positions=44 beyond_budget_mismatches=[] within_budget_mismatches=[] matches=yes
```

Similar lines for other arms reside in the same artifact (PC1–PC4 4 lines per arm, readings across all eight arms equal expected values; B5 2 lines per arm). The trajectory of B5 first request (one line per step) is in `name=trace id=B5` lines in the same artifact. Mutual comparisons (identical across arms item by item after removing read faults, identical across two Q2 arms under Q1 not-merged) are accompanied by pinned absolute values: B5's 127, 77, 50; B6's 384 cells with 0 rejections; A1's 8; PC's 2, 3, 30, 5 25; F1 (any arm unable to read planted effect) was not triggered.

### Readings for Each Row: Segment One (Q1)

Thresholds are uniformly "this quantity ≥ 1 on a given cell", judged across twelve points (arm, S, reserve pool interpretation, c_max); twelve points sharing true or false are recorded as `stable_hit` / `stable_no_hit`, mixed true and false recorded as `unstable` (Registration Section 8 "Notation for Unstable Judgments"). Lines copied verbatim from `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out` (line numbers at head; `name=judgment` lines for each point total 576 lines, omitted, showing stability lines for the same group of points):

```
4658:name=stability segment=first id=Q1-1 scope=persistent_near_full arm=nomerge hit_points=12 points=12 observed_maximum=104 stability=stable_hit
4659:name=stability segment=first id=Q1-1 scope=persistent_near_full arm=pushB hit_points=12 points=12 observed_maximum=104 stability=stable_hit
4660:name=stability segment=first id=Q1-1 scope=persistent_near_full arm=pushW hit_points=12 points=12 observed_maximum=104 stability=stable_hit
4661:name=stability segment=first id=Q1-1 scope=persistent_near_full arm=today hit_points=12 points=12 observed_maximum=104 stability=stable_hit
4714:name=stability segment=first id=Q1-1b scope=persistent_near_full arm=nomerge hit_points=12 points=12 observed_maximum=45 stability=stable_hit
4715:name=stability segment=first id=Q1-1b scope=persistent_near_full arm=pushB hit_points=12 points=12 observed_maximum=44 stability=stable_hit
4716:name=stability segment=first id=Q1-1b scope=persistent_near_full arm=pushW hit_points=12 points=12 observed_maximum=44 stability=stable_hit
4717:name=stability segment=first id=Q1-1b scope=persistent_near_full arm=today hit_points=12 points=12 observed_maximum=44 stability=stable_hit
4770:name=stability segment=first id=Q1-2 scope=every_persistence_every_pool arm=nomerge hit_points=6 points=12 observed_maximum=1 stability=unstable
4771:name=stability segment=first id=Q1-2 scope=every_persistence_every_pool arm=pushB hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4772:name=stability segment=first id=Q1-2 scope=every_persistence_every_pool arm=pushW hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4773:name=stability segment=first id=Q1-2 scope=every_persistence_every_pool arm=today hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4826:name=stability segment=first id=Q1-2b scope=every_persistence_every_pool arm=nomerge hit_points=6 points=12 observed_maximum=1 stability=unstable
4827:name=stability segment=first id=Q1-2b scope=every_persistence_every_pool arm=pushB hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4828:name=stability segment=first id=Q1-2b scope=every_persistence_every_pool arm=pushW hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4829:name=stability segment=first id=Q1-2b scope=every_persistence_every_pool arm=today hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
4858:name=stability segment=first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushB hit_points=10 points=12 observed_maximum=104 stability=unstable
4859:name=stability segment=first id=Q1-3 scope=empty_pushing_arms_near_full arm=pushW hit_points=10 points=12 observed_maximum=104 stability=unstable
4914:name=stability segment=first id=Q1-3-empties-in-one-admission scope=empty_pushing_arms_near_full arm=pushB hit_points=12 points=12 observed_maximum=7 stability=stable_hit
4915:name=stability segment=first id=Q1-3-empties-in-one-admission scope=empty_pushing_arms_near_full arm=pushW hit_points=12 points=12 observed_maximum=9 stability=stable_hit
4970:name=stability segment=first id=Q1-3-empty-blocks-in-one-mount scope=empty_pushing_arms_near_full arm=pushB hit_points=12 points=12 observed_maximum=80 stability=stable_hit
4971:name=stability segment=first id=Q1-3-empty-blocks-in-one-mount scope=empty_pushing_arms_near_full arm=pushW hit_points=12 points=12 observed_maximum=80 stability=stable_hit
5022:name=stability segment=first id=Q1-4-extra-rejected scope=transient_near_full arm=nomerge hit_points=12 points=12 observed_maximum=104 stability=stable_hit
5078:name=stability segment=first id=Q1-4-crossed scope=transient_every_pool arm=nomerge hit_points=6 points=12 observed_maximum=1 stability=unstable
5134:name=stability segment=first id=Q1-5-extra-rejected scope=roomy arm=nomerge hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
5190:name=stability segment=first id=Q1-5-crossed scope=roomy arm=nomerge hit_points=0 points=12 observed_maximum=0 stability=stable_no_hit
```

Lines for the other three arms in Q1-4 and Q1-5 are isomorphic (extra rejections due to read failure in Q1-4 all four arms `stable_hit`, max 104; crossing occurs only in not merged for 6 / 12 points; Q1-5 roomy pools all four arms `stable_no_hit`). Q1-6 (capping taking effect) is in `name=stability ... id=Q1-6-cap-active` and `id=Q1-6-cap-blocked` lines: not merged arm 0, other three arms all hit twelve points (pushB max 113). Discriminative power self-test (threshold shifted from 1 to observed max + 1) in `name=threshold_shift` lines has 50 lines in segment one: 29 lines `flips=yes`, 21 lines `flips=not_applicable_observed_below_1` (observed max is 0, inherently false), no `flips=no`. Whether path was exercised (F6) resides in `no_hit_claim` field of `name=judgment` line for each point: `grep -c 'no_hit_claim=path_exercised'` 226 lines, `grep -c 'no_hit_claim=path_not_exercised'` 48 lines; all 48 lines are in the extra-rejections row of Q1-5 (roomy pools, four arms twelve points each, `cells_with_path_evidence=0`: F-raising count, capping block count, empty-push count are all 0 in that set of cells), written as "path not exercised" per F6; all other unhit points had their paths exercised.

Position counts distinguishing arms (counted by script `e173-admission-ceiling-bad-ring-slot-dispositions-arm-table-2026-10-05.py` from artifacts, output verbatim in `…-arm-table-2026-10-05.out`; total positions 1296 = sum across four history families × e / g × c_max two tiers × 3S − 1 for each S; columns in order are extra rejections, single mount entirely ENOSPC, non-empty roots crossed, F higher than truth ceiling, states lost, empty push, empty push failing to acquire placement, capping blocked, five rejection causes: budget exhausted / upper bound not above F / F-raising sequence failing to acquire placement / empty push failing to acquire placement / F-raising reporting own error, each column counting positions where the rejection cause appeared):

```
3:('P1', 'near0', 'clause', 'nomerge') 1296 0 0 36 36 12 0 0 0 8 648 0 0 0
4:('P1', 'near0', 'clause', 'pushB') 1296 0 0 0 0 0 24 16 24 32 624 0 16 0
5:('P1', 'near0', 'clause', 'pushW') 1296 0 0 0 0 0 24 16 24 16 624 0 16 0
6:('P1', 'near0', 'clause', 'today') 1296 0 0 0 0 0 0 0 24 8 648 0 0 0
11:('P1', 'near2', 'clause', 'nomerge') 1296 6 6 24 24 0 0 0 0 0 648 0 0 0
12:('P1', 'near2', 'clause', 'pushB') 1296 14 14 0 0 0 36 24 36 36 612 0 24 0
13:('P1', 'near2', 'clause', 'pushW') 1296 14 14 0 0 0 36 24 36 12 612 0 24 0
14:('P1', 'near2', 'clause', 'today') 1296 18 18 0 0 0 0 0 36 0 648 0 0 0
15:('P1', 'near2', 'code', 'nomerge') 1296 552 552 0 0 0 0 0 0 0 312 908 0 0
16:('P1', 'near2', 'code', 'pushB') 1296 528 528 0 0 0 24 24 24 0 312 860 24 0
17:('P1', 'near2', 'code', 'pushW') 1296 528 528 0 0 0 24 24 24 0 312 860 24 0
18:('P1', 'near2', 'code', 'today') 1296 528 528 0 0 0 0 0 24 0 336 860 0 0
27:('P2', 'near0', 'clause', 'nomerge') 1296 48 48 36 36 12 0 0 0 0 588 0 0 108
28:('P2', 'near0', 'clause', 'pushB') 1296 48 48 0 0 0 24 16 24 16 564 0 16 116
29:('P2', 'near0', 'clause', 'pushW') 1296 48 48 0 0 0 24 16 24 0 564 0 16 116
30:('P2', 'near0', 'clause', 'today') 1296 48 48 0 0 0 0 0 24 0 588 0 0 108
59:('T1', 'near2', 'clause', 'nomerge') 1296 0 0 0 0 0 0 0 0 0 648 0 0 0
62:('T1', 'near2', 'clause', 'today') 1296 0 0 0 0 0 0 0 0 0 648 0 0 0
22:('P1', 'roomy', 'clause', 'today') 1296 0 0 0 0 0 0 0 0 0 0 0 0 0
```

### Readings for Each Row: Segment Two (Q2)

Copied verbatim from `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out` (`name=stability` four lines per (scope, arm), z family and H5 each 16 lines, transcribing z family 16 lines and H5 "more than all-zeros not merged" line):

```
7490:name=stability segment=second id=Q2-1-not-merged-has-more scope=z_h1_to_h4 arm=nomerge hit_points=0 points=12 stability=stable_no_hit
7491:name=stability segment=second id=Q2-1-merged-has-more scope=z_h1_to_h4 arm=nomerge hit_points=0 points=12 stability=stable_no_hit
7492:name=stability segment=second id=Q2-2-merged-all-enospc scope=z_h1_to_h4 arm=nomerge hit_points=12 points=12 stability=stable_hit
7493:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=z_h1_to_h4 arm=nomerge hit_points=0 points=12 stability=stable_no_hit
7494:name=stability segment=second id=Q2-1-not-merged-has-more scope=z_h1_to_h4 arm=pushB hit_points=6 points=12 stability=unstable
7495:name=stability segment=second id=Q2-1-merged-has-more scope=z_h1_to_h4 arm=pushB hit_points=0 points=12 stability=stable_no_hit
7496:name=stability segment=second id=Q2-2-merged-all-enospc scope=z_h1_to_h4 arm=pushB hit_points=12 points=12 stability=stable_hit
7497:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=z_h1_to_h4 arm=pushB hit_points=3 points=12 stability=unstable
7498:name=stability segment=second id=Q2-1-not-merged-has-more scope=z_h1_to_h4 arm=pushW hit_points=6 points=12 stability=unstable
7499:name=stability segment=second id=Q2-1-merged-has-more scope=z_h1_to_h4 arm=pushW hit_points=0 points=12 stability=stable_no_hit
7500:name=stability segment=second id=Q2-2-merged-all-enospc scope=z_h1_to_h4 arm=pushW hit_points=12 points=12 stability=stable_hit
7501:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=z_h1_to_h4 arm=pushW hit_points=3 points=12 stability=unstable
7502:name=stability segment=second id=Q2-1-not-merged-has-more scope=z_h1_to_h4 arm=today hit_points=6 points=12 stability=unstable
7503:name=stability segment=second id=Q2-1-merged-has-more scope=z_h1_to_h4 arm=today hit_points=0 points=12 stability=stable_no_hit
7504:name=stability segment=second id=Q2-2-merged-all-enospc scope=z_h1_to_h4 arm=today hit_points=12 points=12 stability=stable_hit
7505:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=z_h1_to_h4 arm=today hit_points=3 points=12 stability=unstable
7481:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=h5_holes arm=pushB hit_points=4 points=12 stability=unstable
7485:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=h5_holes arm=pushW hit_points=4 points=12 stability=unstable
7489:name=stability segment=second id=Q2-2-merged-all-enospc-beyond-not-merged scope=h5_holes arm=today hit_points=4 points=12 stability=unstable
```

Literal Q2-2 (`id=Q2-2-merged-all-enospc`, single mount all ENOSPC for all-zeros merged alone compared to the run with read faults removed) hits all twelve points across all four Q1 arms on z family, including Q1 not merged arm: on Q1 not merged arm both Q2 arms are identical position by position (F5), so the hits there represent common blockage shared by both Q2 arms (oldest valid root in ring pinned under read failure, reclamation halted, `rej_own_error` and `rej_raise_no_placement` on code-interpretation pools), not caused by all-zeros merged; only the supplemented `beyond-not-merged` (all-zeros merged extra-rejected on first try, all-zeros not merged succeeding, followed by no subsequent requests succeeding) distinguishes the two Q2 arms: not merged arm has 0, the other three arms hit 3 / 12 points on z family and 4 / 12 points on H5 (the 4 points on H5 are clause interpretation, both c_max tiers on S = 8 and 16, absent on S = 4, hence `unstable`). `threshold_shift` lines in segment two total 32 lines: 13 lines `flips=yes`, 19 lines `flips=not_applicable_observed_below_1`, no `flips=no`.

Position counts comparing both Q2 arms position by position with extra rejections and crossings per arm (`…-arm-table-2026-10-05.out` segment two, line numbers at head; total positions 648 per row for z family, 6 per row for H5; Q2 arm column written in last field of each row, second table columns in order are position count, positions identical item by item, all-zeros not merged crossing more, all-zeros merged crossing more, all-zeros not merged losing more states, all-zeros merged losing more, all-zeros merged extra rejections, all-zeros merged never succeeding after first extra rejection):

```
155:('z', 'P1', 'near0', 'clause', 'today', 'zeromerge') 648 0 0 0 0 0 0 0 12 4 324 0 0 0
156:('z', 'P1', 'near0', 'clause', 'today', 'zeronot') 648 0 0 18 18 6 0 0 0 4 324 0 0 0
171:('z', 'P1', 'near2', 'clause', 'today', 'zeromerge') 648 9 9 0 0 0 0 0 18 0 324 0 0 0
172:('z', 'P1', 'near2', 'clause', 'today', 'zeronot') 648 3 3 12 12 0 0 0 0 0 324 0 0 0
107:('H5', 'none', 'near0', 'clause', 'today', 'zeromerge') 6 0 0 0 0 0 0 0 4 2 4 0 0 0
108:('H5', 'none', 'near0', 'clause', 'today', 'zeronot') 6 0 0 0 0 0 0 0 0 1 1 0 0 0
367:('z', 'P1', 'near0', 'clause', 'nomerge') 648 648 0 0 0 0 0 0
368:('z', 'P1', 'near0', 'clause', 'pushB') 648 390 18 0 6 0 0 0
370:('z', 'P1', 'near0', 'clause', 'today') 648 390 18 0 6 0 0 0
378:('z', 'P1', 'near2', 'clause', 'today') 648 372 12 0 0 0 6 6
402:('z', 'P2', 'near2', 'clause', 'today') 648 372 12 0 0 0 6 6
450:('z', 'T2', 'near2', 'clause', 'today') 648 372 12 0 0 0 6 6
343:('H5', 'none', 'near0', 'clause', 'nomerge') 6 6 0 0 0 0 0 0
346:('H5', 'none', 'near0', 'clause', 'today') 6 0 0 0 0 0 4 4
```

Reading: clause interpretation, near-full k = 2, P1, as-of-today, across 648 positions in z family, all-zeros not merged has 12 positions crossing non-empty roots (line 172, `crossed` and `over_truth` columns) and 3 positions with extra rejections due to read failure; all-zeros merged has 0 positions crossing and 9 positions with extra rejections due to read failure (line 171); across 648 positions, the two arms are identical item by item in 372 positions, with 6 positions where all-zeros merged has extra rejections and never succeeds in issuing a single request after the first extra rejection (line 378). Minimum and closing values of K(F) (rollback candidate count) reside in `candidates_lowest` and `candidates_final` in each `name=cell` line; `lowest_candidates_*` in `name=q2_comparison` lines has a minimum value of 1 across every scope and arm in segment two (Q2-3, no threshold configured, reported as observed).

### Unit Tests and Mutations

17 unit tests (counted by single command: `grep -c '^test .* \.\.\. ok$' e173-admission-ceiling-bad-ring-slot-dispositions-unit-tests-2026-10-05.out` outputs 17; in the same file `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`). Mutation table `scripts/mutation/tables/e173_admission_ceiling_bad_ring_slot_dispositions.tsv` has 23 lines; output of `mutate.sh` in `e173-admission-ceiling-bad-ring-slot-dispositions-mutate-2026-10-05.log`, closing with `count: memory cap hits 0 (limit 1G), timeouts 0, unscheduled 0, squeezed by total limit 0` and `restored, baseline remains all green`; counted by leading symbols of result lines (`grep -c "^✅ "` etc.): caught (✅) 23, invalid (⏭) 0, unkilled (❌) 0, 💥, ⚠️, ⏱, 🧱 each 0, the sum of 23 matching the line count of the table.

Mutation table lines (M and Mm numbers in names are in-page identifiers for this page): M1–M12 are the twelve items from Section 9 of the registration (M6 written as boundary modifications M6a and M6b; M9 written as "oldest entry not compared when absent", see Section 12 Revision 3); M13 (reclamation threshold taking greater changed to taking smaller), M14 (raising even when upper bound equals F), M15 (publish count of F-raising sequence not counted into pushed count) modify core paths; Mm1–Mm7 seven items modify metrics and judgments (threshold for single mount all ENOSPC extra rejections, positions where not merged exceeds merged, upper edge of crossed count, ceiling on lost states, positions where aggregated quantity > 0, judgment hit threshold, peak start point of trajectory). The test name turning red for each mutation is in the output lines of `mutate.sh`. Main agent on 2026-10-05 acknowledged the phrasing of M6 and M9: the two lines in registration Section 9 remain, with their phrasing recorded in registration Section 12 (Revision 3).

Sample-point insensitivity class: prior to running `mutate.sh`, rustc was used to directly compile the unit test binary for pre-checking (draft readings, not taken as basis); Mm1, Mm2, Mm7 did not turn red on first try: in unit tests extra rejections was 4 (cannot distinguish threshold 1 from 2), position counts for the two Q2 arms were entirely different (cannot distinguish `>=` from `>`), and peak fell exactly one step after the first insufficiency (cannot distinguish start point shifted by one step); each was supplemented with a sample point yielding distinct values (extra rejections exactly 1 and on the last request, not-merged arm identical across two Q2 arms position by position, peak falling exactly on the step of first insufficiency); all 23 items turned red in formal `mutate.sh`. Mutation proves that "code with assertions written was tested by mutation", not that "every piece of code touched by the conclusion was tested": specific numbers aggregated by each arm across bad slot positions j = 1..3S − 1 are pinned only by anchor cells, B3 position-by-position expressions, and 17 unit tests, with the aggregation and judgment layer monitored separately by Mm1–Mm7.

### What It Cannot Answer

- The count model does not model journal replay, crash enumeration, or checkers; does not model roots abandoned by administrator rollback or instance table; space is accounted as a single copy, not split across disks; reserve pool takes only two interpretations, not computed dynamically per D28 (Mount-Time Commitments) Settled Item 4 for c_max and ckpt_cost (the two segments of 2026-10-05; the arm computed from ckpt_cost was added by rerun 1).
- Bad slots hiding the ground-truth newest root (j = 0) and entire root ring segments across a whole disk reading bad are not included in scans; subsequent mounts beyond the tested mount (whether capping is released after re-read) are not measured; request stream contains only overwrites, without deletions or other user modifications.
- Five history families, three read failure types × four persistence types, three pool types × two reserve pool interpretations, and sample points for S and c_max are enumeration points, not domains; position j is scanned exhaustively across 1..3S − 1. Judgments on each (arm, S, reserve pool interpretation, c_max) point in segment one mixing true and false are recorded as `unstable`, unmeasured on any geometry outside these points.
- Near-full pools under this set of histories become tight only on the first request (k = 0) and after the oldest valid root in the ring is pinned and reclamation stops (k = 2); code-interpretation reserve pool cannot accommodate an F-raising sequence, and baseline itself rejects entirely on those cells, reported separately in readings as `baseline_rejected`.
- t2 is taken literally (ending read failure only after the first F-raising sequence completes); when the first F-raising is blocked by the read failure itself, t2 shares readings with P2 on those positions.
- The two knobs in reclamation threshold—bad slots pinned to maximum txg (clause interpretation) and three consecutive bad slots—were not run (labeled in registration Section 8 as "incidental, not run after sufficient basis"); whether these two knobs flip readings of Q1 and Q2 was unmeasured.
- The three places where clause and code interpretations diverge (whether in-process re-read readable but failing self-witnessing counts oldest pinned txg or recorded maximum txg, whether reading back all zeros merges into upper bound, reserve pool 10 + 7c vs ckpt_cost) are taken according to code in E173 (Three Treatments for Upper Bound Blocked on Bad Slots) in the two segments of 2026-10-05, with clause-side readings unmeasured; both interpretations of the third place (reserve pool 10 + 7c vs ckpt_cost) were measured in rerun 1, with readings in the "Rerun 1" part, the alternate interpretation of the first place was not run, handed over to main agent.
- What rerun 1 cannot answer: the model changes only the reserve pool, and neither the ring window nor the switch reserve enters the model (X6 not run after sufficient basis); the metadata half of an overwrite takes the c_actual of an empty publish, which is 2 blocks fewer than real on `s8_g_min_64mib`, `s16_g_min_128mib`, `s16_g_1` and `s16_g_4` (X5); c_actual takes only the empty publishes in the warm-up and unmount chain for which the core layer returned output, and the floor-raising chain and the pushed empty publishes inside admission were not measured on real disk; κ was measured only on a history of one file, and κ after multiple files, snapshots, the deletion queue, lineage and a grown retirement tree is not included.
- In rerun 1 the "as of today" half is still the behavior of the frozen tree of 2026-10-05, and whether today's `crates/` still matches it on K1–K7 was not re-checked this time; segment one measured on `crates/` of 2026-11-02 (registration revision 18 recorded the hashes of four core-layer files, with uncommitted changes relative to HEAD).
- The judgments of rerun 1 are sensitive to geometry: one row of Q1-3 and of Q2-2 flips between G-min and G-4 and between S = 4 and S = 16 (F7, 48 lines), so a difference between the readings can only be written as a function of pool size and S, not as a property of the reading; the 9 geometries are enumeration points, not a domain. The two Q2-2 columns differ in the number of positions, and both readings hit in the aggregate judgment.
- The new code reading of rerun 1 (reserve pool = κ) turns 33 cells that the old code reading missed into hits (Z2): the code-reading numbers cited in the "user decision" item of the 2026-10-05 text (crossing 0 / 0 and so on) are those of the old code reading (reserve pool = c), not computed from ckpt_cost; on 2026-11-02 the user settled to reopen Q1 and Q2: bad root slots "push empty", root slots that read back all zeros "merge" (the "Rerun 1" section of this page).

### Path and Conclusion Registration

| # | Path | Computation Method | Source Code Location | Shared Items Read |
|---|---|---|---|---|
| 1 | Apparatus count model | Independently hand-written, deduced step by step per rules in registration Sections 2, 3, 5: each mount executes J = 2 × 3S + 8 overwrite requests, each request traversing the acceptance loop; X values: bad slot position j scanned exhaustively across 1..3S − 1, history family, read failure, persistence, pool, S, c_max are enumeration points (rationale: opposing sample points selected in registration Section 8) | `scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs:1488 run_a_mount`, `:1001 ceiling_pass`, `:511 txgs_ring_slots_bad_at_mount_can_hold` | Constants from D16 (Publish Semantics) Settled Items 1, 8, 9 and D22 (Synthesizing Unit Atomicity) Settled Item 2 transcribed in registration Section 2 (B = 8, c_max = 5 / 10, S = 4 / 8 / 16, d = 8, placement formula) and text interpretations of frozen tree behavior; shares no code with `crates/` |
| 2 | Use cases on frozen tree | `mount_writable` and `raise_rollback_floor_to_the_admission_ceiling` in `crates/` of frozen tree executed genuinely across two disk layouts, asserting pinned upper bounds | `crates/singlefs-verification/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs:280 admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount` (and four other use cases in same file at lines 316, 339, 381, 409) | Two disk layout constructions copied from use case header and settled decisions on RGall, all-zeros read back not merged, and reclamation threshold pinning; apparatus constructs disk layouts per that text, sharing no code |
| 3 | κ of rerun 1 segment one, local recomputation against the core-layer value | The in-tree apparatus recomputes locally the κ before each publish by the sum list of D28 (Mount-Time Commitments) Settled Item 4 (current height of each record tree + nodes per publish of the accounting tree + 1 for the tree table; the allocation record tree term by K1), also reads the value of `checkpoint_cost_of_the_version_to_build_on` in the core layer, and compares them publish by publish (halting clause S2); X values: the 9 geometries (S × pool size) are enumeration points, and every publish in every geometry is compared | `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:452 local_checkpoint_cost`, `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:528 checkpoint_cost_reading` | The sum list of D28 (Mount-Time Commitments) Settled Item 4 (both paths are written from it, so a wrong list makes both wrong); the tree-table term 1 (a local constant; in the core layer it is a function-local constant that cannot be compared back); format constants 812, 169 and so on (local constants compared back with the same-named constants of `singlefs_format`, halting clause S4) |
| 4 | c_actual of rerun 1 segment one, the two fixed-point routes | Fixed-point slots newly taken by one empty publish: route one adds the spans of the roles attributed to the "checkpoint reserve pool" in this publish's `rewritten`; route two adds, disk by disk, the spans of the unreleased records of this version whose allocation generation equals this txg, then subtracts the slots of the ordinary-allocation and instance-switch-reserve roles; the routes are accepted only when identical disk by disk (halting clause S3); X values: the publishes in the warm-up and unmount chain for which the core layer returned output, with the 9 geometries as enumeration points | `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:592 fixed_point_account` | The two-route measurement basis of the apparatus of E181 (Landing Determination and Mount Lower Bound); the `TransactionOutput` returned by the same publish; the definitions of the ordinary-allocation and instance-switch-reserve roles |

Shared item query: If clause constants (B, c_max, S, d, placement formula) are erroneous, both paths will err together—this clause cannot guard against "kb and implementation erring together". Cross-path comparison assertions are pinned in apparatus unit test `frozen_tree_use_case_cells_are_reproduced_on_every_arm` (`scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs`): comparing apparatus readings on the two disk layouts cell by cell against literal values pinned by use cases; no assertion directly bridges the outputs of both trees (frozen tree in copy, not in main workspace), which remains owed. The cross-path comparison assertions of rows 3 and 4 are pinned in the segment-one source: `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:2625 the_short_history_completes_and_the_local_kappa_equals_the_core_kappa_on_every_publish` (the local κ equals the core-layer κ) and `crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:2653 both_routes_agree_on_every_device_on_every_obtained_publish_of_a_short_history` (the two routes are identical disk by disk), and the 15 segment-one unit tests pass; in the artifact these two are also halting clauses S2 and S3, and neither halted.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) Settled Item 1 | Supports | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 changed: rerun 1 supports the four current clause rows of this item (in the "acceptance" and "`df`" rows the reserve pool is computed fresh from ckpt_cost; in the "Ceiling for raising F" and "Root slot read error this time" rows bad slots and slots reading back all zeros are merged into the upper bound, with empty publishes pushed when the upper bound is blocked on them); fork sheet row 1 judged B, and on 2026-11-02 the user settled to change the clause to follow the code and, reopening Q1 and Q2, chose "push empty" for Q1 and "merge" for Q2; the decision text and the decisions history were changed the same day; previously 2026-11-02 unaffected: rerun 1 only added the arm with the reserve pool computed from ckpt_cost, and the readings and artifacts of the original two segments of 2026-10-05 are unchanged; previously 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only wording revised (time words, location references and self-references), conclusions, figures and artifacts unchanged |
| D16 (Publish Semantics) Settled Item 8 | Does not affect | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: rerun 1 only added one arm of the reserve-pool reading and is unrelated to this item (warm-up); previously 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only wording revised (time words, location references and self-references), conclusions, figures and artifacts unchanged |
| D16 (Publish Semantics) Settled Item 9 | Does not affect | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: rerun 1 only added one arm of the reserve-pool reading and is unrelated to this item (empty publishes also write units); previously 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only wording revised (time words, location references and self-references), conclusions, figures and artifacts unchanged |
| D28 (Mount-Time Commitments) Settled Item 4 | Supports | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-03 changed: on 2026-11-02 the user decided from Z1 and Z3 of rerun 1 that the reserve pool is computed from ckpt_cost (D16 (Publish Semantics) Settled Item 1 changed the same day), and the basis section of this item cites E173 (Three Treatments for Upper Bound Blocked on Bad Slots) back; before that, 2026-11-02 unaffected: rerun 1 measured on real disk layouts the value of ckpt_cost of this item (κ, 9 geometries) and the blocks newly taken by one empty publish, and also ran the reserve-pool arm computed from it; the basis section of this item does not cite E173 (Three Treatments for Upper Bound Blocked on Bad Slots), so the relationship is written as preparation, and whether this row should be promoted to supporting is for the main agent; previously 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only wording revised (time words, location references and self-references), conclusions, figures and artifacts unchanged |
| D22 (Synthesizing Unit Atomicity) Settled Item 2 | Does not affect | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: rerun 1 only added one arm of the reserve-pool reading and is unrelated to how this item takes the slots per region S; previously 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only wording revised (time words, location references and self-references), conclusions, figures and artifacts unchanged |

## Historical Versions

### 2026-11-02: Rerun 1, the arm with the reserve pool computed from ckpt_cost, fork sheet row 1 judged B

Rerun registration `e173-r1-prereg.md` (written 2026-10-20 before the apparatus; Section 12 revisions 1–20 were added after the apparatus and before the matching artifacts); segment one (in-tree apparatus, measuring κ and c_actual in 9 geometries) and segment two (the new mode `checkpoint-cost-reserve`, two readings × 9 geometries) both completed, segment two passes 30 unit tests and all 33 mutation table rows are caught (0 invalid, 0 unkilled), and the Z2 and F7 reading scripts read the segment-two artifact to produce readings. Fork sheet row 1 is judged B (the code reading computed from ckpt_cost differs from the clause reading: 152 cells hit under the code reading only and 0 under the clause reading only, Z2 has 35 changed cells, and the difference varies with pool size and S); ③B, ③A, ③C and F7 are sufficient to judge, and X6 was not run after sufficient basis; the readings and the verbatim artifact lines are in the "Rerun 1" part above. In segment one artifact verdict lines, `code_reserve_holds_the_empty_publishes_of_one_admission=false` (under minimal disk geometry, the code reading reserve pool cannot hold all empty publishes of one admission). The page status changed from "Completed (two knobs in segment three not run after sufficient basis)" to the same sentence plus "rerun 1 done"; in "Affected Decisions" the row of D28 (Mount-Time Commitments) Settled Item 4 changed from does not affect to preparation, and the review dates of the other four rows were updated.

### 2026-10-05: Page created, segments one and two run

Pre-registration written before apparatus was implemented (`e173-preregistration.md`), 11 items added in Section 12 "Revisions" after apparatus, unit tests and mutation table were written and before artifacts were run; segment one (supplementing Q1) and segment two (supplementing Q2) completed in single run, both Q1 and Q2 rows had sufficient basis for judgment, segment three two knobs not run after sufficient basis.
