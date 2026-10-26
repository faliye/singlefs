## E172 Dry Run of Bytes Written by Deletion — run (sufficient to decide): all five rows of the second question sheet are decidable; all 32 mutations were caught. Neither this page nor the apparatus `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` contains a retirement item (`grep -c 退役` is 0 for the apparatus and finds only this sentence on the page). The apparatus was neither updated nor rerun after the retirement tree was added to `crates/`; owed check C775 (E172 apparatus lacks retirement items).

[中文](172-删除写出的字节的干跑.md) · **English** · [日本語](172-削除で書き出されるバイトのドライラン.ja.md)

<!-- doc-lint:not-numbers L03 -->

The second-run preregistration is `e172-r2-prereg.md` (reading B; the amendments section 12 is empty, with no amendments); the question sheet is `e172-r2-questions.md` (five rows mapping to rows 1–8 of the first sheet `m3-delete-dryrun-questions.md`). The first preregistration is `e172-preregistration.md`. The tested provisions are all of `.claude/kb/layout/03-third-txn.md`; the system-configuration field table in `.claude/kb/layout/01-first-txn.md` (line 149 T_time “5000 (in milliseconds, i.e. 5 seconds)”, offset 433 on line 158) and its section 8 segment sequences; D8 (Core Index Structure) settled items 16 and 6 (inode record offsets 64–95); and D16 (Publish Semantics) settled item 5.

**Current status** (second run): all five rows of `e172-r2-questions.md` are decidable, none of invalidation conditions V1–V6 or stop conditions S0–S4 fired, and all 18 applicable positive-control pairs were green before and red after mutation.

- Row 1 (rows 1, 2, 3 and 7 of the first question sheet on the 2026-10-05 export and H3) holds throughout: among 279 L03 cell rows in twelve deletion publications across H1, H2 and H3, zero discriminable rows differ, and every cell is hit in every history; T_time is 5000 in all 38 system-configuration slot writes in the three windows; zero unregistered bytes and zero nonzero bytes in the constant-zero class. Segment strings, type strings, roles and root slots match the registration for all H1 and H3 deletion publications and H2's intent publication, and the full-window segment strings of H1 and H3 match. The td and tt header levels and key ranges have the same values across the three histories (levels 0 / 1, [1, 1], marked as entry-key ranges and recorded without judgment).
- Row 2 (does a continuation batch have the same shape as a session batch of the same geometry, H2): no; the additional units have a basis in the provisions (B). The continuation 2 → 1 batch and final batch each write an extra pair for allocation record tree leaf 62; all changed records are attributed to allocation or freeing by that publication, with zero unmatched. These are the same readings as in the first run.
- Row 3 (H3: another size-0 inode is in the same leaf): as currently written. The final H3 batch writes inode leaf container t3 (inode 2 remains in the leaf); its role set matches the 2 → 1 batch, with empty differences in both directions.
- Row 4 (record offsets 64–95, H2 and H3): under this reading, the three timestamp seconds in H3's intent version equal the caller's time (60 seconds later than the sequential write and distinguishable); each batch copies them, and the change count equals that batch's txg. The same equalities hold for both H2 continuation batches.
- Row 5 (can writes before the mount's row write be split by publication, R8.c): core does not return the number acquisition write account (R8.a has zero hits), so only subtraction is possible. Subtraction splits H2 correctly (R8.c has zero misattributions and two number acquisition writes); PC-F and PC-G were green before and red after. Only the H2 sample was measured.

There is also a preregistered anchor mismatch (section 10 F3, not invalidating): A-journal understates eight rows across H1 and H3 by 8192 bytes, owing to the preregistration's assumption that “counter = txg.” This is the same erroneous derivation investigated in the first run; the second registration used that assumption again for H3.

The model is deterministic: both exports match byte-for-byte across two runs (three for H3), and apparatus results match byte-for-byte across two runs. This establishes absence of hidden state, not statistical stability; the strength of the verdict rests on unit tests fixing absolute values, positive controls and the mutation table.

### What was done

- Read-only export on the `crates/` side (main workspace, 2026-10-05, after formalization batch 30 changed T_time to milliseconds): run `cargo build -q -p singlefs-checker-tier --bin deletion_write_dump`, then the resulting `target/debug/deletion_write_dump` directly. Without arguments (H1, H2), both runs had sha256 `b7e87c4ed73978b2f6d89a00449bb46cb70a4d36a5e7a5a614ea64464bb090c0`, committed as `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out` (462 lines). Three runs with `--history h3` had sha256 `9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e`, committed as `research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out` (150 lines). Before the build and before and after export, hashes were recorded for `deletion_write_dump.rs` (`9b3f565e…cb`), `scenario.rs` (`9d007793…59`) and the sha256 of the per-file sha256 values in `crates/singlefs-core/src/` (`85af2670…59`); all matched.
- The new H1H2 export differs from the first run's `e172-deletion-write-dump-2026-10-05.out`: `diff` shows 30 lines on each side, all `name=device_region_bytes` for the two system-configuration slots (offsets 0 and 4096), reflecting the four T_time bytes changed to milliseconds and the corresponding whole-slot checksum. Both are retained.
- The research apparatus `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` was modified in place according to changes 1–15 in registration section 5.8: accept both exports; decode another four bytes from offset 433 of system configuration (R1.t); make the L03-5 intent cell discriminable when the export supplies `intent_write_time_seconds=`; branch L03-7 by the previous version's leaf record count; ignore allocation record tree leaf ordinal in H2/H3 role comparisons and report ordinals separately; delete R8.b and PC-E, add PC-G, PC-H and PC-I; give each of the two extra R8.c requirements its own constant. Copies of the first apparatus and mutation table are in `research/prompts/e172-r1-frozen/` (sha256 `a626e18a…4b`, `6e21494a…b3`).
- Output `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out` (817 lines, sha256 `4a03f73e37175aba3fe5169c2fe0a5c36110c16a1c66e36712db320a3f82537b`) was identical byte-for-byte across two runs of the same binary. The first result `e172-deletion-write-dry-run-2026-10-05.out` comes from a different apparatus reading a different export and does not match; it is retained but no longer registered for replay.

### Replay
```
# 仓根：两份导出（峰值表里实测 30M、17M）
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 60M cargo run -q -p singlefs-checker-tier --bin deletion_write_dump > <H1H2 导出>
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 34M cargo run -q -p singlefs-checker-tier --bin deletion_write_dump -- --history h3 > <H3 导出>
# research/：装置（峰值 88M）
nice -n 19 cargo build --release --bin e172-deletion-write-dry-run
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 176M ./target/release/e172-deletion-write-dry-run results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out results/e172-r2-deletion-write-dump-h3-2026-10-05.out
# 登记的复跑（driver_e172：先现跑两份导出进临时文件，再跑装置）
bash scripts/experiment-apparatus/replay.sh E172
# 单测 38 条与变异表 32 条
cd scripts/experiment-apparatus && nice -n 19 bash ../hosts/run-with-memory-cap.sh 898M cargo test --release --bin e172-deletion-write-dry-run
cd scripts/experiment-apparatus && nice -n 19 bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutations/e172_deletion_write_dry_run.tsv
```

### Verdict (lines copied intact from `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`)

```
E7RESULT name=verdict row=1 not_equal_cells=0 discriminable_false_rows=2 t_time_not_5000=0 not_equal_cells_h1=0 cells_without_hits_h1=0 cells_without_hits_list_h1=none not_equal_cells_h2=0 cells_without_hits_h2=0 cells_without_hits_list_h2=none not_equal_cells_h3=0 cells_without_hits_h3=0 cells_without_hits_list_h3=none t_time_not_5000_h1=0 t_time_not_5000_h2=0 t_time_not_5000_h3=0
E7RESULT name=verdict row=3 h1_segment_mismatches=0 h1_kind_mismatches=0 h1_role_mismatches=0 h1_root_mismatches=0 h1_window_string_equal=true h2_segment_mismatches=0 h2_kind_mismatches=0 h2_role_mismatches=0 h2_root_mismatches=0 h2_window_string_equal=not_reported h3_segment_mismatches=0 h3_kind_mismatches=0 h3_role_mismatches=0 h3_root_mismatches=0 h3_window_string_equal=true
E7RESULT name=verdict row=4 pairs_with_unique_roles=2 unique_units_without_basis=0
E7RESULT name=verdict row=5 h3_final_writes_t3=true h3_matches_registered=true
E7RESULT name=verdict row=6 r6a_not_equal_rows=0 r6a_discriminable_false_rows=1 r6b_not_equal_rows=0 r6b_discriminable_false_rows=0 r6b_intent_differs_from_sequential=true
E7RESULT name=verdict row=7 recorded_nodes=6
E7RESULT name=verdict row=8 attributed_writes=156 r8c_misattributed_writes=0
E7RESULT name=verdict row=2 unregistered_bytes=0 constant_zero_nonzero_bytes=0 double_claimed_bytes=0
E7RESULT name=verdict check=stops s2_structure_failures=0 s3_role_mismatches=0 s3_index_notation_only=0 s3_l03_7_second_branch_not_exactly_h3=0 s4_tiling_failures=0
E7RESULT name=verdict check=voids a_lines_equal_h1h2=true a_lines_equal_h3=true a_crc_equal=true histories_present=true a_content_equal_h1=true configuration_matches_h1=true a_content_equal_h2=true configuration_matches_h2=true a_content_equal_h3=true configuration_matches_h3=true positive_controls_failed=0 positive_controls_failed_h1=0 positive_controls_failed_h2=0 positive_controls_failed_h3=0 positive_controls_run=18 positive_controls_expected=18 anchor_not_equal=8
```

Output lines 452, 476, 568, 572, 586, 596, 753, 813, 815 and 816. Verdict rows retain the first question sheet's numbering: `row=1,2,3,7` constitute row 1 of the second sheet; `row=4`, row 2; `row=5`, row 3; `row=6`, row 4; and `row=8`, row 5 (registration section 6.0). `discriminable_false_rows=2` is the H1 and H2 intent cell, where the caller's intent time in the export is the record's existing time; `r6a_discriminable_false_rows=1` is the H2 cell among these. `h2_window_string_equal=not_reported` is explicitly withheld by the registration because the H2 window contains segments that are not deletion publications. See “Eight A-journal anchor mismatches” for `anchor_not_equal=8`.

### Readings for each row (lines copied intact from `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`)

**Row 1:** Count L03 cell rows by history and txg (`grep 'name=l03_cell' F | grep -o 'history=h[123] txg=[0-9]* publish=[a-z_]*' | sort | uniq -c`): H1 txg 5 / 6 / 7 / 8 have 30 / 23 / 23 / 17 rows respectively; H2 txg 5 / 11 / 12 / 13 have 30 / 23 / 23 / 17; H3 txg 6 / 7 / 8 / 9 have 30 / 23 / 23 / 17. Total 279 rows; `grep 'name=l03_cell' F | grep -c 'equal=false'` → `0`. The minimum H3 hit count per cell (lines 397–410) is 3, for example:

```
E7RESULT name=r1b history=h3 cell=L03-5 discriminable_hits=3 not_equal=0
E7RESULT name=r1b history=h3 cell=L03-13 discriminable_hits=4 not_equal=0
```

The final H3 batch takes L03-7's second branch (inode 2 remains in the previous version's leaf), lines 378–381, 395–396 and 814:

```
E7RESULT name=l03_cell cell=L03-7 field=inode_root_entry_count history=h3 txg=9 publish=unlink_batch expected=1 actual=1 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-7 field=other_records_in_written_containers history=h3 txg=9 publish=unlink_batch expected=2 actual=2 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-7 field=inode_leaf_containers_written history=h3 txg=9 publish=unlink_batch expected=1 actual=1 equal=true discriminable=true
E7RESULT name=l03_7_record history=h3 txg=9 inode_root_key_range=1/1 judged=false
E7RESULT name=l03_7_branch history=h3 txg=9 branch=other_records other_records_before=1
E7RESULT name=l03_7_branch_check second_branch_histories=h3 exactly_h3=true
```

The final H1 and H2 batches take the first branch (`branch=only_record`, lines 179 and 287). Accounting tag 6: each deletion version is 98304 for H1/H2 and 98386 for H3 (n_live = 1, H = 0; line 391 `field=committed_reservation_h0_live1 … expected=98386 actual=98386 equal=true`). T_time (R1.t, lines 419, 442, 451):

```
E7RESULT name=r1t_summary history=h1 system_configuration_slot_writes=8 t_time_not_5000=0
E7RESULT name=r1t_summary history=h2 system_configuration_slot_writes=22 t_time_not_5000=0
E7RESULT name=r1t_summary history=h3 system_configuration_slot_writes=8 t_time_not_5000=0
```

Segment strings (lines 473–475); H2's verdict includes only its intent publication (reading B). The three continuation batch strings are still reported: txg 12 and 13 are `26+2+1+2` and `24+2+1+2` (each has an extra leaf pair), and are not included in row 1.

```
E7RESULT name=r3d history=h1 expected=26+2+1+2+24+2+1+2+22+2+1+2+20+2+1+2 actual=26+2+1+2+24+2+1+2+22+2+1+2+20+2+1+2 equal=true
E7RESULT name=r3d history=h3 expected=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 actual=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 equal=true
E7RESULT name=r3_geometry history=h3 txg=9 expected_only_record=20+2+1+2 expected_other_records=22+2+1+2 actual=22+2+1+2 only_record_green=false other_records_green=true expectations_differ=true exactly_one_green=true
```

H2/H3 role comparisons omit allocation record tree leaf ordinals (eight `name=r3c_ordinal … judged=false` lines). The ordinals of all four H3 publications and H2's intent are `t5@0.0.61,t5@0.1.61,t5@1.0.0,t5@1.1.0`, the same as registered for H1. Coverage (lines 811–812): `E7RESULT name=r2a history=h3 total_bytes=1771520 layout03_cell_bytes=6824 layout01_cell_bytes=75776 constant_zero_bytes=1688920 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true`, `E7RESULT name=r2c history=h3 constant_zero_nonzero_bytes=0`; H1 and H2 have the same numbers as the first run (lines 807–810). td and tt (lines 593–594): `E7RESULT name=r7 history=h3 txg=6 role=td level=0 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false`, `E7RESULT name=r7 history=h3 txg=6 role=tt level=1 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false`; H1 and H2 have the same values.

**Row 2** (lines 477–478 and 505; all four `name=r4b` lines have `unmatched=0 has_clause_basis=true`):

```
E7RESULT name=r4a data_units_after=2 continuation_txg=11 session_txg=6 continuation_units=12 session_units=12 continuation_only= session_only= same_shape=true
E7RESULT name=r4a data_units_after=1 continuation_txg=12 session_txg=7 continuation_units=13 session_units=11 continuation_only=t5@0.0.62,t5@0.1.62 session_only= same_shape=false
E7RESULT name=r4a data_units_after=0 continuation_txg=13 session_txg=8 continuation_units=12 session_units=10 continuation_only=t5@0.0.62,t5@0.1.62 session_only= same_shape=false
```

Geometry (`name=r4c`, lines 556–567): H1's three session batches have placements 50283–50318 and H3's four publications at 50280–50332; both write only leaf 61. H2 continuation txg 12 and 13 land at 50343–50368 and write leaves 61 and 62 (leaf 62 covers [50344, 51156)). H3 has one extra publication and its positions shift by 12 slots but still do not cross into leaf 62.

**Row 3** (lines 570–571):

```
E7RESULT name=r5b history=h3 two_to_one_txg=8 final_txg=9 final_writes_t3=true only_in_two_to_one=none only_in_final=none matches_registered=true
E7RESULT name=r5b_discrimination history=h3 h1_expectation_green=false h3_expectation_green=true exactly_one_green=true
```

The H1 half is incidental and excluded from the verdict, line 569: `E7RESULT name=r5a history=h1 two_to_one_txg=7 final_txg=8 only_in_two_to_one=t3 only_in_final=none matches_registered=true incidental=true`.

**Row 4** (six `name=r6a` and six `name=r6b` lines, all `equal=true`): the H3 intent cell and the two times, lines 580 and 585:

```
E7RESULT name=r6b cell=L03-5 history=h3 txg=6 publish=unlink_intent expected=1788000120/1788000120/1788000120 actual=1788000120/1788000120/1788000120 equal=true discriminable=true
E7RESULT name=r6b_intent_time history=h3 intent_write_time_seconds=1788000120 sequential_write_time_seconds=1788000060 differ=true
```

Both H3 batches (txg 7 and 8) copy all three cells from the intent version (1788000120), and change counts 6 / 7 / 8 equal the current txg. H2's intent cell is `discriminable=false`; the three cells in continuation txg 11 and 12 are 1788000060, with change counts 11 / 12.

**Row 5:** A current check in the main workspace for R8.a: `grep -n 'writes\|acquir'` on `MountOutput` in `crates/singlefs-core/src/mount.rs` exits 1 (no hits); `grep -n 'acquisition_writes\|writes_of_the_acquisition\|instance_acquisition' crates/singlefs-core/src/mount.rs` exits 1. Its 19 `pub` fields have the same names as in the first run, and none returns an account of number acquisition writes. All 156 R8.c writes have `match=true`; the two number acquisition writes (lines 597–598):

```
E7RESULT name=r8c history=h2 step=95 device=0 kind=system_configuration_slot decoded=instance_acquisition exported_segment=mount_writes_before_its_first_publish match=true
E7RESULT name=r8c history=h2 step=96 device=1 kind=system_configuration_slot decoded=instance_acquisition exported_segment=mount_writes_before_its_first_publish match=true
```

Positive controls (lines 783–806; 24 lines): 18 applicable pairs have `applies=true target_green_before=true target_red_after=true`; six inapplicable pairs are PC-F and PC-G in H1/H3 and PC-I in H1/H2, with `applies=false` and reasons. After mutation PC-F requires exactly one R8.c misattribution at the first write of the row-writing publication (`decoded=publish_txg8`, segment before the mount row write); PC-G requires exactly one at the final write of that part (step 96, system-configuration slot, `decoded=instance_acquisition`, segment `mount_instance_row_txg8`); PC-H requires exactly one T_time ≠ 5000, value 5, and the same number of whole-slot checksum failures as before mutation; PC-I requires the H3 intent cell to differ, with all three actual cells equal to the sequential-write time.

**Trajectory** (lines 755–782): H3 `deleted_inodes` peaks at 1, is positive 3 times and ends at 0; data units peak at 3, are positive 3 times and end at 0; tag 6 is 98386 in all four versions. H1/H2 have the same counts as the first run, and tag 4 is 0 in all three histories.

### Eight A-journal anchor mismatches (section 10 F3: check both sides, not invalidating)

Lines 19, 25, 31, 35, 64, 70, 76, 80; the first H3 line is:

```
E7RESULT name=anchor anchor=A-journal history=h3 txg=6 expected=0:16797696,1:16797696 actual=0:16805888,1:16805888 equal=false
```

All eight differ by 8192 bytes (two record slots); the other 69 anchor lines match (`anchor_not_equal=8`, 77 `name=anchor` lines).

- Provision side: layout/01 line 319, “record n lands at (counter − 1) mod slot count × 4096,” holds in the window. Reading writes with magic `SFSJ` on device 0 from the committed H3 export (instance generation, counter, checkpoint_txg and transaction number at offsets 16, 20, 26, 78), the verbatim output is: `h3 step 28 offset 16805888 jsn_instance 1 jsn_counter 8 checkpoint_txg 6 transaction 6`, `h3 step 63 offset 16809984 jsn_instance 1 jsn_counter 9 checkpoint_txg 7 transaction 7`, `h3 step 96 offset 16814080 jsn_instance 1 jsn_counter 10 checkpoint_txg 8 transaction 8`, `h3 step 129 offset 16818176 jsn_instance 1 jsn_counter 11 checkpoint_txg 9 transaction 9`; each equals 16 MiB + (counter − 1) × 4096. The four H1 lines have the same values as in the first run (counters 7–10).
- Registration side: section 7 A-journal (H3) of the second registration still derives using “counter = txg” (warm-ups 1, 2, new-pool file creation 3, inode 2 creation 4, sequential write 5). The first run had already found that this step does not follow from the provision and that the sequential-write publication writes three records (derived); H3's counter exceeds txg by 2, consistent with the sequential write adding two records.
- Classification: the registration's derived anchor is wrong; this is not a mismatch between implementation and layout/01. It is outside row 1's L03 cell table and the segment string, so row 1's verdict is unaffected.

### Unit tests and mutations

- 38 unit tests (last `cargo test --release --bin e172-deletion-write-dry-run` log on 2026-10-05: `test result: ok. 38 passed; 0 failed`): eight of the first run's 26 tests were changed, one deleted (R8.b not registered), and 13 added (t1–t13 in registration section 5.8: decode T_time from a synthetic system-configuration slot, R1.t row counts and expectations, L03-7 branch two, H3 intent time, synthetic batch-time expectations, R5.b and self-test rows, H3 A-tag6, copies of PC-G / PC-H / PC-I, two synthetic R8.c attribution cases, and H3's four anchor types). The new tests fix expectation fields, structure and synthetic inputs only; they do not assert H3's observed readings.
- Mutation table `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` has 32 entries: the first run's 20 (M19's text rewritten for indentation), plus M1b, M5, M10b and M22–M30. Each anchor matches exactly once in source (Python count); at the time of this note the complete table had not yet been run and was assigned to the main agent.

### What this cannot answer

1. Only batch size 1, two 4 GiB disks and one file of three data units were measured (row 1 says “measured only at batch size 1”). H3 adds only one size-0 record; cases with two or more additional records in the leaf or where the emptied leaf is not the final leaf were not measured.
2. T_time has only one sample: there is no mount-options channel (C422), so the implementation writes the built-in default in all three histories. R1.t judges “four exported bytes = 5000 from layout/01 line 149,” and PC-H establishes discriminative power. It cannot answer whether a different mount-time value would be written correctly.
3. Row 5 was measured only at the H2 sample: the instance generation changes across number acquisition and the two number acquisition writes have tail equal to the counter of the unmount part's final publication. Each of R8.c's two requirements beyond R8.b has a synthetic unit test with a sensitive sample (T11, T12); opposite geometry cannot be produced in the export. “Subtraction splits correctly” applies only to this export.
4. Row 2 has only three geometry points (H1 session batches, H3 session batches, H2 continuation batches), and only the continuation batches cross into leaf 62. “A session batch crossing leaf 62 would also write that leaf” is an inference (H4 was not run after sufficient evidence).
5. Row 1 coverage (R2) reads by position: bytes within a registered cell's offset range count as registered, regardless of what that cell contains. The remaining two payload bytes of extent upper-leaf entry tag 1 and the instance-table row record rely only on the field sequence of D18 (Information Carried in a Block), settled item 11.
6. The td and tt header levels and key ranges are recorded without judgment: the level clauses are D8 (Core Index Structure) settled item 16 item 3 (td) and D18 (Information Carried in a Block) settled item 24 item 1 (tt), and the key range clause is D18 (Information Carried in a Block) settled item 2 (td is a leaf-level code 2 node and takes the keys of its first and last entries; tt takes the subtree coverage range); the tt key range is judged by the pool-level checker, and the td key range is also judged by the pool-level checker in the tree table walk (`read_index_node` in `crates/singlefs-checker/src/walk.rs` takes the `FirstAndLastEntries` reading and calls `check_index_node_keys` in `crates/singlefs-checker/src/lib.rs`; a header range that differs from the keys of the first and last entries turns I-1.1 (block header states its logical address) red). The final H3 batch's inode-tree root key range [1, 1] is also recorded without judgment.
7. The window starts before the intent publication: bytes for mkfs, number acquisition, warm-up, new-pool file creation, inode 2 creation and sequential write are outside the export. The A-journal section's pre-window derivation relies only on write counts and counters.
8. The apparatus compares implementation with KB registration: it cannot see a shared KB/implementation error when the KB wording does not constrain that cell. The question sheet does not ask whether fields of the H3 inode 2 record (rewritten as a consequence) are copied, so that is not judged.
9. The main agent ran all 32 mutations on 2026-10-05: “32 ✅”, “counts: 0 hit memory ceiling (3G cap), 0 timed out, 0 unschedulable, 0 excluded by the global limit”; the last line says “restored, baseline still all green” (`e172-r2-mutate-2026-10-05.log`). This establishes only that the asserted locations turn red, not coverage.

### Paths and conclusions register

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Role set: decoder identifies bytes | Classify each unit by offset, length and magic; identify it by tree ID, level, packed record type, container number, device segment and position-based ordinal at the start of the allocation record tree key range. Tree roots use the root record and tree table root pointer. Scan every unit write in all three history windows, without sampling. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:1104 role_identity_of` (name: `:1129 role_name_of`) | Offsets from layout/01 lines 175–190, 259–263, 285–290, 297–299, 386–387 and D18 settled items 11, 18; allocation record tree leaves have 812 slots and level 1 covers 137228 slots (layout/01 line 298); role naming copied from `TransactionUnit::tag` in `crates/singlefs-core/src/transaction.rs` |
| 2 | Role set: rewritten roles returned by `crates/` | Core returns `TransactionOutput::rewritten` at each publication; `TransactionUnit::tag` names them and the export writes `rewritten_roles=` on the boundary row. | `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:119 segment_boundary_fields` | `TransactionUnit::tag` naming |
| 3 | Segment attribution: decoder attributes each write | Units use the header birth generation, journal records and root slots use checkpoint_txg, and system-configuration slots use R8.c (test acquisition first; rotation requires tail equal to that publication's jsn counter and matching instance generation). Count every write during mount. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:2431 attribute_mount_writes` (system-configuration slots: `:2395 attribute_system_configuration`) | layout/01 lines 152–153 (tail, instance generation), 186 (birth generation), 344–345 (jsn, checkpoint_txg), 373 (root-record checkpoint_txg) |
| 4 | Segment attribution: export partition | Partition directly invoked publications by recorded-stream length before and after the call; partition unmount and mount by each publication's write account returned by core. Calculate the part before the mount row write by subtracting the sum of publication write accounts from the mount-part write count. | `crates/singlefs-harness/src/scenario.rs:779 unmount_then_mount_continuation` | Core `TransactionOutput::writes` |

Shared-input challenge: paths 1 and 2 share the role naming convention, so S3 cannot see a naming error shared by both. Row 2 therefore judges shape using path 1's identities, not names (R4.a). Paths 3 and 4 share no code, only the recorded stream. Cross-path comparisons in source: `role_name_comparison` (same file line 2056) compares the multisets from paths 1 and 2; `is_match` in `attribute_mount_writes` compares paths 3 and 4. Unit tests `decoded_structure_passes_every_stop_check_on_the_export` and `mount_writes_are_attributed_to_the_same_segments_the_export_cuts` pin both results on the committed export.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure), settled item 16 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D8 (Core Index Structure), settled item 6 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D8 (Core Index Structure), settled item 14 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 24 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 18 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 11 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 2 | Does not affect | 2026-10-26 unaffected: the body cites it only as the source of the key range reading; conclusions and artifacts unchanged |
| D16 (Publish Semantics), settled item 5 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D23 (Journal Role & Format), settled item 7 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |

## Historical Versions

### 2026-10-05: second run (rerun), all five question-sheet rows decidable

Rerun registration `e172-r2-prereg.md`, question sheet `e172-r2-questions.md`; exports `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out` and `research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out`; apparatus output `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`. The E172 (Dry Run of Bytes Written by Deletion) row of `scripts/experiment-apparatus/replay.sh` was changed to point at this output. The 32-entry mutation table was written but not yet run. The first run's status and sections remain unchanged under “2026-10-05: page created, first and second segments run.”

### 2026-10-05: page created, first and second segments run

#### First run: status

**Status:** Question-sheet rows 1, 2, 3 and 7 are decidable on H1 (H1 alone covers all 14 row-1 cells and row-2/3 readings match registration). All H2 verdicts are invalid under V4: PC-E's target R8.b was already red before mutation (output line 719). On 2026-10-05 the main agent judged this a faulty criterion (R8.b did not specify precedence between its two branches, and the original order misattributes two writes in the uncorrupted export). R8.c added by amendment 1 relaxes the criterion and cannot replace this run's verdict. Rows 4, 8 and the H2 half of row 6 are therefore not decidable and are rejudged in the third segment under R8.c; rows 5 and 6 each lack their H3 half (third segment, requiring H3 added to the `crates/` export first). The conflict between row 4's H2 observation that continuation batches write leaf 62 additionally and layout/03's statement that continuation batches have the same shape as session batches awaits the third-segment rejudgment.

- Row 1 all equal: eight deletion publications in H1/H2 produce 186 L03 cell rows, zero discriminable mismatches, and at least four hits for each of 14 cells. The two intent caller-time rows are not discriminable in H1/H2 (`discriminable=false`).
- Row 2 zero: across the two history windows, 1705984 + 3314176 bytes, zero unregistered and zero nonzero in the constant-zero class.
- Row 3 equal: the four H1 deletion publications' segment strings, kind strings, unit counts and role names per disk, and root-slot FUA all match layout/03 lines 51–55 and 59.
- Row 4 B (different shape, extra units have a provision basis): continuation 2 → 1 and final batches write two additional copies of allocation record tree leaf 62 (`t5@0.0.62`, `t5@0.1.62`) compared with the session batches of the same geometry. Every changed record in those leaves is attributed to allocation or freeing by that publication, zero unmatched. The difference follows whether the publication's positions cross into the slots covered by leaf 62 (continuation txg 12 lands at 50343–50356; leaf 62 begins at 50344), not the fact it is a continuation batch.
- Row 5's H1 half matches the current wording (the final batch lacks only t3 compared with 2 → 1); row 6's H1/H2 change counts and batch times match.
- Row 7 records values: td level 0, tt level 1, both key ranges [1, 1], marked “entry-key range.”
- Row 8: `MountOutput` does not return an acquisition write account (the “cannot” branch). Against decoder per-write attribution, the segment obtained by subtraction misattributes two writes under R8.b's original order and zero under amendment 1's R8.c. PC-E, R8.b's positive control, is already red before mutation on H2; literal section 11 V4 invalidates all H2 verdicts, referred to the main agent.

Deterministic model: the two exports and the two apparatus outputs each match byte-for-byte. Repeating demonstrates no hidden state, not statistical stability; verdict strength rests on absolute-value unit tests, positive controls and the mutation table.

#### First run: what was done

- Read-only `crates/` export: `cargo run -q -p singlefs-checker-tier --bin deletion_write_dump` in the main workspace (batch 18 X, merged 2026-10-05) runs H1 and H2 in sequence on two 4 GiB memory disks, emitting complete bytes for every write, every barrier and each segment boundary in the window beginning before the intent publication. Both runs have sha256 `5a9b3fa27dc5b23531b767b1ed03a5b83a40d8a279485314c990df64fbbbd397`, committed as `e172-deletion-write-dump-2026-10-05.out` (462 lines, 10118080 bytes). Before and after each run, hashes of `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` (`b9e9facc…d86577`), `crates/singlefs-harness/src/scenario.rs` (`fbe2208c…a3fc6cb`) and the sha256 of per-file sha256 values in `crates/singlefs-core/src/` (`12582193…ad9eb`) matched all three times.
- Research apparatus `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` (bin `e172-deletion-write-dry-run`): independent decoder, with no `use` of any `singlefs-*` crate. Offsets and widths are copied as local constants with source line numbers from layout/01, layout/03 and D18 settled items 11 / 18 / 24. Write types are identified only by offset, length and magic; exported fields read are `kind=` (FUA), segment boundaries and `rewritten_roles=` (for the differential only).
- One apparatus runs both first and second segments (registration section 5.5), yielding one output `e172-deletion-write-dry-run-2026-10-05.out` (728 lines, sha256 `8df41fda2bf597cea89e3c266ee9110d51c012a531704db091f43e5b91b9bbe2`). The `-s1` suffix specified in section 5.6 was not used because the file also contains second-segment rows. Two runs were byte-identical.

#### First run: replay (the E172 row of `scripts/experiment-apparatus/replay.sh` now points to the second run's output)

```
# 仓根：导出（峰值表里实测 31M）
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh auto cargo run -q -p singlefs-checker-tier --bin deletion_write_dump > <导出文件>
# research/：装置
nice -n 19 cargo build --release --bin e172-deletion-write-dry-run
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh auto ./target/release/e172-deletion-write-dry-run results/e172-deletion-write-dump-2026-10-05.out
# 登记的复跑（driver_e172：先现跑导出进临时文件，再跑装置）
bash scripts/experiment-apparatus/replay.sh E172
# 单测 26 条与变异表 20 条
cd scripts/experiment-apparatus && cargo test --release --bin e172-deletion-write-dry-run
cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutations/e172_deletion_write_dry_run.tsv
```

#### First run: verdict (lines copied intact from `e172-deletion-write-dry-run-2026-10-05.out`; rows for artifacts moved out of the workspace are one-line pointers)

```
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 258 行（name=verdict）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 268 行（name=verdict）
E7RESULT name=verdict row=4 pairs_with_unique_roles=2 unique_units_without_basis=0
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 358 行（name=verdict）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 369 行（name=verdict）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 376 行（name=verdict）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 689 行（name=verdict）
E7RESULT name=verdict row=2 unregistered_bytes=0 constant_zero_nonzero_bytes=0 double_claimed_bytes=0
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 726 行（name=verdict）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 727 行（name=verdict）
```

Output lines 258, 268, 356, 358, 369, 376, 689, 725, 726 and 727. No stop condition S1–S4 fired (S1: both files currently exist; S2, S3 and S4 are the four zeros on the `check=stops` row). Invalidation conditions V1 (export exit 0), V2 (`a_lines_equal=true`), V3 (same sha256 on both exports), V5 (`a_crc_equal` and `a_content_equal_*` all `true`), and V6 (`configuration_matches_*` all `true`) did not fire; see “Row 8 and amendment 1” for V4.

#### First run: readings for each row (lines copied intact from `e172-deletion-write-dry-run-2026-10-05.out`; rows for artifacts moved out of the workspace are one-line pointers)

**Row 1** (field-by-field comparison of layout/03 registered fields and implementation): hits and mismatches per cell, lines 244–257:

```
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 244 行（name=r1b）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 248 行（name=r1b）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 250 行（name=r1b）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 256 行（name=r1b）
```

The other ten cells have the same form (`not_equal=0`, 6–40 hits). Individual rows are among 186 `name=l03_cell` lines (eight deletion publications: 30 rows for each of two intents, 23 for each of four non-final batches and 17 for each of two final batches). Accounting tag 6 is 98304 for every H1/H2 deletion version (n_live = 0, H = 0), equal to section 7 anchor A-tag6. The `deleted_inodes` root td is written at intent, not by batches, and the final batch zeroes tree table root pointer 65544. Tombstone record (1, 3, 0, 2⁶⁴ − 1, 5).

**Row 2** (unregistered bytes), lines 721–724:

```
E7RESULT name=r2a history=h1 total_bytes=1705984 layout03_cell_bytes=6824 layout01_cell_bytes=70700 constant_zero_bytes=1628460 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true
E7RESULT name=r2c history=h1 constant_zero_nonzero_bytes=0
E7RESULT name=r2a history=h2 total_bytes=3314176 layout03_cell_bytes=14904 layout01_cell_bytes=170003 constant_zero_bytes=3129269 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true
E7RESULT name=r2c history=h2 constant_zero_nonzero_bytes=0
```

Measurement basis (registration section 1.2, “read by position”): header byte ranges are fully assigned to cells using D18 settled item 18's code-2 offset field table, item 11's code-3 identity-section table, and layout/01's system configuration (line 158), root record (365–383), and journal record header (359). Unit test `layout_field_tables_tile_their_registered_totals` pins their widths to totals 86, 107 and 466. Entries are identified by tree and width; unrecognized entries remain unregistered. Two qualifications: the instance-table unit (`ti` in H2's row-writing publication) has its row-record field sequence only in D18 item 11 lines 302–307, referenced by layout/01 line 385, and counts as registered by following that reference. The 88-byte tag-1 payload in an extent upper-leaf entry holds an 86-byte lower-root pointer; the remaining two bytes lie inside the offset range of the cell on layout/01 line 288 and count as registered by position, but the provision does not say what they are (both are zero).

**Row 3** (segment and kind strings): all four H1 rows match, lines 259–262 and 267; the full string:

```
E7RESULT name=r3d history=h1 expected=26+2+1+2+24+2+1+2+22+2+1+2+20+2+1+2 actual=26+2+1+2+24+2+1+2+22+2+1+2+20+2+1+2 equal=true
```

On line 259 (intent), the role cells are `expected_roles=13:t3,t4,t5,t5@0.0.61,t5@0.1.61,t5@1.0.0,t5@1.1.0,t6,t7,t8,td,tt,tt@0 actual_roles=13:…（同） roles_equal=true`; root slot `expected_root=0:7344128 actual_root=0:7344128`. H2's analogous rows are reported but excluded from the verdict (`counts_for_row3=false`); continuation 2 → 1 has `actual_segments=26+2+1+2` and the final batch `24+2+1+2` (each adds a leaf pair), unlike layout/03 lines 53 and 55. Using the corresponding H1 geometry as their expected value necessarily yields red (discriminative power self-test, registration section 8.2).

**Row 4** (whether continuation and session batches have the same shape), lines 269–323:

```
E7RESULT name=r4a data_units_after=2 continuation_txg=11 session_txg=6 continuation_units=12 session_units=12 continuation_only= session_only= same_shape=true
E7RESULT name=r4a data_units_after=1 continuation_txg=12 session_txg=7 continuation_units=13 session_units=11 continuation_only=t5@0.0.62,t5@0.1.62 session_only= same_shape=false
E7RESULT name=r4b history=h2 txg=12 role=t5@0.0.62 previous_in_window=false changed_records=12 allocated_in_this_publish=12 released_in_this_publish=0 reclaimed_under_the_floor=0 unmatched=0 has_clause_basis=true
E7RESULT name=r4a data_units_after=0 continuation_txg=13 session_txg=8 continuation_units=12 session_units=10 continuation_only=t5@0.0.62,t5@0.1.62 session_only= same_shape=false
E7RESULT name=r4b history=h2 txg=13 role=t5@0.0.62 previous_in_window=true changed_records=24 allocated_in_this_publish=12 released_in_this_publish=12 reclaimed_under_the_floor=0 unmatched=0 has_clause_basis=true
```

The two `t5@0.1.62` rows have the same counts as `t5@0.0.62` (lines 284, 323). Geometry (`name=r4c`, two geometry-sensitive points for row 4): H1's three session batches land at 50283–50318 and write leaf 61 only. H2 continuation txg 11 lands at 50330–50342 (leaf 61 only, same shape as session), txg 12 at 50343–50356 and txg 13 at 50357–50368, both writing leaves 61 and 62. Leaf 62 covers [50344, 51156) (812 slots per leaf, layout/01 line 298). Third point H4 was not run after sufficient evidence.

**Row 5** (H1 half), line 357:

```
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 357 行（name=r5a）
```

**Row 6:** Ten `name=r6a` lines: change counts for H1's intent and two batches and H2's two continuation batches (the final batch does not write this record) equal the publication's checkpoint_txg; each batch copies the previous version's three timestamp seconds (all 1788000060); zero discriminable mismatches. The intent cell is `discriminable=false` because the export's caller intent time is the record's existing time (registration section 4 item 8).

**Row 7**, lines 370–375:

```
E7RESULT name=r7 history=h1 txg=5 role=td level=0 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false
E7RESULT name=r7 history=h1 txg=5 role=tt level=1 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false
E7RESULT name=l03_11_record history=h1 txg=5 identity_reference_container_birth=5 container_header_birth=5 judged=false
```

Both H2 rows have the same values. Each tree contains one entry. Under subtree coverage the ranges at these nodes are [0, 2⁶⁴ − 1] (td) and [1, 2⁶⁴ − 1] or [0, 2⁶⁴ − 1] (tt), unlike the written [1, 1], allowing the “entry-key range” label. Correctness belongs to C478 (Is the Key Range of the Code-2 Header the Entry Key Range or the Subtree Coverage Range), level to C713 (No provision for the two deletion-tree root levels); neither is judged here.

**Trajectory** (values used as predicate inputs by the provisions, lines 694–710): H1 `deleted_inodes` peaks at 1, is positive in 3 versions and ends at 0; data units peak at 3, are positive 3 times and end at 0; tag 6 is 98304 in all four versions. H2 has the same shape (8 positive versions, including unmount floor raise, row write and warm-up); tag 4 is 0 in both histories.

#### First run: four A-journal anchor mismatches (section 10 F3: check both sides, not invalidating)

Lines 15, 21, 27 and 31; the first is:

```
E7RESULT name=anchor anchor=A-journal history=h1 txg=5 expected=0:16793600,1:16793600 actual=0:16801792,1:16801792 equal=false
```

All four differ by 8192 bytes (two record slots). The other 48 anchor lines match (`anchor_not_equal=4`).

- Provision side: layout/01 line 319, “record n lands at `(counter − 1) mod slot count × 4096`,” holds in the window. Reading journal headers from the committed export gives H1's four deletion publications jsn counters 7, 8, 9, 10 and offsets 16801792, 16805888, 16809984, 16814080, each equal to 16 MiB + (counter − 1) × 4096. The registered anchor adds “counter = txg” (warm-ups 1, 2, new-file creation 3, sequential write 4), which is not in the provision: layout/01 only specifies counters 1, 2, 3 in the new-pool file creation stream.
- Implementation side: the intent record's transaction number (offset 78) is 5, so four transactions already existed in the same instance. D16 settled item 5's splitting rule “at most one unit of user data per transaction” makes the three-unit sequential write three transactions; D23 settled item 7 has one record per transaction, so the sequential-write publication writes three records (counters 4, 5, 6). This is inferred: pre-window bytes are not exported, only the write count agrees—101 pre-window writes = mkfs 21 + acquisition 2 + warm-up 10 + new-file creation 29 + sequential write 39; 39 = 15 units × 2 + 3 records × 2 + root slot 1 + system configuration 2.
- Classification: the registration's anchor derivation is wrong, not a mismatch between implementation and layout/01. It belongs neither to row 1's L03 cell table nor to row 3's segment string, so neither verdict changes.
- Read command (repository root, committed export read-only): `python3 -c "…"` takes `name=device_region_bytes` lines for device-0 writes with magic `SFSJ` and reads instance generation, counter, checkpoint_txg and transaction at offsets 16, 20, 26, 78. The first four output lines verbatim: `h1 step 28 offset 16801792 jsn_instance 1 jsn_counter 7 checkpoint_txg 5 transaction 5 sequence_in_publish 1 flags 1`, `h1 step 63 offset 16805888 jsn_instance 1 jsn_counter 8 checkpoint_txg 6 transaction 6 sequence_in_publish 1 flags 1`, `h1 step 96 offset 16809984 jsn_instance 1 jsn_counter 9 checkpoint_txg 7 transaction 7 sequence_in_publish 1 flags 1`, `h1 step 127 offset 16814080 jsn_instance 1 jsn_counter 10 checkpoint_txg 8 transaction 8 sequence_in_publish 1 flags 1`.

#### First run: row 8 and amendment 1

- R8.a: in the main workspace `MountOutput` at `crates/singlefs-core/src/mount.rs` lines 919–965, `grep -n 'writes\|acquir'` has no hits and `grep -n 'acquisition_writes\|writes_of_the_acquisition\|instance_acquisition' crates/singlefs-core/src/mount.rs` has no hits. Its 19 `pub` fields (`instance`, `chosen_root`, `effective_root`, `journal`, `rows_written`, `row_publish`, `warm_up_publishes`, `shadow_ledger`, `shadow_ledger_branch`, `isolated_slots_per_device`, `abandoned_roots_unreadable`, `space_admission`, `parameters_and_device_table`, `rereads`, `deletion_queue_continuation`, `floor_raises_pushed_by_the_continuation`, `refusals_that_pushed_the_continuation_floor_raises`, `deletion_queue_continuation_outcome`, `instance_table_sweep`) do not return number acquisition writes ⇒ the “cannot” branch: the export can partition `mount_writes_before_its_first_publish` only by “mount-part write count − sum of publication write accounts.”
- R8.b (original registration order, rotation first): two of 156 mount-part writes misattributed, lines 377–378:

```
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 377 行（name=r8b）
产物行见 git show 18b30adbfe94:e172-deletion-write-dry-run-2026-10-05.out 第 378 行（name=r8b）
```

  These are number acquisition writes (instance generation 1 → 2, tail remains 9); tail 9 equals the counter of the unmount part's txg 7 publication and follows its root slot. Both original branches match, so rotation-first attributes them to the instance-1 publication.
- R8.c (amendment 1: acquisition first, rotation additionally requires matching instance generation): zero misattributions, two number acquisition writes. The segment cut by subtraction agrees with per-write attribution on H2.
- Positive controls: PC-E (target R8.b), line 719 `E7RESULT name=r_pc pc=PC-E history=h2 target_green_before=false target_red_after=true`: target already red before mutation. PC-F (target R8.c), line 720 `target_green_before=true target_red_after=true`. Literally section 11 V4 invalidates all H2 verdicts when PC-E is red before mutation, affecting row 4 (all continuation batches are H2), row 6's H2 half and row 8. Whether this is a faulty criterion itself (`.claude/singlefs-ai-sop/rules/evidence-discipline.md`, “criteria can themselves be wrong: identify which case after a hit”: both R8.b branches match these writes, so the hit reflects unspecified precedence, not the export partition), and whether row 8 and V4 use R8.b or R8.c, are referred to the main agent. The other eight controls (PC-A–PC-D in H1 and H2) were green before and red after (lines 711–718).

#### First run: unit tests and mutations

- 26 unit tests (`grep -c '^test tests::.* ok$'` on the final 2026-10-05 `cargo test --release --bin e172-deletion-write-dry-run` log gives 26). Sixteen read the committed export and fail if it is absent; ten do not depend on it (CRC32C check value, SHA-256 check value, three capacity calculations, tag-6 formula, file byte count, root-slot/journal/system-configuration formulas, field-table widths, key-range reading, synthetic segmentation, synthetic padding coverage).
- Mutation table `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` has 20 entries: registered section 9 M1–M4 and M6–M19 (M1b, M5 and M10b are sensitive only on H3 and belong to segment three), plus M20 (code-2 payload CRC starts after reserved bits) and M21 (do not check whole-unit checksum in the parent-pointer position entry). Each anchor matches exactly once in source (Python count); the complete table had not yet run and was assigned to the main agent (registration section 5.6 step 5).

#### First run: what this cannot answer

1. Only batch size 1 (test-only knob), two 4 GiB disks and a file of three data units were measured. With product batch size 144, a three-unit file's first batch is its final batch, so the two non-final cells cannot be reached (registration section 8.2). Row 1 says “measured only at batch size 1.”
2. Row 5's variant half (another size-0 inode in the same leaf) and row 6's intent-time cell require H3, available only in segment three. In H1/H2 the caller's intent time equals the record's existing time and is not discriminable.
3. Row 2's “0” is positional: a byte in a registered cell offset range counts as registered regardless of content. The two remaining payload bytes of extent upper-leaf entry tag 1 and the instance-table row record rely only on D18 settled item 11's field sequence: their positions are registered but content is incomplete in provisions or only in the decision.
4. Row 7 records values only: each tree root has one entry. Correctness of level and key range belongs to C478 (Is the Key Range of the Code-2 Header the Entry Key Range or the Subtree Coverage Range) and C713 (No provision for the two deletion-tree root levels).
5. Row 4 has only two geometry points: session batches all land in leaf 61, while two continuation batches cross into leaf 62. “A session batch crossing into leaf 62 also writes that leaf” is inferred, not measured (H4 was not run after sufficient evidence).
6. The window begins before intent publication; mkfs, acquisition, warm-up, new-file creation and sequential-write bytes are not in the export. The A-journal section's pre-window inference relies only on write counts.
7. The apparatus compares implementation with KB registration. It cannot see a shared KB/implementation error when the KB wording does not constrain the cell, and does not judge whether the provision itself is correct.
8. The main agent ran the 20 mutations on 2026-10-05: “20 ✅”; “counts: 0 hit memory ceiling (3G cap), 0 timed out, 0 unschedulable, 0 excluded by global limit”; final line “restored, baseline still all green” (`e172-mutate-2026-10-05.log`). This establishes only that asserted points turn red, not coverage. M1b, M5 and M10b are sensitive only in H3 and belong to segment three.
9. The row 8 and V4 verdict depends on the registered R8.b order and is referred to the main agent.

#### First run: paths and conclusions register (source line numbers refer to the first-run apparatus)

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Role set: decoder identifies bytes | Classify each unit by offset, length and magic; identify it by tree ID, level, packed-record type, container number, device segment and position ordinal at the start of the allocation record tree key range. Tree roots use root record and tree table root pointer. Scan every unit write in both history windows, without sampling. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:1090 role_identity_of` (name: `:1115 role_name_of`) | Offsets from layout/01 lines 175–190, 259–263, 285–290, 297–299, 386–387 and D18 settled items 11, 18; allocation record tree leaves have 812 slots and level 1 covers 137228 slots (layout/01 line 298); role naming copied from `TransactionUnit::tag` at `crates/singlefs-core/src/transaction.rs` line 2357 |
| 2 | Role set: rewritten roles returned by `crates/` | Core returns `TransactionOutput::rewritten` at each publication; `TransactionUnit::tag` names them and export writes `rewritten_roles=` on boundary rows. | `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:80 segment_boundary_fields` | `TransactionUnit::tag` naming |
| 3 | Segment attribution: decoder attributes each write | Units use header birth generation, journal records and root slots use checkpoint_txg, and system-configuration slots use tail and instance generation (calculate both original R8.b order and amendment 1 R8.c reading). Count every mount-part write. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:2291 attribute_mount_writes` | layout/01 lines 152–153 (tail, instance generation), 186 (birth generation), 344–345 (jsn, checkpoint_txg), 373 (root-record checkpoint_txg) |
| 4 | Segment attribution: export partition | Partition directly invoked publications by recorded-stream length before and after the call; partition unmount and mount by each publication's write account returned by core. Calculate the part before the mount row write by subtracting the sum of publication write accounts from the mount-part write count. | `crates/singlefs-harness/src/scenario.rs:687 unmount_then_mount_continuation` | Core `TransactionOutput::writes` |

Shared-input challenge: paths 1 and 2 share role naming, so S3 cannot see a naming error shared by both. Row 4 therefore judges shape using path 1's identities, not names (R4.a). Paths 3 and 4 share no code, only the recorded stream. Cross-path comparisons in source: `role_name_comparison` (same file line 1952) compares paths 1 and 2's multisets; `is_match` in `attribute_mount_writes` compares paths 3 and 4. Unit tests `decoded_structure_passes_every_stop_check_on_the_export` and `mount_writes_are_attributed_to_the_same_segments_the_export_cuts` pin both results on the committed export.

#### First run: decisions affected (review on the first-run date)

| Decision item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure), settled item 16 | Supporting material | 2026-10-05 unaffected: tested provision; intent, each batch, final batch and continuation bytes equal layout/03's registration cell by cell (row 1 all equal). Continuation 2 → 1 and final batches each write an additional allocation record tree leaf 62 pair with a provision basis (row 4 B). Layout/03 line 56, “each batch considered alone has the same shape as a session batch of the same geometry (inferred, dry-run verification pending),” does not hold for H2. That line belongs to layout/03; the item text itself does not specify the continuation batch role set. The item's evidence paragraph does not cite E172. |
| D8 (Core Index Structure), settled item 14 | Supporting material | 2026-10-05 unaffected: allocation record tree is position-addressed; leaf 62's identity is determined by key-range start ÷ 812 and the extra leaf follows the landing position. |
| D18 (Information Carried in a Block), settled item 24 | Supporting material | 2026-10-05 unaffected: tombstone record, tombstone container and tombstone-tree root equal layout/03 lines 14–17 and 30–36 cell by cell. |
| D18 (Information Carried in a Block), settled item 18 | Supporting material | 2026-10-05 unaffected: every code-2 unit in the window passes header checksum, payload CRC and entry count × entry width = declared length (S2 is 0); zero nonzero bytes in the constant-zero padding class. |
| D18 (Information Carried in a Block), settled item 11 | Supporting material | 2026-10-05 unaffected: code-3 identity sections are fully assigned to cells; registration of the instance-table row record relies only on this item's field sequence (row 2 counts it as registered). |
| D16 (Publish Semantics), settled item 5 | Supporting material | 2026-10-05 unaffected: the A-journal mismatch section uses it to infer three records from the pre-window sequential-write publication (inferred). |
| D23 (Journal Role & Format), settled item 7 | Supporting material | 2026-10-05 unaffected: the same section uses one record per transaction. |

#### Recorded when the page was first created

Preregistration `e172-preregistration.md`; export `e172-deletion-write-dump-2026-10-05.out`; apparatus output `e172-deletion-write-dry-run-2026-10-05.out`. Section 12 amendment 1 was written after the unit tests and before output: row 8 adds R8.c and PC-F while R8.b and PC-E remain unchanged. The 20-entry mutation table was written but not yet run.
