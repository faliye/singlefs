# m2-wave1-code-r1 云端攻方腿（Opus）报告：Y1 回落与挡位 / Y2 拒绝与失败路径 / Y3 写量计数

立场：攻方。攻击面只有 Y1、Y2、Y3（Y4 归本地攻方，Y5 / Y6 归正推腿，本报告一个字都不判那三格）。
时刻：2026-09-17 UTC（本机时钟 UTC，东京时间 +9）。

## 零、复跑命令与产物指纹

副本在 `/tmp/claude-1000/m2-wave1-code-r1-opus/repo`（`rsync -a --exclude target --exclude .git`，工作区无提交点，拷贝时刻 2026-09-17 UTC）。
四个探针是副本上新加的一个测试文件，`crates/` 的代码一个字节都没改。

```
cp research/prompts/m2-wave1-code-r1-opus-model/opus_probe_y1_y2_y3.rs \
   <副本>/crates/singlefs-harness/tests/opus_probe_y1_y2_y3.rs
cd <副本> && nice -n 19 cargo test -p singlefs-harness --test opus_probe_y1_y2_y3 -- --nocapture --test-threads=1
```

| 文件 | sha256 |
|---|---|
| `research/prompts/m2-wave1-code-r1-opus-model/opus_probe_y1_y2_y3.rs` | `99c61ad957207d605e43f844c0ae7efb01e32e43d4f4a227ec9e550cb67474f1` |
| `research/prompts/m2-wave1-code-r1-opus-model/probe-run.log` | `34e2606a88d3256a825b1afaff75041cf6abd7817d75727a22b39a7aea4bbb0d` |

⚠️ **副本上量出的数不进 kb**（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）：下面每个「量过」的数都要主 agent 在入库装置上重做一次才引。

## 一、各格判定一览

| 格 | 判定 | 满足的是判据字面的哪一句 | 证据 |
|---|---|---|---|
| Y1-a 落在被挡或仍被引用的槽上 | **没打中** | Y1 触发列第一句「一次分配落在被挡或仍被引用的槽上」——三条取点函数与 `mark_allocated` 的前置逐条对得上，扫遍了六个改状态的方法 | 第二节第 1 小节（读码，没打中） |
| Y1-b 同一次发布的两个单元同槽 | **没打中** | Y1 触发列第二句 | 第二节第 1 小节 |
| Y1-c 回落置空开放段之后用户数据落进那一段 | **打中**（答的是 Y1「问」列第三句，**不满足触发列字面**） | Y1 问列「回落把开放段置空之后，用户数据会不会落进刚才那一段而那一段里还有这次发布的提交内生块」 | 探针 y1，量过：数据单元 50240，同一次发布的七个提交内生块 50242–50249 |
| Y1-d 记账行「全空聚簇段数」把开不了的段算进去 | **打中**（Y1「挡位」那一半，**不满足触发列字面**） | 无对应触发句；它落在 D3 已定项 10 ① 对「全空聚簇段数」的逐字定义上 | 探针 y1b，量过：整段 64 槽被隔离，记账行 3310 → 3310 不变 |
| Y2-a 三种拒绝在对称两盘池上走得到吗 | **没打中**（实现比条款窄，但不是错） | Y2 问列第一句 | 第三节第 1 小节（读码枚举，没打中） |
| Y2-b 拒绝时分配器逐字节不变 | **没打中** | Y2 触发列「拒绝之后分配器指纹有变」 | 第三节第 2 小节 |
| Y2-c 拒绝时**盘上**逐字节不变 | **打中** | Y2 触发列「走到拒绝，……盘上是不是逐字节不变」 | 探针 y2，量过：挂载被写行那次发布拒掉，两块盘的实例代号 1 → 2、录制流多 3 步 |
| Y3-a 有写不经过 `PoolWriter` 而进设备计数 | **没打中** | Y3 问列第一句 | 第四节第 1 小节 |
| Y3-b 快照差在发布失败、重试时对不对 | **打中** | Y3 触发列「一次发布按种类的合计与设备一层不等」 | 探针 y3，量过：按种类 21 次 / 344 576 字节，设备落盘 25 次 / 442 880 字节 |
| Y3-c `identity` 被填错而合计仍对 | **没打中** | Y3 触发列第二句 | 第四节第 1 小节 |

**反向接受条款那一格**：Y1 / Y2 / Y3 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮。
四个打中里 Y2-c 与 Y3-b 落在这条条款的射程里；Y1-c 与 Y1-d 先要主 agent 判「算不算 Y1 打中」——它们答的是 Y1 问列的句子，不满足 Y1 触发列的字面（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中归错了判据」那一格）。

## 二、Y1 回落与挡位

### 1. 没打中的那两句：落点不会落在被挡或仍被引用的槽上，同一次发布也不会同槽

三处取点函数与「挡位」的关系逐条核过（行号现查 `crates/singlefs-core/src/allocator.rs`）：

| 取点 | 函数 | 它过的闸 |
|---|---|---|
| 用户数据 | `lowest_user_data_slot`（第 395 行） | `is_free`（第 239 行）= `!allocated && !isolated && !held_until_floor_takes_effect`，两槽各判一次 |
| 开放段 bump | `bump_slot_in_open_segment`（第 755 行） | 跨度里任一槽在**任一块盘**上 `is_blocked_for_commit_generated`（第 315 行，同样三样位）就整个往后挪 |
| 回落 | `lowest_commit_generated_fallback_slot`（第 436 行） | 同一个 `is_blocked_for_commit_generated`，整跨度都要不被挡 |
| 开新段 | `lowest_empty_segment`（第 415 行） | `used_per_segment == 0 && isolated_per_segment == 0 && held_per_segment == 0` |

改这三样位的方法只有六个，全在 `DeviceFreeMap` 上，逐个核过它们与位、与逐段计数是不是同步：
`mark_released`（266）只加 `deferred_slots`、不动分配位 ⇒ 进 defer 的槽仍然「已分配」、发不出去；
`isolate`（278）、`hold_until_floor_takes_effect`（299）各自「没置过才计数」、与逐段计数同步；
`release_holds`（321）位与计数一起清；`mark_reclaimed`（328）、`mark_allocated`（371）位与 `used_per_segment` 同步。
`PoolAllocator::record`（588）对每块盘调一次 `mark_allocated`，而它前面的 `agreement_across_devices`（147）要求每块盘自己的取点函数都答出同一个槽 ⇒ `mark_allocated` 那条「跨度里有已分配的槽」断言的前置在每块盘上都成立。

同一次发布两个单元同槽：`PublishPlan::rewritten_roles`（`transaction.rs` 第 1036–1058 行）把 `Data` 排在第一个（第 1039 行），
`publish_admitted` 的取点循环（`transaction.rs` 第 1459–1468 行）逐个取、取一个标一个 ⇒ 后面的取点看得见前面的落点。
⇒ **Y1 触发列的两句都没打中。** 取样范围：读码扫了三个取点函数、四个挡位来源、六个改状态的方法；四个探针跑出来的落点也全部互不重叠。

### 2. 打中：回落把开放段置空之后，用户数据落进刚才那个聚簇段，与同一次发布的提交内生块同段

`try_allocate_commit_generated` 的回落分支第一句就是 `self.open_segment = None;`（`allocator.rs` 第 743 行）。
而用户数据的候选集只排除**当前那一个**开放段——`lowest_user_data_slot` 第 399 行 `open_segment.is_some_and(|open| …)`，`None` 时一个段都不排除。
⇒ 回落之后，那个装着本池提交内生块、将来还要被重新开成聚簇段的 64 槽区域，对用户数据完全开放。

探针 `y1_user_data_lands_inside_the_cluster_segment_that_the_fallback_closed`，原样输出（副本上量的）：

```
PROBE y1 open_segment_after_A=50240
PROBE y1 fallback_at_publish=7 open_segment_now=None
PROBE y1 reclaimed_placements=57
PROBE y1 data_slot=50240 commit_generated_of_the_same_publish_in_that_segment=[("ExtentRoot", 50242), ("InodeLeaf", 50244), ("InodeRoot", 50243), ("AllocationTree", 50246), ("AccountingTree", 50247), ("MappingTree", 50248), ("TreeTable", 50249)]
```

数据单元落 50240，正是聚簇段 [50240, 50304) 的段首；**同一次发布的七个提交内生块落 50242–50249，与它同段**。
这一段从此再也不是「全空聚簇段」（D3（空间分配） 已定项 10 ① 逐字「『全空聚簇段数』= 段内 64 槽都没有未释放分配记录的段数」，
`.claude/kb/decisions/03-空间分配.md:392`），只要那个文件版本还活着就开不了——而这个量正是 D26（后台整理与放置回收） 已定项 1 停机谓词读的那一个。

**这一步与条款的关系，两种读法要主 agent 裁**：按 D3（空间分配） 已定项 8 第 1 条逐字
（`.claude/kb/decisions/03-空间分配.md:253`）「不在任何开放的聚簇段里」，段已经不开放 ⇒ 不违反；
按同一项第 2 条逐字（`:255`）「**聚簇段只给提交内生块**」，这一段里装着本池的提交内生块、又收了用户数据 ⇒ 违反。
两条在同一项里，今天的代码只实现了第 1 条那半句。

**探针里两件事是我直接构造的，要标清楚**：① 段下方 [50176, 段起点) 被我在空闲图上占满（`fill_below`），
站位的是「老化之后底下没位置了」；② 回收的门槛我直接传了现行 txg，站位的是「抬 F 之后按 `max(F_生效, 环里最旧有效根)` 回收」
（`mount.rs` 第 377、629 行两处调 `reclaim_released_up_to`）。
「一个全空段都不剩」用的是已有用例同一个手法（`second_transaction_supplement_two_commit_generated_fallback.rs` 文件头自陈）。
**没有回收就造不出这个形态**（推的，读码）：段里被 COW 换下的槽只进 defer、分配位不清 ⇒ 段满之后段内没有偶数对齐的空槽对；
bump 留下的空洞（字节表里的 50241）都是单个奇数槽，用户数据要的是 32768 对齐的一对。
⇒ 这一格的前置是步 5 的回收或一次重开挂载，不是凭空。

### 3. 打中：记账行「全空聚簇段数」把分配器永远开不了的段算进去

`DeviceFreeMap::empty_segments`（`allocator.rs` 第 354 行）只数 `used_per_segment == 0`，
而 `lowest_empty_segment`（第 415 行）还要 `isolated_per_segment == 0 && held_per_segment == 0`。
影子账隔离的槽与抬 F 扣住的槽都不动分配位（`isolate` 第 278 行、`hold_until_floor_takes_effect` 第 299 行），
所以一整段被隔离之后，记账那一行照旧把它算成全空。

探针 `y1b_the_accounting_row_counts_segments_the_allocator_will_never_open`，原样输出：

```
PROBE y1b isolated_segment=50304 empty_segments_row_before=3310 empty_segments_row_after=3310 lowest_empty_segment_now=Some(50368) open_segment_before=Some(SlotNumber(50240))
```

整段 64 槽在两块盘上都隔离之后，记账行 3310 → 3310（一个都没减），而分配器能开的最低全空段从 50304 跳到 50368。
这一行是 `publish_admitted` 里 `STATISTIC_EMPTY_CLUSTER_SEGMENTS`（`transaction.rs` 第 1626–1629 行）写进记账树的那一行。
同一族还有 `free_slots`：`isolate` 与 `hold_until_floor_takes_effect` 都不减它，`STATISTIC_FREE_BYTES` 因此把发不出去的槽报成空闲
（推的，读码；`hold` 那一半的代码注释自陈「记账已经算它空闲，分配器却不许发出去」，`allocator.rs` 第 200 行）。
⚠️ **这一条可能已经被 C318（影子账隔离的单元没进准入不等式） 罩着**——那笔账的题面是准入不等式，这里多的是**记账树里那两行**与 D26（后台整理与放置回收） 已定项 1 的停机谓词；
是同一笔账的新一格还是另立一笔，主 agent 判。

## 三、Y2 拒绝与失败路径

### 1. 三种拒绝在对称的两盘池上走得到吗：只有第一种走得到（没打中，但实现的判别力比条款窄）

`PlacementRefusal`（`allocator.rs` 第 120–136 行）三个成员里，后两个（`SomeDevicesFullDeviceSetSelectionUndefined`、
`UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported` / `CommitGeneratedPlacementsDifferAcrossDevices…`）
要各盘的空闲图不一样才出得来。在**等大**的两盘池上它们出不来，理由是每一处改空闲图的入口都是对称的：

| 入口 | 位置 | 对称性 |
|---|---|---|
| `DeviceFreeMap::new` | `allocator.rs` 第 210 行 | 单元区槽数 = `device_bytes / SLOT_BYTES − UNIT_AREA_START_SLOT`，等大 ⇒ 相同 |
| `PoolAllocator::record` | 第 588 行 | `for device in &mut self.devices` 逐盘 `mark_allocated`，同一个槽 |
| `PoolAllocator::release` | 第 619 行 | 同样逐盘 |
| `rebuild_from_records` | 第 522 行 | 按 `record.device` 逐条标，而记录由 `record` 逐盘各写一条 ⇒ 每盘一条同槽的 |
| `reclaim_released_up_to` | 第 801 行 | 同上，逐条按记录的盘 |
| `isolate_slots_referenced_only_by_abandoned_roots` | `mount.rs` 第 280 行 | 隔离的键是 (盘, 槽)，来源同样是逐盘各一条的分配记录 |

⇒ 等大池上两张空闲图逐位相同，`agreement_across_devices`（第 147 行）只会走 `Agreed` 或 `NoAnswerOnAnyDevice`。
**层 0 两条流、`second_transaction_*` 除 `unequal_devices` 之外的全部用例都是等大 4 GiB 镜像 ⇒ 后两个拒绝在它们上面是死码。**
这不是错（收口表第 20′ 行已经把「盘不等大」记成条款空白），但它说明这两个成员的判别力全押在 `unequal_devices` 那一个文件上。
没打中，取样范围：六个改空闲图的入口全部枚举过。

### 2. 分配器指纹：拒绝之后确实没变（没打中）

`try_allocate_user_data`（第 647 行）与 `try_allocate_commit_generated`（第 695 行）都在算完各盘答案之后才动状态，
三个 `return Err` 分支前面一行状态都没改；回落分支装不下时开放段照旧开着（第 700–704 行只读）。
发布层 `publish_version`（`transaction.rs` 第 1157 行）在 `publish_admitted` 之前整份克隆分配器、失败就换回去（第 1211–1215 行）。
探针 y3 的第一次发布（被设备拒掉、写到一半）之后拿同一个上一版重试成功，就是这条路的一次实测。

### 3. 打中：拒绝时**盘上**不是逐字节不变——取号先落盘，发布层拒绝之后不回卷

`mount.rs` 的次序是写死的：第 829 行 `acquire_expected_instance`（两条超级块槽写 + 一道屏障，
`transaction.rs` 第 369–384 行 `write_acquired_instance`），第 866 行才是写行那次发布 `publish_rows_on_file_version`。
发布层任何一种拒绝（`NoSpaceFor`、`AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、`ContentExceedsDataUnit`）
在这里都只是 `?` 往上抛（`mount.rs` 第 880 行），**没有任何东西把取号写回去**——回卷只在取号自己报错时做（`roll_back_acquisition`，`transaction.rs` 第 227 行）。

探针 `y2_a_refused_row_publish_leaves_the_bumped_instance_generation_on_disk`，原样输出：

```
PROBE y2 refused_at_publish=50 failure=AllocationRecordsExceedOneNode { records: 820, capacity: 812 } records=804
PROBE y2 mount=Some(Publish(AllocationRecordsExceedOneNode { records: 814, capacity: 812 })) instance_before=[1, 1] instance_after=[2, 2] operations_added=3
```

一个只有一个文件、被覆盖写 49 次的池（没有任何手工改空闲图，全部是真发布）：第 50 次发布被分配记录树的准入挡住；
冷重开之后 `mount_writable` 也被写行那次发布挡住，而那时**两块盘的超级块实例代号已经从 1 写成 2**，录制流多了 3 步（两条超级块槽写 + 一道屏障）。
这个池从此再也可写挂载不了，而**每一次尝试都再烧一个实例代号、再写两条超级块槽**，没有上界。

四句核对：
① 分不分辨臂——这一轮判的是一份实现、不是几条臂，这一格不适用；它分得开的是「取号在发布之前」与「取号在发布之后」两种次序。
② 被判的系统当时看得到吗——看得到：准入三条（内容、分配记录树、记账树）都在 `publish_version` 里、在动分配器之前算得出，
而它们要的输入（上一版的记录数、盘数、内容长度）在取号之前就已经在手里（`mount.rs` 第 977 行 `rebuild_previous_version` 已经跑完）。
③ 满足的是哪一句——Y2 触发列「走到拒绝，……盘上是不是逐字节不变」那一句；不是「分配器指纹有变」那一句（分配器确实没变）。
④ 跑前条款的改法在这一格还中不中——条款只给一个改法「改代码、补一条会红的用例与一条变异行」，它对这一格起作用（见第五节）。

⚠️ 射程要说清：探针用 `AllocationRecordsExceedOneNode` 当触发器（收口表第 28 行已记「树装不下只报错、没有分裂」），
但**打中的不是那个错误，是取号与发布的次序**——U3 的三种拒绝经 `allocate_*().ok()`（`allocator.rs` 第 639、681 行）
丢掉原因、在 `transaction.rs` 第 1466 行变成同一个 `NoSpaceFor`，走的也是这条路。
收口表第 20′ 行只记了「发布层统一报 `NoSpaceFor`（假性 ENOSPC）」，没记「拒绝时取号已经落盘且不回卷」。

## 四、Y3 写量计数

### 1. 没打中的两句：没有写绕过 `PoolWriter`，`identity` 也没被填错

全仓 `write_at(` 的调用点现查（`grep -rn "write_at(" crates/ --include=*.rs | grep -v "fn write_at"`，共 36 处）：
生产代码里只有 `make_filesystem.rs` 第 227、232、284、303 行（mkfs，材料正文第 27 行自陈「取号与 mkfs 的写不在任何一次发布的账里」）
与 `transaction.rs` 第 128、136、153、216 行（`PoolWriter` 自己的四处），其余全在测试与块设备实现里。
`PoolWriter` 那四处每一处后面都跟着一次 `count_write_call`（第 130、137、161、221 行），**设备报成功之后才记**。
恢复路径不写（`recovery.rs` 里一个 `write_at` 都没有，同一次 grep），挂载里的写只有取号与几次发布，都过 `PoolWriter`。

`identity` 填错：`CommitStep::WriteUnitToEveryDevice` 的 `identity` 只从 `PublishedUnit::identity` 来
（`transaction.rs` 第 1968–1974 行），而 `PublishedUnit` 是 `rewritten_unit(identity, bytes)`（第 1850 行）与
`carried_unit`（第 1230 行）两处造的，两处都把同一个 `identity` 装进去；
`WrittenStructureKind::of_unit`（`write_accounting.rs` 第 49 行）是九个角色到九种的双射，
副本上的单测 `report_order_lists_every_kind_once_so_the_reported_kinds_add_up_to_the_total`（第 234 行）用 2 的幂长度把「漏一种 / 重一种」钉住。
⇒ 没打中。取样范围：36 个写点、九个角色、十二种结构名。

### 2. 打中：一次写到一半的发布 + 重试，按种类的合计与设备一层对不上

一次发布的账是 `pool.writes_by_structure_kind.since(&writes_before_this_publish)`
（`transaction.rs` 第 1967 行取基线、第 2005–2007 行取差；零单元发布在第 495、540–542 行）。
两次快照都在**成功**那条路上：`publish_admitted` 第 1968–1988 行的六个 `?` 任一处返回，
这次发布已经交给设备并成功的那几次写留在累计里，却**不属于任何一次发布的账**——
下一次发布的基线在它们之后取，差值把它们减掉了。

探针 `y3_a_failed_publish_and_its_retry_break_the_per_publish_equality_with_the_device_layer`，原样输出：

```
PROBE y3 first_attempt=Some(BlockDevice(OutOfRange { offset: DeviceOffsetInBytes(823296000), length: 32768, device_size: 4294967296 }))
PROBE y3 by_kind_write_calls=21 by_kind_written_bytes=344576 device_attempted_writes=26 device_attempted_bytes=475648 device_landed_writes=25 device_landed_bytes=442880
```

第一次发布在第 3 个单元写上被设备拒掉（数据单元两盘各一次、extent 树根两盘各一次已经落盘），拿同一个上一版重试成功。
这一段窗口里：按种类的合计 **21 次 / 344 576 字节**（重试那次自己的账，与字节表逐字节相同），
设备真落盘 **25 次 / 442 880 字节**，交给设备 **26 次 / 475 648 字节**。
差的 4 次 / 98 304 字节（= 2×32768 + 2×16384）属于失败那次发布，**记在累计里、不记在任何一次发布的账上**。

这一格直接压着里程碑「增补 1」的验收第一条逐字（`.claude/kb/milestone/02-second-txn.md:293`）
「真设备二进制报出发布 B 按单元种类的写字节，合计 344 576、写调用 21，与 E152（按里程碑对比六家文件系统的文件性能） 的块层读数逐个对上」：
写路径上出现一次 EIO 并重试，这个「逐个对上」就不成立，而重试正是 `publish_version` 那句注释
（第 1151–1153 行「准入之后任何一步失败，分配器退回到进来时的样子……拿同一个上一版重试」）留出来的能力。

四句核对：
① 分不分辨臂——不适用（判的是一份实现）；它分得开的是「账 = 两次快照之差」与「账 = 这次发布自己发出的写」两种口径。
② 被判的系统当时看得到吗——看得到：`perform` 的返回值就在 `publish_admitted` 手里，失败那一刻它知道已经写了几次（累计减基线就是）。
③ 满足的是哪一句——Y3 触发列第一句「一次发布按种类的合计与设备一层不等」。不是第二句（没有一种被记到别的种类）。
④ 跑前条款的改法在这一格还中不中——条款只给一个改法，它起作用（见第五节）。

⚠️ **今天的装置照不出这一格**：真设备二进制 `first_transaction_on_device.rs` 的两处窗口比较
（第 442–447、496–501 行）只在整条路都成功时才走到，任何一次发布报错都 `map_err` 之后整程序退出 ⇒
`publish_writes_against_device` 的 `matches=false` 这一支在失败路径上的判别力是零。
同族还有一处（推的，读码、没量）：取号全或无失败时的回卷写 `roll_back_acquisition`（第 227 行）
经 `write_superblock_slot` 记成 `SuperblockSlot`（第 221 行），同样不属于任何一次发布的账。

## 五、改法：跑前条款那一条，与我自己提的几条

跑前反向接受条款只给一条：「改代码、补一条会红的用例与一条变异行、再攻一轮」。
它在四个打中的格上都起作用（不是空选项），但**它没说改成什么**，所以下面这张表是我提的。

⚠️ **下面每一条都只在我自己的副本探针上量过，被攻过零轮**（`.claude/rules/three-way-inference.md`
「攻方腿自己提的收严，只在它自己的模型上量过，算『没被攻过』」）。
每一格标「量过」（副本上的原样输出）或「推的」（按代码推、没实现没跑）。

| # | 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|---|
| A1 | 用户数据的候选集不排除「当前开放段」，排除「任何还装着未释放提交内生块的 64 槽段」（`lowest_user_data_slot` 多带一个判据，或者干脆按 `used_per_segment > 0 ∧ 段曾被开过` 排除） | Y1-c | **推的**：没实现、没跑；它会改第一个事务之后每一次发布的用户数据落点，字节表与层 0 两条流的状态数都要重算 |
| A2 | 回落分支不把 `open_segment` 置空，只把游标推到段尾（段还挂着，用户数据照旧被排除） | Y1-c | **推的**：没实现、没跑。⚠️ 它与 C369 那条会红用例的现状可能打架——现状用例断言的是回落之后的落点，不是开放段的去留 |
| A3 | `empty_segments()` 与 `free_slots` 一起减掉隔离与扣住的槽（或者给记账行换一个「可开段数」的口径） | Y1-d | **推的**：没实现、没跑。⚠️ 它改的是写进盘上记账树的值 ⇒ 字节表、E142（第一个事务的干跑） 干跑、层 0 都要重跑 |
| B1 | 把三条准入（内容、分配记录树、记账树）与「这次发布要几个落点、池里够不够」一起挪到 `acquire_expected_instance` 之前；取号之后再拒绝的，按取号失败那条路回卷 | Y2-c | **推的**：没实现、没跑。落点那一半要在不动分配器的前提下预判，今天没有这样的只读接口 |
| B2 | 只加回卷：发布层拒绝之后照 `roll_back_acquisition` 把两份超级块写回旧实例代号 | Y2-c | **推的**：没实现、没跑。⚠️ 它多两条超级块槽写，会改「第一次之后的可写挂载」那一行的段序列 |
| C1 | 失败那次的写也归一次账：`publish_admitted` 在每个 `?` 上把 `since(&writes_before_this_publish)` 塞进错误里交回 | Y3-b | **推的**：没实现、没跑 |
| C2 | 真设备二进制在窗口比较里把「失败的发布」也算一格，`publish_writes_against_device` 的 `publishes` 带上失败那次的账 | Y3-b | **推的**：没实现、没跑。⚠️ 今天二进制在任何发布报错时整程序退出，先要有一条「失败也接着报」的路 |

**会红的用例与变异行**（我只写了形状，没落地）：
Y1-c 的用例形状就是探针 y1（断言数据单元的槽不在任何装着提交内生块的段里）；变异行「回落不把开放段置空 ⇒ 那条用例必须绿、去掉排除判据必须红」。
Y2-c 的用例形状就是探针 y2（断言被拒的可写挂载前后两块盘的实例代号相等）；变异行「去掉回卷 ⇒ 红」。
Y3-b 的用例形状就是探针 y3（断言窗口里按种类的合计等于设备落盘数）；变异行「失败那次的账不交回 ⇒ 红」。
四条探针今天**全部是绿的**（它们断言的是现状），要做成会红的检查得先把断言翻过来，这一步我没做。

## 六、没打中的形状：试过哪些、取样多大

| 形状 | 试法 | 结果 |
|---|---|---|
| 回落的落点落在影子账隔离 / 抬 F 扣住 / 已分配的槽上 | 读 `lowest_commit_generated_fallback_slot`（第 436 行）与 `is_blocked_for_commit_generated`（第 315 行）；四条探针跑出的每个落点都核过 | 没打中：回落与 bump 用的是同一个挡位函数，三样位都在里面 |
| 开新段时 `.expect(…)` 炸掉（`allocator.rs` 第 739 行） | 找一条「各盘都答同一个全空段、而段里有槽被挡」的历史 | 没打中：`lowest_empty_segment`（第 415 行）同时要 `used / isolated / held` 三个逐段计数都为 0，而三个计数与三个位数组逐个方法核过是同步的 |
| `mark_allocated` 的两条断言（跨度越界、跨度里有已分配的槽）被走到而 panic | 枚举 `record`（第 588 行）、`mark_format_time_units`（第 559 行）、`rebuild_from_records`（第 522 行）三个入口的前置 | 没打中：三个入口的槽要么来自逐盘各答一个、答得一致的取点，要么来自盘上那条盘自己的分配记录 |
| 同一个 (盘, 槽) 在 `records` 里出现两条 | 读 `record` 第 588–615 行的复用分支（`reclaimed.remove` 命中就改写、不追加） | 没打中：一个槽要能被再发出去必须先进 `reclaimed`，而进 `reclaimed` 与清分配位是同一句 `mark_reclaimed` 前后 |
| 对称两盘池上走到后两种拒绝 | 枚举六个改空闲图的入口（第三节第 1 小节那张表） | 没打中：等大池上两张图逐位相同 |
| 一次写绕过 `PoolWriter` 进了设备计数 | 全仓 36 个 `write_at(` 调用点逐个看 | 没打中：生产代码里只有 mkfs 那 4 处，而它按设计不进发布的账 |
| `identity` 被某条路径填错而合计仍对 | 读两处造 `PublishedUnit` 的地方 + `of_unit` 的九对一映射 | 没打中 |
| 快照差在「一次挂载里多次发布」上串味 | 读 `establish_instance`（`mount.rs` 第 814 行）的写行 + 暖机循环、`raise_rollback_floor`（第 566 行）的多次发布 | 没打中：每次发布各自取基线、各自算差 |
| `writable_mount` 那段窗口的起点选错（把取号的两条写算进去） | 读 `counts_when_first_barrier_arrived`（`first_transaction_on_device.rs` 第 223–225 行）与 `write_acquired_instance` 的次序（`transaction.rs` 第 375–383 行：先两条写、后一道屏障） | 没打中：快照在第一道屏障那一刻拍，取号那条写已经计在里面、被减掉了 |

## 七、这条腿自己的限度

1. **四个探针是我自己写的剧本，被攻过零轮。** 尤其 y1：段下方占满与回收门槛两件事是我在空闲图上直接构造的，
   它们站位的是「老化」与「抬 F / 重开时回收」；这两步在产品里怎么到达，我只给了读码的论证（第二节第 2 小节末），没量。
2. **副本上的数不进 kb。** 21 / 344 576、25 / 442 880、3310、1 → 2 这些数都要主 agent 在入库装置上重做一次才引。
3. **没跑门禁、没跑层 0。** `.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段
   （`awk -F'\t' -v me=three-way-attack '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv` 无输出）。
   我改的只有副本上多出来的一个测试文件，`crates/` 一个字节没动。
4. **Y1-c、Y1-d 两格算不算 Y1 打中，我判不了。** 它们答的是 Y1「问」列里的句子，不满足 Y1「触发的观测」列的字面；
   按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的「打中归错了判据」那一格，这一步归主 agent。
5. **Y1-d 可能是 C318（影子账隔离的单元没进准入不等式） 的射程内**，我没有去核那笔账今天的第四列到底写到哪一步——只读了背景材料里抄的那一行。
6. **「没打中」按规则要抽两次。** 本报告的「没打中」是一次读码 + 一次探针跑出来的落点核对，
   不是两次独立抽样；按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，它们只够记「这一轮我没想到」，不够记「没有」。

## 八、没做什么

- 不判 Y4（本地攻方的三张数表）、Y5 / Y6（正推腿的代码与条款、写回），一个字都没写。
- 没读禁读清单里的四类文件（这一轮别的腿的提示与产出、主 agent 的核实）。
- 没改 `crates/`、没改 kb、没改变异表、没做任何 git 写操作。副本在 `/tmp/claude-1000/m2-wave1-code-r1-opus/repo`，不入库。
- 没跑 `gate.sh`、没跑门禁 54 / 55 / 57 / 59 号，没在虚机里跑真设备二进制。
- 没有把探针做成会红的检查（第五节末尾说明了差哪一步）。
- 收口表第 20′ 行点的「盘不等大」那一族只读了代码、没造池——那一半的攻击面本轮我只用来论证「对称池上后两种拒绝走不到」。
