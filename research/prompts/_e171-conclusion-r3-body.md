# E171（三oracle合谋测试） 的结论撑不撑 D13（验证路线） 已定项 6：第三轮正文（2026-10-03，最终轮）

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 K1 K2 K3 K4 R1 R2 R3 R4 R5 R6 M4 W -->

## 一、题面

前两轮判决：`research/prompts/e171-conclusion-r1-main-verification.md`、`research/prompts/e171-conclusion-r2-main-verification.md`（正文 `research/prompts/_e171-conclusion-r1-body.md`、`research/prompts/_e171-conclusion-r2-body.md`，四份腿报告 `research/prompts/e171-conclusion-r1-sonnet-output.md`、`research/prompts/e171-conclusion-r1-opus-output.md`、`research/prompts/e171-conclusion-r2-sonnet-output.md`、`research/prompts/e171-conclusion-r2-opus-output.md`）。第三轮是最终轮：只攻第二轮判决第三节的形态（真盲区候选名单与另几类的归属、D13（验证路线） 已定项 6 的五条写回形态、C407（三 oracle 合谋测试没有代码） 的改写），不重复前两轮攻过的角度（逐行三分的条款读法、O3 的 A(c) 来源、坏实现表的来源偏向、写序口径与已定项 12、O3 独有的那一类与 RefFS、自崩行）；这一轮新冒出的零轮形态不再开轮，写进判决的交用户表。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- E171 的行级读数都是 s4 冻结树 `/tmp/claude-1000/e171-s4/frozen-tree/` 上的（快照摘要 d1586465…）；今天仓里的 `crates/` 已打进删除批，与它 83 个文件不同（第二轮攻方现数）。冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/` 是第一轮开工那一刻的仓（清单 `research/prompts/e171-conclusion-r1-snapshot/crates-sha256.txt`）。代码引文写这两棵之一的路径与行号，写明是哪一棵。
- E171 产物 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`、导出目录 `research/target/e171-dumps/`、实验页 `.claude/kb/experiments/171-三oracle合谋测试.md`。
- 崩溃注入第二截判拒可写合不合法的是 `crates/singlefs-harness/src/mount_transparency.rs` 的 `judge_writable_mount_refusal`，调用点在 `crates/singlefs-checker-tier/src/crash_injection.rs`。
- 方案按 `crates/` 今天的实现来谈；E171 的读数按 s4 树谈。

## 三、格

### K1　写回句与读数、原文对不对得上
第二轮判决第三节 D13 已定项 6 的五条写回形态与 C407 改写：每一句说的是不是 E171 读数（s4 树）与 kb 原文撑得住的那么多，有没有写宽、写窄、放错格（「判什么」「射程」「验不到什么」「依据」「欠」各管什么，照 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针」）；「依据」引实验的那几句与实验页「影响的决策」的关系（支撑 / 推翻 / 备料）对不对得上。

### K2　真盲区候选名单完整不完整
第二轮之后的名单：15、23、42、242、269、270、271；另几类：27、112、277（三条不判实现恢复里靠记录重建的那一段）、34（分配政策）、92（决策正文欠一句）。只靠一条的 23 行、未触达的 238 行、自崩的第 125 行里，有没有按前两轮的口径该进名单而没进的；名单里有没有按同一口径该出去的。

### K3　C407 改写的判官骗不骗得过
「读已入库产物：全放行集合 = 登记集合、每列至少一样只有它红（O3 用 Q2 的 O3 独有坏镜像）」：一个坏掉的 oracle（永远答「在」或永远答「红」）、一份过期的产物、一次少跑了行的整表，能不能让它判绿；它与 `research/scripts/replay.sh` 的 E171 复跑行、与门禁 checker-tier-formal-verification 的红例钉表在什么地方重叠。

### K4　第二轮报出的数能不能核
第二轮判决引的数（W 的 15 / 63 态、61 个见证拒、第 125 行 D = 38、O3 独有 29 张、RefFS 27 / 27 与 36 态），出处在哪、用的哪棵树、能不能在入库装置上重做、写进 kb 要标什么。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K4 | 逐句把写回形态对到产物整行与 kb 原文，给出每一句该怎么写才不宽不窄，写成可直接写回的文字（标零轮） |
| 云端攻方（Opus） | K2、K3，兼 K1 | 攻名单完整性与判官的判别力：造一个让判官判绿的坏输入、找一行该进名单而没进的；不重复前两轮攻过的角度 |

## 五、跑前写死的判据

- 第三轮之后不再开轮。两腿同判且主 agent 现查坐实 ⇒ 按那一判写回；打中而坐实 ⇒ 写回时改；新冒出的形态是零轮，写进交用户表（登记欠账或另立一题），不写回 kb 正文。
- 前两轮站住、第三轮没被打中的形态写回 D13（验证路线） 已定项 6 与 C407（三 oracle 合谋测试没有代码）。

交付：腿的报告写 `research/prompts/e171-conclusion-r3-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/e171-conclusion-r3-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
