# 实审 A3a 实现员报告：坏盘输入的 panic 面（代码审阅第 24 / 29 / 32 / 33 / 35 / 38 条、C476 落在 core 读者这一侧的那几处）

写于 2026-09-27。规格 `/tmp/claude-1000/impl-rev-a3a/spec.md`。在副本 `/tmp/claude-1000/impl-rev-a3a/work` 里改、交补丁 `/tmp/claude-1000/impl-rev-a3a/patch/`。

## 一、结论

- 规格表六行都做了：每处先造坏镜像看改之前 panic（或静默收下），再改成带名字的成员往上交；新测试文件 31 条用例、core 单测 1 条，每条一行变异，副本里逐条证红（第六节）。
- 第 33 / 24 条照用户定案「重建时把已经读到的节点照冷走读那套判全」：`recovery::rebuild_version` 与 `walk_to_file` 调同一组判定函数（从 `walk_to_file` 里抽出来的 12 个，一处定义），四棵树与中央映射树都按 `EveryHeaderAgainstItsReference` 读。不多读一个单元。
- 另坐实 C476 普查「缺口」表里 `pointer.rs` 那一行（I-2.5 升序断言）**走得到**：树表条目或第 0 代根里一条逆序的指针，可写挂载在 `NodePointer::write_to` 的断言上 panic（副本里探针实测，第三节 C476 那一格）；改成重建时按 I-2.5 判。
- **判全带来三处别人的测试要跟着改**（不在我的清单里，没改，第七节给原文）：`second_transaction_supplement_two_tree_split` 1 条、`second_transaction_supplement_three_bad_disk_input` 1 条的期望成员变了（可写挂载现在报的与冷走读相同，都在任何写之前拒）；变异表里 252、256 两行（`records.rs` / `transaction.rs` 的锚点）经这条测试再也红不了，要重定向或删。`second_transaction_step_three_second_instance` 那一条红在基线副本上同样红，不是这一件带来的。
- `lib.rs` 不用加导出行（各模块本来就 `pub mod`）；新成员让 `singlefs-harness/src/history.rs` 的穷举 match 编不过，要加的两臂原文在第八节。

**什么现象会推翻**：主工作区打上补丁、补上第八节两臂之后，第六节任一行在门禁 59 号里没红；或者某条合法历史（层 0、崩溃注入、随机历史）的可写挂载被这一件新加的判定拒掉（那说明冷走读的某条判定对重建那一版不成立，要回头看是判定错还是重建错）。

## 二、这一轮写过的文件

补丁 `crates.patch` 里（相对快照 28ef00a，快照取自主工作区 2026-09-27；主工作区这七份源文件此刻的 sha256 与快照相同，见第十节）：

- `crates/singlefs-core/src/pointer.rs`：`PointerHeadFieldOutsideTheFirstVersion`、三种指针的 `read_judging_the_encryption_and_compression_fields_from`；`LocationEntriesNotAscendingByDevice`、`location_entries_ascend_by_device` 与 `NodePointer::location_entries_ascend_by_device`；两处升序断言消息改写。
- `crates/singlefs-core/src/unit.rs`：`UnitError::EncryptionReservedBytesNotZero` 与三个解析器的预留位判；条目宽 / 记录宽 0 而条数非 0 判结构错（`ENTRY_WIDTH_ZERO_WITH_ENTRIES`、`RECORD_WIDTH_ZERO_WITH_RECORDS`）。
- `crates/singlefs-core/src/code_two_tree.rs`：`parse_internal_entry` 交 `Result<_, InternalEntryRefusal>`、子指针头部判；`MULTI_LEVEL_CODE_TWO_TREE_CHILD_POINTER_HEAD_OUTSIDE_THE_FIRST_VERSION`；`OnlyWhatTheShapeNeeds` 的文档改成今天谁在用；规划拒绝那一格补单测 `a_previous_shape_whose_separator_hides_a_key_is_refused_by_the_planner`。
- `crates/singlefs-core/src/journal.rs`：新根段两条指针按判的读法读；`record_offset` 的 expect 消息指到读者一侧的环长下界。
- `crates/singlefs-core/src/allocator.rs`：`DeviceEndsBeforeTheUnitAreaStart`；`unit_area_slots_of_device_starting_at` 改 `checked_sub` 交 `Result`；`unit_area_slots_of_device`、`with_unit_area_start` 的 expect 与 `index` / `isolate` 的消息写明入口判过什么。
- `crates/singlefs-core/src/recovery.rs`：`RecoveryFailure::SystemConfigurationValueRefused` / `DeviceEndsBeforeTheUnitAreaStart`，`SystemConfigurationValueOutsideWhatThisReaderAccepts`；`placement_lies_in_the_unit_area_of_its_device`、`every_device_reaches_the_unit_area_start`；冷走读与重建共用的 12 个判定函数；重建判全、I-2.5 判；`allocation_records_of_version_without_file` 判全。
- `crates/singlefs-core/src/mount.rs`：挂载入口判盘容量；影子账的指针槽号判在单元区；`format_time_allocator` 的两个落点过几何判。
- `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs`：新建，31 条用例。

补丁之外：`mutations-append.tsv` 追加 31 行（名字全以「A3a」起头，第六节逐行列）；`mutations-replacements.tsv` 换 7 行（第六节末）。

副本里改过、**不进补丁**的：`crates/singlefs-harness/src/history.rs` 补两臂（第八节原文，只为副本编得过）。副本里临时加过又删掉的两份探针测试文件（`zz_scratch_probe_*.rs`），不在补丁里。

## 三、规格表六行各自怎么做的

| 条 | 改之前（副本里造坏镜像看到的） | 改成什么 | 用例 |
|---|---|---|---|
| 32 | 被抛弃根（树表 0 条那一臂）的指针槽号只判两条位置条目同槽，就进 `PoolAllocator::isolate_abandoned` → `DeviceFreeMap::index` 的 expect；mkfs 那一版的两条指针直接进 `mark_format_time_units` → 同一处 expect | 影子账（`mount::placements_referenced_by_root`）逐盘按 `recovery::placement_lies_in_the_unit_area_of_its_device` 判，不在单元区的那条根与「两条位置条目不同槽」同一个结局：账解不开，只计数（`abandoned_roots_unreadable`）；`format_time_allocator` 把两个落点在每块盘上要记成的分配记录先过 `allocation_records_fit_the_pool_geometry`（与盘上分配记录同一道），不过报 `MountError::Recovery(AllocationRecordOutsideThePoolGeometry)`，在动分配器之前 | `a_format_time_instance_table_pointer_below_the_unit_area_is_refused_before_the_allocator_is_touched`、`an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_counts_as_unreadable_instead_of_panicking` |
| 33（与 24） | 重建读记账树、映射树只按「拼得成一棵树」核，重复 key 的叶原样交给写行那次发布；extent、分配记录树只核位置；inode 树根只解节点；I-7.8、I-9.2、树根出生身份、分配记录每盘一条、记账与映射条目数都不判 | `rebuild_version` 判全：四棵树与映射树 `EveryHeaderAgainstItsReference`；inode 树根走 `tree_root_checked_against_its_pointer` 加层级 1；树表 key 宽、I-7.8、inode 条目 I-9.2 / I-1.2 / I-9.4、分配记录每盘一条与代 / 跨度、记账条目数与代 / seq、映射条目数、读到手的数据单元按查找路径核，与 `walk_to_file` 调同一组函数；树表 0 条那一版的分配记录树（`allocation_records_of_version_without_file`）同样判全。`code_two_tree.rs` 的严格递增判定因此在重建路径上开着 | 重复 key 记账叶 / 映射叶、I-7.8、I-9.2、inode / extent / 分配记录树根出生序号、树表 0 条那一版的分配记录树，共 8 条 |
| 35 | 盘比单元区起点（784 MiB）短：可写挂载走到 `DeviceFreeMap::new`，单元区槽数的减法下溢 panic | 那一处减法改 `checked_sub`、交 `DeviceEndsBeforeTheUnitAreaStart`；可写挂载在择系统配置之后、读根环之前逐盘判（`every_device_reaches_the_unit_area_start`），报 `RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`；恢复判分配记录落点那一道同样报它。mkfs 一侧**今天已经判**：`make_filesystem.rs:305` `if end_of_the_units_written_by_make_filesystem > smallest {`（`UnitAreaBeyondDevice`），`make_filesystem.rs` 不在我的清单、没动 | `a_device_ending_before_the_unit_area_start_is_refused_by_the_writable_mount_before_any_write`、`the_unit_area_slot_count_of_a_device_ending_before_the_unit_area_start_is_an_error` |
| 38 | 环长、pbs 不设界：环长 1000 时写行那次发布在 `journal::record_offset` 的 expect 上 panic（取号两次写之后）；环长 2 GiB 时扫环把单元区当环扫；pbs 定读根槽的缓冲；码 2 / 码 3 条目宽 0 而条数非 0 照收（切出 0 条、头里说 3 条） | 择系统配置时每一槽判：固定结构槽距在格式允许的区间（与逐档找槽 1 同一个上下界）、pbs ∈ [457, 槽距]、环长能装 F 条记录且末端不越过编译期单元区起点——都是 mkfs `check_geometry` 用的同一组界；越界整池拒（`SystemConfigurationValueRefused`，与每区槽数 S 越界同一个处置）。条目宽 / 记录宽 0 而条数非 0 判结构错。checker `lib.rs` 的同一写法（`chunks(entry_width.max(1))` 在 `crates/singlefs-checker/src/lib.rs:403`、`chunks(record_width.max(1))` 在 `:556`）没改，交主 agent 另派 | 系统配置 5 条（槽距、pbs 两头、环长两头）、宽 0 两条 |
| 29 | 指针头部跳过 36 字节不判；单元头 29 字节预留位不判；系统配置的格式版本、加密类型读者不看 | 单元三个解析器判 29 字节全 0（I-2.4），非 0 报 `UnitError::EncryptionReservedBytesNotZero`；指针新加判的读法（MAC / nonce 全 0、算法类型 0、压缩码 0、压后长度 0），换上它的是我清单里的两处读者：多层码 2 树内部条目（`parse_internal_entry`）与 journal 记录新根段；系统配置格式版本认 1、加密类型认 0（关），不认得的整池拒 | 单元三类 + 挂载一条、指针三条、journal 一条、系统配置两条 |
| C476 | 普查那份清单（R1–R13、「缺口」表、算术表）里落在我清单内的：R6 / R7 / R8 / R9（分配器族）今天已被 `allocation_records_fit_the_pool_geometry` 挡住，剩的是第 32 条那两个入口；R5 已由 `slot_shared_by_both_location_entries` 改成报错；算术表里 `allocator.rs:212` 的下溢即第 35 条、`recovery.rs:909` 的扫环即第 38 条、`recovery.rs:969` 与 `mount.rs:957` 的 txg 加一今天已是 `checked_add`（`CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`、`checkpoint_txg_after_the_version`）；「缺口」表 `pointer.rs:108/149` 升序断言：**实测走得到**（副本探针：树表里 extent 树根指针两条位置条目对调、第 0 代根的实例表指针对调，两份镜像都让可写挂载 panic 在 `pointer.rs:222` 的断言上） | 重建时按 I-2.5 判：根记录四条指针（树表 0 条那一版也判）、树表条目根指针、extent 树节点与数据指针、inode 叶容器、分配记录树、记账树、映射树每个节点的指针，树表 0 条那一版的分配记录树节点指针；逆序或相同报 `InvariantViolated { invariant: "I-2.5" }` | `a_tree_table_root_pointer_with_descending_location_entries_is_refused_by_the_writable_mount`、`a_genesis_root_whose_instance_table_pointer_has_descending_location_entries_is_refused_by_the_writable_mount` |

C476 里不在我清单内、今天仍开着的（交主 agent）：R12、R13（checker `image.rs` / `walk.rs`）；「缺口」表 `lib.rs:268`、`walk.rs`、`inode_tree.rs:88`（普查行号，没按今天的文件重查）；指针头部判的读法在我清单之外还没换上的读者——`root_record.rs:131/135/136/137`、`records.rs:328/357/369`、`instance_table.rs:206`、`extent_tree.rs:322/327/688`、`allocation_record_tree.rs:774/869`（今天主工作区的行号），这几处换成 `read_judging_the_encryption_and_compression_fields_from` 要各自定「判到了当什么」（根槽当不可择、条目当损坏……），我没替它们定。

## 四、做了判断、交主 agent 核的几处（按已有条款或已有先例定的，不是新定的；有异议就改）

1. **被抛弃根的指针槽号不在单元区（第 32 条）当「账解不开」只计数，不拒挂载。** 依据是同一个函数里已有的写法：`placements_referenced_by_root` 的文档「读不出、解不开交回 `None`（调用方只计数，不拒绝挂载）」，两条位置条目不同槽、分配记录过不了几何判（带文件那一臂经 `allocation_records_under_root`）今天都走这条。规格那一行写的是「越界报损坏成员」；对被抛弃根，「报」的形态就是 `MountOutput::abandoned_roots_unreadable` 加一。要改成拒挂载，是改影子账的处置口径，不是这一件。
2. **系统配置的格式版本、加密类型、槽距、pbs、环长越界一律整池拒，不退化成「这一槽不可择」。** 先例是每区槽数 S 越界（`RecoveryFailure::RootRingSlotsPerRegionOutOfRange` 的文档：池级字段两盘四槽同值，换一槽读到的还是它）。另一种读法是像 incompat 位图不认识那样「这一槽不可择，一份可择的都没有才报」，两种结局在「只有一槽坏」时不同。
3. **pbs、环长、槽距的读者一侧上下界取 mkfs `check_geometry` 用的同一组**：pbs ≥ 根记录 457（`make_filesystem.rs:262`）且 ≤ 槽距（`:311`）；环长 ≥ F × 4096（在飞上限非 0）且末端 ≤ 编译期单元区起点（`:333`，C475）；槽距 ∈ [4096, 1 MiB − 4096]（逐档找槽 1 那个区间）。「环长 ≤ 最小那块盘 ÷ 4」（`:285`，D23 已定项 19 ③「越界拒绝 mkfs」）没在读者一侧判：条款只说拒 mkfs，挂载一侧有了「末端不越过单元区起点」这道，环长的上界已是 768 MiB。
4. **指针头部 MAC / nonce 非 0 判损坏**：D19（块指针的结构与宽度预算） 已定项 3 只写了「留成空位」，没写读到非 0 怎么办；这里按 I-2.4 给单元头 29 字节定的「恒 0、非 0 判损坏」同一个读法（规格那一行也是这么要求的）。extent 偏移那 2 字节第一版写 0、读到非 0 怎么处置没有条款，**没判**。
5. **盘容量下界取「完整槽数 ≥ 单元区起始槽号」**，是 `DeviceFreeMap` 的前置条件；mkfs 的下界更严（要装下 mkfs 写的实例表与树表第 0 版）。挂载一侧没有条款写下界取哪个。
6. **I-2.5 的读侧判放在重建（`rebuild_version` 与树表 0 条那一版的分配记录树）**，只罩可写挂载要照抄写回的那一版；冷走读、只读挂载、影子账读别的根不判——它们不写回。I-2.5 判的是「严格升序」，写者断言是 `<=`；读者按不变量的字面判（相同也拒）。
7. **判全之后，一份坏镜像挂载报的成员跟冷走读报的一样**，于是有几道更深的守卫在已有坏法上再也走不到（第七节）。没有为了保住旧的报错次序去调判定的先后。

### 条款要补的原句（建议，交主 agent / 用户定）

- D22（单元原子性怎么合成） 已定项 2 补：「读者择系统配置时判根槽宽：`physical_block_size` ∈ [根记录宽 457, 固定结构槽距]，槽距 ∈ [4096, 根环基址 − 4096]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置。」
- D23（journal 的角色与格式） 已定项 18 / 19 补：「读者择系统配置时判环长：在飞上限（环槽数 ÷ F）≥ 1，且环末端不越过单元区起点；越界整池拒绝挂载。」
- D19（块指针的结构与宽度预算） 已定项 3 补：「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏（同 I-2.4 那 29 字节）。」extent 偏移那 2 字节同样要一句。
- D9（加密） 已定项 10 射程那句「读路径跳过它们，池级 checker 不判它们」随这一件过时：core 读路径今天判单元头 29 字节、系统配置加密类型、我清单内两处读者的指针头部。
- 挂载一侧盘容量下界取哪个（第 5 点）。

### checker 那一侧（只报告，没改）

- I-2.4 那 29 字节：checker `check_unit`（`crates/singlefs-checker/src/lib.rs:313` 起）判 magic、flags、类标签、长度与两道校验和，不判预留位全 0，也不判格式版本——与 core 改之前同一处漏。
- 条目宽 0 而条数非 0：`lib.rs:403` `chunks(entry_width.max(1))`、`:556` `chunks(record_width.max(1))`，同一写法（B1 那族，交主 agent 另派）。

## 五、层 0 与崩溃注入的钉值：为什么不跟着变

- **写路径一个字节没动**：补丁里没有一处改写者——`build_index_node` / `build_packed_unit` / `build_data_unit`、`JournalRecord::to_bytes`、`RootRecord::to_slot`、`SystemConfiguration::to_slot`、各 `write_to` 都没改（`pointer.rs` 两处 `write_to` 只改了断言消息的字）。合法历史的录制流、段序列、写表因此不变。
- **读路径新加的都是拒绝**：只对坏内容报错。冷走读 `walk_to_file` 的判定抽成函数之后次序与报的成员逐字不变（抽出来的 12 个函数就是原来那几段代码，调用点在原来的位置）；它在层 0 每个崩溃状态上跑，合法状态本来就要全过。重建新加的判定与冷走读是同一组函数，所以一个合法崩溃状态若被新的可写挂载拒掉，同一个状态上冷走读也报错——那在今天的层 0 神谕里已经是红的。
- 没跑层 0 与崩溃枚举用例（重型，归提交时的崩溃验证员）。推翻条件：提交时层 0 / 崩溃注入的计数有一格变了。
- 补丁没动 `crates/singlefs-checker/src/`，不用「受影响的层 0 流与崩溃枚举用例」那一节。

## 六、新测试与证红

新测试文件 `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 31 条用例，每条的文档注释写了场景、预期与改之前是什么样；core 单测 1 条（`code_two_tree::tests::a_previous_shape_whose_separator_hides_a_key_is_refused_by_the_planner`）。改之前的样子都是副本里实测的：用例本身在改后全绿，每条对应的变异（把改法撤回）由 `research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a3a/work` 施加、经内存包装跑、判红、还原。

证法的边界：prove-red 按变异行第五段的参数跑（`-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- <用例名前缀>`），每次只跑点名的那一条，「同时红了哪些测试」没有逐条记；基线（不施加变异）每组参数先跑一遍，全绿。系统配置那 7 条第一遍证红时变异让可写挂载报的是 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`（调用方参数挡住了），我据此把用例改成先判只读恢复（没有调用方参数）、再判可写挂载，重新证红（下表那 7 行的日志在 `prove-red-logs-rerun/`，红在只读恢复那一句）。

日志路径都相对 `/tmp/claude-1000/impl-rev-a3a/`。同一个日志目录先后三次用：core 那 5 行的日志被随后 harness 那一批覆盖、`prove-red-logs/001.log` 最后是 R7 那一行的——core 那 5 行与 R7 的判定以 `prove-red-run.out` 的原样行为准。

| # | 变异（`crates/mutations.tsv` 的名字）| 改坏哪一处 | 红在哪（用例 → panic 落点与消息开头）| 日志 |
|---|---|---|---|---|
| 1 | A3a 第 33 条：重建读记账树退回只按形状核（重复 key 的记账叶又交给写行那次发布规划） | `recovery.rs` `&MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifier…` | `an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write` → crates/singlefs-core/src/transaction.rs:6720:5「Accounting：计划里叶的 key 与这次装出来的条目的 key 不是同一个集合」 | `prove-red-logs-rerun/001.log` |
| 2 | A3a 第 24 条：重建不判树表条目的树 ID 低于水位（I-7.8） | `recovery.rs` `for entry in &tree_table_entries {` | `a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount` → crates/singlefs-core/src/transaction.rs:6339:5「树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：八个号连号发、中央映射树不进树表，剩下七条仍按发号次序升序」 | `prove-red-logs/002.log` |
| 3 | A3a 第 24 条：重建拿叶容器头自己的身份核它（I-9.2 条目身份与子头不比） | `recovery.rs` `identity: identity_in_the_entry,` | `an_inode_internal_entry_whose_identity_differs_from_its_leaf_container_is_refused_by_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(1)、txg CheckpointTxg(3)」 | `prove-red-logs/003.log` |
| 4 | A3a 第 24 条：树根出生序号不与指针比（重建读 inode 树根不核出生身份） | `recovery.rs` `if node.birth_sequence != pointer.birth_sequence {` | `an_inode_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(1)、txg CheckpointTxg(3)」 | `prove-red-logs/004.log` |
| 5 | A3a 第 33 条：重建读 extent 树退回只核位置 | `recovery.rs` `tree: tree_identifiers.extent,` | `an_extent_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(1)、txg CheckpointTxg(3)」 | `prove-red-logs/005.log` |
| 6 | A3a 第 33 条：重建读分配记录树退回只核位置 | `recovery.rs` `tree_identifiers.allocation_records,` | `an_allocation_record_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(1)、txg CheckpointTxg(3)」 | `prove-red-logs/006.log` |
| 7 | A3a 第 33 条：重建读中央映射树退回只按形状核（重复 key 的映射叶照收） | `recovery.rs` `let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUs…` | `a_central_mapping_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write` → crates/singlefs-core/src/transaction.rs:6720:5「CentralMapping：计划里叶的 key 与这次装出来的条目的 key 不是同一个集合」 | `prove-red-logs/007.log` |
| 8 | A3a 第 24 条：树表 0 条那一版的分配记录树退回只核位置 | `recovery.rs` `TreeIdentifier(crate::transaction::TREE_IDENTIFIER_NONE),` | `the_allocation_record_tree_of_a_version_without_file_is_judged_like_the_cold_walk_on_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(2)、txg CheckpointTxg(4)」 | `prove-red-logs/008.log` |
| 9 | A3a 第 32 条：mkfs 那一版的两个落点不过几何判就进 mark_format_time_units | `mount.rs` `allocation_records_fit_the_pool_geometry(devices, &records_the_form…` | `a_format_time_instance_table_pointer_below_the_unit_area_is_refused_before_the_allocator_is_touched` → crates/singlefs-core/src/allocator.rs:416:81「槽号在单元区内：盘上读来的分配记录与 mkfs 那一版两条指针在 recovery::allocation_records_fit_the_」 | `prove-red-logs/009.log` |
| 10 | A3a 第 32 条：被抛弃根的指针槽号不判在不在单元区就进 isolate_abandoned | `mount.rs` `placement_lies_in_the_unit_area_of_its_device(` | `an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_counts_as_unreadable_instead_of_panicking` → crates/singlefs-core/src/allocator.rs:416:81「槽号在单元区内：盘上读来的分配记录与 mkfs 那一版两条指针在 recovery::allocation_records_fit_the_」 | `prove-red-logs/010.log` |
| 11 | A3a 第 35 条：可写挂载入口不判盘容量下界 | `mount.rs` `every_device_reaches_the_unit_area_start(&*devices)?;` | `a_device_ending_before_the_unit_area_start_is_refused_by_the_writable_mount_before_any_write` → crates/singlefs-core/src/allocator.rs:371:14「建空闲图的盘末尾不在单元区起点之前：mkfs 判过 UnitAreaBeyondDevice，可写挂载读盘之前判过 every_device」 | `prove-red-logs/011.log` |
| 12 | A3a 第 35 条：单元区槽数退回直接减（盘比单元区起点短时下溢） | `allocator.rs` `absolute_slot_count_of_device(device_bytes)` | `the_unit_area_slot_count_of_a_device_ending_before_the_unit_area_start_is_an_error` → crates/singlefs-core/src/allocator.rs:322:8「attempt to subtract with overflow」 | `prove-red-logs/012.log` |
| 13 | A3a 第 29 条：系统配置的格式版本不判 | `recovery.rs` `if format_version != crate::system_configuration::FORMAT_VERSION {` | `a_system_configuration_of_an_unrecognized_format_version_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：NoFile { root: (InstanceGeneration(0), Ch」 | `prove-red-logs-rerun/002.log` |
| 14 | A3a 第 29 条：系统配置的加密类型不判 | `recovery.rs` `if encryption_type != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF {` | `a_system_configuration_whose_encryption_type_is_not_off_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：NoFile { root: (InstanceGeneration(0), Ch」 | `prove-red-logs-rerun/003.log` |
| 15 | A3a 第 38 条：固定结构槽距不判上界 | `recovery.rs` `|| fixed_structure_slot_spacing > largest_fixed_structure_slot_spac…` | `a_fixed_structure_slot_spacing_outside_the_format_range_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：NoFile { root: (InstanceGeneration(0), Ch」 | `prove-red-logs-rerun/004.log` |
| 16 | A3a 第 38 条：physical_block_size 不判下界（根槽装不下根记录） | `recovery.rs` `if u64::from(sizes.physical_block_size) < ROOT_RECORD_BYTES` | `a_physical_block_size_narrower_than_the_root_record_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：Failed { root: None, failure: NoValidRoot」 | `prove-red-logs-rerun/005.log` |
| 17 | A3a 第 38 条：physical_block_size 不判上界（根槽盖到下一个槽） | `recovery.rs` `|| sizes.physical_block_size > sizes.fixed_structure_slot_spacing` | `a_physical_block_size_wider_than_the_fixed_structure_slot_spacing_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：Failed { root: None, failure: NoValidRoot」 | `prove-red-logs-rerun/006.log` |
| 18 | A3a 第 38 条：journal 环长不判下界（装不下 F 条记录） | `recovery.rs` `if journal_in_flight_record_limit(sizes.journal_ring_bytes) == 0` | `a_journal_ring_too_short_for_the_safety_factor_is_refused_before_any_write` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：NoFile { root: (InstanceGeneration(0), Ch」 | `prove-red-logs-rerun/007.log` |
| 19 | A3a 第 38 条：journal 环长不判上界（末端越过单元区起点） | `recovery.rs` `|| sizes.journal_ring_bytes > journal_ring_bytes_up_to_the_compiled…` | `a_journal_ring_reaching_past_the_unit_area_start_is_refused` → 新测试文件:678:9「只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：NoFile { root: (InstanceGeneration(0), Ch」 | `prove-red-logs-rerun/008.log` |
| 20 | A3a 第 29 条：码 2 读者不判 29 字节预留位 | `unit.rs` `check_payload_checksum(bytes, header_end, header_end - 10)?;` | `an_index_node_with_a_non_zero_encryption_reserved_byte_is_refused` → 新测试文件:850:5「assertion `left == right` failed」 | `prove-red-logs/020.log` |
| 21 | A3a 第 29 条：码 2 读者不判 29 字节预留位（可写挂载那一侧） | `unit.rs` `check_payload_checksum(bytes, header_end, header_end - 10)?;` | `an_accounting_root_with_a_non_zero_encryption_reserved_byte_is_refused_by_the_writable_mount` → 新测试文件:384:24「坏镜像被可写挂载收下了：这一版是实例 InstanceGeneration(1)、txg CheckpointTxg(3)」 | `prove-red-logs/021.log` |
| 22 | A3a 第 29 条：码 3 读者不判 29 字节预留位 | `unit.rs` `check_payload_checksum(bytes, header_end, 89)?;` | `a_packed_unit_with_a_non_zero_encryption_reserved_byte_is_refused` → 新测试文件:880:5「assertion `left == right` failed」 | `prove-red-logs/022.log` |
| 23 | A3a 第 29 条：码 1 读者不判 29 字节预留位 | `unit.rs` `check_payload_checksum(bytes, header_end, 101)?;` | `a_data_unit_with_a_non_zero_encryption_reserved_byte_is_refused` → 新测试文件:908:5「assertion `left == right` failed」 | `prove-red-logs/023.log` |
| 24 | A3a 第 38 条：码 2 条目宽 0 而条目数非 0 照收 | `unit.rs` `if entry_width == 0 && entry_count != 0 {` | `an_index_node_declaring_entries_of_width_zero_is_refused` → 新测试文件:1141:5「assertion `left == right` failed」 | `prove-red-logs/024.log` |
| 25 | A3a 第 38 条：码 3 记录宽 0 而记录数非 0 照收 | `unit.rs` `if record_width == 0 && record_count != 0 {` | `a_packed_unit_declaring_records_of_width_zero_is_refused` → 新测试文件:1171:5「assertion `left == right` failed」 | `prove-red-logs/025.log` |
| 26 | A3a 第 29 条：指针头部 MAC / nonce 非 0 不判 | `pointer.rs` `if !self.is_every_mac_and_nonce_byte_zero {` | `a_node_pointer_head_carrying_encryption_or_compression_is_refused_field_by_field` → 新测试文件:1018:9「assertion `left == right` failed: 指针头部偏移 0 写成 1」 | `prove-red-logs/026.log` |
| 27 | A3a 第 29 条：指针头部算法类型不判 | `pointer.rs` `if self.algorithm_type != 0 {` | `a_data_pointer_head_carrying_an_encryption_algorithm_is_refused` → 新测试文件:1050:5「assertion `left == right` failed」 | `prove-red-logs/027.log` |
| 28 | A3a 第 29 条：多层码 2 树内部条目的子指针头部不判 | `code_two_tree.rs` `let child = NodePointer::read_judging_the_encryption_and_compressio…` | `a_code_two_internal_entry_whose_child_pointer_head_is_not_the_first_version_is_refused` → 新测试文件:1073:5「assertion `left == right` failed」 | `prove-red-logs/028.log` |
| 29 | A3a 第 29 条：journal 记录新根段的树表指针头部不判 | `journal.rs` `let new_tree_table =` | `a_journal_record_whose_new_tree_table_pointer_carries_compression_is_not_parsed` → 新测试文件:1115:5「assertion `left == right` failed」 | `prove-red-logs/029.log` |
| 30 | A3a C476 缺口 pointer.rs：重建出来的这一版不按 I-2.5 判指针（照抄写回时在升序断言上 panic） | `recovery.rs` `pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;` | `a_tree_table_root_pointer_with_descending_location_entries_is_refused_by_the_writable_mount` → crates/singlefs-core/src/pointer.rs:260:9「位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过」 | `prove-red-logs/030.log` |
| 31 | A3a C476 缺口 pointer.rs：重建不按 I-2.5 判根记录里的指针（零单元发布照抄时 panic） | `recovery.rs` `root_record_pointers_ascend_by_device(root)?;` | `a_genesis_root_whose_instance_table_pointer_has_descending_location_entries_is_refused_by_the_writable_mount` → crates/singlefs-core/src/pointer.rs:260:9「位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过」 | `prove-red-logs/031.log` |

换锚点与改指向的 7 行（`mutations-replacements.tsv`，名字、改法、必须红的测试与原来一样，只换原文里对不上的那几行；592 行另把目标改成上面那条 core 单测）：

| 行名 | 为什么换 | 证红 |
|---|---|---|
| 分配记录的几何判：盘不在池里当成一块无限大的盘（R9 的那句 expect 又轮得到跑） | 那段判定挪进 `placement_lies_in_the_unit_area_of_its_device`，变量名 `record.device` → `device` | 抓到（core --lib，`record_on_the_device_outside_the_pool_is_refused`） |
| 分配记录的几何判：不判槽号在不在单元区起点之上（R6 的那个减法又会下溢） | 同上，`record.slot` → `slot` | 抓到 |
| 分配记录的几何判：不判跨度越不越过单元区末尾（R6 的那条断言又轮得到跑） | 同上，`span` → `span_slots` | 抓到 |
| 分配记录的几何判：跨度上界判成开区间（贴着单元区末尾的合法记录被误拒） | 同上 | 抓到 |
| 树分裂 规划删不掉上一版的 key 也照常往下走（从盘上重建的映射树分隔 key 坏了，取号之前不拒） | 原来靠 `tree_split` 那条用例，判全之后那条用例走不到规划（第七节）；改指新单测 | 抓到 |
| 普查 R7：分配记录的几何判不判跨度（被抛弃根那棵账里的坏记录又喂进 isolate 的跨度断言） | 同第三行 | **没红**。在基线副本（快照原样）上同一行同样没红：那条用例里坏记录先被 `allocation_record_tree.rs` 的位置判拦下（用例自己断言的就是「不在它所在叶按位置罩的那一段里」），几何判那一道走不到。先前就有的，不是这一件带来的；照实交主 agent |
| 树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败） | 映射条目数那一道抽成 `mapping_entry_count_is_one_per_mapped_unit`，原文改成调用处 `accounting_tree_nodes: roots.accounting.version.node_count(),` | 目标名字带 layer0，prove-red 跳过，留给提交时的 59 号 |

## 七、判全之后要跟着改的别人的测试与变异行（不在我的清单里，没改；原文给主 agent）

基线红集（快照 28ef00a 原样、`/tmp/claude-1000/impl-rev-a3a/baseline` 另一份 target 跑）与改后逐条比：

| 测试 | 基线 | 改后 | 结论 |
|---|---|---|---|
| `second_transaction_step_three_second_instance::torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg` | 红（`NewerStateStillUnreadableAfterOneReread`，66.25 s 那一遍） | 红（同一句） | 不是这一件带来的，照原样报 |
| `second_transaction_supplement_two_tree_split::rebuilt_central_mapping_root_whose_separator_hides_the_key_is_refused_before_the_instance_generation_is_acquired` | 绿 | 红：挂载报 `Recovery(InvariantViolated { invariant: "I-1.1", detail: "分隔 key 大于孩子头里的最小 key" })` | 这一件带来的：重建照冷走读判分隔 key，在规划之前就拒（仍在取号之前、盘上不变） |
| `second_transaction_supplement_three_bad_disk_input::every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking` | 绿（42.36 s） | 红：四种坏法可写挂载报的成员变成与「恢复」那一列相同（下表） | 这一件带来的 |

`bad_disk_input` 那四格（副本里把断言改成逐条打印的探针跑出来的，探针文件跑完删了）：

| 坏法 | 表里写的可写挂载成员 | 改后可写挂载实际报的 |
|---|---|---|
| `NarrowedEntryWidthOfTheAccountingTreeRoot` | `EntryNarrowerThanItsFieldTable { what: "记账条目"` | `InvariantViolated { invariant: "I-1.1", detail: "根 key 区间与条目不符" }` |
| `NarrowedEntryWidthOfTheCentralMappingTreeRoot` | `EntryNarrowerThanItsFieldTable { what: "映射条目"` | 同上 |
| `AllocationRecordReleasedOnTheSecondDeviceOnly` | `ReleaseTargetAlreadyReleased { unit: InstanceTable, …` | `InvariantViolated { invariant: "E142 走读同款", detail: "分配记录不是每个落点每盘各一条…` |
| `RelabelledInodeWatermarkAccountingRow` | `InodeNumberWatermarkRowMissingFromTheAccountingTree` | `InvariantViolated { invariant: "I-1.1", detail: "根 key 区间与条目不符" }` |

建议改法（二选一，交主 agent 定）：

- 甲：照改后的成员改期望——`second_transaction_supplement_three_bad_disk_input.rs` 第 375、404 行改成 `"InvariantViolated { invariant: \"I-1.1\", detail: \"根 key 区间与条目不符\" }"`，第 409–410 行改成 `"InvariantViolated { invariant: \"E142 走读同款\", detail: \"分配记录不是每个落点每盘各一条"`，第 415 行改成与第 414 行相同；`second_transaction_supplement_two_tree_split.rs` 第 480–486 行改成 `Err(MountError::Recovery(RecoveryFailure::InvariantViolated { invariant: "I-1.1", detail: "分隔 key 大于孩子头里的最小 key" }))`。**代价**：变异表里靠这条测试的几行再也红不了——252「普查 R2：核心层映射条目读者的字段表宽度判去掉」（锚点 `records.rs`）、256「普查 R10：释放前的校验退回只核第一条位置条目那块盘」（锚点 `transaction.rs`）；R11 那一道（`InodeNumberWatermarkRowMissingFromTheAccountingTree`）表里没有行。R10 那道守卫今天从盘上已走不到（重建判每盘一条之后两盘的账不会不对称），要么删行、要么改成直接调发布路径的单测；R2、R11 那两道仍走得到，但要一份过得了表头判定的坏镜像。
- 乙：改坏法（`crates/singlefs-harness/src/bad_disk_input.rs`）让它们过得了冷走读那几道再走到更深的守卫：条目宽那两种顺手把节点头的 key 区间改成剩下那一条的 key；水位那一行换成仍排在最后的标签（比如 13）并改头里的最大 key；R10 那一种从盘上走不到了，换成调发布路径的单测。期望与变异行都不用动。

变异表第 592 行「树分裂 规划删不掉上一版的 key 也照常往下走…」锚点在 `code_two_tree.rs`（我的清单里），它原来靠上面那条 `tree_split` 用例；这一件里已经把它改指新单测 `code_two_tree::tests::a_previous_shape_whose_separator_hides_a_key_is_refused_by_the_planner`（`mutations-replacements.tsv`，副本里证红抓到），与上面甲、乙两条都不冲突。

## 八、要加进别人文件的穷举分支（主 agent 打补丁时一起加）

`crates/singlefs-harness/src/history.rs` 的 `recovery_failure_member`，在 `RecoveryFailure::NoValidRoot => …` 那一臂之前加两臂（副本里就是这样加的，编得过、clippy 过）：

```rust
        RecoveryFailure::SystemConfigurationValueRefused { value, .. } => {
            format!("RecoveryFailure::SystemConfigurationValueRefused（{value:?}）")
        }
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. } => {
            "RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart".to_string()
        }
```

`MountError` 没加成员；`first_transaction_on_device.rs` 与 E158 bin 里没有 `RecoveryFailure` 的穷举 match（`grep` 零命中），不用加。`UnitError` 加了 `EncryptionReservedBytesNotZero`，工作区里 `unit.rs` 之外没有它的穷举 match。`code_two_tree::parse_internal_entry` 改交 `Result`：它在我清单外唯一的调用点 `second_transaction_supplement_two_accounting_node_full.rs:185` 用的是 `.expect(…)`，`Result` 上照样编得过。`lib.rs` 不用加导出行。

### 顺带撞见、没修的一处走得到的 panic（交主 agent）

- 第 2 行变异（撤掉重建的 I-7.8 判）把可写挂载打 panic 在 `transaction.rs:6339`（副本里的行号）「树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）」的断言上：写行那次发布照抄盘上读来的树表条目，而树表条目的树 ID 次序读者一处都不判。I-7.8 挡住的只是「号越过水位」那一形；水位之下的号换个次序（比如 livelist 与稀疏旁表那两条没有根的条目互换树 ID：没有根节点可核树 ID，判全也挡不住）照样走到这句断言——推的，没造镜像实测。断言在 `transaction.rs`（不在我的清单），拦的地方可以在 `recovery.rs` 重建读树表那一步（`tree_table_entries_each_kind_at_most_once` 旁边），冷走读今天也不判它，要不要加、报哪个成员交主 agent 定。

## 九、第 4 步那几样的末尾原样输出（都在副本 `/tmp/claude-1000/impl-rev-a3a/work` 上跑，线程上限 4、内存上限 8G）

- 新测试二进制（整个）：`cargo test -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking` 最后一遍末行 `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.78s`（`new-tests-run5.log`）；第二遍批跑里同一个二进制 `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 77.64s`。
- `cargo test -p singlefs-core --lib`：`test result: ok. 131 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s`。
- `cargo fmt --all -- --check`：退 0，无输出。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条（照 `.claude/singlefs-ai-sop/scripts/check.sh:72-80` 抄）：退 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.61s`。
- `cargo build --offline --all-targets`：退 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.36s`。
- 与重建 / 挂载 / 系统配置读法相关的另外 15 个测试二进制（整个二进制，名字都不含 layer0；清单与每个的末行在 `affected-run2.out`）：13 个绿；红的两个是第七节那两条（`step_three` 基线红，`tree_split` 这一件带来的期望变化）。`second_transaction_supplement_three_bad_disk_input` 只在第一遍（加 I-2.5 判之前）跑过整个二进制：8 过 1 红（第七节那条）1 忽略，599 秒；加 I-2.5 判之后没再整跑，推翻条件：它的快档在提交时出了新的 panic 或「读回没提交过的内容」。

登记给实现员的门禁阶段（`stage-owners.tsv` 里第二列有 implementation-writer 的 7 道，在改后副本上逐个 `nice -n 19 bash .claude/gate.d/<文件>`，日志 `gate-<文件>.log`）：

| 阶段 | 退出码 | 末行 / 判定 |
|---|---|---|
| 33 | 1 | ✗ `crates/mutations.tsv:1182`–`1184` 三行「层 0 发现日志：…」原文在 `crash.rs` 里命中 0 次。那三行是主工作区快照之后打上的层 0 发现日志补丁带来的，副本里的 `crash.rs` 还是快照那一版；不是这一件的行。我追加与换的 38 行在副本里逐行核过恰好命中一次（`make_mutation_rows.py` 的核对输出 `rows 31 replacements 7 ok True`） |
| 53 | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 74 | 1 | ✗ 随机历史测试二进制 21 过 3 红（`crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、`rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`）。**三条在基线副本（快照原样）上同样红**：前两条单跑基线 `test result: FAILED. 0 passed; 2 failed`，挂载报 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`；快档两边都是「已知红收尾 {}、新发现 46」、同一个签名 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`、第一个种子 7463871032432355114（`fast-tier-baseline.log` / `fast-tier-work.log`）。是 C554 乙那一族、快照里的 `history.rs` 还没跟上，不是这一件带来的 |
| 92 | 0 | `第一条纯 SSD 布局线（…）：.claude/kb/layout/02-second-txn.md` |
| 94 | 0 | 末行是「这一道判不了的」说明；判定行绿 |
| 93 | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，…）` |
| 89 | 77 | 本次未跑：`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`——与这一件无关，照写 |

## 十、补丁与打法

- `patch/crates.patch`（sha256 `00be023a962bd25ac246577426419f4d77757c5011dd7b1bb86adbb59dae6297`）：七份 core 源文件与新测试文件；对**主工作区 2026-09-27 的现状** `git apply --check` 退 0；这一刻主工作区那七份源文件的 sha256 与我取快照时逐份相同（`baseline-sources.sha256`）。`research/scripts/apply-writer-patch.py patch --dry-run` 输出 `✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1230 行`。
- `patch/mutations-append.tsv`（`6ce7fd0d…f55880`，31 行）、`patch/mutations-replacements.tsv`（`95a2ec0b…6a6ba6`，7 行）。
- 补丁之外、要一起打的：`/tmp/claude-1000/impl-rev-a3a/history-arms.patch`（`a9692f46…07d23da`，第八节那两臂；对主工作区现状 `git apply --check` 退 0）。不打它，打上 `crates.patch` 之后 `singlefs-harness` 编不过。
- 第七节甲 / 乙二选一之后，`tree_split` 与 `bad_disk_input` 两条才回绿；这两条在主 agent 决定之前是红的。
- 副本里 `git diff --stat -- crates litmus` 原样（`crates/mutations.tsv` 那 78 行里混着主工作区快照之后别人改的行——我合表时拿的是主工作区此刻的表；新测试文件未跟踪、不在 stat 里）：

```
 crates/mutations.tsv                      |   78 ++-
 crates/singlefs-core/src/allocator.rs     |   50 +-
 crates/singlefs-core/src/code_two_tree.rs |  131 +++-
 crates/singlefs-core/src/journal.rs       |   22 +-
 crates/singlefs-core/src/mount.rs         |   48 +-
 crates/singlefs-core/src/pointer.rs       |  168 ++++-
 crates/singlefs-core/src/recovery.rs      | 1015 +++++++++++++++++++++--------
 crates/singlefs-core/src/unit.rs          |   38 +-
 crates/singlefs-harness/src/history.rs    |    6 +
 9 files changed, 1239 insertions(+), 317 deletions(-)
```

## 十一、没做什么

- 没走三方对抗；层 0（全量与快档）、QEMU、herd7、crates 变异整表（门禁 59）没跑，归提交时的 `crash-verifier` 与整轮门禁；没提交。变异表第 575 行（层 0 目标）换了锚点，没证红，留给 59 号。
- 没改我清单外的文件：`history.rs`（第八节）、`tree_split` 与 `bad_disk_input` 两份测试与 `bad_disk_input.rs`（第七节）、checker（第四节）、`make_filesystem.rs`、`system_configuration.rs`（格式版本与加密类型的偏移因此在 `recovery.rs` 里按写者的字段表现算，`SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32 + 16 + 12 + 4`，用例里逐槽核了这两个偏移上的值）、清单外那几处指针读者（第三节末）。
- 系统配置的「结构常量」（节点大小、单元大小、落点粒度、位置条目宽度、记录尺寸、F、R、P、chunk、根环起点、w_max、g、单元区起始槽号、MAC 长度声明）读者仍不看：规格那一行只要格式版本与加密类型；它们住 `system_configuration.rs`（`parse_slot` 里有注释说节点大小不读回的理由），要判得那边定。
- journal 记录自己的算法类型、nonce、MAC 预留（D23 已定项 4 的字段）没判：没有条款写读到非 0 怎么办，规格也没点。
- `transaction.rs` 那句树表条目树 ID 升序断言（第七节末）没修，只报告。
- 变异证红只证「点名那条会红」，没记同时红了哪些别的测试（prove-red 按用例名前缀只跑那一条）。

## 十二、草稿与删掉的东西

- 删了：基线副本 `/tmp/claude-1000/impl-rev-a3a/baseline` 两次（第一次 2.3G、第二次 1.1G，各在核完基线红集之后）；副本里临时建的两份探针测试文件（`zz_scratch_probe_every_fixed_panic_site.rs`、`zz_scratch_probe_ascending.rs`，跑完就删，不在补丁里）。
- 交回前删了：仓副本 `/tmp/claude-1000/impl-rev-a3a/work`，19G（含它的 target；补丁、报告与证红日志都在它外面）。
- 留着的（主 agent 核用）：`/tmp/claude-1000/impl-rev-a3a/` 下的 `patch/`、`history-arms.patch`、`report.md`、`progress.md`、各次跑的日志（`affected-logs*/`、`prove-red-logs*/`、`gate-*.log`、`fast-tier-*.log`、`probe-*.log`）与我写的辅助脚本（`*.sh`、`*.py`）。没入 `research/results/`：这一件交的是代码补丁与变异行，日志只是证红与核基线的过程材料；主 agent 要留就从这里拷。
