# m3-b44-code-r2 正推腿报告（Sonnet，正推；J1–J5）

日期：2026-10-17。冻结副本：`~/.local/share/singlefs/work/m3-b44-code-r2/repo/`，下文简称「冻结副本」；代码行号按冻结副本。
被引的 core、harness、checker-tier 源文件与主工作区逐份相同（`cmp` 全部无差，见第六节命令），所以 `cite-check` 在仓根判也对得上。
没编译、没跑测试：本轮全是读码对条款，能 grep 的事实在文中贴了命令与输出。

## 一、J1：重建时 I-2.5 那一判并进三棵容器树的指针

### 1.1 条款与代码逐句对

| 条款一句 | 代码一处 | 判定 |
|---|---|---|
| 「任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序」（`crates/singlefs-checker/invariants/04-校验和.md:10`） | 「location_entries_ascend_by_device(&self.locations)」（`crates/singlefs-core/src/pointer.rs:319`）；严格小于判在 crates/singlefs-core/src/pointer.rs:87 的 `locations[0].device < locations[1].device` | 对得上 |
| 「两条位置条目设备身份相同或逆序，一律判损坏」（`crates/singlefs-checker/invariants/04-校验和.md:10`） | 「RecoveryFailure::InvariantViolated {」（`crates/singlefs-core/src/recovery.rs:5097`）下一行 `invariant: "I-2.5"`；重建出口「pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;」（`crates/singlefs-core/src/recovery.rs:3259`）把它按 `?` 交回 | 对得上 |
| 「非全零的指针不享这条豁免」（`crates/singlefs-checker/invariants/04-校验和.md:10`） | 「if *self == Self::empty_root() {」（`crates/singlefs-core/src/pointer.rs:316`）：整个结构（头、两条位置条目、实例、出生序号）都等于全零才豁免 | 对得上 |
| 第一轮判决 K4：「读侧该交错误，不该放进来再在写者那里 panic」（`research/prompts/m3-b44-code-r1-main-verification.md:23`） | 三棵树并进清单：「.chain(tombstone_tree_pointers)」（`crates/singlefs-core/src/recovery.rs:5194`）、「.chain(retirement_tree_pointers);」（`crates/singlefs-core/src/recovery.rs:5196`），谱系树在 5195 行；写者断言仍在「位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过」（`crates/singlefs-core/src/pointer.rs:327`） | 对得上 |

### 1.2 并进清单全没全：`TransactionOutput` 里每一类指针对一遍

`TransactionOutput` 在 `crates/singlefs-core/src/transaction.rs:2841` 起；下表逐字段列出带 `NodePointer` / `DataPointer` 的，右列是 `pointers_of_the_rebuilt_version_ascend_by_device`（`crates/singlefs-core/src/recovery.rs:5136`）里判它的那一处。

| 指针 | 在 `TransactionOutput` 的哪 | 判它的行（recovery.rs） | 进了吗 |
|---|---|---|---|
| 根记录四条（树表、实例表、中央映射树根、分配记录树根） | `root` | 5139（`root_record_pointers_ascend_by_device(&output.root)?;`，函数体 5104 起） | 进了 |
| 树表每条的根指针（含 inode 树、extent 树、deleted_inodes、livelist 等十三棵树的根） | `tree_table_entries` | 5161–5164（`tree_table_entries` 逐条的 `entry.root`） | 进了 |
| extent 树上段、下段节点 | `extent_tree.upper_nodes` / `lower_nodes` | 5165–5178 的两条 `.chain(` | 进了 |
| inode 叶容器 | `inode_leaf_containers` | 5179 起的 `.chain(` | 进了 |
| 分配记录树节点 | `allocation_record_tree.nodes` | 5185 起的 `.chain(` | 进了 |
| 记账树、中央映射树节点 | `accounting_tree.pointers`、`central_mapping_tree.pointers` | 5192、5193 | 进了 |
| 墓碑、谱系、退役三棵树的容器指针 | `tombstone_containers` 等 `.pointer` | 5194–5196（W8 新并入） | 进了 |
| 三棵树根之下的码 2 节点指针 | `*_tree_index_nodes.pointers_below_the_root` | 同上，经 `container_tree_pointers_read_from_disk`（crates/singlefs-core/src/recovery.rs:5120） | 进了 |
| 数据指针 | `data_pointers` | 5202–5204 的循环（无全零豁免，与写者一致） | 进了（旧） |
| 实例表链各片的指针 | 不在 `TransactionOutput` 里：只在 `InstanceTableRewrite` 里「pub replaced_chain: Vec<NodePointer>,」（`crates/singlefs-core/src/transaction.rs:4123`） | 不判 | 没有写回路径：这些指针只用来释放旧链，新链的指针现发，所以不是「读来要照抄写回」的类；不算漏 |
| journal 记录里的新根段四条指针 | `record`（`JournalRecord`，crates/singlefs-core/src/journal.rs:160 起） | 不判 | 下一次发布只读 `record_bytes` 取反向链，不写回指针；不算漏 |

根之下节点的过滤与交出在「.filter(|(position, _, _)| !self.index_nodes.shape.is_the_root(*position))」（`crates/singlefs-core/src/recovery.rs:4061`）：根的指针住树表条目，随树表条目判；层级 ≥ 2 的树里根之下全部码 2 节点都在这个集合里，叶容器在 `containers`。所以一棵树的「根、根之下节点、叶容器」三段指针各有一处判，没有漏一段。

结论：这一版里我没有找到还没进这一判的、会被照抄写回的节点指针类。

### 1.3 坏镜像在写之前交类型化错误、两块盘不变

| 条款/定案一句 | 代码一处 | 判定 |
|---|---|---|
| K4 判决改法：「把三棵容器树的容器指针与 `pointers_below_the_root` 并进那张清单」（`research/prompts/m3-b44-code-r1-main-verification.md:23`） | 重建失败映射成类型化错误：「RebuildVersionFailure::Walk(recovery_failure) => MountError::Recovery(recovery_failure),」（`crates/singlefs-core/src/mount.rs:1081`）；重建发生在取号之前的读阶段：「match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {」（`crates/singlefs-core/src/mount.rs:7666`），早于 `establish_instance`（crates/singlefs-core/src/mount.rs:6174） | 对得上 |
| 同上：盘不变 | 新用例断言 `MountError::Recovery(RecoveryFailure::InvariantViolated { invariant: "I-2.5", .. })`（`crates/singlefs-harness/tests/container_tree_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount.rs:66-85`，辅助函数名 `writable_mount_refused_before_any_write` 表明「拒在任何写之前」） | 对得上（用例读码，未跑） |

### 1.4 会不会误拒合法写路径写出的镜像

| 形 | 读码结论 |
|---|---|
| 全零空指针 | 豁免在 crates/singlefs-core/src/pointer.rs:316；容器树的零根、空树表条目走这一支，放行 |
| 合法写路径写出的指针 | 写者在 `write_to` 里断言同一判（`crates/singlefs-core/src/pointer.rs:325-328`），升序是写出之前的前提；读侧用同一个函数判，所以写者写得出的，读侧一定放行 |
| 单盘池 | 池恒两盘：发布路径对两条位置条目同盘的映射条目已经拒（「if locations[0].device == locations[1].device {」，`crates/singlefs-core/src/transaction.rs:3627`），没有「单盘池写出两条同盘位置条目」的合法写路径 |
| 同一指针两条位置条目同槽、不同盘 | 只比设备身份（crates/singlefs-core/src/pointer.rs:87），槽号不比；`slot_shared_by_both_location_entries` 是另一判，不受影响 |

### 1.5 条款没写的地方（不是反例）

1. 重建所选那一版失败时，可写挂载会按 F2 退到见证过的较早一版再建一次（`crates/singlefs-core/src/mount.rs:7676` 起）。较早一版建得出就挂载成功、不交 I-2.5 错。这条路对树表、extent 树等旧清单里的指针早就如此，I-2.5 与 K4 判决都没有说这一形怎么算；新并入的三棵树沿用同一处置。
2. 同一个重建函数还被两处用：回退目标重建（「let target_version = match rebuild_version_in_the_unit_area_starting_at(」，`crates/singlefs-core/src/mount.rs:8500`）和 inode 号水位扫环（`crates/singlefs-core/src/mount.rs:8253` 起，`Err(_) => None` 把建不出的根不计入水位）。三棵树的新判让更多坏镜像上的环根在这两处被当成「建不出」，条款对水位扫描怎么处置建不出的根没有话。
3. 根之下节点指针的会红用例只有墓碑树一份（`crates/singlefs-harness/tests/container_tree_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount.rs:141`）；谱系树、退役树只有一层容器的用例（同文件 104、123 行）。三棵树走的是同一个泛型读函数 `ContainerTreeRead::index_nodes_version`（crates/singlefs-core/src/recovery.rs:4054），逻辑上同一段，缺的是覆盖不是逻辑。

什么现象会推翻 J1「对得上」：造一份 `TransactionOutput` 里带 `NodePointer` 而 crates/singlefs-core/src/recovery.rs:5136 起那串没有判的字段，并让它被下一次发布原样写回（即第 1.2 节表漏一行）；或造一份合法写路径写出、读侧判成非升序的指针（写者的断言会先红，所以只可能出在读侧比较与写者比较不同的地方，crates/singlefs-core/src/pointer.rs:316/87 与 crates/singlefs-core/src/pointer.rs:325 是同一个函数，我没看到这样的缝）。

## 二、J2：取号之前把删除续做的每一步都预演一遍

### 2.1 条款与代码逐句对

| 条款一句 | 代码一处 | 判定 |
|---|---|---|
| 岔路单第一行：「这一错在取号之前交回、挂载拒，盘一个字节不变」（`research/prompts/m3-b44-code-r1-forks.md:5`） | 预演调用「refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(」（`crates/singlefs-core/src/mount.rs:6320`）在取号函数调用「} = acquire_then_publish_until_the_continuation(」（`crates/singlefs-core/src/mount.rs:6376`）之前、也在逐盘核「let devices_without = devices_without_the_selected_version(」（`crates/singlefs-core/src/mount.rs:6344`）之前，`?` 交回；预演函数拿的是 `&PoolWriter`（`crates/singlefs-core/src/mount.rs:5254` 的 `pool: &PoolWriter<'_, Device>,`）与 `&PoolAllocator`，在内存盘那一支拿的是不可变切片「devices: &[(DeviceIdentity, Device)],」（`crates/singlefs-core/src/mount.rs:6950`），类型上写不了盘 | 对得上 |
| 用户 2026-10-16 定案：「只拒 C841，别的错不挪」（`records/2026-10-13-代码欠账批.md:158`） | 只有一个 `true`：「PublishError::NextContainerNumberPastTheTopOfItsRange(_) => true,」（`crates/singlefs-core/src/mount.rs:5306`），其余变体逐个列在 `=> false`（`crates/singlefs-core/src/mount.rs:5334` 收尾），没有通配臂 | 对得上 |
| 同一句调度记录的前半「预演整个续做」 | 预演函数里是 `loop`，每轮取队首、算计划、预演、接着走，队列空才 `return Ok(())`（`crates/singlefs-core/src/transaction.rs:6278-6281`，迭代上界的论证在 6264 的文档注释） | 对得上 |
| 岔路单：不动真分配器 | 第一趟吃的是值传进来的分配器拷贝（`crates/singlefs-core/src/mount.rs:5106` 的 `let mut copy = allocator.clone();` 一路带到 `allocator_after_the_last_publish`，5221–5225 交出） | 对得上 |
| 「拒了之后两块盘逐字节不变、再挂一次同样拒」（正文第三节 J2 判什么） | 拒的路径上没有任何可变状态落到盘或真分配器；用例对同一组盘挂两次、两次都比镜像：`let second_refusal = writable_mount_under_deletion_switches_refused(`（`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:603`），另两条用例 699、938 行同形 | 对得上（用例读码，未跑） |

### 2.2 预演与真做是不是同一段代码：调用链

真做（可写挂载续做）：

1. `establish_instance`（crates/singlefs-core/src/mount.rs:6174）→ 取号到续做之前：`acquire_then_publish_until_the_continuation`（crates/singlefs-core/src/mount.rs:6705，调用点 6376）；里面写行用 `publish_rows_on_file_version`（crates/singlefs-core/src/mount.rs:4974）、暖机用 `publish_empty_after`（crates/singlefs-core/src/mount.rs:5034），再 `allocator.release_reclaim_holds();`（crates/singlefs-core/src/mount.rs:6864）。
2. → 续做：`continue_the_deletion_queue_until_it_is_empty`（crates/singlefs-core/src/mount.rs:6515，调用点 6404）→ `walk_the_deletion_queue_until_it_is_empty`（crates/singlefs-core/src/transaction.rs:6330，调用点 crates/singlefs-core/src/mount.rs:6545）→ `publish_unlink_step`（crates/singlefs-core/src/transaction.rs:6177，调用点 6346）→ 先 `next_unlink_step_after`（crates/singlefs-core/src/transaction.rs:6206，调用点 6185），再 `publish_deletion_step`（crates/singlefs-core/src/transaction.rs:6362，调用点 6191）→ `deletion_step_plan`（crates/singlefs-core/src/transaction.rs:6382，调用点 6373）与 `publish_version_of_trees`（crates/singlefs-core/src/transaction.rs:6589，调用点 6370）→ 先拒冻结（6599）、再拷分配器（6601）、再 `prepare_the_version_publish`（crates/singlefs-core/src/transaction.rs:6658，调用点 6602，核校验和用 `ReadEveryReplacedCopyBeforeReleasingIt`，6608）→ 落盘 `persist_the_publish_or_freeze_it`（crates/singlefs-core/src/transaction.rs:1341）。

预演第一趟（只算不写，分配器拷贝）：

1. `establish_instance` → `dry_run_of_the_publishes_after_acquisition`（crates/singlefs-core/src/mount.rs:5097，调用点 6264）：写行、暖机各走 `prepare_the_version_publish`（crates/singlefs-core/src/mount.rs:5127、5187，校验和用 `SkippedByTheDryRunBeforeAcquisition`），交回最后一版与最后的分配器拷贝（5221–5225）。
2. → `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`（crates/singlefs-core/src/mount.rs:5251，调用点 6320）→ `rehearse_every_step_of_the_deletion_queue_walk`（crates/singlefs-core/src/transaction.rs:6269，调用点 crates/singlefs-core/src/mount.rs:5267）→ 每轮 `next_unlink_step_after`（crates/singlefs-core/src/transaction.rs:6282）、`deletion_step_plan`（6291）、`prepare_the_version_publish`（6288，`SkippedByTheDryRunBeforeAcquisition`，6294）。

预演第二趟（第一趟遇到空间不够才走）：crates/singlefs-core/src/mount.rs:5277 → `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`（crates/singlefs-core/src/mount.rs:6946）→ 里面调的就是真做的 `acquire_then_publish_until_the_continuation`（crates/singlefs-core/src/mount.rs:6972）与 `continue_the_deletion_queue_until_it_is_empty`（crates/singlefs-core/src/mount.rs:6981），同一个函数、同一份源码。

所以：
- 第二趟与真做是同一段代码，只有盘（内存盘）与分配器（拷贝）不同。
- 第一趟与真做共用 `next_unlink_step_after`、`deletion_step_plan`、`prepare_the_version_publish` 这三个函数，其余不同：没有 `refuse_while_a_publish_is_frozen`（crates/singlefs-core/src/transaction.rs:6599）、没有分配器备份与换回（6601）、不落盘、不读盘核校验和。

### 2.3 预演那一趟的输入，与真做逐项比

| 项 | 第一趟（拷贝上只算） | 真做 | 差 |
|---|---|---|---|
| 分配器 | `allocator.clone()`（crates/singlefs-core/src/mount.rs:5106）经写行、暖机推到最后 | 真分配器经真写行、暖机，随后放开扣住位（crates/singlefs-core/src/mount.rs:6864） | 第一趟没放开扣住位：被扣住的槽「`is_free` 与开段都绕开它」（`crates/singlefs-core/src/allocator.rs:660`），可用槽比真做少，空间判得更紧；错的方向是把「真做放得下」判成「空间不够」，随后走第二趟（精确）；见 2.4 |
| 盘 | 不读不写（`SkippedByTheDryRunBeforeAcquisition` 只做「映射条目位置项指池外盘/同盘」那一判，不读盘，crates/singlefs-core/src/transaction.rs:6636 起的枚举文档） | 读盘核每一份换下的单元的校验和（crates/singlefs-core/src/transaction.rs:6608） | 核出对不上的单元，真做把它留在已分配、第一趟照常释放：第一趟空间比真做宽；错的方向是把「真做空间不够」判成「放得下」，2.4 第 2 条 |
| 上一版 | 第一趟吃 `version_after_the_last_publish`（crates/singlefs-core/src/mount.rs:5221-5225），是同一个 `prepare_the_version_publish` 造出来的 `TransactionOutput` | 真做吃 `current`（真发完暖机、抬 F 后的一版） | 写行、暖机的计划都是 `file: None`、`new_inode_records: &[]`（`crates/singlefs-core/src/mount.rs:4963-4965`、5003-5005），删除半边是「NEITHER_A_DELETION_NOR_A_ROLLBACK」（crates/singlefs-core/src/mount.rs:5135）：队列、墓碑树、谱系树、退役树照抄（「意图加一个、最后一批删一个、别的照抄（升序）」，`crates/singlefs-core/src/transaction.rs:4223`）。写行那次发布不改续做看到的 `deleted_inodes`、墓碑与退役容器 |
| 实例代号 | `publishes_planned.instance_to_acquire`（crates/singlefs-core/src/mount.rs:5273） | `acquire_expected_instance` 交回的号；「instance, instance_to_acquire,」（`crates/singlefs-core/src/mount.rs:6759`）在写之后断言相等 | 同 |
| 批大小、容器树容量 | `allocator.deletion_switches()`（crates/singlefs-core/src/mount.rs:5266）逐项传进去 | 「let switches = allocator.deletion_switches();」（`crates/singlefs-core/src/mount.rs:6538`），再 `pool.set_container_tree_capacities`（6544） | 同一个分配器字段；`prepare_the_version_publish` 里别处读的是写入口的码 2 节点容量与记录点名容量，两边都是 `PoolWriter::new` 的默认档 |
| 写入时刻 | 没有时钟：对 mount.rs、transaction.rs、mounted_session.rs、recovery.rs 搜 `SystemTime`、`Instant::`、`UNIX_EPOCH` 零命中（命令见第六节） | 同 | 删除各批的「写入时间照抄上一版这条记录的值」：「write_time_seconds: record.write_time_seconds,」（`crates/singlefs-core/src/transaction.rs:6242`）在 `next_unlink_step_after` 里，两边同一行 |
| txg | `checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?`（crates/singlefs-core/src/transaction.rs:6220），两边同一行 | 同 | 真做在写行之后可能多推了准入的抬 F，txg 比第一趟大；所以 txg 到顶这一错不拒（crates/singlefs-core/src/mount.rs:5329 的臂）。C841 不依赖 txg，在 `ordinary_allocation_part` 里（crates/singlefs-core/src/transaction.rs:5304、5340）算 |
| 树号 | `version_rehearsed.tree_identifiers`（crates/singlefs-core/src/transaction.rs:6293） | `previous.tree_identifiers`（publish_deletion_step 里） | 同 |

### 2.4 第一趟与真做的两处不对称，及它们落在哪一支

1. 扣住位未放开（第一趟空间更紧）：第一趟见空间不够 → 走第二趟（crates/singlefs-core/src/mount.rs:5276）→ 第二趟走的是真做的函数、放开扣住位（crates/singlefs-core/src/mount.rs:6864）→ 结局是精确的。这一方向只多花一趟内存里的发布，不会误拒。
2. 核校验和被跳过（第一趟空间更宽）：真做核出对不上的换下单元留在已分配、可用槽更少。C841 出现在 `ordinary_allocation_part`（crates/singlefs-core/src/transaction.rs:6680），早于同一步的落点与准入判，所以同一步内顺序是 C841 先；但在 C841 那一步之前的某一步，真做可能因为留在已分配的槽而空间不够、推抬 F 仍推不满，挂载照样做成并把队列留着（`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`，crates/singlefs-core/src/mount.rs:6606），永远走不到 C841 那一步；而第一趟没看到那次空间不够，会照 C841 拒。形：一份换下单元校验和对不上的坏盘 ∧ 池几乎满 ∧ 退役容器号在 u64::MAX。这是读码推的，没造镜像；方向是误拒（挂载本来能做成），条款对这一形没有话（岔路单只定了「预演」，没定预演与真做的空间判有差时以谁为准）。
3. 另一类是「预演不出」的盘面：写失败、屏障失败、读故障在第一趟与第二趟都预演不到（内存盘的 `write_at`、`barrier` 只判对齐与越界，见 crates/singlefs-core/src/mounted_session.rs:476、492）。这些错真做时才出，走真做自己的支（crates/singlefs-core/src/mount.rs:6569 起，块设备错 `BlockDevice(_)` 不是空间不够，交回 `MountError::Publish`）。

### 2.5 只拒 C841 写在哪一行，别的错走哪一支

| 预演里报的 | 去哪 | 行 |
|---|---|---|
| 第一趟走完 `Ok` | 不拒，挂载往下 | crates/singlefs-core/src/mount.rs:5275 |
| 第一趟报空间不够（`SpaceAdmissionRefused`、`PlacementRefused`，crates/singlefs-core/src/mounted_session.rs:913） | 走第二趟：第二趟返回 `Some(错)` 就进下面的过滤，返回 `None`（续做走到空、推满仍放不下、抬 F 自己报错、取号到续做之前报错）就不拒 | crates/singlefs-core/src/mount.rs:5276-5287 |
| 第一趟报别的错 | 原样当 `refusal`，进过滤 | crates/singlefs-core/src/mount.rs:5289 |
| 过滤 `rehearsed_continuation_refusal_is_returned_before_acquisition`（crates/singlefs-core/src/mount.rs:5304） | C841 → `true`（5306）→ 构造 `MountError::Publish`（5294）；其余每个变体 → `false`（5334 收尾）→ `return Ok(())`（5292），挂载往下做，真做自己撞到同一个错 | crates/singlefs-core/src/mount.rs:5291-5299 |
| 真做撞到非空间不够的错 | 「if !refusal_is_short_of_space(&refusal) {」（`crates/singlefs-core/src/mount.rs:6569`）→ `MountError::Publish`，带真实写账；空间不够 → 推抬 F 再试 | crates/singlefs-core/src/mount.rs:6569-6585 |

补一条读码事实：续做的各步（批、最后一批）里 C841 只能从退役树来。墓碑树只在意图那一步追加（「DeletionStepOfThePublish::Intent { tombstone, .. } => {」，`crates/singlefs-core/src/transaction.rs:5292`），批与最后一批走 `carried`；谱系树只在管理员回退那次追加；退役树在每个有叶容器身份消失的发布里追加（「.map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;」，`crates/singlefs-core/src/transaction.rs:5340`）。所以取号之前预演能拒的 C841 在今天的续做里只有退役树一种，新用例也只造了退役树的三形（`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:565`、667、840 行的用例名）。

什么现象会推翻 J2「对得上」：造一份镜像，第一趟与真做看到不同的版（例如写行那次发布改了队列或退役容器），或第一趟 `Ok` 而真做撞 C841，或第一趟报 C841 而真做做成且队列走到空；2.4 第 2 条是我找到的最近的一形，没造。

## 三、J3：预演遇到空间不够时，在只进内存的盘与分配器拷贝上把取号之后那一串照真做走一遍

### 3.1 条款与代码逐句对

| 条款一句 | 代码一处 | 判定 |
|---|---|---|
| 用户 2026-10-16 定「抬 F 也预演进去」（`records/2026-10-13-代码欠账批.md:162`） | 第一趟报空间不够就走第二趟：「match walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory(」（`crates/singlefs-core/src/mount.rs:5277`）；第二趟里取号之后那一串的抬 F（`push_floor_raises_after_the_row_publish`，函数在 crates/singlefs-core/src/mount.rs:7042）与续做里为空间推的抬 F（「match push_one_floor_raise_within_the_admission_budget(」，`crates/singlefs-core/src/mount.rs:6588`）都在被调的两个真做函数里 | 对得上 |
| 主 agent 收下：「用只进内存的盘走真做那一串，沿用 2026-10-09 用户定的预演机制，不另抄抬 F 算法」（`records/2026-10-13-代码欠账批.md:164`） | 同一个类型、同一种构造已在会话的推环预演里用过：「(*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))」（`crates/singlefs-core/src/mounted_session.rs:664`），挂载这边是 crates/singlefs-core/src/mount.rs:6963 同形 | 对得上 |
| 内存盘文档：「读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘」（`crates/singlefs-core/src/mounted_session.rs:356`） | 读：先查 `written_blocks`，没写过的块穿到 `self.disk.read_at`（crates/singlefs-core/src/mounted_session.rs:429-467）；写与写零：只 `keep_in_memory`（477、488）；屏障：「fn barrier(&mut self) -> Result<(), BlockDeviceError> {」后直接 `Ok(())`（crates/singlefs-core/src/mounted_session.rs:492-494）；底下那块盘是 `disk: &'disk dyn BlockDevice`（crates/singlefs-core/src/mounted_session.rs:362），类型上调不到改盘的方法 | 对得上 |
| 同文档：「请求先过两个后端那一份判据」（`crates/singlefs-core/src/mounted_session.rs:359`）、「真盘会拒的这里照样拒」（`crates/singlefs-core/src/mounted_session.rs:360`） | `check_request` 先于读、写、写零（crates/singlefs-core/src/mounted_session.rs:435、476、486） | 对得上 |

### 3.2 内存里走的与真做：同一段代码、同一份输入吗

| 项 | 内存里（第二趟） | 真做 | 差 |
|---|---|---|---|
| 代码 | `acquire_then_publish_until_the_continuation`（调用点 crates/singlefs-core/src/mount.rs:6972）、`continue_the_deletion_queue_until_it_is_empty`（6981） | 同两个函数（调用点 6376、6404） | 同一段源码，泛型的 `Device` 不同（内存盘 / 真盘） |
| 分配器 | 「allocator.clone(),」（`crates/singlefs-core/src/mount.rs:6975`），拷的是 `&allocator`，即 `establish_instance` 里取号之前设好各开关之后、真做即将整个吃进去的那一份（crates/singlefs-core/src/mount.rs:6320 调用时它还没被动过） | 那一份本身 | 同一个值 |
| 计划 | `publishes_planned.clone()`（crates/singlefs-core/src/mount.rs:6976），结构带 `#[derive(Clone)]`（crates/singlefs-core/src/mount.rs:6677） | 同一个结构被移进真做（crates/singlefs-core/src/mount.rs:6376） | 同一个值 |
| 实例代号 | 第二趟自己再调一次 `acquire_expected_instance`（crates/singlefs-core/src/mount.rs:6721），在内存盘上写系统配置槽；重算与计划里的号不等或读不出 → `.ok()?`（crates/singlefs-core/src/mount.rs:6980）交回 `None`，不拒 | 同一函数同一行 | 同一段；第二趟的取号写进内存盘，不落真盘 |
| 写入时刻 | 没有时钟（第 2.3 节） | 同 | 同 |
| 读到的盘 | 底下那几块真盘此刻的字节：`&*pool.devices`（crates/singlefs-core/src/mount.rs:5279） | 紧接着真做读的就是这几块 | 同；但这时逐盘核（crates/singlefs-core/src/mount.rs:6344）还没做，第二趟可能读到一块不带所选那一版的盘；读不出的单元在发布路径上记为隔离而不是错，所以我没看到 panic 的路，但没造镜像 |
| 影子账那一臂、参数 | `start.shadow_ledger`、`parameters` | 同 | 同 |
| 续做之前已落盘的写账 | 传 `Vec::new()`（crates/singlefs-core/src/mount.rs:6988 的那条注释：只随错交出，不影响怎么走） | 真值 | 只影响错里带的账，不影响走哪支；第二趟只取 `failed.cause`（crates/singlefs-core/src/mount.rs:6993） |

### 3.3 只进内存的盘与真盘分得开的地方

| 项 | 内存盘 | 真盘 | 对 C841 那一判有无影响 |
|---|---|---|---|
| 写零 | `write_zeroes_at` 先判对齐越界，再 `vec![0u8; length]` 逐块记全 0（crates/singlefs-core/src/mounted_session.rs:481-490） | 一个动作清零 | 语义同；发布路径不写零，只有 mkfs 写零（「发布路径不写零，只有 mkfs 写零」，`crates/singlefs-core/src/mounted_session.rs:364`；core 非测试代码里调 `write_zeroes_at(` 的只有 `make_filesystem.rs` 558、560、562，`block_device.rs` 里的调用是各后端自己的实现与单测，见第六节输出），所以 `length` 大到撑爆内存的路走不到 |
| 读没写过的块 | 穿到底下那块盘，连续没写过的块合成一次读（crates/singlefs-core/src/mounted_session.rs:437-467） | — | 读次数少于真做（被覆盖的块不再读盘），见 3.4 |
| 读故障 | 穿到底下的读错原样 `?` 交回；被内存盖住的块读不到底下，所以不会碰到底下那块的读错 | 读该块会撞读错 | 第二趟里撞读错 → 发布或取号报错 → 不是 `Publish` 的变成 `None`，是 `Publish` 的（块设备错）被 `BlockDevice(_) => false`（crates/singlefs-core/src/mount.rs:5334）滤掉：不拒，留给真做；方向是少拒，不是误拒 |
| 设备报错（写、屏障） | `write_at` 与 `barrier` 除对齐越界外不失败（crates/singlefs-core/src/mounted_session.rs:476、492） | 可失败 | 预演不出写失败；真做撞到时走真做自己的支（crates/singlefs-core/src/mount.rs:6569 起）。不影响「预演报 C841」是否成立，影响的是「预演说能做成、真做半路写失败」，这本来就不是预演的职责 |
| 持久序、FUA | `_durability` 丢掉（crates/singlefs-core/src/mounted_session.rs:474） | 有 | 不影响判 C841 |
| `journal_record_offsets_hint` | 没覆写，取 trait 默认 `None`（crates/singlefs-core/src/block_device.rs:152-158） | 内存盘可答非空 | 只用于读阶段扫环（`scan_journal`，crates/singlefs-core/src/recovery.rs:3496-3513）；我没追到取号之后、抬 F、续做这条路里有调用（`grep -n 'scan_journal(' *.rs` 只命中 mounted_read.rs:594、crates/singlefs-core/src/recovery.rs:5794、5845、crates/singlefs-core/src/mount.rs:7383），没逐链追完，标「读码未证」 |

### 3.4 预演多出的读，对带读故障注入的设备会不会改变真做那一趟的读结局

先量边界：第一趟不读盘（`prepare_the_version_publish` 里读盘只有 `copies_failing_the_release_checksum_check` 一处，只在 `ReadEveryReplacedCopyBeforeReleasingIt` 那一臂，`crates/singlefs-core/src/transaction.rs:6764-6770`；第一趟走 `SkippedByTheDryRunBeforeAcquisition`）。所以多出的读只在第二趟，而第二趟只在第一趟见空间不够时才走（crates/singlefs-core/src/mount.rs:5276）。没有空间不够的挂载，新代码一个盘上的读都不加。

第二趟的读会不会改变真做看到的读结局：会，读码可证。
- 注入计划按调用序数触发：每次读先 `decide`（`crates/singlefs-harness/src/fault_injection.rs:986-989`），里面 `state.calls.reads += 1`（crates/singlefs-harness/src/fault_injection.rs:839）、匹配的调用数 `state.matching_calls_across_the_pool += 1`（crates/singlefs-harness/src/fault_injection.rs:847）、「if !schedule.occurrence.fires_at(ordinal) {」（`crates/singlefs-harness/src/fault_injection.rs:854`）按序数决定打不打。
- 第二趟里没被内存盖住的块的读会穿到底下的 `FaultInjectingBlockDevice::read_at`（crates/singlefs-harness/src/fault_injection.rs:980），所以每次都推进序数。第二趟的读次数不等于真做的读次数（被盖住的块不穿），两件事后果：
  1. 排了「第 n 次读出错」的计划，n 可能落在第二趟里：那次读错被第二趟吃掉（第二趟报错 → `None` → 不拒），真做那一趟再也看不到它；
  2. 落不进第二趟的 n，在真做里对应的读整体前移了第二趟读的次数。
- 主 agent 在调度记录里已记下这一形：「预演多出的读落在底下那块盘上」（`records/2026-10-13-代码欠账批.md:164`），原话还写「这一批 15 个 harness 目标都绿」「整体再验时看崩溃轻型的崩溃注入几项」；这一腿不能跑崩溃轻型，复核不了它们是否仍绿。
- 这一形不是 C841 判错，是注入计划的计数口径被改。条款侧：岔路单、用户定案都没有说预演允许读底下的盘时对故障注入怎么算。跑前条款第三行适用（反例落在条款没写的地方 ⇒ 交用户定条款）。

### 3.5 内存占用有没有上界

| 部分 | 上界 |
|---|---|
| 第一趟 | 常数个版本（`current.clone()` 一份、每步换下一份，crates/singlefs-core/src/transaction.rs:6277、6298）加一份分配器拷贝；步数线性的是时间不是内存。步数的上界：每一步让队首文件少至少一批数据单元（`EXTENT_TREE_LOWER_LEAF_DATA_UNITS` = 144，`crates/singlefs-format/src/lib.rs:179`）或删掉队首号；合法的一版 `deleted_inodes` 至多一条 key（「previous.deleted_inodes.is_empty(),」，`crates/singlefs-core/src/transaction.rs:6131` 的前置断言，调用方先把队列走空再发意图）。调度记录量过的每多一步续做多约 9–12 毫秒（`records/2026-10-13-代码欠账批.md:162`） |
| 第二趟 | 内存里的盘每个被写过的物理块存一份 `Vec<u8>`（crates/singlefs-core/src/mounted_session.rs:365 的 `BTreeMap<u64, Vec<u8>>`），没有容量上限的判断（`keep_in_memory` 只插入，crates/singlefs-core/src/mounted_session.rs:416）。占用 ≈ 第二趟里所有发布写到的互不相同的物理块数 × 块大小 + 每块一份 map 项的开销。互不相同的块数受两样限制：池里的槽数；以及回收窗口让早先槽复用的快慢。我没量，不下数；写进「要主 agent 跑的命令」不合适（需造大镜像，属攻方腿） |

结论（J3）：同一段代码、同一份输入对得上；内存盘语义与真盘在写失败、屏障失败、读次数上分得开，前两样不影响 C841 判，读次数那一样改变故障注入的序数口径（3.4）；内存占用没有代码里的上界，只有「合法镜像队列至多一条」这条前提。

什么现象会推翻 J3「同一份输入」：第二趟里取号之后那一串报了与真做不同的错或不同的落点（例如 `assert_eq!(placements_taken, planned)` 在第二趟里 panic 而真做不 panic，断言在 `crates/singlefs-core/src/mount.rs:6882` 起）。

## 四、J4：取号之前那一拒复用 `MountError::Publish`、写账是空的

### 4.1 条款与代码逐句对

| 条款一句 | 代码一处 | 判定 |
|---|---|---|
| C841 用户定案：「开新容器那一次发布用 checked 加法，溢出返回类型化错误、不 panic，读盘不变」（`records/2026-10-13-代码欠账批.md:103`） | 取号之前那一拒返回类型化错误且没有写：「writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,」（`crates/singlefs-core/src/mount.rs:5296`）连同 5297、5298 两份空账 | 对得上 |
| `MountError::Publish` 成员文档：「续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员，那时一个写都没发，三份写账都是空的」（`crates/singlefs-core/src/mount.rs:118`） | 同上 5294-5299：三个账字段各一个空值 | 对得上 |
| 同成员文档第一句：「取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成」（`crates/singlefs-core/src/mount.rs:116`） | 取号之前的拒不在这一句里，靠 118 行补一句才说得通；`PublishSequenceFailed` 类型文档仍写「自己开写入口、接连推的一串发布里有一次没做成：可写挂载取号之后那一串」（`crates/singlefs-core/src/mount.rs:809`）与「写入口是这一串自己开的、随错一起丢掉」（`crates/singlefs-core/src/mount.rs:811`），取号之前这一拒没有写入口、没有发布 | 文档部分对不上（见 5.1），不是行为对不上 |

### 4.2 冻结副本里每一处匹配 `MountError::Publish` 的地方

`grep -rn 'MountError::Publish' --include=*.rs .`（在 `crates/` 下）共 38 行（命令与整份输出见第六节）。扣掉测试文件、注释行，源码里的使用点逐处：

| 位置 | 它认什么 | 在取号之前那一拒（写账空）上 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs:5294` | 新增的构造点 | 本身 |
| `crates/singlefs-core/src/mount.rs:5476`（`publish_failed_after_acquisition`） | 构造：写行、暖机失败，带真实已落盘账 | 不涉及：只在取号之后 |
| `crates/singlefs-core/src/mount.rs:6577` | 构造：续做里非空间不够的一步，带真实账 | 不涉及 |
| 「if let Err(MountError::Publish(failed)) = continuation {」（`crates/singlefs-core/src/mount.rs:6992`） | 第二趟里只取 `failed.cause`，不看账 | 成立：它拿到的 `Publish` 只来自 6577（取号到续做之前的错在 6980 的 `.ok()?` 已经滤掉），取号之前这一拒不会被它收到 |
| 「MountError::Publish(_)」（`crates/singlefs-core/src/mount.rs:4209`，在 `floor_raise_refused_for_space`，crates/singlefs-core/src/mount.rs:4197） | 抬 F 那一串的错判是不是空间不够；`Publish` 在「不是」那一排 | 不涉及：抬 F 路径不返回取号之前那一拒 |
| 「MountError::Publish(failed)」（`crates/singlefs-harness/src/model_comparison.rs:815`） | 只取 `failed.cause` 映射成理由；C841 在「PublishError::NextContainerNumberPastTheTopOfItsRange(_)」（`crates/singlefs-harness/src/model_comparison.rs:725`）一臂，映射成 `Unexplained` | 成立：不看账，只看 cause |
| 「MountError::Publish(_)」（`crates/singlefs-harness/src/model_comparison.rs:930`、`crates/singlefs-harness/src/model_comparison.rs:1045`） | 通配到 `None`（读坏的根环槽、抬 F 上限两个函数） | 成立：与账无关 |
| 「MountError::Publish(failed) => {」（`crates/singlefs-harness/src/history.rs:3006`） | 拼错误成员字符串 `MountError::Publish(<cause 成员>)` | 成立 |
| 「let member = mount_error_member(&error);」（`crates/singlefs-harness/src/history.rs:4406`；另一处 5169 传的是变量 `publishes_completed`，不是 0） | 随机历史执行器：把成员、理由、`publishes_completed`（传 0）、「wrote_anything: stream.operation_count() != stream_length_before,」（`crates/singlefs-harness/src/history.rs:1506`，现算）交给模型 | 成立，且比以前更准：C841 原来在写行落盘之后才报，这里传 0 次发布而录制流有写；现在取号之前拒，0 次发布、没写，两样都是真的。C841 在 `crates/singlefs-harness/src/model_comparison.rs:725` 是 `Unexplained`，模型照旧报「模型要求成功而实现拒了」，这是原样（健康镜像走不到） |
| 「MountError::Publish(_)」（`crates/singlefs-harness/src/mount_transparency.rs:2320`） | 通配到 `None`（D23 单子上的理由） | 成立 |
| 「MountError::Publish(failed) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:3204`） | 把账两份交给「划续做段」：写行之后第一个写了只有续做才写的结构的那次起是续做（crates/singlefs-checker-tier/src/crash_injection.rs:3252 起）；账全空 ⇒ `None`，字段文档「划不出续做、别的错都是 None：写行之后都算暖机」（`crates/singlefs-checker-tier/src/crash_injection.rs:2885`） | 成立：账空得 `None`；这一拒的挂载录制流里没有任何写，下游按写切段（`crates/singlefs-checker-tier/src/writable_mount_phases.rs:116` 的 `writable_mount_phase_of_each_write`）拿到的是空表，「写行之后都算暖机」的前提落不到任何写上 |
| 「MountError::Publish(failed) => {」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798`） | 先 `counts_when_the_acquisition_barrier_arrived(...)?`（取号那道屏障到了才有数），再把账交 `describe_failed_window` | 对这一拒不成立：取号之前没写、没屏障，那一步会返回「盘 {} 在可写挂载里一道屏障都没收到：取号那道屏障没发」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:739`）并停下，是停止行不是错判；而这个装置的池是新池新建文件之后冷重开，删除队列恒空，走不到 C841，所以不可达 |
| 「MountError::Publish(_)」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1241`） | 抬 F 路径的穷举，`Publish` 在「没走到第一次空发布」那一排 | 不涉及 |

`mounted_session.rs` 没有匹配（命令与输出在第六节）：它只在 `DevicesHandedIn(Box<MountError>)` 里携带 `MountError`（`crates/singlefs-core/src/mounted_session.rs:864`），不拆 `Publish`。

测试文件里的匹配（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:510`、`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:1068`、`crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs:151`、`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:979`、`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:546`）都在 `crates/singlefs-harness/tests/` 下：前四条认的是取号之后、写入口有真实账的 `Publish`（块设备错），不经过取号之前那一拒；最后一条是新用例本身，断言两份账为空（该文件 `assert_refused_before_acquisition_by_the_retirement_container_number`，553–557 行）。

### 4.3 有没有调用方把「没写」当成「写了一半」

没找到。唯一把 `Publish` 与「取号屏障已到」绑在一起的是 new_pool 装置那一处（上表倒数第二行），那一步是返回停止行；该装置的池恒无删除队列，所以这一绑定不会被这一拒碰到。

### 4.4 条款没写的地方

1. 取号之前的几种拒各有自己的成员（`RowPublishAdmissionRefusedBeforeAcquisition`、`WarmUpAdmissionRefusedBeforeAcquisition`、`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`），C841 这一拒复用 `Publish`。调度记录 2026-10-16 写了理由（新开成员要改 checker 档两处穷举 `match`，那两份在别的会话手里）；C841 用户定案只说「写路径报类型化错误」，没有说用哪个成员。
2. 顺序：这一拒排在逐盘核之前（crates/singlefs-core/src/mount.rs:6320 对 6344）。`.claude/kb/decisions/18-块里携带什么信息.md:320` 说「交进来的盘有一块不带就在取号之前、任何写之前拒可写」，没有说它与别的取号之前的拒谁先。同时有一块落后盘和 C841 队列的池，今天报 C841 的 `Publish`，改前报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，两样都是没写的拒。取号之前的预演（写行落点、暖机落点）本来就排在逐盘核之前（crates/singlefs-core/src/mount.rs:6264 对 6344），所以这一拒与已有的取号之前的拒同序，不是新的次序破例。归「条款没写」，不构成判错的反例。

什么现象会推翻 J4「调用方认的东西在这一拒上成立」：有调用方在 `Publish` 上读 `writes_of_persisted_publishes.len()` 当「写行已落盘」的证据（我搜到的读账的点：crates/singlefs-checker-tier/src/crash_injection.rs:3204、new_pool 装置 798、history.rs 的 `RaiseFloorSequencePublishFailed` 一臂，没有这样用的）。

## 五、J5：W8 新写、W10 删过日期与经过的注释

规则：`.claude/singlefs-ai-sop/rules/code-discipline.md@注释、断言、常量、重复` 里「注释里指别的文件的位置，写函数名、标题或唯一的锚串，不写行号。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:177`）与「在注释里记改动历史和作者」一行的处置「历史进 git 和 CHANGELOG」（`.claude/singlefs-ai-sop/rules/code-discipline.md:300`）。

### 5.1 注释说的是不是今天代码做的事

| 注释一处 | 今天代码 | 判定 |
|---|---|---|
| `MountError::Publish` 成员文档 crates/singlefs-core/src/mount.rs:118-119：取号之前 C841 同样交这个成员、三份写账空 | crates/singlefs-core/src/mount.rs:5294-5299 | 对得上 |
| 预演函数文档 crates/singlefs-core/src/mount.rs:5237-5250：批大小与容器树容量「续做读同一份」；空间不够的那两种「续做真发时推一串抬 F 再试的就是这两种」；C841 不拒的后果；别的错留给续做；空间不够以外的错之后不再预演 | 5266-5272 与 6538 同取自 `allocator.deletion_switches()`；`refusal_is_short_of_space` 的两个 `true`（crates/singlefs-core/src/mounted_session.rs:913）与续做里的 6569；5289-5292；transaction.rs 的 `?`（6282-6297） | 对得上 |
| `rehearsed_continuation_refusal_is_returned_before_acquisition` 文档 crates/singlefs-core/src/mount.rs:5302-5303 | 5306 唯一的 `true` | 对得上 |
| 5327-5328 的行内注释：「txg 到顶不拒（岔路单那一行只定了 C841，别的错不扩大）：分配器拷贝上那一趟不算写行之后为空间推的抬 F，续做各步的 txg 在那里算的不是真发的那一个。」 | 第一句（只定了 C841）两趟都成立；第二句的理由只说第一趟，第二趟走真做函数、txg 是精确的，不拒是因为第一句 | 部分：理由只讲了一半，不影响行为 |
| `next_unlink_step_after` 文档 crates/singlefs-core/src/transaction.rs:6201-6202：真发与预演共用这一份计划 | 调用点 6185（真发）、6282（预演） | 对得上 |
| `rehearse_every_step_of_the_deletion_queue_walk` 文档 crates/singlefs-core/src/transaction.rs:6258-6268：同一张计划、树号取上一步、不读盘核、分配器拷贝、队列空交 `Ok`、迭代上界、`# Errors` | 6279-6299 | 对得上 |
| `DeviceKeepingWritesInMemory` 文档 crates/singlefs-core/src/mounted_session.rs:356-360（加了挂载预演那一句） | 第三节表 | 对得上 |
| 第二趟函数文档 crates/singlefs-core/src/mount.rs:6941：「续做报的错就是真发时会报的那一个」 | 确定性的错（C841、释放判定、计划错）两边同一段代码同一输入，成立；写失败、屏障失败、读故障在内存盘上预演不出（3.3），真发时才有 | 部分：对 C841 成立，对设备错说过头，设备错在第二趟之后被 `BlockDevice(_) => false` 滤掉（crates/singlefs-core/src/mount.rs:5334），不影响拒与不拒 |
| `acquire_then_publish_until_the_continuation` 文档：「两边是同一段代码」、写入口「同一份参数、同一档节点容量」 | 6376 与 6972 同一函数；`PoolWriter::new` 的调用在 crates/singlefs-core/src/mount.rs:6190 与 6720 | 对得上 |
| 搬进去的旧注释（crates/singlefs-core/src/mount.rs:6783 的「2026-09-14 的改名注」、6859 的「用户 2026-10-04 选甲」） | 旧注释原样搬家，内容仍真 | 对得上；带日期，但不是 W8 新写，不在「W8 新写」的射程 |
| `dry_run_of_the_publishes_after_acquisition` 文档 crates/singlefs-core/src/mount.rs:5081：「交回每一次取到的落点」 | 今天返回 `PublishesRehearsedBeforeAcquisition`，另带最后一版与最后的分配器拷贝（5221-5225）；结构体自己的文档写了（5228 起） | 函数文档没跟着改，漏一半 |
| `PublishSequenceFailed` 类型文档 crates/singlefs-core/src/mount.rs:809、811：取号之后、写入口随错丢掉 | 取号之前那一拒也构造它（5294），没有写入口 | 文档没跟着改，行为不变 |
| 第一趟与真做的两处差（扣住位不放开、不读盘核）在预演文档里的交代 | 校验和那一处有：crates/singlefs-core/src/transaction.rs:6636 起 `ReleaseChecksumCheck` 的文档；扣住位那一处：`grep -n 'release_reclaim_holds' mount.rs` 只命中 4795、6864（第六节），预演文档里没写 | 漏写一处（见 2.4 第 1 条，方向安全） |

### 5.2 删经过时有没有把「为什么」一起删掉

我只看得到删之前（W8 之前）与删之后（今天）两头，看不到 W8 原文与 W10 的删法，所以只能核「今天的每个新函数、每个新分支是不是都带着自己的为什么」：
- 为什么取号之前拒 C841：在（crates/singlefs-core/src/mount.rs:5243-5246，不拒的后果）。
- 为什么多一趟第二趟：在（crates/singlefs-core/src/mount.rs:6937-6938，「分配器拷贝上那一趟不读不写盘，推不了抬 F……空间不够的那一步之后就接不上一版」）。
- 为什么只拒 C841：在（crates/singlefs-core/src/mount.rs:5303 与 5327）。
- 为什么 `DeviceKeepingWritesInMemory` 放宽可见性：在（crates/singlefs-core/src/mounted_session.rs:357-358）。
- 为什么 `next_unlink_step_after`、`deletion_step_plan` 拆出来：在（共用计划，crates/singlefs-core/src/transaction.rs:6201-6202、6381）。

没找到哪个新分支缺为什么。

### 5.3 还剩的、与调度记录自己的口径对不上的

调度记录 158 行写的是「新注释里带日期与「改前」的叙述、指向判决文件的句子进下一批清理」（`records/2026-10-13-代码欠账批.md:158`），W10 的任务只写「W8 新注释删日期与经过」（`records/2026-10-13-代码欠账批.md:161`）。今天冻结副本里新注释还剩 5 处指向判决或调度文件的句子，日期和「改前」没找到：
- `crates/singlefs-core/src/mount.rs:5237`（岔路单）、`:5238`（K6 判决）、`:5245`（攻方报告第五节）；
- `crates/singlefs-core/src/transaction.rs:6259`（岔路单）；
- `crates/singlefs-core/src/recovery.rs:5135`（K4 判决）。
它们写的是文件加节，没有行号，合 code-discipline 那一条的写法；同文件里已有同类句子（`crates/singlefs-core/src/mount.rs:6507` 的「代码轮 V2」）。这五处算不算「指向判决文件的句子」要主 agent 按 158 行的口径定，我不判。

什么现象会推翻 J5「注释说的是今天代码做的事」：上表「对得上」那几行里某一句的某个事实被代码否定。我逐句对过，没有；有五行是部分对得上、漏写或没跟着改（5327 行内注释、mount.rs:6941、5081、809/811、扣住位那一差）。

## 六、命令与原样输出

1. 被引源文件与主工作区逐份相同（工作目录：仓根；`F` = 冻结副本）：

```
for f in crates/singlefs-core/src/{mount,recovery,transaction,mounted_session,pointer,allocator}.rs crates/singlefs-harness/src/{history,model_comparison,mount_transparency,fault_injection}.rs crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs; do cmp -s $f $F/$f && echo "same $f" || echo "DIFF $f"; done
same crates/singlefs-core/src/mount.rs
same crates/singlefs-core/src/recovery.rs
same crates/singlefs-core/src/transaction.rs
same crates/singlefs-core/src/mounted_session.rs
same crates/singlefs-core/src/pointer.rs
same crates/singlefs-core/src/allocator.rs
same crates/singlefs-harness/src/history.rs
same crates/singlefs-harness/src/model_comparison.rs
same crates/singlefs-harness/src/mount_transparency.rs
same crates/singlefs-harness/src/fault_injection.rs
same crates/singlefs-checker-tier/src/crash_injection.rs
same crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
```

2. 预演与真做路径上没有时钟（工作目录：冻结副本 `crates/singlefs-core/src`）：

```
cat mount.rs transaction.rs mounted_session.rs recovery.rs | grep -c -E 'SystemTime|Instant::|UNIX_EPOCH'
0
```

3. 扣住位只在两处放开（同目录）：

```
grep -n 'release_reclaim_holds' mount.rs allocator.rs | grep -v '///'
allocator.rs:2194:    pub fn release_reclaim_holds(&mut self) {
allocator.rs:2816:        pool.release_reclaim_holds();
mount.rs:4795:    allocator.release_reclaim_holds();
mount.rs:6864:    allocator.release_reclaim_holds();
```

4. `MountError::Publish` 的匹配（工作目录：冻结副本 `crates/`）：

```
grep -rn 'MountError::Publish' --include=*.rs . | wc -l
38
grep -rn 'MountError::Publish' --include=*.rs . | grep -v '^[^:]*:[0-9]*:[[:space:]]*//' | awk -F: '{print $1":"$2}' | sort -t: -k1,1 -k2,2n
singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798
singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1241
singlefs-checker-tier/src/crash_injection.rs:3204
singlefs-core/src/mount.rs:4209
singlefs-core/src/mount.rs:5294
singlefs-core/src/mount.rs:5476
singlefs-core/src/mount.rs:6577
singlefs-core/src/mount.rs:6992
singlefs-harness/src/history.rs:3006
singlefs-harness/src/history.rs:3008
singlefs-harness/src/model_comparison.rs:815
singlefs-harness/src/model_comparison.rs:930
singlefs-harness/src/model_comparison.rs:1045
singlefs-harness/src/mount_transparency.rs:2320
singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:510
singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:511
singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs:151
singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs:152
singlefs-harness/tests/fault_injection_fast_tier.rs:979
singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:546
singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:547
singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:1068
singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:1069
grep -c 'MountError::Publish' singlefs-core/src/mounted_session.rs
0
```

5. 写零的调用方（core 非测试代码，工作目录：冻结副本 `crates/singlefs-core/src`）：

```
grep -n 'write_zeroes_at(' make_filesystem.rs
558:        device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
560:            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
562:        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
grep -c 'write_zeroes_at(' block_device.rs
12
```

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J1 | 对得上 | 三棵容器树的容器指针与根之下码 2 节点指针都进了重建时的 I-2.5 判；其余指针类（根四条、树表、extent、inode 叶、分配记录树、记账、中央映射、数据指针）旧清单已含；实例表链与 journal 记录指针不写回，不算漏；不会误拒合法写路径的镜像。条款没写：F2 回退、回退目标与水位扫描也吃到同一判；根之下用例只有墓碑树一份 |
| J2 | 对得上 | 第二趟与真做同一个函数；第一趟与真做共用计划与发布准备三个函数；只 C841 一个 `true`（crates/singlefs-core/src/mount.rs:5306），其余走真做自己的支。条款没写：第一趟不读盘核校验和、不放开扣住位，前者会让预演比真做乐观（读码推的误拒形，没造） |
| J3 | 对得上，旁带一处条款没写 | 内存里走的是真做的同两个函数、同一份计划与分配器值；内存盘与真盘在写失败、屏障失败上分得开，不影响 C841 判；第二趟的读穿到底下的盘，会推进注入计划的读序数，吃掉或前移「第 n 次读出错」；内存占用没有代码里的上界 |
| J4 | 对得上 | 38 行匹配逐处核过，没有调用方把取号之前的拒（写账全空）当成写了一半；new_pool 装置那一处会停在「取号屏障没发」，但那个池恒无删除队列，不可达。条款没写：这一拒排在逐盘核之前、成员复用 `Publish` |
| J5 | 对得上，五处文档部分对得上或没跟着改 | 新注释说的事与今天代码一致；`dry_run_of_the_publishes_after_acquisition`、`PublishSequenceFailed` 两处文档漏一半；预演文档没写扣住位那一差；5 处指向判决文件的句子是否算「经过」待主 agent 定 |

跑前条款三行的对应：一格被造出判错的段（第二行）——这一腿没造出，也没打中；一格的反例落在条款没写的地方（第三行）——J2 的校验和不对称（读码推）、J3 的读序数、J4 的顺序与成员复用、J1 的 F2 回退与水位扫描；第一行（两腿都没反例 ⇒ 对得上）等攻方腿。

## 八、没做什么

- 没编译、没跑任何测试、没起 GPU、没跑门禁；本腿所有「用例绿」都是读码，不是跑过。调度记录里「merge-and-verify 全绿」我复核不了（没有日志可对，也没权跑）。
- 没造镜像：J2 的校验和不对称（第一趟乐观）、J3 的读序数改变故障注入口径，都是读码推，没复现；造镜像归攻方腿。
- 没量 J3 的内存占用与第二趟的耗时，只给了结构上界。
- `journal_record_offsets_hint` 在第二趟里有没有被调用，没追完整条链（只证了 `scan_journal` 的三处调用点在读阶段与恢复入口）。
- 不判别的腿的格（攻方的反例）、不替主 agent 采纳或出判决；第 5.3 节的口径、4.4 与 3.4 的「交用户定条款」只是列出，不是我定。
- 崩溃轻型与变异轻型不归这一腿，没有要主 agent 跑的命令；若主 agent 想核 3.4 那一形，可在攻方腿或自己手上造「第 n 次读出错落进第二趟」的计划，我没有给命令。
