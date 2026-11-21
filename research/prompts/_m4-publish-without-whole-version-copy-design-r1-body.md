# 发布不再整版照抄：版本里随池大小涨的那几样怎么表示（m4-publish-without-whole-version-copy-design-r1）

日期：2026-11-18。

## 问题

一次用户写加发布的 CPU 用时随池里已有的东西线性涨，整池造下来是平方。E202（GPU与CPU两种模式GC对照组）的起步池在登记的最小档（36608 个活数据单元、18304 步交错写）造不出来：上一次量到 300 秒只做 95 步（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:1106`，任务 16507）。用户 2026-11-18 定「现在做造池提速」，随后定「同时开设计轮改形态」（`records/2026-11-17-里程碑四收口续做.md` 第 112、134 行）。

不动发布结果形态的那一类改法另走实现（两批，`~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/`）。这一题只判剩下那一类：发布的产出 `TransactionOutput` 里装着整版的东西，每次发布把它们整份重拼。要给出：

1. `TransactionOutput` 里哪几样随池大小涨、每次发布整份重建；它们各自被谁读、读的是全版还是只读这次改的。
2. 换成什么表示，一次发布的代价只与这次改到的东西有关：候选至少比三条——甲「持久共享结构」（没改的与上一版共用节点，版本仍然读得到全版）、乙「版本只带这次的差量，全版由写者持有的一份可变索引给」、丙「版本带差量加指向上一版的引用，读全版时沿链合出」。别的候选腿可以加。
3. 每条候选对读者的代价：崩溃点重放、池级 checker、验证包的内存池、GPU 判器的输入表、整理与区域扫描各读哪几样，换表示之后哪些读者要改、改多少处。
4. 版本的身份与相等：今天版本之间能逐字段比（测试、对拍、发布失败换回都靠它），换表示之后「两版相等」「回到发布前」怎么判、代价多少。
5. 发布失败换回、取号全或无失败的回卷、整理一批发两次这几条路，在各候选下还成不成立。
6. 分步：能不能一样一样换（先单元列表、再映射、再分配记录），每一步都能单独验、单独合。

盘上字节、发布语义、分配次序与落点一个都不变；这一题只动内存里的表示。

## 实现今天的样子

主 agent 的观测（现读 `crates/` 与任务日志，行号是主工作区此刻的）：

- `crates/singlefs-core/src/transaction.rs:3375` 起的 `pub struct TransactionOutput`：`units: Vec<PublishedUnit>`、`mapping_keys: Vec<Vec<u8>>`、`allocation_records: Vec<AllocationRecord>`、`accounting_entries: Vec<AccountingEntry>`、`deadlist_entries: Vec<DeadlistEntry>`、`mapping_entries_kept_for_the_snapshots`、`tree_table_entries`、`data_pointers` 等都是整份的 `Vec`；另有各棵树的版本（`allocation_record_tree`、`extent_tree`、`accounting_tree`、`central_mapping_tree`、`deadlist_tree`）。`PublishedUnit`（`:3358`）是 `slot`、`identity` 与 `bytes: Arc<[u8]>`：单元字节已经共享，列表本身每次重拼。
- 读这几样的地方（`grep -rn` 数的，`crates/*/src` 与 `crates/*/tests`）：`.units` 386 处、89 份文件；`.allocation_records` 206 处、64 份文件；`.mapping_keys` 27 处、12 份文件。读 `.units` 的 `src` 文件里有 `crates/singlefs-core/src/region_scan.rs`、`mount.rs`、`compaction_plan.rs`、`mounted_session.rs`，`crates/singlefs-checker/src/walk.rs`，`crates/singlefs-verification/src/memory_pool.rs`、`crash_judge_tables.rs`、`crash_verify_gpu.rs`。
- 发布的路：`crates/singlefs-core/src/mounted_session.rs:2951` 的 `publish_user_change` → `transaction.rs:13994` 的 `settle_the_allocation_record_tree`（结算，固定点各轮）→ `transaction.rs:15681` 的 `publish_admitted`（真发，重拼全版映射条目、中央映射树与 `units`）。
- 量到的（任务 19831，第一批不动形态的改法之后，P=4096 交错写整段 4224 次发布，本机负载高、只看占比；分段行 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/section-timing-after-the-first-speedup-task-19831.txt`）：`user_change.total` 180.205 秒；其中重拼形态里那几样的三段 `admitted.central_mapping` 12.134、`admitted.allocation_records_sorted_copy` 11.418、`admitted.units_assembly` 6.938，合 30.490 秒；`publish.allocator_clone_before` 14.195（发布前整份克隆分配器，分配器里有 `records` 整表）。不动形态的第二批在改 `settle.resolve`（45.087）、上一版按叶分组（28.097）与发布前克隆，做完之后上面三段占的比例会升（推的，没量过）。
- 整理一批发两次、每次整版深拷贝：调查员读到 `crates/singlefs-core/src/mounted_session.rs:5368`–`:5371`、`:5727`（`~/.local/share/singlefs/work/e202-pool-construction-speedup-investigation/result.md`，主 agent 没逐行复核）。
- `crates/` 里没有持久共享的容器：`grep -rn 'im::\|rpds\|persistent' crates/*/src` 主 agent 没跑过，腿自己核；工作区依赖只有 `std` 与少数几个包的，以各包 `Cargo.toml` 为准。

方案按 `crates/` 今天的实现来谈；改法写成对实现的改动（改哪个文件的哪个结构、读者哪几处跟着改）。

## 压着这件事的条款

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：运行时决策路径的代价不许随盘容量增长；运行时与 checker 必须用不同的算法。
- `.claude/kb/milestone/04-fourth-txn.md`「优化项：不在出口，进入条件已登记」表里「发布不再整版照抄」那一行（进入条件写「先开设计轮三方再实现」）。
- `.claude/rules/fs-design.md`「一个事务层，所有结构共用」：不为某个路径开第二套事务机制。
- `.claude/rules/verification.md`「checker 即规范」与「功能正确性靠模型对拍」：池级 checker 与模型对拍读版本的方式不能因为换表示而变成读写者自己的索引（审计与被审计不共用一段代码）。
- `.claude/singlefs-ai-sop/rules/code-discipline.md`「类型：让非法状态写不出来」。

## 判据格

每条候选各答一格，答不出的写「推不出」：

| 格 | 问什么 |
|---|---|
| 一次发布的代价 | 换表示之后，一次只改一个单元的发布在这几样上做的事是多少（与改到的条数、与全版条数各是什么关系）；写出最坏的那一种发布 |
| 读全版的代价 | 读者要全版（按槽找单元、按 key 找映射、遍历全部分配记录）时一次读的代价；今天是数组下标或二分的地方变成什么 |
| 读者改动面 | 386 处 `.units`、206 处 `.allocation_records`、27 处 `.mapping_keys` 里要改的有多少、能不能留一个与今天同形的只读视图让大多数读者不改 |
| 版本相等与换回 | 两版逐字段相等怎么判、代价多少；发布失败换回、回卷之后「与发布前相同」怎么钉住 |
| 审计独立 | 池级 checker、模型对拍、崩溃点重放读版本时读不读写者持有的可变状态；读的话算不算审计与被审计共用一段代码 |
| 内存 | 留着多个版本（崩溃枚举留每一版、整理留上一版）时内存怎么涨；与今天整份 `Vec` 比 |
| 依赖 | 要不要引新的外部包；不引的话自己写的结构有多少行、怎么验（对拍、变异） |
| 分步 | 能不能一样一样换，每一步之后全部测试不改期望 |
| 发布语义不变的证据 | 拿什么证明换表示前后每次发布写出的字节、落点、版本内容逐项相同 |

## 分工

- 正推腿：照「问题」六条逐条给出一条推荐的表示与分步，判据格逐格答；每一格的依据现读 `crates/`，引文件与行；`TransactionOutput` 里随池涨的字段列全（上面「实现今天的样子」只列了主 agent 读到的）。
- 攻方腿：攻三条候选各自最弱的一格，尤其是「读者改动面」「审计独立」「版本相等与换回」；找一种发布（或一条读路径）让某条候选的代价退回到与全版成正比，或让换表示之后的某个读者读到与今天不同的内容；能建小模型量的建模型，命令写进报告交主 agent 跑。
- 两条腿都不改 `crates/`。

## 交用户的

暂无；判决时列。
