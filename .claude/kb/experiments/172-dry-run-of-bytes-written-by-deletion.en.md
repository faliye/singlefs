## E172 Dry Run of Bytes Written by Deletion — run (sufficient to decide): third rerun (C775 (E172 apparatus lacks retirement items)) decides row 2 of the question sheet as A; with retirement items added, the pinned-value layer, the provision layer and the discriminating arm all equal what `crates/` writes today cell by cell, the 23 in-scope positive-control pairs are green before and red after, and all 49 mutations were caught; the eight A-journal mismatches are reported as before and do not enter row 2; the second run's five-row readings are kept in the historical versions

[中文](172-删除写出的字节的干跑.md) · **English** · [日本語](172-削除で書き出されるバイトのドライラン.ja.md)

<!-- doc-lint:not-numbers L03 M31 M42 M43 M44 M45 M46 M47 -->

The third-rerun preregistration is `research/prompts/e172-r3-prereg.md` (amendments 1–5 in section 12 were written by the runner after the apparatus was changed, amendment 6 and amendments 7.1–7.4 before the product; the original criteria are kept unchanged; amendment 7.2, which changes PC-G's expectation, was fixed by the main agent in the dispatch prompt and is not a tightening). It answers row 2 of the question sheet `research/prompts/m3-close-b19-p5-forks.md` (C775 (E172 apparatus lacks retirement items): A, the apparatus with retirement items added equals `crates/` field by field / B, some field differs). The second-rerun preregistration `e172-r2-prereg.md` (reading B), its question sheet `e172-r2-questions.md` and the first preregistration `e172-preregistration.md` have their readings under Historical Versions. The tested provisions are all of `.claude/kb/layout/03-third-txn.md` (the rows “retirement container”, “tree-table retirement-tree entry” and “value of accounting tag 6” and the section 5 retirement record are new in this run); the system-configuration field table in `.claude/kb/layout/01-first-txn.md` (line 149 T_time “5000 (in milliseconds, i.e. 5 seconds)”, offset 433 on line 158); the offset tables of the root record and the journal record header (lines 360–388) and the section 8 segment sequences; D8 (Core Index Structure) settled item 16 and settled item 6 (inode record offsets 64–95) and settled item 9 (tree-kind code table, retirement tree code 13); D16 (Publish Semantics) settled item 5; and D3 (Space Allocation) settled item 2 (the tag 6 formula).

**Current status** (third run): row 2 of the question sheet is decidable and decided as **A**. With the retirement items added to the apparatus (the three retirement slots and 35 × n_records in the tag 6 formula, retirement container tr@0 and retirement tree root tr, 12 tree-table entries and watermark 65549, new cells L03-15–L03-17), every cell in row 2's scope in the product `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out` (966 lines) equals the bytes `crates/` writes today; none of invalidation conditions V1–V7 or stop conditions S0–S4 fired, the discriminating arm separates in every deletion version, and all 23 in-scope positive-control pairs were green before and red after.

- Pinned-value layer: the tag 6 decoded at every H1 and H2 deletion publication equals the number pinned on line 198 of the harness test — 196 678 for intent, 3 → 2 and 2 → 1, and 196 608 for the final batch (8 rows `equal=true`, `pin_not_equal=0`, product lines 929–944); H2 equals H1 version by version (four `harness_pin_geometry` rows `same_value=true`). The 196 760 cell is the version before deletion and lies outside the window, so it was **not compared version by version** (one `harness_pin_out_of_window` row per history, `in_window=false`); the version in the window that equals 196 760 is the final H3 batch, for which only the same value was reported (line 952 `same_value_as_harness_first=true`); that is the formula's value and does not count as compared with the pinned number.
- Provision layer, tag 6 and tree table: all twelve A-tag6 rows are equal (H1 and H2 196 678 → 196 608, H3 196 830 → 196 760, literal constants, and the two constant sets judged against each other leave exactly one green); the L03-13 formula with four inputs is equal in 4 rows per history; L03-12 (12 tree-table entries, key range [65536, 65548], the retirement-tree entry) and L03-14 (watermark 65549) are equal in every version of every history.
- Provision layer, the retirement cells: in the final H1 and H2 batches L03-15, L03-16 and L03-17 report one retirement container written, one retirement record (birth tree 65537, packed record type 2, container number 1, container birth 3, death txg = the txg of this publication: 8 for H1, 13 for H2) and the retirement tree root; no other deletion publication and no H3 publication wrote them; `retirement_cells_not_equal=0`, also 0 for each history separately; R1.b has at least one hit per cell per history.
- Provision layer, roles, segment strings and coverage: the 9 R3 rows that enter the judgment (four H1 deletion publications, H2's intent, four H3 deletion publications) are equal in segment string, type string, roles and root slot; the full-window segment strings of H1 and H3 each equal the four registered strings; the final H1 and H2 batches gain tr@0 and tr, the final H3 batch still writes inode leaf t3 and no retirement unit, and judging each history with both role expectations leaves exactly one green; unregistered bytes, nonzero bytes in the constant-zero class and doubly claimed bytes are 0 in all three histories.
- Discriminating arm: the 12 R-ret rows are `discriminating=true` — in every deletion version the expectation with the retirement share equals the decoded value and the one without it (98 304 for H1 and H2, 98 386 for H3) differs; zero rows where both expectations are equal (V7 did not fire).
- Positive controls: the product has 33 `name=r_pc` rows; all 25 applicable pairs were green before and red after, and the 8 inapplicable pairs lie within what the registration states; the 23 in-scope pairs (without the incidental PC-F and PC-G) were all green before and red after, `positive_controls_failed=0`. PC-G turns red on H2 owing to the expectation changed by amendment 7.2 (see “What was done”).
- The second run's set of cells: R1.a, R1.b, R1.t, R2.b, R3.e and R7 are read as in the second run, `not_equal_cells=0` and `t_time_not_5000=0`; the quantities of the second sheet's other four rows are reported incidentally and do not enter row 2.
- Mismatches, reported: eight A-journal rows `equal=false`, each 8192 bytes short of the actual value (identical, row by row, to the eight rows of the second product), owing to the registration's wrong premise for the anchor; they do not enter row 2 (amendment 7.4). All eight A-system-configuration rows are `equal=true` — the generation formula for this cell was changed in the apparatus from `txg + 2` to `txg + 3` (amendment 7.3; acquisition now writes two rounds).
- Not run after sufficiency: R-flip and R-dump (registration 6.4; not counted when row 2 is decided A).

The model is deterministic: two runs of the same binary give the same product sha256 (`6a31bf33…`), and one more run of each of the two exports is byte-identical to the checked-in file; this establishes absence of hidden state, not statistical stability. The strength of the verdict rests on unit tests fixing absolute values, positive controls and the mutation table. The tested item of C775 (the apparatus lacks retirement items) now exists in the apparatus; whether this owed check can be paid off is for the main agent to decide.

### What was done

- The apparatus `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` was changed in place per section 5.8 changes 16–30 of the preregistration (not a byte of `crates/` changed): the tag 6 expectation now uses the formula with the retirement share (the `2 + H_retirement + 1` slots and 35 × n_records), and A-tag6 becomes literal constants per (history, deletion publish); it recognizes the `tr@<leaf order>` and `tr` units; when the final batch empties the leaf, the registered roles gain tr@0 and tr; the tree table expects 12 entries, the key range [65536, 65548] and the retirement-tree entry; the watermark expects 65549; new cells L03-15 (retirement container), L03-16 (retirement record), L03-17 (retirement tree root); the pinned-value layer `name=harness_pin` (against line 198 of the harness test), the discriminating arm `name=r_ret` (expectations with and without the retirement share on the same reading), positive controls PC-J, PC-K, PC-L (25 applicable pairs); the verdict line `name=verdict row=c775`; and `main` calls the acceptance entry `admit_experiment_run_or_exit("E172")` first.
- The unit tests that read the two checked-in exports failed in two places on their first run (task 4893, `50 passed; 2 failed`); both causes were in the apparatus, not in `crates/`, and amendments 7.1–7.3 changed the apparatus before the product: ① the decoding constants stood before C773 (Reclamation threshold pins for bad and all-zero root slots only in memory) — journal record header 487 → 496 and root record 457 → 466 (the reclaim threshold t and the pinned slot p entered the new root segment and the root record, layout/01 lines 360–388); an investigator computed CRC32C record by record: of the 34 journal records in the three histories, 34/34 equal the stored value when computed from 496 and 0/34 from 487; ② PC-G's expectation had been fixed for one acquisition round, and after D18 (What Information a Block Carries) settled item 11 changed this to two rounds it contradicted the “original attribution” half; it now takes only the original attribution (fixed by the main agent in the dispatch prompt, not a tightening; R8.c itself is unchanged); ③ the generation number in A-system-configuration `txg + 2` → `txg + 3`. The A-journal expectation formula is not changed (amendment 7.4): tasks 5070 and 5071 checked both sides — the publication of the sequential write before the h1 window writes 3 journal records, and in the window the counter = txg + 2; in the copy with per-data-unit record splitting switched off the intent falls back on the apparatus's expectation; what is wrong is “the sequential write records only one counter” in the registration's derivation.
- 52 unit tests pass: task 5082 `test result: ok. 52 passed; 0 failed`, including the 30 that read the two r3 exports; `NOT_RUN_ARCHIVED_PRODUCT` appears 0 times.
- The mutation table `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` has 49 rows (the second run's 32, M31–M42, and M43–M47: decoding constants changed back to 487 / 457, width table without the reclaim threshold and pinned slot, PC-G's expectation changed back to acquisition, generation changed back to `txg + 2`); the main agent ran it as task 5087: “计数：抓到 49 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 49 条、复用 0 条、拒算 0 条” (caught 49, not red 0, invalid 0, unfinished 0, name blind spots 0, broken anchors 0, judged now 49, reuse 0, refused 0), last line “已还原，基线仍全绿”, 49 `^✅` lines, and M43–M47 each go red on the registered unit tests.
- Product: task 5086 built (`build_exit=0`, `warnings=0`) and the same binary ran twice, `run1_exit=0`, `run2_exit=0`, `cmp_exit=0`, both sha256 `6a31bf33c014defc56bcdc84e3beffc4d91238b341d4c1cff90479993580a12d`, copied to `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out` (966 lines; the first line is the acceptance input-fingerprint line `E7INPUT name=input_fingerprint key=E172 …`). One more run of each of the two exports (tasks 4795, 4796) as task 4891 is byte-identical to the checked-in files, first 16 hex digits of sha256 `229934e319f1f9e6` and `f8a72c560fb7d76f`; before starting, S6 (the harness test `tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image` green, line 198 pinning `196_760, 196_678, 196_678, 196_678, 196_608`) was checked by the main agent as task 4703.
- The E172 (Dry Run of Bytes Written by Deletion) row of the replay table `scripts/experiment-apparatus/replay.sh` points to `e172-r3-deletion-write-dry-run-2026-11-02.out`; the second product and the two second-run exports, the three comparison objects, are not registered as units (three “not registered” lines in `.claude/gate.d/replay-comparison-units.tsv`), owed in C930 (E172 has no comparison unit table), and are not needed today (R-flip and R-dump did not run).

### Replay

```
# 仓根：两份导出（峰值表里实测 30M、17M）
task-server/task submit -- bash -c 'cargo run -q -p singlefs-verification --bin deletion_write_dump > <H1H2 导出>'
task-server/task submit -- bash -c 'cargo run -q -p singlefs-verification --bin deletion_write_dump -- --history h3 > <H3 导出>'
# 装置：在仓根经服务编并跑（读两份导出，先 H1H2 后 H3；头一行是准入的输入指纹）
task-server/task submit -- cargo run --release --manifest-path scripts/experiment-apparatus/Cargo.toml --bin e172-deletion-write-dry-run -- research/results/e172-r3-deletion-write-dump-h1h2-2026-11-02.out research/results/e172-r3-deletion-write-dump-h3-2026-11-02.out
# 登记的复跑（driver_e172：先现跑两份导出进临时文件，再跑装置；输入没变时准入记「输入没变」、不跑）
task-server/task submit -- bash scripts/experiment-apparatus/replay.sh --admission-only E172
task-server/task submit -- bash scripts/experiment-apparatus/replay.sh E172
# 单测 52 条与变异表 49 条
task-server/task submit --cwd scripts/experiment-apparatus -- cargo test --release --bin e172-deletion-write-dry-run
task-server/task submit --cwd scripts/experiment-apparatus -- bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutation/tables/e172_deletion_write_dry_run.tsv
```

### Verdict (lines copied intact from `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out`)

`grep -n 'name=verdict' research/results/e172-r3-deletion-write-dry-run-2026-11-02.out` gives 11 lines, copied intact:

```
E7RESULT name=verdict row=1 not_equal_cells=0 discriminable_false_rows=2 t_time_not_5000=0 not_equal_cells_h1=0 cells_without_hits_h1=0 cells_without_hits_list_h1=none not_equal_cells_h2=0 cells_without_hits_h2=0 cells_without_hits_list_h2=none not_equal_cells_h3=0 cells_without_hits_h3=0 cells_without_hits_list_h3=none t_time_not_5000_h1=0 t_time_not_5000_h2=0 t_time_not_5000_h3=0
E7RESULT name=verdict row=3 h1_segment_mismatches=0 h1_kind_mismatches=0 h1_role_mismatches=0 h1_root_mismatches=0 h1_window_string_equal=true h2_segment_mismatches=0 h2_kind_mismatches=0 h2_role_mismatches=0 h2_root_mismatches=0 h2_window_string_equal=not_reported h3_segment_mismatches=0 h3_kind_mismatches=0 h3_role_mismatches=0 h3_root_mismatches=0 h3_window_string_equal=true
E7RESULT name=verdict row=4 pairs_with_unique_roles=2 unique_units_without_basis=0
E7RESULT name=verdict row=5 h3_final_writes_t3=true h3_matches_registered=true
E7RESULT name=verdict row=6 r6a_not_equal_rows=0 r6a_discriminable_false_rows=1 r6b_not_equal_rows=0 r6b_discriminable_false_rows=0 r6b_intent_differs_from_sequential=true
E7RESULT name=verdict row=7 recorded_nodes=8
E7RESULT name=verdict row=8 attributed_writes=162 r8c_misattributed_writes=0
E7RESULT name=verdict row=2 unregistered_bytes=0 constant_zero_nonzero_bytes=0 double_claimed_bytes=0
E7RESULT name=verdict row=c775 pin_not_equal=0 committed_reservation_anchor_not_equal=0 clause_not_equal=0 retirement_cells_not_equal=0 retirement_arm_not_discriminating=0 retirement_arm_both_expectations_equal=0 positive_controls_failed_in_scope=0 pin_not_equal_h1=0 retirement_cells_not_equal_h1=0 pin_not_equal_h2=0 retirement_cells_not_equal_h2=0 retirement_cells_not_equal_h3=0
E7RESULT name=verdict check=stops s2_structure_failures=0 s3_role_mismatches=0 s3_index_notation_only=0 s3_l03_7_second_branch_not_exactly_h3=0 stop_retirement_unit_histories_not_registered=0 s4_tiling_failures=0
E7RESULT name=verdict check=voids a_lines_equal_h1h2=true a_lines_equal_h3=true a_crc_equal=true histories_present=true a_content_equal_h1=true configuration_matches_h1=true a_content_equal_h2=true configuration_matches_h2=true a_content_equal_h3=true configuration_matches_h3=true positive_controls_failed=0 positive_controls_failed_h1=0 positive_controls_failed_h2=0 positive_controls_failed_h3=0 positive_controls_run=25 positive_controls_expected=25 anchor_not_equal=8
```

Product lines 537, 561, 657, 661, 675, 687, 850, 928, 961, 964 and 965. Row 2 of the question sheet is read directly from the `row=c775` line (line 961): `pin_not_equal`, `committed_reservation_anchor_not_equal`, `clause_not_equal`, `retirement_cells_not_equal`, `retirement_arm_not_discriminating`, `retirement_arm_both_expectations_equal` and `positive_controls_failed_in_scope` are all 0, as are the per-history `*_h1`, `*_h2`, `*_h3` counts; the six counts of `check=stops` are all 0 (line 964); the flags of `check=voids` are `true`, with `positive_controls_failed=0`, `positive_controls_run=25` and `positive_controls_expected=25` (line 965). No verdict field has the value `false` or `not_run`. The fields that are neither zero nor true, named one by one:

- Line 965 `anchor_not_equal=8`: the eight A-journal rows, within the registration's expectation (amendment 7.4), not part of row 2; apart from these eight the product has no `equal=false` at all (`grep -n ' equal=false' P | grep -v ' anchor=A-journal '` prints nothing).
- Line 537 `discriminable_false_rows=2` and line 675 `r6a_discriminable_false_rows=1`: the `times` field of L03-5 has the same three time seconds at the intent publication of H1 and H2 (`expected` and `actual` are both `1788000060/1788000060/1788000060`, `equal=true discriminable=false`, product lines 92 and 229; the R6.a copy is line 663), because the caller's time at intent in the export is the time the record already had, so this cell cannot tell “copied” from “changed”. The values equal the second product's (`discriminable_false_rows=2`, `r6a_discriminable_false_rows=1`); not part of row 2.
- Line 561 `h2_window_string_equal=not_reported`: fixed by the registration as not reported (the H2 window contains segments that are not deletion publications, and only the intent publication of H2 enters the segment-string judgment), as in the second run.
- Line 687 `recorded_nodes=8` and line 850 `attributed_writes=162`: record-only quantities, 6 and 156 in the second run, the reason was not traced, not part of row 2.

The verdict lines keep the row numbers of the first question sheet: `row=1, 2, 3, 7` are row 1 of the second sheet, `row=4` is row 2, `row=5` is row 3, `row=6` is row 4, `row=8` is row 5; `row=c775` is row 2 of this run's sheet.

### Readings for each row (lines copied intact from `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out`)

Below “P” is `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out`, and commands start at the repository root.

**Completeness (V2, A-lines)**: the first line, the last line and the line counts of the two exports (lines 1, 2, 4 and 966):

```
E7INPUT name=input_fingerprint key=E172 sha256=5af5127dca7199809c5565518c74b17eca3408c7ca89ebf55a8b1a867105e621 units=4118
E7RESULT name=export_file file=h1h2 sha256=229934e319f1f9e6b634d91cd60ffb584dfdff627159dad95d2170f5489abc38 lines=474 anchored_lines=474 done_emitted=474 non_hexadecimal_lines=121 malformed_lines=0
E7RESULT name=export_file file=h3 sha256=f8a72c560fb7d76f0c5624deb07c7d2156504a982680008db8bbb99064947e55 lines=150 anchored_lines=150 done_emitted=150 non_hexadecimal_lines=36 malformed_lines=0
E7RESULT name=done emitted=965
```

`grep -c '^E7INPUT ' P` → `1`, `grep -c '^E7RESULT ' P` → `965`, `wc -l < P` → `966`, equal to the last line's `emitted=965`; the exports' `lines=474` and `lines=150` equal the line counts of the checked-in files, the two `A-lines` rows are `equal=true` (lines 3 and 5), and `A-crc` on line 6 is `equal=true`. The acceptance fingerprint line `key=E172 … units=4118` is not `sha256=refused`.

**Pinned-value layer (P-pin)**: `grep '^E7RESULT name=harness_pin ' P | wc -l` → `8`, `… | grep -c ' equal=true$'` → `8`, `… | grep -c ' equal=false$'` → `0`.

```
E7RESULT name=harness_pin history=h1 txg=5 publish=intent harness_index=2 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h1 txg=6 publish=three_to_two harness_index=3 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h1 txg=7 publish=two_to_one harness_index=4 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h1 txg=8 publish=final harness_index=5 pinned=196608 actual=196608 equal=true
E7RESULT name=harness_pin history=h2 txg=5 publish=intent harness_index=2 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h2 txg=11 publish=three_to_two harness_index=3 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h2 txg=12 publish=two_to_one harness_index=4 pinned=196678 actual=196678 equal=true
E7RESULT name=harness_pin history=h2 txg=13 publish=final harness_index=5 pinned=196608 actual=196608 equal=true
```

The out-of-window, same-value and version-by-version same-value rows:

```
E7RESULT name=harness_pin_out_of_window history=h1 harness_index=1 pinned=196760 in_window=false
E7RESULT name=harness_pin_out_of_window history=h2 harness_index=1 pinned=196760 in_window=false
E7RESULT name=harness_pin_out_of_window history=h3 harness_index=1 pinned=196760 in_window=false
E7RESULT name=harness_pin_same_value history=h3 txg=9 actual=196760 same_value_as_harness_first=true
E7RESULT name=harness_pin_geometry publish=intent session_actual=196678 continuation_actual=196678 same_value=true
E7RESULT name=harness_pin_geometry publish=three_to_two session_actual=196678 continuation_actual=196678 same_value=true
E7RESULT name=harness_pin_geometry publish=two_to_one session_actual=196678 continuation_actual=196678 same_value=true
E7RESULT name=harness_pin_geometry publish=final session_actual=196608 continuation_actual=196608 same_value=true
```

**A-tag6 and A-system-configuration**: `grep '^E7RESULT name=anchor anchor=A-tag6 ' P | wc -l` → `12`, `… | grep -c ' equal=true$'` → `12`.

```
E7RESULT name=anchor anchor=A-tag6 history=h1 txg=5 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h1 txg=6 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h1 txg=7 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h1 txg=8 expected=196608 actual=196608 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h2 txg=5 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h2 txg=11 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h2 txg=12 expected=196678 actual=196678 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h2 txg=13 expected=196608 actual=196608 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h3 txg=6 expected=196830 actual=196830 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h3 txg=7 expected=196830 actual=196830 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h3 txg=8 expected=196830 actual=196830 equal=true
E7RESULT name=anchor anchor=A-tag6 history=h3 txg=9 expected=196760 actual=196760 equal=true
E7RESULT name=a_tag6_geometry history=h1 own_constants_green=true constants_of=h3 other_constants_green=false exactly_one_green=true
E7RESULT name=a_tag6_geometry history=h3 own_constants_green=true constants_of=h1 other_constants_green=false exactly_one_green=true
```

The eight A-system-configuration rows (generation `txg + 3`, amendment 7.3): `grep '^E7RESULT name=anchor anchor=A-system-configuration ' P | wc -l` → `8`, `… | grep -c ' equal=true$'` → `8`.

```
E7RESULT name=anchor anchor=A-system-configuration history=h1 txg=5 expected=0:0:generation8,1:0:generation8 actual=0:0:generation8,1:0:generation8 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h1 txg=6 expected=0:4096:generation9,1:4096:generation9 actual=0:4096:generation9,1:4096:generation9 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h1 txg=7 expected=0:0:generation10,1:0:generation10 actual=0:0:generation10,1:0:generation10 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h1 txg=8 expected=0:4096:generation11,1:4096:generation11 actual=0:4096:generation11,1:4096:generation11 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h3 txg=6 expected=0:4096:generation9,1:4096:generation9 actual=0:4096:generation9,1:4096:generation9 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h3 txg=7 expected=0:0:generation10,1:0:generation10 actual=0:0:generation10,1:0:generation10 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h3 txg=8 expected=0:4096:generation11,1:4096:generation11 actual=0:4096:generation11,1:4096:generation11 equal=true
E7RESULT name=anchor anchor=A-system-configuration history=h3 txg=9 expected=0:0:generation12,1:0:generation12 actual=0:0:generation12,1:0:generation12 equal=true
```

**L03-12 – L03-17**, per cell and per history (`for c in L03-12 … L03-17; for h in h1 h2 h3; … rows= not_equal= discriminable_true=`, output as printed):

```
L03-12 h1 rows=28 not_equal=0 discriminable_true=28
L03-12 h2 rows=28 not_equal=0 discriminable_true=28
L03-12 h3 rows=28 not_equal=0 discriminable_true=28
L03-13 h1 rows=4 not_equal=0 discriminable_true=4
L03-13 h2 rows=4 not_equal=0 discriminable_true=4
L03-13 h3 rows=4 not_equal=0 discriminable_true=4
L03-14 h1 rows=12 not_equal=0 discriminable_true=12
L03-14 h2 rows=12 not_equal=0 discriminable_true=12
L03-14 h3 rows=12 not_equal=0 discriminable_true=12
L03-15 h1 rows=5 not_equal=0 discriminable_true=5
L03-15 h2 rows=5 not_equal=0 discriminable_true=5
L03-15 h3 rows=4 not_equal=0 discriminable_true=4
L03-16 h1 rows=9 not_equal=0 discriminable_true=9
L03-16 h2 rows=9 not_equal=0 discriminable_true=9
L03-16 h3 rows=4 not_equal=0 discriminable_true=4
L03-17 h1 rows=4 not_equal=0 discriminable_true=4
L03-17 h2 rows=4 not_equal=0 discriminable_true=4
L03-17 h3 rows=4 not_equal=0 discriminable_true=4
```

`grep '^E7RESULT name=l03_cell cell=L03-1[5-7] ' P | grep -v ' equal=true '` prints nothing; `grep -n 'name=r1b' P | grep -c 'discriminable_hits=0 '` → `0` (`name=r1b` has 51 lines in all). The L03-12 – L03-17 rows of the final H1 batch (txg 8, retirement tree written) and of the final H3 batch (txg 9, inode 2 remains in the leaf, retirement tree zero-rooted):

```
E7RESULT name=l03_cell cell=L03-12 field=entry_count history=h1 txg=8 publish=unlink_batch expected=12 actual=12 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree_table_key_range history=h1 txg=8 publish=unlink_batch expected=65536/65548 actual=65536/65548 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65544 history=h1 txg=8 publish=unlink_batch expected=kind9/head65537/root_zerotrue actual=kind9/head65537/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65545 history=h1 txg=8 publish=unlink_batch expected=kind10/head65537/root_zerofalse actual=kind10/head65537/root_zerofalse equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65546 history=h1 txg=8 publish=unlink_batch expected=kind11/head0/root_zerotrue actual=kind11/head0/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65547 history=h1 txg=8 publish=unlink_batch expected=kind12/head0/root_zerotrue actual=kind12/head0/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65548 history=h1 txg=8 publish=unlink_batch expected=kind13/head65537/root_zerofalse actual=kind13/head65537/root_zerofalse equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-13 field=committed_reservation_tombstone_levels0_retirement_levels0_live0_records0 history=h1 txg=8 publish=unlink_batch expected=196608 actual=196608 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=root_record_watermark history=h1 txg=8 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=journal_watermark_device0 history=h1 txg=8 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=journal_watermark_device1 history=h1 txg=8 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=retirement_records_written history=h1 txg=8 publish=unlink_batch expected=1 actual=1 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-15 field=retirement_containers_written history=h1 txg=8 publish=unlink_batch expected=1 actual=1 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-15 field=retirement_container history=h1 txg=8 publish=unlink_batch expected=code3/type3/width34/birth_tree65548/records_at136/container1/count1 actual=code3/type3/width34/birth_tree65548/records_at136/container1/count1 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=birth_tree history=h1 txg=8 publish=unlink_batch expected=65537 actual=65537 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=packed_record_type history=h1 txg=8 publish=unlink_batch expected=2 actual=2 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=container_number history=h1 txg=8 publish=unlink_batch expected=1 actual=1 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=container_birth history=h1 txg=8 publish=unlink_batch expected=3 actual=3 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=death_txg history=h1 txg=8 publish=unlink_batch expected=8 actual=8 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-17 field=retirement_root history=h1 txg=8 publish=unlink_batch expected=tag2/tree65548/key_width8/width120/count1/separator1/birth_tree65548/type3/container1/children0:50318,1:50318 actual=tag2/tree65548/key_width8/width120/count1/separator1/birth_tree65548/type3/container1/children0:50318,1:50318 equal=true discriminable=true
```

```
E7RESULT name=l03_cell cell=L03-12 field=entry_count history=h3 txg=9 publish=unlink_batch expected=12 actual=12 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree_table_key_range history=h3 txg=9 publish=unlink_batch expected=65536/65548 actual=65536/65548 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65544 history=h3 txg=9 publish=unlink_batch expected=kind9/head65537/root_zerotrue actual=kind9/head65537/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65545 history=h3 txg=9 publish=unlink_batch expected=kind10/head65537/root_zerofalse actual=kind10/head65537/root_zerofalse equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65546 history=h3 txg=9 publish=unlink_batch expected=kind11/head0/root_zerotrue actual=kind11/head0/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65547 history=h3 txg=9 publish=unlink_batch expected=kind12/head0/root_zerotrue actual=kind12/head0/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-12 field=tree65548 history=h3 txg=9 publish=unlink_batch expected=kind13/head65537/root_zerotrue actual=kind13/head65537/root_zerotrue equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-13 field=committed_reservation_tombstone_levels0_retirement_levels0_live1_records1 history=h3 txg=9 publish=unlink_batch expected=196760 actual=196760 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=root_record_watermark history=h3 txg=9 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=journal_watermark_device0 history=h3 txg=9 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-14 field=journal_watermark_device1 history=h3 txg=9 publish=unlink_batch expected=65549 actual=65549 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-16 field=retirement_records_written history=h3 txg=9 publish=unlink_batch expected=0 actual=0 equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-15 field=retirement_container_written history=h3 txg=9 publish=unlink_batch expected=false actual=false equal=true discriminable=true
E7RESULT name=l03_cell cell=L03-17 field=retirement_root_written history=h3 txg=9 publish=unlink_batch expected=false actual=false equal=true discriminable=true
```

The L03-15 – L03-17 rows of the final H2 batch (txg 13) are on lines 335–343; its death txg is 13 (line 342).

**R3 final batch and full window, R2.a / R2.c**: `grep '^E7RESULT name=r3 ' P | grep ' counts_for_row3=true ' | wc -l` → `9`, and the `grep -c` of those with `segments_equal=false`, `kinds_equal=false`, `roles_equal=false` or `root_equal=false` → `0`.

```
E7RESULT name=r3d history=h1 expected=26+2+1+2+24+2+1+2+22+2+1+2+24+2+1+2 actual=26+2+1+2+24+2+1+2+22+2+1+2+24+2+1+2 equal=true
E7RESULT name=r3d history=h3 expected=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 actual=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 equal=true
E7RESULT name=r3_geometry history=h3 txg=9 expected_only_record=24+2+1+2 expected_other_records=22+2+1+2 actual=22+2+1+2 only_record_green=false other_records_green=true expectations_differ=true exactly_one_green=true
E7RESULT name=r3_geometry_roles history=h1 txg=8 only_record_green=true other_records_green=false exactly_one_green=true
E7RESULT name=r3_geometry_roles history=h3 txg=9 only_record_green=false other_records_green=true exactly_one_green=true
```

```
E7RESULT name=r2a history=h1 total_bytes=1804288 layout03_cell_bytes=8732 layout01_cell_bytes=72102 constant_zero_bytes=1723454 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true
E7RESULT name=r2c history=h1 constant_zero_nonzero_bytes=0
E7RESULT name=r2a history=h2 total_bytes=3420672 layout03_cell_bytes=18812 layout01_cell_bytes=172858 constant_zero_bytes=3229002 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true
E7RESULT name=r2c history=h2 constant_zero_nonzero_bytes=0
E7RESULT name=r2a history=h3 total_bytes=1771520 layout03_cell_bytes=8424 layout01_cell_bytes=76428 constant_zero_bytes=1686668 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true
E7RESULT name=r2c history=h3 constant_zero_nonzero_bytes=0
E7RESULT name=verdict row=2 unregistered_bytes=0 constant_zero_nonzero_bytes=0 double_claimed_bytes=0
```

`grep -c '^E7RESULT name=r2b ' P` and `grep -c '^E7RESULT name=r2c_range ' P` both give `0`. In `name=r2a`, `layout01_cell_bytes` and `constant_zero_bytes` move with the decoding constants (amendment 7.1: every journal record and every root-slot write has 9 more bytes claimed by layout/01 cells and 9 fewer in the constant-zero padding); no unit test pins these two numbers, and only `unregistered_bytes=0`, `double_claimed_bytes=0` and `sum_matches_total=true` guard them.

**R-ret (discriminating arm)**: `grep '^E7RESULT name=r_ret ' P | wc -l` → `12`, `… | grep -c ' discriminating=true$'` → `12`, `… | grep -c ' with_equal=true '` → `12`, `… | grep -c ' without_equal=true '` → `0`, and no line has both `with_equal=true` and `without_equal=true`.

```
E7RESULT name=r_ret history=h1 txg=5 publish=intent with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h1 txg=6 publish=three_to_two with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h1 txg=7 publish=two_to_one with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h1 txg=8 publish=final with_retirement=196608 without_retirement=98304 actual=196608 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h2 txg=5 publish=intent with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h2 txg=11 publish=three_to_two with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h2 txg=12 publish=two_to_one with_retirement=196678 without_retirement=98304 actual=196678 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h2 txg=13 publish=final with_retirement=196608 without_retirement=98304 actual=196608 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h3 txg=6 publish=intent with_retirement=196830 without_retirement=98386 actual=196830 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h3 txg=7 publish=three_to_two with_retirement=196830 without_retirement=98386 actual=196830 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h3 txg=8 publish=two_to_one with_retirement=196830 without_retirement=98386 actual=196830 with_equal=true without_equal=false discriminating=true
E7RESULT name=r_ret history=h3 txg=9 publish=final with_retirement=196760 without_retirement=98386 actual=196760 with_equal=true without_equal=false discriminating=true
```

**Positive controls (R-pc)**: `grep '^E7RESULT name=r_pc ' P | wc -l` → `33`, `… | grep -c ' applies=true '` → `25`, `grep -c` of `target_green_before=true target_red_after=true` without PC-F and PC-G → `23`, with them → `25`, no `applies=true` line is anything but green-then-red, and `grep -c 'target_red_after=not_run' P` → `0`. The 8 inapplicable rows and samples of PC-F, PC-G, PC-J, PC-K and PC-L:

```
E7RESULT name=r_pc pc=PC-F history=h1 applies=false reason=no_mount_writes_before_its_first_publish
E7RESULT name=r_pc pc=PC-G history=h1 applies=false reason=no_mount_writes_before_its_first_publish
E7RESULT name=r_pc pc=PC-I history=h1 applies=false reason=impl_config_without_intent_write_time_seconds
E7RESULT name=r_pc pc=PC-I history=h2 applies=false reason=impl_config_without_intent_write_time_seconds
E7RESULT name=r_pc pc=PC-F history=h3 applies=false reason=no_mount_writes_before_its_first_publish
E7RESULT name=r_pc pc=PC-G history=h3 applies=false reason=no_mount_writes_before_its_first_publish
E7RESULT name=r_pc pc=PC-J history=h3 applies=false reason=no_retirement_container_written
E7RESULT name=r_pc pc=PC-L history=h3 applies=false reason=no_retirement_root_written
E7RESULT name=r_pc pc=PC-J history=h1 applies=true target_green_before=true target_red_after=true
E7RESULT name=r_pc pc=PC-K history=h1 applies=true target_green_before=true target_red_after=true
E7RESULT name=r_pc pc=PC-L history=h1 applies=true target_green_before=true target_red_after=true
E7RESULT name=r_pc pc=PC-F history=h2 applies=true target_green_before=true target_red_after=true
E7RESULT name=r_pc pc=PC-G history=h2 applies=true target_green_before=true target_red_after=true
```

**Trajectories (R-traj)**:

```
E7RESULT name=r_traj_summary history=h1 series=data_units peak=3 positive_versions=3 final=0
E7RESULT name=r_traj_summary history=h1 series=deleted_inodes peak=1 positive_versions=3 final=0
E7RESULT name=r_traj_summary history=h1 series=inode_records peak=1 positive_versions=3 final=0
E7RESULT name=r_traj_summary history=h1 series=retirement_levels peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h1 series=retirement_records peak=1 positive_versions=1 final=1
E7RESULT name=r_traj_summary history=h1 series=tag4 peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h1 series=tag6 peak=196678 positive_versions=4 final=196608
E7RESULT name=r_traj_summary history=h2 series=data_units peak=3 positive_versions=8 final=0
E7RESULT name=r_traj_summary history=h2 series=deleted_inodes peak=1 positive_versions=8 final=0
E7RESULT name=r_traj_summary history=h2 series=inode_records peak=1 positive_versions=8 final=0
E7RESULT name=r_traj_summary history=h2 series=retirement_levels peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h2 series=retirement_records peak=1 positive_versions=1 final=1
E7RESULT name=r_traj_summary history=h2 series=tag4 peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h2 series=tag6 peak=196678 positive_versions=9 final=196608
E7RESULT name=r_traj_summary history=h3 series=data_units peak=3 positive_versions=3 final=0
E7RESULT name=r_traj_summary history=h3 series=deleted_inodes peak=1 positive_versions=3 final=0
E7RESULT name=r_traj_summary history=h3 series=inode_records peak=2 positive_versions=4 final=1
E7RESULT name=r_traj_summary history=h3 series=retirement_levels peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h3 series=retirement_records peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h3 series=tag4 peak=0 positive_versions=0 final=0
E7RESULT name=r_traj_summary history=h3 series=tag6 peak=196830 positive_versions=4 final=196760
```

**T_time (R1.t)**:

```
E7RESULT name=r1t_summary history=h1 system_configuration_slot_writes=8 t_time_not_5000=0
E7RESULT name=r1t_summary history=h2 system_configuration_slot_writes=24 t_time_not_5000=0
E7RESULT name=r1t_summary history=h3 system_configuration_slot_writes=8 t_time_not_5000=0
```

**Eight A-journal mismatches**:

```
E7RESULT name=anchor anchor=A-journal history=h1 txg=5 expected=0:16793600,1:16793600 actual=0:16801792,1:16801792 equal=false
E7RESULT name=anchor anchor=A-journal history=h1 txg=6 expected=0:16797696,1:16797696 actual=0:16805888,1:16805888 equal=false
E7RESULT name=anchor anchor=A-journal history=h1 txg=7 expected=0:16801792,1:16801792 actual=0:16809984,1:16809984 equal=false
E7RESULT name=anchor anchor=A-journal history=h1 txg=8 expected=0:16805888,1:16805888 actual=0:16814080,1:16814080 equal=false
E7RESULT name=anchor anchor=A-journal history=h3 txg=6 expected=0:16797696,1:16797696 actual=0:16805888,1:16805888 equal=false
E7RESULT name=anchor anchor=A-journal history=h3 txg=7 expected=0:16801792,1:16801792 actual=0:16809984,1:16809984 equal=false
E7RESULT name=anchor anchor=A-journal history=h3 txg=8 expected=0:16805888,1:16805888 actual=0:16814080,1:16814080 equal=false
E7RESULT name=anchor anchor=A-journal history=h3 txg=9 expected=0:16809984,1:16809984 actual=0:16818176,1:16818176 equal=false
```

All eight differ by 8192 bytes (two record slots) and are identical, row by row, to the eight rows of the second product (lines 19, 25, 31, 35, 64, 70, 76, 80 of that product). Rows and equal counts of each `name=anchor` kind (`grep '^E7RESULT name=anchor ' P | sed 's/.*anchor=\([^ ]*\) .*equal=\(true\|false\)$/\1 \2/' | sort | uniq -c`, output as printed):

```
      9 A-blocks true
      4 A-capacity true
      3 A-content true
      1 A-crc true
      8 A-journal false
      2 A-lines true
     17 A-root-slot true
      9 A-size true
      8 A-system-configuration true
     12 A-tag6 true
      5 A-txg true
```

### Eight A-journal anchor mismatches (section 10 F3: check both sides, not invalidating)

Lines 21, 27, 33, 37, 66, 72, 78 and 82; the first H3 row is:

```
E7RESULT name=anchor anchor=A-journal history=h3 txg=6 expected=0:16797696,1:16797696 actual=0:16805888,1:16805888 equal=false
```

All eight differ by 8192 bytes (two record slots) and are identical, row by row, to the eight rows of the second product; the other 70 anchor rows are equal (`anchor_not_equal=8`, 78 `name=anchor` rows).

- The provision side: line 319 of layout/01, “record n lands at `(counter − 1) mod slot count × 4096`”, holds in the window. Reading the writes on device 0 whose magic is `SFSJ` from the checked-in H3 export (instance code, counter, checkpoint_txg and transaction number at offsets 16, 20, 26 and 78), output as printed: `h3 step 28 offset 16805888 jsn_instance 1 jsn_counter 8 checkpoint_txg 6 transaction 6`, `h3 step 63 offset 16809984 jsn_instance 1 jsn_counter 9 checkpoint_txg 7 transaction 7`, `h3 step 96 offset 16814080 jsn_instance 1 jsn_counter 10 checkpoint_txg 8 transaction 8`, `h3 step 129 offset 16818176 jsn_instance 1 jsn_counter 11 checkpoint_txg 9 transaction 9`; each equals 16 MiB + (counter − 1) × 4096. The four H1 rows are counters 7–10 (`h1 step 28 offset 16801792 … jsn_counter 7 checkpoint_txg 5`, then steps 63, 96 and 131 with 8, 9 and 10). This was read again from the two checked-in third-run exports and agrees, row by row, with what the second-run page recorded.
- The registration side: section 7 A-journal of the registration derives “counter = txg” (warm-up 1 and 2, new-pool file creation 3, creating inode 2 gives 4, the sequential write 5). F3 checked both sides (amendment 7.4, probes in copies, run by the main agent): task 5070 — the publication of the sequential write before the h1 window (txg 4) writes 3 journal records with counters 4 / 5 / 6, and in the window the counter = txg + 2; task 5071 (the copy with per-data-unit record splitting switched off, the refuting condition) — the h1 intent (txg 5) has counter 5 at offset 16793600 and the h3 intent (txg 6) has counter 6 at offset 16797696, equal to the apparatus's expectation. So a counter larger than the txg by 2 is “the sequential write writes 2 more records”; the provision is not wrong, what is wrong is “the sequential write records only one counter” in the registration's derivation.
- Classification: the registration's anchor derivation is wrong, and the implementation does not disagree with layout/01; the cell is not in the L03 cell table and does not enter the segment strings, so the judgment of row 2 is unaffected (amendment 7.4: the expectation formula is not changed and not reversed by the readings; all eight rows are reported).

### Unit tests and mutations

- 52 unit tests (task 5082, 2026-11-02, `cargo test --release --bin e172-deletion-write-dry-run`: `test result: ok. 52 passed; 0 failed`, including the 30 that read the two r3 exports; `NOT_RUN_ARCHIVED_PRODUCT` appears 0 times), 14 more than the second run's 38. The added and changed ones cover role names and claiming of the retirement units, the 12-entry tree table and the watermark, a synthetic sampling point of the L03-13 formula on the retirement share (229 446), the two expectations of the discriminating arm, the version-by-version comparison of the pinned-value layer, how PC-J, PC-K and PC-L are built and their resealed checksums, and what amendments 7.1–7.3 changed: the decoding constants and width table, the A-system-configuration expectation and the PC-G expectation. The synthetic tests pin only the expectation column, the structure and synthetic inputs; the 30 that read the exports run decoding, the stop checks and the cross-comparison on the two checked-in exports.
- The mutation table `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` has 49 rows: the second run's 32, M31–M42 (the retirement batch) and M43–M47 (five places of amendments 7.1–7.3: journal header 496 changed back to 487, root record 466 back to 457, width table without the reclaim threshold and pinned slot, PC-G's expectation back to acquisition, generation back to `txg + 2`). Each anchor matches exactly once in the source (`rows 49 not_unique 0`). The main agent's task 5087 finished: “计数：抓到 49 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 49 条、复用 0 条、拒算 0 条” (caught 49, not red 0, invalid 0, unfinished 0, name blind spots 0, broken anchors 0, judged now 49, reuse 0, refused 0), last line “已还原，基线仍全绿”. M44 and M45 go red in the same unit test, each catching its own direction (constant changed back, table changed back); no unit test pins the concrete `layout01_cell_bytes` and `constant_zero_bytes` of `name=r2a`, so mutations do not guard them.

### What this cannot answer

“Row N” in items 1–9 below refers to the second question sheet `e172-r2-questions.md`; what row 2 of this run's sheet (C775 (E172 apparatus lacks retirement items)) cannot answer is in items 10–13.

1. Only batch size 1, two 4 GiB disks and one file of three data units were measured (row 1 says “measured only at batch size 1”). H3 adds only one size-0 record; cases with two or more additional records in the leaf or where the emptied leaf is not the final leaf were not measured.
2. T_time has only one sample: there is no mount-options channel (C422), so the implementation writes the built-in default in all three histories. R1.t judges “four exported bytes = 5000 from layout/01 line 149,” and PC-H establishes discriminative power. It cannot answer whether a different mount-time value would be written correctly.
3. Row 5 was measured only at the H2 sample: the instance generation changes across number acquisition and the two number acquisition writes have tail equal to the counter of the unmount part's final publication. Each of R8.c's two requirements beyond R8.b has a synthetic unit test with a sensitive sample (T11, T12); opposite geometry cannot be produced in the export. “Subtraction splits correctly” applies only to this export.
4. Row 2 has only three geometry points (H1 session batches, H3 session batches, H2 continuation batches), and only the continuation batches cross into leaf 62. “A session batch crossing leaf 62 would also write that leaf” is an inference (H4 was not run after sufficient evidence).
5. Row 1 coverage (R2) reads by position: bytes within a registered cell's offset range count as registered, regardless of what that cell contains. The remaining two payload bytes of extent upper-leaf entry tag 1 and the instance-table row record rely only on the field sequence of D18 (Information Carried in a Block), settled item 11.
6. The td and tt header levels and key ranges are recorded without judgment: the level clauses are D8 (Core Index Structure) settled item 16 item 3 (td) and D18 (Information Carried in a Block) settled item 24 item 1 (tt), and the key range clause is D18 (Information Carried in a Block) settled item 2 (td is a leaf-level code 2 node and takes the keys of its first and last entries; tt takes the subtree coverage range); the tt key range is judged by the pool-level checker, and the td key range is also judged by the pool-level checker in the tree table walk (`read_index_node` in `crates/singlefs-checker/src/walk.rs` takes the `FirstAndLastEntries` reading and calls `check_index_node_keys` in `crates/singlefs-checker/src/lib.rs`; a header range that differs from the keys of the first and last entries turns I-1.1 (block header states its logical address) red). The final H3 batch's inode-tree root key range [1, 1] is also recorded without judgment.
7. The window starts before the intent publication: bytes for mkfs, number acquisition, warm-up, new-pool file creation, inode 2 creation and sequential write are outside the export. The A-journal section's pre-window derivation relies only on write counts and counters.
8. The apparatus compares implementation with KB registration: it cannot see a shared KB/implementation error when the KB wording does not constrain that cell. The question sheet does not ask whether fields of the H3 inode 2 record (rewritten as a consequence) are copied, so that is not judged.
9. The main agent ran all 32 mutations on 2026-10-05: “32 ✅”, “counts: 0 hit memory ceiling (3G cap), 0 timed out, 0 unschedulable, 0 excluded by the global limit”; the last line says “restored, baseline still all green” (`e172-r2-mutate-2026-10-05.log`). This establishes only that the asserted locations turn red, not coverage.
10. The pinned-value layer was compared version by version only with that one harness test: H1 is the same kind of history (a three-unit file, batch size 1, two disks, intent followed by three batches in one process), H2 equals H1 version by version, and H3 has no pinned number; the 196 760 cell lies outside the window and was not compared, and the 196 760 of the final H3 batch was reported only as the same value. Batch size has only the single sampling point 1; the value of tag 6 for batch size > 1 or for larger files was not measured.
11. Geometry of the retirement items: whether the retirement tree is written was measured at only two sampling points (the deleted record is the only one in the leaf: H1 and H2 write tr@0 and tr; another record remains in the leaf: H3 does not); the levels H_retirement and H_tombstone of the retirement and tombstone trees were measured at the single sampling point 0 (neither tree grows taller in the three histories), and the formula's sensitivity at H ≥ 1 is given by synthetic unit tests (`committed_reservation_with_one_retirement_level_and_one_dead_record_is_229446` and others), not measured on the exports; the retirement container holds a single record (n_records = 1), and its limit of 959 records is arithmetic (A-arith) and was not built.
12. The provision layer compares “the bytes and offsets the apparatus takes from the kb registrations” with “the bytes `crates/` writes”: when the kb and the implementation are wrong together and the kb's wording does not constrain that cell, this cannot see it. The two mismatches seen this time (the apparatus's decoding constants standing before C773 (Reclamation threshold pins for bad and all-zero root slots only in memory), and the derivations behind the expectations of A-journal and A-system-configuration) were both the apparatus or the registration lagging behind the kb, not the kb disagreeing with the implementation.
13. PC-G's expectation is now “R8.c's original attribution of that write” (amendment 7.2), which shows that moving a segment boundary turns R8.c's rows red; branch ① of R8.c recognizes only the first acquisition round, and the two system-configuration slot writes of the second acquisition round are attributed to `other_mount_step` (product lines 690 and 691); this is the reading of an incidental quantity, was not judged right or wrong, and does not enter row 2.

### Paths and conclusions register

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Role set: decoder identifies bytes | Classify each unit by offset, length and magic; identify it by tree ID, level, packed record type, container number, device segment and position-based ordinal at the start of the allocation record tree key range. Tree roots use the root record and tree table root pointer. Scan every unit write in all three history windows, without sampling. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:1131 role_identity_of` (name: `:1156 role_name_of`) | Offsets from layout/01 lines 175–190, 259–263, 285–290, 297–299, 386–387 and D18 settled items 11, 18; allocation record tree leaves have 812 slots and level 1 covers 137228 slots (layout/01 line 298); role naming copied from `TransactionUnit::tag` in `crates/singlefs-core/src/transaction.rs` |
| 2 | Role set: rewritten roles returned by `crates/` | Core returns `TransactionOutput::rewritten` at each publication; `TransactionUnit::tag` names them and the export writes `rewritten_roles=` on the boundary row. | `crates/singlefs-verification/src/bin/deletion_write_dump.rs:119 segment_boundary_fields` | `TransactionUnit::tag` naming |
| 3 | Segment attribution: decoder attributes each write | Units use the header birth generation, journal records and root slots use checkpoint_txg, and system-configuration slots use R8.c (test acquisition first; rotation requires tail equal to that publication's jsn counter and matching instance generation). Count every write during mount. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:2653 attribute_mount_writes` (system-configuration slots: `:2617 attribute_system_configuration`) | layout/01 lines 152–153 (tail, instance generation), 186 (birth generation), 344–345 (jsn, checkpoint_txg), 373 (root-record checkpoint_txg) |
| 4 | Segment attribution: export partition | Partition directly invoked publications by recorded-stream length before and after the call; partition unmount and mount by each publication's write account returned by core. Calculate the part before the mount row write by subtracting the sum of publication write accounts from the mount-part write count. | `crates/singlefs-verification/src/scenario.rs:780 unmount_then_mount_continuation` | Core `TransactionOutput::writes` |

Shared-input challenge: paths 1 and 2 share the role naming convention, so S3 cannot see a naming error shared by both. Row 2 of the second sheet therefore judges shape using path 1's identities, not names (R4.a). Paths 3 and 4 share no code, only the recorded stream. Cross-path comparisons in source: `role_name_comparison` (same file line 2278) compares the multisets from paths 1 and 2; `is_match` in `attribute_mount_writes` compares paths 3 and 4. Unit tests `decoded_structure_passes_every_stop_check_on_the_export` and `mount_writes_are_attributed_to_the_same_segments_the_export_cuts` pin both results on the committed export.

### Decisions affected

| Decision item | Relationship | 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, Review |
|---|---|---|
| D8 (Core Index Structure), settled item 16 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D8 (Core Index Structure), settled item 6 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D8 (Core Index Structure), settled item 14 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 24 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 18 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 11 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D18 (Information Carried in a Block), settled item 2 | Does not affect | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: the body cites it only as the source of the key range reading; conclusions and artifacts unchanged |
| D16 (Publish Semantics), settled item 5 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D23 (Journal Role & Format), settled item 7 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: the third rerun produced its product (row 2 of the question sheet decided as A) and read the retirement items, the tag 6 formula and the decoding constants; this item's basis and conclusion are unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-29 unaffected: the third rerun only changed the apparatus (retirement items added); the exports and the product have not run yet, so conclusions and products are unchanged; earlier 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-18 unaffected: previous artifacts archived in the repository (recoverable from git history); text references changed to filenames only; conclusions and numbers unchanged. |
| D3 (Space Allocation), settled item 2 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: this row is new. In the third product, tag 6 in every deletion version of the three histories equals the formula with the retirement term (twelve A-tag6 rows and eight pinned-value rows all equal, and the twelve R-ret rows of the discriminating arm separate); the basis section of this item does not cite E172 (Dry Run of Bytes Written by Deletion), so whether this cell should be promoted to supporting is left to the main agent |
| D8 (Core Index Structure), settled item 9 | Supporting material | 2026-11-05 unaffected: the rename only changes wording; conclusions, numbers, and artifacts unchanged; previously, 2026-11-02 unaffected: this row is new. The 12 tree-table entries, the retirement tree's tree ID 65548 and kind code 13, and watermark 65549 equal the registration in every deletion version of the three histories (L03-12 and L03-14 all equal); the basis section of this item does not cite E172 (Dry Run of Bytes Written by Deletion), so whether this cell should be promoted to supporting is left to the main agent |

## Historical Versions

### 2026-11-02: third run (rerun), row 2 of the question sheet decided as A

Rerun preregistration `research/prompts/e172-r3-prereg.md`, question sheet `research/prompts/m3-close-b19-p5-forks.md` row 2 (C775 (E172 apparatus lacks retirement items)); exports `research/results/e172-r3-deletion-write-dump-h1h2-2026-11-02.out` and `research/results/e172-r3-deletion-write-dump-h3-2026-11-02.out`, apparatus product `research/results/e172-r3-deletion-write-dry-run-2026-11-02.out` (966 lines); the E172 (Dry Run of Bytes Written by Deletion) row of `scripts/experiment-apparatus/replay.sh` points to this product. The page body now carries the third run's current status, what was done, verdict, readings, eight A-journal mismatches, unit tests and mutations (49 rows), what this cannot answer (items 10–13 added), paths and conclusions register (source line numbers updated to current values) and decisions affected (rows for D3 (Space Allocation) settled item 2 and D8 (Core Index Structure) settled item 9 added, every row reviewed again). The second run's current status, what was done, replay, verdict, readings, A-journal mismatches and unit tests and mutations were moved unchanged into the entry “2026-10-05: second run” below (headings changed to “Second run: …”), with the quoted second-product lines turned into pointers (that product was archived in 59ce5a865; get it with `git show 59ce5a865^:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`); the product line numbers mentioned in those sections are the second product's.

### 2026-10-05: second run (rerun), all five question-sheet rows decidable

Rerun registration `e172-r2-prereg.md`, question sheet `e172-r2-questions.md`; exports `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out` and `research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out`; apparatus output `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`. The E172 (Dry Run of Bytes Written by Deletion) row of `scripts/experiment-apparatus/replay.sh` was changed to point at this output. The 32-entry mutation table was written but not yet run. The first run's status and sections remain unchanged under “2026-10-05: page created, first and second segments run.”

(Product line numbers in the sections below are those of `e172-r2-deletion-write-dry-run-2026-10-05.out`; that product was archived in 59ce5a865, get it with `git show 59ce5a865^:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out`.)

#### Second run: current status

The second-run preregistration is `e172-r2-prereg.md` (reading B; the amendments section 12 is empty, with no amendments); the question sheet is `e172-r2-questions.md` (five rows mapping to rows 1–8 of the first sheet `m3-delete-dryrun-questions.md`). The first preregistration is `e172-preregistration.md`. The tested provisions are all of `.claude/kb/layout/03-third-txn.md`; the system-configuration field table in `.claude/kb/layout/01-first-txn.md` (line 149 T_time “5000 (in milliseconds, i.e. 5 seconds)”, offset 433 on line 158) and its section 8 segment sequences; D8 (Core Index Structure) settled items 16 and 6 (inode record offsets 64–95); and D16 (Publish Semantics) settled item 5.

**Current status** (second run): all five rows of `e172-r2-questions.md` are decidable, none of invalidation conditions V1–V6 or stop conditions S0–S4 fired, and all 18 applicable positive-control pairs were green before and red after mutation.

- Row 1 (rows 1, 2, 3 and 7 of the first question sheet on the 2026-10-05 export and H3) holds throughout: among 279 L03 cell rows in twelve deletion publications across H1, H2 and H3, zero discriminable rows differ, and every cell is hit in every history; T_time is 5000 in all 38 system-configuration slot writes in the three windows; zero unregistered bytes and zero nonzero bytes in the constant-zero class. Segment strings, type strings, roles and root slots match the registration for all H1 and H3 deletion publications and H2's intent publication, and the full-window segment strings of H1 and H3 match. The td and tt header levels and key ranges have the same values across the three histories (levels 0 / 1, [1, 1], marked as entry-key ranges and recorded without judgment).
- Row 2 (does a continuation batch have the same shape as a session batch of the same geometry, H2): no; the additional units have a basis in the provisions (B). The continuation 2 → 1 batch and final batch each write an extra pair for allocation record tree leaf 62; all changed records are attributed to allocation or freeing by that publication, with zero unmatched. These are the same readings as in the first run.
- Row 3 (H3: another size-0 inode is in the same leaf): as currently written. The final H3 batch writes inode leaf container t3 (inode 2 remains in the leaf); its role set matches the 2 → 1 batch, with empty differences in both directions.
- Row 4 (record offsets 64–95, H2 and H3): under this reading, the three timestamp seconds in H3's intent version equal the caller's time (60 seconds later than the sequential write and distinguishable); each batch copies them, and the change count equals that batch's txg. The same equalities hold for both H2 continuation batches.
- Row 5 (can writes before the mount's row write be split by publication, R8.c): core does not return the number acquisition write account (R8.a has zero hits), so only subtraction is possible. Subtraction splits H2 correctly (R8.c has zero misattributions and two number acquisition writes); PC-F and PC-G were green before and red after. Only the H2 sample was measured.

There is also a preregistered anchor mismatch (section 10 F3, not invalidating): A-journal understates eight rows across H1 and H3 by 8192 bytes, owing to the preregistration's assumption that “counter = txg.” This is the same erroneous derivation investigated in the first run; the second registration used that assumption again for H3.

The model is deterministic: both exports match byte-for-byte across two runs (three for H3), and apparatus results match byte-for-byte across two runs. This establishes absence of hidden state, not statistical stability; the strength of the verdict rests on unit tests fixing absolute values, positive controls and the mutation table.

#### Second run: what was done

- Read-only export on the `crates/` side (main workspace, 2026-10-05, after formalization batch 30 changed T_time to milliseconds): run `cargo build -q -p singlefs-checker-tier --bin deletion_write_dump`, then the resulting `target/debug/deletion_write_dump` directly. Without arguments (H1, H2), both runs had sha256 `b7e87c4ed73978b2f6d89a00449bb46cb70a4d36a5e7a5a614ea64464bb090c0`, committed as `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out` (462 lines). Three runs with `--history h3` had sha256 `9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e`, committed as `research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out` (150 lines). Before the build and before and after export, hashes were recorded for `deletion_write_dump.rs` (`9b3f565e…cb`), `scenario.rs` (`9d007793…59`) and the sha256 of the per-file sha256 values in `crates/singlefs-core/src/` (`85af2670…59`); all matched.
- The new H1H2 export differs from the first run's `e172-deletion-write-dump-2026-10-05.out`: `diff` shows 30 lines on each side, all `name=device_region_bytes` for the two system-configuration slots (offsets 0 and 4096), reflecting the four T_time bytes changed to milliseconds and the corresponding whole-slot checksum. Both are retained.
- The research apparatus `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs` was modified in place according to changes 1–15 in registration section 5.8: accept both exports; decode another four bytes from offset 433 of system configuration (R1.t); make the L03-5 intent cell discriminable when the export supplies `intent_write_time_seconds=`; branch L03-7 by the previous version's leaf record count; ignore allocation record tree leaf ordinal in H2/H3 role comparisons and report ordinals separately; delete R8.b and PC-E, add PC-G, PC-H and PC-I; give each of the two extra R8.c requirements its own constant. Copies of the first apparatus and mutation table are in `research/prompts/e172-r1-frozen/` (sha256 `a626e18a…4b`, `6e21494a…b3`).
- Output `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out` (817 lines, sha256 `4a03f73e37175aba3fe5169c2fe0a5c36110c16a1c66e36712db320a3f82537b`) was identical byte-for-byte across two runs of the same binary. The first result `e172-deletion-write-dry-run-2026-10-05.out` comes from a different apparatus reading a different export and does not match; it is retained but no longer registered for replay.

#### Second run: replay

```
# 仓根：两份导出（峰值表里实测 30M、17M）
task-server/task submit -- bash -c 'cargo run -q -p singlefs-checker-tier --bin deletion_write_dump > <H1H2 导出>'
task-server/task submit -- bash -c 'cargo run -q -p singlefs-checker-tier --bin deletion_write_dump -- --history h3 > <H3 导出>'
# research/：装置（峰值 88M）
task-server/task submit --cwd research -- cargo run --release --manifest-path ../scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e172-deletion-write-dry-run -- results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out results/e172-r2-deletion-write-dump-h3-2026-10-05.out
# 登记的复跑（driver_e172：先现跑两份导出进临时文件，再跑装置）
bash scripts/experiment-apparatus/replay.sh E172
# 单测 38 条与变异表 32 条
task-server/task submit --cwd scripts/experiment-apparatus -- cargo test --release --bin e172-deletion-write-dry-run
task-server/task submit --cwd scripts/experiment-apparatus -- bash ../mutation/mutate.sh e172-deletion-write-dry-run e172_deletion_write_dry_run.rs ../mutation/tables/e172_deletion_write_dry_run.tsv
```

#### Second run: verdict (lines copied intact from `e172-r2-deletion-write-dry-run-2026-10-05.out`; rows for artifacts moved out of the workspace are one-line pointers)

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 452 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 476 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 568 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 572 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 586 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 596 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 753 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 813 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 815 行（name=verdict）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 816 行（name=verdict）
```

Output lines 452, 476, 568, 572, 586, 596, 753, 813, 815 and 816. Verdict rows retain the first question sheet's numbering: `row=1,2,3,7` constitute row 1 of the second sheet; `row=4`, row 2; `row=5`, row 3; `row=6`, row 4; and `row=8`, row 5 (registration section 6.0). `discriminable_false_rows=2` is the H1 and H2 intent cell, where the caller's intent time in the export is the record's existing time; `r6a_discriminable_false_rows=1` is the H2 cell among these. `h2_window_string_equal=not_reported` is explicitly withheld by the registration because the H2 window contains segments that are not deletion publications. See “Eight A-journal anchor mismatches” for `anchor_not_equal=8`.

#### Second run: readings for each row (lines copied intact from `e172-r2-deletion-write-dry-run-2026-10-05.out`; rows for artifacts moved out of the workspace are one-line pointers)

**Row 1:** Count L03 cell rows by history and txg (`grep 'name=l03_cell' F | grep -o 'history=h[123] txg=[0-9]* publish=[a-z_]*' | sort | uniq -c`): H1 txg 5 / 6 / 7 / 8 have 30 / 23 / 23 / 17 rows respectively; H2 txg 5 / 11 / 12 / 13 have 30 / 23 / 23 / 17; H3 txg 6 / 7 / 8 / 9 have 30 / 23 / 23 / 17. Total 279 rows; `grep 'name=l03_cell' F | grep -c 'equal=false'` → `0`. The minimum H3 hit count per cell (lines 397–410) is 3, for example:

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 401 行（name=r1b）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 409 行（name=r1b）
```

The final H3 batch takes L03-7's second branch (inode 2 remains in the previous version's leaf), lines 378–381, 395–396 and 814:

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 379 行（name=l03_cell）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 380 行（name=l03_cell）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 381 行（name=l03_cell）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 395 行（name=l03_7_record）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 396 行（name=l03_7_branch）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 814 行（name=l03_7_branch_check）
```

The final H1 and H2 batches take the first branch (`branch=only_record`, lines 179 and 287). Accounting tag 6: each deletion version is 98304 for H1/H2 and 98386 for H3 (n_live = 1, H = 0; line 391 `field=committed_reservation_h0_live1 … expected=98386 actual=98386 equal=true`). T_time (R1.t, lines 419, 442, 451):

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 419 行（name=r1t_summary）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 442 行（name=r1t_summary）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 451 行（name=r1t_summary）
```

Segment strings (lines 473–475); H2's verdict includes only its intent publication (reading B). The three continuation batch strings are still reported: txg 12 and 13 are `26+2+1+2` and `24+2+1+2` (each has an extra leaf pair), and are not included in row 1.

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 473 行（name=r3d）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 474 行（name=r3d）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 475 行（name=r3_geometry）
```

H2/H3 role comparisons omit allocation record tree leaf ordinals (eight `name=r3c_ordinal … judged=false` lines). The ordinals of all four H3 publications and H2's intent are `t5@0.0.61,t5@0.1.61,t5@1.0.0,t5@1.1.0`, the same as registered for H1. Coverage (lines 811–812): `E7RESULT name=r2a history=h3 total_bytes=1771520 layout03_cell_bytes=6824 layout01_cell_bytes=75776 constant_zero_bytes=1688920 unregistered_bytes=0 double_claimed_bytes=0 sum_matches_total=true`, `E7RESULT name=r2c history=h3 constant_zero_nonzero_bytes=0`; H1 and H2 have the same numbers as the first run (lines 807–810). td and tt (lines 593–594): `E7RESULT name=r7 history=h3 txg=6 role=td level=0 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false`, `E7RESULT name=r7 history=h3 txg=6 role=tt level=1 key_range=1..1 entry_keys=1..1 reading=entry_key_range judged=false`; H1 and H2 have the same values.

**Row 2** (lines 477–478 and 505; all four `name=r4b` lines have `unmatched=0 has_clause_basis=true`):

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 477 行（name=r4a）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 478 行（name=r4a）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 505 行（name=r4a）
```

Geometry (`name=r4c`, lines 556–567): H1's three session batches have placements 50283–50318 and H3's four publications at 50280–50332; both write only leaf 61. H2 continuation txg 12 and 13 land at 50343–50368 and write leaves 61 and 62 (leaf 62 covers [50344, 51156)). H3 has one extra publication and its positions shift by 12 slots but still do not cross into leaf 62.

**Row 3** (lines 570–571):

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 570 行（name=r5b）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 571 行（name=r5b_discrimination）
```

The H1 half is incidental and excluded from the verdict, line 569: `E7RESULT name=r5a history=h1 two_to_one_txg=7 final_txg=8 only_in_two_to_one=t3 only_in_final=none matches_registered=true incidental=true`.

**Row 4** (six `name=r6a` and six `name=r6b` lines, all `equal=true`): the H3 intent cell and the two times, lines 580 and 585:

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 580 行（name=r6b）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 585 行（name=r6b_intent_time）
```

Both H3 batches (txg 7 and 8) copy all three cells from the intent version (1788000120), and change counts 6 / 7 / 8 equal the current txg. H2's intent cell is `discriminable=false`; the three cells in continuation txg 11 and 12 are 1788000060, with change counts 11 / 12.

**Row 5:** A current check in the main workspace for R8.a: `grep -n 'writes\|acquir'` on `MountOutput` in `crates/singlefs-core/src/mount.rs` exits 1 (no hits); `grep -n 'acquisition_writes\|writes_of_the_acquisition\|instance_acquisition' crates/singlefs-core/src/mount.rs` exits 1. Its 19 `pub` fields have the same names as in the first run, and none returns an account of number acquisition writes. All 156 R8.c writes have `match=true`; the two number acquisition writes (lines 597–598):

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 597 行（name=r8c）
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 598 行（name=r8c）
```

Positive controls (lines 783–806; 24 lines): 18 applicable pairs have `applies=true target_green_before=true target_red_after=true`; six inapplicable pairs are PC-F and PC-G in H1/H3 and PC-I in H1/H2, with `applies=false` and reasons. After mutation PC-F requires exactly one R8.c misattribution at the first write of the row-writing publication (`decoded=publish_txg8`, segment before the mount row write); PC-G requires exactly one at the final write of that part (step 96, system-configuration slot, `decoded=instance_acquisition`, segment `mount_instance_row_txg8`); PC-H requires exactly one T_time ≠ 5000, value 5, and the same number of whole-slot checksum failures as before mutation; PC-I requires the H3 intent cell to differ, with all three actual cells equal to the sequential-write time.

**Trajectory** (lines 755–782): H3 `deleted_inodes` peaks at 1, is positive 3 times and ends at 0; data units peak at 3, are positive 3 times and end at 0; tag 6 is 98386 in all four versions. H1/H2 have the same counts as the first run, and tag 4 is 0 in all three histories.

#### Second run: eight A-journal anchor mismatches (section 10 F3: check both sides, not invalidating)

Lines 19, 25, 31, 35, 64, 70, 76, 80; the first H3 line is:

```
产物行见 git show d4d697a9ce22:research/results/e172-r2-deletion-write-dry-run-2026-10-05.out 第 64 行（name=anchor）
```

All eight differ by 8192 bytes (two record slots); the other 69 anchor lines match (`anchor_not_equal=8`, 77 `name=anchor` lines).

- Provision side: layout/01 line 319, “record n lands at (counter − 1) mod slot count × 4096,” holds in the window. Reading writes with magic `SFSJ` on device 0 from the committed H3 export (instance generation, counter, checkpoint_txg and transaction number at offsets 16, 20, 26, 78), the verbatim output is: `h3 step 28 offset 16805888 jsn_instance 1 jsn_counter 8 checkpoint_txg 6 transaction 6`, `h3 step 63 offset 16809984 jsn_instance 1 jsn_counter 9 checkpoint_txg 7 transaction 7`, `h3 step 96 offset 16814080 jsn_instance 1 jsn_counter 10 checkpoint_txg 8 transaction 8`, `h3 step 129 offset 16818176 jsn_instance 1 jsn_counter 11 checkpoint_txg 9 transaction 9`; each equals 16 MiB + (counter − 1) × 4096. The four H1 lines have the same values as in the first run (counters 7–10).
- Registration side: section 7 A-journal (H3) of the second registration still derives using “counter = txg” (warm-ups 1, 2, new-pool file creation 3, inode 2 creation 4, sequential write 5). The first run had already found that this step does not follow from the provision and that the sequential-write publication writes three records (derived); H3's counter exceeds txg by 2, consistent with the sequential write adding two records.
- Classification: the registration's derived anchor is wrong; this is not a mismatch between implementation and layout/01. It is outside row 1's L03 cell table and the segment string, so row 1's verdict is unaffected.

#### Second run: unit tests and mutations

- 38 unit tests (last `cargo test --release --bin e172-deletion-write-dry-run` log on 2026-10-05: `test result: ok. 38 passed; 0 failed`): eight of the first run's 26 tests were changed, one deleted (R8.b not registered), and 13 added (t1–t13 in registration section 5.8: decode T_time from a synthetic system-configuration slot, R1.t row counts and expectations, L03-7 branch two, H3 intent time, synthetic batch-time expectations, R5.b and self-test rows, H3 A-tag6, copies of PC-G / PC-H / PC-I, two synthetic R8.c attribution cases, and H3's four anchor types). The new tests fix expectation fields, structure and synthetic inputs only; they do not assert H3's observed readings.
- Mutation table `scripts/mutation/tables/e172_deletion_write_dry_run.tsv` has 32 entries: the first run's 20 (M19's text rewritten for indentation), plus M1b, M5, M10b and M22–M30. Each anchor matches exactly once in source (Python count); at the time of this note the complete table had not yet been run and was assigned to the main agent.

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

- Read-only `crates/` export: `cargo run -q -p singlefs-checker-tier --bin deletion_write_dump` in the main workspace (batch 18 X, merged 2026-10-05) runs H1 and H2 in sequence on two 4 GiB memory disks, emitting complete bytes for every write, every barrier and each segment boundary in the window beginning before the intent publication. Both runs have sha256 `5a9b3fa27dc5b23531b767b1ed03a5b83a40d8a279485314c990df64fbbbd397`, committed as `e172-deletion-write-dump-2026-10-05.out` (462 lines, 10118080 bytes). Before and after each run, hashes of `crates/singlefs-verification/src/bin/deletion_write_dump.rs` (`b9e9facc…d86577`), `crates/singlefs-verification/src/scenario.rs` (`fbe2208c…a3fc6cb`) and the sha256 of per-file sha256 values in `crates/singlefs-core/src/` (`12582193…ad9eb`) matched all three times.
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
| 2 | Role set: rewritten roles returned by `crates/` | Core returns `TransactionOutput::rewritten` at each publication; `TransactionUnit::tag` names them and export writes `rewritten_roles=` on boundary rows. | `crates/singlefs-verification/src/bin/deletion_write_dump.rs:80 segment_boundary_fields` | `TransactionUnit::tag` naming |
| 3 | Segment attribution: decoder attributes each write | Units use header birth generation, journal records and root slots use checkpoint_txg, and system-configuration slots use tail and instance generation (calculate both original R8.b order and amendment 1 R8.c reading). Count every mount-part write. | `scripts/experiment-apparatus/e172_deletion_write_dry_run.rs:2291 attribute_mount_writes` | layout/01 lines 152–153 (tail, instance generation), 186 (birth generation), 344–345 (jsn, checkpoint_txg), 373 (root-record checkpoint_txg) |
| 4 | Segment attribution: export partition | Partition directly invoked publications by recorded-stream length before and after the call; partition unmount and mount by each publication's write account returned by core. Calculate the part before the mount row write by subtracting the sum of publication write accounts from the mount-part write count. | `crates/singlefs-verification/src/scenario.rs:687 unmount_then_mount_continuation` | Core `TransactionOutput::writes` |

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
