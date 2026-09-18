# 核对表：m2-step45-code-r2-local-defense 英文提示逐句核转述

机械对照，逐行列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。

| 英文项（提示里出现的句子，概括标注） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Candidate set definition (选择性、T ≤ Ti、txg ≥ F_effective) | `.claude/kb/decisions/23-journal的角色与格式.md:1206`；`.claude/kb/decisions/16-发布语义.md:358` | 无（一次写对：两个合取条件、"无 i 的行，或有行…T ≤ Ti" 都保留） | 见提示 BACKGROUND FACTS 第 4 段 |
| F_effective 定义（per-device max 再取 min） | `.claude/kb/decisions/16-发布语义.md:358`（"生效"那一行） | 无 | 见提示 BACKGROUND FACTS 第 5 段 |
| 影子账机制：每次挂载算、读被抛弃根自己的账、跳过 is_released、隔离位独立且不清零 | `crates/singlefs-core/src/mount.rs:186-252`（代码与注释）；`crates/singlefs-core/src/allocator.rs:185-202`（isolate 函数与注释） | 无 | 见提示 BACKGROUND FACTS 第 6 段 |
| D23 主句逐字（"被抛弃时间线的根离开根环之前……只隔离其中只被被抛弃根引用的槽"） | `.claude/kb/decisions/23-journal的角色与格式.md:1206` | 无（"只隔离其中只被被抛弃根引用的槽" 的"只被…引用"限定词保留） | 见提示 BACKGROUND FACTS 第 7 段第 1 句 |
| 2026-09-16 窄读法定案（"仍被有效根引用的槽不在其内"） | 同上（milestone 引用同一句，`.claude/kb/milestone/02-second-txn.md:158-186` 内嵌引述） | 无 | 见提示 BACKGROUND FACTS 第 7 段第 2 句 |
| 保守读法与窄读法字面不一致、预想偏离、交 alloc-basis 判 | `.claude/kb/milestone/02-second-txn.md:158-186`（"与用户 2026-09-16 定的窄读法措辞……字面不一致——预想、偏离用户定案的措辞，alloc-basis 那一轮当一条臂判"） | 首稿漏译"当一条臂判"这半句 | 定稿补上"pending a separate decision to confirm or correct it"，并明写"You should treat this as an honest, already-disclosed fact" |
| P1 / P2 两种续接方式的定义 | `crates/singlefs-core/src/mount.rs:597`（P1，doc 注释）、`:658`（P2，行内注释） | 无 | 见提示 ITEM 1 第 1 段 |
| P2 优于 P1 的理由（B 的记录已提交、根未持久时的窗口冲突） | `crates/singlefs-core/src/mount.rs:658-660`；判决 `research/prompts/m2-step45-code-r1-main-verification.md` 第四节 X2 行 | 无 | 见提示 ITEM 1 第 2 段 |
| C340（回退之后记录链从哪条之后接没有定义） | `.claude/kb/checks-owed.md:311`（"D23 已定项 14 回退段……只写『不施加 R_old 之后的任何记录』，切换段……写『链从所选根覆盖的最后一条记录之后接』；回退之后新实例的第一条记录接在哪……要从切换那一段借『所选根』推出来，回退自己那一段没有一句"） | 首稿把"要从切换那一段借『所选根』推出来"简化掉 | 定稿明写"does not say... whose prefix that phrase refers to: R_old's own prefix, or the whole ring's prefix"，并加"Do not treat this open item as settled" |
| mount_rollback 顶部 doc 注释仍写 P1、与函数体的 P2 不一致 | `crates/singlefs-core/src/mount.rs:597` 对照 `:658` | 无（主 agent 现读代码发现，非转述自 kb） | 见提示 ITEM 1 第 4 段 |
| X1 攻击问题整句 | `research/prompts/_m2-step45-code-r2-body.md:35` | 首稿把"这是 C332 那一族还是新的"简化成"is this a new problem"，漏译具体编号 | 定稿改写为"the same family of problem as a previously tracked issue about rollback being undone when both of a rollback instance's roots are unreadable"，保留原意但去掉裸编号（本地模型读不到 kb，编号无意义，改写为可理解的完整描述，语义未删减） |
| X2 攻击问题整句 | `research/prompts/_m2-step45-code-r2-body.md:36` | 无 | 见提示 ITEM 2 攻击问题 |
| shadow ledger 每次挂载算、此前只在回退那一次算 | `crates/singlefs-core/src/mount.rs:188`（"影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）"） | 无 | 见提示 ITEM 2 第 1 段最后一句 |
| X3 攻击问题整句 | `research/prompts/_m2-step45-code-r2-body.md:37` | 无 | 见提示 ITEM 3 攻击问题 |
| F_生效 候选集判据 D16 已定项 1 表格行 | `.claude/kb/decisions/16-发布语义.md:358`（回退候选集、生效两行） | 无 | 见提示 ITEM 3 第 1 段 |
| X4 攻击问题整句 | `research/prompts/_m2-step45-code-r2-body.md:38` | 无 | 见提示 ITEM 4 攻击问题 |
| reclaim_floor 两处 oldest_valid_root 来源不同（重建用最新根表、抬 F 用 current 自己的表） | `crates/singlefs-core/src/mount.rs:221-233`（重建）；`:357-366` 与 `:333`（抬 F，current 的表） | 无 | 见提示 ITEM 4 第 1 段 |
| X5 攻击问题整句 | `research/prompts/_m2-step45-code-r2-body.md:39` | 无 | 见提示 ITEM 5 攻击问题 |
| rebuild_from_records 认第 0 版树表的机制与理由（跨进程） | `crates/singlefs-core/src/allocator.rs:353-356`（doc）、`:376-389`（代码）；`crates/singlefs-core/src/allocator.rs:725`（单测名 `rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released`，未直接引用但机制一致） | 无 | 见提示 ITEM 5 第 1 段 |
