# 审阅报告 B：block_device / allocator / allocation_record_tree / admission / write_accounting / write_request_split

只读审阅。没跑 cargo，没跑脚本或门禁。各条的置信度这样标：「已确认」表示读代码能把推理走通，没实跑；「疑似」表示要跑一次才能定。按严重度排序。

### 1. ckpt_cost 把分配记录树算成「树高」个块，而实现每次发布在每块盘的子树上各改一条路径，保留池与 c_max 每块盘少算 ≥ 2 块
- 位置：crates/singlefs-core/src/admission.rs:483-504（`checkpoint_cost_of_the_version_to_build_on`）；对照 crates/singlefs-core/src/allocation_record_tree.rs:8-12（根里按盘分路）、allocation_record_tree.rs:901-907（4 GiB 两盘树高 3）；crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs:126、203-213（暖机空发布写 5 个分配记录树节点）
- 严重度：高
- 置信度：计数差已确认（代码公式对照测试里钉住的写数）；满盘时 checkpoint 写不出去这个后果是推的，没跑
- 代码做了什么：
  ```rust
  Some(file_version) => {
      file_version.position_addressed_tree_heights_read_from_the_root_node_headers().allocation_record_tree
          + file_version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping)
          + u64::try_from(file_version.accounting_tree.node_count())...
  ```
  `None` 臂同样取 `AllocationRecordTreeGeometry::of_allocator(allocator).height()`（admission.rs:499）。
- 为什么错：分配记录树在根之下按盘分路（allocation_record_tree.rs:12「根里按盘分路、根之下每个节点只属于一块盘」）。每个单元在每块盘上各有一条记录，所以任何一次发布（空发布也算）至少改动每块盘各一片叶，连同各自那一串祖先。改动的节点数至少是 盘数 × (高 − 1) + 1，不是「高」。写账测试把这个数钉成了常量：`const ALLOCATION_RECORD_TREE_NODES_REWRITTEN: u64 = 5;`，后续可写挂载的暖机空发布就按它写（`later_warm_up_publish_by_kind`）。4 GiB 两盘时代码算出的分配记录树那一项是 3，实写 5；1 TiB（高 4）时算出 4，实写至少 7。E148 的 9 块模型是一树一路径（E148 页第 61-62 行），早于按盘分路的 K1 取法。
- 失败场景：4 GiB 两盘、第一个事务规模。ckpt_cost = 3（分配记录树高）+ 映射树高 + 记账树节点数 + 1；映射树高与记账树节点数在这个规模下都按 1 推，没量，这样算出 6。一次暖机空发布每块盘实写 5 + 1 + 1 + 1 = 8 块。checkpoint 保留池（式子第八项）与切换预留里的 R × c_max 都按 6 扣，每块盘差 2 块。近满盘时，普通分配在准入口径下正好耗到只剩保留池；接着推的抬 F 空发布要 8 块，只有 6 块，取落点失败。这正是保留池要防的「死锁 2」（D23 已定项 24）。代码忠实于 D28 已定项 4 的字面「Σ 每棵记录树当前的高」，但那句字面没吃进实现员按盘分路的取法，D28 已定项 4 自己也写着高怎么读「随三方 m2-keyspace-r1」。这一格要交主 agent 判：该改条款，还是改实现。

### 2. 单元区起点写死成 UNIT_AREA_START_SLOT，journal 环长却是 mkfs 参数；环长超过 768 MiB 时，journal 记录会写进分配器发出去的单元
- 位置：crates/singlefs-core/src/allocator.rs:235-237（`unit_area_slots_of_device`）、allocator.rs:482-483、allocator.rs:512-513、allocator.rs:526-527（落点都从常量起算）；对照 crates/singlefs-core/src/make_filesystem.rs:152-175（`check_geometry`）、make_filesystem.rs:47（`INSTANCE_TABLE_SLOT = UNIT_AREA_START_SLOT`）、crates/singlefs-core/src/journal.rs:178-183（`record_offset`）、crates/singlefs-format/src/lib.rs:409-413
- 严重度：高
- 置信度：已确认（读代码能走通。仓里没有一处用大于 768 MiB 的环跑过，也就没有测试撞上）
- 代码做了什么：
  ```rust
  pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 {
      absolute_slot_count_of_device(device_bytes) - UNIT_AREA_START_SLOT
  }
  ```
  mkfs 那一侧按参数算单元区起点：`let unit_area_start = (JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES) * SLOT_BYTES;`，只拒 `ring_bytes > smallest / 4`。
- 为什么错：`UNIT_AREA_START_SLOT = 50176` 只在环长等于默认 768 MiB 时才是「journal 环末尾的下一个槽」（format lib.rs:409-413 的断言钉的就是默认值）。D3 已定项 10 ④ 定的是「单元区起始槽号 = journal 环末尾的下一个槽」。`check_geometry` 不拒大于默认值的环，系统配置读回来时也不判环长。`record_offset` 按 `(counter - 1) % (ring_bytes / 4096)` 在整个环里轮转，环长大于 768 MiB 时，记录偏移会越过 784 MiB，进入单元区。
- 失败场景：两块 4 GiB 盘，mkfs 参数 `journal_ring_bytes = 1 GiB`（≤ 4 GiB / 4，`check_geometry` 放行）。mkfs 先把 [16 MiB, 1040 MiB) 清零，再把实例表写在 784 MiB（槽 50176）。之后第 196 609 条起的 journal 记录落在 ≥ 784 MiB 处，盖掉实例表、树表以及分配器按 50176 起发出去的单元。两边都不报错：journal 写与单元写各自对齐、各自在盘内。同一个根因还有一个变体：系统配置里的 `journal_ring_bytes < 4096` 让 `ring_slots = 0`，`record_offset` 里 `% 0` 直接 panic（journal.rs:179-181）。
- 应该做什么：`check_geometry` 与系统配置的读者都拒 `journal_ring_bytes != JOURNAL_RING_DEFAULT_BYTES`（至少拒大于默认值与小于一条记录）；或者照 D3 已定项 10 ④ 把单元区起点做成几何量，分配器从几何里取，不再用常量。

### 3. 被抛弃根引用的落点直接进 `isolate` / `clear_isolation_of_slot`，槽号没判在不在单元区里，坏镜像能让可写挂载 panic
- 位置：crates/singlefs-core/src/allocator.rs:273-278（`DeviceFreeMap::index`）、allocator.rs:339-351（`isolate`）、allocator.rs:361-369（`clear_isolation_of_slot`）、allocator.rs:1346-1353（`isolate_abandoned`）、allocator.rs:1439-1461；落点来源 crates/singlefs-core/src/allocation_record_tree.rs:864-871（`node_pointers_as_far_as_readable`）与 crates/singlefs-core/src/mount.rs:786-790
- 严重度：中
- 置信度：已确认（读代码能走通；要触发，得有一份 CRC 自洽、但指针槽号越界的镜像，没造过）
- 代码做了什么：
  ```rust
  let offset_in_slots = slot.0.checked_sub(UNIT_AREA_START_SLOT).expect(
      "槽号在单元区内：盘上读来的分配记录在 recovery::allocation_records_fit_the_pool_geometry 判过，...");
  ...
  assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
  ```
  mount.rs 那一侧：`let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;` 之后直接 `placements.push((*identity, slot, unit.span_slots()))`，这些落点随后交给 `allocator.isolate_abandoned`（mount.rs:722）。
- 为什么错：`index` 的文档说前置条件由 `recovery::allocation_records_fit_the_pool_geometry` 守着，但这条路进来的不是分配记录。它们是被抛弃根的树表指针、实例表各片的指针，以及 `node_pointers_as_far_as_readable` 交回的分配记录树子节点指针。后者按文档，「读不出、解不开、位置对不上的节点它自己的指针照样交回」（allocation_record_tree.rs:833-835）。这几种指针只核过「两条位置条目同槽」（`slot_shared_by_both_location_entries`），没核槽号 ≥ 50176、也没核跨度在单元区里。
- 失败场景：一条按实例表判被抛弃的根，它的分配记录树内部节点 CRC 正确，但有一条子指针的槽号是 100（落在 journal 环里）或者超出单元区末尾。可写挂载算影子账时，`isolate` 在 `checked_sub(...).expect` 或 `assert!(end <= len)` 上 panic，池挂不上可写。仓里做过 panic 面普查（allocator.rs:1044、pointer.rs:54 提到的 R5 / R10），这一处不在其中。

### 4. 记账行「全空聚簇段数」（`empty_segments()`）与 D3 已定项 10 ① 的定义有两处不一致，而且没有不变量或 checker 对它
- 位置：crates/singlefs-core/src/allocator.rs:244、allocator.rs:429-437；写进记账行的地方 crates/singlefs-core/src/transaction.rs:5867；定义在 .claude/kb/decisions/03-空间分配.md:200
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
  ...
  self.used_per_segment.iter().filter(|used| **used == 0).count()
  ```
- 为什么错：定义原文是「『全空聚簇段数』= 段内 64 槽都没有未释放分配记录的段数」。第一处：`used_per_segment` 按位图计，已释放、还在 defer 窗口里的槽，位图照旧是占着的。所以一个段里只剩已释放记录时，定义算它全空，代码不算。第二处：`div_ceil` 把单元区末尾不足 64 槽的零头段也算成一段，零头段空着时计入全空段数；可同一文件里的 `lowest_empty_segment`（allocator.rs:503）只认整段，零头段永远开不成聚簇段。`grep -n "聚簇段" .claude/kb/invariants.md` 零命中，checker 里也搜不到对这一行的核对，这一行不管取什么值都不会红。
- 失败场景：① 4 GiB + 16 KiB 的盘，单元区 211 969 槽，末段只有 1 槽：新盘上报 3313，按定义应是 3312。② 覆盖写把某段里唯一一个单元换下、释放代还没过回收门槛时，定义下那一段算全空，记账行不算，两者差 1。D26 已定项 6 拿这个读数当整理触发的依据。

### 5. 运行时分配路径的代价随盘容量与记录数线性增长，与 fs-design「记账是事务的副产品」那张表的第一格字面冲突
- 位置：crates/singlefs-core/src/allocator.rs:478-497（`lowest_user_data_slot`：逐偶数槽扫，每个候选再对 `cluster_segments` 全表 `any`）、allocator.rs:501-516、allocator.rs:522-537、allocator.rs:615-631（`make_room_for_record_on_device`：每次分配、每块盘扫一遍全部记录）、allocator.rs:1040-1044（`release` 的 `find`）、allocator.rs:1303-1314（`reclaim_released_records_up_to`：每次发布经 `record_root_written_by_this_process` 扫一遍全部记录）、allocator.rs:429-437（每次发布扫全部段）
- 严重度：中
- 置信度：已确认（复杂度读代码可得；没测时长）
- 代码做了什么：
  ```rust
  while candidate + 1 < end {
      let inside_cluster_segment = cluster_segments.iter().any(|segment| { ... });
  ```
  ```rust
  let intersecting_record_slots: Vec<SlotNumber> = records.iter().filter(|record| record.device == device && ...)
  ```
- 为什么错：`.claude/rules/fs-design.md` 那张表第一格（运行时决策路径：分配、ENOSPC 准入、defer 窗口）写的是「不许，且代价不许随盘容量增长」。这里的扫描都在内存上，分配那一刻有答案，也不与 checker 共用算法；按那一节的 ⚠️，真正被禁的两件事它都没犯。但表格那一格的字面被它违反了，仓里也没有欠账登记（`.claude/kb/checks-owed.md` 与里程碑文件里 grep「线性扫」「随盘容量」零命中）。另外，发布失败时整个分配器要换回，所以每次发布都 clone 一次分配器，而分配器里有三张按槽的 `Vec<bool>`：1 TiB 一块盘约 6400 万槽，每次发布拷约 190 MB。
- 失败场景：1 TiB 两盘、前 90% 已占。每次用户数据分配先扫约 5800 万个偶数槽，每个槽再对本次挂载开过的聚簇段逐个比；每次发布再扫全部分配记录（每单元每盘一条）。时长随盘长。要交主 agent 判：记成欠账，还是改成增量结构。

### 6. `root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring` 是 `#[should_panic]` 测试，而变异表第 462 行点名它；门禁 59 号的正则认不出它红
- 位置：crates/singlefs-core/src/allocator.rs:2107-2113；crates/mutations.tsv:462；.claude/gate.d/59-crates-mutation-replay.sh:397-402；同一文件 allocator.rs:1935 的注释
- 严重度：中
- 置信度：疑似（正则与同一文件的注释对得上；libtest 输出里带「 - should panic」这段是按记忆写的，没实跑 59 号）
- 代码做了什么：
  ```rust
  #[test]
  #[should_panic(expected = "装了根环表的进程写的根 txg 逐个加一")]
  fn root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring() {
  ```
  59 号：`red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)`
- 为什么错：同一文件 1935 行的注释原文是「不用 `should_panic`：门禁 59 号按『test 名 ... FAILED』认红，`should_panic` 那一行多一段『- should panic』认不出」。可 2109 行照样用了 `should_panic`，变异表第 462 行又拿它当期望变红的测试。
- 失败场景：59 号跑到第 462 行，变异把断言改成恒真，测试失败时输出 `test allocator::tests::root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring - should panic ... FAILED`。`red_pattern` 与 `ran_pattern` 都匹配不上，这一条会被报成「点名的测试没跑到」。结果要么 59 号恒红，要么这条变异从来没被真正证明过。改法照 1937-1965 行那个用例，换成 `catch_unwind`。

### 7. 「重开」镜像文件时 `set_len` 会静默截短或撑长已有镜像
- 位置：crates/singlefs-core/src/block_device.rs:178-207
- 严重度：低（只影响测试后端，但它是「接着用这块盘」的入口）
- 置信度：已确认
- 代码做了什么：
  ```rust
  .create(false).truncate(false).open(path) ...
  file.set_len(size_in_bytes).map_err(BlockDeviceError::InputOutput)?;
  ```
- 为什么错：文档说重开的语义是「接着用这块盘」，文件不在就报错、不静默建空盘。可文件在而大小不同时，它静默改大小：传小了截掉尾部数据，传大了用 0 撑长。截掉的数据没法恢复，撑长则悄悄改了单元区槽数与分配记录树的根层级（两者都由 `size_in_bytes` 算）。该做的是比对现有长度，不等就报错。
- 失败场景：调用方重开时传的 `size_in_bytes` 比 mkfs 时小，尾部的单元被截掉，而重开返回 Ok。

### 8. `release_leaving_the_record_allocated_on` 的文档与 API 允许逐盘不对称地释放，一旦真这么用，分配器会永久拒绝用户数据分配；没有断言守着对称
- 位置：crates/singlefs-core/src/allocator.rs:1026-1057；对照 crates/singlefs-core/src/transaction.rs:2783-2788、transaction.rs:2866-2876（现行政策是两份都留）
- 严重度：低（今天两个调用方都按对称给名单）
- 置信度：已确认
- 代码做了什么：文档原文「逐盘各算：另一块盘上核得上的那一份照常释放」，实现是 `if devices_whose_record_stays_allocated.contains(&device.device) { continue; }`。
- 为什么错：现行政策（transaction.rs:2785-2786）是「任一份核出对不上，这个单元在每块盘上的分配记录都留在已分配……另一块盘上那一份对得上也一起留」。按分配器文档那种逐盘用法，被释放那块盘上的槽回收之后，`lowest_user_data_slot` 在两块盘上给出不同答案。`try_allocate_user_data` 每次都走 `AnswersDiffer`，报 `UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`，直到那块盘上的槽被占掉；可要占它又得两盘答案一致，所以永远拒下去。该做的：断言名单要么为空、要么是全部盘，或者文档改成现行政策。

### 9. `payload_of` 只拦「内容比切分时短」，内容更长时静默丢掉尾部
- 位置：crates/singlefs-core/src/write_request_split.rs:39-53
- 严重度：低
- 置信度：已确认
- 代码做了什么：`assert!(self.payload_end_exclusive.0 <= content_length, "切分算出的载荷区间越出内容：...")`
- 为什么错：文档写的是「切分读的内容长度与取载荷读的不是同一份」，断言却只拦了短的那一半。今天唯一的调用方（transaction.rs:5072）用的是同一份 `file.content`，不会触发。要拦住整类错，得另外带着切分时的长度一起比（例如最后一个事务要求等于内容长度）。
- 失败场景：用 N 字节切分，再拿 N + k 字节的内容取载荷，最后 k 字节不进任何单元，不报错。

### 10. `AllocationRecord::key_bytes` 在 crates/ 里没有调用方，而且同一个 key 布局另有一份 `allocation_record_key_bytes`
- 位置：crates/singlefs-core/src/allocator.rs:64-68；crates/singlefs-core/src/allocation_record_tree.rs:57-62
- 严重度：低
- 置信度：已确认（`grep -rn "\.key_bytes()" crates/ --include=*.rs` 只命中 transaction.rs:4021、5881，调用对象是记账条目；research/e7-index-bench 那一处用的是它自己的 `AllocationRecord` 结构，e142_first_transaction_dry_run.rs:1878）
- 代码做了什么：`pub fn key_bytes(&self) -> Vec<u8> { self.to_bytes()[..10].to_vec() }`
- 为什么错：用不到的代码是没人验的路径（code-discipline「用不到的代码删掉」），而且它和 `allocation_record_key_bytes` 是手抄的第二份 key 编码。

### 11. `ProbeSysfs` 按路径的文件名查 `/sys/class/block/<名>/queue`，分区和 `/dev/disk/by-*` 这类符号链接一律探测失败
- 位置：crates/singlefs-core/src/block_device.rs:356-370、block_device.rs:445-451
- 严重度：低（失败方向是报错，不会静默拿错值）
- 置信度：已确认（分区在 sysfs 下没有 `queue/` 子目录；符号链接的 `file_name()` 是链接名，不是内核设备名）
- 代码做了什么：`let device_name = device_path.file_name()?.to_str()?;` 然后读 `sysfs_class_block.join(device_name).join("queue").join(attribute)`
- 失败场景：`open_existing(Path::new("/dev/disk/by-id/virtio-xxx"), ..., ProbeSysfs)` 或 `/dev/vda1` 都返回 `ProbeFailed { attribute: "physical_block_size" }`，只能换成 `/dev/vda` 这种写法。

## 读过的文件与行数

审阅范围内的六个文件都逐行读完了，测试模块也在内：

| 文件 | 行数 |
|---|---|
| crates/singlefs-core/src/block_device.rs | 912 |
| crates/singlefs-core/src/allocator.rs | 2114 |
| crates/singlefs-core/src/allocation_record_tree.rs | 1028 |
| crates/singlefs-core/src/admission.rs | 999 |
| crates/singlefs-core/src/write_accounting.rs | 307 |
| crates/singlefs-core/src/write_request_split.rs | 247 |

为核对上面几条，还按需读了这些片段：recovery.rs:995-1084（`allocation_records_fit_the_pool_geometry`）；mount.rs:690-730、740-805、810-875、1610-1660、3495-3530；transaction.rs:245-320、385-410、2770-2880、5630-5680、5845-5885；make_filesystem.rs:40-70、147-190、255-300、405-440；journal.rs:178-183；system_configuration.rs:355-370、440-500；format lib.rs:209-266、400-413；harness 的 lib.rs:300-400（`RecordingBlockDevice`）、tests/common/mod.rs:95-145、tests/second_transaction_supplement_one_write_accounting.rs:120-215、tests/second_transaction_step_zero_test_only_switches.rs:60-150；.claude/gate.d/59-crates-mutation-replay.sh:385-410；crates/mutations.tsv:462；decisions/28-挂载期承诺量.md:76-119；decisions/03-空间分配.md:141、167、200；experiments/148-*.md:6、61-62。

另外核过、没发现问题的几处（不列成条目）：`write_zeroes_in_chunks` 的拆块与末块截短；`check_aligned_and_in_range` 的 `checked_add`；`mark_allocated` / `mark_reclaimed` 里 runs 的增量公式；`make_room_for_record_on_device` 在两种相交形态下的改写或删除；`read_node_and_its_subtree` 对层级、key 区间、叶内严格递增、跨度为 0 的判定（跨度 0 由 `record_fits_in_its_leaf` 拒掉，所以不会进 `mark_allocated`）；`instance_switch_reserve_on_one_device` 与 D28 已定项 3 的式子逐项一致；`available_on_each_device` 的八项各扣一次；`write_accounting` 在写成功之后才记账（transaction.rs:273-312）；`split_sequential_write_into_one_unit_transactions` 首尾相接、不重不漏。

## 注释与代码不一致清单

| # | 位置 | 注释说的 | 代码做的 |
|---|---|---|---|
| a | allocator.rs:10-11 | 「全空聚簇段数逐设备——全部在分配那一刻增量维护，运行时不扫盘」 | `empty_segments()`（429-437）每次装记账行都扫一遍 `used_per_segment`；只有逐段计数是增量维护的 |
| b | allocator.rs:794-796 | `placements_reclaimed_on_release_by_the_forced_zero_reuse_window`：「逐盘各算一条，与 `reclaim_released_up_to` 的返回值口径不同」 | 1063-1066 行加的正是 `reclaim_released_up_to(...)` 的返回值长度，而那个返回值只按盘 0 报（1291-1299）。唯一钉它的测试只断言 `> 0`（second_transaction_step_zero_test_only_switches.rs:123-127），没钉绝对值 |
| c | allocator.rs:1026-1029 | 「逐盘各算：另一块盘上核得上的那一份照常释放」 | 现行政策是两份都留在已分配（transaction.rs:2785-2786，第 8 条） |
| d | allocator.rs:267-270 | `index` 的前置条件：盘上读来的都由 `recovery::allocation_records_fit_the_pool_geometry` 判过 | 被抛弃根的指针落点没走那一判就进了 `isolate` / `clear_isolation_of_slot`（第 3 条） |
| e | admission.rs:304-308 | 计数「发布与发布之间与最近一次发布的记账行逐项相等」，例外只有两处（挂载重建、抬 F 扣住） | 还有第三处：零单元发布在根落盘之后才调 `record_root_written_by_this_process`，按谓词回收（mount.rs:1980-1981、2592-2593；allocator.rs:1375-1377），而零单元发布不写新的记账行，所以分配器计数先于记账行变了 |
| f | admission.rs:18-19、472-481 | ckpt_cost「按 D28 已定项 4 的 Σ 名单」，分配记录树按树高 | 实现每次发布改每块盘各一条路径，实写节点数多于树高（第 1 条） |
| g | block_device.rs:1、105 | 模块文档列的六个动作是「读、写、屏障、FUA 写、探测、写零」 | trait 的六个方法是 read / write / write_zeroes / barrier / probe / size_in_bytes：FUA 是 `write_at` 的参数，`size_in_bytes` 才是第六个 |
| h | block_device.rs:256-257 | 「块大小 4 MiB 是 512 与 4096 的整数倍，所以每一块都还是对齐的」 | 只在物理块大小整除 4 MiB 时成立；`PhysicalBlockSizeSource::Declared` 与 sysfs 探到的值都没判这一条（今天的取值 512 / 4096 满足，只是注释的量词比代码守的宽） |
| i | make_filesystem.rs:259-261（在范围外，与第 2 条同根） | 「它们互不重叠……单元区从 784 MiB 起」 | 只在 `journal_ring_bytes` 等于默认值时成立；环长是参数，`check_geometry` 不拒大于默认值 |
| j | write_request_split.rs:41-42 | 「给的内容比切分时那份短 ⇒ … 断言拦住」 | 说的是对的，但「不是同一份输入」只拦了一半，更长的内容静默截掉（第 9 条） |
