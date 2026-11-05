# m4-admission-delta-r1 正推腿报告（Sonnet，2026-11-05）

腿：三方推论正推腿。输入：`research/prompts/_m4-admission-delta-r1-body.md` 第四节 A1–A5。没写模型，没跑任何命令（派发提示：重型测试不跑）。

约定：
- 代码行号取冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/`（下称「冻结副本」）。主工作区里 `transaction.rs`、`container_tree.rs`、`mounted_session.rs` 之后挪过行，别的文件我 `cmp` 过与冻结副本逐字节相同。
- 标注：「读」= 读冻结副本源码所得，`grep -n` 可复核，没运行；「推」= 由读到的事实推的，没跑；「没量」= 用时与内存没量过。
- 符号（都是「读」到的、随它涨的量）：R_i = inode 树记录总数；C_t / C_r / C_l = 墓碑 / 退役 / 谱系容器片数；F = extent 上段有条目的 inode 数；P = 一版 `units` 总数；M = 一版 `mapped_units` 总数；A = 记账行数；S_c = 本次挂载开过的聚簇段数；R_a = 分配器 `records` 总数；E_d = deadlist 条目数；E_k = 为快照留存的映射条目数；Δ = 这一次碰到的容器、节点、记录数；h = 树高。
- 「落点判」= `admit_the_units_landing_on_every_device`（C545 加的那一道）；「式子判」= D28 已定项 1 的式子（`admit_on_every_device`）。为什么要分两个名字见第 0 节。

## 0　先说一件背景材料与实现对不上的事：发布路径里有两道准入，多层码 2 树的规划夹在两道中间

正文第二节把 `plan_the_multi_level_trees`、`mapped_units_of_a_container_tree` 与 `admit_the_units_landing_on_every_device` 放在一起讲，实现员报告也写「这几步都在 `resolve`（`:8718`）里、准入（`:12513` 起）之前」。冻结副本里 `judge_the_publish_before_any_read_or_write` 一个函数里有两道准入，多层码 2 树的规划夹在两道中间。

整行并排（冻结副本，同一个函数 `judge_the_publish_before_any_read_or_write`，`crates/singlefs-core/src/transaction.rs:12489` 起）：

```
12511:         .ordinary_allocation_part(previous, trees, deletion_and_lineage)?
12523:         admit_the_units_landing_on_every_device(
12536:     let settled = settle_the_allocation_record_tree(
12621:         admit_on_every_device(
13063:         let resolved = plan.resolve(
 8828:         } = self.plan_the_multi_level_trees(
```

读：`resolve`（8718）只有一个调用点，13063（冻结副本 `crates` 下 `grep -rn '\.resolve(' --include=*.rs` 除 `singlefs-unit-graph` 外只命中这一处），在 `settle_the_allocation_record_tree`（13017）里；`settle_...` 在 12536 才被调，排在落点判（12523）之后、式子判（12621）之前。`plan_the_multi_level_trees` 只被 `resolve` 调（8828），`mapped_units_of_a_container_tree` 只被 `plan_the_multi_level_trees` 调（9368 墓碑树、9374 谱系树、9392 退役树，三处）。复核命令：`grep -n 'plan_the_multi_level_trees\|\.resolve(\|fn resolve' crates/singlefs-core/src/transaction.rs`（在冻结副本里起）。

所以：
- 落点判之前的步骤，不含多层码 2 树的规划（记账树、deadlist 树、中央映射树）、不含 `mapped_units_of_a_container_tree`、不含 `node_origins`；含的是 `ordinary_allocation_part`（8325）与 ckpt_cost 的一串（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`，`crates/singlefs-core/src/admission.rs:1211`）。
- 式子判之前的步骤，含上面全部再加 `settle_the_allocation_record_tree` 里按轮重跑的 `resolve`。正文第二节点名的几处大都在这一段。
- 条款字面（`.claude/kb/decisions/08-核心索引结构.md:516`「`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表里运行时决策路径「代价不许随盘容量增长」，在这一格读成只管准入之前（用户 2026-10-31 定，原话在变更史）：准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」）和 fs-design 那张表（`.claude/rules/fs-design.md:23`「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」）说的是「ENOSPC 准入」，没分哪一道。代码自己的注释也把式子判之前叫「准入之前」（冻结副本 transaction.rs 第 7978 行的注释开头写「准入之前算记账树形状」，转述；它说的记账行表是 `resolve` 里用的那一份，在落点判之后）。
- 下面 A1 两张表分开列：表一 = 落点判之前（字面区间），表二 = 落点判到式子判之间（正文点名的几处在这里）。两张表合起来才是「ENOSPC 准入之前」。

这一节的判定：正文第二节对「哪几步在准入之前」的写法 **冲突**（只对式子判成立）；A1 字面区间（到 `admit_the_units_landing_on_every_device` 为止）要换一份清单。

## A1　准入之前每一步随哪个总量涨

### 表一　落点判之前（`prepare_the_version_publish` → `prepare_the_version_publish_relocating_units`（12659）→ `judge_the_publish_before_any_read_or_write`（12489）开头到 12523）

「类型」：拷 = 把上一版的一份东西再复制一份（换表示就能去掉）；扫 = 算法本身要看全量（换表示去不掉）。

| # | 步 | 位置（冻结副本） | 随什么涨 | 类型 | 说明 |
|---|---|---|---|---|---|
| 1 | 逐盘取身份 | `transaction.rs:12501` 起 `device_identities_of_the_accounting_rows` | 盘数 | — | 常数级 |
| 2 | 拷全部 inode 叶容器 | `transaction.rs:8332` → `transaction.rs:4205`（`inode_leaf_container_contents`，4208 行逐片 `contents.clone()`） | R_i | 拷 | 容器数有条款上限：`crates/singlefs-core/src/inode_tree.rs:1065`「Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers, capacity }) => capacity == 135」，一片至多 233 条（`crates/singlefs-core/src/inode_tree.rs:157`「self.records.len() <= 233,」），所以 R_i ≤ 135 × 233 = 31455（推），这一步今天的总量是封顶的 |
| 3 | 写者再拷一遍 | `crates/singlefs-core/src/inode_tree.rs:1093`「let mut containers = copy_of_the_leaf_containers(containers_before_this_publish);」；删最后一批走 `remove_record_from_leaf_containers`（`inode_tree.rs:1692`，进门 `LeafContainersBeingMerged::from_the_previous_version`，逐行没核它是不是整份拷，写「读到名字、没核内部」） | R_i | 拷 | 同封顶 |
| 4 | 每条写的记录找上一版里的旧记录 | `transaction.rs:8390` → `transaction.rs:4188`（`inode_record_found_by_its_leaf`，`partition_point` 找叶再折半） | log(容器数) × 写入条数 | — | 不随总数涨（对数）；r2 判决 T4 那一格已改 |
| 5 | 改 size 的几种步骤找旧记录 | `transaction.rs:8419` → `transaction.rs:9501`（`size_change_of_a_file`）→ `transaction.rs:9507` → `transaction.rs:4176`（`inode_record_of`，`flat_map` 加 `find`，逐条线性） | R_i（封顶同上） | 扫 | 只在截断、变长、部分写这几种步骤走；写文件、unlink 不走 |
| 6 | 第一个文件之外的文件逐个造一份计划 | `transaction.rs:8473` → `transaction.rs:7660`（`plan_the_files_other_than_the_first_after_this_publish`，7674 起对上一版每个文件 `map` 出一份并拷它的 `extent_lower_nodes`） | F_n（第一个文件之外有条目的文件数）× 各自下段节点数 | 拷 | 加 `file_other_than_the_first`（`transaction.rs:4165`）线性 `find` |
| 7 | extent 树计划 | `transaction.rs:8551`（写第一个文件）、8527 / 8536（删除批）、8562（写别的文件）、8503（别的文件改 size）→ 都先 `transaction.rs:4090`（`inodes_with_an_extent_upper_leaf_entry`，把全部有条目的 inode 收成 `BTreeSet`）→ `transaction.rs:7470` 起 `extent_tree_plan_with_the_unified_upper_segment` → `crates/singlefs-core/src/extent_tree.rs:212`（`plan_the_extent_upper_segment`） | F，加上段节点数 U（`extent_tree.rs:231` 附近 `previous_upper_nodes.contains` 是对 `Vec` 的线性 `contains`，所以是 U 的平方级，读） | 扫 | `crates/singlefs-core/src/extent_tree.rs:180`（`canonical_upper_segment_of_the_inodes_with_an_entry`）对每个 inode 走一条上段路径：F × 根层级 |
| 8 | deleted_inodes | `transaction.rs:8590`（`deleted_inodes_after_this_publish`，6213 行 `before.to_vec()`）、`crates/singlefs-core/src/deleted_inodes.rs:57` | 队列长 | — | `crates/singlefs-checker/invariants/09-inode树结构.md:36`「I-9.24 | 删除队列至多一条 key」，所以是常数（推，由不变量推） |
| 9 | 墓碑树 | `transaction.rs:8601`–8606（逐片 `contents.clone()`）；追加那一步（Intent）→ `crates/singlefs-core/src/tombstone_tree.rs:164` → `crates/singlefs-core/src/container_tree.rs:3575`（frozen，`copy_of_the_containers`）与 3576（`shape_before_this_publish.clone()`）；其它步走 `transaction.rs:8638`（`TombstoneTreeAfterThisPublish::carried` + `tombstone_shape_before.clone()`） | C_t（每片至多 815 条）；形状拷贝随 C_t / 节点数 | 拷 | 容器号可到 u64（没有 135 那种封顶） |
| 10 | 退役树 | `transaction.rs:8646`–8651 同样先全拷；`transaction.rs:8659` 的 `append_retirement_records_to_the_open_container` 每次发布无条件调（没有退役记录也调），里面又 `copy_of_the_containers` + `shape.clone()` | C_r（每片至多 959 条） | 拷 | 每次发布拷两遍，不看这次有没有退役 |
| 11 | logged_ops | `transaction.rs:8678`（`logged_operations_after_this_publish`，6301 行 `before.to_vec()`） | 活意图数 | — | 签发处 `assert!` 全池至多一条（`transaction.rs:6307`、6330），常数 |
| 12 | 取这一半重写的角色 | `transaction.rs:12511` → `OrdinaryAllocationPartOfThePublish::rewritten_roles`（8103）→ `container_tree_roles_rewritten`（6893，`rewritten_nodes()` 只走路径）与 `extent_tree_roles_rewritten`（7847，`flat_map` 遍历每个文件的 `lower_nodes` 再过滤） | 路径长；加 F_n × 下段节点数（第二项） | 扫 | 容器树那一半是 Δ·h，extent 那一半随 F_n |
| 13 | ckpt_cost 读四个根头 | `crates/singlefs-core/src/admission.rs:1211`（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`）→ `transaction.rs:4422`（`position_addressed_tree_heights_read_from_the_root_node_headers`，4426 行 `parse_index_node(&self.unit(root_role).bytes)`，三处）与 `transaction.rs:4404`（`height_read_from_the_root_node_header`，4410 行同样）→ `transaction.rs:4348`（`unit`）的 4351 行 `.find(|unit| unit.identity == identity)` | P（`units` 上线性 `find`，根在 bump 次序里排各树最末，推；一次 ckpt_cost 至多 4 次） | 扫 | 同一个文件里 `transaction.rs:4345` 的文档自己写了「在 `units` 上线性找：一次调用里只查几次的用它」。墓碑、退役两棵树的根头已经改成按装 `units` 时记下的下标取（`transaction.rs:3299` 的 `container_tree_root_unit_positions`、`transaction.rs:4273` 的 `tombstone_root_unit`），这四个根没改 |
| 14 | ckpt_cost 的记账树一项 | `admission.rs:1325`（`accounting_tree_nodes_after_the_publish_built_on`）：`admission.rs:1349`「.chain(version_to_build_on.inode_number_watermark_steps())」把全部记账行与台阶行收成 `BTreeSet`，`admission.rs:1358`「let nodes = match plan_the_tree_after_this_publish(」跑一遍全量规划器（`code_two_tree.rs:637`：`code_two_tree.rs:650` 收上一版全部 key，`code_two_tree.rs:653` 把上一版整棵树展成 `PlanningNode`） | A = 3 + 6 × 盘数 + S（S = 台阶行数） | 扫 | 台阶行随根 COW、一直留着（冻结副本 transaction.rs 第 4238 行的注释，转述），行内没有收缩；S 随水位变过的发布次数涨（读；有没有别处裁剪没找到）。这是落点判之前唯一一处跑多层码 2 树规划器的地方 |
| 15 | ckpt_cost 的中央映射树一项 | `admission.rs:1437`（`central_mapping_tree_nodes_at_each_level`，按层 `partition_point`）、`admission.rs:1484`（`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`，逐层算式）、条目数取 `mapped_units.len() + mapping_entries_kept_for_the_snapshots.len()`（O(1)） | h × log(节点数) | — | 不随总数涨（对数）；这是仓里已有的「差量上界」式子，A3 要用 |
| 16 | ckpt_cost 的分配记录树一项 | `admission.rs:1381`（`allocation_record_tree_positions_over_the_unit_areas`）→ `admission.rs:1413` 起逐盘逐层 | 盘数 × 根层级 | — | 几何上整棵树（K1），不读状态；值随单元区线性涨，算它不随 |
| 17 | 每盘取落点余量 | `admission.rs:992`（`of_every_device_of_the_allocator`）→ `crates/singlefs-core/src/allocator.rs:574`（`unblocked_slot_pairs_outside_the_cluster_segments`），`allocator.rs:578`「let pairs_inside_the_cluster_segments: u64 = cluster_segments」逐段 `map` 求和 | 盘数 × S_c（本次挂载开过的聚簇段数，`allocator.rs:2484` 每开一段 `insert`，只增） | 扫 | `transaction.rs:12509` 注释写「读的是分配器增量维护的两个数」，其中 `unblocked_slot_pairs_per_segment` 与 `unblocked_slot_pairs` 是增量的（`allocator.rs:549`、553 每次槽位状态翻转时更新），聚簇段内那一减项不是 |
| 18 | 拼要落的单元 | `admission.rs:1025`（`units_of_a_publish_to_land`）：两遍 `filter` 数角色 | 这一半角色数 | — | 随这次写的单元数，是 Δ |
| 19 | 判 | `admission.rs:1064`（`admit_the_units_landing_on_every_device`） | 盘数 | — | 常数级 |

表一小结（读）：落点判之前随总数涨的有 #2、#3（封顶 135 片）、#5（只在改 size 的几种步骤）、#6、#7、#9、#10、#12 的 extent 一半、#13、#14、#17。正文第二节点名的 `inode_leaf_container_contents`（#2、#3）在这一段；`mapped_units_of_a_container_tree`、`node_origins`、`plan_the_multi_level_trees`、中央映射树的 `BTreeSet` **不在**这一段。

### 表二　落点判之后、式子判之前（`settle_the_allocation_record_tree`，`transaction.rs:12536` 调，`transaction.rs:13017` 起）

每轮（最多 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT` = 64 轮，`transaction.rs:13141`；实际轮数没量）都重跑表里带「每轮」的行。

| # | 步 | 位置（冻结副本） | 随什么涨 | 类型 | 说明 |
|---|---|---|---|---|---|
| 20 | 上一版分配记录树节点集 | `transaction.rs:13045`（`previous_version.allocation_record_tree.node_set()`，进门一次） | 分配记录树节点数 | 拷 | |
| 21 | 每轮：分配记录树计划 | `crates/singlefs-core/src/allocation_record_tree.rs:429`（`plan_the_tree_after_this_publish`），入参两个 `BTreeSet<节点>` | 分配记录树节点数 | 扫 | 每轮 `rewritten.intersection(...)` 也是全集 |
| 22 | 每轮：`resolve` 重跑普通半 | `transaction.rs:8729` 又调一次 `ordinary_allocation_part` | 同表一 #2–#12 | 同表一 | 表一的行在这里再来一遍，所以落点判之前的开销也被乘进每轮 |
| 23 | 每轮：谱系树 | `transaction.rs:8734` 调、`transaction.rs:8928` 的 `lineage_tree_after_this_publish`：逐片 `contents.clone()`，管理员回退那次再 `append_lineage_record_to_the_open_container` | C_l | 拷 | |
| 24 | 每轮：deadlist 改动 | `transaction.rs:6540`（`deadlist_changes_of_this_publish`）：6581 行 `previous_version.mapped_units_by_role()` 先建全索引；6557 / 6578 建按需索引；6659 起 `entries_after` 把上一版全部 deadlist 条目收成 `BTreeMap`，随后把留存的映射条目全部收成 `BTreeMap` | M log M；E_d；E_k | 拷、扫 | 没有活快照时 E_d、E_k 为 0，但 6581 那一行建索引与有没有快照无关 |
| 25 | 每轮：多层码 2 树规划 | `transaction.rs:8828` → `transaction.rs:8976`（`plan_the_multi_level_trees`）：记账树 9015 起（`code_two_tree::plan_the_tree_after_this_publish`，全量规划器）、deadlist 树、`transaction.rs:9048` 又 `mapped_units_by_role()`、中央映射树 9399 起把 `mapped_units` 与留存映射条目收成 `BTreeSet<CodeTwoTreeKey>`、再 `plan_the_tree_after_this_publish`（同一个全量规划器：上一版全部 key、上一版整棵树展成 `PlanningNode`、`difference` 两趟、摊平、`replaced_previous_nodes` 过滤上一版全部节点） | M log M；A；E_d | 扫 | 正文第二节点名的主体；记账树这一项与表一 #14 是同一个函数在做同一件事 |
| 26 | 每轮：`mapped_units_of_a_container_tree` | `transaction.rs:9368`（墓碑）、9374（谱系）、9392（退役）→ `transaction.rs:6912`：6923 行 `for position in 0..tree.containers.len() {`、6937 行 `for (position, origin) in tree.node_origins() {`（`container_tree.rs:2926`（frozen）`self.shape.positions_in_bump_order()`，全部节点） | C_t、C_l、C_r 与各自节点数 | 扫 | 产出每片容器、每个节点一条，照抄的也各取一把上一版的 key（`carried`，按角色查索引） |
| 27 | 每轮：整理搬迁标路径 | `transaction.rs:9416` 起 `plan_rewriting_the_paths_to_the_moved_entries`：标路径本身是 搬迁 key 数 × h × log N，但尾部 `plan.replaced_previous_nodes = previous_shape.nodes().iter()...filter(!carried.contains)` 又走上一版全部节点 | 中央映射树节点数 | 扫 | |
| 28 | 每轮：释放判定 | `transaction.rs:12880`（`placements_released_by_this_publish`）→ `transaction.rs:4602`（`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`，4613 行在第一次查时建 `MappedUnitsOfAVersionReadByRole`，里面 `units_by_role()` 与 `mapped_units_by_role()` 两张全索引）、`transaction.rs:5118`（`mapped_roles_released_by_this_publish`，5132 行建 `FilesOtherThanTheFirstOfAVersionReadByInode`） | P log P；M log M；F log F | 扫 | 索引按需建，换下的角色里有一个要查就建；写文件的发布一定要查 |
| 29 | 每轮：演习分配器 | `transaction.rs:13098` 行「let mut rehearsal = allocator.clone();」 | 分配器状态（逐盘按槽位图、`records`、`reclaimed`）的大小，随盘容量与 R_a | 拷 | 条款对「准入之后装单元、拷字节」的豁免若只算落点判之后，这一行在式子判之前 |
| 30 | 每轮：哪些分配记录树节点变了 | `allocation_record_tree.rs:367`（`nodes_whose_contents_changed`）：`allocation_record_tree.rs:372`「let before = records_of_each_leaf(geometry, previous_records);」与下一行 `after` 各把全部记录分到叶；`allocation_record_tree.rs:337`（`nodes_holding_records`）再分一遍 | R_a | 扫 | 每轮 `nodes_whose_contents_changed` 在 13107、13120 两处，`nodes_holding_records` 在 13110、13121 两处 |
| 31 | 记账行表 | `transaction.rs:7980`（`accounting_entries_of_this_publish`）、8017 把台阶行并进来 | A | 扫 | 每轮 25 里调一次 |
| 32 | 式子判的读数 | `transaction.rs:12621` 的 `admit_on_every_device` 入参 `admission_reading_before_a_publish_on_a_ring_of`（`admission.rs:1744`）→ `admission_reading_with_the_switch_reserve_of`（`admission.rs:1711`）又调一次 ckpt_cost（表一 #13–#16） | 同表一 #13、#14 | 扫 | 一次发布里 ckpt_cost 至少算两遍（落点判、式子判各一遍） |
| 33 | 式子判的需求 | `admission.rs:1664`（`demand_of_the_roles_on_each_device`）：对 `settled.resolved.rewritten_roles` 求跨度和 | 这一版重写的角色数 | — | 是 Δ 加固定点那一串 |

表二小结（读）：正文第二节点名的几处全在表二；分配记录树的固定点（#20、#21、#29、#30）、deadlist（#24）、记账树行表（#25、#31，`transaction.rs:7978` 那一行）也在表二，随的总量是节点数、R_a、E_d、E_k、A，不是容器总数。正文没点名而表二里有的：#28 两张全索引、#29 分配器拷贝、#30 全记录分叶。

## A2　规划按差量（候选 A2：规划只读这一次进出的 key 与碰到的路径，准入判照旧读规划的结果）

候选 A2 没说准入判读哪一道，我按「ENOSPC 准入」两道都算：目标是表一、表二里随总数涨的行都去掉。分两档，差别在规划器的输出：
- A2 弱：规划器改成按差量跑，输出的形状仍是今天那种平铺的、按（层级, 层内序号）排好的 `Vec`（`code_two_tree.rs:203`「pub fn nodes(&self) -> &[CodeTwoTreeShapeNode] {」，`code_two_tree.rs:221` `node` 用 `binary_search_by_key` 找）。
- A2 强：输出也换成持久形状（改了的节点带稳定身份，没改的与上一版共享）。

### A2.1　要上一版的什么，怎么拿到而不收全量

多层码 2 树（记账、deadlist、中央映射）。读：
- 全量规划器的开销在 key 一级：`code_two_tree.rs:650`「let previous_keys: BTreeSet<CodeTwoTreeKey> =」收上一版全部 key，`code_two_tree.rs:653`「Some(previous_root) => planning_node_of(previous, previous_root.position),」把整棵树连 key 一起克隆成 `PlanningNode`，`code_two_tree.rs:656`「for deleted in previous_keys.difference(keys_after_this_publish) {」与插入那一趟又做两遍差集；尾部 `flatten` 摊平全树、`replaced_previous_nodes` 过滤上一版全部节点。
- 但每一次删、插只走一条路径：`insert_below`（`code_two_tree.rs:467`）、`delete_below`（`code_two_tree.rs:534`）（路由用 `route_for_insertion`、`route_for_deletion`，在每个内部节点上 `rposition` 找孩子），分裂 `split_in_the_middle` 只动溢出的那个节点与它的右半，空节点摘除只沿这条路径往上，根只剩一个孩子时降高的循环只动根。
- 所以按差量需要上一版的：根；沿每把进出 key 的「根到叶路径上的每个节点（含内部节点的全部孩子分隔 key，路由要用）」；这些节点只需按引用读，上一版的形状本来就按（层级, 层内序号）排好，`node(position)` 二分取（`code_two_tree.rs:221`）。拿法：规划器改成「接收两张已按 key 升序的表（要删的、要插的）」，沿路径按需把节点克隆成 `PlanningNode`（写时复制），不再 `planning_node_of` 整棵展开。

调用方要交出的两张表（今天的调用方是用集合差现算的）：
- 删的 = 上一版里被这次换下的角色的映射 key（`mapped_units` 里那几个角色在上一版的 key）加这次放掉的 deadlist 条目；插的 = 这次重写的角色的新 key（出生序号现算，9368 起 `node_key`）加这次新留存的映射条目。
- 要网掉：同一把 key 这次既出又进的不算（D8 已定项 13 的状态折叠）。快照留存的映射条目从 `mapped_units` 挪到 `kept_mapping_entries`，key 不变，必须从两张表里同时去掉。今天靠 `BTreeSet::difference` 自动网，差量版要调用方自己网（`transaction.rs:9399` 起那段里 `assert_eq!(mapping_keys.len(), ...)` 的互不相撞断言要改成只对插的那张表判）。
- 记账树 **不是差量**：每次发布整批换代，所有行代段都变（`.claude/kb/decisions/08-核心索引结构.md@已定项 1：记账取幂等完整值，整批换代`），差量 = 全部非台阶行；规划只看 key，行数 A = 3 + 6 × 盘数 + S，S 是台阶行（保留、不收缩，表一 #14）。所以记账树按差量规划仍是 O(A)，A 里 S 随水位变过的发布次数涨。要去掉得动 D8 已定项 1 / D5 已定项 2 射程（台阶行不收缩），或用 A3 的闭式。

三棵容器树（墓碑、谱系、退役）。读：追加只改最右端：`append_records_to_the_open_container`（`container_tree.rs:3537`），重写的容器是最右连着的几片、重写的节点是它们的祖先后缀（`rewritten_nodes`，`container_tree.rs:2842`；`ancestor_suffix_starts_of_the_rightmost_containers`）。今天开销在 `container_tree.rs:3575`「let mut containers = copy_of_the_containers(containers_before_this_publish);」与 3576 `shape_before_this_publish.clone()`，以及调用方先把上一版全部容器内容克隆一遍（表一 #9、#10 的 8606、8651 行）。差量版：借用上一版的容器与形状，只产出「新追加的容器、被改的最右那几片、形状的最右路径变化」。形状今天是 `Vec<Vec<usize>>`（每层每节点的孩子数，`container_tree.rs:1505` 的 `copy_of_the_children_counts`），改最右端只动每层最后一格，但「借用」下要产出新形状就得拷一份，A2 弱下拷形状仍随节点数涨；要去掉得把形状变成持久的或消费上一版（落到 A4 丙）。

inode 树与 extent 树（表一 #2、#3、#6、#7）。读：inode 树写者 `inode_tree.rs:1093` 整份拷再改；差量版同理借用、只返回重写的与新分裂的容器。extent 上段：节点集合由「有条目的 inode 号集合 S」定（`extent_tree.rs:180`；I-9.40），差量版 = 只对这次改了条目的 inode 加减它的上段路径，路径上的节点要不要摘取决于别的 inode 还有没有落在它的范围里：用有序集合的区间查询，O(h log F)，前提是 S 由 `BTreeSet` 变成随一版带着的有序结构（今天 `inodes_with_an_extent_upper_leaf_entry`（`transaction.rs:4090`）每次现收）。第一个文件之外的文件表同理，要随一版带着的有序表（今天线性 `find`，`transaction.rs:4165`）。

### A2.2　在冻结副本上具体改哪几步；改完还随总数涨的

| 步（表号） | A2 弱怎么改 | 改完还随什么涨 |
|---|---|---|
| 表一 #2、#3（inode 叶拷贝） | 借用上一版；写者只返回重写 / 新分裂的容器（`inode_tree.rs:1093` 改）；`write_records_into_leaf_containers` 与 `remove_record_from_leaf_containers` 的 ensures 要换 | 不涨（封顶 135 片，今天也封顶） |
| 表一 #5 | `size_change_of_a_file`（`transaction.rs:9507`）把 `inode_record_of` 换成已有的 `inode_record_found_by_its_leaf`（`transaction.rs:4188`），一行 | 对数 |
| 表一 #6、#7、#12 extent 一半 | S 与文件表随一版带着（有序结构）；`plan_the_extent_upper_segment` 改按改动的 inode 加减路径；`extent_tree_roles_rewritten` 只走被改的文件 | 随 F 的部分去掉；`files_other_than_the_first` 表若仍是 `Vec` 则版本交接拷贝仍是 F_n（落到 A4） |
| 表一 #9、#10 | 借用容器；退役树没记录时不调 append（`transaction.rs:8659` 前加判断） | 形状拷贝 O(节点数)，直到形状持久化 |
| 表一 #13 | 记下四个根在 `units` 里的下标（照 `container_tree_root_unit_positions`，`transaction.rs:3299`） | 不涨 |
| 表一 #14 | 去不掉：整批换代下 A 就是改动量；换闭式属于 A3 | O(A)，A 含 S |
| 表一 #17 | 分配器里加一个「聚簇段内没挡的槽对数」增量计数，翻转时更新（`allocator.rs:549`、553 旁） | 不涨；这是分配器改动，不在规划里 |
| 表二 #25 | 规划器改差量（A2.1），记账树仍 O(A) | 形状输出平铺：中央映射树、deadlist 树仍 O(节点数)（节点数是 key 数除以每节点装的条目数，百级，推；仍随总数涨） |
| 表二 #26 | 容器树只列重写的节点；carried 的节点不再逐个产出 key | 前提是 `mapped_units` 本身不再是平铺 `Vec`，否则 #25、#28 的索引照建 |
| 表二 #24、#28 | 把 `mapped_units_by_role`、`units_by_role` 两张索引改成随一版带着（装单元时建，装单元在准入之后，条款豁免）；`entries_after` / `kept_mapping_after` 两个 `BTreeMap` 换差量 | 索引本身的内存 O(M)，建索引的 O(M log M) 挪到准入之后；准入之前不涨 |
| 表二 #27 | `replaced_previous_nodes` 由触碰路径上被替换的节点现收，不再过滤上一版全部节点 | 不涨 |
| 表二 #20、#21、#29、#30（分配记录树的固定点） | **A2 的定义里没有**。要去掉得把演习分配器的 `clone()` 换成可回滚的日志，把 `nodes_whose_contents_changed` 改成只看这一轮释放 / 分配碰到的记录 | 不改就仍随 R_a 与分配器状态大小涨，每轮一次 |

### A2.3　分裂、合并、长高、降低、整理搬迁：差量规划与全量规划重写的节点集合是不是逐个相同

读完 `code_two_tree.rs` 的 `insert_below`（`mark_rewritten` 在路径上每个节点）、`delete_below`、`split_in_the_middle`、`insert_at_the_root`、`delete_at_the_root` 与 `plan_rewriting_the_paths_to_the_moved_entries`，推：
- 分裂：同一批操作按同一个次序回放（先删后插、各自按 key 升序）时，被标成重写的节点 = 各条进出路径上的节点 ∪ 分裂切出的右半 ∪ 新根。这个集合只依赖路径上的节点内容，不依赖没碰到的节点。差量与全量逐个相同（推）。
- 合并：条款没有（D8 已定项 11 ③「收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目」，`.claude/kb/decisions/08-核心索引结构.md:372`）；摘空节点沿路径向上，同样只依赖路径。相同（推）。
- 长高、降低：只动根与根的孩子。降低时幸存的孩子成为根、它的 `carried_from` 保留（`delete_at_the_root` 里 `*root = only_child`），不算重写；差量版要照样保留。相同（推）。
- 整理搬迁：`plan_rewriting_the_paths_to_the_moved_entries` 对每把搬迁 key 把根到叶整条路径标成重写；路径本身只依赖这把 key，相同（推）。差别只在尾部的 `replaced_previous_nodes`（全量：过滤上一版全部节点）要改成现收（A2.2 表二 #27）。
- 三处不是「逐个相同」而是「要小心才相同」：①操作次序，差量版必须由调用方交升序表并先删后插（`code_two_tree.rs:656` 起的次序）；②网掉（A2.1）；③位置编号。`flatten`（`code_two_tree.rs:583`）按深度优先给每层节点依次编号，一次分裂或一次摘空会让它之后的 carried 节点的（层级, 层内序号）整体平移，下游又用位置当角色（`transaction.rs:9305`、9335 的 `role_of_node(node.position, ...)`）、`CodeTwoTreeVersion.pointers` 与 `nodes` 按下标对齐。所以「重写集合」作为一组树节点相同，但作为一组位置不同；今天的下游靠 `CarriedFrom(上一版位置)` 把旧身份找回来。A2 弱的输出仍是平铺编号，平移就得在输出里付 O(节点数)（这是 A2 弱改完还随总数涨的根本原因）；A2 强要把角色身份从位置改成稳定标识。位置是否出现在盘上：我没找到（节点头只带层级，孩子由指针引用），但「点名单元的 journal 记录」用什么标识命名单元我没核，写「待核」。

这些都是读码推的；要不要信，得有一份把全量规划器与差量规划器喂同一批随机删插序列、逐个比重写集合的对照测试（规划器不在 `verus!` 里，可以直接写 Rust 单测）。我没写、没跑。

## A3　准入判挪到差量上界（候选 A3：准入之前只由差量算出这一次分配量的上界，判这个上界；全量规划挪到准入之后）

候选 A3 的「准入判」我拆成两个读法：A3 甲 = 只把落点判挪到上界；A3 乙 = 落点判与式子判都挪到上界。

### A3.1　今天准入判读的是精确量还是上界（读）

| 判 | 读到的量 | 精确还是上界 | 位置与证据 |
|---|---|---|---|
| 落点判，普通半 | 数据单元数，extent / inode / 墓碑 / 退役 / logged_ops 的节点与容器 | 精确：和 `resolve` 里排进 bump 次序的是同一个函数 `ordinary_allocation_part` 的输出（`transaction.rs:12511` 与 8729），但**不含** deadlist 树的节点与实例表几片 | `admission.rs:1025`；`transaction.rs:8102` 的文档「实例表的几片夹在数据单元与 extent 树之间，不在这一半里」；deadlist 的角色在 `admission.rs:1606` 记为普通分配，但 `ordinary_allocation_part` 不产它（deadlist 在 `resolve` 里才算，表二 #24） |
| 落点判，提交内生半 | ckpt_cost | 上界，且不是这次发布的上界：它是「一次空发布至多写多少」的上界（K1 整棵分配记录树 + 中央映射树按空发布的删插 + 记账树 + 谱系 + 退役 + 树表），不含 deadlist 树（`admission.rs:1288`–1292 的五项，外加树表 1） | `admission.rs:1211`；`.claude/kb/checks-owed.md:348`「有普通分配的一次发布比空发布多删插，ckpt_cost 不是这一次的上界」；同一行记着 2026-10-12 判决量到：一次写 4000–5400 单元时真写固定点比 ckpt_cost 多 1–16 槽，n = 5385–5393 九格落得下放行、走固定点 `PlacementRefused`，三种拒都在写之前（C573 开着，去向里程碑十） |
| 落点判，盘上余量 | 每盘段外成对空槽数、没挡的槽数 | 精确（计数器增量维护），聚簇段内那一减项逐段求和（表一 #17） | `allocator.rs:574` |
| 式子判，需求 | `settled.resolved.rewritten_roles` 跨度和 | 精确（固定点走完、全部树规划完之后的最终角色） | `admission.rs:1664`；`.claude/kb/decisions/28-挂载期承诺量.md:38`「- 需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）」 |
| 式子判，读数 | 分配器计数、保留池 = ckpt_cost、已承诺预留、环窗口 | 计数精确；保留池是上界（同上）；已承诺预留是闭式，用发布之后的 H 与 n_live、n_记录，后两者随一版按差量维护（`admission.rs:288`，r2 T4 已改） | `transaction.rs:12621` 前后；`.claude/kb/decisions/28-挂载期承诺量.md:117`「拿得准的上界只有整棵树」是分配记录树那一项为什么取 K1 的原话 |

条款对时序的写法（读）：`.claude/kb/decisions/28-挂载期承诺量.md:35`「- 发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。」落点判（C545）今天已经排在「固定点之前」，是仓里已有的「判在全部规划之前」的先例，但它读的是精确普通半 + 空发布上界，没有哪一道读「按这次的差量算出的上界」。

### A3.2　上界式子在每棵树上各是什么，紧不紧

输入都能在不跑任何全量规划时拿到（读）：这次写的数据单元数 w（`file_content_transactions_and_the_rewritten_data_units`，`transaction.rs:8228`）、这次写的 inode 记录数、上一版各树的高（表一 #13 修好后 O(1)）、上一版各树节点数（`accounting_tree.node_count()`、`central_mapping_tree.shape` 的按层计数 `admission.rs:1437`）、上一版映射条目数（`mapped_units.len() + mapping_entries_kept_for_the_snapshots.len()`）、这次换下的角色数（计数即可，不要列表）。

| 树 | 上界（推） | 紧不紧（推；没量，E199 几何我没取） |
|---|---|---|
| inode 树 | ≤ 写入条数 + 新分裂 1 + 根 1 | 紧：追加写最右叶，今天实际是 1 或 2 片加根 |
| extent 树 | 下段：这个文件的 `lower_segment_nodes_of_a_file_without_holes(w)`（精确，是 w 的函数，`extent_tree.rs` 同名函数）；上段：每个被改的 inode 至多 (上段根层级 + 2) 个节点 | 紧 |
| 墓碑 / 谱系 / 退役 | 每次追加 ≤ 2 个容器 + (H + 1) 个祖先 + 长高 1；仓里已有同构算式 `checkpoint_cost_of_one_lineage_container_cow`（`admission.rs:184`，谱系一项） | 紧 |
| 分配记录树 | K1 整棵树，`admission.rs:1381`（O(盘数 × 根层级)，不读状态，固定点的自指一并罩住：它重写的节点不会超过几何上的位置数） | 松：`.claude/kb/decisions/28-挂载期承诺量.md:117` 自己记着「量到一块盘改 3–4 片叶」，K1 对两块 4 GiB 盘是 529 个节点；每块盘实际 3–4 片叶加祖先，估成 5–10 个节点级，K1 / 实际约 50 倍（推，由这两个数算） |
| 记账树 | ≤ 上一版节点数 + ⌈插入行数 ÷ ⌈叶容量 ÷ 2⌉⌉ + 新层，插入行数 ≤ A + 1；闭式，替掉表一 #14 的规划器运行 | 紧到 ±1 个叶（推；今天那个函数自己记着「第 463 级台阶起 +2」） |
| deadlist 树 | 没有活快照（`previous_snapshot_txg_of_the_live_head == 0`）时 0；有活快照时 ≤ 被钉住的换下角色数 × (h + 1) + 分裂；被钉住的数要查每个换下角色的判定，上界取换下的用户可见角色数 | 无快照时精确；有快照时松（取换下数而不是被钉数） |
| 中央映射树 | 用已有的 `central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`（`admission.rs:1484`），把 `changes.deleted_at_most` / `inserted_at_most` 从「空发布的几项」换成「这次所有树的换下角色数、重写角色数」；O(h) | 小树上松：今天空发布那一项里分配记录树按 K1 计入 529 把 key，`min(这一层节点数, 删 + 插)` 在叶层就等于整层。一棵叶层少于约 (K1 + 记账节点) 个节点的中央映射树，上界 = 整个叶层；这一项今天已经在保留池里，A3 乙把它同时放进需求 |
| 树表 | 1 | 精确 |

合起来的 D̂ = 各行之和；A3 甲只要 D̂ 的提交内生那一半，普通半仍可用精确值（普通半的精确值今天靠表一 #2–#12 全量规划拿，A3 甲不去掉它们，要去掉得连普通半也换成上表的闭式）。

### A3.3　挪到上界之后：会不会放行而真实不够，会不会提前拒

放行而真实不够（推）：
- 上界 ≥ 精确量时不会发生。今天的落点判已经不满足这一条（C573：ckpt_cost 当上界，量到 9 格放行、真写多出 1–16 槽；再加上不含 deadlist 树的节点）。所以 A3 甲若只把 ckpt_cost 当上界、不加这次的删插，等于把 C573 的问题原样搬过来。
- 落点判漏过的后果有一层兜底：`settle_...` 的演习分配在任何写之前（`transaction.rs:13117` 的 `allocate_placement_for_role`），槽不够返回 `PlacementRefused`（会话同样推抬 F 再试，`mounted_session.rs` 的 `refusal_is_short_of_space` 把两种拒都算空间不够）。所以漏过落点判不伤盘，只多跑一遍规划。
- 式子判漏过没有这层兜底：落点只管槽有没有，不管保留池、切换预留、环窗口这些「承诺量」。需求被低估而放行，演习分配仍可能成功，但吃掉的是保留池里留给 checkpoint 自己的固定点的槽（D28 已定项 4 的保留池保证；权威句在 D23 已定项 24，我没重读）。这条是推出来的风险，没有历史把它打出来。只有 A3 乙碰它。
- 补救（候选里没有）：A3 乙下，规划走完之后仍用精确需求再判一次，只当断言（违反时返回 `SpaceAdmissionRefused`，写之前，盘不变）。这样低估变成「多跑一遍规划」，不是穿透保留池。

提前拒（推）：上界大于精确量的差，就是被误拒的区间。
- 落点判今天就有这个差（K1 对 3–4 片叶，A3.2 表），大概每块盘几百个槽（推：529 个节点级对 5–10 个节点级）。
- A3 乙把同样的松量再放进式子判的需求：今天保留池里已经有一份 K1 + 空发布的几项，需求里是精确的几个；A3 乙之后是保留池一份、需求再一份，双倍。误拒区间比今天多一份 (上界 − 精确)。
- 同一批误拒的出口是会话的推抬 F：误拒仍然触发（`mounted_session.rs` 的 `refusal_is_short_of_space`），推满 B = 8 次发布后才报 ENOSPC（`.claude/kb/decisions/16-发布语义.md:44`「准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判」与「一次准入最多 B = 4 + 2 k_tol 次发布」）；这些推出来的空发布各自是一次完整发布（表一、表二全算）。所以 A3 之后「准入之前」的代价在没被拒的路径上是 O(Δ + h)，在被拒的路径上仍是至多 8 次整发布。条款 D8 已定项 16 那一句写的是「准入之前的步」，没分拒与不拒。

与条款是否合：
- D16 已定项 1 的「准入」那一行（`16-发布语义.md:44`）和 C283：C283 的内容是「准入失败时不先推发布就报 ENOSPC」（`.claude/kb/checks-owed.md:662`），会话机制不变，仍满足。
- D28 已定项 1 的需求定义（`28-挂载期承诺量.md:38`「需求 = 这次发布新写的全部槽」）与时序（`28-挂载期承诺量.md:35`「算定这次发布的形状之后」）：A3 乙把需求换成上界、把时点挪到形状之前，与这两句字面冲突，要改条款。A3 甲不碰这两句（它碰的是落点判，条款没写落点判的上界该多大，正是 C573 / C574 开着的那件：`checks-owed.md:348`「准入先拒的上界用真上界，还是条款写明留给落点那一道，交用户」）。
- `df` 一致：`16-发布语义.md:45` 的 `df` 行不含这个松量，而 `16-发布语义.md:71` 记着 E139 的判据「各格按自己的 `df` 假性 ENOSPC 为 0」。A3 乙（和今天已有的 K1 那份松量）会在 `df` 还报有空间时拒写；这条判据在今天的落点判上是否已经被 K1 打破我没核，要主 agent 查。

### A3.4　分配记录树的固定点（自己的分配会改自己的形状）在上界下怎么办

- 判之前：用 K1（几何上界，对自指的回答是：它不看这次发布分配了什么，所以不自指）。仓里已有，条款已定（`28-挂载期承诺量.md:117`）。
- 判之后：`settle_the_allocation_record_tree`（`transaction.rs:13017`）照今天跑，演习、重写集、`nodes_whose_contents_changed`（表二 #20、#21、#29、#30）都在判之后，条款豁免（「准入之后装单元、拷字节不算运行时决策路径」，`08-核心索引结构.md:516`）。判之后再加一句断言：固定点算出的实际重写节点数 ≤ K1。K1 是几何上限，这条断言按构造恒成立，不是靠测试；违反只会是几何实现错了。
- 残留：固定点每轮的 `allocator.clone()`（表二 #29）与全记录分叶（#30）在判之后，所以 A3 之后它们不再算「准入之前」，不是被消掉了。

### A3.5　A3 之后，判之前还剩哪几步（读）

A3 甲：表一 #13（修成按下标取根头）、#15、#16、#17（修成增量计数）、#18、#19，加上 #14 换成闭式（A3.2 记账树一行）；普通半（#2–#12）若仍用精确值就还在。A3 乙：再加一个由上表闭式组成的 D̂，式子判之前表二全部可以挪到判之后（表二 #20–#31 随之出了「准入之前」）。

## A4　只换表示够不够（候选 A4：不改规划算法，改上一版的表示：乙 共享、丙 按值交进来、丁 只记差量）

答案（推，由读码）：单独换表示，准入之前不能不随总数涨。换表示去掉的是表里「拷」那一类的行，「扫」那一类的行算法不动就还在。

乙丙丁三种表示下，问的那两样还在不在：

| 东西 | 位置 | 乙 共享 | 丙 按值交进来 | 丁 只记差量 |
|---|---|---|---|---|
| 规划器里上一版全部 key 的 `BTreeSet` | `code_two_tree.rs:650` 起，由 `previous.keys_in_order()`（`code_two_tree.rs:236`）收成，入参 `previous: &CodeTwoTreeShape` | 在：规划器的入参类型没变，共享只让「拿到上一版」不拷贝，收 `BTreeSet` 的那一步仍逐个 key | 在：消费上一版不改规划器怎么用它 | 在，而且更贵：要先把差量层层合成出上一版的 key 才能 `keys_in_order()` |
| 调用方收的这一版全部映射 key 的 `BTreeSet` | `transaction.rs:9399` 起，源头是 `mapped_units` 整张表 | 在：除非 `mapped_units` 是按 key 排好的持久结构，调用方才可能「上一版 ± 差量」而不重收；但规划器要的是两个集合的差，得换算法（落到 A2） | 在 | 差量本身就是 A2 要交给规划器的两张表；但规划器今天不收，要改（落到 A2） |
| `keys_in_order()` 在 `admission.rs:1358` 的记账树那一遍 | 同上 | 在 | 在 | 在 |
| `node_origins()` 全部节点 | `container_tree.rs:2926`（frozen）`positions_in_bump_order()` | 在：它的规格就是遍历全部节点（`container_tree.rs:2878` 起 ensures） | 在 | 在 |
| `units` 线性 `find` 取根头 | `transaction.rs:4351` | 在 | 在 | 在，更慢 |
| `mapped_units_by_role()` / `units_by_role()` 建索引 | `transaction.rs:6581`、9048、4613 | 在，除非索引本身也共享、随版本带着 | 在 | 在 |
| 表一 #14 记账树规划 | `admission.rs:1358` | 在 | 在 | 在 |
| 分配器 `clone()` 与全记录分叶 | `transaction.rs:13098`、`allocation_record_tree.rs:372` | 在：这是分配器，不是 `TransactionOutput` | 在 | 在 |

换表示确实能去掉的（读）：把上一版拷一份的那几行——表一 #2（`transaction.rs:4208` 逐片克隆）、#3（`inode_tree.rs:1093` 整份拷再改，要持久结构或消费才能不拷）、#6（逐文件造计划时拷 `extent_lower_nodes`）、#9、#10（`transaction.rs:8606`、8651 的克隆与 `container_tree.rs:3575`、3576 的 `copy_of_the_containers` 与 `shape.clone()`）、表二 #23 的谱系克隆、#24 的两个 `BTreeMap`。
- 乙对这些都适用。
- 丙对表二不适用：`settle_the_allocation_record_tree` 每一轮都要用上一版（`transaction.rs:13053` 起的循环里 `plan.resolve(previous, ...)`，最多 64 轮），消费一次就没法进下一轮；落点判被拒后会话还要拿同一个上一版推抬 F 再试。丙只能用在最后装单元那一步（准入之后）。
- 丁对这几行适用（差量天然不拷），代价是读路径合成。

E199 的界限（读）：E199 页只量照抄字节与现算重建字节（`.claude/kb/experiments/199-发布照抄上一版的代价.md@口径`：「准入之前与准入之后两段各报各的」），不判这些「扫」类的行。所以「A4 能不能单独让准入之前不随总数涨」这一轮的答案是：不能；E199 的数只能回答它们各自去掉「拷」的那一半值多少。

## A5　与已定条款的冲突

分项号与条目号我都在 kb 文件里现查了行号。「改条款」= 候选落地要改某条已定条款的字句或补定一句；「只改实现」= 条款不动，改 `crates/`（含 Verus 规格，那是项目里的规格，不是 kb 条款）。

| 候选 | 碰到的已定条款（分项号，行号） | 碰到的不变量（条目号） | 改条款还是只改实现 |
|---|---|---|---|
| A2 弱 / 强 | D8 已定项 16 射程那一句（`08-核心索引结构.md:516`，A2 是在满足它）；D8 已定项 11 ①–⑤ 码 2 树怎么长怎么收（`08-核心索引结构.md:372`，规划器输出逐个相同则不变；A2 强把角色身份从位置改成稳定标识时 ⑤「树高 = 根节点码 2 头里的层级 + 1」不受影响，节点头只带层级）；D8 已定项 13 状态折叠（差量版调用方自己网，`08-核心索引结构.md:414` 起）；D8 已定项 1 整批换代（`08-核心索引结构.md:30` 起，记账树不是差量）；D8 已定项 11「内部条目 = 本树 key + 子指针 86」不动 | I-1.1（块头自述的 key 区间，D8 11 ④ 分隔 key 逐层判）、I-1.10（码 2 条目宽）、I-1.11 / I-1.14（映射 key 与单元头相符；同一映射 key 在各版里的单元逐字节相同：差量版的 key 表必须逐个等于全量版）、I-3.13（记账行代段 = 这一版 txg）、I-9.13、I-9.21、I-9.22、I-9.36、I-9.37（容器只在最右端重写、非最右装满）、I-9.40（extent 上段形状由有条目的 inode 号集合定）、I-9.24（删除队列至多一条 key） | 只改实现，一个例外：记账树要严格不随总数涨得动 D8 已定项 1 的整批换代或 D5 已定项 2 射程的台阶行不收缩。Verus 规格要改（见下） |
| A3 甲（落点判挪到上界） | D28 已定项 4 的 ckpt_cost 名单（`28-挂载期承诺量.md:117` 附近，要补进「有普通分配那次发布多出的删插」与 deadlist 树，正是 C573 的内容）；D3 已定项 5 / C545 落点判（`checks-owed.md:742`）；C573、C574（`checks-owed.md:348`、349，条款没定，交用户） | I-3.20（标签 6 闭式，不动）、I-8.1（环几何够大，W 式子是现成的上界，不动） | 要补定条款（C573 开着：用真上界还是留给落点那一道）。实现部分：改 `transaction.rs:12511`–12534 与 ckpt_cost |
| A3 乙（两道都挪） | D28 已定项 1 的需求定义（`28-挂载期承诺量.md:38`）与时序（`28-挂载期承诺量.md:35`）；D28 已定项 4（同上）；D16 已定项 1 的「准入」行（`16-发布语义.md:44`）与 `df` 行（`16-发布语义.md:45`）及 E139 判据（`16-发布语义.md:71`）；D8 已定项 16 射程（`08-核心索引结构.md:516`，需不需要写「被拒路径除外」） | 同 A3 甲；另外 `<!-- gate:admission-formula -->`（`28-挂载期承诺量.md:18`）是门禁钉住式子字句的标记，改式子读法要过它 | 要改条款：D28 已定项 1 的需求与时序两句；`df` 要不要同步扣这份松量要用户定 |
| A4 乙 / 丙 / 丁 | D8 已定项 16 射程不动；E199（实验页，不是条款） | 无（不变量判的是盘上，A4 只动内存表示） | 只改实现；Verus 规格：容器树的 `containers` 所有权类型变了（乙）会碰 `container_tree.rs` 里 `containers_are_the_same`、`copy_of_the_containers` 的规格（`container_tree.rs:2648` 一带） |

### Verus 规格要怎么改（A2；读；不判难度，没动手）
- `node_origins`（`container_tree.rs:2874`，frozen）：规格的 ensures 就是「遍历全部节点」（`node_origins.len() == bump_order_positions(...).len()` 加逐项）。它决策路径上的唯一调用点是 `transaction.rs:6937`。差量版不改它的规格，而是停止在决策路径上调它：重写的节点已有 `rewritten_nodes()`（`container_tree.rs:2842`，ensures 是 `ancestors_below_level(...)`，O(路径)），carried 的节点 key 不再逐个产出，所以 `requires` 里的 `every_node_absent_from_the_previous_version_is_rewritten`（`container_tree.rs:463`）要作为引理单独导出一份给「carried ⇒ 上一版计数里有这个位置」，现在它夹在 `node_origins` 的循环不变式里（`container_tree.rs:2938` 一带）。
- `append_records_to_the_open_container` 的 ensures 引 `appending_keeps_the_key_to_entry_mapping(containers_before@, levels_before, records@, ..., after)`（`container_tree.rs:3104`），其中 `after.containers` 是全序列；要不拷，返回类型改成「重写的几片 + 新开的几片」，规格用 ghost 序列 `before ++ delta` 描述 `after`。工作量没估。
- `write_records_into_leaf_containers`（`inode_tree.rs:1026` 起）同理：ensures 里 `writing_progress(before@, records@, ..., after.containers@, after.rewritten@)`（`inode_tree.rs:979`）。
- `code_two_tree.rs` 不在 `verus!` 里（正文第二节已写；我也没找到 `verus!` 命中，`grep -n 'verus!' code_two_tree.rs` 零命中），规划器改差量不碰 Verus；正确性靠对照测试，不靠证明。
- 重型：改了 `verus!` 内的函数就要过形式化验证档（`.claude/rules/verification.md` 的 checker 档，重型）。

## 各候选一行代价量级（推；没量，符号见开头）

| 候选 | 落点判之前 | 落点判到式子判之间（准入之前，字面 ENOSPC 准入） |
|---|---|---|
| A1 今天 | R_i（封顶 31455 条）+ (C_t + C_r) × 容器大小 + F_n × 下段节点 + F × 根层级 + U² + A + 盘数 × S_c + 至多 4P | 再加 (1 + 轮数) × [M log M + P log P + A + E_d + E_k + 谱系 / 墓碑 / 退役节点 + 分配记录树节点] + 每轮 O(R_a) 与一份分配器克隆 |
| A2 弱 | Δ × h + 形状拷贝（节点数）+ A | Δ × h × log N + 平铺输出 O(节点数) + A；分配记录树固定点不改就仍是每轮 O(R_a) 加分配器克隆 |
| A2 强 | Δ × h + A | Δ × h + A；固定点同上 |
| A3 甲 | 提交内生半 O(Δ + h + 盘数 × 根层级)，普通半仍是今天的精确规划（不换闭式就还是 A1 那一列） | 不变 |
| A3 乙 | O(Δ + h + 盘数 × 根层级)，表一 #13、#17 先修 | 全量规划在判之后；被拒路径至多 8 次整发布 |
| A4 乙 / 丙 / 丁 | 去掉「拷」类行后剩 #5、#7、#13、#14、#17 的「扫」 | 去掉「拷」类行后剩 #25（规划器全 key）、#26（全节点）、#28（索引）、#29–#31 |

## 新候选（各条腿可以提）

- N1　只修落点判的前置，重述射程。把表一 #2–#14、#17 逐项改成不随总数涨（借用、按下标取根、增量计数、记账树闭式），落点判读的量不变（精确普通半 + ckpt_cost），表二不动；把 D8 已定项 16 射程里的「准入之前」限定为落点判之前，同步改 `.claude/rules/fs-design.md:23` 那一格的读法。与 A2 的不同：A2 两道准入之前都要去掉；与 A3 甲的不同：甲连判读的量也换，N1 不换。风险：fs-design 那一格写的是「ENOSPC 准入」，字面上式子判才是 ENOSPC 的最终判，N1 靠用户改读法。
- N2　A3 乙 + 精确复核做断言。上界判在前、全量规划在后，规划走完后用精确需求再判一次，不够就在写之前返回 `SpaceAdmissionRefused`（盘不变）。与 A3 乙的不同：A3 乙去掉了精确判，N2 留着，低估变成多跑一遍规划，不穿透保留池（A3.3）。条款仍要改 D28 已定项 1 的需求与时序两句；精确复核排在全量规划之后，「准入之前」的口径要用户写明指哪一道。

## 核过的命令与原样输出

在冻结副本里起（只读）：

```
$ grep -n 'plan_the_multi_level_trees\|\.resolve(\|fn resolve\|fn settle_the_allocation_record_tree\|settle_the_allocation_record_tree(\|admit_the_units_landing_on_every_device(\|admit_on_every_device(' crates/singlefs-core/src/transaction.rs  (过滤掉 /// 注释行)
1876:fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
8718:    pub fn resolve(
8828:        } = self.plan_the_multi_level_trees(
8976:    fn plan_the_multi_level_trees(
12523:        admit_the_units_landing_on_every_device(
12536:    let settled = settle_the_allocation_record_tree(
12621:        admit_on_every_device(
13017:fn settle_the_allocation_record_tree(
13063:        let resolved = plan.resolve(
$ grep -c verus! crates/singlefs-core/src/code_two_tree.rs
0
$ cmp 冻结副本与主工作区（同名文件）
admission.rs 逐字节相同
code_two_tree.rs 逐字节相同
inode_tree.rs 逐字节相同
allocation_record_tree.rs 逐字节相同
allocator.rs 逐字节相同
tombstone_tree.rs 逐字节相同
retirement_tree.rs 逐字节相同
extent_tree.rs 逐字节相同
container_tree.rs 不同
transaction.rs 不同
mounted_session.rs 不同
```

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1　正文第二节对「准入之前有哪几步」的写法 | 冲突 | 多层码 2 树规划、`mapped_units_of_a_container_tree`、`node_origins` 在落点判之后（12523 < 12536 < 13063），只在式子判（12621）之前 |
| A1　落点判之前的清单 | 一致（正文点名的 `inode_leaf_container_contents` 在内），另有正文没点名的 #5、#6、#7、#9、#10、#13、#14、#17 | 随 R_i（封顶 135 片）、F、C_t、C_r、P、A、S_c 涨 |
| A1　式子判之前的清单 | 一致（正文点名的几处都在），另有 #28 两张全索引、#29 分配器克隆、#30 全记录分叶 | 每轮重跑，至多 64 轮 |
| A2　差量规划的重写节点集合 | 规则没说（条款没规定规划器怎么做）；推得出逐个相同 | 路径局部；位置编号会平移，输出平铺就得 O(节点数)；记账树不是差量 |
| A2　`node_origins` 规格 | 规则没说 | 规格本身遍历全部节点，不改它，停止在决策路径上调；carried 引理要单独导出 |
| A3　今天读精确还是上界 | 一致（落点判普通半精确、提交内生半上界；式子判需求精确、保留池上界） | 落点判的上界不是这次发布的上界（C573 开着），也漏 deadlist 树 |
| A3 甲　放行而真实不够 / 提前拒 | 规则没说（C573 / C574 开着） | 漏过落点判有演习分配兜底；提前拒的差是 K1 对实际约 50 倍（推） |
| A3 乙　同上 | 冲突（D28 已定项 1 需求与时序两句） | 漏过式子判没有兜底，吃保留池；要加精确复核 |
| A4　只换表示 | 不够 | 去掉「拷」行，规划器全 key 集、全节点遍历、线性取根头、建索引、固定点全记录都还在 |
| A5　已定条款 | A2 只改实现；A3 甲补定条款；A3 乙改条款；A4 只改实现 | 分项号与条目号见表 |

## 每条结论什么现象会推翻它

- 「多层码 2 树规划在落点判之后」：任何一条走到 `plan_the_multi_level_trees` 或 `mapped_units_of_a_container_tree` 而不经过 `resolve`（13063 唯一调用点）的路径，或 `resolve` 另有一个排在 12523 之前的调用点。上面的 `grep` 是这一条的判据，零命中以外的第二个调用点就推翻。
- 「表一 #13 随 P」：`units` 里这四个根的位置若在向量最前，线性 `find` 很快停；量一次根在 `units` 里的下标即可推翻（我没量，写的是推）。
- 「A2 重写集合逐个相同」：对同一批随机删插序列，差量版与全量版的重写集合（按 `CarriedFrom` 对回上一版位置之后）出现任何不同。
- 「A4 单独不够」：有人在不改规划器、`node_origins`、取根头与建索引的前提下，只换表示就让表一表二里「扫」类的行不随总数涨。
- 「A3 乙漏过式子判会吃保留池」：把需求压低的历史上，演习分配成功且保留池读数低于 ckpt_cost 而没有任何一道拒；没有历史能打出这条，推翻靠造一条。
- 「inode 树这一步今天封顶」：`inode_tree.rs:1065` 的 135 若被放宽或不再作为写者拒绝条件。

## 要主 agent 跑的命令

没有。本报告没写模型、脚本或装置，没有往服务交任务，也没有要交服务的命令。下面几件需要先有人写代码才跑得了，命令写不出，属「复核不了」：
- 表一、表二各步在大盘上的用时与内存：需要给每一步加计时或计数的装置（仓里 `lookup_work` 有按调用点计查找次数的机制，`transaction.rs:6552` 一带 `lookup_work::entering`，我没读它能不能挪来量用时）。
- 全量规划器与差量规划器的对照测试（A2.3 末句）。
- 落点判之前每一步随总数涨还是不涨的钉代价测试（r2 判决要求的那种「形状做成从最左读起就溢出」的法子），要先改实现。

## 没做什么

- 不判别的腿的格；不替主 agent 选候选，不下「选哪条」的结论；没往统一任务服务交任务，没有判决要引的数需要主 agent 复跑。
- 没读禁读清单里的文件（`research/prompts/m4-admission-delta-r1-opus-output.md` 与 `m4-admission-delta-r1-opus-model/`）。
- 没量任何用时与内存；表里带「推」的数都没跑。
- 没逐行读：`remove_record_from_leaf_containers` 内部（`inode_tree.rs:1692` 起）、`truncate.rs` 的 `plan_the_extent_tree_after_a_size_change`、`unlink.rs` 的两个删除批规划里除 `inodes_with_an_extent_upper_leaf_entry` 之外的部分、`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots` 与 `mapped_roles_released_by_this_publish` 的内部（只读到它们建索引的入口）。
- 没读 `transaction.rs:1876` 起的另一条 `settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`（版本无文件时写行那条路径，`transaction.rs:2072` 调）；它不经 `judge_the_publish_before_any_read_or_write`，本报告的两张表不含它。
- 没核「点名单元的 journal 记录」是否用位置当标识命名单元（A2.3 ③，写了「待核」）。
- 没重读 D23 已定项 24 保留池的权威句（A3.3 引的是 D28 已定项 4 射程里的转述）。
- 没核 `df` 行（`16-发布语义.md:45`）在今天的落点判上是否已经被 K1 的松量打破（A3.3，要主 agent 查）。
- 没取 E199 的几何与取样点，所以「在 E199 的几何上大约多估多少」没有答（A3.2 写的是通用式子与 K1 对实际约 50 倍）。
