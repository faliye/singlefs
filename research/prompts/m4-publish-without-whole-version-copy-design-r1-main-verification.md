# m4-publish-without-whole-version-copy-design-r1：判决（2026-11-18）

正文 `research/prompts/_m4-publish-without-whole-version-copy-design-r1-body.md`，开工快照 `research/prompts/m4-publish-without-whole-version-copy-design-r1-snapshot/`。设计推论轮第一轮。同一件事 2026-11-06 判过一轮定位与改法归类：`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`；正文没引它，是主 agent 漏了，这份判决把两轮对上。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md` | 推荐甲（持久共享结构），起步用二层分块持久表；随池涨、每次发布整份重建的字段 15 个；乙与「版本被留着、被读、换回靠值」不相容；丙的链长乘在发布路径对上一版的取值上；分七步换；只换表示不够，建树函数仍整棵遍历 |
| 攻方（Opus） | `research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md`，模型 `research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-model/` | 三条共有：结算与发布前整份克隆分配器、分配记录从分配器整表拷出，都不在 `TransactionOutput` 里；甲在带搬迁的发布上把全版每处指针可写地走一遍；乙最脆弱；丙沿链走、整条链被拽着；背景材料两处写错 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-publish-without-whole-version-copy-design-r1 --out ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r1-tree --repository /home/fy5090/code/singlefs`，任务 19862 退 0。
- `python3 scripts/agents/cite-check.py research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/m4-publish-without-whole-version-copy-design-r1-tree --background research/prompts/_m4-publish-without-whole-version-copy-design-r1-background.md`，末行「✓ 核了 35 处引文、197 处裸行号，对上 1 处，没判 231 处（其中裸行号没有引文可比 0 处；逐处列在上面）」。快照只记了 kb 两份，`crates/` 不在倒推出的树里，腿引 `crates/` 的行它都列成没判；判决只引主 agent 在主工作区现读过的行。
- 攻方的计数模型：主 agent 拷到 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/model` 复跑，任务 19861 退 0，「✓ 全部断言对上」「name=rerun_model_exit exit=0」，模型文件 sha256 与腿报的相同。它量的是腿自写的小模型，不是核心层。
- 主 agent 现读（主工作区此刻）：`crates/singlefs-core/src/transaction.rs:13213`「let allocator_before_this_publish = allocator.clone();」、`:14076`「let mut rehearsal = allocator.clone();」、`:14085` 调 `nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records())`、`:15822`「let mut relocated_version = previous_version.clone();」、`:16232`「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」、`:16245` 起那条恒开的断言里再调 `nodes_whose_contents_changed`；`crates/singlefs-checker/Cargo.toml` 的依赖只有 `singlefs-format`；`crates/singlefs-unit-graph/Cargo.toml` 的 `[dependencies]` 下没有本仓的包，`grep -rn '\.units\b' crates/singlefs-unit-graph --include=*.rs` 137 行；`grep -rn '\.mapping_keys\b' crates/*/src --include=*.rs` 8 行都不是读 `TransactionOutput` 的这个字段；`grep -rn 'assert_eq!(pool.output, current_before'` 8 处，其中 `crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:429`。
- 核不动：正推的代价公式（一次发布 O(d·h)）与每步改动量都是推的，没有读数；攻方「甲在带搬迁的发布上会把整棵树拷一遍」是读 `:4006`–`:4107` 推的，主 agent 没逐行读那一段。

## 二、判决

背景材料的错（两腿报的，主 agent 现查坐实）：

1. 「池级 checker 读版本」不成立：checker 只依赖 `singlefs-format`，不读 `TransactionOutput`。「压着这件事的条款」第四条写的「审计与被审计不共用一段代码」原文在 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」，不在 `.claude/rules/verification.md`。
2. 「386 处 `.units`」里 137 行是函数单元图包里的同名字段，与版本无关；`.mapping_keys` 在 `crates/*/src` 里没有读 `TransactionOutput` 这个字段的。读者改动面比正文写的小。
3. 「整理一批发两次」不对：攻方读的是一批只发一次、带搬迁的发布整版拷三次（`mounted_session.rs:5369`–`:5371`、三个分支之一、`transaction.rs:15822`）；`:15822` 主 agent 现读属实，另两处没逐行核。
4. 崩溃枚举不留 `TransactionOutput`（攻方读 `crates/singlefs-verification/src/memory_pool.rs:889`、`:1038`，主 agent 没核）。「留着多个版本时内存怎么涨」那一格的前提要重写。

与 2026-11-06 那一轮的关系（主 agent 现读它的第二节 Q2、Q3）：

- 那一轮已判：扫分配记录组与准入之前固定点每轮整份装 `mapped_units` 落在 `.claude/rules/fs-design.md` 运行时决策路径那一行（代价不许随盘容量增长）；整版深拷不落那一行。
- 那一轮 Q2.3 ②「碰过的叶作为副产品」带两个前提：全量遍历保留为参照；`assert!` 照原失败路径留着、不挪。造池提速第一批把 `:16245` 那条断言改成只重比改过的叶，全量写法只留在测试里当参照——合不合那两个前提，进第一批的代码轮判，这份判决不替它判。
- 那一轮 Q2.3 ③「分配器拷贝换回滚日志」判「按调查员的写法做不到」：落盘失败那一支要的是发布之后那一份分配器。造池提速第二批第 3 条（发布前整份克隆分配器改撤销步）撞在这一条上；派发提示写了罩不全就停，交回时主 agent 对着这一条核，没有另给冻结重发那一支出路的不收。
- 那一轮量到「缺页是 P=2048 建池挂钟的大头」（关掉 glibc 归还后建池秒数约减半）。今天几趟计时之间机器快慢差两倍多，除了本机负载，这一条也可能在里面；没重量过，记进改法。

三条候选（第一轮，打中的先挂起）：

| 候选 | 两腿 | 这一轮的判 |
|---|---|---|
| 乙：版本只带差量，全版由写者持有的可变索引给 | 两腿一致打中：一次调用交回几版、返回后逐版读全版；8 处 `assert_eq!(pool.output, current_before, …)` 会变成比差量；冻结发布要发布前后两份全态并存 | 打中，挂起。主 agent 现查坐实的只有那 8 处断言的存在；「会空转」是两腿一致推的 |
| 丙：差量加指向上一版的引用 | 两腿一致打中：冷键沿链走，链长乘在发布路径对上一版的取值上；只留最新一版时整条链被拽着 | 打中，挂起（推的，攻方模型量的是自写模型） |
| 甲：没改的与上一版共用节点 | 正推推荐；攻方打中三处做不下去的点：带搬迁的发布把全版每处指针可写地走一遍、`units` 的次序与下标被读者依赖、仓里没有持久共享容器 | 站住但带三处要解的点，挂起。正推自己也报 `units` 的次序不是 `TransactionUnit` 的派生次序，要另立键 |

三条共有、不分辨臂的一格：结算与发布前整份克隆分配器、分配记录从分配器整表拷出、建树函数整棵遍历，都不在 `TransactionOutput` 的表示里；只换版本表示买不到「一次发布只与改到的有关」。照 `.claude/rules/evidence-discipline.md`「打中不分辨臂」，这一格不拿来判臂，另立一笔：它与不动形态那两批是同一批东西，次序与归属第二轮定。

第一轮两腿结论不一致、有打中，照 `.claude/rules/three-way-inference.md`「开不开下一轮（设计推论轮）」开第二轮。

## 三、改法

- 甲加「另立单元的键、留同形的只读视图、分七步换」（正推）：零轮。
- 甲在带搬迁的发布上要另加映射 key 到指针的反查索引（攻方）：零轮。
- 那条恒开的整表断言与增量比对怎么并存（正推第 4 点：发布路径走增量、整表断言留测试构建；与 2026-11-06 那一轮 Q2.3 ② 的「照原失败路径留着」相反）：零轮，第二轮攻。
- 建池挂钟里缺页那一截（2026-11-06 那一轮量过一个点）：今天没重量；第二轮之前主 agent 用装置在同一档上各量一趟归还开、关，读数进第二轮正文。

## 交用户的

暂无。第三轮之后交，已知一条：要不要引外部的持久容器包（正推写「推不出，缺用户口径」），还是自己写。

## 这一轮被判过的文件

- .claude/kb/milestone/04-fourth-txn.md：判了「优化项：不在出口，进入条件已登记」表里「发布不再整版照抄」那一行点的三处与它的进入条件（先开设计轮三方）——这一轮是那场设计轮的第一轮，三条候选都挂起，那一行这一轮不改。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-18 不受影响：只在对上 2026-11-06 那一轮 Q2.3 ③ 时引到（落盘失败那一支要发布之后的分配器），条款本身不动 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-11-18 不受影响：三条候选都只换内存里的表示，不碰分配记录条目的盘上编码 |
