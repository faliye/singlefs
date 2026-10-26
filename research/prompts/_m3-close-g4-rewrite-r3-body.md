# m3-close-g4-rewrite-r3：两槽全废盘面由谁判、在哪判（推论轮第三轮，最终轮）正文（2026-10-22）

<!-- doc-lint:not-numbers G4 Q3 Y1 Y2 Y3 Y4 V1 V2 V3 V4 W1 W2 W3 W4 -->

## 一、题面

前两轮：
- 第一轮判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`：红的测试走甲；G4 那一臂注释收窄到 mkfs 返回之后；打中一处空档——补写闸放过的两槽全废盘面挂上了就没人判。
- 第二轮判决 `research/prompts/m3-close-g4-rewrite-r2-main-verification.md`：
  - 甲-二 不做；
  - G4-丙 按第一轮的形态站不住，三处：挂在崩溃注入只看得见整写子集；与 G4 拒绝那一臂一件事记两条；判槽自证用核心层的 `parse_slot`，与被判的实现共用同一段算法；
  - 交第三轮：丙-置、丙-择、G4-戊、拒绝那一臂撤与留、条文、攻方报的数。

这一轮是最终轮：只攻前两轮站住的形态与第二轮交来的候选，这一轮里新冒出来的零轮形态不再为它开一轮，判决里标「零轮」交用户或另立一题。

第二轮之后主 agent 现查到一处前两轮的材料都没点名的条文：池级 checker 的 I-8.5（`crates/singlefs-checker/invariants/03-journal.md` 第 12 行）把「单盘全废」排出违例，走旁路报告，理由是「可写挂载会拒」（`crates/singlefs-checker/src/image.rs` 的 `SystemConfigurationRedundancyReport` 文档注释与 `system_configuration_redundancy_report`）。取号补写例外之后，过得了补写闸的单盘全废盘不再被拒，这个理由对这一类盘不成立。崩溃点重放跑池级 checker，旁路报告不进违例通道，所以攻方第二轮在撕裂域量到的单盘全废态崩溃点重放不报（线索，量在副本原型上，第二轮判决第一节第 3 条）。

这一轮要回答四件：
1. 第二轮判决里「已定」的几条站不站得住（甲-二 不做；G4-丙 原形态出局的三处理由）。
2. 两槽全废盘面由谁判、在哪判。候选：
   - 丙-置：G4-丙 改挂成崩溃点重放的观察者，只判 mkfs 返回之后的段（撕裂域在内），判槽自证读池级 checker 的读法；
   - 丙-I（主 agent 起名，零轮）：不另挂观察者，改 I-8.5 本身——mkfs 返回之后的盘面上单盘全废进违例通道，旁路报告只留给 mkfs 中途；
   - 两者的差：判官住在哪一档、能不能分清 mkfs 中途与之后、对已有判定库的单元键动多少、与 C670、Q3、G4 拒绝那一臂各记几条；
   - 附带：丙-择（择不到系统配置也记一条）、G4-戊（第三截只判新增的无槽盘）、G4 拒绝那一臂撤与留（辩方第二轮：E171 装置与变异第 2406–2408 行押在它上面，撤不得；攻方第二轮：撤掉、三行改指新判法）。
3. 未定项清单全不全：除了上面这些，D13（验证路线） 第 72、187 行、D23（journal 的角色与格式） 第 396 行、I-8.5 第 12 行条文各改成什么；补写那一支进不了崩溃测试、运行时看不出闸哪一件没过（第二轮判决记账那一行，欠账表新立）还缺什么。
4. 第二轮攻方报的数能不能核：W1 四组历史合计 172 366 态零发现、MUT-B2 撕裂域 16 个单盘全废态与 1 个两盘全废态、MUT-B1 整写域 8 态与撕裂域 34 态。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区：HEAD 841b3beab 加未提交的改动，含批 14 补丁；快照在 `research/prompts/m3-close-g4-rewrite-r3-snapshot/`）

- 池级 checker 的 I-8.5：`crates/singlefs-checker/src/image.rs` 里判「全池皆废」的那一处用 `SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION`（第 1467 行起）；单盘全废走 `system_configuration_redundancy_report`（第 1492 行），交回 `devices_without_a_self_verified_slot` 与 `devices_whose_writable_mount_would_be_refused`，文档注释写「不进违例通道（I-8.5 对着只读恢复判，这两样只读恢复都挂得上）」。
- 崩溃点重放的观察者：登记在 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 第 134 行的 `OBSERVER_ENTRIES`；观察者跑完之后的池级 checker 在 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 1504 行 `pool_checker_of_the_images_after_the_observers`。观察者入口进判器的单元键（`judging_units_entries.rs` 文件头）。
- 判官 G4 那一臂：`crates/singlefs-harness/src/mount_transparency.rs` 第 3031 行起的判官、第 3066 行 G4 那一臂、第 2419 行复算；判槽自证用核心层的 `SystemConfiguration::parse_slot`（第 2374 行）与 `choose_system_configuration`（第 36 行引入，第 2422 行起调）。
- 补写闸：`crates/singlefs-core/src/mount.rs` 第 5758 行 `devices_without_the_selected_version`、第 5851 行判法。
- 崩溃注入第二、三截：`crates/singlefs-checker-tier/src/crash_injection.rs`，`grep -c Tearable` 为 0。
- 第二轮两条腿的模型：辩方 `research/prompts/m3-close-g4-rewrite-r2-sonnet-model/`（主 agent 复跑 same=yes），攻方 `research/prompts/m3-close-g4-rewrite-r2-opus-model/`（报告末节有复跑命令，主 agent 没复跑）。
- 同时在写、不在快照里的：本会话根环一簇的修红实现员在副本里改 `mount.rs`、`mount_transparency.rs`、`recovery.rs` 等；别的会话下一批要改 `image.rs`、`walk.rs`。腿只按快照里的样子谈。

方案按 `crates/` 今天的实现来谈。

## 三、格

| 格 | 问什么 |
|---|---|
| Y1 | 第二轮已定的几条撑不撑得住：甲-二 不做；G4-丙 原形态出局的三处理由（整写子集、一件事记两条、共用核心层算法）各自成不成立；第一轮的甲（测试盘面另坏一个单元）在丙-置 或 丙-I 之下还要不要 |
| Y2 | 丙-置 与 丙-I 各自：撕裂域里正确实现在 mkfs 返回之后留不留得下单盘全废态（逐类历史，量的写复跑命令）；怎么分清 mkfs 中途与之后；挂在哪、判几次、与 G4 拒绝那一臂、C670、Q3 同一张盘面各记几条；判槽自证读谁的读法；对判定库单元键的影响。丙-择、G4-戊、拒绝那一臂撤与留各给判定 |
| Y3 | 未定项清单：D13 第 72、187 行，D23 第 396 行，I-8.5 第 12 行各写出改后条文候选；还有哪一处说的是同一件事而前两轮没点到（全仓 grep「可写挂载会拒」「单盘全废」「两槽全废」与它们的同义说法，写出命令与命中数） |
| Y4 | 第二轮攻方的数：在它的模型上复跑核上；能换一条不共用代码的路子核的，换路子核一次 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Y1–Y4 | 从 `crates/` 今天的代码与附录条款正推：丙-置 与 丙-I 各自要改哪几处、判出什么，写出条文候选；Y4 用不共用攻方原型的路子（读码推算或自己的小模型）核攻方的数 |
| 云端攻方（Opus） | Y1–Y4 | 攻前两轮站住的形态与这两个候选：找让丙-置 或 丙-I 误报、漏报、重报的盘面与历史；复跑第二轮自己的原型核 Y4 的数；不许重复前两轮攻过的角度（mkfs 末段、C670 与 Q3 拒因位置、甲-二 的开关） |

引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
