# 实审 A2a 报告：代码审阅第 15、16、21、25、37 条

时刻：2026-09-26 UTC（本机时钟）。规格 `/tmp/claude-1000/impl-rev-a2a/spec.md`；派发提示要求不改 `crates/mutations.tsv`，变异行在 `/tmp/claude-1000/impl-rev-a2a/mutations-append.tsv`（17 行，六段，与表同格式）。

## 结论

- 五条审阅意见先写会红的用例，在改前的代码上都复现了：新测试文件 14 条用例里 12 条改前红（第 15 条 4 条、第 16 条 5 条、第 21、25、37 条各 1 条），另 2 条是「保留下来的行为」的对照（写成空内容的文件照旧按一个单元打开；前一条记录不在盘上时链判不了、照样施加），改前绿。没有一条红不起来，审阅没有判错的。
- 改后 14 条全绿；`singlefs-core --lib` 124 条全绿（新加 1 条单测）。
- 17 条变异逐条经 `research/scripts/prove-red.sh` 证红，17/17 抓到。
- 两条只做了一部分，停下交主 agent（见「停下交主 agent 的设计问题」）：
  - 第 15 条：做了「mkfs 在任何写之前拒掉 journal 环盖住单元区、环短于一条记录、单元区在盘外」，**没做**「单元区起点按环长现算、实例表 / 树表 / 分配器用同一个现算值」——分配器的单元区起点在 `crates/singlefs-core/src/allocator.rs`，不在我的文件单里，只改 mkfs 这一侧会让分配器与落点对不上（推的，没跑：环短于默认时实例表会落在分配器位图的起点之前，`allocator_after_make_filesystem` 标 mkfs 占的槽时 `DeviceFreeMap::index` 的 expect 当场 panic）。于是环长大于默认 768 MiB 的一律按「第一版不支持」拒（C475（非默认环长下单元区起点取编译期常量） 那一格还开着）。
  - 第 37 条：做了「槽 0 自证不过的盘按池里第一块槽 0 可择的盘记的槽距找槽 1」；**全池没有一个槽 0 可择**（审阅报的那个场景：两盘同一步轮换写槽 0 时掉电）仍按最小槽距 4096 试，槽距大于 4096 的池在那一格上照旧挂不上。规格写的另一条路「挂载探得的 io_min」要 `PoolReader`（或 `BlockDevice`）交得出探测值，实现 `PoolReader` 的类型在 harness 里有 8 处（B3a-2、B3b 的文件与几份测试），不在我的文件单里。
- 推翻条件：主工作区（等 A1b 把 `mount.rs` 改到编得过之后）跑 `cargo test -p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode` 有一条红，或 17 条变异里有一条在门禁 59 号上没红，本报告的「做完」就不成立。

## 这一轮写过的文件

- `crates/singlefs-core/src/make_filesystem.rs`（第 15、16 条）
- `crates/singlefs-core/src/journal.rs`（第 15 条：`record_offset` 不再裸 `%`；新常量 `FIRST_JOURNAL_COUNTER`）
- `crates/singlefs-core/src/recovery.rs`（第 25、37 条）
- `crates/singlefs-core/src/mounted_read.rs`（第 21 条）
- `crates/singlefs-core/src/write_request_split.rs`（第 21 条：读侧单元数的函数与一条单测）
- `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs`（新建，14 条用例）
- `crates/singlefs-core/src/system_configuration.rs`：在单里，**没改**（第 15 条没做的那一半才要动它写的那 8 字节单元区起点）。
- `crates/mutations.tsv`：**没动**。要追加的 17 行在 `/tmp/claude-1000/impl-rev-a2a/mutations-append.tsv`，名字全以「实审 A2a 第 N 条：」起头，与主工作区 `crates/mutations.tsv` 现有的名字逐个比过，不撞。

我这几份文件改动之前的样子存在 `/tmp/claude-1000/impl-rev-a2a/orig/`（开工时从主工作区拷的，那时这几份已经带着别的会话没提交的改动）；我的改动 = 主工作区现状对它的 diff。各文件改了多少行（`diff orig 现状 | grep -c '^[<>]'`）：make_filesystem 168、journal 14、recovery 108、mounted_read 15、write_request_split 49、system_configuration 0；新测试文件 757 行。

## 每条怎么改

### 第 15 条（C475（非默认环长下单元区起点取编译期常量））

- `make_filesystem.rs` 的 `check_geometry`：单元区越界不再按「16 MiB + 环长」现算的起点判，改按 mkfs 真正写实例表单元的那个偏移（`INSTANCE_TABLE_SLOT`，今天是编译期常量 50176 槽 = 784 MiB）判，末端取树表单元第 0 版之后（`TREE_TABLE_GENESIS_SLOT` + `NODE_BYTES`）——100 MiB 的盘、16 MiB 的环改前清完根环与 journal 环之后才撞上块设备 `OutOfRange`，改后在任何写之前报 `UnitAreaBeyondDevice { unit_area_start: 822083584, .. }`。
- 新成员 `JournalRingShorterThanOneRecord { ring_bytes, record_bytes }`：环短于 4096 字节在开头拒。
- 新成员 `JournalRingPastTheCompiledUnitAreaStartUnsupported { journal_ring_end_in_bytes, unit_area_start_in_bytes }`：环末端（16 MiB + 环长，按字节、不截断到槽）越过单元区起点就拒。默认 768 MiB 的环末端正好是 784 MiB，放行；小于默认的环照收（`history.rs` 的小盘用 128 MiB 环，单元区照旧从 784 MiB 起，只是中间白放着）。成员文档写明它是「第一版不支持」：条款 D23（journal 的角色与格式） 已定项 19 ③ 写单元区起点随环长走，而分配器、恢复、mkfs 自己的落点都还按常量算。
- `journal.rs` 的 `record_offset`：`(counter - 1) % ring_slots` 改成 `checked_rem(..).expect("环至少装得下一条记录：mkfs 的 check_geometry 拒短于一条记录的环（JournalRingShorterThanOneRecord）")`，文档补 `# Panics`。环长 < 4096 时照样 panic，只是带上了它依赖的那条前置条件——这一改在行为上与原来等价，没有能单独证红的变异（见「每条新测试」表后的说明）。
- 顺带删掉了 `make_filesystem.rs` 末尾那句 `let _ = JOURNAL_RECORD_BYTES;`（审阅第四节「低级」点过）：这个常量现在真用上了。

### 第 16 条

- 新类型 `FixedStructure`（封闭枚举：系统配置槽 i、根环区域 r、journal 环）与 `FixedStructureExtent { structure, start: DeviceOffsetInBytes, length_in_bytes }`；`fixed_structure_extents` 按 mkfs 写它们的同一组式子列出六段（槽 i 在 i × 槽距、宽 4096；`root_ring::region_start` / `region_length_in_bytes`；journal 环从槽 1024 起、长 = 环长）。
- `check_geometry` 按次序判：① 根槽宽（physical_block_size）< 457 ⇒ `RootSlotNarrowerThanTheRootRecord`（开头，不再走到 `RootRecord::to_slot` 的断言）；② 环长下界、环长 ≤ 容量 ÷ 4、根环末端在盘内、单元区在盘内；③ 根槽宽 > 槽距 ⇒ `RootSlotWiderThanTheFixedStructureSlotSpacing`（每条根写会盖到同区下一个槽）；④ 六段两两比，第一对有共同字节的 ⇒ `FixedStructuresOverlap { first_in_layout_order, second_in_layout_order }`（半开区间，首尾相接不算）；⑤ journal 环末端 vs 单元区起点（第 15 条那个成员）；⑥ 原有的区域归属两判。
- 「槽距给上界」由 ④ 给出，没有另立常量：两槽要落在根环基址 1 MiB 之前（槽距 ≤ 1 MiB − 4096），区域长 S × 槽距不能越过区域间距 P × chunk = 3 MiB（S = 8 时槽距 ≤ 384 KiB，S = 16 时 ≤ 192 KiB，S = 4 时 ≤ 768 KiB）。用例钉住 S = 8 时 384 KiB 放行、384 KiB + 512 拒。
- 比的是**设备内偏移这一个地址空间**，不分哪块盘背哪个区域：mkfs 在每块盘上把三个区域都清零（`make_filesystem.rs` 那段注释写明了为什么），规格也写的是三个区域两两比。这比「只比同一块盘上真背根的区域」严，代价见设计问题第 3 条。

### 第 21 条

- `write_request_split.rs` 新加读侧的 `DataUnitsOfTheInode`（封闭枚举：`NoneWritten` / `SomeWritten`）与 `data_unit_count_implied_by_the_file_size(size, …)`：一次都没写过内容的 inode 长度 0 ⇒ 0 个单元；写过的仍用写侧那条除法（长度 0 的内容也写了一个声明长度 0 的单元，写侧的 `.max(1)` 不动）。
- `mounted_read.rs` 的 `open_file`：按上段叶里这个 inode 的条目认是哪一种（缺席或标签 0 ⇒ 没写过；内联或下段根 ⇒ 写过），再用上面那个函数算单元数去比记录条数。
- 规格原话「长度 0 时数据单元数取 0，与没有 extent 条目一致」：照字面对所有长度 0 的 inode 都取 0，会把「覆盖写成空内容」的文件（长度 0、一条 extent 记录）打不开——用例 `a_file_overwritten_with_empty_content_still_opens_with_its_one_data_unit` 钉住它照旧打得开。我按「有没有 extent 条目」分两种口径（D 组报告第 4 条的写法），见设计问题第 4 条。
- `recovery.rs` 的 `walk_to_file`（第 2718 行附近）同一条除法没改：它只读 `FIRST_INODE_NUMBER`，那个 inode 在第一个文件那次就写了内容。

### 第 25 条（用户定案「恢复补反向链检查」）

- `recovery.rs` 新加 `BackChainJudgement`（`Holds` / `Broken` / `PreviousRecordNotOnDisk`）与 `judge_back_chain_against_the_previous_record_on_disk`：本实例内逻辑前一条 = 同一实例、计数器小 1、扫环时收下的那一条；按 `journal::record_offset` 重读它那一槽，取 `device_identities()` 次序里第一份解得出、且与扫环时收下的那一条相同的镜像，按**盘上原样字节**算 `journal::back_chain_of`（311 字节头、`header_csum` 按 0），与这一条的 `back_chain` 比。
- `replay_journal` 的循环在 jsn 连续 / 链首两判之后判它：`Broken` ⇒ `break`（这一条与它所在的这次发布都不施加）；前一条不在盘上 ⇒ 不判（与池级 checker 判 I-8.6「前一条不在盘上不判」同口径）。
- 模块文档的前缀口径加上了反向链。

### 第 37 条

- `choose_system_configuration` 改成两遍：先读遍每块盘的槽 0（每块盘仍各读一次槽 0、一次槽 1，读的次数不变，跨盘的先后变了：原来 盘0槽0、盘0槽1、盘1槽0、盘1槽1，现在 盘0槽0、盘1槽0、盘0槽1、盘1槽1）；再逐盘找槽 1：自己的槽 0 可择 ⇒ 用它记的槽距；不可择 ⇒ 用池里第一块槽 0 可择的盘记的槽距；全池一个都没有 ⇒ 最小槽距 4096（没改的那一格）。
- `SystemConfigurationSlotReading` 加了 `recorded_fixed_structure_slot_spacing`（只有可择的那一臂交得出）。
- 槽 0 的 S 越界照旧整池拒；因为槽 0 先读遍，两块盘都有越界槽时点名的盘可能从「盘 0 的槽 1」变成「盘 1 的槽 0」（成员不变，带的设备号可能变）。

## 每条新测试：改前红在哪、改坏哪一行 → 哪条断言红

测试文件 `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs`（名字不含 layer0，全是内存稀疏盘上的快用例，整个二进制 0.04 秒）；单测在 `crates/singlefs-core/src/write_request_split.rs` 的 tests 模块。

**改前**（副本 `/tmp/claude-1000/impl-rev-a2a/prefix-copy`：crates 取 A1b 开工时的快照 `/tmp/claude-1000/impl-rev-a1b/base/crates`——我那 6 份源文件与它逐字节相同——加一份探针版测试：第 15、16 条那几条改成只判「拒了、拒之前一个字节都没写」，因为改前没有那几个成员）：`test result: FAILED. 2 passed; 12 failed`，日志 `/tmp/claude-1000/impl-rev-a2a/prefix-run.log`。红在哪（原样摘自日志）：

| 用例（改前版） | 改前红在 |
|---|---|
| 1 GiB 环 / 默认环长加一条记录 / 半条记录长的环 / 槽距 1 MiB / 槽距 1024 / 槽距 384 KiB + 512 / 根槽 8 KiB | 测试第 155 行 `assert_refused`：「…：mkfs 交回了 Ok」 |
| 100 MiB 的盘 | 第 177 行：`Some(BlockDevice(OutOfRange { offset: DeviceOffsetInBytes(822083584), length: 32768, device_size: 104857600 }))` |
| 根槽 256 字节 | `crates/singlefs-core/src/root_record.rs:77:9` 「根槽装不下根记录」（mkfs 中途的断言） |
| `a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit` | 「新建的空 inode 打得开: ExtentRecordCountDoesNotMatchTheFileSize { inode: InodeNumber(2), extent_records: 0, data_units_implied_by_the_file_size: 1, file_size_in_bytes: 0 }」 |
| `a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix` | 「链值不等的记录不进重放前缀（I-8.6）：恢复停在所选根」 left: `Some((InstanceGeneration(1), CheckpointTxg(4)))` |
| `slot_one_is_looked_for_at_the_slot_spacing_another_devices_slot_zero_records` | 「盘 0 的槽 1 在槽距 8192 处找到」 left: `DeviceIdentity(1)` |

**改后变异证红**：副本 `/tmp/claude-1000/impl-rev-a2a/work-copy`（同一份 A1b 快照的 crates + 我这 7 份文件，与主工作区逐字节相同，`cmp` 核过）；17 行追加进副本自己的 `crates/mutations.tsv`，经 `bash research/scripts/prove-red.sh --copy <副本> --memory 8G <crate> <名…>` 跑。基线红集为空：harness 那组基线 `test result: ok. 14 passed; 0 failed`，core 那组 `test result: ok. 124 passed; 0 failed`。17/17 抓到：`✓ 点名 16 条：跑了 16 条，跳过 0 条，跑的都抓到了`、`✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了`。

| # | 变异（名字省去前缀「实审 A2a」） | 必须红的用例 → 红在哪一条断言 | 同时红的（同一个二进制里） |
|---|---|---|---|
| 1 | 第 15 条：journal 环末端越过编译期单元区起点也不拒（`> unit_area_start` → `> unit_area_start * 2`） | `a_journal_ring_reaching_past_the_compiled_unit_area_start_is_refused_before_any_write` → 第 158 行 matches!「1 GiB 的环盖住实例表单元：None」 | `a_journal_ring_one_record_longer_than_the_default_is_refused_and_the_default_ring_is_not` |
| 2 | 第 15 条：环末端正好是单元区起点也当越过（`>` → `>=`） | `a_journal_ring_one_record_longer_than_the_default_…` | 默认几何上 mkfs 就被拒：另 7 条（两条第 21 条、两条第 25 条、第 37 条、根槽一样宽、区域首尾相接） |
| 3 | 第 15 条：短于一条记录的环不拒（`< JOURNAL_RECORD_BYTES` → `< JOURNAL_RECORD_BYTES / 4`） | `a_journal_ring_shorter_than_one_record_is_refused_before_any_write` → 第 212 行「半条记录长的环：None」 | 无 |
| 4 | 第 15 条：单元区越界改回按环长现算的起点判 | `a_unit_area_starting_past_the_device_end_is_refused_before_any_write` → 第 250 行「单元区起点在盘外：Some(BlockDevice(OutOfRange …))」 | 无 |
| 5 | 第 16 条：根槽装不下根记录不在开头拒（`< ROOT_RECORD_BYTES` → `< ROOT_RECORD_BYTES / 2`） | `a_root_slot_narrower_than_the_root_record_is_refused_before_any_write` → `root_record.rs:77:9`「根槽装不下根记录」 | 无 |
| 6 | 第 16 条：根槽宽过槽距不拒（`> 槽距` → `> 槽距 * 2`） | `a_root_slot_wider_than_the_slot_spacing_is_refused_and_one_as_wide_is_not` → 第 385 行「根槽 8 KiB、槽距 4 KiB：None」 | 无 |
| 7 | 第 16 条：固定结构重叠只认起点相同 | `system_configuration_slots_closer_than_a_slot_width_are_refused_before_any_write` → 第 281 行「槽距 1024：None」 | `root_ring_regions_longer_than_…` |
| 8 | 第 16 条：固定结构首尾相接也算重叠（`<` → `<=`） | `root_ring_regions_longer_than_the_region_step_are_refused_and_regions_that_just_touch_are_not` | 默认几何上两槽首尾相接也被拒：另 8 条 |
| 9 | 第 16 条：两两比漏掉排布上相邻的一对（`&extents[index + 1..]` → `.skip(index + 2)`） | `system_configuration_slots_closer_…` → 第 281 行「槽距 1024：None」 | 槽 1 落区域 0 那条（报成区域 0 / 区域 1）、区域那条 |
| 10 | 第 16 条：固定结构里只列系统配置槽 0 | `a_system_configuration_slot_one_landing_on_root_ring_region_zero_is_refused_before_any_write` → 第 281 行：报的是 `RootRingRegion { region: 0 }` 与区域 1，不是槽 1 与区域 0 | `system_configuration_slots_closer_…` |
| 11 | 第 21 条：只读挂载把没有 extent 条目的 inode 当写过内容 | `a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit` → 第 507 行 expect「新建的空 inode 打得开: ExtentRecordCountDoesNotMatchTheFileSize { … extent_records: 0, data_units_implied_by_the_file_size: 1 … }」 | 无 |
| 12 | 第 21 条：只读挂载把指着数据单元的 inode 当没写过 | `a_file_overwritten_with_empty_content_still_opens_with_its_one_data_unit` → 第 545 行 expect「…extent_records: 1, data_units_implied_by_the_file_size: 0…」 | 无 |
| 13 | 第 21 条：没写过的 inode 长度 0 也按除法给一个单元（`-p singlefs-core --lib`） | `write_request_split::tests::a_never_written_inode_of_length_zero_implies_no_data_unit_and_a_written_one_implies_one` → `write_request_split.rs:277:9` assert_eq | 无（lib 124 条里只红它） |
| 14 | 第 25 条：链值不等的记录照样进重放前缀（`Broken => break` → `Broken => {}`） | `a_journal_record_whose_back_chain_disagrees_…` → 第 664 行「链值不等的记录不进重放前缀（I-8.6）：恢复停在所选根」 | 无 |
| 15 | 第 25 条：前一条不在盘上也断前缀 | `a_journal_record_whose_previous_record_is_not_on_disk_is_replayed_without_judging_its_back_chain` → 第 704 行「前一条不在盘上：链判不了，txg 4 照样施加」 | 无 |
| 16 | 第 25 条：链值比较写反 | `a_journal_record_whose_back_chain_disagrees_…` → 第 642 行正对照「链没动：txg 4 由记录施加」 | 无 |
| 17 | 第 37 条：槽 0 不可择的盘不借池里别的盘槽 0 记的槽距（`.or(…)` → `.or(None)`） | `slot_one_is_looked_for_at_the_slot_spacing_another_devices_slot_zero_records` → 第 744 行「盘 0 的槽 1 在槽距 8192 处找到」 | 无 |

- 每条新用例至少被一行证过：14 条 harness 用例由 1、2、3、4、5、6、7、8、10、11、12、14、15、17 各证一次；9、16 是额外的，也跑过、也抓到。日志：`/tmp/claude-1000/impl-rev-a2a/prove-red-logs/002.log`–`016.log`（harness 第 2–16 行）、`prove-red-logs/001.log`（core 那一行，覆盖了 harness 第 1 行的同名日志）、`prove-red-logs-harness-first/001.log`（harness 第 1 行单独重跑）。第 5 行红在 core 的断言上：被测代码里那句是 `assert!`，不是 `debug_assert!`，release 下同样会红，没另跑 `--release`。
- 没写变异的改动：`journal.rs` 的 `record_offset`（`%` → `checked_rem(..).expect(..)`）与原式在所有输入上同一个结局（环槽数 0 都 panic），是等价改写，没有能单独证红的变异；它的前置条件由第 3 行钉在 mkfs 上。`judge_back_chain_…` 按盘上原样字节算、不按 `JournalRecord::to_bytes()` 重写：写者写出的记录两者逐字节相同（第 25 条用例开头断言了 `to_bytes() == record_bytes`），把它改成按重写字节算是等价变异，现有用例抓不到；要钉它得造一条头里读者不看的字节（算法类型、nonce、MAC、指针保留位）非 0、整条校验和重算过的记录，这一格与审阅第 29 条（读者不判保留位恒 0）相交，我没做。

## 停下交主 agent 的设计问题

1. **第 15 条没做的那一半：单元区起点随环长走。** D23（journal 的角色与格式） 已定项 19 ③ 逐字「单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）」。要做成它，同一个现算值得进这几处，前三处不在我的文件单里：`crates/singlefs-core/src/allocator.rs`（`unit_area_slots_of_device`、`DeviceFreeMap::index` / `is_free` / 三处从 `UNIT_AREA_START_SLOT` 起扫的落点，`DeviceFreeMap::new` 要多带一个起点）；`crates/singlefs-core/src/mount.rs`（`format_time_allocator` 建空闲图，A1b 在改）；harness 里照常量算的 `crash.rs:647`、`model.rs:919/955`、`history.rs:112-118` 与几份测试（B3a-2 / B3b 的文件）；在单里而我没动的：`make_filesystem.rs` 的 `INSTANCE_TABLE_SLOT` / `TREE_TABLE_GENESIS_SLOT`（`pub const`，`tests/common/mod.rs` 等在用）、`system_configuration.rs:390` 写进系统配置偏移 417 的那 8 字节（今天写常量）、`recovery.rs` 的 `allocation_records_fit_the_pool_geometry`（必须与分配器同一个起点，否则盘上记录判过了、进分配器照样 panic）。checker 已经按系统配置里那 8 字节读起点（`crates/singlefs-checker/src/image.rs:168`），不用跟着改。盘上字段与格式常量都不用动：系统配置里本来就有「单元区起点」这一格。**今天的状态**：环长 ≤ 768 MiB 照收（单元区照旧从 784 MiB 起，中间白放），> 768 MiB 按 `JournalRingPastTheCompiledUnitAreaStartUnsupported` 拒。
2. **第 37 条全池槽 0 都自证不过那一格。** 规格给的两条来源里「系统配置记的槽距」这一格拿不到（槽 1 的位置正是要找的东西）；「挂载探得的 io_min」要 `PoolReader` 或 `BlockDevice` 交得出 io_min——两个 trait 今天都没有这个动作，加动作要改 harness 里 8 处 `impl PoolReader`（`crash.rs:392/443`、`read_tally.rs:96`、`fault_injection.rs:378` 与四份测试）或全部块设备实现。可选的三条：(a) 给 `PoolReader` 加「这块盘探得的 io_min」，挂载按 D2（RAID 条带策略） 已定项 19 的式子算槽距去找；(b) 不探测：在 [4096, 根环基址 − 4096] 里按 512 字节一格扫，只收「自证过、且自己记的槽距正好等于它所在偏移」的那一槽（审阅 E 组第 9 条提的「按格式允许的几档逐个试」的一种）——代价是这一格最多每盘约 2040 次 4 KiB 读，风险是同一块盘上更早一次 mkfs 留下、槽距不同的旧槽 1 也自证得过（今天按 4096 试也有同一个风险）；(c) 维持现状。我没选，代码停在 (c)。
3. **第 16 条比的是整个设备内偏移空间**，不分哪块盘背哪个区域。后果：S × 槽距必须 ≤ 3 MiB（S = 8 时槽距 ≤ 384 KiB），io_min 取 1 MiB 的设备（mdraid chunk 1 MiB 这类）mkfs 会拒；只比同一块盘上真背根的区域（0 与 2 同在盘 0，间距 6 MiB）则放宽到 S × 槽距 ≤ 6 MiB，但 mkfs 在每块盘上清三个区域时清零的几段会互相盖住（今天那段注释说它们互不重叠、不加屏障）。规格写的是「三个根环区域两两不重叠」，我照字面取了严的那种。另外 mkfs 仍不核「槽距 = `slot_spacing_for(io_min)`」（A 组报告第 2 条 (a)），规格没点，没做。
4. **第 21 条的读侧口径**：规格原话「长度 0 时数据单元数取 0」。照字面对所有长度 0 的 inode 取 0，会打不开覆盖写成空内容的文件（它有一个声明长度 0 的数据单元与一条 extent 记录，`publish_overwrite(… content: &[] …)` 写得出来、用例里现写了一个）。我按「上段叶里有没有指着数据单元的条目」分两种口径。若要的是字面那种（空内容的写不再写单元），改的是写侧切分（`write_request_split.rs` 的 `.max(1)`）与发布路径，不止读侧。
5. **第 25 条的两处取法**，条款没逐字写、我取的：① 链值按盘上原样字节算（与 checker 同一个输入），代价是施加路径上每条要判的记录多读一槽（≤ 在飞上限条）；按解出来的字段重写再算，对写者写出的记录结果相同、不多读，但头里读者不看的字节非 0 时与 checker 判法分叉。② 两块盘上前一条的两份镜像：取 `device_identities()` 次序里第一份解得出、与扫环收下的那一条相同的——扫环也按这个次序收；checker 是逐盘判（每块盘上的记录配同一块盘上的前一条）。两份镜像逐字节相同时两种取法一样；不同时（只能是改出来的镜像）恢复看第一份、checker 两份都判。
6. **journal 环短于三条记录**：在飞上限 = 环槽数 ÷ 3 = 0，恢复一条记录都不施加（`replay_journal` 的 `take(in_flight_limit)`）；mkfs 今天照收。I-8.1（环几何够大） 管的是这一格，规格没点，我没拒。
7. `record_offset` 在环长 < 4096 时仍 panic（带了消息）：mkfs 拒了这种环；挂载时盘上读回来的环长不设界（审阅第 38 条，归 A3），可写挂载拿调用方参数还是盘上系统配置写记录归 A1b（审阅第 17 条）。

## checker 要跟着改什么（归 B2，我没动 checker）

- I-8.6 的判法 checker 早就有（`crates/singlefs-checker/src/walk.rs` 的 `judge_journal_back_chain`，此刻在第 4605 行起，B2 正在改这份文件，行号会漂）。恢复现在按同一条把链值不等的记录挡在前缀外，剩下三处口径差：
  1. checker 逐盘判（每块盘上的记录配同一块盘上的前一条）；恢复取 `device_identities()` 次序里第一份解得出、与扫环收下的相同的那份前一条。两份镜像逐字节相同时一样。
  2. checker 在「计数器 1」与「计数器 − 1 那一条属于别的实例」两种情形判链值恒 0；恢复的前缀里不会出现本实例第一条（前缀都接在所选根覆盖的那一条之后，同实例），只在「同实例前一条在盘上」时判，别的一律不判。推的（没造镜像量）：新实例接着「前缀末 + 1」写、而它的某一条没持久、那一槽还躺着上一条时间线的残留时，checker 的第 ③ 种情形会把下一条判成「本实例第一条、链应为 0」而报违例，恢复这时不判——要不要收，B2 定。
  3. `tree_table_pointer_of_the_version_the_next_mount_applies_first`（此刻第 3244 行起）取「最新根的 txg + 1、同实例、带提交标记、根槽没落盘」那条记录的树表指针来判 I-3.10，不看它的反向链；恢复现在遇到链值不等就不施加那一版。要对齐就在那里加「链值对得上前一条」这一条件（它文档里写着「这一版恢复会不会真施加这里不判」）。
- 第 37 条同一个回落：`crates/singlefs-checker/src/image.rs:261` 槽 0 解不出时按 `FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES` 读槽 1。恢复现在先借池里别的盘槽 0 记的槽距，checker 没跟上时，槽距 > 4096 而一块盘槽 0 坏了的镜像上，恢复多看得见那块盘的槽 1、checker 看不见。
- 第 15 条：checker 读单元区起点取系统配置偏移 417 那 8 字节（`image.rs:168`），与实现今天写的常量一致，不用改。

## 层 0、崩溃注入、故障注入可能改判的（都是推的，没跑）

- 反向链判定：写者每条记录的反向链都按前一条写出的字节算（`transaction.rs` 用 `back_chain_of(&previous.record_bytes)`），两份镜像写同一份字节，层 0 不生成撕裂态（审阅第 4 条）⇒ 枚举出的每个崩溃状态上，前一条要么在盘上且链对得上、要么不在盘上（不判）⇒ 恢复施加的前缀不变、层 0 与崩溃注入的钉值不变。改判的只会是改出来的镜像。
- 多出来的读：`replay_journal` 对每条要判的记录重读前一条那一槽（≤ 在飞上限条，默认几何 65536）。钉了挂载期 journal 环读次数的只有 `second_transaction_parallel_line_two_mounted_read.rs` 第 690 行附近（等于整扫两遍），那一池所选根就是最新根、前缀为空、一条都不判 ⇒ 不变。故障注入按「第 N 次调用」摆注入点的（`fault_injection.rs`、`second_transaction_supplement_three_fault_injection.rs` 的快档，`e158_root_choice_repair.rs` 的几格），恢复路径上的读多了、择系统配置的读换了先后，注入点可能落到别的读上：判的是「不 panic、没有已知红清单外的失败」，会不会翻只能跑了才知道。
- 择系统配置：每块盘读槽 0、槽 1 各一次不变（按盘数注入的 `second_transaction_step_three_formatted_pool.rs` 第 73 行那个「第 3 次读槽 0」不受影响）；跨盘先后变了。默认槽距 4096 的池上三种来源给出的槽距都是 4096，读的偏移不变。
- mkfs 新拒的几何：默认几何、`history.rs` 小盘（环 128 MiB）、E158 那三档（环 3 MiB、S 4 / 8 / 16、槽距 4096）、E142（pbs = io_min = 512）、真设备那条（pbs = io_min = 4096）按数值都过得了新判（按式子算的，除默认几何与 128 MiB 环之外没跑）。
- 只读挂载：没有 extent 条目的 inode 以前打开就报 `ExtentRecordCountDoesNotMatchTheFileSize`，现在打得开；仓里钉这个成员的唯一一条（`second_transaction_parallel_line_two_mounted_read.rs` 第 1292 行，4 条记录配 6 个单元大小）走的是「写过内容」那一臂，数不变。
- 要在提交时跑的：层 0 全量、崩溃注入、故障注入快档、门禁 59 号（连这 17 行一起）。

## 第 4 步那几样的末尾原样输出

开跑前 `ps` 看到别的会话在跑 cargo（`cargo build --offline --release -p singlefs-harness --bin e158_root_choice_repair`、`cargo test … -p singlefs-harness --lib`、`… --test second_transaction_supplement_two_commit_generated_fallback` 等），没有 qemu / fio / vm-bench / e152；都加了 `nice -n 19` 与 `capped.sh 4`，主工作区那次 build 等锁连编一共 2 分 07 秒。开工时主工作区 `mount.rs` 正被 A1b 改到一半、编不过（`missing field disagreeing_device_table` 等 3 个错），所以证红与 clippy 另在副本 `work-copy`（A1b 开工快照的 crates + 我这 7 份文件）上做；收尾时主工作区已经编得过，下面标「主」的是在主工作区跑的。

- 主，`cargo test --offline -p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode`（经内存包装 8G）：`test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`
- 主，`cargo test --offline -p singlefs-core --lib`（经内存包装 8G）：`test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.64s`
- 主，`cargo build --offline --all-targets`：退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 07s`；唯一的告警不在我的文件里：`warning: unused import: \`judge_location_entries_order\`  --> crates/singlefs-checker/src/walk.rs:26:35`（B2 在改的文件）。
- 主，`cargo fmt --all -- --check`：退出码 1，差异全在别人的文件：`singlefs-checker/src/walk.rs` 7 处、`singlefs-harness/src/crash.rs` 4 处、`src/segments.rs` 3 处、`tests/checker_narrow_invariants_and_abandoned_roots.rs` 14 处、`tests/entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs` 9 处、`tests/writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration.rs` 1 处；我这 7 份文件 0 处。副本 `work-copy` 上 `cargo fmt --check` 退出码 0、输出 0 行。
- `cargo clippy`（check.sh 那一套：`-D warnings` 加七条 `-D clippy::…`）：
  - 主，`--all-targets --all-features`：退出码 101，4 个错全在 `crates/singlefs-checker/src/walk.rs`（unused import、unused variable `previous_parent_unit`、very complex type、立即解引用的引用），末行 `error: could not compile \`singlefs-checker\` (lib test) due to 4 previous errors`；`-p singlefs-core --all-targets` 退出码 0；`-p singlefs-harness --test core_review_…` 退出码 101，因为要先查 checker：`error: could not compile \`singlefs-checker\` (lib) due to 4 previous errors`。
  - 副本 `work-copy`（checker 是 A1b 快照那一版），碰过文件强制重查：`-p singlefs-core --all-targets --all-features` 退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.84s`；`-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode --all-features` 退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.85s`。整仓 `--all-targets` 在副本上退出码 101，3 个 `shadow_unrelated` 全在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（3312、3880、3882 行，不是我的文件）。
- 登记给 implementation-writer 的门禁阶段（主工作区，`nice -n 19 bash .claude/gate.d/<文件>`）：
  - 33 号：退出码 1。`✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次`，点名 9 行：第 116、153、784、785、859 行（原文在 `crates/singlefs-harness/src/crash.rs`）与第 865、874、877、878 行（原文在 `crates/singlefs-checker/src/walk.rs`）——都不在我的文件里（B3a-2、B2 在改）。我另用脚本核过：表里原文落在我那 6 份源文件上的 99 行，改后每行仍恰好命中一次。我的 17 行没进主表，它们的锚点由 `prove-red.sh` 在副本上逐条核过（恰好命中一次才施加）。
  - 53 号：退出码 0，`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
  - 74 号（经阶段自己的内存包装，默认 8G）：退出码 0，`✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）`，六段步数 2708 / 1448 / 1448 / 4832 / 1724 / 4832。
  - 89 号：退出码 77，`⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`——按「本次未跑」记，不算通过。
  - 92 号：退出码 0，`✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个…`
  - 93 号：退出码 0，`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；…）`
  - 94 号：退出码 0，`✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`…`

## `git diff --stat -- crates litmus`（主工作区，原样；别的会话同时在改 crates/，这张表分不出谁改的，我的文件以「这一轮写过的文件」一节为准；新测试文件还没被 git 跟踪，不在表里）

```text
 crates/mutations.tsv                               |  579 +-
 crates/singlefs-checker/src/image.rs               |  174 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1373 +++-
 crates/singlefs-core/src/admission.rs              |  272 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   24 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  179 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   22 +-
 crates/singlefs-core/src/recovery.rs               |  493 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 3339 ++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2479 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  820 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  101 +-
 .../tests/checker_known_bad_images.rs              |  805 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  156 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  566 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  563 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  196 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  342 +-
 ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  258 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ...plement_two_instance_table_second_page_write.rs |    4 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
 ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 85 files changed, 24179 insertions(+), 10861 deletions(-)
```

## 删掉的副本与留下的材料

- 删了 `/tmp/claude-1000/impl-rev-a2a/prefix-copy`（1.6G，其中 target 1.5G）与 `/tmp/claude-1000/impl-rev-a2a/work-copy`（12G，其中 target 12G）。要复现证红：照 A1b 开工快照或主工作区拷一份副本，把 `mutations-append.tsv` 追加进副本的 `crates/mutations.tsv`，跑 `bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <16 个名>` 与 `… singlefs-core "实审 A2a 第 21 条：没写过的 inode 长度 0 也按除法给一个单元"`。
- 留着：`orig/`（我那 6 份源文件改前的样子）、`mutations-append.tsv`、各次跑的日志（`prefix-run.log`、`prove-red-logs*/`、`main-*.log`、`gate-*.log`）、`make_probe.py` / `make_rows.py`（造改前探针与变异行的草稿脚本）、`progress.md`。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。
- 没跑名字带 layer0 的测试、全量 `cargo test`、故障注入 / 崩溃注入 / 随机历史以外的测试二进制；只跑了我动到的两个二进制（新测试文件、`singlefs-core --lib`）与门禁 74 号里的随机历史（阶段自己跑的）。上面「可能改判的」一节全是推的。
- 第 15 条「单元区起点随环长走、实例表 / 树表 / 分配器用同一个现算值」没做（设计问题第 1 条）；第 37 条全池槽 0 都坏那一格没修（第 2 条）。
- 没改 checker（第 25 条对齐那半归 B2，要改的三处写在「checker 要跟着改什么」）。
- 没钉「链值按盘上原样字节算、不按重写的字节算」的变异（等价变异，见每条新测试一节末尾）。
- 没往 `crates/mutations.tsv` 写一行（派发提示要求），17 行在 `mutations-append.tsv` 等主 agent 追加。
- 没回扫 kb：C475（非默认环长下单元区起点取编译期常量） 那一行的「今天一次都没被走到」「mkfs 不拒」这类现状句，改后 mkfs 会拒环长 > 768 MiB 的那一半，kb 不归我写。
