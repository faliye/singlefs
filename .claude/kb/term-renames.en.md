# Term Rename Cross-Reference Table

[中文](term-renames.md) · **English** · [日本語](term-renames.ja.md)

Sole authoritative cross-reference between old and new names for concepts renamed repo-wide. **Everywhere else must use new names**—`scripts/kb/sweep-term.py --check` sweeps the whole repo, and turns red if any old name appears outside the few places registered in `.claude/term-rename-exempt`.

This document is itself registered in that exemption table: to state "X was renamed to Y", one must be able to write X, and **old names must be searchable in one place so historical artifacts and old commit fields can be reconciled**. To look up the current name of an old name, look only here; to look up which files were changed and why on the day of a rename, see that day's entry in the decision change history.

## Superblock → System Configuration (From 2026-09-20; English Identifiers and Frozen Directory 2026-09-21)

User verdict: That 481-byte structure on disk is not called "superblock"—the index trees of this project are ordinary trees that can be reconstructed upon loss, and there is nothing "super" about it; that structure is **system configuration**. Verdict on 2026-09-21: English identifiers, file names, and frozen directories changed simultaneously "until nothing can be found across the entire project".

## Names Based on Milestones → Names Based on Content (From 2026-09-27)

User verdict: Tests, modules, harnesses, gates, and kb documents must not be named by milestone ("I think renaming by milestone was a terrible decision, this should be modified", "change them all"), but named by content: the first write stream creates a new file on an empty pool freshly formatted with mkfs, named "new pool file creation" (`new_pool_file_creation`); the second write stream is file overwrite on the same file (`file_overwrite`); Milestone 2 implements file overwrite, release, rollback, and reuse; Milestone 3 was named crash verification redo on the day, now named "deletion and multiple COW" (the rename journey is noted in that row of the table). Chinese names of the milestones themselves were changed simultaneously (user chose "change together" on the same day). Document names numbered by milestone under `.claude/kb/milestone/` and `.claude/kb/layout/` (`01-first-txn.md` etc.) are not changed (user determined on the same day that "the naming is right"), so hyphenated abbreviations are registered only by specific use; follow-up changed file names, gate names, and test file names are cross-referenced in Section "Batch 2" of `records/2026-09-27-验证两档拆分.md`.

## Layer 0 → Crash Point Replay (From 2026-10-12)

The user pointed out on 2026-10-11 that renaming "Layer 0" to "crash point replay" was only partially done, and settled on replacing it across the entire repo: stacked uses of "Layer 0 crash point replay" drop "Layer 0"; "Layer 0 full / quick tier" is written as "crash point replay full / quick tier"; "Layer 0 test cases" is written as "crash enumeration test cases"; English identifiers (`layer0_*`, gate name `checker-tier-layer0-replay`) were left untouched at the time, but were changed in the 2026-10-13 batch, registered in the section "First Stream / Second Stream, Two-Segment Pipeline, Tiers of the Three-Tier Table → Names Based on Content (From 2026-10-13)" at the end of the text. The two categories not replaced are registered per `.claude/term-rename-exempt`: test rigs, retained artifacts, mutation names, and gate fixtures (whole files); string literals in code and verbatim quoted old texts (the original third category, the "Layer 0" tier juxtaposed with Layer 1 and Layer 2 in the tiered table of settled item 9 of D13 (verification roadmap), has no target left as the three tiers were renamed smoke tier, regular tier, and heavy-load tier).

## First Stream / Second Stream, Two-Segment Pipeline, Tiers of the Three-Tier Table → Names Based on Content (From 2026-10-13)

User verdict: Names based on ordinal numbers in reports and documentation ("first stream", "second stream", "first segment", "second segment", "Layer 0", "Layer 1", "Layer 2") do not convey what they point to; replace them throughout the repository with names expressing content, with batch replacements exempting third parties; in modal prompt selected "change according to suggested names", stating "for example, we previously ran the new file COW of the first transaction, now running multiple COW of the second transaction, I think such description is clearer and simpler", "of course you can adjust this naming according to practical situation".
Two workloads of crash point replay: first stream → new pool file creation stream; second stream → file overwrite fixed script stream (that script performs overwrite, release, reopen write row, warm-up, rollback, raise F, reuse once each, up to E).
Two-segment pipeline of crash amplification: first segment → ingestion segment (stream ingested into store first); second segment → judgment segment (read back from store to judge, on GPU or CPU). This pair is not registered in table rows and gates do not check it: the same words represent other quantities in experiment registration segments, executor turn-in by segment, segment sequence, and KV implementation segments; clerks reviewed 314 lines in kb and records line-by-line, with 0 lines genuinely pointing to the pipeline, and all 19 matches by context-based regexes were false positives; targets reside only in script comments and output text (three scripts in scripts/crash already replaced), and future authors will write with new names.
Three tiers of the three-tier table in settled item 9 of D13 (verification roadmap): Layer 0 smoke → smoke tier; Layer 1 regular → regular tier; Layer 2 full → heavy-load tier; scattered "Layer 0" still replaced with crash point replay per the 2026-10-12 section. English identifiers (such as layer0_enumeration, checker-tier-layer0-replay, record) were left untouched in this round.
The English identifier rename batch (`records/2026-10-13-复用键与全仓改名.md#3`, row 8 of dependency table) renamed the English identifiers that the previous sentence left untouched, per user verbatim words: "function names must not be named with layer0 or similar names; functionality and purpose must be identifiable": crates, scripts/gates, and docs were updated, and old names registered row-by-row in the table below (bare words `layer0`, `LAYER0`, `Layer0` are not registered; occurrences in test rigs, retained artifacts, and verbatim quotes are preserved as-is, registered in `.claude/term-rename-exempt`). Rows in this batch are written as regexes: names start literally, not immediately flanked by ASCII letters, digits, or underscores (for names with hyphens, hyphens count as word characters), evaluated same as word boundaries, also matching adjacent Chinese characters. Not prefixed with `\b` because lines beginning with `\b` make repo-wide scanning much slower: on the first 2,000,000 characters of E178 artifact `e178-leaf-and-record-death-counts-segment-two.out`, the original 56 lines took 0.19s, adding these 234 literal-start lines took 0.71s, while writing them as word boundaries took 2.96s.

<!-- term-renames:table -->
One row per pair, `scripts/kb/sweep-term.py` replaces and checks according to this table; **adding a row automatically places it under gate enforcement**.
"Match" specifies `Full string`, `Word boundary`, or `Regex`: `Word boundary` is used for abbreviations to prevent matching letters inside other words; when the same string represents another quantity elsewhere, `Regex` is used, where the "Old" column contains a Python regex matching only this concept's phrasing (e.g. `(?<![每第内外里叶二])(?<![0-9] )层 1(?![0-9])` matches "Layer 1 regular", but not "134 per layer", "2 in Layer 1", "outer layer 16.33 ms"), pipes written as `\|`, capturing groups forbidden, and "New" column replaced literally; full restrictions on patterns are in the header of `scripts/kb/sweep-term.py`. Longer patterns are placed earlier to prevent shorter ones from consuming them halfway.

| Old | New | Match | What it is |
|---|---|---|---|
| 超级块 | 系统配置 | Full string | Chinese term |
| SUPER_BLOCK | SYSTEM_CONFIGURATION | Full string | Constant name with underscore |
| SUPERBLOCK | SYSTEM_CONFIGURATION | Full string | Constant name |
| Super_Block | SystemConfiguration | Full string | UpperCamelCase with underscore |
| SuperBlock | SystemConfiguration | Full string | UpperCamelCase, type name |
| Superblock | SystemConfiguration | Full string | UpperCamelCase with lowercase 'b' |
| superBlock | systemConfiguration | Full string | lowerCamelCase |
| super_block | system_configuration | Full string | snake_case with underscore |
| super-block | system-configuration | Full string | Kebab-case (binary name, artifact file name) |
| superblock | system_configuration | Full string | snake_case identifier, module name, file name |
| sb_ | system_configuration_ | Word boundary | Abbreviation prefix (sb_mac) |
| _sb | _system_configuration | Word boundary | Abbreviation suffix (tail_sb) |
| sb | system_configuration | Word boundary | Bare abbreviation. 'sb' was originally for superblock; when concept was removed abbreviation was also removed, deleted from SOP abbreviation table on same day |
| harness 耗时用例 | harness 慢档用例 | Full string | Test cases running 60s or more under debug in harness tier, annotated with `#[ignore]`; user settled on "quick tier / slow tier" on 2026-10-01, sharing name with .claude/rules/test-script-discipline.md |
| 耗时用例 | 慢档用例 | Full string | Short form of "harness 耗时用例" without harness prefix |
| 轻用例 | 快档用例 | Full string | Cases not marked `#[ignore]` in harness tier, run after each modification; renamed same day as slow tier cases |
| harness 重档 | harness 慢档用例 | Full string | Cases running full 60s single-threaded in harness tier; renamed to disambiguate from heavy testing |
| 里程碑三（名字待定） | 里程碑「删除与多次 COW」 | Full string | Milestone name. Initially named crash verification redo; from commit 4e8b0370 (2026-09-29) crash verification merged into Milestone "overwrite, release, rollback, and reuse" Addendum 4, Milestone 3 renamed "deletion and multiple COW" |
| 第二个事务 | 覆盖写、释放、回退与复用 | Full string | Milestone name, also name of that write stream |
| 第一个事务 | 新池新建文件 | Full string | Milestone name, also name of that write stream |
| FIRST_TRANSACTION | NEW_POOL_FILE_CREATION | Full string | Constant name |
| FirstTransaction | NewPoolFileCreation | Full string | UpperCamelCase, type name and enum variant |
| first_transaction | new_pool_file_creation | Full string | snake_case identifier, module name, file name |
| first-transaction | new-pool-file-creation | Full string | Kebab-case (run mode name, stage file name) |
| SECOND_TRANSACTION | FILE_OVERWRITE | Full string | Constant name |
| SecondTransaction | FileOverwrite | Full string | UpperCamelCase, enum variant |
| second_transaction | file_overwrite | Full string | snake_case identifier, metric name |
| second-transaction | file-overwrite | Full string | Kebab-case (run mode name) |
| first_txn | new_pool_file_creation | Full string | Abbreviated snake_case |
| e142-first-txn-dry-run | e142-new-pool-file-creation-dry-run | Full string | Artifact file name prefix |
| 32-first-txn-fields | 32-new-pool-file-creation-fields | Full string | Stage file name |
| 42-first-txn-trio | 42-new-pool-file-creation-trio | Full string | Stage file name |
| 76-second-txn-hooks | 76-overwrite-release-rollback-reuse-hooks | Full string | Stage file name |
| first-txn-journal-implies-units | new-pool-file-creation-journal-implies-units | Full string | Litmus file name |
| first-txn-root-implies-units | new-pool-file-creation-root-implies-units | Full string | Litmus file name |
| formatted-layer0-same-as-first-txn | formatted-layer0-same-as-new-pool-file-creation | Full string | Label |
| formatted-layer0-first-txn-reference | formatted-layer0-new-pool-file-creation-reference | Full string | Label |
| second_txn | overwrite_release_rollback_reuse | Full string | Abbreviated snake_case |
| third_txn | crash_verification_rebuild | Full string | Abbreviated snake_case |
| 按条记录 | 判定库记录 | Full string | Records in crates mutation table verdict library |
| 判法指纹 | 判法单元指纹 | Full string | Digest of crates mutation table verdict library directory (briefly called judge version) |
| 底座指纹 | 判法单元指纹 | Full string | Earlier name for judging fingerprint |
| 判法版本 | 判法单元指纹 | Full string | From 2026-10-15 no library-wide judge version; individual fingerprints of function units enter unit key of each verdict |
| judging-fingerprint | judge-version | Full string | Subcommand of `crates-mutation-rows.py` |
| judging_fingerprint | judge_version | Full string | Function name and record field |
| lib_item_verdicts | lib_verdict_library | Full string | File rename (path move per `.claude/rules/path-moves.md`), checked together per term |
| GATE_FORMAL_VERDICT_STORE | GATE_FORMAL_VERDICT_LIBRARY | Full string | Formal verification gate environment variable |
| store-in-tmpdir | library-in-tmpdir | Full string | Item of `FORMAL_VERIFICATION_BREAK` |
| 快速档 | 快档 | Full string | Tier in E165; part of quick/slow tier nomenclature alongside harness slow tier |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?0(?![0-9.])(?! ?冒烟)` | 崩溃点重放 | Regex | Chinese term; does not match tree levels like "Layer 1 0" or D13 "Layer 0 smoke" |
| 第一条流 | 新池新建文件流 | Full string | Chinese term: first workload of crash point replay |
| 第二条流 | 覆盖写固定脚本流 | Full string | Chinese term: second workload of crash point replay (fixed script to E) |
| 层 0 冒烟 | 冒烟层 | Full string | Label for tier 1 in three-tier table of settled item 9 of D13 |
| 层 1 常规 | 常规层 | Full string | Label for tier 2 in same table |
| 层 2 全量 | 大负载层 | Full string | Label for tier 3 in same table |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?1(?![0-9.])(?! ?常规)` | 常规层 | Regex | Chinese term: when tier 2 of three-tier table written alone; does not match tree levels |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?2(?![0-9.])(?! ?全量)` | 大负载层 | Regex | Chinese term: when tier 3 of three-tier table written alone; does not match tree levels |
| `checker-tier-layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]checker-tier-layer0-replay)` | checker-tier-crash-point-replay | Regex | Gate name, stage file name, fixture directory name |
| `layer0-shard-configuration-check(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-configuration-check)` | crash-point-replay-shard-configuration-check | Regex | Script name for two-host shard configuration judgment |
| `layer0-shard-run(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-run)` | crash-point-replay-shard-run | Regex | Script name for two-host sharding wrapper of crash point replay |
| `layer0-sample-tools(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-sample-tools)` | crash-point-replay-sample-tools | Regex | Directory name for fake tools in gate fixture root (`.layer0-sample-tools`) |
| `singlefs-layer0-logs(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]singlefs-layer0-logs)` | singlefs-crash-point-replay-logs | Regex | Log directory name in common-dir for gates |
| `singlefs-layer0-shard(?![A-Za-z0-9_])(?<![A-Za-z0-9_]singlefs-layer0-shard)` | singlefs-crash-point-replay-shard | Regex | Temp directory prefix for two-host sharding wrapper |
| src/layer0_library.rs | src/crash_library.rs | Full string | Verdict library source file referenced in `Cargo.toml` comments |
| `layer0_enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration)` | crash_state_enumeration | Regex | Module name, `.rs` file name, verdict library class name and manifest kind |
| `layer0_progress(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_progress)` | crash_point_replay_progress | Regex | Module name and `.rs` file name |
| `layer0-enumeration-library(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-enumeration-library)` | crash-state-verdicts | Regex | Subdirectory name in verdict library for crash enumeration cases |
| `layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-replay)` | crash-case-replay | Regex | Cell name in crash point replay gate |
| `LAYER0_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_NAME)` | CRASH_CASE_REPLAY_CELL_NAME | Regex | Cell name constant in stages |
| `LAYER0_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_TITLE)` | CRASH_CASE_REPLAY_CELL_TITLE | Regex | Cell title constant in stages |
| `LAYER0_FEATURE_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_NAME)` | FEATURE_GATED_TESTS_CELL_NAME | Regex | Cell name constant for feature-gated-tests in stages |
| `LAYER0_FEATURE_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_TITLE)` | FEATURE_GATED_TESTS_CELL_TITLE | Regex | Cell title constant for feature-gated-tests in stages |
| `layer0-stage(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stage)` | crash-point-replay-stage | Regex | Stage kind name in heavy test guard (`lib_heavy_tests.py` STAGE_KIND) |
| `stage_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]stage_layer0)` | stage_crash_point_replay | Regex | Stage name in heavy test guard selftest |
| `layer0_admission_module(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_admission_module)` | crash_point_replay_admission_module | Regex | Stage variable |
| `layer0_break(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_break)` | crash_point_replay_break | Regex | Stage variable |
| `layer0_cell_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_cell_state)` | crash_point_replay_cell_state | Regex | Stage variable |
| `layer0_check_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_check_cells)` | crash_point_replay_check_cells | Regex | Stage variable |
| `layer0_found(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_found)` | crash_point_replay_found | Regex | Stage variable |
| `layer0_full_base(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_base)` | crash_point_replay_full_base | Regex | Stage variable |
| `layer0_full_rc(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_rc)` | crash_point_replay_full_rc | Regex | Stage variable |
| `layer0_input_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_paths_text)` | crash_point_replay_input_paths_text | Regex | Stage variable |
| `layer0_input_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_table)` | crash_point_replay_input_table | Regex | Stage variable |
| `layer0_judge_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_paths)` | crash_point_replay_judge_paths | Regex | Stage variable |
| `layer0_kept_rows(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_kept_rows)` | crash_point_replay_kept_rows | Regex | Stage variable |
| `layer0_list_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_cells)` | crash_point_replay_list_cells | Regex | Stage variable |
| `layer0_list_items(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_items)` | crash_point_replay_list_items | Regex | Stage variable |
| `layer0_list_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_table)` | crash_point_replay_list_table | Regex | Stage variable |
| `layer0_log_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_log_directory)` | crash_point_replay_log_directory | Regex | Stage variable |
| `layer0_named_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_cases)` | crash_point_replay_named_cases | Regex | Stage variable |
| `layer0_named_case(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_case)` | crash_point_replay_named_case | Regex | Stage variable |
| `layer0_named_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_value)` | crash_point_replay_named_value | Regex | Stage variable |
| `layer0_registered_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_input_paths)` | crash_point_replay_registered_input_paths | Regex | Stage variable |
| `layer0_registered_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_paths_text)` | crash_point_replay_registered_paths_text | Regex | Stage variable |
| `layer0_report_not_run(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_report_not_run)` | crash_point_replay_report_not_run | Regex | Stage variable |
| `layer0_resolved_cargo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_resolved_cargo)` | crash_point_replay_resolved_cargo | Regex | Stage variable |
| `layer0_runs_cell(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_runs_cell)` | crash_point_replay_runs_cell | Regex | Stage variable |
| `layer0_sample_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_mode)` | crash_point_replay_sample_mode | Regex | Stage variable |
| `layer0_sample_tool_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_mode)` | crash_point_replay_sample_tool_mode | Regex | Stage variable |
| `layer0_sample_tool_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_path)` | crash_point_replay_sample_tool_path | Regex | Stage variable |
| `layer0_sample_tool_problems(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_problems)` | crash_point_replay_sample_tool_problems | Regex | Stage variable |
| `layer0_sample_tools_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tools_directory)` | crash_point_replay_sample_tools_directory | Regex | Stage variable |
| `layer0_sample_tool(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool)` | crash_point_replay_sample_tool | Regex | Stage variable |
| `layer0_scratch_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_scratch_directory)` | crash_point_replay_scratch_directory | Regex | Stage variable |
| `layer0_selected_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_selected_cells)` | crash_point_replay_selected_cells | Regex | Stage variable |
| `layer0_shard_check_script_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_check_script_directory)` | crash_point_replay_shard_check_script_directory | Regex | Stage variable |
| `layer0_shard_configuration_note(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_configuration_note)` | crash_point_replay_shard_configuration_note | Regex | Stage variable |
| `layer0_shard_driver(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_driver)` | crash_point_replay_shard_driver | Regex | Stage variable |
| `layer0_sharded(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sharded)` | crash_point_replay_sharded | Regex | Stage variable |
| `layer0_stage_file_name(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_file_name)` | crash_point_replay_stage_file_name | Regex | Stage variable |
| `layer0_stage_repository(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_repository)` | crash_point_replay_stage_repository | Regex | Stage variable |
| `layer0_stage_script_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_script_path)` | crash_point_replay_stage_script_path | Regex | Stage variable |
| `layer0_start_over(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_start_over)` | crash_point_replay_start_over | Regex | Stage variable |
| `layer0_static_exit(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_exit)` | crash_point_replay_static_exit | Regex | Stage variable |
| `layer0_static_judge(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_judge)` | crash_point_replay_static_judge | Regex | Stage variable |
| `layer0_tier(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tier)` | crash_point_replay_tier | Regex | Stage variable |
| `layer0_tree_ready(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tree_ready)` | crash_point_replay_tree_ready | Regex | Stage variable |
| `layer0_unknown_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unknown_cases)` | crash_point_replay_unknown_cases | Regex | Stage variable |
| `layer0_unlisted_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unlisted_input_paths)` | crash_point_replay_unlisted_input_paths | Regex | Stage variable |
| `Layer0DigitOfASegment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0DigitOfASegment)` | CrashPointReplayDigitOfASegment | Regex | Type name |
| `Layer0EnumerationTables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0EnumerationTables)` | CrashStateEnumerationTables | Regex | Type name |
| `Layer0Enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Enumeration)` | CrashStateEnumeration | Regex | Type name |
| `Layer0FindingsLogBegin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogBegin)` | CrashPointReplayFindingsLogBegin | Regex | Type name |
| `Layer0FindingsLogSection(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogSection)` | CrashPointReplayFindingsLogSection | Regex | Type name |
| `Layer0FindingsLog(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLog)` | CrashPointReplayFindingsLog | Regex | Type name |
| `Layer0FindingSample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSample)` | CrashPointReplayFindingSample | Regex | Type name |
| `Layer0FindingSignature(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSignature)` | CrashPointReplayFindingSignature | Regex | Type name |
| `Layer0Findings(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Findings)` | CrashPointReplayFindings | Regex | Type name |
| `Layer0Finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Finding)` | CrashPointReplayFinding | Regex | Type name |
| `Layer0Item(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Item)` | CrashPointReplayItem | Regex | Type name |
| `Layer0JudgeRoute(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0JudgeRoute)` | CrashPointReplayJudgeRoute | Regex | Type name |
| `Layer0ObserverCounts(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ObserverCounts)` | CrashPointReplayObserverCounts | Regex | Type name |
| `Layer0Observer(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Observer)` | CrashPointReplayObserver | Regex | Type name |
| `Layer0OraclesOfThePieces(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OraclesOfThePieces)` | CrashPointReplayOraclesOfThePieces | Regex | Type name |
| `Layer0Oracles(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Oracles)` | CrashPointReplayOracles | Regex | Type name and enum variant (E171 JudgingPiece) |
| `Layer0OracleViolationKind(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolationKind)` | CrashPointReplayOracleViolationKind | Regex | Enum name |
| `Layer0OracleViolation(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolation)` | CrashPointReplayOracleViolation | Regex | Type name |
| `Layer0PublishOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0PublishOfState)` | CrashPointReplayPublishOfState | Regex | Type name |
| `Layer0RedPass(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0RedPass)` | CrashPointReplayRedPass | Regex | Enum name |
| `Layer0SegmentExpansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentExpansion)` | CrashPointReplaySegmentExpansion | Regex | Type name |
| `Layer0SegmentOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentOfState)` | CrashPointReplaySegmentOfState | Regex | Type name |
| `Layer0SegmentSlice(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentSlice)` | CrashPointReplaySegmentSlice | Regex | Type name |
| `Layer0StateJudgement(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateJudgement)` | CrashStateJudgement | Regex | Type name |
| `Layer0StateObserving(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateObserving)` | CrashStateObserving | Regex | Type name |
| `Layer0StatePlan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePlan)` | CrashStatePlan | Regex | Type name |
| `Layer0StatePosition(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePosition)` | CrashStatePosition | Regex | Type name |
| `Layer0Tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Tally)` | CrashPointReplayTally | Regex | Type name |
| `Layer0ToolchainIdentity(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ToolchainIdentity)` | CrashPointReplayToolchainIdentity | Regex | Type name |
| `Layer0VersionAtTheRoot(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionAtTheRoot)` | CrashPointReplayVersionAtTheRoot | Regex | Type name |
| `Layer0VersionTable(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionTable)` | CrashPointReplayVersionTable | Regex | Type name |
| `enumerate_layer0_quick_tier_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_quick_tier_versions)` | enumerate_crash_states_quick_tier_versions | Regex | Function name |
| `enumerate_layer0_selecting_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting_versions)` | enumerate_crash_states_selecting_versions | Regex | Function name |
| `enumerate_layer0_selecting(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting)` | enumerate_crash_states_selecting | Regex | Function name |
| `enumerate_layer0_through_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_through_the_library)` | enumerate_crash_states_through_the_library | Regex | Function name |
| `enumerate_layer0_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_versions)` | enumerate_crash_states_versions | Regex | Function name |
| `enumerate_layer0_with_expansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_with_expansion)` | enumerate_crash_states_with_expansion | Regex | Function name |
| `enumerate_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0)` | enumerate_crash_states | Regex | Function name; also invocation recognizing cases in `crash-case-check.py` |
| `judge_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow_with_slices)` | judge_crash_state_range_of_the_read_back_flow_with_slices | Regex | Function name |
| `judge_layer0_state_range_of_the_read_back_flow(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow)` | judge_crash_state_range_of_the_read_back_flow | Regex | Function name |
| `judge_layer0_state_range_against_the_version_table_naming_the_publish(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_against_the_version_table_naming_the_publish)` | judge_crash_state_range_against_the_version_table_naming_the_publish | Regex | Function name (local closure) |
| `judge_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range)` | judge_crash_state_range | Regex | Function name |
| `layer0_enumeration_tables_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables_with_slices)` | crash_state_enumeration_tables_with_slices | Regex | Function name |
| `layer0_enumeration_tables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables)` | crash_state_enumeration_tables | Regex | Function name |
| `layer0_judge_route(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_route)` | crash_point_replay_judge_route | Regex | Function name |
| `layer0_ordinal_with_the_same_landing(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_ordinal_with_the_same_landing)` | crash_state_ordinal_with_the_same_landing | Regex | Function name |
| `layer0_plan_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_plan_state_count)` | crash_point_replay_plan_state_count | Regex | Function name |
| `layer0_state_count_with_the_torn_third_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_the_torn_third_state)` | crash_state_count_with_the_torn_third_state | Regex | Function name |
| `layer0_state_count_with_torn_in_place_overwrites(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_torn_in_place_overwrites)` | crash_state_count_with_torn_in_place_overwrites | Regex | Function name |
| `layer0_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count)` | crash_state_count | Regex | Function name |
| `layer0_state_ranges_by_segment_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment_with_slices)` | crash_state_ranges_by_segment_with_slices | Regex | Function name |
| `layer0_state_ranges_by_segment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment)` | crash_state_ranges_by_segment | Regex | Function name |
| `layer0_tally_from_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_from_the_library)` | crash_point_replay_tally_from_the_library | Regex | Function name |
| `layer0_tally_under_the_expansion_of_the_plan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_under_the_expansion_of_the_plan)` | crash_point_replay_tally_under_the_expansion_of_the_plan | Regex | Function name |
| `layer0_worker_threads(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_worker_threads)` | crash_point_replay_worker_threads | Regex | Function name |
| `sample_layer0_states_on_the_cpu_with_threads_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_with_threads_against_the_version_table)` | sample_crash_states_on_the_cpu_with_threads_against_the_version_table | Regex | Function name |
| `sample_layer0_states_on_the_cpu_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_against_the_version_table)` | sample_crash_states_on_the_cpu_against_the_version_table | Regex | Function name |
| `sample_layer0_states_on_the_cpu(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu)` | sample_crash_states_on_the_cpu | Regex | Function name |
| `visit_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range_of_the_read_back_flow_with_slices)` | visit_crash_state_range_of_the_read_back_flow_with_slices | Regex | Function name |
| `visit_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range)` | visit_crash_state_range | Regex | Function name (local closure) |
| `synthetic_layer0_log(?![A-Za-z0-9_])(?<![A-Za-z0-9_]synthetic_layer0_log)` | synthetic_crash_point_replay_log | Regex | Function synthesizing log in `admission.py` selftest |
| `run_layer0_stage_shard_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_shard_cells)` | run_crash_point_replay_stage_shard_cells | Regex | Function in `admission.py` |
| `run_layer0_stage_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_cells)` | run_crash_point_replay_stage_cells | Regex | Function in `admission.py` |
| `layer0_lines(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_lines)` | crash_point_replay_lines | Regex | Local name in `admission.py` |
| `LAYER0_FINDING_SAMPLES_KEPT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING_SAMPLES_KEPT)` | CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT | Regex | Constant name |
| `LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE | Regex | Constant name |
| `LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE | Regex | Constant name |
| `LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE | Regex | Constant name |
| `LAYER0_RED_PASS_POOL_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_POOL_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER | Regex | Constant name |
| `LAYER0_RED_PASS_RECORD_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_RECORD_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER | Regex | Constant name |
| `LAYER0_SAMPLING_ITEM(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SAMPLING_ITEM)` | CRASH_POINT_REPLAY_SAMPLING_ITEM | Regex | Constant name |
| `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE | Regex | Constant name |
| `READ_FAULTS_OF_A_LAYER0_STATE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]READ_FAULTS_OF_A_LAYER0_STATE)` | READ_FAULTS_OF_A_CRASH_STATE | Regex | Constant name |
| `LAYER0_STAGE_CASES(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_STAGE_CASES)` | CRASH_POINT_REPLAY_STAGE_CASES | Regex | Constant in `admission.py` |
| `LAYER0_PARALLEL_FINISHED_PREFIX(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED_PREFIX)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX | Regex | Constant in `admission.py` |
| `LAYER0_THREADS_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_THREADS_VARIABLE)` | CRASH_POINT_REPLAY_THREADS_VARIABLE | Regex | Constant in `admission.py` |
| `LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME)` | CRASH_POINT_REPLAY_START_OVER_LIBRARY_DIRECTORY_NAME | Regex | Constant in `admission.py` |
| `layer0-start-over(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-start-over)` | crash-point-replay-start-over | Regex | Value of preceding constant (directory name for restart library kind) |
| `LAYER0_SHARD_CONFIGURATION_KEYS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_KEYS)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS | Regex | Constant for two-host shard configuration judgment |
| `SINGLEFS_LAYER0_THREADS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_THREADS)` | SINGLEFS_CRASH_POINT_REPLAY_THREADS | Regex | Environment variable |
| `SINGLEFS_LAYER0_FINDINGS_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_FINDINGS_FILE)` | SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE | Regex | Environment variable |
| `SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE | Regex | Environment variable |
| `SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_EXIT_PAUSE | Regex | Environment variable (two-host wrapper selftest) |
| `LAYER0_SHARD_CONFIGURATION_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_FILE)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE | Regex | Variable printed by configuration judgment `--emit-assignments` |
| `GATE_LAYER0_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]GATE_LAYER0_BREAK)` | GATE_CRASH_POINT_REPLAY_BREAK | Regex | Gate break switch |
| `LAYER0_SHARD_RUN_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_BREAK)` | CRASH_POINT_REPLAY_SHARD_RUN_BREAK | Regex | Two-host wrapper break switch |
| `LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER)` | CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER | Regex | Driver variable for two-host wrapper |
| `STUB_LAYER0_REUSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_REUSED)` | STUB_CRASH_POINT_REPLAY_REUSED | Regex | Stub variable for fake cargo in gate fixtures |
| `STUB_LAYER0_EXIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_EXIT)` | STUB_CRASH_POINT_REPLAY_EXIT | Regex | Stub variable for fake cargo in gate fixtures |
| `LAYER0_ACQUISITION_BARRIER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_ACQUISITION_BARRIER)` | CRASH_POINT_REPLAY_ACQUISITION_BARRIER | Regex | Count line prefix |
| `LAYER0B_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B_FAST)` | CRASH_POINT_REPLAY_FIXED_SCRIPT_FAST | Regex | Count line prefix for file overwrite fixed script stream quick tier (B is ordinal substitute, changed with prefix) |
| `LAYER0B(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B)` | CRASH_POINT_REPLAY_FIXED_SCRIPT | Regex | Count line prefix for file overwrite fixed script stream |
| `LAYER0_FINDINGS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS)` | CRASH_POINT_REPLAY_FINDINGS | Regex | Summary marker for finding logs |
| `LAYER0_FINDING(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING)` | CRASH_POINT_REPLAY_FINDING | Regex | Per-item marker for finding logs |
| `layer0_finding_threshold(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_threshold)` | crash_point_replay_finding_threshold | Regex | Line kind for finding logs |
| `layer0_finding_new(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_new)` | crash_point_replay_finding_new | Regex | Line kind for finding logs |
| `layer0_findings_begin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_begin)` | crash_point_replay_findings_begin | Regex | Line kind for finding logs |
| `layer0_findings_summary(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_summary)` | crash_point_replay_findings_summary | Regex | Line kind for finding logs |
| `layer0_finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding)` | crash_point_replay_finding | Regex | Line kind for finding logs |
| `LAYER0_FLOW_RECORDED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FLOW_RECORDED)` | CRASH_POINT_REPLAY_FLOW_RECORDED | Regex | Log marker |
| `LAYER0_IO_MIN_8192_MOUNT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_IO_MIN_8192_MOUNT)` | CRASH_POINT_REPLAY_IO_MIN_8192_MOUNT | Regex | Count line prefix |
| `LAYER0_JUDGE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_JUDGE)` | CRASH_POINT_REPLAY_JUDGE | Regex | Log marker |
| `LAYER0_PARALLEL_FINISHED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED | Regex | Log marker |
| `LAYER0_PARALLEL_LINE_ONE_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE_FAST)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH_FAST | Regex | Count line prefix for multi-record publish stream quick tier |
| `LAYER0_PARALLEL_LINE_ONE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH | Regex | Count line prefix for multi-record publish stream |
| `LAYER0_PARALLEL_START(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_START)` | CRASH_POINT_REPLAY_PARALLEL_START | Regex | Log marker |
| `LAYER0_POSITION_ADDRESSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_POSITION_ADDRESSED)` | CRASH_POINT_REPLAY_POSITION_ADDRESSED | Regex | Count line prefix |
| `LAYER0_PROGRESS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PROGRESS)` | CRASH_POINT_REPLAY_PROGRESS | Regex | Progress line marker |
| `LAYER0_SPILL_OVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SPILL_OVER)` | CRASH_POINT_REPLAY_SPILL_OVER | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW | Regex | Count line prefix |
| `LAYER0_TREE_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT | Regex | Count line prefix |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD_FAST | Regex | Count line prefix |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD | Regex | Count line prefix |
| `layer0-unlink-after-the-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-unlink-after-the-multi-record-publish-stream)` | unlink-after-the-multi-record-publish-stream | Regex | crash-case key (`crash-case:` trailing part) |
| `layer0-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-multi-record-publish-stream)` | multi-record-publish-stream | Regex | crash-case key |
| `layer0-position-addressed-tree-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-position-addressed-tree-streams)` | position-addressed-tree-streams | Regex | crash-case key |
| `layer0-tree-split-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-tree-split-streams)` | tree-split-streams | Regex | crash-case key |
| `layer0-first-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-first-stream)` | new-pool-file-creation-stream | Regex | crash-case key: new pool file creation stream |
| `layer0-second-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-second-stream)` | overwrite-fixed-script-stream | Regex | crash-case key: file overwrite fixed script stream |
| `a_recording_recovery_that_differs_from_the_layer0_report_is_counted(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_recording_recovery_that_differs_from_the_layer0_report_is_counted)` | a_recording_recovery_that_differs_from_the_crash_point_replay_report_is_counted | Regex | Test function name |
| `the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix)` | the_state_ordinal_decodes_to_the_same_persisted_set_as_crash_point_replay_mixed_radix | Regex | Test function name |
| `worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine(?![A-Za-z0-9_])(?<![A-Za-z0-9_]worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine)` | worker_threads_come_from_the_crash_point_replay_variable_then_the_checking_threads_variable_then_the_machine | Regex | Test function name |
| `a_layer0_item_name_cannot_be_a_path_segment_with_a_slash(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_layer0_item_name_cannot_be_a_path_segment_with_a_slash)` | a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash | Regex | Test function name |
| `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations)` | crash_point_replay_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations | Regex | Test function name (also `stage-inputs.tsv` `test=` column) |
| `layer0_quick_tier_matches_the_full_tally_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches_the_full_tally_shape)` | crash_point_replay_quick_tier_matches_the_full_tally_shape | Regex | Test function name |
| `the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally)` | the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_crash_point_replay_tally | Regex | Test function name |
| `the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape)` | the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_crash_point_replay_shape | Regex | Test function name |
| `acquisition-barrier-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]acquisition-barrier-layer0)` | acquisition-barrier-crash-point-replay | Regex | Pool name |
| `formatted-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-fast)` | formatted-crash-point-replay-fast | Regex | Pool name |
| `formatted-layer0-new-pool-file-creation-reference(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-new-pool-file-creation-reference)` | formatted-crash-point-replay-new-pool-file-creation-reference | Regex | Label |
| `formatted-layer0-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-registered)` | formatted-crash-point-replay-registered | Regex | Pool name |
| `formatted-layer0-same-as-new-pool-file-creation(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-same-as-new-pool-file-creation)` | formatted-crash-point-replay-same-as-new-pool-file-creation | Regex | Label |
| `parallel-line-one-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-fast)` | multi-record-publish-crash-point-replay-fast | Regex | Pool name |
| `parallel-line-one-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-full)` | multi-record-publish-crash-point-replay-full | Regex | Pool name |
| `unlink-after-multi-record-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-fast)` | unlink-after-multi-record-crash-point-replay-fast | Regex | Pool name |
| `unlink-after-multi-record-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-full)` | unlink-after-multi-record-crash-point-replay-full | Regex | Pool name |
| `unlink-after-multi-record-layer0-rerun(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-rerun)` | unlink-after-multi-record-crash-point-replay-rerun | Regex | Pool name |
| `spill-over-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]spill-over-layer0)` | spill-over-crash-point-replay | Regex | Pool name |
| `layer0-b-controls(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-b-controls)` | crash-point-replay-b-controls | Regex | Pool name |
| `layer0-c-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-c-registered)` | crash-point-replay-c-registered | Regex | Pool name |
| `layer0-d-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-d-registered)` | crash-point-replay-d-registered | Regex | Pool name |
| `layer0-e-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-fast)` | crash-point-replay-e-fast | Regex | Pool name |
| `layer0-e-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-full)` | crash-point-replay-e-full | Regex | Pool name |
| `layer0-reuse-that-never-landed(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-reuse-that-never-landed)` | crash-point-replay-reuse-that-never-landed | Regex | Pool name |
| `layer0-residual-record(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-residual-record)` | crash-point-replay-residual-record | Regex | Pool name |
| `layer0-stale-tail(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stale-tail)` | crash-point-replay-stale-tail | Regex | Pool name |
| `layer0-threaded-sampling(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-threaded-sampling)` | crash-point-replay-threaded-sampling | Regex | Pool name |
| `layer0-partial(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-partial)` | crash-point-replay-partial | Regex | Pool name |
| `layer0-control(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-control)` | crash-point-replay-control | Regex | Pool name |
| `layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-full)` | crash-point-replay-full | Regex | Pool name; hyphens count as word characters |
| `caught_with_the_layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]caught_with_the_layer0_threads_value)` | caught_with_the_crash_point_replay_threads_value | Regex | Local variable |
| `layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_threads_value)` | crash_point_replay_threads_value | Regex | Local variable |
| `layer0_quick_tier_matches(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches)` | crash_point_replay_quick_tier_matches | Regex | Local variable |
| `sample-layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample-layer0)` | sample-crash-point-replay | Regex | Gate fixture name |
| `sample_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0)` | sample_crash_point_replay | Regex | Fixture name in heavy test guard selftest |
| `layer0-by-name(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-by-name)` | crash-point-replay-by-name | Regex | Break switch for `prove-red.sh` |
| `named_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]named_layer0)` | named_crash_point_replay | Regex | Name in `prove-red.sh` |
| `demo_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]demo_layer0)` | demo_crash_point_replay | Regex | Fixture target for `prove-red.sh` |
| `some_layer0_stream(?![A-Za-z0-9_])(?<![A-Za-z0-9_]some_layer0_stream)` | some_crash_point_replay_stream | Regex | Fixture binary name in heavy test guard selftest |
| `layer0-sample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-sample)` | crash-point-replay-sample | Regex | Version string for fixture fake tool (`0.0.0-layer0-sample`) |
| `layer0-demo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-demo)` | crash-point-replay-demo | Regex | Tag prefix for two-host wrapper selftest |
| `layer0-shaped-case-records-and-imports(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shaped-case-records-and-imports)` | crash-case-shaped-records-and-imports | Regex | Selftest cell name in two crash amplification scripts |
| `library-layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]library-layer0-shape)` | library-crash-case-shape | Regex | Library directory name in two crash amplification scripts selftest |
| `layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shape)` | crash-case-shape | Regex | Fixture file name in two crash amplification scripts selftest |
| `layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shape)` | crash_case_shape | Regex | Variable name in two crash amplification scripts selftest |
| `STUB_CASE_SHAPE=layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_CASE_SHAPE=layer0)` | STUB_CASE_SHAPE=crash-case | Regex | Fake case shape in two crash amplification scripts selftest |

**Follow-up renamed file names**: `crates/singlefs-core/src/superblock.rs` → `system_configuration.rs`; five test rigs, five mutation tables, five retained artifacts (E100 (three-segment geometry of system configuration), E115 (completeness and byte budget of system configuration field table), E124 (system configuration field table recomputed per settled item 14 of D18), E126 (costs of four candidates for system configuration slot width), E147 (system configuration field table recomputed per new pool file creation byte table)). How migrations are done is described in [.claude/rules/path-moves.md](../rules/path-moves.md) under "How to do it".

**Unchanged**: The **value** `*b"SFSB"` of `SUPERBLOCK_MAGIC` remains untouched—that is a magic number written on disk, and changing it is equivalent to changing the disk format. The constant name itself was renamed.

**Originals before rename**: In git. The previous state before the Chinese batch is prior to commit `495bede`; English identifiers and frozen directories batch is seen in workspace modifications after `4105e70`. Retained artifacts and source code were changed together in the same run, so byte-for-byte exact comparison in `replay.sh` holds before and after the rename—this is proven by the full rerun after changes, not merely claimed.
