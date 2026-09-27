# 验证两档拆分：代码轮与定义轮合一，第一轮正文（2026-09-27）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 -->

## 一、这一轮要判什么

用户 2026-09-27 定案：`crates/` 下的验证代码分两档——**harness**（`crates/singlefs-harness`，单元与集成测试，改了代码随时跑）与 **checker 档**（`crates/singlefs-checker` 包的 `tests/`，崩溃枚举用例与注入战役，默认只在提交时跑快档、全量由用户要求或夜间）；上游 SOP 不许绑这个项目的验证手段。定案已写进 D13（验证路线） 已定项 15，落地在 `.claude/rules/verification.md`。**这一轮不判要不要分两档（用户已定），只判落地对不对**：代码搬得对不对、闸判得对不对、门禁与定义说的和做的是不是一回事、上游解耦漏没漏。经过与用户原话在 `records/2026-09-27-验证两档拆分.md`。

主 agent 自己做的全部改动（用户定不派实现员），diff 在 `research/prompts/_verification-split-r1-diff.md`（`git diff HEAD -M`，只含这一轮的路径，39 个文件、+524 / −226）。

## 二、实现今天的样子（主 agent 的观测，开工快照 `refs/sop/verification-split-r1-snapshot`，时刻在 `research/prompts/verification-split-r1-snapshot/dispatched-at-utc.txt`）

腿读代码一律读那棵快照树（`git show refs/sop/verification-split-r1-snapshot:<路径>`），它是工作区整份，含别的会话未提交的改动；被判文件的 sha256 在 `research/prompts/verification-split-r1-snapshot/judged-files-sha256.txt`。

- `crates/singlefs-checker/Cargo.toml`：`[dependencies]` 仍只有 `singlefs-format`；新加 `[dev-dependencies]` `singlefs-core`、`singlefs-harness`。库源码 `crates/singlefs-checker/src/{lib,image,walk,position_addressed}.rs` 一行没动。
- `crates/singlefs-checker/tests/`：11 个文件由 `git mv` 从 `crates/singlefs-harness/tests/` 搬来，内容只改 `mod common;`、`mod common_tree_split;`、`mod common_admission;` 三种声明，前面各加一行 `#[path = "../../singlefs-harness/tests/<模块>/mod.rs"]`：`first_transaction_step_seven_layer0`、`second_transaction_step_zero_layer0`、`second_transaction_parallel_line_one_layer0`、`second_transaction_supplement_two_tree_split_layer0`、`second_transaction_position_addressed_trees_layer0`、`record_checker_judges_absence_by_the_persisted_set`、`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`、`second_transaction_supplement_three_crash_injection`、`second_transaction_parallel_line_three_spill_over_layer0`、`second_transaction_step_three_acquisition_barrier_layer0`、`second_transaction_step_three_formatted_pool_layer0`。`crates/singlefs-harness/tests/common/mod.rs`、`common_tree_split/mod.rs`、`common_admission/mod.rs` 里没有 `env!`、`CARGO_MANIFEST_DIR`、`CARGO_BIN_EXE_`、`include_str!`（`grep` 零命中）。
- `crates/singlefs-harness/tests/` 剩 95 个文件，名字含 `layer0` 的 0 个；`crash_enumeration_resumes_from_its_progress_file.rs:383` 上面加了 `// crash-case-check:not-a-crash-case …`。`crates/singlefs-harness/src/crash.rs`、`segments.rs`、`layer0_progress.rs` 等引擎一行没动。
- `crates/mutations.tsv`：65 行第 5 列 `-p singlefs-harness --test <搬走的目标>` 改成 `-p singlefs-checker`，6 行第 2 列路径改到 `crates/singlefs-checker/tests/`；门禁 33 号 1354 条锚点各命中一次。
- `.claude/hooks/lib_heavy_tests.py` `cargo_use`：清单与范围提前算；次序是 `--workspace / --all` → 工作区根裸跑 → 范围等于全部成员 → **checker 包判**（`members.get("singlefs-checker")` 在范围里，且不挑目标、或 `--tests` / `--all-targets` / `--test`）→ 旧的名字含 `layer0` 判 → 登记的崩溃枚举用例带 `--ignored` 判；`classify` 里直接执行的测试二进制先查 `checker_test_targets`（只认离命令最近的仓，仓外才退回本 hook 所在仓）。`KIND_CATEGORY` 加 `checker-tier-cargo`、`checker-tier-binary` → 「checker 档」。自证 174 种全绿。
- `.claude/hooks/heavy-test-guard.sh`：`AGENT_KINDS["crash-verifier"]` 加那两个 kind；`POLICY` 句改；自证 699 种全绿。主 agent 不带前缀单跑 `cargo test --release -p singlefs-checker --test first_transaction_step_seven_layer0` 实测被拒。
- `.claude/gate.d/54-layer0-replay.sh`：`run_layer0_test_binary`（两条流各一次 `cargo test --release -p singlefs-harness --test <流>`）换成 `run_checker_package_tests`（一次 `cargo test --release -p singlefs-checker -- --nocapture`）；`run_checker_quick_tier` 把每个测试二进制那一行 `test result` 的 passed / ignored 加总，`passed_total == 0` 判红；标记不作数的用例逐条往 `GATE_NOT_RUN_FILE` 写一行、打 `!`、`exit 0`（原来 `✗` 与 `exit 1`）。`--full` 那一路没动。`research/scripts/admission.py` 自证 280 格全绿（54 号那几格的期望改成报「本次未跑」）。
- `.claude/gate.d/94-checker-implementation-disjoint.sh`：`DEPENDENCY_TABLE`、`DEPENDENCY_ITEM_TABLE` 两个正则去掉 `dev-`；绿样本的 checker `Cargo.toml` 加 `[dev-dependencies] singlefs-core` 仍判绿。
- `research/scripts/crash-case-check.py`：判法改成「直接调 `enumerate_layer0` 一族（非 `quick_tier`）的测试函数要住在 `singlefs-checker` 包；那个包里标 `#[ignore]` 的要登记成 `crash-case:`，不标的是快档不用登记」；真仓 11 条全过；5 个弄坏开关各自判红。
- `.claude/gate.d/stage-inputs.tsv`：8 行 `crash-case:`，第三列都是 `test=singlefs-checker:…`；新登记 `crash-case:layer0-position-addressed-tree-streams`（没有 `count-line=`：它每条流各打一行同前缀的 `LAYER0_POSITION_ADDRESSED`）。
- 实测：`cargo build --all-targets --offline` 通过（21 s 增量）。`SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh .` **快档判红**：`first_transaction_step_seven_layer0` 里 `layer0_quick_tier_matches_the_full_tally_shape`、`positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle`、`removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker` 三条 FAILED（`research/prompts/verification-split-tmp-evidence/gate54-quick-run.log`）。主 agent 的归因：C577（系统配置没见证到的最新根，乙罩不到） 在发布末尾多加了一道屏障（`crates/singlefs-core/src/transaction.rs:553`、`:1188`），第一条流的钉值（`QUICK_TIER_STATES = 54`、`FULL_STATES_WITH_TWO_STATES_PER_WRITE = 67_108_885`）还是旧的；会话 singlefs-99 报告它下一步「合入后验证」要改这些钉值。**这一归因是推的，没在 HEAD 上单独复现**——腿要核。
- 规则与定义：`.claude/rules/verification.md` 新立（开头「定义」一节两句话，之后两档表、harness 轻重、checker 档快档与全量、用例住哪、从上游收回的四节、门禁管哪一半）；`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」整节改成「checker 档只在提交时跑，harness 随时跑」；`CLAUDE.md` 规则清单加一行；`.claude/main-agent.md` 4 处、`.claude/agent-common.md` 2 处、`.claude/agents/implementation-writer.md` 3 处、`crash-verifier.md` 2 处、`gate-triage.md` 1 处、`mutation-triage.md` 1 处。两个包的 `Cargo.toml` `description` 各写一句定义。
- kb：D13（验证路线） 新立已定项 15；已定项 9 定案表层 0 那一行的触发改成「checker 档的全量：用户要求或夜间跑；提交时默认只跑快档」、射程改成现值；已定项 5 射程末尾加「集成测试不算共享」一句。变更史 2026-09-27（其十九）。
- 上游三语仓（`../singlefs-ai-sop-{zh,en,ja}`，未 bump、未提交）：`rules/test-discipline.md` 删「崩溃一致性只能靠崩溃点重放验证」「功能正确性靠模型对拍」「checker 即规范」三节、「门禁管哪一半」改通用；`rules/evidence-discipline.md` 删末节「文件系统特有的反推缺口」；`scripts/gate.sh` 未实现清单只留「最终判据」「命名纪律（shell）」两个共享键、项目键从项目根 `.claude/gate-not-implemented.tsv` 读；`scripts/selftest.sh` 600 例全绿；`skills/gate/SKILL.md` 两段改通用。本仓 `.claude/gate-not-implemented.tsv` 登记「崩溃点重放」「模型对拍」。没动：`skills/crash-test/SKILL.md`、`GLOSSARY.md:47-48`、`scripts/env.sh:35`。

## 三、判据表

每格给一个能核的观测；「判绿的观测」出现就绿，「判红的观测」出现就红，两样都没出现写「没量到」。

| # | 问题 | 判绿的观测 | 判红的观测 |
|---|---|---|---|
| Q1 | 闸按包判有没有漏：有没有一条命令会跑到 `singlefs-checker` 包 `tests/` 下的用例，而 `lib_heavy_tests.classify` 返回 `None` 或返回的 kind 不在 `KIND_CATEGORY` 的「checker 档」里 | 构造 ≥ 10 种写法（`-p`、`--package=`、cwd 在包里、`--manifest-path`、`--test <名>`、`--tests`、`--all-targets`、通配 `--test '*'`、nextest、别名、`cargo test -p singlefs-core -p singlefs-checker`、直接执行二进制、经 `bash -c` / `env` / `nice`）逐条跑 `classify`，都判 checker 档 | 任一条返回 `None` 或别的类，给出命令与返回 |
| Q2 | `#[path = "../../singlefs-harness/tests/<模块>/mod.rs"]` 把 harness 的共用模块编进 checker 的测试二进制：语义有没有变（`cfg`、`env!`、相对路径、`CARGO_*`、`crate::` 引用） | 三个 `mod.rs` 里没有读环境或路径的地方；`common_tree_split/mod.rs:27` 的 `use crate::common::…` 在两个二进制里都解析到同一份 `common` | 指出行号与两个二进制里不同的行为 |
| Q3 | 54 号快档「标记不作数报本次未跑、不判红」：`gate.sh` 收到 `GATE_NOT_RUN_FILE` 那几行之后，`# gate-covers: 崩溃点重放` 这一轮算不算覆盖；单跑（`GATE_NOT_RUN_FILE` 不在）时还有没有别的东西告诉人全量没跑 | `gate.sh` 的 `drain_not_run_side_channel` 把 `STAGE_REPORTED_NOT_RUN` 置 1、`gate-covers` 不算覆盖（引行号）；单跑时那几行 `!` 与 `→` 在输出里 | 有一条路让 54 号退 0 而汇总里既没有「本次未跑」也没有「未实现」 |
| Q4 | 94 号不读 `[dev-dependencies]`：dev-dependency 能不能让 checker **库**代码链到 `singlefs_core`——库里 `#[cfg(test)]` 的单测、doctest、`build.rs`；D13（验证路线） 已定项 5 的射程句有没有把这一格说清 | 说清「库的 `cfg(test)` 单测能 `use singlefs_core`，94 号 ② 扫源码那一条会红」或指出它不会红 | 94 号 ② 对 `#[cfg(test)]` 块里的 `singlefs_core` 引用不判而射程句没写 |
| Q5 | 定义自相矛盾：`.claude/main-agent.md` 禁止第 1 条（挂钟长不是收窄它的理由）与 verification.md「全量默认不在提交时跑」；`crash-verifier.md` 第 1、1b、1c、2、4 步与 54 号新语义；`gate-triage.md` 第 2、3 步；`implementation-writer.md` 第 4、7a 步；`agent-common.md`「不做」一节 | 逐句对下来没有一句要求跑全量、或按名字含 `layer0` 判的 | 引出矛盾的两句原文 |
| Q6 | `crash-case-check.py` 新判法漏洞：harness 里经共用函数间接调 `enumerate_layer0` 的测试、宏生成的测试、checker 包里不标 `#[ignore]` 的大流；`not-a-crash-case` 注明能不能滥用 | 射程句写明认不出的形态；真仓里经共用函数间接调的有几处（grep `enumerate_layer0` 在 `crates/singlefs-harness/tests/common*/`）| 有一处间接调用没被任何检查罩住而没写进射程 |
| Q7 | D13（验证路线） 已定项 15 与已定项 4、5、7、9 之间有没有互相矛盾的句子；已定项 9 定案表层 0 那一行改了触发之后，「层 0 冒烟 = 写请求数两位数」这一格还成不成立 | 逐句对下来不矛盾，或指出哪一格要改 | 引出矛盾的两句原文 |
| Q8 | 上游解耦漏没漏：三语仓里还剩哪些绑本项目的内容（`skills/crash-test/SKILL.md`、`GLOSSARY.md`、`scripts/env.sh`、`templates/`、`README.md`、`CLAUDE.md`）；`gate.sh` 读项目登记表那段有没有会误判的输入（制表符、空第三列、CRLF、重复键） | 列出剩余清单与各自该不该改；登记表那段的边角逐个试 | 有一处绑着本项目而正文里没列 |
| Q9 | 搬迁遗漏：仓里（除 `research/prompts`、`research/results`）还有没有指旧路径、旧包名的地方——`research/scripts/replay.sh`、55 号、`vm-harness.md`、`Cargo.toml`、`.claude/gate.d/*.sh`、`research/scripts/*.sh|py` | 对 11 个文件名各 grep 一遍，命中只剩 `lib_heavy_tests.py:916` 那处样本 | 列出漏改的行 |

## 四、分工表

| 腿 | 立场 | 攻击面 / 任务 | 为什么这么分 |
|---|---|---|---|
| 云端攻方（Opus，`three-way-attack`） | 攻 | Q1、Q2、Q4、Q6、Q9：代码与闸的漏洞，要读源码、构造命令、跑 `classify` | 都要在仓里动手试，云端腿能读快照树、跑 python |
| 云端正推（Sonnet，`three-way-forward`） | 正推 | Q3、Q5、Q7：条款 ↔ 门禁 ↔ 定义逐句对，核「做的是不是说的」 | 是文本对文本的核对，正推腿的活 |
| 本地攻方（`three-way-local-attack`） | 攻 | Q8，另填一张表：20 条 cargo / 二进制 / 脚本命令逐格填「重型 / 不重型」与该属哪一档，与 `classify` 的真值对拍（真值由派腿的一方先跑 `lib_heavy_tests.py` 得出、写死在提示的事实表里） | 能落成表、逐格判；攻击面是上游残留与命令表，与云端攻方（仓内代码与闸）不重叠 |

两条攻方腿的攻击面不重叠：云端攻方管仓内代码、闸与门禁脚本；本地攻方管上游三语仓的残留与命令分类表。

## 五、引用清单

- D13（验证路线） 已定项 5、7、9、15（`.claude/kb/decisions/13-验证路线.md`）。
- `.claude/rules/verification.md` 全文；`.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」一节。
- `records/2026-09-27-验证两档拆分.md`（用户原话与去向）。
- diff：`research/prompts/_verification-split-r1-diff.md`。
