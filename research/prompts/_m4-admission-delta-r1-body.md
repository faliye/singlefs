# m4-admission-delta-r1：准入之前的步骤怎么做到不随总数涨（设计轮正文，2026-11-04）

<!-- doc-lint:not-numbers A1 A2 A3 A4 A5 -->

## 一、题面

代码轮 m4-close-code-r2 判决（`research/prompts/m4-close-code-r2-main-verification.md` 第四节第一行与「用户定案」）交用户的一件：准入之前还有几步随盘上容器总数、记录总数涨。用户先定「现在就改」，实现员（报告 `~/.local/share/singlefs/work/m4-close-code-r2-fix-reader-writer/result.md` 第 3 条）改到一半停下：这几步的产物都是整版的全量序列，下游也是全量的，只改这几步准入之前照旧随总数涨；用户随后定「现在就重新设计」。这一轮推的是：准入之前算什么、用什么表示，才能让准入之前每一步的代价只随这一次碰到的容器、节点、记录数（加树高）涨，同时准入判出的结论（放不放行、拒成哪一种）与今天相同或可证更严。

被判条款（整条抄在附录）：D8（核心索引结构） 已定项 16 射程里那一句「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」；`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表的运行时决策路径一格。准入本身的语义（准入判什么、不够时先推空发布抬 F 再判）归 D16（发布语义） 已定项 1。

与 E199（发布照抄上一版的代价）的分工：E199 量的是四条臂（岔路单 `research/prompts/m4-transaction-output-copy-forks.md`：甲 今天每版完整一份、乙 版本间共享、丙 上一版按值交进来、丁 只记差量）在照抄字节与现算重建字节上的代价，它第三份登记的全扫（`research/prompts/e199-preregistration-r3.md` 第 262 行起）已把这一轮点名的几处归成甲臂的「照抄·进模型」与「现算重建·进模型」。这一轮不重量 E199 的数，要回答 E199 四条臂之外的问题：

- 准入判需要的是「这一次要分配多少、落在哪类空间」，能不能由差量（这一次进出的 key、碰到的路径）算出一个上界，把全量的规划挪到准入之后；
- 每条候选在今天的代码上要改哪几步，哪几步改完还随总数涨；
- 每条候选碰到的形式化规格与不变量。

## 二、实现今天的样子（主 agent 的观测，2026-11-04，主工作区）

方案按 `crates/` 今天的实现来谈。读过的路径与看到的事实：

- 发布的入口 `crates/singlefs-core/src/transaction.rs:12409` `prepare_the_version_publish`；准入判在 `:12489` `judge_the_publish_before_any_read_or_write` 里，`:12523` 调 `admission::admit_the_units_landing_on_every_device`（`crates/singlefs-core/src/admission.rs:1064`）。
- 多层码 2 树（记账树、deadlist 树、中央映射树）在 `transaction.rs:8976` `plan_the_multi_level_trees` 里规划：记账树 `:9015` 调 `code_two_tree::plan_the_tree_after_this_publish`；中央映射树先由三棵容器树各调一次 `mapped_units_of_a_container_tree`（`:9368`、`:9374`、`:9392`，函数在 `:6912`，`:6937` 对 `tree.node_origins()` 逐个走），再在 `:9399` 起把这一版全部映射 key 与留存的映射条目收进一个 `BTreeSet`，交给 `:9417` 的 `plan_the_tree_after_this_publish`。
- `crates/singlefs-core/src/code_two_tree.rs:637` `plan_the_tree_after_this_publish(previous, keys_after_this_publish, capacity)` 拿的是这一版全部 key；`:651` 用 `previous.keys_in_order()` 把上一版全部 key 收成一个 `BTreeSet`。这个函数不在 `verus!` 里。
- `crates/singlefs-core/src/container_tree.rs:2874` `node_origins` 在 `verus!` 里，规格（`:2878` 起，`every_node_absent_from_the_previous_version_is_rewritten`）交回全部节点。
- `transaction.rs:4205` `inode_leaf_container_contents` 拷一遍全部 inode 叶；写者 `crates/singlefs-core/src/inode_tree.rs:1093` 又调 `copy_of_the_leaf_containers`（`:1287`）整份拷一遍。
- 分配记录树另走一条：`transaction.rs:1910` 调 `crates/singlefs-core/src/allocation_record_tree.rs:429` 的规划；`:15201` 起的注释写「重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）」。
- 准入之前的 inode 记录计数已改成随一版带着、按差量维护（`crates/singlefs-core/src/admission.rs:288` `InodeRecordCounts`，r2 判决 T4 一格）；这一份是「按差量维护」在本仓已有的样子。
- 中央映射树的计划之后另有 `plan_rewriting_the_paths_to_the_moved_entries`（`transaction.rs:9416` 起），把整理搬迁的 key 所在路径标成重写。

主 agent 没量过：准入之前每一步在大盘上的实际用时与内存；准入判的输入里到底哪几样只能从全量规划里得到。

## 三、候选（各自的定义）

| 候选 | 定义 |
|---|---|
| A1 今天 | 准入之前做全量规划（全部 key、全部节点），准入判读规划的结果 |
| A2 规划改成按差量 | 准入之前的规划只读这一次进出的 key 与它们碰到的路径（节点身份、分裂与合并），上一版的形状按引用读、不收全量 key；准入判照旧读规划的结果 |
| A3 准入判挪到差量上界 | 准入之前只由差量算出这一次分配量的上界（每棵树按「这一次进出的 key 数 × 树高」之类的式子），准入判这个上界；全量规划照今天的写法挪到准入之后，规划出的实际量不超过上界由断言或证明守住 |
| A4 E199 的乙丙丁 | 不改规划算法，改上一版的表示（共享、按值交进来、只记差量），由 E199 量代价；这一轮只判它们能不能单独让准入之前不随总数涨 |

各条腿可以提新候选，提的写进报告并写明它与表里哪一条不同。

## 四、格

### A1　准入之前到底有哪几步随总数涨
从 `prepare_the_version_publish` 到 `admit_the_units_landing_on_every_device` 之间走一遍，列出每一步随哪个总量涨（容器总数、记录总数、映射条目总数、节点总数），写文件:行。第一节点名的几处之外还有没有（分配记录树的固定点、deadlist、记账树的行表 `transaction.rs:7978`）。

### A2　按差量规划
多层码 2 树与三棵容器树的规划改成只读差量，要上一版的什么（形状的哪几层、哪几条路径）、怎么拿到而不收全量；分裂、合并、长高、降低、整理搬迁（`plan_rewriting_the_paths_to_the_moved_entries`）几种情形里，差量规划与全量规划重写的节点集合是不是逐个相同；`node_origins` 的 Verus 规格要怎么改。

### A3　差量上界准入
准入判今天读的是精确量还是上界；挪到上界之后，有没有一条历史让上界放行而真实分配不够（那是正确性问题），或上界拒了而真实够（那是提前 ENOSPC，问它与 D16（发布语义） 已定项 1 的「准入」那一行、C283（准入失败时不先推发布就报 ENOSPC） 合不合）；上界式子在每棵树上各是什么、紧不紧（在 E199 的几何上大约多估多少，推的就写推的）；分配记录树的固定点（自己的分配会改自己的形状）在上界下怎么办。

### A4　只换表示够不够
乙、丙、丁三种表示下，准入之前那几步的全量 `BTreeSet` 与 `keys_in_order()` 还在不在；不改规划算法、只换表示，准入之前能不能不随总数涨。

### A5　与已定条款的冲突
每条候选碰到哪几条已定条款与不变量（D16（发布语义） 已定项 1、D8（核心索引结构） 已定项 11 ①「路径上的节点整条 COW」、码 2 树形状的不变量），写分项号与 `crates/singlefs-checker/invariants/` 的条目号；哪一条要改条款、哪一条只改实现。

## 五、第一轮分工（照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）

正推腿逐格推每条候选在今天的代码上怎么落、改完哪几步还随总数涨。攻方腿第一轮攻过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱：找一条历史或一个形状，让某条候选准入判出的结论与今天不同，或让它在某一步上仍随总数涨。各条腿报告里给每条候选一行代价的量级（推的写推的），不下「选哪条」的结论，选择交用户。
