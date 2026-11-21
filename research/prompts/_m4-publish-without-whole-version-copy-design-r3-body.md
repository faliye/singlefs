# 发布不再整版照抄：第三轮，最终轮（m4-publish-without-whole-version-copy-design-r3）

日期：2026-11-18。

## 问题

前两轮判决：`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`、`research/prompts/m4-publish-without-whole-version-copy-design-r2-main-verification.md`；同一件事更早的一轮 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`。

前两轮站住的形态（下称甲）：`TransactionOutput` 里的单元列表、映射条目、分配记录三张表换成增量更新时与上一版共用没改的块的持久表，版本仍然是值、读得到全版；另立单元的键复刻今天 `units` 的次序；带搬迁的发布用一张写死的「角色 → 指针所在处」对照表，不另建反查索引；相等按内容比。量到的依据（任务 19949、19957，2288MiB 档 P=4096 造池三段）：交错写段一次发布改到的单元中位 18（全版中位 4500）、映射 27（4468）、分配记录 72（43582）；跑前写死的翻面线 T1–T4 `flipped=no`，T5 `flipped=yes`（`leftover_over_records_max=0.861538`，分配器的已回收集交错写段最大 51790 项）。

这一轮是最终轮，论证前两轮的结论和判定过程，攻三样（不重复前两轮已攻过的角度：三条候选谁最脆弱、甲三处难点的解法、测试里的读者分类）：

1. 「已定」项撑不撑得住。甲与这几条已有的东西并不并得住，逐条现读：
   - 不动形态的两批（补丁 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch`，还没进主工作区）加进去的东西：分配器按盘与槽的索引、待回收集合、演练做完撤回的撤销步、上一版按叶分组跨发布带着走的缓存、中央映射树只按改到的 key 规划。甲把分配器的记录表换成持久表之后，这几样哪些白做了、哪些要重写、哪些正好是甲的一部分；两条线的合入次序怎么排才不做两遍。
   - 2026-11-06 那一轮 Q2.3 ③ 与 D16（发布语义） 已定项 17：落盘失败那一支要发布之后那一份分配器。甲下「发布前」「发布后」两份分配器各是什么、冻结重发那一支拿哪一份。
   - `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：甲之后一次发布里还剩哪几处随全版涨（建树函数、落盘、别的），列全并各给出处。
2. 未定项清单的完整性。前两轮留下的未定项：容器自己写还是引包、分步先换哪张表、已回收集为什么只涨不缩、乙强与丙强零轮、整理发布与「一次改很多」的发布没量。还漏了什么：
   - 已回收集（`reclaimed_placements`）：现读它在分配器里是什么、谁往里加、谁删、为什么随发布次数涨到与记录同阶；它是不是该有界（比如随回收门槛推进清掉）；它涨是这一题的事，还是另一个缺陷。
   - 内存：留着上一版与这一版共用块时常驻内存怎么变；E202 最小档（36608 个活数据单元）与最大档下甲的常驻估多少。
   - 并发：里程碑后面读写并发时，持久表被读者持着的旧版什么时候放。
   - 验证包的内存池、崩溃引擎读版本的地方（前两轮只看了 `src` 的一部分）。
3. 那一轮报出的数字能不能核。第二轮探针的数：d 的定义（「改到」算不算照抄但换了下标的项、算不算只改了落点的项）与甲真要复制的块数对不对得上；`copy64_over_n` 假设 64 条一块，换 16、256 结论翻不翻；探针只量了造池三段，给出一条在整理发布上也能量的命令（装置有没有不要卡就能走到整理发布的子命令，现读装置）；第二轮判决表里的每个数在任务 19957 的日志里逐字找得到。

最后给出交用户的清单与分步的终稿：每一步改哪个文件的哪个结构、验收、能不能单独合。

盘上字节、发布语义、分配次序与落点一个都不变；这一题只动内存里的表示。

## 实现今天的样子

主 agent 的观测（现读主工作区的 `crates/` 与任务日志）：

- `crates/singlefs-core/src/transaction.rs:3375` 起的 `pub struct TransactionOutput`：`units: Vec<PublishedUnit>`、`mapping_keys: Vec<Vec<u8>>`、`allocation_records: Vec<AllocationRecord>` 等整份的 `Vec`；`:3479`「pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,」。
- `transaction.rs:13213`「let allocator_before_this_publish = allocator.clone();」、`:14076`「let mut rehearsal = allocator.clone();」、`:15822`「let mut relocated_version = previous_version.clone();」、`:16232`「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」、`:11390`「RelocatedUnitHolder::RoleOfThisVersion(*role),」。
- 仓根 `Cargo.toml:28`「debug-assertions = true」。`crates/singlefs-checker/Cargo.toml` 的依赖只有 `singlefs-format`。
- 不动形态的两批在副本里验过（没进主工作区）：新旧对拍测试、已有 60 条核心单测、验证包点名的测试目标、build 都退 0，clippy 剩一处写法；两批之后 P=4096 带计时的建池 63.438 秒（改前带计时那一趟 193.899 秒，本机负载高，只看趋势），交错写每步最早八分之一 8.334、最晚八分之一 45.737 毫秒（任务 19899）。两批之后量到的分段行：`~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/section-timing-after-the-second-speedup-task-19899.txt`。
- 第二轮探针的产物与汇总：`~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/publish-change-probe-2288MiB-P4096.out`，汇总任务 19957 的日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-19957.attempt-1.log`；探针的 diff `research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/probe/publish-change-probe.diff`。

方案按 `crates/` 今天的实现来谈；改法写成对实现的改动。

## 压着这件事的条款

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
- `.claude/kb/milestone/04-fourth-txn.md`「优化项：不在出口，进入条件已登记」表里「发布不再整版照抄」那一行。
- `.claude/rules/fs-design.md`「一个事务层，所有结构共用」。
- `.claude/singlefs-ai-sop/rules/code-discipline.md`「类型：让非法状态写不出来」。

## 判据格

| 格 | 问什么 |
|---|---|
| 与不动形态两批并不并得住 | 哪几样白做、哪几样重写、合入次序 |
| 冻结重发 | 甲下发布前后两份分配器各是什么 |
| 甲之后还剩什么随全版涨 | 列全、带出处 |
| 已回收集 | 是什么、为什么涨、该不该有界、归这一题还是另一个缺陷 |
| 内存与旧版的释放 | 常驻怎么变；并发时旧版什么时候放 |
| d 的定义对不对得上 | 探针数的与甲真要复制的是不是一回事；块大小换了翻不翻 |
| 整理发布上的 d | 给一条能量的命令 |
| 数能不能核 | 第二轮判决表里的数逐字在日志里 |
| 分步终稿 | 每步改什么、验收、能不能单独合 |

## 分工

- 正推腿：「分步终稿」「与不动形态两批并不并得住」「冻结重发」「甲之后还剩什么随全版涨」「内存与旧版的释放」，并给交用户的清单。
- 攻方腿：「已回收集」「d 的定义对不对得上」「整理发布上的 d」「数能不能核」，并攻正推那几格里前两轮没被看过的地方；未定项清单还漏了什么。
- 两条腿都不改主工作区的 `crates/`。

## 交用户的

暂无；判决时列。已知两条：持久容器自己写还是引外部包；分步先换哪一张表。
