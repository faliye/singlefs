# 发布不再整版照抄：第二轮（m4-publish-without-whole-version-copy-design-r2）

日期：2026-11-18。

## 问题

第一轮判决 `research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`：乙（版本只带差量、全版由写者持有的可变索引给）与丙（差量加指向上一版的引用）两腿一致打中，甲（没改的与上一版共用节点）站住但带三处要解的点，三条都挂起。同一件事 2026-11-06 判过一轮定位与改法归类：`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`。

这一轮攻三样（照设计推论轮第二轮的攻击面：第一轮的修补本身、产生结论的方法、从未被看过的地方），不重复第一轮已攻过的角度（乙的「版本是值」、丙的链长、甲的三处难点本身的存在）：

1. 第一轮的修补本身。甲的三处难点各自的解法站不站得住：
   - 带搬迁的发布把全版每处指针可写地走一遍（第一轮攻方读 `crates/singlefs-core/src/transaction.rs:4006`–`:4107` 的 `take_the_mapping_positions_over_the_location_hints`）：攻方给的解法是另加一份映射 key 到指针的反查索引。这份索引谁维护、放哪、它自己随不随全版涨、与「审计与被审计不共用一段代码」冲不冲突。
   - `units` 的次序与下标被读者依赖（正推报次序不是 `TransactionUnit` 的派生次序，要另立键；攻方报 `ContainerTreeRootUnitPositions` 存的是 `units` 下标）：另立的键能不能复刻今天的次序、同一角色多项时怎么排、存下标的那几处换成什么。
   - 仓里没有持久共享容器：自己写（正推的二层分块持久表）还是引外部包；自己写的话多少行、怎么验。
2. 产生结论的方法。第一轮说「甲最站得住」靠的是读代码与攻方自写的小模型，没有一个数出自核心层。给出一个能在核心层量的探针（不改发布语义，只数）：一次发布在各样上改到的条数 d 的分布（造池的顺序写、交错写、删垫片三段各是多少），拿它判「一次发布的代价只与 d 有关」在甲下值多少；d 本身与全版成正比的发布形状（正推列了删或截断大文件、整文件覆盖、整理搬大批）在 E202 的造池与正式跑里占不占。
3. 从未被看过的地方：
   - 三条共有的那一格的归属与次序。不动形态的两批做完之后量到的剩余（任务 19899，P=4096 交错写整段 48.083 秒，带计时、本机负载高，只看占比）：`admitted.allocation_record_tree` 6.915、`admitted.allocation_records_sorted_copy` 5.373、`publish.allocator_clone_before` 5.379、`settle.previous_node_set` 7.044、`settle.resolve_apart_from_the_allocation_record_tree` 6.845、`publish.persist` 6.072、`admitted.central_mapping` 4.701、`admitted.units_assembly` 2.801。发布前整份克隆分配器两次都没改成（2026-11-06 那一轮 Q2.3 ③；这一次实现员报空闲图六种改动没有撤销步、落盘失败要冻结发布之后那一份）。问：分配器的记录表与空闲图换成持久共享之后，这份克隆是不是自然变成常数代价；这算不算甲的一部分、放在分步的哪一步。
   - 那条恒开的整表断言（`transaction.rs:16245` 一带）与增量比对怎么并存：2026-11-06 那一轮 Q2.3 ② 要「全量遍历保留为参照、断言照原失败路径留着」，第一轮正推提「发布路径走增量、整表断言留测试构建」。两种读法各自丢什么。
   - 读者里测试与装置那一部分第一轮没逐行分类（攻方报告「没做什么」）：`crates/*/tests` 里读 `.units`、`.allocation_records` 的按用法分类（按下标、按次序、整体相等、遍历找），各多少处。
4. 乙与丙有没有被错杀（辩方腿的格）：第一轮判它们打中的依据够不够得着；给乙、丙各一个最强的形态（例如乙加「调用返回的每一版都物化成全版」、丙加定期压平），判它在第一轮打中的那几格上还中不中，以及这样补完之后与甲还差什么。

盘上字节、发布语义、分配次序与落点一个都不变；这一题只动内存里的表示。

## 实现今天的样子

主 agent 的观测（现读主工作区的 `crates/` 与任务日志）：

- `crates/singlefs-core/src/transaction.rs:3375` 起的 `pub struct TransactionOutput`：`units: Vec<PublishedUnit>`、`mapping_keys: Vec<Vec<u8>>`、`allocation_records: Vec<AllocationRecord>` 等整份的 `Vec`，另有各棵树的版本；`PublishedUnit`（`:3358`）的 `bytes` 是 `Arc<[u8]>`。
- `transaction.rs:13213`「let allocator_before_this_publish = allocator.clone();」、`:14076`「let mut rehearsal = allocator.clone();」、`:15822`「let mut relocated_version = previous_version.clone();」、`:16232`「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」、`:16245` 起那条恒开的断言里调 `nodes_whose_contents_changed`。
- 池级 checker 不读版本：`crates/singlefs-checker/Cargo.toml` 的依赖只有 `singlefs-format`。函数单元图包不依赖核心层，`grep -rn '\.units\b' crates/singlefs-unit-graph --include=*.rs` 的 137 行是它自己的同名字段。`grep -rn '\.mapping_keys\b' crates/*/src --include=*.rs` 的 8 行都不是读 `TransactionOutput` 的这个字段。
- `grep -rn 'assert_eq!(pool.output, current_before'` 8 处，其中 `crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:429`。
- 不动形态的两批（结算按叶分组只重比改过的叶、演练做完撤回、上一版分组跨发布带着走、中央映射树只按改到的 key 规划、分配器按盘与槽的索引）在副本里，没进主工作区；补丁 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch`。主工作区里上面那几行是改前的样子。两批之后量到的分段行：`~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/section-timing-after-the-second-speedup-task-19899.txt`。
- 关掉 glibc 归还的对照今天重量过一个点（任务 19871，P=4096 整池 170.991 秒，不关的任务 19694 是 159.792 秒，各一次、本机负载高）：没看到 2026-11-06 那一轮在 P=2048 上量到的减半。

方案按 `crates/` 今天的实现来谈；改法写成对实现的改动。

## 压着这件事的条款

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：运行时决策路径的代价不许随盘容量增长；审计与被审计不用同一段代码，运行时与 checker 必须用不同的算法。
- `.claude/kb/milestone/04-fourth-txn.md`「优化项：不在出口，进入条件已登记」表里「发布不再整版照抄」那一行。
- `.claude/rules/fs-design.md`「一个事务层，所有结构共用」。
- `.claude/singlefs-ai-sop/rules/code-discipline.md`「类型：让非法状态写不出来」。

## 判据格

| 格 | 问什么 |
|---|---|
| 反查索引 | 甲在带搬迁的发布上的解法：索引的代价、归属、与审计独立冲不冲突 |
| 次序与下标 | 另立的键复刻不复刻今天 `units` 的次序；存下标的读者换成什么；换完哪些测试的期望会动 |
| 容器 | 自己写还是引包；自己写的行数与验法；引包的话引哪一个、它的依赖 |
| d 的探针 | 探针怎么数、数出来什么读数让「甲值得做」翻面 |
| 共有那一格 | 分配器的记录表与空闲图进不进甲；发布前那份克隆在各种做法下的代价 |
| 整表断言 | 两种并存读法各丢什么；哪一种合 2026-11-06 那一轮 Q2.3 ② 的两个前提 |
| 测试里的读者 | 按用法分类的处数 |
| 乙、丙的最强形态 | 补完之后在第一轮打中的几格上还中不中、与甲差什么 |

## 分工

- 辩方腿：第 4 条（替乙、丙辩护，复核第一轮判决够不够得着），并答「整表断言」「容器」两格。
- 攻方腿：第 1、2、3 条，攻甲的三处解法与共有那一格；给出 d 的探针（在副本里写、命令交主 agent 跑）；「测试里的读者」那一格逐行分类。撤回自己第一轮给的方案比打中新东西更有价值。
- 两条腿都不改主工作区的 `crates/`。

## 交用户的

暂无；第三轮之后列。已知一条：要不要引外部的持久容器包。
