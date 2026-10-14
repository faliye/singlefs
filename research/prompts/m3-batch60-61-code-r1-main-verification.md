# m3-batch60-61-code-r1 判决：删除线第 60、61 批代码轮第一轮（2026-10-11）

被判的改动（按路径点名）：`crates/singlefs-checker-tier/src/verdict_store.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs`、`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs`、`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs`、`crates/mutations.tsv`。

<!-- doc-lint:not-numbers N1 N2 N3 N4 P1 P2 P3 P4 P5 Q1 Q2 Q3 Q4 Q5 B1 B2 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 主工作区 `sha256sum -c research/prompts/m3-batch60-61-code-r1-snapshot/crates-sha256.txt` 与 `kb-sha256.txt` | 都对上，腿跑着的时候被判文件与 kb 没被改 |
| 正推腿 `research/prompts/m3-batch60-61-code-r1-sonnet-output.md`（158 行，sha256 d9d159c32ca1c32a…）、攻方 `research/prompts/m3-batch60-61-code-r1-opus-output.md`（321 行，sha256 02620e11ef6d7ee6…） | `cite-check.py` 两份报告，`--root` 取仓根，`--background` 取背景材料 | 「✓ 核了 84 处引文，对上 84 处，没判 0 处」（以冻结副本为根时 kb 那 10 处路径找不到，换仓根后全对上） |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-verify/model/`，`SINGLEFS_SCRIPTS` 指仓的脚本，跑 base-build、base-batch、base-h1–h4、mut-mv、mut-mi-pipe、mut-mp、compare | 见第六节 |
| 合入后验证（主 agent 在主工作区跑） | `merge-and-verify.sh --skip-apply` | 「✓ 测试全绿（194 个测试二进制，共 2019 条测试）」，fmt、clippy、build 过，日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-11-523716` |
| 第 61 批 checker 档用例与证红（实现员副本） | 两个测试二进制、库单测、`prove-red.sh` 12 行 | 「16 passed」「27 passed」「2 passed」；「点名 12 条：跑了 12 条，抓到 12 条」 |
| 第 60 批 GPU 对拍（主工作区，`gpu-quiesce-run.sh`） | `--test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- knob_two_agrees_with_the_cpu_on` | 「test result: ok. 12 passed; 0 failed」，含第 59 批 L1 退役那条（原先 GPU 表放不下） |
| GPU 变异证红 6 行（第 59 批 L1 三行、第 60 批 N1 三行） | 小副本 `~/.local/share/singlefs/work/m3-batch60/gpu-proof/repo` 上 `prove-red.sh` | 见第六节末 |
| 崩溃与变异两条轻型 | 根环线在主工作区排的那一趟（轻型崩溃全部加轻型变异，含第 60、61 批） | 结果出来补进第四节；形式化轻型 pinned-red-examples-complete 格的红是已知的那份账缺失，不归这两批 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| P1 GPU 远左短片与精确红集 | 一致（卡上部分复核不了） | 没打中，只读码 | 一致，过。卡上 12 条主 agent 跑绿；「正确判法下顺带红出别的违例」两腿都只引第 59 批替身，卡上对拍的精确红集断言在 12 条上都过，等于在被判那份代码上量过 |
| P2 统计量 7 与分配记录判序 | 一致 | 没打中，量过（13、43 条全绿，3957、3958 各只红自己那条） | 一致，过 |
| P3 轻型核心行修补 | 新取样点一致；根槽清零再写回是造的 | 没打中：清零与「挂载那一刻读不出」在根环表里同一类，是可达历史的替身 | 过。checker 档三行（84、2728、3959）两腿都没跑，归根环线那趟变异轻型 |
| P4 按计划导入的正确性 | 一致（读码） | **打中**（P4-1）：排键时核对行一行都不点，第 60、61 批用例照绿；其余（读回逐项相同、断了重跑补齐、给定状态表那一路）没打中，量过 | **打中，采纳**：测试缺口，产品代码在原样下读回相同（主 agent 复跑 base-h1 `without_verifier=0`、mut-mv `without_verifier=1`、`planned_vs_whole=1`）。改法 Q1 |
| P5 代价与钉值 | 代价一致，21 行两路核过 | 代价没打中，量过（对方库 0 到 120000 块，Get 都是 193 次）；**打中（小）**（P5-2）：`imported_blocks` 去掉「已有且相同」那一项照绿 | **打中，采纳**（主 agent 复跑 h07 第二次导入 base `imported=2`、mut-mi-pipe `imported=0`）。改法 Q2 |
| 正推附带 B1 | 换批、跨批编号、断了重跑补齐没有用例；批长 64 MiB 用例只走一批 | 攻方在批长改 256 KiB 的副本上杀进程 16 个时刻都补齐，冻结副本原样批长没跑完 | 采纳为测试缺口：改法 Q3 |
| 正推附带 B2 | 事实行补写靠 `facts_digests_named.insert` 排在跳过判之前，没有用例与变异 | — | 采纳：改法 Q3 |
| 正推附带 A | I-9.36、I-9.37 不在 `invariants.md`，I-9.22 缺「最右那片之外 == 旋钮值」半句 | — | 已在提交前 kb 写回清单里，不进这一批 |

## 三、采纳的改法（第 62 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| Q1 | 第 61 批流水线导入用例（或新一条同形）设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`，断言导入那一趟 `blocks_without_verifier == 0`、按块读回的核对行与整库导入（或一台判完那份库）相同；取攻方 h02 那种「第二台那一块只在对方库里核过」的形状 | 变异表追加攻方 `mv`（`crash_amplification.rs` 排键处两种核对代码都不点）那一行，点名新断言、证红 | P4-1 |
| Q2 | 第 61 批两条用例各在导入之后再导一次，钉第二次的 `imported_blocks` 等于第一次（全是已有且相同） | 攻方 `mi-pipe`、`mi-given` 两行进变异表、证红 | P5-2 |
| Q3 | 按计划导入的换批：批长做成可由测试给的参数（产品默认 64 MiB 不变），用例在小批长下走多批，断言跨批编号接续、读回与一批相同；再造一份事实行缺的本库（核对行在、事实行不在）重导，断言事实行补上 | 至少三行变异：换批判据反过来、跨批编号重起、`facts_digests_named.insert` 挪到跳过判之后；各点名、证红 | 正推 B1、B2 |
| Q4 | GPU 配置下点名导入：给定状态表那一路与流水线那一路，漏掉 GPU 判器那一种结论、漏掉 GPU 核对代码那一把摘要，各有一条带 `gpu` 特性的用例会红 | 两行变异各点名、卡上证红（主 agent 照清场规则排卡） | 第 61 批第二位交回、攻方 P4-1 第 4 句 |

## 四、轻型与欠账

- 崩溃与变异两条轻型：根环线那一趟出结果后补一行在这里，红了先查是不是这两批带进来的。
- 补（2026-10-11）：根环线那一趟因驱动版本对不上整趟退 78、没有结果；重启之后主 agent 在主工作区（第 60–63 批都在）重跑轻型崩溃：`CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=16 collected=16 green=12 red=3 not_run=1`。红的三项都只红在第二台那一份、本机那一份绿，两桩成因都在双机装置一侧（第二台进程认错自己是哪台、第二台算的计数向量与段区间对不上），不是这两批改的文件；各派一个调查员，记录在 `records/2026-10-01-删除与并发事务.md`。轻型变异改归提交时的 checker-tier-crates-mutation-replay 整表。形式化轻型 pinned-red-examples-complete 那份账缺失已修（从归档提交 31128273 的父提交取回 kill-bounds 产物），三格全绿。
- 照旧记欠账：改法 2（按段预算前缀持久掩码）先量再定；`imported_blocks` 几份库同一块各数一次（旧导入同形，不是回退）；导入改完之后的秒数没量（攻方挂钟在负载 50–130 时量的，不作数）；J4 与第 59 批 L6、E9 那几行 GPU 变异归根环线；「只判最右之前 k 片」对足够大的 k 有限组排布关不死；形式化轻型那份红例账缺失归谁修待定。
- 「没打中」各格都只抽了一次样；P4、P5 的「没打中」攻方都是量出来的（8 段历史放开份数、先后、快照、先判着、重导、ignore、核与不核；对方库四档），不只读码。

## 五、第 62 批派发

一个实现员关 Q1–Q4（都落在 `crash_amplification.rs`、`verdict_store.rs`、`tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 与一份带 `gpu` 的测试文件），交补丁；checker 档测试与证红由主 agent 跑。

## 回看决策

不涉及决策：第 60 批只改测试与变异表；第 61 批改崩溃放量判定库的导入（验证装置），不碰盘上格式与任何决策分项的条款。

## 六、复跑核对（攻方模型）

在 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-verify/` 跑，base-h1、base-h2、base-h3、base-h4、base-batch 的输出与攻方 `expected/` 逐字相同（`diff -q` 无输出）；`compare` 报「一致 mut-mv.txt」「一致 mut-mi-pipe.txt」「一致 mut-mp.txt」，另 11 份不一致是这次没跑的阶段（没有输出文件）。逐行：

```
base-h1：OPUS_P4 history=h02 step=planned-import-0 share=0/2 checked=0 imported=2 unjudged=0 violations=0 stopped=false compared=45 without_verifier=0 without_judge=0 disagreements=0
mut-mv： OPUS_P4 history=h02 readback_lines=10 planned_vs_whole=1 planned_vs_one_host=1 whole_vs_one_host=0
mut-mv： OPUS_P4 history=h02 step=planned-import-0 share=0/2 checked=0 imported=2 unjudged=0 violations=0 stopped=false compared=33 without_verifier=1 without_judge=0 disagreements=0
base-h4：OPUS_P4 history=h07 step=planned-import-1 share=0/2 checked=0 imported=2 …
mut-mi-pipe：OPUS_P4 history=h07 step=planned-import-1 share=0/2 checked=0 imported=0 …
```

GPU 变异证红（第 59 批 L1 三行、第 60 批 N1 三行）：

```
✓ 点名 6 条：跑了 6 条，抓到 6 条，本次未跑 0 条，跑了的都抓到了
```

六行各红在点名的那条对拍上（第 59 批 L1 退役那一行原先等 GPU 表上限，这一次也抓到）；日志 `~/.local/share/singlefs/work/m3-batch60/gpu-proof/prove.log`。
