## E164 Small-Scale Functional Testing of New Crash Amplification Design — Segments One and Two Run (2026-09-28)

[中文](164-崩溃放量新设计的小范围功能测试.md) · **English** · [日本語](164-クラッシュ増幅の新設計に対する小規模機能テスト.ja.md)

<!-- doc-lint:not-numbers F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 W1 W2 W3 W4 W5 V0 V1 V2 V3 V4 V5 V6 V7 V8 V10 V11 S1 S2 S3 S4 S5 S6 M1 M3 M12 M22 R1 R2 R6 P7 -->

Pre-registration `e164-preregistration.md` (section twelve revisions R1–R9); question list `m3-function-test-questions.md` (F1–F13, W1–W5). The subject is the design table in section four of the third-round verdict `research/prompts/m3-prune-gpu-r3-main-verification.md`, not the current `crates/` implementation. Apparatus `scripts/experiment-apparatus/e164_crash_amplification_design_function_test.rs` (bin `e164-crash-amplification-design-function-test`, neither depends on nor links `crates/`, starts only whitelisted subprocesses in repository copies; supporting text in the same-named directory). All 12 unit tests passed (`cargo test --release --bin e164-crash-amplification-design-function-test`; the test reading the main repository's git uses `E164_REPOSITORY_ROOT`); all 22 mutations in `scripts/mutation/tables/e164_crash_amplification_design_function_test.tsv` caught. Segment three was not run (missed failures on crash states, changed processes, I-7.8 (root record tree ID watermark not lower than the pool-wide maximum tree ID), body, record checker keys, P7 key).

### What was done

- **Segment one, without reading the main repository** (artifact `…-models.out`): enumerate reachable states for six F10 pipeline forms and three F11 block-claim forms at 16 geometry points (blocks 2 and 3, cards 1 and 2, backlog cap 1 and 2, batch 1 and 2); compile each of two F12 label-type forms × five writings once (`rustc --emit=metadata`); section-seven pinned-value assertions.
- **Segment one, reading the main repository** (artifact `…-repository.out`, each line carries starting HEAD and `crates/` raw digest): W3 scans renames and `#[test]` identity uniqueness across all 318 commits; W5 computes crate closures of 97 nodes (8 crash enumerations, 89 tests), deciding rerun scope for all 20 commits touching `crates/*/src/*`; F13 changes six S1–S6 settings in a repository copy with `.git`, calculates both arm fingerprints once each, and also performs same-build-directory recompilation controls for S1–S5.
- **Segment two** (artifact `…-variants.out`): apply one variant to each of 12 repository copies (V0, V0b, V1–V8, V6b, V11), compile a harness-tier probe per copy, record H-short (mkfs → number acquisition → two warm-up runs → new-pool file creation) and H-long (followed by overwrite, sequential write, overwrite), calculate three write-table hashes and parent output step by step, and judge reuse for seven recording arms at each step; also calculate judging digest, three node-code digests and targets reading their own source, and run named tests for outcomes.

### Result rows copied from artifacts (rows moved out of the workspace are represented by one-line pointers)

**Segment one, without main-repository reads (`research/results/e164-crash-amplification-design-function-test-2026-09-28-models.out`)**

```
E7RESULT name=anchors_summary all_hold=true
E7RESULT name=q10_weak_form_summary form=record_flag_before_data points_with_its_own_error=16 of=16 reproduced=true
E7RESULT name=q10_weak_form_summary form=checked_flag_before_result points_with_its_own_error=16 of=16 reproduced=true
E7RESULT name=q10_weak_form_summary form=result_then_flag_with_appended_tally points_with_its_own_error=16 of=16 reproduced=true
E7RESULT name=q10_weak_form_summary form=checked_flag_is_a_boolean points_with_its_own_error=16 of=16 reproduced=true
E7RESULT name=q10_weak_form_summary form=batch_larger_than_backlog_cap_without_flush points_with_its_own_error=4 of=16 reproduced=true
E7RESULT name=q10_traj_supplement_summary points_with_backlog_peak_over_cap_without_judging_change=0
E7RESULT name=q10_strongest_summary errors_over_all_points=0 kill_transitions=56566 judging_change_transitions=6104 terminal_states=32 points_with_backlog_peak_over_cap=12 holds=true
E7RESULT name=q11_summary form=claims_persisted_without_lease dead_block_terminals=383 other_four_errors=0 card_drop_transitions=41370 takeover_after_lease_transitions=0
E7RESULT name=q11_summary form=claims_in_memory dead_block_terminals=0 other_four_errors=0 card_drop_transitions=102200 takeover_after_lease_transitions=0
E7RESULT name=q11_summary form=claims_persisted_with_lease dead_block_terminals=0 other_four_errors=0 card_drop_transitions=2152320 takeover_after_lease_transitions=113420
E7RESULT name=q12_summary weak_writings_one_to_four_compiling=1 strict_writings_one_to_four_compiling=0 strict_writing_five_compiles=true
```

**Segment one, reading main repository (`e164-crash-amplification-design-function-test-2026-09-28-repository.out`; trailing head and crates_raw_digest removed, same value on every row: head=e5253e8a2afd26d3352a6f02fe015d67858eab89)**

```
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 2 行（name=w3_scan）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 3 行（name=w3_reoccupied_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 4 行（name=w3_uniqueness）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 5 行（name=w3_duplicate_identity_by_path）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 104 行（name=w5_nodes）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 105 行（name=w5_class_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 106 行（name=w5_class_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 127 行（name=w5_class_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 128 行（name=w5_class_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 129 行（name=q13_closure）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 130 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 131 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 132 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 133 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 134 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 135 行（name=q13_setting）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 136 行（name=q13_summary）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out 第 137 行（name=q13_geometry）
```

**Segment two (`e164-crash-amplification-design-function-test-2026-09-28-variants.out`; trailing head and crates_raw_digest removed, trace removed from recording_arm rows, left and right removed from q5_red rows)**

```
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 26 行（name=determinism）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 205 行（name=q3_digest）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 206 行（name=q3_digest）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 207 行（name=q3_digest）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 208 行（name=q3_digest）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 209 行（name=q3_lib）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 210 行（name=q4_flip）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 214 行（name=q4_flip）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 218 行（name=q4_miss）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 219 行（name=q4_miss）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 220 行（name=q4_miss）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 224 行（name=q6）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 226 行（name=q6_own_source_targets）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 66 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 74 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 100 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 108 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 180 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 188 行（name=write_table_difference）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 33 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 41 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 43 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 44 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 45 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 47 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 48 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 49 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 51 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 52 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 53 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 55 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 56 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 57 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 59 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 60 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 61 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 65 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 67 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 68 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 69 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 73 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 77 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 78 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 79 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 83 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 85 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 86 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 87 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 91 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 95 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 96 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 99 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 103 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 104 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 107 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 115 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 123 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 131 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 139 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 141 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 142 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 143 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 144 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 147 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 149 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 150 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 151 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 152 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 155 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 163 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 171 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 177 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 178 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 179 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 185 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 186 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 187 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 195 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 203 行（name=recording_arm）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 221 行（name=q5_red）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 222 行（name=q5_red）
产物行见 git show 5d0b7e7c1667:research/results/e164-crash-amplification-design-function-test-2026-09-28-variants.out 第 223 行（name=q5_red）
```

### Row-by-row judgments

| Row | Judgment | Supporting artifact rows |
|---|---|---|
| F1 | Digest half: barrier-aware write-table hash (C) holds. Positive controls: A misses rerecording on V2, B on V1 (H-short 2 steps, H-long 5). C misses 0 on V1/V2 and has 0 unnecessary rerecords on V0b/V6. All V1 rerecorded steps differ only by barrier or segment boundary (H-short 2, H-long 5). Missed-failure half belongs to segment three | `recording_arm` f1_* rows, `write_table_difference variant=V1` |
| F2 | Premise holds: C is unchanged step by step on V3. Implement arm (add recording-code digest) rerecords all on V3 and 0 on V6. Omit arm rerecords 0 on V3; what it misses (states whose recovery process changed) requires segment three | `write_table_difference variant=V3`, `f2_with_recording_code`, `f1_with_barriers variant=V3` |
| F3 | Digest half: implement arm (judging digest covers harness and checker tier) changes on V4, not V8; omit arm unchanged on V4. Positive control did not reproduce in harness tier: `cargo test -p singlefs-harness --lib` has 0 tests with differing V0/V4 outcomes. Move F3 positive control to segment three (I-7.8 (root record tree ID watermark not lower than the pool-wide maximum tree ID)) | `q3_digest`, `q3_lib` |
| F4 | Holds: crate closure changes on V5 and V11, misses 0; function-only misses 2 (V5, V11), visited-files misses 1 (V11). Both named tests turn from pass to fail | `q4_flip`, `q4_miss` |
| F5 | Holds: implement arm has 0 false failures on V6 and 1 failure on V10; omit arm has 1 false failure on V6; both have 0 on V6b | `q5_red` |
| F6 | Holds: V8 changes the named test's exit code 0 → 101; omit arm's digest unchanged (1 miss), implement arm's changes (0 misses); implement arm unchanged on V6. One repository-wide target reads its own source (`crates/singlefs-harness:--lib` reads `model.rs`) | `q6`, `q6_own_source_target(s)` |
| F7 | Holds: omit arm (parent identity as input) misses one V7 step (step 0, 4 states); implement arm (parent output as input) misses 0 and has 0 unnecessary rerecords on V0b | `f7_parent_*` |
| F10 | All five errors in the strongest form are 0 at all 16 points; kill, judging change and terminal states were reached. Each of five weak forms reproduces its own error (first four 16/16, batch larger than backlog cap without flushing 4/16). Q10-traj: backlog (recorded blocks not checked under current judging) peaks above the recording process's waiting threshold at 12/16 points; supplemental arm R1 disables judging changes and yields 0/16. All increases happen at judging changes, which mark previously checked blocks for recheck in the store. These remain in the store; the GPU claims one fixed-size batch at a time and never exceeds its available-space placement cap. Registration called the recording wait threshold and GPU capacity cap the same term, so this cell's reversal observation (“backlog cap not maintained”) does not hold | `q10_*` |
| F11 | Holds: persisted claims without lease yield 383 dead-block terminal states; in-memory claims and persisted claims with leases yield 0 dead blocks and 0 other four errors. Lease expiry followed by takeover by another GPU card occurs 113420 times. “In memory” uses R6's weakest reading: no mutual exclusion between cards | `q11_summary` |
| F12 | Holds: weak form allows writing ④ (`to_string` as key); strict form rejects ①–④ (E0308, E0277, E0308, E0599), accepts ⑤ | `q12_*` |
| F13 | **Q13-false implement arm = 1, FC-fix hit**: S5 (`CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`) recompilation control reports no recompile (workspace `Cargo.toml` already specifies `overflow-checks = true` in `[profile.release]`), but implement-arm fingerprint changes: it records which key is set, not the effective value. Q13-miss implement arm = 0 (S4 requires recompile and changes fingerprint). Omit arm changes fingerprints on S1/S2/S3 while controls say no recompile, reproducing third round's “fingerprint changes without affecting the judgment.” Geometry control: E162 (Storage Backend Selection for Crash Amplification Judgment) closure includes a package with a build script, and implement arm changes configuration fingerprint on S3 there; layer0 test closure does not, so it does not | `q13_*` |
| W3 | Both counts of old names being reoccupied are 0 (433 renames, 169 of `.rs` files containing `#[test]`); freezing filenames holds. QW3-dup measured only first two identity components (`<code filename>::<#[test] function name>`), without step number or hash: duplicate by repository path in 28 commits, all the same location—gate fixture (old path in those commits; current `.claude/gate.d/fixtures/checker-tier-crates-mutation-replay.sh/red/crates/tiny/src/lib.rs`) has `#[test] fn doubles_three_to_six` in each of two modules. By filename duplicates in 120 commits (up to 41 in one commit). **Full identity does not collide**: function bodies and hashes differ; using cargo's module-qualified test name makes the first two components unique too. FC-rename's “identity collision” does not hold; this cell only says use repository path for code filename and module-qualified cargo test name for target | `w3_*` |
| W5 | “Crash-enumeration nodes cannot be saved” confirmed in observed history: all 20 commits rerun 8/8, including five changing only harness (checker-tier closure includes harness); at most 3/89 test nodes saved. None of the 20 changed only core, checker or format, so these three classes have no sample points. Measured by crate closure, not function-unit key (D13 (Verification Route) settled item 17); do not use it elsewhere as evidence that pruning cannot save work | `w5_*` |

### Geometry sensitivity

F1, F2 and F7 give the same direction on H-short and H-long (missed/rerecorded step counts vary with length, zero versus nonzero does not flip). F4 at V5/V11: visited-files arm differs (V5 changes, V11 does not), matching registration. F5 at V6/V6b: omit arm differs (1, 0). F6 at V8/V6: implement arm changes only on V8. F10/F11 report each of 16 points; strongest form and three claim forms yield same-direction judgments throughout; batch greater than backlog cap without flushing errors only at four points with batch 2 and cap 1. F13 on layer0 and E162 closures: implement arm's S3 result differs. W3 first two identity components duplicate both by filename and repository path (120 and 28 commits), while full identities with hash do not collide either way. W5 windows 10 and 30 (actually 20) yield same-direction judgment.

### Mutations

All 22 caught (`scripts/mutation/mutate.sh`, launched under `research/`, `E164_REPOSITORY_ROOT` points to main repository; ends “restored, baseline still all green”). M1 (remove segment number from C) and M3 (FUA does not close segment) did not fail initially; per category three of `.claude/rules/mutation-sampling.md`, sample points were insensitive. Adding two sensitive sample points in registration revision R2 caught them. M22 (closure omits dev-dependencies) was found insensitive before the run (no node's closure includes a workspace package solely through dev-dependencies), so registration replaced it with M22b.

### Rerun

`bash scripts/experiment-apparatus/replay.sh E164` registers only the artifact independent of current main-repository state (`…-models.out`, exact); replay on 2026-09-28 was byte-identical. The other two depend on main HEAD, `crates/` and git history and are not registered for byte comparison. Rerun under `research/`: `E164_SCRATCH_DIRECTORY=<草稿目录> bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh 8 target/release/e164-crash-amplification-design-function-test repository`, and the same wrapper with `… variants <草稿目录>`. If HEAD or `crates/` digest differs from artifact line endings, the input changed; rerun under new inputs.

### Path and conclusion registration

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Apparatus segmentation | Independently written segmentation and two-state closed form from the literal D13 settled item 4; compute every step of 12 variants × 2 histories (all recorded steps, not samples) | `scripts/experiment-apparatus/e164_crash_amplification_design_function_test.rs:3336` `segment_index_of_each_operation` | D13 settled item 4 segmentation rule; probe recording stream |
| 2 | Harness segmentation | Harness tier's own `split_into_segments` and `closed_form_state_count`; probe calls them in the copy and emits `harness_segments` rows | `crates/singlefs-harness/src/segments.rs:159` `split_into_segments` | D13 settled item 4 segmentation rule; the same recording stream |

Cross-path assertion: `segment_crosscheck` (apparatus source line 581) compares segment sequences and closed forms step by step, stopping on mismatch (S-seg); artifact `probe` rows' `segment_mismatches=none` is its reading. Both paths share assumptions that D13 settled item 4's segmentation rule and the recording stream are correct; a mistaken rule makes both wrong. Section 7.1's pinned-value assertions give expected values from the clause literally and are the sole external pin on this assumption.

### What it cannot answer

- No cells from segment three were measured (half of F1, F2, F3; F8, F9, W1, W2, W4). A checker-tier probe for the F1 / W4 and F2 halves was written (expand segment lengths ≤ 15 including torn states; emit judgment and recovery outcome for each state) and tried once on the V0 copy on 2026-09-28: 141 states, 0 failures, segment sequence identical to segment-two harness segmentation. It was not integrated into the apparatus or run on V1–V3; this trial is not an artifact. Probe source was deleted from the apparatus directory (no target references it); retrieve from git history when integrating segment three: `git show dbcc0d2f:research/e7-index-bench/src/bin/e164_crash_amplification_design_function_test/probe_crash_states.rs`. H-long has no segment with 16–18 writes, so W4's large and small domains are the same states on H-long. The P7 key, two record checker keys and two custom scenarios required by F3, F8, F9 / W1, W2 were not written.
- F3 positive control did not reproduce at harness tier (`q3_lib differing_outcomes=0`); “covering harness disk-reading code” proves only the digest changes, not that it catches an actual judgment change.
- V11 (change one `DATA_POINTER_BYTES` value) makes the probe exit 101; no recording-arm cell has a reading for this variant. F4 uses only its node digest and named test.
- W5's 20 commits include none changing only core, checker or format, so those three classes have no rerun-ratio samples.
- F10/F11 are pure models: “claims only in memory” uses the weakest interpretation (R6); store atomic-write semantics and lease implementation were not measured against a real KV store.
- The apparatus itself has not undergone a three-party review.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 4 | Background | 2026-10-15 unaffected: full copied artifact rows replaced with pointers; numbers, conclusions and artifacts unchanged |
| D13 (Verification Route) settled item 9 | Background | 2026-10-15 unaffected: full copied artifact rows replaced with pointers; numbers, conclusions and artifacts unchanged |
| D13 (Verification Route) settled item 15 | Background | 2026-10-15 unaffected: full copied artifact rows replaced with pointers; numbers, conclusions and artifacts unchanged |
| D13 (Verification Route) settled item 17 | Supports | 2026-10-15 changed: F4 measured misses V5/V11 if fingerprinting functions only without referenced constants (V11 changes only `DATA_POINTER_BYTES`). Item 17 therefore includes referenced constants, types, statics and macros in the function unit; after the change rerun F4 with V5/V11 as “must invalidate” examples. W5's “crash-enumeration nodes cannot be saved” was measured only by crate closure, not function-unit keys; do not use elsewhere as proof pruning cannot save work |

## Historical Versions

### 2026-09-28: page created, segments one and two run

The executor was stopped after getting stuck writing the apparatus. The main agent took over apparatus, unit tests and mutation table, ran segments one and two, and placed three artifacts in `research/results/`; revisions R1–R9 were recorded in pre-registration section twelve. Segment three was not run.
