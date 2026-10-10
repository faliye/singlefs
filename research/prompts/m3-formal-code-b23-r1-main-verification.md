# 第 23 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F3 H2 A1 B1 -->

被判的文件：`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`，`crates/mutations.tsv` 23-1 追加的 7 行。

这组用例（崩溃注入第二、三截的接线、发布臂与 F3）经第 18、20、21、23 批四次代码轮；照 `.claude/rules/three-way-inference.md`「第三轮之后停」，下一批的代码轮是这组改法的最后一轮，再冒出的零轮形态写进交用户表或记账，不再开轮。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b23-r1-body.md`、`_m3-formal-code-b23-r1-diff.md`、`_m3-formal-code-b23-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b23-r1-snapshot/`（crates 287 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b23-r1/tree/`；kb） | 冻结副本对 crates 快照全过，kb 对快照全过 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b23-r1-sonnet-output.md`（sha256 b55e514d…），模型 `…-sonnet-model/` | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b23-r1-opus-output.md`（sha256 eb49bc27…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh all` 在会话草稿 `b23-opus-rerun/` 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本的 crates 加主仓 .claude、research、records 拼的根> --background <背景材料>`：「✓ 核了 82 处引文，对上 82 处，没判 0 处（逐处列在上面）」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 K3-a 的态与计数 | 一致：写下标与攻方 b21 `k3-sweep.txt` 逐个对上，判官层在 7 个配置上量过 | 没打中：48 段历史正确接线绿、接错红；覆盖写一个写都没写出的 24 段在 `expect` 处直接 panic，不悄悄摆错 | 站住 |
| K1 F3-c 抽函数 | 一致：200 万随机加 262 万穷举 0 处不同 | 没打中：20 万组 0 处不同 | 站住 |
| K1 F3-c 4095 断言的判别力 | 部分：抓得住折得太少的，抓不住 `.take(12)`、`.take(13)`、末 12 个写窗口、低 32 位、掩码当整数；真历史段长到 28 | 打中（弱）：六种有害改坏照绿（只折前 12 / 16 个写、按位拼接、去段号、去历史种子、或代异或）；用例固定种子 0、段号 16、段长 12，真历史候选段长到 26 / 30 | **坐实**（复跑 `f3-variants.txt`、`f3-segments.txt`、`f3-fix.txt` 同）。两腿方向一致。改法 F3-d 进下一批：段号 0..=63 各 4095 个子集、长 28 的段末 12 个写 4095 种、历史种子 0 与 1，种子各自两两不同；照攻方表各配变异 |
| K2 A1 字段 | — | 没打中：结构体六个字段全比，9 种坏发布与三行 core 变异都红；盘上另九样字段 `parse` 不读（读码推） | 站住；盘上那九样不在这组用例的射程里，不另立 |
| K2 A1 三行 core 变异牵连 | — | 三行都让 harness 档 `many_inodes_write_and_read` 红，nlink 那行另红 4 个、size 那行另红 1 个 | 采为说明，不改：变异行点名的是 A1 那条，别处也红只说明 harness 档同样盯着；A1 多出的判别力在第二截那一支自己的参数上（写入时间 0、建 2 个） |
| K2 B1 136 槽 | — | 稳：分得开的那一档 134–138 槽，136 在正中 | 站住 |
| K2 B1 对别的盘面参数 | — | 打中（弱）：那一档每组只宽 5 槽、每多一个数据单元上移 2 槽；盘面往下漂 3–8 槽时用例照绿而「建 0 个」变异不再红，用例没有前提钉住 | **坐实**（复跑 `b1-slots.txt`、`b1-neighbourhood.txt` 同；`summarize_b1.py` 汇总「36 class=GREEN-KILLS-BOTH」「40 class=GREEN-KILLS-SOME-ONLY」「446 class=TEST-RED-ON-CORRECT」）。改法 B1-c 进下一批：前提块加「同一份盘面新挂一次，那一支建 0 个能做成」 |
| K3 H2 前提断言 | 成立：S = 5–15 上两条原样用例绿、放宽一个 txg 的变异都红；S = 4、16 在那条路径上编不过 | — | 站住；常数注释「16 上量过」照现值改（下一批） |
| K3 注释 | `writable_mount_refusal…` 第 287–290 行对得上；模块文档第 4–5 行仍是旧措辞（b21 判决 K1-d 指过、23-1 只改了函数注释）；`root_record.rs:26` 对得上，但挂在「= 0」变体下易读成记号恒为 0 | — | 采：模块文档两行与 `root_record.rs:26` 措辞进下一批 |

## 三、去向

1. 23-1 的改动站住，不回改。
2. 下一批实现员：F3-d、B1-c、`writable_mount_refusal…` 模块文档第 4–5 行、H2 常数注释、`root_record.rs:26` 措辞。下一批的代码轮是这组改法的最后一轮。
3. 提交时：23-1 与下一批的 checker 档用例、变异整表。

## 四、复跑核对

主 agent 在会话草稿 `b23-opus-rerun/` 照 `WORK=<草稿> THREADS=7 bash research/prompts/m3-formal-code-b23-r1-opus-model/tools/rerun.sh all` 复跑，退 0；`a1.txt`、`b1-neighbourhood.txt`、`b1-slots.txt`、`collateral.txt`、`f3-fix.txt`、`f3-model.txt`、`f3-segments.txt`、`f3-variants.txt`、`k3a-variants.txt` 与模型目录 `out/` 逐个 cmp 相同；`summarize_b1.py` 对复跑产物退 0，汇总与报告 K2-b 那段同。

核不动：checker 档用例与变异没跑；A1 盘上那九样字段编码器写错 A1 看不见，读码推的；攻方「没打中」各格只抽一次。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | F3-d、B1-c | 攻方 | 只在攻方模型上量过 | 下一批实现员；不用用户定 |

不涉及要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-04 不受影响：判别力缺口在用例，不在条款 |
