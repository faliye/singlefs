<!-- knowledge-sync -->
# 验证两档拆分 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/gate-triage.md、.claude/agents/implementation-writer.md、.claude/gate.d/54-layer0-replay.sh、.claude/gate.d/94-checker-implementation-disjoint.sh、.claude/gate.d/stage-inputs.tsv、.claude/hooks/heavy-test-guard.sh、.claude/hooks/lib_heavy_tests.py、.claude/rules/implementation-workflow.md、.claude/rules/verification.md、CLAUDE.md、crates/singlefs-harness/src/bin/e158_root_choice_repair.rs、crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs、research/scripts/admission.py、research/scripts/crash-case-check.py、.claude/gate.d/13-vague-names.sh、.claude/gate.d/55-qemu-first-transaction.sh、.claude/gate.d/80-absolute-assertions.sh、.claude/gate.d/stage-owners.tsv、crates/singlefs-checker-tier/src/bad_disk_input.rs、crates/singlefs-checker-tier/src/bin/e142_first_transaction_write_dump_one_device.rs、crates/singlefs-checker-tier/src/bin/e142_first_transaction_write_dump.rs、crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs、crates/singlefs-checker-tier/src/bin/first_transaction_device_log_check.rs、crates/singlefs-checker-tier/src/bin/first_transaction_region_bytes.rs、crates/singlefs-checker-tier/src/crash_injection.rs、crates/singlefs-checker-tier/src/crash.rs、crates/singlefs-checker-tier/src/device_log.rs、crates/singlefs-checker-tier/src/layer0_progress.rs、crates/singlefs-checker-tier/src/lib.rs、crates/singlefs-checker-tier/src/on_device_modes.rs、crates/singlefs-core/src/admission.rs、crates/singlefs-core/src/system_configuration.rs、crates/singlefs-core/src/transaction.rs、crates/singlefs-format/src/lib.rs、crates/singlefs-harness/src/first_transaction_regions.rs、crates/singlefs-harness/src/lib.rs、crates/singlefs-harness/src/memory_pool.rs、research/scripts/e152-run.sh、research/scripts/e152-stage-root.sh、research/scripts/layer0-shard-run-selftest.sh、research/scripts/layer0-shard-run.sh、research/scripts/replay.sh、.claude/agents/experiment-designer.md、.claude/agents/experiment-runner.md、.claude/agents/three-way-attack.md、.claude/gate.d/86-experiment-orphans.sh、.claude/gate.d/96-experiment-source-discipline.sh、.claude/hooks/agent-write-scope.tsv、.claude/hooks/bash-command-detector.sh、.claude/hooks/write-guard.sh、.claude/rules/implementation-first.md、.claude/rules/path-moves.md、crates/singlefs-checker/src/walk.rs、crates/singlefs-checker-tier/src/bin/first_transaction_on_device.rs、crates/singlefs-harness/src/fault_injection.rs、crates/singlefs-harness/src/history.rs、research/scripts/agent-watch.py、research/scripts/compile-then-swap.py、research/scripts/rerun-failed-tests.py

做成的事（2026-09-27，会话 singlefs-8b，主 agent 自己做、不派人）：`crates/` 下验证代码分两档三个包（D13（验证路线） 已定项 15）：新建 checker 档包 `crates/singlefs-checker-tier`，崩溃态枚举引擎、记录核对器、断点续跑、崩溃注入与坏盘输入、设备日志比对、8 个装置二进制与 21 个测试文件从 harness 搬进去；harness 只留脚手架库与日常用例，两边都要的辅助函数各留一份；池级 checker 仍只依赖 `singlefs-format`；重型测试闸只按包判，按测试名与 `--ignored` 认的分支连同自检用例删掉；54 号快档改跑 `-p singlefs-checker-tier --lib --tests`、标记不作数报「本次未跑」；94 号加依赖方向那一条；新立 13 号禁含糊名字；上游 SOP 解绑（未发版）；各闸与门禁的自检样本改成新布局（`path-moves.md` 第 7 步）。

## 搜索

回扫按旧说法搜（除 research/prompts、research/results、target、.git，它们是冻结证据或产物）：

- `grep -rn "singlefs-harness/tests/<搬走的 11 个文件名>" .` → 搬之前 30 处（kb 20、records 3、bins 4、变异表 6、admission 0），改成 `singlefs-checker-tier/tests/` 之后剩 0
- `grep -rn "\-p singlefs-harness --test <搬走的目标>" crates/mutations.tsv` → 改成 `-p singlefs-checker-tier`，`grep -c -- '-p singlefs-checker-tier --test' crates/mutations.tsv` → 171、`-p singlefs-checker --test` → 0；第 2 列指到搬走文件的行改路径；33 号绿
- `grep -rn "重型测试只在提交时跑" .` → 10 处：本仓 `.claude/agents/mutation-triage.md:30`、`.claude/skills/crash-test/SKILL.md:12` 改成新小节名「checker 档只在提交时跑，harness 随时跑」（10 号从红转绿）；其余在 records、变更史、上游副本的说明句，说的是那一天的事，留
- `grep -rn "层 0 全量" .` → 116 处：现状句改了 `README.md:122-126`（跑法）、`.claude/kb/tooling.md:648`（54 号那一行）；`.claude/main-agent.md:58`、54 号第 4 与 194 行、`stage-inputs.tsv` 是改后的现值；records、变更史、`verification-build.md:138`、`invariants.md` 里的是事件句或说全量本身，留
- `grep -rln "名字含 layer0" .`（除冻结目录与 records）→ 改完 0 个文件：`lib_heavy_tests.py`、`heavy-test-guard.sh`、`agent-watch.py` 按包判之后，按名字判的那一支连同自检用例删掉（用户「面向测试的测试和用例不应该改为符合现状的吗」）；records 里的是那一天的事，留
- 搬进 checker 档包的 35 个文件（`crates/singlefs-checker-tier/` 下除 `src/lib.rs` 外的 src、src/bin、tests）逐个按旧路径 `crates/singlefs-harness/<同一相对路径>` 全仓搜（除 research/prompts、research/results、target、.git）→ 改之前 19 处（变更史 5、实验变更史 4、records 10），全改；复搜 0
- `grep -rn "singlefs-checker/tests" .`（两包那一版的落点）→ 3 个文件：变更史与两份 records，都是记那一版的事件句或历史节，留
- `grep -rn "集成测试不算共享\|dev-dependencies 不算共享" .claude/kb` → 0：已定项 5 那句随三个包撤掉
- `grep -rn "只有在提交时候才跑" .` → 3 处：记录与 54 号文件头引用户原话，留
- 崩溃枚举用例住哪：`python3 research/scripts/crash-case-check.py .` 判 11 条都在 checker 档包、标 ignore 的都登记；harness 里剩 `crash_enumeration_resumes_from_its_progress_file.rs:383` 一条小流测试，加了「不是崩溃枚举用例」的注明

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/agents/mutation-triage.md:30 | `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」 | 改了：小节名改成「checker 档只在提交时跑，harness 随时跑」 |
| .claude/skills/crash-test/SKILL.md:12 | 谁在什么时候跑、带什么前缀见 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」 | 改了：小节名改成「checker 档只在提交时跑，harness 随时跑」 |
| README.md:124 | `cargo test --workspace  # 平时的单测；登记的崩溃枚举用例（层 0 全量等）都标 ignored，这里只跑快档与缩小版` | 改了：平时跑 `cargo test -p singlefs-harness`；checker 档快档带 `SINGLEFS_HEAVY_TESTS=user-request` 跑 `cargo test --release -p singlefs-checker-tier` |
| README.md:126 | 单跑层 0 快档（两条流里不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记 | 改了：54 号快档跑 checker 档包不标 ignored 的用例，标记不作数报「本次未跑」 |
| .claude/kb/tooling.md:648 | 用例标 `#[ignore]`，平时 `cargo test --workspace` 报 1 ignored 是正常的 | 改了：用例住 `crates/singlefs-checker-tier/tests/`，`cargo test -p singlefs-checker-tier` 报几条 ignored 正常，harness 档跑不到它们 |
| .claude/kb/milestone/02-second-txn.md:168 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/milestone/02-second-txn.md:421 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/layout/01-first-txn.md:405 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/layout/01-first-txn.md:406 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/layout/01-first-txn.md:407 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/layout/01-first-txn.md:408 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/layout/01-first-txn.md:412 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:78 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:302 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:516 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:574 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:575 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/checks-owed.md:755 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/verification-build.md:154 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments.md:165 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:1 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:183 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:187 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:281 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:286 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:287 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/experiments/142-第一个事务的干跑.md:340 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/decisions-history/2026-09.md:25 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| .claude/kb/decisions-history/2026-09.md:9274 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| records/2026-09-24-里程碑二收尾调度.md:162 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| records/2026-09-24-里程碑二收尾调度.md:170 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| records/2026-09-24-里程碑二收尾调度.md:291 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| records/2026-09-24-里程碑二收尾调度.md:295 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:5053 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:17503 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:17747 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:12 | `crates/singlefs-harness/tests/<搬走的测试文件>` | 改了：路径改到 `crates/singlefs-checker-tier/tests/`（path-moves：路径不在冻结范围内；e158 第 17747 行是字串常量，装置只拿它当说明打、不按它读文件） |
| crates/mutations.tsv:798 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |
| crates/mutations.tsv:909 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |
| crates/mutations.tsv:910 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |
| crates/mutations.tsv:911 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |
| crates/mutations.tsv:1079 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |
| crates/mutations.tsv:1080 | 第 2 列 `crates/singlefs-harness/tests/<搬走的测试文件>.rs` | 改了：变异要改坏的文件改到 `crates/singlefs-checker-tier/tests/`，33 号锚点各命中一次 |

另有三批命中不进表：搬进 checker 档包的文件在变更史、实验变更史与 records 里的 19 处旧路径（改了，逐处见「搜索」那一条）；`crates/mutations.tsv` 第 5 列 65 行 `-p singlefs-harness --test <搬走的目标>` 改成 `-p singlefs-checker-tier`（改了，一行一条列不完，`grep -c -- '-p singlefs-checker-tier --test' crates/mutations.tsv` → 171）；`research/prompts` 130 个文件 404 处、`research/results` 8 个文件 86 处旧路径不改：冻结证据与产物，改产物 87 号复跑对不上（path-moves 第 6 步），这一轮记在 `records/2026-09-27-验证两档拆分.md`。

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 54 号快档真跑（旧钉值下三条 FAILED） | research/prompts/verification-split-tmp-evidence/gate54-quick-run.log | 44 |
| admission.py 自证（54 号那几格按新语义） | research/prompts/verification-split-tmp-evidence/admission-selftest.log | 282 |
| 上游 zh 仓 selftest.sh 全绿 | research/prompts/verification-split-tmp-evidence/upstream-selftest.log | 47 |
| crash-case-check 真仓判定 | research/prompts/verification-split-tmp-evidence/crash-case-check.log | 1 |
| lib_heavy_tests 自证 | research/prompts/verification-split-tmp-evidence/lib-heavy-tests-selftest.log | 1 |
| heavy-test-guard 自证 | research/prompts/verification-split-tmp-evidence/heavy-test-guard-selftest.log | 2 |
| 那 14 条只重跑：暂存树上 | research/prompts/verification-split-tmp-evidence/rerun-failed-on-staged-tree.log | 236 |
| 那 14 条只重跑：纯 HEAD 上（同样 14 条红，拆包之前就红） | research/prompts/verification-split-tmp-evidence/rerun-failed-on-head.log | 247 |
| harness 档拆包之后全量跑（no-fail-fast，失败 14 条，HEAD 上同样红） | research/prompts/verification-split-tmp-evidence/harness-daily-after-split.log | 3415 |
| 改完之后旧路径全仓搜索（除冻结目录） | research/prompts/verification-split-tmp-evidence/old-path-search-after.txt | 1 |
| 旧说法全仓搜索（除冻结目录） | research/prompts/verification-split-tmp-evidence/old-wording-search.txt | 145 |
