# m4-close-code-r2 正推腿报告（Sonnet，2026-11-03）

立场：正推。逐格核「代码做的是不是条款与判决说的」，从 `crates/` 的实现推出「应该是什么」，再与被判对象逐格比。
读码一律读冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r2/repo/`（下文路径省去这一前缀，行号以它为准）；kb 与规则的行号是在仓里 `grep -nF` 现查的。
没起任何脚本、没跑 Verus、没跑测试；所有「量的」都是读码与 grep 的原样结果，凡要起命令才核得了的，写在第七节「要主 agent 跑的命令」。

## 零、先说三个背景材料里要更正的地方

1. 正文第一节说「D16（发布语义） 已定项 10 的统计量那一句」。在 D16 里 `grep -n '统计量' .claude/kb/decisions/16-发布语义.md` 零命中（上面那份文件里没有这句）。
   管「准入之前」这条读法的条款在 D8（核心索引结构） 已定项 16 的射程，下面各格按它判：
   `.claude/kb/decisions/08-核心索引结构.md:516`：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」。
   用户定案的原话在 `records/2026-10-13-里程碑四预开发.md:1341`：「五处加准入现在改，根源先量」。
2. 正文第二节说 `crates/singlefs-core/src/transaction.rs:4018` 是 `pointer_below_the_root`、`:3309` 是 `ContainerTreeRootUnitPositions`、`:4188` 是 `inode_record_found_by_its_leaf`：三处我都在冻结副本里现查，行号对。
   `container_tree.rs` 的七处（1941、2078、2256、2369、2612、2729、2874）也对。
3. 正文说 `mounted_session.rs` 的 `run_compaction_in_the_background` 是这一轮不改的那一个：冻结副本里同名函数有两个（`crates/singlefs-core/src/mounted_session.rs:2401` 是裸会话的，`:5565` 是攒批会话的）。
   T5 说的「排空待发之后才起整理」是 `:5565` 那一个。

## 一、T1　开片原地改与上一版只记每层节点数

结论：成立。

### 1.1 新一版形状与「逐层照抄再改」逐个相同

- 改法：`crates/singlefs-core/src/container_tree.rs:2369`：「pub fn after_a_container_is_opened_at_the_right_end(self, entries_per_node: usize) -> (opened: Self)」按值拿形状。
- 它的 `ensures` 还是旧的那一句，对着同一个规格函数：`crates/singlefs-core/src/container_tree.rs:2374`：「opened.levels() == levels_after_opening_at_the_right_end(self.levels(), entries_per_node as int),」。
  规格函数本身这一轮没改（补丁里它所在的 hunk 只在它的右花括号之后加了新规格）：`crates/singlefs-core/src/container_tree.rs:431`：「pub open spec fn levels_after_opening_at_the_right_end(levels: Seq<Seq<usize>>, entries_per_node: int) -> Seq<Seq<usize>> {」。
  所以「长高、正好填满一层、连开几片跨层」这三种形状，正确性由 Verus 的 `ensures` 罩着（规格是「逐层照抄再改」那一套的数学写法，旧代码证的是同一句），不靠取样。
- 我另外手推了两个边界（扇出 2），代码读法与规格一致：
  - 已有 2 片：`levels = [[2]]`，开第 3 片时 `stop = 0`、`grows = (2 >= 2)`，第 0 层末尾 `push(1)`，再 `push` 新根 `[2]`，得 `[[2,1],[2]]`。
  - 已有 4 片：`[[2,2],[2]]`，开第 5 片时 `stop` 爬到根层，`grows = true`，第 0 层 `push(1)`、根层 `push(1)`、再 `push` 新根 `[2]`，得 `[[2,2,1],[2,1],[2]]`。
  两个结果和测试 `a_full_root_grows_a_level_with_the_old_root_kept_in_place_and_a_new_right_half` 里的 `(3, 0)` 两个孩子一致。
- 循环里的调用：`crates/singlefs-core/src/container_tree.rs:3658`：「shape = shape.after_a_container_is_opened_at_the_right_end(entries_per_node);」。
  进循环前的那一份仍是照抄：`crates/singlefs-core/src/container_tree.rs:3576`：「let mut shape = shape_before_this_publish.clone();」。这一处就是正文点名、归 E199 的三处之一，不在这一格里。

### 1.2 只记节点数之后，哪个判断读不到上一版的孩子数了

答：没有。上一版现在是：`crates/singlefs-core/src/container_tree.rs:2612`：「pub node_counts_before_this_publish: ContainerTreeNodeCounts,」。
读它的地方我在冻结副本里全找了一遍（`grep -rn 'node_counts_before_this_publish' crates`），一共四类，每类只问层数、某个位置有没有节点、哪个是根：

| 读它的判断 | 位置 | 要的是什么 |
|---|---|---|
| 长没长高 | `crates/singlefs-core/src/container_tree.rs:2804`：「self.node_counts_before_this_publish.root_level() > 0」 | 层数 |
| 照抄的节点上一版有没有 | `node_origins` 里 `has_a_node_at`，规格 `crates/singlefs-core/src/container_tree.rs:463`：「pub open spec fn every_node_absent_from_the_previous_version_is_rewritten(」 | 每层节点数 |
| 上一版的根在哪 | `crates/singlefs-core/src/transaction.rs:6936`：「let root_before_this_publish = tree.node_counts_before_this_publish.root_position();」与 `crates/singlefs-core/src/transaction.rs:14154`：「let root_before_this_publish = after.node_counts_before_this_publish.root_position();」 | 层数 |
| 长高那次旧根留在原位置不被换下 | `previous_root_is_carried_down_unchanged`（同一文件） | 层数 |

`ContainerTreeNodeCounts` 自己的文档也把这一点写死了：`crates/singlefs-core/src/container_tree.rs:2496`：「四样都只看层数与」。
旧代码对上一版形状的读，看改动全文里被删掉的 `-` 行（`grep -n '^-.*shape_before_this_publish' research/prompts/_m4-close-code-r2-diff.md` 一共 30 行左右）：`.levels().len()`、`.root_level()`、`.root_position()`、`.has_a_node_at(position)`、`.is_the_root(position)`，规格里是 `node_is_in_the_levels` 的存在判断，没有一行读某个节点的孩子数。
`.containers()` 那一问在新代码里换成 `assert!(shape_before_this_publish.containers() == containers_before_this_publish.len(), …)`，读的是 `ContainerTreeShape` 自己记的 `container_total`，不是上一版的孩子数。

### 1.3 要说清的两件小事（不改结论）

- 对拍取样点（`shapes_for_the_ancestor_differential`）的扇出是 2、3、135 各开 1 到 40 片：扇出 135 的 40 片只有一层，不碰长高。长高与跨层由扇出 2、3 的 80 份和「先扇出 2 开 9 片再换产品扇出」的 31 份罩。这是方法一侧，攻方腿的格，我只记事实。
- 补丁里 `a_full_root_grows_...` 测试的注释 `crates/singlefs-core/src/container_tree.rs:3770`：「(1, 2) 满了开 (1, 3)」，和我手推的 `[[2,2,1],[2,1],[2]]`（第 5 片只开 (1, 2)、(2, 1)）对不上；它自己的断言（列了六个位置）是对的，只是注释字面有误。不影响代码。

什么会推翻 T1：`after_a_container_is_opened_at_the_right_end` 在某个 `(levels, entries_per_node)` 上交回的 `levels` 与规格函数不等（Verus 的 `ensures` 不过，或补丁外有人绕过 `requires` 调它）；或者读到某处还在用上一版某节点的孩子数，而 `ContainerTreeNodeCounts` 里没有这个数。我没找到。

## 二、T2　求祖先从最右端往左

结论：成立（格里问的那个前提在每一条会重写容器的发布路径上都成立）。另有一处条款与代码之间还差的，不在这一格的问法里，单列在 2.4，已被 E199 点名。

### 2.1 前提「重写的容器是最右端连着的那几片」为什么在每条路径上成立

容器小树只有三棵（墓碑树、谱系树、退役树），`ContainerTreeAfterThisPublish` 只有两个造法：

- 追加：`crates/singlefs-core/src/container_tree.rs:3661`：「if rewritten_containers.is_empty() || rewritten_containers[rewritten_containers.len() - 1] != rewritten_position {」。
  每装一条记录，要么装进最右那片（`open_container_has_room`），要么在最右端开新一片；`rewritten_position` 恒是 `containers.len() - 1`，清单只在末项不等于它时才推。所以清单是「最右那片（若上一版的开放容器还有位）+ 之后新开的几片」，一片挨一片到最右端。
- 不追加：`crates/singlefs-core/src/container_tree.rs:2683`：「pub fn carried(containers: Vec<RecordContainer<Record>>, shape: ContainerTreeShape) -> (carried: Self)」，清单恒空。

「这三棵树只增不减」这一点，代码里有明文：`crates/singlefs-core/src/container_tree.rs:2702`：「容器只增不减，没有「删空一片」那一格」。我另用 `grep` 看了会不会有别的地方造这个结构或改它的容器清单：

- `grep -rn 'ContainerTreeAfterThisPublish {' crates`：只有 `crates/singlefs-core/src/container_tree.rs:3714`（追加）和 `crates/singlefs-core/src/container_tree.rs:4079`（测试里拼的）。
- 追加只有三个调用点：`crates/singlefs-core/src/transaction.rs:8618`：「append_tombstone_to_the_open_container(」、`crates/singlefs-core/src/transaction.rs:8659`：「let retirement_tree = append_retirement_records_to_the_open_container(」、`crates/singlefs-core/src/transaction.rs:8950`：「append_lineage_record_to_the_open_container(」。每个发布每棵树至多调一次。

格里列的六条路径逐条：

| 路径 | 碰不碰这三棵树的容器 | 依据 |
|---|---|---|
| 追加 | 是，只往最右端装 | 上面 `crates/singlefs-core/src/container_tree.rs:3661` |
| 开片 | 是，只在最右端开 | 同上，且 `:3658` 调的就是最右端开片 |
| unlink 合并叶 | 合并的是 inode 树的叶；它退出的身份写一条退役记录，进退役树的**追加**（`crates/singlefs-core/src/transaction.rs:8659`，记录由 `retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg)` 来） | 退役树也只增不减 |
| 截断 | 截断改 inode 记录的 size、改 extent 树与数据单元，不碰这三棵树 | `DeletionStepOfThePublish::TruncationIntent` 等一族只在 `ordinary_allocation_part` 里改 `record_writes`（`crates/singlefs-core/src/transaction.rs:8334`），墓碑树只有 `Intent` 一支追加（`:8618` 所在的 `match`） |
| 整理搬迁 | 不碰：搬迁换的是中央映射里单元的位置。墓碑树、谱系树整个照抄（走 `carried`）；退役树照样走追加那一支，但整理的发布不改 inode 树、没有退出的身份，追加零条，清单为空 | 整理那一趟的发布 `file: None`、`new_inode_records: &[]`、步骤是 `NotADeletion`（`crates/singlefs-core/src/transaction.rs:10844`：「fn publish_compaction_step_after_its_check<Device: BlockDevice>(」起那一段） |
| 回退之后第一次发布 | 三棵树取现行那一版的，不取目标那一版的 | `crates/singlefs-core/src/mount.rs:10214`：「tombstone_containers: current.tombstone_containers.clone(),」、`:10217`：「lineage_containers: current.lineage_containers.clone(),」、`:10221`：「retirement_containers: current.retirement_containers.clone(),」；回退这次往谱系树**追加**一条（`:8950`） |

### 2.2 前提万一不成立，是漏算还是多算——都不是静默的

清单不是最右端连着的那几片时，下面几处会当场 panic，不会悄悄漏掉或多算：

- `crates/singlefs-core/src/container_tree.rs:2786`：「self.rewritten_containers[listed] == total - rightmost_count + listed,」，在 `rightmost_count_of_the_rewritten_containers` 里逐片核，代价是重写的片数。
- `first_rewritten_container`（`crates/singlefs-core/src/container_tree.rs:2729`）、`rewritten_nodes`、`node_origins`（`:2874`）三个入口都先过这一核。
- 两处 `position >= first` 成员判断：`crates/singlefs-core/src/transaction.rs:6925`：「if first_rewritten_container.is_some_and(|first| position >= first) {」与 `crates/singlefs-core/src/transaction.rs:14114`：「let node = if first_rewritten_container.is_some_and(|first| position >= first) {」，`first` 各自在 `:6922`、`:14110` 取一次。
  清单是一段后缀时，「序号不小于头一片」与旧写法的「在清单里」逐个相等；Verus 的 `ensures`（`first_rewritten_container` 那条 `forall`）罩着这一句。

按 `.claude/singlefs-ai-sop/rules/code-discipline.md:150`：「不变量被破坏是 bug，用断言」，这样用 `assert!` 合规；panic 消息写明了「造它的只有 append_records_to_the_open_container 与 carried」。
这几处断言不依赖盘上字节：清单是进程里算出来的，形状与容器出自同一版（`append_records_to_the_open_container` 开头 `assert!` 核过两者的片数相等）。

### 2.3 求祖先的两个新入口本身

`ancestor_suffix_starts_of_the_rightmost_containers`（`crates/singlefs-core/src/container_tree.rs:2078`）从最右端往左，一层只读到被标过的头一个就停；`ensures` 写的是「该节点是最右端那几片的祖先 <==> 序号不小于这一层的起点」。给 0 片或多于容器数时 `assert!` panic（`# Panics` 写了），调用方的 `rightmost_count == 0` 都在调用前分流（`rewritten_nodes` 与 `node_origins` 里各一处）。

### 2.4 条款与代码之间还差的一处（不在这一格的问法里，单列）

`.claude/kb/decisions/08-核心索引结构.md:516`：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」。
容器小树这一支，准入之前（`resolve` 在 `judge_the_publish_before_any_read_or_write` 里走，`crates/singlefs-core/src/transaction.rs:12536`：「let settled = settle_the_allocation_record_tree(」）还留着两处按全部容器、全部节点走一遍的：

- `crates/singlefs-core/src/transaction.rs:6923`：「for position in 0..tree.containers.len() {」（`mapped_units_of_a_container_tree`，由 `plan_the_multi_level_trees` 走到，`:9368` 起三棵树各一次）。
- `crates/singlefs-core/src/container_tree.rs:2926`：「let positions = self.shape.positions_in_bump_order();」（`node_origins` 里先列出全部节点）。

所以模块文档 `crates/singlefs-core/src/container_tree.rs:10`：「求祖先、判哪一片哪个节点重写了，都只读最右端那几段，不随容器总数涨」只在字面（「求祖先」「判…重写了」两件）上成立；包着它们的这一步仍是 O(容器数 + 节点数)。
这两处是遍历、不是照抄，E199 的跑前登记已点名不进模型：`research/prompts/e199-preregistration.md:196`：「是遍历，不是照抄」。它们不在正文那「三处照抄」里，也不在这一轮五格里。我只记事实，是否算这一轮的缺口由主 agent 判。

什么会推翻 T2：在 `append_records_to_the_open_container` 之外找到另一个造 `rewritten_containers` 的地方（`grep` 说没有），或某条发布路径给这三棵树的容器做了「删空一片」/「改左边的片」；或 Verus 里 `first_rewritten_container` 那条 `forall` 不过。

## 三、T3　孩子区间顺着累加与按位置取指针

结论：成立。三个小件各自成立，格里问的两件事（units 次序有没有在三处之外被改、断言合不合纪律）答案都是「没有」「合」。

### 3.1 孩子区间顺着累加

- `children_of` 删了：`grep -nF 'fn children_of(' crates/singlefs-core/src/container_tree.rs crates/singlefs-core/src/transaction.rs` 零命中。
- 新入口 `crates/singlefs-core/src/container_tree.rs:1941`：「pub fn children_of_every_node(&self) -> (children: Vec<Vec<Range<usize>>>)」，`ensures` 写的是每个节点的区间起点等于本层前面各节点孩子数之和、终点等于起点加自己的孩子数，和旧 `children_of` 的 `ensures` 是同一句话（补丁里被删的那条 `children_of` 的 `ensures` 与它同形，只差变量名）。
- 调用点只有一处：`crates/singlefs-core/src/transaction.rs:14156`：「let children_of_every_node = after.shape.children_of_every_node();」，每建一棵容器树的单元算一次，之后按 `[层下标][层内序号]` 取。位置来自 `after.node_origins()`，层级恒不小于 1，`usize::from(position.level) - 1` 不会下溢。

### 3.2 按位置取指针

- 取法：`crates/singlefs-core/src/transaction.rs:4018`：「pub fn pointer_below_the_root(」，下标是「这一层之前各层节点数之和 + 同层序号」，之后 `assert_eq!(listed_position, position, …)`（`:4037` 一带）再核一次那一格上记的就是这个位置。
- 前提是「`pointers_below_the_root` 恰按 bump 次序列根之下的全部节点」。两个造它的地方我都读了：
  - 发布：`crates/singlefs-core/src/transaction.rs:14263`：「nodes_below_the_root.push(ContainerTreeNodeBelowTheRootUnit {」，推的次序就是 `after.node_origins()` 的次序（`positions_in_bump_order`：层级升序、同层从左到右，根最末），根不推；装进 `ContainerTreeIndexNodesVersion` 时按这个 Vec 原序 `.iter().map(|node| (node.position, node.node.pointer))`（`crates/singlefs-core/src/transaction.rs:16636` 一带）。
  - 重建：`crates/singlefs-core/src/recovery.rs:5638`：「levels_from_the_root_down.reverse();」后按层级升序、层内从左到右展平成 `nodes`；`crates/singlefs-core/src/recovery.rs:5512`：「.filter(|(position, _, _)| !self.index_nodes.shape.is_the_root(*position))」去掉根。次序由读法决定，不取自盘上字节；盘上形状不合（每层孩子数之和对不上、根不唯一）时 `crates/singlefs-core/src/recovery.rs:5645`：「.ok_or_else(malformed)?;」交回 `Err`，不 panic。
- 所以 `.expect(...)` 与 `assert_eq!` 只会在进程自己的次序被写坏时响，不会被坏盘触发。纪律对照：
  `.claude/singlefs-ai-sop/rules/code-discipline.md:145`：「能恢复的失败」（I/O 错、盘上数据损坏、资源不够）走 `Result`——这里没有坏盘能走到的 panic；
  `.claude/singlefs-ai-sop/rules/code-discipline.md:150`：「不变量被破坏是 bug，用断言」、`.claude/singlefs-ai-sop/rules/code-discipline.md:151`：「不写 `unwrap()`，写 `expect`」——两样都照了，`# Panics` 写在文档里。
  结论：panic 不是返错，合规矩。
- 旧写法交回 `None` 的情形是「表里没有这一项」；新写法对「形状里有、又不是根、表里没有」改成 panic。这一类只在表与形状不是同一次造的时候才有，两个造法都是同一次，所以没有合法历史走到。
  测试里造过「表头一项记成要问的位置、带另一个指针」的反例（`a_node_pointer_below_the_root_is_taken_at_its_position_in_bump_order`），断言取的是下标那一格——这条测试把「逐项找」与「按下标取」分开了。

### 3.3 两个根单元的下标

`crates/singlefs-core/src/transaction.rs:3309`：「pub struct ContainerTreeRootUnitPositions {」与 `:3317`：「pub fn of_the_units(units: &[PublishedUnit]) -> Self {」。三处造 `TransactionOutput` 的地方各记一次：

| 地方 | 行 |
|---|---|
| 发布 | `crates/singlefs-core/src/transaction.rs:16584`：「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」 |
| 从盘上重建 | `crates/singlefs-core/src/recovery.rs:3394`：「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」 |
| 管理员回退 | `crates/singlefs-core/src/mount.rs:10178`：「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」 |

「三处之外还有没有造 `TransactionOutput` 的地方」：`grep -rn 'TransactionOutput {' crates --include=*.rs`，去掉签名、`impl`、注释之后只剩 `crates/singlefs-core/src/recovery.rs:3402`、`crates/singlefs-core/src/mount.rs:10179`、`crates/singlefs-core/src/transaction.rs:16585` 三处，加一个 checker-tier 里字符串字面量。

「三处之外 units 的次序有没有被改动」：我 `grep` 了 `crates` 下非测试、非 checker 的源码里对 `.units` 的 `push / insert / remove / retain / sort / swap / truncate / drain / clear / extend / splice / dedup / reverse / rotate / append / pop / resize / = `。命中只有四类，都不是 `TransactionOutput.units`：`crates/singlefs-core/src/region_scan.rs:899`（一张 map）、`singlefs-unit-graph` 里两个文件（别的结构）、`crates/singlefs-harness/src/memory_pool.rs:1175`（`PublishWrites` 里的下标表）。
对 `TransactionOutput.units` 的写只有 `crates/singlefs-core/src/transaction.rs:3925`：「version.units[*position].slot = slot;」，改的是槽，不挪项。产出新一版的路径只有上面这三个造法：发布（`publish_admitted`，`:16584` 在 `units` 装好之后才记）、重建、回退合成；整理的发布、截断、挂载续做都是发布，各自重装一份新的 `units`。
读的人拿下标再核 `identity`：`crates/singlefs-core/src/transaction.rs:4328`：「fn unit_at_the_recorded_position(」里 `get(position)` 加 `assert_eq!(unit.identity, identity, …)`，下标挪了会当场响。

什么会推翻 T3：在三处造 `TransactionOutput` 的地方之外找到改 `units` 项次序的代码（`grep` 说没有）；或造出一份 `pointers_below_the_root` 与形状不是同一次造的版本（例如测试里手拼的，测试之外没有）；或重建时盘上坏字节能让 `read_container_tree_index_nodes` 交出与 bump 次序不一致的 `nodes`（读法按层级展平，我没看到路径）。

## 四、T4　准入不遍历 inode 树

结论：成立（对格里的问法：n_live 与 n_记录不再遍历、每条改 inode 记录的路径在代码里都走维护、恢复数一遍不在准入热路径上）。另有两件不在问法里、但与这一格相邻的事实：逐版比对的测试取样比文档写的窄（4.3）；准入之前同一个函数里还有一处拷整棵 inode 叶的（4.5，已被 E199 点名）。

### 4.1 改法做的是不是条款与判决说的

- 条款：`.claude/rules/fs-design.md:27`：「必须在事务提交时增量维护出来」；用户定案 `records/2026-10-13-里程碑四预开发.md:1341`：「五处加准入现在改，根源先量」，其中一件是「准入改成提交时增量维护 n_live、记录数」。
- 结构：`crates/singlefs-core/src/admission.rs:288`：「pub struct InodeRecordCounts {」，随一版带着（`TransactionOutput.inode_record_counts`，`crates/singlefs-core/src/transaction.rs:16701`：「inode_record_counts: resolved.inode_record_counts,」）。
- 准入读它：`crates/singlefs-core/src/transaction.rs:6979`：「live_objects: resolved.inode_record_counts.live_objects,」与 `:6982`：「inode_records: resolved.inode_record_counts.records,」。旧的两个遍历函数 `live_objects_of_the_inode_tree`、`inode_records_of_the_inode_tree` 在冻结副本里 `grep -rn` 零命中，已删净。
- 同一个函数里，inode 树和两个数用的是同一份输入：`crates/singlefs-core/src/transaction.rs:8334`：「let mut record_writes = self.inode_record_writes();」，写 inode 树的 `write_records_into_leaf_containers` / `remove_record_from_leaf_containers` 与改数的 `after_writing` / `after_removing` 吃的都是它与同一个 `previous`（`crates/singlefs-core/src/transaction.rs:8392`：「let inode_record_counts = match deletion_and_lineage.step {」）。
  inode 树的写入口全仓只有这一处（`grep -rn 'write_records_into_leaf_containers\|remove_record_from_leaf_containers' crates`，去掉测试与 `inode_tree.rs` 自身，只剩 `transaction.rs` 的 `:8367`、`:8376`），所以「改 inode 记录却没改数」只能是这个函数里的两个 `match` 对不上。

### 4.2 格里列的六条路径逐条

两个 `match` 都按 `DeletionStepOfThePublish` 穷举、没有 `_ =>` 臂（合 `.claude/singlefs-ai-sop/rules/code-discipline.md` 的「分支：穷举写全」），新加一个步骤变体会编译不过。

| 路径 | inode 树怎么变 | 数怎么跟 |
|---|---|---|
| unlink 的 nlink 归零（`Intent`） | `record_writes` 推 `record_with_nlink_zero`，原地换 | `after_writing`：换掉的那条原来活、现在不活，n_live 减一，n_记录不变 |
| unlink 中间批（`Batch`） | 推 `record_after_the_batch`，原地换 | `after_writing`，活性不变 |
| unlink 最后一批（`FinalBatch`） | `assert!(record_writes.is_empty(), …)` 之后 `remove_record_from_leaf_containers` | `crates/singlefs-core/src/transaction.rs:8393`：「DeletionStepOfThePublish::FinalBatch { inode } => inode_record_counts_before.after_removing(」，上一版那条按叶序折半找，不在时 `expect`（前面的 remove 已经拒过） |
| 截断改 size（`TruncationIntent`、`SizeChangeWithoutAnIntent`、`PartialWrite`） | 推记录，原地换 | `after_writing` |
| 截断中间批与最后一批（`TruncationBatch`、`TruncationFinalBatch`） | 不推记录（size 在意图那次已改） | `after_writing` 写零条，数不变 |
| 建快照、删快照、整理 | 整理那趟是 `file: None`、`new_inode_records: &[]`、步骤 `NotADeletion`（`crates/singlefs-core/src/transaction.rs:10844` 起），写零条；建删快照要写 inode 记录也只能经 `record_writes`（没有第二个入口，见上） | 同一个 `match`，写零条时数不变 |
| 管理员回退 | 合成的上一版 inode 叶取目标那一版的 | `crates/singlefs-core/src/mount.rs:10234`：「inode_record_counts: target_version.inode_record_counts,」，与 `inode_leaf_containers: target_version…` 同一份来源（`crates/singlefs-core/src/mount.rs:10223`） |

建 inode（`CreateInode`、`CreateInodes`）与写文件版本也在 `record_writes` 里：`after_writing` 对「上一版没有这个号」的记录 n_记录加一、n_live 按活性加。

`after_writing` 对「写进来的号在不在树里」的两分，和 `write_records_into_leaf_containers` 的落法一致：`crates/singlefs-core/src/inode_tree.rs:1016`：「它的 inode 号已经在树里 ⇒ **原地换掉那一条**」，大于树里每个 key 追加，中间插入和不严格升序整批拒（`# Errors` 三格）。拒了就不会走到改数那一步（`?` 在前）。

### 4.3 维护值与现数不等时，谁会先看见

- 核心一侧没有任何运行期比较（维护值一路链下去，只有冷恢复那一刻重数一遍当起始值，会把漂移悄悄抹平）。能自己响的只有减法：`crates/singlefs-core/src/admission.rs:341`：「counts.live_objects = counts.live_objects」那一行减 `is_live(&replaced_record)`，`live_objects` 已经是 0 却要换一条活记录时下溢；`Cargo.toml:23`：「overflow-checks = true」在 release 档也开，下溢会 panic，不会回绕。这是极端漂移才响的。
- 测试一侧：`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs:35`：「fn counted_here(version: &TransactionOutput) -> InodeRecordCounts {」自己另写一份数法，历史里每一步之后拿会话现行那一版的维护值比（`:155`：「version.inode_record_counts, counted,」）。
  但它取样的广度比文档写的窄，这是我读 `GenerationWeights` 数出来的：这份测试只用两组比重，`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs:226`：「&GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,」与 `:247`：「compare_the_counts_over_the_seeds(&GenerationWeights::BROAD)」。
  这两组在 `crates/singlefs-harness/src/history.rs:1441`、`:1236` 起的定义里，抽到的操作种类只有 CloseAndMountWritable、ColdStartRecover、CrashRecoveryAbandoningTheNewestRoot、CreateInode、PublishFirstFile、PublishOverwrite、PublishWithoutUnits、RaiseRollbackFloor、RollBackWhileMounted、Unlink（我用 `awk` 取两段定义再 `grep -o 'HistoryOperationKind::…' | sort | uniq -c` 数的）。
  `crates/singlefs-harness/src/history.rs:987` 的 `HistoryOperationKind` 里有而这两组一次都不抽的：Truncate、CreateSnapshot、DestroySnapshot、CompactWhileMounted、CreateInodes、WriteOfAnInode、WriteAtOffset 等。
  所以测试的 `:243` 那句文档「快档那组比重（建文件、覆盖写、建 inode、unlink、回退、冷启动读回、可写挂载、快照、整理都抽）」与 `BROAD` 的定义对不上（快照、整理 `BROAD` 没有；`BROAD_WITH_COMPACTION_WHILE_MOUNTED`、`SNAPSHOT_OPERATIONS`、`TRUNCATION`、`PARTIAL_WRITES`、`WRITES_BY_INODE_NUMBER` 才有，这份测试没用）。
  结论：截断（意图、变长、部分写）、建删快照、整理、多 inode 建与写这几条路径，维护值与现数的逐版比对**没有直接取样**；它们靠的是 4.2 那条结构论证（同一个 `record_writes`、同一个 `previous`、同一个穷举 `match`）和 checker 的 I-3.20 (b)。这是方法一侧的缺口（攻方腿的格），不改 4.2 的结论，我记在这里。
- checker 一侧有，而且是独立算法：`crates/singlefs-checker/src/walk.rs:22647`：「fn judge_the_committed_reservation_against_its_closed_form(」（I-3.20 (b)），闭式要的 n_live 与 n_记录由 checker 自己沿盘上的树表、inode 树读出（`crates/singlefs-checker/src/walk.rs:22599`：「let live_objects = records.values().filter(|record| record.nlink != 0).count();」）。
  因为标签 6 是用维护值写进记账行的，维护值错了，盘上的标签 6 就与闭式对不上，checker 在那一版是最新根的镜像上会判红。限度：它只判最新根。
- 回答格里的问：核心自己的运行期不会先看见；先看见的是测试里的逐版比对（只对建文件、覆盖写、建 inode、unlink、回退、可写挂载那几条路径），其次是 checker 的 I-3.20 (b)（最新根，所有路径）。

### 4.4 恢复数一遍在不在准入热路径上

`crates/singlefs-core/src/recovery.rs:3396`：「let inode_record_counts = InodeRecordCounts::counted_over_the_inode_tree(」在 `rebuild_version…` 里。它的调用者我 `grep` 到三类：挂载时重建上一版（`crates/singlefs-core/src/mount.rs:1330`：「fn rebuild_previous_version<Device: BlockDevice>(」）、管理员回退重建目标那一版（`crates/singlefs-core/src/mount.rs:10345`）、管理员回退为算 inode 号水位把环里每一条根各重建一遍（`crates/singlefs-core/src/mount.rs:10038`：「fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(」）。三类都是整版从盘上读，数一遍只是在读完的叶上再扫一次，没有一条在准入（`judge_the_publish_before_any_read_or_write`）里。叶容器非空由重建时判（`crates/singlefs-core/src/recovery.rs:3712`：「"inode 叶容器没有记录",」），折半找的前提因此成立。

### 4.5 条款字面还差的一处（不在这一格的问法里，单列）

`.claude/kb/decisions/08-核心索引结构.md:516`：「准入之前的步代价不随盘上容器总数、记录总数涨」。
同一个函数里，`ordinary_allocation_part` 在 `crates/singlefs-core/src/transaction.rs:8331`：「let containers_before = previous」处调 `inode_leaf_container_contents`（`crates/singlefs-core/src/transaction.rs:4205`：「pub fn inode_leaf_container_contents(&self) -> Vec<InodeLeafContainer> {」），把上一版全部叶容器的记录拷一份；它在 `judge_the_publish_before_any_read_or_write` 里被调（`crates/singlefs-core/src/transaction.rs:12511` 一带），每轮固定点还会再调。所以准入这一步的「读数」不再遍历 inode 树，但准入之前同一个发布里仍有一次随记录总数涨的拷贝。
这一处不是正文列的「三处照抄」（那三处是形状的 `clone`），E199 的跑前登记已把它列进甲臂表：`research/prompts/e199-preregistration.md:184`：「上一版 inode 叶容器内容」。我只记事实，不判算不算这一轮的缺口。

什么会推翻 T4：找到一条会改 inode 记录、却不经 `ordinary_allocation_part` 里那两个 `match` 的路径；或一个历史让 `inode_record_counts_kept_with_each_version` 的逐版比对红；或 I-3.20 (b) 在一条走完 `after_writing` 的版本上红。我读码没找到，历史没跑（待主 agent 跑，见第七节）。

## 五、T5　整理门与根环成形

结论：部分成立。G7 的门：成立（core 与协议这一门是同一件事，靠结构、没有运行期核；我没找到让 core 发出这一门不许的整理发布的历史）。Y5 的两条：换算算术对；但「在 core 的值上成立」没有任何代码或证明在罩，规格没有产品代码的使用者，不能记成立。

### 5.1 G7：协议这一门写的是什么

- 门：`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:250`：「RunRingStep::CompactionCheckpoint => state.roots.len() >= 1」接 `:251`：「&& state.head.logical == state.roots[state.roots.len() - 1].view.logical,」。r1 判决里打中的旧写法是 `research/prompts/m4-close-code-r1-main-verification.md:33`：「RunRingStep::CompactionCheckpoint => true,」。
- 投影引理多的那条等式：`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:1409`：「&&& run_ring_step_is_enabled(state, step) == compaction_publish_is_accepted(observe_the_run_ring(state))」，右边的定义在 `:1384`：「pub open spec fn compaction_publish_is_accepted(observation: RingObservation) -> bool {」，两个合取项（至少一个根、现行内容等于最后一个根的内容）与 `:250-251` 逐项相同。
- 门拦下的那段历史：`:1090`：「pub proof fn lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root()」。我手核了这条引理 `ensures` 里的各个合取：起点一个根、现行 = 根；搬迁（`Relocate`）只换映射、不换逻辑内容，整理的发布放行；`UserPublish { new_view: deleted }` 只换现行（`:297` 一带的 `UserPublish` 臂，不推根），现行逻辑内容为空而根里有 key 7，所以整理的发布不放行；`UserCheckpoint`（`:307` 一带）把现行推成根，现行 = 根，又放行。`ring_state_of` 八个实参次序与定义（`:105`：「pub open spec fn ring_state_of(」）对得上。我没起 Verus，这是读规格，不是证明被检验过（主 agent 经服务跑的 725 / 155 verified 在背景材料里，我没复跑）。

### 5.2 core 做的是不是同一件事

协议里「现行领先最后一个根」对应 core 里的什么：协议里 `UserPublish` 只换现行、`UserCheckpoint` 才推根；core 的不经会话的入口每次改动同步发布，条款原话 `.claude/kb/decisions/16-发布语义.md:269`：「仍每次改动同步发布」，所以没有「现行已改、根还没落」这一态；唯一的类似物是攒批会话的待发（收下、还没发布的改动），D16 把整理列进屏障：`.claude/kb/decisions/16-发布语义.md:271`：「unlink、截断、建删快照、整理、回退、卸载是屏障，进门先排空待发、自己单发」。

代码上这条屏障：

- 攒批会话把底下的会话和待发都设成私有字段（`crates/singlefs-core/src/mounted_session.rs:4743`：「session: MountedSession,」），外面只有 `crates/singlefs-core/src/mounted_session.rs:5163`：「pub fn session(&self) -> &MountedSession {」这一个只读口；这个类型的 `pub fn` 里没有显式整理入口，只有 `crates/singlefs-core/src/mounted_session.rs:5565`：「pub fn run_compaction_in_the_background<Device: BlockDevice>(」。显式的 `run_compaction`（`:2207`）在裸会话上，而裸会话没有待发。
- 排空在起跑判过之后、跑之前：`crates/singlefs-core/src/mounted_session.rs:5597`：「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」；排空没发成走 `Err` 分支返回（`:5614` 起的 `return Err(`），不跑整理。没起跑就不排空、也不整理。
- 排空之后整理的每一批发布接在会话的现行那一版后面。`crates/singlefs-core/src/transaction.rs:10844`：「fn publish_compaction_step_after_its_check<Device: BlockDevice>(」里 `PublishPlan` 的 `file: None`、`new_inode_records: &[]`，步骤 `NotADeletion`，谱系 `NoLineageRecord`，树基是 `Some(previous)`，所以新根与上一个根逻辑内容相同、只换映射；同一个 `PublishPlan` 里 `crates/singlefs-core/src/transaction.rs:10879`：「rollback_floor: previous.root.rollback_floor,」——带的是上一版根的持久 F，原样不变。
- 冻结时：`crates/singlefs-core/src/mounted_session.rs:3002`：「整理那一趟自己的发布本来就在发布路径第一道被冻结拒」；发布路径第一道是 `crates/singlefs-core/src/transaction.rs:1541`：「fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator) -> Result<(), PublishError> {」。冻结时现行那一版可能领先最后一个落盘的根（`crates/singlefs-core/src/mounted_session.rs:5082`：「报错之后这次发布可能已成也可能未成」），整理在这一态被拒，与协议的门同向。

### 5.3 有没有一条历史让 core 发出协议不许的整理发布

我逐条找了，没找到：

| 候选历史 | 为什么发不出 |
|---|---|
| 待发非空时起整理 | 起跑后先排空；排空被拒就返回错、不跑（`crates/singlefs-core/src/mounted_session.rs:5597`、`:5614`） |
| 持有裸会话又持有一份待发 | 待发只住在攒批会话里，裸会话没有；攒批会话只交出只读口 |
| 冻结之后整理 | 发布路径第一道拒 |
| 回退之后整理 | 回退是一次向前的发布，立即成根（`crates/singlefs-core/src/mount.rs:10178` 起合成的上一版是回退发布的底，`crates/singlefs-core/src/mount.rs:10234` 的 `inode_record_counts` 取目标那一版）；之后现行 = 最后一个根 |
| 挂载之后第一次整理 | 现行是恢复选出的那一版，或写行那次发布的输出，两样都是已落成根的版本（我没有逐行追 `establish_instance`，这一行是读法，不是追过的） |
| 排空途中先推抬 F 的空发布 | 空发布也是发布、也成根（`.claude/kb/decisions/16-发布语义.md:254`：「空发布也是发布」），现行 = 根 |

协议的门是「逻辑内容相等」（映射可以不同），core 的整理批在同一次发布里搬单元并成根，所以 core 的每一次整理发布只是协议允许历史的一个子集（搬迁与整理的发布合成一步）。反过来，协议允许而 core 不发的：用户改了现行、还没落成根时，协议放行搬迁（`Relocate`）但不放行整理的发布；core 在这一态（待发非空）两样都不做。

限度：这条对应是**结构上的**，不是运行期核出来的。core 里没有一行比较「现行逻辑内容」与「最后一个根」；它成立靠的是「core 里唯一能让现行领先根的东西（待发）在整理入口被排空」这一条 API 纪律。以后谁给攒批会话加一个不排空的整理入口，协议的门在 core 里就没人拦（这一轮不改 `mounted_session.rs`，正文已说明）。

### 5.4 Y5 两条：换算对不对

补的两条在 `crates/singlefs-core/src/verified_model.rs:218`：「&&& ring.in_memory_floor.0 <= ring.roots.last().checkpoint_txg.0」与 `:219`：「&&& forall|index: int| #[trigger] root_is_protected(ring, index) ==> ring.roots.len() - ring.window <= index」。协议里对应的是 `crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:129`：「&&& state.in_memory_floor < state.roots.len()」与 `:131`：「&&& state.roots.len() - state.persisted_floor <= state.window」。

换算：把 F（一个 checkpoint 号）对到下标 p = 第一个号不小于 F 的根。`roots_ascend_by_checkpoint_txg` 成立时（严格递增）：

- 「内存 F 之上至少一个根」⇔ 存在号不小于 F 的根 ⇔ 最后一个根的号不小于 F，这就是 `:218`。方向与边界（等号）都对。
- 受保护的根 `root_is_protected`（`crates/singlefs-core/src/verified_model.rs:277`，号不小于持久 F）是一段后缀 [p, len)，`:219` 说 p ≥ len − window，即后缀长度不超过 window，与协议 `:131` 同；窗口比根数大时右边为负，平凡成立，与协议里自然数减法的行为一致（协议里 `len − F ≤ window` 在 F 取 0 时同样平凡）。
- 补的见证 `witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window` 的三个根环我手核：基础根环五条都满足；内存 F = 3 只破 `:218`；容量 = 1 只破 `:219`。

「在 core 的值上成立吗」读不出来，原因有四：

1. 没有使用者。`grep -rn 'root_ring_is_shaped\|RootRingModel\|root_is_protected' crates`，去掉 `verified_model.rs` 自己，只剩 `crates/singlefs-core/verus-must-red-worlds.tsv:454` 一行必红世界；文件头自己写明：`crates/singlefs-core/src/verified_model.rs:5`：「这份文件是规格底座：只放 spec 与钉住它们取值的见证（proof fn），不接协议核，也不被产品代码调用」。没有任何一条 ensures 把 `RootRingModel` 的字段连到 core 运行时的 F、根号、根环容量。所以这两条是「规格里补了」，不是「core 的值满足」。
2. 内存 F ≤ 最后一条根的号：健康的池上由抬 F 的上限保证（上限取各盘最新持久有效根的最小，卸载抬到现行那一版的号，D16 的「抬 F 的上限」一行），都不高于池里最新的根（`.claude/kb/decisions/16-发布语义.md:38`：「min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」）。但 core 自己记了一个反例形：`crates/singlefs-core/src/mount.rs:3407`：「F_生效 高于某块盘上最新的有效根」（卸载抬 F 的一串写了系统配置里的新 F、带新 F 的根没落满每块盘就崩，再丢一块盘）。模型里没有丢盘这一步（崩溃是两份同时崩），所以不冲突；但「roots 取全池还是取某块盘上的」要在桥里写明，否则这一条在丢盘的形上不成立。
3. 持久 F 之上的根装得进根环：协议里整理的发布在装满时把持久 F 加一（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:211`：「pub open spec fn persisted_floor_after_a_compaction_publish(state: RunRingState) -> nat {」）；core 的整理发布不抬 F（上面 `crates/singlefs-core/src/transaction.rs:10879`），保护靠回收门槛取大：`crates/singlefs-core/src/mount.rs:1532`：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」，环里最旧的根被盖掉时门槛自己前移。所以 `roots` 取「环里实际在的根」时 `:219` 近乎平凡（根数不超过容量）；取「全部历史」时它要求 F 追着环走，而 core 不这么做。桥要选前一种读法。
4. 号严格递增只对「按实例表仍然有效」的根成立：崩溃后 checkpoint 号会被重发（`.claude/kb/decisions/08-核心索引结构.md:227`：「跨崩溃不成立：崩溃后 checkpoint 号会被重发」），盘上的原始根槽里可以同号并存；`.claude/kb/decisions/16-发布语义.md:36`：「按实例表判仍然有效的根的最小 checkpoint_txg」这一行取的是有效根。桥里的 `roots` 必须先过实例表这一滤，否则下标↔号的换算（上面第一段）不成立。

什么会推翻 T5：G7 一侧，找到一条历史让 `run_compaction_in_the_background`（`:5565`）在待发非空或现行领先最后一个根时发出整理的发布；Y5 一侧，造出一个 core 运行时的状态（健康池、有效根按号递增）里内存 F 大于最后一条根的号，或持久 F 之上的有效根多于环容量；或桥建起来之后上面第 2、3 点的读法对不上。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 开片原地改与上一版只记节点数 | 成立 | 开片的 `ensures` 仍对着没改的规格函数，长高、填满一层、跨层三形由 Verus 罩；上一版现在只被问层数、有没有节点、根在哪，旧代码也只问这三样，没有读孩子数的判断丢了 |
| T2 求祖先从最右端往左 | 成立 | 三棵容器小树只增不减，清单只有追加造出来，六条路径要么追加最右端、要么不碰这三棵树；前提破了是 `assert!` 当场 panic，不静默。另：准入之前仍有按全部容器、全部节点走一遍的两处（2.4），E199 登记已点名不进模型 |
| T3 孩子区间顺着累加与按位置取指针 | 成立 | 三个造 `TransactionOutput` 的地方各记一次下标，`units` 次序在这之外没人动；指针表次序由造法决定、不取自盘上字节，断言只响进程自己的错，panic 合 `code-discipline.md` 的「不变量被破坏是 bug」 |
| T4 准入不遍历 inode 树 | 成立（代码） | 数与 inode 树吃同一份 `record_writes` 与同一个 `previous`、两个穷举 `match`，没有第二个写入口；恢复数一遍不在准入里。两件相邻事实：逐版比对的测试只抽 10 种操作，不含截断、快照、整理、多 inode（4.3）；准入之前仍拷一遍全部 inode 叶（4.5，E199 甲臂表有） |
| T5 整理门与根环成形 | 部分成立 | G7 的门成立（core 靠「待发在整理入口被排空」这条结构，没有运行期核，没找到违反的历史）；Y5 两条的下标↔号换算算术对，但 `RootRingModel` 在产品代码里没有使用者，「在 core 的值上成立」没人罩，换算还要桥里写明 roots 取什么 |

## 七、要主 agent 跑的命令

我没起任何脚本，下面是读码之后想让主 agent 核的，每条标了轻重。命令前缀照项目惯例，我没核 `.cargo/config.toml` 现在要不要加前缀。

1. T1–T3 补丁自带的容器小树测试（轻型）：
   `task-server/task submit -- cargo test -p singlefs-core --lib container_tree::tests`
   `task-server/task submit -- cargo test -p singlefs-core --lib a_node_pointer_below_the_root_is_taken_at_its_position_in_bump_order`
2. T4 的逐版比对（重型，取样 48 个种子 × 30 步 × 两组比重）：
   `task-server/task submit -- cargo test -p singlefs-harness --test inode_record_counts_kept_with_each_version`
   这条只验证 4.3 说的那 10 种操作。要验截断、快照、整理、部分写、多 inode：需要把这份测试的比重换成 `GenerationWeights::TRUNCATION`、`SNAPSHOT_OPERATIONS`、`COMPACTION_WHILE_MOUNTED`、`PARTIAL_WRITES`、`WRITES_BY_INODE_NUMBER`、`SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST` 各跑一遍。改测试属于拷来改的装置，我不交；主 agent 要的话新派一条腿做，派发提示写「承接：research/prompts/m4-close-code-r2-sonnet-output.md 第四节 4.3」。
3. T5 的 Verus 轻型（已有任务 4486、4802 的读数，我没复跑；判决要引的数照旧由主 agent 复跑）。

## 八、没做什么

- 没替主 agent 采纳或出判决；没跑任何测试、Verus、脚本，没往统一任务服务交任务；判决要引的数没有复跑，第七节的命令都还没跑。
- 没写模型，没有模型目录，也就没有复跑命令与 sha256。
- 没判别的腿的格：对拍取样点是否罩住改法、必红世界行（51 行）是否真罩住、钉代价的测试是否真会红，是攻方腿的格。4.3 里「取样比文档写的窄」是我读 `GenerationWeights` 顺手数出来的事实，原样记下，没往下判。
- 没读攻方腿这一轮的报告与模型（禁读清单）。r1 的判决与腿报告我读了（背景材料指到它们）。
- 没追 `establish_instance`（挂载之后的第一个现行版本）；5.3 表里那一行是读法，已标明。
- 没验 Verus 的 `ensures` 在补丁之后是否仍过（背景材料里的 725 / 155 verified 是主 agent 的读数）。
