# e203-b8-criterion-r1：B8 的判据改成按这一次碰到的单元算（2026-11-09）

<!-- doc-lint:not-numbers B1 B4 B8 F2 Q6 Q8 P2a -->

## 一、题面

用户 2026-11-09 弹窗定：B8 的判据改成按这次改动，也就是步数除以这一次碰到的单元数，看随总量涨不涨。是新一轮，原判据下的输先记下（`q8_peak axis=data_units alpha=0.978`，F2 字面打中）。改判据不等于选 B8，选臂另问。这一轮问：新判据写成什么才不是放宽。

原判据与读数：`research/prompts/e203-preregistration.md` 第十二节修订 8；`research/prompts/e206-preregistration.md` Q8；`.claude/kb/experiments/206-发布固定点轮数的代价.md`「B8 那一格的读数」；来源调查 `~/.local/share/singlefs/work/e206-b8-data-units-peak/result.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

`crates/` 今天没有 B8 的实现，步数是模型 `scripts/experiment-apparatus/e206_fixed_point_cost.rs` 按 E203 登记第五节式子算的。主工作区固定点已是 P2a。方案按 `crates/` 今天的实现来谈。

## 三、要回答的问题

1. 分母写成哪个量：这次释放的单元数、重写的角色数，还是两者之和；有没有办法用一个大改动把分母撑大、遮掉随总量涨的项。
2. 按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在跑前写死」三步：记一次输、只许收严、写明收严在哪。这个新判据比原判据多要求了什么，说不出就是放宽。
3. 用现有 40 份产物，新判据对 B1、B4、B8 各判什么；`files_then_big_*`（总量在、改动不在）还要不要量。
4. `.claude/rules/fs-design.md`「记账是事务的副产品」运行时决策路径「代价不许随盘容量增长」这句，新判据还罩不罩得住。

## 四、压着的条款

D8（核心索引结构） 已定项 16；D28（挂载期承诺量） 已定项 1。
