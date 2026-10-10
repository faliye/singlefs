# 根环实现第 10z 批第二十四、二十五块：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R C S14 S14b S19 G14 G15 G16 G17 N01 N02 N03 N15 W8 W9 X6 X7 -->

## 一、题面

这一轮攻两块：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第二十四块（core） | 判决 `research/prompts/m3-batch10z-vw-code-r1-main-verification.md` 第二节 K1-2、K1-3、K2、K3 与第三节；用户 2026-10-09 弹窗定「现在补第二十四块再攻一轮」，同日又定「撤掉身份，文档照实写」：要出路先照 `publish_user_change` 那样核盘表次序，不符拒成新成员、不预演；不给报加会话身份，`SessionMoved…` 那一段文档照实写「只比实例代号与 txg，别的会话的报这两个数恰好相同时不拦」；补用例拦住 `walkSkipped`、报里记的字段 8 种改坏、物理块 4096 | `crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第二十五块（checker 档测试） | 同一份判决 K3、K4 两行与第三节第二十五块：两道条件补用例钉住；括号包住的测试模块、卡池 `impl Drop` 与借卡入口取第一处、叫停类型返回类型换写法三种绕法能拦的补到会拦、拦不了的写进名单；合法写法判红的写明故意拒；两份切口口径对齐 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 的测试模块 |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 29 行（回退候选集保留最近 4 个可退到的不同状态）、第 36 行「抬 F 的上限」那一行、第 37 行「根槽这一次读坏」那一行；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（冻结那一道）。
- 用户 2026-10-08 弹窗原话「显式推环命令加带出路的错」；2026-10-09 弹窗原话「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」「现在补第二十四块再攻一轮」「撤掉身份，文档照实写」。

改动全文：`research/prompts/_m3-batch10z-xy-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-x2/patch/report.md`（第二十四块，返工那一份；撤身份之前那一份 `~/.local/share/singlefs/work/m3-r1/batch10z-x/patch/report.md` 只作来历）、`~/.local/share/singlefs/work/m3-r1/batch10z-y/patch/report.md`（第二十五块）。上一轮两腿报告：`research/prompts/m3-batch10z-vw-code-r1-opus-output.md`、`research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md`；上一轮攻方模型 `research/prompts/m3-batch10z-vw-code-r1-opus-model/`（`rerun.sh` 第 50 行变异表那一段读参数的写法在没有换行收尾时退 1，主 agent 补跑用的是 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-main-rerun/rest.sh`），上一轮正推模型 `research/prompts/m3-batch10z-vw-code-r1-sonnet-model/`（扫源码变体在 `scan/`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree/`，取在两块都合入之后、删除线第 50 批合入之前；被判文件 sha256 在 `research/prompts/m3-batch10z-xy-code-r1-snapshot/crates-sha256.txt`（348 份），与合入目录 `~/.local/share/singlefs/work/m3-r1/merge-10z-xy/after.sha256` 逐个相同。
- 第二十四块关键位置（冻结副本 `crates/singlefs-core/src/mounted_session.rs`）：第 215 行 `StuckBehindTheWideProtection`、第 268 行 `WaysOutOfTheWideProtectionRefused`（`SessionMoved…` 文档照实写；第 287 行新成员 `DeviceTableOtherThanTheOneOfTheMount`）、第 1029 行 `publish_user_change` 核盘表次序那一判（没动）、第 1230 行推环入口（没动：核盘表走第 1358 行 `parameters_of_the_pool_agreeing_with_the_caller_inputs`，不看次序）、第 1268 行 `ways_out_of_the_wide_protection`（第 1301 行新加的次序核，在版本判之后、预演之前）、第 1618 行 `#[cfg(test)] mod tests`。`mount.rs` 没动，`refusal_is_short_of_space` 函数体逐字未改。
- 第二十五块：改动都在两份文件的 `#[cfg(test)] mod tests` 里，用例、名单与故意拒的写法在实现员报告里，行号是冻结副本的行号。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-xy/summary.txt`）：merge-and-verify rc=0（编译、harness 档全部测试二进制、fmt、clippy）；mutation-tables 格 rc=0；static-check `CHECKED 3635 322 10`；e158 `95 passed`；e168、E171 装置编过，E171 源文件 sha 前 12 位 ab35105f861d；`raise_floor_takes_…` `--include-ignored` 67 passed、另两份慢档 23 / 16 passed、第九项相关 10 份全绿；checker 档快档 61 份 0 失败；带 gpu 的 lib 与 12 份挂 gpu 的测试全过、NOT_RUN 0；合后副本上 prove-red：第二十五块 17 行（实现员只在抽出来的草稿小程序上证过）`✓ 点名 17 条：跑了 17 条，跳过 0 条，跑的都抓到了`。第二十四块 21 行实现员在副本上 prove-red 证过（core 10、harness 11），合入时锚点与原文没动、不重证。
- 实现员自定、被攻过零轮的：
  - 第二十四块：盘表不符单开一个成员（不并进 `SessionMoved…`），带两份盘表身份；核在版本判之后；四种改动的报逐项比被拒那一次交进去的改动；`walkSkipped` 那一形取正推的 k=4、每块盘 4 个隔离槽、留 30 个没挡的槽（注入级）；
  - 第二十五块：头两行前一个词限 `;`、`}`、`]` 或文件开头，另加「圆括号、方括号也在第 0 层」（超出派发字面，挡 S14b：宏调用圆括号里先放一条以分号收尾的条目再接测试模块）；卡池模块本身、`impl Drop`、四个借卡入口改按词恰好一处、前面不许挂 `#[cfg]` / `#[cfg_attr]`（卡池模块那一处超出派发字面）；两份切口统一按行逐字认头两行（相邻两整行逐字是 `#[cfg(test)]` 与 `mod tests {`，不带多余空白与行尾注释、不认 CRLF）；N03（类型别名）、N15（类型宏）写进名单。
- 主 agent 已收、报出来不算打中的：要出路时会话已不在被拒那一版上拒成 `SessionMoved…`、不读盘；别的会话的报两个数恰好相同时不拦（用户定撤掉身份、文档照实写，记欠账）；第二十五块另有三处「取第一处」写进名单没改（`the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool` 的两个入口、「借之前那一步」、「卡交出自己的设备」）；合法写法里函数体的 `mod`、`#[cfg(any())] extern crate`、头两行换写法、CRLF 判红是故意拒。
- 已知没做、不在这一轮被判范围的：C762、C554；F、H 推完之后那一段在上一轮扫过的范围里分不开。
- 用户已定、报出来不算打中的：tri 的判法本身与它在「不知道」那一支上多护的那一截；显式推环会让回退候选集掉到少于 4 个不同状态；不选乙；预演在要出路时才做、每要一次整串推一遍环的代价；两进程同读一份空闲的竞态；G2 乙；不给报加会话身份。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第二十四块 要出路先核盘表

1. 新加的次序核：上一轮攻方那几族形（`m3-batch10z-vw-code-r1-opus-model/` 里九形 283 格、实现员三形、两槽 C、被抛弃根 A 与 C、窄四档，`table-reversed`、`table-swapped`、`table-one`、`table-stale` 与同会话同盘表的动作）在新实现上逐格对一遍：要出路交回的结局与照出路二真推环再发同一个改动的真结局是否相同？有没有该能预演而被拒成新成员、或该拒而照样预演的形？拒的时候盘一字未读、会话不动吗？
2. 推环入口自己核盘表不看次序（第 1358 行）：调用方交次序对调的盘表直接调推环入口，会发生什么——写落到对的盘上没有、之后的发布怎么判、回退候选集与 C 那一圈对不对？推环入口要不要也先比次序，条款没写；把两种做法在同一组形上的结局都给出来。
3. `SessionMoved…` 照实写的那一段文档：「别的会话的报这两个数恰好相同时不拦，照这个会话的状态预演，出路一的文字仍是报里的数，列出路二时其中的 R 那一槽与要推的次数也是报里的」与代码逐句对不对得上？

### K2　判别力

第二十四块追加 21 行、第二十五块追加 17 行：能不能拦住改回去？上一轮两腿活下来的变异（攻方 `walkSkipped`；正推 K3-3 的 17 个里记进报里的字段 8 个、`checkRequestBlock512`、`walkSkipped`；第二十三块 `W8`、`W9`、`X6`、`X7`）现在红不红？`walkSkipped` 那条注入级用例钉的是不是条款说的那件事？还有哪几种改坏的写法照绿？

### K3　第二十五块 扫源码（只核补法）

扫源码这一格已是第三次代码轮之后的补法，**不开新的攻击面**：只核上一轮点名的写法（`S14`、`S14b`、`S19`、`G14`、`G16`、`N01`、`N02`）在新判法上判红、对照（`G15`、`G17`、原样源码）照旧，合法写法判红的是不是都写明了故意拒，名单列的拦不住的写法（`N03`、`N15`、三处「取第一处」）是不是名单里都有；新加的「圆括号、方括号也在第 0 层」与「卡池模块恰好一处」有没有把合法的产品写法判红或 panic。

### K4　E171 走的路径

形式化线要在「全部合完」之后把那一刻主工作区的 `crates/` 冻结成一份快照、重编 E171（三 oracle 合谋测试）装置：这两块改动有没有改变挂载路径（`crates/singlefs-core/src/mount.rs` 第 33 行引 `mounted_session::refusal_is_short_of_space`）或发布路径上任何不报宽护的形的行为？有没有一处测试之外的改动不在第二十四块题面里？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G；编出来的测试二进制也经它跑，不直接执行。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
