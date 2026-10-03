# 里程碑三 三·1（删除设计轮）实现侧事实表

日期：2026-09-30。仓的 HEAD：`fcc4d340`（工作区只多一份未跟踪的 records 文件，crates/ 与 kb 未改）。只读，没改仓内文件，没跑 cargo。
行号都是这一次 `grep -n` / `awk NR` 现取的；「推的」是我从代码读出来的推断，没跑过。

## 最要紧的几条（先看这里）

1. **实现今天只有「一个带数据的文件」**：数据只属于 inode 1（`crates/singlefs-core/src/transaction.rs:109` `FIRST_INODE_NUMBER = 1`）；恢复重建一版时要求 extent 树上段恰好一条叶条目且是 inode 1（`crates/singlefs-core/src/recovery.rs:3351-3360`，`extents_of_the_first_file`），要求 inode 树里有 inode 1 那条记录（`recovery.rs:2499-2506`，`rebuild_version_in_the_unit_area_starting_at`），否则报 `UnitMalformed`。内存里的一版 `TransactionOutput` 也只有一个 `inode_record`（`transaction.rs:2501`）与一串 `data_pointers`（`transaction.rs:2482`）。「删掉 inode 1 之后读回没有文件」今天在恢复这一侧会被当成坏镜像拒（推的，按上面两处的判法）；「没有文件」今天只有树表 0 条那一种（`recovery.rs:2269-2273` `RebuiltVersion::WithoutFile`）。
2. **mkfs 一棵树都不建**：mkfs 写的树表第 0 版是 0 条（`crates/singlefs-core/src/make_filesystem.rs:457-469`，条目传 `&[]`），分配记录树根也是空指针（`make_filesystem.rs:551-553`）。七条树表条目（含三棵根指针为零的 livelist / 稀疏旁表 / deadlist）是**第一个文件版本那次发布**写的（`transaction.rs:6482-6524`）。所以「mkfs 建不建空树」在今天的代码里实际问的是「第一个文件版本建不建」。
3. **inode 记录里 nlink 恒写 1、读的时候不读**：`crates/singlefs-core/src/records.rs:56` `writer.put_u32(1); // nlink`；`InodeRecord` 结构（`records.rs:38-44`）没有 nlink 字段，`parse` 在 `records.rs:85` 把 locality / mode / uid / gid / nlink 一起跳过。写 nlink = 0 要先给结构加字段（推的）。
4. **记账里「待删占用」那一行已经在写、恒 0**：统计量标签 4（`records.rs:18`），每次发布作为池级三行之一写出（`transaction.rs:6283-6285` 取 `admission::PENDING_DELETE_OF_THE_FIRST_VERSION`，`crates/singlefs-core/src/admission.rs:151-152`），准入式子也已经扣它（`admission.rs:361` 起 `available_on_each_device`）。checker 不读这一行（下面 (e) 的 grep 空）。
5. **崩溃点名字的实际编码是 `<入口>.<这一段写的种类>.<这个入口里这一种的第几段>`**（`crates/singlefs-checker-tier/src/crash_amplification.rs:2381-2389`），「种类」按写的类型与单元类标签取（`crash_amplification.rs:2234-2259`）。里程碑表里「`unlink.<段>.<步>`」对应的是：给新入口起名 `unlink`，段名由写种类自动生成（例如 `unlink.index_node+packed_records.1`、`unlink.journal_record.2`），不需要新编码（推的：前提是删除的写只落码 1/2/3 单元、journal 记录、根槽 FUA、系统配置槽这几种）。

## (a) 树

### a1 今天盘上有哪几棵树

| 树 ID / 种类码 | 名字 | key（宽与字段） | 条目 / value | 形状 | 定义处 |
|---|---|---|---|---|---|
| 11 / 1 | extent 树 | 24 = locality 8 + inode 8 + offset 8（`crates/singlefs-format/src/lib.rs:98`） | 下段叶记录 112 = key 24 + 数据指针 88（`lib.rs:101`）；上段叶条目 113 = key 24 + 标签 1 + 载荷 88（`lib.rs:160`）；内部条目 110 = key 24 + 子指针 86（`lib.rs:150`）；节点头 163（`lib.rs:65`） | 按位置寻址两段、不分裂；上段叶罩 143 个 inode 号（`lib.rs:163`），下段叶罩 144 个数据单元（`lib.rs:156`）；上段叶条目标签 0/1/2（`crates/singlefs-core/src/extent_tree.rs:40-44`） | `extent_tree.rs:1-17`（结构说明）；`records.rs:119-129` `build_extent_record` |
| 12 / 2 | inode 树 | 8 = inode 号（`recovery.rs:3235`） | 根与内部节点是码 2，条目 120 = 分隔 key 8 + 身份引用 26 + 子指针 86（`lib.rs:107`，`records.rs:101-117`）；叶是码 3 打包记录类型 2，记录 140（`lib.rs:104`），一叶 233 条（`lib.rs:110`）；根头 131（`lib.rs:59`） | 今天只有「根 + 一层叶」：叶数超过一个根装得下的 135 片就拒（`crates/singlefs-core/src/inode_tree.rs:108-111`、`:142`）；只在最右叶追加、末尾分裂，没有合并（`inode_tree.rs:16-17`） | `inode_tree.rs`；记录 `records.rs:38-99` |
| 13 / 3 | 分配记录树 | 10 = 设备 4 + 槽号 6（`lib.rs:122`） | value 10 = 跨度 2（最高位是已释放标志）+ 分配代或释放代 8（`lib.rs:123`，`crates/singlefs-core/src/allocator.rs:26-40`）；条目 20；内部条目 96（`lib.rs:143`）；头 135（`lib.rs:68`） | 按绝对槽号按位置寻址，叶罩 812 槽（`lib.rs:140`）；根指针另住根记录（`lib.rs:187-188` 字段表） | `crates/singlefs-core/src/allocation_record_tree.rs` |
| 14 / 4 | 记账树 | 22 = 标签 2 + 树 ID 8 + 设备 4 + 代 8（`lib.rs:128`） | 条目 34 = key 22 + value 8 + seq 4（`lib.rs:130`）；内部条目 108（`lib.rs:133`）；头 159（`lib.rs:71`） | 多层码 2（`transaction.rs:2053` `MultiLevelCodeTwoTree`）；一次发布写 15 行（`lib.rs:166`） | `records.rs:131` 起 `AccountingEntry`；行表 `transaction.rs:4031-4135` |
| 15 / 5 | 中央映射树 | 27（`lib.rs:92`） | 条目 55 = key 27 + 位置条目 14 × 2（`lib.rs:95`）；内部条目 113（`lib.rs:136`）；头 169（`lib.rs:74`） | 多层码 2；**根住根记录、不进树表**（`transaction.rs:6472`） | `records.rs` 映射条目；`transaction.rs:2935` 查找 |
| 16 / 6 | livelist 共享树 | 代码里没有 | 代码里没有 | 树表条目根指针恒零（`transaction.rs:6508-6512`） | 只有常量：`lib.rs:180`、`records.rs:265` |
| 17 / 7 | 稀疏旁表树 | 代码里没有 | 代码里没有 | 同上（`transaction.rs:6514-6518`） | `lib.rs:181`、`records.rs:266` |
| 18 / 8 | deadlist 树 | 代码里没有 | 代码里没有 | 同上（`transaction.rs:6520-6524`） | `lib.rs:182`、`records.rs:267` |

另有两个不算树、但同是码 2 / 码 3 的结构：树表单元（码 2，key = 树 ID 8，条目 200，`lib.rs:169`，`records.rs:272-340`）与实例表单元（码 3 打包记录类型 4，记录 88，`lib.rs:113`）。树 ID 从 11 起、mkfs 水位 11、第一次发布之后 19（`lib.rs:174-185`）。种类码只追加不重用、0 保留（`.claude/kb/decisions/08-核心索引结构.md@已定项 9：树的种类码的登记表`）；**代码里没有 `deleted_inodes`、`logged_ops` 两个名字**：`grep -rn 'logged_ops\|deleted_inodes' crates/ --include=*.rs` 只命中两处注释（`crates/singlefs-checker/src/walk.rs:6133`、`inode_tree.rs:16`）。

### a2 mkfs 与第一个文件版本各建哪几棵

| 时刻 | 写什么 | 依据 |
|---|---|---|
| mkfs | 实例表第 0 片 + 树表第 0 版（**0 条**）；分配记录树根写空指针；水位 11 | `make_filesystem.rs:457-469`（`build_index_node(..., &[])`）；`:551-553`；`:535` |
| 第一个文件版本（`publish_first_file`，`transaction.rs:4624`） | 从水位连号发八个树 ID（`FileVersionTreeIdentifiers::issued_from_watermark`，`transaction.rs:2390`，偏移表 `:2374-2383`），写树表 7 条：extent、inode、分配记录、记账四棵有根，livelist、稀疏旁表、deadlist 三棵根指针为零；水位写 19 | `transaction.rs:6472-6530`；断言升序 `:6525-6530` |

### a3 加一棵新树要碰的地方（今天的代码与 kb；按「加一棵进树表的新树」数）

| 位置 | 今天的样子 | 要动什么（推的） |
|---|---|---|
| `crates/singlefs-format/src/lib.rs:172`、`:175-185` | 树表条目数 7、八个 `TREE_IDENTIFIER_*`、水位 11 → 19 | 加一个树 ID 常量；水位常量与条目数常量跟着变；若新树有码 2 节点，另加节点头宽常量（照 `lib.rs:59-74` 那一组）。带 `format-const:` 的常量由门禁 27 号与 kb 标记绑值（`lib.rs:1-6`） |
| `crates/singlefs-core/src/records.rs:261-267` | 七个 `TREE_KIND_*` | 加种类码（登记表 9 起） |
| `transaction.rs:2354-2363`、`:2374-2383`、`:2390-2440` | `FileVersionTreeIdentifiers` 八个字段、偏移表长 8 | 加字段、偏移表加一格 |
| `transaction.rs:6482-6524` | 树表条目 `vec!` 七条 | 加一条（根指针、头 ID） |
| `transaction.rs:2006-2052`（`TransactionUnit`）及它的穷举处 | 15 个成员；`grep -rn 'TransactionUnit::AccountingTree\b' crates/` 在 `transaction.rs` 里 17 处，另有 `admission.rs` 3、`write_accounting.rs` 2、`mount.rs` 1、`model_comparison.rs` 2 与十几个 harness 测试 | 新树若有节点要落盘，加角色成员；释放路径的 `match`（`transaction.rs:2765-2803`、`:2876-2912`、`:2958-2977`）逐个加臂 |
| `transaction.rs:2466-2523`（`TransactionOutput`） | 每棵有节点的树一个版本字段 | 加字段 |
| `recovery.rs:2187-2195` | `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7]`（I-9.16 发号次序） | 加一格 |
| `recovery.rs:2350-2374` | 重建时按种类取七棵的根与树 ID | 加一棵 |
| `recovery.rs:2823-2829`、`:3232-3240` | 按种类分派、`key_width_for_kind`（livelist / 稀疏旁表 / deadlist 给 `None`） | 加臂 |
| `mount.rs:4969-4980` | 管理员回退新根的树表：只有 extent、inode 两棵取目标那一版的根，别的照现行那一版 | C118 行的要求：`deleted_inodes` 也取 R_old 那一版（`.claude/kb/checks-owed.md` C118 行，`.claude/kb/decisions/08-核心索引结构.md@已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用`「key 与 inode 号」段），这里加种类 |
| checker `walk.rs:43-59` | 自己抄一份七个种类码与发号次序表 | 加一格（checker 与 core 各写一份，D13 已定项 5） |
| checker `walk.rs:160-167` | `field_table_width_of_tree_kind` 只罩 1–4 | 加新树条目宽（I-1.10） |
| checker `walk.rs:1292` 起（`walk_tree_table_entry`） | 按种类走 inode / 记账 / 分配记录 / extent；`key_schema_for_tree_kind` 不认的种类直接 `return`（`walk.rs:1305-1307`） | 加一支走新树 |
| checker `walk.rs:2690-2711`（`TreeKindForReferenceScan`） | 不认的种类落 `WithoutWalkableEntryFormat`：注释说「真有了根节点，那时这条根的引用集合数不全」 | 加成员，否则 I-3.1 / I-7.4 的引用集合缺这棵树的节点（推的） |
| checker `crates/singlefs-checker/src/lib.rs:586` 起 | `key_schema_for_tree_kind` 罩 1–5 | 加 key 形态 |
| checker `crates/singlefs-checker/src/image.rs:75` 起 | `IMPLEMENTED_INVARIANTS` 清单 | 三·3 的新不变量进清单 |
| `.claude/kb/freeze-layer-membership.md` | 每个 `TREE_IDENTIFIER_*` 在第 3 层（key 编码）与第 4 层（节点内部布局）各一行；门禁 `.claude/gate.d/doc-decisions.sh:1629` 从 format 现读树清单，`:1717`、`:1719` 漏行判红 | 新树两行 |
| `.claude/kb/layout/01-first-txn.md:49`、`:78`、`:367`、`:379`、`:383` | 「树表第 1 版 7 条」「水位 19」写在新池新建文件的字节表里 | 若新树在第一个文件版本就注册（照 deadlist 的先例），这几行与 E142 干跑都要改；若惰性建，改的是 `layout/03-third-txn.md`（三·6 建档） |
| 断言 7 条的测试与代码 | `transaction.rs:2670`（`expect("树表第 1 版起恒有七条")`）、`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:321`、`tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs:37`、`checker_known_bad_images.rs:2872` | 跟着改 |

## (b) 释放：覆盖写释放旧 extent 走的路

硬规则 1 原文（`.claude/kb/decisions/19-块指针的结构与宽度预算.md@已定项 5：位置条目降为提示，中央映射是唯一入口`）：「1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。」

| 次序 | 函数（文件:行） | 做什么 |
|---|---|---|
| 1 | `prepare_the_version_publish`（`transaction.rs:5057`）→ `settle_the_allocation_record_tree`（`:5249`） | 分配记录树重写集走固定点（上限 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`，`:5329`）；每一轮在分配器的**拷贝**上先释放后取落点（`:5290-5293`） |
| 2 | `placements_released_by_this_publish`（`:5188`） | 换下的上一版角色 = `resolved.previous_roles_replaced_by_this_publish`；有上一版走 `placements_to_release_via_mapping`，没有（第一个文件版本）走 `format_time_tree_table_to_release`（`:2700`，释放 mkfs 那片树表）；实例表旧链排最前（`instance_table_chain_to_release`） |
| 3 | `roles_replaced_via_mapping`（`:2845`） | 这次重写的角色，加上「文件变短」时上一版多出来的数据单元（`:2851-2870`）。**释放的对象按角色（数据单元序号、extent 节点位置）点名，不按 extent key 点名** |
| 4 | `placements_to_release_via_mapping`（`:2758`） | 进映射的角色（数据、extent 上下段节点与根、inode 叶与根、分配记录树、记账树）按上一版的映射 key 在上一版映射叶里查位置项（`mapping_locations_of_a_mapped_unit` → `central_mapping_locations_in_the_version`，`:2935`）；映射树、树表按根记录的指针；查不到报 `ReleaseNotInMapping` |
| 5 | `placement_to_release_after_checking_every_device`（`:3186`） | 两条位置条目同槽、池里每块盘都有一条在册、未释放、跨度对得上的分配记录，三样有一样不对就报错、一个都不释放 |
| 6 | `copies_failing_the_release_checksum_check`（`:3070`，调用在 `:5146`） | 硬规则 1 的读盘核：逐份读、对不上或读不出重读一次，两次都没对上交回两份（`:3093` 起） |
| 7 | `publish_admitted`（`:6048`）里 `:6089-6100` | 真释放：对每个落点 `allocator.release_leaving_the_record_allocated_on(placement, txg, 读盘核失败的盘)`；释放代 = 这次发布的 txg |
| 8 | `PoolAllocator::release_leaving_the_record_allocated_on`（`allocator.rs:1285`） | 每盘那条分配记录改写成「已释放 + 释放代」，**条目不删**；`DeviceFreeMap::mark_released`（`allocator.rs:549`）只加 `deferred_slots`，槽仍占着、`allocated_slots` 不减、空闲不加 |
| 9 | `publish_admitted` `:6115` → `record_root_written_by_this_process`（`allocator.rs:1637`） | 这次的根盖掉根环里一个槽之后，按 `reclaim_floor = max(F_生效, 环里最旧有效根)`（`mount.rs:1154-1159`）回收：`reclaim_released_records_up_to` 挑「已释放 ∧ 释放代 ≤ floor」的记录，`mark_reclaimed`（`allocator.rs:657`）清位图、`allocated_slots` 与 `deferred_slots` 各减、`free_slots` 加 |
| 10 | 挂载时 `rebuilt_allocator`（`mount.rs:1489`） | 从盘上分配记录重建（`PoolAllocator::rebuild_from_records`，`allocator.rs:1158`：已释放的进 defer），再按 floor 回收（`mount.rs:1544`） |

defer 窗口：一个落点从释放到可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（`allocator.rs:827-831` 的 `ReuseWindow::GatedByTheRollbackFloor` 注释；D16 已定项 1）。只供测试的开关 `ReuseWindow::ForcedToZero`（`allocator.rs:826-844`）让释放那一刻就回收。管理员回退还有一条「复活」路：`resurrect_released_record`（`allocator.rs:1333`），释放用同一段读盘核（`mount.rs:5227`）。

对删除的含义（推的）：今天的释放入口按「这次重写的角色 + 上一版多出来的数据单元」算换下集，没有「按 extent key 区间删一段」的入口；分批删 extent 要么每批发一次「文件变短」式的发布（每批把文件截短一段、复用 `roles_replaced_via_mapping` 的变短那一支），要么另写一个按区间点名的换下集。一批内「施加时先核要删的东西还在」（D8 已定项 5 第 4 条）在今天的路径里对应第 5 步那三样核（在册、未释放、跨度对）——重复释放同一落点会在这里报 `ReleaseTargetAlreadyReleased`（`transaction.rs:3219-3220`）而不是静默减两次。

## (c) inode 记录

### c1 字段表：盘上 140 字节（D8 已定项 6）与实现结构对照

| 偏移 | 字段（kb） | 写的时候（`records.rs:48-73` `to_bytes`） | 读的时候（`records.rs:76-98` `parse`） | `InodeRecord` 结构有没有（`records.rs:38-44`） |
|---|---|---|---|---|
| 0 | inode 8 | `self.inode` | 读 | 有 `inode` |
| 8 | 对象出生代 8 | `self.object_birth` | 读 | 有 `object_birth` |
| 16 | locality_id 8 | 写死 0（`:52`） | 跳过（`:85`） | 没有 |
| 24 | mode / uid / gid / nlink 各 4 | mode 写死 `0o100_644`、uid 0、gid 0、**nlink 写死 1**（`:53-56`） | 跳过（`:85` `reader.skip(8 + 16)`） | 没有 nlink、mode |
| 40 | size / blocks / rdev 各 8 | `size`；blocks = ⌈size ÷ 512⌉ 现算（`:59`）；rdev 0 | 读 size、跳过 blocks 与 rdev（`:86-87`） | 有 `size`；blocks 不是字段 |
| 64 | atime / mtime / ctime 秒各 8 | 三个都写 `write_time_seconds`（`:62-64`） | 读第一个 | 有 `write_time_seconds` |
| 88 | 改动计数 8 | `change_count` | 读 | 有 `change_count` |
| 96–139 | 纳秒 12、填充 4、flags 8、预留 20 | 全 0 | 偏移 108 起非 0 判不可用（`:77-79`，I-9.7） | 没有 |

checker 另写一份偏移：`walk.rs:120` `INODE_RECORD_SIZE_OFFSET = 40`。`grep -n 'nlink' crates/singlefs-checker/src/*.rs` 输出为空：checker 今天不读 nlink（I-9.5 的「nlink = 0」一半没有落点）。

### c2 inode 号的分配与水位

| 事实 | 依据 |
|---|---|
| 第一个文件恒 inode 1 | `transaction.rs:109`；`inode_record_of_file_version` 写死 `inode: FIRST_INODE_NUMBER`（`:4593-4601`） |
| 下一个号 = 记账里「inode 号水位」那一行（统计量 12，`records.rs:23`），记在 inode 树的树 ID 下 | `TransactionOutput::inode_number_watermark`（`transaction.rs:2583-2592`）；行定义 `PoolWideAccountingRow::tree`（`transaction.rs:4055`） |
| 每次发布重写水位 = 上一版水位 + 这次新建数；第一个文件版本写 2 | `transaction.rs:6266-6270` |
| 建 N 个 inode：号从水位起连号，size 0、出生代 = 这次 txg，不写数据、不动 extent 树 | `publish_new_inodes`（`transaction.rs:4830`，`:4839-4853`） |
| 只许追加到最右叶，中间插入拒 | `InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf`（`inode_tree.rs:132`） |
| 管理员回退那次发布的水位取 max(现行那一版, 环里读得出的根的) | `mount.rs:5197-5205`（`highest_inode_number_watermark_in_the_ring`，`:4902`） |
| checker 判 I-9.6：水位 > inode 树内最大 key；「> 墓碑对象 ID」那一半今天没有对象 | `walk.rs:6130-6140`，注释 `:6132-6133` |

删除之后号不重发（C479）靠的就是水位只加不减这一条：删掉 inode 1 不会让下一次 `publish_new_inodes` 再发 1（推的，按 `:6266-6270` 只读上一版水位、不看树里最大号）。

## (d) 可写挂载的写序列

入口：`mount_writable`（`crates/singlefs-core/src/mount.rs:4165`）→ `mount_writable_with_space_admission`（`:4177`）→ `mount_writable_with_test_only_switches`（`:4191`）→ `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`（`:4430`）→ `establish_instance`（`:3688`）。

| # | 步 | 函数（文件:行） | 写不写盘 |
|---|---|---|---|
| 1 | 盘数、盘表去重 | `admit_the_writable_device_count`（`mount.rs:4439`）、`refuse_device_identities_handed_in_more_than_once` | 不写 |
| 2 | 择系统配置 | `choose_system_configuration`（`mount.rs:4441`） | 不写 |
| 3 | 每块盘到得了单元区起点 | `every_device_reaches_the_unit_area_start`（`mount.rs:4454`） | 不写 |
| 4 | 读阶段：择根、扫 journal、重放；系统配置见证过更新的发布就重读一次 | `read_stage_settled_at_most_on_the_one_reread`（`mount.rs:4465`，本体 `:4391`）→ `read_stage`（`:4353`）：`choose_root`（`:4359`）、`scan_journal`（`:4360`）、`replay_journal`（`:4361`） | 不写 |
| 5 | 重建上一版（树表、inode 树、extent 树、分配记录树、记账树、映射树） | `rebuild_previous_version`（`mount.rs:4487`，本体 `:1033`）→ `recovery::rebuild_version_in_the_unit_area_starting_at`（`recovery.rs:2309`） | 不写 |
| 6 | 新实例的第一个 txg、环里树 ID 水位 | `first_txg_of_new_instance`（`mount.rs:4495`）、`tree_identifier_watermark_of_the_ring`（`:4496`） | 不写 |
| 7 | 重建分配器、按 floor 回收、影子账隔离 | `rebuilt_allocator`（`mount.rs:4502`，本体 `:1489`；回收 `:1544`；隔离 `:1549`） | 不写 |
| 8 | 进 `establish_instance` | `mount.rs:4521` → `:3688` | — |
| 9 | 算暖机要推哪几次 | `warm_up_publish_txgs`（`mount.rs:3709`，本体 `:3054`） | 不写 |
| 10 | 算要取的号、要写的行 | `next_instance_generation_to_acquire`（`:3711`）、`instance_rows_to_write`（`:3719`） | 不写 |
| 11 | 空间准入（实例切换预留） | `admit_on_every_device`（`:3737` 起） | 不写 |
| 12 | 取号之后那一串（写行 + 暖机）在分配器拷贝上整串预演 | `dry_run_of_the_publishes_after_acquisition`（`:3767`，本体 `:2874`） | 不写 |
| 13 | 逐盘核（带不带所选那一版） | `:3812-3825` | 不写 |
| 14 | **取号**（这次挂载的第一个写） | `acquire_expected_instance`（`:3838`；`transaction.rs:797`） | 写系统配置 |
| 15 | 写行：带文件的一版上 `publish_rows_on_file_version`（`:3872`，本体 `:2758`）；树表 0 条的一版上零单元发布或只重写实例表（`:3887-3929`） | 一次发布 |
| 16 | 暖机：按第 9 步的计划连推空发布，直到本实例的根覆盖每块盘（D16 已定项 8 戊，`:3931` 注释） | 循环 `:3940-3955`，每次 `publish_empty_after`（`:3946`，本体 `:2818`） | 每次一次发布 |
| 17 | 准入在取号前判「不够」且上一版带文件：推抬 F 的空发布再判 | `push_floor_raises_after_the_row_publish`（`:4005`，本体 `:4087`） | 若干次发布 |
| 18 | 交回 | `Ok(Mounted { … })`（`:4018`），`MountOutput` 带 `row_publish`、`warm_up_publishes` | — |

mkfs 同一个进程里那条流不走可写挂载：`make_filesystem` → `acquire_instance`（`transaction.rs:747`）→ `warm_up`（`transaction.rs:907`）→ `publish_first_file`（`transaction.rs:4624`）。挂着之后的用户发布经 `MountedSession::publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:238`），准入不够时先推抬 F（每次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST = 8` 次发布，`mount.rs:2123`）。

「每次可写挂载先续做意图」能挂在哪（**全部是推的**）：

| 候选挂点 | 理由 | 要跟着动的 |
|---|---|---|
| 暖机循环之后（`mount.rs:3955` 之后）、抬 F（`:4005`）之前 | 暖机保证本实例的根先覆盖每块盘（D16 已定项 8 戊），之后的发布才是「本实例的发布」；续做一批就是一次普通发布，接在这里与用户发布同形 | 第 12 步的预演（`:3767`）与第 11 步准入要把续做那几批算进去，否则取号前的准入罩不住实际推的发布——同一个问题注释里已经点过暖机（`:3931-3932`「两处各算各的……准入就罩不住」）；`MountOutput` 要加一格记续做的发布 |
| 抬 F 之后、`Ok(Mounted)`（`:4018`）之前 | 续做释放空间，放在抬 F 之后准入读数更紧、放在之前抬 F 可能不必推（推的，未量） | 同上 |
| 读阶段之后、取号之前（第 5–7 步之间）只「读出」意图、不写 | 意图在树里，第 5 步重建一版时顺带读出来（`recovery.rs:2350-2374` 那一串按种类取树）；写必须在取号之后 | `RebuiltVersion` / `TransactionOutput` 加意图字段 |

只读路径 `recovery::recover`（冷启动）不写盘、不会续做：带着未完成意图的镜像只读打开时读回的是「删了一半」的中间态，D8 已定项 5 第 3 条要求它本身合法（推的）。
（冷启动入口 `recovery::recover` 在 `recovery.rs:4222`。）

## (e) 记账：发布时怎么增量维护，I-3.1 / I-3.11 各落在哪

| 统计量（标签，`records.rs:15-23`） | core 里维护处 | 写进记账行 | checker 读法 |
|---|---|---|---|
| 已分配字节（1，逐盘） | `DeviceFreeMap::allocated_slots`：`mark_allocated` 加（`allocator.rs:704` 起）、`mark_reclaimed` 减（`:657` 起）；释放（`mark_released`，`:549`）**不减** | `transaction.rs:6297` `allocated_slots() * SLOT_BYTES` | I-3.1：`walk.rs:6288`，已分配 == 全部有效根引用的槽之并 + 隔离豁免 + F 之下仍在 defer 的（`walk.rs:6265-6294`）；最新根带的 F 低于 F 生效值时报不适用（`:6331-6336`） |
| 空闲字节（2，逐盘） | `free_slots`：分配减、回收加（`allocator.rs:704`、`:657`） | `transaction.rs:6299` | I-5.2：空闲 + 已分配 == 单元区（`walk.rs:6304-6306` 附近） |
| defer 队列（5，逐盘） | `deferred_slots`：释放加（`:549`）、回收减（`:657`）、回退复活减（`:560`） | `transaction.rs:6304` | I-3.11：defer + 最新根走读（+ 隔离豁免）== 已分配（`walk.rs:6308-6328`，判定 `:6324`） |
| 不可回收（3，逐盘） | 常量 0（`admission.rs:147` `UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`） | `transaction.rs:6300-6302` | 不判 |
| 碎片段数（10）、全空聚簇段数（11），逐盘 | `free_runs`、`used_per_segment` 在 `mark_allocated` / `mark_reclaimed` 里增量改 | `transaction.rs:6305-6306` | 不在这次核的范围 |
| inode 号水位（12，池级） | 上一版 + 这次新建数 | `transaction.rs:6266-6281` | I-9.6（`walk.rs:6130` 起） |
| **待删占用（4，池级）** | **常量 0**：`PENDING_DELETE_OF_THE_FIRST_VERSION`（`admission.rs:151-152`，注释「第一版不删对象……恒 0」） | `transaction.rs:6283-6285` | **不读**：`grep -n '待删\|PENDING_DELETE' crates/singlefs-checker/src/*.rs` 输出为空 |
| 已承诺预留（6，池级） | 常量 0（`admission.rs:156`） | `transaction.rs:6286-6288` | 不读 |

一次发布写的记账行是整张表重写（池级三行 + 每盘六行，代 = 这次 txg、seq 恒 1），由 `accounting_entries_of_this_publish`（`transaction.rs:4101`）按两张枚举 `PoolWideAccountingRow`（`:4031`）、`PerDeviceAccountingRow`（`:4068`）展开；值全部取分配器那一刻的内存计数，不扫盘。core 里没有判 I-3.1 / I-3.11 的代码，只有维护（`grep -rn 'I-3\.1\b\|I-3\.11' crates/singlefs-core/src/*.rs` 命中的都是注释：`admission.rs:232`、`allocator.rs:257`、`transaction.rs:1718`、`:5208`、`mount.rs:2441`、`:2495`）；判只在 checker（`walk.rs:6288`、`:6324`），清单在 `crates/singlefs-checker/src/image.rs:82`、`:86`。

「待删占用」若启用会进的结构：

| 层 | 结构（文件:行） | 今天 |
|---|---|---|
| 统计量标签 | `STATISTIC_PENDING_DELETE_BYTES = 4`（`records.rs:18`） | 已登记 |
| 记账行 | `PoolWideAccountingRow::PendingDeleteBytes`（`transaction.rs:4031` 起），树 ID 段写 0（`:4055-4062`），设备段写 `0xFFFF_FFFF`（`records.rs:25`） | 每次发布写 0 |
| 单位 | `BytesSummedOverAllReplicas`：按全部副本之和（`admission.rs:69-73`，C375 第三轮写回的口径） | — |
| 准入 | `PoolWideCommitments::pending_delete`（`admission.rs:249`），`of_the_first_version` 填常量（`:259-265`）；`available_on_each_device`（`:361`）扣「待删占用 ÷ 副本数」 | 扣 0 |
| checker | 无 | I-3.4 未实现（`.claude/kb/invariants.md:139`） |

推的一个口径问题（没量过、没核条款）：要删的 extent 在释放之前仍在「已分配」里（`allocated_slots` 只在回收时减），准入式子已经扣「已分配」；若「待删占用」按这些 extent 的字节记，同一批字节会被扣两次。三·1 要定「待删占用」记的是什么（尚未释放的 extent、还是别的量），以及它与「已分配」「defer」的关系。

## (f) 一次发布里记录条数与单元数的上限

| 常量 / 函数 | 值 | 文件:行 | 管什么 |
|---|---|---|---|
| `journal_record_limit_of_one_publish(ring)` = min(`journal_in_flight_record_limit(ring)`, `u32::MAX`) | 默认环 805 306 368 字节时 = 805306368 ÷ 4096 ÷ 3 = **65536**（`python3 -c 'print(805306368//4096//3)'` 输出 `65536`） | `transaction.rs:5387-5389`；`system_configuration.rs:88-90`；`JOURNAL_RING_DEFAULT_BYTES` `lib.rs:213`，`JOURNAL_RECORD_BYTES` `lib.rs:191`，`JOURNAL_SAFETY_FACTOR` `lib.rs:221` | 一次发布最多切几条 journal 记录；多了在动分配器之前拒（`refuse_a_publish_of_more_journal_records_than_the_limit`，`transaction.rs:5396`，调用 `:1242`、`:1649`、`:5112`） |
| `JOURNAL_NAMED_ENTRIES_PER_RECORD` | 67 | `lib.rs:206` | 一条记录点名几个单元；N 个数据单元 = N 个事务 = 前 N − 1 条各点名一个数据单元，其余角色全点名在最后一个事务里、装不下再跨记录（`roles_named_by_each_record_of_the_publish`，`transaction.rs:5343`；文档 `:5331-5341`） |
| 只供测试：`JournalRecordNamedEntryCapacity::CappedForTests` | 1..=67 | `transaction.rs:140-178`，装入 `:251-262` | 压小一条记录的点名项数，用少量单元造出跨记录发布（「分批必须能被测试强制进入」的现成先例） |
| `leaf_containers_one_root_node_holds()` | 135（(16384 − 131) ÷ 120） | `inode_tree.rs:108-111`；拒因 `:142` | 这次发布之后 inode 叶容器数的上限；乘 `INODE_LEAF_RECORDS` 233（`lib.rs:110`）⇒ inode 树最多 31 455 条记录（算术，推的） |
| `EXTENT_TREE_UPPER_LEAF_INODES` / `EXTENT_TREE_LOWER_LEAF_DATA_UNITS` | 143 / 144 | `lib.rs:163`、`lib.rs:156` | 不是上限：extent 树按位置长层（`extent_tree.rs:89-101`） |
| `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT` | 64 | `transaction.rs:5329` | 分配记录树固定点轮数，不是单元数 |
| `PUBLISHES_PER_ADMISSION_AT_MOST` | 8 | `mount.rs:2123` | 一次准入（含推的抬 F）最多几次发布 |
| 只供测试：`CodeTwoTreeNodeCapacities::CappedForTests` | 叶 ≥ 1、内部 ≥ 2 | `transaction.rs:218-240` | 压小记账树、映射树节点容量 |

「一次发布几个单元」没有单独的常量：`grep -rn 'pub const [A-Z_]*: [a-z0-9]* = ' crates/singlefs-core/src/*.rs | grep -i 'most\|limit\|max\|cap\|per_'` 只列出 `address.rs:90` `COUNTER_LIMIT`、`admission.rs:160`、`:163`、`:167`、`transaction.rs:5329`、`mount.rs:2123` 与 extent 标签三个。单元数实际受三样约束：记录条数上限（数据单元一单元一条）、空间准入（`admit_on_every_device`）、inode 叶 135 片。另注意 `publish_sequential_write` 的文档注释（`transaction.rs:4768`）还写着报 `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`，而这个名字在 crates/ 里只剩这一处注释（`grep -rn` 只命中 `transaction.rs:4768`），是一句过时的注释。

## (g) 理想模型与随机历史

### g1 模型（`crates/singlefs-harness/src/model.rs`）

| 东西 | 今天的样子 | 文件:行 |
|---|---|---|
| 一次发布重写哪些角色 `ModelPublishKind`（私有） | `FirstFileVersion`、`OverwriteFileVersion`、`RowsOnFileVersion`、`EmptyOnFileVersion`、`RowsOnVersionWithoutFile`、`ZeroUnit` 六种；角色表在 `:130-150` 附近 | `model.rs:98-113` |
| 公开的操作种类 `ModelOperationKind` | `PublishFirstFile`、`PublishOverwrite`、`PublishWithoutUnits`、`MountWritable`、`RollbackWhileMounted`、`RaiseRollbackFloor`、`ColdStartRecover` 七种 | `model.rs:377-385` |
| 一版的文件 | `ModelRoot::file: Option<ModelFileVersion>`，注释「树表 0 条（第 0 代根、暖机根、零单元发布）时没有文件」——模型里整池**只有一个文件**，内容一份 | `model.rs:199-200`、`ModelFileVersion` `:180-183` |
| 读回答案 `ModelReadBack` | `NoFile { root }`、`FileRead { root, content }` | `model.rs:365-373` |
| 观测侧 `ObservedFile` | `NoFile`、`Decoded(Vec<u8>)`、`Undecodable { what }` | `model.rs:441-448` |
| 单元角色 `ModelUnitRole` | Data、ExtentRoot、InodeLeaf、InodeRoot、AllocationTree、AccountingTree、MappingTree、TreeTable、InstanceTable | `model.rs:65-75` |
| 拒因 `ModelRefusalReason` | 含 `ContentExceedsDataUnitPayload`：模型只接受一个数据单元装得下的内容 | `model.rs:286`、`:321` |
| 每步的答案与比对 | `answer_publish_first_file`（`:1107`）、`answer_publish_overwrite`（`:1149`）、`answer_publish_without_units`（`:1189`）、`answer_mount_writable`（`:1252`）、`answer_rollback_while_mounted`（`:1363`）、`answer_raise_rollback_floor`（`:1544`）、`answer_cold_start_recover`（`:1625`）；`judge_and_advance`（`:1694`） | — |

模型不认 `publish_new_inodes`、`publish_sequential_write`（多单元）这两条今天已有的入口（`ModelOperationKind` 里没有，`ContentExceedsDataUnitPayload` 拒多单元）。所以「删后读到没有文件」在模型里可以直接落到 `ModelRoot::file = None` / `ModelReadBack::NoFile`（推的），但今天 `None` 的语义是「树表 0 条」，模型里有几处按它判「这一版有没有树」（例如 `:199` 注释、`ModelPublishKind::RowsOnVersionWithoutFile` / `ZeroUnit` 两种按树表 0 条选），删后的一版树表不是 0 条，要分开（推的）。

### g2 随机历史（`crates/singlefs-harness/src/history.rs`）

`HistoryOperation`（`history.rs:349-371`）八个成员：`PublishFirstFile(ContentChoice)`、`PublishOverwrite(ContentChoice)`、`PublishWithoutUnits`、`CloseAndMountWritable`、`RollBackWhileMounted(RollbackTargetChoice)`、`RaiseRollbackFloor(FloorTargetChoice)`、`ColdStartRecover`、`CrashRecoveryAbandoningTheNewestRoot`；种类枚举 `HistoryOperationKind`（`:375-384`）与 `ALL: [_; 8]`（`:387-396`）。

### g3 加一个删除操作要碰的地方（按 `PublishOverwrite` 今天出现的位置数；`grep -rn 'HistoryOperation::PublishOverwrite\|HistoryOperationKind::PublishOverwrite\|ModelOperationKind::PublishOverwrite\|answer_publish_overwrite\|ModelPublishKind::OverwriteFileVersion' crates/ --include=*.rs`）

| 文件 | 函数 / 位置（文件:行） | 要做什么（推的） |
|---|---|---|
| `model.rs` | `ModelPublishKind`（`:98`）与角色表（`:134`、`:144`）；`ModelOperationKind`（`:377`）；`answer_publish_overwrite` 同形的新 `answer_*`（`:1149` 为样板，`:1172`、`:1178`）；`ModelUnitRole`（`:65`，新树的单元）；`ModelReadBack` / `ModelRoot::file` 的「删后没有文件」语义 | 加删除种类、答案与「没有文件」读回 |
| `model_comparison.rs` | `observed_root_of_file_version`（`:127`）、`observed_file_of_file_version`（`:146`），`ObservedFile::NoFile` 今天只由树表 0 条那一版给（`:399`、`:502`） | 从删后的一版取观测 |
| `history.rs` | `HistoryOperation`（`:349`）、`HistoryOperationKind` + `ALL`（`:375`、`:387`）、`kind()`（`:401`，`:404`）；比重表（`:475`、`:479`、`:513`、`:517`、`:547`、`:551`、`:575`、`:600`）；`draw_operation`（`:652`，`:661`）；`expected_session_after`（`:703`，`:723`）；`note_outcome`（`:1874`，`:1959`）；`settle_user_change`（`:2818`，`:2865`、`:2946`）；`apply_operation`（`:3748`，`:3762`）；`simpler_variants`（`:4244`，`:4272`）；`:4723`、`:4955`、`:5021` 三处测试 | 加成员与各处分支 |
| `crates/singlefs-harness/src/fault_injection.rs` | `system_configuration_instance_before_a_mount_step`（`:1658`，`:1674`）、`inject_one_fault`（`:2173`，`:2320`）、`:3165` | 加分支 |
| 用 `HistoryOperation` 的测试与装置 | `crates/singlefs-harness/tests/random_histories.rs`（`:140` 等）、`fault_injection_fast_tier.rs:184`、`admission_formula_on_the_mounted_state.rs:397`、`root_ring_turn_in_one_mount.rs:189`；checker 档 `crash_injection_campaign.rs:272`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs:244`；装置 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:17888`、`:18953`（有 `match`，穷举时要加臂） | 穷举 `match` 处加臂 |

## (h) `grep -rn '没有删除' crates/ research/ .claude/kb/ --include=*.rs --include=*.md`

命中 45 行（`| wc -l`），按文件分：crates/ 4 行（`inode_tree.rs` 2、`admission.rs` 1、`walk.rs` 1），research/ 下 `e7-index-bench` 1 行、`research/prompts/` 35 行，.claude/kb/ 5 行（`checks-owed.md` 2、`invariants.md` 1、`milestone/02-second-txn.md` 1、`milestone/03-third-txn.md` 1）。里程碑表点名的两处代码注释都在：`crates/singlefs-core/src/inode_tree.rs:16`、`crates/singlefs-checker/src/walk.rs:6133`；另有 `inode_tree.rs:28`（「位置一旦定下就不再挪」靠没有删除）与 `admission.rs:154`（已承诺预留恒 0 靠没有删除路径）两处代码注释的前提会随删除失效。原样输出（不截断）：

```text
crates/singlefs-core/src/inode_tree.rs:16://! **合并不在这个模块里**：这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点）），
crates/singlefs-core/src/inode_tree.rs:28:/// 位置随左边有没有别的容器而变，两者不是一个量。这一版没有删除与合并 ⇒ 位置一旦定下就不再挪
crates/singlefs-core/src/admission.rs:154:/// 式子里的「已承诺预留」（记账第 6 项，含墓碑）：第一版没有删除路径，墓碑的预付（D3（空间分配） 已定项 2，量是 C84（墓碑单元的粒度没人定）
crates/singlefs-checker/src/walk.rs:6133:    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
research/prompts/governance-defs-r1-sonnet-output.md:34:丢判据检查：这处是纯增补（原句「禁止在subagent中跑重型测试（…）」整句保留，只加一个分句），没有删除任何原文，不涉及丢判据。
research/prompts/governance-defs-r1-sonnet-output.md:166:丢判据检查：experiment-runner.md 步骤 3 旧句「写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表」被拆成两支（入库装置一支 / research 一支），research 一支的原判据（跑 `mutate.sh`、路径写法、bin 名核对、「基线就是红的」排查提示、「已还原，基线仍全绿」要求）全部原样保留在新的步骤 3 后半段（`.claude/agents/experiment-runner.md:27` 现查这几句都还在），没有删除，只是前面加了入库装置的分支判断。产出一节旧句「变异三个数与分类」被替换为「变异三个数（抓到/无效/没红）与分类，另报 `mutate.sh` 收尾的内存撞顶与超时（不为 0 的整轮已判失败）」——这与 mutation-triage.md 的同类修改（C-13，`governance-rot-r2-audit-C.md` 第 13 行「只报三个数会把这两类丢掉」）对应，是补全不是删减。
research/prompts/_m2-closeout-code-r3-background.md:771:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/m2-safety-r2-local-attack-translation-audit.md:15:| Fact 3：Part 1 历史里 pending_delete=0、committed_reservation=0、unreclaimable=0 | 非 kb 逐字引用；是我对 Fact 13 那段历史（Z19-B 形态：mkfs 后只做同一个文件的同形覆盖写）本身范围的刻画，套用与 `m2-safety-r1-local-attack.md` Fact 4（`crates/singlefs-harness` 历史定义）同一条推理：这段历史没有删除、没有墓碑操作、池不跑 zoned | 首稿即如此，无「首稿缺」——这条本身就是构造性陈述，不是摘自某一行 kb 文本 | 保留构造性陈述的写法，不假装它是逐字摘句 |
research/prompts/m2-safety-r2-local-attack-translation-audit.md:49:| 待删占用 ÷ 副本数 | 第一版恒 0 的常量（这段历史没有删除、没有墓碑） | `admission.rs:142` `pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas = ...` | 0 | 0 |
research/prompts/m2-treesplit-r1-main-verification.md:33:**分隔 key：主 agent 定「跟着维护」，不交用户。** 攻方打中：把 D8 已定项 6 给 inode 树定的「分隔 key = 孩子创建时的最小 key、之后不变」搬到分配记录、映射、extent 这几棵插入不单调的树上，读者按分隔 key 会找不到一条确实在树里的 key，checker 在恢复后判红——t1.out 取这一条的 24 格全红（check_red 34952–562983），同一批历史改成「跟着维护」（插到最左分隔 key 之下时把它压低；借一条之后把右孩子的分隔 key 改成它新的最小 key）24 格全是 0。判据靠的是块头自带的 key 区间：分隔 key ≤ 孩子头的最小 key、> 左邻孩子头的最大 key，逐层可判。另一处打中是推的：D8 已定项 6 的「左从右借」对 inode 树本身也会让 I-9.4「左容器的最大 key < 右容器号」当场为假；今天实现没有删除，还没有历史走到，记欠账。
research/prompts/m2-treesplit-r1-main-verification.md:94:| D8（核心索引结构） 已定项 6 | 不影响 | 2026-09-24 不受影响：「左从右借」对 inode 树为假是推的，今天没有删除，记欠账，定案这一轮不改 |
research/prompts/m3-prune-gpu-r1-sonnet-output.md:46:| **等实现** | Z1（删除，`crates/singlefs-core/src/inode_tree.rs:16` 已写明「这一版没有删除（`deleted_inodes` 的形态无落点，C118）」）；Z15（真设备上的崩溃状态，55 号只有真实负载） | `crates/` 里对应机制不存在（grep 零命中） |
research/prompts/_m3-prune-gpu-r1-background.md:2619:| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
research/prompts/_m2-treesplit-r1-background.md:1378:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：回退之后号会被重发，被抛弃时间线的墓碑不算，否则 checker 按扫描读在 0 故障的回退之后就判红；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/m2-final-code-r4-local-attack-translation-audit.md:37:COMMITTED_RESERVATION_OF_THE_FIRST_VERSION, docs at :140-141 and :145-146: "第一版没有删除路径
research/prompts/m2-final-code-r4-local-attack-translation-audit.md:38:...⇒ 恒 0" / "第一版没有删除路径，墓碑的预付...都不发生 ⇒ 恒 0").
research/prompts/_m3-prune-gpu-r1-appendix.md:2254:| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
research/prompts/m2-treesplit-r1-opus-output.md:89:**推到 inode 树本身（推的，没在实现上跑）**：inode 树的右半「容器号」就是它的分隔 key（`crates/singlefs-core/src/inode_tree.rs` 的 `separator_key`），合并纪律写的是「合并：只许**左吸收右 / 左从右借**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份」（08 第 169 行）。左从右借一条 r：r ≥ 右容器号（I-9.4 前半），借走以后左容器的最大 key = r ≥ 右容器号，而 I-9.4（`.claude/kb/invariants.md` 第 271 行）写「沿叶序容器号严格递增，且左容器的最大 key < 右容器号」——**任何一次「左从右借」都让 I-9.4 当场为假**，除非右容器换号，而换号就是换身份。今天这一版没有删除（`inode_tree.rs` 模块注释「合并不在这个模块里」），所以还没有历史走到这里。
research/prompts/m2-treesplit-r1-opus-output.md:275:| inode 树删掉「左从右借」这一半，只留左吸收右 | T1 打中一的借那一形（inode 树） | 推的：没在实现上跑，今天没有删除 |
research/prompts/m2-treesplit-r1-opus-output.md:305:- 「inode 树的左从右借必破 I-9.4」是从条款推的，没在实现上跑；今天实现没有删除。
research/prompts/m3-prune-gpu-r1-facts-case-design.md:204:| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
research/prompts/_m3-prune-gpu-r2-appendix.md:2205:| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
research/prompts/_m2-wave3-code-r1-diff.md:6447:    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
research/prompts/m2-safety-r3-verifier-output.md:241:- 没有删除 `/tmp/claude-1000/m2-safety-r3-verifier/opus-rerun/out/`、`sonnet-rerun-verify.log`、`selftest/`、
research/prompts/m2-step3-code-r1-main-verification.md:14:| 1（X3 分配器重建） | 1a NO HIT（算术复述）；1b HIT：「建文件 → 删文件 → 重开，已释放而无根引用的单元仍在已分配里、checker 并集不含 ⇒ I-3.1 红」；1c 指出 `STATISTIC_INODE_WATERMARK` 每次发布写死 `FIRST_INODE_NUMBER + 1`；1d / 1e NO HIT | 1a NO HIT；1b HIT（同一构造）；1c 同；1d NO HIT | 1b 两份一致，但构造用了「删文件」：第一版没有删除，释放只发生在被换下那一刻（D3（空间分配） 已定项 7），换下它的那条根仍在环里、仍引用它，并集里有它。**没打中**；它说的机理正是回退与抬 F 之后要面对的（被抛弃根、F 之下的根引用的单元），步 4 / 步 5 的代码按实例表与 F 收候选集，另立一轮判。1c 两份一致：今天一个 inode 恒真，多 inode（并行线三）要改成从 inode 树取，记进并行线三的决策点 |
research/prompts/_m2-treesplit-r1-appendix.md:824:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：回退之后号会被重发，被抛弃时间线的墓碑不算，否则 checker 按扫描读在 0 故障的回退之后就判红；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/m2-safety-r2-sonnet-output.md:207:  恒 0，`admission.rs:142-143`，这是「没有删除路径」这句话在代码里的样子，所以「删文件腾出空间」这条问句在
research/prompts/m2-rev-i111-implementer-report.md:8:- `crates/mutations.tsv`：没有追加、没有替换、没有删除任何行。检查过点名 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 的两行（第 874、878 行，锚点在 `crates/singlefs-checker/src/walk.rs`），它们的锚文本落在 `walk.rs` 里，这一轮没有改 `walk.rs`（不在「要动的 crates 文件」清单里），锚文本原样还在、恰好命中一次，没有腐化。
research/prompts/_m2-closeout-code-r1-background.md:776:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md:111:| C116 | 叶容器的分裂 / 合并纪律没有会失败的检查 | D8（核心索引结构） 已定项 6 的三条写路径纪律对着一个镜像判不了：末尾分裂（左半保留全部记录与身份、右半容器号 = 触发分裂的新记录号）、合并保留左半身份、记录数为 0 的类型 2 容器不落盘。三条任一条写错，容器身份就会重复或空叶在重建时没有 key 范围，而 I-9（inode 树结构） 类只能在事后从镜像上抓到症状 | 模型对拍：随机创建 / 删除 / 克隆序列下逐次发布比对容器身份集合与 I-9.4（容器号不超最小 key）；崩溃点重放：在分裂 / 合并那次发布的每个写请求后截断，重放后不许出现记录数为 0 的类型 2 容器。**分裂那三条已还（2026-09-22）**：`crates/singlefs-core/src/inode_tree.rs` 把「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」做成不发一个写的纯函数，三条纪律在它上面是后置条件、10 条单测判得了，另一层是恢复加 oracle（`crates/singlefs-harness/tests/second_transaction_parallel_line_three_many_inodes.rs`：期望态带着分裂前的容器身份，发布后冷启动重建逐项比）；`crates/mutations.tsv` 末尾三条变异各把一条纪律改坏（右半容器号给错、分裂点不取末尾、COW 重写时身份跟着换出生代），三条都红。**仍欠合并与空容器那两条**：这一版没有删除（C118（`deleted_inodes` 树的形态无落点）），记录只增不减，合并（只许左吸收右 / 左从右借）与「记录数为 0 的类型 2 容器不落盘」都没有对象 | 事务层、崩溃点重放（verification-build.md；两样 2026-09-14 都有了，第一个事务里没有叶容器的分裂与合并） | 2026-09-05 D8（核心索引结构） 已定项 6 第三轮正推腿结论表、反推腿详核 1.2 / 1.3 |
research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md:462:| C536 | inode 树的「左从右借」让容器号条件为假 | D8（核心索引结构） 已定项 6 给 inode 树定的「左从右借」：借走右容器最小的那条之后，左容器的最大 key 可以 ≥ 右容器号，I-9.4（容器号不超最小 key） 的「沿……」那一半当场为假（推的，今天实现没有删除，没有历史走到） | inode 树删除落地时改借法或改 I-9.4（容器号不超最小 key） 的判据；一条用例：左从右借一次之后跑 checker，I-9.4（容器号不超最小 key） 那一半必须判得出（改借法之前红、改后绿） | 开着：inode 树删除不在里程碑「第二个事务」 | 2026-09-24 三方 `research/prompts/m2-treesplit-r1-main-verification.md` 「分裂与收缩规则」那一节 |
research/prompts/_m2-rollback-forward-r2-appendix.md:572:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：回退之后号会被重发，被抛弃时间线的墓碑不算，否则 checker 按扫描读在 0 故障的回退之后就判红；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/_m2-closeout-code-r1-appendix.md:238:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/invariants.md:278:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/_m2-closeout-code-r3-appendix.md:365:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/_m2-rollback-forward-r2-background.md:987:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：回退之后号会被重发，被抛弃时间线的墓碑不算，否则 checker 按扫描读在 0 故障的回退之后就判红；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/prompts/_m2-five-decisions-code-r1-diff.md:1293:    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
research/prompts/_m3-prune-gpu-r2-background.md:2585:| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/invariants.md:279:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
research/e7-index-bench/src/bin/e104_current_version.rs:1429:    // 闭式：逐根数它引用的身份数（没有删除 ⇒ 每个根可见的身份恰是它引用的），不归并时每个身份一次平局
.claude/kb/invariants.md:280:| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
.claude/kb/checks-owed.md:110:| C116 | 叶容器的分裂 / 合并纪律没有会失败的检查 | D8（核心索引结构） 已定项 6 的三条写路径纪律对着一个镜像判不了：末尾分裂（左半保留全部记录与身份、右半容器号 = 触发分裂的新记录号）、合并保留左半身份、记录数为 0 的类型 2 容器不落盘。三条任一条写错，容器身份就会重复或空叶在重建时没有 key 范围，而 I-9（inode 树结构） 类只能在事后从镜像上抓到症状 | 模型对拍：随机创建 / 删除 / 克隆序列下逐次发布比对容器身份集合与 I-9.4（容器号不超最小 key）；崩溃点重放：在分裂 / 合并那次发布的每个写请求后截断，重放后不许出现记录数为 0 的类型 2 容器。**分裂那三条已还（2026-09-22）**：`crates/singlefs-core/src/inode_tree.rs` 把「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」做成不发一个写的纯函数，三条纪律在它上面是后置条件、10 条单测判得了，另一层是恢复加 oracle（`crates/singlefs-harness/tests/many_inodes_write_and_read.rs`：期望态带着分裂前的容器身份，发布后冷启动重建逐项比）；`crates/mutations.tsv` 末尾三条变异各把一条纪律改坏（右半容器号给错、分裂点不取末尾、COW 重写时身份跟着换出生代），三条都红。**仍欠合并与空容器那两条**：这一版没有删除（C118（`deleted_inodes` 树的形态无落点）），记录只增不减，合并（只许左吸收右 / 左从右借）与「记录数为 0 的类型 2 容器不落盘」都没有对象 | 事务层、崩溃点重放（verification-build.md；两样 2026-09-14 都有了，新池新建文件里没有叶容器的分裂与合并） | 2026-09-05 D8（核心索引结构） 已定项 6 第三轮正推腿结论表、反推腿详核 1.2 / 1.3 |
.claude/kb/checks-owed.md:427:| C536 | inode 树的「左从右借」让容器号条件为假 | D8（核心索引结构） 已定项 6 给 inode 树定的「左从右借」：借走右容器最小的那条之后，左容器的最大 key 可以 ≥ 右容器号，I-9.4（容器号不超最小 key） 的「沿……」那一半当场为假（推的，今天实现没有删除，没有历史走到） | inode 树删除落地时改借法或改 I-9.4（容器号不超最小 key） 的判据；一条用例：左从右借一次之后跑 checker，I-9.4（容器号不超最小 key） 那一半必须判得出（改借法之前红、改后绿） | 开着：inode 树删除不在里程碑「覆盖写、释放、回退与复用」 | 2026-09-24 三方 `research/prompts/m2-treesplit-r1-main-verification.md` 「分裂与收缩规则」那一节 |
.claude/kb/milestone/03-third-txn.md:78:- **仓里已有的**：概念层已定：D8（核心索引结构） 已定项 5（可能修改无界数量 extent 的操作先把意图写进 logged_ops 树再分批推进、崩溃后续做；删除中的 inode 单独进 `deleted_inodes` 树、由后台回收；意图只声明目标态、续做是每次恢复都跑的常规动作、每个中间态都是合法状态、删除意图与最后一批工作同一事务提交）与已定项 6（回收完成前 inode 记录留在 inode 树里、nlink = 0，已发布的 (inode 号, 对象出生代) 不复用）。格式层没落点：C118（`deleted_inodes` 树的形态无落点）（key、value、树 ID、新池新建文件建不建空树都没写），意图记录住哪棵树全仓没写。代码里零删除：`crates/singlefs-core/src/inode_tree.rs` 第 16 行与 `crates/singlefs-checker/src/walk.rs` 第 6133 行都写明「这一版没有删除」；`crates/singlefs-harness/src/model.rs` 的操作枚举只有第一个文件版本、覆盖写、写行、空发布、零单元发布。未实现的不变量：I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-3.4（可用空间扣待删占用）。意图机制还有两个消费者：D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表、D26（后台整理与放置回收） 的整理。安全设计轮第三轮「崩在删除类发布中间」（近满之后截断成 0 那一次发布，一条流 16 777 223 个状态）没跑（`research/prompts/m2-safety-r3-main-verification.md` 第四节）。
.claude/kb/milestone/02-second-txn.md:578:- 与六家的口径：fio 的元数据作业按路径建文件、stat、删；这条线没有名字空间、没有删除，只能报「建 N 个 inode 并持久 / 秒」，性能表里那一格写「不可比」。删除（`deleted_inodes` 树，C118（`deleted_inodes` 树的形态无落点））不在这条线里。
```

（同一条 grep 跑了两次，两份输出排序之后逐行相同（`diff <(sort …) <(sort …)` 输出 `sorted-same`），只是 `grep -r` 的遍历次序不同；上面贴的是第一次那份。）

## (i) 崩溃放量用的固定脚本流

### i1 定义在哪

| 东西 | 文件:行 | 说明 |
|---|---|---|
| 公共步骤模块 | `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` | 崩溃点的文件是这一份（`file!()`），几条流调同一个函数就得同一个身份、同一串路径段，前缀在库里只核一次（`mod.rs:1-4`） |
| 已发的两段 | `record_pool_build`（`mod.rs:27-38`） | `acquire_instance_and_warm_up`、`publish_first_file` 两段，按 `build_pool` 已经写下的录制流下标记 |
| 各入口的包装 | `overwrite_step`（`:41-63`，名 `publish_overwrite`）、`mount_writable_step`（`:66-73`）、`roll_back_step`（`:76-93`）、`raise_rollback_floor_step`（`:96-116`）、`unmount_step`（`:120-152`） | 每个都是 `recorder.step(file!(), "<入口名>", || …)` |
| 跑一条录好的流 | `amplify_prepared_flow`（`:162-286`） | 展开上限读 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`；断言红 0 |
| 整条写表当一个点 | `one_crash_point_over_every_write`（`:308-320`） | 只录一次发布的那几条流用 |
| 记录器 | `CrashPointRecorder`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2266`）：`record_span`（`:2288`）、`step`（`:2306`）、`into_crash_points`（`:2325`） | 段不接上前一段就 panic（`:2293-2297`）；流尾有没归点的操作就 panic |
| 段名 | `into_crash_points` 里 `format!("{entry_name}.{period}.{step_in_period}")`（`:2389`；`kinds.join("+")` 在 `:2381`） | 每个屏障段一个点；period = 这一段写的种类去重后用 `+` 接；step = 同一入口里同一种 period 的第几段 |
| 写种类 | `period_of_write`（`:2234-2249`）、`period_of_unit_class`（`:2253-2259`） | 单元写按头第 6 字节类标签：1 → `data_unit`、2 → `index_node`、3 → `packed_records`；其余按 `StepKind` 名（`journal_record`、`root_record_fua`、`system_configuration_slot` …） |
| 判法摘要不含它 | `crates/singlefs-checker-tier/src/crash_identity.rs:800-803`（`JUDGE_ENTRY_FILES` = `crash.rs`、`crash_judge_gpu.rs`）；自测 `:1261` 断言改 `common_crash_points/mod.rs` 不动摘要 | 反过来：改 `recovery.rs`、`walk.rs` 会动摘要（同一自测的 changed 列表），库里全部块要重判（推的：按摘要换键） |

### i2 用它的测试

| 测试 | 流 | 调了哪几步 |
|---|---|---|
| `crash_amplification_fixed_script_through_rollback.rs` | mkfs → 取号加暖机 → A → B → 重开可写挂载 → C → 回退到 A（D） | `record_pool_build`、`overwrite_step`、`mount_writable_step`、`roll_back_step`（`:17`） |
| `crash_amplification_fixed_script_through_unmount.rs` | 到 D 之后再五次覆盖写（txg 10–14）→ 抬 F 到 11（txg 15、16）→ 复用 E（txg 17）→ 正常卸载（txg 18、19）；78 段、477 次写、19 条根槽写（`:2-5`） | 上面四个加 `raise_rollback_floor_step`、`unmount_step`（`:18-21`）；78 个点的名字整表写死在 `:234-312`，数量断言 `:201` |
| `crash_amplification_two_flows_share_their_prefix.rs` | 两条流共前缀、尾巴各自覆盖写 | `record_pool_build`、`overwrite_step`（`:15`） |
| `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 六条流，其一是固定脚本到 D | 同 through_rollback（`:18`） |
| `crash_enumeration_multi_record_publish_stream.rs` | 取号 → 暖机 → A → B（两单元）→ C（三单元） | `record_pool_build`（`:431`）、`amplify_prepared_flow`（`:470`） |
| `crash_enumeration_floor_raise_pushed_by_admission.rs`、`crash_enumeration_position_addressed_trees.rs`、`crash_enumeration_tree_split_streams.rs`、`record_checker_judges_absence_by_the_persisted_set.rs` | 各自的小流 | 只用 `one_crash_point_over_every_write` / `amplify_prepared_flow` |

层 0 全量那条同形的流另在 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`，不经 `common_crash_points`：`Script` 枚举（`:71-87`）的 `ReuseAfterRaisingFloorThenNormalUnmount` 就是 through_unmount 那一条，流名 `file_overwrite_fixed_script_through_e_then_unmount`（`:90`），两态闭式与三态状态数在 `:91-100`（`FULL_STATES_THROUGH_THE_UNMOUNT = 1_662_648_564`，`:100`）；另有 `ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish`（到 E 之后不卸载、再覆盖写两次）。

### i3 末尾加一段 unlink 要碰的文件（推的）

| 文件 | 改什么 |
|---|---|
| `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` | 加 `unlink_step`（`recorder.step(file!(), "unlink", …)`）；分批时每批一次发布，段名自动成 `unlink.index_node+packed_records.1`、`unlink.journal_record.1`、`unlink.root_record_fua.1`、`unlink.system_configuration_slot.1`、…`.2`（同一入口同一种 period 递增）。若每批是一次单独的入口调用，就是同名的几段、树上位置不同（`crash_amplification.rs:2264-2265` 注释「同一个入口调几次就是几段路径，名字相同、位置不同」） |
| `crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs`（或新开一条流的测试文件） | 在 E（`:140` 起）之后、`unmount_step`（`:153`）之前或之后调它；改文件头的段数 / 写数说明（`:2-5`）、点名整表（`:234-312`）与 `:314` 的说明、`versions` 表（每批一版） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 若层 0 全量也要带 unlink：`Script` 加成员（`:71-87`）、流名与闭式状态数（`:90-100`） |
| `crates/singlefs-harness/tests/common/mod.rs` | 被上面几条经 `#[path]` 引（through_unmount `:10`）；若要一个「删后」的起点池，在这里加构造 |
| `crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `period_of_write` | 只在删除写出新的单元类或新的写种类时要动；落在码 1/2/3、journal、根槽、系统配置之内就不动 |

里程碑表（`.claude/kb/milestone/03-third-txn.md` 三·4）写的另一条「新流 mkfs → A → unlink → 再建第一个文件版本」今天走不通第二步：`publish_first_file` 要求那一版树表 0 条（`transaction.rs:4642` 的 `FirstFileVersionOnAVersionThatAlreadyHasAFile`），删后的一版树表不是 0 条（推的）；要「再建」得走 `publish_new_inodes` 加写数据，而今天 `publish_new_inodes` 不写数据、写数据只认 inode 1（上面「最要紧的几条」第 1 条）。

## 什么现象会推翻上面的结论

- 「只有一个带数据的文件」：crates/ 里出现给 inode 1 以外的号写 extent 的发布入口，或 `extents_of_the_first_file`（`recovery.rs:3347`）不再要求上段恰好一条。
- 「mkfs 不建树」：`make_filesystem.rs:457-469` 的条目参数不再是 `&[]`。
- 「nlink 恒 1、不读」：`records.rs:56` 或 `:85` 改了，或 `InodeRecord` 加了字段。
- 「待删占用恒 0、checker 不读」：`admission.rs:151-152` 不再是 `ZERO`，或 checker 里出现标签 4 的读法。
- 「崩溃点名字按写种类自动生成」：`crash_amplification.rs:2389` 的 `format!` 改了。
- 这份表按 HEAD `fcc4d340` 与 2026-09-30 的工作区取；本机别的会话在同一个仓里干活，之后 crates/ 有改动的，行号以现查为准。

## 没做什么

- 没改仓内任何文件；没跑 cargo build / test，没跑门禁；没提交。
- 「推的」各格没有用代码验证：删后恢复会不会拒（第 1 条）、续做挂点、待删占用的双扣、unlink 段名的具体样子，都只按读代码推。
- (e) 里碎片段数、全空聚簇段数两行的 checker 判法没核。
- (g) 里 checker 档测试与 `e158_root_choice_repair.rs` 对 `HistoryOperation` 的 `match` 是不是穷举没逐个看，只按 grep 命中列出。
- (a) 的「加一棵树要碰的地方」按今天引用树常量与种类码的文件列（`grep -rln 'TREE_IDENTIFIER_\|TREE_KIND_DEADLIST\|TREE_KIND_LIVELIST' crates/ --include=*.rs` 列出 19 个文件），其中 `crates/singlefs-harness/tests` 下 10 个（同一条 grep 限到那个目录再 `| wc -l`），没逐个看改不改。
- 草稿目录 `/tmp/claude-1000/m3-r1/delete-facts/` 里留着 grep 原样输出（`grep-no-delete.txt`、`grep-no-delete-2.txt`）与三份摘录（`treeid.txt`、`tx-fns.txt`、`mount-main.txt`），都是纯文本，不是编译目录或仓副本。
