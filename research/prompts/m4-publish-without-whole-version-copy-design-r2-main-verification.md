# m4-publish-without-whole-version-copy-design-r2：判决（2026-11-18）

正文 `research/prompts/_m4-publish-without-whole-version-copy-design-r2-body.md`，开工快照 `research/prompts/m4-publish-without-whole-version-copy-design-r2-snapshot/`。设计推论轮第二轮；第一轮判决 `research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`，同一件事更早的一轮 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/m4-publish-without-whole-version-copy-design-r2-sonnet-output.md` | 第一轮判乙、丙「两腿一致打中」只对没加强的形态成立；乙加撤销日志与版本句柄、丙加定期压平之后那几格够不着；甲的三处难点里只有容器是甲独有；整表断言「照原失败路径」的意图推不出；容器自己写还是引包推不出，候选包四个 |
| 攻方（Opus） | `research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-output.md`，探针与分类表 `research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/` | 撤回第一轮自己的反查索引；另立的键打中一处；正推的二层分块第一版两版之间一块不共用、按块派生的相等会假红；分配器里另有三样随池涨；测试与装置里的读者逐行分完；探针跑前写死五条翻面线 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-publish-without-whole-version-copy-design-r2 --out ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r2-tree --repository /home/fy5090/code/singlefs`，任务 19958 退 0。
- `python3 scripts/agents/cite-check.py research/prompts/m4-publish-without-whole-version-copy-design-r2-sonnet-output.md research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r2-tree --background research/prompts/_m4-publish-without-whole-version-copy-design-r2-background.md`，末行「✓ 核了 32 处引文、130 处裸行号，对上 1 处，没判 161 处（其中裸行号没有引文可比 0 处；逐处列在上面）」。快照只记 kb，引 `crates/` 的行它都列成没判；判决只引主 agent 现读过的行与复跑出的读数。
- 攻方的探针（核心层发布路径上只数不改的计数，diff 在模型目录 `probe/publish-change-probe.diff`）：主 agent 在腿建的副本上交装置的 `construction-timing --unit-area 2288MiB --live-data-units 4096 --chunks shared`，任务 19949 退 0，产物 6685 行；汇总与判翻面线的脚本拷到 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/model-r2` 再交，任务 19957 退 0，末行「✓ 两条路的分布逐格对上、完整性闸过」，三段的 `name=integrity` 行都是 `holds=yes`（数到的发布次数与装置报的相同：2048、4224、260）。
- 主 agent 现读（主工作区）：`crates/singlefs-core/src/transaction.rs:11390`「RelocatedUnitHolder::RoleOfThisVersion(*role),」、`:3479`「pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,」；仓根 `Cargo.toml:28`「debug-assertions = true」；`crates/singlefs-verification/tests/container_tree_root_units_read_at_their_recorded_positions.rs` 在。
- 核不动：辩方的「乙强、丙强补完后与甲还差什么」全是推的，没有模型；攻方「二层分块第一版每次重建」攻的是正推第一轮报告里的写法，没有代码；测试与装置里读者的逐行分类表主 agent 没抽查。

## 二、判决

探针的读数（任务 19957 的汇总行，2288MiB 档 P=4096；d 是一次发布改到的条数，n 是全版条数）：

| 段 | 表 | d 中位（最小–最大） | n 中位 | d÷n 中位 |
|---|---|---|---|---|
| 顺序写 2048 次发布 | 单元 | 17（12–34） | 1130 | 0.014668 |
| 顺序写 | 映射 | 27（19–61） | 1120 | 0.023204 |
| 顺序写 | 分配记录 | 66（46–134） | 11446 | 0.005663 |
| 交错写 4224 次发布 | 单元 | 18（11–31） | 4500 | 0.003954 |
| 交错写 | 映射 | 27（16–53） | 4468 | 0.005939 |
| 交错写 | 分配记录 | 72（44–124） | 43582 | 0.001605 |
| 删垫片 260 次发布 | 单元 | 34（14–56） | 5639 | 0.005243 |
| 删垫片 | 映射 | 42（19–86） | 5592 | 0.006904 |
| 删垫片 | 分配记录 | 100（50–190） | 66290 | 0.001509 |

五条跑前写死的翻面线（攻方报告第二·2 节）：

- T1（分配记录表按 64 条一块的持久表，交错写段每次要复制的比例中位）：`copy64_over_n_median=0.025178`，线 0.25，`flipped=no`。
- T2（照抄换角色的项比真写的多）：`carried_under_a_new_role_median=0`、`rewritten_roles_median=18`，`flipped=no`。
- T3（同一角色多项、或没有只看角色的静态全序）：`repeated_roles_max=0`、`order_flips_among_common_roles_max=0`，`flipped=no`；`descents_within_a_role_kind_max=0`。
- T4（中央映射条目）：`copy64_over_n_median=0.074861`，线 0.25，`flipped=no`。
- T5（记录表与空闲图换持久之后，发布前那份克隆里剩下的集合与记录条数之比的最大值）：`leftover_over_records_max=0.861538`，线 0.05，`flipped=yes`；交错写段 `reclaimed_placements_max=51790`、`cluster_segments_max=450`。

逐条判：

- 「一次发布改到的条数与全版无关」在 E202 造池的三段上成立（量的）：改到的条数三样都在两位数到一百多，全版涨到几万条时不跟着涨。甲在单元、映射、分配记录三张表上值得做（T1、T2、T3、T4 都没翻）。射程：只量了造池的三种发布形状、一个档；正推第一轮列的 d 与全版成正比的形状（删或截断大文件、整文件覆盖、整理搬大批）这一趟没走到，E202 正式跑里的整理发布没量。
- 另立单元的键能复刻今天的次序（T3 没翻，量的，射程同上）：造池三段里没有同一角色多项、没有次序翻转。攻方打中的那一处（`container_tree_root_units_read_at_their_recorded_positions.rs` 靠同一角色在 `units` 里出现两项造世界）是测试造的世界，不是发布路径会产生的形状；甲下那份测试要改造法，记为改动面里的一处。
- T5 翻了（量的）：分配器里的「已回收集」随发布次数涨到与记录条数同阶（最大比 0.861538）。分配器的记录表与空闲图换成持久共享，发布前那份克隆也到不了常数，要连这几样集合一起换，或让它们不随发布涨。它是三条候选共有的一格，不分辨臂。另记一笔：已回收集为什么只涨不缩，是不是该在某一步清，这一轮没人读过，第三轮看。
- 反查索引：攻方撤回，主 agent 现读 `transaction.rs:11390` 属实（搬迁项带着它在这一版里的角色）。第一轮记在甲名下的三处难点减为两处（次序与下标、容器），次序那一处被 T3 的读数压住大半。
- 乙、丙是不是被错杀：辩方的复核站得住的一半——第一轮打中的是没加强的形态，判决那张表的「两腿一致打中」改成「对纯差量、不压平的形态打中」。乙强（撤销日志加版本句柄）与丙强（定期压平）没有模型、没被攻过，零轮。甲与它们的差别按辩方的读法是：甲要写或引一个容器，乙强把「版本是值」从编译器保证换成撤销完整性（今天五处「拷一份再改」的写法要逐处换）。照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「类型：让非法状态写不出来」，值语义由类型保证的甲更合条款；这是主 agent 的判断，推的。
- 正推的二层分块第一版（每次重建、两版之间不共用）：攻方打中，采。甲要的是增量更新时与上一版共用没改的块的结构；按块派生的相等会把内容相同的两版判成不等（只假红），相等要自己写成按内容比。
- 整表断言：辩方列了三种并存读法，「照原失败路径留着」的意图推不出。主 agent 读 2026-11-06 那一轮 Q2.3 ② 的原文：攻方当时的原话是「挪 `release` 下的 `assert!` 会改失败路径」，判决的「照原失败路径留着，不挪」说的是断言在控制流上的位置不许挪（不许从发布路径挪到别处或挪到失败之后），没有说断言比的范围不许从整表缩成改到的叶。所以「恒开、位置不动、比的范围缩成改到的叶，整表写法留作测试里的参照」不违那一条字面；它够不够格算「全量遍历保留为参照」，归造池提速那两批的代码轮判。
- 容器自己写还是引包：两腿都推不出，交用户。

三轮多数还没到；第二轮仍有打中（T5、二层分块第一版、相等假红）与零轮的形态（乙强、丙强），照设计推论轮开第三轮（最终轮）。

## 三、改法

- 甲在单元、映射、分配记录三张表上换成增量共用的持久表：量过一个档的造池三段（任务 19949、19957），T1–T4 没翻；整理发布与 d 随全版涨的形状没量。
- 角色到指针所在处的写死对照表（代替反查索引）：零轮。
- 分配器的已回收集、聚簇段集、被抛弃根的落点一并换持久或不随发布涨：零轮；T5 的读数说明不处理就到不了常数。
- 相等按内容比：零轮。
- 乙强、丙强：零轮，没有模型。

## 交用户的

暂无。第三轮之后交，已知两条：持久容器自己写还是引外部包（候选 imbl、im、rpds、immutable-chunkmap，许可与缺口在辩方报告）；甲的分步里先换哪一张表。

## 这一轮被判过的文件

- .claude/kb/milestone/04-fourth-txn.md：判了「发布不再整版照抄」那一行的做法在核心层量到的依据——一次发布改到的单元、映射、分配记录条数不随全版涨，持久共享的表示在这三张表上值得做；那一行等第三轮之后改。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-18 不受影响：冻结发布之后那一份分配器在持久共享下照样给得出，条款不动 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-11-18 不受影响：只换内存里的表示，不碰分配记录条目的盘上编码 |
