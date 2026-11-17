<!-- knowledge-sync -->
# m4-gate-alignment 阶段同步

阶段：全量提交门禁对齐与全绿验证阶段同步，基准 HEAD 到工作区。

触发文件：
- .claude/agent-common.md
- .claude/gate.d/break-switch-replay-files.tsv
- .claude/gate.d/code-source-discipline.sh
- .claude/gate.d/code-tooling.sh
- .claude/gate.d/comment-history-lag.tsv
- .claude/gate.d/crash-point-replay.sh
- .claude/gate.d/doc-process-records.sh
- .claude/gate.d/quoted-product-unreplayable.tsv
- .claude/gate.d/refused-tests.tsv
- .claude/gate.d/stage-owners.tsv
- .claude/hooks/lib_project_code_runs.py
- .claude/rules/implementation-workflow.md
- crates/singlefs-core/src/instance_table_sweep.rs
- crates/singlefs-core/src/mount.rs
- crates/singlefs-core/src/recovery.rs
- crates/singlefs-verification/src/bin/e158_root_choice_repair.rs
- crates/singlefs-verification/src/bin/e182_unreadable_abandoned_root_forks.rs
- crates/singlefs-verification/src/bin/e205_c971_inverted_first_txg_arm_costs.rs
- crates/singlefs-verification/src/crash_injection.rs
- crates/singlefs-verification/src/crash_library.rs
- crates/singlefs-verification/src/fault_injection.rs
- crates/singlefs-verification/src/history.rs
- crates/singlefs-verification/src/model.rs
- crates/singlefs-verification/src/model_comparison.rs
- crates/singlefs-verification/src/mount_transparency.rs
- crates/singlefs-verification/src/reference_filesystem_translation.rs
- scripts/experiment-apparatus/e203-admission-scan-cost-run.sh
- scripts/experiment-apparatus/replay.sh
- scripts/gate/admission.py
- scripts/gate/apply-writer-patch.py
- scripts/gate/merge-and-verify.sh
- scripts/gate/refused-test-registry.py

## 搜索
回扫命令与命中计数：
- git diff --name-only 8ed2ea4845ef0023fcf9a41e3ac09fece4423c0d | wc -l → 384
- grep -rn 'knowledge-sync' .claude/gate.d/doc-process-records.sh | wc -l → 18

## 命中处置
| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/agent-common.md:1 | <!-- agent-common --> | 改了：对齐提交门禁自测与检查单约束 |
| .claude/gate.d/quoted-product-unreplayable.tsv:13 | e205-c971-inverted-first-txg-arm-costs-torn-rotation.out | 补了：登记进不可复跑表供正文引用核验 |
| .claude/gate.d/code-source-discipline.sh:1 | #!/usr/bin/env bash | 不改：已核验源码纪律规约与剪枝标记一致 |

## 原始证据
| 材料 | 路径 | 行数 |
|---|---|---|
| 撂下轮次登记表 | research/prompts/abandoned-rounds.tsv | 12 |
| B20代码轮判决 | research/prompts/m3-close-b20-code-r1-main-verification.md | 55 |
| E203岔路轮判决 | research/prompts/e203-forks-r2-main-verification.md | 40 |
| E205跑前登记 | research/prompts/e205-preregistration.md | 697 |
