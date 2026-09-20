# 背景材料：增补 3 第 2 件（理想模型与对拍）的代码三方对抗第一轮（2026-09-19）

<!-- doc-lint:not-numbers M1 M2 M3 M4 M5 M6 V1 V2 V3 V4 V5 V6 B1 B2 B3 B5 B6 N2 -->

## 一、被判的对象

`.claude/kb/milestone/02-second-txn.md`「增补 3　捶打 `crates/`」第 2 件（理想模型与对拍）与它的验收（同一节「验收标准」第一条，第 2 件括注里「攻方腿的六条变异在模型接上之后都要判红」）。实现员报告 `research/prompts/m2-supp3-item2-implementer-report.md`，日志与脚本在 `research/prompts/m2-supp3-item2-implementer/`。基准提交 `5efaf79`；两份新文件没进 git，按「文件::项名」给。**开工快照**：`research/prompts/m2-supp3-item2-code-r1-start-snapshot.sha256`（19 个文件）。

| 文件 | 项 |
|---|---|
| `crates/singlefs-harness/src/model.rs`（新，1880 行） | `IdealModel` 与它的全部方法，重点：`effective_rollback_floor`、`rollback_floor_ceiling`、`answer_raise_rollback_floor`、回退的候选判定与新实例的行、`capacity_wall_is_permitted`、冷启动该读回哪一版、每条写出的根与分配代的比法；模块内 5 条单测 |
| `crates/singlefs-harness/src/model_comparison.rs`（新，286 行） | 胶水：core 类型 → 模型的观测、错误成员 → 模型的拒绝理由（`refusal_reason_of_*`）；单测 `the_model_module_uses_only_the_standard_library_and_the_format_constants` |
| `crates/singlefs-harness/src/history.rs` | `judge_by_model`（第 593 行）、`HistoryPool` 带模型与录制流、各 `apply_*` 问模型与比结局、`FailureObservation::model_disagreement`、`FailureSignature::ModelDisagreement`、已知红匹配要求模型没对不上、`HistoryTally` 的模型计数与 `render` 的「模型对拍 N 步」行、`GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`、`RollbackTargetDraw`、`RollbackTargetChoice::RingRootAtTheNewestFloor` |
| `crates/singlefs-harness/src/lib.rs` | 两个模块声明 |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` | `assert_the_model_judged_every_kind_of_answer`；快档与第 121 行取样点的路径断言；新用例 `rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms`、`rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`；大档认 `SINGLEFS_RANDOM_HISTORY_WEIGHTS=rollback` |
| `crates/mutations.tsv` | 第 146–155 行 |
| `.claude/gate.d/74-model-differential.sh`（新，主 agent 写） | 声明 `# gate-covers: 模型对拍`；release 下单跑那个测试二进制，三段都要有「模型对拍 N 步」且 N > 0；样本在 `.claude/gate.d/fixtures/74-model-differential.sh/` |

`crates/singlefs-core` 与 `crates/singlefs-checker` 一行没改（实现员报告第二节；主 agent `git status --short crates/` 现查只有上表那几个文件）。

**实现今天的样子**（主 agent 读 `crates/` 与报告看到的，都是观测；方案按 `crates/` 今天的实现来谈）：

- 模型答的七类问题与各自的条款出处在报告第三节那张表；照代码今天的保守读法写、标「预想」的五处在同一节末（`grep -n '预想' crates/singlefs-harness/src/model.rs`）。
- 每一步三选一：条款要求拒（给理由集合）、做成（给写出的每条根）、区间里成与拒都对。「允许拒」的格：抬 F 时实例表还是 mkfs 那一片（`RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`）与两面容量墙（报告第六节第 1 条、第七节）。
- 模型不记落点：每条写出的根里「每个单元的分配记录每块盘一条、仍分配、分配代 = 写它的那次发布的 txg」一项，单元的落点由胶水从实现交回的 `TransactionOutput::units` 取（报告第四节末）。
- 模型不建崩溃、设备错与 journal 施加：冷启动答「环里 (txg, 实例) 最大的那条根那一版」，可写挂载写的上一个实例那一行 W 恒 0。
- 今天的代码上三段都没有对不上的格：快档 1912 步、第 121 行取样点 1183 步、回退取样点 913 步（主 agent 在工作区 release 下亲跑一次，输出行 `模型对拍 1912 步：该拒而拒 572、区间里拒 34、该成而成 1306；…`，三段同报告第三节的数）。
- 六条攻方变异（`research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv` 的 B1、B2、B3、B5、B6、N2）在副本上都判红（报告第四节）；B2 语义那一格（F_生效 > 0 的候选被拒）快档与第 121 行取样点里都没跑到，新回退取样点 20 个 48 种子窗里 2 个窗是 0 段，另加了固定历史用例、变异表第 147 行点名它。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| V1 | 模型每一类答案与它引的条款原文一致；照代码写、标「预想」的那几处之外，没有别处在照抄实现 | 报告第三节那张表逐行的出处：D16（发布语义） 已定项 1、6、8、9；D23（journal 的角色与格式） 已定项 14、16；D22（单元原子性怎么合成） 已定项 2、7、16；D3（空间分配） 已定项 3、7、11；D4（校验和位置） 已定项 5；D18（块里携带什么信息） 已定项 11、16；D5（快照 / 空间记账机制） 已定项 8；D8（核心索引结构） 已定项 11 |
| V2 | 「允许拒」与容量墙区间只放过条款真没答的格，不遮住实现在别处该成而拒的错 | 里程碑「增补 3」预想的细节第二条；收口表第 39 行 |
| V3 | 胶水把每个错误成员映射到的理由是那个成员的意思，没有把一种错映射成另一种合法的拒 | 同上；`crates/singlefs-core` 各错误成员的文档注释 |
| V4 | 模型与实现不共用代码（D13（验证路线） 已定项 5）；胶水从实现取的那几样（落点、上限的报数）不会让实现的错原样变成模型的答案 | D13（验证路线） 已定项 5 |
| V5 | 六条变异的判红有余量；B2 语义那一格有专门的取样点与固定历史 | 里程碑「增补 3」第 2 件括注那条验收 |
| V6 | 门禁 74 号判的是「三段都跑了、都判过、没对不上」，写回里程碑的文字与代码、产物对得上 | `.claude/singlefs-ai-sop/rules/show-me-test.md`「扫到 0 项也不是通过」；里程碑「增补 3」验收第一条 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| M1 答案对条款 | V1：模型某一类答案与条款原文不一致，或某处照抄了实现而没标预想 | 一个格（一段历史的一步）上，按条款原文该拒而模型答该成，或反过来；或一处答案的算法与 `crates/singlefs-core` 的对应函数逐行同构、而条款没有给这个算法 |
| M2 遮蔽 | V2：「允许拒」与两面墙的区间会不会接走实现该成而拒的错 | 一条对 `crates/singlefs-core` 的变异，让实现在一个条款说该成的格上拒，而那一次拒落在「允许拒」或墙的区间里，三段都不红 |
| M3 映射 | V3：胶水的映射会不会把判错理由的拒放过 | 一条变异让实现拒对了、理由报错了（例：被抛弃的根报成低于 F，或反过来），三段都不红；或一个错误成员映射到的理由与它文档注释的意思不符 |
| M4 共同误解 | V4：从实现取落点与上限报数，会不会让实现的错跟着进模型的答案 | 一条变异让实现写错落点或分配记录，而模型用同一份落点比出「一致」，三段都不红（第 1 件的执行器判定关掉时也不红） |
| M5 余量 | V5：六条变异换几何（种子区间、每段步数、比重）还红不红；B2 语义那一格的余量 | 一个只改规模或比重的取样点上六条之一不红；或 B2 语义那一格在一个合理的取样点上判不出 |
| M6 门禁与写回 | V6：74 号在什么输入上判绿却没对拍；实现员报告的数与产物、代码对不对得上 | 一份能让 74 号判绿、而某一段模型一步没判或对不上被吞的输出；或报告里一个数与复跑对不上 |

**反向接受条款**：M1 打中 ⇒ 改模型那一格（照条款原文），补用例与变异；M2、M3、M4 打中 ⇒ 收窄区间、给错误成员加判别字段或改映射、让模型自己算那一样，补会红的变异；M5 打中 ⇒ 加取样点或调比重；M6 打中 ⇒ 改 74 号或改写回。实现员停下交的五个设计问题（报告第六节）这一轮不判，归收口表与交用户的那一批。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb，要在入库装置上重做才引。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 在仓副本上对 `crates/singlefs-core` 施加变异、造历史，打穿模型与胶水的判定 | M2、M3、M4、M5 | M1、M6 |
| Sonnet 正推（`three-way-forward`） | 逐行核报告第三节那张答案表与条款原文；核「只共享格式常量」与「core 一行没改」；核 74 号与写回 | M1、M6 | 不替攻方找变异 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按事实表逐格算：分配记录树一个节点 812 条的算式、内容上限 32634、抬 F 上限（给定几张根表逐格算 min(每盘最新有效根, 第 4 新的非空有效根)）、F_生效（给定各盘根带的 F 逐格算）；每格同时写条款原文怎么说 | M1 的算术那一半 | M2–M6 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-supp3-item2-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-supp3-item2-code-r1-main-verification.md`）；同时在飞的另两轮 `m2-s1-r1`、`c381-r1` 的全部材料与产出；`research/prompts/e152-*`。实现员报告与产物可读。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
