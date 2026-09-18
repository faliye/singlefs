<!-- knowledge-sync -->
# m2-wave1 阶段同步

里程碑二收口第一波：清单重建、增补 1 开工（第 1、3 件与 E155 四段）、代码第一波四批改动、代码三方第一轮与三处改法。回扫由 `sweep` 做（2026-09-18），逐处判决与写回由主 agent。

触发文件（这一阶段改动范围里的，逐个按路径写全）：.claude/agent-common.md、.claude/gate.d/66-abandoned-rounds.sh、.claude/gate.d/67-milestone-closeout-owed.sh、.claude/gate.d/stage-owners.tsv、.claude/rules/implementation-workflow.md、crates/singlefs-core/src/write_accounting.rs、crates/singlefs-core/src/transaction.rs、crates/singlefs-core/src/allocator.rs、crates/singlefs-core/src/mount.rs、crates/singlefs-core/src/recovery.rs、crates/singlefs-core/src/lib.rs、crates/singlefs-harness/src/crash.rs、crates/singlefs-harness/src/scenario.rs、crates/singlefs-harness/src/bin/first_transaction_on_device.rs、crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs、research/scripts/replay.sh

## 搜索

关键词 25 个，每个在现状句的载体里搜一遍（载体清单 `.claude/agent-common.md`、`.claude/agents/`、`.claude/rules/`、`.claude/kb/`、`CLAUDE.md`、`README.md`、`records/` 正文；不搜 `research/prompts/` 与 `briefs/`）。命令形态 `grep -rn "<关键词>" <载体清单> | wc -l`，逐个的计数：

agent-common.md → 45；66-abandoned-rounds → 2；67-milestone-closeout-owed → 2；stage-owners.tsv → 8；implementation-workflow.md → 9；02-second-txn.md → 22；write_accounting.rs → 2；transaction.rs → 16；allocator.rs → 4；mount.rs → 15；recovery.rs → 17；crash.rs → 2；scenario.rs → 0；first_transaction_on_device.rs → 0；first_transaction_device_log_check.rs → 0；mutations.tsv → 18；abandoned-rounds.tsv → 3；e155_fsync_write_volume → 2；E155 → 9；m2-wave1 → 5；c364-r3 → 3；runner-dispatch-guard.sh → 5；agent-watch.py → 13；cache-keepalive.sh → 4；omitClaudeMd → 47。

进度词（没做、还没、没试跑、没用过、没拿真活、没核过、没测、没有读数、只试跑、只做了一轮、待、欠、开着、计划、下一步）与关键词同句的命中共 30 处，分五类的结果在下表。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/agent-common.md:4 | 计划与来历在 `records/2026-09-16-subagent拆分提案.md`。这些定义 2026-09-17 写成；试跑、对抗与真活上的使用到哪一步，只记在那份计划里，不在这里写第二份。 | 不改：另一个会话（singlefs-d3）在本阶段结束前已经把「没拿真活用过」那半句改成指到计划，现句成立 |
| records/2026-09-16-subagent拆分提案.md:542 | **第三批没做的**：都已了结。 | 不改：同一个会话已改完，看门狗与续派闸都已在真派发里用过 |
| records/2026-09-16-subagent拆分提案.md:583 | （「## 历史版本」之前） | 补了：新增第二十二节「m2-wave1 阶段的实测」——18 个子 agent、1801 次调用、新输入 660.4 万、整份重写 0 次；起步上下文 6.5k–1.3 万（开 `omitClaudeMd`）对 12.5–12.9 万（内置 `general-purpose`）；`cache-keepalive.sh` 只有崩溃验证员真起过、两轮之后没再续、出现 17 分钟无调用 |
| .claude/kb/milestone/02-second-txn.md:64 | 还没做的（2026-09-17 核，固定脚本已到 E）：预置基镜像、只供测试的开关里的三个…… | 改了：三个开关逐个写清今天的样子（复用窗口那条必红用例是直接调回收造的、没有具名开关；根槽读失败开关全仓零命中；一条记录装几个事务归并行线一） |
| .claude/kb/milestone/02-second-txn.md:308 | （第 19–34 行各行的去向列） | 改了：第一波四批改动、代码三方第一轮四处打中与三处改法、E155 四行够判逐行写回；新增第 20a–20d、23′、23″、34′ 各行 |
| .claude/rules/implementation-workflow.md:16 | （原文没有开工快照这一条） | 补了：新增「代码轮派腿之前记一份开工快照」一节，依据是本轮核查员自陈「靠 mtime + git diff 补救纯属运气」 |
| .claude/kb/checks-owed.md:341 | C366 现状「2026-09-17 现查工作区：通用发布路径已改成 `previous.record.counter + 1`，暖机路径仍写 txg（别的会话未收尾）」 | 改了：暖机路径已改成按记录号接着数（`warm_up_after_journal_counter`），挂载层今天仍分不出两个量 |
| .claude/kb/checks-owed.md:343 | C368 现状与前置 | 改了：三处落点改成按盘各答一个、一致才分配，用例与变异已有；仍欠发布层把三种拒绝统一报成 `NoSpaceFor` |
| .claude/kb/checks-owed.md:344 | C369 现状与前置 | 改了：回落已实现（`lowest_commit_generated_fallback_slot`），C146 第 ② 条随之还清一半 |
| .claude/kb/checks-owed.md:353 | （原文没有这两笔） | 补了：C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）、C379（记账的全空聚簇段数与分配器能开的段两个口径） |
| .claude/kb/checks-owed.md:173 | C146 那一行「② 那条政策函数有了定义，会红的检查仍欠」 | 改了：② 的检查 2026-09-17 已有（回落实现 + 四条用例 + 变异），① ③ 仍开着 |
| .claude/kb/checks-owed.md:227 | （原文只记第一、二轮判决） | 补了：各加一句「第三轮发了腿没收尾，登记在 `research/prompts/abandoned-rounds.tsv`」 |
| records/2026-09-17-已分配口径三方与两个实验.md:162 | 1e 行「自检接进门禁 63 号；实派一次被当场拒绝」 | 改了：补上 2026-09-18 续派闸第一次在真派发里放行（派 E155（每次持久化的写量：三种 fsync 形态与反事实上界） 执行员） |
| .claude/kb/verification-build.md:76 | 运行时计数那一行写 `fallback_policy_mismatches` | 改了：代码里叫 `policy_mismatches`（allocator.rs 第 494 行），且比较那一步已删、恒 0，口径要改 |
| .claude/kb/verification-build.md:77 | （原文没有按结构种类的写量这一行） | 补了：每次发布按 12 种结构的写调用与写字节（`write_accounting.rs`），欠失败路径那一半 |
| .claude/kb/milestone/01-first-txn.md:144 | C196（中央映射树的根住哪全仓无条款） 那一行 | 不改：这一阶段没碰它的条款与检查，撂下登记表只记轮次去向，与这一行的「条款已写、检查仍欠」不冲突 |
| .claude/kb/prior-art.md:19 | （三家 fsync 的旧说法） | 不改：这一阶段的调研报告存在 `research/prompts/m2-s1-prior-art-report.md`，要不要并进 kb 等增补 1 第 4 件定案（回扫列为「分不清」，主 agent 判为不改） |
| .claude/rules/implementation-first.md:26 | 「实现今天的样子」那一节 | 不改：新模块 `write_accounting.rs` 属于「读过的 `crates/` 路径」那一类，规则本身不点名具体模块 |
| .claude/kb/tooling.md:12 | 本地腿的说法 | 不改：这一阶段本地腿两份样本一次判绿，与那一节记的口径一致，没有新读数要改 |
| records/2026-09-17-已分配口径三方与两个实验.md:167 | 子 agent 与脚本的定时监控「已做」 | 不改：那一行说的是看门狗与 hook 做成了，本阶段实测的是 `cache-keepalive.sh` 的续期行为，补在第二十二节，不改那一行 |
| research/scripts/replay.sh:1 | （E155（每次持久化的写量：三种 fsync 形态与反事实上界） 的复跑登记行） | 补了：执行员登记 E155（每次持久化的写量：三种 fsync 形态与反事实上界） 行并指到 stage4 产物 |
| .claude/agents/crash-verifier.md:23 | （定义里没写计时器） | 不改：共用约束「不做」一节已经写了每次结束本轮去等之前起一次，定义里不写第二份；实测「只续两轮」写进计划第二十二节 |
