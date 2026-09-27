# 实审 A3b 实现员报告：单元区起点随环长；重建判树表排序契约；mkfs 在飞上限 4 字节；读不出不无声放过的两处

写于 2026-09-27 JST（本机 UTC）。规格 `/tmp/claude-1000/impl-rev-a3b/spec.md`。在副本 `/tmp/claude-1000/impl-rev-a3b/work/` 里改，交 `patch/`。

## 一、结论

1. **第 15 条（C475）做完，一处来源**：`journal::slot_after_the_journal_ring(环长)` 是唯一的式子。
   - mkfs：`check_geometry` 按环长现算 `UnitAreaStart::following_the_journal_ring` 并把它交回给写的那一步。实例表写在起点、树表写在起点加 2，`MakeFilesystemOutput` 多一个字段 `unit_area_start`，另有 `instance_table_placement()` / `tree_table_genesis_placement()` 两个方法。`allocator_after_make_filesystem` 按这个起点建空闲图。
   - 起点不在 64 槽段边界上，在任何写之前拒成新成员 `MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`。
   - `JournalRingPastTheCompiledUnitAreaStartUnsupported` 删了：起点就是环末尾的下一个槽，环不会越过它。
   - 系统配置：`to_slot` 写偏移 417 时写 `slot_after_the_journal_ring(环长)`，写之前断言位置。新增 `unit_area_start_slot_recorded_in_the_slot`，读那 8 字节。
   - 读者（`recovery::system_configuration_values_this_reader_accepts`）：编译期上界（环末端 ≤ 784 MiB）改成三判，都报 `JournalRingBytesOutsideTheSupportedRange`，与 checker `journal_ring_bytes_lie_in_the_supported_range` 同一个判法：
     - 环末端不越过同一槽记着的 417 起点；
     - 在飞上限不为 0；
     - 在飞上限装得进 4 字节。
     另加两个新成员：
     - 417 那 8 字节 ≠ 环长现算的起点 ⇒ `UnitAreaStartNotTheSlotAfterTheJournalRing`；
     - 起点不在段边界上 ⇒ `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`。
   - `scan_journal` 没有提示时，逐槽列偏移只列到那块盘的末尾（理由见第三节 Q3）。
   - 恢复与挂载：
     - `allocation_records_fit_the_pool_geometry`、`placement_lies_in_the_unit_area_of_its_device`、`every_device_reaches_the_unit_area_start` 都多一个 `unit_area_start` 参数。
     - `rebuild_version`、`walk_to_file`、`allocation_records_under_root`、`allocation_records_of_version_without_file` 各多一份带起点的 `…_in_the_unit_area_starting_at`。原名保留，签名不变，改成先择一次系统配置现算起点（调用方是单外的用例与实验装置）。
     - 挂载、管理员回退、`recover()` 走带起点的那一份，起点取 `unit_area_start_of_the_chosen_system_configuration`（择到的那份系统配置按环长算；择的时候判过它与 417 相等、在段边界上）。
     - `mount.rs` 的 `rebuilt_allocator` 改成 `DeviceFreeMap::with_unit_area_start`。
   - harness：
     - `crash.rs` 的单元扫描候选下界、journal 扫描候选的环长，改成读这块盘系统配置槽 0 自述的 417 与 333。槽 0 自证不过时退到能罩住任何几何的那一档。
     - `model.rs` 模型自己一份式子：新构造 `after_make_filesystem_on_a_journal_ring_of`；原 `after_make_filesystem` 取默认环，它在单外的三处构造点签名不变。
     - `history.rs` 的三档小盘见第三节 Q1。
   - checker 没改。`crates/singlefs-checker/src/image.rs:226` `let unit_area_start_slot = read_u64(slot, 417);`，1 GiB 与 128 MiB 两条用例在 mkfs 之后、可写挂载之后都跑池级 checker，0 违例（第四节）。
   - 默认环 768 MiB 下起点仍是 50176。mkfs 到第一个文件两块盘整盘 CRC-32C 与改之前逐字节相同：基线副本现算 `name=a3b_default_ring_first_transaction_images_crc32 value=1783297687`，钉进用例。
2. **第 3 条（C11b 第 5 条）**：mkfs 在任何写之前拒成 `JournalInFlightRecordLimitWiderThanItsFourByteField { ring_bytes, in_flight_record_limit }`（紧跟「F 条记录」那一判）；读者同一格报 `JournalRingBytesOutsideTheSupportedRange`。改之前的现象：48 TiB 环、200 TiB 稀疏盘上 mkfs 清完环、写完单元与根，在 `to_slot` 的 expect 上 panic（变异 007 复现，第五节）。
3. **第 2 条（A3c Q-A 甲）**：`rebuild_version` 在 `tree_table_entries_each_kind_at_most_once` 之后判两道，都报 `InvariantViolated { invariant: "D8 已定项 8 排序契约", detail }`，常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`。
   - ① 盘上次序树 ID 严格升序，detail「树表条目不按树 ID 严格升序」。
   - ② 七棵树按发号次序树 ID 严格升序，detail 写明次序。
   - 为什么要第 ② 道见第三节 Q2：探针一过得了 ①。
   - 两个探针的用例都钉「可写挂载在任何写之前拒、两块盘逐字节不变」。`transaction.rs` 那条断言没动：它此后依赖的读者判定就是 `recovery::rebuild_version_in_the_unit_area_starting_at` 里这两道（①② 各管一种坏法）。**那条断言的消息要改成指到这两处，归改 `transaction.rs` 的人。**
4. **第 4 条（C554 乙 Q6）**，照乙的形态收，被攻过零轮：
   - 可写挂载：`rebuilt_allocator` 里算生效 F 不再自己读实例表，用影子账那一张表（`instance_table_of_the_newest_root_read_at_most_twice` 读的、读不出重读一次）。读不出照乙已有的 `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)` 拒可写，取号之前。新函数 `effective_rollback_floor_under_the_newest_roots_table`。
   - 挂着时抬 F（`raise_the_floor_through`），在动分配器与任何写之前拒这一次抬，两个新 `RecoveryFailure` 成员，经 `MountError::Recovery` 交出：
     - 算生效 F：`effective_rollback_floor_rereading_the_newest_instance_table_once`，读根环、择最新根、读表；读不出立即重读一次；仍读不出报 `InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`。
     - 回收门槛与影子账：`readable_roots_rereading_unreadable_root_ring_slots_once`，读不出（`BadRootRingSlotReading::Unreadable`）的槽立即重读一次；仍读不出报 `RootRingSlotStillUnreadableAfterOneReread { ring_slot }`。读得出、自证不过的槽照旧当没有根。
   - 为什么不加 `MountError` 成员、名字怎么起：见第三节 Q4。
5. 交回前验证：
   - 新测试二进制 14 条全绿。
   - core `--lib` 132 条全绿，harness `--lib` 97 条全绿。
   - fmt 过。
   - clippy（带 `CODE_DISCIPLINE_LINTS`）与 `build --all-targets` 只红在单外的 `core_review_geometry_back_chain_and_empty_inode.rs` 两处，引了删掉的成员。
   - 74 号改后红 3 条：基线就红 1 条，新增红 2 条，都是小盘上按步数钉的用例（第三节 Q1、第六节）。
   - 33 号在改锚点之后绿。
   - 证红：新测试 14 条、单元扫描候选 1 条都抓到。其余见第五节。

**推翻条件**：
- 在副本上把第五节任一行变异施加回去，它点名的测试不红。
- 默认环下 mkfs 到第一个文件整盘 CRC 不是 1783297687。
- `recover` 在 1 GiB 或 128 MiB 环的池上读不回第一个文件。
- 读者在 417 与环长对不上时照常挂上。

## 二、这一轮写过的文件

副本 `/tmp/claude-1000/impl-rev-a3b/work/`，补丁 `/tmp/claude-1000/impl-rev-a3b/patch/crates.patch` 就按这些生成：

- `crates/singlefs-core/src/system_configuration.rs`：写 417 按环长算；新增 `UNIT_AREA_START_SLOT_OFFSET`、`unit_area_start_slot_recorded_in_the_slot`、`journal_in_flight_record_limit_fits_its_four_byte_field`。
- `crates/singlefs-core/src/make_filesystem.rs`：起点现算并交回给写的那一步；两个新成员，删一个成员；`MakeFilesystemOutput::unit_area_start` 与两个落点方法；`allocator_after_make_filesystem` 按起点建图。
- `crates/singlefs-core/src/recovery.rs`：
  - 读者三判加两个新成员；两个 `RecoveryFailure` 新成员；
  - 几何判带起点；四个带起点的公开入口，原名做包装；`scan_journal` 列到盘尾为止；
  - 树表排序两道；生效 F 三个入口（原函数、按给定表算、重读一次）；读根环重读一次。
- `crates/singlefs-core/src/mount.rs`：建空闲图与判落点都带起点；生效 F 用影子账那张表；抬 F 两处改成重读一次的版本；回退与 inode 号水位走带起点的那一份。
- `crates/singlefs-harness/src/crash.rs`：扫描候选按盘上系统配置槽 0 取环长与起点。
- `crates/singlefs-harness/src/model.rs`：模型的单元区起点随环长。
- `crates/singlefs-harness/src/history.rs`：两个新 `RecoveryFailure` 成员的分支；三档小盘环 6 MiB、盘宽 = 起点 + 单元区槽数；模型按环长建。
- 新建 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`（14 条用例）。
- `crates/mutations.tsv` 的改动走补丁目录的三个文件，不进 `crates.patch`：
  - 追加 16 行，名字以「实审 A3b」开头（`patch/mutations-append.tsv`）；
  - 整行替换 11 行（`patch/mutations-replacements.tsv`，33 号锚点，第五节）；
  - 删 2 行（`patch/mutations-delete.txt`）：「实审 A2a 第 15 条：journal 环末端越过编译期单元区起点也不拒」「实审 A2a 第 15 条：环末端正好是单元区起点也当越过（默认环被拒）」。它们守的那一判（mkfs 的编译期上界）已经不存在；点名的用例在单外的 `core_review_geometry_back_chain_and_empty_inode.rs`，要跟着改（第四节）。

## 三、停下交主 agent 的设计问题与取法

**Q1 `history.rs` 三档小盘：环从 128 MiB 改成 6 MiB，盘宽改成「起点 + 单元区槽数」（这是我做的取法，交主 agent 核）。**
规格设想的是「小盘（128 MiB 环）起点变、钉值跟着变」。这条路走不通：
- 起点随环长之后，128 MiB 环的起点是 9216。三档小盘今天的盘宽是 `(50176 + N) × 16 KiB`（约 790 MiB），照旧的话单元区有四万多槽。「单元区 384 / 256 / 240 槽」这三档（`UnitAreaOf*Slots`）要逼近的单元区墙就走不到了。
- 盘宽改成 `9216 + N` 槽（约 150 MiB），mkfs 的「环 ≤ 容量 ÷ 4」又拒 128 MiB 的环。
- 我取 6 MiB 环：起点 1024 + 384 = 1408 = 22 × 64，在段边界上。三档盘宽 1792 / 1664 / 1648 槽，最窄那档 25.75 MiB，环不超过它的四分之一。7 MiB 在 256、240 两档超了。在飞上限 512 条。
- 代价（推的，没量）：
  - 分配记录树按绝对槽号按位置寻址，叶罩 812 槽。单元区从 [50176, 50560) 挪到 [1408, 1792)，跨的叶与层级都变了，每次发布重写几个节点跟着变，所以小盘上「第几步撞墙」的钉值全变。
  - 环只有 1536 个记录槽，长历史会绕环。
- 74 号实测：快档与各取样点那几段都过；新增红 2 条，都是这一类钉步数的用例（第六节原样）。`second_transaction_supplement_three_random_history.rs` 不在我的单里，没改。另一条路：给这几档另立一个不受「环 ≤ 容量 ÷ 4」约束的盘宽，条款不许，我没走。

**Q2 树表排序：第 ② 道（发号次序）是我加的，规格只写了第 ① 道。**
- 探针一互换 livelist 与稀疏旁表两条的种类之后，盘上次序仍按树 ID 升序，只判第 ① 道拦不住。写者（`crates/singlefs-core/src/transaction.rs:6507`）按种类的固定次序装条目、断言升序，照抄上一版的号，所以降序、panic。
- 依据是 D8 已定项 8 ②「八棵树的号从水位起连号发，次序照格式常量 11..18 那一组」。写者每次带文件的发布都断言这一格，本实现写出的镜像恒满足它。
- 这一道要不要立成不变量、`walk_to_file` 与 checker 要不要同步判，交主 agent / 书记员。今天只在 `rebuild_version` 判，冷走读与 checker 不判。不变量名用规格给的「D8 已定项 8 排序契约」，立号之后换 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT` 那一个常量。

**Q3 读者的环长上界：从编译期常量改成三条，都是我的取法，条款没写（读者一侧的界本来就「条款没有写，实审 A3a 的报告给了要补的原句」）。**
- 「环末端 ≤ 417 记着的起点」照 checker 的判法。「在飞上限装得进 4 字节」防的是挂载之后 `to_slot` 写这 4 字节时 panic。「417 = 环长现算的起点」是 D3 已定项 10 ④「第一版 = 环末尾的下一个槽」的读者形态。
- 这三条都不按盘的字节数判：按盘判会让单外 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 那条「700 MiB 盘报 `DeviceEndsBeforeTheUnitAreaStart`」换成别的成员。所以一块盘上两处自洽的 8 字节仍能把环长定到约 48 TiB。
- 扫环（`scan_journal`）没有提示时，逐槽列偏移只列到盘尾，扫环的内存由盘宽定。盘尾之后的槽本来就读不出，读回结果不变。
- 写记录按环长取模，在可写挂载里有 `every_device_reaches_the_unit_area_start`（起点 ≤ 盘尾）挡着。
- 要不要在读者一侧也按「环 ≤ 容量 ÷ 4」判，交主 agent。

**Q4 新错误成员为什么不加在 `MountError` / `StillUnreadableAfterOneReread` 上。**
这两个枚举在单外的 `crates/singlefs-harness/src/model_comparison.rs` 里被穷举 match（第 742–757 行、第 826、862 行），它在 harness 的库里。加成员整个 harness 编不过，我的新测试也跑不起来。所以：
- 可写挂载那一格复用乙已有的 `InstanceTableOfTheNewestRootForTheShadowLedger`：算生效 F 用的就是影子账那一张表。
- 抬 F 两格加成 `RecoveryFailure` 的成员，同一族起名（`…StillUnreadableAfterOneReread`），经 `MountError::Recovery` 交出。`RecoveryFailure` 只在我单里的 `history.rs` 被穷举，已补分支。
- 模型对拍把 `MountError::Recovery(_)` 记成 `Unexplained`。要是要它们与乙那一族同级（`MountError::NewerStateStillUnreadableAfterOneReread` 的新子成员），要同时改 `model_comparison.rs` 第 742 行那个 match 与 `first_transaction_on_device.rs`，交主 agent 另派。
- `first_transaction_on_device.rs`、E158 bin 的穷举 match 这次没有要加的分支：`MountError` 没加成员；`MakeFilesystemError` 在它们里面没有穷举 match，现查 `grep -rn 'MakeFilesystemError::' crates` 只在用例里有。

**Q5 公开入口的原名改成「先择一次系统配置」（`rebuild_version`、`walk_to_file`、`allocation_records_under_root`、`allocation_records_of_version_without_file`）。**
- 签名不变，是为了单外几十处调用点（用例、E156、E158）不改。
- 代价：这四个入口多读每块盘两个系统配置槽。系统配置择不到时报择系统配置的错，改之前照走。
- 产品路径（挂载、回退、`recover`）走带起点的那一份，不多读。
- 带读故障包装的读者（E158 的 `wrapped`、fault injection）调原名时读序多了几读，推的，没量。

**Q6 环短于 1 MiB 的那一族用例今天被 mkfs 拒了（照规格「起点不在段边界上在 mkfs 拒」）。**
- 在飞上限 1、2 这种小环（3、6 条记录长）的起点落在 1025、1026，不在段边界上。它们在单外用例里被用来测「一次发布的记录条数上限」：`core_review_unit_area_start_and_publish_limits.rs` 的六条、三条环，`core_review_tree_table_duplicates_and_slot_one_search.rs` 的「恰好三条放行」。
- 这些用例要换环长（1 MiB 起、在飞上限 85），或换别的办法造小上限。要不要为测试留一个「起点向上取整到段边界」的口子，条款没写，交主 agent。

**Q7 还剩两处读不出就无声放过，不在规格点名的两处里，没改。**
- `transaction.rs:368` 写系统配置前算生效 F 调的是原 `effective_rollback_floor`（C577 在改 `transaction.rs`）。
- `mount.rs` 管理员回退的候选判（`rollback_candidate`，第 4266 行附近）也调原函数。`RollbackError` 同样在 `model_comparison.rs` 里被穷举，加成员要一起改。
- 原函数的文档注释里已写明这两处之外的去处。

**Q8 E158 bin（3 MiB 环）自己拼的分配器与 mkfs 对不上了（单外，没改）。**
- `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:449-460` 用 `DeviceFreeMap::new` 加 `INSTANCE_TABLE_SLOT` 建 mkfs 同一个进程的分配器。
- 3 MiB 环的起点现在是 1216 = 19 × 64，mkfs 把实例表写在 1216，这个分配器却按 50176 记。E158 的装置行为会变。
- 改法：换成 `allocator_after_make_filesystem(&parameters, &devices, &genesis)`，或用 `genesis.instance_table_placement()` 与 `DeviceFreeMap::with_unit_area_start(…, genesis.unit_area_start)`。它的跑前登记与产物要不要重跑，交主 agent。
- `second_transaction_supplement_three_fault_injection.rs:595` 同样手拼（默认 4 GiB 盘不受影响，环境变量选小盘那一档受影响）。

**Q9 「按盘上那 8 字节建空闲图」的形态。**
- `SystemConfiguration` 的内存表示不带 417（加字段会动单外几处结构体字面量：`transaction.rs:381`、`system_configuration_mutability_classes.rs`）。
- 所以读者择的时候判「417 = 环长现算的起点」。挂载建图按环长现算，与 417 那 8 字节是同一个数。

## 四、第 1 条盘出的、单外要跟着改的测试（没改；前两条实测，其余静态推的）

- **编不过**：`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:161`、`:186` 引了删掉的 `JournalRingPastTheCompiledUnitAreaStartUnsupported`（build / clippy 原样见第六节）。同一文件第 146 行起 3 条用例的前提都变了：1 GiB 环现在放行；默认环加一条记录现在报 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`；100 MiB 盘、16 MiB 环现在起点 2048 槽，放行。
- **实测红（只换钉值）**：`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:623`。实例表那一片的读次数 `(5, 3)` 变成 `(4, 2)`：生效 F 不再自己读一遍实例表。证红基线原样：`left: (4, 2)`、`right: (5, 3)`。
- **实测红（74 号）**：`second_transaction_supplement_three_random_history.rs` 的 `writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged` 与 `seed_4000000204_raising_the_floor_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device`。小盘上撞墙的步数变了，原样见第六节。
- **静态推的**（没跑，按「用了小盘三档 / 非默认环 / 手拼 50176 的分配器」搜出来）：
  - `core_review_unit_area_start_and_publish_limits.rs`：六条、三条环 mkfs 被拒。
  - `core_review_tree_table_duplicates_and_slot_one_search.rs:821` 那一段「恰好三条放行」。
  - `second_transaction_admission_raises_the_floor_before_refusing.rs:427`：`(UNIT_AREA_START_SLOT + 16) × 16 KiB` 的盘配 6 MiB 环，单元区不再是 16 槽。
  - `admission_checkpoint_cost_per_device_paths.rs:59`、`:861`：128 MiB 环与 `UNIT_AREA_START_SLOT + N` 的盘宽。
  - `second_transaction_allocation_record_tree_geometry_of_writer_and_reader.rs:36`：128 MiB 环，起点 9216。
  - `second_transaction_supplement_two_root_ring_turn_in_one_mount.rs:208`：小盘一档。
  - 用 `HistoryDeviceWidth::UnitAreaOf*` 的其余几份：`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`、`a_floor_raise_refused_for_space_counts_as_short_of_space.rs`、`admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`、`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`、`second_transaction_supplement_two_admission_formula.rs`、`second_transaction_supplement_three_fault_injection.rs`（小盘档）、`second_transaction_supplement_three_crash_injection.rs`（崩溃枚举，标了 ignore）。
- **名字带 layer0 的与崩溃枚举用例**：层 0 各流都用默认环，起点与扫描候选逐项不变（候选下界读盘上 417 = 50176、环长 768 MiB，与改之前同一个数），钉的数不变（推的，没跑）。`second_transaction_supplement_three_crash_injection.rs` 里用小盘档的那几条会变。

## 受影响的层 0 流与崩溃枚举用例

没改 checker（`crates/singlefs-checker/src/` 一处没动）。层 0 各流用默认 768 MiB 环：
- mkfs 与第一个事务的字节不变（整盘 CRC 钉住）；
- `crash.rs` 的扫描候选在默认环下与改之前同一份（下界 50176、环长 768 MiB）；
- 写表、段序列、录制流不变。

所以层 0 钉值不受影响（推的，没跑层 0）。崩溃枚举用例里用 `HistoryDeviceWidth::UnitAreaOf*` 小盘档的会变（第四节）。

## 五、证红：改坏哪一行 → 哪条断言红（`research/scripts/prove-red.sh`，副本 `work/`，内存上限 8G，线程上限 4）

新测试 14 条，每条一行变异证过（`prove-red-run-1.log`、`prove-red-run-2.log`；日志 `prove-logs/001–015.log`、`prove-logs-2/001.log`）。基线（不改源码）14 条全绿（`prove-logs-2/baseline.log`：`test result: ok. 14 passed; 0 failed`）。表里「同时红」是同一个二进制里跟着红的测试。

| 变异（`patch/mutations-append.tsv` 的名字，前缀「实审 A3b」省略） | 点名的测试 | 同时红 |
|---|---|---|
| 第 15 条：mkfs 把树表第 0 版写到起点加 4 | the_default_journal_ring_keeps_…_bytes_unchanged（整盘 CRC 断言） | — |
| 第 15 条：系统配置偏移 417 改回写编译期常量 50176 | a_one_gibibyte_journal_ring_…（417 = 66560 断言） | 128 MiB 那条 |
| 第 15 条：可写挂载建空闲图改回默认环的起点 | a_128_mebibyte_journal_ring_… | 1 GiB 那条 |
| 第 15 条：mkfs 不按环长现算起点 | a_journal_ring_whose_next_slot_is_off_the_cluster_segment_boundary_… | 1 GiB、128 MiB 两条 |
| 第 15 条：读者不判偏移 417 是不是环长现算的起点 | a_system_configuration_recording_a_unit_area_start_other_than_… | — |
| C11b 第 5 条：读者不判在飞上限装不装得进 4 字节 | a_system_configuration_whose_in_flight_limit_overflows_… | — |
| C11b 第 5 条：mkfs 不判在飞上限（写系统配置那一步 panic） | a_journal_ring_whose_in_flight_limit_overflows_four_bytes_… | — |
| Q-A 探针一：重建不判七棵树按发号次序升序 | a_tree_table_whose_two_entries_swapped_their_kinds_…（改后 panic 在写者装树表的断言上） | — |
| Q-A 探针二：重建不判盘上次序严格升序 | a_tree_table_whose_two_entries_carry_the_same_tree_identifier_…（detail 换成发号次序那一句） | — |
| Q6：生效 F 重读仍读不出改回「不按表滤」（抬 F 照做） | raising_the_floor_while_the_newest_roots_instance_table_stays_unreadable_… | the_effective_floor_rereads_… |
| Q6：同上（算 F 那一段本身） | the_effective_floor_rereads_the_newest_roots_instance_table_once | raising_the_floor_while_the_newest_roots_… |
| Q6：重读那一遍读出的表也不认 | the_newest_roots_instance_table_read_on_the_one_reread_lets_the_floor_be_raised | the_effective_floor_rereads_… |
| Q6：根槽重读仍读不出改回「当没有根」 | raising_the_floor_while_a_root_ring_slot_stays_unreadable_… | a_root_ring_slot_unreadable_once_… |
| Q6：读不出的根槽不重读 | a_root_ring_slot_unreadable_once_is_read_on_the_reread_… | — |
| 第 15 条：崩溃镜像的单元扫描候选仍从 50176 起 | a_128_mebibyte_journal_ring_…（第一次没红：checker 少扫几槽不判红；补了「候选从 417 起、罩住实例表」的断言后抓到） | — |
| 第 15 条：模型的单元区起点改回 50176 | unit_area_wall_sampling_on_small_devices_…（随机历史二进制） | **没证，留给 59 号**：那个二进制基线就红 1 条（`crash_recovery_abandoning_a_newest_root_…`），prove-red 会在基线那一步停下 |

33 号锚点改过的 11 行（`patch/mutations-replacements.tsv`），各跑一次（`prove-red-run-3/4/5.log`）：
- 抓到 8 行：「步 4：被抛弃根的树表读不出时不计数」「实三 …inode 号水位…」「A3a 第 32 条」两行「A3a 第 35 条」「A3a 第 38 条」「K1 × 模型 …」「分配记录的几何判：不判槽号在不在单元区起点之上」。A3a 第 38 条那一行改后换红法：417 仍是 50176、环 2 GiB，拿掉「环末端 ≤ 417」之后下一判报 `UnitAreaStartNotTheSlotAfterTheJournalRing`，成员对不上，照红。
- **基线就没红（与这次改动无关）**：「步 4：影子账把被抛弃根账里已释放的落点也隔离」「步 5：抬 F 之后不按新候选集重算影子账」。点名的 `raising_the_floor_to_the_first_release_generation_…` 没红。换到基线副本 `baseline/`（主工作区开工时的代码、原锚点）上跑同样不红（`prove-red-baseline-1.log`：`✗ 点名 2 条：跑了 2 条 … 有没红`）。照写，交主 agent（mutation-triage）。
- **没证**：「代码审阅第 22 条：…照改之前按空表往下走」，它点名的那一条用例改后红（第四节第二条，读次数钉值变了），prove-red 基线那一步停下。

## 六、交回前的验证（末尾原样；副本 `work/`）

- `cargo test -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over`（经 8G 内存包装）：`test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.79s`
- `cargo test -p singlefs-core --lib`：`test result: ok. 132 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s`
- `cargo test -p singlefs-harness --lib`：`test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 633.61s`
- `cargo fmt --all -- --check`：退 0。
- `cargo clippy --keep-going --all-targets --all-features -- -D warnings`，加 `CODE_DISCIPLINE_LINTS` 七条：退 101，只有 `core_review_geometry_back_chain_and_empty_inode.rs` 这一个目标编不过（单外，第四节）：
  `error[E0599]: no variant named `JournalRingPastTheCompiledUnitAreaStartUnsupported` found for enum `singlefs_core::make_filesystem::MakeFilesystemError``（2 处）；
  `error: could not compile `singlefs-harness` (test "core_review_geometry_back_chain_and_empty_inode") due to 2 previous errors`。其余目标 0 告警。
- `cargo build --offline --all-targets --keep-going`：退 101，同一个目标同两处，别的都编过。
- 门禁（登记给我的；副本不是 git 仓，带 `SINGLEFS_GATE_FULL=1`）：
  - 33 号：锚点改完之后退 0，`✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1316 条的原文各命中源码一次；…`。改锚点之前红 13 行，是这次改动搬走的原文，处置见第五节。
  - 53 号：退 0，`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`。
  - 92 号：退 77（本次未跑），`! /tmp/claude-1000/impl-rev-a3b/work 不是 git 仓，本阶段跳过`。
  - 94 号：退 0，`✓ checker 与实现只共享常量模块 `singlefs-format`（…checker 的 4 份源码零处引 `singlefs_core`…）`。
  - 93 号：退 0，`✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（…扫了 57 个 .rs…）`。
  - 89 号：退 77（本次未跑），末行 `「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：…`，与这一件无关。
  - 74 号：退 1。改后 `test result: FAILED. 21 passed; 3 failed; 2 ignored; …; finished in 96.36s`，红 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_…`、`seed_4000000204_raising_the_floor_on_narrow_devices_…`、`writable_mount_whose_own_publishes_find_no_placement_…`。基线副本同一跑法 `test result: FAILED. 23 passed; 1 failed; 2 ignored; …; finished in 76.70s`，只红第一条。新增红的两条原样（`random-history-work-3.log`）：
    - `left: Applied(Mounted { instance: InstanceGeneration(2), publishes: 3, … }) right: Refused { member: "MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)" }`
    - `种子 4000000204 第 31 步：这一串的第二次在预演里取不到落点，一次都不发（历史停在：Completed） left: Some(Applied(RaisedFloor { new_floor: CheckpointTxg(20), publishes: 3, reclaimed_placements: 22, … })) right: Some(Refused { member: "MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(publish 2 of 3, PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))" })`
    都是小盘撞墙的步数变了（第三节 Q1）。快档与四个取样点那几段都过，模型对拍没有新签名。
- 补丁对主工作区 `git apply --check patch/crates.patch`：退 0。`python3 research/scripts/apply-writer-patch.py <patch> --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1335 行`。主表此刻 1321 行，开工时 1307 行，别的会话同时追加了。

## 七、`git diff --stat -- crates litmus`（主工作区，原样）

这次改动没进主工作区，全在 `patch/crates.patch` 里。下面这张表是别的会话此刻在主工作区的改动，照原样贴，分不出谁改的；这一轮写过哪些文件以第二节为准。

```
 crates/mutations.tsv                               |   265 +-
 crates/singlefs-checker/src/image.rs               |   323 +-
 crates/singlefs-checker/src/lib.rs                 |   103 +-
 crates/singlefs-checker/src/walk.rs                |   159 +-
 crates/singlefs-core/src/admission.rs              |   405 +-
 crates/singlefs-core/src/allocator.rs              |   308 +-
 crates/singlefs-core/src/code_two_tree.rs          |   131 +-
 crates/singlefs-core/src/inode_tree.rs             |    33 +-
 crates/singlefs-core/src/journal.rs                |    22 +-
 crates/singlefs-core/src/mount.rs                  |   612 +-
 crates/singlefs-core/src/mounted_session.rs        |    30 +-
 crates/singlefs-core/src/pointer.rs                |   168 +-
 crates/singlefs-core/src/recovery.rs               |  1015 +-
 crates/singlefs-core/src/transaction.rs            |   357 +-
 crates/singlefs-core/src/unit.rs                   |    38 +-
 crates/singlefs-harness/src/bad_disk_input.rs      |   338 +-
 .../src/bin/e156_allocation_basis_counts.rs        |    10 +-
 .../src/bin/e158_root_choice_repair.rs             | 11827 ++++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |   159 +-
 crates/singlefs-harness/src/crash.rs               |   936 +-
 crates/singlefs-harness/src/crash_injection.rs     |    22 +-
 crates/singlefs-harness/src/fault_injection.rs     |    10 +-
 crates/singlefs-harness/src/history.rs             |    93 +-
 crates/singlefs-harness/src/layer0_progress.rs     |   720 +-
 crates/singlefs-harness/src/model.rs               |   332 +-
 crates/singlefs-harness/src/model_comparison.rs    |   450 +-
 .../admission_checkpoint_cost_per_device_paths.rs  |   539 +-
 .../tests/checker_known_bad_images.rs              |   205 +-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |     8 +-
 crates/singlefs-harness/tests/common/mod.rs        |   311 +
 .../singlefs-harness/tests/common_admission/mod.rs |     3 +-
 .../crash_enumeration_sharded_across_processes.rs  |    23 +-
 .../singlefs-harness/tests/instance_acquisition.rs |    11 +-
 .../tests/publish_order_matches_litmus.rs          |    12 +-
 ...n_admission_raises_the_floor_before_refusing.rs |   276 +-
 ...inside_the_floor_raise_pushed_by_the_session.rs |    14 +-
 .../tests/second_transaction_step_five_reuse.rs    |     2 +-
 .../tests/second_transaction_step_one_overwrite.rs |    29 +-
 ...second_transaction_step_three_formatted_pool.rs |   504 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   267 +-
 ...transaction_supplement_three_crash_injection.rs |     4 +-
 ..._transaction_supplement_three_random_history.rs |   333 +-
 ...second_transaction_supplement_two_tree_split.rs |    34 +-
 ...d_transaction_supplement_two_unequal_devices.rs |    56 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |    44 +-
 ...ion_slot_is_overwritten_only_after_a_barrier.rs |    11 +-
 46 files changed, 19834 insertions(+), 1718 deletions(-)
```

## 八、没做什么

- 没走三方对抗，四件都被攻过零轮。没提交，补丁没打进主工作区（交 `patch/`，由主 agent 用 `research/scripts/apply-writer-patch.py` 打）。
- 层 0、崩溃枚举用例、QEMU、herd7、crates 变异整表（59 号）、全量 `cargo test`、`check.sh`、`gate.sh` 整轮都没跑，归提交时的 `crash-verifier` 与整轮门禁。
- 单外文件一个没改：`core_review_geometry_back_chain_and_empty_inode.rs` 编不过、第四节那一串钉值、E158 与故障注入里手拼的分配器、`model_comparison.rs`、`transaction.rs` 断言消息。第四节列的静态推断没有逐个跑实证。
- 没写 kb：「D8 已定项 8 排序契约」立不变量号、读者一侧环长上界的条款、小盘几何，都交主 agent。
- 模型那一行变异、`代码审阅第 22 条` 那一行变异没证（第五节写了原因），留给 59 号。「步 4 / 步 5」两行基线就没红，照写。
- 新测试在 debug 下跑；没有 `debug_assert` 抢先红的情况（证红日志里红的都是用例自己的断言或点名的 panic）。

## 九、草稿目录清理

删了副本 `/tmp/claude-1000/impl-rev-a3b/work`（20G，大都是它自己的 target）、`/tmp/claude-1000/impl-rev-a3b/baseline`（4.0G，基线副本，算过整盘 CRC、跑过基线 74 号与基线证红）、`/tmp/claude-1000/impl-rev-a3b/orig`（9.1M，开工时 `crates`、`litmus` 的原件，拿来生成补丁）。

留下的都在 `/tmp/claude-1000/impl-rev-a3b/` 下，合计约 700K，是报告里引的日志：
- `patch/`；
- `prove-logs*/`、`prove-red-*.log`；
- `gate*.log`、`random-history-work-3.log`、`probe-baseline.log`（基线 CRC 的原样输出）；
- `build-*.log`、`clippy-*.log`、`run-*.log`、`progress.md`。
