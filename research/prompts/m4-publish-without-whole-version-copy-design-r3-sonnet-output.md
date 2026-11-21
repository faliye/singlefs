# m4-publish-without-whole-version-copy-design-r3 正推腿报告

日期：2026-11-18。设计推论轮第三轮（最终轮），正推腿（Sonnet）。方案按 `crates/` 今天的实现来谈，没改主工作区的 `crates/`，没动补丁与三份实现员报告（只读）。没读禁读清单里的文件。

标记：【读】读到的（文件与行，此刻现读，行号是主工作区此刻的树，`transaction.rs`、`allocator.rs`、`allocation_record_tree.rs`、`code_two_tree.rs` 与 HEAD 相同，`git diff --stat HEAD` 对这四份没有输出）；【量】命令此刻跑出来的数或产物里的数；【推】推的，没量过。补丁里的行号是 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch` 的行（下称补丁；两批合在一份）。

没写模型：本腿的数全是读码、读产物与按字段算出来的，没有要复跑的脚本，所以没有模型目录与 `rerun.sh`。要主 agent 跑的只有一条量测命令（末尾「要主 agent 跑的命令」）。

## 先给读码得到的四个前提（后面各格都用）

1. 甲（前两轮站住的形态）在判决里只换三张表（`units`、映射条目、分配记录）。`TransactionOutput` 的字段不止这三张：`data_pointers`、`files_other_than_the_first` 里各文件的数据指针、`central_mapping_tree.shape`（每片叶带全部 key）、`allocation_record_tree.nodes`、`extent_tree` 等也随池涨，三张表换完它们仍整份拷（问题 3 列全）。所以「甲之后版本克隆是 O(1)」只在这些字段也换成共享之后才成立。
2. 版本今天已经是 `Arc` 包着的值：「WithFile(Arc<TransactionOutput>),」（`crates/singlefs-core/src/transaction.rs:2560`），留着旧版的地方是 `Vec<Arc<TransactionOutput>>`（`crates/singlefs-core/src/mounted_session.rs:2040`、`crates/singlefs-core/src/mount.rs:8376`）。
3. 文件读路径只拿根记录，不读版本里的表：「self.current.root(),」（`crates/singlefs-core/src/mounted_session.rs:2899`）。读者不钉内存里的表，钉的是盘上的单元（问题 4）。
4. 分配记录条数 R 随 P 涨得有限（有上界、造池里很早就近饱和），U（单元数）、M（映射条目数）、P（数据单元数）随 P 线性涨：分配记录罩住的槽互不相交（I-5.4，`crates/singlefs-checker/invariants/07-分配器.md:9`），每块盘上每条记录至少罩一个槽。E202 三档每块盘槽数 146432、262144、524288（【读】`research/prompts/e202-preregistration.md:458`，「数据量 P 36608、65536、131072」同一行）。造池探针只量了 P=4096，R 在那一档已到 66342（【量】任务 19957 日志第 26 行「allocator_records_max=66342」，第 21 行「free_map_slots=292864」是两块盘合计的槽数，见问题 5）。这条在「分步先换哪张表」里翻面（问题 5）。

## 问题 1：与不动形态两批（补丁）并不并得住

补丁的结构，逐样对着甲。「甲之后」一列按「分配器的记录表与版本的分配记录是同一张持久表」的甲来读（前两轮判决与正推第一轮 3.3 步 3 同）。

| 补丁里加的 | 补丁位置 | 甲之后 | 判 |
|---|---|---|---|
| 分配器按盘与槽的索引：记录按记下先后的 `Vec`、序号表、(盘, 起点槽)→序号的 `BTreeMap`、已回收集合 | 「struct AllocationRecordsIndexedBySlot {」（补丁:740）、「recording_sequence_of_each_slot: BTreeMap<(DeviceIdentity, SlotNumber), u64>,」（补丁:746），加行 505（补丁 678–1268 的加行数，【量】`sed -n 678,1268p … \| grep -c '^+[^+]'`；按补丁块的界数，不是逐函数） | 持久表按 (盘, 槽) 有序，表本身就是索引；`Vec` 加序号表加 `BTreeMap` 三份结构不要。`records()` 的次序从「记下的先后」变成 (盘, 槽) 升序 | 结构白做，接口要重写：`insert_the_record_at`、`remove_the_record`、改写守卫（`AllocationRecordBeingRewritten`，写回时断言 (盘, 槽) 没变）、`changes_made`、`keys_changed_while_noted` 这几个接口与它们的断言搬到新表的外壳上 |
| 待回收集合：已释放未回收的记录按 (释放代, 盘, 槽) | 「released_and_not_reclaimed: BTreeSet<(CheckpointTxg, DeviceIdentity, SlotNumber)>,」（补丁:751） | 它是记录表的第二索引。甲不碰它，分配器克隆要 O(1) 就得让它也共享；它与 `reclaimed`（今天 `allocator.rs:2296`）同属问题 3 的「剩下随池涨的集合」 | 要重写（换持久集合或 `Arc` 分块）；它存在的理由（回收只取释放代不大于门槛的那一截、不走整表）在主工作区今天是缺的：`reclaim_released_records_up_to` 整表 `.iter()` 过滤（`crates/singlefs-core/src/allocator.rs:3327`）；甲之后回收仍要它，所以这一样不是白做 |
| 演练做完撤回的撤销步 | 记录一侧「struct UndoOfTheRecordsRehearsal {」（补丁:762）；空闲图一侧「struct DeviceFreeMapBeforeAChange {」（补丁:370），加行 228；分配器自己那几样「struct UndoOfTheAllocatorRehearsal {」（补丁:1311） | 记录一侧：演练等于「存下旧根、改、放回旧根」，撤销步整套不要。空闲图一侧：甲不碰空闲图（四张逐槽 `Vec<bool>`：「allocated: Vec<bool>,」`allocator.rs:439`、`isolated` 462、`held_until_floor_takes_effect` 470、`is_under_an_unreleased_record` 488，另有逐段计数），演练仍要么整份拷空闲图、要么用撤销步 | 记录一侧白做；空闲图一侧甲之后仍有用，直到空闲图也换成共享块（分步里单独一步）才白做 |
| 上一版按叶分组跨发布带着走的缓存 | 「struct LeavesOfTheLastPublishedVersion {」（补丁:1296）、「pub struct AllocationRecordLeavesOfThePublishedVersion {」（补丁:309）；每片叶的记录是 `Arc<Vec<_>>` | 分配记录表按叶分块（块界取分配记录树的叶界）之后，「上一版第 k 片叶的记录」就是上一版表的第 k 块，不需要另分组、另缓存、另对账（改动计数、从头分组的回退路径） | 簿记白做；「每片叶一个 `Arc` 块」的块形状正是甲的表要的形状，是甲的原型，留用 |
| 中央映射树只按改到的 key 规划 | 「pub fn plan_the_tree_from_the_changed_keys<'previous>(」（补丁:3263） | 规划器的输入契约是「要删的 key 集、要插的 key 集」，正是映射条目持久表按改到的 key 给得出的；甲不替代它 | 正好是甲的一部分，留。它没做完的两样（`into_plan` 摊平成整棵、上一版 key 逐个走一遍）要靠甲的步 5 去掉（问题 3） |
| 结算拆成随轮变与不随轮变两半 | 「struct CentralMappingKeysApartFromTheAllocationRecordTree<'previous> {」（补丁:4836）、`resolve_apart_from_the_allocation_record_tree`（补丁:4536） | 拆法留；不随轮变那一半里的三处整版遍历（收全版映射 key 进 `BTreeSet`、走上一版中央映射树全部 key、普通分配那一半）要改成吃持久表的差异 | 骨架留，里面三处要重写（补丁作者自己列的：`result-3.md` 第 283–287 行，补丁树的 `transaction.rs:8949`、`9710`、`10012`、`8966`） |

小结（【量】加行数按补丁块界；【推】归类）：补丁在 `allocator.rs` 里加的生产代码约 1070 行（空闲图撤销 228、记录索引 505、分配器撤销与缓存 339），甲之后被取代的是记录索引 505 行的结构、记录一侧撤销步与缓存的簿记；空闲图一侧撤销 228 行留到空闲图换共享块那一步；测试 622 行（补丁 1802–2451 的加行数）里对着记录索引内部的几条要重写。其余（`code_two_tree.rs` 加行 1128 含测试、结算两半、`transaction.rs` 加行 3008）基本留。

### 合入次序

- **推荐：补丁先合，甲接在后面**（【推】）。理由三条：补丁已在副本验过（新旧对拍、60 条核心单测、验证包点名目标、build 退 0；背景材料「实现今天的样子」），甲还没有一行代码；补丁里对拍用的「改前写法留作参照」（`code_two_tree.rs` 的 `planning_every_node_as_the_reference`，见 `result-3.md` 第 12 行；`transaction.rs` 的 `comparison_with_the_reference_in_tests`，见 `result.md` 第 58 行）正是甲每一步要的旧新对拍预言机；补丁块 678–1268 与甲的步 3 改同一处 `allocator.rs`，两条线并行写同一份文件的记录部分会撞。
- **做两遍的量**（【推】）：约 500 行记录索引加 339 行里的记录一侧与缓存，加对应测试几百行；补丁第三条（发布之前整份克隆分配器）本来就停了（`result-3.md` 第 277 行起），所以甲不推翻补丁里任何已验的结论。
- 若先甲后补丁：补丁要对着甲重新写第 1、2、4 样，已验的状态作废，要重走代码三方；不推荐。若合入一份删掉第 1、2、4 样的瘦补丁：那是没验过的新组合，同样要重走。
- 什么现象会推翻「补丁先」：补丁的代码轮里被要求大改记录索引（比如索引整块不许进核心层）；或用户定造池提速整批挪到里程碑八（见问题 5 交用户清单第 3 条），补丁不合、甲也不做。

## 问题 2：冻结重发

### 今天的两份分配器【读】

- 发布前那份：「let allocator_before_this_publish = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:13213`，树表 0 条那一版写行的发布同写法在 `transaction.rs:1930`）。它在「冻结着一次没重发的发布就拒」那一道之后拷（`transaction.rs:13211`），所以它的 `frozen_publish` 一定是空。
- 发布后那份：同一个 `allocator`，经 `prepare_the_version_publish_relocating_units` 原地改完（释放、取落点、记根）。
- 落盘失败那一支：「let allocator_after_the_publish =」（`transaction.rs:1662`）接「std::mem::replace(allocator, allocator_before_this_publish);」（`transaction.rs:1663`）：调用方手里的分配器换回发布前那份，改过的那份（发布后）挪进 `FrozenPublish`（「allocator_after_the_publish: PoolAllocator,」`transaction.rs:1583`），连同这次发布的版本。重发成功时「*allocator = allocator_after_the_publish;」（`transaction.rs:1705`）。准备阶段被拒与落盘前检查被拒两支只换回发布前那份：「*allocator = allocator_before_this_publish;」（`transaction.rs:1936`、`13230`、`13235`），发布后那份丢掉。

### 甲下

- 「发布前」是对 `PoolAllocator` 一次克隆得到的值，「发布后」是原地改过的那一份；两份共享没改的块，两份同时存在。落盘失败那一支拿的仍是「发布后」那份冻结、「发布前」那份还给调用方，这三行一个字不用改。
- 「克隆是 O(1)」要对 `PoolAllocator` 的每个字段成立（字段表【读】`crates/singlefs-core/src/allocator.rs:2284`–`:2348`）：`devices`（空闲图，四张逐槽 `Vec<bool>`）、「cluster_segments: BTreeSet<SlotNumber>,」（`allocator.rs:2293`）、「records: Vec<AllocationRecord>,」（`allocator.rs:2294`）、「reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,」（`allocator.rs:2296`）、`allocation_record_tree_of_the_version_without_file`（里面带一整份记录：「pub records: Vec<AllocationRecord>,」`allocator.rs:1602`）、`root_ring`（被抛弃根的落点整份）、`compaction`。只把记录表换持久（甲的三表），克隆仍随空闲图（每档常数：两块盘合计 292864 槽，【量】任务 19957 日志第 21 行「free_map_slots=292864」，每槽四个 `bool` 约 1.2 MB，【推】）和上面那几个集合涨。要「发布前」是常数代价，这些字段都得换（分步的步 6）。
- 有一个字段是故意不共享的：「other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,」（`allocator.rs:2347`，文档串 2345–2346 写「各份拷贝共用」）。甲下「克隆后在拷贝上改、原件逐字段不变」的测试构建检查（攻方第二轮提的）要把它排除。
- 能加的 O(1) 一致性检查【推】：发布出口的 `version.allocation_records` 与 `allocator` 的记录表同一个根（`Arc::ptr_eq`），冻结时与重发前各断言一次；今天只能整表比（O(R)）。

### D16（发布语义）已定项 17 的字面要不要动：不动

- 17 的定案是对调用方的承诺：「fsync 报错之后，这次发布的数据**可以出现也可以不出现**」（`.claude/kb/decisions/16-发布语义.md:385`）、「**报错之后不许假设可以照旧复用同一份内存态重试。**」（`.claude/kb/decisions/16-发布语义.md:386`）。四句里没有一句点名分配器的表示、克隆还是撤销；甲只换内存里的表示。
- 落盘失败后冻结、重发的机制在 D23（journal 的角色与格式）已定项 14「这一版的失败处置」（`.claude/kb/decisions/23-journal的角色与格式.md:409`，转述：发布不接受失败，失败的冻结，下一次发布之前逐字节原样重发），不在 17。
- 17 的「欠」行写「`C381`（根已落盘之后发布失败，分配器仍退回）（改法落地之前这一格还开着）」（`.claude/kb/decisions/16-发布语义.md:402`）。今天的代码在失败时把调用方的分配器换回发布前（`transaction.rs:1663`），与 17「报错不等于没成立」之间的缝是这条欠账，甲不补它也不加深它；C381 的修法若要在「读现行根」之后二选一（发布前或发布后那份），两份值在甲下都现成，比撤销日志那种只给得出「发布前」的写法好。
- 2026-11-06 那一轮 Q2.3 ③（落盘失败那一支要发布之后那份）：补丁第三条停下的理由（`result-3.md` 第 277–282 行）是撤销账罩不全真发改到的字段、且不嵌套；甲下发布前后是两个值，不需要撤销账，那一条难处消失，但只在分配器所有字段都共享之后。
- 什么现象会推翻「不动」：C381 的修法改成「报错之后分配器取发布后那份」或「卸载重挂」，那样 17 的第二句要改写；或甲的实现里某个字段克隆后仍共享可变内容（上面 `Arc<Mutex>` 那一类）而冻结发布后那份读到发布前的改动。

## 问题 3：甲之后一次发布里还剩哪几处随全版涨

前提：甲 = 三张表换持久表；补丁已合（补丁处理了的写「补丁：…」）。行号是主工作区此刻的树（补丁没进）。N 栏是随谁涨：R 分配记录数、U 单元数、M 映射条目数、P 数据单元数、I inode 数、C 墓碑 / 谱系 / 退役容器数、S 活快照数。

| 处 | 位置 | 做什么 | 涨 | 三表换完 | 补丁 |
|---|---|---|---|---|---|
| 映射条目装配 | 「let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =」（`transaction.rs:16448`）至 `:16641` | 逐个走全部数据指针（16449）、其他文件（16463）、extent 节点（16477）、inode 叶容器（16518）、分配记录树节点（16530）、记账与 deadlist 节点（16537）、删除三树（16565） | P、U | 仍整版走；持久表要的是「这次的编辑」，不是「全版装好再灌表」 | 没碰 |
| 映射 key 与条目 | 「let mapping_keys: Vec<Vec<u8>> = mapping_entries_by_key」（`transaction.rs:16666`）、`mapping_entries_by_key` 的 `BTreeMap`（16651–16665，每条 `build_mapping_entry`）、`mapped_units` 的 `Vec`（16642） | 每次发布从头建 | M | 换成按改到的项改持久表 | 没碰 |
| 建树函数 | `build_multi_level_tree`（`transaction.rs:17796`）：「let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();」（17811）、全节点循环（17828）、「shape: plan.shape.clone(),」（17915） | 对每个节点判照抄或重写，`shape` 整份深拷（每把 key 是两个 `Vec`，`code_two_tree.rs:77`） | M（中央映射树）；记账、deadlist 树小 | 不在三表内，仍整树 | 规划器做了一半（`into_plan` 摊整棵），没碰这个函数 |
| `units` 装配 | 「let mut units: Vec<PublishedUnit> = Vec::new();」（`transaction.rs:17032`）起：数据单元循环（17033）、其他文件「for file_plan in &resolved.files_other_than_the_first.files {」（17048）、各族 | 每项 `carried_unit` 去上一版索引取（`transaction.rs:14297`） | U | 仍逐项走；索引不再要整建 | 没碰 |
| 按角色索引整建 | 「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」（`transaction.rs:16089`）、「let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, …」（17458）、「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」（17619，两次整扫） | 每次发布整建两份索引加两次整扫 | U | 表自己是索引，这三处删 | 第一批让索引各轮共用，仍每次发布建 |
| 文件数据指针 | 「data_pointers: carried.data_pointers[..carried_data_units].to_vec(),」（`transaction.rs:15668`）、「let mut file_data_pointers: Vec<DataPointer> = files_of_the_previous_version」（16105）至 16153 | 照抄的文件把整串数据指针再拷一遍（`DataPointer` 64 字节【推】：头 16、两条位置项各 16、写序 16） | P | **不在甲的三表里**；`TransactionOutput` 的 `data_pointers`（`transaction.rs:3391`）与各文件那串仍整份拷 | 没碰 |
| 分配记录出口 | 「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」（`transaction.rs:16232`）、排序（16233）、恒开断言「nodes_whose_contents_changed(」（16245）与「&& nodes_holding_records(&allocation_geometry, &allocation_records)」（16251） | 整表拷出、整表排序、断言比全表 | R | 版本表直接取分配器表的根，三处消失 | 断言改成只重比改过的叶；整表拷与排序没动 |
| 分配记录树 | `build_allocation_record_tree`（`transaction.rs:14791`）、「previous_version.allocation_record_tree.node_set(),」（14022）、`pointer_of` 线性找 | 装树、取上一版节点集 | R ÷ 叶容量（节点数）；装树 R | 装树改按叶取块；节点集仍随节点数 | 装重写的叶只取那一段记录 |
| 结算每轮 | `settle_the_allocation_record_tree`（`transaction.rs:13994`）：「plan.resolve(」（14041）、「let mut rehearsal = allocator.clone();」（14076）、`nodes_whose_contents_changed`（14085、14105） | 每轮整版 `resolve`、整份克隆分配器、按全表比叶 | U、R、空闲图 | 克隆变共享；`resolve` 的整版遍历要改成吃差异 | 拆两半；整版遍历在不随轮变那一半里留了三处（`result-3.md` 第 283–287 行）；克隆换撤销步 |
| 中央映射树规划 | 「let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units」（`transaction.rs:9594`）、`plan_the_tree_after_this_publish`（`code_two_tree.rs:637`） | 收全部映射 key 进有序集、整棵规划 | M | 输入改成持久表的差异 | 规划器只碰改到的 key；上一版 key 逐个走一遍与 `into_plan` 摊整棵仍在 |
| 发布前克隆 | 「let allocator_before_this_publish = allocator.clone();」（`transaction.rs:13213`、`1930`） | 整份克隆分配器 | 空闲图（每档常数）、R、集合 | 只换记录表：仍随空闲图与集合；全字段共享才变小 | 没碰（第三条停下） |
| 带搬迁发布整版拷 | 「let mut relocated_version = previous_version.clone();」（`transaction.rs:15822`） | 把整个 `TransactionOutput` 深拷一份再改位置 | 所有字段 | 仍深拷，除非每个字段都换共享（含 `data_pointers`、各树 `shape`） | 没碰 |
| 会话层预演拷版本 | `mounted_session.rs:1866`、`:2362`、`:5371`，`transaction.rs:12791`，`crates/singlefs-core/src/mount.rs:5810` | 预演、整理每一步入口、抬 F 预演各拷一整版 | 所有字段 | 同上；每次事件一次，不是每次发布一次 | 没碰 |
| 分配器线性找与回收 | 「…record.device == device && record.slot == slot)」（`allocator.rs:2756`，`record_for` 的线性 `find`）、`release_*` 与 `resurrect` 的 `.iter_mut().find`（2790 起）、`reclaim_released_records_up_to`（`allocator.rs:3327`，整表过滤）、`make_room_for_record_on_device` 的 `retain`（1586） | 每次按 (盘, 槽) 线性找，每次回收整表过滤 | R | 持久表按键取、按键范围取 | 索引化；`Vec::remove` 搬动与 `records()` 切片接口留着（`result.md` 第 173 行） |
| 落盘 | `persist_publish_writes`（`transaction.rs:1538`–`:1564`） | 只遍历 `writes.units`（改到的单元）、journal 记录、根槽 | 读码不随全版涨 | 不变 | — |

### 落盘那一格：读码与产物不一致，原因推不出

- 读码：落盘只走改到的单元，不读版本里的表。
- 产物：任务 19899 的计时树（P=4096，交错写段，每段 528 次发布）里 `publish.persist` 每次发布最早八分之一 0.348 毫秒（`section-timing-after-the-second-speedup-task-19899.txt:760`，0.183629 秒 ÷ 528）、最晚八分之一 2.975 毫秒（同文件第 872 行，1.570764 秒 ÷ 528），涨 8.6 倍；同两个窗口里每次发布的记录条数 25376→62870（`publish.allocator_clone_before` 的 items，第 758、870 行）、单元数 2554→6447（`admitted.units_assembly` 的 items，第 787、899 行）只涨 2.5 倍左右。
- 【推】设备端块存储第一次触碰的缺页，不是版本表（2026-11-06 那一轮量过缺页是建池挂钟的大头，`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md` 第二节最后一条）；缺：`publish.persist` 内部再分段。本机负载高，只看趋势。

### 补丁合入之后各段每次发布多少毫秒（【量】，P=4096，交错写段，任务 19899 的计时树）

| 段 | 最早八分之一 | 最晚八分之一 | 条数涨 | 毫秒涨 | 出处行 |
|---|---|---|---|---|---|
| `publish.allocator_clone_before` | 0.279 | 2.691 | 2.5 倍 | 9.6 倍 | 758、870 |
| `admitted.allocation_record_tree` | 0.803 | 2.627 | 2.5 倍 | 3.3 倍 | 783、895 |
| `admitted.allocation_records_sorted_copy` | 0.585 | 2.012 | 2.5 倍 | 3.4 倍 | 801、913 |
| `settle.resolve_apart_from_the_allocation_record_tree` | 0.571 | 3.301 | 2.5 倍 | 5.8 倍 | 802、914 |
| `admitted.central_mapping` | 0.392 | 2.212 | 2.5 倍 | 5.6 倍 | 785、897 |
| `admitted.units_assembly` | 0.129 | 1.370 | 2.5 倍 | 10.7 倍 | 787、899 |
| `publish.total` | 3.789 | 20.264 | — | 5.3 倍 | 757、869 |

毫秒涨得比条数多：每条的代价自己也在涨（缓存、分配器），所以按条数线性外推会低估。嵌套关系（哪段包着哪段）没逐层核，不加总。

## 问题 4：内存与旧版的释放

### 今天一版占多少（【推】，按字段布局算，没量 `size_of`）

每项字节（`PublishedUnit`：「pub struct PublishedUnit {」`transaction.rs:3358`，`slot` 8、`TransactionUnit` 带两个 `u64` 的成员取 24、`Arc<[u8]>` 胖指针 16，共 48；`mapped_units` 一项 `(TransactionUnit, Vec<u8>)` 48 加 27 字节 key 的堆块约 32，共 80；`mapping_keys` 一项约 56；中央映射树 `shape` 里一把 `CodeTwoTreeKey` 两个 `Vec`（「pub struct CodeTwoTreeKey {」`code_two_tree.rs:76`）约 120；`DataPointer` 64（`PointerHead` 16、两条位置项各 16、写序 16，`crates/singlefs-core/src/pointer.rs:828`、`:200`、`:35`）；`AllocationRecord` 24（`crates/singlefs-core/src/allocator.rs:72`））。

每个数据单元（P）对应的字节：单元数 U 在造池末了约是 P 的 1.64 倍（【量】`units_in_the_version=6724`，P=4096，任务 19949 产物 `publish-change-probe-2288MiB-P4096.out` 第 6681 行）。U 项 48 + 80 + 56 + 120 约 304 字节，乘 1.64 得 500 字节，加数据指针 64，共约 570 字节每个 P。分配记录 R 另算，R 有上界（前提 4）：每块盘每条记录至少罩一个槽、数据单元罩两个槽。

| 档 | P | 一版里 P 相关的表（推） | 分配记录（R×24，R 的界按两块盘各槽数、两槽一条到一槽一条） | 一版合计（推） |
|---|---|---|---|---|
| 单元区 2288 MiB | 36608 | 约 21 MB | 3.5 到 7 MB（R ≤ 146432 到 292864） | 约 25 到 28 MB |
| 4 GiB | 65536 | 约 37 MB | 6.3 到 12.6 MB | 约 44 到 50 MB |
| 8 GiB | 131072 | 约 75 MB | 12.6 到 25 MB | 约 88 到 100 MB |

- 对照：P=4096 那档整个进程常驻 1086238720 字节，其中设备块存储已存 700973056 字节（【量】同一产物第 6681 行 `resident_bytes=1086238720`、`chunk_bytes_stored=700973056`）；按上表 P=4096 时一版约 2.3 MB 加 R 约 1.6 MB，约占常驻的 0.4%（推）。登记里最大档内存「约 24 GiB」是推的（`research/prompts/e202-preregistration.md:458`），一版约占 0.4%。**版本表在 E202 里不是内存的大头，设备块存储才是。**
- 发布峰值今天：上一版（借用，`previous: Option<&TransactionOutput>`）加这次新装的一整版加发布前整份分配器克隆，再加结算每轮的克隆（补丁合入后每轮改撤销步）。多出一整版约 25 到 100 MB，随档涨。
- 补丁合入后分配器每份克隆更大（补丁作者自己写了「R1 这一份克隆会比改前贵」，`result.md` 第 18 行）：每条记录多一份 `BTreeMap` 项与两个集合项，约 150 字节每条（推），R 在 7 万到 100 万之间时一份克隆 10 到 150 MB。

### 甲下常驻怎么变

- 只留当前一版（V=1）：同一份表内容加块头、脊与 `Arc` 计数，多百分之几（推）；不比今天少。
- 留着上一版与这一版共用块：多出来的只是这一版独有的块加一条脊。量的依据是跑前写死的复制比：按 64 条一块，交错写段每次发布要复制的比例中位，分配记录 0.025178、映射条目 0.074861、`units` 0.073899（【量】任务 19957 日志第 17–19 行）；按 256 条一块（与一片分配记录树叶里的记录数同量级：叶至多装 812 条，P=4096 时每片叶约 183 条，推）分配记录 0.032199、映射条目 0.161365、`units` 0.164920（【量】`publish-change-probe-2288MiB-P4096.out` 第 6396–6398 行 `copy256_over_n_median`，判决表没报这一列）。留着多一版约等于多一份「复制比 × 一整张表」，P 涨时 d 不涨，比例在更大的档上更小（推，没量 P>4096）。
- 留 V 版：今天 V 份整版，甲下一份加 V 份独有块，上界是每张表一整份（独有块不会比整张表多）。
- 乙强与丙强的内存读法同第一轮（零轮，没模型），这一格不判。

### 读写并发：读者持着的旧版什么时候放

- 今天（串行）：会话读文件只拿根记录（`self.current.root(),` `mounted_session.rs:2899`），读者不持有版本里的表。持有旧版的地方是 `versions_published`（`mounted_session.rs:2040`）、`steps_published`（`crates/singlefs-core/src/mount.rs:8376`）与冻结的发布（`FrozenPublish.version`，`transaction.rs:1580`）：它们放手的时刻由调用方定。
- 甲下节点按 `Arc` 计数，最后一个持有者丢掉时才放，只放那一版独有的块；没有周期、没有 epoch。析构不递归过深：脊加块两层（推，攻方第二轮同）。
- 里程碑六读写并发（`.claude/kb/milestone/06-sixth-txn.md:78`「| 二·1（六·10） | Q10 读写并发 | 先改设备所有权…」）：条款只写了先改设备所有权，读者拿哪一版、拿多久、版本放在哪一把锁后面都没写，**推不出**；缺的是六·10 的设计。能说的两条：持旧版的读者占的内存 ≤ 它持有期间各次发布独有块之和，封顶一整版；读者真正要的是旧根引用的盘上单元不被回收，那由回收下界与延迟窗口管（D16（发布语义）已定项 1），不由内存里的表管，与甲无关。
- 什么现象会推翻这一节：里程碑六让读者经版本里的表读（比如按键取单元字节），那样旧版被钉的就是表的块而不是根，持有时长要重算。

## 问题 5：分步终稿

约定：「合」指能不能单独合进主工作区，每一步都要走代码三方（`.claude/rules/implementation-workflow.md` 的三步）与 `crates/mutations.tsv` 的变异行。「不改期望」指已有测试的断言一字不改，只改取值的写法。

先排次序（【推】）：步 0 在 HEAD 上录；补丁在步 0 之后合，合入后复跑步 0（金标逐字相同，这一遍同时是补丁「字节不变」的独立证据，补丁自己的对拍全在进程内）；其余步在补丁之后。

| 步 | 改哪个文件的哪个结构 | 验收 | 合 | 与补丁 |
|---|---|---|---|---|
| 0 取证与预言机 | 验证包新增一个测试目标（名字说内容，如逐次发布摘要的场景集），读 `crates/singlefs-verification/src/memory_pool.rs:1037` 的 `PublishWrites`；不碰核心层 | 三路互不共用代码：逐次发布写进设备的 (偏移, 长度, 字节) 摘要；版本的 `{:#?}` 规范文本摘要；`recovery::rebuild_version`（`recovery.rs:8047`）从盘字节重建后三张表逐条相等。证红：把一条分配记录的 `generation` 加一，前两路摘要必须变。场景集覆盖：树分裂各层、整理搬一批、快照下覆盖写、回退、落盘失败冻结加重发；某形状 0 次就报「没覆盖」 | 能：只加测试 | 在 HEAD 上录金标，补丁合入后复跑 |
| 1 `mapping_keys` 改方法 | `transaction.rs:3396` 的字段、`:16666` 与 `:17644` 的构造、`recovery.rs:3787`、`crates/singlefs-core/src/mount.rs:10805`；验证包 9 个文件 19 行（【量】`grep -rnE '\.mapping_keys\b' crates/singlefs-verification/tests`） | 不改期望（只有取值写法变）；步 0 金标相同；`crates/singlefs-core/src` 里没有读 `TransactionOutput.mapping_keys` 字段的地方（`mounted_session.rs:342` 等是别的结构的同名字段） | 能 | 补丁后 |
| 2 分配记录包一层 | 新类型 `AllocationRecords` 包住 `Vec`，先 `Deref<Target=[AllocationRecord]>`；换 `TransactionOutput.allocation_records`（`transaction.rs:3396` 附近）、`PoolAllocator::records()`（`allocator.rs:2743`）、三个构造点（`transaction.rs:16232`、`recovery.rs:3778`、`crates/singlefs-core/src/mount.rs:10806`）；之后去掉 `Deref`，由编译错误带路把 `&[AllocationRecord]` 签名（【量】112 行 28 个文件）逐个换成表的接口，热点读者（区域扫描 `region_scan.rs`、`allocation_record_tree.rs` 的 `nodes_whose_contents_changed`:368 与 `nodes_holding_records`:338、`records_of_each_leaf`:312）换按叶与按范围的接口 | 每个小步金标相同、不改期望；记编译错误数 | 能：新类型处处在用，不是死代码 | 补丁后（补丁在 `allocator.rs` 记录一侧加了 505 行，这一步的 `Deref` 先包在补丁的外壳外面） |
| 3 分配记录表换成按叶分块 | 新文件放分配记录表（每块盘一条脊，每片叶一个 `Arc` 块，叶界取分配记录树的叶界 `leaf_of_slot`，`allocation_record_tree.rs:186`；块内按 (盘, 槽) 有序；`Arc::make_mut` 复制一块）；换掉补丁的 `AllocationRecordsIndexedBySlot`（补丁:740）与叶缓存（补丁:1296）、分配器 `records`（`allocator.rs:2294`）、`AllocationRecordTreeOfTheVersionWithoutFile.records`（`allocator.rs:1602`）；删 `transaction.rs:16232`–`:16233` 整表拷与排序；演练里记录一侧的撤销改成「存旧脊、放回旧脊」；`PartialEq` 手写：脊与块 `ptr_eq` 快路，否则按内容比 | 表自己与 `BTreeMap` 的随机对拍（含克隆后再改旧克隆不变）放在验证包（不与核心共用代码）；补丁的 `record_lookups_through_the_slot_index_match_scanning_every_record_over_a_random_history_with_reuse` 与新旧对拍测试改成对着新表（参照写法留）；新增变异行：跳过块复制原地改、叶界差一、`ptr_eq` 快路不看内容、克隆后改动漏复制；已有 60 条核心单测与验证包点名目标不改期望，**至少要改参照次序的一条**是补丁第一批第 5 条（它比「记录表连先后」，`result.md` 第 56 行）：甲之后 `records()` 按 (盘, 槽) 升序；补丁其余几条对拍有没有靠记下先后，步 3 编译后逐条看（推不出）。已有测试里 `assert_eq!(…records()…)` 命中 6 处（【量】`grep -rnE` 全 `crates`），列出的都是拿 `records_before` 前后比，不依赖次序 | 能（与步 2 之后） | 补丁后；先补丁再做，补丁里 505 行索引与记录一侧撤销、缓存的簿记被取代 |
| 4 `units` 与映射条目 | `TransactionOutput.units`（`transaction.rs:3387` 附近）、`mapped_units`（`transaction.rs:3459`）包成 `UnitsOfAVersion`、`MappedUnitsOfAVersion`，先 `Deref`，再换成按 `UnitRank`（复刻今天 `units` 次序，不能直接用 `TransactionUnit` 的派生次序，正推第一轮已读出）有序的持久表；删三个访问器（`transaction.rs:4539`–`:4557`）与每次发布整建的索引（`:16089`、`:17458`）；`ContainerTreeRootUnitPositions`（`transaction.rs:3489` 起，装的地方 `:17619`、`crates/singlefs-core/src/mount.rs:10794`、`recovery.rs:3770`）改成版本里各留两个根单元（O(1)），删 `container_tree_root_units_read_at_their_recorded_positions.rs` 与它在 `crates/mutations.tsv` 里的 4 行，「准入不逐项找」改用 `lookup_work` 计数守 | 步 0 的断言先行：`UnitRank` 在全部现有测试里严格递增、同一角色至多一项（今天 `FirstEntryOfEachIdentity` 取最先一项，`transaction.rs:3853`）；金标相同；验证包里下标、改写、次序、交切片 17 行要改取值写法（攻方第二轮分类，【转述】）不改期望；相等 4 行手写按内容比 | 能，但只换表示、发布里仍整版走（问题 3）：单独合不带来每次发布的收益，只带来克隆变便宜 | 补丁后 |
| 5 装配只交这次的编辑（分三小步：映射树、分配记录树版本、文件数据指针，各自能合） | 映射树小步：`CodeTwoTreeShape` 的节点换 `Arc`（`code_two_tree.rs:197`），`build_multi_level_tree`（`transaction.rs:17796`）只循环重写与改到的节点，`planned_keys` 断言限到改到的叶，接补丁的规划器；分配记录树版本小步：`AllocationRecordTreeVersion.nodes` 共享、`pointer_of` 按键取；文件数据指针小步：`TransactionOutput.data_pointers` 与各文件那串换持久的按位置序列，`carried_file_version_units`（`transaction.rs:15651`）与 `:16105` 起只改改到的位置；再把 `mapped_units_with_locations`（`transaction.rs:16448`）的整版装配拆成「按树发编辑」 | 每小步：金标相同；改前整版写法留在 `#[cfg(test)]` 当参照（补丁同做法），随机历史逐发布比；新增变异行；不改期望 | 每小步能；文件数据指针小步在甲的三表之外（问题 3） | 映射树小步依赖补丁的规划器；另两小步不依赖补丁 |
| 6 分配器其余字段共享 | 空闲图四张逐槽表与逐段计数换块共享（块 4096 槽，【量】`free_map_blocks4096_touched_max=8`、`free_map_slots_changed_median=112`，任务 19957 日志第 21 行）、`cluster_segments`、`reclaimed`、待回收集合、`root_ring` 里被抛弃根的落点；之后删补丁里空闲图一侧撤销 228 行与 `rehearse_and_undo`，演练改回「克隆、改、丢」 | 新增「克隆后在拷贝上改、原件逐字段不变」测试构建检查（排除 `allocator.rs:2347` 那个故意共享的字段）；补丁的演练对拍改作参照；`publish.allocator_clone_before` 的毫秒与条数脱钩（量） | 能 | 补丁后，步 3 之后 |
| 7 收尾 | 去 `Deref` 过渡、影子比对只留测试构建 | 全部已有测试不改期望 | 能 | — |

### 验收口径补充

- 「不改期望的已有测试」，甲的每一步都要跑的集合（点名，不跑整包）：补丁副本里那几组（核心 lib 里分配器与分配记录树的 60 条、`code_two_tree` 的 8 条、验证包的 `overwrite_in_one_instance` 13 条、`admission_checkpoint_cost_per_device_paths` 4 条、`allocation_record_tree_settling_walks_the_release_chain_to_its_closure`、`allocation_record_tree_settling_of_a_row_publish_on_a_version_without_file`、`first_file_version_checks_the_previous_record`，名字与点名写法在 `result-3.md` 第 99–140 行），加 `assert_eq!(pool.output, current_before` 的 8 处（【量】`grep -rnE 'assert_eq!\(pool\.output, current_before' crates | wc -l` 输出 8）与崩溃枚举里读版本的几条用例（范围声明与缩域写法按共用约束，由承接的实现员写）。
- 步 3、4、5 的对拍都用「改前写法留作 `#[cfg(test)]` 参照」，随机历史逐发布比，种子打进断言消息；不用本腿或实现里的同一段代码当预言机。
- 每步新增的变异行各至少要有一条在步骤内的对拍里红（证红命令由主 agent 照 `scripts/mutation/prove-red.sh` 交）。

## 交用户的清单

每条：候选、各自的代价、翻面观测。标「零轮」的是本腿提的、没被攻过的。

1. **持久容器：自己写还是引外部包**（前两轮留下的）
   - 候选「自己写」：两个小结构。分配记录用「每盘一条脊、每叶一个 `Arc` 块」（块界取分配记录树的叶界，不用通用树）；`units` 与映射条目用按 `UnitRank` 有序的持久表（B 树或二层分块）。代价：代码量前者 150 到 250 行、后者 400 到 700 行（正推第一轮的估计，推的，没写过）；每个结构要对拍加变异行；不动依赖（`crates/singlefs-core/Cargo.toml` 的 `[dependencies]` 今天只有 `libc`、`singlefs-format`、`singlefs-replay-prefix`、`vstd`）。
   - 候选「引包」（前两轮列过 imbl、im、rpds、immutable-chunkmap）。代价：加依赖要过许可与离线构建（本机 `~/.cargo/registry/src` 里没有这几个包，【量】`ls ~/.cargo/registry/src/*/ | grep -ciE '^(im|imbl|rpds)-'` 输出 0）；通用结构的块界不按键，分配记录的「叶界」要再套一层；结构差异（找出两版哪几块不同）多数包不暴露，要靠写者自己记改到的键（补丁的 `keys_changed_while_noted`，补丁:757，是先例）。
   - 候选「混合」：分配记录自己写（结构简单、块界要按叶），`units` 与映射条目引包。
   - 翻面观测：写者记改到的键就够用（不需要结构差异）、且 64 条一块的通用结构够用（量到的复制比 0.025，问题 4）→ 引包站得住；若要叶界块（分配记录树按叶重写）→ 分配记录那张必须自己写。
2. **分步先换哪张表**（前两轮留下的）
   - 候选「分配记录先」（步 2、3，再步 4、5）。代价：改 28 个文件 112 行签名（机械）、有补丁的预言机；收益：P=4096 上分配记录一侧的段占最晚八分之一每次发布 20.264 毫秒里的 10.7 毫秒（`allocator_clone_before` 2.691、`allocation_record_tree` 2.627、`allocation_records_sorted_copy` 2.012、`previous_node_set` 3.377 之和，【量】问题 3 表；嵌套关系没逐层核，所以只当量级）。
   - 候选「单元与映射先」：`units`、映射条目与文件数据指针先（步 4、5，再步 2、3）。代价：每张表换完没有收益，要等装配改成「只交编辑」（步 5）；改动面大、风险高。收益：每个单元项的代价约是每条分配记录的 10 倍（【量】最晚八分之一：`central_mapping` 346、`units_assembly` 212、`resolve_apart` 524、`resolve_into_the_whole_version` 122，共约 1204 纳秒每项；分配记录一侧 43、42、32，共约 117 纳秒每条），而 R 随 P 涨得有限（前提 4：到槽数为止）、U 与 P 随 P 线性涨：P=36608 时 R 至多 146432 到 292864、U 约 6 万，一侧约 17 到 34 毫秒、另一侧约 72 毫秒（推，按 P=4096 的每项代价外推；每项代价在这一趟里从最早到最晚涨了 3 到 10 倍，所以只会更大）。
   - 候选「空闲图先」（步 6 提前）。代价：独立于三表；收益：空闲图每次发布只变 112 个槽、至多 8 个 4096 槽块（【量】任务 19957 日志第 21 行），但 `allocator_clone_before` 里它占多少没量。
   - 翻面观测：补丁合入后在 P=16384 与 8192 上量各段（末尾命令）：`allocation_records_in_the_version` 与 `units_in_the_version` 两列之比、逐发布毫秒的涨幅指数。每项代价之比（单元一侧对分配记录一侧）在 10 倍上下，所以 R÷U 小于 10 选「单元与映射先」，大于 10 选「分配记录先」（P=4096 时 R÷U 约 9.75，两边相当）。
3. **甲做不做、何时做**（用户 2026-11-08 的话与 E202 现状顶着）
   - 候选「立即做甲」：补丁合入后立即做甲（里程碑四收口内、E202 跑前）。代价：步 0 到 7，仅步 2 的签名就涉及 28 个文件（【量】问题 5 步 2），每步一轮代码三方。
   - 候选「先量再定」：补丁合入后先量，够 E202 三档在四小时上限内造完就把甲归里程碑八。代价：零代码，一次量测；风险：最大档跑不完。
   - 候选「不做甲」：只跑补丁够得着的档。
   - 依据：用户 2026-11-08 那句原话里的一段「后续我们有专门的里程碑来优化性能」（`.claude/kb/milestone/04-fourth-txn.md:180`，同一行写「归里程碑八」）；E202「用户定这一次不跑，等造池提速做完再跑」（`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:12`）。
   - 外推（推）：造池总挂钟 ∝ 发布次数（随 P 线性）× 每次发布代价（随全版线性），即 P 的平方。补丁合入后 P=4096 带计时 63.438 秒（背景材料，任务 19899），P=36608、65536、131072 各是它的 80、256、1024 倍：约 85 分钟、4.5 小时、18 小时。这一趟里每次发布代价的涨幅比线性更陡（3.789→20.264 毫秒，条数涨 2.5 倍），所以只会更长。最大两档在四小时上限（`.claude/agent-common.md` 的「累计满四小时服务停这一次尝试」）之外。
   - 翻面观测：补丁合入后 P=8192、16384 的 `construction_seconds`：取两点的指数 log2(t16384 ÷ t8192)。翻面线（本腿定，零轮）：指数 ≤ 1.5 → 「先量再定」站得住（最大档按 63.438 秒 × 32 的 1.5 次方约 3.2 小时，在四小时上限内）；指数 ≥ 2 → 「立即做甲」（最大档约 18 小时）。
4. **合入次序**
   - 候选：补丁先（本腿推荐，零轮）；甲先（补丁第 1、2、4 样不写，已验的状态作废）；瘦补丁（删掉第 1、2、4 样，没验过）。代价与翻面见问题 1。
5. **`records()` 的次序**
   - 候选：接受 (盘, 槽) 升序，至少改补丁第一批第 5 条对拍的参照次序（违「不改期望」的字面，要主 agent 判）；或表里另记序号保住记下先后（回到补丁的三份结构，不推荐）。
   - 翻面观测：验证包里依赖 `records()` 先后的读者数。攻方第二轮分类 145 行 `.allocation_records` 里「次序」0 行（转述，没抽查）；`.records()` 全验证包 133 行，本腿没逐行分类。
6. **准入读墓碑树、退役树两个根单元**（攻方第二轮留给主 agent 的）
   - 候选：版本里各留两个根单元，准入 O(1)；或按角色 `get`，O(log U)。仓里没判过 O(log) 算不算「随盘容量增长」（`.claude/rules/fs-design.md` 运行时决策路径那一格）。
   - 翻面观测：无要量的数，是条款解释；前者代价是版本多存两个单元引用。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 与不动形态两批并不并得住（问题 1） | 答了：与已定条款一致，规则没写合入次序 | 六样里：记录索引结构与记录一侧撤销、缓存簿记白做；待回收集合与结算不随轮半的三处遍历要重写；规划器、结算两半骨架、空闲图一侧撤销（到空闲图共享前）留用；次序：补丁先、甲后（推，零轮） |
| 冻结重发（问题 2） | 一致，D16 已定项 17 字面不动 | 发布前是克隆、发布后是原地改的那份，落盘失败那一支仍把后者冻结、前者还给调用方，三行不改；前提是 `PoolAllocator` 全字段共享（`Arc<Mutex>` 那个故意共享的除外） |
| 甲之后还剩什么随全版涨（问题 3） | 答了，列 15 处 | 三表之外还有数据指针、各树 `shape`、整版克隆；映射条目装配、建树函数、`units` 装配仍整版走；落盘读码不涨、产物涨 8.6 倍原因推不出 |
| 内存与旧版的释放（问题 4） | 答了，数是推的；并发推不出 | 一版约 25 到 100 MB（三档），约占常驻 0.4%，不是内存大头；甲下留旧版只多独有块；读写并发缺六·10 的设计 |
| 分步终稿（问题 5） | 答了 | 步 0 到 7，补丁在步 0 后、其余步前；步 2、3、4、5 各能单独合（步 4 合了不带每次发布的收益） |
| 交用户的清单 | 六条 | 容器、先换哪张、做不做与何时、合入次序、`records()` 次序、准入根单元 |

### 什么现象会推翻本腿的结论

| 结论 | 推翻它的现象 |
|---|---|
| 补丁先合 | 补丁代码轮要求大改记录索引；或用户把造池提速整批挪到里程碑八 |
| 甲之后版本克隆仍随 P 与 M 涨（三表之外的字段） | 步 5 之前就测得 `TransactionOutput::clone()` 的深拷元素数与 U、P 无关（那说明这些字段已共享，我读漏了） |
| 先换哪张表随档翻 | P=16384、8192 上量到 R 与 U 之比不降（R 没到上界）或每项代价之比不到 10 倍 |
| 17 字面不动 | C381 的修法把落盘失败后的分配器改成取发布后那份，或甲的实现里某字段克隆后仍共享可变内容 |
| 版本表不是内存大头 | 量到 `TransactionOutput` 实占（`size_of` 加堆）超过常驻的 5%；我只按字段布局推，没量 |
| 落盘涨 8.6 倍不是版本表引起的 | 给 `publish.persist` 内部分段后，涨幅落在版本相关的段上 |

## 要主 agent 跑的命令

1. 补丁合入后的造池端到端量测（外推与「先换哪张表」的翻面观测都读它；在补丁副本里编，副本是实现员交接的那一棵，`.deliverables` 写着留给主 agent）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/copy -- cargo run --release -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- construction-timing --unit-area 2288MiB --live-data-units 8192,16384 --chunks shared --product /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-sonnet/construction-timing-2288MiB-P8192-P16384.out`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/copy`（副本）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；产物里每个 P 一行 `name=construction_timing … construction_seconds=`；每段 `name=construction_phase … phase=interleaved_half … step_milliseconds_mean_of_the_earliest_eighth= … step_milliseconds_mean_of_the_latest_eighth= … units_in_the_version= … allocation_records_in_the_version=`。算：指数 log2(P=16384 的秒数 ÷ P=8192 的秒数)、两个 P 的 R 与 U 之比。预计 P=8192 约四五分钟、P=16384 约十几分钟（按 P 的平方从 63.438 秒推，推的）。范围声明闸若要求，种类写 `target`，为什么写「补丁合入后量造池随 P 的涨幅，判甲先换哪张表与做不做」。
2. 本腿没有别的命令：没写模型、没写脚本、没编译。

## 没做什么

- 不判攻方腿的格（已回收集、d 的定义、整理发布上的 d、数能不能核）；没替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑（问题 3 的毫秒表与问题 4 的复制比都是读产物文件得到的数，不是复跑）。
- 没量 `size_of`（问题 4 的每项字节按字段布局推，三档一版大小全是推的）；没量 P>4096 的任何一格；没给 `publish.persist` 分段；没抽查攻方第二轮的读者分类表。
- 没逐行读 `transaction.rs` 全部 18532 行：读了结算（13994 起）、发布出口（15681 到 17739）、冻结与重发（1566 到 1712）、建树函数（17796 起）、`carried_unit` 与数据指针拷贝（14297、15651）；`build_extent_tree`（14431）、`build_inode_tree_units`（14912）、`build_deletion_tree_units`（15312）、`resolve`（8913）内部只读了调用处与入口，所以问题 3 里这几个函数各自还剩几处整版遍历、步 5 的映射树与分配记录树两小步里每个函数要改几处，**推不出**；缺：这几个函数的逐行读。
- 没核补丁的每一行：读了补丁里与本题相关的定义（记录索引、撤销步、叶缓存、规划器入口、结算两半），没读补丁的测试与变异行；补丁在副本里的验证结果（退 0 的几条）只是背景材料与 `result-3.md` 的转述，没复跑。
- 没读禁读清单里的文件与目录；没改 `crates/`；没动补丁与三份实现员报告。
- 草稿目录 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-sonnet` 里只有认领时建的文件，没有副本与编译目录。
