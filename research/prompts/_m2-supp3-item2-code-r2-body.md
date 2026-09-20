# 背景材料：增补 3 第 2 件（理想模型与对拍）的代码三方对抗第二轮（2026-09-19）

<!-- doc-lint:not-numbers M2 M3 W1 R1 Z1 Z2 Z3 Z4 -->

## 一、被判的对象

第一轮判决 `research/prompts/m2-supp3-item2-code-r1-main-verification.md` 第三节第 1、2、4 条的改法（第 3 条是门禁 74 号，主 agent 改）。实现员报告 `research/prompts/m2-supp3-item2-fix-implementer-report.md`，日志与脚本在 `research/prompts/m2-supp3-item2-fix-implementer/`。被判的改动是附录二那份 diff：`research/prompts/_m2-supp3-item2-code-r2-diff.md`（基准是实现员开工前的仓副本，它在 `crates/` 下的六个文件与第一轮开工快照逐个同 sha256）。**开工快照**：`research/prompts/m2-supp3-item2-code-r2-start-snapshot.sha256`。

| 文件 | 项 |
|---|---|
| `crates/singlefs-checker/src/walk.rs` | 新函数 `allocation_record_count_under_root`：按 checker 自己的字段表数一条根下的分配记录 |
| `crates/singlefs-core/src/mount.rs` | `MountError::RollbackTargetNotACandidate { target, exclusion: RollbackCandidateExclusion }`，四个值：不在环里、低于 F_生效、被抛弃、树表 0 条；原来单列的 `RollbackTargetNotInRing`、`RollbackToVersionWithoutFileUnsupported` 两个成员删掉、并进这个字段 |
| `crates/singlefs-core/src/transaction.rs` | `PublishError::NoSpaceFor` 换成 `PlacementRefused { unit, refusal: PlacementRefusal }`，分配器的拒绝原因原样带上 |
| `crates/singlefs-core/src/allocator.rs` | 分配器交出拒绝原因，不再用 `.ok()` 丢掉 |
| `crates/singlefs-harness/src/model.rs` | 分配记录墙：上界那一头照旧，另外要求「镜像上数出的条数 + 按准入口径新增的条数 > 812」才放行，数不出条数不放行；模块文档写明模型不记落点、落点与分配记录归池级 checker 判 |
| `crates/singlefs-harness/src/model_comparison.rs` | 胶水按字段一对一映射；只有「每块盘上都没有空槽」映射成单元区墙，另三种分配器原因映射成模型里没有的理由（出现即判对不上）；「两条之一」那条路删掉 |
| `crates/singlefs-harness/src/history.rs` | 墙拒时在镜像上数模型点名的那一版（发布、抬 F 数拒之前最后写出的那一版；可写挂载、回退数这一步的起点）；新执行方式 `PerStepChecker::Skipped`（每一步不跑池级 checker） |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` | 新取样点「逼近分配记录墙」（种子 [0, 32) × 150 步，不跑 checker）与新用例；另四个测试文件改断言跟错误成员走 |
| `crates/mutations.tsv` | 第 156–164 行，九条变异 |
| `.claude/gate.d/74-model-differential.sh`（主 agent 改） | 两类缺口列完再退出；查的段从三段加到四段（新取样点那一段） |

**实现今天的样子**（主 agent 读 `crates/`、报告与日志看到的，都是观测；方案按 `crates/` 今天的实现来谈）：

- 主 agent 在工作区 release 下跑门禁 74 号（2026-09-19 15:3x UTC），四段都判过：快档 1912 步、复用取样点 1183 步、回退取样点 913 步，与改之前逐数相同；新取样点 4001 步（该拒而拒 504、区间里拒 335、该成而成 3162），「分配记录墙按镜像上的真条数放行 326 次」。
- 实现员报告的判出段数：W1（墙的 `>` 写成 `>=`）在进门禁的新取样点里 9 / 32 段、攻方长历史设置（96 段 × 200 步，broad / reuse 两组比重）12 / 33 段、基线新发现 0；R1（被抛弃的根报成低于 F）在回退取样点 12 / 48 段、快档 4 / 96 段。九条新变异在副本上逐条施加、点名的测试都红（`research/prompts/m2-supp3-item2-fix-implementer/logs/proof-*.log`）。门禁 59 号整表复跑还没做。
- 实现员报的四件要主 agent 定的事（报告「要你定的」）：① 「树表 0 条」放进了 `RollbackTargetNotACandidate` 的字段，而它按 D16（发布语义） 已定项 1 可以在候选集里、只是第一版不支持；② `NoSpaceFor` 没拆成三个平级成员，换成了一个带原因的成员；③「真条数」取准入口径（镜像上的条数 + 每个重写的角色每盘一条），数不出一律不放行；④ 新取样点不跑 checker（跑的话根环转一圈之后历史停在已知红第 0 条，走不到 812 条），它看不见只有 checker 判得出的问题。④ 主 agent 已定：把那一段接进 74 号。
- kb 里还写着旧成员名的（`grep -n "RollbackTargetNotInRing\|RollbackToVersionWithoutFileUnsupported\|NoSpaceFor" .claude/kb/**/*.md` 现查）：`.claude/kb/milestone/02-second-txn.md` 第 117、141、166、329、330 行，`.claude/kb/checks-owed.md` C368（分配器落点只看盘 0，盘不等大时断言失败） 那一行；`crates/` 里测试注释一处（`second_transaction_supplement_two_commit_generated_fallback.rs` 第 215 行，说的是补回落之前的事）。模型自己的理由名 `ModelRefusalReason::RollbackTargetNotInRing` 还在，那是模型的理由、不是 core 的成员。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| Z1 | 墙拒时拿镜像上数出的条数判：数出的条数 + 准入口径的新增 ≤ 812 而实现拒了，算对不上；数的是模型点名的那一版；数不出不放行 | D3（空间分配） 已定项 11；里程碑收口表第 39 行（准入保留上界）；第一轮判决第三节第 1 条 |
| Z2 | 回退的拒绝按四种排除原因一对一映射；「树表 0 条」归进候选排除 | D16（发布语义） 已定项 1（回退候选集）；D23（journal 的角色与格式） 已定项 14（回退候选集那一句）；第一轮判决第三节第 2 条 |
| Z3 | 发布的落点拒绝带原因，只有「每块盘都没有空槽」映射成单元区墙，另三种算对不上 | D3（空间分配） 已定项 8、已定项 10；D2（RAID 条带策略） 已定项 2、已定项 10；收口表第 20、20′ 行 |
| Z4 | 新取样点不跑池级 checker，由模型与执行器判，门禁 74 号查它；原来三段的对拍计数不变 | `.claude/singlefs-ai-sop/rules/show-me-test.md`「扫到 0 项也不是通过」；里程碑「增补 3」验收第一条 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| M1 墙的误红与漏判 | Z1 在合法历史上会不会误红（数出的条数与实现准入用的量口径不一，例如复用已回收记录之后真写的条数更少、或点名的版本取错），会不会漏掉「差一」以外的误拒（例如准入把某个角色算两次） | 一段合法历史在基线代码上被判对不上；或一条让实现在 ≤ 812 时误拒的 core 变异（不是 W1 那一条）四段都不红 |
| M2 映射 | Z2、Z3 的映射会不会把一种拒放成另一种合法的拒，或在合法历史上判对不上；「树表 0 条」算候选排除是不是条款的意思 | 一条只换排除原因或拒绝原因的变异四段都不红；或基线在某种几何（盘不等大、小盘写满）上误红；或条款原文把树表 0 条的根算进候选集而实现报「不是候选」 |
| M3 改法与报告对得上 | 报告里的数（8 条新测试、9 条变异、各设置的判出段数、三段计数不变）与日志、代码对不对得上；所有调用方的 `match` 有没有写 `_ =>`；kb 里旧成员名还有几处 | 一个数与日志对不上；一处 `_ =>`；kb 里一处旧名没列进写回 |
| M4 跳过 checker 的那一段 | 新取样点不跑 checker，会不会让某类只有 checker 判得出的错在门禁里没人看 | 一条变异只在新取样点那一类历史上走得到、而 checker 判得出、模型判不出，四段都不红 |

**反向接受条款**：M1、M2、M4 任一格被打中，按那一格的改法改代码，改完走 `crash-verifier`（59 号整表、层 0），不再开第三轮（第二轮之后停，打中的写进交用户表或另立一题）；M3 打中的由主 agent 在写回时改。
**失败条款**：腿在副本装置上量出的数不进 kb；「没打中」一次不算，本地腿抽两次；腿引的条款与附录原文不符，那一格作废。

## 四、腿的分工（三条推论腿，攻击面不重叠，也不重复第一轮攻过的角度）

第一轮攻过的：墙区间只有上端（W1）、按文字映射（R1）、模型不记落点（M4 旧那一格）、六条攻方变异的余量、74 号的两支样本。这一轮不重攻它们本身，只攻改法。

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 在仓副本上对 `crates/singlefs-core` 与胶水施加变异、造历史（含盘不等大、小盘写满、回退到树表 0 条的根），打穿 Z1–Z4 | M1、M2、M4 | M3 |
| Sonnet 正推（`three-way-forward`） | 逐项核改法做的是不是第一轮判决第三节说的；核报告的数与日志、代码；核 `match` 没有通配臂；列 kb 里要跟着改的旧成员名；核「树表 0 条」按条款原文是不是候选排除 | M3、M2 里「树表 0 条」那一格 | 不替攻方造历史 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按写死的事实表逐格算：给定几组「镜像上的条数、这次重写的角色数、盘数、是否复用已回收记录」，按准入口径算放不放行，与 812 比；给定四种排除原因与四种落点拒绝原因的定义句，逐格填映射到模型的哪一种理由 | M1 的算术、M2 的映射表 | 其余 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-supp3-item2-code-r2-*-output*.md`）与主 agent 的核实（`research/prompts/m2-supp3-item2-code-r2-main-verification.md`）；同时在飞的别的轮次（`research/prompts/_c381-*`、`research/prompts/c381-*`、`research/prompts/_m2-s1-*`、`research/prompts/m2-s1-*`）；`research/prompts/e152-*`、`research/prompts/e155-*`。第一轮的材料、产出与判决可读；实现员报告与日志可读。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
