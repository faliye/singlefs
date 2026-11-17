# d3-fallback-placement-r1：提交内生块回落时的落点，要不要优先已在重写集里的叶（2026-11-09）

<!-- doc-lint:not-numbers P2a F1 -->

## 一、题面

用户 2026-11-09 弹窗定：固定点上界 64 与回落形状「现在就改落点政策」，回落时优先取已在重写集里的叶。主 agent 推荐的是登欠账，用户没选。这一轮不再问要不要改，问怎么改才对：改法的定义、它压着的条款、它会不会打破别的已定项，代价是多少。

现状条款：`.claude/kb/decisions/03-空间分配.md` D3（空间分配） 已定项 8 第 2 条：「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」。实现：`crates/singlefs-core/src/allocator.rs:2570` 一带。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 固定点已是 P2a（`crates/singlefs-core/src/transaction.rs` `merge_into_the_rewrite_set_walking_the_release_chain`，释放链一轮走到闭包），取落点那一半仍可能一轮一跳。
- 量到的（模型任务 11382，`~/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/rerun-1/observations.txt`，副本装置，不进 kb）：改空闲图造的盘，每片叶 2 个空槽时 `fallback sweep` 185 次探里 5 次 `outcome=did_not_settle_limit_64`；`natural` 那一段（用户操作写到盘满）没走到这个形状。
- E206 第一段 16 个点 P2a 最多 5 轮，产物里没有回落这一列。
- 方案按 `crates/` 今天的实现来谈。

## 三、要回答的问题

1. 「优先已在重写集里的叶」的精确定义：按叶还是按段；空槽不够时退到哪里；多盘时每盘答案要不要相同（已定项 8 第 5 条末段第一版盘不等大的限制）。
2. 它碰到的条款：D3（空间分配） 已定项 8、已定项 5、已定项 10；D26（后台整理与放置回收） 已定项 3 的底额；分配记录树自身落点会不会形成新的环；放进重写集的叶里放新节点，崩溃状态与 checker（`crates/singlefs-checker/invariants/`）有没有不变量要动。
3. 它修得了哪一格：`did_not_settle_limit_64` 那 5 次探在新政策下还撞不撞（模型 `research/prompts/m4-close-fixes-code-r3-opus-model/` 的 `fallback` 那一段可以直接改臂量）；有没有反过来变差的形状（聚集写放大、局部碎片）。
4. 代价数：每条候选的步数、写放大，按 `.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」。

## 四、压着的条款

D3（空间分配） 已定项 8 第 1、2 条，已定项 10；C146（无空段时的回落政策全仓无定义）。
