# 第 24 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 F3 B1 H2 -->

这组崩溃注入改法（第二、三截的接线、发布臂与 F3）经第 18、20、21、23 批四次代码轮，这一轮是最后一轮（`.claude/rules/three-way-inference.md`「第三轮之后停」）：只攻这一批的改动站不站得住，这一轮里新冒出的零轮形态写进交用户表或记账，不再开轮。

## 一、题面

第 24 批（`records/2026-10-01-里程碑三形式化接入.md#24`）合入了 24-1 一份补丁，另加主 agent 合入后改的一句注释，改动全文 `research/prompts/_m3-formal-code-b24-r1-diff.md`。它照判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第三节第 2 条改：

- F3-d：`crates/singlefs-checker-tier/src/crash_injection.rs` 的 tests 模块，F3-c 之后加一份共用搭建与三条用例：A 段号 0..=63 共 262080 个种子跨段号两两不同；B 长 28 的段、前 16 个写都落了、末 12 个写 4095 种，种子两两不同；C 历史种子 0 与 1 共 8190 个两两不同。6 行变异：前 12 个写、前 16 个写点名 B；按位拼接、去段号、或代异或点名 A；去历史种子点名 C。
- B1-c：`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 前提块之后加「同一份盘面再挂一次，真发一次建 0 个 inode 的空发布，断言它做成」；变异「`UNIT_AREA_SLOTS` 136 → 133」。主 agent 合入后改了同文件第 40 行一句注释（128 槽上建 0 个是被落点拒，不是被空间准入拒）。
- 注释：`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 模块文档；`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` H2 常数注释（S 取 5–15、「3k − 1 从 k ≥ 2 起成立」）；`crates/singlefs-core/src/root_record.rs` 卸载记号那句挪到枚举文档。

条款：判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md`、`m3-formal-code-b23-r1-main-verification.md`；`.claude/kb/decisions/08-核心索引结构.md`「#### 已定项 16」第 4、5 条。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证（日志 `/tmp/claude-1000/merge-and-verify/2026-10-04-2058068/`）：harness 档 150 个测试二进制全绿、fmt 过；`clippy --all-targets --all-features` 红 4 处，全在这一批之外：E171 第 1 次重跑执行员正在写、没进 git 的 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 与经它 `#[path]` 编进去的 RefFS 源码，归那件返工。
- checker 档用例与 7 行变异只编过、没跑；24-1 实现员在 harness 档写探针量：六种改坏各红在点名的那条，数 B = 1、1，A = 4096、4095、13254，C = 4095，与攻方 `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 逐个同；F3-c 原用例在这六种下全绿；B1 新前提在 136、134、138 槽为真，133、128 槽为假（空发布报 `PlacementRefused`）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　F3-d
三条用例的取样是不是覆盖了真历史会走到的段号与段长；六行变异是不是各自只让点名的那条红；还有没有第七种有害改坏三条都放过。

### K2　B1-c 与注释
新前提是不是真钉住了「那一支建 0 个」那行变异的判别力（盘面往别的参数漂时，前提与变异同真同假）；改过的四处注释与代码逐句对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K2 | 逐句核代码做的是不是条款与判决说的，每格给判定与引文 |
| 云端攻方（Opus） | K1、K2 | 攻哪一格会错；量得了的在副本上写小模型或 harness 用例量，checker 档不跑、写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 这是最后一轮，改法若是零轮形态，写进判决的交用户表或记进欠账，不再开轮。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」，判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b24-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b24-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
