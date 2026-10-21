# m3-close-c693-r1：K 代撤回之后的四块重推（推论轮第一轮）正文（2026-10-20）

<!-- doc-lint:not-numbers W1 W2 W3 W4 -->

## 一、题面

欠账 C693（K 代撤回之后要重推的论证）。D5（快照 / 空间记账机制） 已定项 2 已从「记账树只保留最近 K 代」改成「每条根的记账树只装这条根那一代的行，不留多代」。摸底报告 `~/.local/share/singlefs/work/m3-close-w8-c693-survey/report.md`（逐句表 ①–④ 与第 3 项）列了要重推的四块；问题单 `research/prompts/c693-forks.md`。三份实验已跑完：

| 实验 | 回答 | 产物 | 结论（实验页） |
|---|---|---|---|
| E186（不认识的树·记账维重推） | 问题单 Q1、Q2 | `research/results/e186-unknown-tree-accounting-rederive.out.gz` | `.claude/kb/experiments/186-不认识的树-记账维重推.md` |
| E71（记账 key 的条目数与宽度）第一次重跑 | Q3 E71 那一半 | `research/results/e71-accounting-keys-single-generation-2026-10-20.out` | `.claude/kb/experiments/71-记账key的条目数与宽度.md` 2026-10-20 一节 |
| E97（记账与分配记录的条目编码）第五次重跑 | Q3 E97 那一半 | `research/results/e97-entry-encoding-single-generation-2026-10-20.out` | `.claude/kb/experiments/97-记账与分配记录的条目编码.md` 2026-10-20 一节 |

这一轮判：三份实验的读数对 D8（核心索引结构） 已定项 8 ㊀ ㊁、D5 已定项 4 与已定项 3 第 3 行意味着什么，哪几句要改、改成什么。用户 2026-10-20 定 ③ 块走「另立实验重跑」（E186），实验之后的推论照规矩走三方。

## 二、实现今天的样子（主 agent 的观测，2026-10-20，主工作区）

- 每次发布写这一代的行：`crates/singlefs-core/src/transaction.rs` 的 `fn accounting_entries_of_this_publish`（池级三行 + 每盘六行 + 照抄上一版台阶行），没有删除行或墓碑行（E71、E97、E186 三位执行员停机条款都现读过）。
- 记账树整棵重写：`crates/singlefs-core/src/code_two_tree.rs` 的 `plan_the_tree_after_this_publish`（旧 key 删、新 key 插）。
- 未知 flags 位：`crates/singlefs-core/src/recovery.rs:2804` 让整张树表读不下去（`EntryMalformed`），D8 已定项 8 ㊁ 写的是「该条目 EIO（对象级，不拒整个池）」（E186 设计员报的出入）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### W1　D8 已定项 8 ㊀（旧写者遇到不认识的树，记账行怎么处置）
E186 Q1：合法点 `r_tree_dim = 1`、`h_new > 0` 的 3200 点 `Carry` 一次都不是胜者（`lpc = 0` 胜者 `NoCarry`，`lpc = 1` `NoCarry` 与随根留着并列）；`r_tree_dim = 2` 的 3200 点胜者 `Carry`；`h_new = 0` 三条并列（产物 `check=q1_winners` 那几行与实验页）。㊀ 要不要改、改成哪一条或按 `r_tree_dim` 分支（分支照 `.claude/rules/fs-design.md`「分支的三条约束」与「五条硬要求」判能不能分）；改了要动的条款（D8:261、D8:383、D5 已定项 2 与 I-3.13 的例外）。

### W2　D8 已定项 8 ㊁（未知 flags 位）
E186 Q2：读「计」时 `w_gens = 0` 的点 flags 胜者由 `Skip`、`RejectEntry` 并列换成 `Skip`，`w_gens > 0` 两读法胜者都是 `RejectEntry`；`RejectEntry` 对 `Skip` 的判别力自证由绿转红。㊁ 要不要改；`recovery.rs:2804` 整表拒与 ㊁「该条目 EIO」的出入谁改。

### W3　D5 已定项 4 的依据与「树高恒为 2」
E71 判决格（上界臂、30 字节、1024 × 64）新口径 17 694 720 字节不过界（旧口径过界）；E97 宽读 t=1024/d=64 42%、t=8192/d=8 47% 都不过界，窄读最紧 53% → 2%。D5 已定项 4 依据两句（D5:145、D5:146，另有把 E97 的数记在 E71 名下、宽读窄读写反两处出入）失去支撑之后，已定项 4「哪几项按树分」的取舍还站不站；D5 已定项 1、已定项 5「树高恒为 2」在一代口径下变成窄读 1、宽读 2，怎么改。

### W4　① 块：点删与墓碑那一族
摸底报告 ① 一节（A1–A14）：写路径每次发布写当前代、从不点删，D5 已定项 3 第 3 行「点删的墓碑不参加取最大」与 C344 的前提还在不在。摸底报告列的两个待答问题（开 write buffer 前端之后整批换代是重建整棵记账树还是写新一代点删上一代；删一个可写头时它的台阶行怎么离开记账树）在 `crates/` 今天的实现上答，答案决定 D5 已定项 3 第 3 行与 C344 保留、缩题还是销账。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W4 | 从三份实验的产物与 `crates/` 今天的实现出发，推每一块该怎么改，逐句写出改后的条文与要动的条款 |
| 云端攻方（Opus） | W1–W4 | 攻「实验读数能推出条款要改」这一步：取样点与几何是不是撑得住、反向接受条款的字面是不是被误读、改了之后哪一格会错；攻 W4 的两个答案 |
