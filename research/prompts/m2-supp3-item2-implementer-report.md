# 增补 3 第 2 件（理想模型与对拍）实现员报告

时刻都是 UTC（东京 JST = UTC + 9）。工作 05:08 起（限额重置后续做），报告 06 点前后写完。

## 一、结论

- 模型接上了：`crates/singlefs-harness/src/model.rs`（只 `use std` 与 `singlefs_format`），胶水 `crates/singlefs-harness/src/model_comparison.rs`（可以用 core 的类型），`history.rs` 的执行器每一步调入口之前问模型、调完拿实现的结局与模型比，对不上算失败（签名 `FailureSignature::ModelDisagreement`）。
- 今天的代码上三档全绿、收尾与接模型之前逐字相同：快档 96 段「跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0」（与接模型之前的基线同一行），模型判了 1912 步；第 121 行取样点 48 段收尾也与之前相同，模型判了 1183 步。
- 六条攻方变异（B1、B2、B3、B5、B6、N2）接上模型之后都判红，逐条的种子、步、两边的答案在第四节。N2 与 B6 在把第 1 件那两条执行器判定关掉的副本上照样红，红在模型（第四节末）。
- B2 在快档与第 121 行取样点里「回退到 txg = F_生效 > 0 的根」一次都没跑到（两档都是 0 次），另加了一个取样点（回退目标一半取候选集的下沿）和一条固定历史的用例；变异表那一行点名固定历史那一条。
- 多了 10 行变异（`crates/mutations.tsv` 第 146–155 行），在副本上按门禁 59 号的判法只跑这 10 行，10 行各自红在点名的测试上。
- 停下交主 agent 的设计问题 5 个、容量墙的区间 2 个，在第六、七节。新发现：今天的代码上模型与实现没有对不上的格。

**什么现象会推翻「今天的代码上模型与实现一致」**：大档（更多种子、更长的历史）里出现 `ModelDisagreement` 签名的新发现；或者主 agent 改了第六节那几格的读法，模型要跟着改。

## 二、写过的文件

| 文件 | 做了什么 |
|---|---|
| `crates/singlefs-harness/src/model.rs` | 新建：理想模型与它的 5 条单测 |
| `crates/singlefs-harness/src/model_comparison.rs` | 新建：胶水（core 类型 → 模型的观测、错误成员 → 模型的拒绝理由）与 1 条单测（模型模块只 use std 与 singlefs_format） |
| `crates/singlefs-harness/src/history.rs` | 执行器接模型：`HistoryPool` 带模型与录制流，每个 `apply_*` 问模型、比结局；`FailureObservation` 加 `model_disagreement`；签名加一类；已知红的匹配要求模型没对不上；计数与报告加模型那一行；比重加 `ROLLBACK_AFTER_RAISING_THE_FLOOR` 与回退目标的抽法 `RollbackTargetDraw`（原两组比重用第一版抽法，生成的历史逐项不变）；回退目标加 `RingRootAtTheNewestFloor` |
| `crates/singlefs-harness/src/lib.rs` | 加两个模块声明 |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` | 快档的路径断言加「模型判过每一类答案」；新增回退取样点用例、回退到 F_生效 的固定历史用例；大档认 `SINGLEFS_RANDOM_HISTORY_WEIGHTS=rollback` |
| `crates/mutations.tsv` | 末尾追加 10 行（第 146–155 行），变异名见下 |

追加的 10 行变异名（`grep -n '^增补 3 第 2 件' crates/mutations.tsv | cut -d$'\t' -f1`）：

```text
146:增补 3 第 2 件（理想模型，r1 攻方变异 B1）：内容正好装满载荷容量也报装不下
147:增补 3 第 2 件（理想模型，r1 攻方变异 B2）：回退到 txg = F_生效 的根也拒
148:增补 3 第 2 件（理想模型，r1 攻方变异 B3）：抬 F 上限不取第 4 新的非空根（只取每盘最新有效根）
149:增补 3 第 2 件（理想模型，r1 攻方变异 B5）：非空判定恒真（空发布也算非空）
150:增补 3 第 2 件（理想模型自身）：载荷容量少算一字节
151:增补 3 第 2 件（理想模型自身）：抬 F 上限取第 3 新的非空根
152:增补 3 第 2 件（理想模型自身）：txg = F_生效 的回退目标判成低于 F
153:增补 3 第 2 件（理想模型自身）：F_生效取各盘最大 F 的最大（只一块盘带新 F 也生效）
154:增补 3 第 2 件（理想模型自身）：分配代只要不小于写它的那次发布就算对
155:增补 3 第 2 件（理想模型自身，D13 已定项 5）：模型模块 use 了 singlefs_core
```

B6、N2 两条没另加行：第 140、141 行（第 1 件留的）原文与这两条相同、点名快档，今天快档在这两条下同时红在执行器判定与模型上（第四节）；表里一行只改一处，「关掉第 1 件的判定再施加变异」写不成一行，那一格只在副本上跑过。

`git diff --stat -- crates litmus` 原样（新建的两个文件没进索引，不在这里；本机此刻别的会话没有在改 `crates/`，开工时 `git status --short crates/` 为空）：

```text
 crates/mutations.tsv                               |  10 +
 crates/singlefs-harness/src/history.rs             | 635 +++++++++++++++++----
 crates/singlefs-harness/src/lib.rs                 |   2 +
 ..._transaction_supplement_three_random_history.rs | 134 ++++-
 4 files changed, 659 insertions(+), 122 deletions(-)
```

新建文件的行数：`model.rs` 1880 行、`model_comparison.rs` 286 行（`grep -c ''`）。

## 三、模型记什么、答什么，对拍的入口

**记的**（`model.rs` 的 `IdealModel`）：根环 24 个槽里各是哪条根（落点照 D22 已定项 2 / 16 的「第 n 次发布写区域 n mod R 的槽 (n div R) mod S」另算一份）；每条根的 (txg, 实例)、jsn、带的 F、那一版的文件（写它的那次发布与内容）、那一版实例表的行、每个角色的单元是哪次发布写的、分配记录条数与占槽数沿来路的上界；取过的最大实例代号、发布过的最大 txg 与 jsn；这个进程有没有可写会话。

**答的**（每一步三选一：条款要求拒〔给理由集合〕、做成〔给写出的每条根〕、区间里成与拒都对）：

| 问题 | 模型怎么答 | 条款 |
|---|---|---|
| 冷启动该读回哪一版 | 环里 (txg, 实例) 最大的那条根那一版的文件（不建崩溃，没有要施加的记录） | D22 已定项 7（择根序）；D23 已定项 14 |
| 回退到这条根该不该被拒 | 不在环里、txg < F_生效、被最新根的实例表判成被抛弃、树表 0 条（第一版不支持）四条逐条判；都不中就做成：新实例 = 最大 + 1，行 = (r_old, T_old, 0, 回退) 加中间实例 (i, 0, 0)，第一条根 txg = 最大 + 1，暖机到两块盘都有 | D23 已定项 14；D16 已定项 1「回退候选集」；D16 已定项 8；D18 已定项 11 |
| 抬 F 该不该被拒 | 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根)，不足 4 个取最旧的有效根；超过就必须拒；拒与成都要与实现报的上限相等；做成时推空发布到两块盘都有带新 F 的根 | D16 已定项 1「抬 F 的上限」「生效」「非空」从盘上怎么认 |
| 内容装不装得下 | 32768 − 105 − 29 = 32634 字节，多一个字节才拒 | D4 已定项 5；D18 已定项 16 |
| 这个错误该不该出现 | 每个错误成员映射到模型的理由（胶水 `refusal_reason_of_*`），不在「要求拒 ∪ 允许拒 ∪ 区间允许的墙」里就对不上；I/O、盘坏、走读失败这类成员一律没有理由；拒之前录制流多了写或屏障也算对不上 | 各行见第七节区间与第六节停下的格 |
| 每条写出的根对不对 | (txg, 实例)、jsn、F、有没有文件逐项比；带文件的一版里每个单元的分配记录每块盘一条、仍分配、分配代 = 写它的那次发布的 txg | D16 已定项 6；D23 已定项 14 第 3 条；D3 已定项 3 / 7（value = 分配代）；D16 已定项 9（哪些角色每次发布重写） |
| 挂载写的行与取到的号 | 逐项比 | D18 已定项 11；D23 已定项 16 |

**只共享格式常量**：`model.rs` 的 use 只有 `std::collections`、`std::rc` 与 `singlefs_format`（`grep -n '^use' crates/singlefs-harness/src/model.rs` 三行：第 14、15、17 行）；这一条由 `model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants` 判（变异表第 155 行钉着）。

**照代码今天的保守读法写、标了预想的几处**（`grep -n '预想' crates/singlefs-harness/src/model.rs` 现取）：

- 第 633 行 `effective_rollback_floor`：F_生效 = 各盘所带 F 最大值的最小值，被抛弃时间线上的根带的 F 也算——预想，跟收口表第 ② 行。
- 第 662 行 `rollback_floor_ceiling`：上限里「有效」的 F 取 F_生效、一块盘上没有有效根时不算它——预想，跟收口表第 ② 行。
- 第 1022 行：回退之后新实例的 jsn 接在环里最大的 jsn 之后（C340 取 P2）——预想，跟收口表第 ① 行；第五条前缀（回退行的 W）只在施加记录时起作用，不建崩溃就没有施加，模型里没写。
- 第 1025 行：新实例的根带的 F = 恢复后的 F_生效——预想，跟收口表第 ② 行。
- 第 1224 行起 `capacity_wall_is_permitted`：分配记录树那面墙——预想，跟收口表第 39 行（第七节）。

**对拍的入口**（给 `# gate-covers: 模型对拍` 那一段用）：模型接在 `singlefs_harness::history::execute_history_observing` 里（`history.rs` 第 593 行 `judge_by_model`），凡是走随机历史执行器的用例都在对拍。判了多少步由每档报告里「模型对拍 N 步」那一行给出（`HistoryTally::render`），今天的数：

| 用例（都在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`） | 模型判的步数 | 该拒而拒 / 区间里拒 / 该成而成 |
|---|---|---|
| `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`（第 173 行，快档 96 段） | 1912 | 572 / 34 / 1306 |
| `reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms`（第 201 行） | 1183 | 242 / 37 / 904 |
| `rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms`（第 237 行，新） | 913 | 400 / 28 / 485 |

命令：`cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history`（`check.sh` 的 `cargo test --all` 带上它）。快档另比过根 1862 条、分配记录 31250 条、冷启动内容 78 次、抬 F 上限 164 次；这几个数快档里各断言 ≥ 1（第 146 行起 `assert_the_model_judged_every_kind_of_answer`）。

**多线程**：模型每段历史一份、住在那段历史的线程里，计数按种子次序并（`HistoryTally::absorb`）。同一份代码上 release 下核过：快档参数（种子 [0, 96)、30 步）与回退取样点参数各跑 1 线程与 16 线程，报告去掉「用时」行之后逐字相同（各 67 行，`cmp` 无差；模型那一行：broad 1912 步、rollback 1838 步）。

## 四、六条攻方变异：红在哪个种子、哪一步，模型与实现各答了什么

做法：每条一份仓副本（`rsync -a --exclude target --exclude .git`，各用各的 target，`/tmp/claude-1000/m2-supp3-item2-implementer/copies/<名>`），原文与替换文照 `research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv` 那一行、原文在文件里恰好命中一次（`apply-mutation.py`），跑随机历史整个测试二进制（`cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --test-threads 8`）。基线红集：不改动的副本 `baseline` 8 条全绿（05:33:44 UTC 结束，`test result: ok. 8 passed`），所以下面每条的红都在基线红集之外。日志 `/tmp/claude-1000/m2-supp3-item2-implementer/<名>.log`，汇总 `mutants-summary.txt`。

新发现的段数按「回退取样点 / 第 121 行取样点 / 快档」三档列（每档种子数 48 / 48 / 96）。

| 变异 | 快档里第一个判红的种子与步 | 模型答 | 实现答 | 新发现段数 | 同时红的测试 |
|---|---|---|---|---|---|
| B1 内容正好装满也报装不下 | 种子 1，第 16 步（覆盖写） | `PublishOverwrite 该成（写出 [(21, 4)]；允许拒的只有容量墙区间与 []）` | `拒了：PublishError::ContentExceedsDataUnit` | 22 / 36 / 47 | 三档 campaign 3 条 |
| B2 回退到 txg = F_生效 的根也拒 | 种子 6，第 10 步（回退） | `MountRollback 该拒：["回退到树表 0 条的根（第一版不支持）"]` | `拒了：MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）` | 48 / 2 / 27 | 三档 campaign 与固定历史用例 `rolling_back_to_the_root_at_the_effective_floor_…`，共 4 条（副本 `B2-directed`） |
| B3 抬 F 上限取 max | 种子 0，第 19 步（抬 F）；另一类种子 8，第 11 步 | 前者 `RaiseRollbackFloor 该拒：["要抬的 F 超过上限"]`；后者 `上限 Some(5)` | 前者 `做成了`；后者 `实现报上限 10` | 38 / 46 / 65 | 三档 campaign 与 3 条固定历史用例（`an_allocated_statistic_over_count_…`、`row_and_warm_up_publishes_…`、`raising_the_floor_into_the_gap_…`），共 6 条 |
| B5 非空判定恒真 | 种子 0，第 19 步（抬 F）；另一类种子 1，第 12 步 | 前者 `上限 Some(0)`；后者 `该拒：["要抬的 F 超过上限"]` | 前者 `实现报上限 17`；后者 `做成了` | 38 / 45 / 63 | 同 B3，6 条 |
| B6 补齐字节从最后一个载荷字节算起 | 种子 0，第 1 步（冷启动） | `所选根 (txg 4, 实例 1)，读回 23416 字节（末字节 Some(73)）` | `读回失败：InvariantViolated { invariant: "I-2.3", detail: "补齐字节非零" }` | 14 / 4 / 45 | 三档 campaign 3 条 |
| N2 复用改写已回收记录时不改分配代 | 种子 3，第 25 步（覆盖写） | `(txg 29, 实例 5) 的 Data：每块盘一条、仍分配、分配代 29（写它的那次发布）` | 两块盘槽 50176 的记录 `generation: 5, is_released: false` | 7 / 28 / 18 | 三档 campaign 与 `row_and_warm_up_publishes_…`，共 4 条 |

说明：

- B1 那一步的内容长度没单独打出来；变异只把 `>` 改成 `>=`，被它多拒的只有「正好 32634 字节」这一种。
- B2 在快档里红的是理由：F_生效 = 0 时回退到 txg 0 的 mkfs 根，条款上它只该按「树表 0 条」拒（txg 0 ≥ F_生效 0，是候选），变异让实现报「txg 低于 F」。语义那一格（F_生效 > 0 的候选被拒）在回退取样点里是种子 7、36 的第 16 步：模型 `MountRollback 该成（写出 [(19, 4), (20, 4)]…）`，实现 `拒了：MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）`；固定历史用例第 6 步：模型 `该成（写出 [(12, 3), (13, 3)]…）`，实现同上。语义那一格在回退取样点上的余量薄：release 下 B2 副本逐窗数 [0, 960) 的 20 窗（每窗 48 个种子），每窗「模型说该成、实现拒了」的段数是 2、0、2、2、2、3、3、1、0、4、2、1、5、1、1、1、3、4、1、1（日志 `B2-window-*.log`），所以变异表那一行（第 147 行）点名固定历史用例，不点名取样点。
- B3 / B5 两类签名是同一个错的两面：上限算高了，要抬的 F 落在模型的上限之上就是「该拒、实现做成了」，落在两者之下就只剩实现报的上限与模型不等。
- B6、N2 在今天的副本上签名取的是第 1 件的执行器判定（签名次序：panic 先于执行器判定先于模型），同一步的观察里模型那一格也对不上（`B6.log` 第 81 行、`N2.log` 第 89 行）。

**N2、B6 不靠第 1 件的判定也判红**：副本 `N2-model-only`、`B6-model-only` 先把 `history.rs` 里那两条执行器判定关掉（分配代：`(!records.is_empty())` 前加 `false &&`；冷启动读回报错：`Failed` 那一臂加 `if false`、落进 `None` 那一臂），再施加变异。只关判定、不施加变异的副本 `judgements-disabled-baseline` 8 条全绿。结果（`model-only-summary.txt`）：

| 副本 | 快档 | 签名 | 模型答 / 实现答 |
|---|---|---|---|
| `N2-model-only` | 新发现 18 段，种子与今天的副本逐个相同（3, 7, 15, 20, 26, 27, 38, 46, 51, 54, 56, 57, 62, 66, 68, 69, 71, 94），第一个种子 3 第 25 步 | `ModelDisagreement { aspect: "单元的分配代" }` | 同上表 N2 那一行 |
| `B6-model-only` | 新发现 45 段，第一个种子 0 第 1 步 | `ModelDisagreement { aspect: "冷启动读回" }` | 同上表 B6 那一行 |

两份副本里红的测试与施加变异、不关判定时相同（N2 4 条、B6 3 条）。模型答得了分配代，依据是 D3 已定项 3 / 7（value = 分配代）加 D16 已定项 9（每次发布重写哪些角色）：模型记「这一版每个角色是哪次发布写的」，不记落点，落点由胶水从实现交回的 `TransactionOutput::units` 里取。这是内存里的判定；盘上要不要新立一条不变量（收口表第 44 行）仍没人做，见第六节。

## 五、每条新测试的会红证明（改坏哪一行 → 哪条断言红）

副本各自 rsync、各用各的 target；基线：不改动的副本 lib 测试二进制 33 条全绿（`model-baseline`，05:39:17 UTC），随机历史二进制 8 条全绿（`baseline`，05:33:44 UTC）。下面引的行号是施加变异那一刻副本里的行号，与主工作区现在的 `model.rs` 相同（`diff -q` 无差）。

| 新测试 | 改坏哪一行 | 哪条断言红 | 同时红的 |
|---|---|---|---|
| `model::tests::content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused` | `model.rs` `data_unit_payload_capacity_in_bytes` 里 `DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET` 后面加 `- 1`（变异表第 150 行） | 第 1729 行 `assert_eq!(data_unit_payload_capacity_in_bytes(), 32768 - 105 - 29)`：left 32633、right 32634 | lib 二进制里只它一条（32 passed、1 failed） |
| `model::tests::the_ceiling_is_the_fourth_newest_non_empty_root_capped_by_the_newest_root_on_every_device` | `match non_empty_txgs.get(3)` 改成 `get(2)`（第 151 行） | 第 1751 行：上限 left `Some(7)`、right `Some(6)` | 只它一条 |
| `model::tests::rollback_to_the_effective_floor_is_accepted_and_abandons_the_newer_roots_of_that_instance` | `if target.checkpoint_txg < self.effective_rollback_floor()` 的 `<` 改成 `<=`（第 152 行） | 第 1793 行 `assert_eq!(rollback.expected_mount, …)`：left `None`（模型说该拒） | 只它一条 |
| `model::tests::a_raised_floor_carried_by_one_device_only_does_not_take_effect` | `effective_rollback_floor` 末尾各盘最大 F 的 `.min()` 改成 `.max()`（第 153 行） | 第 1835 行：left `ModelCheckpointTxg(6)`、right `ModelCheckpointTxg(0)` | 只它一条 |
| `model::tests::a_carried_unit_keeps_the_generation_of_the_publish_that_wrote_it` | `&& record.generation == *written_at` 改成 `>=`（第 154 行） | 第 1877 行 `.expect_err("照抄的数据单元分配代写成了这次的 txg")` | 只它一条 |
| `model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants` | `model.rs` 的 `use std::rc::Rc;` 之后加一行 `use singlefs_core as _;`（第 155 行） | `model_comparison.rs` 第 272 行：`模型模块里有一行提到了实现或 checker：use singlefs_core as _;` | 只它一条 |
| `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`（新，固定历史） | B2：`mount.rs` `if target.checkpoint_txg < effective_floor {` 的 `<` 改成 `<=`（第 147 行） | `assert_eq!(run.ending, HistoryEnding::Completed, …)`：实际 `NewFinding { signature: ModelDisagreement { aspect: "模型说该成、实现拒了" } … position: Operation(6) …}` | 三档 campaign（共 4 条） |
| 同上用例的计数断言 | `model.rs` 里 `counts.rollbacks_accepted_at_the_effective_floor += 1;` 换成注释 | `模型数到一次落在 F_生效 上的回退`：left 0 | 回退取样点那条的 `回退到 txg = F_生效 > 0 的根一次都没做成（B2 那一格没跑到）` |
| `rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms`（新取样点） | 分类那一条断言：六条攻方变异下都红（第四节表的第一档）；覆盖那一条断言：同上一行的计数变异 | `「已知红」清单外的失败` / `回退到 txg = F_生效 > 0 的根一次都没做成` | 见第四节 |
| 快档新加的 `assert_the_model_judged_every_kind_of_answer` | `model.rs` 里 `counts.cold_start_contents_compared += 1;` 换成注释 | `模型一次冷启动读回的内容都没比过` | 固定历史那一条的 `冷启动读回的内容模型比过`（left 0） |

那两条计数变异只在副本上跑过、没进变异表：它们钉的是计数断言活着，不是一处三方打中之后的改法。

**变异表新加的 10 行按门禁 59 号的判法跑过**：副本 `copies/gate59-new-rows` 的 `crates/mutations.tsv` 只留表头与这 10 行，`bash .claude/gate.d/59-crates-mutation-replay.sh <副本>`（`GATE_MUTATION_TARGET_DIR` 指副本自己的目录），末行原样：

```text
  ✓ crates 变异表复跑：10 条变异各自红在点名的测试上（原文都恰好命中一次）
```

整张表 150 行的 59 号没跑（归 `crash-verifier`）。

## 六、停下交主 agent 的设计问题（照定义第 6 步；`crates/singlefs-core` 一行没改）

模型答不了、或答了但依据是「照代码今天的读法」的格：

1. **抬 F 时现行版本的实例表还是 mkfs 那一片**（`MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`）。D16 已定项 1 没有这个拒绝；mkfs 的表没有行，候选集照样判得出。实现拒是因为它从现行版本的单元里读表、那一版里没有实例表单元（`crates/singlefs-core/src/mount.rs` 第 40–43 行的注释）。模型把它划成「允许拒」（`model.rs` `answer_raise_rollback_floor` 里 `permitted_refusals.insert(…RaiseWithFormatTimeInstanceTableUnsupported)`），不判成该成也不判成该拒。走得到：接模型之前的基线快档 34 次、第 121 行取样点 37 次（`baseline-fast.log` 第 94、29 行）。要定的：这一格该成（实现改成从盘上读 mkfs 的表），还是登记成「第一版不支持」。
2. **另三个「第一版不支持」照代码判成必须拒**：树表 0 条的一版上要写行（`InstanceRowsOnVersionWithoutFileUnsupported`，基线快档 92 次）、回退到树表 0 条的根（`RollbackToVersionWithoutFileUnsupported`，125 次）、空池挂载形状不像第一个事务（`FormattedPoolMountNotShapedLikeTheFirstTransaction`，0 次）。三处条款都没写，实现的文档注释写了「没有条款、第一版不支持」；模型照它写成必须拒，理由集合里各一条。定案改了模型跟着改。
3. **候选集与上限用哪张实例表、哪个 F**：模型用最新那条根的表与 F_生效；实现回退用最新根的表与 F_生效，抬 F 用现行版本的表与现行根带的 F。不建崩溃时三者相同（每一步都比根带的 F，没对不上过），第 3 件（崩溃注入）起会分开。跟收口表第 ② 行一起定。
4. **往下抬 F 与「一块盘上没有有效根」两格模型不答**：前者返回 `ModelDisagreementAspect::NotModeled`；走不到，因为生成器取目标是 `现行 F + 选择子 mod (txg − F + 3)`（`history.rs` `apply_raise_rollback_floor`），现行 F 取的是实现现行根带的 F，而模型每一步都把根带的 F 与实现比过、不等就停。后者在上限里按「不算那块盘」写（与实现相同）；走不到，因为抬 F 与暖机都推到每块盘都有一条 txg 大于 F 的根（D16 已定项 1「生效」、已定项 8），回退之后新实例的暖机同样。
5. **收口表第 44 行（分配代盘上没有判据）仍开着**：模型在内存里判了分配代（第四节），盘上要不要新立一条不变量（例：已分配记录的分配代等于把它写成已分配的那次发布的 txg）是 kb 的事，没做。连带一格：`RollbackTargetNotACandidate` 不带是哪一条（低于 F 还是被抛弃），胶水只能映射成「两条之一」，实现报错了理由（例如被抛弃的根报成低于 F）模型分不出；要不要给这个成员加一个判别字段，交主 agent。

另记一格不影响今天判定的：可写挂载写的上一个实例那一行 W 模型恒写 0（不建崩溃就没有施加的记录，D23 已定项 14 第 4 条）；第 3 件接崩溃时要按施加了哪些记录算。

## 七、容量墙：模型答的「允许拒绝的区间」（单列）

| 墙 | 下端（必须拒） | 上端（允许拒） | 两端的出处 |
|---|---|---|---|
| 分配记录树一个节点（`AllocationRecordNodeWall`；实现的 `AllocationRecordsExceedOneNode`、`RowPublishAdmissionRefusedBeforeAcquisition`、`WarmUpAdmissionRefusedBeforeAcquisition`） | 这次之后的真条数 > 812。模型不知道真条数，这一端不判（真装不下还去写会 panic，由第 1 件的 panic 判定接） | 沿来路的上界 > 812：mkfs 两个单元每盘一条，之后每次发布每个重写的角色每盘加一条，释放、回收、复用都不扣。发布与抬 F 按拒的那一次判；可写挂载与回退在取号之前一串判，计划里任一次超了就允许 | 812 = (16384 − (86 + 2 × 10 + 29)) ÷ 20：D8 已定项 11（节点头）、D3 已定项 11（key 10、条目 20）；「一条记一个单元、释放只改写不删」：D3 已定项 7；「每个角色每盘新增一条」的上界准入：收口表第 39 行，2026-09-18 用户定保留。标「预想，跟收口表第 39 行」（`model.rs` 第 1224 行起）。模型的上界沿来路累加，不是实现「分配器此刻条数 + 这次新增」那一个算法，只会更宽 |
| 单元区（`UnitAreaWall`；实现的 `NoSpaceFor`） | 同样不判 | 沿来路的占槽上界 × 64（一个聚簇段的槽数）> 单元区槽数（盘字节 ÷ 16384 − 50176） | D28 已定项 1 的准入式子（第一版没实现，各项没有现值）；保留池 10 + 7 c_max（D16 已定项 1）、切换预留（D28 已定项 3）；× 64 是我取的倍数（已分配 + defer ≤ 2 × 上界、每个落点最坏独占一段、保留池与切换预留都在里头），预想、没有条款。4 GiB 的盘上单元区 211968 槽，快档里走不到 |

另有两面墙模型答得出确切值、不是区间：记账树（3 + 6 × 盘数 > 477 才拒，两块盘恒不拒；D5 已定项 8）、实例表一片（这一版的行 + 要写的行 + 1 > 370 必须拒；D18 已定项 11）。

## 八、check.sh、门禁阶段、多花的时间

`nice -n 19 bash .claude/scripts/check.sh`（05:48 UTC 起，主工作区，跑前 `ps` 没有 qemu / vm-bench / e152 / fio / cargo / gate.sh 在跑）末尾原样：

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四段都过（`grep '✓' check.log`：格式通过、clippy 通过、构建通过、单测通过），退出码 0，墙钟 84 秒（含改动之后的重编译）。日志 `/tmp/claude-1000/m2-supp3-item2-implementer/check.log`；快档那一段在里面第 492 行起，模型那一行第 526 行。

登记给 `implementation-writer` 的阶段（`awk … stage-owners.tsv` 列出 33、53 两个），05:58 UTC 在最终状态上跑：

```text
33 退出码 0
  ✓ 141 个实验二进制都有成形的变异表，1475 条变异的原文各命中源码一次；crates/mutations.tsv 150 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）
53 退出码 0
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
```

**加了模型之后 check.sh 多花多少秒**：拿 HEAD 的 `crates/` 做一份副本（`git show HEAD:<文件>` 取回改过的四个文件、删掉两个新文件），先热一趟，再与主工作区交替各跑 check.sh（`check-timing.txt`）：

| 趟 | HEAD 副本 | 主工作区（加了模型） |
|---|---|---|
| 第 1 对 | 130 秒（与我在副本上跑的两条计数变异撞在一起，不算） | 83 秒（开头与那两条变异的尾巴有重叠） |
| 第 2 对（机器只跑它） | 67 秒 | 70 秒 |

check.sh 多约 3 秒。随机历史那个测试二进制单独计时（默认线程数，交替两趟）：HEAD 34.9 / 35.2 秒，加了模型 42.3 / 42.2 秒；多出来的主要是新加的回退取样点与另两档抢核（三档同时各起 16 个线程）。模型本身的开销在噪声里：只接模型、还没加回退取样点时快档自报 34.7 秒，接模型之前 36.6 秒（同一命令 `--test-threads 8`，`baseline-fast.log` 与 `model-fast-1.log`）。

## 九、发现与收口表

- 今天的代码上模型与实现没有对不上的格（三档共判 4008 步：1912 + 1183 + 913；固定历史用例另计）。第 1 件的两条已知红照旧：快档 {0: 50, 1: 1}，与接模型之前相同；已知红的匹配现在另要求「模型没对不上」（`history.rs` 第 1081 行 `only_allocated_statistic_above_walked`），今天的收尾没因此变。
- 建议进增补 2 收口表的（我只报，不写 kb）：第六节第 1 格（抬 F 时实例表还是 mkfs 那一片，条款没有这个拒绝）；第 5 格（第 44 行那一格现在内存里判了、盘上仍没有判据；`RollbackTargetNotACandidate` 不带理由）；第 3 格并进第 ② 行。第 2 格三个「第一版不支持」各自已在实现的注释里写明，要不要进表由主 agent 定。

## 十、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与整张 `crates/mutations.tsv` 的复跑（门禁 59 号）归 `crash-verifier`，只在副本上按 59 号的判法跑了新加的 10 行；没提交。
- 门禁阶段声明 `# gate-covers: 模型对拍` 没写（`.claude/gate.d/` 不在我的写范围），入口与步数在第三节。
- 没重跑第 1 件的判别力（`crates/mutations.tsv` 第 41、121 行改回去快档在写死的种子数之内判红）：模型只多加失败条件，不会让原来红的变绿，但这一点没在副本上核；整表复跑时一起看。
- 模型不建崩溃、不建设备错、不建 journal 施加：可写挂载那一行的 W 恒 0，回退的第五条前缀规则没写；第 3–5 件接上时要补。
- 大档没跑（快档参数的 1 线程 / 16 线程对比在 release 下跑过，第三节）。
- 跑出来的东西都在 `/tmp/claude-1000/m2-supp3-item2-implementer/`，没进 `research/results/`（写范围不含 `research/`）：变异副本的日志（`B1.log` … `N2.log`、`*-model-only.log`、`M*.lib.log`、`B2-directed.log`、`counter-*.log`、`gate59-new-rows.log`、`B2-window-*.log`）、汇总（`mutants-summary.txt`、`model-only-summary.txt`、`model-mutants-summary.txt`、`check-timing.txt`）、脚本（`apply-mutation.py`、`run-mutants.sh`、`run-model-only.sh`、`run-model-mutants.sh`、`run-one.sh`、`time-check.sh`、`model-mutants.tsv`）、仓副本（`copies/`，约 150 MB 一份不含 target）。要不要拷进仓由主 agent 定。
- 进度记录 `progress.log` 里有一行时刻是估的，下一行已更正（变异证明实际 05:33 UTC 起）。
