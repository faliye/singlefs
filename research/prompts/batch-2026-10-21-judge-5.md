# 阶段同步第二段逐行判（batch-2026-10-21，组 F25-F38）

基准 1bb3a24dc，结束「工作区」（提交 0b926bd3f、767668191、511fe067b、e1b1ceb95、841b3beab 都在基准之后）。候选表 `/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv`，sha256 a2913e391a222ec455f068bb72d7123c94659be2da960181125ecf2bc132e6d9（已核）。只判不改，仓里没动一个文件。

## 摘要

分到的组 F25 F26 F27 F31 F32 F33 F34 F35 F36 F37 F38（候选表里没有 F28-F30，新立行不出候选），共 265 行，每行都判了。各判定行数（候选 265 行 + 反向问出的 3 行「要补」）：

- 要改 46 行
- 要补 3 行（M1 M2 M3，见表末）
- 要人看 4 行（F27 两行、F34 两行）
- 事件句不改 58 行
- 不相干 157 行

每组：F25 21（要改 1）、F26 29（要改 1）、F27 20（要改 1、要人看 2）、F31 14（要改 3）、F32 14（要改 7）、F33 7（要改 3）、F34 30（要改 4、要人看 2）、F35 15（要改 1）、F36 5（要改 4）、F37 36（要改 12）、F38 74（要改 9）。

最要紧的几处（要改的里头，主 agent 先看这几条）：

1. `CLAUDE.md:8`「当前里程碑：**「删除与多次 COW」**」没跟 README 走（README.md:45 已写「共享与回收」），F25 第 8 行，给了改后句子。`git status --short CLAUDE.md` 无输出，这一阶段没改过它。
2. C821（GPU 判器没有扫描重建内核）已还清：`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl` 入库，`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 现为 `&[]`（`crash_judge_conclusions.rs:438`）。但 `checks-owed.md:536` 与中英日三份译文还当它开着（F32 第 2、4、6 行）。该用例在 GPU 上绿不绿我没跑（重型），销账前由主 agent 看提交时结果。
3. 里程碑四分组表（`04-fourth-txn.md:34-36`）「1 设计」「2 实现」「3 覆盖」状态格仍写未开，而核心层、扫描重建、新崩溃流都已合入（F31 第 4-6 行）。
4. 「crates 里没有扫描重建 / 没有整理代码 / 没有往 deadlist 写的路径」这一类现状句有一批：D18 射程（F32 第 7 行）、C113 仍欠（F32 第 5 行）、D26 射程「命中 11 份」现为 65 份（F33 第 2 行）、`04-fourth-txn.md:44/59/86`（F33 第 3 行、F34 第 14 行、F36 第 3 行）、C243/C364/C373 前置格（F37）。
5. 「代码今天的 `empty_segments` 是占用口径」已过时：`allocator.rs:847` 现返回记录口径的 `full_segments_without_an_unreleased_record`；D3 已定项 10 ①、D26 已定项 1 ①、C372 前置格要改（F38 第 2、4、6-12 行）。C413 对应的 I-3.6/I-3.7/I-3.16/I-4.4 在不变量状态格都写已实现（2026-10-15）。
6. 事实表写的那几处矛盾，现查结果：C821 见上；F31 分组表状态仍是未开（要改）；F9、F59、F46、F73 不在我的组里，没判。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F25 | .claude/kb/checks-owed.md:413 | 不相干 | 这一行说「D13（验证路线） 已定项 9 定由主 agent 把「上一次全量」写进当前里程碑记录一行」，指里程碑记录里登记上一次全量，不是说哪个里程碑是当前里程碑，不受「当前是共享与回收」影响。 |
| F25 | .claude/kb/checks-owed.md:507 | 不相干 | 「全仓还在引那句的地方（CLAUDE.md 当前里程碑一段、里程碑二记录、kb）已回扫」是 C761 登记行记的回扫范围，说的是引 26.16 亿那句的位置，不是哪个里程碑当前。 |
| F25 | .claude/kb/decisions/13-验证路线.md:259 | 不相干 | 「「上一次全量」由主 agent 写进当前里程碑记录一行」是一条做法，写在哪份记录里随当前里程碑走，不断言当前是哪一个，不受影响。 |
| F25 | .claude/kb/milestone/09-ninth-txn.ja.md:90 | 事件句不改 | 日文译文引用户 2026-10-16 的原话「把当前里程碑的 真盘端点挪动到里程碑9」，那一天的当前里程碑是里程碑四，说的是那次挪动，换成新说法也不改这句引话。 |
| F25 | .claude/kb/milestone/09-ninth-txn.md:90 | 事件句不改 | 「用户 2026-10-16 定挪进这一份（原话「把当前里程碑的 真盘端点挪动到里程碑9」）」是那一天的决定和原话，不是现状句。 |
| F25 | .claude/main-agent.md:3 | 不相干 | 「公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`」只说 CLAUDE.md 里放什么，不说当前里程碑是哪个。 |
| F25 | .claude/main-agent.md:10 | 不相干 | 这一行讲崩溃点测试优先级和全量由谁跑，「里程碑收尾、动了崩溃点重放或判器的那一批」里的「当前里程碑」字样出现在后半段的引用里，不断言当前是哪个里程碑。 |
| F25 | CLAUDE.md:8 | 要改 | CLAUDE.md 现状句「当前里程碑：**「删除与多次 COW」**」已过时：README.md:45 写当前里程碑是「共享与回收」（`.claude/kb/milestone/04-fourth-txn.md`），里程碑三主体已完结、还开着段 4。改后：当前里程碑：**「共享与回收」**（`.claude/kb/milestone/04-fourth-txn.md`，作业点、验收与次序在那份文件；调度在 `records/2026-10-13-里程碑四预开发.md`）。上一个里程碑**「删除与多次 COW」**（`.claude/kb/milestone/03-third-txn.md`）主体已完结，还开着的是段 4「优化」（二·6、六·4）。再上一个**「覆盖写、释放、回退与复用」**已结束；后面「已结束，没做完的转进里程碑三项五」「它的崩溃放量已接进 `crates/singlefs-checker-tier`」两句沿用。（现查：`ls crates` 八个 crate 与「八个 crate」相符；`git status --short CLAUDE.md` 无输出，这一阶段没改过它。） |
| F25 | CLAUDE.md:12 | 不相干 | 「`CLAUDE.md` 只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实」只列 CLAUDE.md 放什么类别的内容，不断言当前里程碑是哪个。 |
| F25 | README.md:45 | 不相干 | README.md:45 已写新说法「当前里程碑是「共享与回收」」，并写里程碑三「主体已完结，还开着的是段 4「优化」（二·6、六·4）」，与结束那一版一致，仍是真话。 |
| F25 | records/2026-09-24-里程碑二收尾调度.md:23 | 事件句不改 | 2026-09-24 里程碑二收尾调度表「`CLAUDE.md` 当前里程碑那一行」是那次收尾要做的一步，不是现状句。 |
| F25 | records/2026-09-28-里程碑三作业点规划.md:24 | 事件句不改 | 表里「`CLAUDE.md` 当前里程碑那一行」一格的状态是「做了」，是 2026-09-28 作业点规划里的一行任务状态，说的是那次做了什么。 |
| F25 | records/2026-09-28-里程碑三作业点规划.md:51 | 事件句不改 | 「当前里程碑那一行加「删除与多次 COW」并行」是 2026-09-28 规划里要对 CLAUDE.md 做的改动，说的是那一次的计划。 |
| F25 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:51 | 事件句不改 | 2026-09-28 调度记录里「收尾第 8 步 E152 跑完；第 9 步简报与里程碑状态写了」是那一天收尾做了什么；后文提到 CLAUDE.md 当前里程碑那一行也是那天的改动。 |
| F25 | records/2026-09-30-里程碑三开工第一轮.md:193 | 事件句不改 | 用户 2026-10-01 的原话「将当前里程碑中的 删除事务和并发事务 转交给对话」和随后的转交经过，说的是那一天的事。 |
| F25 | records/2026-10-01-README更新与知识腐烂治理.md:27 | 事件句不改 | 「CLAUDE.md 的当前里程碑换成「删除与多次 COW」」是 2026-10-01 那一次的改动记录；换成新说法这句仍是那天做过的事，不改。 |
| F25 | records/2026-10-01-删除与并发事务.md:1291 | 事件句不改 | 2026-10-08 记录「这句话要回扫（CLAUDE.md 当前里程碑一段也引它）」是当天对回扫范围的记述，不是现状句。 |
| F25 | records/2026-10-13-里程碑四预开发.md:307 | 事件句不改 | 「用户 2026-10-16：「把当前里程碑的 真盘端点挪动到里程碑9」」是那一天的决定与原话。 |
| F25 | records/2026-10-21-doc门禁违规修复.md:5 | 事件句不改 | 2026-10-21 记录开头记的弹窗定案和「README「三主体完结，当前里程碑四」」是这一轮的决定，已与新说法一致，是事件记述。 |
| F25 | records/2026-10-21-doc门禁违规修复.md:18 | 事件句不改 | 「README 三语改成里程碑三主体完结、当前里程碑四」是这一轮的课题，说的是那一次要做的改动。 |
| F25 | records/2026-10-21-doc门禁违规修复.md:43 | 事件句不改 | 「主 agent 改 README 三语「当前状态」一节：当前里程碑「共享与回收」」是这一轮做过的改动，是事件记述，与新说法一致。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:415 | 不相干 | 这一行的「七行在这一批三条流上每行至少一条红」是 E171 三 oracle 合谋测试里那三条被测流，不是里程碑三一·1 要写的多次 COW 三条流。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:418 | 不相干 | 「三趟 × 三条流 × 两批 18 格每格都探过」是 E171 的格数，三条流指 E171 的被测流，与多次 COW 三条流无关。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:567 | 不相干 | 「三条流的第 0、1 次调用是取号、暖机」说的是 E171 三条被测流的调用次序，不是一·1 的流清单。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:662 | 不相干 | 「Q1 的甲只说「七行在这一批三条流、批 1 上每行至少一条红」」是 E171 判决的读法，三条流指实验流。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:980 | 不相干 | 「三条流的分配器带 mkfs 的根环表那三行两份相同」是 E171 重跑两份产物的对比，与一·1 无关。 |
| F26 | .claude/kb/experiments/171-三oracle合谋测试.md:1036 | 不相干 | 「三条流全量态，枚举点不是扫遍」说的是 E171 比较器的取态，不是一·1 的三条多次 COW 流。 |
| F26 | .claude/kb/milestone/03-third-txn.md:58 | 要改 | 先给 core 加按 inode 号写数据的入口已经做完：`crates/singlefs-core/src/mounted_session.rs` 已有 `UserChange::OverwriteOfAnInode`、`UserChange::SequentialWriteOfAnInode`（`transaction::publish_overwrite_of_an_inode`、`publish_sequential_write_of_an_inode`，提交 0b926bd3f），三条流的测试 `crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream`、`crash_enumeration_several_inodes_overwritten_in_turn_stream`、`crash_enumeration_large_file_overwritten_across_lower_leaves_stream` 已入库并登记在 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行（现查 `git log -S OverwriteOfAnInode` 与 `grep -n overwritten .claude/gate.d/stage-inputs.tsv`）。改后：一·1 的三条流：用户 2026-10-18 定三条都新写，先给 core 加按 inode 号写数据的入口（动 `crates/singlefs-core/src/transaction.rs`），入口与三条流已随里程碑四核心层那批代码合入（提交 0b926bd3f）。同文件第 55 行（一·1 作业点行，未进候选）的「写数据的入口只收第一个文件……排在里程碑四之后」是同一类过时句，一并改。 |
| F26 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:119 | 不相干 | 「三条流的共享前缀占比极低 也没有关系」是 2026-09-28 用户谈固定脚本流共享前缀的原话，那三条流与一·1 的多次 COW 流不是同一批，也是那天的决定。 |
| F26 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:175 | 不相干 | 「三条流每个状态 GPU 与 CPU 判器逐项比」是对拍用例覆盖的固定脚本流，与一·1 的多次 COW 流无关。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:592 | 不相干 | 「要带 `gpu` 特性跑三条流的对拍看回退流」是 2026-09-30 的 GPU 对拍线索，说的是当时那几条流，不是一·1。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:608 | 不相干 | 「对拍打印每行截到 700 字」一段里的流是 GPU 对拍的红，说的是 2026-09-30 那次排查，与一·1 的三条流无关。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:751 | 不相干 | 「`deletion-through-the-last-batch` 三条流状态 156–164 不一致」是 2026-09-30 对拍的红，三条流指删除线那三条，不是一·1。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:753 | 不相干 | 「删到最后一批三条，`状态 1[56][0-9]：` 27 行」是 2026-09-30 对拍结果，说的是删除线那一次的数。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:1012 | 不相干 | 「E6 两条边界用例也红」是 GPU 表容量排查，三条流是那一次撞表的流，不是一·1。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:1015 | 不相干 | 「E7 在 naga 中间表示上断言站点 29 走不到」一段里的「三条流基线」指 GPU 判器的基线流，与一·1 无关。 |
| F26 | records/2026-09-30-里程碑三开工第一轮.md:1017 | 不相干 | 「三条流先撞的都是 journal 记录表 `MAX_RECORDS` 512」是 GPU 容量排查里的流，与一·1 的多次 COW 流无关。 |
| F26 | records/2026-10-01-删除与并发事务.md:1498 | 不相干 | 「它派调查员查三条流各撞哪张表」说的是 2026-10-01 GPU 容量排查，不是一·1。 |
| F26 | records/2026-10-11-崩溃放量两段与脚本审计.md:92 | 不相干 | C790 入口侧交回记录里的流是崩溃放量分项驱动的，与一·1 的多次 COW 三条流无关；也是 2026-10-13 的事件记述。 |
| F26 | records/2026-10-11-崩溃放量两段与脚本审计.md:93 | 不相干 | 「第三条流 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels` 第 0 段 322 写」是位置寻址树那一族的流，与一·1 无关。 |
| F26 | records/2026-10-11-崩溃放量两段与脚本审计.md:96 | 不相干 | 「第三条流第 0 段 322 次全是单元写」仍是位置寻址树那一族的流，说的是 2026-10-13 的 panic 排查。 |
| F26 | records/2026-10-13-复用键与全仓改名.md:769 | 不相干 | 「三条流累计 2046、2048、2086 条」是 GPU 扫描内核去重表容量的流，不是一·1 的三条流。 |
| F26 | records/2026-10-13-复用键与全仓改名.md:967 | 不相干 | 「第 8 条 position-addressed-tree-streams 第三条流在跑」是 2026-10-18 的监控记录，说的是位置寻址树流。 |
| F26 | records/2026-10-13-里程碑四预开发.md:168 | 不相干 | 「三条流按真写表数：F1 建快照 15 段」是快照三条崩溃流的数，是里程碑四预开发的记录，不是一·1 的多次 COW 流。 |
| F26 | records/2026-10-13-里程碑四预开发.md:312 | 不相干 | 「三条截断流（缩到 0」是截断三条流，里程碑四的内容，与一·1 的多次 COW 三条流不是同一批。 |
| F26 | records/2026-10-13-里程碑四预开发.md:682 | 不相干 | 「谱系乱序三条流与三行变异」是 GPU 池级判器的坏流，不是一·1。 |
| F26 | records/2026-10-18-里程碑三收尾与近期欠账.md:53 | 事件句不改 | 「一·1 三条流要 core 先加按 inode 号写数据（动 `transaction.rs`），整包「等里程碑四」」是 2026-10-18 当天用户四条定案并进包表的记述，换成新说法这句仍是那天的决定，不改。 |
| F26 | records/2026-10-18-里程碑三收尾与近期欠账.md:329 | 不相干 | 出口 4 丙交回里的门禁格与流是 2026-10-18 的事件记述，「三条流」不是一·1 的多次 COW 流。 |
| F26 | records/2026-10-18-里程碑三收尾与近期欠账.md:424 | 不相干 | 「unlink-under-a-snapshot / truncate-into-a-unit-then-extend / truncate-under-a-snapshot 三条流的必红变异行」是快照与截断的三条流，不是一·1。 |
| F26 | records/2026-10-18-里程碑三收尾与近期欠账.md:431 | 不相干 | 「三条流各追加一行必红变异」仍是上一行那三条快照与截断的流，与一·1 的多次 COW 流无关。 |
| F27 | .claude/kb/checks-owed.ja.md:295 | 不相干 | 日文译文这一行是 C409 的欠账行，「常规层、大负载层在ゲート里没有阶段」仍是真话（`ls .claude/gate.d/` 没有这两层的阶段），去向在同一行后文写明，已是新说法。 |
| F27 | .claude/kb/checks-owed.md:294 | 不相干 | C409 登记行「常规层、大负载层在门禁里没有阶段」仍是真话，行内已写「用户 2026-10-18 弹窗定「立」」与去向挪里程碑四八·3，与结束那一版一致。 |
| F27 | .claude/kb/decisions/13-验证路线.md:263 | 不相干 | 「常规层、大负载层在门禁里没有阶段」仍是真话；这一行讲的是冒烟层快档与全量的射程，不涉及常规层、大负载层在哪个里程碑立阶段。 |
| F27 | .claude/kb/decisions/13-验证路线.md:266 | 要改 | 「无实验……常规层、大负载层 的负载与挂钟都还没有，没有可量的量」已不全是真话：E184（常规层与大负载层的代价计数）第一段「负载规模与重叠」已跑（`.claude/kb/experiments/184-常规层与大负载层的代价计数.md` 标题「部分已跑」，产物 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` 950 行，量了 127 条候选流的态数区间与重叠），第二到四段未跑。改后：- 实验：分层是门禁挂钟的设计约束；常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数）第一段已量候选负载的规模与态数区间重叠，冷吞吐、块键作废面、固定开销三段未跑。 |
| F27 | .claude/kb/decisions/13-验证路线.md:274 | 不相干 | 「欠：C409（常规层、大负载层在门禁里没有阶段）；C635」两笔欠账仍开着，与这一行事实无冲突，仍是真话。 |
| F27 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:5 | 不相干 | E184 页这一段是跑前登记与回答的问题单，「D13（验证路线） 已定项 9 层表里的常规层，该收哪些负载」描述实验问题，不断言常规层立不立阶段。 |
| F27 | .claude/kb/milestone/03-third-txn.md:25 | 不相干 | 「真设备崩溃注入已挪里程碑九，用户 2026-10-18 定」已是新说法，与结束那一版一致，仍是真话。 |
| F27 | .claude/kb/milestone/03-third-txn.md:141 | 不相干 | 「三层分档是 D13（验证路线） 已定项 9，只有冒烟层（C409……已挪 [04-fourth-txn.md]」仍是真话（门禁里只有冒烟层）；同一行「登记的流 29 条」另属 F59 那一行（stage-inputs.tsv 现 35 条 crash-case），在那一组判。 |
| F27 | .claude/kb/milestone/03-third-txn.md:149 | 要人看 | 03-third-txn.md 六·4 这一行同时写「常规层、大负载层 在这个里程碑立阶段」和「C409……（已挪 [04-fourth-txn.md] 八·3）」；而 checks-owed.md:294 写「去向：2026-10-19 挪出里程碑三，归 04-fourth-txn.md 八·3（用户 2026-10-19 弹窗定「挪到里程碑4」）」，04-fourth-txn.md:159 八·3 也写「常规层、大负载层各立一个门禁阶段」。立阶段到底算里程碑三六·4 还是里程碑四八·3，同一行两说，需要用户或主 agent 定后改一边。 |
| F27 | .claude/kb/milestone/03-third-txn.md:152 | 要人看 | 03-third-txn.md:152「常规层、大负载层 立阶段（用户 2026-10-18 定「立」，负载形态先开三方定）」列在里程碑三「开工前要先定的」里，但 C409 已于 2026-10-19 挪里程碑四八·3（checks-owed.md:294、04-fourth-txn.md:159）；里程碑三里还留不留这一条要人定。同行「真设备崩溃注入已挪里程碑九」已是新说法、仍真。 |
| F27 | .claude/kb/milestone/04-fourth-txn.md:151 | 事件句不改 | 「C409（常规层、大负载层在门禁里没有阶段）「挪到里程碑4」」是用户弹窗的原话摘记，说的是那次决定。 |
| F27 | .claude/kb/milestone/04-fourth-txn.md:159 | 不相干 | 「常规层、大负载层各立一个门禁阶段」与「C409……（用户定挪里程碑四；负载形态等……G1–G4 与 E184 的代价数）」已是新说法，E184 只跑了第一段，「等代价数」仍是真话。 |
| F27 | .claude/kb/milestone/09-ninth-txn.md:91 | 不相干 | 「从里程碑三挪来的真设备崩溃注入（用户 2026-10-18 定）」已是新说法，仍是真话。 |
| F27 | .claude/kb/term-renames.md:27 | 不相干 | term-renames 这行说的是「层 0」档随分层表三档改成冒烟层、常规层、大负载层，与它们在不在门禁里立阶段无关。 |
| F27 | records/2026-09-28-里程碑三第十项清单.md:255 | 事件句不改 | 2026-09-28 清单表里 C409 一行「项一」「项六」两格是当时判去向的记录，不是现状。 |
| F27 | records/2026-10-01-对接上游测试脚本规范.md:444 | 事件句不改 | 「已定项 9 的索引行与定案里的「夜间档」一律改叫「全量档」」是 2026-10-01 的改名结论，说的是那一次改了什么。 |
| F27 | records/2026-10-01-对接上游测试脚本规范.md:447 | 事件句不改 | 「改前：`git show HEAD:...` 里 C409 那一行」是 2026-10-01 记录改动前的取证命令，说的是那一次。 |
| F27 | records/2026-10-18-里程碑三收尾与近期欠账.md:48 | 事件句不改 | 「六·2 真设备崩溃注入：……挪里程碑九」是 2026-10-18 用户定案的当日摘记，与新事实同向，说的是那次决定。 |
| F27 | records/2026-10-18-里程碑三收尾与近期欠账.md:49 | 事件句不改 | 「C409：「立」——常规层、大负载层在门禁里立阶段」是 2026-10-18 用户那一条定案的记录，说的是那天定了什么。 |
| F27 | records/2026-10-18-里程碑三收尾与近期欠账.md:101 | 事件句不改 | 「批 4 写回员交回」一行记 2026-10-18 写回员做了什么（「真设备崩溃注入挂里程碑九」等），是事件记述。 |
| F31 | .claude/kb/milestone/03-third-txn.md:39 | 不相干 | 这是里程碑三分组表的「1 设计」行，状态格写的是里程碑三自己的「七·1 做了……三·1 做了」，不是里程碑四的分组表，不受影响。 |
| F31 | .claude/kb/milestone/03-third-txn.md:40 | 不相干 | 这是里程碑三分组表的「2 实现」行，状态格写里程碑三七·2、七·3 的进度，与里程碑四分组表无关。 |
| F31 | .claude/kb/milestone/03-third-txn.md:41 | 不相干 | 这是里程碑三分组表的「3 覆盖」行，状态格写「七·6：用户 2026-10-02 定做」等里程碑三的进度，不是里程碑四。 |
| F31 | .claude/kb/milestone/04-fourth-txn.md:34 | 要改 | 里程碑四分组表「1 设计」行（一·1（D5（快照 / 空间记账机制） 重开与只读快照形态）∥ 二·1……）的状态格仍写「未开」，但设计轮已有判决入库：`ls research/prompts/` 有 m4-snapshot-r1（一·1）、m4-gc-r1（二·1）、m4-trunc-r1 与 m4-trunc-r2（六·1）的 main-verification.md；四·1 的扫描重建题面随 `crates/singlefs-checker/src/rebuild.rs` 合入（提交 0b926bd3f）。改后状态格：做了（一·1 `research/prompts/m4-snapshot-r1-main-verification.md`；二·1 `research/prompts/m4-gc-r1-main-verification.md`，E10 的跑前登记见 E10 索引行；四·1 题面随 `rebuild.rs` 写成；六·1 `research/prompts/m4-trunc-r1-main-verification.md`、`m4-trunc-r2-main-verification.md`）。各作业点是否逐项做完，以主 agent 对各作业点行的核对为准。 |
| F31 | .claude/kb/milestone/04-fourth-txn.md:35 | 要改 | 里程碑四分组表「2 实现」行（一·2 → 二·2（共享先落地……）∥ 四·2 ∥ 六·2）状态格仍写「未开」，但核心层已合入 master（提交 0b926bd3f，`git log -S OverwriteOfAnInode` 命中）：`crates/singlefs-core/src/` 已有 `compaction_plan.rs`、`logged_operations.rs`、`region_scan.rs`、`truncate.rs`、`system_configuration_repair.rs`，`crates/singlefs-checker/src/rebuild.rs` 在。改后状态格：核心层已合入 master（提交 0b926bd3f）：一·2 建只读快照 / 快照下覆盖写与删文件 / 删快照，二·2 整理（选段与搬迁计划、意图记录、区域扫描、`publish_relocation_batch`；产品路径在签整理意图之前停下），四·2 `rebuild.rs`，六·2 `truncate.rs`；各作业点还差什么见各作业点行与 `checks-owed.md`。 |
| F31 | .claude/kb/milestone/04-fourth-txn.md:36 | 要改 | 里程碑四分组表「3 覆盖」行（一·4、二·3、三·1、六·3 的流跑在崩溃放量上全域）状态格仍写「未开」，但新崩溃枚举流已入库（提交 0b926bd3f，`crates/singlefs-checker-tier/tests/` 下 `crash_enumeration_snapshot_creation_stream.rs`、`crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs`、`crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs`、`crash_enumeration_compaction_relocation_stream.rs`、`crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs`、`crash_enumeration_lifecycle_closure_stream.rs`、三条截断流）。改后状态格：流已入库并登记在 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行（一·4、二·3、三·1、六·3 对应的流）；全域 0 红是否跑过、二·4 的性质证明做没做，按各作业点行写（二·4 的性质证明与带整理的模型对拍做到哪，以二·4 作业点行为准，我没逐项核）。 |
| F31 | .claude/kb/milestone/05-fifth-txn.md:76 | 不相干 | 这是里程碑五分组表的「2 实现」行（二·2（新类、checker、GPU 判器、layout）→ 三·1（打包））状态「未开」说的是里程碑五（小数据打包容器），这一阶段没动它，不受影响。 |
| F31 | .claude/kb/milestone/05-fifth-txn.md:77 | 不相干 | 这是里程碑五分组表的「3 覆盖」行，「三·3 四条流跑在崩溃放量上全域」仍未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/11-eleventh-txn.md:27 | 不相干 | 这是里程碑十一分组表的「2 实现」行，「二·1 ∥ 二·2，再 二·3」仍未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/11-eleventh-txn.md:28 | 不相干 | 这是里程碑十一分组表的「覆盖 三·1 依赖二 未开」一行，状态未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/12-twelfth-txn.md:36 | 不相干 | 这是里程碑十二分组表的「2 实现」行，「二·1 → 二·2」仍未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/12-twelfth-txn.md:37 | 不相干 | 这是里程碑十二分组表的「3 覆盖」行，「都依赖二」，状态未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/13-thirteenth-txn.md:30 | 不相干 | 这是里程碑十三分组表的「3 覆盖」行，「依赖一、二」，状态未开，与里程碑四无关。 |
| F31 | .claude/kb/milestone/14-fourteenth-txn.md:28 | 不相干 | 这是里程碑十四分组表的「3 覆盖」行，「依赖一、二」，状态未开，与里程碑四无关。 |
| F32 | .claude/kb/checks-owed.en.md:99 | 要改 | 英文译文 C113 行同中文原行，该行的「仍欠」一句（`crates/` 里没有扫描重建的读侧，`grep -rn 'fn .*scan_rebuild' crates --include=*.rs` 零命中）已过时：`crates/singlefs-checker/src/rebuild.rs` 已合入（`choose_current_versions`、`rebuild_index_tables`，提交 0b926bd3f）。照中文行改后的句子译；该行开头「扫描重建同时找到同一逻辑身份的多个物理版本」的问题描述仍是真话。 |
| F32 | .claude/kb/checks-owed.en.md:540 | 要改 | 英文译文 C821 行「GPU judge没有扫描重建kernel」已还清：`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl` 的第三个入口 `scan_rebuild_main` 判 I-1.13，`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 现为 `&[]`（现查 `grep -n 'LEFT_UNSET_BY_THE_GPU_KERNELS: &\[usize\]' crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 得 `= &[]`），对拍用例 `crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 入库；照中文行改。 |
| F32 | .claude/kb/checks-owed.ja.md:99 | 要改 | 日文译文 C113 行同中文原行，「仍欠」一句（`crates/` 里没有扫描重建的读侧）已过时，`crates/singlefs-checker/src/rebuild.rs` 已合入（提交 0b926bd3f）；照中文行改后的句子译，开头的问题描述不动。 |
| F32 | .claude/kb/checks-owed.ja.md:540 | 要改 | 日文译文 C821 行「GPU 判定器没有扫描重建カーネル」已还清，理由同中文行：扫描内核第三个入口 `scan_rebuild_main` 判 I-1.13，`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 现为 `&[]`；照中文行改。 |
| F32 | .claude/kb/checks-owed.md:99 | 要改 | C113 行末「**仍欠**：`crates/` 里没有扫描重建的读侧（`grep -rn 'fn .*scan_rebuild' crates --include=*.rs` 零命中，账在 C768……），不在第 50 批里」已过时：池级 checker 有了只读扫描重建 `crates/singlefs-checker/src/rebuild.rs`（`choose_current_versions` 择现行版本，`rebuild_index_tables` 重建三张派生表），比对在 `walk.rs` 末尾的 I-1.13（提交 0b926bd3f）。改后：**仍欠**：扫描重建的读侧第一版已在池级 checker（`crates/singlefs-checker/src/rebuild.rs`，只读、不写回，三张派生表：中央映射、每个 inode 的数据单元集合、inode 记录集合），分配记录树、记账树、deadlist 与 livelist 不重建，写回（救援）不在出口；条款没定的格交回 `ScanRebuildNotDecided` 报不适用，账在 C768 等登记行。 |
| F32 | .claude/kb/checks-owed.md:536 | 要改 | C821「GPU 判器没有扫描重建内核」已还清，该行改成已还清表里的一行：扫描内核源码新增第三个入口 `scan_rebuild_main`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl`），判 I-1.13；`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 现为空表；对拍用例 `crates/singlefs-checker-tier/tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 入库（提交 0b926bd3f）。该用例在 GPU 上绿不绿我没跑（重型），销账前由主 agent 跑或看提交时的结果。 |
| F32 | .claude/kb/decisions/18-块里携带什么信息.md:134 | 要改 | D18 射程里「而 `crates/` 里没有扫描重建（`grep -rn "扫描重建" crates --include=*.rs` 零命中）」已过时：现查同一条命令 `grep -rn "扫描重建" crates --include=*.rs ｜ wc -l` 得 235 行，`crates/singlefs-checker/src/rebuild.rs` 是只读扫描重建。改后：「而 `crates/singlefs-checker/src/rebuild.rs` 的扫描重建是只读的第一版（不写回），第一版的 checker（`crates/singlefs-checker`）不开加密、不判 A6（AAD 期望值不来自明文侧）」。 |
| F32 | .claude/kb/experiments/177-逐树不写墓碑的复活数.md:9 | 不相干 | E177 页这一行讲实验回答的是哪个岔路、扫描重建「会复活几条」，是实验问题的描述，扫描重建有没有实现不改它的真假。 |
| F32 | .claude/kb/experiments/177-逐树不写墓碑的复活数.md:287 | 事件句不改 | 「读侧是模型：`crates/` 里没有扫描重建的读侧（登记第三节 S17）」写的是 E177 当时装置用模型当读侧，是那次实验的口径；装置没变，换成新说法「读侧是模型」这句话仍要成立，所以不改。 |
| F32 | .claude/kb/milestone/04-fourth-txn.md:27 | 不相干 | 里程碑四「不在出口」里的「扫描重建的写回（救援）」仍是真话：`crates/singlefs-checker/src/rebuild.rs` 只读重建，写回不在出口，与新事实一致。 |
| F32 | records/2026-10-11-里程碑五打包与决策进度列.md:107 | 事件句不改 | 2026-10-11 里程碑五规划记录里「十四　scrub、救援与单盘 ditto」一行是当时的里程碑拆分与估计，说的是那一天的规划，不是现状。 |
| F32 | records/2026-10-18-里程碑三收尾与近期欠账.md:165 | 事件句不改 | 「POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS = [92]（I-1.13，C821……）」是 2026-10-18 调查员交回当天主 agent 现查到的值，说的是那一次的发现。 |
| F32 | records/2026-10-18-里程碑三收尾与近期欠账.md:166 | 事件句不改 | 「用户定双机导入冲突的岔路：「补 GPU 扫描重建内核」（还 C821……）」是 2026-10-18 的用户定案与当天的回话，不是现状。 |
| F32 | records/2026-10-18-里程碑三收尾与近期欠账.md:239 | 事件句不改 | 「b6 告：映射条目表那件也打进主工作区，可以以此刻主工作区为底派 C821」是 2026-10-18 的派活记录，说的是当天的计划与量到的展开量。 |
| F33 | .claude/kb/decisions.md:211 | 不相干 | 「3. 算法：ChaCha20-Poly1305 vs AES-256-GCM —— 已定 · 未实现」说的是加密算法决策的进度，与整理有没有代码无关。 |
| F33 | .claude/kb/decisions/26-后台整理与放置回收.md:184 | 要改 | 「自建那条线只有三个只做计算的模块……没有整理的触发、发布入口或搬数据的代码。`grep -rln "整理" crates --include=*.rs` 命中 11 份文件」已过时：现查 `grep -rln "整理" crates --include=*.rs 加 wc -l` 得 65；`crates/singlefs-core/src/transaction.rs:9447` 有 `publish_relocation_batch`，`crates/singlefs-core/src/mounted_session.rs` 有 `run_compaction` 与用户改动后跑至多一批的触发 `compaction_starts_after_a_user_change`，另有 `system_configuration_repair.rs`。改后：交给盘那条线在 `crates/` 里不存在；自建那条线在 core 里有意图条目格式（`logged_operations.rs`）、区域扫描（`region_scan.rs`）、选段与计划（`compaction_plan.rs`）、发布入口（`transaction.rs` 的 `publish_relocation_batch`）与显式入口 `MountedSession::run_compaction`；批大小条款没定，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。后面「命中 N 份文件」的清单按现查的数重列，或删掉数。 |
| F33 | .claude/kb/milestone/04-fourth-txn.md:59 | 要改 | 「代码：……只把已释放的落点放回空闲、不搬活数据」与「`grep -rn "discard" crates/singlefs-core` 零命中」已过时：现查同一条 grep 得 184 行（`block_device.rs` 第七个动作 discard，`mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`）；「GC 意图的载体是 logged_ops 树……今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`……没有写者」也过时：`crates/singlefs-core/src/logged_operations.rs` 与 `transaction.rs` 的整理发布路径已有写者。改后：把「仓里已有的」这一格的代码句换成现状（意图条目格式 `logged_operations.rs`、`compaction_plan.rs`、`region_scan.rs`、`publish_relocation_batch`、`run_compaction`，discard 已进 `BlockDevice`；产品路径在批大小条款没定处停下），删掉两处「零命中」「没有写者」；「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」是 D3 登记表的逐字引文，随 D3 那一行是否已重开另判。「E10 未跑」另属 E10 那一组判。 |
| F33 | .claude/kb/verification-build.md:71 | 要改 | 「整理落点那一行还没有（整理还没进代码，C146 ① 仍开着）」里「整理还没进代码」已过时：整理已进 core 代码（`transaction.rs` 的 `publish_relocation_batch`、`mounted_session.rs` 的 `run_compaction`），产品路径在批大小条款没定处停下。改后：整理落点那一行还没有（整理已进 core 代码但产品路径在批大小条款没定处停下，C146（无空段时的回落政策全仓无定义） ① 仍开着）。 |
| F33 | records/2026-09-13-总审核.md:254 | 事件句不改 | 2026-09-13 总审核记录里「`(全空聚簇段数 ≥ N₀)` 第一版 N₀ = 0（整理未实现，写明重开条件）」是当天对登记位的收件结论，说的是那一天的状态。 |
| F33 | records/2026-10-11-里程碑五打包与决策进度列.md:24 | 事件句不改 | 2026-10-11 记录写进 decisions.md「进度」一段的判法与「整理本身没有，记未实现」是当天对 D26 已定项 1 的判，换成新说法这句仍是那天的判；D26 进度列现在要不要改另核（见报告额外发现）。 |
| F33 | records/2026-10-13-里程碑四预开发.md:71 | 事件句不改 | 「整理代码还没有」出现在 2026-10-13 欠账写回员乙交回的转述里，说的是那一天不搬 C373 的理由，是事件记述。 |
| F34 | .claude/kb/checks-owed.md:101 | 不相干 | C116 行这一处命中的是「合并发布的每个写请求后截断还没有流罩着」，这里的截断是崩溃点重放在写请求后切断，不是文件截断（truncate）有没有代码，与这一事实无关。 |
| F34 | .claude/kb/decisions.md:174 | 不相干 | 「18. 克隆 / 可写快照要加三条 —— 已定 · 未实现」管的是克隆与可写快照，仍没有代码（README.md:49 写「没有可写快照」），仍是真话。 |
| F34 | .claude/kb/decisions.md:175 | 要人看 | 「19. 生死判定的运行时形态与销毁快照的代价 —— 已定 · 未实现」：该项正文（`.claude/kb/decisions/05-快照-空间记账机制.md:29`）是「先问旁表，计数大于 1 就减计数，否则 `birth > previous_snapshot_txg` 即释放、余者进 deadlist」。源码已按「D5 已定项 19」点名（`crates/singlefs-core/src/records.rs:592`、`transaction.rs:5993`），销毁快照也有了（`publish_snapshot_destruction_intent`）；但稀疏旁表只有注册、没有写者（`grep -n sparse_side_table crates/singlefs-core/src/*.rs` 只有 recovery.rs:3211 与 transaction.rs 的树标识符字段），「先问旁表」那一半没实现。进度记已实现还是仍未实现，要主 agent 定；定了改 05 正文这一项的进度标记后跑 `doc-decisions.sh --write`。 |
| F34 | .claude/kb/decisions.md:297 | 要人看 | 「14. 快照发布点的位置 —— 已定 · 未实现」：D16 已定项 14 写「与 checkpoint 边界对齐，钉在这个 checkpoint 全部数据变更之后；同窗两个快照用 `(txg, 快照 id)` 打破并列」。`publish_create_snapshot` 已是独立发布（`mounted_session.rs` 的 `UserChange::CreateSnapshot` 注释「再发一次独立发布，不与别的改动合在一起」），但 `grep -rn '快照发布点' crates --include=*.rs` 与 `grep -rn '同窗' crates/singlefs-core/src/*.rs` 都零命中，没有源码按这一项点名，同窗并列的打破也没找到。进度要不要改已实现，由主 agent 核后定。 |
| F34 | .claude/kb/decisions/05-快照-空间记账机制.md:31 | 要改 | D5 已定项 21「快照列表要不要一个可枚举的持久结构」进度写「未实现」已过时：核心层建快照会往树表写码 14 条目（`transaction::publish_create_snapshot`，提交 0b926bd3f），I-3.7（活快照可枚举）已实现（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.7 状态格「已实现（2026-10-15……`IMPLEMENTED_INVARIANTS` 下标 94」）。改后：把这一行末尾的 `**进度：未实现。**` 改成 `**进度：已实现。**`，再跑 `bash .claude/gate.d/doc-decisions.sh --write` 重生成 decisions.md 的进度列。 |
| F34 | .claude/kb/decisions/05-快照-空间记账机制.md:456 | 要改 | 「I-3.7（活快照可枚举） 的 checker 未实现」已过时：`crates/singlefs-checker/invariants/05-空间记账.md` I-3.7 的状态格写「已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions`……下标 94」。同一句前面的「全部根的 txg 列表（没有）」也要跟着改：快照的两条树表条目（码 14 快照 inode 根、码 15 快照 extent 根）就是列表。改后：I-3.7（活快照可枚举） 的 checker 已实现（2026-10-15）；全部根的 txg 列表现在是树表里不带销毁中记号的码 14 条目集合。 |
| F34 | .claude/kb/decisions/08-核心索引结构.md:403 | 不相干 | 「没有快照引用 A–D」是 2026-09-19 对 D16 已定项 14 的引用，用来说明 A–D 可以不落，说的是那条理由，不是 core 有没有快照代码。 |
| F34 | .claude/kb/experiments/177-逐树不写墓碑的复活数.md:288 | 事件句不改 | 「只有单头、干净状态：没有快照与克隆头、没有崩溃支线」写的是 E177 当时装置的覆盖范围，是那次实验的口径，换成新说法也仍是它当时没覆盖。 |
| F34 | .claude/kb/experiments/182-被抛弃根读不出的三条岔路.md:342 | 不相干 | 「没有采样、没有截断」里的截断是不裁掉取态序列，与文件截断（truncate）无关。 |
| F34 | .claude/kb/experiments/35-多可写头两种形态.md:11 | 不相干 | 「`key = (locality_id, inode, offset)` —— 三段，没有快照维」是 D8 已定项 3 的 key 形态，快照靠另一棵树的版本，不因 core 加了快照代码而变。 |
| F34 | .claude/kb/milestone/02-second-txn.md:126 | 事件句不改 | 里程碑二作业点说明里「没有快照，释放直接走，不进 deadlist」是里程碑二（2026-09-29 判定结束）那一段的范围，说的是那一阶段没有快照，不是现状。 |
| F34 | .claude/kb/milestone/02-second-txn.md:132 | 事件句不改 | 「第一版没有快照，直接释放，一条都不写」同样是里程碑二那一段的范围描述；里程碑二已判定结束，这句话说的是那一阶段。 |
| F34 | .claude/kb/milestone/02-second-txn.md:143 | 不相干 | 「没有快照的释放直接进 defer、一条 deadlist 条目都不写」转述的是 D3 已定项 7 的规则：没有快照时释放直接进 defer，这条规则本身没变，不受快照代码入库影响。 |
| F34 | .claude/kb/milestone/04-fourth-txn.md:44 | 要改 | 这一格「仓里已有的」里几处现状句已过时：「没有任何写它们的操作」「I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 未实现」「删除批与覆盖写今天共用……没有按快照分支，也没有往 deadlist 写的路径」。现查：`crates/singlefs-core/src/transaction.rs` 有 `publish_create_snapshot`（9207）、`publish_snapshot_destruction_intent`（9285），被快照引用的块进 deadlist（5993 行注释「D5 已定项 19」）；I-3.6、I-3.7 在 `invariants/05-空间记账.md` 状态格都是「已实现（2026-10-15」；稀疏旁表仍只有注册、没有写者。改后：deadlist 与 livelist 已有写者（建快照、快照下覆盖写与删文件、销毁快照），稀疏旁表仍只注册、没有写者；`grep -rn "克隆" crates/singlefs-core/src` 仍只命中那条 TODO 注释，没有克隆代码；I-3.6、I-3.7 已实现（2026-10-15），I-3.5 部分实现（以状态格为准）；删除批与覆盖写共用的释放按快照分支（一·2 已改）。 |
| F34 | .claude/kb/milestone/05-fifth-txn.md:52 | 要改 | 「判决 `research/prompts/m4-trunc-r2-main-verification.md` 给截断定了位 2，代码还没写」已过时：`crates/singlefs-core/src/system_configuration.rs:35` 已有 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04`，且在 `SUPPORTED_INCOMPAT_BITS` 里。改后：`crates/singlefs-core/src/system_configuration.rs`：位 0（退役）、位 1、位 2（截断字段，`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT`）在用 ｜ 二·2 取下一个空位（位 3），开工时现查。 |
| F34 | .claude/kb/prior-art.md:838 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，不是本工程的 core 有没有快照代码。 |
| F34 | .claude/kb/prior-art.md:839 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程 core 的快照代码无关。 |
| F34 | .claude/kb/prior-art.md:840 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:841 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:842 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:843 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:844 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:845 | 不相干 | 「F2FS 没有快照，GC 从不处理共享块」说的是 F2FS，与本工程无关。 |
| F34 | .claude/kb/prior-art.md:891 | 不相干 | 「段式回收与本工程相近；F2FS 没有快照」说的是 F2FS，与本工程 core 的快照代码无关。 |
| F34 | .claude/kb/prior-art.md:902 | 不相干 | 「F2FS（没有快照）」是对别家文件系统的查证结论，「没有」只覆盖所列六家，不涉本工程。 |
| F34 | README.md:49 | 不相干 | README.md:49「没有可写快照、加密、压缩」已是新说法，与结束那一版一致，仍是真话。 |
| F34 | records/2026-08-29-组合对攻轮.md:226 | 事件句不改 | 2026-08-26 的 D8 已定项 3 引文「三段，没有快照维」是组合对攻轮当时的转述，说的是那一次的论证。 |
| F34 | records/2026-10-13-里程碑四预开发.md:156 | 事件句不改 | 「kb 写回员交回……D16 进度已实现 14 → 13、未实现 2 → 3」是 2026-10-15 前后一次写回的记录，说的是那一次改了什么。 |
| F34 | records/2026-10-13-里程碑四预开发.md:226 | 事件句不改 | 「代码轮攻方腿交回……T5 整理搬过单元之后回退」是攻方腿当次报告的摘记，说的是那一次的发现。 |
| F34 | records/2026-10-21-doc门禁违规修复.md:43 | 事件句不改 | 2026-10-21 记录里「不能做的把「没有快照」改成「没有可写快照」」是这一轮对 README 做过的改动，与新说法一致，是事件记述。 |
| F35 | .claude/kb/checks-owed.en.md:249 | 不相干 | 英文译文 C337 行讲被销毁的可写头（克隆头）何时不算活引用没有条款，仍是真话；快照这一半已在行后文的「已定项 20 例外句已改成「不再钉住块」」里写成新说法。 |
| F35 | .claude/kb/checks-owed.ja.md:249 | 不相干 | 日文译文 C337 行同中文行，后文已写「確定項目 20 例外句已改成「不再钉住块」」，克隆头那一半仍没有条款，仍是真话。 |
| F35 | .claude/kb/checks-owed.md:248 | 不相干 | C337 行已写新说法：「D5（快照 / 空间记账机制） 已定项 20 例外句已改成「不再钉住块」」，克隆头那一半仍欠，与结束那一版一致。 |
| F35 | .claude/kb/decisions/05-快照-空间记账机制.md:287 | 不相干 | D5 射程「被销毁的快照从写意图那一刻起不再钉住块，这是已定项 20「待删除结构算作活引用」的例外」已是新说法；「条目形态定了，会红的检查还没有」指克隆头那一半，仍是真话。 |
| F35 | .claude/kb/decisions/05-快照-空间记账机制.md:441 | 不相干 | D5 已定项 20 定案「例外：被销毁的快照从写意图那一刻起不再钉住块（已定项 12）」已是新说法，仍是真话。 |
| F35 | .claude/kb/decisions/05-快照-空间记账机制.md:443 | 不相干 | D5 已定项 20 射程讲 rm 之后建的快照会钉住待删 extent、另一条路会破坏 O(1)，并写「被销毁的可写头从哪一刻起不算活引用……没有条款」，克隆头那一半仍没有条款，仍是真话。 |
| F35 | .claude/kb/decisions/05-快照-空间记账机制.md:448 | 事件句不改 | 「判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S9：「活引用」在已定项 14、17 与已定项 20 是两个口径，例外句改成「不再钉住块」」是依据栏记那一份判决改了什么，说的是那一次的判决。 |
| F35 | .claude/kb/decisions/05-快照-空间记账机制.md:450 | 不相干 | 「欠：C337（被销毁的头何时不算活引用没有条款）」仍是真话：克隆头那一半仍没有条款。 |
| F35 | .claude/kb/decisions/16-publish-semantics.en.md:66 | 不相干 | 英文译文「implemented in milestone 4 batch 3丙2: core `crates/singlefs-core/src/mount.rs` `user_visible_trees_changed`」与现查一致：`mount.rs:3509` 的 `user_visible_trees_changed` 已比 `snapshot_tree_roots`；「prior to merge」是过去时，仍是真话。 |
| F35 | .claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:66 | 不相干 | 日文译文「マージ前は `user_visible_trees_changed` は inode ツリーと extent ツリーのみを比較していた」是过去时，现行 `user_visible_trees_changed` 已比快照树根（`mount.rs:3515`），与新事实一致。 |
| F35 | .claude/kb/decisions/16-发布语义.md:66 | 不相干 | 中文「合入之前 `user_visible_trees_changed` 只比 inode 树与 extent 树」是过去时，现行代码三样都比（`crates/singlefs-core/src/mount.rs:3513-3515`），这一行与结束那一版一致。同一条款的进度标记（D16 已定项 1 仍写「未实现」）另见报告额外发现。 |
| F35 | .claude/kb/milestone/04-fourth-txn.md:44 | 要改 | 同 F34 第 14 行：这一格「仓里已有的」写「没有任何写它们的操作」「I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 未实现」，而核心层已能建快照、销毁快照并往 deadlist 写（`transaction.rs` 的 `publish_create_snapshot`、`publish_snapshot_destruction_intent`），每个快照有码 14、码 15 两条树表条目，I-3.6、I-3.7 在 `invariants/05-空间记账.md` 状态格都是「已实现（2026-10-15」。改后句子见 F34 第 14 行。 |
| F35 | .claude/kb/milestone/14-fourteenth-txn.md:44 | 不相干 | 里程碑十四的「仓里已有的」讲 D6 已定项与可写头（克隆头）的欠账，「C337（被销毁的头何时不算活引用没有条款）」仍欠，不受只读快照落地影响。 |
| F35 | .claude/kb/milestone/14-fourteenth-txn.md:50 | 不相干 | 里程碑十四二·2「销毁头：分叉点闸、分批续做、释放边界」是可写头的销毁，列的 C337 仍欠，与只读快照销毁无关。 |
| F35 | records/2026-10-13-里程碑四预开发.md:149 | 事件句不改 | 2026-10-14 用户弹窗四问的回答，「建快照那次发布「算非空」」是那一天的决定，已在代码里落地（`mount.rs:3515`），这句仍是那天定了什么。 |
| F36 | .claude/kb/milestone/04-fourth-txn.md:44 | 要改 | 同 F34 第 14 行：这一格「仓里已有的」里「没有任何写它们的操作」「I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 未实现」已过时（核心层已能建、销毁快照并往 deadlist 写；两条不变量已实现）。这一行里「C301（deadlist 与 livelist 的态别没写）」列在相关欠账里仍可留着：登记行写态别已定 deadlist 一半（权威态），仍欠 livelist 那一半与 checker。改后句子见 F34 第 14 行。 |
| F36 | .claude/kb/milestone/04-fourth-txn.md:50 | 不相干 | 一·2 压着的欠账里列「C301（deadlist 与 livelist 的态别没写）」仍是真话：登记行（`checks-owed.md:230`）写「仍欠 livelist 那一半（可写克隆）与 checker」，这一行只是点名它压着一·2，不断言它没定。 |
| F36 | .claude/kb/milestone/04-fourth-txn.md:86 | 要改 | 「checker 里没有用扫描结果重建索引的代码（2026-10-12 现查）」去掉日期后读成今天的样子，已不是真话：`crates/singlefs-checker/src/rebuild.rs` 合入（提交 0b926bd3f），`walk.rs` 末尾比对 I-1.13。改后：checker 的扫描方向按候选槽扫……供 I-7.8、I-1.8 与 journal 那几条判定用，另有只读扫描重建 `rebuild.rs` 用扫描结果重建三张派生表（中央映射、每个 inode 的数据单元集合、inode 记录集合），与遍历方向的比对在 `walk.rs` 末尾（I-1.13）。其中「C301（deadlist 与 livelist 的态别没写）」一处不变，仍欠 livelist 那一半。 |
| F36 | .claude/kb/milestone/04-fourth-txn.md:92 | 要改 | 四·2 依赖栏「deadlist 与 livelist 那一部分等 C301（deadlist 与 livelist 的态别没写） 定」已部分过时：C301 登记行写 deadlist 条目与被钉住单元的映射条目是权威态、销毁队列是派生态，已定；仍欠 livelist（可写克隆）那一半。改后：四·1；deadlist 条目与被钉住单元的映射条目是权威态（C301 已定这一半，扫描不重建它们），销毁队列（派生态）第一版不重建、不比对，livelist 那一部分等 C301 的另一半定。 |
| F36 | .claude/kb/milestone/04-fourth-txn.md:104 | 要改 | 「deadlist / livelist 等快照落地之后、C301（deadlist 与 livelist 的态别没写） 定了再比」已过时：快照已落地（提交 0b926bd3f），C301 的 deadlist 一半已定（权威态；销毁队列是派生态，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S9）。改后：分配记录树、记账树是派生态（D21 已定项 5），第一版不重建、不比对（账在 C122、C413）；deadlist 条目与被钉住单元的映射条目是权威态，不由扫描重建；销毁队列（带销毁中记号的码 14 条目）是派生态，第一版不重建、不比对；livelist（可写克隆）等 C301 的另一半定了再比。 |
| F37 | .claude/kb/checks-owed.en.md:117 | 不相干 | 英文译文 C140 行「意图完成不等于空段被腾出」讲目标态达成不等于 R 变成全空，前置格已是新说法（停机谓词改成 R 本地、全局差值只决定签不签下一条），仍欠的 checker 没变，仍是真话。 |
| F37 | .claude/kb/checks-owed.en.md:118 | 要改 | 英文译文 C141 行「停机criterion的输入不持久」的修法格仍写「换成产出谓词……全空聚簇段数(现在) − segs0 ≥ 1」并以终止时产出谓词取值相同为检查形态，仍把产出谓词当停机判据，已过时：D26 已定项 1 第 2 条现为 R 本地谓词（删意图那一刻 R 按记录全空），全局差值只决定签不签下一条，segs0 只作观测量（`.claude/kb/decisions/26-后台整理与放置回收.md:29`）。C141 不在 2026-10-13 改写的八行之列。改后：把「换成产出谓词」一段改成上面的新口径，检查形态改为：断言中断 k 次与不中断的一批整理，删意图那一刻 R 按记录全空的取值相同、签不签下一条的取值相同；判别力自证（把 segs0 改成每次续做时重取当前值，断言必须由绿转红）可保留。「2026-09-06 撤回」是那天的事，不改。照中文行。 |
| F37 | .claude/kb/checks-owed.en.md:191 | 要改 | 英文译文 C243 行「停机谓词那个量从来没有过轨迹」的实现侧自证格写「后台整理进代码：`git grep -n -i 'scrub\｜migrat'` 零命中（2026-10-11 现查），实现侧自证没有被测对象」，去掉日期读成今天的样子已不是真话：整理已进 core（`git grep -n -i compaction -- 'crates/*/src/**'` 现得 1415 行，`transaction.rs:9447` 的 `publish_relocation_batch`、`mounted_session.rs` 的 `run_compaction`）。改后：后台整理已进 core 代码（产品路径在批大小条款没定处停下），实现侧自证有了被测对象，但 `verification-build.md` 必红用例表里关掉「R 在意图期内不可作分配目的地」那一行还没做，仍欠。照中文行。 |
| F37 | .claude/kb/checks-owed.en.md:268 | 要改 | 英文译文 C364 行「整理意图记录没有字段表」的前置格写「仍欠实现（`logged_operations.rs` 在第 2 批补丁里）与 I-4.4 的判法（字段白名单）」，两项都已做：`crates/singlefs-core/src/logged_operations.rs` 已合入（提交 0b926bd3f），I-4.4 在 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md` 状态格写「已实现（2026-10-15 起判 logged_ops 里的条目」。改后前置格：字段表已定且已实现（key 10、value 34、条目 44，`logged_operations.rs`），I-4.4 的白名单判法已实现；是否还欠「门禁登记位逐字段比对」以「怎么拦」格为准，由主 agent 核后决定销账。照中文行。 |
| F37 | .claude/kb/checks-owed.en.md:273 | 要改 | 英文译文 C373 行「停机谓词判真时活意图删不删」的前置格末尾「`crates/` 里还没有整理代码」已过时：整理代码已在（`transaction.rs` 的 `publish_relocation_batch`、`mounted_session.rs` 的 `run_compaction`），崩溃流 `crash_enumeration_compaction_relocation_stream.rs` 入库；「删意图那一刻 R 内没有分配代 ≤ c0 的未释放落点」逐步断言与有界批数检查按名字搜不到（`grep -c C373` 两个整理崩溃流文件零命中；零命中不等于没有该断言，要主 agent 读流文件核），先按仍欠写。改后：仍欠整理的崩溃点重放里那条逐步断言与有界批数的检查（整理代码与崩溃流已入库，断言尚未加）。照中文行。 |
| F37 | .claude/kb/checks-owed.ja.md:117 | 不相干 | 日文译文 C140 行「意图完成不等于空段被腾出」同英文行：前置格已是新说法，仍欠的 checker 不变，仍是真话。 |
| F37 | .claude/kb/checks-owed.ja.md:118 | 要改 | 日文译文 C141 行「停机判定基準的输入不持久」同英文行：修法格仍把产出谓词当停机判据（「换成产出谓词」），已过时，D26 已定项 1 第 2 条现为 R 本地谓词、全局差值只决定签不签下一条；改法照英文行（中文行）。 |
| F37 | .claude/kb/checks-owed.ja.md:191 | 要改 | 日文译文 C243 行「停机谓词那个量从来没有过轨迹」同英文行：「实现侧自证没有被测对象」已过时，整理已进 core 代码；改法照英文行（中文行）。 |
| F37 | .claude/kb/checks-owed.ja.md:268 | 要改 | 日文译文 C364 行「整理意图记录没有字段表」同英文行：前置格「仍欠实现……与 I-4.4 的判法」两项已做；改法照英文行（中文行）。 |
| F37 | .claude/kb/checks-owed.ja.md:273 | 要改 | 日文译文 C373 行「停机谓词判真时活意图删不删」同英文行：前置格「`crates/` 里还没有整理代码」已过时；改法照英文行（中文行）。 |
| F37 | .claude/kb/checks-owed.md:117 | 不相干 | C140 行「意图完成不等于空段被腾出」的前置格已写新说法：「停机谓词改成 `R` 本地（删意图那一刻 `R` 按记录全空）加签发前判空洞够不够，全局差值只决定签不签下一条」，与结束那一版一致。 |
| F37 | .claude/kb/checks-owed.md:118 | 要改 | C141 行「停机判据的输入不持久」的修法格仍写「换成**产出谓词**：`全空聚簇段数(现在) − segs0 ≥ 1`」并以「终止时的产出谓词取值相同」为检查形态，把产出谓词当停机判据，已过时：D26 已定项 1 第 2 条现为「停机：删意图那一刻 `R` 按记录全空」（`R` 本地谓词），全局差值只决定签不签下一条，`segs0` 仍以值写进意图记录、只作观测量（`decisions/26-后台整理与放置回收.md:29`）。2026-10-13 改写的八行（C364、C372、C140、C417、C301、C336、C337、C142）里没有 C141。改后：把「换成产出谓词」一段改成上面的新口径；检查形态改为：断言中断 k 次与不中断的一批整理，删意图那一刻 `R` 按记录全空的取值相同、签不签下一条的取值相同；判别力自证（把 `segs0` 改成每次续做时重取当前值，断言必须由绿转红）可保留；「2026-09-06 撤回」是那天的事，不改。 |
| F37 | .claude/kb/checks-owed.md:191 | 要改 | C243 行实现侧自证格写「后台整理（D26）进代码：`git grep -n -i 'scrub\｜migrat' -- 'crates/*/src/**'` 零命中（2026-10-11 现查），实现侧自证没有被测对象」，去掉日期读成今天的样子已不是真话：现查 `git grep -n -i compaction -- 'crates/*/src/**' ｜ wc -l` 得 1415，整理已进 core（`transaction.rs:9447` 的 `publish_relocation_batch`、`mounted_session.rs` 的 `run_compaction`）。同一格末尾「等整理进第一版」也已过时。改后：后台整理已进 core 代码（产品路径在批大小条款没定处停下），实现侧自证有了被测对象，但 `verification-build.md` 必红用例表第九条（关掉「`R` 在意图期内不可作分配目的地」）还没做，仍欠。 |
| F37 | .claude/kb/checks-owed.md:267 | 要改 | C364 行前置格「字段表已定（……）；仍欠实现（`crates/singlefs-core/src/logged_operations.rs` 在第 2 批补丁里）与 I-4.4（意图只描述目标态） 的判法（字段白名单）」两项都已做：`logged_operations.rs` 已合入（提交 0b926bd3f），I-4.4（`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md`）状态格现查写「已实现（2026-10-15 起判 logged_ops 里的条目……白名单」。改后：字段表已定且已实现（key 10、value 34、条目 44，`crates/singlefs-core/src/logged_operations.rs`），I-4.4 的字段白名单判法已实现；「怎么拦」格的门禁登记位逐字段比对是否已有，由主 agent 核后决定销账。 |
| F37 | .claude/kb/checks-owed.md:272 | 要改 | C373 行前置格「仍欠整理的崩溃点重放里那条逐步断言与有界批数的检查，`crates/` 里还没有整理代码」后半句已过时：整理代码已在（`crates/singlefs-core/src/transaction.rs:9447` 的 `publish_relocation_batch`、`mounted_session.rs` 的 `run_compaction`），整理崩溃流 `crash_enumeration_compaction_relocation_stream.rs` 已入库；逐步断言与有界批数检查按名字搜不到（`grep -c C373` 两个整理崩溃流文件零命中；零命中不等于没有该断言，要主 agent 读流文件核），先按仍欠写。改后：仍欠整理的崩溃点重放里那条逐步断言与有界批数的检查（整理代码与崩溃流已入库，断言尚未加）。 |
| F37 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:29 | 不相干 | 英文译文已定项 1 第 2 条「if an intention completes but global difference `completely empty cluster segment count(now) − segs0 ≥ 1` is false, the next one is not issued」已是新口径（R 本地谓词停机、全局差值只决定签不签下一条），仍是真话。 |
| F37 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:32 | 不相干 | 英文译文「`segs0` resides in intention record while new-pool file creation contains no compaction intentions」只讲 segs0 住意图记录，不涉停机谓词取什么，仍是真话。 |
| F37 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:66 | 不相干 | 英文译文字段表行「`segs0` 8 30 Value for disk where `R` resides in accounting Item 11」只讲 segs0 的位置、宽度和来源，「observable only」已是新口径，仍是真话。 |
| F37 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:29 | 不相干 | 日文译文已定项 1 第 2 条「1つの意図が完了しても全体差分 `完全空クラスタセグメント数(現在) − segs0 ≥ 1` が偽であれば次を発行しない」已是新口径，仍是真话。 |
| F37 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:32 | 不相干 | 日文译文「`segs0` は意図レコードに格納されるが新規プールでのファイル作成には整理意図が含まれない」只讲 segs0 住意图记录，仍是真话。 |
| F37 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:66 | 不相干 | 日文译文字段表行「発行時のパブリッシュが書き込んだ会計第11項」「観測量としてのみ機能」已是新口径，仍是真话。 |
| F37 | .claude/kb/decisions/26-后台整理与放置回收.md:29 | 不相干 | 中文已定项 1 第 2 条「停机：删意图那一刻 `R` 按记录全空」「一条意图完成而全局差值……为假就不签下一条；`segs0` 仍以值写进意图记录，只作观测量」本身就是新口径，与结束那一版一致。 |
| F37 | .claude/kb/decisions/26-后台整理与放置回收.md:32 | 不相干 | 「`segs0` 住意图记录而新池新建文件不含整理意图」只讲 segs0 的住处，不涉停机谓词取什么，仍是真话。 |
| F37 | .claude/kb/decisions/26-后台整理与放置回收.md:34 | 不相干 | 射程「意图达成之后有没有产出由第 2 条的 `R` 本地谓词判，全局差值只决定签不签下一条」已是新口径，仍是真话。 |
| F37 | .claude/kb/decisions/26-后台整理与放置回收.md:40 | 事件句不改 | 依据栏「用户定案（三水位、产出谓词、`R` 排他），原话在变更史」是记用户当时定了什么，说的是那次定案，不是现状句。 |
| F37 | .claude/kb/decisions/26-后台整理与放置回收.md:66 | 不相干 | 字段表行「签发那次发布写下的记账第 11 项（全空聚簇段数……）里 `R` 所在盘的值，只作观测量」与 `crates/singlefs-core/src/logged_operations.rs:139` 的 `segs0` 注释同一句，仍是真话。 |
| F37 | records/2026-09-10-D3项8第一轮.md:38 | 事件句不改 | 2026-09-10 记录「D26 已定项 1 的停机谓词是 `全空聚簇段数(现在) − segs0 ≥ 1`」是那一轮核查当天对条款的转述，说的是那一天条款的样子。 |
| F37 | records/2026-10-01-删除与并发事务.md:185 | 事件句不改 | 2026-10-01 记录里「已定项 1 第 2 条「segs0 必须以『值』写进意图记录」」是主 agent 当天现查条款原文的转述，说的是那一次。 |
| F37 | records/2026-10-13-里程碑四预开发.md:172 | 事件句不改 | 「丁 3 交回……意图上盘（`TransactionUnit::LoggedOperationsRoot`、`publish_compaction_step`」是 2026-10-13 一次交回的摘记，说的是那次做了什么。 |
| F37 | records/2026-10-13-里程碑四预开发.md:225 | 事件句不改 | 代码轮攻方腿交回里「T6 不一致一处（签发那次发布开新聚簇段时 `segs0` 写 3310、记账第 11 项 3309」是那一次判定的发现，说的是那次。 |
| F37 | records/2026-10-13-里程碑四预开发.md:234 | 事件句不改 | 「代码轮判决写成……T6（腾空 R 那一批不删意图；`segs0` 差一）」是 2026-10-13 判决的摘记，说的是那一次打中了什么。 |
| F37 | records/2026-10-13-里程碑四预开发.md:240 | 事件句不改 | 「六条都改了：……F3 签发先算账、`segs0` 不等就换记账值重算」是 2026-10-13 一次交回的摘记，说的是那次改了什么。 |
| F37 | records/2026-10-13-里程碑四预开发.md:254 | 事件句不改 | 「代码轮第二轮开：轮名 `m4-segment-code-r2`」及其「R3 `segs0`」格是那一轮开轮的记录，说的是那一天开了什么。 |
| F37 | records/2026-10-13-里程碑四预开发.md:255 | 事件句不改 | 这一行记代码轮第二轮判决的摘记，「T6 前半 F3 照 D26 字面，但把停机谓词「全局差值」基线挪低一格」是那一次判决说的，说的是那次。 |
| F37 | records/2026-10-13-里程碑四预开发.md:259 | 事件句不改 | 「用户 2026-10-16 弹窗两题：意图里的 `segs0`「取签发之前」」是那一天的用户定案记录，说的是那次决定。 |
| F37 | records/2026-10-13-里程碑四预开发.md:262 | 事件句不改 | 「H3 模型一侧停下——模型不记不比 `segs0`（丁 3 那条读法），……主 agent 定：照旧不比」是 2026-10-13 一次交回后主 agent 当次的定案，说的是那次。 |
| F38 | .claude/kb/checks-owed.en.md:176 | 不相干 | 英文译文这一行讲 E122 装置的断言范围「empty_segments_are_pinned_to_absolute_values 写死 `stride = 2`」，是实验装置里一个测试名与输出字段 `empty_segs_at_32`，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/checks-owed.en.md:272 | 要改 | 英文译文 C372 行的前置格仍写「仍欠代码 `empty_segments` 从占用口径改成记录口径、checker 的 C413 那一半」，已过时：`crates/singlefs-core/src/allocator.rs:847` 的 `empty_segments` 现返回 `full_segments_without_an_unreleased_record`（记录口径，提交 0b926bd3f），C413 对应的 I-3.16 在 `invariants` 状态格写「已实现（2026-10-15，……`walk::judge_fragmentation_and_empty_segment_statistics`」。改后前置格：口径已收口按记录，代码 `empty_segments` 已改成记录口径，checker 的 I-3.16 已实现；销账前核「怎么拦」格三处（记账第 11 项的写行、整理的停机谓词、`DeviceFreeMap::empty_segments`）在只剩 defer 槽的段上给同一个判定、有会红的用例。照中文行。 |
| F38 | .claude/kb/checks-owed.ja.md:176 | 不相干 | 日文译文这一行讲 E122 装置的断言范围「empty_segments_are_pinned_to_absolute_values は `stride = 2` をハードコード」，是实验装置里的测试名与输出字段，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/checks-owed.ja.md:272 | 要改 | 日文译文 C372 行的前置格同英文行：「仍欠代码 `empty_segments` 从占用口径改成记录口径、checker 的 C413 那一半」已过时，代码已改记录口径（`allocator.rs:847`），I-3.16 已实现；改法照英文行（中文行）。 |
| F38 | .claude/kb/checks-owed.md:176 | 不相干 | 这一行讲 E122 装置的断言范围「装置里的断言范围一直是对的（`e122_directory_locality.rs` 的 `empty_segments_are_pinned_to_absolute_values` 写死 `stride = 2`」，是实验装置里的测试名与输出字段 `empty_segs_at_32`，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/checks-owed.md:271 | 要改 | C372 行前置格「口径已收口按记录（……）；仍欠代码 `empty_segments` 从占用口径改成记录口径、checker 的 C413（碎片度与全空段统计没有不变量） 那一半」已过时：`crates/singlefs-core/src/allocator.rs:847` 的 `empty_segments` 现返回 `full_segments_without_an_unreleased_record`（记录口径，提交 0b926bd3f）；C413 对应的 I-3.16（碎片度与全空段统计对得上）状态格写「已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_fragmentation_and_empty_segment_statistics`」。改后前置格：口径已收口按记录（D26 已定项 1 ①、D3 已定项 10 ①）；代码 `empty_segments` 已改成记录口径，checker 的 I-3.16 已实现；销账前核「怎么拦」格：记账第 11 项的写行、整理的停机谓词、`DeviceFreeMap::empty_segments` 三处在一个只剩 defer 槽的段上给同一个判定，有会红的用例（判别力自证：把实现换成另一种定义，那一格必须红）。其余「去向：挪出里程碑三，归里程碑四」不动。 |
| F38 | .claude/kb/decisions/03-space-allocation.en.md:212 | 要改 | 英文译文 D3 已定项 10 ① 里「current code `empty_segments` in `crates/singlefs-core/src/allocator.rs` uses occupancy measurement basis, to be changed to record measurement basi[s]」已过时：`allocator.rs:847` 的 `empty_segments` 现返回按记录口径维护的 `full_segments_without_an_unreleased_record`。照中文行改。 |
| F38 | .claude/kb/decisions/03-領域割り当て.ja.md:212 | 要改 | 日文译文 D3 已定项 10 ① 里「コードの現状の `empty_segments`（`crates/singlefs-core/src/allocator.rs`）は専有計測条件であり、レコード計測条件に変更する必要がある」已过时，代码已是记录口径（`allocator.rs:847`）。照中文行改。 |
| F38 | .claude/kb/decisions/03-空间分配.md:212 | 要改 | 「代码今天的 `empty_segments`（`crates/singlefs-core/src/allocator.rs`）是占用口径、要改成记录口径，C372（全空聚簇段数两份定义相反）」已过时：`allocator.rs:847` 的 `empty_segments` 现返回按记录口径维护的 `full_segments_without_an_unreleased_record`（提交 0b926bd3f）。改后：「全空聚簇段数」= 段内 64 槽都没有未释放分配记录的段数（记账行记的就是它；代码 `empty_segments`（`crates/singlefs-core/src/allocator.rs`）已是记录口径，C372（全空聚簇段数两份定义相反）的代码那一半已做）。 |
| F38 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:28 | 要改 | 英文译文 D26 已定项 1 ① 里「current `empty_segments` in code uses occupancy basis」（原文 `empty_segments` 占用口径）一句已过时：代码已是记录口径（`allocator.rs:847`）。照中文行改。 |
| F38 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:28 | 要改 | 日文译文 D26 已定项 1 ① 里「コードの現在の `empty_segments` は占有基準であり、レコード基準」一句已过时，代码已是记录口径（`allocator.rs:847`）。照中文行改。 |
| F38 | .claude/kb/decisions/26-后台整理与放置回收.md:28 | 要改 | 「代码今天的 `empty_segments` 是占用口径、要改成记录口径（C372（全空聚簇段数两份定义相反）），与 D3（空间分配） 已定项 10 ① 同一句」已过时：`allocator.rs:847` 的 `empty_segments` 现按记录口径。改后：全空聚簇段数按记录口径（段内 64 槽都没有未释放分配记录）；代码 `empty_segments` 已是记录口径（C372 的代码那一半已做），与 D3（空间分配） 已定项 10 ① 同一句。 |
| F38 | .claude/kb/experiments.md:130 | 不相干 | E106 行里「代价（自洽占用口径）甲 1544 / 乙 2511」的占用口径是条带成员表那次实验里算代价的口径，与 core 的 `empty_segments` 取记录还是占用口径无关。 |
| F38 | .claude/kb/experiments/122-benefits-bought-by-directory-based-grouping.en.md:63 | 不相干 | E122 页英文译文「Assertion scope in apparatus was always correct (`empty_segments_are_pinned_to_absolute_values` hardcoded `stride = 2`」讲实验装置的测试名，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/experiments/122-ディレクトリ集約で何が得られるか.ja.md:63 | 不相干 | E122 页日文译文「装置内のアサート範囲は常に正しかった（`empty_segments_are_pinned_to_absolute_values`」讲实验装置的测试名，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/experiments/122-按目录聚在一起买到什么.md:63 | 不相干 | E122 页「装置里的断言范围一直是对的（`empty_segments_are_pinned_to_absolute_values` 写死 `stride = 2`」讲实验装置的测试名，不是 core 的 `empty_segments` 口径。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:10 | 事件句不改 | E151 页概述里「期末全空聚簇段数在第一次跑的 40 格上恒为 0（`any_container_cell_has_empty_segments=false`）」和第二次跑的轨迹是那几次跑的结果，`any_container_cell_has_empty_segments` 是装置输出字段，不受 core 改口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:52 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=first_fit load=uniform 」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:53 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_seg load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:54 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=arrival load=uniform me」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:55 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k16 load=unif」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:56 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k64 load=unif」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:57 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=first_fit load=uniform 」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:58 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_seg load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:59 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=arrival load=uniform me」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:60 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k16 load=unif」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:61 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k64 load=unif」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:62 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=first_fit load=runs8 me」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:63 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_seg load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:64 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=arrival load=runs8 meta」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:65 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k16 load=runs」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:66 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k64 load=runs」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:67 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=first_fit load=runs8 me」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:68 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_seg load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:69 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=arrival load=runs8 meta」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:70 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k16 load=runs」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:71 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=container_k64 load=runs」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:72 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_nb_r1 load=uniform」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:73 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_nb_r1 load=runs8 m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:74 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_nb_r2 load=uniform」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:75 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_nb_r2 load=runs8 m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:76 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b32 load=unifor」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:77 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b32 load=runs8 」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:78 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b128 load=unifo」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:79 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b128 load=runs8」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:80 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b256 load=unifo」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:81 | 不相干 | E151 页的实验产物行「E7RESULT name=grid arm=bump_cp_b256 load=runs8」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:82 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=uniform met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:83 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:84 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:85 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=uniform met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:86 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:87 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:88 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=uniform met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:89 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:90 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=uniform m」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:91 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=runs8 metad」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:92 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:93 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:94 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=runs8 metad」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:95 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:96 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:97 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_arrival load=runs8 metad」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:98 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:99 | 不相干 | E151 页的实验产物行「E7RESULT name=verdict_container load=runs8 met」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:100 | 不相干 | E151 页的实验产物行「E7RESULT name=answer arrival_ties_bump_in_ever」里的 empty_segments 字样是装置输出的字段名，值是那几次跑的原样结果，不是 core 的 `DeviceFreeMap::empty_segments` 口径，不受 C372 改记录口径影响。 |
| F38 | records/2026-10-13-里程碑四预开发.md:53 | 事件句不改 | 2026-10-13 预开发记录里「空段计数只在回收那一步才涨（停机谓词按占用口径在最后一批那一刻恒假，C372 两份定义相反）」是当天实现事实调研（O1–O11）的转述，说的是那一天代码的样子。 |
| F38 | records/2026-10-13-里程碑四预开发.md:172 | 事件句不改 | 「丁 3 交回……三条做完：意图上盘（`TransactionUnit::LoggedOperationsRoot`」是 2026-10-13 一次交回的摘记，说的是那次做了什么。 |
| F38 | records/2026-10-13-里程碑四预开发.md:178 | 事件句不改 | 「主 agent 定（零轮）：① 全空段口径——条款按记录、核心层 `empty_segments` 按占用（C372 代码那一半没做）……核心层改记录口径记进验证段之后的下一批」是 2026-10-13 当天的定案与当时代码状态，说的是那一次。 |
| F38 | records/2026-10-13-里程碑四预开发.md:188 | 事件句不改 | 「还有四件已知要写的小活……核心层 `empty_segments` 改按记录口径（C372 代码那一半）」是 2026-10-13 写段收尾时的待办清单，说的是那一天，已做（`allocator.rs:847`）。 |
| F38 | records/2026-10-13-里程碑四预开发.md:189 | 事件句不改 | 「写段收尾件交回……补丁……判器调用点 7 处不改代码、登 7 行」是 2026-10-13 一次交回的摘记，说的是那次做了什么。 |
| F38 | records/2026-10-13-里程碑四预开发.md:204 | 事件句不改 | 「合取 7 被拒不推抬 F……理由按 `empty_segments` 记录口径现核——认」是 2026-10-13 一次交回后主 agent 当次认下的结论，说的是那次。 |
| F38 | records/2026-10-13-里程碑四预开发.md:253 | 事件句不改 | 「写表变化来自写段收尾件 C372（`empty_segments` 改记录口径，段 0 记账叶第 11 项每盘 3301 → 3304）」是 2026-10-13 一次交回对写表变化的解释，说的是那次的数。 |
| F38 | records/2026-10-13-里程碑四预开发.md:264 | 事件句不改 | 「甲三交回……H3：F3 的换值重算整段删、签发照写签发之前的全空段数」是 2026-10-13 一次交回的摘记，说的是那次改了什么。 |
| M1 | .claude/kb/feature-bits.md:19 | 要补 | incompat 位 2（inode 记录带截断字段）没登记：代码 `crates/singlefs-core/src/system_configuration.rs:35` 的 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04` 随提交 0b926bd3f 引入，注释写「位号的登记归阶段同步」；feature-bits.md 第 19 行仍写「incompat 分了位 0（退役）与位 1，别的位一位都没分」，位表里也没有位 2 一行，第 21 行「代码侧引用了哪几位」也没提它。要补：位表加一行（位号 2、类别 incompat、名称 INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT、引入 commit 0b926bd3f、状态在用，语义一句话要逐字含 D15 已定项 4 登记表里同一位的含义，登记表也要先加这一位），改第 19、21 行；三语译文同改。门禁 code-source-discipline 的 feature-bits 格按这张表比对，补完让主 agent 提交前那一轮判。 |
| M2 | .claude/kb/layout/04-fourth-txn.md（文件不存在） | 要补 | 里程碑四一·3、六·3 要求的新盘上形态登记没有文件：`ls .claude/kb/layout/` 只有 01、02、03 三份各三语。随提交 0b926bd3f 进代码的盘上形态没有字节登记：树表码 14（快照 inode 根）与码 15（快照 extent 根）两条条目、销毁中记号（flags 位 0）、logged_ops 整理意图条目（key 10、value 34、条目 44）与截断意图（种类 2）、incompat 位 2 与 inode 记录偏移 120 起的 T 与 N_w。`records/2026-10-13-里程碑四预开发.md:188` 写「`layout/04` 字节登记（一·3，kb，归阶段同步）」，这一阶段同步该建；字节来源以 `crates/singlefs-core/src/records.rs`、`logged_operations.rs`、`system_configuration.rs` 与 `invariants/09-inode树结构.md` 现查。 |
| M3 | .claude/kb/decisions/26-后台整理与放置回收.md:13 | 要补 | D26 已定项 1、2、3、5、6 的进度标记仍是「未实现」（本文件第 13、14、15、17、18 行；`decisions.md:67` 进度列「已实现 0 项 / 未实现 11 项 / 无对象 2 项」），而源码已按项号点名：`grep -rhoE 'D26（后台整理与放置回收） 已定项 [0-9]+' crates --include=*.rs 加 sort 与 uniq -c` 得项 1 共 20 处、项 5 共 6 处、项 2 共 4 处、项 3 共 2 处、项 6 共 1 处，另有 `transaction.rs:9447` 的 `publish_relocation_batch` 与 `mounted_session.rs` 的 `run_compaction`。按 `records/2026-10-11-里程碑五打包与决策进度列.md:24` 的判法逐项现查后改标记，再跑 `bash .claude/gate.d/doc-decisions.sh --write` 重生成进度列；产品路径在批大小条款没定处停下，项 1 的触发与停机哪些真做了要主 agent 判，我没逐项判，不整批改。 |

## 额外发现

不在候选表里、或判定时顺带看到的，主 agent 自取：

1. `.claude/kb/checks-owed.md:101`（C116）「证据在没提交的改动里（`crash_enumeration_unlink_merging_inode_leaf_containers_stream.rs`……），这一批提交之后销账」：该流文件已在 HEAD（`git status --short` 对它无输出，`git ls-files --error-unmatch` 通过），「没提交的改动」已过时，可销账或改写。F34 第 1 行因命中的「截断」不是文件截断，判了不相干，这一句另算。
2. `.claude/kb/checks-owed.md:514`（C768）「`crates/` 里没有消费者」：`crates/singlefs-checker/src/rebuild.rs` 的模块文档写「退役记录……按退役树认领」，同一行后文又写「第一版这一格报不适用」，两说并存，要核代码后改一边。
3. D16 已定项 1「defer 窗口取多大」进度仍写未实现（`decisions/16-发布语义.md:11`）：records/2026-10-13 里程碑四预开发 156 行记的是未提交时把它改成未实现，现 `mount.rs:3509` 的 `user_visible_trees_changed` 已比 `snapshot_tree_roots`，条文「快照树根并进非空与去重」已实现，要不要改回要看该项其余分句。
4. `.claude/kb/milestone/03-third-txn.md:55`（一·1 作业点行）与第 58 行同一类过时句：「写数据的入口只收第一个文件……要动 `transaction.rs`，排在里程碑四之后」，第 55 行没进候选，改法同 F26 第 7 行。
5. 事实表写的矛盾里 F9（102 与 103 条）、F59（流的条数）、F46（E10 索引）、F73（oov-check 引用）不在我的组里，没判；我路过时看到 `invariants/README.md:29` 仍写「判 102 条」、`stage-inputs.tsv` 的 `grep -c '^crash-case:'` 现得 35，与事实表所述一致。

## 命令与输出

- 候选表校验：`sha256sum candidates.tsv` 前 16 位 a2913e391a222ec4，与派发给的 a2913e39… 对得上。
- 数行：`awk '/^## 逐行判定/{f=1;next} /^## /{f=0} f&&/^\| (F|M)[0-9]/' report.md | wc -l` 得 268（候选 265 + M1 M2 M3）；只数候选行把正则换成 `^\| F[0-9]` 得 265；各判定行数同样用这条 awk 接 `awk -F' \\| ' '{print $3}' | sort | uniq -c` 得 不相干 157、事件句不改 58、要人看 4、要改 46（M 行 3 条另判要补）。
- `python3 scripts/kb/stale-candidates.py --check-report candidates.tsv report.md --groups F25-F38` 退 2：`✗ --groups 里有候选表没有的组：F28、F29、F30`（F28-F30 是新立行，不出候选，范围写法要避开它们）。
- 改成 `--groups F25-F27,F31-F38` 退 0，末行：`  ✓ 报告判全了：265 行候选都有逐行判定`

## 没做什么

- 没改仓里任何文件，没改 `stale=`；所有改后句子都是建议，措辞和落点以主 agent / kb-scribe 为准。
- 没跑重型测试：C821 的 GPU 扫描重建对拍用例、整理崩溃流是否全域 0 红、各流的变异证红都没跑，只凭代码与登记行现查判；这类判定的推翻条件是跑出红。
- 候选行的原文在候选表里被截到 500 字，我对被截掉的后半段另用 `sed -n` 读了原文件相关行；没逐个读完所有行的全文，判定里没引到的后半段若还有过时句，我没看到。
- 没读会话记录，没碰 `~/.claude/`。「要人看」四行（F27 两行：里程碑三六·4 与「开工前要先定的」是否还留常规层、大负载层立阶段；F34 两行：D5 已定项 19、D16 已定项 14 的进度标记）需要主 agent 或用户定。
- 事实表里没写到的事实、检索词搜不到的句子，不在我的判定里（M1-M3 是反向问出来的，不保证穷尽）。
