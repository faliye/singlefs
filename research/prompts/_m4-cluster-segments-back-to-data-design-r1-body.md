# 这次挂载开过的聚簇段怎么在挂载期间还给用户数据（m4-cluster-segments-back-to-data-design-r1）

日期：2026-11-18。

## 问题

一次挂载里小的发布很多时，盘上还有大量空槽，用户数据却报没有空间，只有重挂才收得回来。用户 2026-11-18 定「算缺陷，现在开设计轮」（`records/2026-11-17-里程碑四收口续做.md` 第 168 行）。

量到的（任务 20071，E202 装置在 2288MiB 档造起步池、16384 个活数据单元、没打任何补丁的代码加只数不改的探针；日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-20071.attempt-3.log`）：
- 被拒那一刻的探针行：「label=refusal publishes=21987 segments=2288 cluster_segments=1614 clusters_in_area=1614 core_pairs_outside=0 core_pairs_outside_once_released=0 core_empty_segments=8 core_unblocked_slots=100916 commit_generated_fell_back=0 scan_outside_segments=674 scan_outside_full=674 scan_outside_empty=0 scan_outside_free_pairs=0 scan_cluster_free_slots=101389 scan_cluster_free_pairs=49770 scan_cluster_live_histogram=0:8,1:1416,16-31:1,2:140,3-7:46,8-15:3」。2288 个段里 1614 个是这次挂载开过的聚簇段，其中 1416 个只剩一个活槽；聚簇段里空着的槽对 49770 个，段外可给数据的槽对 0 个。
- 拒因四次都是 `judged_short_by: SlotPairsOutsideTheClusterSegmentsForDataUnits`，`most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(0)`。
- 重挂之后：「label=after_remount … cluster_segments=1 … core_pairs_outside=49503」，「remount_retry=published versions=1」：同一次写发得出去。

这一轮要给出：这次挂载开过的聚簇段，在什么条件下、由哪一步在挂载期间还给用户数据，使上面这种历史不再报没有空间；并且不破坏聚簇段原本要保的东西。逐条答：

1. 聚簇段对用户数据关着要保的是什么：现读 D3（空间分配） 已定项 8、已定项 10 与它们的依据，列出这条规则护着的每一样（提交内生块与用户数据不混放的理由、崩溃一致性上的理由、整理与 discard 的理由、分配次序的确定性）。每一样在「段里只剩一两个活的提交内生块」「段已经全空」两种情形下还成不成立。
2. 候选至少比四条，可以加：
   - 甲「全空即还」：这次挂载开过的聚簇段一旦全空（回收之后没有活槽、没有被扣住的槽）就从集合里摘掉，对数据开放。
   - 乙「低于门槛即还」：活槽数低于某个门槛的聚簇段对数据开放，混放允许（重挂之后今天本来就允许混放）。
   - 丙「整理把稀疏的聚簇段搬空再还」：靠后台整理把只剩一两个活槽的聚簇段里的提交内生块搬走，搬空后按甲还。
   - 丁「提交内生块复用稀疏的聚簇段」：新的提交内生块优先落进这次挂载开过、有空槽的聚簇段，不总是开新段，让聚簇段的个数不随发布次数涨。
3. 每条候选在上面那段历史上：被拒那一刻的直方图是「一个活槽的段 1416 个、全空的段 8 个」——甲只还得回 8 个段；说清每条候选各还得回多少、够不够。
4. 每条候选对已有保证的影响：崩溃点重放的状态空间变不变、池级 checker 的哪几条不变量要改（现读 `crates/singlefs-checker/invariants/` 里提到聚簇段的条目）、整理的停机谓词（全空聚簇段数）还成不成立、分配落点的确定性（同一历史两次跑落点相同）。
5. 准入那一判与「抬 F 把能放的都放了也放不下」的上界要不要跟着改，改成什么；两处与分配器真实行为的一致性怎么钉住。
6. 写成对实现的改动：改 `crates/singlefs-core/src/allocator.rs` 的哪一步、`admission.rs` 的哪一判、条款怎么改句。

## 实现今天的样子

主 agent 的观测（现读主工作区的 `crates/` 与 kb）：

- `crates/singlefs-core/src/allocator.rs:2289`–`:2293`：字段 `cluster_segments: BTreeSet<SlotNumber>` 的文档「这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）」「段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块」。
- `grep -n 'cluster_segments\.insert\|cluster_segments\.remove\|cluster_segments\.clear\|cluster_segments\.retain' crates/singlefs-core/src/allocator.rs` 只有一行：`:3075` 的 `self.cluster_segments.insert(segment_start);`。一次挂载里这个集合只进不出。
- `allocator.rs:862`–`:885` 一带：用户数据落得下的槽对数与「全放开之后落得下的槽对数」两个量都先减掉 `cluster_segments` 里每一段的槽对（`pairs_inside_the_cluster_segments`，`:579`）。
- `crates/singlefs-core/src/admission.rs:845` 的拒因成员 `SlotPairsOutsideTheClusterSegmentsForDataUnits`。
- `.claude/kb/decisions/03-空间分配.md` 第 168 行（D3（空间分配） 已定项 8 第 2 条）：「聚簇段在挂载期间对用户数据关着：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给用户数据…重开之后…上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许」。
- 调查员的读码结论（`~/.local/share/singlefs/work/e202-pool-construction-no-space-investigation/result.md`，主 agent 没逐处复核）：数据落点跳过全部聚簇段（`allocator.rs:1329`）；「抬 F 全放出来也放不下」的上界在 `crates/singlefs-core/src/mounted_session.rs:6079`；重挂时集合经 `Self::new` 清空（`allocator.rs:2674`）；造数据那一次挂载里装置不跑整理。
- 「整理在本次挂载里也还不回来」是调查员推的，没量过：`crates/` 里整理搬空一个聚簇段之后那一段还算不算聚簇段，腿自己现读。

方案按 `crates/` 今天的实现来谈；改法写成对实现的改动。

## 压着这件事的条款

- D3（空间分配） 已定项 8「用户数据块不进聚簇段，落点是已选设备内起点最小的空槽」、已定项 10「聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序」。
- D26（后台整理与放置回收） 已定项 1（整理的停机谓词用全空聚簇段数）。
- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」（准入路径的代价不许随盘容量增长）与「分支的三条约束」第 1 条（只在操作期间不会变的输入上分支）。
- `.claude/rules/format-evolution.md`：聚簇段集合只在内存、不落盘；候选要是需要落盘的状态，写明动了哪些字节。

## 判据格

| 格 | 问什么 |
|---|---|
| 规则护着什么 | 逐样列，各自在稀疏段、全空段上还成不成立 |
| 还得回多少 | 各候选在任务 20071 那段历史被拒那一刻各放出多少槽对、够不够那一次写；换一段历史（长寿的提交内生块钉住段）还成不成立 |
| 崩溃一致性 | 各候选改不改盘上字节、崩溃点重放的状态空间、重挂之后的落点规则 |
| 不变量与 checker | 哪几条不变量要改或新立，池级 checker 怎么判 |
| 整理 | 停机谓词、整理的目的地选择受不受影响；丙要整理做的事今天的整理做不做得到 |
| 准入与上界 | 两处怎么改，怎么与分配器的真实行为钉成一致 |
| 落点确定性 | 同一历史两次跑落点相同还成不成立 |
| 代价 | 准入路径上新增的计算随什么涨 |

## 分工

- 正推腿：逐条答「问题」六条，推荐一条或几条的组合，写成对实现的改动与条款改句；判据格逐格答。
- 攻方腿：攻四条候选各自最弱的一格；给每条候选各找一段让它还不回空间或破坏已有保证的历史（能建小模型的建，命令交主 agent 跑）；核「规则护着什么」那一格里有没有漏掉的理由（读 D3 已定项 8、10 的依据与欠账表里提到聚簇段的条目）。
- 两条腿都不改 `crates/`。

## 交用户的

暂无；判决时列。
