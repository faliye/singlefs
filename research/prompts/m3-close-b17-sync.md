<!-- knowledge-sync -->
# m3-close-b17 阶段同步

触发文件：.claude/agent-common.md、.claude/gate.d/crates-mutation-light-rows.tsv、.claude/gate.d/lib-forbidden-notations.py、.claude/gate.d/replay-comparison-units.tsv、.claude/gate.d/timing-load-field-lag.tsv、.claude/hooks/heavy-test-guard.sh、.claude/hooks/lib_heavy_tests.py、.claude/rules/implementation-workflow.md、.claude/rules/three-way-inference.md、.claude/rules/verification.md、crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs、crates/singlefs-checker-tier/src/crash_judge_dispatch.rs、crates/singlefs-checker-tier/src/crash_state_enumeration.rs、scripts/experiment-apparatus/README.md、scripts/experiment-apparatus/e184-segment-one-long-run.sh、scripts/experiment-apparatus/e184_r2_verdict_compare.mutations.tsv、scripts/experiment-apparatus/e184_r2_verdict_compare.py、scripts/experiment-apparatus/e184_r2_verdict_compare_test.py、scripts/experiment-apparatus/replay.sh、scripts/mutation/broad-mutation.py、scripts/mutation/crates-mutation-rows.py、scripts/mutation/lib_mutation_units.py、.claude/agents/clerk.md、.claude/agents/sweep.md、.claude/agents/tooling-writer.md、.claude/gate.d/break-switch-headers-exempt.tsv、.claude/gate.d/break-switch-outside-selftest-pending.tsv、.claude/gate.d/break-switch-replay-files.tsv、.claude/gate.d/checker-direction-entries.tsv、.claude/gate.d/checker-independence-and-sync.sh、.claude/gate.d/checker-tier-broad-mutation.sh、.claude/gate.d/checker-tier-lkmm.sh、.claude/gate.d/clause-enum-pairs.tsv、.claude/gate.d/code-source-discipline.sh、.claude/gate.d/decision-evidence-lag.tsv、.claude/gate.d/deferral-registries.tsv、.claude/gate.d/divergence-events.tsv、.claude/gate.d/entry-scripts.tsv、.claude/gate.d/experiment-number-directories.tsv、.claude/gate.d/field-pointer-blanks.tsv、.claude/gate.d/hardcoded-quota-exempt.tsv、.claude/gate.d/harness-model-differential-and-scenarios.sh、.claude/gate.d/harness-test-environment.sh、.claude/gate.d/lib-lag-baseline.py、.claude/gate.d/mount-transparency-d13-rows-without-bytes.tsv、.claude/gate.d/path-moves.tsv、.claude/gate.d/stage-break-switch-replay-exempt.tsv、.claude/gate.d/test-function-names-exempt.tsv、.claude/gate.d/trilingual-glossary-longer-words.tsv、.claude/gate.d/trilingual-untranslated-exempt.tsv、.claude/gate.d/unreachable-claims-stock.tsv、.claude/gate.d/verus-trust-list.tsv、.claude/hooks/ask-user-claim-guard.sh、.claude/hooks/completion-cleanup-guard.sh、.claude/hooks/continuation-guard.sh、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a0a2dfee322e26786.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a28fe10b58d88beec.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a303d33b0ccd5e68f.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a3652270eb80e4fd0-delivery-colon.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a3652270eb80e4fd0-section-colon.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a3652270eb80e4fd0.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a42154ccf8829dc32.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a4737c6f3660f4f7e-own-result.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a4737c6f3660f4f7e.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a61301bfaed0a3433.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a75e9e30847f57bcd-forbidden-own.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a75e9e30847f57bcd.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a77dffd2065a0ba7e.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a789e7091d91430aa.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a86c5c8fb8ec28789.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a8a3bd6f093e19673.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a8afb2185cfa840cc.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/a9717cf07fb6e7c21.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/aa9ec3423c1f54bba-own-result.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/aa9ec3423c1f54bba.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/ab7f880bc2d28e03a.txt、.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/sources.tsv、.claude/hooks/handback-guard.sh、.claude/hooks/lib_long_run.py、.claude/hooks/lib_prompt_scratch_line.py、.claude/hooks/lib_remind_waiter.py、.claude/hooks/lib_run_scope.py、.claude/hooks/lib_snapshot_conflicts.py、.claude/hooks/lib_three_way_legs.py、.claude/hooks/lib_work_root.py、.claude/hooks/lib_workspace_cargo_lock.py、.claude/hooks/lib_write_scope.py、.claude/hooks/lib_writer_lock.py、.claude/hooks/remind-rearm-stop-guard.sh、.claude/hooks/session-start.sh、.claude/hooks/shared-index-stale-reminder.sh、.claude/hooks/timer-only-remind-guard.sh、crates/singlefs-background-gpu/src/adapter_inventory.rs、crates/singlefs-background-gpu/src/background_gpu.rs、crates/singlefs-background-gpu/src/background_gpu_compute.rs、crates/singlefs-background-gpu/src/checksum_kernel_bitwise.wgsl、crates/singlefs-background-gpu/src/checksum_kernel_table_driven.wgsl、crates/singlefs-background-gpu/src/checksum_kernels.rs、crates/singlefs-background-gpu/src/gpus_of_this_host.rs、crates/singlefs-background-gpu/src/lib.rs、crates/singlefs-background-gpu/src/live_slot_kernel_searching_sorted_records.wgsl、crates/singlefs-background-gpu/src/live_slot_kernel_walking_every_record.wgsl、crates/singlefs-background-gpu/src/live_slot_kernels.rs、crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs、crates/singlefs-background-gpu/src/region_record_kernel_searching_sorted_indexes.wgsl、crates/singlefs-background-gpu/src/region_record_kernel_walking_the_mapping.wgsl、crates/singlefs-background-gpu/src/region_record_kernels.rs、crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs、crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs、crates/singlefs-background-gpu/src/work_split_by_measured_cost.rs、crates/singlefs-checker-tier/src/bad_disk_input.rs、crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs、crates/singlefs-checker-tier/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs、crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs、crates/singlefs-checker-tier/src/crash_full_run_backlog.rs、crates/singlefs-checker-tier/src/crash_given_states.rs、crates/singlefs-checker-tier/src/crash_identity.rs、crates/singlefs-checker-tier/src/crash_library.rs、crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs、crates/singlefs-checker-tier/src/lib.rs、crates/singlefs-core/src/background_compute.rs、crates/singlefs-core/src/checksum.rs、crates/singlefs-core/src/journal.rs、crates/singlefs-core/src/unit.rs、crates/singlefs-harness/src/read_tally.rs、crates/singlefs-harness/src/temporary_files.rs、crates/singlefs-harness/src/verification_read.rs、crates/singlefs-replay-prefix/src/README.md、crates/singlefs-replay-prefix/src/gc_crash_proofs.rs、crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs、crates/singlefs-unit-graph/src/bin/unit_graph.rs、crates/singlefs-unit-graph/src/build_features.rs、crates/singlefs-unit-graph/src/closure.rs、crates/singlefs-unit-graph/src/entry_groups.rs、crates/singlefs-unit-graph/src/lib.rs、crates/singlefs-unit-graph/src/manifest.rs、crates/singlefs-unit-graph/src/sha256.rs、crates/singlefs-unit-graph/src/units.rs、crates/singlefs-unit-graph/src/verus_ghost_code.rs、crates/singlefs-unit-graph/src/workspace.rs、scripts/README.md、scripts/agents/README.md、scripts/agents/agent-cost.py、scripts/agents/cite-check.py、scripts/agents/copy-originals.py、scripts/agents/remind.sh、scripts/agents/snapshot-tree.py、scripts/agents/subagent_transcript.py、scripts/agents/tests/README.md、scripts/agents/tests/test_agents.py、scripts/agents/three-way-materials.py、scripts/crash/crash-amplification-item.sh、scripts/crash/crash-amplification-lib.sh、scripts/crash/crash-point-replay-shard-configuration-check.sh、scripts/crash/gpu-exemptions-check.py、scripts/crash/gpu-quiesce-run.sh、scripts/devices/README.md、scripts/devices/dm-owned.sh、scripts/devices/e125-zoned-wp-probe.sh、scripts/devices/e129-tear-injector.sh、scripts/devices/e129-thin-neighbour.sh、scripts/devices/e129-thin-rmw.sh、scripts/devices/e152-run.sh、scripts/devices/e152-stage-root.sh、scripts/devices/e34-iomin-probe.sh、scripts/devices/snapshot-check.sh、scripts/devices/stripe-map-probe.sh、scripts/devices/tests/test_devices.py、scripts/devices/vm-bench.sh、scripts/devices/vm-geom.sh、scripts/devices/vm-kernel.sh、scripts/experiment-apparatus/device-code-key.py、scripts/experiment-apparatus/e116_pack_settle.rs、scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/fake_multi_host_run.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/peer_side.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_11.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_11.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_on_the_revision_6_tree_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_on_the_revision_8_tree_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_two_on_the_revision_6_tree_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_two_revision_10_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_two_revision_11_multi_host.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_11.sh、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_11.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_default_threads.py、scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_frozen_o3.py、scripts/experiment-apparatus/e185-r2-segment-run.sh、scripts/experiment-apparatus/e185-r3-segment-run.sh、scripts/experiment-apparatus/e185-r4-segment-run.sh、scripts/experiment-apparatus/e185_r1_compare.mutations.tsv、scripts/experiment-apparatus/e185_r1_compare.py、scripts/experiment-apparatus/e185_r1_compare_test.py、scripts/experiment-apparatus/e185_r4_baseline_rejudge.mutations.tsv、scripts/experiment-apparatus/e185_r4_baseline_rejudge.py、scripts/experiment-apparatus/e185_r4_baseline_rejudge_test.py、scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py、scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py、scripts/experiment-apparatus/e43_extension_point_budget.rs、scripts/experiment-apparatus/e49_chain_width.rs、scripts/experiments/README.md、scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/test_default_threads.py、scripts/gate/baseline-diff.sh、scripts/gate/checker-direction-overlap.py、scripts/gate/data-name-lint.py、scripts/gate/gate-structure-check.py、scripts/gate/hook-swap.py、scripts/gate/lib_rebuilt_packages.py、scripts/gate/lib_workspace_lock.py、scripts/gate/lkmm-selftest.sh、scripts/gate/staged-tree-worktree.sh、scripts/gate/test-targets-parallel.py、scripts/gate/tests/test_review_regressions.py、scripts/gate/wallclock-assert-lint.py、scripts/hosts/capped.sh、scripts/hosts/clean-session-scratch.py、scripts/hosts/minimal-copy.py、scripts/hosts/resource_release_broadcast.py、scripts/hosts/rocksdb-prebuilt.py、scripts/hosts/run-build-directory.sh、scripts/hosts/run_build_directory.py、scripts/hosts/source-stamp.py、scripts/hosts/test-environment-check.py、scripts/hosts/tests/test_review_regressions.py、scripts/hosts/work-directory-ownership.sh、scripts/kb/README.md、scripts/kb/batch-target.py、scripts/kb/claim-experiment.sh、scripts/kb/doc-consolidate.py、scripts/kb/history-consolidate.py、scripts/kb/insert-row.py、scripts/kb/kb-sections.py、scripts/kb/lib_basis_paths.py、scripts/kb/lib_experiment_number_directories.py、scripts/kb/lib_notation_check.py、scripts/kb/migrate-changelog-format.py、scripts/kb/milestone-close-owed-check.py、scripts/kb/move-directory-in-use-check.py、scripts/kb/replace-batch.py、scripts/kb/sweep-term.py、scripts/mutation/crates-mutation-light-rows.py、scripts/mutation/mutation-prune.py、scripts/mutation/prove-red.sh、.claude/gate.d/lib-governance-refs.py

改动范围取四样的并集，再按 `.claude/gate.d/knowledge-sync-triggers.tsv` 的正则筛：提交对象 `1e3079d7a07304ceaf5a9de84f080853d61f52b5` 相对 `841b3beabb98` 的 183 份路径（`git diff --name-only 841b3beabb98 1e3079d7a07304ceaf5a9de84f080853d61f52b5`）；范围清单里只进宽命令的 8 份与没进命令的 28 份；四份写回报告改过的 42 份；主 agent 自己改的 `.claude/gate.d/replay-comparison-units.tsv`、`.claude/kb/experiments.en.md`、`.claude/kb/checks-owed*.md`、`.claude/kb/decisions.md`。并集 234 份，命中正则 22 份。

## 搜索

事实表那一段（范围 `--base 841b3beabb98 --target 1e3079d7a07304ceaf5a9de84f080853d61f52b5`，`<草稿>` 是事实表的草稿目录）：

- `python3 scripts/kb/stale-candidates.py --changes --base 841b3beabb98 --target 1e3079d7a07304ceaf5a9de84f080853d61f52b5 --out <草稿>/changes.md` → 8 条变更记录
- `python3 scripts/kb/stale-candidates.py --check-facts <草稿>/facts.tsv --base 841b3beabb98 --target 1e3079d7a07304ceaf5a9de84f080853d61f52b5` → 8 条变更记录都有出处，49 行事实的检索词都在基准那一版的现状句里命中（新立事项除外）
- `python3 scripts/kb/stale-candidates.py --facts <草稿>/facts.tsv --base 841b3beabb98 --target 1e3079d7a07304ceaf5a9de84f080853d61f52b5 --out <草稿>/candidates.tsv` → 候选表 1350 行（1169 个不同的行），27 组；F20、F28–F48 这 22 行不出候选（只改措辞或新立）
- 每个检索词在基准与结束那一版现状载体里的命中行数（`python3 <草稿>/tools/count.py <检索词>…`，与 `--check-facts` 同一套语料与匹配）：
  - F1 `依赖版本|NOT_JUDGING_FILES|EXCLUDED_PROGRAM_EDGES` → 基准 16、结束 25
  - F2 `判法单元指纹|覆盖号|coverage_id` → 基准 76、结束 95
  - F3 `复跑副本|复跑的输出|复跑目录` → 基准 19、结束 25
  - F4 `GPU 那一趟|GPU 趟|每条变异两趟` → 基准 24、结束 25
  - F5 `gpu-verdicts|IMPORT_GPU|第二台的库|导回|导入本机` → 基准 6、结束 13
  - F6 `路径尾部|符号链接|本仓那一份` → 基准 34、结束 43
  - F7 `e165-pin-red-examples|全部实验复跑|experiment-replay` → 基准 85、结束 87
  - F8 `I-7\.1[89]|回收门槛字段|根记录宽|钉槽 p` → 基准 23、结束 54
  - F9 `C560|满环|i718-oldslot` → 基准 13、结束 41
  - F10 `环里最旧有效根|六处同取` → 基准 70、结束 70
  - F11 `第 0 代根` → 基准 60、结束 65
  - F12 `C478|子树覆盖区间|check_index_node_keys` → 基准 37、结束 45
  - F13 `C743|落点公式` → 基准 71、结束 82
  - F14 `C656|C659` → 基准 82、结束 88
  - F15 `(11|12) 段|(78|80) 段` → 基准 66、结束 80
  - F16 `E161|崩溃放量的去重与分段耗时` → 基准 64、结束 80
  - F17 `gpu-quiesce-run|verdict-store,gpu` → 基准 75、结束 82
  - F18 `E184|常规层与大负载层的代价计数` → 基准 49、结束 79
  - F19 `16777240|16777243|16842782|16842785|16842807|16842815|1358954634|1358954642` → 基准 47、结束 68
  - F20 `钉值冒烟|missing-pinned|缺钉值` → 基准 0、结束 3
  - F21 `TornImageTables|撕裂表|撕裂镜像` → 基准 15、结束 24
  - F22 `findings_log|QUICK_TIER_STATES` → 基准 14、结束 19
  - F23 `一次性库|tables_reused|after_the_oracle_fixes` → 基准 43、结束 47
  - F24 `front_hole|前洞|TXG_AT_WHICH_THE_FRONT_HOLE` → 基准 7、结束 10
  - F25 `逐行上限` → 基准 15、结束 18
  - F26 `session-names|会话号` → 基准 34、结束 40
  - F27 `deferred-items-registered|补登记|没写去向` → 基准 19、结束 25
  - F28 `照落点公式摆根` → 基准 0、结束 3
  - F29 `E183|E165` → 基准 406、结束 429
  - F30 `C879` → 基准 0、结束 7
  - F31 `C892` → 基准 0、结束 9
  - F32 `C893` → 基准 0、结束 4
  - F33 `C901` → 基准 0、结束 9
  - F34 `C908` → 基准 0、结束 6
  - F35 `C909` → 基准 0、结束 6
  - F36 `C910` → 基准 0、结束 4
  - F37 `C911` → 基准 0、结束 4
  - F38 `C912` → 基准 0、结束 5
  - F39 `C913` → 基准 0、结束 4
  - F40 `九十八、|第九十八节` → 基准 0、结束 6
  - F41 `一百零六` → 基准 0、结束 8
  - F42 `一百一十八` → 基准 0、结束 3
  - F43 `一百二十四` → 基准 0、结束 1
  - F44 `gov-b17` → 基准 0、结束 11
  - F45 `batch-target` → 基准 0、结束 0
  - F46 `i718-known-red|checker-tier-fix-2` → 基准 0、结束 6
  - F47 `crash-light-rerun` → 基准 0、结束 5
  - F48 `收拢表 37|collection-table` → 基准 0、结束 2
  - F49 `展开大小|inlined_size|内联大小|表达式数` → 基准 18、结束 20

逐行判那一段，八段各跑一次 `python3 scripts/kb/stale-candidates.py --check-report <候选表> <那一段的报告> --groups <组>`：

- `--groups F1-F5` → 183 行候选都有逐行判定，crates/ 下判要改要补的 0 行
- `--groups F6-F8` → 184 行，0 行
- `--groups F9-F11` → 176 行，0 行
- `--groups F12-F14` → 215 行，0 行
- `--groups F15-F16` → 160 行，0 行
- `--groups F17-F18` → 161 行，0 行
- `--groups F19,F21,F22,F23` → 158 行，0 行
- `--groups F24-F27,F49` → 113 行，crates/ 下判要改要补的 1 行都在「交实现员」一节里
- 八段合起来（主 agent）→ 1350 行候选都有逐行判定，crates/ 下判要改要补的 1 行都在「交实现员」一节里

八段在候选表之外另搜的：

- s1：`git grep -c EXCLUDED_PROGRAM_EDGES` 在 HEAD 与 `1e3079d7…` → 0（没有输出）；`git show :scripts/gate/lib_function_units.py | grep -c EXCLUDED_PROGRAM_EDGES` → 11
- s4：`git show :.claude/kb/checks-owed.md | grep -nE '^\| (C656|C659) |^### 已还清'` → 3 行（第 597、874、875 行）；同一条对 `1e3079d7a…` → 3 行（第 426、428、599 行）；`git show :crates/singlefs-core/src/root_ring.rs | grep -c root_sits_in_the_ring_slot_the_formula_gives` → 5
- s5（候选外那一次）：`git grep -n -E '1 678 377 204|1,678,377,204|1678377204|16 777 240|16,777,240|481 次写|481 writes|481 回' 1e3079d7… -- .claude/kb crates/singlefs-checker/invariants README.md CLAUDE.md` → 33 行；去掉已判要改的 7 行与事件句 3 行，剩 23 行进下表「s5 候选外」那几行
- s6：`grep -c -i 'e184' .claude/gate.d/stage-inputs.tsv` → 0
- s7：`git grep '取号写两轮\|取号两轮'` 结束那一版 `.claude/kb/checks-owed.md` → 只命中 C555 那一行
- s8（反向问，工作区）：`git grep -c -I -E <检索词> -- '*.md' ':!research/prompts/**' ':!briefs/**'` 逐个检索词 → C892 欠账表三语各 2 处，C893 各 1 处，C901 各 1 行，C908–C913 各 1 行，`gov-b17` 调度记录 11 处、D13 变更史三语各 2 处，`i718-known-red|checker-tier-fix-2` 6 处，`crash-light-rerun` 调度记录 5 处，`收拢表 37|collection-table` 2 处
- s8：`grep -n '248 594\|332 394' crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 只看注释 → 0 行
- 报告只写了计数、没贴命令的（照报告原话记）：s3「对象与基准的 `walk.rs` 里 `I-7.18` 都是 0 处，工作区 6 处」「工作区 `root_record.rs` 里 `reclaim_threshold` 8 处，基准与结束那一版对象里 0 处」；s7「`git grep` 结束那一版 kb 与不变量清单里「mkfs 用过的池」「撞内存」「71.6」，只命中无关的几处」

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| `.claude/kb/verification-build.md:271` | 块键只由内容定（判器单元闭包、段写表与起步镜像摘要、段展开、工具链、解析出的依赖版本） | 改了：「解析出的依赖版本」改成「解析出的闭包真引到的依赖版本」（写回乙第 1 条） |
| `.claude/agent-common.md:58` | 全部实验复跑（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格）、E152 装置 | 不改：工作区已是新句，随提交一起进 |
| `.claude/rules/implementation-workflow.md:49` | 与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` | 不改：工作区已是新句，随提交一起进 |
| `.claude/rules/verification.md:136` | 与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7 | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/checks-owed.en.md:350` | 普通覆盖写那一形的在同一文件第 1360 行，实现批把这两处改成恰好 I-7.18 红；还欠三样：GPU 一侧没有对应的判别力变异（CPU 一侧的变异行由实现批新加）；实现并进 I-7.18 的实现批（场景期望与变异行）（英文版同一处） | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.ja.md:350` | 普通覆盖写那一形的在同一文件第 1360 行，实现批把这两处改成恰好 I-7.18 红；还欠三样：GPU 一侧没有对应的判别力变异（CPU 一侧的变异行由实现批新加）；实现并进 I-7.18 的实现批（场景期望与变异行）（日文版同一处） | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.md:350` | 普通覆盖写那一形的在同一文件第 1360 行，实现批把这两处改成恰好 I-7.18 红；还欠三样：GPU 一侧没有对应的判别力变异（CPU 一侧的变异行由实现批新加）；实现并进 I-7.18 的实现批（场景期望与变异行） | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.md:495` | 钉只在这个进程的内存里；根记录没有回收门槛字段（`crates/singlefs-core/src/root_record.rs` 只有 `rollback_floor`），池级 checker 取 max(F_生效, 环里最旧有效根)；实现并进实现批（系统配置落地补丁合入之后那一批） | 改了：现象格改成「实现批之前，钉只在内存里、没有门槛字段」并补现状（t 在偏移 428、p 在 436，宽 466，池级 checker 判 I-7.18、I-7.19），前置格改成已落地、写明还差两样；主 agent 认留在欠表（写回甲第 2 条） |
| `.claude/kb/checks-owed.md:546` | C773（读坏与全零根槽的回收门槛钉只在内存里） 的合成形态先落地；去向：…与 C773 同一作业点，实现批（系统配置落地补丁合入之后那一批） | 改了：前置格改成「已满足……那一批没做这一条。去向：未排进里程碑，下一批实现员并进」（写回甲第 3 条） |
| `.claude/kb/decisions/16-发布语义.md:39` | （C773（读坏与全零根槽的回收门槛钉只在内存里） 的合成形态，用户 2026-10-20 定，被攻过零轮；动格式，实现在系统配置落地补丁合入之后的实现批） | 改了：括注改成「已随它的实现批实现：根记录宽 466，t 在偏移 428、p 在偏移 436，池级 checker 判 I-7.18、I-7.19」，变更史 D16 节加 2026-10-26 块（写回甲第 6 条） |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:74` | 根槽宽（`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距] | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:353` | `physical_block_size` ∈ [根记录宽 457, 槽距] | 不改：工作区已是新句，随提交一起进 |
| `crates/singlefs-checker/invariants/02-根环.md:3` | 已实现（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根） 除外：形态 2026-10-20 定，随 C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批） | 不改：工作区已是新句，随提交一起进 |
| `crates/singlefs-checker/invariants/05-空间记账.md:8` | 钉落盘之后的并法（C773 定案，用户 2026-10-20 认；checker 未实现，随 C773 实现批，GPU 判器那一侧没有现成件） | 不改：工作区已是新句，随提交一起进 |
| `crates/singlefs-checker/invariants/README.md:29` | （新增） | 补了：在「同日加下标 103 的 I-3.17」之后补 C773 实现批加下标 104 的 I-7.18 与下标 105 的 I-7.19、GPU 判器走读内核同判（写回乙第 6 条） |
| `.claude/kb/checks-owed.en.md:350` | the implementation batch changes both to exactly I-7.18 red；the 20 images × 3 post-recovery expectations change to all green；(the CPU-side mutation row is added by the implementation batch)；implementation joins the I-7.18 implementation batch (scenario expectations and mutation row) | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.en.md:495` | (`crates/singlefs-core/src/mount.rs` line 2908)；(`crates/singlefs-core/src/make_filesystem.rs` line 826) | 改了：两个行号改成函数名 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`（写回甲第 2 条，C773 留在欠表） |
| `.claude/kb/checks-owed.ja.md:350` | 実装バッチがこの二箇所をちょうど I-7.18 が赤に改める；リカバリ後の 20 枚 × 3 個の期待はすべて緑に改める；（CPU 側のミューテーション行は実装バッチが新しく加える）；実装は I-7.18 の実装バッチ（シナリオの期待とミューテーション行）に併合する | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.ja.md:495` | （`crates/singlefs-core/src/mount.rs` 第 2908 行）；（`crates/singlefs-core/src/make_filesystem.rs` 第 826 行） | 改了：两个行号改成函数名 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`（写回甲第 2 条，C773 留在欠表） |
| `.claude/kb/checks-owed.md:350` | 实现批把这两处改成恰好 I-7.18 红；恢复之后的 20 张 × 3 个期望改成全绿；GPU 一侧没有对应的判别力变异（CPU 一侧的变异行由实现批新加）；实现并进 I-7.18 的实现批（场景期望与变异行） | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.md:495` | （`crates/singlefs-core/src/mount.rs` 第 2908 行）；（`crates/singlefs-core/src/make_filesystem.rs` 第 826 行） | 改了：两个行号改成函数名 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`（写回甲第 2 条，C773 留在欠表） |
| `.claude/kb/checks-owed.md:495` | （`crates/singlefs-core/src/mount.rs` 第 2908 行）；（`crates/singlefs-core/src/make_filesystem.rs` 第 826 行） | 改了：两个行号改成函数名 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`（写回甲第 2 条，C773 留在欠表） |
| `.claude/kb/checks-owed.md:350` | 实现批把这两处改成恰好 I-7.18 红；恢复之后的 20 张 × 3 个期望改成全绿；GPU 一侧没有对应的判别力变异（CPU 一侧的变异行由实现批新加）；实现并进 I-7.18 的实现批（场景期望与变异行） | 改了：C560 第 4、5 格改成恢复之前恰好 I-7.18 红已落地、恢复之后期望全绿不写张数、CPU 一侧变异行指 `crates/mutations.tsv` 第 5414 行、GPU 判别力变异记在 C908 ②（写回甲第 1 条） |
| `.claude/kb/checks-owed.md:495` | （`crates/singlefs-core/src/mount.rs` 第 2908 行）；（`crates/singlefs-core/src/make_filesystem.rs` 第 826 行） | 改了：两个行号改成函数名 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`（写回甲第 2 条，C773 留在欠表） |
| `.claude/kb/checks-owed.en.md:588` | the "**Owed**" of D18 (Information Carried in a Block) settled item 2 says "none"；once this has a number, the "**Owed**" of D18 (Information Carried in a Block) settled item 2 must point to it | 改了：问题格删「没登号」、「欠」写「无」改成已指 C909，前置格改成已接上这一笔（写回甲第 4 条） |
| `.claude/kb/checks-owed.en.md:905` | key 区间那一格归 C478（英文版同一处，原话「deletion线两个树根的层级」所在行） | 不改：工作区已是新句（C713 已挪进已还清表、不再提 C478），随提交一起进 |
| `.claude/kb/checks-owed.ja.md:905` | key 区间那一格归 C478（日文版同一处，原话「削除线两个树根的层级」所在行） | 不改：工作区已是新句（C713 已挪进已还清表、不再提 C478），随提交一起进 |
| `.claude/kb/checks-owed.ja.md:588` | D18 (ブロックに含める情報) 確定項目 2 の「**未済**」は「なし」と書いている；番号を付けた後、D18 (ブロックに含める情報) 確定項目 2 の「**未済**」をこの一件につなぐ必要がある | 改了：问题格删「没登号」、「欠」写「无」改成已指 C909，前置格改成已接上这一笔（写回甲第 4 条） |
| `.claude/kb/checks-owed.md:905` | 删除线两个树根的层级没有条款；key 区间那一格归 C478 | 不改：工作区已是新句（C713 已挪进已还清表、写「条款已写」、不再提 C478），随提交一起进 |
| `.claude/kb/checks-owed.md:588` | C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 实现员报告「交主 agent 的设计判断」第 1 条点名这一处，没登号；D18（块里携带什么信息） 已定项 2 的「**欠**」写「无」；立号之后 D18（块里携带什么信息） 已定项 2 的「**欠**」要接上这一笔 | 改了：问题格删「没登号」、「欠」写「无」改成已指 C909，前置格改成已接上这一笔（写回甲第 4 条） |
| `.claude/kb/decisions/18-information-carried-in-a-block.en.md:55` | Checker's check_index_node_keys ... checks the keys of the first and last entries; the root of parallel line 3 wrote out [2, 2] while subtree coverage was [2, 234]; both places must be modified per subtree coverage range (C478 ...) | 不改：工作区已是新句（s4 记的是改名前的 `18-块里携带什么信息.en.md` 第 55 行，工作区改名后的这一份已是中文第 55 行的译文），随提交一起进 |
| `.claude/kb/decisions/18-information-carried-in-a-block.en.md:64` | **Owed checks**: C478 | 不改：工作区已是新句（s4 记的是改名前的 `18-块里携带什么信息.en.md` 第 64 行），随提交一起进 |
| `.claude/kb/decisions/18-ブロックに含める情報.ja.md:55` | checker の check_index_node_keys ... 両箇所ともサブツリーカバー区間に従って修正が必要 (C478 ...) | 不改：工作区已是新句（s4 记的是改名前的 `18-块里携带什么信息.ja.md` 第 55 行），随提交一起进 |
| `.claude/kb/decisions/18-ブロックに含める情報.ja.md:64` | **未済の検査**：C478 | 不改：工作区已是新句（s4 记的是改名前的 `18-块里携带什么信息.ja.md` 第 64 行），随提交一起进 |
| `.claude/kb/experiments/172-删除写出的字节的干跑.md:164` | td、tt 头的层级与 key 区间只记值，对错归 C478（…） 与 C713（…） | 改了：「对错归 C478 与 C713」改成层级条款 D8 已定项 16 第 3 条与 D18 已定项 24 第 1 条、区间条款 D18 已定项 2，td 与 tt 的区间都由池级 checker 判，不另登账（写回丙第 4 条、收尾小批第 1 条） |
| `.claude/kb/layout/03-third-txn.en.md:13` | key range belongs to C478 (...) | 改了：状态格改成 key 区间已定（D18 已定项 2）、代码写队列首末两个号同值、区间由池级 checker 在树表走读里判 I-1.1，不另登账（写回丙第 4 条、收尾小批第 1 条） |
| `.claude/kb/layout/03-third-txn.ja.md:13` | キー範囲は C478 (...)に帰属 | 改了：状态格改成 key 区间已定（D18 已定项 2）、代码写队列首末两个号同值、区间由池级 checker 在树表走读里判 I-1.1，不另登账（写回丙第 4 条、收尾小批第 1 条） |
| `.claude/kb/layout/03-third-txn.md:13` | key 区间是格式级空白，条款没写这一格；key 区间归 C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） | 改了：状态格改成 key 区间已定（D18 已定项 2）、代码写队列首末两个号同值、区间由池级 checker 在树表走读里判 I-1.1，不另登账（写回丙第 4 条、收尾小批第 1 条） |
| `.claude/kb/milestone/02-second-txn.en.md:565` | key range of inode tree root takes start and end of separator keys, not subtree range (aligns with checker's current check_index_node_keys, measurement basis ledger in C478 (...)) | 改了：改成过去时「当时取分隔 key 的首尾，跟当时 checker 的 `check_index_node_keys` 走」，再接 C478 已还清、今天 inode 树根取子树覆盖区间（D18 已定项 2）（写回乙第 5 条） |
| `.claude/kb/milestone/02-second-txn.ja.md:565` | inode ツリー根のキー区間はセパレータキーの始端と終端を取り、部分木の被覆範囲は取らない（現在の checker の check_index_node_keys に追従、計測条件の記録は C478 (...)） | 改了：改成过去时「当时取分隔 key 的首尾，跟当时 checker 的 `check_index_node_keys` 走」，再接 C478 已还清、今天 inode 树根取子树覆盖区间（D18 已定项 2）（写回乙第 5 条） |
| `.claude/kb/milestone/02-second-txn.md:576` | inode 树根的 key 区间取分隔 key 的首尾、不取子树范围（跟 checker 今天的 check_index_node_keys 走，口径的账在 C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间）） | 改了：改成过去时「当时取分隔 key 的首尾，跟当时 checker 的 `check_index_node_keys` 走」，再接 C478 已还清、今天 inode 树根取子树覆盖区间（D18 已定项 2）（写回乙第 5 条） |
| `crates/singlefs-checker/invariants/01-块自描述.md:7` | （新增） | 不改：工作区已是新句（条文与状态列已写孩子是码 3 容器那几棵树的子树覆盖区间，写回乙现查），随提交一起进 |
| `.claude/kb/decisions/16-publish-semantics.en.md:191` | the formula check is not yet implemented, owed in C743 (...) | 不改：工作区已是新句（s4 记的是改名前的 `16-发布语义.en.md` 第 186 行），随提交一起进 |
| `.claude/kb/decisions/16-publish-semantics.en.md:207` | **Owed checks**: C88（行末 C743 那一段） | 不改：工作区已是新句（s4 记的是改名前的 `16-发布语义.en.md` 第 202 行），随提交一起进 |
| `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:191` | 公式照合はまだ実装されておらず、C743 (...) に未済として記録 | 不改：工作区已是新句（s4 记的是改名前的 `16-发布语义.ja.md` 第 186 行），随提交一起进 |
| `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:207` | **未済の検査**：C88（行末 C743 那一段） | 不改：工作区已是新句（s4 记的是改名前的 `16-发布语义.ja.md` 第 202 行），随提交一起进 |
| `.claude/kb/decisions/16-发布语义.md:191` | 公式核还没实现，欠在 C743（重建分配器记新类时不核根住不住这一槽） | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/16-发布语义.md:207` | ；C743（…）：读根环时的落点公式核 | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:141` | the formula check is not yet implemented, owed in [checks-owed.md](../checks-owed.md) C743 (...) | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.en.md` 第 140 行），随提交一起进 |
| `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:175` | C743 (...): the placement formula check when reading the root ring. | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.en.md` 第 171 行），随提交一起进 |
| `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:141` | 公式照合はまだ実装されておらず、[checks-owed.md](../checks-owed.md) C743 (...) に未済 | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.ja.md` 第 140 行），随提交一起进 |
| `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:175` | C743 (...)：ルートリング読み出し時の配置先公式照合 | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.ja.md` 第 171 行），随提交一起进 |
| `crates/singlefs-checker/invariants/02-根环.md:24` | core 那一半随根环实现批，今天没写。 | 改了：改成「core 那一半已实现」，点名 `root_ring.rs` 的 `root_sits_in_the_ring_slot_the_formula_gives` 与 recovery.rs、mount.rs 两处调用，写 C743 已还清（写回乙第 6 条） |
| `.claude/kb/checks-owed.en.md:884` | mkfs 中途崩溃的语义no clause defined | 不改：工作区已是新句（C656 已挪进已还清表），随提交一起进 |
| `.claude/kb/checks-owed.en.md:885` | Instance generation invariant red when mkfs is redone | 不改：工作区已是新句（C659 已挪进已还清表），随提交一起进 |
| `.claude/kb/checks-owed.ja.md:884` | mkfs 中途崩溃的语义条項が未定義 | 不改：工作区已是新句（C656 已挪进已还清表），随提交一起进 |
| `.claude/kb/checks-owed.ja.md:885` | fsid の先頭 8 バイトが同じまま mkfs をやり直す | 不改：工作区已是新句（C659 已挪进已还清表），随提交一起进 |
| `.claude/kb/checks-owed.md:884` | 该挂上旧池、整池拒还是别的，没有条款；条款（三方或用户定） | 不改：工作区已是新句（C656 已挪进已还清表），随提交一起进 |
| `.claude/kb/checks-owed.md:885` | 造一张坏镜像…钉住 | 不改：工作区已是新句（C659 已挪进已还清表），随提交一起进 |
| `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:201` | **Owed checks**: C79 (...) ...; C659 (...) | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.en.md` 第 197 行，C659 那一段已去掉），随提交一起进 |
| `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:201` | **未済の検査**：C79 (...) ...；C659 (...) | 不改：工作区已是新句（s4 记的是改名前的 `22-单元原子性怎么合成.ja.md` 第 197 行，C659 那一段已去掉），随提交一起进 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:201` | **欠**：C79（系统配置与 journal 环的放置没人定）……；C659（…） | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:18` | **进度：未实现。**（已定项 7） | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:19` | **进度：未实现。**（已定项 8） | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions/16-发布语义.md:16` | **进度：未实现。**（已定项 6） | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/decisions.md:63` | 已实现 15 项 / 未实现 3 项 / 无对象 8 项 | 不改：工作区已是新句（已实现 17 项 / 未实现 1 项 / 无对象 8 项），随提交一起进 |
| `.claude/kb/milestone/01-first-txn.md:161` | 取号 = 逐盘一次系统配置槽写（世代号按那块盘自证过的槽里最大的 + 1）+ 一道屏障……首次挂载路径上那道屏障就是暖机开场那道（写入口不再发第二次），录制流不变（C322（…） 当时崩溃点重放状态数 262165 不变；C577（…） 加发布末尾屏障之后新池新建文件流是 11 段、闭式 16 777 240） | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `.claude/kb/milestone/01-first-txn.md:219` | （C577（系统配置没见证到的最新根，乙罩不到） 加发布末尾屏障之后新池新建文件流是 11 段、闭式 16 777 240、全量 16 777 260） | 改了：括注补「取号写两轮之后是 12 段、43 次写、闭式 16 777 243、全量 16 777 268」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.md:70` | 2026-10-17 注：固定脚本今天做到正常卸载，78 段、481 次写、全量 1 678 377 204 个状态 | 改了：改成「2026-10-17 注、2026-10-26 改数：……80 段、485 次写、全量 1 678 377 220 个状态（取号写两轮之后）」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.md:227` | 这条流后来改成 481 次写、全量枚举 1 678 377 204 个状态（`FULL_STATES_THROUGH_THE_UNMOUNT`；每次写只取两态的闭式是 1 678 377 089）；2026-10-11 全量 f 第 2 项按新流全域重判 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.md:229` | （新池新建文件，11 段、闭式 16 777 240、全量 16 777 260；固定脚本到正常卸载，78 段、481 次写、全量 1 678 377 204 个状态；（2026-10-11 注：后来屏障改动把新池新建文件流改成 11 段、闭式 16 777 240） | 改了：两条流改成 12 段、43 次写、16 777 243 / 16 777 268 与 80 段、485 次写、1 678 377 220，2026-10-11 注后补两轮之后的 12 段、16 777 243（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.md:486` | 这条流后来改成 481 次写、全量枚举 1 678 377 204 个状态（`FULL_STATES_THROUGH_THE_UNMOUNT`；每次写只取两态的闭式是 1 678 377 089）；按新流全域重判 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/03-third-txn.md:50` | 这条流后来改成 481 次写、全量枚举 1 678 377 204 个状态（`FULL_STATES_THROUGH_THE_UNMOUNT`；每次写只取两态的闭式是 1 678 377 089）；按新流全域重判 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:410` | bash scripts/experiments/replay.sh E161 | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:411` | E161_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- feasibility | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/verification-build.md:6` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204；登记的崩溃枚举用例 14 条 | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/verification-build.md:130` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204 | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/verification-build.en.md:6` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204（英文版同一处） | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/verification-build.en.md:103` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204（英文版同一处） | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/verification-build.ja.md:6` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204（日文版同一处） | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/verification-build.ja.md:103` | 新池新建文件流全量 16 777 260、两态 16 777 240，覆盖写固定脚本流全量 1 678 377 204（日文版同一处） | 不改：工作区已是新句（16 777 268 / 16 777 243 与 1 678 377 220，写回乙现查），随提交一起进 |
| `.claude/kb/milestone/04-fourth-txn.md:76` | 这条流后来改成 481 次写、全量枚举 1 678 377 204 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 3 条） |
| `.claude/kb/milestone/04-fourth-txn.en.md:76` | 这条流后来改成 481 次写、全量枚举 1 678 377 204（英文版同一句） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 3 条） |
| `.claude/kb/milestone/04-fourth-txn.ja.md:76` | 这条流后来改成 481 次写、全量枚举 1 678 377 204（日文版同一句） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 3 条） |
| `.claude/kb/milestone/01-first-txn.en.md:134` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `.claude/kb/milestone/01-first-txn.en.md:183` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：括注补「取号写两轮之后是 12 段、43 次写、闭式 16 777 243、全量 16 777 268」（写回乙第 2 条） |
| `.claude/kb/milestone/01-first-txn.ja.md:134` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `.claude/kb/milestone/01-first-txn.ja.md:183` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：括注补「取号写两轮之后是 12 段、43 次写、闭式 16 777 243、全量 16 777 268」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.en.md:58` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：改成「2026-10-17 注、2026-10-26 改数：……80 段、485 次写、全量 1 678 377 220 个状态（取号写两轮之后）」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.en.md:212` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.en.md:214` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：两条流改成 12 段、43 次写、16 777 243 / 16 777 268 与 80 段、485 次写、1 678 377 220，2026-10-11 注后补两轮之后的 12 段、16 777 243（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.en.md:474` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204）；s5 记第 472 行 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.ja.md:58` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：改成「2026-10-17 注、2026-10-26 改数：……80 段、485 次写、全量 1 678 377 220 个状态（取号写两轮之后）」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.ja.md:212` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.ja.md:214` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：两条流改成 12 段、43 次写、16 777 243 / 16 777 268 与 80 段、485 次写、1 678 377 220，2026-10-11 注后补两轮之后的 12 段、16 777 243（写回乙第 2 条） |
| `.claude/kb/milestone/02-second-txn.ja.md:474` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204）；s5 记第 472 行 | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/03-third-txn.en.md:50` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/milestone/03-third-txn.ja.md:50` | （s5 候选外只列行号：中文要改那 7 行的译文，旧数 11 段、16 777 240 / 78 段、481 次写、1 678 377 204） | 改了：「后来改成 481 次写」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式改成 1 678 377 095，「按新流全域重判」改成「按 481 次写那一版全域重判」（写回乙第 2 条） |
| `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:55` | 登记的留存产物 `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` 是整份产物第 31 至 69 行 | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:411` | E161_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- feasibility | 不改：工作区已是新句，随提交一起进 |
| `.claude/kb/experiments/142-新池新建文件的干跑.md:22` | 补第三态后全量 16777260，全量由 … 在门禁 checker-tier-crash-point-replay 里枚举 | 不改：工作区已是新句（写回丙第 1 条现查），随提交一起进 |
| `.claude/kb/experiments/142-新池新建文件的干跑.md:310` | 今天的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260，本页装置不枚举它 | 不改：工作区已是新句（写回丙第 1 条现查），随提交一起进 |
| `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:246` | 候选甲追不追原登记第十二节由主 agent 定 | 改了：「候选甲追不追原登记第十二节由主 agent 定」改成「候选甲（往原登记第十二节追加）不追，乙已成」（写回丙第 2 条） |
| `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:578` | 手写 python 一行算 A1 的 16842782 | 改了：工作区已写了 16842785 与 16842782 两种段形，补上前 12 段得两态闭式 16777243，指到「做了什么」第 6 条（写回丙第 2 条） |
| `.claude/kb/layout/01-first-txn.en.md:407` | acquiring number, warm-up, regular publish three rows and entire stream sentence verbatim identical to artifact lines 36–39 | 改了：改成暖机、普通发布两行同产物第 37、38 行逐字相同，取号行与整条流是取号写两轮之后的形状、由 `recorded_paths_match_the_registered_segment_sequences` 钉住，末句改成 16777243 是两态闭式、三态全量 16777268 在门禁里枚举（写回丙第 3 条） |
| `.claude/kb/layout/01-first-txn.ja.md:409` | 発番、暖機、通常パブリッシュの 3 行およびストリーム全体の文は成果物第 36–39 行と逐語的に同一 | 改了：改成暖机、普通发布两行同产物第 37、38 行逐字相同，取号行与整条流是取号写两轮之后的形状、由 `recorded_paths_match_the_registered_segment_sequences` 钉住，末句改成 16777243 是两态闭式、三态全量 16777268 在门禁里枚举（写回丙第 3 条） |
| `.claude/kb/layout/01-first-txn.md:408` | 取号、暖机、普通发布三行与整条流那一句同产物第 36–39 行逐字相同 | 改了：改成暖机、普通发布两行同产物第 37、38 行逐字相同，取号行与整条流是取号写两轮之后的形状、由 `recorded_paths_match_the_registered_segment_sequences` 钉住，末句改成 16777243 是两态闭式、三态全量 16777268 在门禁里枚举（写回丙第 3 条） |
| `.claude/kb/milestone/01-first-txn.en.md:134` | Instance acquisition writes generation 2 slot 0, warm-up txg 1 / 2 writes generations 3 / 4, transaction txg 3 writes generation 5 slot 1 | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `.claude/kb/milestone/01-first-txn.ja.md:134` | 番号取得は世代 2 スロット 0、ウォームアップ txg 1 / 2 は世代 3 / 4、トランザクション txg 3 は世代 5 スロット 1 | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `.claude/kb/milestone/01-first-txn.md:161` | 取号写世代 2 槽 0、暖机 txg 1 / 2 写世代 3 / 4、事务 txg 3 写世代 5 槽 1 | 改了：世代号改成取号两轮写 2、3，暖机 4 / 5，事务 6 槽 0；验收括注改成与八那一节相同的 `2+2` 与闭式 16777243，E142 第八次跑产物另写成取号写一轮那一版；C577 括注补两轮之后 12 段、43 次写、16 777 243（写回乙第 2+4 条；取号定义句工作区已是两轮写法，没照 s7 改成「背靠背、并成一道」） |
| `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1860` | `    /// C506、C478 那一批改了 \`crash_judge_common.wgsl\`、\`crash_judge_recovery.wgsl\` 与 \`crash_judge_checker.wgsl\`，钉值没跟上：恢复 246 460 → 248 562、`（续第 1861、1862 行） | 不改：补的那一句要问后来重钉的会话，主 agent 判交下一批随 C908 那批实现员一起问、一起改 |
| `.claude/kb/decisions/16-publish-semantics.en.md:39` | changes format, implemented in the implementation batch after the system configuration landing patch merges | 改了：括注改成「已随它的实现批实现：根记录宽 466，t 在偏移 428、p 在偏移 436，池级 checker 判 I-7.18、I-7.19」，变更史 D16 节加 2026-10-26 块（写回甲第 6 条） |
| `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:39` | フォーマットを変更し、実装はシステム構成着地パッチのマージ後の実装バッチで行う | 改了：括注改成「已随它的实现批实现：根记录宽 466，t 在偏移 428、p 在偏移 436，池级 checker 判 I-7.18、I-7.19」，变更史 D16 节加 2026-10-26 块（写回甲第 6 条） |
| `.claude/kb/experiments/184-cost-counts-for-the-regular-and-large-workload-tiers.en.md:158` | whether to append it is for the main agent to decide. | 改了：「追不追由主 agent 定」改成「主 agent 定不追（乙已成）」（写回丙第 2 条） |
| `.claude/kb/experiments/184-通常層と大規模負荷層のコスト計数.ja.md:158` | 追記するかどうかはメイン agent の判断。 | 改了：「追不追由主 agent 定」改成「主 agent 定不追（乙已成）」（写回丙第 2 条） |
| `.claude/kb/experiments/184-cost-counts-for-the-regular-and-large-workload-tiers.en.md:246` | whether candidate A is appended to section 12 of the original registration is for the main agent to decide | 改了：「候选甲追不追原登记第十二节由主 agent 定」改成「候选甲（往原登记第十二节追加）不追，乙已成」（写回丙第 2 条） |
| `.claude/kb/experiments/184-通常層と大規模負荷層のコスト計数.ja.md:246` | 候補甲を元の登録の第12節に追記するかはメイン agent の判断 | 改了：「候选甲追不追原登记第十二节由主 agent 定」改成「候选甲（往原登记第十二节追加）不追，乙已成」（写回丙第 2 条） |
| `.claude/gate.d/replay-comparison-units.tsv:18` | # E165 r5 #15 到 #19 这五条「不登记的比对对象」就是 .claude/kb/checks-owed.md 的 C879（E165 根选择与杀伤界正式跑没有单元）：那几条正式跑在 formal_run_units 里补了单元，就删掉这五行、改登记成正式行。 | 改了：「这五条…五行」改成「r5 #15 到 #19 与 r7 #1 到 #3 这八条…八行」（主 agent 用 replace-once 改，调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 1783 行；不在四份写回报告里） |
| `.claude/kb/experiments/184-常规层与大负载层的代价计数.md:158` | 同时备好了甲要追加进原登记第十二节的文字，追不追由主 agent 定。 | 改了：「追不追由主 agent 定」改成「主 agent 定不追（乙已成）」（写回丙第 2 条） |
| `.claude/kb/decisions/16-发布语义.md:39` | 与英日第 39 行同一处，s2 已判（工作区仍是旧句）：动格式，实现在系统配置落地补丁合入之后的实现批 | 改了：括注改成「已随它的实现批实现：根记录宽 466，t 在偏移 428、p 在偏移 436，池级 checker 判 I-7.18、I-7.19」，变更史 D16 节加 2026-10-26 块（写回甲第 6 条） |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表（F1–F49） | research/prompts/m3-close-b17-facts.tsv | 50 |
| 候选表（27 组 1350 行） | research/prompts/m3-close-b17-candidates.tsv | 1351 |
| 逐行判定报告 s1（F1–F5） | research/prompts/m3-close-b17-judge-s1.md | 275 |
| 逐行判定报告 s2（F6–F8） | research/prompts/m3-close-b17-judge-s2.md | 243 |
| 逐行判定报告 s3（F9–F11） | research/prompts/m3-close-b17-judge-s3.md | 253 |
| 逐行判定报告 s4（F12–F14） | research/prompts/m3-close-b17-judge-s4.md | 336 |
| 逐行判定报告 s5（F15–F16 与候选外） | research/prompts/m3-close-b17-judge-s5.md | 229 |
| 逐行判定报告 s6（F17–F18） | research/prompts/m3-close-b17-judge-s6.md | 213 |
| 逐行判定报告 s7（F19、F21–F23） | research/prompts/m3-close-b17-judge-s7.md | 215 |
| 逐行判定报告 s8（F24–F27、F49 与对象外 27 份） | research/prompts/m3-close-b17-judge-s8.md | 224 |
| 写回甲报告（欠账表三语与 D16 三语） | research/prompts/m3-close-b17-writeback-a.md | 82 |
| 写回乙报告（里程碑、验证落地与不变量清单） | research/prompts/m3-close-b17-writeback-b.md | 95 |
| 写回丙报告（实验页与布局表） | research/prompts/m3-close-b17-writeback-c.md | 84 |
| 收尾小批报告（layout/03、E172、layout/01、里程碑三） | research/prompts/m3-close-b17-writeback-d.md | 89 |
