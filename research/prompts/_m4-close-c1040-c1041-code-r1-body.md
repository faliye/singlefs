# C1040、C1041 合入与文件区间交集长度证明：代码轮第一轮（m4-close-c1040-c1041-code-r1）

日期：2026-11-18。

## 问题

这一轮攻两批已进主工作区的 `crates/` 改动，一批一轮：

1. **C1040、C1041 两批补丁**（11 月 17 日合入，合入后验证 19176，第 2 步 19262–19266，打补丁 19272、回读 19273，记在 `records/2026-11-17-里程碑四收口续做.md` 第 52 行）。C1041（系统配置四格几何读者整段跳过）：core、池级 checker 与验证包两处 CPU 读法判系统配置槽偏移 305、309、313、321 四格等于格式常量（N 等于 0），不等整池拒；C1040（挂载不比对探测到的物理块大小与两盘容量）的 `physical_block_size` 那一半：可写挂载逐盘探测、与择到的系统配置记着的值不等就拒；mkfs 参数里的 `physical_block_size` 与任一块盘探测值不等就在任何写之前拒。实现员报告 `~/.local/share/singlefs/work/m4-close-c1040-c1041/result.md`（批 1）与 `result-batch-2.md`（批 2）。
2. **用户新加的文件区间交集长度证明**：`crates/singlefs-replay-prefix/src/file_range_proofs.rs` 加 `spec fn range_intersection_length`、两条组合引理（`lemma_adjacent_unit_intersection_lengths_add_up`、`lemma_split_write_intersection_lengths_add_up`）与无前提见证 `witness_intersection_length_partitions`，`bytes_in_unit` 与 `checked_bytes_in_indexed_unit` 的后置条件补交集长度；`src/README.md` 补说明；`verus-must-red-worlds.tsv` 新增 8 行必红、3 行旧行 `proof_errors` 由 1 改 2；`.claude/gate.d/verus-pins.tsv` 这个包的义务数 211 → 214。形式化验证任务 20208（22 项）退 0。

两批没有共用文件，并进一轮只因为它们在同一窗口合入、都还没走过代码轮。

被判文件（开工快照 `research/prompts/m4-close-c1040-c1041-code-r1-snapshot/crates-sha256.txt`，改动整份在同目录 `changes.patch`，是 `git diff HEAD` 取的；这 16 份文件相对 HEAD 的改动全属这两批，`crates/mutations.tsv` 除外）：

| 批 | 文件 |
|---|---|
| C1041 | `crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-verification/src/verification_read.rs`、`crates/singlefs-verification/src/memory_pool.rs`、`crates/singlefs-verification/tests/system_configuration_unit_geometry_fields_are_refused_by_core_and_the_pool_checker.rs`、`crates/singlefs-verification/tests/memory_pool_scan_candidates_do_not_take_the_geometry_of_a_slot_zero_carrying_a_refused_unit_geometry_field.rs` |
| C1040 | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-verification/tests/writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs`、`crates/singlefs-verification/tests/make_filesystem_refuses_a_physical_block_size_parameter_other_than_the_probed_one_before_any_write.rs`、`crates/singlefs-verification/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`（穷举 `match` 加一臂）、`crates/singlefs-verification/tests/core_review_geometry_back_chain_and_empty_inode.rs`（受牵连那一处） |
| 证明 | `crates/singlefs-replay-prefix/src/file_range_proofs.rs`、`crates/singlefs-replay-prefix/src/README.md`、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` |
| 变异 | `crates/mutations.tsv` 里 C1040、C1041 追加的 17 行（名字以「C1040：」「C1041：」起头；这份表相对 HEAD 还有别的会话的改动，不在这一轮） |

## 实现今天的样子

主 agent 的观测（现读主工作区，行号是此刻的）：

- C1041 core：`crates/singlefs-core/src/system_configuration.rs:192` `unit_geometry_fields_recorded_in_the_slot` 只切字节；`crates/singlefs-core/src/recovery.rs:1421` `system_configuration_values_this_reader_accepts` 在加密类型之后、槽距之前调 `:1533` `unit_geometry_fields_of_the_format`，不等报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的四个新成员。
- C1041 池级 checker：`crates/singlefs-checker/src/image.rs:1066` `pub fn unit_geometry_field_that_disagrees`，`:1233` `read_system_configuration_slot` 在 `check_system_configuration_slot` 之后、`geometry_of` 之前调；验证包 `crates/singlefs-verification/src/verification_read.rs:94` 与 `crates/singlefs-verification/src/memory_pool.rs:758` 直接调这个池级 checker 函数。GPU 判器两个内核（`crash_judge_checker.wgsl`、`crash_judge_common.wgsl`）这四格没同判，`crates/singlefs-checker/invariants/02-根环.md` 第 20 行 I-7.13 记着欠在 C1041。
- C1040 可写挂载：`crates/singlefs-core/src/mount.rs:7566` `device_table_disagreeing_with` 末尾接 `:7616` `physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`（`:7627` 调 `probe_physical_block_size`）。只读挂载没有这一比：`grep -n 'probe_physical_block_size' crates/singlefs-core/src/mounted_read.rs crates/singlefs-core/src/recovery.rs` 零命中（实现员批 2 报告「结论」第 1 条停下）；而 `.claude/kb/decisions/12-目标介质.md` 第 103 行 D12（目标介质） 已定项 5 定案写「代码已做这一比（可写与只读挂载都拒；mkfs 时参数与探测不同也拒）」。
- C1040 mkfs：`crates/singlefs-core/src/make_filesystem.rs:608` `first_device_probing_a_physical_block_size_other_than_the_parameter`，`:632` `make_filesystem` 在 `check_geometry` 之后、第一次写之前调。
- 已知的红：验证包目标 `crash_given_states_of_the_images_harness_scenarios_hand_out` 的 `write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push`（别的会话的任务 19999），断言在 `crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3798`：期望「报被宽护卡住那一次不预演」时探测物理块大小次数与改盘调用数是 (0, 0)，读到 (2, 0)。C1040 让写路径探测物理块大小之后才红；那一批合入后验证没跑到这个目标。没定是「那条路不该探测」还是「测试的判法要换」。
- 证明：`crates/singlefs-core/src/write_request_split.rs:383` `bytes_written_into` 把 `checked_bytes_in_indexed_unit` 交回元组的第三项放进 `length_in_bytes`；两份执行函数的函数体相对 HEAD 不变，只改后置条件。

方案按 `crates/` 今天的实现来谈。

## 压着这件事的条款

- D22（单元原子性怎么合成） 已定项 2（根环的参数）与已定项 9（系统配置字段表）、已定项 16（读者收的界）：四格的值与 mkfs 按 `physical_block_size` 划根槽。
- D21（权威态与派生态的分界） 已定项 16「单元几何在挂载时求值一次」、已定项 8「N 住系统配置，第一版 N = 0」。
- D12（目标介质） 已定项 4（挂载时比对哪几条）与已定项 5 第 103 行那一句。
- 不变量 I-7.13（系统配置池级字段在读者收的范围里），`crates/singlefs-checker/invariants/02-根环.md` 第 20 行。
- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：运行时与 checker 必须用不同的算法（验证包两处读法直接调池级 checker 的函数，判它算不算审计与被审计共用）。
- 证明：`records/2026-11-14-文件区间形式化扩展.md` 的出口，与 `crates/singlefs-replay-prefix/src/README.md` 新加那一段写的射程（只涉字节区间与返回长度，不代表缓冲区拷贝、设备 I/O 或持久化顺序已精化）。

### 检查单：走不到的理由二次查实

批 1 报告「走不到的理由」原样：「无：产品代码 `crates/singlefs-core/src/` 里这一批没有新加 `todo!`、`unreachable!`、`assert!`，也没有以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾的错误成员。」批 2 报告原样：「无：产品代码 `crates/singlefs-core/src/` 里这一批没有新加 `todo!`、`unreachable!`、`assert!`。新错误成员是定了的条款（D22 已定项 2 根槽宽一格），不以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾。」批 1 报告「写过的文件」另写 `system_configuration.rs` 的 `to_slot`「写四格前断言位置」：腿核这条断言与「没有新加 `assert!`」对不对得上、它在什么输入上会触发。池级 checker 与验证包两处的新分支、证明的执行函数没有新的不可达分支（函数体不变）。

### 检查单：类型与名字

新类型与成员：`UnitGeometryFieldsRecordedInTheSlot`（core）、`UnitGeometryFieldThatDisagrees` 与 `RefusedSystemConfigurationValue::UnitGeometryFieldDisagrees`（池级 checker）、`SystemConfigurationValueOutsideWhatThisReaderAccepts` 的 `UnitBytesNotTheFormatConstant`、`SlotGranularityBytesNotTheFormatConstant`、`LocationEntryBytesNotTheFormatConstant`、`ExtensionPointDeclaredBytesNotTheFirstVersionValue`、`DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration`、`MakeFilesystemError::PhysicalBlockSizeParameterDiffersFromTheProbedValue { device, probed_physical_block_size, physical_block_size_in_the_parameters }`；常量 `EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION` 与四格偏移常量。证明一侧：`range_intersection_length` 与三个 proof fn。腿核：四格的值是不是裸整数、与已有 newtype（`PhysicalBlockSizeInBytes` 这类）是否混用；`physical_block_size_in_the_parameters` 的类型与 `probed_physical_block_size` 是否同一个 newtype；每处 `expect` 写没写依赖的不变量；`match` 有没有 `_ =>`。

### 检查单：不兼容旧代码与旧格式

改动只收紧读者：四格不是格式常量、探测与记录不等就拒，没有认旧版本、旧布局号的分支。腿核 `memory_pool.rs:758` 的 `.filter` 退到 `SlotZeroNotSelfDescribing` 与 `verification_read.rs:94` 交 `Err(())` 是不是只为别的池的槽放宽了什么。

### 检查单：不新建判定库

新测试都只用内存盘（`SparseBlockDevice`、`RecordingBlockDevice`、`MemoryPool`），不读写判定存放处；证明沿用门禁 formal-verification 的逐项判定。腿核新测试文件里有没有写文件、TSV、库。

### 检查单：所有任务多机可用

新测试不读仓里的文件、`CARGO_MANIFEST_DIR`、主机名与本机状态（两份报告「拒算的测试」一节）；证明只依赖 Verus 与登记表。腿核。

### 检查单：测试优先多线程

新测试每条一个场景、只在内存盘上做 mkfs 与一次挂载或一次读槽，规模在毫秒级，没有切片；不能并行的理由没有写在用例旁。腿判这些用例该不该切片、有没有写死线程数。

### 检查单：带 --item 的命令与项

两份报告「要主 agent 跑的命令」里的点名测试与 17 行变异的 `--item` 已交（变异 19266「抓到 17 条、没红 0 条」）；证明新增 8 行必红与 3 个 proof 项由 formal-verification 列项，任务 20208 退 0。腿核：两份报告里有没有新目标没被点名、`crates-mutation-replay --list-items` 与 `formal-verification --list-items` 列不列得出新项（命令写进报告交主 agent 跑）；那条已知的红测试所在目标合入后验证为什么没跑到。

### 检查单：新崩溃流多罩了什么

本轮改动没有新的崩溃点重放流或崩溃枚举用例。两份报告「受影响的流与崩溃枚举用例」推：写系统配置槽的只有 `SystemConfiguration::to_slot`，四格照常量写，各条流造盘时探测值取的就是参数，所以已有流的判定集合与钉值不变（推的，没跑）。腿核这条推论：有没有流或装置在探测值与参数不同的盘上做 mkfs 或可写挂载，有没有写系统配置槽的第二个写者。

## 分工

- 正推腿：逐份核「代码做的是不是条款说的」：四格的判定次序与 core、池级 checker、字段表（D22 已定项 9）三处是否一致；可写挂载三个入口（报告批 1「结论」第 2 条：可写挂载、挂着之后收盘表的入口、`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）都走到新比对；mkfs 的判在任何写之前；证明的规格与两条组合引理的前提是不是最弱的、必红行是不是每行红在点名的那一格；八格逐格回「成立 / 不成立 / 推不出」，每格的依据引文件与行。
- 攻方腿：找哪一格会错：一种输入（盘、槽内容、探测值组合）让 core 与池级 checker 判得不同、让 mkfs 写了一字节才拒、让可写挂载绕过比对；验证包两处直接调池级 checker 函数算不算「审计与被审计共用一段代码」；已知那条红测试该判「那条路不该探测」还是「测试判法要换」，给出依据；D12 已定项 5 第 103 行与只读挂载实现的不一致；证明的后置条件有没有空真、组合引理有没有用不到的前提、必红行有没有能被别的错误顶替的；能建小模型的建、命令写进报告交主 agent 跑。
- 两条腿都不改 `crates/`。

## 交用户的

暂无；判决时列。
